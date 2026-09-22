//! The placement, derive, edge, dependency, snapshot, and register guard.
//!
//! `cargo xtask check-placement <document>` reads the architecture document and
//! runs every rule the section 1.5 contract states. Each rule carries an id
//! `PG<n>` and section 1.9 pairs each one with its one probe `PP<n>` (DR5). The
//! guard runs in the `plan-lint` job of `.github/workflows/ci.yml`, never in
//! `scripts/dod.sh`.
//!
//! The guard is fail-closed. It exits 0 when it finds nothing, 1 when it finds
//! at least one breach, and 2 when it cannot decide. It carries no hidden set:
//! the candidate drop list, the primitive trait sets, and the primitive sizes
//! are each a fenced block of the document, and the guard holds no fallback for
//! any of them.
//!
//! [`DATA_BLOCKS`] is the ONE register of every block the guard reads and of
//! every `rows>=` floor those blocks state. The [`sync_floors`] module writes
//! those floors from the row count each block itself holds, so no floor is a
//! hand number.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write as _};
use std::path::Path;

use crate::Outcome;

pub(crate) mod pattern {
    //! A small backtracking matcher for the patterns this guard needs.
    //!
    //! The prototype is written against Python's `re`, so the port needs the
    //! same matching semantics: leftmost matching, greedy and lazy repeats,
    //! alternation, capture groups, word boundaries, and fixed-width negative
    //! lookbehind. The engine works over `char` positions, never bytes, so an
    //! offset a rule records is the offset Python records.

    /// One member of a character class.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum ClassItem {
        /// One literal character.
        One(char),
        /// Every character between two bounds, both ends included.
        Range(char, char),
        /// The `\d` set.
        Digit,
        /// The `\D` set.
        NotDigit,
        /// The `\w` set.
        Word,
        /// The `\W` set.
        NotWord,
        /// The `\s` set.
        Space,
        /// The `\S` set.
        NotSpace,
    }

    impl ClassItem {
        /// Whether this member accepts one character.
        fn accepts(self, value: char) -> bool {
            match self {
                Self::One(item) => item == value,
                Self::Range(low, high) => low <= value && value <= high,
                Self::Digit => value.is_ascii_digit(),
                Self::NotDigit => !value.is_ascii_digit(),
                Self::Word => is_word(value),
                Self::NotWord => !is_word(value),
                Self::Space => value.is_whitespace(),
                Self::NotSpace => !value.is_whitespace(),
            }
        }
    }

    /// Whether one character is a word character, as a `\w` class reads it.
    pub(crate) fn is_word(value: char) -> bool {
        value.is_alphanumeric() || value == '_'
    }

    /// A character class, with the `^` form recorded.
    #[derive(Debug, Clone, PartialEq, Eq)]
    struct Class {
        /// Whether the class accepts every character its members refuse.
        negated: bool,
        /// Every member the class states.
        items: Vec<ClassItem>,
        /// Whether the class folds case, which `re.I` asks for.
        fold: bool,
    }

    impl Class {
        /// Whether any member of this class accepts one character.
        fn holds(&self, value: char) -> bool {
            self.items.iter().any(|item| item.accepts(value))
        }

        /// Whether this class accepts one character.
        fn accepts(&self, value: char) -> bool {
            let folded = self.fold
                && value
                    .to_lowercase()
                    .chain(value.to_uppercase())
                    .any(|other| self.holds(other));
            (self.holds(value) || folded) != self.negated
        }
    }

    /// One single-character test a repeat can run without a sub-program.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Simple {
        /// One literal character.
        Char(char),
        /// One character class, by its index in the program.
        Class(usize),
        /// Any character, the newline included.
        AnyAll,
        /// Any character but a newline.
        AnyLine,
    }

    /// One instruction of a compiled pattern.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Inst {
        /// Accept one literal character.
        Char(char),
        /// Accept one character the class at this index accepts.
        Class(usize),
        /// Accept any character, the newline included.
        AnyAll,
        /// Accept any character but a newline.
        AnyLine,
        /// Try the first target and keep the second for a backtrack.
        Split(usize, usize),
        /// Continue at this target.
        Jump(usize),
        /// Record the current position in this capture slot.
        Save(usize),
        /// Run one single-character test between `min` and `max` times.
        Repeat {
            /// The single-character test each repetition runs.
            unit: Simple,
            /// The fewest repetitions the pattern accepts.
            min: usize,
            /// The most repetitions the pattern accepts.
            max: usize,
            /// Whether the repeat takes the longest run first.
            greedy: bool,
        },
        /// Assert a word boundary, or its negation.
        Boundary(bool),
        /// Assert the start of the subject, or of a line under `(?m)`.
        LineStart,
        /// Assert the end of the subject, or of a line under `(?m)`.
        LineEnd,
        /// Assert that a sub-program matches, or that it does not.
        Look {
            /// Where the sub-program starts.
            start: usize,
            /// Whether the assertion refuses a match instead of demanding one.
            negative: bool,
            /// How many characters a look-behind steps back, or zero ahead.
            behind: usize,
        },
        /// Report a match.
        Done,
    }

    /// One node of a parsed pattern.
    #[derive(Debug, Clone, PartialEq, Eq)]
    enum Ast {
        /// The pattern that accepts the empty string.
        Empty,
        /// One literal character.
        Char(char),
        /// One character class.
        Class(Class),
        /// Any character.
        Any,
        /// Every node in order.
        Concat(Vec<Self>),
        /// The first branch that matches.
        Alt(Vec<Self>),
        /// One node, repeated.
        Repeat {
            /// The node the repeat runs.
            node: Box<Self>,
            /// The fewest repetitions the pattern accepts.
            min: usize,
            /// The most repetitions the pattern accepts.
            max: usize,
            /// Whether the repeat takes the longest run first.
            greedy: bool,
        },
        /// One group, with the capture slot it fills.
        Group {
            /// The capture number, or `None` for a `(?:...)` group.
            index: Option<usize>,
            /// The node inside the group.
            node: Box<Self>,
        },
        /// One look-around assertion.
        Look {
            /// Whether the assertion refuses a match instead of demanding one.
            negative: bool,
            /// Whether the assertion reads backward from the position.
            behind: bool,
            /// The node the assertion runs.
            node: Box<Self>,
        },
        /// A word boundary, or its negation.
        Boundary(bool),
        /// The start of the subject, or of a line under `(?m)`.
        LineStart,
        /// The end of the subject, or of a line under `(?m)`.
        LineEnd,
    }

    /// How many capture slots one compiled pattern may fill.
    const SLOTS: usize = 24;

    /// Every capture position of one match attempt, unset positions included.
    type Slots = [usize; SLOTS];

    /// The position value an unfilled capture slot carries.
    const UNSET: usize = usize::MAX;

    /// One entry of the backtracking stack.
    #[derive(Debug, Clone, Copy)]
    enum Frame {
        /// Resume one alternative at a recorded position.
        Try {
            /// Where the alternative starts.
            pc: usize,
            /// The subject position the alternative resumes at.
            sp: usize,
            /// The capture slots the alternative resumes with.
            slots: Slots,
        },
        /// Resume a simple repeat with one fewer or one more repetition.
        Repeat {
            /// Where the instruction after the repeat starts.
            pc: usize,
            /// The subject position the repeat began at.
            base: usize,
            /// The repetition count this entry resumes with.
            next: usize,
            /// The fewest repetitions the pattern accepts.
            min: usize,
            /// The longest run the subject offers.
            high: usize,
            /// Whether the repeat takes the longest run first.
            greedy: bool,
            /// The capture slots the repeat resumes with.
            slots: Slots,
        },
    }

    /// One match of a compiled pattern over a subject.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct Match {
        /// Where the match starts, in characters.
        start: usize,
        /// Where the match ends, in characters.
        end: usize,
        /// Every capture group span, unfilled groups included.
        groups: Vec<Option<(usize, usize)>>,
    }

    impl Match {
        /// Where the match starts, in characters.
        pub(crate) const fn start(&self) -> usize {
            self.start
        }

        /// Where the match ends, in characters.
        pub(crate) const fn end(&self) -> usize {
            self.end
        }

        /// The span of one capture group, or of the whole match at index zero.
        pub(crate) fn span(&self, index: usize) -> Option<(usize, usize)> {
            if index == 0 {
                return Some((self.start, self.end));
            }
            self.groups.get(index - 1).copied().flatten()
        }

        /// The text of one capture group, or of the whole match at index zero.
        pub(crate) fn group(&self, index: usize, text: &[char]) -> Option<String> {
            let (start, end) = self.span(index)?;
            slice(text, start, end)
        }

        /// The text of one capture group, or the empty string when it is unset.
        pub(crate) fn text(&self, index: usize, text: &[char]) -> String {
            self.group(index, text).unwrap_or_default()
        }
    }

    /// The characters between two positions, or `None` when the span is bad.
    pub(crate) fn slice(text: &[char], start: usize, end: usize) -> Option<String> {
        text.get(start..end).map(|part| part.iter().collect())
    }

    /// One subject, as the characters the matcher reads.
    pub(crate) fn chars(text: &str) -> Vec<char> {
        text.chars().collect()
    }

    /// A compiled pattern.
    #[derive(Debug, Clone)]
    pub(crate) struct Regex {
        /// Every instruction, the look-around sub-programs included.
        code: Vec<Inst>,
        /// Every character class the instructions name.
        classes: Vec<Class>,
        /// How many capture groups the pattern states.
        groups: usize,
        /// Whether `^` and `$` read a line rather than the whole subject.
        multiline: bool,
    }

    /// Compile one pattern, or report why it does not compile.
    ///
    /// # Errors
    /// Returns an error when the pattern states a form this engine declines.
    pub(crate) fn build(source: &str) -> anyhow::Result<Regex> {
        Regex::new(source)
    }

    /// Every character of a literal, escaped so a pattern reads it as text.
    pub(crate) fn quote(text: &str) -> String {
        let mut out = String::new();
        for value in text.chars() {
            if !value.is_ascii_alphanumeric() && value != '_' {
                out.push('\\');
            }
            out.push(value);
        }
        out
    }

    /// The state one pattern parse carries.
    #[derive(Debug)]
    struct Parser {
        /// Every character of the pattern.
        source: Vec<char>,
        /// Where the parse has reached.
        at: usize,
        /// How many capture groups the parse has opened.
        groups: usize,
        /// Whether `.` accepts a newline.
        dotall: bool,
        /// Whether a literal and a class fold case.
        fold: bool,
    }

    impl Parser {
        /// The character at the parse position, without consuming it.
        fn peek(&self) -> Option<char> {
            self.source.get(self.at).copied()
        }

        /// Whether the parse position opens with this text.
        fn looking_at(&self, text: &str) -> bool {
            let wanted: Vec<char> = text.chars().collect();
            self.source
                .get(self.at..self.at.saturating_add(wanted.len()))
                .is_some_and(|found| found == wanted.as_slice())
        }

        /// Step over this text when the parse position opens with it.
        fn eat(&mut self, text: &str) -> bool {
            if self.looking_at(text) {
                self.at = self.at.saturating_add(text.chars().count());
                return true;
            }
            false
        }

        /// The next character, consumed.
        fn take(&mut self) -> Option<char> {
            let found = self.peek()?;
            self.at = self.at.saturating_add(1);
            Some(found)
        }

        /// Record one inline flag, and report whether it was `(?m)`.
        const fn set_flag(&mut self, flag: char) -> bool {
            match flag {
                's' => self.dotall = true,
                'i' => self.fold = true,
                _ => return flag == 'm',
            }
            false
        }

        /// Every branch of an alternation, to the end or to a closing bracket.
        fn parse_alt(&mut self) -> anyhow::Result<Ast> {
            let mut branches = vec![self.parse_concat()?];
            while self.peek() == Some('|') {
                self.at = self.at.saturating_add(1);
                branches.push(self.parse_concat()?);
            }
            if branches.len() == 1 {
                return branches.pop().ok_or_else(|| anyhow::anyhow!("no branch"));
            }
            Ok(Ast::Alt(branches))
        }

        /// Every node of one branch, in order.
        fn parse_concat(&mut self) -> anyhow::Result<Ast> {
            let mut parts = Vec::new();
            loop {
                match self.peek() {
                    None | Some('|' | ')') => break,
                    Some(_) => parts.push(self.parse_repeat()?),
                }
            }
            if parts.is_empty() {
                return Ok(Ast::Empty);
            }
            if parts.len() == 1 {
                return parts.pop().ok_or_else(|| anyhow::anyhow!("no part"));
            }
            Ok(Ast::Concat(parts))
        }

        /// One atom, with the quantifier that follows it.
        fn parse_repeat(&mut self) -> anyhow::Result<Ast> {
            let node = self.parse_atom()?;
            let Some((min, max)) = self.parse_bounds() else {
                return Ok(node);
            };
            let greedy = self.peek() != Some('?');
            if !greedy {
                self.at = self.at.saturating_add(1);
            }
            Ok(Ast::Repeat {
                node: Box::new(node),
                min,
                max,
                greedy,
            })
        }

        /// The bounds one quantifier states, or `None` when none follows.
        fn parse_bounds(&mut self) -> Option<(usize, usize)> {
            match self.peek() {
                Some('*') => {
                    self.at = self.at.saturating_add(1);
                    Some((0, usize::MAX))
                },
                Some('+') => {
                    self.at = self.at.saturating_add(1);
                    Some((1, usize::MAX))
                },
                Some('?') => {
                    self.at = self.at.saturating_add(1);
                    Some((0, 1))
                },
                Some('{') => self.parse_braces(),
                Some(_) | None => None,
            }
        }

        /// The bounds the text after `{` states, or `None` when it states none.
        fn brace_bounds(&mut self, low: Option<usize>) -> Option<(usize, usize)> {
            if self.peek() == Some(',') {
                self.at = self.at.saturating_add(1);
                let high = self.parse_digits();
                return Some((low.unwrap_or(0), high.unwrap_or(usize::MAX)));
            }
            low.map(|count| (count, count))
        }

        /// The bounds a `{n}`, `{n,}`, or `{n,m}` quantifier states.
        fn parse_braces(&mut self) -> Option<(usize, usize)> {
            let opened = self.at;
            self.at = self.at.saturating_add(1);
            let low = self.parse_digits();
            let found = self.brace_bounds(low);
            let Some(bounds) = found.filter(|_| self.peek() == Some('}')) else {
                self.at = opened;
                return None;
            };
            self.at = self.at.saturating_add(1);
            Some(bounds)
        }

        /// The digit run at the parse position, or `None` when there is none.
        fn parse_digits(&mut self) -> Option<usize> {
            let mut value: Option<usize> = None;
            while let Some(digit) = self.peek().and_then(|found| found.to_digit(10)) {
                let step = usize::try_from(digit).unwrap_or(0);
                value = Some(value.unwrap_or(0).saturating_mul(10).saturating_add(step));
                self.at = self.at.saturating_add(1);
            }
            value
        }

        /// One atom: a group, a class, an escape, an anchor, or a literal.
        fn parse_atom(&mut self) -> anyhow::Result<Ast> {
            if self.eat("(?:") {
                return self.close_group(None);
            }
            if self.eat("(?=") {
                return self.close_look(false, false);
            }
            if self.eat("(?!") {
                return self.close_look(true, false);
            }
            if self.eat("(?<=") {
                return self.close_look(false, true);
            }
            if self.eat("(?<!") {
                return self.close_look(true, true);
            }
            if self.eat("(") {
                self.groups = self.groups.saturating_add(1);
                let index = self.groups;
                return self.close_group(Some(index));
            }
            if self.eat("[") {
                return Ok(Ast::Class(parse_class(self)?));
            }
            match self.take() {
                None => Ok(Ast::Empty),
                Some('.') => Ok(Ast::Any),
                Some('^') => Ok(Ast::LineStart),
                Some('$') => Ok(Ast::LineEnd),
                Some('\\') => self.parse_escape(),
                Some(found) => Ok(self.literal(found)),
            }
        }

        /// One literal node, with the case-folding form when `re.I` is on.
        fn literal(&self, value: char) -> Ast {
            if !self.fold || !value.is_alphabetic() {
                return Ast::Char(value);
            }
            let items = std::iter::once(ClassItem::One(value))
                .chain(
                    value
                        .to_lowercase()
                        .chain(value.to_uppercase())
                        .map(ClassItem::One),
                )
                .collect();
            Ast::Class(Class {
                negated: false,
                items,
                fold: false,
            })
        }

        /// The body of a group, with its closing bracket consumed.
        fn close_group(&mut self, index: Option<usize>) -> anyhow::Result<Ast> {
            let node = self.parse_alt()?;
            anyhow::ensure!(self.eat(")"), "a group is not closed");
            Ok(Ast::Group {
                index,
                node: Box::new(node),
            })
        }

        /// The body of a look-around, with its closing bracket consumed.
        fn close_look(&mut self, negative: bool, behind: bool) -> anyhow::Result<Ast> {
            let node = self.parse_alt()?;
            anyhow::ensure!(self.eat(")"), "a look-around is not closed");
            Ok(Ast::Look {
                negative,
                behind,
                node: Box::new(node),
            })
        }

        /// One escape outside a class.
        fn parse_escape(&mut self) -> anyhow::Result<Ast> {
            let found = self
                .take()
                .ok_or_else(|| anyhow::anyhow!("a pattern ends in a backslash"))?;
            Ok(match found {
                'd' => Ast::Class(Self::one_item(ClassItem::Digit)),
                'D' => Ast::Class(Self::one_item(ClassItem::NotDigit)),
                'w' => Ast::Class(Self::one_item(ClassItem::Word)),
                'W' => Ast::Class(Self::one_item(ClassItem::NotWord)),
                's' => Ast::Class(Self::one_item(ClassItem::Space)),
                'S' => Ast::Class(Self::one_item(ClassItem::NotSpace)),
                'b' => Ast::Boundary(true),
                'B' => Ast::Boundary(false),
                'n' => Ast::Char('\n'),
                't' => Ast::Char('\t'),
                'r' => Ast::Char('\r'),
                other => self.literal(other),
            })
        }

        /// A class of one member.
        fn one_item(item: ClassItem) -> Class {
            Class {
                negated: false,
                items: vec![item],
                fold: false,
            }
        }

        /// Whether a `-` at the parse position opens a class range.
        fn opens_range(&self) -> bool {
            self.peek() == Some('-') && self.source.get(self.at.saturating_add(1)) != Some(&']')
        }

        /// The character that closes a class range.
        fn class_range_end(&mut self) -> anyhow::Result<char> {
            let high = self
                .take()
                .ok_or_else(|| anyhow::anyhow!("a class range is not closed"))?;
            if high != '\\' {
                return Ok(high);
            }
            match self.class_escape()? {
                Ok(value) => Ok(value),
                Err(_set) => anyhow::bail!("a class range ends in a set"),
            }
        }

        /// One escape inside a class: a character, or a whole member set.
        fn class_escape(&mut self) -> anyhow::Result<Result<char, ClassItem>> {
            let found = self
                .take()
                .ok_or_else(|| anyhow::anyhow!("a class ends in a backslash"))?;
            Ok(match found {
                'd' => Err(ClassItem::Digit),
                'D' => Err(ClassItem::NotDigit),
                'w' => Err(ClassItem::Word),
                'W' => Err(ClassItem::NotWord),
                's' => Err(ClassItem::Space),
                'S' => Err(ClassItem::NotSpace),
                'n' => Ok('\n'),
                't' => Ok('\t'),
                'r' => Ok('\r'),
                other => Ok(other),
            })
        }
    }

    /// Resume the newest backtrack entry, as `(instruction, position, slots)`.
    fn backtrack(stack: &mut Vec<Frame>) -> Option<(usize, usize, Slots)> {
        while let Some(frame) = stack.pop() {
            if let Some(found) = resume(frame, stack) {
                return Some(found);
            }
        }
        None
    }

    /// Resume one backtrack entry, or report that it is exhausted.
    ///
    /// A `Repeat` entry resumes with one fewer repetition when the repeat is
    /// greedy and with one more when it is lazy, and it pushes its own successor
    /// so the whole run costs one entry.
    fn resume(frame: Frame, stack: &mut Vec<Frame>) -> Option<(usize, usize, Slots)> {
        let Frame::Repeat {
            pc,
            base,
            next,
            min,
            high,
            greedy,
            slots,
        } = frame
        else {
            let Frame::Try { pc, sp, slots } = frame else {
                return None;
            };
            return Some((pc, sp, slots));
        };
        if greedy && next < min {
            return None;
        }
        if !greedy && next > high {
            return None;
        }
        let step = if greedy {
            next.checked_sub(1).filter(|count| *count >= min)
        } else {
            next.checked_add(1).filter(|count| *count <= high)
        };
        if let Some(count) = step {
            stack.push(Frame::Repeat {
                pc,
                base,
                next: count,
                min,
                high,
                greedy,
                slots,
            });
        }
        Some((pc, base.saturating_add(next), slots))
    }

    /// How far a search steps after one match, so an empty match still advances.
    const fn step_after(one: &Match) -> usize {
        if one.end == one.start {
            one.end.saturating_add(1)
        } else {
            one.end
        }
    }

    /// Read every inline flag group at the pattern start.
    ///
    /// It returns whether the pattern carries `(?m)`, which is the one flag the
    /// matcher reads after the compile.
    fn read_flags(parser: &mut Parser) -> bool {
        let mut multiline = false;
        loop {
            let opened = parser.at;
            if !parser.eat("(?") {
                return multiline;
            }
            let mut letters = 0_usize;
            while let Some(flag) = parser.peek().filter(|found| "msixau".contains(*found)) {
                parser.at = parser.at.saturating_add(1);
                multiline = parser.set_flag(flag) || multiline;
                letters = letters.saturating_add(1);
            }
            if letters == 0 || !parser.eat(")") {
                parser.at = opened;
                return multiline;
            }
        }
    }

    /// One bracketed class, with its closing bracket consumed.
    fn parse_class(parser: &mut Parser) -> anyhow::Result<Class> {
        let negated = parser.eat("^");
        let mut items = Vec::new();
        let mut first = true;
        loop {
            let found = parser
                .take()
                .ok_or_else(|| anyhow::anyhow!("a class is not closed"))?;
            if found == ']' && !first {
                break;
            }
            first = false;
            let member = if found == '\\' {
                parser.class_escape()?
            } else {
                Ok(found)
            };
            let low = match member {
                Ok(value) => value,
                Err(item) => {
                    items.push(item);
                    continue;
                },
            };
            if parser.opens_range() {
                parser.at = parser.at.saturating_add(1);
                let high = parser.class_range_end()?;
                items.push(ClassItem::Range(low, high));
                continue;
            }
            items.push(ClassItem::One(low));
        }
        Ok(Class {
            negated,
            items,
            fold: parser.fold,
        })
    }

    /// The width every branch of an alternation shares, or `None`.
    fn alt_width(branches: &[Ast]) -> Option<usize> {
        let mut width: Option<usize> = None;
        for branch in branches {
            let found = fixed_width(branch)?;
            if width.is_some_and(|other| other != found) {
                return None;
            }
            width = Some(found);
        }
        width
    }

    /// How many characters one node always accepts, or `None` when it varies.
    fn fixed_width(node: &Ast) -> Option<usize> {
        match *node {
            Ast::Empty | Ast::Boundary(_) | Ast::LineStart | Ast::LineEnd | Ast::Look { .. } => {
                Some(0)
            },
            Ast::Char(_) | Ast::Class(_) | Ast::Any => Some(1),
            Ast::Concat(ref parts) => {
                let mut total = 0_usize;
                for part in parts {
                    total = total.saturating_add(fixed_width(part)?);
                }
                Some(total)
            },
            Ast::Alt(ref branches) => alt_width(branches),
            Ast::Repeat {
                ref node,
                min,
                max,
                greedy: _,
            } => {
                if min != max {
                    return None;
                }
                fixed_width(node)?.checked_mul(min)
            },
            Ast::Group { index: _, ref node } => fixed_width(node),
        }
    }

    /// The state one pattern compile carries.
    #[derive(Debug)]
    struct Compiler {
        /// Every instruction, the look-around sub-programs included.
        code: Vec<Inst>,
        /// Every character class the instructions name.
        classes: Vec<Class>,
        /// Whether `.` accepts a newline.
        dotall: bool,
        /// Every look-around whose sub-program is not compiled yet.
        pending: Vec<(usize, Ast)>,
    }

    impl Compiler {
        /// Add one instruction and report where it landed.
        fn emit(&mut self, inst: Inst) -> usize {
            let at = self.code.len();
            self.code.push(inst);
            at
        }

        /// Replace one instruction the compile reserved.
        fn patch(&mut self, at: usize, inst: Inst) {
            if let Some(slot) = self.code.get_mut(at) {
                *slot = inst;
            }
        }

        /// The single-character test one node runs, or `None`.
        fn simple(&mut self, node: &Ast) -> Option<Simple> {
            match *node {
                Ast::Char(value) => Some(Simple::Char(value)),
                Ast::Class(ref class) => {
                    let at = self.classes.len();
                    self.classes.push(class.clone());
                    Some(Simple::Class(at))
                },
                Ast::Any => Some(if self.dotall {
                    Simple::AnyAll
                } else {
                    Simple::AnyLine
                }),
                Ast::Empty
                | Ast::Concat(_)
                | Ast::Alt(_)
                | Ast::Repeat { .. }
                | Ast::Group { .. }
                | Ast::Look { .. }
                | Ast::Boundary(_)
                | Ast::LineStart
                | Ast::LineEnd => None,
            }
        }

        /// Compile one node into the instruction list.
        fn compile(&mut self, node: &Ast) -> anyhow::Result<()> {
            match *node {
                Ast::Empty => {},
                Ast::Char(value) => {
                    self.emit(Inst::Char(value));
                },
                Ast::Class(ref class) => {
                    let at = self.classes.len();
                    self.classes.push(class.clone());
                    self.emit(Inst::Class(at));
                },
                Ast::Any => {
                    let inst = self.any_inst();
                    self.emit(inst);
                },
                Ast::Concat(ref parts) => self.compile_all(parts)?,
                Ast::Alt(ref branches) => self.compile_alt(branches)?,
                Ast::Repeat {
                    ref node,
                    min,
                    max,
                    greedy,
                } => self.compile_repeat(node, min, max, greedy)?,
                Ast::Group { index, ref node } => self.compile_group(index, node)?,
                Ast::Look {
                    negative,
                    behind,
                    ref node,
                } => self.compile_look(negative, behind, node)?,
                Ast::Boundary(wanted) => {
                    self.emit(Inst::Boundary(wanted));
                },
                Ast::LineStart => {
                    self.emit(Inst::LineStart);
                },
                Ast::LineEnd => {
                    self.emit(Inst::LineEnd);
                },
            }
            Ok(())
        }

        /// The instruction `.` compiles to, which the dotall flag decides.
        const fn any_inst(&self) -> Inst {
            if self.dotall {
                Inst::AnyAll
            } else {
                Inst::AnyLine
            }
        }

        /// Compile every node of a concatenation, in order.
        fn compile_all(&mut self, parts: &[Ast]) -> anyhow::Result<()> {
            for part in parts {
                self.compile(part)?;
            }
            Ok(())
        }

        /// Compile one group, with the capture slots a numbered group fills.
        fn compile_group(&mut self, index: Option<usize>, node: &Ast) -> anyhow::Result<()> {
            let Some(number) = index else {
                return self.compile(node);
            };
            let slot = number.saturating_sub(1).saturating_mul(2);
            self.emit(Inst::Save(slot));
            self.compile(node)?;
            self.emit(Inst::Save(slot.saturating_add(1)));
            Ok(())
        }

        /// Reserve one look-around instruction and queue its sub-program.
        fn compile_look(&mut self, negative: bool, behind: bool, node: &Ast) -> anyhow::Result<()> {
            let width = if behind {
                fixed_width(node)
                    .ok_or_else(|| anyhow::anyhow!("a look-behind is not fixed width"))?
            } else {
                0
            };
            anyhow::ensure!(!behind || width > 0, "a look-behind states no width");
            let at = self.emit(Inst::Look {
                start: 0,
                negative,
                behind: width,
            });
            self.pending.push((at, node.clone()));
            Ok(())
        }

        /// The split one repeat emits, with the branch order its greed asks for.
        const fn split_for(greedy: bool, body: usize, end: usize) -> Inst {
            if greedy {
                Inst::Split(body, end)
            } else {
                Inst::Split(end, body)
            }
        }

        /// Compile an alternation: the first branch that matches wins.
        fn compile_alt(&mut self, branches: &[Ast]) -> anyhow::Result<()> {
            let Some((last, leading)) = branches.split_last() else {
                return Ok(());
            };
            let mut jumps = Vec::new();
            for branch in leading {
                let split = self.emit(Inst::Split(0, 0));
                self.compile(branch)?;
                jumps.push(self.emit(Inst::Jump(0)));
                let next = self.code.len();
                self.patch(split, Inst::Split(split.saturating_add(1), next));
            }
            self.compile(last)?;
            let end = self.code.len();
            for jump in jumps {
                self.patch(jump, Inst::Jump(end));
            }
            Ok(())
        }

        /// Compile a repeat, with the one-character fast form where it fits.
        fn compile_repeat(
            &mut self,
            node: &Ast,
            min: usize,
            max: usize,
            greedy: bool,
        ) -> anyhow::Result<()> {
            if let Some(unit) = self.simple(node) {
                self.emit(Inst::Repeat {
                    unit,
                    min,
                    max,
                    greedy,
                });
                return Ok(());
            }
            for _ in 0..min {
                self.compile(node)?;
            }
            if max == usize::MAX {
                let start = self.code.len();
                let split = self.emit(Inst::Split(0, 0));
                self.compile(node)?;
                self.emit(Inst::Jump(start));
                let end = self.code.len();
                let body = split.saturating_add(1);
                self.patch(split, Self::split_for(greedy, body, end));
                return Ok(());
            }
            let mut splits = Vec::new();
            for _ in min..max {
                splits.push(self.emit(Inst::Split(0, 0)));
                self.compile(node)?;
            }
            let end = self.code.len();
            for split in splits {
                let body = split.saturating_add(1);
                self.patch(split, Self::split_for(greedy, body, end));
            }
            Ok(())
        }

        /// Compile every look-around sub-program the main program reserved.
        fn compile_looks(&mut self) -> anyhow::Result<()> {
            while let Some((at, node)) = self.pending.pop() {
                self.compile_one_look(at, &node)?;
            }
            Ok(())
        }

        /// Compile one look-around sub-program and patch its instruction.
        fn compile_one_look(&mut self, at: usize, node: &Ast) -> anyhow::Result<()> {
            let start = self.code.len();
            self.compile(node)?;
            self.emit(Inst::Done);
            let Some(&Inst::Look {
                start: _,
                negative,
                behind,
            }) = self.code.get(at)
            else {
                anyhow::bail!("a look-around instruction is lost");
            };
            self.patch(
                at,
                Inst::Look {
                    start,
                    negative,
                    behind,
                },
            );
            Ok(())
        }
    }

    impl Regex {
        /// Compile one pattern.
        ///
        /// # Errors
        /// Returns an error when the pattern states a form this engine declines.
        fn new(source: &str) -> anyhow::Result<Self> {
            let mut parser = Parser {
                source: source.chars().collect(),
                at: 0,
                groups: 0,
                dotall: false,
                fold: false,
            };
            let multiline = read_flags(&mut parser);
            let node = parser.parse_alt()?;
            anyhow::ensure!(
                parser.at >= parser.source.len(),
                "a pattern holds an unread tail"
            );
            anyhow::ensure!(
                parser.groups.saturating_mul(2) <= SLOTS,
                "a pattern states more groups than this engine holds"
            );
            let mut compiler = Compiler {
                code: Vec::new(),
                classes: Vec::new(),
                dotall: parser.dotall,
                pending: Vec::new(),
            };
            compiler.compile(&node)?;
            compiler.emit(Inst::Done);
            compiler.compile_looks()?;
            Ok(Self {
                code: compiler.code,
                classes: compiler.classes,
                groups: parser.groups,
                multiline,
            })
        }

        /// Whether the character at one position passes a single-character test.
        fn accepts(&self, unit: Simple, text: &[char], at: usize) -> bool {
            let Some(&found) = text.get(at) else {
                return false;
            };
            match unit {
                Simple::Char(value) => found == value,
                Simple::Class(index) => self.classes.get(index).is_some_and(|c| c.accepts(found)),
                Simple::AnyAll => true,
                Simple::AnyLine => found != '\n',
            }
        }

        /// The longest run of one single-character test, capped at `max`.
        fn run_length(&self, unit: Simple, text: &[char], at: usize, max: usize) -> usize {
            let mut count = 0_usize;
            while count < max && self.accepts(unit, text, at.saturating_add(count)) {
                count = count.saturating_add(1);
            }
            count
        }

        /// Whether the position sits on a word boundary.
        fn boundary(text: &[char], at: usize) -> bool {
            let before = at
                .checked_sub(1)
                .and_then(|index| text.get(index))
                .is_some_and(|&found| is_word(found));
            let after = text.get(at).is_some_and(|&found| is_word(found));
            before != after
        }

        /// Whether the position opens a line, or the subject.
        fn at_line_start(&self, text: &[char], at: usize) -> bool {
            if at == 0 {
                return true;
            }
            self.multiline
                && at
                    .checked_sub(1)
                    .and_then(|index| text.get(index))
                    .is_some_and(|&found| found == '\n')
        }

        /// Whether the position closes a line, or the subject.
        fn at_line_end(&self, text: &[char], at: usize) -> bool {
            if at >= text.len() {
                return true;
            }
            if self.multiline {
                return text.get(at) == Some(&'\n');
            }
            at.saturating_add(1) == text.len() && text.get(at) == Some(&'\n')
        }
    }

    impl Regex {
        /// Enter one simple repeat, as the instruction and position that follow.
        fn enter_repeat(
            &self,
            text: &[char],
            state: (usize, usize, Slots),
            unit: Simple,
            bounds: (usize, usize, bool),
            stack: &mut Vec<Frame>,
        ) -> Option<(usize, usize)> {
            let (pc, sp, slots) = state;
            let (min, max, greedy) = bounds;
            let high = self.run_length(unit, text, sp, max);
            if high < min {
                return None;
            }
            let after = pc.saturating_add(1);
            let count = if greedy { high } else { min };
            let step = if greedy {
                count.checked_sub(1).filter(|value| *value >= min)
            } else {
                count.checked_add(1).filter(|value| *value <= high)
            };
            if let Some(next) = step {
                stack.push(Frame::Repeat {
                    pc: after,
                    base: sp,
                    next,
                    min,
                    high,
                    greedy,
                    slots,
                });
            }
            Some((after, sp.saturating_add(count)))
        }

        /// Whether one look-around assertion holds at a position.
        fn look_holds(&self, text: &[char], at: usize, start: usize, behind: usize) -> bool {
            if behind == 0 {
                return self.execute(text, start, at, None).is_some();
            }
            let Some(back) = at.checked_sub(behind) else {
                return false;
            };
            self.execute(text, start, back, Some(at)).is_some()
        }

        /// Run the program from one instruction and one position.
        fn execute(
            &self,
            text: &[char],
            start_pc: usize,
            at: usize,
            anchor_end: Option<usize>,
        ) -> Option<(usize, Slots)> {
            let mut stack: Vec<Frame> = Vec::new();
            let mut state = (start_pc, at, [UNSET; SLOTS]);
            loop {
                match self.step(text, state, &mut stack, anchor_end) {
                    Outcome::Done(end) => return Some((end, state.2)),
                    Outcome::Go(pc, sp) => state = (pc, sp, state.2),
                    Outcome::Save(slot, pc) => state = Self::saved(state, slot, pc),
                    Outcome::Fail => state = backtrack(&mut stack)?,
                }
            }
        }

        /// The state one `Save` instruction produces.
        fn saved(state: (usize, usize, Slots), slot: usize, pc: usize) -> (usize, usize, Slots) {
            let (_pc, sp, mut slots) = state;
            if let Some(cell) = slots.get_mut(slot) {
                *cell = sp;
            }
            (pc, sp, slots)
        }

        /// Continue at one instruction when the test holds, and fail when it does not.
        const fn gate(holds: bool, pc: usize, sp: usize) -> Outcome {
            if holds {
                Outcome::Go(pc, sp)
            } else {
                Outcome::Fail
            }
        }

        /// Run one instruction.
        fn step(
            &self,
            text: &[char],
            state: (usize, usize, Slots),
            stack: &mut Vec<Frame>,
            anchor_end: Option<usize>,
        ) -> Outcome {
            let (pc, sp, slots) = state;
            let next = pc.saturating_add(1);
            let ahead = sp.saturating_add(1);
            match self.code.get(pc) {
                None => Outcome::Fail,
                Some(&Inst::Done) => {
                    Self::gate(anchor_end.is_none_or(|end| end == sp), usize::MAX, sp).into_done(sp)
                },
                Some(&Inst::Char(value)) => Self::gate(text.get(sp) == Some(&value), next, ahead),
                Some(&Inst::Class(index)) => Self::gate(
                    text.get(sp).is_some_and(|&found| {
                        self.classes
                            .get(index)
                            .is_some_and(|class| class.accepts(found))
                    }),
                    next,
                    ahead,
                ),
                Some(&Inst::AnyAll) => Self::gate(sp < text.len(), next, ahead),
                Some(&Inst::AnyLine) => Self::gate(
                    text.get(sp).is_some_and(|&found| found != '\n'),
                    next,
                    ahead,
                ),
                Some(&Inst::Split(first, second)) => {
                    stack.push(Frame::Try {
                        pc: second,
                        sp,
                        slots,
                    });
                    Outcome::Go(first, sp)
                },
                Some(&Inst::Jump(target)) => Outcome::Go(target, sp),
                Some(&Inst::Save(slot)) => Outcome::Save(slot, next),
                Some(&Inst::Repeat {
                    unit,
                    min,
                    max,
                    greedy,
                }) => self
                    .enter_repeat(text, state, unit, (min, max, greedy), stack)
                    .map_or(Outcome::Fail, |(pc2, sp2)| Outcome::Go(pc2, sp2)),
                Some(&Inst::Boundary(wanted)) => {
                    Self::gate(Self::boundary(text, sp) == wanted, next, sp)
                },
                Some(&Inst::LineStart) => Self::gate(self.at_line_start(text, sp), next, sp),
                Some(&Inst::LineEnd) => Self::gate(self.at_line_end(text, sp), next, sp),
                Some(&Inst::Look {
                    start,
                    negative,
                    behind,
                }) => Self::gate(
                    self.look_holds(text, sp, start, behind) != negative,
                    next,
                    sp,
                ),
            }
        }

        /// The spans one successful run filled, as a capture list.
        fn captured(&self, slots: &Slots) -> Vec<Option<(usize, usize)>> {
            let mut groups = Vec::with_capacity(self.groups);
            for index in 0..self.groups {
                let first = index.saturating_mul(2);
                let start = slots.get(first).copied().unwrap_or(UNSET);
                let end = slots.get(first.saturating_add(1)).copied().unwrap_or(UNSET);
                groups.push(Self::span_of(start, end));
            }
            groups
        }

        /// One capture span, or `None` when either end is unset.
        const fn span_of(start: usize, end: usize) -> Option<(usize, usize)> {
            if start == UNSET || end == UNSET {
                None
            } else {
                Some((start, end))
            }
        }

        /// The match that starts exactly at one position, as `re.match` reads it.
        pub(crate) fn match_at(&self, text: &[char], at: usize) -> Option<Match> {
            let (end, slots) = self.execute(text, 0, at, None)?;
            Some(Match {
                start: at,
                end,
                groups: self.captured(&slots),
            })
        }

        /// The match that covers the whole subject, as `re.fullmatch` reads it.
        pub(crate) fn full_match(&self, text: &[char]) -> Option<Match> {
            let (end, slots) = self.execute(text, 0, 0, Some(text.len()))?;
            Some(Match {
                start: 0,
                end,
                groups: self.captured(&slots),
            })
        }

        /// The first position at or after `from` that the program can open on.
        ///
        /// A program whose first instruction is a literal or a line start can
        /// only match where that character sits, so the scan steps over every
        /// other position instead of setting up a run there.
        fn next_start(&self, text: &[char], from: usize) -> Option<usize> {
            match self.code.first() {
                Some(&Inst::Char(value)) => Self::next_char(text, from, value),
                Some(&Inst::LineStart) => self.next_line(text, from),
                Some(_) | None => Some(from),
            }
        }

        /// The next position that holds one literal character.
        fn next_char(text: &[char], from: usize, value: char) -> Option<usize> {
            text.iter()
                .skip(from)
                .position(|found| *found == value)
                .map(|at| from.saturating_add(at))
        }

        /// The next position that opens a line, or the subject.
        fn next_line(&self, text: &[char], from: usize) -> Option<usize> {
            if from == 0 {
                return Some(0);
            }
            if !self.multiline {
                return None;
            }
            (from..=text.len()).find(|at| self.at_line_start(text, *at))
        }

        /// The first match at or after one position, as `re.search` reads it.
        pub(crate) fn find_at(&self, text: &[char], from: usize) -> Option<Match> {
            let mut at = from;
            while at <= text.len() {
                at = self.next_start(text, at)?;
                match self.match_at(text, at) {
                    Some(found) => return Some(found),
                    None => at = at.saturating_add(1),
                }
            }
            None
        }

        /// The first match anywhere in the subject.
        pub(crate) fn find(&self, text: &[char]) -> Option<Match> {
            self.find_at(text, 0)
        }

        /// Every non-overlapping match, as `re.finditer` reads them.
        pub(crate) fn find_iter(&self, text: &[char]) -> Vec<Match> {
            let mut found = Vec::new();
            let mut at = 0_usize;
            while let Some(one) = self.find_at(text, at) {
                at = step_after(&one);
                found.push(one);
            }
            found
        }

        /// Whether the subject holds a match anywhere.
        pub(crate) fn is_match(&self, text: &[char]) -> bool {
            self.find(text).is_some()
        }

        /// The subject with every match replaced by what `build` returns.
        pub(crate) fn replace_all(
            &self,
            text: &[char],
            build: &mut dyn FnMut(&Match, &[char]) -> String,
        ) -> String {
            let mut out = String::new();
            let mut last = 0_usize;
            for one in self.find_iter(text) {
                out.push_str(&slice(text, last, one.start).unwrap_or_default());
                out.push_str(&build(&one, text));
                last = one.end;
            }
            if let Some(part) = slice(text, last, text.len()) {
                out.push_str(&part);
            }
            out
        }

        /// The subject cut at every match, as `re.split` reads it.
        pub(crate) fn split(&self, text: &[char]) -> Vec<String> {
            let mut out = Vec::new();
            let mut last = 0_usize;
            for one in self.find_iter(text) {
                out.push(slice(text, last, one.start).unwrap_or_default());
                last = one.end;
            }
            out.push(slice(text, last, text.len()).unwrap_or_default());
            out
        }
    }

    impl Outcome {
        /// This outcome, with a continue turned into a reported match.
        const fn into_done(self, end: usize) -> Self {
            match self {
                Self::Go(_pc, _sp) => Self::Done(end),
                Self::Save(slot, pc) => Self::Save(slot, pc),
                Self::Done(at) => Self::Done(at),
                Self::Fail => Self::Fail,
            }
        }
    }

    /// What one instruction asks the run loop to do next.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum Outcome {
        /// Continue at one instruction and one position.
        Go(usize, usize),
        /// Record the position in one capture slot, then continue.
        Save(usize, usize),
        /// Report a match that ends at this position.
        Done(usize),
        /// Take the newest backtrack entry.
        Fail,
    }
}

/// The two crate rows that may name a framework type.
const APP_ROWS: [&str; 2] = ["crates/duet", "duet-agent"];

/// The marker a declaration carries when the audio thread owns its value.
///
/// It is the second source of the root set (critic C15-3), so a one-for-one
/// swap in the block fails in both directions.
const AUDIO_MARK: &str = "**Audio-owned**";

/// The four prototypes PG29 reads for their own rule ids.
///
/// This guard holds every `PG` id but PG25 and PG32; `roster_compile.sh` holds
/// PG25, `closure_check.py` holds PG32, and `conversion_check.py` holds every
/// `CG` id.
const PROTOTYPES: [&str; 4] = [
    "placement_check.py",
    "conversion_check.py",
    "roster_compile.sh",
    "closure_check.py",
];

/// Every membership kind PG27 implements (WR-18).
///
/// The document names one kind per registered block in its own `block-members`
/// block, and a kind this list does not hold is exit 2: a rule the guard cannot
/// run is not a rule.
const MEMBER_KINDS: [&str; 31] = [
    "carrier-end",
    "crate-name",
    "pin-name",
    "map-crate",
    "not-declared",
    "ownership-row",
    "drop-name",
    "const-crate",
    "limit-row",
    "rule-id",
    "block-id",
    "verdict-name",
    "external-name",
    "expr-head",
    "sub-id",
    "declared-name",
    "first-declared",
    "primitive-name",
    "unknown-name",
    "field-path",
    "cited-elsewhere",
    "gate-site",
    "fault-arm",
    "site-crate",
    "site-path",
    "chunk-id",
    "mechanism-name",
    "impl-site",
    "review-id",
    "line-owner",
    "chunk-pair",
];

/// Every verb that states an edge between two crates in prose (PG11).
const EDGE_VERBS: [&str; 9] = [
    "uses", "reads", "holds", "carries", "calls", "paints", "drains", "writes", "depends",
];

/// The nine traits PG19 closes over.
///
/// `Debug` and `Clone` are outside the rule: `Debug` is denied by lint
/// everywhere and `Clone` follows `Copy`.
const CLOSURE_TRAITS: [&str; 9] = [
    "Copy",
    "Default",
    "Serialize",
    "Deserialize",
    "PartialEq",
    "Eq",
    "Hash",
    "Ord",
    "PartialOrd",
];

/// The two traits VR1 governs from revision 11 (PG22).
///
/// `Eq` left the rule, because PG23 now demands it wherever every field
/// supplies it.
const VR1_TRAITS: [&str; 2] = ["Hash", "Ord"];

/// A smart pointer whose single unsized argument makes the value a fat pointer.
const FAT_POINTER_HEADS: [&str; 3] = ["Box", "Arc", "Rc"];

/// The heads whose size the argument decides, so a head row states nothing.
const INLINE_HEADS: [&str; 3] = ["Option", "SmallVec", "ArrayVec"];

/// Every standard container that owns a heap allocation (PG26).
const HEAP_NAMES: [&str; 12] = [
    "Box", "Vec", "String", "Arc", "Rc", "BTreeMap", "BTreeSet", "HashMap", "HashSet", "VecDeque",
    "Cow", "PathBuf",
];

/// Every container that GROWS, which may call the allocator after it is built.
const GROW_NAMES: [&str; 10] = [
    "Vec", "String", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "VecDeque", "Cow", "PathBuf",
    "SmallVec",
];

/// Every lock (`PG26c`). The last path segment is the name.
const LOCK_NAMES: [&str; 10] = [
    "Mutex",
    "RwLock",
    "ReentrantLock",
    "Condvar",
    "Barrier",
    "OnceLock",
    "LazyLock",
    "MutexGuard",
    "RwLockReadGuard",
    "RwLockWriteGuard",
];

/// The two lints a declaration expects at its own site, in Appendix B.1 order.
const EXPECTED_LINTS: [&str; 2] = ["missing_copy_implementations", "variant_size_differences"];

/// Every primitive with no spare bit pattern.
///
/// `bool` and `char` carry one, and so does every other name this model does
/// not decide. PG24 reads the set to tell a direct tag from a niche tag.
const NICHE_FREE: [&str; 14] = [
    "u8", "u16", "u32", "u64", "u128", "usize", "i8", "i16", "i32", "i64", "i128", "isize", "f32",
    "f64",
];

/// The one value the audio thread holds, which roots the `PG26e` walk.
const AUDIO_ROOT_OF_ROOTS: &str = "EngineProcess";

/// The fewest chunk rows PG40 may decide.
const LINE_CHUNK_FLOOR: usize = 50;

/// The fewest budget rows PG37 may decide.
const VALUE_FLOOR: usize = 9;

/// The most rows the `phase-pair-exempt` block may hold.
const EXEMPT_CEILING: usize = 24;

/// The families section 1.7 indexes, which PG39 holds against the live set.
const INDEX_FAMILIES: [&str; 4] = ["PG", "PP", "CG", "CP"];

/// The four Appendix B.1 count sentences, each with the block it introduces.
const B1_COUNT_SENTENCES: [(&str, &str); 4] = [
    ("suppressions in `duet-time::convert`", "b1-convert"),
    ("complexity suppressions", "b1-complexity"),
    ("`missing_copy_implementations` expectations", "b1-copy"),
    ("`variant_size_differences` expectations", "b1-variant"),
];

/// Every number word a count sentence may state.
const WORD_NUMBERS: [(&str, usize); 22] = [
    ("one", 1),
    ("two", 2),
    ("three", 3),
    ("four", 4),
    ("five", 5),
    ("six", 6),
    ("seven", 7),
    ("eight", 8),
    ("nine", 9),
    ("ten", 10),
    ("eleven", 11),
    ("twelve", 12),
    ("thirteen", 13),
    ("fourteen", 14),
    ("fifteen", 15),
    ("sixteen", 16),
    ("seventeen", 17),
    ("eighteen", 18),
    ("nineteen", 19),
    ("twenty", 20),
    ("twenty-one", 21),
    ("twenty-two", 22),
];

/// Every type-expression head that names one end of a cross-thread carrier.
///
/// Each head is mapped to the END it is: `write`, `read`, or `share` for a
/// queue behind an `Arc`, whose two ends are one value (critic C23I-W1).
const CARRIER_HEADS: [(&str, &str); 11] = [
    ("Sender", "write"),
    ("SyncSender", "write"),
    ("Producer", "write"),
    ("Input", "write"),
    ("OneshotSender", "write"),
    ("Receiver", "read"),
    ("CoreReceiver", "read"),
    ("Consumer", "read"),
    ("Output", "read"),
    ("OneshotReceiver", "read"),
    ("ArrayQueue", "share"),
];

/// The five primitives section 5.8 decides, plus the two channel spellings.
const CARRIER_PRIMITIVES: [&str; 6] = [
    "triple_buffer",
    "rtrb",
    "ArrayQueue",
    "async_channel",
    "sync_channel",
    "oneshot",
];

/// The shape one registered block carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockKind {
    /// A markdown table under the marker.
    Table,
    /// A fenced `text` block under the marker.
    Text,
    /// A fenced `rust` block under the marker.
    Rust,
}

impl BlockKind {
    /// The fence language this kind demands, for a fenced block.
    const fn fence(self) -> &'static str {
        match self {
            Self::Rust => "rust",
            Self::Table | Self::Text => "text",
        }
    }
}

/// One registered block: its id, its heading, its shape, and its floor.
#[derive(Debug, Clone, Copy)]
struct BlockSpec {
    /// The id the marker line states.
    id: &'static str,
    /// The `####` heading the block sits under.
    heading: &'static str,
    /// Whether the block is a table, a text fence, or a Rust fence.
    kind: BlockKind,
    /// The fewest rows the block may hold.
    minimum: usize,
}

/// One entry of the block register.
const fn block(
    id: &'static str,
    heading: &'static str,
    kind: BlockKind,
    minimum: usize,
) -> BlockSpec {
    BlockSpec {
        id,
        heading,
        kind,
        minimum,
    }
}

/// Every block and every table this guard reads, by the id its marker states.
///
/// The document states the same id and the same minimum in a marker line on the
/// line before the block; a disagreement is exit 2, because a count the document
/// states and a count the guard expects must be one number (DR7, PG27).
const DATA_BLOCKS: &[BlockSpec] = &[
    block(
        "crate-table",
        "The crate dependency table",
        BlockKind::Table,
        16,
    ),
    block(
        "carrier-table",
        "Every cross-thread carrier, and the two ends it needs",
        BlockKind::Table,
        25,
    ),
    block("name-map", "The third-party name map", BlockKind::Text, 23),
    block("edge-list", "The internal edge list", BlockKind::Text, 18),
    block("framework-types", "Framework types", BlockKind::Text, 7),
    block(
        "ownership-table",
        "The type ownership table",
        BlockKind::Table,
        16,
    ),
    block("drop-list", "The candidate drop list", BlockKind::Text, 8),
    block(
        "constants",
        "Every workspace constant",
        BlockKind::Rust,
        122,
    ),
    block(
        "shared-limits",
        "Every shared limit and its enforcers",
        BlockKind::Text,
        5,
    ),
    block(
        "probe-table",
        "Every rule, its probe, and the recorded result",
        BlockKind::Table,
        64,
    ),
    block(
        "external-verdicts",
        "Every external type, and its verdict",
        BlockKind::Table,
        34,
    ),
    block(
        "external-paths",
        "Where every external name comes from",
        BlockKind::Text,
        70,
    ),
    block(
        "pins",
        "The external crate pins the roster compile uses",
        BlockKind::Text,
        13,
    ),
    block(
        "recorded-sizes",
        "Every size the guard records",
        BlockKind::Text,
        51,
    ),
    block(
        "substitutions",
        "Every substitution the roster compile applies",
        BlockKind::Text,
        8,
    ),
    block(
        "drop-impls",
        "Every declaration with a hand-written Drop impl",
        BlockKind::Text,
        3,
    ),
    block(
        "impl-sites",
        "Every impl block the roster compiles",
        BlockKind::Text,
        65,
    ),
    block(
        "heap-names",
        "Every heap-owning name the audio rules refuse",
        BlockKind::Text,
        12,
    ),
    block(
        "grow-names",
        "Every growable name the audio rules refuse",
        BlockKind::Text,
        10,
    ),
    block(
        "lock-names",
        "Every lock name the audio rules refuse",
        BlockKind::Text,
        10,
    ),
    block(
        "primitive-traits",
        "Every primitive trait set the guard uses",
        BlockKind::Text,
        3,
    ),
    block(
        "primitive-sizes",
        "Every primitive size the guard uses",
        BlockKind::Text,
        16,
    ),
    block(
        "justified-unknowns",
        "The justified unknowns",
        BlockKind::Table,
        2,
    ),
    block(
        "vr1-table",
        "Every VR1 derive and its use",
        BlockKind::Table,
        27,
    ),
    block(
        "audio-owned",
        "Every audio-owned declaration",
        BlockKind::Text,
        44,
    ),
    block(
        "audio-exempt",
        "Every audio-owned field the rule exempts",
        BlockKind::Text,
        1,
    ),
    block(
        "audio-reachable-leaf",
        "Every reachable leaf the closure rule allows",
        BlockKind::Text,
        46,
    ),
    block(
        "block-members",
        "Every registered block and its membership rule",
        BlockKind::Text,
        51,
    ),
    block(
        "budget-table",
        "Every budget and bound",
        BlockKind::Table,
        148,
    ),
    block("b1-convert", "Conversion suppressions", BlockKind::Table, 7),
    block(
        "b1-complexity",
        "Complexity suppressions",
        BlockKind::Table,
        6,
    ),
    block(
        "b1-copy",
        "Expectations for missing_copy_implementations",
        BlockKind::Table,
        4,
    ),
    block(
        "b1-variant",
        "Expectations for variant_size_differences",
        BlockKind::Table,
        3,
    ),
    block("phase-table", "The phase table", BlockKind::Table, 16),
    block(
        "selected-tests",
        "Every test this section selects by name",
        BlockKind::Table,
        8,
    ),
    block(
        "snapshot-table",
        "High-rate traffic: latest value, lock free, no event",
        BlockKind::Table,
        7,
    ),
    block(
        "rule-blocks",
        "Which rule reads which block",
        BlockKind::Table,
        33,
    ),
    block(
        "closure-r16",
        "The revision-16 review, over the frozen document",
        BlockKind::Table,
        46,
    ),
    block(
        "closure-r17",
        "The revision-17 review, over the frozen document",
        BlockKind::Table,
        26,
    ),
    block(
        "closure-r18",
        "The revision-18 review, over the frozen document",
        BlockKind::Table,
        20,
    ),
    block(
        "closure-r19",
        "The revision-19 review, over the frozen document",
        BlockKind::Table,
        45,
    ),
    block(
        "closure-r20",
        "The revision-20 review, over the frozen document",
        BlockKind::Table,
        23,
    ),
    block(
        "closure-r21-inner",
        "The revision-21 inner review, over the frozen document",
        BlockKind::Table,
        13,
    ),
    block(
        "closure-r21",
        "The revision-21 external review, over the frozen document",
        BlockKind::Table,
        45,
    ),
    block(
        "closure-r22-inner",
        "The revision-22 inner review, over the frozen document",
        BlockKind::Table,
        35,
    ),
    block(
        "closure-r23-inner",
        "The revision-23 inner review, over the frozen document",
        BlockKind::Table,
        31,
    ),
    block(
        "line-map",
        "Every chunk line and the crate it owns",
        BlockKind::Text,
        16,
    ),
    block(
        "audio-asserted",
        "Every audio-owned root the closure does not reach",
        BlockKind::Text,
        14,
    ),
    block(
        "phase-pair-exempt",
        "Every same-phase crate edge that does not bind",
        BlockKind::Text,
        24,
    ),
    block(
        "gate-defects",
        "Six planted gate defects, and the rule that catches each one",
        BlockKind::Table,
        6,
    ),
    block(
        "fault-messages",
        "Every engine fault and the line the user reads",
        BlockKind::Table,
        16,
    ),
];

/// The register entry one block id names, or `None`.
fn spec_of(block_id: &str) -> Option<&'static BlockSpec> {
    DATA_BLOCKS.iter().find(|entry| entry.id == block_id)
}

/// Whether the register holds one block id.
fn registered(block_id: &str) -> bool {
    spec_of(block_id).is_some()
}

/// The marker line that anchors one registered block.
const MARKER_PATTERN: &str = r"(?m)^<!-- GUARD BLOCK id=([a-z0-9-]+) rows>=([0-9]+) -->$";

/// Every whitespace-separated token of one line.
fn tokens(line: &str) -> Vec<String> {
    line.split_whitespace().map(str::to_owned).collect()
}

/// One text with every leading and trailing character of `set` removed.
fn trim_set(text: &str, set: &str) -> String {
    text.trim_matches(|found| set.contains(found)).to_owned()
}

/// One text with its surrounding backticks removed.
fn strip_ticks(text: &str) -> String {
    trim_set(text, "`")
}

/// Whether the text is a non-empty run of ASCII digits.
fn all_digits(text: &str) -> bool {
    !text.is_empty() && text.chars().all(|found| found.is_ascii_digit())
}

/// Whether the text holds a cased character and no lowercase one.
fn is_upper(text: &str) -> bool {
    text.chars().any(char::is_uppercase) && !text.chars().any(char::is_lowercase)
}

/// The integer one digit run states, with every underscore dropped.
fn digits_value(text: &str) -> Option<usize> {
    let bare: String = text.chars().filter(|found| *found != '_').collect();
    if !all_digits(&bare) {
        return None;
    }
    bare.parse::<usize>().ok()
}

/// One finding with a subject and a reason, as the run prints it.
type Pairs = Vec<(String, String)>;

/// The section 1.9 external-verdict table, as its verdicts and its trait sets.
type Verdicts = (BTreeMap<String, String>, BTreeMap<String, TraitPair>);

/// The head one exemption row allows, by the `Type.field` path it names.
type Exemptions = BTreeMap<(String, String), String>;

/// The section 1.5 table, as the crate per type and the sections per crate.
type Ownership = (BTreeMap<String, String>, BTreeMap<String, BTreeSet<String>>);

/// Every VR1 derive with no stated use, as the run prints it.
type Vr1Rows = Vec<(String, String, Vec<String>)>;

/// Every constant, with the crate that declares it and its integer value.
type ConstantOwners = BTreeMap<String, (String, Option<usize>)>;

/// Every crate dependency a declaration proves, as a crate and a pin.
type Proven = BTreeSet<(String, String)>;

/// Every arm of one enum, with the fields each arm states.
type ArmFields = Vec<(String, Vec<(String, String)>)>;

/// A borrowed view of the arms one enum declares.
type ArmFieldList = [(String, Vec<(String, String)>)];

/// One sized declaration: its name, its crate, its size, its alignment, its arms.
type SizeRow = (String, String, usize, usize, Vec<(String, usize)>);

/// One declaration the model cannot size, with every field that stopped it.
type OpenRow = (String, String, Pairs);

/// Every floor the document should state, by block id.
type Floors = BTreeMap<&'static str, usize>;

/// One finding with three fields, as the run prints it.
type Triples = Vec<(String, String, String)>;

/// One finding with four fields, as the run prints it.
type Quad = (String, String, String, String);

/// One finding with five fields, as the run prints it.
type Quint = (String, String, String, String, String);

/// A counted run and every finding it produced.
type Counted = (usize, Pairs);

/// Every enum arm this document declares, by the enum that declares it.
type ArmMap = BTreeMap<String, Vec<(String, String)>>;

/// A size and an alignment, by the expression or the name that carries it.
type SizeMap = BTreeMap<String, (usize, usize)>;

/// One row of a registered block.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Row {
    /// The cells of one markdown table row.
    Cells(Vec<String>),
    /// One non-empty line of a fenced block.
    Line(String),
}

impl Row {
    /// The cells of a table row, or nothing for a fenced line.
    fn cells(&self) -> &[String] {
        match *self {
            Self::Cells(ref cells) => cells,
            Self::Line(_) => &[],
        }
    }

    /// One cell by index, or the empty string.
    fn cell(&self, index: usize) -> String {
        self.cells().get(index).cloned().unwrap_or_default()
    }

    /// The row as one line of text, as a membership rule reads it.
    fn text(&self) -> String {
        match *self {
            Self::Cells(ref cells) => cells.join(" | "),
            Self::Line(ref line) => line.clone(),
        }
    }

    /// The whitespace tokens of a fenced line, or nothing for a table row.
    fn tokens(&self) -> Vec<String> {
        match *self {
            Self::Cells(_) => Vec::new(),
            Self::Line(ref line) => tokens(line),
        }
    }

    /// The fenced line this row holds, or the empty string for a table row.
    fn line(&self) -> String {
        match *self {
            Self::Cells(_) => String::new(),
            Self::Line(ref line) => line.clone(),
        }
    }
}

/// Every registered block the document holds, in register order.
#[derive(Debug, Default)]
struct Blocks {
    /// One entry per block that parsed, in register order.
    entries: Vec<(&'static str, Vec<Row>)>,
}

impl Blocks {
    /// The rows of one block, or `None` when the block did not parse.
    fn get(&self, block_id: &str) -> Option<&[Row]> {
        self.entries
            .iter()
            .find(|(id, _rows)| *id == block_id)
            .map(|(_id, rows)| rows.as_slice())
    }

    /// The rows of one block, or nothing when the block did not parse.
    fn rows(&self, block_id: &str) -> &[Row] {
        self.get(block_id).unwrap_or(&[])
    }

    /// Whether the guard parsed one block.
    fn holds(&self, block_id: &str) -> bool {
        self.get(block_id).is_some()
    }

    /// Every parsed block, in register order.
    fn iter(&self) -> impl Iterator<Item = (&'static str, &[Row])> {
        self.entries.iter().map(|(id, rows)| (*id, rows.as_slice()))
    }
}

/// The cells of one markdown table row, as a renderer reads them.
///
/// It splits on every `|` the author did NOT escape and then unescapes each
/// cell, because `\|` inside a backtick span is one literal pipe and not a cell
/// wall (critic C21-5).
fn split_row(line: &str) -> anyhow::Result<Vec<String>> {
    let wall = pattern::build(r"(?<!\\)\|")?;
    let pieces = wall.split(&pattern::chars(line));
    let last = pieces.len().saturating_sub(1);
    Ok(pieces
        .into_iter()
        .enumerate()
        .filter(|(index, _piece)| *index > 0 && *index < last)
        .map(|(_index, piece)| piece.replace("\\|", "|").trim().to_owned())
        .collect())
}

/// How many cells one markdown row holds.
///
/// A renderer splits on every `|` the author did not escape, so this splits on
/// the same rule.
fn cell_count(row: &str) -> anyhow::Result<usize> {
    let wall = pattern::build(r"(?<!\\)\|")?;
    Ok(wall.split(&pattern::chars(row)).len().saturating_sub(2))
}

/// One registered block, as its rows or the reason it does not read.
///
/// A registered block carries a `####` heading of its own, a marker line on the
/// line before its opening fence or its table header, a minimum row count inside
/// that marker, and a row in [`DATA_BLOCKS`]. Every one of those is checked here
/// (PG27, DR7).
///
/// The anchor is the marker and never a search: revision 12 read each block with
/// a lazy pattern from the heading to the next fence, so a deleted block under a
/// kept heading parsed the next fence in the document (critic CR-14, WR-16).
fn read_block(
    source: &[char],
    spec: &BlockSpec,
    register: bool,
) -> anyhow::Result<Result<Vec<Row>, String>> {
    let heading = pattern::build(&format!(r"(?m)^#### {}\s*$", pattern::quote(spec.heading)))?;
    let found = heading.find_iter(source);
    if found.len() != 1 {
        return Ok(Err(format!(
            "the heading `#### {}` appears {} times",
            spec.heading,
            found.len()
        )));
    }
    let start = found.first().map_or(0, pattern::Match::end);
    let Some(rest) = source.get(start..) else {
        return Ok(Err("the heading ends past the document".to_owned()));
    };
    let next_heading = pattern::build(r"(?m)^#{1,4} ")?;
    let region = next_heading
        .find(rest)
        .map_or(rest, |stop| rest.get(..stop.start()).unwrap_or(rest));
    let marker = pattern::build(MARKER_PATTERN)?;
    let marks = marker.find_iter(region);
    if marks.len() != 1 {
        return Ok(Err(format!(
            "the marker line appears {} times under the heading",
            marks.len()
        )));
    }
    let Some(mark) = marks.first() else {
        return Ok(Err("the marker line is lost".to_owned()));
    };
    let stated_id = mark.text(1, region);
    if stated_id != spec.id {
        return Ok(Err(format!(
            "the marker states id `{stated_id}` and the register states `{}`",
            spec.id
        )));
    }
    let stated_rows = mark.text(2, region);
    let stated = digits_value(&stated_rows).unwrap_or(usize::MAX);
    if register && stated != spec.minimum {
        return Ok(Err(format!(
            "the marker states rows>={stated_rows} and the register states {}",
            spec.minimum
        )));
    }
    let minimum = if register { spec.minimum } else { stated };
    let Some(tail) = region.get(mark.end()..) else {
        return Ok(Err("the marker ends past the block".to_owned()));
    };
    if tail.first() != Some(&'\n') {
        return Ok(Err("the marker does not end its own line".to_owned()));
    }
    let Some(tail) = tail.get(1..) else {
        return Ok(Err("the marker ends the document".to_owned()));
    };
    let rows = match read_rows(tail, spec.kind)? {
        Ok(rows) => rows,
        Err(reason) => return Ok(Err(reason)),
    };
    if rows.len() < minimum {
        return Ok(Err(format!(
            "it holds {} rows and the stated minimum is {minimum}",
            rows.len()
        )));
    }
    Ok(Ok(rows))
}

/// The rows one block body states, or the reason the body does not read.
fn read_rows(tail: &[char], kind: BlockKind) -> anyhow::Result<Result<Vec<Row>, String>> {
    if kind == BlockKind::Table {
        let header = pattern::build(r"\|[^\n]*\|[ \t]*\n\|[-\s|:]+\|[ \t]*\n")?;
        let Some(head) = header.match_at(tail, 0) else {
            return Ok(Err("no table header follows the marker".to_owned()));
        };
        let Some(body) = tail.get(head.end()..) else {
            return Ok(Err("the table header ends the document".to_owned()));
        };
        let text: String = body.iter().collect();
        let mut rows = Vec::new();
        for line in text.split('\n') {
            if !line.starts_with('|') {
                break;
            }
            rows.push(Row::Cells(split_row(line)?));
        }
        return Ok(Ok(rows));
    }
    let fence = pattern::build(r"(?ms)```([a-z]*)\n(.*?)^```")?;
    let Some(found) = fence.match_at(tail, 0) else {
        return Ok(Err("no fenced block follows the marker".to_owned()));
    };
    let language = found.text(1, tail);
    if language != kind.fence() {
        return Ok(Err(format!(
            "the fence is `{language}` and `{}` is required",
            kind.fence()
        )));
    }
    let body = found.text(2, tail);
    Ok(Ok(body
        .split('\n')
        .filter(|line| !line.trim().is_empty())
        .map(|line| Row::Line(line.to_owned()))
        .collect()))
}

/// Every registered block, with the reason each absent one does not read.
///
/// The register and the section 1.9 rule-to-block map are held to one set here,
/// so a block the register holds and no rule claims, and a claim over an id the
/// register does not hold, are each a failure (PG27).
fn read_blocks(source: &[char]) -> anyhow::Result<(Blocks, Pairs)> {
    let mut parsed = Blocks::default();
    let mut failures = Vec::new();
    for spec in DATA_BLOCKS {
        match read_block(source, spec, true)? {
            Ok(rows) => parsed.entries.push((spec.id, rows)),
            Err(reason) => failures.push((spec.id.to_owned(), reason)),
        }
    }
    if !parsed.holds("rule-blocks") {
        return Ok((parsed, failures));
    }
    let span = pattern::build(r"`([a-z0-9-]+)`")?;
    let mut claimed = BTreeSet::new();
    for row in parsed.rows("rule-blocks") {
        let cells = row.cells();
        if cells.len() >= 2 {
            let cell = pattern::chars(&row.cell(1));
            for one in span.find_iter(&cell) {
                claimed.insert(one.text(1, &cell));
            }
        }
    }
    let held: BTreeSet<String> = DATA_BLOCKS.iter().map(|spec| spec.id.to_owned()).collect();
    for block_id in held.difference(&claimed) {
        failures.push((
            block_id.clone(),
            "the section 1.9 rule-to-block map claims no rule for it".to_owned(),
        ));
    }
    for block_id in claimed.difference(&held) {
        failures.push((
            block_id.clone(),
            "the rule-to-block map names it and the register does not hold it".to_owned(),
        ));
    }
    Ok((parsed, failures))
}

/// Every `GUARD BLOCK` marker this document carries, checked (PG27).
///
/// `read_block` anchors a block on the marker inside its own heading region, so
/// a marker that sits outside that region, and a marker whose id the register
/// does not hold, are both invisible to it (critic C-13, C-14).
fn stray_markers(source: &[char]) -> anyhow::Result<Vec<(String, String)>> {
    let marker = pattern::build(MARKER_PATTERN)?;
    let mut seen: BTreeMap<String, usize> = BTreeMap::new();
    for one in marker.find_iter(source) {
        *seen.entry(one.text(1, source)).or_insert(0) += 1;
    }
    let mut failures = Vec::new();
    for (block_id, count) in seen {
        if !registered(&block_id) {
            failures.push((
                block_id,
                "the marker id is not a registered block".to_owned(),
            ));
            continue;
        }
        if count != 1 {
            failures.push((
                block_id,
                format!("the marker appears {count} times in this document"),
            ));
        }
    }
    Ok(failures)
}

/// A map that keeps the order its keys were first inserted.
#[derive(Debug, Clone)]
struct OrderedMap<V> {
    /// Every key, in the order the map first saw it.
    order: Vec<String>,
    /// Every value, by key.
    values: BTreeMap<String, V>,
}

impl<V> Default for OrderedMap<V> {
    fn default() -> Self {
        Self {
            order: Vec::new(),
            values: BTreeMap::new(),
        }
    }
}

impl<V: Default> OrderedMap<V> {
    /// The value one key holds, creating an empty one when the key is new.
    fn entry(&mut self, key: &str) -> &mut V {
        if !self.values.contains_key(key) {
            self.order.push(key.to_owned());
            self.values.insert(key.to_owned(), V::default());
        }
        self.values.entry(key.to_owned()).or_default()
    }
}

impl<V> OrderedMap<V> {
    /// The value one key holds.
    fn get(&self, key: &str) -> Option<&V> {
        self.values.get(key)
    }

    /// Whether the map holds one key.
    fn holds(&self, key: &str) -> bool {
        self.values.contains_key(key)
    }

    /// How many keys the map holds.
    const fn len(&self) -> usize {
        self.order.len()
    }

    /// Every key and value, in insertion order.
    fn iter(&self) -> impl Iterator<Item = (&String, &V)> {
        self.order
            .iter()
            .filter_map(|key| self.values.get(key).map(|value| (key, value)))
    }

    /// Every key, in name order.
    fn sorted_keys(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    /// Every key, as a set.
    fn key_set(&self) -> BTreeSet<String> {
        self.values.keys().cloned().collect()
    }
}

/// One declaration a Rust block states.
#[derive(Debug, Clone, Default)]
struct Decl {
    /// Whether the declaration is a struct, an enum, or a trait.
    kind: String,
    /// The declaration body, with every enum arm name masked.
    payload: String,
    /// The attributes that sit above the declaration.
    attrs: String,
}

/// Every declaration one parse found, by name, in document order.
type DeclMap = OrderedMap<Vec<Decl>>;

impl DeclMap {
    /// The first declaration one name states.
    fn first(&self, name: &str) -> Option<&Decl> {
        self.get(name).and_then(|entries| entries.first())
    }
}

/// Every whitespace token of a text block, as a list.
fn token_rows(rows: &[Row]) -> Vec<String> {
    rows.iter().flat_map(|row| tokens(&row.line())).collect()
}

/// Every code span of one markdown cell that names an identifier.
fn cell_names(cell: &str) -> anyhow::Result<Vec<String>> {
    let span = pattern::build(r"`([A-Za-z0-9_]+)`")?;
    let text = pattern::chars(cell);
    Ok(span
        .find_iter(&text)
        .iter()
        .map(|one| one.text(1, &text))
        .collect())
}

/// Every match of one pattern's first group over one text.
fn captures_of(source: &str, expression: &str) -> anyhow::Result<Vec<String>> {
    let compiled = pattern::build(expression)?;
    let text = pattern::chars(source);
    Ok(compiled
        .find_iter(&text)
        .iter()
        .map(|one| one.text(1, &text))
        .collect())
}

/// Whether one pattern matches anywhere in one text.
fn holds_pattern(source: &str, expression: &str) -> anyhow::Result<bool> {
    Ok(pattern::build(expression)?.is_match(&pattern::chars(source)))
}

/// The section 1.5 candidate drop list, as a set of names (PG4).
fn candidate_drop_list(blocks: &Blocks) -> BTreeSet<String> {
    token_rows(blocks.rows("drop-list")).into_iter().collect()
}

/// The closure traits one external row supplies, and the unconditional ones.
type TraitPair = (BTreeSet<String>, BTreeSet<String>);

/// The section 1.9 external-verdict table.
///
/// PG18 reads the verdict. There is no drop list: every external type a
/// declaration names has a row with `Copy`, `not Copy`, `transparent`, or
/// `undecided` (critic R2). PG19 reads the `Traits` cell, where a trait written
/// with a trailing `!` is unconditional.
fn external_verdicts(blocks: &Blocks) -> anyhow::Result<Verdicts> {
    let mut verdicts = BTreeMap::new();
    let mut traits: BTreeMap<String, TraitPair> = BTreeMap::new();
    let span = pattern::build(r"`([A-Za-z]+!?)`")?;
    for row in blocks.rows("external-verdicts") {
        let cells = row.cells();
        if cells.len() < 4 {
            continue;
        }
        let verdict = strip_ticks(&row.cell(1)).to_lowercase();
        let mut supplied = BTreeSet::new();
        let mut unconditional = BTreeSet::new();
        let cell = pattern::chars(&row.cell(2));
        for one in span.find_iter(&cell) {
            let token = one.text(1, &cell);
            let bare = token.trim_end_matches('!').to_owned();
            if !CLOSURE_TRAITS.contains(&bare.as_str()) {
                continue;
            }
            if token.ends_with('!') {
                unconditional.insert(bare.clone());
            }
            supplied.insert(bare);
        }
        for name in cell_names(&row.cell(0))? {
            verdicts.insert(name.clone(), verdict.clone());
            traits.insert(name, (supplied.clone(), unconditional.clone()));
        }
    }
    Ok((verdicts, traits))
}

/// Every external name whose section 1.9 `Why` cell states one word.
fn externals_saying(blocks: &Blocks, word: &str) -> anyhow::Result<BTreeSet<String>> {
    let says = pattern::build(&format!(r"(?i)\b{}\b", pattern::quote(word)))?;
    let mut names = BTreeSet::new();
    for row in blocks.rows("external-verdicts") {
        let cells = row.cells();
        if cells.len() < 4 || !says.is_match(&pattern::chars(&row.cell(3))) {
            continue;
        }
        names.extend(cell_names(&row.cell(0))?);
    }
    Ok(names)
}

/// The section 1.9 primitive block, as a trait set per name (PG4, PG19).
///
/// Each line holds one colon token, the names stand to its left, and the traits
/// every one of those names supplies stand to its right.
fn primitive_traits(blocks: &Blocks) -> BTreeMap<String, BTreeSet<String>> {
    let mut supplied = BTreeMap::new();
    for row in blocks.rows("primitive-traits") {
        let line = row.line();
        let halves: Vec<&str> = line.split(':').collect();
        if halves.len() != 2 {
            continue;
        }
        let traits: BTreeSet<String> = halves
            .get(1)
            .map(|half| {
                half.split_whitespace()
                    .filter(|token| CLOSURE_TRAITS.contains(token))
                    .map(str::to_owned)
                    .collect()
            })
            .unwrap_or_default();
        for name in halves.first().map(|half| tokens(half)).unwrap_or_default() {
            supplied.insert(name, traits.clone());
        }
    }
    supplied
}

/// The section 1.9 primitive-size block, as a size and alignment per name.
fn primitive_sizes(blocks: &Blocks) -> BTreeMap<String, (usize, usize)> {
    let mut sizes = BTreeMap::new();
    for row in blocks.rows("primitive-sizes") {
        let parts = row.tokens();
        if parts.len() != 3 {
            continue;
        }
        let (Some(name), Some(size), Some(align)) = (parts.first(), parts.get(1), parts.get(2))
        else {
            continue;
        };
        if !all_digits(size) || !all_digits(align) {
            continue;
        }
        let (Some(size), Some(align)) = (size.parse().ok(), align.parse().ok()) else {
            continue;
        };
        sizes.insert(name.clone(), (size, align));
    }
    sizes
}

/// The section 5.7 audio-owned set, and every token section 1.5 places nowhere.
fn audio_owned(
    blocks: &Blocks,
    owned: &BTreeMap<String, String>,
) -> (BTreeSet<String>, Vec<String>) {
    let mut names = BTreeSet::new();
    let mut bad = Vec::new();
    for token in token_rows(blocks.rows("audio-owned")) {
        if owned.contains_key(&token) {
            names.insert(token);
        } else {
            bad.push(token);
        }
    }
    (names, bad)
}

/// The section 1.9 Drop block, as its names and every token 1.5 places nowhere.
fn drop_impl_names(
    blocks: &Blocks,
    owned: &BTreeMap<String, String>,
) -> (BTreeSet<String>, Vec<String>) {
    let mut names = BTreeSet::new();
    let mut bad = Vec::new();
    for token in token_rows(blocks.rows("drop-impls")) {
        if owned.contains_key(&token) {
            names.insert(token);
        } else {
            bad.push(token);
        }
    }
    (names, bad)
}

/// The section 5.7 exemption block, as a head per `Type.field` path.
///
/// Each line is one `Type.field` path, then the HEAD type the row exempts, then
/// a reason token. The head is part of the row: a field whose head is anything
/// else falls through to the heap test (critic CR on the exemption).
fn audio_exempt(blocks: &Blocks, owned: &BTreeMap<String, String>) -> (Exemptions, Vec<String>) {
    let mut paths = BTreeMap::new();
    let mut bad = Vec::new();
    for row in blocks.rows("audio-exempt") {
        let line = row.line();
        let parts = tokens(&line);
        let Some(first) = parts.first() else {
            bad.push(line.trim().to_owned());
            continue;
        };
        if parts.len() < 3 || !first.contains('.') {
            bad.push(line.trim().to_owned());
            continue;
        }
        let (holder, field) = first.split_once('.').unwrap_or((first.as_str(), ""));
        let head = parts.get(1).cloned().unwrap_or_default();
        let upper = head.chars().next().is_some_and(char::is_uppercase);
        if !owned.contains_key(holder) || field.is_empty() || !upper {
            bad.push(line.trim().to_owned());
            continue;
        }
        paths.insert((holder.to_owned(), field.to_owned()), head);
    }
    (paths, bad)
}

/// The outermost head name of a field type expression, or `None` (PG26).
fn expr_head(text: &str) -> Option<String> {
    let head = text.trim();
    if head.starts_with('&') || head.starts_with('[') || head.starts_with('(') {
        return None;
    }
    let head = head
        .split_once('<')
        .map_or(head, |(left, _rest)| left)
        .trim();
    let head = head.rsplit("::").next().unwrap_or(head).trim();
    if head.is_empty() {
        return None;
    }
    Some(head.to_owned())
}

/// Every name the section 1.9 external-path block maps (PG27, PG25).
fn external_path_names(blocks: &Blocks) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    for row in blocks.rows("external-paths") {
        let parts = row.tokens();
        if parts.len() >= 2 {
            names.extend(parts.first().cloned());
        }
    }
    names
}

/// The framework type list that section 1.3 declares.
fn framework_names(blocks: &Blocks) -> BTreeSet<String> {
    token_rows(blocks.rows("framework-types"))
        .into_iter()
        .collect()
}

/// The section 1.2 third-party name map, and every row it cannot resolve.
///
/// It fails closed on a row it cannot resolve (critic WR-16). A line that is not
/// one name and one crate, or that names a crate no section 1.2 row carries, is
/// a damaged row rather than a mapping the guard may skip.
fn name_map(
    blocks: &Blocks,
    rows: &BTreeMap<String, BTreeSet<String>>,
) -> (BTreeMap<String, String>, Vec<String>) {
    let known: BTreeSet<String> = rows.values().flatten().cloned().collect();
    let mut mapping = BTreeMap::new();
    let mut bad = Vec::new();
    for row in blocks.rows("name-map") {
        let line = row.line();
        let parts = tokens(&line);
        let (Some(name), Some(crate_name)) = (parts.first(), parts.get(1)) else {
            bad.push(line.trim().to_owned());
            continue;
        };
        if parts.len() != 2 {
            bad.push(line.trim().to_owned());
            continue;
        }
        if !known.is_empty() && !known.contains(crate_name) {
            bad.push(line.trim().to_owned());
            continue;
        }
        mapping.insert(name.clone(), crate_name.clone());
    }
    (mapping, bad)
}

/// The 1.3 spelling of a 1.5 crate name.
fn normalize_crate(name: &str) -> String {
    if name == "crates/duet" {
        "duet".to_owned()
    } else {
        name.to_owned()
    }
}

/// The section 1.5 table, as the crate per type and the sections per crate.
fn ownership_table(blocks: &Blocks) -> anyhow::Result<Ownership> {
    let mut owned = BTreeMap::new();
    let mut declared_in: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let section = pattern::build(r"\d+\.\d+[a-z]?")?;
    for row in blocks.rows("ownership-table") {
        let cells = row.cells();
        if cells.len() < 2 {
            continue;
        }
        let crate_name = strip_ticks(&row.cell(0));
        for name in cell_names(&row.cell(1))? {
            owned.insert(name, crate_name.clone());
        }
        if cells.len() >= 3 {
            let cell = pattern::chars(&row.cell(2));
            let found: BTreeSet<String> = section
                .find_iter(&cell)
                .iter()
                .map(|one| one.text(0, &cell))
                .collect();
            declared_in.insert(crate_name, found);
        }
    }
    Ok((owned, declared_in))
}

/// The section 1.2 crate table, as the third-party crates each row names.
fn dependency_table(blocks: &Blocks) -> anyhow::Result<BTreeMap<String, BTreeSet<String>>> {
    let mut rows = BTreeMap::new();
    let span = pattern::build(r"`([A-Za-z_][A-Za-z0-9_-]*)`")?;
    for row in blocks.rows("crate-table") {
        let cells = row.cells();
        if cells.len() < 4 {
            continue;
        }
        let crate_name = strip_ticks(&row.cell(0));
        if !crate_name.starts_with("duet") {
            continue;
        }
        let cell = pattern::chars(&row.cell(3));
        let found: BTreeSet<String> = span
            .find_iter(&cell)
            .iter()
            .map(|one| one.text(1, &cell))
            .collect();
        rows.insert(crate_name, found);
    }
    Ok(rows)
}

/// The section 1.3 edge list, as the dependencies of each crate.
fn edge_list(blocks: &Blocks) -> anyhow::Result<BTreeMap<String, BTreeSet<String>>> {
    let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let word = pattern::build(r"[a-z][a-z-]*")?;
    let mut current: Option<String> = None;
    for row in blocks.rows("edge-list") {
        let line = row.line();
        let split = line.split_once("->");
        if split.is_none() && current.is_none() {
            continue;
        }
        let right = match split {
            Some((left, tail)) => {
                let name = left.trim().to_owned();
                edges.entry(name.clone()).or_default();
                current = Some(name);
                tail.to_owned()
            },
            None => line.clone(),
        };
        let Some(ref owner) = current else {
            continue;
        };
        let text = pattern::chars(&right);
        for one in word.find_iter(&text) {
            let name = one.text(0, &text);
            if name.starts_with("duet") {
                edges.entry(owner.clone()).or_default().insert(name);
            }
        }
    }
    Ok(edges)
}

/// The transitive closure of the section 1.3 graph.
fn reachable(edges: &BTreeMap<String, BTreeSet<String>>) -> BTreeMap<String, BTreeSet<String>> {
    let mut every: BTreeSet<String> = edges.keys().cloned().collect();
    every.extend(edges.values().flatten().cloned());
    let mut closure = BTreeMap::new();
    for crate_name in every {
        let mut seen: BTreeSet<String> = BTreeSet::new();
        let mut stack: Vec<String> = edges
            .get(&crate_name)
            .map(|targets| targets.iter().cloned().collect())
            .unwrap_or_default();
        while let Some(target) = stack.pop() {
            if !seen.insert(target.clone()) {
                continue;
            }
            if let Some(next) = edges.get(&target) {
                stack.extend(next.iter().cloned());
            }
        }
        closure.insert(crate_name, seen);
    }
    closure
}

/// One `### N.N` heading and where it starts.
type SectionMark = (usize, String);

/// Every `### N.N` heading of the document, with its offset.
fn section_of(source: &[char]) -> anyhow::Result<Vec<SectionMark>> {
    let heading = pattern::build(r"(?m)^### (\d+\.\d+[a-z]?) ")?;
    Ok(heading
        .find_iter(source)
        .iter()
        .map(|one| (one.start(), one.text(1, source)))
        .collect())
}

/// The section number that contains one offset, or `None`.
fn section_at(marks: &[SectionMark], position: usize) -> Option<String> {
    let mut current = None;
    for (start, number) in marks {
        if *start <= position {
            current = Some(number.clone());
        } else {
            break;
        }
    }
    current
}

/// Every fenced Rust block, as its offset and its code.
fn rust_blocks(source: &[char]) -> anyhow::Result<Vec<(usize, String)>> {
    let fence = pattern::build(r"(?s)```rust\n(.*?)```")?;
    Ok(fence
        .find_iter(source)
        .iter()
        .map(|one| (one.start(), one.text(1, source)))
        .collect())
}

/// Code with every line comment removed; a name in a comment is prose.
fn strip_comments(code: &str) -> anyhow::Result<String> {
    let comment = pattern::build(r"//[^\n]*")?;
    Ok(comment.replace_all(&pattern::chars(code), &mut |_one, _text| String::new()))
}

/// Code with every block comment removed, such as `/* private */`.
fn strip_block_comments(code: &str) -> anyhow::Result<String> {
    let comment = pattern::build(r"(?s)/\*.*?\*/")?;
    Ok(comment.replace_all(&pattern::chars(code), &mut |_one, _text| " ".to_owned()))
}

/// Code with every `::Name` segment removed.
fn strip_paths(code: &str) -> anyhow::Result<String> {
    let path = pattern::build(r"::[A-Z][A-Za-z0-9]*")?;
    Ok(path.replace_all(&pattern::chars(code), &mut |_one, _text| "::".to_owned()))
}

/// The index of the brace that closes the one at `open_index`.
fn matching_brace(code: &[char], open_index: usize) -> usize {
    let mut depth = 0_i64;
    let mut index = open_index;
    while index < code.len() {
        match code.get(index) {
            Some(&'{') => depth = depth.saturating_add(1),
            Some(&'}') => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return index;
                }
            },
            Some(_) | None => {},
        }
        index = index.saturating_add(1);
    }
    code.len()
}

/// Each top-level arm of an enum body, as raw text.
fn top_level_arms(body: &str) -> Vec<String> {
    let mut depth = 0_i64;
    let mut start = 0_usize;
    let mut out = Vec::new();
    let chars: Vec<char> = body.chars().collect();
    for (index, found) in chars.iter().enumerate() {
        match *found {
            '(' | '{' | '[' => depth = depth.saturating_add(1),
            ')' | '}' | ']' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                out.push(pattern::slice(&chars, start, index).unwrap_or_default());
                start = index.saturating_add(1);
            },
            _ => {},
        }
    }
    out.push(pattern::slice(&chars, start, chars.len()).unwrap_or_default());
    out.into_iter()
        .filter(|arm| !arm.trim().is_empty())
        .collect()
}

/// An enum body with the leading identifier of each top-level arm removed.
///
/// An arm name is not a type: `StripKind::Track` must not make the guard read
/// the struct `Track` as a field of `StripKind` (PG8).
fn mask_arms(body: &str) -> anyhow::Result<String> {
    let lead = pattern::build(r"^(\s*)[A-Z][A-Za-z0-9]*")?;
    let mut parts = Vec::new();
    for arm in top_level_arms(body) {
        let text = pattern::chars(&arm);
        let masked = lead.match_at(&text, 0).map_or(arm, |found| {
            let keep = found.text(1, &text);
            let tail = pattern::slice(&text, found.end(), text.len()).unwrap_or_default();
            format!("{keep}{tail}")
        });
        parts.push(masked);
    }
    Ok(parts.join(","))
}

/// Every declaration in one block, by name, in the order the block states them.
///
/// `payload` is the declaration body with enum arm names masked, so it holds
/// field types and nothing else. A tuple body opens before the next `{` and
/// before the next `;`, so a unit struct yields an empty body (critic C-6).
fn declarations(code: &str) -> anyhow::Result<DeclMap> {
    let header = pattern::build(
        r"((?:#\[[^\]]*\]\s*|#\[[^;{}]*?\]\s*)*)(?:pub(?:\([a-z]+\))?\s+)?(struct|enum|trait)\s+([A-Z][A-Za-z0-9]*)",
    )?;
    let text = pattern::chars(code);
    let mut found = DeclMap::default();
    for one in header.find_iter(&text) {
        let attrs = one.text(1, &text);
        let kind = one.text(2, &text);
        let name = one.text(3, &text);
        let body = declaration_body(&text, one.end(), &kind);
        let payload = if kind == "enum" {
            mask_arms(&body)?
        } else {
            body
        };
        found.entry(&name).push(Decl {
            kind,
            payload,
            attrs,
        });
    }
    Ok(found)
}

/// The body one declaration states, from the position its header ends at.
fn declaration_body(text: &[char], after: usize, kind: &str) -> String {
    let tail = text.get(after..).unwrap_or(&[]);
    let brace = tail.iter().position(|found| *found == '{');
    let semi = tail.iter().position(|found| *found == ';');
    let paren = tail.iter().position(|found| *found == '(');
    let limit = match (brace, semi) {
        (Some(one), Some(two)) => one.min(two),
        (Some(one), None) => one,
        (None, Some(two)) => two,
        (None, None) => tail.len(),
    };
    let tuple = paren.filter(|open| kind == "struct" && *open < limit);
    if let Some(open) = tuple {
        let close = tail
            .iter()
            .position(|found| *found == ')')
            .unwrap_or(tail.len());
        return pattern::slice(tail, open.saturating_add(1), close).unwrap_or_default();
    }
    let Some(open) = brace else {
        return String::new();
    };
    if semi.is_some_and(|mark| mark < open) {
        return String::new();
    }
    let absolute = after.saturating_add(open);
    pattern::slice(
        text,
        absolute.saturating_add(1),
        matching_brace(text, absolute),
    )
    .unwrap_or_default()
}

/// Every `impl Trait for Type` this document writes by hand.
///
/// `Finite` carries five hand-written impls, because a derive over an `f64`
/// field does not compile (section 2.6a). PG19 and PG22 read them, so a hand
/// impl counts exactly as a derive does.
fn hand_impls(source: &[char]) -> anyhow::Result<BTreeMap<String, BTreeSet<String>>> {
    let header = pattern::build(
        r"\bimpl\b(?:\s*<[^>]*>)?\s+([A-Z][A-Za-z0-9]*)(?:\s*<[^>]*>)?\s+for\s+([A-Z][A-Za-z0-9]*)",
    )?;
    let mut found: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (_offset, block) in rust_blocks(source)? {
        let code = pattern::chars(&strip_comments(&block)?);
        for one in header.find_iter(&code) {
            let trait_name = one.text(1, &code);
            let type_name = one.text(2, &code);
            if CLOSURE_TRAITS.contains(&trait_name.as_str()) {
                found.entry(type_name).or_default().insert(trait_name);
            }
        }
    }
    Ok(found)
}

/// Code with the leading identifier of each top-level enum arm removed.
///
/// An arm name is masked only inside the enum that declares it (PG8). The guard
/// keeps no global variant set, so a struct named `Reverb`, `Track`, or `Peak`
/// is still a candidate.
fn mask_enum_arm_names(code: &str) -> anyhow::Result<String> {
    let header = pattern::build(r"\benum\s+[A-Z][A-Za-z0-9]*\s*\{")?;
    let mut out: Vec<char> = pattern::chars(code);
    let found = header.find_iter(&out);
    for one in found.iter().rev() {
        let Some(open) = out.iter().skip(one.start()).position(|c| *c == '{') else {
            continue;
        };
        let open_index = one.start().saturating_add(open);
        let close_index = matching_brace(&out, open_index);
        let body =
            pattern::slice(&out, open_index.saturating_add(1), close_index).unwrap_or_default();
        let masked = mask_arms(&body)?;
        let head = pattern::slice(&out, 0, open_index.saturating_add(1)).unwrap_or_default();
        let tail = pattern::slice(&out, close_index, out.len()).unwrap_or_default();
        out = pattern::chars(&format!("{head}{masked}{tail}"));
    }
    Ok(out.into_iter().collect())
}

/// Every name in a type position.
fn used_types(code: &str) -> anyhow::Result<BTreeSet<String>> {
    let lead =
        pattern::build(r"(?::|->|impl|<|,)\s*&?(?:mut\s+)?(?:'[a-z]+\s+)?([A-Z][A-Za-z0-9]*)")?;
    let applied = pattern::build(r"\b([A-Z][A-Za-z0-9]*)\s*<")?;
    let text = pattern::chars(code);
    let mut used = BTreeSet::new();
    for one in lead.find_iter(&text) {
        used.insert(one.text(1, &text));
    }
    for one in applied.find_iter(&text) {
        used.insert(one.text(1, &text));
    }
    Ok(used)
}

/// Every type name a struct body or a masked enum body mentions.
///
/// An all-upper token that neither the 1.5 table nor the external table names is
/// a constant, such as `MAX_STRIPS`, and not a type. `I24` is placed, so it
/// stays (critic R2).
fn body_type_names(body: &str, known: &BTreeSet<String>) -> anyhow::Result<BTreeSet<String>> {
    let cleaned = strip_paths(&strip_comments(body)?)?;
    let word = pattern::build(r"\b([A-Z][A-Za-z0-9]*)\b")?;
    let text = pattern::chars(&cleaned);
    let mut names = BTreeSet::new();
    for one in word.find_iter(&text) {
        let name = one.text(1, &text);
        if known.contains(&name) || !is_upper(&name) || name.chars().count() == 1 {
            names.insert(name);
        }
    }
    Ok(names)
}

/// The last path segment of every type-shaped token in a body.
///
/// `gpui_kit::Px` gives `Px`, and a bare `Px` gives `Px`. PG7 runs on the text
/// before the paths are stripped, so a qualified spelling cannot hide a
/// framework name (critic N8).
fn last_segments(body: &str) -> anyhow::Result<BTreeSet<String>> {
    let cleaned = strip_comments(body)?;
    let path = pattern::build(r"(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Z][A-Za-z0-9]*)\b")?;
    let text = pattern::chars(&cleaned);
    let mut names = BTreeSet::new();
    for one in path.find_iter(&text) {
        let name = one.text(1, &text);
        if !is_upper(&name) || name.chars().count() == 1 {
            names.insert(name);
        }
    }
    Ok(names)
}

/// Every declaration whose doc comment carries the audio-owned marker.
///
/// This is the SECOND source of the root set (critic C15-3). The block alone is
/// one source, so a one-for-one swap took `Transport` out of all three TH1 rules
/// with every counter at the baseline.
fn audio_marked(source: &[char]) -> anyhow::Result<BTreeSet<String>> {
    let header =
        pattern::build(r"^(?:pub(?:\([a-z]+\))?\s+)?(?:struct|enum)\s+([A-Z][A-Za-z0-9]*)")?;
    let mut marked = BTreeSet::new();
    for (_offset, code) in rust_blocks(source)? {
        let mut doc: Vec<String> = Vec::new();
        for line in code.lines() {
            let stripped = line.trim();
            if stripped.starts_with("///") {
                doc.push(stripped.to_owned());
                continue;
            }
            let text = pattern::chars(stripped);
            if let Some(found) = header.match_at(&text, 0) {
                let carried = doc.iter().any(|one| one.contains(AUDIO_MARK));
                marked.extend(carried.then(|| found.text(1, &text)));
                doc.clear();
                continue;
            }
            if stripped.starts_with("#[") || stripped.starts_with(")]") || stripped.is_empty() {
                continue;
            }
            doc.clear();
        }
    }
    Ok(marked)
}

/// Split one text on a separator at bracket depth zero.
fn split_top(text: &str, separator: char) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut depth = 0_i64;
    let mut start = 0_usize;
    let mut out = Vec::new();
    for (index, found) in chars.iter().enumerate() {
        match *found {
            '(' | '{' | '[' | '<' => depth = depth.saturating_add(1),
            ')' | '}' | ']' | '>' => depth = depth.saturating_sub(1),
            other if other == separator && depth == 0 => {
                out.push(pattern::slice(&chars, start, index).unwrap_or_default());
                start = index.saturating_add(1);
            },
            _ => {},
        }
    }
    out.push(pattern::slice(&chars, start, chars.len()).unwrap_or_default());
    out.into_iter()
        .filter(|piece| !piece.trim().is_empty())
        .collect()
}

/// The index of the `:` that separates a field name from its type.
///
/// A path separator is not a field separator: the scanner skips a `::` pair and
/// takes the first single colon at bracket depth zero.
fn field_colon(text: &str) -> Option<usize> {
    let chars: Vec<char> = text.chars().collect();
    let mut depth = 0_i64;
    let mut index = 0_usize;
    while index < chars.len() {
        match chars.get(index) {
            Some(&'(' | &'{' | &'[' | &'<') => depth = depth.saturating_add(1),
            Some(&')' | &'}' | &']' | &'>') => depth = depth.saturating_sub(1),
            Some(&':') if depth == 0 => {
                if chars.get(index.saturating_add(1)) == Some(&':') {
                    index = index.saturating_add(2);
                    continue;
                }
                return Some(index);
            },
            Some(_) | None => {},
        }
        index = index.saturating_add(1);
    }
    None
}

/// Every field name and type expression one declaration body states.
///
/// The body comes from the raw block, so a path keeps its segments and PG19
/// resolves it by the last one. A `/* private */` body yields nothing.
fn field_exprs(kind: &str, payload: &str) -> anyhow::Result<Vec<(String, String)>> {
    let body = strip_block_comments(&strip_comments(payload)?)?;
    let mut out = Vec::new();
    if kind == "enum" {
        for arm in split_top(&body, ',') {
            let arm = arm.trim().to_owned();
            let chars: Vec<char> = arm.chars().collect();
            let inner =
                pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
            if (arm.starts_with('(') && arm.ends_with(')'))
                || (arm.starts_with('{') && arm.ends_with('}'))
            {
                named_fields(&inner, &mut out)?;
            }
        }
        return Ok(out);
    }
    named_fields(&body, &mut out)?;
    Ok(out)
}

/// Read one comma list of fields into the output list.
fn named_fields(text: &str, out: &mut Vec<(String, String)>) -> anyhow::Result<()> {
    let attribute = pattern::build(r"#\[[^\]]*\]")?;
    let visibility = pattern::build(r"^pub(?:\([a-z]+\))?\s+")?;
    for piece in split_top(text, ',') {
        let cleaned =
            attribute.replace_all(&pattern::chars(&piece), &mut |_one, _text| " ".to_owned());
        let cleaned = cleaned.trim().to_owned();
        let bare = pattern::chars(&cleaned);
        let cleaned = visibility.match_at(&bare, 0).map_or(cleaned, |found| {
            pattern::slice(&bare, found.end(), bare.len()).unwrap_or_default()
        });
        let cleaned = cleaned.trim().to_owned();
        if cleaned.is_empty() {
            continue;
        }
        match field_colon(&cleaned) {
            None => out.push((String::new(), cleaned)),
            Some(cut) => {
                let chars: Vec<char> = cleaned.chars().collect();
                let name = pattern::slice(&chars, 0, cut).unwrap_or_default();
                let value =
                    pattern::slice(&chars, cut.saturating_add(1), chars.len()).unwrap_or_default();
                out.push((name.trim().to_owned(), value.trim().to_owned()));
            },
        }
    }
    Ok(())
}

/// One parsed type expression.
#[derive(Debug, Clone)]
enum Node {
    /// A named type and its type arguments.
    Name(String, Vec<Option<Self>>),
    /// An array of one element type.
    Array(Box<Option<Self>>),
    /// A slice of one element type.
    Slice(Box<Option<Self>>),
    /// A tuple of items, or a `+` bound list.
    Tuple(Vec<Option<Self>>),
    /// An exclusive reference to one type.
    Exclusive(Box<Option<Self>>),
}

/// One type expression as a node, or `None` when the guard cannot read it.
///
/// A shared reference and an exclusive reference are two types. `&T` is `Copy`
/// and forwards every closure trait to its payload; `&mut T` is never `Copy`
/// and supplies none of the nine closure traits (critic K-6). A `dyn` or an
/// `impl` keyword drops and the trait path stays (critic W-4).
fn parse_type(text: &str) -> anyhow::Result<Option<Node>> {
    let text = text.trim().to_owned();
    let reference = pattern::build(r"^&\s*(?:'[a-z_][a-z0-9_]*\s*)?(mut\s+)?")?;
    let chars = pattern::chars(&text);
    let opened = reference
        .match_at(&chars, 0)
        .filter(|found| found.end() > 0);
    if let Some(found) = opened {
        let rest = pattern::slice(&chars, found.end(), chars.len()).unwrap_or_default();
        let inner = parse_type(&rest)?;
        if found.span(1).is_some() {
            return Ok(Some(Node::Exclusive(Box::new(inner))));
        }
        return Ok(inner);
    }
    if text.is_empty() {
        return Ok(None);
    }
    if pattern::build(r"^(?:dyn|impl)\b")?
        .match_at(&chars, 0)
        .is_some()
    {
        return parse_bounds(&text);
    }
    if text.starts_with('[') && text.ends_with(']') {
        let inner = pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
        let halves = split_top(&inner, ';');
        if halves.len() == 2 {
            let first = halves.first().cloned().unwrap_or_default();
            return Ok(Some(Node::Array(Box::new(parse_type(&first)?))));
        }
        return Ok(Some(Node::Slice(Box::new(parse_type(&inner)?))));
    }
    if text.starts_with('(') && text.ends_with(')') {
        let inner = pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
        let items = split_top(&inner, ',');
        if items.is_empty() {
            return Ok(Some(Node::Tuple(Vec::new())));
        }
        if items.len() == 1 {
            let first = items.first().cloned().unwrap_or_default();
            return parse_type(&first);
        }
        let mut parts = Vec::new();
        for item in items {
            parts.push(parse_type(&item)?);
        }
        return Ok(Some(Node::Tuple(parts)));
    }
    parse_named(&text)
}

/// One `dyn` or `impl` bound list as a node.
fn parse_bounds(text: &str) -> anyhow::Result<Option<Node>> {
    let keyword = pattern::build(r"^(?:dyn|impl)\b\s*")?;
    let chars = pattern::chars(text);
    let rest = keyword.match_at(&chars, 0).map_or_else(
        || text.to_owned(),
        |found| pattern::slice(&chars, found.end(), chars.len()).unwrap_or_default(),
    );
    let bounds: Vec<String> = split_top(&rest, '+')
        .into_iter()
        .map(|piece| piece.trim().to_owned())
        .filter(|piece| !piece.is_empty() && !piece.starts_with('\''))
        .collect();
    if bounds.is_empty() {
        return Ok(None);
    }
    if bounds.len() == 1 {
        let first = bounds.first().cloned().unwrap_or_default();
        return parse_type(&first);
    }
    let mut parts = Vec::new();
    for bound in bounds {
        parts.push(parse_type(&bound)?);
    }
    Ok(Some(Node::Tuple(parts)))
}

/// One named type expression as a node, or `None`.
fn parse_named(text: &str) -> anyhow::Result<Option<Node>> {
    let named = pattern::build(
        r"(?s)^(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*(<.*>)?$",
    )?;
    let chars = pattern::chars(text);
    let Some(found) = named.match_at(&chars, 0) else {
        return Ok(None);
    };
    let name = found.text(1, &chars);
    let mut args = Vec::new();
    if let Some((start, end)) = found.span(2) {
        let raw = pattern::slice(&chars, start.saturating_add(1), end.saturating_sub(1))
            .unwrap_or_default();
        for piece in split_top(&raw, ',') {
            let bare = piece.trim().to_owned();
            if bare.is_empty() || bare.starts_with('\'') {
                continue;
            }
            let first = bare.chars().next();
            if is_upper(&bare)
                || first.is_some_and(|value| value.is_ascii_digit())
                || bare.starts_with('{')
            {
                continue;
            }
            args.push(parse_type(&piece)?);
        }
    }
    Ok(Some(Node::Name(name, args)))
}

/// Every bare name a parsed type expression mentions.
fn leaf_names(node: Option<&Node>) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let Some(node) = node else {
        return names;
    };
    match *node {
        Node::Name(ref name, ref args) => {
            names.insert(name.clone());
            for arg in args {
                names.extend(leaf_names(arg.as_ref()));
            }
        },
        Node::Array(ref inner) | Node::Slice(ref inner) | Node::Exclusive(ref inner) => {
            names.extend(leaf_names(inner.as_ref().as_ref()));
        },
        Node::Tuple(ref items) => {
            for item in items {
                names.extend(leaf_names(item.as_ref()));
            }
        },
    }
    names
}

/// Whether one parsed type expression reaches an exclusive reference.
fn holds_exclusive(node: Option<&Node>) -> bool {
    let Some(node) = node else {
        return false;
    };
    match *node {
        Node::Exclusive(_) => true,
        Node::Name(_, ref args) => args.iter().any(|arg| holds_exclusive(arg.as_ref())),
        Node::Array(ref inner) | Node::Slice(ref inner) => holds_exclusive(inner.as_ref().as_ref()),
        Node::Tuple(ref items) => items.iter().any(|item| holds_exclusive(item.as_ref())),
    }
}

/// The trait names one declaration's `#[derive(...)]` attributes carry.
fn derives_of(attr: &str) -> anyhow::Result<BTreeSet<String>> {
    let derive = pattern::build(r"(?s)#\[derive\(([^)]*)\)\]")?;
    let word = pattern::build(r"[A-Za-z_][A-Za-z0-9_]*")?;
    let text = pattern::chars(attr);
    let mut names = BTreeSet::new();
    for one in derive.find_iter(&text) {
        let inner = pattern::chars(&one.text(1, &text));
        for found in word.find_iter(&inner) {
            names.insert(found.text(0, &inner));
        }
    }
    Ok(names)
}

/// A `name -> (traits, unconditional)` lookup for PG19 and PG23.
///
/// The primitive map comes from the section 1.9 block, never from a constant, so
/// an empty map decides no primitive and every field above one goes undecided
/// (critic K-4).
#[derive(Debug, Clone, Copy)]
struct Resolver<'a> {
    /// Every declaration this document writes.
    decls: &'a DeclMap,
    /// Every hand-written trait impl this document writes.
    impls: &'a BTreeMap<String, BTreeSet<String>>,
    /// Every external name and the traits its row supplies.
    externals: &'a BTreeMap<String, TraitPair>,
    /// Every primitive and the traits it supplies.
    primitives: &'a BTreeMap<String, BTreeSet<String>>,
}

impl Resolver<'_> {
    /// The traits one name supplies, and the ones it supplies unconditionally.
    fn resolve(&self, name: &str) -> anyhow::Result<Option<TraitPair>> {
        if let Some(traits) = self.primitives.get(name) {
            return Ok(Some((traits.clone(), traits.clone())));
        }
        if let Some(entry) = self.decls.first(name) {
            if entry.kind == "trait" {
                return Ok(None);
            }
            let mut supplied: BTreeSet<String> = derives_of(&entry.attrs)?
                .into_iter()
                .filter(|one| CLOSURE_TRAITS.contains(&one.as_str()))
                .collect();
            if let Some(hand) = self.impls.get(name) {
                supplied.extend(hand.iter().cloned());
            }
            return Ok(Some((supplied.clone(), supplied)));
        }
        Ok(self.externals.get(name).cloned())
    }

    /// The closure traits one type expression supplies, or `None` when unknown.
    fn node_traits(&self, node: Option<&Node>) -> anyhow::Result<Option<BTreeSet<String>>> {
        let Some(node) = node else {
            return Ok(None);
        };
        match *node {
            Node::Exclusive(_) => Ok(Some(BTreeSet::new())),
            Node::Tuple(ref items) => self.tuple_traits(items),
            Node::Array(ref inner) => Ok(self.node_traits(inner.as_ref().as_ref())?),
            Node::Slice(ref inner) => {
                let Some(mut found) = self.node_traits(inner.as_ref().as_ref())? else {
                    return Ok(None);
                };
                found.remove("Default");
                Ok(Some(found))
            },
            Node::Name(ref name, ref args) => self.applied_traits(name, args),
        }
    }

    /// The traits every item of a tuple supplies, or `None` when one is unknown.
    fn tuple_traits(&self, items: &[Option<Node>]) -> anyhow::Result<Option<BTreeSet<String>>> {
        let mut supplied: BTreeSet<String> =
            CLOSURE_TRAITS.iter().map(|one| (*one).to_owned()).collect();
        for item in items {
            let Some(inner) = self.node_traits(item.as_ref())? else {
                return Ok(None);
            };
            supplied = supplied.intersection(&inner).cloned().collect();
        }
        Ok(Some(supplied))
    }

    /// Whether every type argument supplies one trait, or `None` when unknown.
    fn args_supply(&self, args: &[Option<Node>], one: &str) -> anyhow::Result<Option<bool>> {
        for arg in args {
            let Some(inner) = self.node_traits(arg.as_ref())? else {
                return Ok(None);
            };
            if !inner.contains(one) {
                return Ok(Some(false));
            }
        }
        Ok(Some(true))
    }

    /// The traits one applied name supplies, given its arguments.
    fn applied_traits(
        &self,
        name: &str,
        args: &[Option<Node>],
    ) -> anyhow::Result<Option<BTreeSet<String>>> {
        let Some((supplied, unconditional)) = self.resolve(name)? else {
            return Ok(None);
        };
        if args.is_empty() {
            return Ok(Some(supplied));
        }
        let mut result = BTreeSet::new();
        for one in &supplied {
            if unconditional.contains(one) {
                result.insert(one.clone());
                continue;
            }
            let Some(holds) = self.args_supply(args, one)? else {
                return Ok(None);
            };
            result.extend(holds.then(|| one.clone()));
        }
        Ok(Some(result))
    }
}

/// Every enum arm this document declares, as the arm name and its body (PG24).
fn enum_arms_of(source: &[char]) -> anyhow::Result<ArmMap> {
    let header = pattern::build(r"\benum\s+([A-Z][A-Za-z0-9]*)\s*\{")?;
    let lead = pattern::build(r"(?s)\s*([A-Z][A-Za-z0-9]*)\s*([\s\S]*)")?;
    let mut found: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    for (_offset, block) in rust_blocks(source)? {
        let code = pattern::chars(&strip_comments(&block)?);
        for one in header.find_iter(&code) {
            let Some(open) = code.iter().skip(one.start()).position(|c| *c == '{') else {
                continue;
            };
            let open_index = one.start().saturating_add(open);
            let body = pattern::slice(
                &code,
                open_index.saturating_add(1),
                matching_brace(&code, open_index),
            )
            .unwrap_or_default();
            let arms = enum_arm_list(&lead, &body);
            found.entry(one.text(1, &code)).or_insert(arms);
        }
    }
    Ok(found)
}

/// Every arm of one enum body, as its name and the text after the name.
fn enum_arm_list(lead: &pattern::Regex, body: &str) -> Vec<(String, String)> {
    let mut arms = Vec::new();
    for arm in top_level_arms(body) {
        let text = pattern::chars(&arm);
        let found = lead
            .match_at(&text, 0)
            .map(|head| (head.text(1, &text), head.text(2, &text).trim().to_owned()));
        arms.extend(found);
    }
    arms
}

/// The label one field carries: its name, or the expression when it has none.
fn field_label(field: &str, expr: &str) -> String {
    if field.is_empty() {
        expr.trim().to_owned()
    } else {
        field.to_owned()
    }
}

/// Every label and type expression one declaration states, arm names kept.
///
/// `declarations` masks an arm name, because an arm name is not a type (PG8).
/// PG26 needs the name for the field chain it prints, so it reads the unmasked
/// arm list the way PG24 does.
fn walk_fields(
    name: &str,
    decls: &DeclMap,
    arms: &BTreeMap<String, Vec<(String, String)>>,
) -> anyhow::Result<Vec<(String, String)>> {
    let Some(entry) = decls.first(name) else {
        return Ok(Vec::new());
    };
    if entry.kind == "trait" {
        return Ok(Vec::new());
    }
    if entry.kind != "enum" {
        return Ok(field_exprs(&entry.kind, &entry.payload)?
            .into_iter()
            .map(|(field, expr)| {
                let label = if field.is_empty() {
                    expr.trim().to_owned()
                } else {
                    field
                };
                (label, expr)
            })
            .collect());
    }
    let mut found = Vec::new();
    for (arm_name, arm_body) in arms.get(name).into_iter().flatten() {
        for (field, expr) in field_exprs("enum", arm_body)? {
            let label = if field.is_empty() {
                arm_name.clone()
            } else {
                format!("{arm_name}.{field}")
            };
            found.push((label, expr));
        }
    }
    Ok(found)
}

/// Whether a declaration body states its fields.
fn spelled_out(payload: &str) -> anyhow::Result<bool> {
    Ok(!strip_block_comments(payload)?.trim().is_empty())
}

/// Whether a block comment stands in for a declaration body.
fn private_body(payload: &str) -> bool {
    payload.contains("/*")
}

/// Every body piece that a comment stands in for (PG3).
fn placeholder_pieces(payload: &str) -> anyhow::Result<Vec<String>> {
    let mut found = Vec::new();
    for piece in split_top(payload, ',') {
        if piece.contains("/*") && strip_block_comments(&piece)?.trim().is_empty() {
            found.push(piece.trim().to_owned());
        }
    }
    Ok(found)
}

/// Whether one attribute list carries a `Copy` derive.
fn derives_copy(attr: &str) -> anyhow::Result<bool> {
    holds_pattern(attr, r"#\[derive\([^)]*\bCopy\b")
}

/// Copy-ness of every name the guard can decide.
///
/// There is no suffix template. Every verdict comes from one of two places: a
/// Rust block in this document, or the external-verdict table of section 1.9
/// (critic R1).
fn copy_verdicts(
    decls: &DeclMap,
    externals: &BTreeMap<String, String>,
    impls: &BTreeMap<String, BTreeSet<String>>,
) -> anyhow::Result<BTreeMap<String, bool>> {
    let mut is_copy = BTreeMap::new();
    for (name, verdict) in externals {
        if verdict == "copy" {
            is_copy.insert(name.clone(), true);
        } else if verdict == "not copy" {
            is_copy.insert(name.clone(), false);
        }
    }
    for (name, entries) in decls.iter() {
        let Some(entry) = entries.first() else {
            continue;
        };
        if entry.kind == "trait" {
            continue;
        }
        let hand = impls
            .get(name)
            .is_some_and(|traits| traits.contains("Copy"));
        is_copy.insert(name.clone(), derives_copy(&entry.attrs)? || hand);
    }
    Ok(is_copy)
}

/// The field type names of one declaration.
///
/// There is no drop list. Only a wrapper the external table marks `transparent`
/// is skipped, because its Copy-ness is its payload's and the payload is a
/// separate token in the same field (critic R2).
fn field_names(
    payload: &str,
    externals: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
) -> anyhow::Result<BTreeSet<String>> {
    Ok(body_type_names(payload, known)?
        .into_iter()
        .filter(|token| externals.get(token).map(String::as_str) != Some("transparent"))
        .collect())
}

/// What one `Copy` audit found, in the order the run prints it.
#[derive(Debug, Default)]
struct CopyReport {
    /// Every all-`Copy` type that derives no `Copy` and carries no expectation.
    missing: Vec<(String, String)>,
    /// Every `Copy` derive over a field this document proves is not `Copy`.
    impossible: Vec<(String, String, String)>,
    /// Every `Copy` derive over a field whose Copy-ness this document leaves open.
    undecided: Vec<(String, String, String)>,
    /// Every undecidable field of any declaration.
    unknown: Vec<(String, String, String)>,
}

impl CopyReport {
    /// Record every field a `Copy` derive cannot carry, and every undecided one.
    fn record_derived(
        &mut self,
        name: &str,
        crate_name: &str,
        verdicts: &[(String, Option<bool>)],
    ) {
        for (field, verdict) in verdicts {
            let row = (name.to_owned(), crate_name.to_owned(), field.clone());
            match *verdict {
                Some(false) => self.impossible.push(row),
                None => self.undecided.push(row),
                Some(true) => {},
            }
        }
    }
}

/// PG9, PG10, `PG10b`, and PG14 over the document's own declarations.
///
/// PG14 counts an undecidable field of ANY declaration, whether or not the
/// declaration derives `Copy` (critic Q2). An exclusive reference decides the
/// declaration on its own: `&mut T` is never `Copy`.
fn copy_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    is_copy: &BTreeMap<String, bool>,
    externals: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
    drops: &BTreeSet<String>,
) -> anyhow::Result<CopyReport> {
    let mut report = CopyReport::default();
    for (name, entries) in decls.iter() {
        let Some(entry) = entries.first() else {
            continue;
        };
        if entry.kind == "trait" || entry.payload.trim().is_empty() {
            continue;
        }
        let carries = derives_copy(&entry.attrs)?;
        let crate_name = owned
            .get(name)
            .cloned()
            .unwrap_or_else(|| "unplaced".to_owned());
        let exclusive = exclusive_fields(&entry.kind, &entry.payload)?;
        if !exclusive.is_empty() && carries {
            for field in &exclusive {
                report
                    .impossible
                    .push((name.clone(), crate_name.clone(), field.clone()));
            }
        }
        let fields = field_names(&entry.payload, externals, known)?;
        if fields.is_empty() {
            continue;
        }
        let verdicts: Vec<(String, Option<bool>)> = fields
            .iter()
            .map(|field| (field.clone(), is_copy.get(field).copied()))
            .collect();
        for (field, verdict) in &verdicts {
            if verdict.is_none() {
                report
                    .unknown
                    .push((name.clone(), crate_name.clone(), field.clone()));
            }
        }
        if carries {
            report.record_derived(name, &crate_name, &verdicts);
            continue;
        }
        if entry.attrs.contains("missing_copy_implementations")
            || drops.contains(name)
            || !exclusive.is_empty()
        {
            continue;
        }
        if verdicts
            .iter()
            .all(|(_field, verdict)| *verdict == Some(true))
        {
            report.missing.push((name.clone(), crate_name));
        }
    }
    report.unknown.sort();
    Ok(report)
}

/// Every field of one declaration body that holds an exclusive reference.
///
/// `&mut T` is never `Copy`, so a field that holds one puts the declaration
/// outside PG9 and makes a `Copy` derive over it a PG10 failure.
fn exclusive_fields(kind: &str, payload: &str) -> anyhow::Result<Vec<String>> {
    let mut found = Vec::new();
    for (field, expr) in field_exprs(kind, payload)? {
        if holds_exclusive(parse_type(&expr)?.as_ref()) {
            found.push(field_label(&field, &expr));
        }
    }
    Ok(found)
}

/// PG3. Every declaration whose body states a comment where a field goes.
///
/// The scan reads each Rust block with its LINE comments replaced by an empty
/// block comment, so a `///` line cannot declare a phantom type and a body that
/// held only a line comment is still visible as a placeholder (critic WR-1).
fn comment_placeholders(
    source: &[char],
    owned: &BTreeMap<String, String>,
) -> anyhow::Result<Vec<(String, String)>> {
    let comment = pattern::build(r"//[^\n]*")?;
    let mut failures = BTreeSet::new();
    for (_offset, block) in rust_blocks(source)? {
        let marked = comment.replace_all(&pattern::chars(&block), &mut |_one, _text| {
            "/**/".to_owned()
        });
        for (name, entries) in declarations(&marked)?.iter() {
            if !holds_placeholder(entries)? {
                continue;
            }
            failures.insert((
                name.clone(),
                owned
                    .get(name)
                    .cloned()
                    .unwrap_or_else(|| "unplaced".to_owned()),
            ));
        }
    }
    Ok(failures.into_iter().collect())
}

/// Whether any declaration of one name states a comment where a field goes.
fn holds_placeholder(entries: &[Decl]) -> anyhow::Result<bool> {
    for entry in entries {
        if !placeholder_pieces(&entry.payload)?.is_empty() {
            return Ok(true);
        }
    }
    Ok(false)
}

/// `PG4b`. A section 1.5 name that no Rust block declares.
fn placed_without_declaration(owned: &BTreeMap<String, String>, decls: &DeclMap) -> Vec<String> {
    owned
        .keys()
        .filter(|name| !decls.holds(name))
        .cloned()
        .collect()
}

/// PG18. An external token a declaration names and the 1.9 table omits.
fn external_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    framework: &BTreeSet<String>,
    externals: &BTreeMap<String, String>,
    known: &BTreeSet<String>,
) -> anyhow::Result<Vec<(String, String)>> {
    let mut missing = BTreeSet::new();
    for (name, entries) in decls.iter() {
        for entry in entries.iter().filter(|entry| entry.kind != "trait") {
            let decided = |token: &String| {
                owned.contains_key(token)
                    || framework.contains(token)
                    || externals.contains_key(token)
            };
            let tokens = body_type_names(&entry.payload, known)?;
            missing.extend(
                tokens
                    .into_iter()
                    .filter(|token| !decided(token))
                    .map(|token| (name.clone(), token)),
            );
        }
    }
    Ok(missing.into_iter().collect())
}

/// PG7. Framework names inside a non-application declaration.
fn framework_misuse(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    framework: &BTreeSet<String>,
) -> anyhow::Result<Vec<(String, String, String)>> {
    let mut failures = Vec::new();
    for (name, entries) in decls.iter() {
        let Some(crate_name) = owned.get(name) else {
            continue;
        };
        if APP_ROWS.contains(&crate_name.as_str()) {
            continue;
        }
        for entry in entries {
            let tokens = last_segments(&entry.payload)?;
            failures.extend(
                tokens
                    .into_iter()
                    .filter(|token| framework.contains(token))
                    .map(|token| (name.clone(), crate_name.clone(), token)),
            );
        }
    }
    Ok(failures)
}

/// PG12. A 1.2 row that omits a crate its own declarations prove it uses.
///
/// It also returns every `(crate, dependency)` pair a declaration PROVES, so the
/// run prints that number and section 1.2 cites it rather than a hand count
/// (critic WR-14).
fn dependency_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    rows: &BTreeMap<String, BTreeSet<String>>,
    mapping: &BTreeMap<String, String>,
) -> anyhow::Result<(Triples, Proven)> {
    let word = pattern::build(r"[A-Za-z_][A-Za-z0-9_]*")?;
    let mut failures = BTreeSet::new();
    let mut proven = BTreeSet::new();
    for (name, entries) in decls.iter() {
        let Some(crate_name) = owned.get(name).map(|found| normalize_crate(found)) else {
            continue;
        };
        let Some(listed) = rows.get(&crate_name) else {
            continue;
        };
        for entry in entries {
            let text = format!(
                "{} {}",
                strip_comments(&entry.attrs)?,
                strip_comments(&entry.payload)?
            );
            let chars = pattern::chars(&text);
            let found = word.find_iter(&chars);
            let used = found
                .iter()
                .filter_map(|one| mapping.get(&one.text(0, &chars)));
            let (carried, missed): (Vec<&String>, Vec<&String>) =
                used.partition(|needed| listed.contains(*needed));
            proven.extend(
                carried
                    .into_iter()
                    .map(|needed| (crate_name.clone(), needed.clone())),
            );
            failures.extend(
                missed
                    .into_iter()
                    .map(|needed| (crate_name.clone(), needed.clone(), name.clone())),
            );
        }
    }
    Ok((failures.into_iter().collect(), proven))
}

/// PG13. Every type the audio thread publishes is declared and is `Copy`.
fn snapshot_audit(
    blocks: &Blocks,
    decls: &DeclMap,
    is_copy: &BTreeMap<String, bool>,
) -> anyhow::Result<(Pairs, Vec<String>)> {
    let published = pattern::build(r"triple_buffer::Output<([A-Za-z0-9]+)>")?;
    let mut failures = Vec::new();
    let mut seen = Vec::new();
    for row in blocks.rows("snapshot-table") {
        let cells = row.cells();
        if cells.len() < 3 || row.cell(0) != "Audio" {
            continue;
        }
        let cell = pattern::chars(&row.cell(2));
        let Some(found) = published.find(&cell) else {
            failures.push((row.cell(2), "no published type name".to_owned()));
            continue;
        };
        let name = found.text(1, &cell);
        seen.push(name.clone());
        if !decls.holds(&name) {
            failures.push((name, "no Rust block declares it".to_owned()));
        } else if is_copy.get(&name) != Some(&true) {
            failures.push((name, "the declaration derives no Copy".to_owned()));
        }
    }
    Ok((failures, seen))
}

/// PG15. A `Declared in` cell that omits a section which declares a type.
fn register_audit(
    block_sections: &BTreeMap<String, Option<String>>,
    owned: &BTreeMap<String, String>,
    declared_in: &BTreeMap<String, BTreeSet<String>>,
) -> Vec<(String, String, String)> {
    let mut failures = BTreeSet::new();
    for (name, section) in block_sections {
        let (Some(crate_name), Some(section)) = (owned.get(name), section.as_ref()) else {
            continue;
        };
        let Some(listed) = declared_in.get(crate_name) else {
            continue;
        };
        if !listed.contains(section) {
            failures.insert((crate_name.clone(), name.clone(), section.clone()));
        }
    }
    failures.into_iter().collect()
}

/// The section 1.9 justified-unknown table, as a set of type names.
fn justified_unknowns(blocks: &Blocks) -> anyhow::Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for row in blocks.rows("justified-unknowns") {
        if row.cells().len() >= 4 {
            names.extend(cell_names(&row.cell(2))?);
        }
    }
    Ok(names)
}

/// PG17. An undecidable field type the section 1.9 table does not name.
fn unknown_audit(
    unknown: &[(String, String, String)],
    justified: &BTreeSet<String>,
) -> Vec<(String, String)> {
    let found: BTreeSet<(String, String)> = unknown
        .iter()
        .filter(|(_name, _crate_name, field)| !justified.contains(field))
        .map(|(name, _crate_name, field)| (name.clone(), field.clone()))
        .collect();
    found.into_iter().collect()
}

/// The closure traits one attribute list derives.
fn closure_derives(attr: &str) -> anyhow::Result<BTreeSet<String>> {
    Ok(derives_of(attr)?
        .into_iter()
        .filter(|one| CLOSURE_TRAITS.contains(&one.as_str()))
        .collect())
}

/// PG19. A derive that reaches a field type without the same trait.
///
/// The rule reads a derive, because only a derive demands the trait of every
/// field. A hand-written impl supplies the trait and demands nothing, which is
/// the whole reason `Finite` carries five of them (section 2.6a).
fn derive_closure_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    framework: &BTreeSet<String>,
    resolver: Resolver<'_>,
) -> anyhow::Result<(Vec<Quint>, Vec<Quad>)> {
    let mut broken = Vec::new();
    let mut undecided = Vec::new();
    for name in decls.sorted_keys() {
        let Some(entry) = decls.first(&name) else {
            continue;
        };
        if entry.kind == "trait" {
            continue;
        }
        let derived = closure_derives(&entry.attrs)?;
        if derived.is_empty() {
            continue;
        }
        let crate_name = owned
            .get(&name)
            .cloned()
            .unwrap_or_else(|| "unplaced".to_owned());
        for (field, expr) in field_exprs(&entry.kind, &entry.payload)? {
            let node = parse_type(&expr)?;
            let leaves = leaf_names(node.as_ref());
            if APP_ROWS.contains(&crate_name.as_str())
                && leaves.iter().any(|leaf| framework.contains(leaf))
            {
                continue;
            }
            let label = if field.is_empty() {
                expr.trim().to_owned()
            } else {
                field.clone()
            };
            let Some(supplied) = resolver.node_traits(node.as_ref())? else {
                undecided.push((
                    name.clone(),
                    crate_name.clone(),
                    label,
                    expr.trim().to_owned(),
                ));
                continue;
            };
            for one in derived.difference(&supplied) {
                broken.push((
                    name.clone(),
                    crate_name.clone(),
                    one.clone(),
                    label.clone(),
                    expr.trim().to_owned(),
                ));
            }
        }
    }
    Ok((broken, undecided))
}

/// PG20. A field type in a crate the 1.3 graph does not reach.
///
/// The rule reads the graph, not the member manifest, so a type reached through
/// a transitive edge passes. The parser still declines four forms, and the
/// compiler is the backstop for all four (critic W-4).
fn reachability_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    closure: &BTreeMap<String, BTreeSet<String>>,
) -> anyhow::Result<Vec<Quad>> {
    let mut failures = Vec::new();
    for name in decls.sorted_keys() {
        let Some(entry) = decls.first(&name) else {
            continue;
        };
        if entry.kind == "trait" {
            continue;
        }
        let crate_name = owned
            .get(&name)
            .map_or_else(String::new, |found| normalize_crate(found));
        if crate_name.is_empty() {
            continue;
        }
        let mut seen = BTreeSet::new();
        for (_field, expr) in field_exprs(&entry.kind, &entry.payload)? {
            seen.extend(leaf_names(parse_type(&expr)?.as_ref()));
        }
        for token in seen {
            let target = owned
                .get(&token)
                .map_or_else(String::new, |found| normalize_crate(found));
            if target.is_empty() || target == crate_name {
                continue;
            }
            let reached = closure
                .get(&crate_name)
                .is_some_and(|targets| targets.contains(&target));
            if !reached {
                failures.push((crate_name.clone(), name.clone(), token, target));
            }
        }
    }
    Ok(failures)
}

/// PG23. A `PartialEq` derive with no `Eq` over a body that supplies `Eq`.
///
/// `clippy::derive_partial_eq_without_eq` is a `nursery` lint and the workspace
/// sets `nursery` to `deny`, so the shape does not build. The rule reads the
/// field expressions PG19 reads and it skips the classes PG19 skips.
fn eq_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    impls: &BTreeMap<String, BTreeSet<String>>,
    framework: &BTreeSet<String>,
    resolver: Resolver<'_>,
) -> anyhow::Result<(Pairs, Pairs)> {
    let mut missing = Vec::new();
    let mut undecided = Vec::new();
    for name in decls.sorted_keys() {
        let Some(entry) = decls.first(&name) else {
            continue;
        };
        if entry.kind == "trait" || !spelled_out(&entry.payload)? {
            continue;
        }
        let derived = derives_of(&entry.attrs)?;
        if !derived.contains("PartialEq") || derived.contains("Eq") {
            continue;
        }
        if impls.get(&name).is_some_and(|traits| traits.contains("Eq")) {
            continue;
        }
        let crate_name = owned
            .get(&name)
            .cloned()
            .unwrap_or_else(|| "unplaced".to_owned());
        let mut nodes = Vec::new();
        for (_field, expr) in field_exprs(&entry.kind, &entry.payload)? {
            nodes.push(parse_type(&expr)?);
        }
        if APP_ROWS.contains(&crate_name.as_str())
            && nodes.iter().any(|node| {
                leaf_names(node.as_ref())
                    .iter()
                    .any(|leaf| framework.contains(leaf))
            })
        {
            continue;
        }
        let mut supplies = Vec::new();
        for node in &nodes {
            supplies.push(resolver.node_traits(node.as_ref())?);
        }
        if supplies.iter().any(Option::is_none) {
            undecided.push((name.clone(), crate_name));
        } else if supplies
            .iter()
            .all(|item| item.as_ref().is_some_and(|traits| traits.contains("Eq")))
        {
            missing.push((name.clone(), crate_name));
        }
    }
    Ok((missing, undecided))
}

/// The section 3.5 VR1 derive-use table, as a set of type names (PG22).
fn vr1_table(blocks: &Blocks) -> anyhow::Result<BTreeSet<String>> {
    let mut names = BTreeSet::new();
    for row in blocks.rows("vr1-table") {
        if row.cells().len() >= 2 {
            names.extend(cell_names(&row.cell(0))?);
        }
    }
    Ok(names)
}

/// PG22. A `Hash` or `Ord` derive the VR1 table names no use for.
///
/// One direction only, and this is the site that says so. A VR1 row that no
/// declaration uses is not a failure. The printed count lets a reader see a
/// stale row.
fn vr1_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    impls: &BTreeMap<String, BTreeSet<String>>,
    listed: &BTreeSet<String>,
) -> anyhow::Result<Vr1Rows> {
    let mut failures = Vec::new();
    for name in decls.sorted_keys() {
        let Some(entry) = decls.first(&name) else {
            continue;
        };
        if entry.kind == "trait" {
            continue;
        }
        let mut carried: BTreeSet<String> = derives_of(&entry.attrs)?
            .into_iter()
            .filter(|one| VR1_TRAITS.contains(&one.as_str()))
            .collect();
        if let Some(hand) = impls.get(&name) {
            carried.extend(
                hand.iter()
                    .filter(|one| VR1_TRAITS.contains(&one.as_str()))
                    .cloned(),
            );
        }
        if !carried.is_empty() && !listed.contains(&name) {
            failures.push((
                name.clone(),
                owned
                    .get(&name)
                    .cloned()
                    .unwrap_or_else(|| "unplaced".to_owned()),
                carried.into_iter().collect(),
            ));
        }
    }
    Ok(failures)
}

/// The Appendix B.1 expectation tables, as the sites each lint lists (PG21).
fn expectation_tables(blocks: &Blocks) -> anyhow::Result<BTreeMap<String, BTreeSet<String>>> {
    let span = pattern::build(r"`([A-Za-z0-9_:-]+)`")?;
    let mut tables: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for (lint, block_id) in EXPECTED_LINTS.iter().zip(["b1-copy", "b1-variant"]) {
        let names = tables.entry((*lint).to_owned()).or_default();
        for row in blocks.rows(block_id) {
            let cell = pattern::chars(&row.cell(0));
            for one in span.find_iter(&cell) {
                let site = one.text(1, &cell);
                let leaf = site.rsplit("::").next().unwrap_or(&site).to_owned();
                names.insert(leaf);
            }
        }
    }
    Ok(tables)
}

/// PG21. The expectation sites and the B.1 tables are one set per lint.
///
/// Both directions and both lints, because B.1 is the list the Orchestrator
/// adjudicates: a row nothing carries is a decision nobody needs, and a site no
/// row names is a suppression nobody approved (critic W7, concern 3).
fn expectation_audit(
    decls: &DeclMap,
    tables: &BTreeMap<String, BTreeSet<String>>,
) -> (Pairs, BTreeMap<String, BTreeSet<String>>) {
    let mut failures = Vec::new();
    let mut carried = BTreeMap::new();
    for lint in EXPECTED_LINTS {
        let sites: BTreeSet<String> = decls
            .iter()
            .filter(|(_name, entries)| {
                entries
                    .first()
                    .is_some_and(|entry| entry.attrs.contains(lint))
            })
            .map(|(name, _entries)| name.clone())
            .collect();
        let empty = BTreeSet::new();
        let listed = tables.get(lint).unwrap_or(&empty);
        for name in sites.difference(listed) {
            failures.push((
                name.clone(),
                format!("carries a {lint} expectation and Appendix B.1 omits it"),
            ));
        }
        for name in listed.difference(&sites) {
            failures.push((
                name.clone(),
                format!("has an Appendix B.1 {lint} row and no declaration carries it"),
            ));
        }
        carried.insert(lint.to_owned(), sites);
    }
    (failures, carried)
}

/// One type expression with every space removed, which is the row key.
fn normalize_expr(text: &str) -> String {
    text.chars()
        .filter(|found| !found.is_whitespace())
        .collect()
}

/// The index of the bracket that closes the one at `open_index`.
fn matching_bracket(source: &[char], open_index: usize) -> usize {
    let mut depth = 0_i64;
    let mut cursor = open_index;
    while cursor < source.len() {
        let found = source.get(cursor).copied().unwrap_or(' ');
        depth = match found {
            '[' => depth.saturating_add(1),
            ']' => depth.saturating_sub(1),
            _ => depth,
        };
        if found == ']' && depth == 0 {
            return cursor;
        }
        cursor = cursor.saturating_add(1);
    }
    source.len()
}

/// Every `#[expect]` attribute of the document, as its site and its text (PG21).
///
/// The site is the item the attribute sits above, and it falls back to the
/// section number when the guard cannot read one.
fn attribute_texts(
    source: &[char],
    marks: &[SectionMark],
) -> anyhow::Result<Vec<(String, String)>> {
    let head = pattern::build(r"#\s*!?\s*\[\s*expect\b")?;
    let item =
        pattern::build(r"\b(?:struct|enum|trait|fn|const|type|mod)\s+([A-Za-z_][A-Za-z0-9_]*)")?;
    let mut found = Vec::new();
    for one in head.find_iter(source) {
        let Some(open) = source.iter().skip(one.start()).position(|c| *c == '[') else {
            continue;
        };
        let cursor = matching_bracket(source, one.start().saturating_add(open));
        if cursor >= source.len() {
            continue;
        }
        let after = cursor.saturating_add(1);
        let tail_end = after.saturating_add(399).min(source.len());
        let tail = source.get(after..tail_end).unwrap_or(&[]);
        let site = item.find(tail).map_or_else(
            || {
                format!(
                    "section {}",
                    section_at(marks, one.start()).unwrap_or_else(|| "None".to_owned())
                )
            },
            |named| named.text(1, tail),
        );
        found.push((
            site,
            pattern::slice(source, one.start(), after).unwrap_or_default(),
        ));
    }
    Ok(found)
}

/// Every Appendix B.1 site and reason pair (PG21).
fn b1_reason_cells(blocks: &Blocks) -> Vec<(String, String)> {
    let mut found = Vec::new();
    for block_id in ["b1-convert", "b1-complexity", "b1-copy", "b1-variant"] {
        for row in blocks.rows(block_id) {
            if row.cells().len() >= 2 {
                found.push((strip_ticks(&row.cell(0)), row.cell(1)));
            }
        }
    }
    found
}

/// Every `///` and `//!` line of every Rust block, as its site and its text.
///
/// PG21 reads a size inside one exactly as it reads a size inside an `#[expect]`
/// reason. A declaration doc comment is code that a chunk copies into the crate
/// (critic WR-15).
fn doc_comment_lines(source: &[char]) -> anyhow::Result<Vec<(String, String)>> {
    let mut found = Vec::new();
    for (offset, block) in rust_blocks(source)? {
        let line_number = source
            .iter()
            .take(offset)
            .filter(|value| **value == '\n')
            .count()
            .saturating_add(1);
        for (index, line) in block.lines().enumerate() {
            let text = line.trim();
            if text.starts_with("///") || text.starts_with("//!") {
                found.push((
                    format!(
                        "a doc comment at line {}",
                        line_number.saturating_add(index).saturating_add(1)
                    ),
                    text.to_owned(),
                ));
            }
        }
    }
    Ok(found)
}

/// Whether one code span names a row of the section 1.9 size block.
fn cites_recorded_row(span: &str, recorded: &BTreeMap<String, (usize, usize)>) -> bool {
    span.split_whitespace()
        .any(|token| recorded.contains_key(&normalize_expr(token)))
}

/// PG21. A size inside a reason is a citation and never a literal.
///
/// A reason writes a `B` id or names the row the PG24 table prints, so every
/// stated size rests on a value a guard measured (critic WR-2).
fn reason_size_audit(
    sites: &[(String, String)],
    recorded: &BTreeMap<String, (usize, usize)>,
) -> anyhow::Result<Counted> {
    let count = pattern::build(r"([0-9][0-9_]*) ?(bytes?)\b")?;
    let span = pattern::build(r"`([^`]*)`")?;
    let lead = pattern::build(r"(?:^|[^0-9A-Za-z_])B$")?;
    let trail = pattern::build(r"[\s,;(]*B[0-9]+\b")?;
    let mut counted = 0_usize;
    let mut failures = Vec::new();
    for (site, text) in sites {
        let chars = pattern::chars(text);
        let spans: Vec<(usize, usize, String)> = span
            .find_iter(&chars)
            .iter()
            .map(|one| (one.start(), one.end(), one.text(1, &chars)))
            .collect();
        for one in count.find_iter(&chars) {
            counted = counted.saturating_add(1);
            let before = chars.get(..one.start()).unwrap_or(&[]);
            if lead.find(before).is_some() {
                continue;
            }
            let after = chars.get(one.end()..).unwrap_or(&[]);
            if trail.match_at(after, 0).is_some() {
                continue;
            }
            let inside = spans.iter().any(|(start, end, body)| {
                *start <= one.start() && one.end() <= *end && cites_recorded_row(body, recorded)
            });
            if inside {
                continue;
            }
            failures.push((site.clone(), one.text(1, &chars)));
        }
    }
    Ok((counted, failures))
}

/// Every `pub const` of section 1.6, as its crate and its integer value.
///
/// The crate is the `// duet-<name>` comment above the declaration, which is the
/// one place this document states where a constant lives. PG24 reads the value
/// as an array length and `PG20b` reads the crate (critic CR-12).
fn constant_owners(blocks: &Blocks) -> anyhow::Result<ConstantOwners> {
    let head = pattern::build(r"\s*// (duet-[a-z]+)")?;
    let declaration =
        pattern::build(r"pub const ([A-Z][A-Z0-9_]*)\s*:\s*([A-Za-z0-9_]+)\s*=\s*([^;]+);")?;
    let mut found = BTreeMap::new();
    let mut crate_name = String::new();
    for row in blocks.rows("constants") {
        let line = row.line();
        let text = pattern::chars(&line);
        if let Some(one) = head.match_at(&text, 0) {
            crate_name = one.text(1, &text);
        }
        let Some(one) = declaration.find(&text) else {
            continue;
        };
        let mut value = None;
        if one.text(2, &text) == "usize" {
            let raw = one.text(3, &text);
            let digits: String = raw.chars().filter(|mark| *mark != '_').collect();
            let digits = digits.trim();
            if all_digits(digits) {
                value = digits.parse::<usize>().ok();
            }
        }
        found.insert(one.text(1, &text), (crate_name.clone(), value));
    }
    Ok(found)
}

/// Every `pub const NAME: usize` of section 1.6, as its value (PG24).
fn usize_constants(owners: &BTreeMap<String, (String, Option<usize>)>) -> BTreeMap<String, usize> {
    owners
        .iter()
        .filter_map(|(name, (_crate_name, value))| value.map(|found| (name.clone(), found)))
        .collect()
}

/// `PG20b`. A constant a declaration names, in a crate the graph misses.
///
/// An array length and a const generic argument are values, so `parse_type`
/// drops them and PG20 cannot see one. A constant inside a function body is
/// invisible to this rule, and PG28 holds that half.
fn constant_reach_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    closure: &BTreeMap<String, BTreeSet<String>>,
    constants: &BTreeMap<String, (String, Option<usize>)>,
) -> anyhow::Result<Vec<Quint>> {
    let token = pattern::build(r"(?<![:\w])([A-Z][A-Z0-9_]{2,})(?![\w])")?;
    let mut failures = BTreeSet::new();
    for name in decls.sorted_keys() {
        let Some(crate_name) = owned.get(&name) else {
            continue;
        };
        let home = normalize_crate(crate_name);
        for entry in decls.get(&name).into_iter().flatten() {
            for (field, expr) in field_exprs(&entry.kind, &entry.payload)? {
                let site = ConstantSite {
                    crate_name,
                    name: &name,
                    field: &field,
                    home: &home,
                };
                failures.extend(constant_misses(&token, &expr, site, closure, constants));
            }
        }
    }
    Ok(failures.into_iter().collect())
}

/// The declaration site one constant reference sits in.
#[derive(Debug, Clone, Copy)]
struct ConstantSite<'a> {
    /// The crate the section 1.5 table gives the declaration.
    crate_name: &'a str,
    /// The declaration that names the constant.
    name: &'a str,
    /// The field whose type expression names it.
    field: &'a str,
    /// The 1.3 spelling of the declaring crate.
    home: &'a str,
}

impl ConstantSite<'_> {
    /// One finding row for a constant this site cannot reach.
    fn row(self, found: String, target: String) -> Quint {
        (
            self.crate_name.to_owned(),
            self.name.to_owned(),
            self.field.to_owned(),
            found,
            target,
        )
    }
}

/// Every constant one field expression names in a crate the graph misses.
fn constant_misses(
    token: &pattern::Regex,
    expr: &str,
    site: ConstantSite<'_>,
    closure: &BTreeMap<String, BTreeSet<String>>,
    constants: &ConstantOwners,
) -> Vec<Quint> {
    let text = pattern::chars(expr);
    token
        .find_iter(&text)
        .iter()
        .map(|one| one.text(1, &text))
        .filter_map(|found| unreached_constant(&found, site.home, closure, constants))
        .map(|(found, target)| site.row(found, target))
        .collect()
}

/// One constant and its crate, when the graph does not reach that crate.
fn unreached_constant(
    found: &str,
    home: &str,
    closure: &BTreeMap<String, BTreeSet<String>>,
    constants: &BTreeMap<String, (String, Option<usize>)>,
) -> Option<(String, String)> {
    let (target, _value) = constants.get(found)?;
    if target.is_empty() || target == home {
        return None;
    }
    let reached = closure
        .get(home)
        .is_some_and(|targets| targets.contains(target));
    (!reached).then(|| (found.to_owned(), target.clone()))
}

/// PG28. A shared limit an enforcing crate cannot reach (critic CR-12).
///
/// Each line of the section 1.6 block is the constant, the crate that declares
/// it, and every crate that enforces it inside a function body. A body is not a
/// field, so PG20 and `PG20b` both miss the use.
fn shared_limit_audit(
    blocks: &Blocks,
    closure: &BTreeMap<String, BTreeSet<String>>,
    constants: &BTreeMap<String, (String, Option<usize>)>,
) -> (usize, Vec<(String, String, String)>) {
    let mut rows = 0_usize;
    let mut failures = Vec::new();
    for row in blocks.rows("shared-limits") {
        let line = row.line();
        let parts = tokens(&line);
        if parts.len() < 3 {
            failures.push((
                line.trim().to_owned(),
                String::new(),
                "the line states no owner and no enforcer".to_owned(),
            ));
            continue;
        }
        let constant = parts.first().cloned().unwrap_or_default();
        let owner = parts.get(1).cloned().unwrap_or_default();
        rows = rows.saturating_add(1);
        let Some((declared, _value)) = constants.get(&constant) else {
            failures.push((
                constant,
                owner,
                "section 1.6 declares no such constant".to_owned(),
            ));
            continue;
        };
        if *declared != owner {
            failures.push((
                constant,
                owner,
                format!("the constant block declares it in {declared}"),
            ));
            continue;
        }
        for enforcer in parts.iter().skip(2) {
            let reaches = closure
                .get(enforcer)
                .is_some_and(|targets| targets.contains(&owner));
            if *enforcer != owner && !reaches {
                failures.push((
                    constant.clone(),
                    enforcer.clone(),
                    format!("{enforcer} does not reach {owner}"),
                ));
            }
        }
    }
    (rows, failures)
}

/// Every rule id the prototypes implement, SCANNED and never typed (PG29).
///
/// A rule id is a TOKEN of a prototype, so an id that survives in a comment
/// after its implementation is deleted still counts. The scan is strictly
/// stronger than a literal, because a deleted file, a renamed id, and an id
/// typed into the table alone are each red.
fn implemented_rule_ids(tools: &Path) -> anyhow::Result<Result<BTreeSet<String>, String>> {
    let rule = pattern::build(r"\b((?:PG|CG)\d{1,2}[a-z]?)\b")?;
    let mut found = BTreeSet::new();
    for name in PROTOTYPES {
        let path = tools.join(name);
        let Ok(text) = fs::read_to_string(&path) else {
            return Ok(Err(format!(
                "the prototype `{name}` does not open, so the rule set is unknown"
            )));
        };
        let chars = pattern::chars(&text);
        for one in rule.find_iter(&chars) {
            found.insert(one.text(1, &chars));
        }
    }
    if found.is_empty() {
        return Ok(Err(
            "the prototypes name no rule id, so the denominator is zero".to_owned(),
        ));
    }
    Ok(Ok(found))
}

/// PG29. The section 1.9 probe table and the rule set are one set (DR5).
///
/// It states its own limit at its section 1.5 site: the recorded text of a cell
/// is unchecked, because only a run can produce it (critic concern 1).
fn probe_table_audit(blocks: &Blocks, rule_ids: &BTreeSet<String>) -> anyhow::Result<Counted> {
    let recorded = pattern::build(r"exit [012]|TBD-RUN")?;
    let mut rows: BTreeMap<String, (String, String)> = BTreeMap::new();
    for row in blocks.rows("probe-table") {
        if row.cells().len() < 5 {
            continue;
        }
        rows.insert(
            strip_ticks(&row.cell(0)),
            (strip_ticks(&row.cell(2)), row.cell(4)),
        );
    }
    let mut failures = Vec::new();
    for rule in rule_ids {
        let Some((probe, cell)) = rows.get(rule) else {
            failures.push((rule.clone(), "the probe table carries no row".to_owned()));
            continue;
        };
        let chars: Vec<char> = rule.chars().collect();
        let head = chars.first().copied().unwrap_or('P');
        let tail = pattern::slice(&chars, 2, chars.len()).unwrap_or_default();
        let wanted = format!("{head}P{tail}");
        if *probe != wanted {
            failures.push((
                rule.clone(),
                format!("the row names probe {probe} and {wanted} is required"),
            ));
        }
        if !recorded.is_match(&pattern::chars(cell)) {
            failures.push((
                rule.clone(),
                "the recorded cell states no exit code".to_owned(),
            ));
        }
    }
    for rule in rows.keys() {
        if rule == "-" || rule == "BASE" {
            continue;
        }
        if !rule_ids.contains(rule) {
            failures.push((
                rule.clone(),
                "no prototype implements this rule id".to_owned(),
            ));
        }
    }
    Ok((rows.len(), failures))
}

/// The section 1.9 recorded-size block, as its expressions and its head rows.
///
/// Each row is `<type expression> <size> <align>` and each one is a compiler
/// fact the roster compile measured (DR3 exemption 4). A row that ends in the
/// word `generic` names a container HEAD, whose size the payload never changes.
fn recorded_sizes(blocks: &Blocks) -> (SizeMap, SizeMap) {
    let mut expressions = BTreeMap::new();
    let mut heads = BTreeMap::new();
    for row in blocks.rows("recorded-sizes") {
        let mut parts = row.tokens();
        let generic = parts.last().is_some_and(|last| last == "generic");
        if generic {
            parts.pop();
        }
        if parts.len() < 3 {
            continue;
        }
        let last = parts.len().saturating_sub(1);
        let before = last.saturating_sub(1);
        let (Some(align), Some(size)) = (parts.get(last), parts.get(before)) else {
            continue;
        };
        if !all_digits(align) || !all_digits(size) {
            continue;
        }
        let (Some(size), Some(align)) = (size.parse::<usize>().ok(), align.parse::<usize>().ok())
        else {
            continue;
        };
        let name = normalize_expr(&parts.get(..before).unwrap_or(&[]).join(" "));
        expressions.insert(name.clone(), (size, align));
        if generic {
            heads.insert(name, (size, align));
        }
    }
    (expressions, heads)
}

/// Every field expression the layout model reads, resolved once.
#[derive(Debug, Default)]
struct FieldIndex {
    /// Every `(field, expression)` a declaration body states.
    plain: BTreeMap<String, Vec<(String, String)>>,
    /// Every arm of each enum, with its own fields.
    arms: BTreeMap<String, ArmFields>,
}

impl FieldIndex {
    /// Read every declaration body and every enum arm once.
    fn build(
        decls: &DeclMap,
        arms: &BTreeMap<String, Vec<(String, String)>>,
    ) -> anyhow::Result<Self> {
        let mut index = Self::default();
        for (name, entries) in decls.iter() {
            let Some(entry) = entries.first() else {
                continue;
            };
            index
                .plain
                .insert(name.clone(), field_exprs("struct", &entry.payload)?);
        }
        for (name, listed) in arms {
            let mut built = Vec::new();
            for (arm_name, arm_body) in listed {
                built.push((arm_name.clone(), field_exprs("enum", arm_body)?));
            }
            index.arms.insert(name.clone(), built);
        }
        Ok(index)
    }

    /// The fields one declaration body states.
    fn fields_of(&self, name: &str) -> &[(String, String)] {
        self.plain.get(name).map_or(&[], Vec::as_slice)
    }

    /// The arms one enum declares.
    fn arms_of(&self, name: &str) -> &ArmFieldList {
        self.arms.get(name).map_or(&[], Vec::as_slice)
    }
}

/// `value` raised to the next multiple of `align`.
const fn round_up(value: usize, align: usize) -> usize {
    if align <= 1 {
        return value;
    }
    let remainder = value % align;
    if remainder == 0 {
        value
    } else {
        value.saturating_add(align).saturating_sub(remainder)
    }
}

/// The size and alignment of a body, from its field list (PG24).
///
/// The fields sort by decreasing alignment, which is the size-optimal
/// `repr(Rust)` order rustc picks, each one pads to its own alignment, and the
/// total rounds up to the largest field alignment.
fn layout_fields(parts: &[(usize, usize)]) -> (usize, usize) {
    if parts.is_empty() {
        return (0, 1);
    }
    let align = parts.iter().map(|part| part.1).max().unwrap_or(1);
    let mut sorted = parts.to_vec();
    sorted.sort_by_key(|part| core::cmp::Reverse(part.1));
    let mut offset = 0_usize;
    for (size, field_align) in sorted {
        offset = round_up(offset, field_align).saturating_add(size);
    }
    (round_up(offset, align), align)
}

/// The length of an array expression, or `None` when no section states it.
fn array_length(text: &str, constants: &BTreeMap<String, usize>) -> anyhow::Result<Option<usize>> {
    let text = text.trim();
    let digits = pattern::build(r"[0-9_]+")?;
    if digits.full_match(&pattern::chars(text)).is_some() {
        return Ok(digits_value(text));
    }
    let leaf = text.rsplit("::").next().unwrap_or(text);
    Ok(constants.get(leaf).copied())
}

/// Whether one type argument is unsized, so a pointer to it is fat.
fn unsized_argument(text: &str) -> anyhow::Result<bool> {
    let text = text.trim();
    if text == "str"
        || pattern::build(r"^(?:dyn|impl)\b")?
            .match_at(&pattern::chars(text), 0)
            .is_some()
    {
        return Ok(true);
    }
    if !text.starts_with('[') || !text.ends_with(']') {
        return Ok(false);
    }
    let chars = pattern::chars(text);
    let inner = pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
    Ok(split_top(&inner, ';').len() == 1)
}

/// The size and alignment of one declaration, with each arm size it carries.
type Measured = (usize, usize, Vec<(String, usize)>);

/// A layout oracle for PG24.
///
/// The primitive map comes from the section 1.9 block, never from a constant, so
/// an empty map decides no primitive size and every declaration above one goes
/// undecided (critic K-4). A cycle through a declaration is undecided, and only a
/// decided answer enters the cache.
#[derive(Debug)]
struct Sizer<'a> {
    /// Every declaration this document writes.
    decls: &'a DeclMap,
    /// Every field expression, resolved once.
    index: &'a FieldIndex,
    /// Every whole-expression size the document records.
    recorded: &'a BTreeMap<String, (usize, usize)>,
    /// Every head-row size the document records.
    heads: &'a BTreeMap<String, (usize, usize)>,
    /// Every `usize` constant section 1.6 declares.
    constants: &'a BTreeMap<String, usize>,
    /// Every primitive size section 1.9 states.
    primitives: &'a BTreeMap<String, (usize, usize)>,
    /// The pattern that reads a head name and its arguments.
    named: pattern::Regex,
    /// Every decided answer, by name.
    cache: BTreeMap<String, Measured>,
    /// Every name the model is measuring right now.
    active: BTreeSet<String>,
}

impl Sizer<'_> {
    /// One type expression as a head name and its arguments, or `None`.
    fn head_and_args(&self, text: &str) -> Option<(String, Vec<String>)> {
        let chars = pattern::chars(text);
        let found = self.named.full_match(&chars)?;
        let args = found.span(2).map_or_else(Vec::new, |(start, end)| {
            let raw = pattern::slice(&chars, start.saturating_add(1), end.saturating_sub(1))
                .unwrap_or_default();
            split_top(&raw, ',')
                .into_iter()
                .map(|piece| piece.trim().to_owned())
                .filter(|piece| !piece.is_empty())
                .collect()
        });
        Some((found.text(1, &chars), args))
    }

    /// The size and alignment of one type expression, or `None` (PG24).
    ///
    /// A recorded row wins, because it is what the compiler measured. After that
    /// the model states three language rules and reads the head of the
    /// expression only, never a nested argument (critic C-5).
    fn size_of_expr(&mut self, text: &str) -> anyhow::Result<Option<(usize, usize)>> {
        let text = text.trim().to_owned();
        if text.is_empty() || text.starts_with('&') {
            return Ok(None);
        }
        let key = normalize_expr(&text);
        if let Some(found) = self.recorded.get(&key) {
            return Ok(Some(*found));
        }
        let chars = pattern::chars(&text);
        if text.starts_with('(') && text.ends_with(')') {
            let inner =
                pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
            let Some(parts) = self.size_parts(&split_top(&inner, ','))? else {
                return Ok(None);
            };
            return Ok(Some(layout_fields(&parts)));
        }
        if text.starts_with('[') && text.ends_with(']') {
            let inner =
                pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
            let halves = split_top(&inner, ';');
            if halves.len() != 2 {
                return Ok(None);
            }
            let first = halves.first().cloned().unwrap_or_default();
            let second = halves.get(1).cloned().unwrap_or_default();
            let count = array_length(&second, self.constants)?;
            let element = self.size_of_expr(&first)?;
            let (Some(count), Some(element)) = (count, element) else {
                return Ok(None);
            };
            return Ok(Some((count.saturating_mul(element.0), element.1)));
        }
        let Some((head, args)) = self.head_and_args(&text) else {
            return Ok(None);
        };
        if FAT_POINTER_HEADS.contains(&head.as_str()) && args.len() == 1 {
            let first = args.first().cloned().unwrap_or_default();
            if unsized_argument(&first)? {
                return Ok(Some((16, 8)));
            }
        }
        if !args.is_empty() && INLINE_HEADS.contains(&head.as_str()) {
            return Ok(None);
        }
        Ok(self.measure(&head)?.map(|found| (found.0, found.1)))
    }

    /// The size and alignment of every expression, or `None` when one is open.
    fn size_parts(&mut self, exprs: &[String]) -> anyhow::Result<Option<Vec<(usize, usize)>>> {
        let mut parts = Vec::new();
        for expr in exprs {
            let Some(part) = self.size_of_expr(expr)? else {
                return Ok(None);
            };
            parts.push(part);
        }
        Ok(Some(parts))
    }

    /// The size, alignment, and arm sizes of one name, or `None`.
    fn measure(&mut self, name: &str) -> anyhow::Result<Option<Measured>> {
        if let Some(&(size, align)) = self.primitives.get(name) {
            return Ok(Some((size, align, Vec::new())));
        }
        if let Some(found) = self.cache.get(name) {
            return Ok(Some(found.clone()));
        }
        if self.active.contains(name) {
            return Ok(None);
        }
        if self.decls.holds(name) {
            self.active.insert(name.to_owned());
            let answer = self.measure_declaration(name)?;
            self.active.remove(name);
            if let Some(ref found) = answer {
                self.cache.insert(name.to_owned(), found.clone());
            }
            return Ok(answer);
        }
        if let Some(&(size, align)) = self.heads.get(name) {
            return Ok(Some((size, align, Vec::new())));
        }
        if let Some(&(size, align)) = self.recorded.get(name) {
            return Ok(Some((size, align, Vec::new())));
        }
        Ok(None)
    }

    /// The layout of one declaration, or `None` when the model cannot read it.
    fn measure_declaration(&mut self, name: &str) -> anyhow::Result<Option<Measured>> {
        let Some(entry) = self.decls.first(name) else {
            return Ok(None);
        };
        let kind = entry.kind.clone();
        let payload = entry.payload.clone();
        if kind == "trait" || private_body(&payload) {
            return Ok(None);
        }
        if kind == "enum" {
            return self.measure_enum(name);
        }
        self.measure_struct(name)
    }

    /// The layout of one struct body.
    fn measure_struct(&mut self, name: &str) -> anyhow::Result<Option<Measured>> {
        let fields: Vec<String> = self
            .index
            .fields_of(name)
            .iter()
            .map(|(_field, expr)| expr.clone())
            .collect();
        let Some(parts) = self.size_parts(&fields)? else {
            return Ok(None);
        };
        let (size, align) = layout_fields(&parts);
        Ok(Some((size, align, Vec::new())))
    }

    /// The layout of one enum, with each arm padded to the payload alignment.
    fn measure_enum(&mut self, name: &str) -> anyhow::Result<Option<Measured>> {
        let arms: Vec<(String, Vec<String>)> = self
            .index
            .arms_of(name)
            .iter()
            .map(|(arm_name, fields)| {
                (
                    arm_name.clone(),
                    fields.iter().map(|(_field, expr)| expr.clone()).collect(),
                )
            })
            .collect();
        let mut laid = Vec::new();
        for (arm_name, exprs) in arms {
            let Some(parts) = self.size_parts(&exprs)? else {
                return Ok(None);
            };
            laid.push((arm_name, layout_fields(&parts)));
        }
        if laid.is_empty() {
            return Ok(None);
        }
        if !laid.iter().any(|(_arm, part)| part.0 != 0) {
            return Ok(Some((
                1,
                1,
                laid.iter().map(|(arm, _part)| (arm.clone(), 0)).collect(),
            )));
        }
        let align = laid
            .iter()
            .map(|(_arm, part)| part.1)
            .max()
            .unwrap_or(1)
            .max(1);
        let widest = laid.iter().map(|(_arm, part)| part.0).max().unwrap_or(0);
        let sizes = laid
            .iter()
            .map(|(arm, part)| (arm.clone(), round_up(part.0, align)))
            .collect();
        Ok(Some((
            round_up(widest.saturating_add(1), align),
            align,
            sizes,
        )))
    }

    /// Every field expression of one declaration the model cannot size.
    fn explain(&mut self, name: &str) -> anyhow::Result<Vec<(String, String)>> {
        let Some(entry) = self.decls.first(name) else {
            return Ok(Vec::new());
        };
        let kind = entry.kind.clone();
        let payload = entry.payload.clone();
        if kind == "trait" {
            return Ok(Vec::new());
        }
        if private_body(&payload) {
            return Ok(vec![("body".to_owned(), "/* private */".to_owned())]);
        }
        if kind != "enum" {
            let fields = self.index.fields_of(name).to_vec();
            return self.open_fields(&fields, "");
        }
        let arms = self.index.arms_of(name).to_vec();
        let mut out = Vec::new();
        for (arm_name, fields) in &arms {
            out.extend(self.open_fields(fields, arm_name)?);
        }
        if arms.is_empty() {
            out.push((
                "body".to_owned(),
                "an arm list this guard cannot read".to_owned(),
            ));
        }
        Ok(out)
    }

    /// Every field of one list the model cannot size, with the label it prints.
    fn open_fields(
        &mut self,
        fields: &[(String, String)],
        arm: &str,
    ) -> anyhow::Result<Vec<(String, String)>> {
        let mut out = Vec::new();
        for (field, expr) in fields {
            if self.size_of_expr(expr)?.is_some() {
                continue;
            }
            let label = if arm.is_empty() {
                field_label(field, expr)
            } else if field.is_empty() {
                arm.to_owned()
            } else {
                format!("{arm}.{field}")
            };
            out.push((label, expr.trim().to_owned()));
        }
        Ok(out)
    }

    /// Whether one type offers a spare bit pattern (PG24).
    ///
    /// rustc lints `variant_size_differences` on a direct tag only. An integer
    /// and a float offer none. `bool` and `char` each offer one. Every other name
    /// offers one, because an unknown must not turn into a failure this guard
    /// cannot justify.
    fn offers_niche(&self, text: &str, seen: &BTreeSet<String>) -> anyhow::Result<bool> {
        let text = text.trim().to_owned();
        if text.is_empty() || seen.contains(&text) {
            return Ok(false);
        }
        if text.starts_with('&') {
            return Ok(true);
        }
        let chars = pattern::chars(&text);
        if text.starts_with('(') && text.ends_with(')') {
            let inner =
                pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
            return self.any_niche(&split_top(&inner, ','), seen);
        }
        if text.starts_with('[') && text.ends_with(']') {
            let inner =
                pattern::slice(&chars, 1, chars.len().saturating_sub(1)).unwrap_or_default();
            let halves = split_top(&inner, ';');
            if halves.len() != 2 {
                return Ok(true);
            }
            let second = halves.get(1).cloned().unwrap_or_default();
            if array_length(&second, self.constants)? == Some(0) {
                return Ok(true);
            }
            let first = halves.first().cloned().unwrap_or_default();
            return self.offers_niche(&first, seen);
        }
        let Some((head, _args)) = self.head_and_args(&text) else {
            return Ok(true);
        };
        if NICHE_FREE.contains(&head.as_str()) {
            return Ok(false);
        }
        let Some(entry) = self.decls.first(&head) else {
            return Ok(true);
        };
        if entry.kind != "struct" || private_body(&entry.payload) {
            return Ok(true);
        }
        let mut deeper = seen.clone();
        deeper.insert(text);
        for (_field, expr) in self.index.fields_of(&head) {
            if self.offers_niche(expr, &deeper)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether any expression of a list offers a spare bit pattern.
    fn any_niche(&self, exprs: &[String], seen: &BTreeSet<String>) -> anyhow::Result<bool> {
        for expr in exprs {
            if self.offers_niche(expr, seen)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Whether rustc gives one enum a direct tag, which PG24 needs.
    ///
    /// rustc puts the tag in a spare bit pattern of the largest arm when that arm
    /// has one, and it returns from the lint before the comparison in that case.
    fn direct_tag(&self, name: &str, arm_sizes: &[(String, usize)]) -> anyhow::Result<bool> {
        let mut widest: Option<&(String, usize)> = None;
        for item in arm_sizes {
            if widest.is_none_or(|found| item.1 > found.1) {
                widest = Some(item);
            }
        }
        let Some((widest, _size)) = widest else {
            return Ok(false);
        };
        for (arm_name, fields) in self.index.arms_of(name) {
            if arm_name != widest {
                continue;
            }
            let exprs: Vec<String> = fields.iter().map(|(_field, expr)| expr.clone()).collect();
            return Ok(!self.any_niche(&exprs, &BTreeSet::new())?);
        }
        Ok(false)
    }
}

/// What the PG24 layout run decided.
#[derive(Debug, Default)]
struct SizeReport {
    /// Every declaration the model sized.
    decided: Vec<SizeRow>,
    /// Every declaration the model cannot size, with the reason.
    undecided: Vec<OpenRow>,
    /// Every enum whose largest arm is more than three times its next largest.
    spread: Vec<(String, String, String, usize, usize)>,
    /// Every spread a single-site expectation takes out of the failure set.
    expected: Vec<(String, String, String, usize, usize)>,
}

/// PG24. Every declaration's size, and `variant_size_differences`.
///
/// The spread reads the arms that carry a payload, each one padded to the
/// payload alignment of the enum. A niche-tagged enum is outside the lint, so
/// `direct_tag` decides before the comparison runs.
fn size_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    sizer: &mut Sizer<'_>,
) -> anyhow::Result<SizeReport> {
    let mut report = SizeReport::default();
    for name in decls.sorted_keys() {
        let Some(entry) = decls.first(&name) else {
            continue;
        };
        let kind = entry.kind.clone();
        let attrs = entry.attrs.clone();
        if kind == "trait" {
            continue;
        }
        let crate_name = owned
            .get(&name)
            .cloned()
            .unwrap_or_else(|| "unplaced".to_owned());
        let Some((size, align, arm_sizes)) = sizer.measure(&name)? else {
            let reasons = sizer.explain(&name)?;
            report.undecided.push((name.clone(), crate_name, reasons));
            continue;
        };
        report.decided.push((
            name.clone(),
            crate_name.clone(),
            size,
            align,
            arm_sizes.clone(),
        ));
        let mut carried: Vec<(String, usize)> = arm_sizes
            .iter()
            .filter(|item| item.1 > 0)
            .cloned()
            .collect();
        carried.sort_by_key(|item| core::cmp::Reverse(item.1));
        let (Some(first), Some(second)) = (carried.first(), carried.get(1)) else {
            continue;
        };
        if first.1 <= second.1.saturating_mul(3) {
            continue;
        }
        if !sizer.direct_tag(&name, &arm_sizes)? {
            continue;
        }
        let row = (name.clone(), crate_name, first.0.clone(), first.1, second.1);
        if attrs.contains("variant_size_differences") {
            report.expected.push(row);
        } else {
            report.spread.push(row);
        }
    }
    Ok(report)
}

/// `PG27b`. A floor BELOW its own block's row count is a failure.
///
/// The floor is EQUAL to the row count and never below it, so an addition raises
/// the floor in the same changeset and a deletion is always red (critic C20-W10,
/// C20-N5). It runs after the membership rules and it reports a finding, not a
/// fail-closed input failure.
fn floor_audit(blocks: &Blocks) -> Vec<(String, String)> {
    let counted = sync_floors::floors(blocks);
    let mut failures = Vec::new();
    for (block_id, rows) in counted {
        let Some(spec) = spec_of(block_id) else {
            continue;
        };
        if rows > spec.minimum {
            failures.push((
                block_id.to_owned(),
                format!(
                    "the block holds {rows} rows and the stated floor is {}; a floor below \
the row count grants one free deletion per addition",
                    spec.minimum
                ),
            ));
        }
    }
    failures
}

/// `PG26d`. The block and the marked set are one set, in both directions.
///
/// The two sources it compares are two hand-written copies of ONE judgement, so
/// holding them consistent tests self-consistency and not truth. `PG26e` is the
/// independent oracle (critic C16-W1).
fn audio_root_audit(roots: &BTreeSet<String>, marked: &BTreeSet<String>) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    for name in roots.difference(marked) {
        failures.push((
            name.clone(),
            "the block names it and its declaration carries no marker".to_owned(),
        ));
    }
    for name in marked.difference(roots) {
        failures.push((
            name.clone(),
            "its declaration carries the marker and the block omits it".to_owned(),
        ));
    }
    failures
}

/// Every declared type the audio root reaches through a declared field.
///
/// The set is DERIVED from the declaration bodies. It is the third source `PG26e`
/// needs, so a coordinated edit of the block and the markers cannot move it
/// (critic C16-W1).
fn audio_reachable(
    decls: &DeclMap,
    arms: &BTreeMap<String, Vec<(String, String)>>,
) -> anyhow::Result<BTreeSet<String>> {
    let word = pattern::build(r"\b([A-Z][A-Za-z0-9_]*)\b")?;
    let mut seen: BTreeSet<String> = BTreeSet::new();
    let mut stack = vec![AUDIO_ROOT_OF_ROOTS.to_owned()];
    while let Some(name) = stack.pop() {
        if !seen.insert(name.clone()) {
            continue;
        }
        if !decls.holds(&name) {
            continue;
        }
        for (_field, expr) in walk_fields(&name, decls, arms)? {
            let text = pattern::chars(&expr);
            let found = word.find_iter(&text);
            let leaves = found
                .iter()
                .map(|one| one.text(1, &text))
                .filter(|leaf| decls.holds(leaf) && !seen.contains(leaf));
            stack.extend(leaves);
        }
    }
    Ok(seen.into_iter().filter(|name| decls.holds(name)).collect())
}

/// `PG26f`. The third source of the audio-owned root set (critic C17-W8).
///
/// Both directions run. A root the closure does not reach and this block does
/// not name is a failure, and a name this block holds that the root set omits is
/// a failure.
fn audio_asserted_audit(
    roots: &BTreeSet<String>,
    reached: &BTreeSet<String>,
    asserted: &BTreeSet<String>,
) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    for name in roots {
        if reached.contains(name) || asserted.contains(name) {
            continue;
        }
        failures.push((
            name.clone(),
            "the root set names it, the closure does not reach it, and the asserted block \
does not name it"
                .to_owned(),
        ));
    }
    for name in asserted.difference(roots) {
        failures.push((
            name.clone(),
            "the asserted block names it and the audio-owned block omits it".to_owned(),
        ));
    }
    failures
}

/// `PG26e`. The audio-owned set is closed under reachability.
///
/// Everything the one audio value reaches through a declared field is on the
/// audio thread by construction, so every reachable declaration is an
/// audio-owned root or a leaf the `audio-reachable-leaf` block names.
fn audio_closure_audit(
    roots: &BTreeSet<String>,
    reached: &BTreeSet<String>,
    leaves: &BTreeSet<String>,
) -> Vec<(String, String)> {
    let mut failures = Vec::new();
    for name in reached {
        if roots.contains(name) || leaves.contains(name) {
            continue;
        }
        failures.push((
            name.clone(),
            format!(
                "{AUDIO_ROOT_OF_ROOTS} reaches it through a declared field and it is neither \
an audio-owned root nor an audio-reachable leaf"
            ),
        ));
    }
    for name in leaves.difference(reached) {
        failures.push((
            name.clone(),
            format!("the leaf block names it and {AUDIO_ROOT_OF_ROOTS} does not reach it"),
        ));
    }
    failures
}

/// One finding of the audio walk: the root, its crate, the expression, the path.
type AudioHit = (String, String, String, String);

/// The name sets the three TH1 rules refuse.
#[derive(Debug, Clone)]
struct AudioSets {
    /// Every heap-owning name PG26 refuses.
    heap: BTreeSet<String>,
    /// Every field the exemption block answers the free half for.
    exempt: BTreeMap<(String, String), String>,
    /// Every wrapper that defers the free to the collector thread.
    defer: BTreeSet<String>,
    /// Every growable name `PG26b` refuses.
    grows: BTreeSet<String>,
    /// Every lock name `PG26c` refuses.
    locks: BTreeSet<String>,
    /// Every read end that hands this thread a shared reference.
    reads: BTreeSet<String>,
}

/// What the audio walk found, one list per rule.
#[derive(Debug, Default)]
struct AudioReport {
    /// PG26. Every heap allocation inside audio-owned state.
    heap: Vec<AudioHit>,
    /// `PG26b`. Every container that grows inside audio-owned state.
    grows: Vec<AudioHit>,
    /// `PG26c`. Every lock inside audio-owned state.
    locks: Vec<AudioHit>,
    /// Every field a deferring wrapper carries to the collector thread.
    deferred: Vec<AudioHit>,
    /// Every field a read end hands over as a shared reference.
    readonly: Vec<AudioHit>,
}

/// Where the audio walk is, and what the wrapper above it suppresses.
#[derive(Debug, Clone, Copy)]
struct WalkState<'a> {
    /// The audio-owned root the walk began at.
    root: &'a str,
    /// The crate that owns the root.
    crate_name: &'a str,
    /// Whether an exemption row or a deferring wrapper stops the heap test.
    inside: bool,
    /// Whether a read end makes everything below a shared reference.
    published: bool,
}

/// The walk PG26, `PG26b`, and `PG26c` run over audio-owned state.
#[derive(Debug)]
struct AudioWalk<'a> {
    /// Every declaration this document writes.
    decls: &'a DeclMap,
    /// Every enum arm this document declares.
    arms: &'a BTreeMap<String, Vec<(String, String)>>,
    /// The name sets the three rules refuse.
    sets: &'a AudioSets,
    /// What the walk found so far.
    report: AudioReport,
}

impl AudioWalk<'_> {
    /// Walk one declaration and everything it reaches through a field.
    ///
    /// The walk continues through a deferring wrapper rather than stopping at
    /// it, because the allocation half and the lock half both have to read what
    /// the wrapper carries. Only the heap test is suppressed below one.
    fn walk(
        &mut self,
        state: WalkState<'_>,
        name: &str,
        path: &[String],
        seen: &BTreeSet<String>,
    ) -> anyhow::Result<()> {
        if seen.contains(name) || !self.decls.holds(name) {
            return Ok(());
        }
        let mut deeper = seen.clone();
        deeper.insert(name.to_owned());
        for (field, expr) in walk_fields(name, self.decls, self.arms)? {
            let mut step = path.to_vec();
            step.push(field.clone());
            let names = leaf_names(parse_type(&expr)?.as_ref());
            let hit = (
                state.root.to_owned(),
                state.crate_name.to_owned(),
                expr.trim().to_owned(),
                step.join("."),
            );
            if names.iter().any(|one| self.sets.locks.contains(one)) {
                self.report.locks.push(hit);
                continue;
            }
            let grows = names.iter().any(|one| self.sets.grows.contains(one));
            if !state.published && grows {
                self.report.grows.push(hit);
                continue;
            }
            let head = expr_head(&expr);
            let reads = names.iter().any(|one| self.sets.reads.contains(one));
            if reads {
                self.report.readonly.push(hit.clone());
            }
            let exempt = self.sets.exempt.get(&(name.to_owned(), field.clone())) == head.as_ref();
            let defers = head
                .as_ref()
                .is_some_and(|one| self.sets.defer.contains(one));
            let mut below = state.inside || exempt;
            if defers {
                self.report.deferred.push(hit);
                below = true;
            } else if !below && names.iter().any(|one| self.sets.heap.contains(one)) {
                self.report.heap.push(hit);
                continue;
            }
            let deeper_state = WalkState {
                root: state.root,
                crate_name: state.crate_name,
                inside: below,
                published: state.published || reads,
            };
            for token in &names {
                self.walk(deeper_state, token, &step, &deeper)?;
            }
        }
        Ok(())
    }
}

/// PG26, `PG26b`, and `PG26c`. What audio-owned state may not hold.
///
/// TH1 states that the audio thread never allocates, never frees, and never
/// locks. Three rules read three halves of that sentence, and each one has its
/// own probe (DR5).
fn heap_audit(
    decls: &DeclMap,
    owned: &BTreeMap<String, String>,
    arms: &BTreeMap<String, Vec<(String, String)>>,
    roots: &BTreeSet<String>,
    sets: &AudioSets,
) -> anyhow::Result<AudioReport> {
    let mut walk = AudioWalk {
        decls,
        arms,
        sets,
        report: AudioReport::default(),
    };
    for root in roots {
        let crate_name = owned
            .get(root)
            .cloned()
            .unwrap_or_else(|| "unplaced".to_owned());
        let state = WalkState {
            root,
            crate_name: &crate_name,
            inside: false,
            published: false,
        };
        walk.walk(state, root, &[], &BTreeSet::new())?;
    }
    Ok(walk.report)
}

/// The text of section 14, or the empty string.
fn section_fourteen(source: &[char]) -> anyhow::Result<String> {
    let anchor = pattern::build(
        r"(?s)## 14\. The measurable completion outcome for version one\n(.*?)\n## Appendix A",
    )?;
    Ok(anchor
        .find(source)
        .map(|one| one.text(1, source))
        .unwrap_or_default())
}

/// Every chunk id and its phase, from the section 13.3 table.
fn chunk_phases(blocks: &Blocks) -> anyhow::Result<BTreeMap<String, usize>> {
    let chunk = pattern::build(r"\b([A-Z]{1,2}\d{1,2})\b")?;
    let mut phases = BTreeMap::new();
    for row in blocks.rows("phase-table") {
        let cells = row.cells();
        if cells.len() < 4 || !all_digits(&row.cell(0)) {
            continue;
        }
        let Some(phase) = row.cell(0).parse::<usize>().ok() else {
            continue;
        };
        let joined = format!("{} {}", row.cell(1), row.cell(3));
        let text = pattern::chars(&joined);
        for one in chunk.find_iter(&text) {
            phases.insert(one.text(1, &text), phase);
        }
    }
    Ok(phases)
}

/// PG16. Every test section 14 selects, against the selected-test table.
fn selected_tests(source: &[char], blocks: &Blocks) -> anyhow::Result<(Pairs, BTreeSet<String>)> {
    let text = section_fourteen(source)?;
    if text.is_empty() {
        return Ok((
            vec![(
                "<section 14>".to_owned(),
                "the section is absent".to_owned(),
            )],
            BTreeSet::new(),
        ));
    }
    let mut selected: BTreeSet<String> = captures_of(&text, r"\btest\(([A-Za-z0-9_]+)\)")?
        .into_iter()
        .collect();
    selected.extend(captures_of(&text, r"(?<![\w-])--test\s+([A-Za-z0-9_]+)")?);
    let mut rows = BTreeMap::new();
    for row in blocks.rows("selected-tests") {
        if row.cells().len() < 6 {
            continue;
        }
        rows.insert(strip_ticks(&row.cell(0)), (row.cell(1), row.cell(2)));
    }
    let phases = chunk_phases(blocks)?;
    let mut failures = Vec::new();
    for name in &selected {
        let Some((chunk, phase)) = rows.get(name) else {
            failures.push((name.clone(), "no row in the selected-test table".to_owned()));
            continue;
        };
        let Some(found) = phases.get(chunk) else {
            failures.push((name.clone(), format!("chunk {chunk} appears in no phase")));
            continue;
        };
        if all_digits(phase) && phase.parse::<usize>().ok() != Some(*found) {
            failures.push((
                name.clone(),
                format!("chunk {chunk} is in phase {found}, not {phase}"),
            ));
        }
    }
    Ok((failures, selected))
}

/// Every `X before Y` row of the section 13.4 table, as pairs.
///
/// The parser takes the ids on each side of the word `before` in the row's FIRST
/// cell and nothing else. A row whose first cell holds no `before` is not a link
/// row.
fn link_rows(source: &str) -> anyhow::Result<Vec<(String, String)>> {
    let shape = pattern::build(
        r"[A-Z]\d{1,2}(?: and [A-Z]\d{1,2})* before [A-Z]\d{1,2}(?:(?:,| and) [A-Z]\d{1,2})*",
    )?;
    let chunk = pattern::build(r"\b([A-Z]\d{1,2})\b")?;
    let mut pairs = Vec::new();
    for line in source.split('\n') {
        if !line.starts_with("| ") || !line.contains(" before ") {
            continue;
        }
        let head = line.split('|').nth(1).unwrap_or("").trim().to_owned();
        if shape.full_match(&pattern::chars(&head)).is_none() {
            continue;
        }
        let Some((left, right)) = head.split_once(" before ") else {
            continue;
        };
        let first = pattern::chars(left);
        let second = pattern::chars(right);
        for before in chunk.find_iter(&first) {
            for after in chunk.find_iter(&second) {
                pairs.push((before.text(1, &first), after.text(1, &second)));
            }
        }
    }
    Ok(pairs)
}

/// PG31. Every section 13.4 link runs forward in the 13.3 phase table.
///
/// The rule reads the links the document WRITES. It refuses a link that runs
/// backward or sideways, and it refuses a link whose chunk no phase holds
/// (critic C16-12).
fn link_audit(source: &str, blocks: &Blocks) -> anyhow::Result<Vec<(String, String)>> {
    let phases = chunk_phases(blocks)?;
    let mut failures = Vec::new();
    for (before, after) in link_rows(source)? {
        let label = format!("{before} before {after}");
        let Some(first) = phases.get(&before) else {
            failures.push((label, format!("{before} is in no phase")));
            continue;
        };
        let Some(second) = phases.get(&after) else {
            failures.push((label, format!("{after} is in no phase")));
            continue;
        };
        if first >= second {
            failures.push((
                label,
                format!("{before} is in phase {first} and {after} is in phase {second}"),
            ));
        }
    }
    Ok(failures)
}

/// Every chunk id mapped to its line, and the crate that line owns.
///
/// A chunk id alone does not state its line, so the map is data (critic C17-W9).
fn chunk_lines(blocks: &Blocks) -> (BTreeMap<String, String>, BTreeMap<String, String>) {
    let mut lines = BTreeMap::new();
    let mut crates = BTreeMap::new();
    for row in blocks.rows("line-map") {
        let parts = row.tokens();
        let (Some(line), Some(crate_name)) = (parts.first(), parts.get(1)) else {
            continue;
        };
        lines.insert(line.clone(), line.clone());
        crates.insert(line.clone(), normalize_crate(crate_name));
    }
    (lines, crates)
}

/// The crate one chunk writes, from the `line-map`, or `None`.
fn crate_of_chunk(chunk: &str, crates: &BTreeMap<String, String>) -> Option<String> {
    if let Some(found) = crates.get(chunk) {
        return Some(found.clone());
    }
    let head: String = chunk.chars().take(1).collect();
    crates.get(&head).cloned()
}

/// PG31. SM6: two chunks of one line never share a phase.
fn line_phase_audit(blocks: &Blocks) -> anyhow::Result<Vec<(String, String)>> {
    let phases = chunk_phases(blocks)?;
    let (lines, _crates) = chunk_lines(blocks);
    let mut seen: BTreeMap<(String, usize), String> = BTreeMap::new();
    let mut failures = Vec::new();
    for (chunk, phase) in &phases {
        let head: String = chunk.chars().take(1).collect();
        let Some(line) = lines.get(chunk).or_else(|| lines.get(&head)) else {
            continue;
        };
        let key = (line.clone(), *phase);
        match seen.get(&key) {
            Some(first) => failures.push((
                format!("{first} and {chunk}"),
                format!("one line, both in phase {phase} (SM6)"),
            )),
            None => {
                seen.insert(key, chunk.clone());
            },
        }
    }
    Ok(failures)
}

/// `PG31b`. Two chunks in one phase whose crates carry a 1.3 edge.
///
/// The rule reads no row: it takes each phase, pairs its line chunks, and asks
/// the section 1.3 edge list whether one crate depends on the other (critic
/// C17-W9).
fn phase_pair_audit(blocks: &Blocks, exempt: &BTreeMap<String, String>) -> anyhow::Result<Counted> {
    let phases = chunk_phases(blocks)?;
    let (_lines, crates) = chunk_lines(blocks);
    let edges = edge_list(blocks)?;
    let mut grouped: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for (chunk, phase) in &phases {
        if chunk.starts_with('M') {
            continue;
        }
        grouped.entry(*phase).or_default().push(chunk.clone());
    }
    let mut failures = Vec::new();
    let mut found = 0_usize;
    for (phase, members) in &grouped {
        let mut members = members.clone();
        members.sort();
        for (index, first) in members.iter().enumerate() {
            for second in members.iter().skip(index.saturating_add(1)) {
                let pair = PhasePair {
                    first,
                    second,
                    phase: *phase,
                };
                pair.audit(&crates, &edges, exempt, &mut found, &mut failures);
            }
        }
    }
    Ok((found, failures))
}

/// Two chunks that share one phase, and the phase they share.
#[derive(Debug, Clone, Copy)]
struct PhasePair<'a> {
    /// The chunk that sorts first.
    first: &'a str,
    /// The chunk that sorts second.
    second: &'a str,
    /// The phase both chunks sit in.
    phase: usize,
}

impl PhasePair<'_> {
    /// Record every crate edge this pair carries, and every one with no row.
    fn audit(
        self,
        crates: &BTreeMap<String, String>,
        edges: &BTreeMap<String, BTreeSet<String>>,
        exempt: &BTreeMap<String, String>,
        found: &mut usize,
        failures: &mut Vec<(String, String)>,
    ) {
        let (Some(one), Some(two)) = (
            crate_of_chunk(self.first, crates),
            crate_of_chunk(self.second, crates),
        ) else {
            return;
        };
        if one == two {
            return;
        }
        let sides = [
            (self.first, self.second, &two),
            (self.second, self.first, &one),
        ];
        for (consumer, producer, other) in sides {
            let home = crate_of_chunk(consumer, crates).unwrap_or_default();
            let linked = edges
                .get(&home)
                .is_some_and(|targets| targets.contains(other));
            if !linked {
                continue;
            }
            *found = found.saturating_add(1);
            if exempt.contains_key(&format!("{consumer} {producer}")) {
                continue;
            }
            failures.push((
                format!("{consumer} and {producer}"),
                format!(
                    "both in phase {}, and {home} depends on {other}; no exemption row \
states why the edge does not bind",
                    self.phase
                ),
            ));
        }
    }
}

/// Every chunk id a section 13.2 line table declares in its own row.
fn chunk_ids(source: &[char]) -> anyhow::Result<BTreeSet<String>> {
    let row = pattern::build(r"(?m)^\| ([A-Z]{1,2}\d{1,2}) \| \d+ \|")?;
    Ok(row
        .find_iter(source)
        .iter()
        .map(|one| one.text(1, source))
        .collect())
}

/// Every chunk row of sections 13.1 and 13.2, as its phase and lock-file claim.
fn chunk_lock_writers(source: &str) -> anyhow::Result<BTreeMap<String, (usize, bool)>> {
    let row = pattern::build(r"^\| ([A-Z]{1,2}\d{1,2}) \| (\d+) \|")?;
    let mut found = BTreeMap::new();
    for line in source.split('\n') {
        let text = pattern::chars(line);
        let Some(one) = row.match_at(&text, 0) else {
            continue;
        };
        let Some(phase) = one.text(2, &text).parse::<usize>().ok() else {
            continue;
        };
        found.insert(one.text(1, &text), (phase, line.contains("Cargo.lock")));
    }
    Ok(found)
}

/// PG35. The per-phase `Cargo.lock` writer sequence is derived, never typed.
///
/// It counts a chunk that DECLARES `Cargo.lock` in its own write scope, so a
/// chunk that writes a member manifest and omits the file from its cell is an
/// SM5 defect this rule cannot see (critic C19-6).
fn lock_sequence_audit(
    source: &str,
    blocks: &Blocks,
) -> anyhow::Result<(Option<Vec<usize>>, Pairs)> {
    let mut phases: Vec<usize> = Vec::new();
    for row in blocks.rows("phase-table") {
        if all_digits(&row.cell(0)) {
            phases.extend(row.cell(0).parse::<usize>().ok());
        }
    }
    let Some(&last_phase) = phases.iter().max() else {
        return Ok((
            None,
            vec![(
                "phase-table".to_owned(),
                "the block states no phase, so no sequence can be derived".to_owned(),
            )],
        ));
    };
    let mut counts = vec![0_usize; last_phase.saturating_add(1)];
    for (_chunk, (phase, writes)) in chunk_lock_writers(source)? {
        if !writes {
            continue;
        }
        if let Some(slot) = counts.get_mut(phase) {
            *slot = slot.saturating_add(1);
        }
    }
    let derived = counts.clone();
    let stated = pattern::build(
        r"Per-phase `Cargo\.lock` writer counts, phases 0 to (\d+):\s*([0-9,\s]+?)\*\*",
    )?;
    let text = pattern::chars(source);
    let Some(one) = stated.find(&text) else {
        return Ok((
            Some(derived),
            vec![(
                "13.3".to_owned(),
                "the document states no per-phase `Cargo.lock` writer sentence".to_owned(),
            )],
        ));
    };
    let mut failures = Vec::new();
    let last = one.text(1, &text).parse::<usize>().unwrap_or(usize::MAX);
    let numbers: Vec<usize> = captures_of(&one.text(2, &text), r"(\d+)")?
        .into_iter()
        .filter_map(|token| token.parse::<usize>().ok())
        .collect();
    if last != last_phase {
        failures.push((
            "13.3".to_owned(),
            format!(
                "the sentence names phases 0 to {last} and the phase table ends at {last_phase}"
            ),
        ));
    }
    if numbers != derived {
        failures.push((
            "13.3".to_owned(),
            format!(
                "the stated sequence is {} and the chunk tables derive {}",
                join_numbers(&numbers),
                join_numbers(&derived)
            ),
        ));
    }
    Ok((Some(derived), failures))
}

/// One number list, as the run prints it.
fn join_numbers(values: &[usize]) -> String {
    values
        .iter()
        .map(usize::to_string)
        .collect::<Vec<String>>()
        .join(", ")
}

/// Every pin the section 13.1 manifest table gives a chunk.
fn manifest_pin_owners(source: &str) -> anyhow::Result<BTreeMap<String, String>> {
    let row = pattern::build(r"^\| (M\d+) \| \d+ \|")?;
    let listed = pattern::build(r"\[workspace\.dependencies\]` only \(([^)]*)\)")?;
    let span = pattern::build(r"`([a-z0-9_-]+)`")?;
    let mut owners = BTreeMap::new();
    for line in source.split('\n') {
        let text = pattern::chars(line);
        let Some(one) = row.match_at(&text, 0) else {
            continue;
        };
        let Some(cell) = listed.find(&text) else {
            continue;
        };
        let pins = pattern::chars(&cell.text(1, &text));
        for pin in span.find_iter(&pins) {
            owners.insert(pin.text(1, &pins), one.text(1, &text));
        }
    }
    Ok(owners)
}

/// Every Appendix B.3 and B.5 pin row, as its pin, its owner, and its appendix.
///
/// B.3's owner is its last cell and B.5's is its fourth, so each appendix is read
/// inside its own heading region and never by a shared column index.
fn appendix_pin_rows(source: &[char]) -> anyhow::Result<Vec<(String, String, String)>> {
    let bare = pattern::build(r"[a-z0-9_-]+")?;
    let mut rows = Vec::new();
    for (label, column, stop) in [("B.3", -1_i64, "B.4"), ("B.5", 3, "Every timeout")] {
        let opened = pattern::build(&format!(r"(?m)^### {} ", pattern::quote(label)))?;
        let Some(found) = opened.find(source) else {
            continue;
        };
        let start = found.start();
        let tail = source.get(start..).unwrap_or(&[]);
        let closed = pattern::build(&format!(r"(?m)^#{{3,4}} {}", pattern::quote(stop)))?;
        let region = closed
            .find(tail)
            .map_or(tail, |end| tail.get(..end.start()).unwrap_or(tail));
        let text: String = region.iter().collect();
        for line in text.split('\n') {
            if !line.starts_with("| `") {
                continue;
            }
            let cells: Vec<String> = trim_set(line.trim(), "|")
                .split('|')
                .map(|cell| cell.trim().to_owned())
                .collect();
            let index = if column < 0 {
                cells.len().checked_sub(1)
            } else {
                usize::try_from(column).ok()
            };
            let ceiling = usize::try_from(column.max(0)).unwrap_or(0);
            if cells.len() <= ceiling {
                continue;
            }
            let pin = strip_ticks(cells.first().map_or("", String::as_str));
            if bare.full_match(&pattern::chars(&pin)).is_none() {
                continue;
            }
            let owner = index
                .and_then(|at| cells.get(at))
                .map(|cell| strip_ticks(cell).trim().to_owned())
                .unwrap_or_default();
            rows.push((pin, owner, label.to_owned()));
        }
    }
    Ok(rows)
}

/// PG36. The two appendix owner columns and the 13.1 pin lists are one set.
///
/// It decides the OWNER of a pin and never the feature set, because a feature
/// list is resolved against a pinned version and this document holds no manifest
/// (critic C19-5).
fn pin_owner_audit(source: &[char]) -> anyhow::Result<Counted> {
    let text: String = source.iter().collect();
    let owners = manifest_pin_owners(&text)?;
    if owners.is_empty() {
        return Ok((
            0,
            vec![(
                "13.1".to_owned(),
                "the manifest table lists no pin, so no owner can be derived".to_owned(),
            )],
        ));
    }
    let rows = appendix_pin_rows(source)?;
    if rows.is_empty() {
        return Ok((
            0,
            vec![(
                "B.3".to_owned(),
                "neither appendix states a pin row, so the rule has no subject".to_owned(),
            )],
        ));
    }
    let mut failures = Vec::new();
    for (pin, stated, appendix) in &rows {
        match owners.get(pin) {
            None => failures.push((
                pin.clone(),
                format!("Appendix {appendix} names owner {stated} and 13.1 pins it nowhere"),
            )),
            Some(owner) if owner != stated => failures.push((
                pin.clone(),
                format!(
                    "Appendix {appendix} names owner {stated} and 13.1 gives the pin to {owner}"
                ),
            )),
            Some(_) => {},
        }
    }
    Ok((rows.len(), failures))
}

/// The number one count word states, or `None`.
fn word_number(word: &str) -> Option<usize> {
    WORD_NUMBERS
        .iter()
        .find(|(name, _value)| *name == word)
        .map(|(_name, value)| *value)
}

/// PG34. The phase table is the source of its own three numbers.
///
/// It reads ONE anchored sentence of the SM6 bullet, the one that opens `The
/// plan is longer and narrower`. A phase count stated anywhere else is outside it
/// (critic C17-W10, C18-W2).
fn tail_phase_audit(source: &[char], blocks: &Blocks) -> anyhow::Result<Vec<(String, String)>> {
    let chunk = pattern::build(r"\b([A-Z]\d{1,2})\b")?;
    let mut widths: Vec<i64> = Vec::new();
    let mut chunks: Vec<Vec<String>> = Vec::new();
    for row in blocks.rows("phase-table") {
        if row.cells().len() < 5 {
            continue;
        }
        widths.push(row.cell(4).trim().parse::<i64>().unwrap_or(-1));
        let cell = pattern::chars(&row.cell(3));
        chunks.push(
            chunk
                .find_iter(&cell)
                .iter()
                .map(|one| one.text(1, &cell))
                .collect(),
        );
    }
    let mut failures = Vec::new();
    let Some(&last_width) = widths.last() else {
        return Ok(vec![(
            "phase-table".to_owned(),
            "the block states no width".to_owned(),
        )]);
    };
    if last_width != 0 {
        failures.push((
            "phase-table".to_owned(),
            format!("the last phase has width {last_width} and not 0"),
        ));
    }
    if let Some(last) = chunks.last().filter(|last| !last.is_empty()) {
        failures.push((
            "phase-table".to_owned(),
            format!(
                "the last phase names {}, and the acceptance run writes no file and \
commits nothing",
                last.join(", ")
            ),
        ));
    }
    let bullet = pattern::build(
        r"(?s)The plan is longer and narrower than revision 5's\.\*\* (.*?)\. The alternative rule",
    )?;
    let Some(found) = bullet.find(source) else {
        failures.push((
            "SM6".to_owned(),
            "the cost bullet that states the phase count is not in this document".to_owned(),
        ));
        return Ok(failures);
    };
    let text = found.text(1, source).to_lowercase();
    let count = pattern::build(r"([a-z-]+) phases replace")?;
    let widest = pattern::build(r"falls from [a-z-]+ to ([a-z-]+)")?;
    let chars = pattern::chars(&text);
    let (Some(count), Some(widest)) = (count.find(&chars), widest.find(&chars)) else {
        failures.push((
            "SM6".to_owned(),
            "the cost bullet states no `<n> phases replace` or no `falls from ... to <n>`"
                .to_owned(),
        ));
        return Ok(failures);
    };
    let stated_count = count.text(1, &chars);
    let stated_widest = widest.text(1, &chars);
    if word_number(&stated_count) != Some(widths.len()) {
        failures.push((
            "SM6".to_owned(),
            format!(
                "the cost bullet states {stated_count} phases and the table holds {}",
                widths.len()
            ),
        ));
    }
    let table_widest = widths.iter().copied().max().unwrap_or(0);
    let wanted = word_number(&stated_widest).and_then(|value| i64::try_from(value).ok());
    if wanted != Some(table_widest) {
        failures.push((
            "SM6".to_owned(),
            format!(
                "the cost bullet states a widest phase of {stated_widest} and the table's \
widest is {table_widest}"
            ),
        ));
    }
    Ok(failures)
}

/// Every `pub fn` name a Rust block of this document declares, with its count.
///
/// It is the THIRD source PG33 needs (critic C18-3). The B.1 Site column and the
/// section 2.3 sentence are both text an author types, so a name can enter both
/// and exist in neither declaration.
fn declared_functions(source: &[char]) -> anyhow::Result<BTreeMap<String, usize>> {
    let header = pattern::build(r"(?m)^\s*pub(?:\(crate\))? fn ([a-z_][A-Za-z0-9_]*)")?;
    let mut found: BTreeMap<String, usize> = BTreeMap::new();
    for (_offset, block) in rust_blocks(source)? {
        let text = pattern::chars(&block);
        for one in header.find_iter(&text) {
            *found.entry(one.text(1, &text)).or_insert(0) += 1;
        }
    }
    Ok(found)
}

/// Every function name more than one Rust block of this document declares.
fn ambiguous_functions(declared: &BTreeMap<String, usize>) -> BTreeSet<String> {
    declared
        .iter()
        .filter(|(_name, times)| **times > 1)
        .map(|(name, _times)| name.clone())
        .collect()
}

/// What PG33 read, or the reason the rule is fail-closed.
#[derive(Debug, Default)]
struct SuppressionReport {
    /// Every site Appendix B.1 gives a reason.
    sites: Vec<String>,
    /// Every name the section 2.3 sentence lists.
    listed: Vec<String>,
    /// Every failure the three sets produced.
    failures: Vec<(String, String)>,
}

/// PG33. Three sets are one set: B.1, the 2.3 list, and the declarations.
///
/// It reads the one section 2.3 sentence anchored on the words `functions carry
/// a suppression`, so a suppression in another crate is outside it. It also
/// compares the sentence's own COUNT WORD with the length of the list (critic
/// C17-2, C18-3, C18-W3).
fn suppression_audit(
    source: &[char],
    blocks: &Blocks,
) -> anyhow::Result<Result<SuppressionReport, String>> {
    let mut sites = Vec::new();
    for row in blocks.rows("b1-convert") {
        if !row.cells().is_empty() {
            sites.extend(cell_names(&row.cell(0))?);
        }
    }
    let sentence = pattern::build(r"(?s)\b([A-Za-z-]+) functions carry a suppression: (.*?)\.\s")?;
    let Some(found) = sentence.find(source) else {
        return Ok(Err(
            "section 2.3 states no list of the functions that carry a suppression".to_owned(),
        ));
    };
    let listed = captures_of(&found.text(2, source), r"`([A-Za-z_][A-Za-z0-9_]*)`")?;
    let declared = declared_functions(source)?;
    let site_set: BTreeSet<String> = sites.iter().cloned().collect();
    let listed_set: BTreeSet<String> = listed.iter().cloned().collect();
    let mut failures = Vec::new();
    for name in site_set.difference(&listed_set) {
        failures.push((
            name.clone(),
            "Appendix B.1 gives it a reason and section 2.3 omits it".to_owned(),
        ));
    }
    for name in listed_set.difference(&site_set) {
        failures.push((
            name.clone(),
            "section 2.3 lists it and Appendix B.1 gives it no reason".to_owned(),
        ));
    }
    let both: BTreeSet<String> = site_set.union(&listed_set).cloned().collect();
    for name in &both {
        if !declared.contains_key(name) {
            failures.push((
                name.clone(),
                "the suppression register names it and no Rust block declares the function"
                    .to_owned(),
            ));
        }
    }
    let ambiguous = ambiguous_functions(&declared);
    for name in both.intersection(&ambiguous) {
        failures.push((
            name.clone(),
            format!(
                "this document declares {} functions of that name, so the suppression \
register resolves to no one site",
                declared.get(name).copied().unwrap_or(0)
            ),
        ));
    }
    let word = found.text(1, source).to_lowercase();
    match word_number(&word) {
        None => failures.push((
            word,
            "the count word of the section 2.3 list is not a number word".to_owned(),
        )),
        Some(stated) if stated != listed.len() => failures.push((
            word.clone(),
            format!(
                "the section 2.3 list states {word} and holds {} names",
                listed.len()
            ),
        )),
        Some(_) => {},
    }
    failures.extend(b1_count_sentences(source, blocks)?);
    Ok(Ok(SuppressionReport {
        sites,
        listed,
        failures,
    }))
}

/// The four Appendix B.1 count sentences, each against its own block.
///
/// Revision 19 read the section 2.3 count word and left these four unread, three
/// lines from the tables the same rule holds (critic C19-W19).
fn b1_count_sentences(source: &[char], blocks: &Blocks) -> anyhow::Result<Vec<(String, String)>> {
    let mut failures = Vec::new();
    for (phrase, block_id) in B1_COUNT_SENTENCES {
        let sentence = pattern::build(&format!(r"\b([A-Za-z]+) {}", pattern::quote(phrase)))?;
        let Some(found) = sentence.find(source) else {
            failures.push((
                phrase.to_owned(),
                "Appendix B.1 states no count sentence for it".to_owned(),
            ));
            continue;
        };
        let word = found.text(1, source);
        let rows = blocks.rows(block_id).len();
        match word_number(&word.to_lowercase()) {
            None => failures.push((
                word.clone(),
                format!("the count word of `{phrase}` is not a number word"),
            )),
            Some(said) if said != rows => failures.push((
                phrase.to_owned(),
                format!(
                    "Appendix B.1 states {} and the `{block_id}` block holds {rows} rows",
                    word.to_lowercase()
                ),
            )),
            Some(_) => {},
        }
    }
    Ok(failures)
}

/// Whether one word sits in a text between two word boundaries.
///
/// It is the `\b<word>\b` test, without a pattern compile, because PG37 runs it
/// over every line of every fenced block for every budget row.
fn contains_word(line: &str, word: &str) -> bool {
    let text: Vec<char> = line.chars().collect();
    let needle: Vec<char> = word.chars().collect();
    let bound = |before: Option<char>, after: Option<char>| {
        before.is_some_and(pattern::is_word) != after.is_some_and(pattern::is_word)
    };
    if needle.is_empty() {
        return (0..=text.len()).any(|at| {
            bound(
                at.checked_sub(1).and_then(|index| text.get(index)).copied(),
                text.get(at).copied(),
            )
        });
    }
    let last = text.len().saturating_sub(needle.len());
    for at in 0..=last {
        if text.get(at..at.saturating_add(needle.len())) != Some(needle.as_slice()) {
            continue;
        }
        let head = bound(
            at.checked_sub(1).and_then(|index| text.get(index)).copied(),
            needle.first().copied(),
        );
        let end = at.saturating_add(needle.len());
        let tail = bound(needle.last().copied(), text.get(end).copied());
        if head && tail {
            return true;
        }
    }
    false
}

/// The one declaration line a budget citation belongs to.
///
/// A `///` doc comment reads DOWN to the first line that is not a comment. A
/// `//` trailing comment reads UP. So each citation resolves to exactly one
/// declaration and never to a neighbour of it (critic C21-W2).
fn adjacent_declaration(lines: &[&str], index: usize) -> String {
    let Some(line) = lines.get(index) else {
        return String::new();
    };
    let text = line.trim();
    let down = text.starts_with("///") || text.starts_with("//!");
    if !down && !text.starts_with("//") {
        return String::new();
    }
    let mut position = if down {
        index.checked_add(1)
    } else {
        index.checked_sub(1)
    };
    while let Some(at) = position {
        let Some(found) = lines.get(at) else {
            break;
        };
        let candidate = found.trim();
        if !candidate.is_empty() && !candidate.starts_with("//") {
            return (*found).to_owned();
        }
        position = if down {
            at.checked_add(1)
        } else {
            at.checked_sub(1)
        };
    }
    String::new()
}

/// What PG37 decided over the section 1.6 budget table.
#[derive(Debug, Default)]
struct ValueReport {
    /// How many budget rows the rule decided.
    decided: usize,
    /// Every budget whose citing declaration line refutes its value.
    failures: Vec<(String, String)>,
    /// Every budget the rule could not decide.
    skipped: Vec<String>,
}

/// PG37. A budget VALUE that a citing declaration line refutes.
///
/// The oracle is the citing line and the line AFTER it, and never the line
/// before. A budget whose value cell opens with an integer and whose citing line
/// declares a capacity is in scope (critic C19-W9, C21-W2, N21-3).
fn budget_value_audit(source: &[char], blocks: &Blocks) -> anyhow::Result<ValueReport> {
    let fence = pattern::build(r"(?s)```(?:rust|text)\n(.*?)```")?;
    let opens = pattern::build(r"^([0-9][0-9_]*)\b")?;
    let capacity = pattern::build(
        r"\[\s*[A-Za-z_][A-Za-z0-9_:<>, ]*;\s*([0-9][0-9_]*)\s*\]|ArrayVec<[^>]*,\s*([0-9][0-9_]*)\s*>|:\s*usize\s*=\s*([0-9][0-9_]*)",
    )?;
    let bodies: Vec<String> = fence
        .find_iter(source)
        .iter()
        .map(|one| one.text(1, source))
        .collect();
    let mut report = ValueReport::default();
    for row in blocks.rows("budget-table") {
        if row.cells().len() < 4 {
            continue;
        }
        let budget = strip_ticks(row.cell(0).trim());
        let value = row.cell(1).trim().to_owned();
        let chars = pattern::chars(&value);
        let Some(found) = opens.match_at(&chars, 0) else {
            report.skipped.push(budget);
            continue;
        };
        let Some(stated) = digits_value(&found.text(1, &chars)) else {
            report.skipped.push(budget);
            continue;
        };
        let mut capacities: BTreeSet<usize> = BTreeSet::new();
        for body in &bodies {
            capacities.extend(cited_capacities(&capacity, body, &budget));
        }
        if capacities.is_empty() {
            report.skipped.push(budget);
            continue;
        }
        report.decided = report.decided.saturating_add(1);
        if !capacities.contains(&stated) {
            report.failures.push((
                budget,
                format!(
                    "the row states {stated} and the declaration line that cites it writes {}",
                    join_numbers(&capacities.into_iter().collect::<Vec<usize>>())
                ),
            ));
        }
    }
    Ok(report)
}

/// Every capacity the lines of one fenced body cite for a budget id.
fn cited_capacities(capacity: &pattern::Regex, body: &str, budget: &str) -> BTreeSet<usize> {
    let lines: Vec<&str> = body.split('\n').collect();
    let mut found = BTreeSet::new();
    for (index, line) in lines.iter().enumerate() {
        if !contains_word(line, budget) {
            continue;
        }
        let window = format!("{line}\n{}", adjacent_declaration(&lines, index));
        found.extend(stated_capacities(capacity, &window));
    }
    found
}

/// Every capacity one citing window declares.
///
/// Two shapes carry a capacity: a `const NAME: usize = <n>;` and a
/// fixed-capacity container spelled `[T; <n>]`, `ArrayVec<T, <n>>`, or
/// `SmallVec<[T; <n>]>`.
fn stated_capacities(capacity: &pattern::Regex, window: &str) -> BTreeSet<usize> {
    let text = pattern::chars(window);
    let mut found = BTreeSet::new();
    for site in capacity.find_iter(&text) {
        let digits = (1..=3_usize).filter_map(|group| site.group(group, &text));
        found.extend(digits.filter_map(|one| digits_value(&one)));
    }
    found
}

/// Every markdown table of this document, outside every fenced block.
fn table_rows_of(source: &str) -> Vec<Vec<(usize, String)>> {
    let mut fenced = false;
    let mut found = Vec::new();
    let mut current: Option<Vec<(usize, String)>> = None;
    for (index, line) in source.split('\n').enumerate() {
        let number = index.saturating_add(1);
        let stripped = line.trim();
        if stripped.starts_with("```") || stripped.starts_with("~~~") {
            fenced = !fenced;
            continue;
        }
        if fenced {
            continue;
        }
        if stripped.starts_with('|') && stripped.ends_with('|') {
            current
                .get_or_insert_with(Vec::new)
                .push((number, stripped.to_owned()));
            continue;
        }
        if let Some(table) = current.take() {
            found.push(table);
        }
    }
    if let Some(table) = current {
        found.push(table);
    }
    found
}

/// PG38. A table row whose cell count differs from its own header.
///
/// A renderer keeps the first `n` cells of a ragged row and DROPS the rest, so
/// an unescaped `|` inside a backtick span silently truncates the row (critic
/// C21-5). The rule decides the CELL COUNT of a row and never the content.
fn ragged_row_audit(source: &str) -> anyhow::Result<Counted> {
    let ruler = pattern::build(r"[|:\- ]+")?;
    let mut checked = 0_usize;
    let mut failures = Vec::new();
    for table in table_rows_of(source) {
        if table.len() < 2 {
            continue;
        }
        let Some((_head_number, head_row)) = table.first() else {
            continue;
        };
        let width = cell_count(head_row)?;
        for (number, row) in table.iter().skip(1) {
            if ruler.full_match(&pattern::chars(row)).is_some() {
                continue;
            }
            checked = checked.saturating_add(1);
            let found = cell_count(row)?;
            if found == width {
                continue;
            }
            let cells = split_row(row)?;
            let head = cells.first().cloned().unwrap_or_default();
            let label: String = trim_set(&head, "`* ").chars().take(24).collect();
            let label = if label.is_empty() {
                format!("line {number}")
            } else {
                label
            };
            failures.push((
                label,
                format!(
                    "the row holds {found} cells and its header holds {width}; a renderer \
drops the extra cells and truncates the row (PG38)"
                ),
            ));
        }
    }
    Ok((checked, failures))
}

/// Every id one section 1.7 cell names, with its ranges expanded.
///
/// `PG1 to PG37` is a numeric range, `PG26b to PG26f` is a letter range over one
/// number, and every other token is one id.
fn expand_index_ids(cell: &str) -> anyhow::Result<BTreeSet<String>> {
    let range = pattern::build(r"\b([A-Z]{2}\d+[a-z]?)\s+to\s+([A-Z]{2}\d+[a-z]?)\b")?;
    let parts = pattern::build(r"([A-Z]{2})(\d+)([a-z]?)")?;
    let single = pattern::build(r"\b([A-Z]{2}\d+[a-z]?)\b")?;
    let mut found = BTreeSet::new();
    let mut text = cell.replace('\u{a0}', " ");
    let chars = pattern::chars(&text);
    let pairs: Vec<(String, String)> = range
        .find_iter(&chars)
        .iter()
        .map(|one| (one.text(1, &chars), one.text(2, &chars)))
        .collect();
    for (opening, closing) in pairs {
        let first = pattern::chars(&opening);
        let last = pattern::chars(&closing);
        let (Some(one), Some(two)) = (parts.full_match(&first), parts.full_match(&last)) else {
            continue;
        };
        let prefix = one.text(1, &first);
        if prefix != two.text(1, &last) {
            continue;
        }
        let first_letter = one.text(3, &first);
        let last_letter = two.text(3, &last);
        let first_number = one.text(2, &first);
        let last_number = two.text(2, &last);
        if !first_letter.is_empty() && !last_letter.is_empty() && first_number == last_number {
            let (Some(low), Some(high)) = (
                first_letter.chars().next().map(u32::from),
                last_letter.chars().next().map(u32::from),
            ) else {
                continue;
            };
            let letters = (low..=high).filter_map(char::from_u32);
            found.extend(letters.map(|letter| format!("{prefix}{first_number}{letter}")));
        } else {
            let (Some(low), Some(high)) = (
                first_number.parse::<usize>().ok(),
                last_number.parse::<usize>().ok(),
            ) else {
                continue;
            };
            for value in low..=high {
                found.insert(format!("{prefix}{value}"));
            }
        }
        text = text.replace(&format!("{opening} to {closing}"), " ");
    }
    let rest = pattern::chars(&text);
    for one in single.find_iter(&rest) {
        found.insert(one.text(1, &rest));
    }
    Ok(found)
}

/// PG39. Section 1.7 and the live rule set are one set.
///
/// The rule expands every range of the index, takes the PG and CG ids from the
/// prototypes and the PP and CP ids from the probe table, and fails on an id in
/// one set and not the other, in both directions (critic C21-W7).
fn rule_index_audit(
    source: &[char],
    rule_ids: &BTreeSet<String>,
    probe_ids: &[String],
) -> anyhow::Result<Counted> {
    let table = pattern::build(
        r"(?m)^\| Id \| Rule, in three words \| Stated in \|\n\|[-| ]+\|\n((?:\|.*\n)+)",
    )?;
    let Some(found) = table.find(source) else {
        return Ok((
            0,
            vec![(
                "<1.7>".to_owned(),
                "the rule index table is absent; the rule is fail-closed".to_owned(),
            )],
        ));
    };
    let mut indexed = BTreeSet::new();
    for line in found.text(1, source).split('\n') {
        let parts: Vec<String> = line.split('|').map(|cell| cell.trim().to_owned()).collect();
        let cells = parts
            .get(1..parts.len().saturating_sub(1))
            .unwrap_or(&[])
            .to_vec();
        if cells.len() < 3 {
            continue;
        }
        indexed.extend(expand_index_ids(cells.first().map_or("", String::as_str))?);
    }
    let mut live: BTreeSet<String> = rule_ids.clone();
    live.extend(probe_ids.iter().cloned());
    let mut failures = Vec::new();
    for family in INDEX_FAMILIES {
        let shape = pattern::build(&format!(r"{family}\d+[a-z]?"))?;
        let stated: BTreeSet<String> = indexed
            .iter()
            .filter(|name| name.starts_with(family))
            .cloned()
            .collect();
        let actual: BTreeSet<String> = live
            .iter()
            .filter(|name| shape.full_match(&pattern::chars(name)).is_some())
            .cloned()
            .collect();
        for name in actual.difference(&stated) {
            failures.push((
                name.clone(),
                "the rule set holds it and section 1.7 omits it".to_owned(),
            ));
        }
        for name in stated.difference(&actual) {
            failures.push((
                name.clone(),
                "section 1.7 names it and the rule set holds no such id".to_owned(),
            ));
        }
    }
    Ok((indexed.len(), failures))
}

/// PG40. A chunk writes only into the crate its own LINE owns.
///
/// The rule reads a path that opens `crates/`, so a chunk that writes `tools/`,
/// `scripts/` or a workflow file is outside it. The M chunks are outside it too,
/// because a manifest row names several crates by design (critic C22I-W3).
fn chunk_crate_audit(source: &[char], blocks: &Blocks) -> anyhow::Result<Counted> {
    let mut owners = BTreeMap::new();
    for row in blocks.rows("line-map") {
        let parts = row.tokens();
        if let (Some(line), Some(crate_name)) = (parts.first(), parts.get(1)) {
            owners.insert(line.clone(), crate_name.clone());
        }
    }
    if owners.is_empty() {
        return Ok((
            0,
            vec![(
                "<line-map>".to_owned(),
                "the line map holds no row, so PG40 has no owner set".to_owned(),
            )],
        ));
    }
    let row = pattern::build(r"(?m)^\| ([A-Z]+\d*) \| (\d+) \|(.*)$")?;
    let letters = pattern::build(r"[A-Z]+")?;
    let manifest = pattern::build(r"M\d*")?;
    let path = pattern::build(r"crates/([a-z0-9-]+)/")?;
    let mut checked = 0_usize;
    let mut failures = Vec::new();
    for one in row.find_iter(source) {
        if cell_count(&one.text(0, source))? != 5 {
            continue;
        }
        let chunk = one.text(1, source);
        let rest = one.text(3, source);
        let name = pattern::chars(&chunk);
        let head = letters
            .match_at(&name, 0)
            .map(|found| found.text(0, &name))
            .unwrap_or_default();
        let Some(owner) = owners.get(&chunk).or_else(|| owners.get(&head)) else {
            if manifest.full_match(&name).is_none() {
                failures.push((
                    chunk,
                    "the `line-map` block carries no line for this chunk id, so no crate \
owns its write scope (PG40)"
                        .to_owned(),
                ));
            }
            continue;
        };
        checked = checked.saturating_add(1);
        let text = pattern::chars(&rest);
        let written: BTreeSet<String> = path
            .find_iter(&text)
            .iter()
            .map(|found| found.text(1, &text))
            .collect();
        for found in written {
            if found != *owner {
                failures.push((
                    chunk.clone(),
                    format!(
                        "the row writes under `crates/{found}/` and line `{head}` owns \
`{owner}` (PG40)"
                    ),
                ));
            }
        }
    }
    Ok((checked, failures))
}

/// Every citation label of this document, as its offset and its label (PG30).
///
/// A label is what a section 1.6 `Used by` cell writes: `5.5`, `14`, `B.5`,
/// `C.13`, or `Appendix A`. A `####` heading carries no label of its own.
fn label_marks(source: &[char]) -> anyhow::Result<Vec<SectionMark>> {
    let heading = pattern::build(r"(?m)^#{2,3} (.+)$")?;
    let head = pattern::build(r"([0-9]+\.[0-9]+[a-z]?|[0-9]+|[BC]\.[0-9]+)[.:]? ")?;
    let mut marks = Vec::new();
    for one in heading.find_iter(source) {
        let title = one.text(1, source).trim().to_owned();
        let label = if title.starts_with("Appendix A") {
            Some("Appendix A".to_owned())
        } else {
            let text = pattern::chars(&title);
            head.match_at(&text, 0).map(|found| found.text(1, &text))
        };
        if let Some(label) = label {
            marks.push((one.start(), label));
        }
    }
    Ok(marks)
}

/// Every Appendix C section that registers a closure block (PG30).
///
/// The set is read from the document, exactly as the rule set is read from the
/// prototypes, so a closure section added with no edit here cannot put every `B`
/// id its rows quote back into PG30's citation set (critic N21-7).
fn closure_sections(source: &[char]) -> anyhow::Result<BTreeSet<String>> {
    let heading =
        pattern::build(r"(?ms)^###\s+(C\.\d+)\b(?:(?!^###\s).)*?<!--\s*GUARD BLOCK id=closure-")?;
    Ok(heading
        .find_iter(source)
        .iter()
        .map(|one| one.text(1, source))
        .collect())
}

/// PG30. Every `B` citation against the section 1.6 `Used by` column.
///
/// It reads this document alone, it skips the budget table itself, and it skips
/// every closure appendix of Appendix C, because a closure row records the
/// finding that created a budget and never uses its value (critic C-1, C18-N4).
fn used_by_audit(source: &[char]) -> anyhow::Result<Counted> {
    let table = pattern::build(
        r"(?m)^\| Id \| Value \| What it bounds \| Used by \|\n\|[-| ]+\|\n((?:\|.*\n)+)",
    )?;
    let Some(found) = table.find(source) else {
        return Ok((
            0,
            vec![("<1.6>".to_owned(), "the budget table is absent".to_owned())],
        ));
    };
    let skipped = closure_sections(source)?;
    if skipped.is_empty() {
        return Ok((
            0,
            vec![(
                "<C>".to_owned(),
                "Appendix C registers no closure block, so PG30 has no skip set".to_owned(),
            )],
        ));
    }
    let cells = budget_cells(&found.text(1, source))?;
    let mut masked: Vec<char> = source.to_vec();
    for slot in masked
        .get_mut(found.start()..found.end())
        .unwrap_or(&mut [])
    {
        *slot = ' ';
    }
    let marks = label_marks(source)?;
    let citation = pattern::build(r"(?<![\w.])B(\d+)(?![\w])")?;
    let mut cited: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for one in citation.find_iter(&masked) {
        let Some(label) = section_at(&marks, one.start()) else {
            continue;
        };
        if label == "1.7" || skipped.contains(&label) {
            continue;
        }
        cited
            .entry(format!("B{}", one.text(1, &masked)))
            .or_default()
            .insert(label);
    }
    let mut order: Vec<&String> = cells.keys().collect();
    order.sort_by_key(|name| {
        name.chars()
            .skip(1)
            .collect::<String>()
            .parse::<usize>()
            .unwrap_or(0)
    });
    let mut failures = Vec::new();
    let empty = BTreeSet::new();
    for budget in order {
        let stated = cells.get(budget).unwrap_or(&empty);
        let real = cited.get(budget).unwrap_or(&empty);
        for label in real.difference(stated) {
            failures.push((
                budget.clone(),
                format!("section {label} cites it and the `Used by` cell omits it"),
            ));
        }
        for label in stated.difference(real) {
            failures.push((
                budget.clone(),
                format!("the `Used by` cell names section {label} and it cites nothing"),
            ));
        }
    }
    for budget in cited.keys() {
        if !cells.contains_key(budget) {
            failures.push((
                budget.clone(),
                "no section 1.6 row declares this id".to_owned(),
            ));
        }
    }
    Ok((cells.len(), failures))
}

/// The labels each section 1.6 `Used by` cell names, by budget id.
fn budget_cells(body: &str) -> anyhow::Result<BTreeMap<String, BTreeSet<String>>> {
    let identifier = pattern::build(r"B\d+")?;
    let adr = pattern::build(r"ADR \d+")?;
    let labelled = pattern::build(r"(?<![\w.])(\d+\.\d+[a-z]?|[BC]\.\d+)(?![\w.])")?;
    let bare = pattern::build(r"(?<![\w.])(\d+)(?![\w.])")?;
    let mut cells = BTreeMap::new();
    for line in body.split('\n') {
        let parts: Vec<String> = line.split('|').map(|cell| cell.trim().to_owned()).collect();
        let inner = parts
            .get(1..parts.len().saturating_sub(1))
            .unwrap_or(&[])
            .to_vec();
        if inner.len() < 4 {
            continue;
        }
        let budget = inner.first().cloned().unwrap_or_default();
        if identifier.full_match(&pattern::chars(&budget)).is_none() {
            continue;
        }
        let used = inner.get(3).cloned().unwrap_or_default();
        let cell = adr.replace_all(&pattern::chars(&used), &mut |_one, _text| String::new());
        let chars = pattern::chars(&cell);
        let mut labels: BTreeSet<String> = labelled
            .find_iter(&chars)
            .iter()
            .map(|one| one.text(1, &chars))
            .collect();
        labels.extend(bare.find_iter(&chars).iter().map(|one| one.text(1, &chars)));
        if cell.contains("Appendix A") {
            labels.insert("Appendix A".to_owned());
        }
        cells.insert(budget, labels);
    }
    Ok(cells)
}

/// The section 1.9 membership block, as the kind each block runs.
///
/// A block with no row, a row over an id the register does not hold, and a kind
/// this guard does not implement are each a failure, so the register and this
/// block are one set (WR-18).
fn block_member_kinds(blocks: &Blocks) -> (BTreeMap<String, String>, Pairs) {
    let mut kinds: BTreeMap<String, String> = BTreeMap::new();
    let mut bad = Vec::new();
    for row in blocks.rows("block-members") {
        let line = row.line();
        let parts = row.tokens();
        let (Some(block_id), Some(kind)) = (parts.first(), parts.get(1)) else {
            bad.push((
                line.trim().to_owned(),
                "the row is not one block id and one kind".to_owned(),
            ));
            continue;
        };
        if !registered(block_id) {
            bad.push((block_id.clone(), "the register does not hold it".to_owned()));
            continue;
        }
        if !MEMBER_KINDS.contains(&kind.as_str()) {
            bad.push((
                block_id.clone(),
                format!("the kind `{kind}` is not one this guard runs"),
            ));
            continue;
        }
        if kinds.contains_key(block_id) {
            bad.push((
                block_id.clone(),
                "the block carries two membership rows".to_owned(),
            ));
            continue;
        }
        kinds.insert(block_id.clone(), kind.clone());
    }
    let mut absent: Vec<&str> = DATA_BLOCKS
        .iter()
        .map(|spec| spec.id)
        .filter(|id| !kinds.contains_key(*id))
        .collect();
    absent.sort_unstable();
    for block_id in absent {
        bad.push((block_id.to_owned(), "no membership row names it".to_owned()));
    }
    (kinds, bad)
}

/// Every set the membership rules read a row against.
#[derive(Debug)]
struct MemberContext<'a> {
    /// The whole document, for the `cited-elsewhere` rule.
    source: &'a str,
    /// Every crate section 1.2 carries.
    crates: BTreeSet<String>,
    /// Every crate the section 1.3 edge list names.
    edge_crates: BTreeSet<String>,
    /// Every third-party crate a section 1.2 dependency cell names.
    third_party: BTreeSet<String>,
    /// Every type a Rust block of this document declares.
    decls: BTreeSet<String>,
    /// Every function a Rust block of this document declares.
    functions: BTreeSet<String>,
    /// Every enum arm this document declares.
    arms: &'a BTreeMap<String, Vec<(String, String)>>,
    /// Every name the section 1.5 table places.
    owned: BTreeSet<String>,
    /// Every framework name section 1.3 declares.
    framework: BTreeSet<String>,
    /// Every external name section 1.9 decides.
    externals: BTreeSet<String>,
    /// Every name the section 1.9 external-path block maps.
    external_paths: BTreeSet<String>,
    /// Every constant section 1.6 declares.
    constants: BTreeSet<String>,
    /// Every rule id the prototypes implement.
    rules: BTreeSet<String>,
    /// Every audio-owned root the section 5.7 block names.
    audio_roots: BTreeSet<String>,
    /// Every field each declaration states.
    fields: BTreeMap<String, BTreeSet<String>>,
    /// Every chunk id section 13.2 declares.
    chunks: BTreeSet<String>,
    /// Every primitive the trait block names.
    primitive_traits: BTreeSet<String>,
    /// Every primitive the size block names.
    primitive_sizes: BTreeSet<String>,
    /// Every name an external row may refer to.
    external_universe: BTreeSet<String>,
    /// Every name a recorded size row may refer to.
    size_universe: BTreeSet<String>,
}

/// One membership failure: the row it names and the reason.
type MemberFail = (String, String);

/// PG27. Every row of every registered block names a referent (WR-18).
///
/// DR7 closes the DELETION class and leaves the ADDITION class open, so a row
/// must name an entity that exists elsewhere in this document or in the external
/// table. The rule decides the referent of a row and never the row's meaning.
fn membership_audit(
    blocks: &Blocks,
    kinds: &BTreeMap<String, String>,
    context: &MemberContext<'_>,
) -> anyhow::Result<(usize, Triples)> {
    let mut checked = 0_usize;
    let mut failures = Vec::new();
    for (block_id, kind) in kinds {
        for row in blocks.rows(block_id) {
            checked = checked.saturating_add(1);
            for (subject, reason) in member_row(context, block_id, kind, row)? {
                failures.push((block_id.clone(), subject, reason));
            }
        }
    }
    Ok((checked, failures))
}

/// What one membership rule says about one row.
fn member_row(
    context: &MemberContext<'_>,
    block_id: &str,
    kind: &str,
    row: &Row,
) -> anyhow::Result<Vec<MemberFail>> {
    if let Some(found) = member_crate_row(context, block_id, kind, row)? {
        return Ok(found);
    }
    if let Some(found) = member_name_row(context, block_id, kind, row)? {
        return Ok(found);
    }
    if let Some(found) = member_id_row(context, kind, row)? {
        return Ok(found);
    }
    if let Some(found) = member_site_row(context, kind, row)? {
        return Ok(found);
    }
    Ok(Vec::new())
}

/// Every membership rule that reads a crate name.
fn member_crate_row(
    context: &MemberContext<'_>,
    block_id: &str,
    kind: &str,
    row: &Row,
) -> anyhow::Result<Option<Vec<MemberFail>>> {
    let text = row.text();
    let tokens = row.tokens();
    let mut failures = Vec::new();
    match kind {
        "crate-name" => {
            let universe = if block_id == "crate-table" {
                &context.edge_crates
            } else {
                &context.crates
            };
            let mut found: BTreeSet<String> = if row.cells().is_empty() {
                BTreeSet::new()
            } else {
                captures_of(&row.cell(0), r"`([A-Za-z0-9_/-]+)`")?
                    .into_iter()
                    .collect()
            };
            if found.is_empty() {
                let word = pattern::build(r"[A-Za-z][A-Za-z0-9_-]*")?;
                let chars = pattern::chars(&text);
                found = word
                    .find_iter(&chars)
                    .iter()
                    .map(|one| one.text(0, &chars))
                    .filter(|token| token.starts_with("duet"))
                    .collect();
            }
            for name in found {
                if !universe.contains(&normalize_crate(&name)) {
                    failures.push((
                        text.clone(),
                        format!("`{name}` is no crate of the other block"),
                    ));
                }
            }
        },
        "ownership-row" => {
            let crate_name = if row.cells().is_empty() {
                String::new()
            } else {
                normalize_crate(&strip_ticks(&row.cell(0)))
            };
            if !context.crates.contains(&crate_name) {
                failures.push((text, format!("`{crate_name}` is no crate of section 1.2")));
            }
        },
        "const-crate" => {
            let head = pattern::build(r"\s*// (duet-[a-z]+)")?;
            let chars = pattern::chars(&text);
            let named = head
                .match_at(&chars, 0)
                .map(|one| one.text(1, &chars))
                .filter(|name| !context.crates.contains(&normalize_crate(name)));
            if let Some(name) = named {
                failures.push((text, format!("`{name}` is no crate of section 1.2")));
            }
        },
        "limit-row" => {
            let Some(first) = tokens.first() else {
                return Ok(Some(failures));
            };
            if !context.constants.contains(first) {
                failures.push((
                    text.clone(),
                    format!("`{first}` is no constant of section 1.6"),
                ));
            }
            let absent = tokens
                .iter()
                .skip(1)
                .filter(|name| !context.crates.contains(&normalize_crate(name)));
            for name in absent {
                failures.push((text.clone(), format!("`{name}` is no crate of section 1.2")));
            }
        },
        "line-owner" => {
            let holds = tokens
                .get(1)
                .is_some_and(|name| context.crates.contains(&normalize_crate(name)));
            if tokens.len() < 2 || !holds {
                failures.push((text, "the row names no crate of section 1.2".to_owned()));
            }
        },
        "site-crate" | "site-path" => {
            let site = if row.cells().is_empty() {
                text
            } else {
                strip_ticks(&row.cell(0))
            };
            let crate_name = normalize_crate(site.split("::").next().unwrap_or(&site));
            if !context.crates.contains(&crate_name) {
                failures.push((site, format!("`{crate_name}` is no crate of section 1.2")));
            } else if kind == "site-path" {
                let leaf = site.rsplit("::").next().unwrap_or(&site).to_owned();
                if !context.decls.contains(&leaf) {
                    failures.push((site, format!("`{leaf}` is no declared type")));
                }
            }
        },
        _ => return Ok(None),
    }
    Ok(Some(failures))
}

/// Every membership rule that reads a type or crate name from a token list.
fn member_name_row(
    context: &MemberContext<'_>,
    block_id: &str,
    kind: &str,
    row: &Row,
) -> anyhow::Result<Option<Vec<MemberFail>>> {
    let text = row.text();
    let tokens = row.tokens();
    let head = tokens.first().cloned().unwrap_or_default();
    let mut failures = Vec::new();
    match kind {
        "pin-name" => {
            if tokens.is_empty() || !context.third_party.contains(&strip_ticks(&head)) {
                failures.push((
                    text,
                    "the crate is in no section 1.2 dependency cell".to_owned(),
                ));
            }
        },
        "map-crate" => {
            let holds = tokens
                .get(1)
                .is_some_and(|name| context.third_party.contains(&strip_ticks(name)));
            if tokens.len() < 2 || !holds {
                failures.push((
                    text,
                    "the crate is in no section 1.2 dependency cell".to_owned(),
                ));
            }
        },
        "not-declared" => {
            for token in &tokens {
                if context.decls.contains(token) || context.owned.contains(token) {
                    failures.push((
                        text.clone(),
                        format!("`{token}` is a type this document declares"),
                    ));
                }
            }
        },
        "drop-name" => {
            for token in &tokens {
                if !context.decls.contains(token) {
                    continue;
                }
                if context.external_paths.contains(token) && context.owned.contains(token) {
                    continue;
                }
                failures.push((
                    token.clone(),
                    "the drop list carries a name this document declares and no external \
block maps"
                        .to_owned(),
                ));
            }
        },
        "declared-name" => {
            let names = if row.cells().is_empty() {
                tokens
            } else {
                cell_names(&row.cell(0))?
            };
            for name in names {
                if !context.decls.contains(&name) {
                    failures.push((
                        name,
                        "no Rust block of this document declares it".to_owned(),
                    ));
                }
            }
        },
        "first-declared" => {
            if !context.decls.contains(&head) {
                failures.push((
                    head,
                    "no Rust block of this document declares it".to_owned(),
                ));
            }
        },
        "impl-site" => {
            if !context.decls.contains(&head) {
                failures.push((text, format!("`{head}` is no declaration of this document")));
            }
        },
        _ => return member_type_row(context, block_id, kind, row),
    }
    Ok(Some(failures))
}

/// Every membership rule that reads a type name or a size expression.
fn member_type_row(
    context: &MemberContext<'_>,
    block_id: &str,
    kind: &str,
    row: &Row,
) -> anyhow::Result<Option<Vec<MemberFail>>> {
    let text = row.text();
    let tokens = row.tokens();
    let head = tokens.first().cloned().unwrap_or_default();
    let mut failures = Vec::new();
    match kind {
        "verdict-name" => {
            let names = if row.cells().is_empty() {
                Vec::new()
            } else {
                cell_names(&row.cell(0))?
            };
            for name in names {
                if context.owned.contains(&name) {
                    failures.push((
                        name,
                        "section 1.5 places this name, so no row decides it".to_owned(),
                    ));
                }
            }
        },
        "external-name" => {
            if !tokens.is_empty() && !context.external_universe.contains(&head) {
                failures.push((text, format!("`{head}` is named in no other block")));
            }
        },
        "expr-head" => {
            let found = if tokens.is_empty() {
                None
            } else {
                expr_head(&head)
            };
            let absent = found.filter(|name| !context.size_universe.contains(name));
            if let Some(name) = absent {
                failures.push((text, format!("`{name}` is no type this document knows")));
            }
        },
        "primitive-name" => {
            if block_id == "primitive-sizes" {
                if !context.primitive_traits.contains(&head) {
                    failures.push((text, format!("`{head}` is in no primitive trait set")));
                }
            } else if text.contains("Copy") && !context.primitive_sizes.contains(&head) {
                failures.push((
                    text,
                    format!("`{head}` is a sized primitive with no size row"),
                ));
            }
        },
        "unknown-name" => {
            let names = if row.cells().len() > 2 {
                cell_names(&row.cell(2))?
            } else {
                Vec::new()
            };
            for name in names {
                if !context.framework.contains(&name) && !context.decls.contains(&name) {
                    failures.push((
                        name,
                        "it is neither a framework name nor a declared name".to_owned(),
                    ));
                }
            }
        },
        "mechanism-name" => {
            let cell = if row.cells().len() > 2 {
                row.cell(2)
            } else {
                text.clone()
            };
            for name in captures_of(&cell, r"\b([A-Z][A-Za-z0-9_]+)\b")? {
                if context.decls.contains(&name)
                    || context.externals.contains(&name)
                    || context.framework.contains(&name)
                {
                    continue;
                }
                failures.push((
                    text.clone(),
                    format!("`{name}` is no type this document knows"),
                ));
            }
        },
        _ => return Ok(None),
    }
    Ok(Some(failures))
}

/// Every membership rule that reads an id.
fn member_id_row(
    context: &MemberContext<'_>,
    kind: &str,
    row: &Row,
) -> anyhow::Result<Option<Vec<MemberFail>>> {
    let text = row.text();
    let tokens = row.tokens();
    let mut failures = Vec::new();
    match kind {
        "rule-id" => {
            let head = if row.cells().is_empty() {
                String::new()
            } else {
                row.cell(0)
            };
            for rule in captures_of(&head, r"\b((?:PG|CG)\d+[a-z]?)\b")? {
                if !context.rules.contains(&rule) {
                    failures.push((
                        text.clone(),
                        format!("`{rule}` is no rule this plan implements"),
                    ));
                }
            }
        },
        "block-id" => {
            let absent = tokens.first().filter(|head| !registered(head));
            if let Some(head) = absent {
                failures.push((text, format!("`{head}` is no registered block")));
            }
        },
        "sub-id" => {
            let shape = pattern::build(r"S\d+")?;
            let holds = tokens
                .first()
                .is_some_and(|head| shape.full_match(&pattern::chars(head)).is_some());
            if !holds {
                failures.push((text, "the row states no substitution id".to_owned()));
            }
        },
        "review-id" => {
            let shape = pattern::build(r"[CN]\d+[A-Z]?-W?\d+")?;
            let head = if row.cells().is_empty() {
                String::new()
            } else {
                row.cell(0).trim().to_owned()
            };
            if shape.full_match(&pattern::chars(&head)).is_none() {
                failures.push((text, "the row states no finding id".to_owned()));
            } else if row.cells().len() < 4 || row.cell(3).trim().is_empty() {
                failures.push((head, "the row names no section".to_owned()));
            }
        },
        "chunk-pair" => {
            for chunk in tokens.iter().take(2) {
                if !context.chunks.contains(chunk) {
                    failures.push((
                        text.clone(),
                        format!("`{chunk}` is no chunk of section 13.2"),
                    ));
                }
            }
        },
        "chunk-id" => {
            let joined = if row.cells().is_empty() {
                text.clone()
            } else {
                [1_usize, 3]
                    .iter()
                    .filter(|index| **index < row.cells().len())
                    .map(|index| row.cell(*index))
                    .collect::<Vec<String>>()
                    .join(" ")
            };
            for chunk in captures_of(&joined, r"\b([A-Z]{1,2}\d{1,2})\b")? {
                if !context.chunks.contains(&chunk) {
                    failures.push((
                        text.clone(),
                        format!("`{chunk}` is no chunk of section 13.2"),
                    ));
                }
            }
        },
        _ => return Ok(None),
    }
    Ok(Some(failures))
}

/// Every membership rule that reads a field path, a site, or an arm.
fn member_site_row(
    context: &MemberContext<'_>,
    kind: &str,
    row: &Row,
) -> anyhow::Result<Option<Vec<MemberFail>>> {
    let text = row.text();
    let tokens = row.tokens();
    let mut failures = Vec::new();
    match kind {
        "field-path" => {
            let Some(first) = tokens.first() else {
                failures.push((text, "the row states no `Type.field` path".to_owned()));
                return Ok(Some(failures));
            };
            if !first.contains('.') {
                failures.push((text, "the row states no `Type.field` path".to_owned()));
                return Ok(Some(failures));
            }
            let (holder, field) = first.split_once('.').unwrap_or((first.as_str(), ""));
            let head = tokens.get(1).cloned().unwrap_or_default();
            let declares = context
                .fields
                .get(holder)
                .is_some_and(|names| names.contains(field));
            if !context.audio_roots.contains(holder) {
                failures.push((text, format!("`{holder}` is in no audio-owned block")));
            } else if !declares {
                failures.push((text, format!("`{holder}` declares no field `{field}`")));
            } else if head.is_empty() {
                failures.push((text, "the row names no head type".to_owned()));
            } else if !context.externals.contains(&head) && !context.decls.contains(&head) {
                failures.push((text, format!("`{head}` is no type this document decides")));
            }
        },
        "carrier-end" => {
            let path = pattern::build(r"`([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)`")?;
            for index in [1_usize, 3] {
                let cell = if row.cells().len() > index {
                    row.cell(index)
                } else {
                    String::new()
                };
                let chars = pattern::chars(&cell);
                let found = path.find_iter(&chars);
                if found.is_empty() {
                    failures.push((
                        text.clone(),
                        format!("cell {index} names no `Type.field` end"),
                    ));
                    continue;
                }
                let ends = found
                    .iter()
                    .map(|one| (one.text(1, &chars), one.text(2, &chars)));
                for (holder, field) in ends {
                    let reason = carrier_end_reason(context, &holder, &field);
                    failures.extend(reason.map(|one| (text.clone(), one)));
                }
            }
        },
        "fault-arm" => {
            let cell = if row.cells().is_empty() {
                String::new()
            } else {
                row.cell(0)
            };
            let head = pattern::build(r"`([A-Za-z_][A-Za-z0-9_]*)")?;
            let chars = pattern::chars(&cell);
            let arms: BTreeSet<String> = context
                .arms
                .get("EngineFault")
                .map(|listed| listed.iter().map(|(name, _body)| name.clone()).collect())
                .unwrap_or_default();
            match head.match_at(&chars, 0) {
                None => failures.push((text, "the row names no `EngineFault` arm".to_owned())),
                Some(one) => {
                    let name = one.text(1, &chars);
                    if !arms.contains(&name) {
                        failures.push((name, "`EngineFault` declares no such arm".to_owned()));
                    }
                },
            }
        },
        "gate-site" => {
            let site = if row.cells().len() > 1 {
                row.cell(1)
            } else {
                String::new()
            };
            let names = captures_of(&site, r"`([^`]+)`")?;
            let mut leaves = BTreeSet::new();
            for token in &names {
                leaves.extend(captures_of(token, r"\b([A-Z][A-Za-z0-9_]*)\b")?);
                let leaf = token.rsplit("::").next().unwrap_or(token).to_owned();
                leaves.extend(captures_of(&leaf, r"\b([a-z_][a-z0-9_]*)\b")?);
            }
            let known = leaves
                .iter()
                .any(|name| context.decls.contains(name) || context.functions.contains(name));
            if names.is_empty() {
                failures.push((text, "the row names no site".to_owned()));
            } else if !known {
                failures.push((
                    site,
                    "no declaration and no function of this document carries the site".to_owned(),
                ));
            }
        },
        "cited-elsewhere" => {
            let site = if row.cells().is_empty() {
                text
            } else {
                strip_ticks(&row.cell(0))
            };
            if context.source.matches(&site).count() < 2 {
                failures.push((site, "no other line of this document names it".to_owned()));
            }
        },
        _ => return Ok(None),
    }
    Ok(Some(failures))
}

/// Why one `Type.field` end of a carrier row does not resolve, or `None`.
fn carrier_end_reason(context: &MemberContext<'_>, holder: &str, field: &str) -> Option<String> {
    if !context.decls.contains(holder) {
        return Some(format!("`{holder}` is no declaration of this document"));
    }
    let declares = context
        .fields
        .get(holder)
        .is_some_and(|names| names.contains(field));
    (!declares).then(|| format!("`{holder}` declares no field `{field}`"))
}

/// Every thread the section 5.7 thread table declares (PG41).
fn thread_names(source: &[char]) -> anyhow::Result<BTreeSet<String>> {
    let table = pattern::build(
        r"(?m)^\| Thread \| Owner \| Owns \| Never does \|\n\|[-| ]+\|\n((?:\|.*\n)+)",
    )?;
    let Some(found) = table.find(source) else {
        return Ok(BTreeSet::new());
    };
    let mut names = BTreeSet::new();
    for line in found.text(1, source).split('\n') {
        let cells = split_row(line)?;
        if let Some(first) = cells.first() {
            names.insert(trim_set(first.trim(), "`*"));
        }
    }
    Ok(names)
}

/// Which end one type expression holds, or `None` (PG41).
fn carrier_end_kind(expr: &str) -> Option<String> {
    CARRIER_HEADS
        .iter()
        .find(|(head, _kind)| contains_word(expr, head))
        .map(|(_head, kind)| (*kind).to_owned())
}

/// PG41. Every cross-thread carrier has two declared ends (TH13).
///
/// The rule runs in both directions. Forward: every row names a carrier, a B id
/// where the primitive needs one, two threads the section 5.7 table declares, and
/// an overflow rule. Reverse: every declared field whose type expression names a
/// carrier head is an end of exactly one row.
fn carrier_audit(
    source: &[char],
    blocks: &Blocks,
    decls: &DeclMap,
    arms: &BTreeMap<String, Vec<(String, String)>>,
    budgets: &BTreeSet<String>,
) -> anyhow::Result<(usize, usize, Pairs)> {
    let rows = blocks.rows("carrier-table");
    let threads = thread_names(source)?;
    if threads.is_empty() {
        return Ok((
            0,
            0,
            vec![(
                "<5.7>".to_owned(),
                "the thread table does not read, so PG41 has no thread set".to_owned(),
            )],
        ));
    }
    let mut ends: BTreeMap<String, (String, String)> = BTreeMap::new();
    for holder in decls.sorted_keys() {
        for (field, expr) in walk_fields(&holder, decls, arms)? {
            if let Some(kind) = carrier_end_kind(&expr) {
                ends.insert(format!("{holder}.{field}"), (kind, expr));
            }
        }
    }
    let mut named: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut failures = Vec::new();
    for row in rows {
        carrier_row(row, &ends, &threads, budgets, &mut named, &mut failures)?;
    }
    for (path, (_kind, expr)) in &ends {
        match named.get(path) {
            None => failures.push((
                path.clone(),
                format!(
                    "the field holds a carrier end, `{expr}`, and no row of the carrier \
table names it (TH13)"
                ),
            )),
            Some(messages) if messages.len() > 1 => failures.push((
                path.clone(),
                "two carrier rows name this end, and one end carries one message".to_owned(),
            )),
            Some(_) => {},
        }
    }
    Ok((rows.len(), ends.len(), failures))
}

/// Read one carrier row, and record the two ends it names.
fn carrier_row(
    row: &Row,
    ends: &BTreeMap<String, (String, String)>,
    threads: &BTreeSet<String>,
    budgets: &BTreeSet<String>,
    named: &mut BTreeMap<String, BTreeSet<String>>,
    failures: &mut Vec<(String, String)>,
) -> anyhow::Result<()> {
    if row.cells().len() < 6 {
        failures.push((
            "<row>".to_owned(),
            "the row holds fewer than six cells".to_owned(),
        ));
        return Ok(());
    }
    let message = trim_set(row.cell(0).trim(), "`");
    let path = pattern::build(r"`([A-Za-z_][A-Za-z0-9_]*)\.([A-Za-z_][A-Za-z0-9_]*)`")?;
    for (index, wanted) in [(1_usize, "write"), (3, "read")] {
        let cell = pattern::chars(&row.cell(index));
        for one in path.find_iter(&cell) {
            let full = format!("{}.{}", one.text(1, &cell), one.text(2, &cell));
            named
                .entry(full.clone())
                .or_default()
                .insert(message.clone());
            match ends.get(&full) {
                None => failures.push((
                    message.clone(),
                    format!("`{full}` holds no carrier end, so it cannot be the {wanted} end of this row"),
                )),
                Some((kind, expr)) if kind != wanted && kind != "share" => failures.push((
                    message.clone(),
                    format!("`{full}` is a {kind} end, `{expr}`, and this cell is the {wanted} end"),
                )),
                Some(_) => {},
            }
        }
    }
    carrier_cells(row, &message, threads, budgets, failures)?;
    Ok(())
}

/// Read the carrier, thread, and overflow cells of one carrier row.
fn carrier_cells(
    row: &Row,
    message: &str,
    threads: &BTreeSet<String>,
    budgets: &BTreeSet<String>,
    failures: &mut Vec<(String, String)>,
) -> anyhow::Result<()> {
    let carrier = row.cell(2);
    let primitives: Vec<&str> = CARRIER_PRIMITIVES
        .iter()
        .filter(|name| carrier.contains(**name))
        .copied()
        .collect();
    if primitives.is_empty() {
        failures.push((
            message.to_owned(),
            "the carrier cell names no primitive section 5.8 decides".to_owned(),
        ));
    } else if primitives != vec!["triple_buffer"] && !holds_pattern(&carrier, r"\bB\d+\b")? {
        failures.push((
            message.to_owned(),
            "the carrier cell names no B id of section 1.6".to_owned(),
        ));
    }
    for budget in captures_of(&carrier, r"\bB(\d+)\b")? {
        if !budgets.contains(&format!("B{budget}")) {
            failures.push((
                message.to_owned(),
                format!("the carrier cell names B{budget} and no section 1.6 row declares it"),
            ));
        }
    }
    let cell = row.cell(4);
    let halves: Vec<&str> = cell.split("->").collect();
    if halves.len() == 2 {
        let named = halves
            .iter()
            .flat_map(|half| half.split(" and "))
            .map(|name| trim_set(name.trim(), "`*"))
            .filter(|name| !name.is_empty() && !threads.contains(name));
        for name in named {
            failures.push((
                message.to_owned(),
                format!("`{name}` is no thread of the section 5.7 table"),
            ));
        }
    } else {
        failures.push((
            message.to_owned(),
            "the thread cell states no `<sender> -> <receiver>` pair".to_owned(),
        ));
    }
    if row.cell(5).trim().is_empty() {
        failures.push((
            message.to_owned(),
            "the row states no overflow or expiry rule".to_owned(),
        ));
    }
    Ok(())
}

/// Every input the guard read before it ran one rule.
#[derive(Debug)]
struct Inputs {
    /// Every registered block the document holds.
    blocks: Blocks,
    /// The framework type list section 1.3 declares.
    framework: BTreeSet<String>,
    /// Every external name and its verdict.
    externals: BTreeMap<String, String>,
    /// Every external name and the traits its row supplies.
    external_traits: BTreeMap<String, TraitPair>,
    /// The name sets the three TH1 rules refuse.
    sets: AudioSets,
    /// The section 1.5 candidate drop list.
    dropped: BTreeSet<String>,
    /// Every primitive and the traits it supplies.
    primitives: BTreeMap<String, BTreeSet<String>>,
    /// Every primitive and its size.
    sizes: BTreeMap<String, (usize, usize)>,
    /// The crate that owns each placed type.
    owned: BTreeMap<String, String>,
    /// The sections each crate declares its types in.
    declared_in: BTreeMap<String, BTreeSet<String>>,
    /// The third-party crates each section 1.2 row names.
    rows: BTreeMap<String, BTreeSet<String>>,
    /// The section 1.2 third-party name map.
    mapping: BTreeMap<String, String>,
    /// Every audio-owned root the section 5.7 block names.
    audio_roots: BTreeSet<String>,
    /// Every declaration with a hand-written `Drop` impl.
    drops: BTreeSet<String>,
    /// The section 1.3 edge list.
    edges: BTreeMap<String, BTreeSet<String>>,
    /// The transitive closure of the section 1.3 graph.
    closure: BTreeMap<String, BTreeSet<String>>,
    /// Every block whose floor sits below its own row count.
    floor_bad: Vec<(String, String)>,
}

/// Why the guard stopped before it ran a rule.
#[derive(Debug)]
struct FailClosed {
    /// Every line the guard prints before it exits 2.
    lines: Vec<String>,
}

/// Read every block and every set the rules run over.
fn load_inputs(source: &[char]) -> anyhow::Result<Result<Inputs, FailClosed>> {
    let (blocks, mut block_bad) = read_blocks(source)?;
    block_bad.extend(stray_markers(source)?);
    if !block_bad.is_empty() {
        let mut lines: Vec<String> = block_bad
            .iter()
            .map(|(block_id, reason)| {
                format!(
                    "FAIL: the `{block_id}` block of this document: {reason}; the guard is \
fail-closed (DR7)."
                )
            })
            .collect();
        lines.push(format!(
            "BLOCKS:          {}     BLOCK BAD: {}",
            DATA_BLOCKS.len(),
            block_bad.len()
        ));
        return Ok(Err(FailClosed { lines }));
    }
    let floor_bad = floor_audit(&blocks);
    let framework = framework_names(&blocks);
    let (externals, external_traits) = external_verdicts(&blocks)?;
    let (owned, declared_in) = ownership_table(&blocks)?;
    let rows = dependency_table(&blocks)?;
    let (mapping, mapping_bad) = name_map(&blocks, &rows);
    if !mapping_bad.is_empty() {
        return Ok(Err(FailClosed {
            lines: mapping_bad
                .iter()
                .map(|line| {
                    format!(
                        "FAIL: the name map holds a row the guard cannot resolve: {line}; the \
guard is fail-closed (critic WR-16)."
                    )
                })
                .collect(),
        }));
    }
    let (audio_roots, audio_bad) = audio_owned(&blocks, &owned);
    let (exempt, exempt_bad) = audio_exempt(&blocks, &owned);
    let (drops, drop_bad) = drop_impl_names(&blocks, &owned);
    let mut lines = Vec::new();
    for token in &audio_bad {
        lines.push(format!(
            "FAIL: the audio-owned block holds `{token}`, which section 1.5 places nowhere; \
the guard is fail-closed (critic CR-14)."
        ));
    }
    for line in &exempt_bad {
        lines.push(format!(
            "FAIL: the audio-exempt block holds `{line}`, which is no `Type.field` path; the \
guard is fail-closed."
        ));
    }
    for token in &drop_bad {
        lines.push(format!(
            "FAIL: the Drop block holds `{token}`, which section 1.5 places nowhere; the \
guard is fail-closed."
        ));
    }
    if !lines.is_empty() {
        return Ok(Err(FailClosed { lines }));
    }
    let edges = edge_list(&blocks)?;
    let closure = reachable(&edges);
    let sets = audio_sets(&blocks, exempt)?;
    Ok(Ok(Inputs {
        framework,
        externals,
        external_traits,
        sets,
        dropped: candidate_drop_list(&blocks),
        primitives: primitive_traits(&blocks),
        sizes: primitive_sizes(&blocks),
        owned,
        declared_in,
        rows,
        mapping,
        audio_roots,
        drops,
        edges,
        closure,
        floor_bad,
        blocks,
    }))
}

/// The name sets the three TH1 rules refuse, from the document's own blocks.
fn audio_sets(
    blocks: &Blocks,
    exempt: BTreeMap<(String, String), String>,
) -> anyhow::Result<AudioSets> {
    let mut heap: BTreeSet<String> = HEAP_NAMES.iter().map(|one| (*one).to_owned()).collect();
    heap.extend(externals_saying(blocks, "heap")?);
    heap.extend(token_rows(blocks.rows("heap-names")));
    let mut grows: BTreeSet<String> = GROW_NAMES.iter().map(|one| (*one).to_owned()).collect();
    grows.extend(externals_saying(blocks, "grows")?);
    grows.extend(token_rows(blocks.rows("grow-names")));
    let mut locks: BTreeSet<String> = LOCK_NAMES.iter().map(|one| (*one).to_owned()).collect();
    locks.extend(externals_saying(blocks, "lock")?);
    locks.extend(token_rows(blocks.rows("lock-names")));
    Ok(AudioSets {
        heap,
        exempt,
        defer: externals_saying(blocks, "defers")?,
        grows,
        locks,
        reads: externals_saying(blocks, "reads")?,
    })
}

/// Every declaration and every type name the document's Rust blocks state.
#[derive(Debug, Default)]
struct Parsed {
    /// How many times each name is declared, after the paths are stripped.
    declared_counts: BTreeMap<String, usize>,
    /// Every declaration, with the paths stripped.
    all_decls: DeclMap,
    /// Every declaration, with the paths kept.
    raw_decls: DeclMap,
    /// The section each declaration first appears in.
    block_sections: BTreeMap<String, Option<String>>,
    /// Every name that appears in a type position.
    used: BTreeSet<String>,
}

/// Read every Rust block of the document.
fn parse_blocks(source: &[char], marks: &[SectionMark]) -> anyhow::Result<Parsed> {
    let mut parsed = Parsed::default();
    for (offset, block) in rust_blocks(source)? {
        let section = section_at(marks, offset);
        let raw = strip_comments(&block)?;
        let code = strip_paths(&raw)?;
        for (name, entries) in declarations(&code)?.iter() {
            *parsed.declared_counts.entry(name.clone()).or_insert(0) += entries.len();
            parsed.all_decls.entry(name).extend(entries.iter().cloned());
            parsed
                .block_sections
                .entry(name.clone())
                .or_insert_with(|| section.clone());
        }
        for (name, entries) in declarations(&raw)?.iter() {
            parsed.raw_decls.entry(name).extend(entries.iter().cloned());
        }
        parsed
            .used
            .extend(used_types(&mask_enum_arm_names(&code)?)?);
    }
    Ok(parsed)
}

/// Every number the run prints that is not the length of a finding list.
#[derive(Debug, Default)]
struct Counts {
    /// The document path, as the caller wrote it.
    path: String,
    /// How many candidate type names the parse found.
    candidates: usize,
    /// How many names a Rust block declares.
    declared: usize,
    /// How many names the section 1.5 table places.
    table_names: usize,
    /// How many rows the external-verdict table holds.
    external_rows: usize,
    /// How many framework names section 1.3 declares.
    framework_names: usize,
    /// How many rows the third-party name map holds.
    name_map: usize,
    /// How many names the candidate drop list holds.
    drop_list: usize,
    /// How many primitives the trait block names.
    primitives: usize,
    /// How many primitives the size block names.
    primitive_sizes: usize,
    /// How many shared-limit rows PG28 read.
    limit_rows: usize,
    /// How many rows the probe table holds.
    probe_rows: usize,
    /// How many rule ids the prototypes implement.
    rule_ids: usize,
    /// How many phases the lock-file sequence covers.
    lock_phases: usize,
    /// How many appendix pin rows PG36 read.
    pin_rows: usize,
    /// How many blocks carry a membership rule.
    member_blocks: usize,
    /// How many rows the membership rules checked.
    member_rows: usize,
    /// How many budget rows the section 1.6 table holds.
    budget_rows: usize,
    /// How many stated sizes PG21 read.
    reason_sizes: usize,
    /// How many rows the VR1 table holds.
    vr1_rows: usize,
    /// How many audio-owned roots the block names.
    audio_owned: usize,
    /// How many exemption rows the audio block holds.
    audio_exempt: usize,
    /// How many declarations write a hand `Drop` impl.
    drop_impls: usize,
    /// How many declarations the audio closure reaches.
    audio_reachable: usize,
    /// How many reachable leaves the block allows.
    reachable_leaves: usize,
    /// How many edges the section 1.3 list states.
    edges_parsed: usize,
    /// How many crate rows section 1.2 holds.
    dep_rows: usize,
    /// How many crate dependencies a declaration proves.
    dep_proven: usize,
    /// How many budget rows PG37 decided.
    value_rows: usize,
    /// How many budget rows PG37 skipped.
    value_skipped: usize,
    /// How many table rows PG38 checked.
    table_rows: usize,
    /// How many ids section 1.7 states.
    index_ids: usize,
    /// How many chunk rows PG40 decided.
    line_chunks: usize,
    /// How many carrier rows the table holds.
    carriers: usize,
    /// How many declared fields hold a carrier end.
    carrier_fields: usize,
    /// How many tests section 14 selects.
    tests_selected: usize,
    /// How many links section 13.4 states.
    plan_links: usize,
    /// How many same-phase crate edges `PG31b` found.
    phase_pairs: usize,
    /// How many exemption rows the phase-pair block holds.
    pair_exempt: usize,
    /// How many suppression sites Appendix B.1 names.
    b1_sites: usize,
    /// How many names the section 2.3 sentence lists.
    listed_23: usize,
    /// How many roots the asserted block names.
    asserted_roots: usize,
    /// How many declarations carry a `missing_copy_implementations` expectation.
    expectations: usize,
    /// How many rows the B.1 copy table holds.
    b1_rows: usize,
    /// How many declarations carry a `variant_size_differences` expectation.
    variant_sites: usize,
    /// How many rows the B.1 variant table holds.
    b1_variant_rows: usize,
    /// How many justified unknowns section 1.9 names.
    justified: usize,
    /// How many snapshots the audio thread publishes.
    snapshots: usize,
    /// How many fields a deferring wrapper carries off the audio thread.
    deferred: usize,
    /// How many fields a read end hands over as a shared reference.
    readonly: usize,
}

/// Every finding the run prints, by rule.
#[derive(Debug, Default)]
struct Lists {
    /// PG3. Every declaration whose body states a comment where a field goes.
    placeholders: Pairs,
    /// PG27. Every row of a registered block that names no referent.
    member_bad: Vec<(String, String, String)>,
    /// PG30. Every `B` citation the `Used by` column disagrees with.
    used_bad: Pairs,
    /// PG38. Every table row whose cell count differs from its header.
    ragged_bad: Pairs,
    /// PG39. Every rule id section 1.7 and the rule set disagree about.
    index_bad: Pairs,
    /// PG40. Every chunk that writes outside the crate its line owns.
    chunk_crate_bad: Pairs,
    /// `PG4b`. Every placed name no Rust block declares.
    undeclared: Vec<String>,
    /// PG18. Every external token the section 1.9 table omits.
    external_misses: Pairs,
    /// PG4. Every declared type the section 1.5 table omits.
    unplaced: Vec<String>,
    /// PG5. Every type two Rust blocks declare.
    duplicated: Vec<String>,
    /// PG6. Every framework name a non-application crate claims.
    misclaimed: Vec<String>,
    /// PG7. Every framework type inside a non-application declaration.
    misused: Vec<(String, String, String)>,
    /// PG9. Every all-`Copy` type that derives no `Copy`.
    copy_missing: Pairs,
    /// PG10. Every `Copy` derive over a field that is not `Copy`.
    copy_impossible: Vec<(String, String, String)>,
    /// `PG10b`. Every `Copy` derive over a field the document does not decide.
    copy_undecided: Vec<(String, String, String)>,
    /// PG14. Every undecidable field of any declaration.
    copy_unknown: Vec<(String, String, String)>,
    /// PG17. Every undecidable field the justified table does not name.
    unjustified: Pairs,
    /// PG19. Every derive-closure break.
    closure_broken: Vec<(String, String, String, String, String)>,
    /// PG19. Every field expression the closure rule cannot read.
    closure_undecided: Vec<(String, String, String, String)>,
    /// PG20. Every field type in a crate the graph does not reach.
    reach_misses: Vec<(String, String, String, String)>,
    /// PG21. Every expectation the Appendix B.1 tables disagree with.
    expectation_misses: Pairs,
    /// PG21. Every stated size that is a literal and not a citation.
    reason_bad: Pairs,
    /// PG26. Every heap allocation inside audio-owned state.
    heap_misses: Vec<AudioHit>,
    /// `PG26d`. Every disagreement between the block and the markers.
    root_bad: Pairs,
    /// `PG26e`. Every break in the audio-owned reachability closure.
    closure_bad: Pairs,
    /// `PG26f`. Every disagreement between the root set and the asserted block.
    asserted_bad: Pairs,
    /// `PG31b`. Every same-phase crate edge with no exemption row.
    pair_bad: Pairs,
    /// PG34. Every phase-table number the SM6 bullet disagrees with.
    tail_bad: Pairs,
    /// PG33. Every disagreement between B.1, section 2.3, and the declarations.
    b1_bad: Pairs,
    /// PG31. Every section 13.4 link that does not run forward.
    link_bad: Pairs,
    /// `PG26b`. Every container that grows inside audio-owned state.
    grow_misses: Vec<AudioHit>,
    /// `PG26c`. Every lock inside audio-owned state.
    lock_misses: Vec<AudioHit>,
    /// PG22. Every `Hash` or `Ord` derive with no VR1 row.
    vr1_misses: Vec<(String, String, Vec<String>)>,
    /// PG23. Every `PartialEq` derive with no `Eq` over an `Eq` body.
    eq_missing: Pairs,
    /// PG23. Every declaration the `Eq` rule cannot decide.
    eq_undecided: Pairs,
    /// PG24. Every enum whose arm spread the lint would refuse.
    spread_bad: Vec<(String, String, String, usize, usize)>,
    /// PG24. Every spread a single-site expectation takes out of the set.
    spread_expected: Vec<(String, String, String, usize, usize)>,
    /// PG24. Every declaration the model sized.
    size_rows: Vec<SizeRow>,
    /// PG24. Every declaration the model cannot size.
    size_undecided: Vec<OpenRow>,
    /// PG11. Every prose sentence that claims an edge the list does not carry.
    edge_misses: Vec<(String, String, String)>,
    /// `PG20b`. Every constant in a crate the graph does not reach.
    const_reach: Vec<(String, String, String, String, String)>,
    /// PG28. Every shared limit an enforcing crate cannot reach.
    limit_bad: Vec<(String, String, String)>,
    /// PG29. Every disagreement between the probe table and the rule set.
    probe_bad: Pairs,
    /// `PG27b`. Every block whose floor sits below its own row count.
    floor_bad: Pairs,
    /// PG37. Every budget value a citing declaration line refutes.
    value_bad: Pairs,
    /// PG35. Every per-phase lock-writer count the sentence disagrees with.
    lock_bad: Pairs,
    /// PG36. Every pin owner the two appendices disagree about.
    pin_bad: Pairs,
    /// PG12. Every crate row that omits a dependency its declarations prove.
    dependency_misses: Vec<(String, String, String)>,
    /// PG13. Every published snapshot type that is not declared or not `Copy`.
    snapshot_misses: Pairs,
    /// PG15. Every `Declared in` cell that omits a declaring section.
    register_misses: Vec<(String, String, String)>,
    /// PG16. Every selected test the table does not place.
    test_misses: Pairs,
    /// PG41. Every carrier row or carrier field with no matching end.
    carrier_bad: Pairs,
}

impl Lists {
    /// Whether any rule found something, which is exit 1.
    const fn any(&self) -> bool {
        !self.placeholders.is_empty()
            || !self.heap_misses.is_empty()
            || !self.grow_misses.is_empty()
            || !self.lock_misses.is_empty()
            || !self.root_bad.is_empty()
            || !self.closure_bad.is_empty()
            || !self.asserted_bad.is_empty()
            || !self.pair_bad.is_empty()
            || !self.tail_bad.is_empty()
            || !self.b1_bad.is_empty()
            || !self.link_bad.is_empty()
            || !self.reason_bad.is_empty()
            || !self.undeclared.is_empty()
            || !self.external_misses.is_empty()
            || !self.unplaced.is_empty()
            || !self.duplicated.is_empty()
            || !self.misclaimed.is_empty()
            || !self.misused.is_empty()
            || !self.copy_missing.is_empty()
            || !self.copy_impossible.is_empty()
            || !self.copy_undecided.is_empty()
            || !self.unjustified.is_empty()
            || self.more()
    }

    /// The second half of the finding test, kept inside the complexity budget.
    const fn more(&self) -> bool {
        !self.closure_broken.is_empty()
            || !self.closure_undecided.is_empty()
            || !self.reach_misses.is_empty()
            || !self.expectation_misses.is_empty()
            || !self.vr1_misses.is_empty()
            || !self.eq_missing.is_empty()
            || !self.spread_bad.is_empty()
            || !self.edge_misses.is_empty()
            || !self.dependency_misses.is_empty()
            || !self.snapshot_misses.is_empty()
            || !self.register_misses.is_empty()
            || !self.test_misses.is_empty()
            || !self.const_reach.is_empty()
            || !self.limit_bad.is_empty()
            || !self.probe_bad.is_empty()
            || !self.member_bad.is_empty()
            || !self.used_bad.is_empty()
            || !self.lock_bad.is_empty()
            || !self.pin_bad.is_empty()
            || !self.floor_bad.is_empty()
            || !self.value_bad.is_empty()
            || !self.ragged_bad.is_empty()
            || !self.index_bad.is_empty()
            || !self.chunk_crate_bad.is_empty()
            || !self.carrier_bad.is_empty()
    }
}

/// Everything the rules read, resolved once.
#[derive(Debug)]
struct Ctx<'a> {
    /// The whole document, as characters.
    source: &'a [char],
    /// The whole document, as text.
    text: &'a str,
    /// Every block and every set the rules run over.
    inputs: &'a Inputs,
    /// Every declaration and every used name.
    parsed: &'a Parsed,
    /// Every `### N.N` heading and its offset.
    marks: &'a [SectionMark],
    /// Every hand-written trait impl.
    impls: &'a BTreeMap<String, BTreeSet<String>>,
    /// Every enum arm this document declares.
    arms: &'a BTreeMap<String, Vec<(String, String)>>,
    /// The Copy verdict of every name the guard can decide.
    is_copy: &'a BTreeMap<String, bool>,
    /// Every name the 1.5 table or the 1.9 table decides.
    known: &'a BTreeSet<String>,
    /// Every constant, with the crate that declares it.
    constants_by_crate: &'a BTreeMap<String, (String, Option<usize>)>,
    /// Every `usize` constant, with its value.
    constants: &'a BTreeMap<String, usize>,
    /// Every whole-expression size the document records.
    recorded: &'a BTreeMap<String, (usize, usize)>,
}

impl Ctx<'_> {
    /// The trait resolver PG19 and PG23 share.
    const fn resolver(&self) -> Resolver<'_> {
        Resolver {
            decls: &self.parsed.raw_decls,
            impls: self.impls,
            externals: &self.inputs.external_traits,
            primitives: &self.inputs.primitives,
        }
    }
}

/// PG11. Every prose sentence that claims an edge the list does not carry.
fn edge_claims(
    source: &[char],
    edges: &BTreeMap<String, BTreeSet<String>>,
) -> anyhow::Result<Vec<(String, String, String)>> {
    let fences = pattern::build(r"(?s)```.*?```")?;
    let spans: Vec<(usize, usize)> = fences
        .find_iter(source)
        .iter()
        .map(|one| (one.start(), one.end()))
        .collect();
    let gap = r"(?:(?!\. )[^`;\n])";
    let verbs = EDGE_VERBS.join("|");
    let claim = pattern::build(&format!(
        r"`(duet(?:-[a-z]+)*|crates/duet)`({gap}{{0,40}}?)\b({verbs})\b({gap}{{0,60}}?)`(duet(?:-[a-z]+)*|crates/duet)`"
    ))?;
    let negation = pattern::build(r"\b(no|not|never)\b")?;
    let mut failures = Vec::new();
    for one in claim.find_iter(source) {
        if spans
            .iter()
            .any(|(start, end)| *start <= one.start() && one.start() < *end)
        {
            continue;
        }
        let around = format!("{} {}", one.text(2, source), one.text(4, source));
        if negation.is_match(&pattern::chars(&around)) {
            continue;
        }
        let subject = normalize_crate(&one.text(1, source));
        let target = normalize_crate(&one.text(5, source));
        if subject == target {
            continue;
        }
        let known = edges.get(&subject);
        if !known.is_some_and(|targets| targets.contains(&target)) {
            failures.push((subject, target, one.text(0, source).trim().to_owned()));
        }
    }
    Ok(failures)
}

/// The exemption pairs of the `phase-pair-exempt` block, and every bad reason.
///
/// The reason is the thing a reviewer reads to accept the exemption, so it must
/// name both chunks of its OWN pair (critic C18-N1). The ceiling refuses an
/// ADDITION, so a twenty-fifth pair is a review decision (N19-1).
fn phase_pair_exemptions(blocks: &Blocks) -> anyhow::Result<(BTreeMap<String, String>, Pairs)> {
    let mut pairs = BTreeMap::new();
    let mut bad = Vec::new();
    for row in blocks.rows("phase-pair-exempt") {
        let parts = row.tokens();
        if parts.len() < 3 {
            continue;
        }
        let pair = parts.get(..2).unwrap_or(&[]).join(" ");
        let reason = parts.get(2..).unwrap_or(&[]).join(" ");
        pairs.insert(pair.clone(), reason.clone());
        for chunk in parts.iter().take(2) {
            let names = pattern::build(&format!(
                r"(?<![A-Za-z0-9]){}(?![0-9])",
                pattern::quote(chunk)
            ))?;
            if !names.is_match(&pattern::chars(&reason)) {
                bad.push((
                    pair.clone(),
                    format!("the reason does not name `{chunk}`, which is one of its own pair"),
                ));
            }
        }
    }
    if pairs.len() > EXEMPT_CEILING {
        bad.push((
            "phase-pair-exempt".to_owned(),
            format!(
                "the block holds {} rows and the ceiling is {EXEMPT_CEILING}; a twenty-fifth \
pair is a review decision and not one more row (N19-1)",
                pairs.len()
            ),
        ));
    }
    Ok((pairs, bad))
}

/// The name set the placement rules decide over.
fn candidate_names(parsed: &Parsed, dropped: &BTreeSet<String>) -> BTreeSet<String> {
    let mut names: BTreeSet<String> = parsed.declared_counts.keys().cloned().collect();
    names.extend(parsed.used.iter().cloned());
    names
        .into_iter()
        .filter(|name| !dropped.contains(name) && name.chars().count() > 1 && !is_upper(name))
        .collect()
}

/// Run every rule that reads a declaration against the section 1.5 table.
fn collect_types(ctx: &Ctx<'_>, lists: &mut Lists, counts: &mut Counts) -> anyhow::Result<()> {
    let inputs = ctx.inputs;
    let decls = &ctx.parsed.all_decls;
    let raw = &ctx.parsed.raw_decls;
    let candidates = candidate_names(ctx.parsed, &inputs.dropped);
    counts.candidates = candidates.len();
    counts.declared = decls.len();
    lists.unplaced = candidates
        .iter()
        .filter(|name| !inputs.owned.contains_key(*name) && !inputs.framework.contains(*name))
        .cloned()
        .collect();
    lists.duplicated = ctx
        .parsed
        .declared_counts
        .iter()
        .filter(|(_name, count)| **count > 1)
        .map(|(name, _count)| name.clone())
        .collect();
    lists.misclaimed = inputs
        .framework
        .iter()
        .filter(|name| {
            inputs
                .owned
                .get(*name)
                .is_some_and(|crate_name| !APP_ROWS.contains(&crate_name.as_str()))
        })
        .cloned()
        .collect();
    lists.misused = framework_misuse(raw, &inputs.owned, &inputs.framework)?;
    let report = copy_audit(
        decls,
        &inputs.owned,
        ctx.is_copy,
        &inputs.externals,
        ctx.known,
        &inputs.drops,
    )?;
    lists.copy_missing = report.missing;
    lists.copy_impossible = report.impossible;
    lists.copy_undecided = report.undecided;
    lists.copy_unknown = report.unknown;
    lists.undeclared = placed_without_declaration(&inputs.owned, decls);
    lists.external_misses = external_audit(
        decls,
        &inputs.owned,
        &inputs.framework,
        &inputs.externals,
        ctx.known,
    )?;
    lists.placeholders = comment_placeholders(ctx.source, &inputs.owned)?;
    let justified = justified_unknowns(&inputs.blocks)?;
    counts.justified = justified.len();
    lists.unjustified = unknown_audit(&lists.copy_unknown, &justified);
    let resolver = ctx.resolver();
    let (broken, undecided) =
        derive_closure_audit(raw, &inputs.owned, &inputs.framework, resolver)?;
    lists.closure_broken = broken;
    lists.closure_undecided = undecided;
    lists.reach_misses = reachability_audit(raw, &inputs.owned, &inputs.closure)?;
    let (eq_missing, eq_undecided) =
        eq_audit(raw, &inputs.owned, ctx.impls, &inputs.framework, resolver)?;
    lists.eq_missing = eq_missing;
    lists.eq_undecided = eq_undecided;
    let vr1_listed = vr1_table(&inputs.blocks)?;
    counts.vr1_rows = vr1_listed.len();
    lists.vr1_misses = vr1_audit(raw, &inputs.owned, ctx.impls, &vr1_listed)?;
    Ok(())
}

/// Run every rule that reads the section 1.2 and 1.3 graphs.
fn collect_graph(ctx: &Ctx<'_>, lists: &mut Lists, counts: &mut Counts) -> anyhow::Result<()> {
    let inputs = ctx.inputs;
    lists.edge_misses = edge_claims(ctx.source, &inputs.edges)?;
    let (misses, proven) = dependency_audit(
        &ctx.parsed.raw_decls,
        &inputs.owned,
        &inputs.rows,
        &inputs.mapping,
    )?;
    lists.dependency_misses = misses;
    counts.dep_proven = proven.len();
    counts.dep_rows = inputs.rows.len();
    counts.edges_parsed = inputs.edges.values().map(BTreeSet::len).sum();
    let (snapshot_misses, snapshots) =
        snapshot_audit(&inputs.blocks, &ctx.parsed.all_decls, ctx.is_copy)?;
    lists.snapshot_misses = snapshot_misses;
    counts.snapshots = snapshots.len();
    lists.register_misses = register_audit(
        &ctx.parsed.block_sections,
        &inputs.owned,
        &inputs.declared_in,
    );
    lists.const_reach = constant_reach_audit(
        &ctx.parsed.raw_decls,
        &inputs.owned,
        &inputs.closure,
        ctx.constants_by_crate,
    )?;
    let (limit_rows, limit_bad) =
        shared_limit_audit(&inputs.blocks, &inputs.closure, ctx.constants_by_crate);
    counts.limit_rows = limit_rows;
    lists.limit_bad = limit_bad;
    Ok(())
}

/// Run every rule that reads audio-owned state.
fn collect_audio(ctx: &Ctx<'_>, lists: &mut Lists, counts: &mut Counts) -> anyhow::Result<()> {
    let inputs = ctx.inputs;
    let raw = &ctx.parsed.raw_decls;
    let marked = audio_marked(ctx.source)?;
    lists.root_bad = audio_root_audit(&inputs.audio_roots, &marked);
    let reached = audio_reachable(raw, ctx.arms)?;
    let asserted: BTreeSet<String> = inputs
        .blocks
        .rows("audio-asserted")
        .iter()
        .filter_map(|row| row.tokens().first().cloned())
        .collect();
    lists.asserted_bad = audio_asserted_audit(&inputs.audio_roots, &reached, &asserted);
    let leaves: BTreeSet<String> = inputs
        .blocks
        .rows("audio-reachable-leaf")
        .iter()
        .filter_map(|row| row.tokens().first().cloned())
        .collect();
    lists.closure_bad = audio_closure_audit(&inputs.audio_roots, &reached, &leaves);
    let report = heap_audit(
        raw,
        &inputs.owned,
        ctx.arms,
        &inputs.audio_roots,
        &inputs.sets,
    )?;
    counts.audio_owned = inputs.audio_roots.len();
    counts.audio_exempt = inputs.sets.exempt.len();
    counts.drop_impls = inputs.drops.len();
    counts.audio_reachable = reached.len();
    counts.reachable_leaves = leaves.len();
    counts.asserted_roots = asserted.len();
    counts.deferred = report.deferred.len();
    counts.readonly = report.readonly.len();
    lists.heap_misses = report.heap;
    lists.grow_misses = report.grows;
    lists.lock_misses = report.locks;
    Ok(())
}

/// Run PG24 and PG21's size rules over every declaration.
fn collect_sizes(
    ctx: &Ctx<'_>,
    heads: &BTreeMap<String, (usize, usize)>,
    lists: &mut Lists,
    counts: &mut Counts,
) -> anyhow::Result<()> {
    let inputs = ctx.inputs;
    let raw = &ctx.parsed.raw_decls;
    let index = FieldIndex::build(raw, ctx.arms)?;
    let mut sizer = Sizer {
        decls: raw,
        index: &index,
        recorded: ctx.recorded,
        heads,
        constants: ctx.constants,
        primitives: &inputs.sizes,
        named: pattern::build(
            r"(?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*([A-Za-z_][A-Za-z0-9_]*)\s*(<[\s\S]*>)?",
        )?,
        cache: BTreeMap::new(),
        active: BTreeSet::new(),
    };
    let report = size_audit(raw, &inputs.owned, &mut sizer)?;
    lists.size_rows = report.decided;
    lists.size_undecided = report.undecided;
    lists.spread_bad = report.spread;
    lists.spread_expected = report.expected;
    let mut sites = attribute_texts(ctx.source, ctx.marks)?;
    sites.extend(b1_reason_cells(&inputs.blocks));
    sites.extend(doc_comment_lines(ctx.source)?);
    let (reason_count, reason_bad) = reason_size_audit(&sites, ctx.recorded)?;
    counts.reason_sizes = reason_count;
    lists.reason_bad = reason_bad;
    Ok(())
}

/// Run every rule that reads section 13, section 14, and the appendices.
fn collect_plan(ctx: &Ctx<'_>, lists: &mut Lists, counts: &mut Counts) -> anyhow::Result<()> {
    let inputs = ctx.inputs;
    let (test_misses, selected) = selected_tests(ctx.source, &inputs.blocks)?;
    lists.test_misses = test_misses;
    counts.tests_selected = selected.len();
    counts.plan_links = link_rows(ctx.text)?.len();
    let mut link_bad = link_audit(ctx.text, &inputs.blocks)?;
    link_bad.extend(line_phase_audit(&inputs.blocks)?);
    lists.link_bad = link_bad;
    let (exempt_pairs, exempt_reason_bad) = phase_pair_exemptions(&inputs.blocks)?;
    let (pair_found, pair_bad) = phase_pair_audit(&inputs.blocks, &exempt_pairs)?;
    counts.phase_pairs = pair_found;
    counts.pair_exempt = exempt_pairs.len();
    lists.pair_bad = pair_bad;
    let mut tail_bad = tail_phase_audit(ctx.source, &inputs.blocks)?;
    tail_bad.extend(exempt_reason_bad);
    lists.tail_bad = tail_bad;
    let (lock_sequence, lock_bad) = lock_sequence_audit(ctx.text, &inputs.blocks)?;
    counts.lock_phases = lock_sequence.map_or(0, |found| found.len());
    lists.lock_bad = lock_bad;
    let (pin_rows, pin_bad) = pin_owner_audit(ctx.source)?;
    counts.pin_rows = pin_rows;
    lists.pin_bad = pin_bad;
    let (used_rows, used_bad) = used_by_audit(ctx.source)?;
    counts.budget_rows = used_rows;
    lists.used_bad = used_bad;
    let value = budget_value_audit(ctx.source, &inputs.blocks)?;
    counts.value_rows = value.decided;
    counts.value_skipped = value.skipped.len();
    lists.value_bad = value.failures;
    if value.decided < VALUE_FLOOR {
        lists.value_bad.push((
            "PG37".to_owned(),
            format!(
                "the rule decided {} budget rows and its floor is {VALUE_FLOOR}; a decided \
set below the floor is a silent shrink of a denominator (critic N21-3)",
                value.decided
            ),
        ));
    }
    let (table_rows, ragged_bad) = ragged_row_audit(ctx.text)?;
    counts.table_rows = table_rows;
    lists.ragged_bad = ragged_bad;
    let (chunk_rows, mut chunk_crate_bad) = chunk_crate_audit(ctx.source, &inputs.blocks)?;
    counts.line_chunks = chunk_rows;
    if chunk_rows < LINE_CHUNK_FLOOR {
        chunk_crate_bad.push((
            "PG40".to_owned(),
            format!(
                "the rule decided {chunk_rows} chunk rows and its floor is \
{LINE_CHUNK_FLOOR}; a decided set below the floor is a silent shrink of a denominator \
(critic C22I-W3)"
            ),
        ));
    }
    lists.chunk_crate_bad = chunk_crate_bad;
    Ok(())
}

/// Build the set every membership rule reads a row against.
fn member_context<'a>(
    ctx: &'a Ctx<'a>,
    rule_ids: &BTreeSet<String>,
) -> anyhow::Result<MemberContext<'a>> {
    let inputs = ctx.inputs;
    let mut edge_crates: BTreeSet<String> = inputs
        .edges
        .keys()
        .map(|name| normalize_crate(name))
        .collect();
    edge_crates.extend(
        inputs
            .edges
            .values()
            .flatten()
            .map(|name| normalize_crate(name)),
    );
    let owned: BTreeSet<String> = inputs.owned.keys().cloned().collect();
    let externals: BTreeSet<String> = inputs.externals.keys().cloned().collect();
    let decls = ctx.parsed.all_decls.key_set();
    let external_paths = external_path_names(&inputs.blocks);
    let mut external_universe = externals.clone();
    external_universe.extend(inputs.framework.iter().cloned());
    external_universe.extend(inputs.mapping.keys().cloned());
    external_universe.extend(inputs.dropped.iter().cloned());
    external_universe.extend(owned.iter().cloned());
    let mut size_universe = decls.clone();
    size_universe.extend(externals.iter().cloned());
    size_universe.extend(inputs.framework.iter().cloned());
    size_universe.extend(inputs.sizes.keys().cloned());
    size_universe.extend(owned.iter().cloned());
    let mut fields = BTreeMap::new();
    for name in ctx.parsed.raw_decls.sorted_keys() {
        let listed: BTreeSet<String> = walk_fields(&name, &ctx.parsed.raw_decls, ctx.arms)?
            .into_iter()
            .map(|(field, _expr)| field)
            .collect();
        fields.insert(name, listed);
    }
    Ok(MemberContext {
        source: ctx.text,
        crates: inputs.rows.keys().cloned().collect(),
        edge_crates,
        third_party: inputs.rows.values().flatten().cloned().collect(),
        decls,
        functions: declared_functions(ctx.source)?.keys().cloned().collect(),
        arms: ctx.arms,
        owned,
        framework: inputs.framework.clone(),
        externals,
        external_paths,
        constants: ctx.constants_by_crate.keys().cloned().collect(),
        rules: rule_ids.clone(),
        audio_roots: inputs.audio_roots.clone(),
        fields,
        chunks: chunk_ids(ctx.source)?,
        primitive_traits: inputs.primitives.keys().cloned().collect(),
        primitive_sizes: inputs.sizes.keys().cloned().collect(),
        external_universe,
        size_universe,
    })
}

/// One fail-closed line, with the reason the guard cannot decide.
fn fail_line(reason: &str, tail: &str) -> String {
    format!("FAIL: {reason}; the guard is fail-closed{tail}.")
}

/// Run every rule and return what the run prints.
fn analyse(
    document: &Path,
    source: &[char],
    text: &str,
    inputs: &Inputs,
) -> anyhow::Result<Result<(Counts, Lists), FailClosed>> {
    let marks = section_of(source)?;
    let impls = hand_impls(source)?;
    let parsed = parse_blocks(source, &marks)?;
    let arms = enum_arms_of(source)?;
    let is_copy = copy_verdicts(&parsed.all_decls, &inputs.externals, &impls)?;
    let mut known: BTreeSet<String> = inputs.owned.keys().cloned().collect();
    known.extend(inputs.externals.keys().cloned());
    let constants_by_crate = constant_owners(&inputs.blocks)?;
    let constants = usize_constants(&constants_by_crate);
    let (recorded, heads) = recorded_sizes(&inputs.blocks);
    let ctx = Ctx {
        source,
        text,
        inputs,
        parsed: &parsed,
        marks: &marks,
        impls: &impls,
        arms: &arms,
        is_copy: &is_copy,
        known: &known,
        constants_by_crate: &constants_by_crate,
        constants: &constants,
        recorded: &recorded,
    };
    let mut counts = Counts {
        path: document.display().to_string(),
        framework_names: inputs.framework.len(),
        name_map: inputs.mapping.len(),
        drop_list: inputs.dropped.len(),
        primitives: inputs.primitives.len(),
        primitive_sizes: inputs.sizes.len(),
        table_names: inputs.owned.len(),
        external_rows: inputs.externals.len(),
        ..Counts::default()
    };
    let mut lists = Lists {
        floor_bad: inputs.floor_bad.clone(),
        ..Lists::default()
    };
    collect_types(&ctx, &mut lists, &mut counts)?;
    collect_graph(&ctx, &mut lists, &mut counts)?;
    let suppression = match suppression_audit(source, &inputs.blocks)? {
        Ok(found) => found,
        Err(reason) => {
            return Ok(Err(FailClosed {
                lines: vec![fail_line(&reason, "")],
            }));
        },
    };
    counts.b1_sites = suppression.sites.len();
    counts.listed_23 = suppression.listed.len();
    lists.b1_bad = suppression.failures;
    collect_audio(&ctx, &mut lists, &mut counts)?;
    collect_sizes(&ctx, &heads, &mut lists, &mut counts)?;
    collect_plan(&ctx, &mut lists, &mut counts)?;
    let tools = document
        .parent()
        .map_or_else(|| Path::new("tools").to_path_buf(), |dir| dir.join("tools"));
    let rule_ids = match implemented_rule_ids(&tools)? {
        Ok(found) => found,
        Err(reason) => {
            return Ok(Err(FailClosed {
                lines: vec![fail_line(&reason, " (PG29)")],
            }));
        },
    };
    counts.rule_ids = rule_ids.len();
    let (probe_rows, probe_bad) = probe_table_audit(&inputs.blocks, &rule_ids)?;
    counts.probe_rows = probe_rows;
    lists.probe_bad = probe_bad;
    let (member_kinds, member_kind_bad) = block_member_kinds(&inputs.blocks);
    let context = member_context(&ctx, &rule_ids)?;
    let (member_rows, member_bad) = membership_audit(&inputs.blocks, &member_kinds, &context)?;
    if !member_kind_bad.is_empty() {
        return Ok(Err(FailClosed {
            lines: member_kind_bad
                .iter()
                .map(|(block_id, reason)| {
                    format!(
                        "FAIL: the `block-members` block of this document: `{block_id}`: \
{reason}; the guard is fail-closed (DR7, critic WR-18)."
                    )
                })
                .collect(),
        }));
    }
    counts.member_blocks = member_kinds.len();
    counts.member_rows = member_rows;
    lists.member_bad = member_bad;
    collect_registers(&ctx, &rule_ids, &mut lists, &mut counts)?;
    Ok(Ok((counts, lists)))
}

/// Run the expectation, index, and carrier rules.
fn collect_registers(
    ctx: &Ctx<'_>,
    rule_ids: &BTreeSet<String>,
    lists: &mut Lists,
    counts: &mut Counts,
) -> anyhow::Result<()> {
    let inputs = ctx.inputs;
    let listed = expectation_tables(&inputs.blocks)?;
    let (mut expectation_misses, carried) = expectation_audit(&ctx.parsed.raw_decls, &listed);
    let empty = BTreeSet::new();
    let copy_lint = EXPECTED_LINTS.first().copied().unwrap_or_default();
    let variant_lint = EXPECTED_LINTS.get(1).copied().unwrap_or_default();
    let copy_sites = carried.get(copy_lint).unwrap_or(&empty);
    counts.expectations = copy_sites.len();
    counts.b1_rows = listed.get(copy_lint).unwrap_or(&empty).len();
    counts.variant_sites = carried.get(variant_lint).unwrap_or(&empty).len();
    counts.b1_variant_rows = listed.get(variant_lint).unwrap_or(&empty).len();
    for name in inputs.drops.intersection(copy_sites) {
        expectation_misses.push((
            name.clone(),
            "carries a missing_copy_implementations expectation and writes a Drop impl, \
which makes the expectation unfulfilled and the build fail (critic WR-12)"
                .to_owned(),
        ));
    }
    lists.expectation_misses = expectation_misses;
    let probe_ids: Vec<String> = inputs
        .blocks
        .rows("probe-table")
        .iter()
        .filter(|row| row.cells().len() >= 5)
        .map(|row| strip_ticks(row.cell(2).trim()))
        .collect();
    let (index_ids, index_bad) = rule_index_audit(ctx.source, rule_ids, &probe_ids)?;
    counts.index_ids = index_ids;
    lists.index_bad = index_bad;
    let budgets: BTreeSet<String> = inputs
        .blocks
        .rows("budget-table")
        .iter()
        .filter(|row| !row.cells().is_empty())
        .map(|row| row.cell(0).trim().to_owned())
        .collect();
    let (carriers, carrier_fields, carrier_bad) = carrier_audit(
        ctx.source,
        &inputs.blocks,
        &ctx.parsed.raw_decls,
        ctx.arms,
        &budgets,
    )?;
    counts.carriers = carriers;
    counts.carrier_fields = carrier_fields;
    lists.carrier_bad = carrier_bad;
    Ok(())
}

/// Print the counters that read the document's registers.
fn print_registers<W: io::Write>(
    out: &mut W,
    counts: &Counts,
    lists: &Lists,
) -> anyhow::Result<()> {
    writeln!(out, "DOCUMENT:        {}", counts.path)?;
    writeln!(
        out,
        "BLOCKS:          {}     BLOCK BAD: 0",
        DATA_BLOCKS.len()
    )?;
    writeln!(
        out,
        "FRAMEWORK NAMES: {}    NAME MAP: {}",
        counts.framework_names, counts.name_map
    )?;
    writeln!(
        out,
        "DROP LIST:       {}     PRIMITIVES: {}     PRIMITIVE SIZES: {}",
        counts.drop_list, counts.primitives, counts.primitive_sizes
    )?;
    writeln!(
        out,
        "CANDIDATE TYPES: {}   DECLARED: {}   PLACEHOLDERS: {}",
        counts.candidates,
        counts.declared,
        lists.placeholders.len()
    )?;
    writeln!(
        out,
        "TABLE NAMES:     {}     UNDECLARED: {}",
        counts.table_names,
        lists.undeclared.len()
    )?;
    writeln!(
        out,
        "EXTERNAL ROWS:   {}     EXTERNAL MISSING: {}",
        counts.external_rows,
        lists.external_misses.len()
    )?;
    writeln!(
        out,
        "PLACED:          {}",
        counts.candidates.saturating_sub(lists.unplaced.len())
    )?;
    writeln!(
        out,
        "UNPLACED:        {}     DUPLICATED:   {}",
        lists.unplaced.len(),
        lists.duplicated.len()
    )?;
    writeln!(
        out,
        "MISCLAIMED:      {}     FRAMEWORK MISUSE: {}",
        lists.misclaimed.len(),
        lists.misused.len()
    )?;
    Ok(())
}

/// Print the counters that read the derive and reachability rules.
fn print_derives<W: io::Write>(out: &mut W, counts: &Counts, lists: &Lists) -> anyhow::Result<()> {
    writeln!(
        out,
        "COPY MISSING:    {}     COPY IMPOSSIBLE:  {}",
        lists.copy_missing.len(),
        lists.copy_impossible.len()
    )?;
    writeln!(
        out,
        "COPY UNDECIDED:  {}     UNKNOWN: {}     JUSTIFIED: {}     UNJUSTIFIED: {}",
        lists.copy_undecided.len(),
        lists.copy_unknown.len(),
        counts.justified,
        lists.unjustified.len()
    )?;
    writeln!(
        out,
        "DERIVE CLOSURE:  {}     DERIVE UNDECIDED: {}",
        lists.closure_broken.len(),
        lists.closure_undecided.len()
    )?;
    writeln!(
        out,
        "REACH BAD:       {}     CONST REACH BAD: {}",
        lists.reach_misses.len(),
        lists.const_reach.len()
    )?;
    writeln!(
        out,
        "LIMIT ROWS:      {}     LIMIT BAD: {}",
        counts.limit_rows,
        lists.limit_bad.len()
    )?;
    writeln!(
        out,
        "PROBE ROWS:      {}     PROBE BAD: {}",
        counts.probe_rows,
        lists.probe_bad.len()
    )?;
    writeln!(
        out,
        "RULE IDS:        {}     LOCK PHASES: {}     LOCK BAD: {}     PIN ROWS: {}     PIN BAD: {}",
        counts.rule_ids,
        counts.lock_phases,
        lists.lock_bad.len(),
        counts.pin_rows,
        lists.pin_bad.len()
    )?;
    writeln!(
        out,
        "MEMBER BLOCKS:   {}     MEMBER ROWS: {}     MEMBER BAD: {}",
        counts.member_blocks,
        counts.member_rows,
        lists.member_bad.len()
    )?;
    writeln!(
        out,
        "BUDGET ROWS:     {}     USED BY BAD: {}",
        counts.budget_rows,
        lists.used_bad.len()
    )?;
    Ok(())
}

/// Print the counters that read the expectation, size, and audio rules.
fn print_sizes<W: io::Write>(out: &mut W, counts: &Counts, lists: &Lists) -> anyhow::Result<()> {
    writeln!(
        out,
        "EXPECTATIONS:    {}     B.1 ROWS: {}     B.1 BAD: {}",
        counts.expectations,
        counts.b1_rows,
        lists.expectation_misses.len()
    )?;
    writeln!(
        out,
        "VARIANT SITES:   {}     B.1 VARIANT ROWS: {}",
        counts.variant_sites, counts.b1_variant_rows
    )?;
    writeln!(
        out,
        "REASON SIZES:    {}     REASON BAD: {}",
        counts.reason_sizes,
        lists.reason_bad.len()
    )?;
    writeln!(
        out,
        "VR1 ROWS:        {}     VR1 BAD: {}",
        counts.vr1_rows,
        lists.vr1_misses.len()
    )?;
    writeln!(
        out,
        "EQ MISSING:     {}     EQ UNDECIDED: {}",
        lists.eq_missing.len(),
        lists.eq_undecided.len()
    )?;
    writeln!(
        out,
        "SIZES DECIDED:  {}     SIZE UNDECIDED: {}     VARIANT SPREAD BAD: {}     VARIANT EXPECTED: {}",
        lists.size_rows.len(),
        lists.size_undecided.len(),
        lists.spread_bad.len(),
        lists.spread_expected.len()
    )?;
    writeln!(
        out,
        "AUDIO OWNED:     {}     AUDIO EXEMPT: {}     AUDIO DEFERRED: {}     HEAP IN AUDIO: {}     DROP IMPLS: {}",
        counts.audio_owned,
        counts.audio_exempt,
        counts.deferred,
        lists.heap_misses.len(),
        counts.drop_impls
    )?;
    writeln!(
        out,
        "GROW IN AUDIO:   {}     LOCK IN AUDIO: {}     AUDIO READ ONLY: {}     ROOT BAD: {}",
        lists.grow_misses.len(),
        lists.lock_misses.len(),
        counts.readonly,
        lists.root_bad.len()
    )?;
    writeln!(
        out,
        "AUDIO REACHABLE: {}     REACHABLE LEAVES: {}     CLOSURE BAD: {}",
        counts.audio_reachable,
        counts.reachable_leaves,
        lists.closure_bad.len()
    )?;
    Ok(())
}

/// Print the counters that read the graph, the plan, and the carrier table.
fn print_plan<W: io::Write>(out: &mut W, counts: &Counts, lists: &Lists) -> anyhow::Result<()> {
    writeln!(
        out,
        "EDGES PARSED:    {}    EDGE CLAIMS BAD: {}",
        counts.edges_parsed,
        lists.edge_misses.len()
    )?;
    writeln!(
        out,
        "DEP ROWS:        {}    DEP PROVEN: {}    DEP MISSING: {}",
        counts.dep_rows,
        counts.dep_proven,
        lists.dependency_misses.len()
    )?;
    writeln!(
        out,
        "SNAPSHOTS:       {}     SNAPSHOT BAD: {}",
        counts.snapshots,
        lists.snapshot_misses.len()
    )?;
    writeln!(
        out,
        "REGISTER BAD:    {}     FLOOR SLACK: {}     VALUE ROWS: {}     VALUE FLOOR: {VALUE_FLOOR}     VALUE SKIPPED: {}     VALUE BAD: {}",
        lists.register_misses.len(),
        lists.floor_bad.len(),
        counts.value_rows,
        counts.value_skipped,
        lists.value_bad.len()
    )?;
    writeln!(
        out,
        "TABLE ROWS:      {}     RAGGED ROWS: {}     INDEX IDS: {}     INDEX BAD: {}",
        counts.table_rows,
        lists.ragged_bad.len(),
        counts.index_ids,
        lists.index_bad.len()
    )?;
    writeln!(
        out,
        "LINE CHUNKS:     {}     CHUNK CRATE BAD: {}     LINE CHUNK FLOOR: {LINE_CHUNK_FLOOR}     PAIR CEILING: {EXEMPT_CEILING}",
        counts.line_chunks,
        lists.chunk_crate_bad.len()
    )?;
    writeln!(
        out,
        "CARRIERS:        {}     CARRIER FIELDS: {}     CARRIER BAD: {}",
        counts.carriers,
        counts.carrier_fields,
        lists.carrier_bad.len()
    )?;
    for (subject, reason) in &lists.carrier_bad {
        writeln!(out, "  CARRIER:    {subject}: {reason}")?;
    }
    writeln!(
        out,
        "TESTS SELECTED:  {}     TEST ROWS BAD: {}",
        counts.tests_selected,
        lists.test_misses.len()
    )?;
    writeln!(
        out,
        "PLAN LINKS:      {}     LINK BAD: {}",
        counts.plan_links,
        lists.link_bad.len()
    )?;
    writeln!(
        out,
        "PHASE PAIRS:     {}     PAIR EXEMPT: {}     PAIR BAD: {}     TAIL BAD: {}",
        counts.phase_pairs,
        counts.pair_exempt,
        lists.pair_bad.len(),
        lists.tail_bad.len()
    )?;
    writeln!(
        out,
        "B.1 SITES:       {}     2.3 LISTED: {}     SUPPRESSION BAD: {}     ASSERTED ROOTS: {}     ASSERTED BAD: {}",
        counts.b1_sites,
        counts.listed_23,
        lists.b1_bad.len(),
        counts.asserted_roots,
        lists.asserted_bad.len()
    )?;
    Ok(())
}

/// Print every decided size, every arm size, and every field with no size.
///
/// `ROSTER_SIZES=1` turns the list on. The mode changes no exit code.
fn print_roster<W: io::Write>(out: &mut W, lists: &Lists) -> anyhow::Result<()> {
    if std::env::var("ROSTER_SIZES").ok().as_deref() != Some("1") {
        return Ok(());
    }
    for (name, crate_name, size, align, arm_sizes) in &lists.size_rows {
        writeln!(
            out,
            "  SIZE:       {crate_name}::{name} = {size} (align {align})"
        )?;
        for (arm_name, arm_size) in arm_sizes {
            writeln!(
                out,
                "  ARM:        {crate_name}::{name}::{arm_name} = {arm_size}"
            )?;
        }
    }
    for (name, crate_name, reasons) in &lists.size_undecided {
        for (field, expr) in reasons {
            writeln!(out, "  NO SIZE:    {crate_name}::{name}: {field} = {expr}")?;
        }
    }
    Ok(())
}

/// Print every finding the register and placement rules produced.
fn print_placement<W: io::Write>(
    out: &mut W,
    lists: &Lists,
    owned: &BTreeMap<String, String>,
) -> anyhow::Result<()> {
    for (name, crate_name) in &lists.placeholders {
        writeln!(
            out,
            "  PLACEHOLDER: {crate_name}::{name} has a comment where a field belongs"
        )?;
    }
    for (block_id, row, reason) in &lists.member_bad {
        writeln!(out, "  MEMBER:     {block_id}: {row}: {reason}")?;
    }
    for (budget, reason) in &lists.used_bad {
        writeln!(out, "  USED BY:    {budget}: {reason}")?;
    }
    for (label, reason) in &lists.ragged_bad {
        writeln!(out, "  RAGGED:     {label}: {reason}")?;
    }
    for (name, reason) in &lists.index_bad {
        writeln!(out, "  INDEX:      {name}: {reason}")?;
    }
    for (chunk, reason) in &lists.chunk_crate_bad {
        writeln!(out, "  CHUNK CRATE:{chunk}: {reason}")?;
    }
    for name in &lists.undeclared {
        writeln!(
            out,
            "  UNDECLARED: {name} is placed by 1.5 and no Rust block declares it"
        )?;
    }
    for (name, token) in &lists.external_misses {
        writeln!(
            out,
            "  EXTERNAL:   {name} names {token}, which no 1.9 row decides"
        )?;
    }
    for name in &lists.unplaced {
        writeln!(out, "  UNPLACED:   {name}")?;
    }
    for name in &lists.duplicated {
        writeln!(out, "  DUPLICATED: {name}")?;
    }
    for name in &lists.misclaimed {
        let claimed = owned.get(name).cloned().unwrap_or_default();
        writeln!(out, "  MISCLAIMED: {name} claimed by {claimed}")?;
    }
    for (name, crate_name, token) in &lists.misused {
        writeln!(out, "  FRAMEWORK:  {crate_name}::{name} holds {token}")?;
    }
    Ok(())
}

/// Print every finding the derive and expectation rules produced.
fn print_traits<W: io::Write>(out: &mut W, lists: &Lists) -> anyhow::Result<()> {
    for (name, crate_name) in &lists.copy_missing {
        writeln!(
            out,
            "  COPY:       {crate_name}::{name} needs Copy or an #[expect]"
        )?;
    }
    for (name, crate_name, field) in &lists.copy_impossible {
        writeln!(
            out,
            "  NOT COPY:   {crate_name}::{name} derives Copy over {field}"
        )?;
    }
    for (name, crate_name, field) in &lists.copy_undecided {
        writeln!(
            out,
            "  NOT DECIDED:{crate_name}::{name} derives Copy over {field}"
        )?;
    }
    for (name, field) in &lists.unjustified {
        writeln!(out, "  UNJUSTIFIED:{name}.{field} is in no section 1.9 row")?;
    }
    for (name, crate_name, one, field, expr) in &lists.closure_broken {
        writeln!(
            out,
            "  CLOSURE:    {crate_name}::{name} derives {one} over {field}: {expr} has no {one}"
        )?;
    }
    for (name, crate_name, field, expr) in &lists.closure_undecided {
        writeln!(out, "  UNREADABLE: {crate_name}::{name}.{field}: {expr}")?;
    }
    for (crate_name, name, token, target) in &lists.reach_misses {
        writeln!(
            out,
            "  REACH:      {crate_name}::{name} holds {token}, which lives in {target}"
        )?;
    }
    for (name, reason) in &lists.expectation_misses {
        writeln!(out, "  B.1:        {name} {reason}")?;
    }
    for (site, count) in &lists.reason_bad {
        writeln!(
            out,
            "  REASON:     {site} states {count} bytes as a literal"
        )?;
    }
    Ok(())
}

/// Print every finding the audio and plan rules produced.
fn print_audio<W: io::Write>(out: &mut W, lists: &Lists) -> anyhow::Result<()> {
    for (root, crate_name, expr, path) in &lists.heap_misses {
        writeln!(
            out,
            "  HEAP:       {crate_name}::{root} holds {expr} through {path}"
        )?;
    }
    for (name, reason) in &lists.root_bad {
        writeln!(out, "  ROOT:       {name}: {reason}")?;
    }
    for (name, reason) in &lists.closure_bad {
        writeln!(out, "  CLOSURE:    {name}: {reason}")?;
    }
    for (name, reason) in &lists.asserted_bad {
        writeln!(out, "  ASSERTED:   {name}: {reason}")?;
    }
    for (name, reason) in &lists.pair_bad {
        writeln!(out, "  PAIR:       {name}: {reason}")?;
    }
    for (name, reason) in &lists.tail_bad {
        writeln!(out, "  TAIL:       {name}: {reason}")?;
    }
    for (name, reason) in &lists.b1_bad {
        writeln!(out, "  SUPPRESS:   {name}: {reason}")?;
    }
    for (link, reason) in &lists.link_bad {
        writeln!(out, "  LINK:       {link}: {reason}")?;
    }
    for (root, crate_name, expr, path) in &lists.grow_misses {
        writeln!(
            out,
            "  GROW:       {crate_name}::{root} holds {expr} through {path}"
        )?;
    }
    for (root, crate_name, expr, path) in &lists.lock_misses {
        writeln!(
            out,
            "  LOCK:       {crate_name}::{root} holds {expr} through {path}"
        )?;
    }
    Ok(())
}

/// Print every finding the remaining rules produced.
fn print_rest<W: io::Write>(out: &mut W, lists: &Lists) -> anyhow::Result<()> {
    for (name, crate_name, carried) in &lists.vr1_misses {
        writeln!(
            out,
            "  VR1:        {crate_name}::{name} derives {} with no VR1 row",
            carried.join(", ")
        )?;
    }
    for (name, crate_name) in &lists.eq_missing {
        writeln!(
            out,
            "  EQ:         {crate_name}::{name} derives PartialEq and every field supplies Eq"
        )?;
    }
    for (name, crate_name, arm_name, largest, second) in &lists.spread_bad {
        writeln!(
            out,
            "  VARIANT:    {crate_name}::{name}: arm {arm_name} is {largest} bytes and the \
next largest is {second}"
        )?;
    }
    for (subject, target, sentence) in &lists.edge_misses {
        let short: String = sentence.chars().take(110).collect();
        writeln!(out, "  EDGE MISS:  {subject} -> {target}: {short}")?;
    }
    for (crate_name, name, field, token, target) in &lists.const_reach {
        writeln!(
            out,
            "  CONST:      {crate_name}::{name}.{field} names {token}, which lives in {target}"
        )?;
    }
    for (constant, crate_name, reason) in &lists.limit_bad {
        writeln!(out, "  LIMIT:      {constant}: {crate_name}: {reason}")?;
    }
    for (rule, reason) in &lists.probe_bad {
        writeln!(out, "  PROBE:      {rule}: {reason}")?;
    }
    for (block_id, reason) in &lists.floor_bad {
        writeln!(out, "  FLOOR:      {block_id}: {reason} (PG27b)")?;
    }
    for (budget, reason) in &lists.value_bad {
        writeln!(out, "  VALUE:      {budget}: {reason}")?;
    }
    for (site, reason) in &lists.lock_bad {
        writeln!(out, "  LOCK SEQ:   {site}: {reason}")?;
    }
    for (pin, reason) in &lists.pin_bad {
        writeln!(out, "  PIN OWNER:  {pin}: {reason}")?;
    }
    for (crate_name, needed, name) in &lists.dependency_misses {
        writeln!(
            out,
            "  DEP MISS:   {crate_name} uses {needed} through {name}"
        )?;
    }
    for (name, reason) in &lists.snapshot_misses {
        writeln!(out, "  SNAPSHOT:   {name}: {reason}")?;
    }
    for (crate_name, name, section) in &lists.register_misses {
        writeln!(
            out,
            "  REGISTER:   {crate_name} declares {name} in {section}, which its cell omits"
        )?;
    }
    for (name, reason) in &lists.test_misses {
        writeln!(out, "  TEST MISS:  {name}: {reason}")?;
    }
    Ok(())
}

/// Print the whole report, counters first and findings after.
fn report<W: io::Write>(
    out: &mut W,
    counts: &Counts,
    lists: &Lists,
    owned: &BTreeMap<String, String>,
) -> anyhow::Result<()> {
    print_registers(out, counts, lists)?;
    print_derives(out, counts, lists)?;
    print_sizes(out, counts, lists)?;
    print_plan(out, counts, lists)?;
    print_roster(out, lists)?;
    print_placement(out, lists, owned)?;
    print_traits(out, lists)?;
    print_audio(out, lists)?;
    print_rest(out, lists)?;
    Ok(())
}

/// Run the placement guard over the architecture document.
///
/// The guard exits 0 when it finds nothing, 1 when it finds at least one
/// breach, and 2 when it cannot decide. PG2 makes a document that does not open
/// exit 2, so the guard is fail-closed on its own input.
///
/// # Errors
/// Returns an error when a write to the output stream fails.
pub(crate) fn run(document: &Path) -> anyhow::Result<Outcome> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let Ok(text) = fs::read_to_string(document) else {
        writeln!(
            out,
            "FAIL: cannot open {}; the guard is fail-closed.",
            document.display()
        )?;
        return Ok(Outcome::FailClosed);
    };
    let source = pattern::chars(&text);
    let inputs = match load_inputs(&source)? {
        Ok(inputs) => inputs,
        Err(stop) => return print_stop(&mut out, &stop),
    };
    let (counts, lists) = match analyse(document, &source, &text, &inputs)? {
        Ok(found) => found,
        Err(stop) => return print_stop(&mut out, &stop),
    };
    report(&mut out, &counts, &lists, &inputs.owned)?;
    if counts.candidates == 0 {
        writeln!(
            out,
            "FAIL: the candidate set is empty, so the parse is broken."
        )?;
        return Ok(Outcome::Findings);
    }
    Ok(if lists.any() {
        Outcome::Findings
    } else {
        Outcome::Clean
    })
}

/// Print every fail-closed line and report the exit code the guard returns.
fn print_stop<W: io::Write>(out: &mut W, stop: &FailClosed) -> anyhow::Result<Outcome> {
    for line in &stop.lines {
        writeln!(out, "{line}")?;
    }
    Ok(Outcome::FailClosed)
}

mod sync_floors {
    //! Write every guard-block floor from the row count the block itself holds.
    //!
    //! `PG27b` refuses a floor below the row count, so a floor is a measurement and
    //! never a hand number (critic C20-W10, C20-N5). The Python prototype held
    //! three copies of each number: the `rows>=` marker of the document, the
    //! register of `placement_check.py`, and the register of
    //! `roster_compile.sh`. This port leaves ONE register, [`DATA_BLOCKS`] above,
    //! so the only copy left to write is the marker inside the document.
    //!
    //! [`floors`] is the measurement `PG27b` reads. [`write()`] is the whole tool: it
    //! rewrites every marker in place and reports each floor it changed.

    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;

    use super::{BlockSpec, Blocks, DATA_BLOCKS, Floors, MARKER_PATTERN, pattern, read_block};

    /// The row count of one block, read with the register test switched off.
    ///
    /// The register test is what `PG27b` decides, so the measurement cannot depend
    /// on it: a block whose marker and register disagree must still count.
    fn counted(source: &[char], spec: &BlockSpec) -> anyhow::Result<Result<usize, String>> {
        Ok(read_block(source, spec, false)?.map(|rows| rows.len()))
    }

    /// The floor every parsed block should state, from its own row count.
    pub(super) fn floors(blocks: &Blocks) -> Floors {
        blocks.iter().map(|(id, rows)| (id, rows.len())).collect()
    }

    /// Every floor the document should state, or the reason a block does not read.
    fn measured(source: &[char]) -> anyhow::Result<Result<Floors, Vec<String>>> {
        let mut counts = BTreeMap::new();
        let mut bad = Vec::new();
        for spec in DATA_BLOCKS {
            match counted(source, spec)? {
                Ok(count) => {
                    counts.insert(spec.id, count);
                },
                Err(reason) => {
                    bad.push(format!(
                        "FAIL: the `{}` block does not read: {reason}",
                        spec.id
                    ));
                },
            }
        }
        if bad.is_empty() {
            Ok(Ok(counts))
        } else {
            Ok(Err(bad))
        }
    }

    /// The document with every marker rewritten, and one line per changed floor.
    fn rewritten(
        label: &str,
        source: &[char],
        counts: &Floors,
    ) -> anyhow::Result<(String, Vec<String>)> {
        let marker = pattern::build(MARKER_PATTERN)?;
        let mut changed = Vec::new();
        let text = marker.replace_all(source, &mut |one, whole| {
            let block_id = one.text(1, whole);
            let stated = one.text(2, whole);
            let Some((id, count)) = counts.get_key_value(block_id.as_str()) else {
                return one.text(0, whole);
            };
            if stated != count.to_string() {
                changed.push(format!("{label}: {id} {stated} -> {count}"));
            }
            format!("<!-- GUARD BLOCK id={id} rows>={count} -->")
        });
        Ok((text, changed))
    }

    /// Write every floor into the document, and report what the run prints.
    ///
    /// It returns the lines the tool prints and the exit outcome, which is 2 on a
    /// document that does not open and on a block that does not read.
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "the guard reads floors and never writes them, so `run` never calls the \
writer; the unit test below is what exercises it"
        )
    )]
    pub(super) fn write(document: &Path) -> anyhow::Result<(Vec<String>, super::Outcome)> {
        let label = document.display().to_string();
        let Ok(text) = fs::read_to_string(document) else {
            return Ok((
                vec![format!(
                    "FAIL: cannot open {label}; the guard is fail-closed."
                )],
                super::Outcome::FailClosed,
            ));
        };
        let source = pattern::chars(&text);
        let counts = match measured(&source)? {
            Ok(counts) => counts,
            Err(bad) => return Ok((bad, super::Outcome::FailClosed)),
        };
        let (written, mut lines) = rewritten(&label, &source, &counts)?;
        fs::write(document, written)?;
        let report = format!(
            "FLOORS WRITTEN: {}     CHANGED: {}",
            counts.len(),
            lines.len()
        );
        lines.push(report);
        Ok((lines, super::Outcome::Clean))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use std::fmt::Write as _;

    use super::pattern;
    use super::{
        BlockKind, DATA_BLOCKS, Outcome, Row, cell_count, contains_word, expand_index_ids,
        expr_head, holds_exclusive, layout_fields, leaf_names, normalize_crate, parse_type,
        round_up, run, split_row, split_top, sync_floors, trim_set,
    };

    /// A scratch directory of this test alone.
    fn scratch(label: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "check-placement-{label}-{}-{nanos}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).expect("scratch directory");
        dir
    }

    /// One document that holds every registered block at its own floor.
    fn synthetic_document() -> String {
        let mut out = String::new();
        out.push_str("# A synthetic document\n\n");
        for spec in DATA_BLOCKS {
            write!(
                out,
                "#### {}\n\n<!-- GUARD BLOCK id={} rows>={} -->\n",
                spec.heading, spec.id, spec.minimum
            )
            .expect("the fixture builds");
            out.push_str(&block_body(spec));
        }
        out
    }

    /// The body of one registered block, at its own floor.
    fn block_body(spec: &super::BlockSpec) -> String {
        let mut out = String::new();
        if spec.kind == BlockKind::Table {
            out.push_str("| a | b | c | d | e | f |\n|---|---|---|---|---|---|\n");
            let rows = (0..spec.minimum).map(|index| format!("| r{index} | b | c | d | e | f |\n"));
            out.extend(rows);
            out.push('\n');
            return out;
        }
        writeln!(out, "```{}", spec.kind.fence()).expect("the fixture builds");
        out.extend((0..spec.minimum).map(|index| format!("row{index} value\n")));
        out.push_str("```\n\n");
        out
    }

    #[test]
    fn the_matcher_reads_the_forms_the_rules_use() {
        let cases: [(&str, &str, &str); 8] = [
            (r"(?<!\\)\|", "a|b\\|c", "1:2"),
            (r"(?m)^#### (.+)$", "x\n#### Heading\ny", "2:14"),
            (r"(?s)```rust\n(.*?)```", "```rust\nlet a = 1;\n```", "0:22"),
            (r"\b([A-Z][A-Za-z0-9]*)\b", "one Two three", "4:7"),
            (r"(?<![\w.])B(\d+)(?![\w])", "see B12 here", "4:7"),
            (
                r"[A-Z]\d{1,2}(?: and [A-Z]\d{1,2})* before [A-Z]\d{1,2}",
                "A1 and A2 before B3",
                "0:19",
            ),
            (r"(?:dyn|impl)\b\s*", "dyn AudioProcess", "0:4"),
            (r"#\[derive\(([^)]*)\)\]", "#[derive(Clone, Copy)]", "0:22"),
        ];
        for (expression, subject, wanted) in cases {
            let compiled = pattern::build(expression).expect("the pattern compiles");
            let text = pattern::chars(subject);
            let found = compiled.find(&text).expect("the subject holds a match");
            assert_eq!(
                format!("{}:{}", found.start(), found.end()),
                wanted,
                "`{expression}` matches the recorded span of `{subject}`"
            );
        }
    }

    #[test]
    fn a_class_and_an_anchor_refuse_what_they_should() {
        let compiled = pattern::build(r"(?m)^\| ([A-Z]{1,2}\d{1,2}) \| \d+ \|").expect("compiles");
        let text = pattern::chars("prose | A2 | 3 |\n| A2 | 3 | goal |");
        let found = compiled.find_iter(&text);
        assert_eq!(found.len(), 1, "the line anchor refuses the inline row");
        let escaped = pattern::build(r"(?<!\\)\|").expect("compiles");
        let cells = escaped.split(&pattern::chars("| a \\| b | c |"));
        assert_eq!(cells.len(), 4, "an escaped pipe is not a cell wall");
    }

    #[test]
    fn a_table_row_splits_the_way_a_renderer_reads_it() {
        let cells = split_row("| `a\\|b` | c |").expect("the row splits");
        assert_eq!(
            cells,
            vec!["`a|b`", "c"],
            "an escaped pipe unescapes to one pipe"
        );
        assert_eq!(
            cell_count("| `a\\|b` | c |").expect("the row counts"),
            2,
            "the count skips the escaped pipe"
        );
    }

    #[test]
    fn the_head_of_a_field_expression_is_its_outermost_name() {
        let cases = [
            ("Box<dyn AudioProcess>", Some("Box")),
            ("basedrop::Owned<GraphState>", Some("Owned")),
            ("&'a mut [f32]", None),
            ("(u32, u32)", None),
            ("[u8; 4]", None),
        ];
        for (expression, wanted) in cases {
            assert_eq!(
                expr_head(expression).as_deref(),
                wanted,
                "`{expression}` states this head"
            );
        }
    }

    #[test]
    fn an_exclusive_reference_is_not_a_shared_one() {
        let shared = parse_type("&'a str").expect("the expression parses");
        let exclusive = parse_type("&'buffers mut [f32]").expect("the expression parses");
        assert!(
            !holds_exclusive(shared.as_ref()),
            "a shared reference holds no exclusive reference"
        );
        assert!(
            holds_exclusive(exclusive.as_ref()),
            "an exclusive reference reaches the rule that refuses a Copy derive"
        );
        let applied = parse_type("BTreeMap<NoteId, Span>").expect("the expression parses");
        let names = leaf_names(applied.as_ref());
        assert!(names.contains("BTreeMap"), "the head is a leaf name");
        assert!(names.contains("NoteId"), "an argument is a leaf name");
        assert_eq!(names.len(), 3, "the expression names three types");
    }

    #[test]
    fn a_const_generic_argument_is_a_value_and_not_a_type() {
        let parsed = parse_type("ArrayVec<SlotSpec, MAX_SLOTS>").expect("the expression parses");
        let names = leaf_names(parsed.as_ref());
        assert!(
            !names.contains("MAX_SLOTS"),
            "a section 1.6 constant is no type of the expression"
        );
        assert!(names.contains("SlotSpec"), "the payload stays a type");
    }

    #[test]
    fn a_body_lays_out_by_decreasing_alignment() {
        assert_eq!(
            round_up(9, 4),
            12,
            "nine rounds up to the next multiple of four"
        );
        assert_eq!(round_up(8, 1), 8, "an alignment of one changes nothing");
        assert_eq!(
            layout_fields(&[(1, 1), (8, 8)]),
            (16, 8),
            "the wider field sorts first and the total rounds up"
        );
        assert_eq!(layout_fields(&[]), (0, 1), "a zero-field body is size zero");
    }

    #[test]
    fn a_rule_index_range_expands_both_ways() {
        let numeric = expand_index_ids("PG1 to PG3").expect("the cell reads");
        assert_eq!(
            numeric.len(),
            3,
            "a numeric range names every id between its ends"
        );
        assert!(numeric.contains("PG2"), "the range holds its middle id");
        let lettered = expand_index_ids("PG26b to PG26d").expect("the cell reads");
        assert_eq!(
            lettered.len(),
            3,
            "a letter range names every id between its ends"
        );
        assert!(lettered.contains("PG26c"), "the range holds its middle id");
    }

    #[test]
    fn a_word_match_reads_the_boundary_a_pattern_would() {
        assert!(contains_word("the B113 row", "B113"), "a bare id matches");
        assert!(
            !contains_word("the B1130 row", "B113"),
            "a longer id does not match"
        );
        assert!(
            !contains_word("the XB113 row", "B113"),
            "a prefixed id does not match"
        );
    }

    #[test]
    fn a_top_level_split_ignores_a_nested_separator() {
        let pieces = split_top("a: Map<K, V>, b: u8", ',');
        assert_eq!(
            pieces.len(),
            2,
            "a comma inside brackets is not a separator"
        );
        assert_eq!(
            trim_set("`*Audio*`", "`*"),
            "Audio",
            "the trim set removes every leading and trailing marker"
        );
        assert_eq!(
            normalize_crate("crates/duet"),
            "duet",
            "the 1.5 spelling of the application crate maps to the 1.3 spelling"
        );
    }

    #[test]
    fn a_row_reads_as_cells_or_as_one_line() {
        let cells = Row::Cells(vec!["a".to_owned(), "b".to_owned()]);
        assert_eq!(cells.text(), "a | b", "a table row joins its cells");
        assert!(
            cells.tokens().is_empty(),
            "a table row states no fenced tokens"
        );
        let line = Row::Line("one two".to_owned());
        assert_eq!(line.tokens().len(), 2, "a fenced line splits on whitespace");
        assert_eq!(line.cell(0), "", "a fenced line states no cell");
    }

    #[test]
    fn a_document_that_does_not_open_is_fail_closed() {
        let dir = scratch("missing");
        let outcome = run(&dir.join("no-such-document.md")).expect("the guard writes its line");
        assert_eq!(
            outcome,
            Outcome::FailClosed,
            "PG2 makes a document that does not open exit 2"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn the_floor_writer_reports_a_block_that_does_not_read() {
        let dir = scratch("nofloors");
        let document = dir.join("architecture.md");
        std::fs::write(&document, "# A document with no registered block\n").expect("write");
        let (lines, outcome) = sync_floors::write(&document).expect("the writer runs");
        assert_eq!(
            outcome,
            Outcome::FailClosed,
            "a block that does not read is fail-closed"
        );
        assert_eq!(
            lines.len(),
            DATA_BLOCKS.len(),
            "the writer names every block it cannot read"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }

    #[test]
    fn the_floor_writer_writes_the_row_count_it_measures() {
        let dir = scratch("floors");
        let document = dir.join("architecture.md");
        let text = synthetic_document();
        std::fs::write(&document, &text).expect("write");
        let (lines, outcome) = sync_floors::write(&document).expect("the writer runs");
        assert_eq!(outcome, Outcome::Clean, "every block of the fixture reads");
        assert_eq!(
            lines.last().map(String::as_str),
            Some(format!("FLOORS WRITTEN: {}     CHANGED: 0", DATA_BLOCKS.len()).as_str()),
            "a document already at its floors changes nothing"
        );
        let stale = text.replace(
            "<!-- GUARD BLOCK id=drop-list rows>=8 -->",
            "<!-- GUARD BLOCK id=drop-list rows>=1 -->",
        );
        std::fs::write(&document, stale).expect("write");
        let (changed, second) = sync_floors::write(&document).expect("the writer runs");
        assert_eq!(
            second,
            Outcome::Clean,
            "a stale floor is not a read failure"
        );
        assert!(
            changed
                .iter()
                .any(|line| line.ends_with("drop-list 1 -> 8")),
            "the writer reports the floor it raised"
        );
        let written = std::fs::read_to_string(&document).expect("read back");
        assert!(
            written.contains("<!-- GUARD BLOCK id=drop-list rows>=8 -->"),
            "the marker now states the row count the block holds"
        );
        std::fs::remove_dir_all(&dir).expect("cleanup");
    }
}
