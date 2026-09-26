//! Refuse a cast and a cast suppression outside the one conversion file.
//!
//! Every rule carries the id architecture section 2.3 gives it. Clippy is the
//! backstop, because the root manifest denies `as_conversions` and the commit
//! gate runs clippy with `-D warnings`. This guard is the earlier, cheaper
//! signal, and `CG6` is the one rule clippy cannot carry: an `#[expect]`
//! silences clippy and does not silence a text scan.
//!
//! - `CG1` takes the file set from the target source paths that
//!   `cargo metadata --no-deps` reports. The build-directory filter is
//!   anchored at the workspace root and at each member root, so a source
//!   module named `target` stays in the set.
//! - `CG2` classifies every byte as code, string, character, line comment, or
//!   block comment before any rule reads the text. `CG2b` keeps a comment
//!   token inside a string inert, and a quotation mark inside a comment inert.
//! - `CG3` reports a whole-word cast token. `CG3b` prefers a false red to a
//!   miss, so an exclusion applies only when both sides parse as a type path.
//! - `CG4` drops a `use` item as a whole item, across lines. `CG4b` fails
//!   closed on a `use` item with no terminating semicolon.
//! - `CG5` drops a balanced qualified path span.
//! - `CG6` reads the whole attribute span, outer and inner alike.
//! - `CG7` matches the one exempt file on both sides as a canonical path, and
//!   the exempt file must sit inside the canonical member root.
//! - `CG1b` derives the member set and the file set from the tree, so a scan
//!   that misses a member fails by name. No environment variable changes it.
//! - `CG8` prints one named line and fails closed.
//! - `CG9` binds each Appendix B.1 `b1-convert` reason cell to the `reason =`
//!   string of the one exempt file, in both directions and character for
//!   character. It runs only when the caller names the document with
//!   `--appendix <path>`, it prints the skip when the caller names none, and it
//!   fails closed on a document that does not open. The compare reads the
//!   DECODED value of the string literal and never its raw source token.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::Outcome;

/// The one member that may hold a cast (`CG7`).
const EXEMPT_MEMBER: [&str; 4] = ["crates", "bc_time", "duet-time", "lang_rust"];

/// The one file inside that member that may hold a cast (`CG7`).
const EXEMPT_FILE: [&str; 2] = ["src", "convert.rs"];

/// Every prefix Rust writes before a string literal, longest first so that a
/// two-letter prefix never matches as a bare one-letter prefix (`CG2`).
const STRING_PREFIXES: [&str; 6] = ["br", "cr", "rb", "b", "c", "r"];

/// The whole-word token `CG3` reports.
const CAST_TOKEN: &str = "as";

/// The whole-word token `CG4` drops as a whole item.
const USE_TOKEN: &str = "use";

/// The directory name Cargo gives its build tree.
const BUILD_DIRECTORY: &str = "target";

/// `CG9`. The opening of the marker line that carries the Appendix B.1 block.
const B1_CONVERT_MARKER: &str = "<!-- GUARD BLOCK id=b1-convert rows>=";

/// `CG9`. The whole-word token that opens a `reason` assignment.
const REASON_TOKEN: &str = "reason";

/// `CG9`. The line the guard prints when the caller names no appendix.
const REASON_SKIP_LINE: &str = "REASON TEXTS:    skipped (no --appendix)";

/// One workspace member, as `cargo metadata` reports it.
#[derive(Debug)]
struct Member {
    /// The canonical directory that holds the member manifest.
    root: PathBuf,
    /// The canonical source path of every target the member declares.
    targets: Vec<PathBuf>,
}

/// One workspace, as `cargo metadata` reports it.
#[derive(Debug)]
struct Workspace {
    /// The canonical workspace root.
    top: PathBuf,
    /// Every member, in the order cargo reports it.
    members: Vec<Member>,
}

/// The derived set sizes the summary line reports.
#[derive(Debug, Clone, Copy)]
struct Counts {
    /// How many members the manifest globs derive.
    members: usize,
    /// How many files the covered members derive.
    files: usize,
}

/// `CG1b`. What the derived coverage sets say about one scan.
#[derive(Debug)]
enum Coverage {
    /// The scan covers every derived member and every derived file.
    Complete(Counts),
    /// The scan misses something. Each line is printable.
    Incomplete(Vec<String>),
}

/// One breach the guard reports.
#[derive(Debug)]
struct Finding {
    /// The file, relative to the workspace root.
    shown: String,
    /// What the breach is.
    kind: &'static str,
    /// Where the breach sits.
    detail: String,
}

/// Run the conversion guard over every member `cargo metadata` reports.
///
/// `appendix` names the architecture document whose Appendix B.1 reason cells
/// bind the `reason =` strings of the one exempt file (`CG9`). The rule runs
/// only when the caller names a document; the guard prints the skip otherwise.
///
/// # Errors
/// Returns an error when `cargo metadata` fails, when a member file cannot be
/// read, and when the derived coverage sets do not match the reported member
/// set (`CG1`, `CG1b`, `CG8`).
pub(crate) fn run(root: &Path, appendix: Option<&Path>) -> anyhow::Result<Outcome> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let rows = match appendix.map(appendix_rows) {
        Some(Err(line)) => {
            writeln!(out, "{line}")?;
            return Ok(Outcome::FailClosed);
        },
        Some(Ok(stated)) => Some(stated),
        None => None,
    };
    let Some(space) = workspace(root) else {
        writeln!(
            out,
            "FAIL: `cargo metadata --no-deps` refused {}; the guard is fail-closed.",
            root.display()
        )?;
        return Ok(Outcome::FailClosed);
    };
    let files = source_files(&space);
    let counts = match coverage(&space, &files) {
        Coverage::Complete(counts) => counts,
        Coverage::Incomplete(lines) => {
            for line in lines {
                writeln!(out, "{line}")?;
            }
            return Ok(Outcome::FailClosed);
        },
    };
    let exempt = exempt_path(&space);
    let mut findings = Vec::new();
    for path in &files {
        match scan_file(path, &space.top, exempt.as_deref()) {
            Ok(mut found) => findings.append(&mut found),
            Err(reason) => {
                writeln!(out, "{reason}")?;
                return Ok(Outcome::FailClosed);
            },
        }
    }
    for finding in &findings {
        writeln!(
            out,
            "  {}: {}: {}",
            finding.kind, finding.shown, finding.detail
        )?;
    }
    writeln!(
        out,
        "MEMBERS: {}   FILES: {}   FINDINGS: {}   EXPECTED MEMBERS: {}   EXPECTED FILES: {}",
        space.members.len(),
        files.len(),
        findings.len(),
        counts.members,
        counts.files
    )?;
    let reasons = match rows {
        None => {
            writeln!(out, "{REASON_SKIP_LINE}")?;
            Vec::new()
        },
        Some(stated) => {
            let pairs = reason_pairs(stated, exempt_reasons(exempt.as_deref()));
            let broken = reason_findings(&pairs);
            for line in &broken {
                writeln!(out, "{line}")?;
            }
            writeln!(
                out,
                "REASON TEXTS:    {}     REASON TEXT BAD: {}",
                pairs.len(),
                broken.len()
            )?;
            broken
        },
    };
    Ok(if findings.is_empty() && reasons.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Findings
    })
}

/// `CG9`. Every site and reason text the one exempt file declares.
///
/// A file that does not open declares nothing, which leaves every
/// `b1-convert` row a finding and never a clean run.
fn exempt_reasons(exempt: Option<&Path>) -> Vec<(String, String)> {
    exempt
        .and_then(|path| fs::read_to_string(path).ok())
        .map(|source| reason_sites(&source))
        .unwrap_or_default()
}

/// `CG3`, `CG4b`, `CG6`, `CG7`, `CG8`. Every breach one member file holds.
///
/// The `Err` holds the one fail-closed line the caller prints.
fn scan_file(path: &Path, top: &Path, exempt: Option<&Path>) -> Result<Vec<Finding>, String> {
    let canonical = realpath(path);
    let shown = relative(&canonical, top).display().to_string();
    let Ok(bytes) = fs::read(path) else {
        return Err(format!(
            "FAIL: {shown}: the file does not open; the guard is fail-closed."
        ));
    };
    let Ok(source) = String::from_utf8(bytes) else {
        return Err(format!(
            "FAIL: {shown}: the bytes are not UTF-8; the guard is fail-closed."
        ));
    };
    let text = lex(&source);
    let spared = exempt.is_some_and(|one| one == canonical);
    let mut findings = Vec::new();
    if !spared {
        for line in suppression_lines(&text) {
            findings.push(Finding {
                shown: shown.clone(),
                kind: "SUPPRESSION",
                detail: format!("line {line}"),
            });
        }
        let dropped = drop_use_items(&text).map_err(|line| {
            format!("FAIL: {shown}: a `use` at line {line} has no `;`; the guard is fail-closed.")
        })?;
        let scanned = drop_qualified_paths(&dropped);
        for start in word_positions(&scanned, CAST_TOKEN) {
            findings.push(Finding {
                shown: shown.clone(),
                kind: "CAST",
                detail: format!("line {}", line_of(&scanned, start)),
            });
        }
    }
    Ok(findings)
}

/// The workspace one directory resolves, or `None` when cargo refuses it (`CG8`).
fn workspace(root: &Path) -> Option<Workspace> {
    parse_workspace(&crate::cargo_metadata(root)?)
}

/// The workspace one `cargo metadata` document describes.
fn parse_workspace(text: &str) -> Option<Workspace> {
    let parsed = serde_json::from_str::<Value>(text).ok()?;
    let top = realpath(Path::new(parsed.get("workspace_root")?.as_str()?));
    let mut members = Vec::new();
    for package in parsed.get("packages")?.as_array()? {
        let manifest = Path::new(package.get("manifest_path")?.as_str()?);
        let mut targets = Vec::new();
        for target in package.get("targets")?.as_array()? {
            targets.push(realpath(Path::new(target.get("src_path")?.as_str()?)));
        }
        members.push(Member {
            root: realpath(manifest.parent()?),
            targets,
        });
    }
    Some(Workspace { top, members })
}

/// `CG1`. The anchored build directories the file set excludes.
fn build_directories(space: &Workspace) -> Vec<PathBuf> {
    let mut found = vec![space.top.join(BUILD_DIRECTORY)];
    for member in &space.members {
        found.push(member.root.join(BUILD_DIRECTORY));
    }
    found
}

/// `CG1b`. The canonical build directories the derived walk must not enter.
fn build_roots(space: &Workspace) -> Vec<PathBuf> {
    build_directories(space)
        .iter()
        .map(|path| realpath(path))
        .collect()
}

/// Whether one member owns one scanned file, outside every build tree.
///
/// The file answers for its own path and for the path a symbolic link resolves
/// to, so neither spelling can drop a file the member compiles.
fn owns(member: &Path, path: &Path, canonical: &Path, builds: &[PathBuf]) -> bool {
    if !(path.starts_with(member) || canonical.starts_with(member)) {
        return false;
    }
    !builds
        .iter()
        .any(|build| path.starts_with(build) || canonical.starts_with(build))
}

/// `CG1`. Every Rust file the target source paths of a member reach.
fn source_files(space: &Workspace) -> Vec<PathBuf> {
    let builds = build_directories(space);
    let mut found = Vec::new();
    for member in &space.members {
        for target in &member.targets {
            found.extend(target_files(member, target, &builds));
        }
    }
    sorted_paths(found)
}

/// `CG1`. Every Rust file one member owns beside one target source path.
fn target_files(member: &Member, target: &Path, builds: &[PathBuf]) -> Vec<PathBuf> {
    let keep_every_directory = |_: &Path| false;
    let mut found = Vec::new();
    if owns(&member.root, target, target, builds) {
        found.push(target.to_path_buf());
    }
    let Some(parent) = target.parent() else {
        return found;
    };
    for path in walk_files(parent, &keep_every_directory) {
        let canonical = realpath(&path);
        if is_rust_path(&path) && owns(&member.root, &path, &canonical, builds) {
            found.push(canonical);
        }
    }
    found
}

/// `CG7`. The one file that may hold a cast, canonical, or `None`.
fn exempt_path(space: &Workspace) -> Option<PathBuf> {
    let mut member = space.top.clone();
    member.extend(EXEMPT_MEMBER);
    let member = realpath(&member);
    if !space.members.iter().any(|one| one.root == member) {
        return None;
    }
    let mut candidate = member.clone();
    candidate.extend(EXEMPT_FILE);
    let candidate = realpath(&candidate);
    candidate.starts_with(&member).then_some(candidate)
}

/// `CG1b`. What the derived sets say about one scan.
fn coverage(space: &Workspace, scanned: &[PathBuf]) -> Coverage {
    if space.members.is_empty() || scanned.is_empty() {
        return Coverage::Incomplete(vec![format!(
            "FAIL: the scan covers {} members and {} files; a zero denominator is not a clean run and the guard is fail-closed.",
            space.members.len(),
            scanned.len()
        )]);
    }
    let Some(wanted) = expected_members(space) else {
        return Coverage::Incomplete(vec![
            "FAIL: the root manifest states no `members` array, so `CG1b` can derive no expected set; the guard is fail-closed."
                .to_owned(),
        ]);
    };
    let covered: HashSet<PathBuf> = space
        .members
        .iter()
        .map(|member| member.root.clone())
        .collect();
    let files = expected_files(space, &covered);
    let mut missing = Vec::new();
    for root in sorted_paths(wanted.iter().cloned().collect()) {
        if !covered.contains(&root) {
            missing.push(format!(
                "FAIL: {} holds a Cargo.toml and the scan does not cover it; the guard is fail-closed.",
                relative(&root, &space.top).display()
            ));
        }
    }
    let held: HashSet<&PathBuf> = scanned.iter().collect();
    for path in sorted_paths(files.iter().cloned().collect()) {
        if !held.contains(&path) {
            missing.push(format!(
                "FAIL: {} sits under a scanned member's src and the scan does not hold it; the guard is fail-closed.",
                relative(&path, &space.top).display()
            ));
        }
    }
    if missing.is_empty() {
        Coverage::Complete(Counts {
            members: wanted.len(),
            files: files.len(),
        })
    } else {
        Coverage::Incomplete(missing)
    }
}

/// `CG1b` part 2. Every member this workspace holds, from the manifest globs.
///
/// A directory that one `members` pattern matches, or that sits beside one at
/// any level of its path, and that holds a `Cargo.toml`, is a member. The set
/// is derived, so an `exclude` line that removes a crate from `cargo metadata`
/// leaves that crate in this set and the run fails by name.
fn expected_members(space: &Workspace) -> Option<HashSet<PathBuf>> {
    let globs = member_globs(&space.top)?;
    let builds = build_roots(space);
    let mut patterns: HashSet<String> = HashSet::new();
    for entry in &globs {
        patterns.extend(sibling_patterns(entry));
    }
    let mut found = HashSet::new();
    for pattern in &patterns {
        for canonical in expand(&space.top, pattern) {
            let inside_build = builds.iter().any(|build| canonical.starts_with(build));
            if !inside_build && canonical.join("Cargo.toml").is_file() {
                found.insert(canonical);
            }
        }
    }
    Some(found)
}

/// `CG1b` part 3. Every Rust file under each covered member's own source tree.
fn expected_files(space: &Workspace, covered: &HashSet<PathBuf>) -> HashSet<PathBuf> {
    let builds = build_roots(space);
    let prune = |child: &Path| {
        let canonical = realpath(child);
        builds.iter().any(|build| canonical.starts_with(build))
    };
    let mut found = HashSet::new();
    for root in covered {
        let source = root.join("src");
        if !source.is_dir() {
            continue;
        }
        for path in walk_files(&source, &prune) {
            if is_rust_path(&path) {
                found.insert(realpath(&path));
            }
        }
    }
    found
}

/// Every `members` glob of the root manifest, as a list of patterns.
///
/// The globs are the source, and `exclude` is deliberately not read: an
/// excluded member is exactly what `CG1b` exists to see.
fn member_globs(top: &Path) -> Option<Vec<String>> {
    let text = fs::read_to_string(top.join("Cargo.toml")).ok()?;
    let chars: Vec<char> = text.chars().collect();
    let mut start = Some(0_usize);
    while let Some(line_start) = start {
        if let Some(open) = members_array_open(&chars, line_start)
            && let Some(close) = chars
                .get(open..)
                .and_then(|tail| tail.iter().position(|mark| *mark == ']'))
        {
            return chars.get(open..open + close).map(quoted_strings);
        }
        start = chars
            .get(line_start..)
            .and_then(|tail| tail.iter().position(|mark| *mark == '\n'))
            .map(|at| line_start + at + 1);
    }
    None
}

/// The offset just after the `[` of a `members` array that opens one line.
fn members_array_open(chars: &[char], line_start: usize) -> Option<usize> {
    let cursor = skip_space(chars, line_start);
    let cursor = take_word(chars, cursor, "members")?;
    let cursor = skip_space(chars, cursor);
    let cursor = take_word(chars, cursor, "=")?;
    let cursor = skip_space(chars, cursor);
    take_word(chars, cursor, "[")
}

/// Every double-quoted run of one span, in order.
fn quoted_strings(span: &[char]) -> Vec<String> {
    let mut found = Vec::new();
    let mut index = 0;
    while index < span.len() {
        let closer = span
            .get(index + 1..)
            .and_then(|tail| tail.iter().position(|mark| *mark == '"'))
            .map(|at| index + 1 + at);
        match closer {
            Some(end) if span.get(index) == Some(&'"') && end > index + 1 => {
                if let Some(inner) = span.get(index + 1..end) {
                    found.push(inner.iter().collect());
                }
                index = end + 1;
            },
            _ => index += 1,
        }
    }
    found
}

/// One `members` entry plus the sibling set at every level of its path.
///
/// A manifest that lists a member by its full path cannot hide the crate
/// beside it, and the top level is the sibling set of every entry.
fn sibling_patterns(entry: &str) -> HashSet<String> {
    let mut found = HashSet::new();
    found.insert(entry.to_owned());
    let parts: Vec<&str> = entry.trim_matches('/').split('/').collect();
    for depth in 0..parts.len() {
        let Some(head) = parts.get(..depth) else {
            continue;
        };
        let joined = head.join("/");
        found.insert(format!("{joined}/*").trim_start_matches('/').to_owned());
    }
    found
}

/// Every directory one `members` pattern matches, the way Cargo matches it.
///
/// Cargo expands a `members` glob with a matcher that has no leading-dot rule,
/// so a pattern such as `crates/*` reaches a directory whose name opens with a
/// dot. A `members` entry is a path of segments, and a segment is matched
/// literally or by the glob syntax.
fn expand(top: &Path, pattern: &str) -> Vec<PathBuf> {
    let mut here = vec![realpath(top)];
    for segment in pattern.trim_matches('/').split('/') {
        let mut next = Vec::new();
        for base in &here {
            expand_segment(base, segment, &mut next);
        }
        here = next;
    }
    here
}

/// Every directory one pattern segment reaches from one base directory.
fn expand_segment(base: &Path, segment: &str, found: &mut Vec<PathBuf>) {
    if !base.is_dir() {
        return;
    }
    if segment.contains(['*', '?', '[']) {
        push_matching_children(base, segment, found);
        return;
    }
    let candidate = realpath(&base.join(segment));
    if candidate.is_dir() {
        found.push(candidate);
    }
}

/// Every child directory of one directory whose name matches one segment.
fn push_matching_children(base: &Path, segment: &str, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(base) else {
        return;
    };
    let wanted: Vec<char> = segment.chars().collect();
    for entry in entries.flatten() {
        let path = entry.path();
        let name: Vec<char> = entry.file_name().to_string_lossy().chars().collect();
        if path.is_dir() && glob_match(&name, &wanted) {
            found.push(realpath(&path));
        }
    }
}

/// Whether one name matches one glob segment.
fn glob_match(name: &[char], pattern: &[char]) -> bool {
    let Some((mark, rest)) = pattern.split_first() else {
        return name.is_empty();
    };
    match *mark {
        '*' => (0..=name.len()).any(|at| name.get(at..).is_some_and(|tail| glob_match(tail, rest))),
        '?' => name
            .split_first()
            .is_some_and(|(_, tail)| glob_match(tail, rest)),
        '[' => glob_class(name, pattern),
        other => name
            .split_first()
            .is_some_and(|(head, tail)| *head == other && glob_match(tail, rest)),
    }
}

/// Whether one name matches a glob segment that opens with a character class.
fn glob_class(name: &[char], pattern: &[char]) -> bool {
    let Some((start, end)) = glob_class_bounds(pattern) else {
        return name.split_first().is_some_and(|(head, tail)| {
            *head == '[' && glob_match(tail, pattern.get(1..).unwrap_or_default())
        });
    };
    let negated = pattern.get(1) == Some(&'!');
    let Some(body) = pattern.get(start..end) else {
        return false;
    };
    let Some((head, tail)) = name.split_first() else {
        return false;
    };
    if glob_class_holds(body, *head) == negated {
        return false;
    }
    glob_match(tail, pattern.get(end + 1..).unwrap_or_default())
}

/// The body bounds of a character class, or `None` when it never closes.
///
/// A `]` that opens the body is a literal, and the body keeps the `!` that
/// negates it.
fn glob_class_bounds(pattern: &[char]) -> Option<(usize, usize)> {
    let mut start = if pattern.get(1) == Some(&'!') { 2 } else { 1 };
    if pattern.get(start) == Some(&']') {
        start += 1;
    }
    let close = pattern
        .get(start..)
        .and_then(|tail| tail.iter().position(|mark| *mark == ']'))
        .map(|at| start + at)?;
    Some((1, close))
}

/// Whether one character class body holds one character.
fn glob_class_holds(body: &[char], wanted: char) -> bool {
    let mut index = usize::from(body.first() == Some(&'!'));
    while index < body.len() {
        let low = body.get(index).copied();
        let dash = body.get(index + 1).copied();
        let high = body.get(index + 2).copied();
        match (low, dash, high) {
            (Some(low), Some('-'), Some(high)) => {
                if (low..=high).contains(&wanted) {
                    return true;
                }
                index += 3;
            },
            (Some(low), _, _) => {
                if low == wanted {
                    return true;
                }
                index += 1;
            },
            (None, _, _) => return false,
        }
    }
    false
}

/// `CG2` and `CG2b`. Classify every byte, then blank everything but code.
///
/// One left-to-right pass decides what a token means from the state it is
/// already in, so a comment opener inside a string opens no comment and a
/// quotation mark inside a comment opens no string. A newline is kept, so a
/// reported line number is the file's own, and every other byte outside code
/// becomes a space, so a later offset stays put.
fn lex(source: &str) -> Vec<char> {
    let text: Vec<char> = source.chars().collect();
    let mut out: Vec<char> = Vec::with_capacity(text.len());
    let mut index = 0;
    while let Some(&mark) = text.get(index) {
        if let Some(end) = literal_at(&text, index, mark) {
            blank_into(&mut out, &text, index, end);
            index = end;
            continue;
        }
        if let Some(end) = comment_at(&text, index) {
            blank_into(&mut out, &text, index, end);
            index = end;
            continue;
        }
        out.push(mark);
        index += 1;
    }
    out
}

/// The end of the string or character literal that opens at one offset.
fn literal_at(text: &[char], index: usize, mark: char) -> Option<usize> {
    if ascii_ident_start(mark)
        && !preceded_by_word(text, index)
        && let Some(end) = prefixed_literal(text, index)
    {
        return Some(end);
    }
    if mark == '"' {
        return Some(end_of_quoted(text, index));
    }
    if mark == '\'' {
        return char_literal_end(text, index);
    }
    None
}

/// The end of the comment that opens at one offset.
fn comment_at(text: &[char], index: usize) -> Option<usize> {
    if starts_with(text, index, "//") {
        return Some(line_comment_end(text, index));
    }
    if starts_with(text, index, "/*") {
        return Some(block_comment_end(text, index));
    }
    None
}

/// The offset of the newline that ends one line comment, or the file end.
fn line_comment_end(text: &[char], index: usize) -> usize {
    text.get(index..)
        .and_then(|tail| tail.iter().position(|mark| *mark == '\n'))
        .map_or(text.len(), |at| index + at)
}

/// The end of one block comment, which nests.
fn block_comment_end(text: &[char], index: usize) -> usize {
    let mut depth = 1_usize;
    let mut probe = index + 2;
    while probe < text.len() && depth > 0 {
        if starts_with(text, probe, "/*") {
            depth += 1;
            probe += 2;
        } else if starts_with(text, probe, "*/") {
            depth -= 1;
            probe += 2;
        } else {
            probe += 1;
        }
    }
    probe
}

/// The end of an ordinary string literal that opens at one offset.
///
/// An ordinary literal processes escapes, so an escaped quotation mark does not
/// close it.
fn end_of_quoted(text: &[char], index: usize) -> usize {
    let mut probe = index + 1;
    while probe < text.len() {
        match text.get(probe) {
            Some('\\') => probe += 2,
            Some('"') => {
                probe += 1;
                break;
            },
            _ => probe += 1,
        }
    }
    probe.min(text.len())
}

/// `CG2`. The end of a prefixed string literal that opens at one offset.
///
/// A prefix that carries an `r` opens a raw literal, which processes no escape
/// and ends at a quotation mark followed by its own hash run, so its text may
/// hold a bare quotation mark. A hash run needs an `r`, because the other
/// prefixes never take one.
fn prefixed_literal(text: &[char], index: usize) -> Option<usize> {
    for prefix in STRING_PREFIXES {
        if !starts_with(text, index, prefix) {
            continue;
        }
        let mut probe = index + prefix.chars().count();
        let mut hashes = 0_usize;
        while text.get(probe) == Some(&'#') {
            hashes += 1;
            probe += 1;
        }
        if text.get(probe) != Some(&'"') {
            continue;
        }
        let raw = prefix.contains('r');
        if hashes > 0 && !raw {
            continue;
        }
        if raw {
            return Some(raw_literal_end(text, probe, hashes));
        }
        return Some(end_of_quoted(text, probe));
    }
    None
}

/// The end of a raw string literal whose quotation mark sits at one offset.
fn raw_literal_end(text: &[char], quote: usize, hashes: usize) -> usize {
    let mut probe = quote + 1;
    while probe < text.len() {
        let closed = text.get(probe) == Some(&'"')
            && (1..=hashes).all(|offset| text.get(probe + offset) == Some(&'#'));
        if closed {
            return probe + 1 + hashes;
        }
        probe += 1;
    }
    text.len()
}

/// The end of a character literal that opens at one offset, or `None`.
///
/// A quotation mark that opens no literal is a lifetime, which is code.
fn char_literal_end(text: &[char], index: usize) -> Option<usize> {
    let body = index + 1;
    if text.get(body) == Some(&'\\') {
        return escaped_char_end(text, body);
    }
    let plain = text
        .get(body)
        .is_some_and(|mark| *mark != '\'' && *mark != '\\');
    if plain && text.get(body + 1) == Some(&'\'') {
        return Some(body + 2);
    }
    None
}

/// The end of an escaped character literal whose backslash sits at one offset.
fn escaped_char_end(text: &[char], slash: usize) -> Option<usize> {
    if text.get(slash + 1) == Some(&'x')
        && text.get(slash + 2).is_some_and(char::is_ascii_hexdigit)
        && text.get(slash + 3).is_some_and(char::is_ascii_hexdigit)
        && text.get(slash + 4) == Some(&'\'')
    {
        return Some(slash + 5);
    }
    if text.get(slash + 1) == Some(&'u') && text.get(slash + 2) == Some(&'{') {
        let digits = (0..6)
            .take_while(|offset| {
                text.get(slash + 3 + offset)
                    .is_some_and(char::is_ascii_hexdigit)
            })
            .count();
        if digits > 0
            && text.get(slash + 3 + digits) == Some(&'}')
            && text.get(slash + 4 + digits) == Some(&'\'')
        {
            return Some(slash + 5 + digits);
        }
    }
    let any = text.get(slash + 1).is_some_and(|mark| *mark != '\n');
    if any && text.get(slash + 2) == Some(&'\'') {
        return Some(slash + 3);
    }
    None
}

/// Append one span with every byte but a newline replaced by a space.
fn blank_into(out: &mut Vec<char>, text: &[char], start: usize, end: usize) {
    let Some(span) = text.get(start..end.min(text.len())) else {
        return;
    };
    out.extend(
        span.iter()
            .map(|mark| if *mark == '\n' { '\n' } else { ' ' }),
    );
}

/// Replace one inclusive span with spaces, so every later offset stays put.
fn blank_span(chars: &mut [char], start: usize, end: usize) {
    for index in start..=end {
        if let Some(slot) = chars.get_mut(index)
            && *slot != '\n'
        {
            *slot = ' ';
        }
    }
}

/// `CG4` and `CG4b`. Remove each `use` item as a whole, from `use` to its `;`.
///
/// The `Err` holds the line of a `use` item with no terminating semicolon,
/// which fails closed rather than blanking to the end of the file.
fn drop_use_items(text: &[char]) -> Result<Vec<char>, usize> {
    let mut out = text.to_vec();
    for start in word_positions(text, USE_TOKEN) {
        if !statement_position(text, start) {
            continue;
        }
        let from = start + USE_TOKEN.chars().count();
        let Some(end) = text
            .get(from..)
            .and_then(|tail| tail.iter().position(|mark| *mark == ';'))
            .map(|at| from + at)
        else {
            return Err(line_of(text, start));
        };
        blank_span(&mut out, start, end);
    }
    Ok(out)
}

/// Whether the token at one offset stands in statement position.
///
/// A `use` token inside a macro pattern does not, and blanking from it would
/// swallow the next statement and every cast in it.
fn statement_position(text: &[char], start: usize) -> bool {
    let line_start = text.get(..start).map_or(0, |head| {
        head.iter()
            .rposition(|mark| *mark == '\n')
            .map_or(0, |at| at + 1)
    });
    let lead: String = text
        .get(line_start..start)
        .map_or_else(String::new, |span| span.iter().collect());
    let trimmed = lead.trim();
    trimmed.is_empty() || trimmed == "pub" || trimmed.ends_with([';', '{', '}'])
}

/// `CG5`. Remove every balanced qualified path span that carries a cast token.
fn drop_qualified_paths(text: &[char]) -> Vec<char> {
    let mut out = text.to_vec();
    let mut position = 0;
    while position < out.len() {
        let opens = out.get(position) == Some(&'<');
        let joined = position
            .checked_sub(1)
            .and_then(|at| out.get(at))
            .is_some_and(|mark| matches!(*mark, '-' | '=' | '<' | '>'));
        if !opens || joined {
            position += 1;
            continue;
        }
        match qualified_path_span(&out, position) {
            Some(close) => {
                blank_span(&mut out, position, close);
                position = close + 1;
            },
            None => position += 1,
        }
    }
    out
}

/// `CG3b` and `CG5`. The end of an exempt qualified path span, or `None`.
///
/// The span is exempt only when it closes on a matching `>`, carries exactly
/// one whole-word cast token at its own depth, and holds one expression-free
/// type path on each side of that token.
fn qualified_path_span(text: &[char], open_index: usize) -> Option<usize> {
    let close = angle_close(text, open_index)?;
    let inner = text.get(open_index + 1..close)?;
    let positions: Vec<usize> = word_positions(inner, CAST_TOKEN)
        .into_iter()
        .filter(|start| balanced_angles(inner, *start))
        .collect();
    let [only] = positions.as_slice() else {
        return None;
    };
    let before = inner.get(..*only)?;
    let after = inner.get(only + CAST_TOKEN.chars().count()..)?;
    let split = after.iter().position(|mark| *mark == '<');
    let head = match split {
        Some(at) => after.get(..at)?,
        None => after,
    };
    if split.is_some() {
        let tail: String = after.iter().collect();
        if !tail.trim_end().ends_with('>') {
            return None;
        }
    }
    (type_path(before) && type_path(head)).then_some(close)
}

/// Whether the angle brackets before one offset balance.
fn balanced_angles(inner: &[char], start: usize) -> bool {
    let Some(head) = inner.get(..start) else {
        return false;
    };
    let opens = head.iter().filter(|mark| **mark == '<').count();
    let closes = head.iter().filter(|mark| **mark == '>').count();
    opens == closes
}

/// The offset of the `>` that closes the `<` at one offset, or `None`.
fn angle_close(text: &[char], open_index: usize) -> Option<usize> {
    let mut depth = 0_usize;
    let mut cursor = open_index;
    while let Some(&mark) = text.get(cursor) {
        if mark == '<' {
            depth += 1;
        } else if mark == '>' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(cursor);
            }
        } else if matches!(mark, ';' | '{' | '}' | '(' | ')' | '[' | ']') {
            return None;
        }
        cursor += 1;
    }
    None
}

/// Whether one span is one type path and holds no expression token.
///
/// `CG3b` prefers a false red to a miss, so the test is strict on both sides of
/// the cast token. A call, a field access, an operator, a literal, and a space
/// between two identifiers all fail it.
fn type_path(span: &[char]) -> bool {
    let text: String = span.iter().collect();
    let stripped = text.trim();
    if stripped.is_empty() {
        return false;
    }
    let shaped: Vec<char> = stripped.chars().collect();
    if !type_path_shape(&shaped) {
        return false;
    }
    !shaped.iter().any(|mark| is_expression_mark(*mark)) && !holds_bounded_digit(&shaped)
}

/// Whether one span has the shape of a type path.
fn type_path_shape(text: &[char]) -> bool {
    let mut cursor = usize::from(text.first() == Some(&'&'));
    if starts_with(text, cursor, "mut") {
        let spaced = skip_space(text, cursor + 3);
        if spaced > cursor + 3 {
            cursor = spaced;
        }
    }
    let Some(after) = ident_end(text, cursor) else {
        return false;
    };
    cursor = after;
    loop {
        let colons = skip_space(text, cursor);
        if !starts_with(text, colons, "::") {
            break;
        }
        let named = skip_space(text, colons + 2);
        let Some(after_name) = ident_end(text, named) else {
            return false;
        };
        cursor = after_name;
    }
    cursor == text.len()
}

/// The offset just after the identifier that opens at one offset, or `None`.
fn ident_end(text: &[char], start: usize) -> Option<usize> {
    if !text.get(start).is_some_and(|mark| ascii_ident_start(*mark)) {
        return None;
    }
    let mut cursor = start + 1;
    while text.get(cursor).is_some_and(|mark| ascii_word(*mark)) {
        cursor += 1;
    }
    Some(cursor)
}

/// Whether one character is an expression token a type path never holds.
const fn is_expression_mark(mark: char) -> bool {
    matches!(
        mark,
        '(' | ')'
            | '{'
            | '}'
            | '['
            | ']'
            | '.'
            | ','
            | ';'
            | '+'
            | '-'
            | '*'
            | '/'
            | '%'
            | '!'
            | '&'
            | '|'
            | '^'
            | '<'
            | '>'
            | '='
            | '?'
            | '"'
            | '\''
    )
}

/// Whether one span holds a digit that opens a word.
fn holds_bounded_digit(text: &[char]) -> bool {
    text.iter().enumerate().any(|(index, mark)| {
        mark.is_ascii_digit()
            && !index
                .checked_sub(1)
                .and_then(|at| text.get(at))
                .is_some_and(|prior| is_word(*prior))
    })
}

/// `CG6`. The line of every cast suppression one file carries.
///
/// The rule accepts the `allow` spelling and the `expect` spelling, the outer
/// form and the inner form, and a `cfg_attr` wrapper, because each one silences
/// the lint and none of them silences this text scan.
fn suppression_lines(text: &[char]) -> Vec<usize> {
    attribute_spans(text)
        .into_iter()
        .filter(|(_, body_start, body_end)| {
            text.get(*body_start..*body_end).is_some_and(suppression)
        })
        .map(|(start, _, _)| line_of(text, start))
        .collect()
}

/// Every attribute of one lexed file, as `(start, body start, body end)`.
///
/// An attribute opens at an outer or an inner bracket, with whitespace anywhere
/// between the tokens, and it closes at its own matching bracket.
fn attribute_spans(text: &[char]) -> Vec<(usize, usize, usize)> {
    let mut spans = Vec::new();
    let mut index = 0;
    while index < text.len() {
        let Some(body_start) = attribute_open(text, index) else {
            index += 1;
            continue;
        };
        if let Some(close) = bracket_close(text, body_start - 1) {
            spans.push((index, body_start, close));
        }
        index = body_start;
    }
    spans
}

/// The offset just after the `[` of an attribute that opens at one offset.
fn attribute_open(text: &[char], index: usize) -> Option<usize> {
    if text.get(index) != Some(&'#') {
        return None;
    }
    let cursor = skip_space(text, index + 1);
    let cursor = if text.get(cursor) == Some(&'!') {
        skip_space(text, cursor + 1)
    } else {
        cursor
    };
    take_word(text, cursor, "[")
}

/// The offset of the `]` that closes the `[` at one offset, or `None`.
fn bracket_close(text: &[char], open_index: usize) -> Option<usize> {
    let mut depth = 0_usize;
    let mut cursor = open_index;
    while let Some(&mark) = text.get(cursor) {
        if mark == '[' {
            depth += 1;
        } else if mark == ']' {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                return Some(cursor);
            }
        }
        cursor += 1;
    }
    None
}

/// Whether one attribute body suppresses the cast lint.
fn suppression(body: &[char]) -> bool {
    ["allow", "expect"]
        .into_iter()
        .any(|word| suppression_word(body, word))
}

/// Whether one attribute body suppresses the cast lint with one spelling.
fn suppression_word(body: &[char], word: &str) -> bool {
    (0..body.len()).any(|start| {
        !preceded_by_word(body, start)
            && starts_with(body, start, word)
            && suppression_tail(body, start + word.chars().count())
    })
}

/// Whether the tail of one suppression names the cast lint.
fn suppression_tail(body: &[char], from: usize) -> bool {
    let cursor = skip_space(body, from);
    let Some(cursor) = take_word(body, cursor, "(") else {
        return false;
    };
    let cursor = skip_space(body, cursor);
    let Some(cursor) = take_word(body, cursor, "clippy") else {
        return false;
    };
    let cursor = skip_space(body, cursor);
    let Some(cursor) = take_word(body, cursor, "::") else {
        return false;
    };
    let cursor = skip_space(body, cursor);
    let Some(cursor) = take_word(body, cursor, "as_conversions") else {
        return false;
    };
    !body.get(cursor).is_some_and(|mark| is_word(*mark))
}

/// The offset just after one literal token, or `None` when it is absent.
fn take_word(text: &[char], start: usize, word: &str) -> Option<usize> {
    starts_with(text, start, word).then(|| start + word.chars().count())
}

/// Whether one span opens with one literal token.
fn starts_with(text: &[char], start: usize, word: &str) -> bool {
    word.chars()
        .enumerate()
        .all(|(offset, wanted)| text.get(start + offset) == Some(&wanted))
}

/// The offset of the first non-whitespace character at or after one offset.
fn skip_space(text: &[char], start: usize) -> usize {
    let mut cursor = start;
    while text.get(cursor).is_some_and(|mark| mark.is_whitespace()) {
        cursor += 1;
    }
    cursor
}

/// Every whole-word occurrence of one token, in order.
fn word_positions(text: &[char], word: &str) -> Vec<usize> {
    let width = word.chars().count();
    let mut found = Vec::new();
    let mut index = 0;
    while index < text.len() {
        if word_at(text, index, word) {
            found.push(index);
            index += width;
        } else {
            index += 1;
        }
    }
    found
}

/// Whether one whole-word token opens at one offset.
fn word_at(text: &[char], index: usize, word: &str) -> bool {
    if !starts_with(text, index, word) {
        return false;
    }
    let after = index + word.chars().count();
    !preceded_by_word(text, index) && !text.get(after).is_some_and(|mark| is_word(*mark))
}

/// Whether a word character sits just before one offset.
fn preceded_by_word(text: &[char], index: usize) -> bool {
    index
        .checked_sub(1)
        .and_then(|at| text.get(at))
        .is_some_and(|mark| is_word(*mark))
}

/// Whether one character can carry a word.
fn is_word(mark: char) -> bool {
    mark.is_alphanumeric() || mark == '_'
}

/// Whether one character can open an identifier.
const fn ascii_ident_start(mark: char) -> bool {
    mark.is_ascii_alphabetic() || mark == '_'
}

/// Whether one character can continue an identifier.
const fn ascii_word(mark: char) -> bool {
    mark.is_ascii_alphanumeric() || mark == '_'
}

/// The one-based line that holds one offset.
fn line_of(text: &[char], index: usize) -> usize {
    text.get(..index)
        .map_or(0, |head| head.iter().filter(|mark| **mark == '\n').count())
        + 1
}

/// Every file under one directory, with no descent into a symbolic link or a
/// pruned directory.
fn walk_files(start: &Path, prune: &dyn Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut stack = vec![start.to_path_buf()];
    while let Some(directory) = stack.pop() {
        let Ok(entries) = fs::read_dir(&directory) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if !path.is_dir() {
                found.push(path);
                continue;
            }
            let plain = entry.file_type().is_ok_and(|kind| kind.is_dir());
            if plain && !prune(&path) {
                stack.push(path);
            }
        }
    }
    found
}

/// Whether one path names a Rust file, without regard to case.
fn is_rust_path(path: &Path) -> bool {
    path.file_name()
        .map(|name| name.to_string_lossy())
        .is_some_and(|name| {
            let mut tail = name.chars().rev();
            matches!(
                (tail.next(), tail.next(), tail.next()),
                (Some('s' | 'S'), Some('r' | 'R'), Some('.'))
            )
        })
}

/// One path with every symbolic link resolved, whether or not it exists.
///
/// The standard resolver refuses a path that does not exist, and `CG7` names a
/// file this workspace has yet to add, so the deepest existing ancestor is
/// resolved and the rest is appended.
fn realpath(path: &Path) -> PathBuf {
    if let Ok(resolved) = fs::canonicalize(path) {
        return resolved;
    }
    let (Some(parent), Some(name)) = (path.parent(), path.file_name()) else {
        return path.to_path_buf();
    };
    realpath(parent).join(name)
}

/// One path, spelled relative to one directory.
fn relative(path: &Path, start: &Path) -> PathBuf {
    let base: Vec<_> = start.components().collect();
    let target: Vec<_> = path.components().collect();
    let shared = base
        .iter()
        .zip(target.iter())
        .take_while(|(left, right)| left == right)
        .count();
    let mut out = PathBuf::new();
    for _ in shared..base.len() {
        out.push("..");
    }
    for part in target.iter().skip(shared) {
        out.push(part);
    }
    if out.as_os_str().is_empty() {
        out.push(".");
    }
    out
}

/// One path list, deduplicated and sorted by the bytes of the path.
fn sorted_paths(mut paths: Vec<PathBuf>) -> Vec<PathBuf> {
    paths.sort_by(|left, right| {
        left.as_os_str()
            .as_encoded_bytes()
            .cmp(right.as_os_str().as_encoded_bytes())
    });
    paths.dedup();
    paths
}

/// `CG9`. The site set and the code set of one run, keyed by the site name.
type ReasonPairs = BTreeMap<String, ReasonPair>;

/// `CG9`. What one site holds on each side of the binding.
#[derive(Debug, Default)]
struct ReasonPair {
    /// The text the `b1-convert` cell states, when a row names the site.
    cell: Option<String>,
    /// The text the `#[expect]` states, when the exempt file declares the site.
    code: Option<String>,
}

/// `CG9`. Read the appendix the caller named, or report that it does not open.
///
/// The `Err` holds the one fail-closed line the caller prints, in the shape
/// `CG8` already uses for a workspace that does not read.
fn appendix_rows(path: &Path) -> Result<Vec<(String, String)>, String> {
    let Ok(document) = fs::read_to_string(path) else {
        return Err(format!(
            "FAIL: {}: the appendix does not open; the guard is fail-closed.",
            path.display()
        ));
    };
    Ok(block_rows(&document))
}

/// `CG9`. Every site and reason text the `b1-convert` block states.
///
/// Cell one names the function and cell three holds the reason between its
/// outer quotation marks. The two rows under the marker are the table header
/// and the alignment row, which carry no site.
fn block_rows(document: &str) -> Vec<(String, String)> {
    let mut rows = document.lines().skip_while(|line| !opens_b1_convert(line));
    let _marker = rows.next();
    rows.take_while(|line| line.trim_start().starts_with('|'))
        .skip(2)
        .filter_map(|line| {
            let cells = split_row(line);
            let site = cells.first()?.trim_matches('`').trim().to_owned();
            let text = quoted_text(cells.get(2)?)?;
            (!site.is_empty()).then_some((site, text))
        })
        .collect()
}

/// `CG9`. Whether one line is the marker that opens the `b1-convert` block.
fn opens_b1_convert(line: &str) -> bool {
    let text = line.trim();
    text.starts_with(B1_CONVERT_MARKER) && text.ends_with("-->")
}

/// `CG9`. The cells of one markdown table row, as a renderer reads them.
///
/// The split takes every `|` the author did not escape, and each cell then
/// answers `\|` as the one vertical bar a reader sees. The first and the last
/// piece sit outside the table walls and carry no cell.
fn split_row(line: &str) -> Vec<String> {
    let mut cells = Vec::new();
    let mut cell = String::new();
    let mut escaped = false;
    for mark in line.trim().chars() {
        if escaped {
            if mark != '|' {
                cell.push('\\');
            }
            cell.push(mark);
            escaped = false;
        } else if mark == '\\' {
            escaped = true;
        } else if mark == '|' {
            cells.push(cell.trim().to_owned());
            cell = String::new();
        } else {
            cell.push(mark);
        }
    }
    if escaped {
        cell.push('\\');
    }
    cells.push(cell.trim().to_owned());
    let last = cells.len().saturating_sub(1);
    cells
        .into_iter()
        .enumerate()
        .filter(|(index, _piece)| *index > 0 && *index < last)
        .map(|(_index, piece)| piece)
        .collect()
}

/// `CG9`. The text one cell holds between its outer quotation marks.
fn quoted_text(cell: &str) -> Option<String> {
    let open = cell.find('"')?;
    let close = cell.rfind('"')?;
    cell.get(open.saturating_add(1)..close).map(str::to_owned)
}

/// `CG9`. Every site of the exempt file and the reason its `#[expect]` states.
///
/// The scan holds the attribute run above each item, so an `#[expect]` that
/// carries a `reason =` string binds to the `fn` item under it. A doc comment
/// and a blank line leave the run intact; every other line ends it.
fn reason_sites(source: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut pending: Option<String> = None;
    let mut attribute = String::new();
    for line in source.lines() {
        let text = line.trim();
        if !attribute.is_empty() || text.starts_with('#') {
            attribute.push_str(line);
            attribute.push('\n');
            if unclosed(&attribute) == 0 {
                pending = expect_reason(&attribute).or(pending);
                attribute.clear();
            }
        } else if let Some(name) = fn_name(text) {
            if let Some(reason) = pending.take() {
                found.push((name, reason));
            }
        } else if !text.is_empty() && !text.starts_with("//") {
            pending = None;
        }
    }
    found
}

/// `CG9`. How many `[` of one attribute span no `]` closes, literals apart.
fn unclosed(attribute: &str) -> usize {
    let text: Vec<char> = attribute.chars().collect();
    let mut depth = 0usize;
    let mut index = 0;
    while let Some(&mark) = text.get(index) {
        if let Some(end) = literal_at(&text, index, mark) {
            index = end;
            continue;
        }
        if mark == '[' {
            depth = depth.saturating_add(1);
        } else if mark == ']' {
            depth = depth.saturating_sub(1);
        }
        index = index.saturating_add(1);
    }
    depth
}

/// `CG9`. The decoded `reason =` text of one `#[expect(...)]` attribute.
///
/// An attribute that is not an `#[expect]`, and one that carries no `reason`
/// assignment outside a literal, each answer `None`.
fn expect_reason(attribute: &str) -> Option<String> {
    if !attribute.trim_start().starts_with("#[expect") {
        return None;
    }
    let text: Vec<char> = attribute.chars().collect();
    let mut index = 0;
    while let Some(&mark) = text.get(index) {
        if let Some(end) = literal_at(&text, index, mark) {
            index = end;
            continue;
        }
        if word_at(&text, index, REASON_TOKEN)
            && let Some(value) = reason_value(&text, index.saturating_add(REASON_TOKEN.len()))
        {
            return Some(value);
        }
        index = index.saturating_add(1);
    }
    None
}

/// `CG9`. The decoded text the `reason` assignment at one offset carries.
///
/// Two literals that sit side by side answer one text, so the value is the
/// concatenation of every literal the assignment states.
fn reason_value(text: &[char], from: usize) -> Option<String> {
    let equals = skip_space(text, from);
    if text.get(equals) != Some(&'=') {
        return None;
    }
    let (mut value, mut next) = decode_literal(text, skip_space(text, equals.saturating_add(1)))?;
    while let Some((more, after)) = decode_literal(text, skip_space(text, next)) {
        value.push_str(&more);
        next = after;
    }
    Some(value)
}

/// `CG9`. The decoded value of the string literal at one offset, and its end.
///
/// The answer is the text a reader sees and never the raw source token. A raw
/// literal keeps every backslash; a plain literal answers each escape
/// [`decode_escape`] names. A literal this decoder does not read answers
/// `None`, which leaves its site out of the code set and makes the row that
/// names the site a finding.
fn decode_literal(text: &[char], start: usize) -> Option<(String, usize)> {
    let mut index = start;
    let mut hashes = 0usize;
    let raw = text.get(index) == Some(&'r');
    if raw {
        index = index.saturating_add(1);
        while text.get(index) == Some(&'#') {
            hashes = hashes.saturating_add(1);
            index = index.saturating_add(1);
        }
    }
    if text.get(index) != Some(&'"') {
        return None;
    }
    index = index.saturating_add(1);
    let mut value = String::new();
    while let Some(&mark) = text.get(index) {
        let after = index.saturating_add(1);
        if mark == '"' && (!raw || closed_by_hashes(text, after, hashes)) {
            return Some((value, after.saturating_add(hashes)));
        }
        if !raw && mark == '\\' {
            let (decoded, next) = decode_escape(text, after)?;
            value.push_str(&decoded);
            index = next;
            continue;
        }
        value.push(mark);
        index = after;
    }
    None
}

/// `CG9`. Whether the stated count of `#` follows one raw-literal quote.
fn closed_by_hashes(text: &[char], after: usize, hashes: usize) -> bool {
    (0..hashes).all(|offset| text.get(after.saturating_add(offset)) == Some(&'#'))
}

/// `CG9`. The text one escape answers, and the offset just after it.
///
/// A backslash before a newline answers the empty text and drops the
/// whitespace under it, which is the continuation a long reason uses. The
/// decoder reads `n`, `r`, `t`, `0`, `\`, `"` and `'`; every other escape,
/// `\x` and `\u` included, answers `None`.
fn decode_escape(text: &[char], index: usize) -> Option<(String, usize)> {
    let &mark = text.get(index)?;
    let after = index.saturating_add(1);
    if mark == '\n' {
        return Some((String::new(), skip_space(text, after)));
    }
    let decoded = match mark {
        'n' => '\n',
        'r' => '\r',
        't' => '\t',
        '0' => '\0',
        '\\' => '\\',
        '"' => '"',
        '\'' => '\'',
        _ => return None,
    };
    Some((decoded.to_string(), after))
}

/// `CG9`. The name of the `fn` item one line opens, or `None`.
///
/// The line must open with item keywords alone, so a `fn` inside a body and a
/// `fn` inside a type never answers.
fn fn_name(line: &str) -> Option<String> {
    let mut words = line.split_whitespace();
    let mut word = words.next()?;
    while item_keyword(word) {
        word = words.next()?;
    }
    if word != "fn" {
        return None;
    }
    let spelled = words.next()?;
    let width = spelled
        .find(|mark: char| !is_word(mark))
        .unwrap_or(spelled.len());
    let name = spelled.get(..width)?;
    (!name.is_empty()).then(|| name.to_owned())
}

/// `CG9`. Whether one word may stand between a line start and its `fn`.
fn item_keyword(word: &str) -> bool {
    matches!(word, "pub" | "const" | "async" | "unsafe" | "extern")
        || word.starts_with("pub(")
        || word.starts_with('"')
}

/// `CG9`. The two sets as one map, keyed by the site each side names.
fn reason_pairs(rows: Vec<(String, String)>, sites: Vec<(String, String)>) -> ReasonPairs {
    let mut pairs = ReasonPairs::new();
    for (site, text) in rows {
        pairs.entry(site).or_default().cell = Some(text);
    }
    for (site, text) in sites {
        pairs.entry(site).or_default().code = Some(text);
    }
    pairs
}

/// `CG9`. Every line the binding fails on, in site order.
fn reason_findings(pairs: &ReasonPairs) -> Vec<String> {
    pairs
        .iter()
        .filter_map(|(site, pair)| {
            let line = match (pair.cell.as_deref(), pair.code.as_deref()) {
                (Some(cell), Some(code)) if cell == code => return None,
                (Some(cell), Some(code)) => {
                    format!("the cell states \"{cell}\" and the code states \"{code}\"")
                },
                (Some(_cell), None) => {
                    "the `b1-convert` row names a site the exempt file does not declare".to_owned()
                },
                (None, Some(_code)) => {
                    "the exempt file states a reason that no `b1-convert` row names".to_owned()
                },
                (None, None) => return None,
            };
            Some(format!("  REASON TEXT: {site}: {line}"))
        })
        .collect()
}
