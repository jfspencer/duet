//! Refuse a closure block that its own review file does not support (`PG32`).
//!
//! Appendix C of `architecture.md` carries one closure row per finding of each
//! review. This guard reads one registered closure block and holds it against
//! the review file the block records. It is a port of the prototype
//! `roadmap/duet-v1/tools/closure_check.py`, and it keeps every rule, every
//! printed line, and every exit code that prototype produces.
//!
//! The rule has six parts.
//!
//! - `CL1` takes the id list from the review's own headings. No id in this
//!   guard and no id in the document is typed. The `review_ids` module below
//!   is the port of `roadmap/duet-v1/tools/review_ids.py`.
//! - `CL2` gives every generated id exactly one row of the registered block,
//!   and every row names a generated id.
//! - `CL3` holds the document against the count sentence this guard builds
//!   from the generated counts, character for character.
//! - `CL4` reads every row for a non-empty finding cell, a state cell from the
//!   three words `CLOSED`, `PARTIAL` and `OPEN`, and a section that a heading
//!   of this document holds when the row says `CLOSED`.
//! - `CL1c` reads the stored copy of the review back from the plan store and
//!   compares its md5 with the file's. There is no way to skip this half: a
//!   run that reaches the end with the half unrun is a failure that says so.
//! - `CL5` holds the digest of the review file this run was handed against the
//!   digest the document records beside the block.
//!
//! The guard exits 2 on a usage or input failure, 1 on a finding, and 0 when
//! the block is complete against the review.
//!
//! The prototype takes a fourth `repo-root` argument, which names the
//! repository that holds `.claude/plan-coordination/db.sh`. This subcommand
//! takes three arguments, so it resolves that repository with the `repo_root`
//! helper of `main.rs`, which is the explicit compile-time answer for a
//! repository-local run. The prototype states that the path is explicit and
//! never an environment variable, and this port reads no environment variable
//! for it.
//!
//! The plan name `db.sh` reads is `roadmap/<name>`, where `<name>` is the
//! directory that holds the document. The register below holds the nine
//! closure blocks alone, because this guard decides a closure block and no
//! other kind of block.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command as Process;

use anyhow::Context as _;
use md5::{Digest as _, Md5};

use crate::Outcome;

/// The three states a closure row may state (`CL4`).
const STATES: [&str; 3] = ["CLOSED", "PARTIAL", "OPEN"];

/// The first revision whose review must carry the count sentence.
const COUNTS_REQUIRED_FROM: u32 = 20;

/// The sentence head that binds a block to its stored copy (`CL1c`).
const STORE_HEAD: &str = "The review file this block records is stored at plan-store key `";

/// One registered closure block.
#[derive(Debug, Clone, Copy)]
struct Block {
    /// The block id the marker line states.
    id: &'static str,
    /// The `####` heading the block carries.
    heading: &'static str,
}

/// Every registered closure block, as `placement_check.py` registers it.
const DATA_BLOCKS: [Block; 9] = [
    Block {
        id: "closure-r16",
        heading: "The revision-16 review, over the frozen document",
    },
    Block {
        id: "closure-r17",
        heading: "The revision-17 review, over the frozen document",
    },
    Block {
        id: "closure-r18",
        heading: "The revision-18 review, over the frozen document",
    },
    Block {
        id: "closure-r19",
        heading: "The revision-19 review, over the frozen document",
    },
    Block {
        id: "closure-r20",
        heading: "The revision-20 review, over the frozen document",
    },
    Block {
        id: "closure-r21-inner",
        heading: "The revision-21 inner review, over the frozen document",
    },
    Block {
        id: "closure-r21",
        heading: "The revision-21 external review, over the frozen document",
    },
    Block {
        id: "closure-r22-inner",
        heading: "The revision-22 inner review, over the frozen document",
    },
    Block {
        id: "closure-r23-inner",
        heading: "The revision-23 inner review, over the frozen document",
    },
];

/// One line of a document, with the byte offset it opens at.
#[derive(Debug, Clone, Copy)]
struct Line<'a> {
    /// The byte offset of the first character of the line.
    start: usize,
    /// The line, without its terminator.
    text: &'a str,
    /// Whether a newline closes the line.
    terminated: bool,
}

/// What the `CL1c` store half produced.
#[derive(Debug)]
struct StoreReport {
    /// The word the summary line prints.
    state: &'static str,
    /// How many stored copies the guard read.
    copies: usize,
    /// Every `CL1c` failure, as printable lines.
    failures: Vec<String>,
}

/// Run the closure guard over one registered block of the document.
///
/// # Errors
/// Returns an error when a write to the output stream fails.
pub(crate) fn run(document: &Path, review: &Path, block: &str) -> anyhow::Result<Outcome> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    for path in [document, review] {
        if !path.is_file() {
            writeln!(
                out,
                "FAIL: cannot open {}; the guard is fail-closed.",
                path.display()
            )?;
            return Ok(Outcome::FailClosed);
        }
    }
    let Some(heading) = heading_of(block) else {
        writeln!(
            out,
            "FAIL: `{block}` is no registered block; the guard is fail-closed."
        )?;
        return Ok(Outcome::FailClosed);
    };
    let source = fs::read_to_string(document)
        .with_context(|| format!("cannot read {}", document.display()))?;
    let lines = lines_of(&source);
    let rows = match read_block(&lines, heading, block) {
        Ok(rows) => rows,
        Err(reason) => {
            writeln!(out, "FAIL: the `{block}` block of this document: {reason}")?;
            return Ok(Outcome::FailClosed);
        },
    };
    let raw = fs::read(review).with_context(|| format!("cannot read {}", review.display()))?;
    let text =
        str::from_utf8(&raw).with_context(|| format!("{} is not UTF-8", review.display()))?;
    let parsed = match generated(review, text) {
        Ok(parsed) => parsed,
        Err(problem) => {
            writeln!(
                out,
                "FAIL: {}: {problem}; the guard is fail-closed.",
                review.display()
            )?;
            return Ok(Outcome::FailClosed);
        },
    };
    let Some(window) = block_section(&source, &lines, heading) else {
        writeln!(
            out,
            "FAIL: the `{block}` block has no section of its own; the guard is fail-closed."
        )?;
        return Ok(Outcome::FailClosed);
    };
    let wanted_digest = hex_digest(&raw);
    let wanted_content = hex_digest(without_trailing_newlines(&raw));
    let mut failures = audit(window, &rows, &parsed, &headings(&lines));
    if !window.contains(&digest_sentence(block, &wanted_digest)) {
        failures.push(format!(
            "DIGEST:    the document states no md5 `{wanted_digest}` for {block} beside the block, so nothing binds this block to the file this run read"
        ));
    }
    let report = store_report(window, block, review, document, &wanted_content);
    failures.extend(report.failures);
    writeln!(
        out,
        "REVIEW: {}   BLOCK: {block}   GENERATED: {}   ROWS: {}   STORE: {}   STORE COPIES: {}   CLOSURE BAD: {}",
        name_of(review),
        parsed.ids.len(),
        rows.len(),
        report.state,
        report.copies,
        failures.len()
    )?;
    for line in &failures {
        writeln!(out, "  {line}")?;
    }
    if failures.is_empty() {
        Ok(Outcome::Clean)
    } else {
        Ok(Outcome::Findings)
    }
}

/// The heading one registered block carries, or `None`.
fn heading_of(block: &str) -> Option<&'static str> {
    DATA_BLOCKS
        .iter()
        .find(|entry| entry.id == block)
        .map(|entry| entry.heading)
}

/// The file name of one path, as the summary line prints it.
fn name_of(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

/// Every line of one text, with its byte offset.
fn lines_of(text: &str) -> Vec<Line<'_>> {
    let mut found = Vec::new();
    let mut start = 0;
    for raw in text.split_inclusive('\n') {
        let stripped = raw.strip_suffix('\n');
        found.push(Line {
            start,
            text: stripped.unwrap_or(raw),
            terminated: stripped.is_some(),
        });
        start += raw.len();
    }
    found
}

/// Whether one line is the `####` heading of one registered block.
fn is_block_heading(line: &str, heading: &str) -> bool {
    line.strip_prefix("#### ")
        .and_then(|rest| rest.strip_prefix(heading))
        .is_some_and(|tail| tail.chars().all(char::is_whitespace))
}

/// Whether one line opens a `###` section.
fn is_section_heading(line: &str) -> bool {
    line.starts_with("### ")
}

/// Whether one line opens a heading of level one to four.
fn is_any_heading(line: &str) -> bool {
    let hashes = line.chars().take_while(|mark| *mark == '#').count();
    (1..=4).contains(&hashes) && line.chars().nth(hashes) == Some(' ')
}

/// The block id and row minimum one guard marker line states.
fn marker_of(line: &str) -> Option<(&str, usize)> {
    let body = line
        .strip_prefix("<!-- GUARD BLOCK id=")?
        .strip_suffix(" -->")?;
    let (id, rows) = body.split_once(" rows>=")?;
    let named = !id.is_empty()
        && id
            .chars()
            .all(|mark| mark.is_ascii_lowercase() || mark.is_ascii_digit() || mark == '-');
    if !named || rows.is_empty() || !rows.chars().all(|mark| mark.is_ascii_digit()) {
        return None;
    }
    rows.parse().ok().map(|minimum| (id, minimum))
}

/// Whether one line is the head row of a markdown table.
fn is_table_head(line: &str) -> bool {
    let trimmed = line.trim_end_matches([' ', '\t']);
    trimmed.len() >= 2 && trimmed.starts_with('|') && trimmed.ends_with('|')
}

/// Whether one line is the rule row of a markdown table.
fn is_table_rule(line: &str) -> bool {
    let trimmed = line.trim_end_matches([' ', '\t']);
    trimmed.len() >= 3
        && trimmed.starts_with('|')
        && trimmed.ends_with('|')
        && trimmed
            .chars()
            .all(|mark| mark == '|' || mark == '-' || mark == ':' || mark.is_whitespace())
}

/// The cells of one markdown table row, as a renderer reads them.
fn split_row(line: &str) -> Vec<String> {
    let mut parts: Vec<String> = Vec::new();
    let mut current = String::new();
    let mut previous = '\0';
    for mark in line.chars() {
        if mark == '|' && previous != '\\' {
            parts.push(core::mem::take(&mut current));
        } else {
            current.push(mark);
        }
        previous = mark;
    }
    parts.push(current);
    let count = parts.len();
    if count < 2 {
        return Vec::new();
    }
    parts
        .into_iter()
        .skip(1)
        .take(count - 2)
        .map(|cell| cell.replace("\\|", "|").trim().to_owned())
        .collect()
}

/// One registered block's rows, or the reason the block cannot be read.
fn read_block(lines: &[Line<'_>], heading: &str, block: &str) -> Result<Vec<Vec<String>>, String> {
    let found: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_at, line)| is_block_heading(line.text, heading))
        .map(|(at, _line)| at)
        .collect();
    let [at] = found.as_slice() else {
        return Err(format!(
            "the heading `#### {heading}` appears {} times",
            found.len()
        ));
    };
    let region: Vec<Line<'_>> = lines
        .iter()
        .skip(at + 1)
        .take_while(|line| !is_any_heading(line.text))
        .copied()
        .collect();
    let marks: Vec<(usize, &str, usize)> = region
        .iter()
        .enumerate()
        .filter_map(|(index, line)| marker_of(line.text).map(|(id, minimum)| (index, id, minimum)))
        .collect();
    let [(index, id, minimum)] = marks.as_slice() else {
        return Err(format!(
            "the marker line appears {} times under the heading",
            marks.len()
        ));
    };
    if *id != block {
        return Err(format!(
            "the marker states id `{id}` and the register states `{block}`"
        ));
    }
    if !region.get(*index).is_some_and(|line| line.terminated) {
        return Err("the marker does not end its own line".to_owned());
    }
    let head = region
        .get(index + 1)
        .is_some_and(|line| line.terminated && is_table_head(line.text));
    let rule = region
        .get(index + 2)
        .is_some_and(|line| line.terminated && is_table_rule(line.text));
    if !head || !rule {
        return Err("no table header follows the marker".to_owned());
    }
    let rows: Vec<Vec<String>> = region
        .iter()
        .skip(index + 3)
        .take_while(|line| line.text.starts_with('|'))
        .map(|line| split_row(line.text))
        .collect();
    if rows.len() < *minimum {
        return Err(format!(
            "it holds {} rows and the stated minimum is {minimum}",
            rows.len()
        ));
    }
    Ok(rows)
}

/// The text of the section that holds one block, heading to heading.
fn block_section<'a>(source: &'a str, lines: &[Line<'_>], heading: &str) -> Option<&'a str> {
    let found: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_at, line)| is_block_heading(line.text, heading))
        .map(|(at, _line)| at)
        .collect();
    let [at] = found.as_slice() else {
        return None;
    };
    let start = lines
        .iter()
        .take(*at)
        .rfind(|line| is_section_heading(line.text))
        .map_or(0, |line| line.start);
    let end = lines
        .iter()
        .skip(at + 1)
        .find(|line| is_section_heading(line.text))
        .map_or(source.len(), |line| line.start);
    source.get(start..end)
}

/// Every section number a heading of this document states.
fn headings(lines: &[Line<'_>]) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    for line in lines {
        if let Some(token) = hash_section(line.text) {
            found.insert(token.to_owned());
        }
        if let Some(number) = numbered_heading(line.text) {
            found.insert(number.to_owned());
        }
    }
    found
}

/// The section number a level two to four heading states, or `None`.
fn hash_section(line: &str) -> Option<&str> {
    let hashes = line.chars().take_while(|mark| *mark == '#').count();
    if !(2..=4).contains(&hashes) {
        return None;
    }
    let rest = line.get(hashes..)?.strip_prefix(' ')?;
    section_token(rest)
}

/// The number a `## <n>. ` heading states, or `None`.
fn numbered_heading(line: &str) -> Option<&str> {
    let rest = line.strip_prefix("## ")?;
    let count = digit_run(rest);
    if count == 0 {
        return None;
    }
    if !rest.get(count..)?.starts_with(". ") {
        return None;
    }
    rest.get(..count)
}

/// The section token at the start of one text, or `None`.
fn section_token(text: &str) -> Option<&str> {
    numbered_token(text).or_else(|| lettered_token(text))
}

/// The `<digits>.<digits>[a-z]` token at the start of one text.
fn numbered_token(text: &str) -> Option<&str> {
    let first = digit_run(text);
    if first == 0 {
        return None;
    }
    let tail = text.get(first..)?.strip_prefix('.')?;
    let second = digit_run(tail);
    if second == 0 {
        return None;
    }
    let letter = tail
        .get(second..)
        .and_then(|after| after.chars().next())
        .is_some_and(|mark| mark.is_ascii_lowercase());
    text.get(..first + 1 + second + usize::from(letter))
}

/// The `<letter>.<digits>` token at the start of one text.
fn lettered_token(text: &str) -> Option<&str> {
    let head = text.chars().next()?;
    if !head.is_ascii_uppercase() {
        return None;
    }
    let tail = text.get(1..)?.strip_prefix('.')?;
    let digits = digit_run(tail);
    if digits == 0 {
        return None;
    }
    text.get(..2 + digits)
}

/// How many ASCII digits open one text.
fn digit_run(text: &str) -> usize {
    text.chars()
        .take_while(char::is_ascii_digit)
        .map(char::len_utf8)
        .sum()
}

/// How many whitespace characters open one text.
fn whitespace_run(text: &str) -> usize {
    text.chars()
        .take_while(|mark| mark.is_whitespace())
        .map(char::len_utf8)
        .sum()
}

/// One text with its opening whitespace removed, when it holds enough of it.
fn skip_spaces(text: &str, least: usize) -> Option<&str> {
    let count = text.chars().take_while(|mark| mark.is_whitespace()).count();
    if count < least {
        return None;
    }
    text.get(whitespace_run(text)..)
}

/// Whether one character belongs to a word.
fn is_word_char(mark: char) -> bool {
    mark.is_alphanumeric() || mark == '_'
}

/// Whether one character blocks a section citation on either side.
fn is_citation_edge(mark: char) -> bool {
    is_word_char(mark) || mark == '.' || mark == '-'
}

/// One text with every fenced block removed.
fn without_fences(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut inside = false;
    for raw in text.split_inclusive('\n') {
        let fence = raw.starts_with("```") || raw.starts_with("~~~");
        if fence {
            inside = !inside;
            if !inside {
                kept.push_str(raw.get(3..).unwrap_or_default());
            }
            continue;
        }
        if !inside {
            kept.push_str(raw);
        }
    }
    kept
}

/// One cell with every code span replaced by a space.
fn without_code_spans(text: &str) -> String {
    let mut kept = String::with_capacity(text.len());
    let mut cursor = 0;
    while let Some(rest) = text.get(cursor..) {
        let Some(mark) = rest.chars().next() else {
            break;
        };
        if let Some(length) = double_span(rest).or_else(|| single_span(rest)) {
            kept.push(' ');
            cursor += length;
            continue;
        }
        kept.push(mark);
        cursor += mark.len_utf8();
    }
    kept
}

/// The byte length of a double-backtick span at the start of one text.
fn double_span(text: &str) -> Option<usize> {
    let body = text.strip_prefix("``")?;
    let head = body.chars().next()?;
    let after = body.get(head.len_utf8()..)?;
    let at = after.find("``")?;
    let span = body.get(..head.len_utf8() + at)?;
    if span.contains('\n') {
        return None;
    }
    Some(span.len() + 4)
}

/// The byte length of a single-backtick span at the start of one text.
fn single_span(text: &str) -> Option<usize> {
    let body = text.strip_prefix('`')?;
    let at = body.find('`')?;
    Some(at + 2)
}

/// Every section number one cell cites (`CL4`).
fn cited_sections(text: &str) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    let mut previous: Option<char> = None;
    let mut cursor = 0;
    while let Some(rest) = text.get(cursor..) {
        let Some(mark) = rest.chars().next() else {
            break;
        };
        let open = previous.is_none_or(|before| !is_citation_edge(before));
        if open && let Some(token) = section_token(rest) {
            let after = rest.get(token.len()..).and_then(|tail| tail.chars().next());
            if after.is_none_or(|next| !is_citation_edge(next)) {
                found.insert(token.to_owned());
                previous = token.chars().next_back();
                cursor += token.len();
                continue;
            }
        }
        previous = Some(mark);
        cursor += mark.len_utf8();
    }
    found
}

/// The md5 of one byte run, as the document records it.
fn hex_digest(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let mut text = String::with_capacity(32);
    for byte in hasher.finalize().iter().copied() {
        let high = usize::from(byte >> 4);
        let low = usize::from(byte & 0x0f);
        if let (Some(&first), Some(&second)) = (HEX.get(high), HEX.get(low)) {
            text.push(char::from(first));
            text.push(char::from(second));
        }
    }
    text
}

/// Every digit one hexadecimal byte prints.
const HEX: [u8; 16] = *b"0123456789abcdef";

/// One byte run with its trailing newlines removed.
///
/// `db.sh append` takes the value as one argument and `db.sh get` prints it
/// with one trailing newline, so a byte compare would answer the shell's
/// quoting and not the review's content.
fn without_trailing_newlines(data: &[u8]) -> &[u8] {
    let mut end = data.len();
    while end > 0 && data.get(end - 1) == Some(&b'\n') {
        end -= 1;
    }
    data.get(..end).unwrap_or_default()
}

/// The one sentence the appendix must hold for this review (`CL3`).
fn count_sentence(criticals: usize, warnings: usize, concerns: usize, total: usize) -> String {
    format!(
        "It returned {criticals} Criticals, {warnings} Warnings, and {concerns} Concerns, and this block holds one row for each of the {total}."
    )
}

/// The one sentence that binds a block to the review file it records.
fn digest_sentence(block: &str, digest: &str) -> String {
    format!("The review file this block records has md5 `{digest}` ({block}).")
}

/// The id list and the family counts of one review, or a reason.
fn generated(review: &Path, text: &str) -> Result<review_ids::Parsed, String> {
    let named = review_ids::prefix_of_path(review).map_or_else(
        || review_ids::revision_of(text).map(|revision| format!("C{revision}")),
        Some,
    );
    let Some(prefix) = named else {
        return Err("the file name states no review and the title states no revision".to_owned());
    };
    review_ids::ids_of(text, &prefix)
}

/// The id one closure row names.
fn row_id(row: &[String]) -> String {
    row.first()
        .map(|cell| cell.trim().trim_matches('`').to_owned())
        .unwrap_or_default()
}

/// Every `CL2`, `CL3` and `CL4` failure of one block, as printable lines.
fn audit(
    window: &str,
    rows: &[Vec<String>],
    parsed: &review_ids::Parsed,
    known: &BTreeSet<String>,
) -> Vec<String> {
    let mut failures = Vec::new();
    let listed: Vec<(String, &Vec<String>)> = rows.iter().map(|row| (row_id(row), row)).collect();
    let have: BTreeSet<&str> = listed.iter().map(|(id, _row)| id.as_str()).collect();
    let wanted: BTreeSet<&str> = parsed.ids.iter().map(String::as_str).collect();
    let mut counted: BTreeMap<&str, usize> = BTreeMap::new();
    for (id, _row) in &listed {
        *counted.entry(id.as_str()).or_insert(0) += 1;
    }
    for (id, times) in counted.iter().filter(|(_id, times)| **times > 1) {
        failures.push(format!(
            "CLOSURE:   {id}: the block holds {times} rows and PG32 states one closure row per finding"
        ));
    }
    for id in &parsed.ids {
        if !have.contains(id.as_str()) {
            failures.push(format!(
                "CLOSURE:   {id}: the review states it and the block holds no row"
            ));
        }
    }
    for (id, row) in &listed {
        if let Some(line) = row_failure(id, row, &wanted, known) {
            failures.push(line);
        }
    }
    let sentence = count_sentence(
        parsed.seen.count(review_ids::Family::Critical),
        parsed.seen.count(review_ids::Family::Warning),
        parsed.seen.count(review_ids::Family::Concern),
        parsed.ids.len(),
    );
    if !window.contains(&sentence) {
        failures.push(format!(
            "COUNT:     the document does not hold the count sentence this review generates: {sentence}"
        ));
    }
    failures
}

/// The `CL2` or `CL4` failure one closure row carries, or `None`.
fn row_failure(
    id: &str,
    row: &[String],
    wanted: &BTreeSet<&str>,
    known: &BTreeSet<String>,
) -> Option<String> {
    if !wanted.contains(id) {
        return Some(format!(
            "CLOSURE:   {id}: the block holds a row and the review states no such finding"
        ));
    }
    let finding = row.get(1).map_or("", |cell| cell.trim());
    let state = row.get(2).map_or("", |cell| cell.trim().trim_matches('*'));
    let section = row.get(3).map_or("", |cell| cell.trim());
    if finding.is_empty() {
        return Some(format!("CLOSURE:   {id}: the row states no finding"));
    }
    let spoken = state.to_uppercase();
    if !STATES.contains(&spoken.as_str()) {
        return Some(format!(
            "CLOSURE:   {id}: the state is `{state}` and the three states are {}",
            STATES.join(", ")
        ));
    }
    if spoken != "CLOSED" {
        return None;
    }
    if section.is_empty() {
        return Some(format!(
            "CLOSURE:   {id}: the row says CLOSED and names no section"
        ));
    }
    closed_failure(id, section, known)
}

/// The `CL4` section failure one `CLOSED` row carries, or `None`.
fn closed_failure(id: &str, section: &str, known: &BTreeSet<String>) -> Option<String> {
    let plain = without_code_spans(section);
    let cited = cited_sections(&plain);
    if cited.is_empty() {
        return Some(format!(
            "CLOSURE:   {id}: the row says CLOSED and its section cell names no section of this document"
        ));
    }
    let unknown = cited.difference(known).next()?;
    Some(format!(
        "CLOSURE:   {id}: the row names section {unknown} and no heading of this document states it"
    ))
}

/// The plan directory as `db.sh` names it.
fn plan_name(document: &Path) -> Option<String> {
    let full = document.canonicalize().ok()?;
    let name = full.parent()?.file_name()?.to_str()?.to_owned();
    Some(format!("roadmap/{name}"))
}

/// The plan-store key and block id one closure section states (`CL1c`).
fn store_key_of(window: &str) -> Option<(&str, &str)> {
    for (at, _head) in window.match_indices(STORE_HEAD) {
        let Some(rest) = window.get(at + STORE_HEAD.len()..) else {
            continue;
        };
        let Some(length) = key_run(rest) else {
            continue;
        };
        let (Some(key), Some(tail)) = (rest.get(..length), rest.get(length..)) else {
            continue;
        };
        let Some(named) = tail.strip_prefix("` (") else {
            continue;
        };
        let Some(shown) = block_run(named) else {
            continue;
        };
        if !named.get(shown..).is_some_and(|end| end.starts_with(").")) {
            continue;
        }
        let Some(block) = named.get(..shown) else {
            continue;
        };
        return Some((key, block));
    }
    None
}

/// Whether one character may sit in a plan-store key.
const fn is_key_char(mark: char) -> bool {
    mark.is_ascii_alphanumeric() || mark == '.' || mark == '_' || mark == ':' || mark == '-'
}

/// How many bytes of one text a plan-store key opens.
fn key_run(text: &str) -> Option<usize> {
    let length: usize = text
        .chars()
        .take_while(|mark| is_key_char(*mark))
        .map(char::len_utf8)
        .sum();
    (length > 0).then_some(length)
}

/// How many bytes of one text a closure block id opens.
fn block_run(text: &str) -> Option<usize> {
    let head = "closure-";
    let body = text.strip_prefix(head)?;
    let length: usize = body
        .chars()
        .take_while(|mark| mark.is_ascii_alphanumeric() || *mark == '-')
        .map(char::len_utf8)
        .sum();
    (length > 0).then_some(head.len() + length)
}

/// The suffix one plan-store key carries.
fn key_suffix(key: &str) -> &str {
    key.split_once('-').map_or(key, |(_head, tail)| tail)
}

/// The one suffix a `critic-spec-<rev>.md` review may be stored under.
fn wanted_suffix(review: &Path) -> Option<String> {
    let name = review.file_name()?.to_str()?;
    let body = name.strip_prefix("critic-spec-")?.strip_suffix(".md")?;
    let rest = body.strip_prefix('r')?;
    let digits = digit_run(rest);
    if digits == 0 {
        return None;
    }
    let tail = rest.get(digits..)?;
    if !tail
        .chars()
        .all(|mark| mark.is_ascii_alphanumeric() || mark == '-')
    {
        return None;
    }
    Some(format!("review-{body}"))
}

/// The store launcher of one repository, or a reason.
fn launcher_of(repo: &Path) -> Result<PathBuf, String> {
    let launcher = repo.join(".claude").join("plan-coordination").join("db.sh");
    if launcher.is_file() {
        return Ok(launcher);
    }
    Err(format!(
        "the store launcher {} does not open",
        launcher.display()
    ))
}

/// Every stored key that shares one key's suffix, or a reason (`CL1c`).
fn sibling_keys(repo: &Path, plan: &str, key: &str) -> Result<Vec<String>, String> {
    let launcher = launcher_of(repo)?;
    let output = Process::new("bash")
        .arg(&launcher)
        .args(["keys", plan])
        .current_dir(repo)
        .output()
        .map_err(|error| format!("the store launcher did not run: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "`db.sh keys` exited {}",
            output.status.code().unwrap_or(-1)
        ));
    }
    let printed = String::from_utf8_lossy(&output.stdout);
    let mut found = quoted_keys(&printed);
    found.sort_unstable();
    let suffix = key_suffix(key);
    let matched: Vec<String> = found
        .into_iter()
        .filter(|name| key_suffix(name) == suffix)
        .collect();
    if !matched.iter().any(|name| name == key) {
        return Err(format!("the store holds no key `{key}`"));
    }
    Ok(matched)
}

/// Every key one `db.sh keys` document names.
fn quoted_keys(printed: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (at, head) in printed.match_indices("\"key\":\"") {
        let Some(rest) = printed.get(at + head.len()..) else {
            continue;
        };
        let Some(length) = key_run(rest) else {
            continue;
        };
        if !rest.get(length..).is_some_and(|tail| tail.starts_with('"')) {
            continue;
        }
        if let Some(key) = rest.get(..length) {
            found.push(key.to_owned());
        }
    }
    found
}

/// The md5 of the stored review at one plan-store key, or a reason.
fn store_digest(repo: &Path, plan: &str, key: &str) -> Result<String, String> {
    let launcher = launcher_of(repo)?;
    let output = Process::new("bash")
        .arg(&launcher)
        .args(["get", plan, key])
        .current_dir(repo)
        .output()
        .map_err(|error| format!("the store launcher did not run: {error}"))?;
    if !output.status.success() {
        let printed = String::from_utf8_lossy(&output.stderr);
        let detail = printed.trim().lines().next().unwrap_or_default();
        return Err(format!(
            "`db.sh get` exited {}: {detail}",
            output.status.code().unwrap_or(-1)
        ));
    }
    if output.stdout.is_empty() {
        return Err(format!("the store holds nothing at key `{key}`"));
    }
    Ok(hex_digest(without_trailing_newlines(&output.stdout)))
}

/// The stored copies one closure section binds this block to, or a report.
fn store_copies(
    window: &str,
    block: &str,
    review: &Path,
    document: &Path,
) -> Result<(PathBuf, String, Vec<String>), StoreReport> {
    let named = store_key_of(window).filter(|(_key, found)| *found == block);
    let Some((key, _found)) = named else {
        return Err(StoreReport {
            state: "absent",
            copies: 0,
            failures: vec![format!(
                "STORE:     the section states no plan-store key for {block}, so nothing binds this block to a copy outside the Architect's write scope (CL1c)"
            )],
        });
    };
    if let Some(suffix) = wanted_suffix(review)
        && key_suffix(key) != suffix
    {
        return Err(StoreReport {
            state: "wrong-suffix",
            copies: 0,
            failures: vec![format!(
                "STORE:     the section names key `{key}`, whose suffix is not `{suffix}`; the review file name decides the suffix and the Architect does not (CL1c)"
            )],
        });
    }
    let reached = crate::repo_root()
        .ok_or_else(|| "the repository root cannot be located".to_owned())
        .and_then(|repo| {
            plan_name(document)
                .ok_or_else(|| "the document names no plan directory".to_owned())
                .map(|plan| (repo, plan))
        })
        .and_then(|(repo, plan)| {
            sibling_keys(&repo, &plan, key).map(|copies| (repo, plan, copies))
        });
    reached.map_err(|reason| StoreReport {
        state: "unreachable",
        copies: 0,
        failures: vec![format!(
            "STORE:     key `{key}`: {reason}; CL1c is fail-closed"
        )],
    })
}

/// The `CL1c` report of one block.
fn store_report(
    window: &str,
    block: &str,
    review: &Path,
    document: &Path,
    wanted: &str,
) -> StoreReport {
    let (repo, plan, copies) = match store_copies(window, block, review, document) {
        Ok(reached) => reached,
        Err(report) => return report,
    };
    let mut failures = Vec::new();
    let mut digests: Vec<(String, String)> = Vec::new();
    for name in &copies {
        match store_digest(&repo, &plan, name) {
            Ok(digest) => digests.push((name.clone(), digest)),
            Err(reason) => {
                failures.push(format!(
                    "STORE:     key `{name}`: {reason}; CL1c is fail-closed"
                ));
                return StoreReport {
                    state: "unreachable",
                    copies: copies.len(),
                    failures,
                };
            },
        }
    }
    if copies.is_empty() {
        failures.push(
            "STORE:     the store half of CL1c did not run, and a rule that can be skipped is the fail-open this rule exists to remove (CL1c)"
                .to_owned(),
        );
        return StoreReport {
            state: "skipped",
            copies: 0,
            failures,
        };
    }
    let mut wrong: Vec<&(String, String)> = digests
        .iter()
        .filter(|(_name, digest)| digest != wanted)
        .collect();
    wrong.sort_by(|left, right| left.0.cmp(&right.0));
    let Some((name, digest)) = wrong.first() else {
        return StoreReport {
            state: "matches",
            copies: copies.len(),
            failures,
        };
    };
    failures.push(format!(
        "STORE:     {} of {} stored copies of this review differ from the file this run read, `{wanted}`; the first is key `{name}` at md5 `{digest}` (CL1c)",
        wrong.len(),
        copies.len()
    ));
    StoreReport {
        state: "differs",
        copies: copies.len(),
        failures,
    }
}

/// Generate the canonical finding-id list of one review (`CL1`).
///
/// This module is the port of `roadmap/duet-v1/tools/review_ids.py`. The id
/// list comes from the review's own headings, and the review's own stated
/// counts are the second source that a heading this parser cannot see makes
/// disagree.
mod review_ids {
    use std::path::Path;

    use super::{COUNTS_REQUIRED_FROM, digit_run, is_word_char, skip_spaces, whitespace_run};

    /// One finding family.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub(super) enum Family {
        /// A Critical finding.
        Critical,
        /// A Warning finding.
        Warning,
        /// A Concern finding.
        Concern,
    }

    impl Family {
        /// Every family, in document order.
        pub(super) const ALL: [Self; 3] = [Self::Critical, Self::Warning, Self::Concern];

        /// The heading word one family carries.
        const fn heading(self) -> &'static str {
            match self {
                Self::Critical => "CRITICAL",
                Self::Warning => "WARNING",
                Self::Concern => "CONCERN",
            }
        }

        /// The word the disagreement message prints.
        const fn title(self) -> &'static str {
            match self {
                Self::Critical => "Critical",
                Self::Warning => "Warning",
                Self::Concern => "Concern",
            }
        }
    }

    /// The numbers each family states.
    #[derive(Debug, Default)]
    pub(super) struct Seen {
        /// Every Critical number, in document order.
        critical: Vec<u32>,
        /// Every Warning number, in document order.
        warning: Vec<u32>,
        /// Every Concern number, in document order.
        concern: Vec<u32>,
    }

    impl Seen {
        /// Record one number of one family.
        fn push(&mut self, family: Family, number: u32) {
            match family {
                Family::Critical => self.critical.push(number),
                Family::Warning => self.warning.push(number),
                Family::Concern => self.concern.push(number),
            }
        }

        /// Every number one family states.
        fn numbers(&self, family: Family) -> &[u32] {
            match family {
                Family::Critical => &self.critical,
                Family::Warning => &self.warning,
                Family::Concern => &self.concern,
            }
        }

        /// How many findings one family states.
        pub(super) fn count(&self, family: Family) -> usize {
            self.numbers(family).len()
        }
    }

    /// The counts one review states about itself.
    #[derive(Debug, Clone, Copy)]
    struct Stated {
        /// The Critical count the sentence states.
        critical: u32,
        /// The Warning count the sentence states.
        warning: u32,
        /// The Concern count the sentence states.
        concern: u32,
    }

    impl Stated {
        /// The count one family carries.
        const fn count(self, family: Family) -> u32 {
            match family {
                Family::Critical => self.critical,
                Family::Warning => self.warning,
                Family::Concern => self.concern,
            }
        }
    }

    /// Every finding id of one review, and the numbers each family states.
    #[derive(Debug)]
    pub(super) struct Parsed {
        /// Every finding id, in family order.
        pub(super) ids: Vec<String>,
        /// The numbers each family states.
        pub(super) seen: Seen,
    }

    /// A cursor over one line, for the two count sentences.
    #[derive(Debug)]
    struct Cursor<'a> {
        /// The text this cursor has still to read.
        rest: &'a str,
    }

    impl<'a> Cursor<'a> {
        /// Open a cursor over one line.
        const fn new(rest: &'a str) -> Self {
            Self { rest }
        }

        /// Take one literal, case insensitive.
        fn word(&mut self, literal: &str) -> bool {
            let Some(head) = self.rest.get(..literal.len()) else {
                return false;
            };
            if !head.eq_ignore_ascii_case(literal) {
                return false;
            }
            self.rest = self.rest.get(literal.len()..).unwrap_or_default();
            true
        }

        /// Take a whitespace run of at least `least` characters.
        fn spaces(&mut self, least: usize) -> bool {
            let count = self
                .rest
                .chars()
                .take_while(|mark| mark.is_whitespace())
                .count();
            if count < least {
                return false;
            }
            self.rest = self
                .rest
                .get(whitespace_run(self.rest)..)
                .unwrap_or_default();
            true
        }

        /// Take a decimal number.
        fn number(&mut self) -> Option<u32> {
            let count = digit_run(self.rest);
            if count == 0 {
                return None;
            }
            let value = self.rest.get(..count)?.parse().ok()?;
            self.rest = self.rest.get(count..).unwrap_or_default();
            Some(value)
        }

        /// Take the optional plural of one family word.
        fn plural(&mut self) {
            let _taken: bool = self.word("s");
        }

        /// Take one family word, its plural, and the comma that closes it.
        fn family(&mut self, word: &str, closing: bool) -> bool {
            if !self.spaces(1) || !self.word(word) {
                return false;
            }
            self.plural();
            if closing {
                return self.word(".");
            }
            self.word(",") && self.spaces(0)
        }
    }

    /// The counts the review states about itself, or `None`.
    fn stated_counts(text: &str) -> Option<Stated> {
        text.split('\n')
            .find_map(bold_counts)
            .or_else(|| text.split('\n').find_map(prose_counts))
    }

    /// The `**Counts: ...**` sentence one line states.
    fn bold_counts(line: &str) -> Option<Stated> {
        let mut cursor = Cursor::new(line);
        if !cursor.word("**Counts:") || !cursor.spaces(0) {
            return None;
        }
        let critical = cursor.number()?;
        if !cursor.family("Critical", false) {
            return None;
        }
        let warning = cursor.number()?;
        if !cursor.family("Warning", false) {
            return None;
        }
        let concern = cursor.number()?;
        if !cursor.family("Concern", true) || !cursor.word("**") {
            return None;
        }
        Some(Stated {
            critical,
            warning,
            concern,
        })
    }

    /// The `This review holds ...` sentence one line states.
    fn prose_counts(line: &str) -> Option<Stated> {
        let mut cursor = Cursor::new(line);
        if !cursor.word("This review holds") || !cursor.spaces(0) {
            return None;
        }
        let critical = cursor.number()?;
        if !cursor.family("Critical", false) {
            return None;
        }
        let warning = cursor.number()?;
        if !cursor.family("Warning", false) || !cursor.word("and") || !cursor.spaces(0) {
            return None;
        }
        let concern = cursor.number()?;
        if !cursor.family("Concern", true) {
            return None;
        }
        Some(Stated {
            critical,
            warning,
            concern,
        })
    }

    /// The id prefix one review file name gives, or `None`.
    pub(super) fn prefix_of_path(review: &Path) -> Option<String> {
        let name = review.file_name()?.to_str()?;
        let body = name.strip_prefix("critic-spec-r")?.strip_suffix(".md")?;
        let digits = digit_run(body);
        if digits == 0 {
            return None;
        }
        let number = body.get(..digits)?;
        let rest = body.get(digits..)?;
        if rest.is_empty() {
            return Some(format!("C{number}"));
        }
        let tag = rest.strip_prefix('-')?;
        let head = tag.chars().next()?;
        if !head.is_ascii_lowercase() {
            return None;
        }
        if !tag
            .chars()
            .skip(1)
            .all(|mark| mark.is_ascii_lowercase() || mark.is_ascii_digit())
        {
            return None;
        }
        Some(format!("C{number}{}", head.to_ascii_uppercase()))
    }

    /// The revision number the review's own title states, or `None`.
    pub(super) fn revision_of(text: &str) -> Option<String> {
        text.split('\n')
            .filter_map(|line| line.strip_prefix('#'))
            .filter(|rest| rest.starts_with(char::is_whitespace))
            .find_map(last_revision)
    }

    /// The last `revision <n>` one line states.
    fn last_revision(line: &str) -> Option<String> {
        let lower = line.to_ascii_lowercase();
        let mut found = None;
        for (at, head) in lower.match_indices("revision") {
            let before = lower.get(..at).and_then(|start| start.chars().next_back());
            if before.is_some_and(is_word_char) {
                continue;
            }
            let Some(after) = lower
                .get(at + head.len()..)
                .and_then(|rest| skip_spaces(rest, 1))
            else {
                continue;
            };
            let count = digit_run(after);
            if count == 0 {
                continue;
            }
            if after
                .get(count..)
                .and_then(|tail| tail.chars().next())
                .is_some_and(is_word_char)
            {
                continue;
            }
            found = after.get(..count).map(str::to_owned);
        }
        found
    }

    /// Every finding id of one review, in document order.
    pub(super) fn ids_of(text: &str, prefix: &str) -> Result<Parsed, String> {
        let stripped = super::without_fences(text);
        let stated = stated_counts(&stripped);
        let tail = prefix.strip_prefix('C').unwrap_or(prefix).to_owned();
        let seen = scan(&stripped, &tail);
        if Family::ALL
            .iter()
            .all(|family| seen.numbers(*family).is_empty())
        {
            return Err("the review states no CRITICAL, WARNING or CONCERN heading".to_owned());
        }
        for family in Family::ALL {
            numbered_from_one(&seen, family)?;
        }
        check_counts(&seen, stated, prefix)?;
        Ok(Parsed {
            ids: ids_from(&seen, prefix, &tail),
            seen,
        })
    }

    /// Refuse a family that is not numbered one to its own count.
    fn numbered_from_one(seen: &Seen, family: Family) -> Result<(), String> {
        let numbers = seen.numbers(family);
        if numbers.is_empty() {
            return Ok(());
        }
        let mut sorted = numbers.to_vec();
        sorted.sort_unstable();
        let wanted = 1..=u32::try_from(numbers.len()).unwrap_or(u32::MAX);
        if sorted.into_iter().eq(wanted) {
            return Ok(());
        }
        Err(format!(
            "the {} numbers are not 1 to {}: {}",
            family.heading(),
            numbers.len(),
            list_of(numbers)
        ))
    }

    /// Refuse a review whose stated counts and heading scan disagree.
    fn check_counts(seen: &Seen, stated: Option<Stated>, prefix: &str) -> Result<(), String> {
        let digits: String = prefix.chars().filter(char::is_ascii_digit).collect();
        let required = digits
            .parse::<u32>()
            .is_ok_and(|revision| revision >= COUNTS_REQUIRED_FROM);
        let Some(stated) = stated else {
            if required {
                return Err(format!(
                    "the review states no `**Counts: <n> Criticals, <n> Warnings, <n> Concerns.**` sentence, which every review from revision {COUNTS_REQUIRED_FROM} carries"
                ));
            }
            return Ok(());
        };
        for family in Family::ALL {
            let count = stated.count(family);
            let scanned = seen.count(family);
            if usize::try_from(count).unwrap_or(usize::MAX) != scanned {
                return Err(format!(
                    "the review states {count} {}s and the heading scan finds {scanned}; a heading this parser cannot see is the one failure a second source exists to catch",
                    family.title()
                ));
            }
        }
        Ok(())
    }

    /// Every finding id, in the order Appendix C records them.
    fn ids_from(seen: &Seen, prefix: &str, tail: &str) -> Vec<String> {
        let mut ids = Vec::new();
        for family in Family::ALL {
            let mut numbers = seen.numbers(family).to_vec();
            numbers.sort_unstable();
            for number in numbers {
                ids.push(id_of(family, prefix, tail, number));
            }
        }
        ids
    }

    /// The id one finding of one family carries.
    fn id_of(family: Family, prefix: &str, tail: &str, number: u32) -> String {
        match family {
            Family::Critical => format!("{prefix}-{number}"),
            Family::Warning => format!("{prefix}-W{number}"),
            Family::Concern => format!("N{tail}-{number}"),
        }
    }

    /// One number list, as the failure line prints it.
    fn list_of(numbers: &[u32]) -> String {
        let parts: Vec<String> = numbers.iter().map(u32::to_string).collect();
        format!("[{}]", parts.join(", "))
    }

    /// Every finding number one review states, by family.
    fn scan(text: &str, tail: &str) -> Seen {
        let mut seen = Seen::default();
        for line in text.split('\n') {
            if let Some((family, number)) = heading_finding(line) {
                seen.push(family, number);
            }
        }
        for line in text.split('\n') {
            if let Some(number) = recovered_warning(line) {
                seen.push(Family::Warning, number);
            }
        }
        for line in text.split('\n') {
            if let Some(number) = recovered_concern(line, tail) {
                seen.push(Family::Concern, number);
            }
        }
        seen
    }

    /// The family and number one `## <FAMILY> <n>` heading states.
    fn heading_finding(line: &str) -> Option<(Family, u32)> {
        let after = skip_spaces(line.strip_prefix("##")?, 1)?;
        for family in Family::ALL {
            let Some(rest) = after
                .strip_prefix(family.heading())
                .and_then(|tail| skip_spaces(tail, 1))
            else {
                continue;
            };
            let count = digit_run(rest);
            if count == 0 {
                continue;
            }
            if rest
                .get(count..)
                .and_then(|tail| tail.chars().next())
                .is_some_and(is_word_char)
            {
                continue;
            }
            let number = rest.get(..count)?.parse().ok()?;
            return Some((family, number));
        }
        None
    }

    /// The number one recovered `**W<n>.` Warning states.
    fn recovered_warning(line: &str) -> Option<u32> {
        let rest = line.strip_prefix("**W")?;
        let count = digit_run(rest);
        if count == 0 || !rest.get(count..)?.starts_with('.') {
            return None;
        }
        rest.get(..count)?.parse().ok()
    }

    /// The number one recovered Concern table row states.
    fn recovered_concern(line: &str, tail: &str) -> Option<u32> {
        let rest = skip_spaces(line.strip_prefix('|')?, 0)?
            .strip_prefix('N')?
            .strip_prefix(tail)?
            .strip_prefix('-')?;
        let count = digit_run(rest);
        if count == 0 {
            return None;
        }
        if !skip_spaces(rest.get(count..)?, 0)?.starts_with('|') {
            return None;
        }
        rest.get(..count)?.parse().ok()
    }
}
