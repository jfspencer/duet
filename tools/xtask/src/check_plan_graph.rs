//! Refuse a chunk file whose front matter breaks a section 13 rule.
//!
//! The guard reads every `*.md` file directly under the plan directory whose
//! first line is `---` and whose front matter carries `id`, `line`,
//! `depends_on`, `write_scope`, `parallelism`, and `completion`. A file without
//! that header is not a chunk.
//!
//! Each check is fail closed:
//! 1. Every id is unique and every `depends_on` id exists.
//! 2. The dependency graph is acyclic.
//! 3. The phase of each chunk (section 13.3 of `architecture.md`) is later than
//!    the phase of every dependency. The one same-phase link allowed comes from
//!    the opening manifest chunk of that phase.
//! 4. Two chunks of one phase never share a write-scope path, except
//!    `Cargo.lock`, and except a crate skeleton shared between the manifest
//!    chunk of the phase and a chunk that depends on it.
//! 5. Two chunks of one line never share a phase.
//! 6. Every chunk of section 13.3 has a file, and every file has a row.
//! 7. Every serial link of section 13.4 appears as a `depends_on` edge.
//! 8. Every file that opens a front-matter fence parses. A file that opens the
//!    fence and that the parser then rejects leaves the rules and the manifest
//!    with no trace, so it is a finding.
//!
//! `plan-graph.md` is generated, so [`ManifestMode::Check`] holds the file
//! against the text the front matter generates now. The rules run first: a run
//! that already has a finding reports that finding and reads the file no
//! further.

use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::io::{self, Write as _};
use std::path::Path;

use anyhow::Context as _;

use crate::Outcome;

/// Every key a chunk file must carry in its front matter.
const REQUIRED: [&str; 6] = [
    "id",
    "line",
    "depends_on",
    "write_scope",
    "parallelism",
    "completion",
];

/// The line that tells a reader `plan-graph.md` is generated.
const MANIFEST_NOTE: &str = "Derived from the chunk front-matter by `cargo xtask check-plan-graph <plan-dir> --write-manifest`. Edit the chunk files, then regenerate; never edit this file by hand.";

/// The objective paragraph of the generated manifest.
const OBJECTIVE: &str = "Duet v1: a vocal-first composition, record, mix, and master application on GPUI Kit, AI-first with a git-like history, on macOS 26 and Ubuntu 26.04. The measurable completion outcome is architecture.md section 14 (three rungs: `scripts/dod.sh` green on both platforms, the named test commands, and the human product review gate).";

/// The name of the generated manifest inside the plan directory.
const MANIFEST_FILE: &str = "plan-graph.md";

/// What one run does with the generated manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ManifestMode {
    /// Report the rules alone and leave `plan-graph.md` untouched.
    Report,
    /// Write `plan-graph.md` from the front matter.
    Write,
    /// Hold `plan-graph.md` against the front matter and report a difference.
    Check,
}

impl ManifestMode {
    /// The mode the two command-line flags name.
    ///
    /// Returns `None` for both flags at once, which is a usage failure: one
    /// run writes the manifest or reads it, never both.
    pub(crate) const fn select(write: bool, check: bool) -> Option<Self> {
        match (write, check) {
            (true, true) => None,
            (true, false) => Some(Self::Write),
            (false, true) => Some(Self::Check),
            (false, false) => Some(Self::Report),
        }
    }
}

/// Run the plan-graph guard over every chunk file of the plan directory.
///
/// # Errors
/// Returns an error when the plan directory, one chunk file, or the generated
/// manifest cannot be read, or when `plan-graph.md` cannot be written.
pub(crate) fn run(plan_dir: &Path, mode: ManifestMode) -> anyhow::Result<Outcome> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let arch_path = plan_dir.join("architecture.md");
    if !arch_path.is_file() {
        writeln!(out, "FAIL: cannot open {}", arch_path.display())?;
        return Ok(Outcome::FailClosed);
    }
    let arch = fs::read_to_string(&arch_path)
        .with_context(|| format!("cannot read {}", arch_path.display()))?;
    let phases = match read_phases(&arch) {
        Ok(phases) => phases,
        Err(reason) => {
            writeln!(out, "FAIL: {reason}")?;
            return Ok(Outcome::FailClosed);
        },
    };
    let links = match read_links(&arch) {
        Ok(links) => links,
        Err(reason) => {
            writeln!(out, "FAIL: {reason}")?;
            return Ok(Outcome::FailClosed);
        },
    };
    let collected = match collect_chunks(plan_dir)? {
        Ok(collected) => collected,
        Err(stop) => {
            writeln!(out, "{}", stop.line())?;
            return Ok(stop.outcome());
        },
    };
    let Collected { chunks, malformed } = collected;
    let mut report = Report::default();
    for name in malformed {
        report.finding(format!("{name} opens front matter the guard cannot parse"));
    }
    check_dependencies(&chunks, &mut report);
    check_cycles(&chunks, &mut report);
    check_rows(&chunks, &phases, &mut report);
    check_phase_links(&chunks, &phases, &mut report);
    let groups = group_by_phase(&chunks, &phases);
    check_phase_scopes(&chunks, &groups, &mut report);
    check_link_edges(&links, &chunks, &mut report);
    if mode == ManifestMode::Check && report.findings.is_empty() {
        let wanted = manifest_text(&chunks, &phases, &groups);
        if let Some(finding) = manifest_drift(plan_dir, &wanted)? {
            report.finding(finding);
        }
    }
    for finding in &report.findings {
        writeln!(out, "FINDING: {finding}")?;
    }
    writeln!(
        out,
        "CHUNKS: {}   PHASES: {}   LINKS 13.4: {}   FINDINGS: {}",
        chunks.count(),
        groups.count(),
        links.len(),
        report.findings.len()
    )?;
    if mode == ManifestMode::Write && report.findings.is_empty() {
        let text = manifest_text(&chunks, &phases, &groups);
        let path = plan_dir.join(MANIFEST_FILE);
        fs::write(&path, text).with_context(|| format!("cannot write {}", path.display()))?;
        writeln!(out, "MANIFEST: {MANIFEST_FILE} written")?;
    }
    Ok(if report.findings.is_empty() {
        Outcome::Clean
    } else {
        Outcome::Findings
    })
}

/// The finding `--check-manifest` reports, or `None` when the file is current.
///
/// A file the front matter no longer generates and a file that is not there
/// are one finding, because both leave the operator with a manifest that no
/// longer states the plan.
///
/// # Errors
/// Returns an error when the file is there and cannot be read.
fn manifest_drift(plan_dir: &Path, wanted: &str) -> anyhow::Result<Option<String>> {
    let path = plan_dir.join(MANIFEST_FILE);
    let drift = format!(
        "{} is out of step with the chunk front-matter; regenerate it with --write-manifest",
        path.display()
    );
    match fs::read_to_string(&path) {
        Ok(found) => Ok((found != wanted).then_some(drift)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Some(drift)),
        Err(error) => Err(error).with_context(|| format!("cannot read {}", path.display())),
    }
}

/// One front-matter value.
#[derive(Debug, Clone)]
enum Value {
    /// A single scalar string.
    Scalar(String),
    /// A list of strings.
    List(Vec<String>),
}

/// The front-matter mapping of one file, in source order.
#[derive(Debug, Default)]
struct FrontMatter {
    /// Key and value pairs in the order the file states them.
    entries: Vec<(String, Value)>,
}

impl FrontMatter {
    /// Replace the value of one key, or add the key.
    fn set(&mut self, key: &str, value: Value) {
        if let Some(entry) = self.entries.iter_mut().find(|(name, _)| name == key) {
            entry.1 = value;
        } else {
            self.entries.push((key.to_owned(), value));
        }
    }

    /// Append one item to the list value of a key.
    ///
    /// The `Err` case is a key that already holds a scalar, which makes the
    /// file malformed and therefore not a chunk.
    fn append(&mut self, key: &str, item: String) -> Result<(), ()> {
        match self.entries.iter_mut().find(|(name, _)| name == key) {
            Some((_, Value::List(items))) => {
                items.push(item);
                Ok(())
            },
            Some((_, Value::Scalar(_))) => Err(()),
            None => {
                self.entries.push((key.to_owned(), Value::List(vec![item])));
                Ok(())
            },
        }
    }

    /// The value of one key.
    fn get(&self, key: &str) -> Option<&Value> {
        self.entries
            .iter()
            .find(|(name, _)| name == key)
            .map(|(_, value)| value)
    }

    /// The scalar value of one key.
    fn scalar(&self, key: &str) -> Option<&str> {
        match self.get(key) {
            Some(Value::Scalar(text)) => Some(text),
            Some(Value::List(_)) | None => None,
        }
    }

    /// The list value of one key.
    fn list(&self, key: &str) -> Option<&[String]> {
        match self.get(key) {
            Some(Value::List(items)) => Some(items),
            Some(Value::Scalar(_)) | None => None,
        }
    }
}

/// Return the front matter of one file, or `None` when the file is not a chunk.
fn parse_front_matter(text: &str) -> Option<FrontMatter> {
    let lines: Vec<&str> = text.lines().collect();
    if lines.first()?.trim() != "---" {
        return None;
    }
    let end = lines.iter().skip(1).position(|line| *line == "---")? + 1;
    let mut front = FrontMatter::default();
    let mut key: Option<String> = None;
    for raw in lines.get(1..end)? {
        let line = if raw.trim_start().starts_with('-') {
            raw.trim_end()
        } else {
            raw.split_once('#')
                .map_or(*raw, |(head, _)| head)
                .trim_end()
        };
        if line.trim().is_empty() {
            continue;
        }
        if let (Some(item), Some(name)) = (list_item(line), key.as_ref()) {
            front.append(name, item).ok()?;
            continue;
        }
        let Some((name, value)) = split_entry(line) else {
            continue;
        };
        front.set(name, value);
        key = Some(name.to_owned());
    }
    REQUIRED
        .iter()
        .all(|name| front.get(name).is_some())
        .then_some(front)
}

/// The item text of a `  - ` list line.
fn list_item(line: &str) -> Option<String> {
    let rest = line.strip_prefix("  - ")?.trim();
    Some(
        rest.split_once('#')
            .map_or(rest, |(head, _)| head)
            .trim()
            .to_owned(),
    )
}

/// Split a `key: value` line into its key and its parsed value.
fn split_entry(line: &str) -> Option<(&str, Value)> {
    let end = line.find(|c: char| !(c.is_ascii_lowercase() || c == '_'))?;
    if end == 0 {
        return None;
    }
    let (key, rest) = line.split_at(end);
    Some((key, parse_value(rest.strip_prefix(':')?.trim())))
}

/// Parse one front-matter value: an inline list, an empty list, or a scalar.
fn parse_value(value: &str) -> Value {
    if value.starts_with('[') {
        let inner = value.trim_matches(|c| c == '[' || c == ']');
        Value::List(
            inner
                .split(',')
                .map(str::trim)
                .filter(|item| !item.is_empty())
                .map(ToOwned::to_owned)
                .collect(),
        )
    } else if value.is_empty() {
        Value::List(Vec::new())
    } else {
        Value::Scalar(value.trim_matches('"').to_owned())
    }
}

/// One chunk file and the front-matter fields the guard reads.
#[derive(Debug)]
struct Chunk {
    /// The `id` field.
    id: String,
    /// The `line` field.
    line: String,
    /// The `depends_on` field.
    depends_on: Vec<String>,
    /// The `write_scope` field.
    write_scope: Vec<String>,
    /// The name of the file the chunk came from.
    file_name: String,
}

/// Every chunk of the plan directory, in file-name order.
#[derive(Debug, Default)]
struct ChunkSet {
    /// The chunks in the order the guard read them.
    items: Vec<Chunk>,
    /// Chunk id to position in `items`.
    index: HashMap<String, usize>,
}

impl ChunkSet {
    /// The chunk with one id.
    fn get(&self, id: &str) -> Option<&Chunk> {
        self.index
            .get(id)
            .and_then(|position| self.items.get(*position))
    }

    /// True when the set holds a chunk with one id.
    fn contains(&self, id: &str) -> bool {
        self.index.contains_key(id)
    }

    /// How many chunks the set holds.
    const fn count(&self) -> usize {
        self.items.len()
    }

    /// The chunks in read order.
    fn iter(&self) -> std::slice::Iter<'_, Chunk> {
        self.items.iter()
    }
}

/// What one read of the plan directory produced.
#[derive(Debug, Default)]
struct Collected {
    /// Every file the guard read as a chunk.
    chunks: ChunkSet,
    /// The name of every file that states a chunk key in a front-matter fence
    /// the parser rejects.
    malformed: Vec<String>,
}

/// True when the first line of one file opens a fence that states a chunk key.
///
/// A file that opens with a horizontal rule, and a file that carries front
/// matter of its own such as `title:`, break no chunk rule. Only a file that
/// states a key of the chunk contract is held to that contract.
fn states_a_chunk_key(text: &str) -> bool {
    let mut lines = text.lines();
    if lines.next().is_none_or(|line| line.trim() != "---") {
        return false;
    }
    lines
        .take_while(|line| *line != "---")
        .filter_map(|line| split_entry(line.trim_end()))
        .any(|(key, _)| REQUIRED.contains(&key))
}

/// Why one read of the plan directory stopped before the rules ran.
///
/// The two stops differ in the word they print AND in the exit code they
/// decide, so each arm carries its own whole line and its own [`Outcome`].
#[derive(Debug)]
enum Stop {
    /// Two chunk files state one id, which breaks rule 1 of this guard.
    ///
    /// The field is the whole line the guard prints.
    DuplicateId(String),
    /// The plan directory holds no chunk file, so the guard measured nothing.
    NoChunkFile,
}

impl Stop {
    /// The part of the line that follows the word.
    fn body(&self) -> &str {
        match self {
            Self::DuplicateId(body) => body,
            Self::NoChunkFile => "no chunk file found; the guard is fail-closed.",
        }
    }

    /// The exit code this stop decides.
    const fn outcome(&self) -> Outcome {
        match self {
            Self::DuplicateId(_) => Outcome::Findings,
            Self::NoChunkFile => Outcome::FailClosed,
        }
    }

    /// The whole line the guard prints for this stop.
    ///
    /// The word comes from [`Self::outcome`] and never from the arm, so a
    /// later arm cannot print `FINDING:` at exit 2 or `FAIL:` at exit 1.
    /// ADR 0010 decision 1 binds the word to the exit code.
    fn line(&self) -> String {
        format!("{} {}", word(self.outcome()), self.body())
    }
}

/// The word one exit code decides (ADR 0010 decision 1).
///
/// Exit 1 is a breach of a rule the guard measures about the plan it reads,
/// and exit 2 is a failure of the guard's own input. [`Outcome::Clean`] takes
/// the fail-closed word because no [`Stop`] decides it: a stop that somehow
/// reported a clean run is a defect, and `FAIL:` is the safe answer to one.
const fn word(outcome: Outcome) -> &'static str {
    match outcome {
        Outcome::Findings => "FINDING:",
        Outcome::Clean | Outcome::FailClosed => "FAIL:",
    }
}

/// Read every chunk file of the plan directory.
///
/// The inner `Err` holds the stop that ends the run before the rules read the
/// chunk set.
///
/// # Errors
/// Returns an error when the plan directory or one chunk file cannot be read.
fn collect_chunks(plan_dir: &Path) -> anyhow::Result<Result<Collected, Stop>> {
    let entries =
        fs::read_dir(plan_dir).with_context(|| format!("cannot read {}", plan_dir.display()))?;
    let mut names: Vec<String> = Vec::new();
    for entry in entries {
        let entry = entry.with_context(|| format!("cannot read {}", plan_dir.display()))?;
        if entry
            .path()
            .extension()
            .is_some_and(|extension| extension == "md")
        {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    let mut collected = Collected::default();
    for name in names {
        let path = plan_dir.join(&name);
        let text =
            fs::read_to_string(&path).with_context(|| format!("cannot read {}", path.display()))?;
        let fenced = states_a_chunk_key(&text);
        let Some(front) = parse_front_matter(&text) else {
            if fenced {
                collected.malformed.push(name);
            }
            continue;
        };
        let (Some(id), Some(line), Some(depends_on), Some(write_scope)) = (
            front.scalar("id"),
            front.scalar("line"),
            front.list("depends_on"),
            front.list("write_scope"),
        ) else {
            if fenced {
                collected.malformed.push(name);
            }
            continue;
        };
        if let Some(first) = collected.chunks.get(id) {
            return Ok(Err(Stop::DuplicateId(format!(
                "duplicate chunk id {id} in {name} and {}",
                first.file_name
            ))));
        }
        let position = collected.chunks.items.len();
        collected.chunks.index.insert(id.to_owned(), position);
        collected.chunks.items.push(Chunk {
            id: id.to_owned(),
            line: line.to_owned(),
            depends_on: depends_on.to_vec(),
            write_scope: write_scope.to_vec(),
            file_name: name,
        });
    }
    if collected.chunks.items.is_empty() {
        return Ok(Err(Stop::NoChunkFile));
    }
    Ok(Ok(collected))
}

/// The section 13.3 phase table.
#[derive(Debug, Default)]
struct PhaseTable {
    /// Chunk ids in the order the table names them.
    order: Vec<String>,
    /// Chunk id to phase number.
    phase: HashMap<String, u32>,
}

impl PhaseTable {
    /// Record one chunk id and its phase.
    fn insert(&mut self, id: &str, phase: u32) {
        if !self.phase.contains_key(id) {
            self.order.push(id.to_owned());
        }
        self.phase.insert(id.to_owned(), phase);
    }

    /// The phase of one chunk id.
    fn get(&self, id: &str) -> Option<u32> {
        self.phase.get(id).copied()
    }
}

/// Read the section 13.3 phase table: chunk id to phase.
fn read_phases(arch: &str) -> Result<PhaseTable, String> {
    let Some((_, after)) = arch.split_once("### 13.3") else {
        return Err("section 13.3 not found in architecture.md".to_owned());
    };
    let body = after.split_once("### 13.4").map_or(after, |(head, _)| head);
    let mut phases = PhaseTable::default();
    for line in body.lines() {
        let Some((phase, owner, running)) = phase_row(line) else {
            continue;
        };
        for cell in [owner, running] {
            for id in chunk_ids(cell) {
                phases.insert(&id, phase);
            }
        }
    }
    if phases.order.is_empty() {
        return Err("section 13.3 phase table parsed to zero rows".to_owned());
    }
    Ok(phases)
}

/// Split one phase-table row into its phase, its manifest owner cell, and its
/// line-chunk cell.
fn phase_row(line: &str) -> Option<(u32, &str, &str)> {
    if !line.starts_with('|') {
        return None;
    }
    let fields: Vec<&str> = line.split('|').collect();
    if fields.len() < 6 {
        return None;
    }
    let number = fields.get(1)?.trim();
    if number.is_empty() || !number.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    Some((
        number.parse().ok()?,
        fields.get(2).copied()?,
        fields.get(4).copied()?,
    ))
}

/// Read the section 13.4 links of the form `X before Y` as `(X, Y)` edges.
fn read_links(arch: &str) -> Result<BTreeSet<(String, String)>, String> {
    let Some((_, after)) = arch.split_once("### 13.4") else {
        return Err("section 13.4 not found in architecture.md".to_owned());
    };
    let body = after.split_once("\n## ").map_or(after, |(head, _)| head);
    let mut links = BTreeSet::new();
    for line in body.lines() {
        if !line.starts_with('|') {
            continue;
        }
        let Some(cell) = line.split('|').nth(1) else {
            continue;
        };
        let Some((sources, targets)) = parse_link(cell) else {
            continue;
        };
        for source in &sources {
            for target in &targets {
                links.insert((source.clone(), target.clone()));
            }
        }
    }
    Ok(links)
}

/// A scan position inside one link cell.
#[derive(Debug)]
struct Cursor<'a> {
    /// The characters of the cell.
    chars: &'a [char],
    /// The position the next step starts from.
    at: usize,
}

impl Cursor<'_> {
    /// Skip every whitespace character at the cursor.
    fn skip_spaces(&mut self) {
        while self.chars.get(self.at).is_some_and(|c| c.is_whitespace()) {
            self.at += 1;
        }
    }

    /// Skip one or more whitespace characters at the cursor.
    fn skip_some_spaces(&mut self) -> bool {
        let start = self.at;
        self.skip_spaces();
        self.at > start
    }

    /// Take one literal word at the cursor.
    fn take_word(&mut self, word: &str) -> bool {
        for (offset, wanted) in word.chars().enumerate() {
            if self.chars.get(self.at + offset) != Some(&wanted) {
                return false;
            }
        }
        self.at += word.chars().count();
        true
    }

    /// Take one uppercase letter plus digits token at the cursor.
    fn take_id(&mut self) -> Option<String> {
        let end = id_end(self.chars, self.at)?;
        let token: String = self.chars.get(self.at..end)?.iter().collect();
        self.at = end;
        Some(token)
    }

    /// Take a comma separator and the id after it.
    fn take_comma_id(&mut self) -> Option<String> {
        let mut probe = Cursor {
            chars: self.chars,
            at: self.at,
        };
        probe.skip_spaces();
        if probe.chars.get(probe.at) != Some(&',') {
            return None;
        }
        probe.at += 1;
        probe.skip_spaces();
        let id = probe.take_id()?;
        self.at = probe.at;
        Some(id)
    }

    /// Take an `and` separator and the id after it.
    fn take_and_id(&mut self) -> Option<String> {
        let mut probe = Cursor {
            chars: self.chars,
            at: self.at,
        };
        if !probe.skip_some_spaces() || !probe.take_word("and") || !probe.skip_some_spaces() {
            return None;
        }
        let id = probe.take_id()?;
        self.at = probe.at;
        Some(id)
    }

    /// Take the `before` keyword and the whitespace around it.
    fn take_before(&mut self) -> bool {
        let mut probe = Cursor {
            chars: self.chars,
            at: self.at,
        };
        if !probe.skip_some_spaces() || !probe.take_word("before") || !probe.skip_some_spaces() {
            return false;
        }
        self.at = probe.at;
        true
    }
}

/// Parse one section 13.4 link cell into its predecessor and successor ids.
fn parse_link(cell: &str) -> Option<(Vec<String>, Vec<String>)> {
    let chars: Vec<char> = cell.chars().collect();
    let mut cursor = Cursor {
        chars: &chars,
        at: 0,
    };
    cursor.skip_spaces();
    let mut sources = vec![cursor.take_id()?];
    while let Some(id) = cursor.take_comma_id() {
        sources.push(id);
    }
    if !cursor.take_before() {
        return None;
    }
    let mut targets = vec![cursor.take_id()?];
    while let Some(id) = cursor.take_comma_id().or_else(|| cursor.take_and_id()) {
        targets.push(id);
    }
    Some((sources, targets))
}

/// The end of an uppercase letter plus digits token that starts at `at`.
fn id_end(chars: &[char], at: usize) -> Option<usize> {
    if !chars.get(at)?.is_ascii_uppercase() {
        return None;
    }
    let mut end = at + 1;
    while chars.get(end).is_some_and(char::is_ascii_digit) {
        end += 1;
    }
    if end == at + 1 { None } else { Some(end) }
}

/// True when the character is a word character for a boundary test.
fn is_word(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

/// Every chunk id of one table cell, in order.
fn chunk_ids(cell: &str) -> Vec<String> {
    let chars: Vec<char> = cell.chars().collect();
    let mut ids = Vec::new();
    let mut start = 0_usize;
    while start < chars.len() {
        if let Some(end) = bounded_id_end(&chars, start) {
            if let Some(token) = chars.get(start..end) {
                ids.push(token.iter().collect());
            }
            start = end;
        } else {
            start += 1;
        }
    }
    ids
}

/// The end of a chunk id that starts at `start` and stands between two word
/// boundaries.
fn bounded_id_end(chars: &[char], start: usize) -> Option<usize> {
    let before = start.checked_sub(1).and_then(|index| chars.get(index));
    if before.is_some_and(|c| is_word(*c)) {
        return None;
    }
    let end = id_end(chars, start)?;
    if chars.get(end).is_some_and(|c| is_word(*c)) {
        return None;
    }
    Some(end)
}

/// The findings of one run, in the order the checks produce them.
#[derive(Debug, Default)]
struct Report {
    /// One message per finding.
    findings: Vec<String>,
}

impl Report {
    /// Record one finding.
    fn finding(&mut self, message: String) {
        self.findings.push(message);
    }
}

/// Report every `depends_on` id that has no chunk file.
fn check_dependencies(chunks: &ChunkSet, report: &mut Report) {
    for chunk in chunks.iter() {
        for dep in &chunk.depends_on {
            if !chunks.contains(dep) {
                report.finding(format!(
                    "{} depends on {dep}, which has no chunk file",
                    chunk.id
                ));
            }
        }
    }
}

/// Where the depth-first walk stands on one chunk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mark {
    /// The walk entered the chunk and has not left it.
    Open,
    /// The walk left the chunk.
    Closed,
}

/// One step of the depth-first walk.
///
/// The walk holds its own stack of these steps. A recursive walk of a plan
/// whose dependency chain is long enough overflows the process stack and
/// aborts, which is a status outside the three the guard contract states.
#[derive(Debug, Clone, Copy)]
enum Step<'a> {
    /// Enter one chunk.
    Enter(&'a str),
    /// Leave one chunk the walk entered.
    Leave(&'a str),
}

/// Where one depth-first walk of the dependency graph stands.
#[derive(Debug, Default)]
struct Walk<'a> {
    /// Where the walk stands on each chunk it reached.
    state: HashMap<&'a str, Mark>,
    /// The chunks the walk entered and has not left, in entry order.
    chain: Vec<&'a str>,
    /// The steps the walk has still to take, the next one last.
    steps: Vec<Step<'a>>,
}

impl<'a> Walk<'a> {
    /// Walk one chunk and every dependency below it.
    fn run(&mut self, root: &'a str, chunks: &'a ChunkSet, report: &mut Report) {
        self.steps.push(Step::Enter(root));
        while let Some(step) = self.steps.pop() {
            match step {
                Step::Enter(id) => self.enter(id, chunks, report),
                Step::Leave(id) => {
                    self.chain.pop();
                    self.state.insert(id, Mark::Closed);
                },
            }
        }
    }

    /// Enter one chunk and queue every dependency below it.
    fn enter(&mut self, id: &'a str, chunks: &'a ChunkSet, report: &mut Report) {
        match self.state.get(id) {
            Some(Mark::Open) => {
                self.chain.push(id);
                report.finding(format!("cycle: {}", self.chain.join(" -> ")));
                self.chain.pop();
                return;
            },
            Some(Mark::Closed) => return,
            None => {},
        }
        self.state.insert(id, Mark::Open);
        self.chain.push(id);
        self.steps.push(Step::Leave(id));
        let Some(chunk) = chunks.get(id) else {
            return;
        };
        for dep in chunk.depends_on.iter().rev() {
            if let Some(next) = chunks.get(dep) {
                self.steps.push(Step::Enter(next.id.as_str()));
            }
        }
    }
}

/// Report every cycle of the dependency graph.
fn check_cycles(chunks: &ChunkSet, report: &mut Report) {
    let mut walk = Walk::default();
    for chunk in chunks.iter() {
        walk.run(&chunk.id, chunks, report);
    }
}

/// Report every chunk file with no phase row and every phase row with no file.
fn check_rows(chunks: &ChunkSet, phases: &PhaseTable, report: &mut Report) {
    for chunk in chunks.iter() {
        if phases.get(&chunk.id).is_none() {
            report.finding(format!(
                "{} has a file and no row in section 13.3",
                chunk.id
            ));
        }
    }
    for id in &phases.order {
        if !chunks.contains(id) {
            report.finding(format!("{id} has a row in section 13.3 and no chunk file"));
        }
    }
}

/// Report every backward link and every same-phase link.
fn check_phase_links(chunks: &ChunkSet, phases: &PhaseTable, report: &mut Report) {
    for chunk in chunks.iter() {
        for dep in &chunk.depends_on {
            let (Some(here), Some(there)) = (phases.get(&chunk.id), phases.get(dep)) else {
                continue;
            };
            if there > here {
                report.finding(format!(
                    "{} (phase {here}) depends on {dep} (phase {there}), a backward link",
                    chunk.id
                ));
            }
            if there == here && !dep.starts_with('M') {
                report.finding(format!(
                    "{} and {dep} share phase {here} with a link between them",
                    chunk.id
                ));
            }
        }
    }
}

/// The chunks of each phase, in first-encounter order.
#[derive(Debug, Default)]
struct PhaseGroups {
    /// Phase number and the chunk ids of that phase.
    groups: Vec<(u32, Vec<String>)>,
}

impl PhaseGroups {
    /// Record one chunk in its phase.
    fn push(&mut self, phase: u32, id: &str) {
        if let Some(group) = self.groups.iter_mut().find(|(number, _)| *number == phase) {
            group.1.push(id.to_owned());
        } else {
            self.groups.push((phase, vec![id.to_owned()]));
        }
    }

    /// How many phases hold at least one chunk.
    const fn count(&self) -> usize {
        self.groups.len()
    }

    /// How many chunks the widest phase holds.
    fn widest(&self) -> usize {
        self.groups
            .iter()
            .map(|(_, ids)| ids.len())
            .max()
            .unwrap_or(0)
    }

    /// The groups in phase order, each with its ids sorted.
    fn sorted(&self) -> Vec<(u32, Vec<String>)> {
        let mut sorted: Vec<(u32, Vec<String>)> = self
            .groups
            .iter()
            .map(|(phase, ids)| {
                let mut ids = ids.clone();
                ids.sort();
                (*phase, ids)
            })
            .collect();
        sorted.sort_by_key(|(phase, _)| *phase);
        sorted
    }
}

/// Group every chunk that has a phase row by its phase.
fn group_by_phase(chunks: &ChunkSet, phases: &PhaseTable) -> PhaseGroups {
    let mut groups = PhaseGroups::default();
    for chunk in chunks.iter() {
        if let Some(phase) = phases.get(&chunk.id) {
            groups.push(phase, &chunk.id);
        }
    }
    groups
}

/// Report every write-scope collision and every repeated line inside a phase.
fn check_phase_scopes(chunks: &ChunkSet, groups: &PhaseGroups, report: &mut Report) {
    for (phase, ids) in &groups.groups {
        for (position, first) in ids.iter().enumerate() {
            for second in ids.iter().skip(position + 1) {
                check_pair(*phase, first, second, chunks, report);
            }
        }
        check_phase_lines(*phase, ids, chunks, report);
    }
}

/// Report every write-scope path two chunks of one phase share.
fn check_pair(phase: u32, first: &str, second: &str, chunks: &ChunkSet, report: &mut Report) {
    let (Some(left), Some(right)) = (chunks.get(first), chunks.get(second)) else {
        return;
    };
    let manifest_pair = (first.starts_with('M') && right.depends_on.iter().any(|id| id == first))
        || (second.starts_with('M') && left.depends_on.iter().any(|id| id == second));
    for here in &left.write_scope {
        for there in &right.write_scope {
            if here.ends_with("Cargo.lock") && there.ends_with("Cargo.lock") {
                continue;
            }
            if manifest_pair && (here.ends_with("Cargo.toml") || here.ends_with("src/lib.rs")) {
                continue;
            }
            if overlap(here, there) {
                report.finding(format!(
                    "phase {phase}: {first} and {second} both write {here} / {there}"
                ));
            }
        }
    }
}

/// Report every line that holds two chunks of one phase.
fn check_phase_lines(phase: u32, ids: &[String], chunks: &ChunkSet, report: &mut Report) {
    let mut seen: HashMap<&str, &str> = HashMap::new();
    for id in ids {
        let Some(chunk) = chunks.get(id) else {
            continue;
        };
        let line = chunk.line.as_str();
        if line == "M" || line == "trunk" {
            continue;
        }
        if let Some(first) = seen.get(line) {
            report.finding(format!(
                "phase {phase}: line {line} has two chunks, {first} and {id}"
            ));
        }
        seen.insert(line, chunk.id.as_str());
    }
}

/// True when one write-scope path is a prefix of the other.
fn overlap(here: &str, there: &str) -> bool {
    let left = here.trim_end_matches('/');
    let right = there.trim_end_matches('/');
    left == right
        || left.starts_with(&format!("{right}/"))
        || right.starts_with(&format!("{left}/"))
}

/// Report every section 13.4 link that is not a `depends_on` edge.
fn check_link_edges(links: &BTreeSet<(String, String)>, chunks: &ChunkSet, report: &mut Report) {
    for (source, target) in links {
        let Some(chunk) = chunks.get(target) else {
            continue;
        };
        if !chunk.depends_on.iter().any(|id| id == source) {
            report.finding(format!(
                "section 13.4 link {source} before {target} is not a depends_on edge of {target}"
            ));
        }
    }
}

/// The text `--write-manifest` writes to `plan-graph.md`.
fn manifest_text(chunks: &ChunkSet, phases: &PhaseTable, groups: &PhaseGroups) -> String {
    let mut out = vec![
        "# Plan graph: roadmap/duet-v1".to_owned(),
        String::new(),
        MANIFEST_NOTE.to_owned(),
        String::new(),
        "## Objective".to_owned(),
        String::new(),
        OBJECTIVE.to_owned(),
        String::new(),
        "## Phases".to_owned(),
        String::new(),
        "| Phase | Chunks | Width |".to_owned(),
        "|---|---|---|".to_owned(),
    ];
    for (phase, ids) in groups.sorted() {
        out.push(format!("| {phase} | {} | {} |", ids.join(", "), ids.len()));
    }
    out.extend([
        String::new(),
        "## Chunks".to_owned(),
        String::new(),
        "| Id | Line | Phase | File | Depends on | Write scope |".to_owned(),
        "|---|---|---|---|---|---|".to_owned(),
    ]);
    for chunk in phase_order(chunks, phases) {
        let phase = phases
            .get(&chunk.id)
            .map_or_else(|| "?".to_owned(), |number| number.to_string());
        let depends = chunk.depends_on.join(", ");
        let depends = if depends.is_empty() {
            "none"
        } else {
            depends.as_str()
        };
        out.push(format!(
            "| {} | {} | {phase} | `{}` | {depends} | {} |",
            chunk.id,
            chunk.line,
            chunk.file_name,
            chunk.write_scope.join("; ")
        ));
    }
    out.extend([
        String::new(),
        "## Parallelism".to_owned(),
        String::new(),
        format!(
            "{} phases; widest phase holds {} chunks. Every line is a serial chain (SM6); every cross-line link is a `depends_on` edge; `Cargo.lock` is the one shared write path and follows the regenerate rule of section 13.0.",
            groups.count(),
            groups.widest()
        ),
        String::new(),
    ]);
    out.join("\n")
}

/// Every chunk sorted by phase and then by id.
fn phase_order<'a>(chunks: &'a ChunkSet, phases: &PhaseTable) -> Vec<&'a Chunk> {
    let mut ordered: Vec<&Chunk> = chunks.iter().collect();
    ordered.sort_by(|left, right| {
        let here = phases.get(&left.id).unwrap_or(99);
        let there = phases.get(&right.id).unwrap_or(99);
        here.cmp(&there).then_with(|| left.id.cmp(&right.id))
    });
    ordered
}
