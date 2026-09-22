//! Compile the section 15 roster in a scratch workspace outside the repository
//! (`PG25`).
//!
//! The guard reads the architecture document; the compiler reads the lint
//! table. `PG1` to `PG24` hold the first half and this rule holds the second.
//! The guard writes a scratch Cargo workspace of one crate per section 1.5
//! crate row, gives each crate the section 1.3 edges as path dependencies,
//! copies the repository's own `[workspace.lints]` table, `.cargo/config.toml`,
//! `clippy.toml`, `rust-toolchain.toml`, and `Cargo.lock`, and runs the real
//! lint invocation over it.
//!
//! This module is the port of the prototype
//! `roadmap/duet-v1/tools/roster_compile.sh`, which carries two embedded Python
//! halves: the generator that writes the workspace and the comparer that holds
//! every recorded size against the measured one. The port keeps every rule and
//! every printed line the prototype produces. It keeps every exit code too,
//! except for the one input class this module states below. The guard exits 2
//! on a usage or input failure, 1 on a finding, and 0 when the roster compiles
//! clean and every recorded size is true.
//!
//! The port differs from the prototype in one input class: a clippy process
//! that exits non-zero and prints no lint diagnostic. The prototype reads every
//! non-zero clippy status as a finding of the plan, and line 1015 of
//! `roster_compile.sh` defines exit 1 for this class. This port reads the
//! clippy report instead. A lint diagnostic is the finding the prototype
//! states. A run that decided nothing is fail closed with a `FAIL:` line that
//! names the cause, and this guard exits 2. The exit code differs because an
//! environment fault is not a breach of a rule. A machine that cannot build the
//! dependency tree must not read as a finding of the plan.
//!
//! One scratch directory and one cargo target, and the target does not outlive
//! the run that made it. A target for this workspace holds the whole `gpui`
//! dependency tree, which is several gigabytes, and one run per revision filled
//! the volume. The owner of the target is the process that creates it. A caller
//! that exports `ROSTER_TARGET_DIR` owns the target and deletes it, so a probe
//! of many shapes pays for one build. This run owns the target when no caller
//! exports one, and it deletes the target on every exit path.
//!
//! The prototype exports `CARGO_TARGET_DIR` into the environment of the shell.
//! This port sets the same variable on each cargo child, because
//! `std::env::set_var` is disallowed and a child variable reaches the same
//! place.
//!
//! The two containment rules read different paths, and the prototype states
//! why. The scratch directory is held against the repository as the caller
//! writes both, so a caller that names the repository one way and the scratch
//! another way passes. The exported target is held against the repository on
//! the resolved path, because that value reaches a directory removal and a
//! symbolic link defeats a path exemption (`CG7`).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command as Process, Stdio};

use anyhow::Context as _;

use crate::Outcome;

/// The content kind one registered block carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// A markdown table.
    Table,
    /// A fenced `text` block.
    Text,
    /// A fenced `rust` block.
    Rust,
}

/// One registered data block of the architecture document.
#[derive(Debug, Clone, Copy)]
struct Block {
    /// The block id the marker line states.
    id: &'static str,
    /// The `####` heading the block carries.
    heading: &'static str,
    /// The content kind the block carries.
    kind: Kind,
    /// The row count the register states.
    minimum: usize,
}

/// Every block this guard reads, by the id the document's marker states.
///
/// The minimum row count here and the minimum the marker states must be one
/// number, exactly as the substitution list and `SUBSTITUTIONS` must be one
/// set.
const DATA_BLOCKS: [Block; 9] = [
    Block {
        id: "ownership-table",
        heading: "The type ownership table",
        kind: Kind::Table,
        minimum: 16,
    },
    Block {
        id: "edge-list",
        heading: "The internal edge list",
        kind: Kind::Text,
        minimum: 18,
    },
    Block {
        id: "external-paths",
        heading: "Where every external name comes from",
        kind: Kind::Text,
        minimum: 70,
    },
    Block {
        id: "pins",
        heading: "The external crate pins the roster compile uses",
        kind: Kind::Text,
        minimum: 13,
    },
    Block {
        id: "constants",
        heading: "Every workspace constant",
        kind: Kind::Rust,
        minimum: 122,
    },
    Block {
        id: "substitutions",
        heading: "Every substitution the roster compile applies",
        kind: Kind::Text,
        minimum: 8,
    },
    Block {
        id: "recorded-sizes",
        heading: "Every size the guard records",
        kind: Kind::Text,
        minimum: 51,
    },
    Block {
        id: "drop-impls",
        heading: "Every declaration with a hand-written Drop impl",
        kind: Kind::Text,
        minimum: 3,
    },
    Block {
        id: "impl-sites",
        heading: "Every impl block the roster compiles",
        kind: Kind::Text,
        minimum: 65,
    },
];

/// Every substitution this guard applies, by the id section 1.9 gives it.
const SUBSTITUTIONS: [&str; 8] = ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8"];

/// The pins only the application row needs.
///
/// Every other row would carry them as an unused dependency, and `crates/duet`
/// is the one row that may name a framework type (section 1.3).
const APP_ONLY_PINS: [&str; 3] = ["gpui-kit", "tokio", "tokio-util"];

/// Every derive the roster drops, because the roster states no body for it.
const DROPPED_DERIVES: [&str; 2] = ["Error", "IntoElement"];

/// Every lint the roster-compile profile relaxes.
const RELAXED_LINTS: [&str; 7] = [
    "missing_docs",
    "missing_docs_in_private_items",
    "must_use_candidate",
    "missing_const_for_fn",
    "new_without_default",
    "missing_panics_doc",
    "missing_errors_doc",
];

/// The crate row the section 1.5 table writes for the application.
const APP_ROW: &str = "crates/duet";

/// The name the application package carries in the scratch workspace.
const APP_PACKAGE: &str = "duet";

/// The member that measures every size this plan states.
const SIZE_MEMBER: &str = "roster-sizes";

/// Run the roster compile over the architecture document.
///
/// # Errors
/// Returns an error when a write to the output stream fails.
pub(crate) fn run(document: &Path, scratch: &Path, repo: &Path) -> anyhow::Result<Outcome> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    if !document.is_file() {
        writeln!(
            out,
            "FAIL: cannot open {}; the guard is fail-closed.",
            document.display()
        )?;
        return Ok(Outcome::FailClosed);
    }
    if !repo.join("Cargo.toml").is_file() {
        writeln!(
            out,
            "FAIL: {} holds no Cargo.toml; the guard is fail-closed.",
            repo.display()
        )?;
        return Ok(Outcome::FailClosed);
    }
    if under(&scratch.to_string_lossy(), &repo.to_string_lossy()) {
        writeln!(
            out,
            "FAIL: the scratch workspace must sit outside the repository."
        )?;
        return Ok(Outcome::FailClosed);
    }
    if fs::create_dir_all(scratch).is_err() {
        return Ok(Outcome::FailClosed);
    }
    let plan = match target_plan(&mut out, repo, scratch)? {
        Choice::Use(plan) => plan,
        Choice::Refuse => return Ok(Outcome::FailClosed),
    };
    let places = Places {
        document: document.to_path_buf(),
        scratch: scratch.to_path_buf(),
        repo: repo.to_path_buf(),
        target: plan.directory.clone(),
    };
    let produced = compile(&mut out, &places);
    let removed = if plan.owned {
        remove_target(&plan.directory)
    } else {
        Ok(())
    };
    produced.and_then(|outcome| removed.map(|()| outcome))
}

/// Where one run reads from and writes to.
#[derive(Debug)]
struct Places {
    /// The architecture document the guard reads.
    document: PathBuf,
    /// The scratch directory the guard writes.
    scratch: PathBuf,
    /// The repository root the guard copies the policy files from.
    repo: PathBuf,
    /// The cargo target every child of this run shares.
    target: PathBuf,
}

impl Places {
    /// The scratch Cargo workspace this run builds.
    fn workspace(&self) -> PathBuf {
        self.scratch.join("workspace")
    }
}

/// The cargo target one run uses, and who owns it.
#[derive(Debug)]
struct TargetPlan {
    /// The directory cargo writes its artifacts to.
    directory: PathBuf,
    /// Whether this run deletes the target when it ends.
    owned: bool,
}

/// The target this run uses, or the refusal the guard prints.
#[derive(Debug)]
enum Choice {
    /// The run proceeds with this target.
    Use(TargetPlan),
    /// The exported target is refused and the guard is fail-closed.
    Refuse,
}

/// Whether one path string equals another, or sits under it.
fn under(path: &str, root: &str) -> bool {
    path == root || path.starts_with(&format!("{root}/"))
}

/// The target this run uses, after it validates an exported one.
///
/// An exported path is validated before it is used, because nothing else does
/// and the value reaches a directory removal. The compare is on the resolved
/// path, because a symbolic link defeats a path exemption.
///
/// # Errors
/// Returns an error when a write to the output stream fails.
fn target_plan(out: &mut impl io::Write, repo: &Path, scratch: &Path) -> anyhow::Result<Choice> {
    let exported = std::env::var_os("ROSTER_TARGET_DIR").unwrap_or_default();
    if exported.is_empty() {
        return Ok(Choice::Use(TargetPlan {
            directory: scratch.join("target"),
            owned: true,
        }));
    }
    let named = PathBuf::from(&exported);
    if !named.is_absolute() {
        writeln!(
            out,
            "FAIL: ROSTER_TARGET_DIR must be an absolute path; the guard is fail-closed."
        )?;
        return Ok(Choice::Refuse);
    }
    let real = resolved(&named);
    let repo_real = repo.canonicalize().unwrap_or_default();
    if under(&real.to_string_lossy(), &repo_real.to_string_lossy()) {
        writeln!(
            out,
            "FAIL: ROSTER_TARGET_DIR must sit outside {}; the guard is fail-closed.",
            repo.display()
        )?;
        return Ok(Choice::Refuse);
    }
    Ok(Choice::Use(TargetPlan {
        directory: named,
        owned: false,
    }))
}

/// One path with every symbolic link of its existing head resolved.
fn resolved(path: &Path) -> PathBuf {
    if path.is_dir() {
        return path
            .canonicalize()
            .unwrap_or_else(|_error| path.to_path_buf());
    }
    let parent = path.parent().unwrap_or_else(|| Path::new(""));
    let base = path.file_name().unwrap_or_default();
    let head = parent.canonicalize().unwrap_or_default();
    PathBuf::from(format!(
        "{}/{}",
        head.to_string_lossy(),
        base.to_string_lossy()
    ))
}

/// Remove the cargo target this run owns.
///
/// The prototype removes the target with `rm -rf` from a shell trap, which
/// reports its own failure on the error stream and leaves the exit code alone.
/// This function holds that behaviour, so the compared output stream carries
/// the same lines.
///
/// # Errors
/// Returns an error when a write to the error stream fails.
fn remove_target(target: &Path) -> anyhow::Result<()> {
    let Err(error) = fs::remove_dir_all(target) else {
        return Ok(());
    };
    if error.kind() == io::ErrorKind::NotFound {
        return Ok(());
    }
    let stderr = io::stderr();
    let mut err = stderr.lock();
    writeln!(err, "rm: {}: {error}", target.display())?;
    Ok(())
}

/// Generate the scratch workspace, resolve it, lint it, and measure it.
///
/// # Errors
/// Returns an error when a write to the output stream fails, or when a file
/// the guard needs cannot be read.
fn compile(out: &mut impl io::Write, places: &Places) -> anyhow::Result<Outcome> {
    let generated = generate(out, places)?;
    if generated != Outcome::Clean {
        return Ok(generated);
    }
    let workspace = places.workspace();
    let before = package_count(&places.repo.join("Cargo.lock"))?;
    if let Resolution::Refused(reason) = generate_lockfile(places, &workspace) {
        writeln!(
            out,
            "FAIL: the scratch workspace does not resolve; the guard is fail-closed. CARGO: {reason}"
        )?;
        return Ok(Outcome::FailClosed);
    }
    let after = package_count(&workspace.join("Cargo.lock"))?;
    writeln!(
        out,
        "ROSTER LOCK:     {before} packages in the repository lock, {after} after resolution"
    )?;
    writeln!(
        out,
        "ROSTER LINT:     cargo clippy --workspace --all-targets -- -D warnings"
    )?;
    out.flush()?;
    match lint(out, places, &workspace)? {
        LintRun::Clean => {},
        LintRun::Diagnostics => {
            writeln!(out, "ROSTER CLIPPY:   FAIL")?;
            return Ok(Outcome::Findings);
        },
        LintRun::Unavailable(reason) => {
            writeln!(out, "FAIL: {reason}; the guard is fail-closed.")?;
            return Ok(Outcome::FailClosed);
        },
    }
    writeln!(out, "ROSTER CLIPPY:   clean")?;
    out.flush()?;
    let Some(measured) = measure(places, &workspace) else {
        writeln!(
            out,
            "FAIL: the size oracle did not run; the guard is fail-closed."
        )?;
        return Ok(Outcome::FailClosed);
    };
    compare_sizes(out, &places.document, &measured)
}

/// The count of `[[package]]` lines one lock file carries.
///
/// # Errors
/// Returns an error when the lock file cannot be read.
fn package_count(lock: &Path) -> anyhow::Result<usize> {
    let text =
        fs::read_to_string(lock).with_context(|| format!("cannot read {}", lock.display()))?;
    Ok(text
        .lines()
        .filter(|line| line.starts_with("[[package]]"))
        .count())
}

/// One cargo invocation inside the scratch workspace.
fn cargo(places: &Places, workspace: &Path) -> Process {
    let binary = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let mut command = Process::new(binary);
    command
        .current_dir(workspace)
        .env("CARGO_TARGET_DIR", &places.target);
    command
}

/// How many characters of a cargo reason one printed line carries.
const REASON_WIDTH: usize = 400;

/// What the two resolution attempts produced.
#[derive(Debug)]
enum Resolution {
    /// One attempt resolved the scratch workspace.
    Resolved,
    /// Neither attempt resolved it, and cargo gave this reason.
    Refused(String),
}

/// Resolve the scratch workspace, offline first and online second.
///
/// Both attempts keep the stderr of cargo, so the caller states the cause
/// beside the verdict. An operator who reads the verdict alone cannot tell a
/// broken roster from a machine that reaches no registry.
fn generate_lockfile(places: &Places, workspace: &Path) -> Resolution {
    let mut reasons: Vec<String> = Vec::new();
    for offline in [true, false] {
        let mut command = cargo(places, workspace);
        command.arg("generate-lockfile");
        if offline {
            command.arg("--offline");
        }
        match command
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
        {
            Ok(done) if done.status.success() => return Resolution::Resolved,
            Ok(done) => reasons.push(cargo_reason(&String::from_utf8_lossy(&done.stderr))),
            Err(error) => reasons.push(format!("cargo generate-lockfile did not start: {error}")),
        }
    }
    Resolution::Refused(reasons.join(" / "))
}

/// What one clippy run over the scratch workspace produced.
#[derive(Debug)]
enum LintRun {
    /// Clippy ran over every crate and every crate passed.
    Clean,
    /// Clippy ran and printed at least one lint diagnostic.
    Diagnostics,
    /// Clippy decided nothing: it did not start, or it stopped before a lint
    /// diagnostic reached the report.
    Unavailable(String),
}

/// Run the real lint invocation over the scratch workspace.
///
/// A roster that breaks a lint and a machine that cannot build the dependency
/// tree are two different verdicts. The first is a finding of the plan; the
/// second is fail closed, because the guard decided nothing about the plan.
/// The two are told apart by the report: a lint diagnostic carries a file
/// position and a bare level, a compiler error carries a code such as
/// `error[E0308]`, and a resolution failure carries no file position at all.
///
/// # Errors
/// Returns an error when a write of the cargo report fails.
fn lint(out: &mut impl io::Write, places: &Places, workspace: &Path) -> anyhow::Result<LintRun> {
    let started = cargo(places, workspace)
        .args([
            "clippy",
            "--workspace",
            "--all-targets",
            "--message-format",
            "short",
            "--",
            "-D",
            "warnings",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output();
    let done = match started {
        Ok(done) => done,
        Err(error) => {
            return Ok(LintRun::Unavailable(format!(
                "cargo clippy did not start: {error}"
            )));
        },
    };
    let report = String::from_utf8_lossy(&done.stdout).into_owned();
    let diagnostics = String::from_utf8_lossy(&done.stderr).into_owned();
    out.write_all(report.as_bytes())?;
    out.flush()?;
    let stderr = io::stderr();
    let mut sink = stderr.lock();
    sink.write_all(diagnostics.as_bytes())?;
    sink.flush()?;
    if done.status.success() {
        return Ok(LintRun::Clean);
    }
    if lint_diagnostic(&diagnostics) || lint_diagnostic(&report) {
        return Ok(LintRun::Diagnostics);
    }
    Ok(LintRun::Unavailable(format!(
        "cargo clippy stopped with no lint diagnostic: {}",
        cargo_reason(&diagnostics)
    )))
}

/// True when one cargo report holds a lint diagnostic.
fn lint_diagnostic(report: &str) -> bool {
    report.lines().any(is_lint_line)
}

/// True when one report line states a lint at a file position.
///
/// `src/lib.rs:2:1: error: missing documentation for a function` is a lint.
/// `src/lib.rs:2:23: error[E0308]: mismatched types` is a compiler error, and
/// `error: could not compile` is the summary cargo prints for either one.
fn is_lint_line(line: &str) -> bool {
    ["error: ", "warning: "]
        .into_iter()
        .filter_map(|level| line.split_once(level))
        .any(|(head, _)| is_position(head))
}

/// True when the head of one report line ends with `path:line:column: `.
fn is_position(head: &str) -> bool {
    let Some(rest) = head.strip_suffix(": ") else {
        return false;
    };
    let mut fields = rest.rsplitn(3, ':');
    let (Some(column), Some(row), Some(file)) = (fields.next(), fields.next(), fields.next())
    else {
        return false;
    };
    !file.is_empty()
        && !row.is_empty()
        && !column.is_empty()
        && row.bytes().all(|mark| mark.is_ascii_digit())
        && column.bytes().all(|mark| mark.is_ascii_digit())
}

/// One line of cargo output that states why a cargo run stopped.
///
/// The first line that names an error carries the cause; the lines after it
/// repeat the cause as a build summary. The text is bounded, because a cargo
/// report is not, and it holds one line, because the caller prints it beside
/// the verdict.
fn cargo_reason(report: &str) -> String {
    let first = report
        .lines()
        .map(str::trim)
        .find(|line| line.contains("error"))
        .or_else(|| report.lines().map(str::trim).find(|line| !line.is_empty()))
        .unwrap_or("cargo printed no reason");
    let bounded: String = first.chars().take(REASON_WIDTH).collect();
    if bounded.chars().count() < first.chars().count() {
        format!("{bounded} ...")
    } else {
        bounded
    }
}

/// The output of the size oracle, or `None` when the oracle did not run.
fn measure(places: &Places, workspace: &Path) -> Option<String> {
    let done = cargo(places, workspace)
        .args(["run", "--quiet", "-p", SIZE_MEMBER])
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .output()
        .ok()?;
    if !done.status.success() {
        return None;
    }
    String::from_utf8(done.stdout).ok()
}

/// The slice of one text between two byte offsets, or the empty text.
fn cut(text: &str, from: usize, to: usize) -> &str {
    text.get(from..to).unwrap_or_default()
}

/// The slice of one text from one byte offset to its end.
fn from(text: &str, at: usize) -> &str {
    text.get(at..).unwrap_or_default()
}

/// The byte at one offset of a text, or `None` past its end.
fn byte(text: &str, at: usize) -> Option<u8> {
    text.as_bytes().get(at).copied()
}

/// Whether one byte belongs to a word.
const fn word_byte(mark: u8) -> bool {
    mark.is_ascii_alphanumeric() || mark == b'_'
}

/// Whether one offset of a text opens or closes a word.
fn boundary(text: &str, at: usize) -> bool {
    let before = at
        .checked_sub(1)
        .and_then(|back| byte(text, back))
        .is_some_and(word_byte);
    let here = byte(text, at).is_some_and(word_byte);
    before != here
}

/// The first offset at or after `at` whose byte fails the test.
fn skip<F: Fn(u8) -> bool>(text: &str, at: usize, holds: F) -> usize {
    let mut pos = at;
    while byte(text, pos).is_some_and(&holds) {
        pos += 1;
    }
    pos
}

/// The offset of the brace that closes the one at `open_at`.
///
/// The offset of the text end answers when no brace closes it, exactly as the
/// prototype answers.
fn matching_brace(code: &str, open_at: usize) -> usize {
    let mut depth = 0_isize;
    let mut pos = open_at;
    while let Some(mark) = byte(code, pos) {
        if mark == b'{' {
            depth += 1;
        } else if mark == b'}' {
            depth -= 1;
            if depth == 0 {
                return pos;
            }
        }
        pos += 1;
    }
    code.len()
}

/// Every piece of one text that one separator divides at bracket depth zero.
fn split_top(text: &str, separator: u8) -> Vec<&str> {
    let mut depth = 0_isize;
    let mut start = 0;
    let mut pieces = Vec::new();
    let mut pos = 0;
    while let Some(mark) = byte(text, pos) {
        if matches!(mark, b'(' | b'{' | b'[' | b'<') {
            depth += 1;
        } else if matches!(mark, b')' | b'}' | b']' | b'>') {
            depth -= 1;
        } else if mark == separator && depth == 0 {
            pieces.push(cut(text, start, pos));
            start = pos + 1;
        }
        pos += 1;
    }
    pieces.push(from(text, start));
    pieces
        .into_iter()
        .filter(|piece| !piece.trim().is_empty())
        .collect()
}

/// One list of names, as the prototype's own language prints it.
fn printed_list(names: &[String]) -> String {
    let body: Vec<String> = names.iter().map(|name| format!("'{name}'")).collect();
    format!("[{}]", body.join(", "))
}

/// The rows of one registered block.
#[derive(Debug)]
enum Rows {
    /// The cells of every row of a markdown table.
    Table(Vec<Vec<String>>),
    /// Every non-empty line of a fenced block.
    Text(Vec<String>),
}

/// Every registered block of one architecture document.
#[derive(Debug, Default)]
struct Blocks {
    /// The section 1.5 type ownership table.
    ownership: Vec<Vec<String>>,
    /// The section 1.3 internal edge list.
    edges: Vec<String>,
    /// Where every external name comes from.
    paths: Vec<String>,
    /// The external crate pins the roster compile uses.
    pins: Vec<String>,
    /// Every workspace constant.
    constants: Vec<String>,
    /// Every substitution the roster compile applies.
    substitutions: Vec<String>,
    /// Every size the guard records.
    sizes: Vec<String>,
    /// Every declaration with a hand-written `Drop` impl.
    drops: Vec<String>,
    /// Every impl block the roster compiles.
    impl_sites: Vec<String>,
}

impl Blocks {
    /// Store the rows of one registered block.
    fn take(&mut self, id: &str, rows: Rows) {
        match (id, rows) {
            ("ownership-table", Rows::Table(cells)) => self.ownership = cells,
            ("edge-list", Rows::Text(lines)) => self.edges = lines,
            ("external-paths", Rows::Text(lines)) => self.paths = lines,
            ("pins", Rows::Text(lines)) => self.pins = lines,
            ("constants", Rows::Text(lines)) => self.constants = lines,
            ("substitutions", Rows::Text(lines)) => self.substitutions = lines,
            ("recorded-sizes", Rows::Text(lines)) => self.sizes = lines,
            ("drop-impls", Rows::Text(lines)) => self.drops = lines,
            ("impl-sites", Rows::Text(lines)) => self.impl_sites = lines,
            (_, _) => {},
        }
    }
}

/// Every registered block, or one reason per block the guard cannot read.
fn read_blocks(source: &str) -> Result<Blocks, Vec<(&'static str, String)>> {
    let mut blocks = Blocks::default();
    let mut failures = Vec::new();
    for block in &DATA_BLOCKS {
        match read_block(source, block) {
            Ok(rows) => blocks.take(block.id, rows),
            Err(reason) => failures.push((block.id, reason)),
        }
    }
    if failures.is_empty() {
        Ok(blocks)
    } else {
        Err(failures)
    }
}

/// The text that follows the marker line of one registered block.
fn block_tail<'a>(source: &'a str, block: &Block) -> Result<&'a str, String> {
    let heads: Vec<usize> = source
        .match_indices('\n')
        .map(|(at, _mark)| at + 1)
        .chain(std::iter::once(0))
        .filter(|at| is_heading_line(from(source, *at), block.heading))
        .collect();
    let [at] = heads.as_slice() else {
        return Err(format!(
            "the heading `#### {}` appears {} times",
            block.heading,
            heads.len()
        ));
    };
    let body = from(source, *at);
    let start = body.find('\n').map_or(body.len(), |end| end + 1);
    let region = region_of(from(body, start));
    let marks: Vec<(usize, &str, usize)> = line_starts(region)
        .filter_map(|open| marker_of(line_at(region, open)).map(|(id, rows)| (open, id, rows)))
        .collect();
    let [(mark_at, id, minimum)] = marks.as_slice() else {
        return Err(format!(
            "the marker line appears {} times under the heading",
            marks.len()
        ));
    };
    if *id != block.id {
        return Err(format!(
            "the marker states id `{id}` and the register states `{}`",
            block.id
        ));
    }
    if *minimum != block.minimum {
        return Err(format!(
            "the marker states rows>={minimum} and the register states {}",
            block.minimum
        ));
    }
    let line = line_at(region, *mark_at);
    let after = mark_at + line.len();
    if byte(region, after) != Some(b'\n') {
        return Err("the marker does not end its own line".to_owned());
    }
    Ok(from(region, after + 1))
}

/// One registered block's rows, or the reason the block cannot be read.
fn read_block(source: &str, block: &Block) -> Result<Rows, String> {
    let tail = block_tail(source, block)?;
    let rows = match block.kind {
        Kind::Table => Rows::Table(table_rows(tail)?),
        Kind::Text | Kind::Rust => Rows::Text(fenced_rows(tail, block.kind)?),
    };
    let count = match &rows {
        Rows::Table(cells) => cells.len(),
        Rows::Text(lines) => lines.len(),
    };
    if count < block.minimum {
        return Err(format!(
            "it holds {count} rows and the stated minimum is {}",
            block.minimum
        ));
    }
    Ok(rows)
}

/// The region one heading opens, up to the next heading of level one to four.
fn region_of(body: &str) -> &str {
    for at in line_starts(body) {
        if is_any_heading(line_at(body, at)) {
            return cut(body, 0, at);
        }
    }
    body
}

/// Every byte offset at which one text opens a line.
fn line_starts(text: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0).chain(text.match_indices('\n').map(|(at, _mark)| at + 1))
}

/// The line one offset opens, without its terminator.
fn line_at(text: &str, at: usize) -> &str {
    let body = from(text, at);
    body.find('\n').map_or(body, |end| cut(body, 0, end))
}

/// Whether one text opens with the `####` heading of one registered block.
fn is_heading_line(body: &str, heading: &str) -> bool {
    let line = body.find('\n').map_or(body, |end| cut(body, 0, end));
    line.strip_prefix("#### ")
        .and_then(|rest| rest.strip_prefix(heading))
        .is_some_and(|tail| tail.chars().all(char::is_whitespace))
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

/// Every row of the markdown table that opens one text.
fn table_rows(tail: &str) -> Result<Vec<Vec<String>>, String> {
    let head = line_at(tail, 0);
    let after_head = head.len() + 1;
    let rule = line_at(tail, after_head);
    if !is_table_head(head) || !is_table_rule(rule) {
        return Err("no table header follows the marker".to_owned());
    }
    let start = after_head + rule.len() + 1;
    let mut rows = Vec::new();
    for at in line_starts(from(tail, start)) {
        let line = line_at(from(tail, start), at);
        if !line.starts_with('|') {
            break;
        }
        rows.push(split_row(line));
    }
    Ok(rows)
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

/// The cells of one markdown table row, as the prototype splits them.
fn split_row(line: &str) -> Vec<String> {
    let parts: Vec<&str> = line.split('|').collect();
    let count = parts.len();
    if count < 2 {
        return Vec::new();
    }
    parts
        .into_iter()
        .skip(1)
        .take(count - 2)
        .map(|cell| cell.trim().to_owned())
        .collect()
}

/// Every non-empty line of the fenced block that opens one text.
fn fenced_rows(tail: &str, kind: Kind) -> Result<Vec<String>, String> {
    let Some(rest) = tail.strip_prefix("```") else {
        return Err("no fenced block follows the marker".to_owned());
    };
    let language: String = rest.chars().take_while(char::is_ascii_lowercase).collect();
    let opened = 3 + language.len();
    if byte(tail, opened) != Some(b'\n') {
        return Err("no fenced block follows the marker".to_owned());
    }
    let body = from(tail, opened + 1);
    let Some(close) = line_starts(body).find(|at| line_at(body, *at).starts_with("```")) else {
        return Err("no fenced block follows the marker".to_owned());
    };
    let wanted = if kind == Kind::Rust { "rust" } else { "text" };
    if language != wanted {
        return Err(format!(
            "the fence is `{language}` and `{wanted}` is required"
        ));
    }
    Ok(cut(body, 0, close)
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(str::to_owned)
        .collect())
}

/// Every row of one fenced block, as a first token and the tokens after it.
fn fenced_map(lines: &[String]) -> BTreeMap<String, Vec<String>> {
    let mut found = BTreeMap::new();
    for line in lines {
        let parts: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
        let Some(head) = parts.first() else {
            continue;
        };
        if parts.len() >= 2 {
            found.insert(head.clone(), parts.into_iter().skip(1).collect());
        }
    }
    found
}

/// Everything the registered blocks state about the roster.
#[derive(Debug)]
struct Roster {
    /// The crate that declares each type.
    owner: BTreeMap<String, String>,
    /// Every crate row, in the order the section 1.5 table states.
    order: Vec<String>,
    /// The crates each crate depends on.
    edges: BTreeMap<String, Vec<String>>,
    /// Where each external name comes from.
    paths: BTreeMap<String, Vec<String>>,
    /// The version and features of each external crate.
    pins: BTreeMap<String, Vec<String>>,
    /// The crate and the declaration of each workspace constant.
    constants: BTreeMap<String, (String, String)>,
    /// Every declaration with a hand-written `Drop` impl.
    drops: Vec<String>,
    /// The crate that holds one impl, when the block states one.
    impl_crate: BTreeMap<(String, String), String>,
    /// The size and the alignment each row of the size block records.
    sizes: BTreeMap<String, Vec<String>>,
}

/// The section 1.3 list writes `duet`; the section 1.5 table writes
/// `crates/duet`.
fn normalize(name: &str) -> String {
    if name == APP_PACKAGE {
        APP_ROW.to_owned()
    } else {
        name.to_owned()
    }
}

/// The package name one crate row names.
fn package_name(row: &str) -> &str {
    if row == APP_ROW { APP_PACKAGE } else { row }
}

/// The Rust identifier one crate row names.
fn crate_ident(row: &str) -> String {
    package_name(row).replace('-', "_")
}

impl Roster {
    /// Every fact the registered blocks state.
    fn read(blocks: &Blocks) -> Self {
        let (owner, order) = ownership(&blocks.ownership);
        Self {
            owner,
            order,
            edges: edge_list(&blocks.edges),
            paths: fenced_map(&blocks.paths),
            pins: fenced_map(&blocks.pins),
            constants: constants(&blocks.constants),
            drops: blocks
                .drops
                .iter()
                .flat_map(|line| line.split_whitespace().map(str::to_owned))
                .collect(),
            impl_crate: impl_crate(&blocks.impl_sites),
            sizes: fenced_map(&blocks.sizes),
        }
    }

    /// The first `Drop` name the section 1.5 table places nowhere.
    fn unplaced_drop(&self) -> Option<&String> {
        self.drops
            .iter()
            .find(|name| !self.owner.contains_key(*name))
    }
}

/// The crate that declares each type, and every crate row in table order.
fn ownership(rows: &[Vec<String>]) -> (BTreeMap<String, String>, Vec<String>) {
    let mut owner = BTreeMap::new();
    let mut order: Vec<String> = Vec::new();
    for cells in rows {
        if cells.len() < 3 {
            continue;
        }
        let (Some(first), Some(second)) = (cells.first(), cells.get(1)) else {
            continue;
        };
        let row = first.trim_matches('`').to_owned();
        if !order.contains(&row) {
            order.push(row.clone());
        }
        for name in quoted_names(second) {
            owner.insert(name, row.clone());
        }
    }
    (owner, order)
}

/// Every name one cell states between backticks.
fn quoted_names(cell: &str) -> Vec<String> {
    let mut found = Vec::new();
    for (index, piece) in cell.split('`').enumerate() {
        let quoted = index % 2 == 1;
        let named = !piece.is_empty()
            && piece
                .chars()
                .all(|mark| mark.is_ascii_alphanumeric() || mark == '_');
        if quoted && named {
            found.push(piece.to_owned());
        }
    }
    found
}

/// The crates each crate depends on.
fn edge_list(lines: &[String]) -> BTreeMap<String, Vec<String>> {
    let joined = join_continuations(&lines.join("\n"));
    let mut edges = BTreeMap::new();
    for line in joined.lines() {
        let Some((left, right)) = line.split_once("->") else {
            continue;
        };
        let targets = right
            .split(',')
            .filter(|piece| !piece.trim().is_empty())
            .map(|piece| normalize(piece.trim()))
            .collect();
        edges.insert(normalize(left.trim()), targets);
    }
    edges
}

/// Every continuation line folded onto the line that opens it.
fn join_continuations(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pos = 0;
    while let Some(mark) = byte(text, pos) {
        if mark != b',' {
            out.push(char::from(mark));
            pos += 1;
            continue;
        }
        let end = skip(text, pos + 1, |next| next.is_ascii_whitespace());
        let run = cut(text, pos + 1, end);
        let folded = run
            .rfind('\n')
            .is_some_and(|at| at + 1 < run.len() || run.len() > 1);
        if folded && run.contains('\n') && run.len() > 1 {
            out.push_str(", ");
            pos = end;
            continue;
        }
        out.push(',');
        pos += 1;
    }
    out
}

/// The crate and the declaration of each workspace constant.
fn constants(lines: &[String]) -> BTreeMap<String, (String, String)> {
    let body = lines.join("\n");
    let mut found = BTreeMap::new();
    let mut pos = 0;
    while let Some(at) = from(&body, pos).find("pub const ") {
        let start = pos + at;
        pos = start + "pub const ".len();
        let Some(declaration) = read_constant(&body, start) else {
            continue;
        };
        let crate_row = last_crate_comment(cut(&body, 0, start));
        found.insert(declaration.name.clone(), (crate_row, declaration.text));
    }
    found
}

/// One workspace constant the roster spells out.
#[derive(Debug)]
struct Constant {
    /// The name the declaration states.
    name: String,
    /// The declaration the roster emits.
    text: String,
}

/// One `pub const` declaration whose value is a literal expression.
fn read_constant(body: &str, at: usize) -> Option<Constant> {
    let mut pos = at + "pub const ".len();
    if !byte(body, pos).is_some_and(|mark| mark.is_ascii_uppercase()) {
        return None;
    }
    let name_end = skip(body, pos, |mark| {
        mark.is_ascii_uppercase() || mark.is_ascii_digit() || mark == b'_'
    });
    let name = cut(body, pos, name_end).to_owned();
    pos = skip(body, name_end, |mark| mark.is_ascii_whitespace());
    if byte(body, pos) != Some(b':') {
        return None;
    }
    pos = skip(body, pos + 1, |mark| mark.is_ascii_whitespace());
    let type_end = skip(body, pos, |mark| {
        mark.is_ascii_alphanumeric() || matches!(mark, b'_' | b':' | b'<' | b'>' | b' ')
    });
    let type_name = cut(body, pos, type_end).trim_end().to_owned();
    if type_name.is_empty() {
        return None;
    }
    pos = skip(body, type_end, |mark| mark.is_ascii_whitespace());
    if byte(body, pos) != Some(b'=') {
        return None;
    }
    pos = skip(body, pos + 1, |mark| mark.is_ascii_whitespace());
    let end = from(body, pos).find(';')?;
    let value = cut(body, pos, pos + end).trim().to_owned();
    if value.is_empty() || !is_literal_value(&value) {
        return None;
    }
    Some(Constant {
        text: format!("pub const {name}: {type_name} = {value};"),
        name,
    })
}

/// Whether one value is a literal, or arithmetic over literals and constants.
fn is_literal_value(value: &str) -> bool {
    let head = value.as_bytes().first().copied().unwrap_or(b' ');
    let opens = head.is_ascii_digit() || head == b'_' || head.is_ascii_uppercase();
    opens
        && value.bytes().all(|mark| {
            mark.is_ascii_digit()
                || mark.is_ascii_uppercase()
                || matches!(mark, b'_' | b'*' | b' ' | b'+' | b'-')
        })
}

/// The last crate a `// duet-<name>` comment of one text states.
fn last_crate_comment(text: &str) -> String {
    let mut found = String::new();
    for line in text.lines() {
        let Some(rest) = line.strip_prefix("// duet-") else {
            continue;
        };
        let end = skip(rest, 0, |mark| mark.is_ascii_lowercase());
        if end > 0 {
            found = format!("duet-{}", cut(rest, 0, end));
        }
    }
    found
}

/// The crate that holds one impl, when the `impl-sites` block states one.
fn impl_crate(rows: &[String]) -> BTreeMap<(String, String), String> {
    let mut found = BTreeMap::new();
    for row in rows {
        let tokens: Vec<&str> = row.split_whitespace().collect();
        let count = tokens.len();
        if count < 4 {
            continue;
        }
        let (Some(target), Some(path), Some(word), Some(home)) = (
            tokens.first(),
            tokens.get(1),
            count.checked_sub(2).and_then(|at| tokens.get(at)),
            tokens.last(),
        ) else {
            continue;
        };
        if *word != "in" {
            continue;
        }
        let trait_name = path.rsplit("::").next().unwrap_or(path);
        found.insert(
            ((*target).to_owned(), trait_name.to_owned()),
            (*home).to_owned(),
        );
    }
    found
}

/// The three declaration keywords the roster parser reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Keyword {
    /// A `struct` declaration.
    Struct,
    /// An `enum` declaration.
    Enum,
    /// A `trait` declaration.
    Trait,
}

impl Keyword {
    /// The word this keyword spells.
    const fn word(self) -> &'static str {
        match self {
            Self::Struct => "struct",
            Self::Enum => "enum",
            Self::Trait => "trait",
        }
    }
}

/// The three body shapes a declaration carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// A tuple body in round brackets.
    Tuple,
    /// A braced body.
    Braced,
    /// No body at all.
    Unit,
}

/// One declaration the roster spells out.
#[derive(Debug, Clone)]
struct Item {
    /// The name the declaration states.
    name: String,
    /// Every attribute line above the declaration.
    attrs: String,
    /// The keyword the declaration opens with.
    kind: Keyword,
    /// The body shape the declaration carries.
    shape: Shape,
    /// The body the declaration carries.
    body: String,
    /// The generic list and, for a trait, the bound list.
    generics: String,
}

/// The head of one declaration, up to its body.
#[derive(Debug)]
struct Head {
    /// Every attribute line above the declaration.
    attrs: String,
    /// The keyword the declaration opens with.
    kind: Keyword,
    /// The name the declaration states.
    name: String,
    /// The generic list and, for a trait, the bound list.
    generics: String,
    /// The offset the head ends at.
    end: usize,
}

/// Every declaration one code block spells out.
fn items(code: &str) -> Vec<Item> {
    let mut found = Vec::new();
    let mut at = 0;
    while at <= code.len() {
        let Some(head) = match_head(code, at) else {
            let Some(mark) = from(code, at).chars().next() else {
                break;
            };
            at += mark.len_utf8();
            continue;
        };
        let end = head.end;
        found.push(item_of(code, head));
        at = if end > at { end } else { at + 1 };
    }
    found
}

/// One declaration, with the body its head opens.
fn item_of(code: &str, head: Head) -> Item {
    let tail = from(code, head.end);
    let brace = tail.find('{');
    let semi = tail.find(';');
    let paren = tail.find('(');
    let limit = brace.unwrap_or(tail.len());
    let tuple = head.kind != Keyword::Trait
        && paren.is_some_and(|open| open < limit && semi.is_none_or(|stop| open < stop));
    let (shape, body) = if let (true, Some(open)) = (tuple, paren) {
        let close = tail
            .find(')')
            .unwrap_or_else(|| tail.len().saturating_sub(1));
        (Shape::Tuple, cut(tail, open + 1, close).to_owned())
    } else if let Some(open) = brace.filter(|at| semi.is_none_or(|stop| *at < stop)) {
        let absolute = head.end + open;
        let close = matching_brace(code, absolute);
        (Shape::Braced, cut(code, absolute + 1, close).to_owned())
    } else {
        (Shape::Unit, String::new())
    };
    Item {
        name: head.name,
        attrs: head.attrs,
        kind: head.kind,
        shape,
        body,
        generics: head.generics,
    }
}

/// The head of one declaration at one offset, or `None`.
fn match_head(code: &str, at: usize) -> Option<Head> {
    let attrs_end = attribute_lines(code, at);
    let attrs = cut(code, at, attrs_end).to_owned();
    let mut pos = skip(code, attrs_end, |mark| mark == b' ' || mark == b'\t');
    pos = visibility(code, pos).unwrap_or(pos);
    let (kind, after) = keyword(code, pos)?;
    pos = skip(code, after, |mark| mark.is_ascii_whitespace());
    if pos == after {
        return None;
    }
    if !byte(code, pos).is_some_and(|mark| mark.is_ascii_uppercase()) {
        return None;
    }
    let name_end = skip(code, pos, |mark| mark.is_ascii_alphanumeric());
    let name = cut(code, pos, name_end).to_owned();
    let (generics, after_generics) = generic_list(code, name_end);
    let (bound, end) = trait_bound(code, after_generics);
    let carried = if kind == Keyword::Trait {
        bound
    } else {
        String::new()
    };
    Some(Head {
        attrs,
        kind,
        name,
        generics: format!("{generics}{carried}"),
        end,
    })
}

/// The offset past every attribute line one declaration carries.
fn attribute_lines(code: &str, at: usize) -> usize {
    let mut pos = at;
    loop {
        let open = skip(code, pos, |mark| mark == b' ' || mark == b'\t');
        if !from(code, open).starts_with("#[") {
            return pos;
        }
        let line = line_at(code, open);
        if !line.ends_with(']') || byte(code, open + line.len()) != Some(b'\n') {
            return pos;
        }
        pos = open + line.len() + 1;
    }
}

/// The offset past one visibility marker, or `None` when there is none.
fn visibility(code: &str, at: usize) -> Option<usize> {
    let rest = from(code, at).strip_prefix("pub")?;
    let mut pos = at + 3;
    if rest.starts_with('(') {
        let close = skip(code, pos + 1, |mark| mark.is_ascii_lowercase());
        if close == pos + 1 || byte(code, close) != Some(b')') {
            return None;
        }
        pos = close + 1;
    }
    let after = skip(code, pos, |mark| mark.is_ascii_whitespace());
    if after == pos { None } else { Some(after) }
}

/// The declaration keyword at one offset, and the offset past it.
fn keyword(code: &str, at: usize) -> Option<(Keyword, usize)> {
    for kind in [Keyword::Struct, Keyword::Enum, Keyword::Trait] {
        let word = kind.word();
        if from(code, at).starts_with(word) {
            return Some((kind, at + word.len()));
        }
    }
    None
}

/// The generic list at one offset, and the offset past it.
fn generic_list(code: &str, at: usize) -> (String, usize) {
    if byte(code, at) != Some(b'<') {
        return (String::new(), at);
    }
    let close = skip(code, at + 1, |mark| {
        !matches!(mark, b'>' | b'{' | b'(' | b';')
    });
    if byte(code, close) != Some(b'>') {
        return (String::new(), at);
    }
    (cut(code, at, close + 1).to_owned(), close + 1)
}

/// The bound list at one offset, and the offset past it.
fn trait_bound(code: &str, at: usize) -> (String, usize) {
    let colon = skip(code, at, |mark| mark.is_ascii_whitespace());
    if byte(code, colon) != Some(b':') {
        return (String::new(), at);
    }
    let end = skip(code, colon + 1, |mark| !matches!(mark, b'{' | b';' | b'('));
    (cut(code, at, end).to_owned(), end)
}

/// One impl block the roster reads.
#[derive(Debug, Clone)]
struct ImplBlock {
    /// The declaration the impl block is for.
    target: String,
    /// The text of the whole impl block.
    text: String,
}

/// Every offset at which one text opens a trait impl, with its head end.
fn trait_impls(code: &str) -> Vec<(usize, usize, String)> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < code.len() {
        if let Some((end, target)) = match_trait_impl(code, at) {
            found.push((at, end, target));
            at = end;
            continue;
        }
        at += 1;
    }
    found
}

/// Every offset at which one text opens an inherent impl, with its head end.
fn inherent_impls(code: &str) -> Vec<(usize, usize, String)> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < code.len() {
        if let Some((end, target)) = match_inherent_impl(code, at) {
            found.push((at, end, target));
            at = end;
            continue;
        }
        at += 1;
    }
    found
}

/// Whether one offset opens the word `impl`.
fn opens_impl(code: &str, at: usize) -> bool {
    from(code, at).starts_with("impl") && boundary(code, at) && boundary(code, at + 4)
}

/// The end offset and the target of one trait impl head at one offset.
fn match_trait_impl(code: &str, at: usize) -> Option<(usize, String)> {
    if !opens_impl(code, at) {
        return None;
    }
    let plain = at + 4;
    let generic = impl_generics(code, plain);
    for start in [generic, Some(plain)].into_iter().flatten() {
        if let Some(found) = trait_impl_tail(code, start) {
            return Some(found);
        }
    }
    None
}

/// The offset past a generic list that follows `impl`, or `None`.
fn impl_generics(code: &str, at: usize) -> Option<usize> {
    let open = skip(code, at, |mark| mark.is_ascii_whitespace());
    if byte(code, open) != Some(b'<') {
        return None;
    }
    let close = skip(code, open + 1, |mark| mark != b'>');
    if byte(code, close) != Some(b'>') {
        return None;
    }
    Some(close + 1)
}

/// The rest of one trait impl head, from the offset past `impl`.
fn trait_impl_tail(code: &str, at: usize) -> Option<(usize, String)> {
    let path_start = skip(code, at, |mark| mark.is_ascii_whitespace());
    if path_start == at {
        return None;
    }
    for name_start in path_segments(code, path_start).into_iter().rev() {
        let Some(after_name) = upper_name(code, name_start) else {
            continue;
        };
        let after_args = impl_generics(code, after_name).unwrap_or(after_name);
        for stop in [after_args, after_name] {
            let Some(after_for) = for_word(code, stop) else {
                continue;
            };
            let Some(after_target) = upper_name(code, after_for) else {
                continue;
            };
            let target = cut(code, after_for, after_target).to_owned();
            let brace = skip(code, after_target, |mark| mark.is_ascii_whitespace());
            if byte(code, brace) == Some(b'{') {
                return Some((brace + 1, target));
            }
        }
    }
    None
}

/// Every offset at which one path may open its final name.
fn path_segments(code: &str, at: usize) -> Vec<usize> {
    let mut found = vec![at];
    let mut pos = at;
    loop {
        if !byte(code, pos).is_some_and(|mark| mark.is_ascii_alphabetic() || mark == b'_') {
            return found;
        }
        let name_end = skip(code, pos, |mark| {
            mark.is_ascii_alphanumeric() || mark == b'_'
        });
        let colon = skip(code, name_end, |mark| mark.is_ascii_whitespace());
        if !from(code, colon).starts_with("::") {
            return found;
        }
        pos = skip(code, colon + 2, |mark| mark.is_ascii_whitespace());
        found.push(pos);
    }
}

/// The offset past an upper-case name at one offset, or `None`.
fn upper_name(code: &str, at: usize) -> Option<usize> {
    if !byte(code, at).is_some_and(|mark| mark.is_ascii_uppercase()) {
        return None;
    }
    Some(skip(code, at, |mark| mark.is_ascii_alphanumeric()))
}

/// The offset past the word `for` and the whitespace around it, or `None`.
fn for_word(code: &str, at: usize) -> Option<usize> {
    let word = skip(code, at, |mark| mark.is_ascii_whitespace());
    if word == at || !from(code, word).starts_with("for") {
        return None;
    }
    let after = skip(code, word + 3, |mark| mark.is_ascii_whitespace());
    if after == word + 3 { None } else { Some(after) }
}

/// The end offset and the target of one inherent impl head at one offset.
fn match_inherent_impl(code: &str, at: usize) -> Option<(usize, String)> {
    if !opens_impl(code, at) {
        return None;
    }
    let name_start = skip(code, at + 4, |mark| mark.is_ascii_whitespace());
    if name_start == at + 4 {
        return None;
    }
    let name_end = upper_name(code, name_start)?;
    let brace = skip(code, name_end, |mark| mark.is_ascii_whitespace());
    if byte(code, brace) != Some(b'{') {
        return None;
    }
    Some((brace + 1, cut(code, name_start, name_end).to_owned()))
}

/// What one code block states about its impl blocks.
#[derive(Debug, Default)]
struct Impls {
    /// Every impl block whose every item carries a body.
    full: Vec<ImplBlock>,
    /// Every impl block that carries a body beside a bodiless signature.
    mixed: Vec<ImplBlock>,
    /// How many impl blocks the parse read.
    count: usize,
}

/// Every impl block one code block spells out.
fn read_impls(code: &str) -> Impls {
    let mut found = Impls::default();
    let heads: Vec<(usize, usize, String)> = trait_impls(code)
        .into_iter()
        .chain(inherent_impls(code))
        .collect();
    found.count = heads.len();
    for (start, head_end, target) in heads {
        let open = head_end.saturating_sub(1);
        let close = matching_brace(code, open);
        let text = cut(code, start, close + 1).to_owned();
        let bodiless = has_bodiless_fn(&text) || has_bodiless_const(&text);
        let inner = text.find('{').map_or(0, |at| at + 1);
        let bodied = has_body(from(&text, inner));
        if bodiless && bodied {
            found.mixed.push(ImplBlock { target, text });
            continue;
        }
        if bodiless {
            continue;
        }
        found.full.push(ImplBlock { target, text });
    }
    found
}

/// Whether one impl block carries a function signature with no body.
fn has_bodiless_fn(text: &str) -> bool {
    let mut at = 0;
    while at < text.len() {
        if from(text, at).starts_with("fn") && boundary(text, at) {
            if bodiless_fn_at(text, at + 2) {
                return true;
            }
            at += 2;
            continue;
        }
        at += 1;
    }
    false
}

/// Whether one function signature from one offset carries no body.
fn bodiless_fn_at(text: &str, at: usize) -> bool {
    let name = skip(text, at, |mark| mark.is_ascii_whitespace());
    if name == at || !byte(text, name).is_some_and(|mark| mark.is_ascii_lowercase() || mark == b'_')
    {
        return false;
    }
    let name_end = skip(text, name, |mark| {
        mark.is_ascii_alphanumeric() || mark == b'_'
    });
    let mut pos = skip(text, name_end, |mark| mark.is_ascii_whitespace());
    if byte(text, pos) == Some(b'<') {
        let close = skip(text, pos + 1, |mark| mark != b'>');
        if byte(text, close) != Some(b'>') {
            return false;
        }
        pos = skip(text, close + 1, |mark| mark.is_ascii_whitespace());
    }
    if byte(text, pos) != Some(b'(') {
        return false;
    }
    let stop = skip(text, pos + 1, |mark| !matches!(mark, b'{' | b';'));
    if byte(text, stop) != Some(b';') {
        return false;
    }
    closes_signature(text, pos + 1, stop)
}

/// Whether a round bracket in one region closes a bodiless signature.
fn closes_signature(text: &str, from_at: usize, stop: usize) -> bool {
    let mut at = stop;
    while at > from_at {
        at -= 1;
        if byte(text, at) != Some(b')') {
            continue;
        }
        let after = skip(text, at + 1, |mark| mark.is_ascii_whitespace());
        if after == stop || from(text, after).starts_with("->") {
            return true;
        }
    }
    false
}

/// Whether one impl block carries a constant signature with no body.
fn has_bodiless_const(text: &str) -> bool {
    let mut at = 0;
    while at < text.len() {
        if from(text, at).starts_with("const") && boundary(text, at) {
            if bodiless_const_at(text, at + 5) {
                return true;
            }
            at += 5;
            continue;
        }
        at += 1;
    }
    false
}

/// Whether one constant signature from one offset carries no value.
fn bodiless_const_at(text: &str, at: usize) -> bool {
    let name = skip(text, at, |mark| mark.is_ascii_whitespace());
    if name == at || !byte(text, name).is_some_and(|mark| mark.is_ascii_uppercase()) {
        return false;
    }
    let name_end = skip(text, name, |mark| {
        mark.is_ascii_alphanumeric() || mark == b'_'
    });
    let colon = skip(text, name_end, |mark| mark.is_ascii_whitespace());
    if byte(text, colon) != Some(b':') {
        return false;
    }
    let stop = skip(text, colon + 1, |mark| !matches!(mark, b'=' | b';' | b'{'));
    byte(text, stop) == Some(b';')
}

/// Whether one impl body carries at least one function with a body.
fn has_body(text: &str) -> bool {
    let mut at = 0;
    while at < text.len() {
        if byte(text, at) == Some(b')') && opens_a_body(text, at + 1) {
            return true;
        }
        at += 1;
    }
    false
}

/// Whether a body opens after one round bracket, with a return type or not.
fn opens_a_body(text: &str, at: usize) -> bool {
    let after = skip(text, at, |mark| mark.is_ascii_whitespace());
    if byte(text, after) == Some(b'{') {
        return true;
    }
    if !from(text, after).starts_with("->") {
        return false;
    }
    let stop = skip(text, after + 2, |mark| !matches!(mark, b';' | b'{'));
    byte(text, stop) == Some(b'{')
}

/// Every fenced Rust block of one document, with its comments removed.
fn code_blocks(source: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut at = 0;
    while let Some(open) = from(source, at).find("```rust\n") {
        let start = at + open + "```rust\n".len();
        let Some(close) = from(source, start).find("```") else {
            break;
        };
        found.push(join_attributes(&strip_docs(cut(
            source,
            start,
            start + close,
        ))));
        at = start + close + 3;
    }
    found
}

/// Every whole-line comment removed (substitution `S1`).
fn strip_docs(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let body = line.trim_start_matches([' ', '\t']);
        if body.starts_with("//") && line.ends_with('\n') {
            continue;
        }
        out.push_str(line);
    }
    out
}

/// Each attribute on one line, so an attribute run is a run of lines.
fn join_attributes(code: &str) -> String {
    let mut out = String::with_capacity(code.len());
    let mut at = 0;
    while at < code.len() {
        if from(code, at).starts_with("#[") {
            let end = attribute_end(code, at);
            out.push_str(&collapse(cut(code, at, end + 1)));
            at = end + 1;
            continue;
        }
        let Some(mark) = from(code, at).chars().next() else {
            break;
        };
        out.push(mark);
        at += mark.len_utf8();
    }
    out
}

/// The offset of the square bracket that closes one attribute.
fn attribute_end(code: &str, at: usize) -> usize {
    let mut depth = 0_isize;
    let mut pos = at + 1;
    while pos < code.len() {
        match byte(code, pos) {
            Some(b'[') => depth += 1,
            Some(b']') => {
                depth -= 1;
                if depth == 0 {
                    return pos;
                }
            },
            _ => {},
        }
        pos += 1;
    }
    pos
}

/// One attribute on one line, with every line break folded to one space.
fn collapse(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut pos = 0;
    while let Some(mark) = byte(text, pos) {
        if mark == b'\\' && byte(text, pos + 1) == Some(b'\n') {
            pos = skip(text, pos + 2, |next| next.is_ascii_whitespace());
            continue;
        }
        if mark.is_ascii_whitespace() {
            let end = skip(text, pos, |next| next.is_ascii_whitespace());
            let run = cut(text, pos, end);
            if run.contains('\n') {
                out.push(' ');
            } else {
                out.push_str(run);
            }
            pos = end;
            continue;
        }
        out.push(char::from(mark));
        pos += 1;
    }
    out
}

/// Every enum arm's leading identifier removed (`PG8`).
fn mask_arms(code: &str) -> String {
    let mut out = code.to_owned();
    for at in enum_heads(code).into_iter().rev() {
        let Some(open) = from(&out, at).find('{').map(|found| at + found) else {
            continue;
        };
        let close = matching_brace(&out, open);
        let body = cut(&out, open + 1, close);
        let masked: Vec<String> = split_top(body, b',')
            .iter()
            .map(|arm| mask_arm(arm))
            .collect();
        out = format!(
            "{}{}{}",
            cut(&out, 0, open + 1),
            masked.join(","),
            from(&out, close)
        );
    }
    out
}

/// One enum arm with its leading identifier removed.
fn mask_arm(arm: &str) -> String {
    let space = skip(arm, 0, |mark| mark.is_ascii_whitespace());
    if !byte(arm, space).is_some_and(|mark| mark.is_ascii_uppercase()) {
        return arm.to_owned();
    }
    let end = skip(arm, space, |mark| mark.is_ascii_alphanumeric());
    format!("{}{}", cut(arm, 0, space), from(arm, end))
}

/// The offset of the brace that opens an enum body at one offset, or `None`.
fn enum_brace(code: &str, at: usize) -> Option<usize> {
    if !from(code, at).starts_with("enum") || !boundary(code, at) {
        return None;
    }
    let name = skip(code, at + 4, |mark| mark.is_ascii_whitespace());
    if name == at + 4 {
        return None;
    }
    let end = upper_name(code, name)?;
    let brace = skip(code, end, |mark| mark.is_ascii_whitespace());
    (byte(code, brace) == Some(b'{')).then_some(brace)
}

/// Every offset at which one text opens an enum declaration.
fn enum_heads(code: &str) -> Vec<usize> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < code.len() {
        if let Some(brace) = enum_brace(code, at) {
            found.push(at);
            at = brace + 1;
            continue;
        }
        at += 1;
    }
    found
}

/// Every declaration and every impl block the document spells out.
#[derive(Debug, Default)]
struct Parse {
    /// Every declaration the section 1.5 table places, by name.
    bodies: BTreeMap<String, Item>,
    /// Every placed declaration, in the order the document states.
    seen: Vec<String>,
    /// Every full impl block, by the declaration it is for.
    impls: BTreeMap<String, Vec<String>>,
    /// Every mixed impl block the document carries.
    mixed: Vec<ImplBlock>,
    /// How many impl blocks the parse read.
    count: usize,
}

/// Every declaration and impl block the registered crates own.
fn parse_document(source: &str, owner: &BTreeMap<String, String>) -> Parse {
    let mut parse = Parse::default();
    let blocks = code_blocks(source);
    for code in &blocks {
        for item in items(code) {
            if parse.bodies.contains_key(&item.name) || !owner.contains_key(&item.name) {
                continue;
            }
            parse.seen.push(item.name.clone());
            parse.bodies.insert(item.name.clone(), item);
        }
    }
    for code in &blocks {
        let read = read_impls(code);
        parse.count += read.count;
        parse.mixed.extend(read.mixed);
        for block in read.full {
            if owner.contains_key(&block.target) {
                parse
                    .impls
                    .entry(block.target)
                    .or_default()
                    .push(block.text);
            }
        }
    }
    parse
}

/// One declaration, as the roster workspace spells it.
fn emit_item(item: &Item) -> String {
    let (attrs, dropped) = rewrite_derives(&item.attrs);
    let body = if item.kind != Keyword::Trait && !item.body.trim().is_empty() {
        make_fields_public(&item.body, item.shape)
    } else {
        item.body.clone()
    };
    let mut text = attrs.trim_end_matches('\n').to_owned();
    if !text.is_empty() {
        text.push('\n');
    }
    let word = item.kind.word();
    let name = &item.name;
    let generics = &item.generics;
    let declaration = match (item.shape, item.kind) {
        (Shape::Tuple, _) => format!("pub {word} {name}{generics}({body});\n"),
        (Shape::Unit, _) => format!("pub {word} {name}{generics};\n"),
        (Shape::Braced, Keyword::Trait) => format!("pub {word} {name}{generics} {{{body}}}\n"),
        (Shape::Braced, Keyword::Struct | Keyword::Enum) => {
            format!("pub {word} {name}{generics} {{\n    {body}\n}}\n")
        },
    };
    text.push_str(&declaration);
    if dropped.contains("Error") {
        text.push_str(&error_impls(name));
    }
    text
}

/// The `Display` and `Error` impls substitution `S3` supplies.
fn error_impls(name: &str) -> String {
    format!(
        "impl core::fmt::Display for {name} {{\n    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{\n        core::fmt::Debug::fmt(self, f)\n    }}\n}}\nimpl core::error::Error for {name} {{}}\n"
    )
}

/// The attribute run with every dropped derive removed.
fn rewrite_derives(attrs: &str) -> (String, BTreeSet<String>) {
    let Some(open) = attrs.find("#[derive(") else {
        return (attrs.to_owned(), BTreeSet::new());
    };
    let Some(close) = from(attrs, open).find(")]") else {
        return (attrs.to_owned(), BTreeSet::new());
    };
    let whole = cut(attrs, open, open + close + 2);
    let inner = cut(attrs, open + "#[derive(".len(), open + close);
    let names: Vec<&str> = inner
        .split(',')
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .collect();
    let kept: Vec<&str> = names
        .iter()
        .copied()
        .filter(|name| !DROPPED_DERIVES.contains(name))
        .collect();
    let dropped: BTreeSet<String> = names
        .iter()
        .filter(|name| DROPPED_DERIVES.contains(*name))
        .map(|name| (*name).to_owned())
        .collect();
    let replacement = if kept.is_empty() {
        String::new()
    } else {
        format!("#[derive({})]", kept.join(", "))
    };
    (attrs.replace(whole, &replacement), dropped)
}

/// Every field becomes `pub` (substitution `S2`).
fn make_fields_public(body: &str, shape: Shape) -> String {
    if shape == Shape::Tuple {
        let pieces: Vec<String> = split_top(body, b',')
            .iter()
            .map(|piece| format!("pub {}", piece.trim()))
            .collect();
        return pieces.join(", ");
    }
    let pieces: Vec<String> = split_top(body, b',')
        .iter()
        .map(|piece| public_field(piece.trim_matches('\n')))
        .collect();
    pieces.join(",\n    ")
}

/// One field of a braced body, with `pub` in front of its name.
fn public_field(text: &str) -> String {
    let prefix_end = attribute_run(text);
    let prefix = cut(text, 0, prefix_end);
    let rest = from(text, prefix_end).trim();
    if names_a_field(rest) {
        return format!("{prefix}pub {rest}");
    }
    format!("{prefix}{rest}")
}

/// The offset past the attribute run that opens one field.
fn attribute_run(text: &str) -> usize {
    let mut end = 0;
    let mut pos = 0;
    loop {
        let open = skip(text, pos, |mark| mark.is_ascii_whitespace());
        if !from(text, open).starts_with("#[") {
            return end;
        }
        let Some(close) = from(text, open).find(']') else {
            return end;
        };
        pos = skip(text, open + close + 1, |mark| mark.is_ascii_whitespace());
        end = pos;
    }
}

/// Whether one piece of a braced body names a field.
fn names_a_field(text: &str) -> bool {
    if !text
        .as_bytes()
        .first()
        .is_some_and(|mark| mark.is_ascii_lowercase() || *mark == b'_')
    {
        return false;
    }
    let end = skip(text, 0, |mark| mark.is_ascii_alphanumeric() || mark == b'_');
    let colon = skip(text, end, |mark| mark.is_ascii_whitespace());
    byte(text, colon) == Some(b':')
}

/// Every declaration and impl block one crate of the roster carries.
type PerCrate = BTreeMap<String, Vec<String>>;

/// The text each crate of the roster workspace holds.
fn per_crate(roster: &Roster, parse: &Parse) -> PerCrate {
    let mut found: PerCrate = BTreeMap::new();
    for name in &parse.seen {
        let (Some(item), Some(home)) = (parse.bodies.get(name), roster.owner.get(name)) else {
            continue;
        };
        found.entry(home.clone()).or_default().push(emit_item(item));
        for text in parse.impls.get(name).into_iter().flatten() {
            let trait_name = impl_trait_name(text);
            let where_it_goes = roster
                .impl_crate
                .get(&(name.clone(), trait_name))
                .unwrap_or(home);
            found
                .entry(where_it_goes.clone())
                .or_default()
                .push(format!("{text}\n"));
        }
    }
    for name in &roster.drops {
        let Some(home) = roster.owner.get(name) else {
            continue;
        };
        found.entry(home.clone()).or_default().push(drop_impl(name));
    }
    found
}

/// The `Drop` impl substitution `S8` supplies.
fn drop_impl(name: &str) -> String {
    format!(
        "impl Drop for {name} {{\n    fn drop(&mut self) {{\n        core::hint::black_box(&*self);\n    }}\n}}\n"
    )
}

/// The final segment of the trait one impl block names.
fn impl_trait_name(text: &str) -> String {
    let at = skip(text, 0, |mark| mark.is_ascii_whitespace());
    if !from(text, at).starts_with("impl") {
        return String::new();
    }
    let after = impl_generics(text, at + 4).unwrap_or(at + 4);
    let start = skip(text, after, |mark| mark.is_ascii_whitespace());
    let Some(stop) = from(text, start).find(" for ") else {
        return String::new();
    };
    let path = cut(text, start, start + stop).trim();
    path.rsplit("::").next().unwrap_or(path).to_owned()
}

/// Every long upper-case name one text states.
fn shouted_names(text: &str) -> BTreeSet<String> {
    scan_names(text, |mark| {
        mark.is_ascii_uppercase() || mark.is_ascii_digit() || mark == b'_'
    })
    .into_iter()
    .filter(|name| name.len() >= 3)
    .collect()
}

/// Every capitalised name one text states.
fn capital_names(text: &str) -> BTreeSet<String> {
    scan_names(text, |mark| mark.is_ascii_alphanumeric())
        .into_iter()
        .collect()
}

/// Every name of one text whose first letter is upper case.
fn scan_names<F: Fn(u8) -> bool>(text: &str, body: F) -> Vec<String> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let opens = byte(text, at).is_some_and(|mark| mark.is_ascii_uppercase());
        if opens && !preceded_by_path(text, at) {
            let end = skip(text, at, &body);
            found.push(cut(text, at, end).to_owned());
            at = end;
            continue;
        }
        at += 1;
    }
    found
}

/// Whether one offset follows a path separator or a word character.
fn preceded_by_path(text: &str, at: usize) -> bool {
    at.checked_sub(1)
        .and_then(|back| byte(text, back))
        .is_some_and(|mark| mark == b':' || word_byte(mark))
}

/// Every block comment of one text replaced by one space.
fn strip_block_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let closed = from(text, at)
            .starts_with("/*")
            .then(|| from(text, at + 2).find("*/"))
            .flatten();
        if let Some(close) = closed {
            out.push(' ');
            at = at + 2 + close + 2;
            continue;
        }
        let Some(mark) = from(text, at).chars().next() else {
            break;
        };
        out.push(mark);
        at += mark.len_utf8();
    }
    out
}

/// Every derive attribute of one text replaced by one space.
fn strip_derives(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let closed = from(text, at)
            .starts_with("#[derive(")
            .then(|| from(text, at).find(")]"))
            .flatten();
        if let Some(close) = closed {
            out.push(' ');
            at = at + close + 2;
            continue;
        }
        let Some(mark) = from(text, at).chars().next() else {
            break;
        };
        out.push(mark);
        at += mark.len_utf8();
    }
    out
}

/// The import lines one crate of the roster workspace needs.
fn imports_of(roster: &Roster, row: &str, text: &str) -> Vec<String> {
    let clean = mask_arms(&strip_block_comments(text));
    let outside = capital_names(&strip_derives(&clean));
    let mut used = capital_names(&clean);
    used.extend(shouted_names(text));
    let mut wanted = Vec::new();
    for dependency in roster.edges.get(row).into_iter().flatten() {
        let names: Vec<String> = used
            .iter()
            .filter(|name| roster.owner.get(*name) == Some(dependency))
            .cloned()
            .collect();
        let ident = crate_ident(dependency);
        match names.as_slice() {
            [] => {},
            [one] => wanted.push(format!("use {ident}::{one};")),
            many => wanted.push(format!("use {ident}::{{{}}};", many.join(", "))),
        }
    }
    for name in &used {
        let Some(path) = roster.paths.get(name) else {
            continue;
        };
        if roster.owner.contains_key(name) {
            continue;
        }
        let head = path.first().map(String::as_str).unwrap_or_default();
        if outside.contains(name) || !(head.starts_with("std::") || head.starts_with("core::")) {
            wanted.push(format!("use {};", path.join(" ")));
        }
    }
    for name in &used {
        let Some((home, _text)) = roster.constants.get(name) else {
            continue;
        };
        if !home.is_empty() && home != row {
            wanted.push(format!("use {}::{name};", crate_ident(home)));
        }
    }
    let unique: BTreeSet<String> = wanted.into_iter().collect();
    unique.into_iter().collect()
}

/// The crate-root lines one crate of the roster workspace opens with.
fn crate_head(row: &str) -> Vec<String> {
    if row != APP_ROW {
        return vec![
            "//! One crate of the scratch roster workspace.".to_owned(),
            String::new(),
        ];
    }
    [
        "//! One crate of the scratch roster workspace.",
        "#![expect(",
        "    missing_copy_implementations,",
        "    reason = \"the roster compile builds the application row as a library and the \\",
        "              real crate is a binary, where the lint cannot fire (section 15.16)\"",
        ")]",
        "#![expect(",
        "    missing_debug_implementations,",
        "    reason = \"the five action types come from `gpui_kit::actions!`, which supplies \\",
        "              the derive the roster does not spell out (section 15.16)\"",
        ")]",
        "",
    ]
    .iter()
    .map(|line| (*line).to_owned())
    .collect()
}

/// One member manifest of the roster workspace.
fn member_manifest(roster: &Roster, row: &str) -> String {
    let package = package_name(row);
    let mut lines = package_head(package, "One crate of the Duet roster compile.");
    lines.push("[lib]".to_owned());
    lines.push(format!("name = \"{}\"", crate_ident(row)));
    lines.push("path = \"src/lib.rs\"".to_owned());
    lines.push(String::new());
    lines.push("[dependencies]".to_owned());
    for dependency in roster.edges.get(row).into_iter().flatten() {
        let name = package_name(dependency);
        lines.push(format!("{name} = {{ path = \"../{name}\" }}"));
    }
    for (pin, spec) in &roster.pins {
        if row != APP_ROW && APP_ONLY_PINS.contains(&pin.as_str()) {
            continue;
        }
        lines.push(pin_entry(pin, spec));
    }
    lines.extend([
        String::new(),
        "[lints]".to_owned(),
        "workspace = true".to_owned(),
        String::new(),
    ]);
    lines.join("\n")
}

/// The package table every member of the roster workspace carries.
fn package_head(name: &str, description: &str) -> Vec<String> {
    [
        "[package]".to_owned(),
        format!("name = \"{name}\""),
        "version = \"0.1.0\"".to_owned(),
        "edition = \"2024\"".to_owned(),
        "rust-version = \"1.98\"".to_owned(),
        "license = \"MIT\"".to_owned(),
        "repository = \"https://example.invalid/duet-roster\"".to_owned(),
        format!("description = \"{description}\""),
        "documentation = \"https://example.invalid/duet-roster\"".to_owned(),
        "readme = \"../README.md\"".to_owned(),
        "keywords = [\"duet\"]".to_owned(),
        "categories = [\"development-tools\"]".to_owned(),
        "publish = false".to_owned(),
        String::new(),
    ]
    .to_vec()
}

/// One dependency line of the roster workspace.
///
/// A leading `-` in the feature list is `default-features = false`.
fn pin_entry(pin: &str, spec: &[String]) -> String {
    let version = spec.first().map(String::as_str).unwrap_or_default();
    let tail: Vec<&String> = spec.iter().skip(1).collect();
    let features: Vec<String> = tail
        .iter()
        .filter(|token| token.as_str() != "-")
        .map(|token| format!("\"{token}\""))
        .collect();
    let mut entry = format!("{pin} = {{ version = \"{version}\"");
    if tail.iter().any(|token| token.as_str() == "-") {
        entry.push_str(", default-features = false");
    }
    if !features.is_empty() {
        let listed = format!(", features = [{}]", features.join(", "));
        entry.push_str(&listed);
    }
    entry.push_str(" }");
    entry
}

/// Write one crate of the roster workspace.
///
/// # Errors
/// Returns an error when a file cannot be written.
fn write_member(
    workspace: &Path,
    roster: &Roster,
    row: &str,
    parts: Parts<'_>,
) -> anyhow::Result<()> {
    let package = package_name(row);
    let directory = workspace.join(package);
    let mut lines = crate_head(row);
    lines.extend(imports_of(roster, row, parts.text));
    lines.push(String::new());
    lines.extend(parts.shared.iter().filter_map(|name| {
        roster
            .constants
            .get(name)
            .filter(|(home, _text)| home == row)
            .map(|(_home, text)| text.clone())
    }));
    lines.push(String::new());
    let body = format!("{}{}", lines.join("\n"), parts.text);
    write_file(&directory.join("src").join("lib.rs"), &body)?;
    write_file(&directory.join("Cargo.toml"), &member_manifest(roster, row))
}

/// What one member of the roster workspace is written from.
#[derive(Debug, Clone, Copy)]
struct Parts<'a> {
    /// Every declaration and impl block the crate carries.
    text: &'a str,
    /// Every long upper-case name the whole roster states.
    shared: &'a BTreeSet<String>,
}

/// Write one text to one path, and create the directories it needs.
///
/// # Errors
/// Returns an error when the directory or the file cannot be written.
fn write_file(path: &Path, text: &str) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot create {}", parent.display()))?;
    }
    fs::write(path, text).with_context(|| format!("cannot write {}", path.display()))
}

/// The rows the size oracle measures, and the crates they come from.
#[derive(Debug, Default)]
struct Oracle {
    /// Every expression the oracle measures, in the order it emits them.
    rows: Vec<String>,
    /// Every name the oracle imports, by the crate that declares it.
    imports: BTreeMap<String, BTreeSet<String>>,
}

/// Every expression the size oracle measures.
fn oracle_rows(roster: &Roster, parse: &Parse) -> Oracle {
    let mut found = Oracle::default();
    for (name, item) in &parse.bodies {
        if !item.generics.is_empty() || item.kind == Keyword::Trait {
            continue;
        }
        let hollow = item.kind == Keyword::Struct
            && item.shape == Shape::Braced
            && strip_block_comments(&item.body).trim().is_empty();
        if hollow {
            continue;
        }
        found.rows.push(name.clone());
        if let Some(home) = roster.owner.get(name) {
            found
                .imports
                .entry(home.clone())
                .or_default()
                .insert(name.clone());
        }
    }
    for (expression, spec) in &roster.sizes {
        if spec.iter().any(|token| token == "generic") {
            continue;
        }
        found.rows.push(expression.clone());
    }
    for row in &found.rows.clone() {
        for token in expression_names(row) {
            let Some(home) = home_of(roster, &token) else {
                continue;
            };
            found.imports.entry(home).or_default().insert(token);
        }
    }
    found
}

/// The crate that declares one name, or that declares the constant it names.
fn home_of(roster: &Roster, token: &str) -> Option<String> {
    if let Some(home) = roster.owner.get(token) {
        return Some(home.clone());
    }
    roster
        .constants
        .get(token)
        .map(|(home, _text)| home)
        .filter(|home| !home.is_empty())
        .cloned()
}

/// Every name one expression states.
fn expression_names(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut at = 0;
    while at < text.len() {
        let opens = byte(text, at).is_some_and(|mark| mark.is_ascii_alphabetic() || mark == b'_');
        if opens && !preceded_by_path(text, at) {
            let end = skip(text, at, |mark| {
                mark.is_ascii_alphanumeric() || mark == b'_'
            });
            found.push(cut(text, at, end).to_owned());
            at = end;
            continue;
        }
        at += 1;
    }
    found
}

/// One expression with every external name spelled in full.
fn qualify(roster: &Roster, text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let opens = byte(text, at).is_some_and(|mark| mark.is_ascii_alphabetic() || mark == b'_');
        if opens && !preceded_by_path(text, at) {
            let end = skip(text, at, |mark| {
                mark.is_ascii_alphanumeric() || mark == b'_'
            });
            let token = cut(text, at, end);
            let spelled = roster
                .paths
                .get(token)
                .filter(|_path| !roster.owner.contains_key(token))
                .and_then(|path| path.first())
                .map_or(token, String::as_str);
            out.push_str(spelled);
            at = end;
            continue;
        }
        let Some(mark) = from(text, at).chars().next() else {
            break;
        };
        out.push(mark);
        at += mark.len_utf8();
    }
    out
}

/// The tail of the size oracle, which prints one line per row.
const ORACLE_TAIL: [&str; 20] = [
    "];",
    "",
    "/// Print one line per row.",
    "///",
    "/// # Errors",
    "/// It returns the error the output handle reports.",
    "fn report(out: &mut impl std::io::Write) -> std::io::Result<()> {",
    "    for (name, size, align) in ROWS {",
    "        writeln!(out, \"{name} {size} {align}\")?;",
    "    }",
    "    Ok(())",
    "}",
    "",
    "/// Write every measured size to the locked output handle.",
    "fn main() -> ExitCode {",
    "    let stdout = std::io::stdout();",
    "    let mut out = stdout.lock();",
    "    if report(&mut out).is_err() {",
    "        return ExitCode::FAILURE;",
    "    }",
];

/// Write the member that measures every size this plan states.
///
/// # Errors
/// Returns an error when a file cannot be written.
fn write_size_oracle(workspace: &Path, roster: &Roster, oracle: &Oracle) -> anyhow::Result<()> {
    let mut lines = vec![
        "//! The size oracle of the roster compile.".to_owned(),
        String::new(),
        "use std::process::ExitCode;".to_owned(),
    ];
    for (row, names) in &oracle.imports {
        let ident = crate_ident(row);
        let listed: Vec<&str> = names.iter().map(String::as_str).collect();
        match listed.as_slice() {
            [] => {},
            [one] => lines.push(format!("use {ident}::{one};")),
            many => lines.push(format!("use {ident}::{{{}}};", many.join(", "))),
        }
    }
    lines.extend([
        String::new(),
        "/// Every expression this workspace measures.".to_owned(),
        "const ROWS: &[(&str, usize, usize)] = &[".to_owned(),
    ]);
    for expression in &oracle.rows {
        let form = qualify(roster, expression);
        lines.push(format!(
            "    (\"{expression}\", size_of::<{form}>(), align_of::<{form}>()),"
        ));
    }
    lines.extend(ORACLE_TAIL.iter().map(|line| (*line).to_owned()));
    lines.extend([
        "    ExitCode::SUCCESS".to_owned(),
        "}".to_owned(),
        String::new(),
    ]);
    let directory = workspace.join(SIZE_MEMBER);
    write_file(&directory.join("src").join("main.rs"), &lines.join("\n"))?;
    write_file(
        &directory.join("Cargo.toml"),
        &oracle_manifest(roster, oracle),
    )
}

/// The manifest of the member that measures every size.
fn oracle_manifest(roster: &Roster, oracle: &Oracle) -> String {
    let mut lines = package_head(SIZE_MEMBER, "The size oracle of the Duet roster compile.");
    lines.push("[[bin]]".to_owned());
    lines.push(format!("name = \"{SIZE_MEMBER}\""));
    lines.push("path = \"src/main.rs\"".to_owned());
    lines.push(String::new());
    lines.push("[dependencies]".to_owned());
    for row in oracle.imports.keys() {
        if row.is_empty() {
            continue;
        }
        let name = package_name(row);
        lines.push(format!("{name} = {{ path = \"../{name}\" }}"));
    }
    for (pin, spec) in &roster.pins {
        lines.push(pin_entry(pin, spec));
    }
    lines.extend([
        String::new(),
        "[lints]".to_owned(),
        "workspace = true".to_owned(),
        String::new(),
    ]);
    lines.join("\n")
}

/// The relaxed rustc half of the roster-compile profile.
const RUST_RELAXED: &str = "# The roster-compile profile, half one. The roster carries declarations\n# and not documentation.\nmissing_docs = \"allow\"\n\n";

/// The relaxed clippy half of the roster-compile profile.
const CLIPPY_RELAXED: &str = "\n# The roster-compile profile, half two. The roster carries declarations\n# and not documentation.\nmissing_docs_in_private_items = \"allow\"\n# The roster carries no function body, so each lint below reads an impl\n# that the roster does not spell out.\nmust_use_candidate = \"allow\"\nmissing_const_for_fn = \"allow\"\nnew_without_default = \"allow\"\nmissing_panics_doc = \"allow\"\nmissing_errors_doc = \"allow\"\n";

/// The repository lint table, relaxed for the roster compile.
fn lint_table(manifest: &str) -> Option<String> {
    let open = manifest.find("[workspace.lints.rust]\n")?;
    let close = find_profiles(manifest, open)?;
    let table = cut(manifest, open, close);
    let mut kept: Vec<&str> = Vec::new();
    for line in table.split_inclusive('\n') {
        let named = RELAXED_LINTS
            .iter()
            .any(|lint| line.starts_with(&format!("{lint} = ")) && line.ends_with('\n'));
        if !named {
            kept.push(line);
        }
    }
    let joined = kept.concat();
    let inserted = joined.replace(
        "[workspace.lints.rustdoc]",
        &format!("{RUST_RELAXED}[workspace.lints.rustdoc]"),
    );
    Some(format!(
        "{}\n{CLIPPY_RELAXED}",
        inserted.trim_end_matches('\n')
    ))
}

/// The offset at which the profile banner closes the lint table.
fn find_profiles(manifest: &str, open: usize) -> Option<usize> {
    let mut at = open;
    while let Some(found) = from(manifest, at).find("\n# ") {
        let start = at + found;
        let rule = skip(manifest, start + 3, |mark| mark == b'-');
        if rule > start + 3 && from(manifest, rule).starts_with("\n# Profiles") {
            return Some(start);
        }
        at = start + 1;
    }
    None
}

/// Write the scratch workspace and report every count the plan states.
///
/// # Errors
/// Returns an error when a write to the output stream fails, or when a file
/// the guard needs cannot be read or written.
fn generate(out: &mut impl io::Write, places: &Places) -> anyhow::Result<Outcome> {
    let source = fs::read_to_string(&places.document)
        .with_context(|| format!("cannot read {}", places.document.display()))?;
    let blocks = match read_blocks(&source) {
        Ok(blocks) => blocks,
        Err(failures) => {
            for (id, reason) in failures {
                writeln!(
                    out,
                    "FAIL: the `{id}` block of this document: {reason}; the guard is fail-closed (DR7)."
                )?;
            }
            return Ok(Outcome::FailClosed);
        },
    };
    if substitutions_disagree(out, &blocks.substitutions)? {
        return Ok(Outcome::FailClosed);
    }
    let roster = Roster::read(&blocks);
    if let Some(name) = roster.unplaced_drop() {
        writeln!(
            out,
            "FAIL: the Drop block names {name}, which section 1.5 places nowhere."
        )?;
        return Ok(Outcome::FailClosed);
    }
    let parse = parse_document(&source, &roster.owner);
    let carried = per_crate(&roster, &parse);
    let manifest = fs::read_to_string(places.repo.join("Cargo.toml"))
        .with_context(|| format!("cannot read {}", places.repo.display()))?;
    write_workspace(places, &roster, &carried, &parse)?;
    let Some(table) = lint_table(&manifest) else {
        writeln!(
            out,
            "FAIL: the repository manifest holds no `[workspace.lints]` table."
        )?;
        return Ok(Outcome::FailClosed);
    };
    write_root(places, &roster, &table)?;
    report(out, &roster, &parse, &blocks)
}

/// Whether the substitution block and this guard disagree.
///
/// # Errors
/// Returns an error when a write to the output stream fails.
fn substitutions_disagree(out: &mut impl io::Write, lines: &[String]) -> anyhow::Result<bool> {
    let applied = fenced_map(lines);
    let listed: Vec<String> = applied.keys().cloned().collect();
    let wanted: Vec<String> = SUBSTITUTIONS.iter().map(|id| (*id).to_owned()).collect();
    if listed != wanted {
        writeln!(
            out,
            "FAIL: the section 1.9 substitution block and the script disagree; the block holds {} and the script applies {}.",
            printed_list(&listed),
            printed_list(&wanted)
        )?;
        return Ok(true);
    }
    for line in lines {
        if states_a_number(line) {
            writeln!(
                out,
                "FAIL: a substitution line states a number, and only a run may state one (critic WR-14): {}",
                line.trim()
            )?;
            return Ok(true);
        }
    }
    Ok(false)
}

/// Whether one substitution line states a number.
///
/// A rule id, a budget id, and a section number each carry a digit and none of
/// the three is a count, so all three leave the text before the search.
fn states_a_number(line: &str) -> bool {
    let tail = line.trim_start().split_once(char::is_whitespace);
    let text = tail.map(|(_head, rest)| rest).unwrap_or_default();
    let without_ids = strip_matches(text, rule_id_at);
    let without_sections = strip_matches(&without_ids, section_at);
    without_sections.bytes().any(|mark| mark.is_ascii_digit())
}

/// One text with every match of one pattern removed.
fn strip_matches<F: Fn(&str, usize) -> Option<usize>>(text: &str, matcher: F) -> String {
    let mut out = String::with_capacity(text.len());
    let mut at = 0;
    while at < text.len() {
        let found = boundary(text, at)
            .then(|| matcher(text, at).filter(|end| boundary(text, *end)))
            .flatten();
        if let Some(end) = found {
            at = end;
            continue;
        }
        let Some(mark) = from(text, at).chars().next() else {
            break;
        };
        out.push(mark);
        at += mark.len_utf8();
    }
    out
}

/// The offset past a rule identifier at one offset, or `None`.
fn rule_id_at(text: &str, at: usize) -> Option<usize> {
    let letters = skip(text, at, |mark| mark.is_ascii_uppercase());
    if letters == at || letters > at + 2 {
        return None;
    }
    let digits = skip(text, letters, |mark| mark.is_ascii_digit());
    if digits == letters {
        return None;
    }
    let tail = skip(text, digits, |mark| mark.is_ascii_lowercase());
    Some(if tail > digits + 1 { digits } else { tail })
}

/// The offset past a section number at one offset, or `None`.
fn section_at(text: &str, at: usize) -> Option<usize> {
    let rest = from(text, at).strip_prefix("section ")?;
    let head = at + "section ".len();
    let major = skip(text, head, |mark| mark.is_ascii_digit());
    if major == head || byte(text, major) != Some(b'.') || rest.is_empty() {
        return None;
    }
    let minor = skip(text, major + 1, |mark| mark.is_ascii_digit());
    if minor == major + 1 {
        return None;
    }
    let tail = skip(text, minor, |mark| mark.is_ascii_lowercase());
    Some(if tail > minor + 1 { minor } else { tail })
}

/// Write every member of the scratch workspace.
///
/// # Errors
/// Returns an error when a file cannot be written.
fn write_workspace(
    places: &Places,
    roster: &Roster,
    carried: &PerCrate,
    parse: &Parse,
) -> anyhow::Result<()> {
    let workspace = places.workspace();
    match fs::remove_dir_all(&workspace) {
        Ok(()) => {},
        Err(error) if error.kind() == io::ErrorKind::NotFound => {},
        Err(error) => {
            return Err(error).with_context(|| format!("cannot clear {}", workspace.display()));
        },
    }
    let shared: BTreeSet<String> = carried
        .values()
        .flat_map(|texts| shouted_names(&texts.concat()))
        .collect();
    for row in &roster.order {
        let text = carried
            .get(row)
            .map(|texts| texts.concat())
            .unwrap_or_default();
        write_member(
            &workspace,
            roster,
            row,
            Parts {
                text: &text,
                shared: &shared,
            },
        )?;
    }
    write_size_oracle(&workspace, roster, &oracle_rows(roster, parse))
}

/// Write the workspace manifest and every policy file the compile needs.
///
/// # Errors
/// Returns an error when a file cannot be written.
fn write_root(places: &Places, roster: &Roster, table: &str) -> anyhow::Result<()> {
    let workspace = places.workspace();
    let mut members: Vec<String> = roster
        .order
        .iter()
        .map(|row| format!("\"{}\"", package_name(row)))
        .collect();
    members.push(format!("\"{SIZE_MEMBER}\""));
    let root = [
        "[workspace]".to_owned(),
        "resolver = \"3\"".to_owned(),
        format!("members = [{}]", members.join(", ")),
        String::new(),
        table.to_owned(),
    ];
    write_file(&workspace.join("Cargo.toml"), &root.join("\n"))?;
    fs::create_dir_all(workspace.join(".cargo"))
        .with_context(|| format!("cannot create {}", workspace.display()))?;
    for name in [
        ".cargo/config.toml",
        "clippy.toml",
        "rust-toolchain.toml",
        "Cargo.lock",
    ] {
        let source = places.repo.join(name);
        if source.is_file() {
            fs::copy(&source, workspace.join(name))
                .with_context(|| format!("cannot copy {}", source.display()))?;
        }
    }
    write_file(&workspace.join("README.md"), "Scratch roster workspace.\n")
}

/// Report every count the roster states, and every floor it breaks.
///
/// # Errors
/// Returns an error when a write to the output stream fails.
fn report(
    out: &mut impl io::Write,
    roster: &Roster,
    parse: &Parse,
    blocks: &Blocks,
) -> anyhow::Result<Outcome> {
    let crates = roster.order.len() + 1;
    let floor = roster.owner.len();
    writeln!(out, "ROSTER CRATES:   {crates}")?;
    writeln!(
        out,
        "ROSTER ITEMS:    {}     FLOOR: {floor}",
        parse.seen.len()
    )?;
    if parse.seen.len() < floor {
        let seen: BTreeSet<&String> = parse.seen.iter().collect();
        let missing = roster.owner.keys().find(|name| !seen.contains(name));
        writeln!(
            out,
            "FAIL: the roster holds {} items and section 1.5 places {floor}; the first name with no declaration is {}.",
            parse.seen.len(),
            missing.map(String::as_str).unwrap_or_default()
        )?;
        return Ok(Outcome::Findings);
    }
    writeln!(out, "ROSTER IMPL MIXED: {}", parse.mixed.len())?;
    if !parse.mixed.is_empty() {
        let mut named: Vec<&ImplBlock> = parse.mixed.iter().collect();
        named.sort_by(|left, right| (&left.target, &left.text).cmp(&(&right.target, &right.text)));
        for block in named {
            writeln!(
                out,
                "FAIL: the impl block for {} carries a body beside a bodiless signature, so the whole block leaves the roster; write every item as a signature or write every item with a body.",
                block.target
            )?;
        }
        return Ok(Outcome::Findings);
    }
    let impl_floor = blocks.impl_sites.len();
    let placed: usize = parse.impls.values().map(Vec::len).sum();
    writeln!(
        out,
        "ROSTER IMPL BLOCKS: {}     FLOOR: {impl_floor}",
        parse.count
    )?;
    writeln!(out, "ROSTER IMPLS:    {placed}")?;
    if parse.count != impl_floor {
        writeln!(
            out,
            "FAIL: the roster parsed {} impl blocks and the `impl-sites` block lists {impl_floor}; the two are one set and they differ by {}.",
            parse.count,
            parse.count.abs_diff(impl_floor)
        )?;
        return Ok(Outcome::Findings);
    }
    writeln!(out, "ROSTER DROPS:    {}", roster.drops.len())?;
    writeln!(out, "ROSTER CONSTS:   {}", roster.constants.len())?;
    writeln!(out, "ROSTER SUBS:     {}", SUBSTITUTIONS.len())?;
    Ok(Outcome::Clean)
}

/// One size the document records.
#[derive(Debug, Clone)]
struct Recorded {
    /// The size the document states.
    size: String,
    /// The alignment the document states.
    align: String,
    /// Whether the row names a container head with no size of its own.
    head: bool,
}

/// Every size the section 1.9 block records.
fn recorded_sizes(source: &str) -> BTreeMap<String, Recorded> {
    let mut found = BTreeMap::new();
    let Some(heading) = source.find("#### Every size the guard records\n") else {
        return found;
    };
    let Some(open) = from(source, heading).find("```text\n") else {
        return found;
    };
    let body = from(source, heading + open + "```text\n".len());
    let Some(close) = body.find("```") else {
        return found;
    };
    for line in cut(body, 0, close).lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let (Some(name), Some(size), Some(align)) = (parts.first(), parts.get(1), parts.get(2))
        else {
            continue;
        };
        found.insert(
            (*name).to_owned(),
            Recorded {
                size: (*size).to_owned(),
                align: (*align).to_owned(),
                head: parts.contains(&"generic"),
            },
        );
    }
    found
}

/// Every size the oracle measured.
fn measured_sizes(text: &str) -> BTreeMap<String, (String, String)> {
    let mut found = BTreeMap::new();
    for line in text.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() != 3 {
            continue;
        }
        let (Some(name), Some(size), Some(align)) = (parts.first(), parts.get(1), parts.get(2))
        else {
            continue;
        };
        found.insert(
            (*name).to_owned(),
            ((*size).to_owned(), (*align).to_owned()),
        );
    }
    found
}

/// Compare every recorded size with the measured one.
///
/// # Errors
/// Returns an error when a write to the output stream fails, or when the
/// document cannot be read.
fn compare_sizes(
    out: &mut impl io::Write,
    document: &Path,
    measured: &str,
) -> anyhow::Result<Outcome> {
    let source = fs::read_to_string(document)
        .with_context(|| format!("cannot read {}", document.display()))?;
    let rows = recorded_sizes(&source);
    let facts = measured_sizes(measured);
    if rows.is_empty() || facts.is_empty() {
        writeln!(
            out,
            "FAIL: a size table is empty; the guard is fail-closed."
        )?;
        return Ok(Outcome::FailClosed);
    }
    let mut checked = 0_usize;
    let mut heads = 0_usize;
    let mut bad: Vec<String> = Vec::new();
    for (expression, recorded) in &rows {
        if recorded.head {
            heads += 1;
            continue;
        }
        let Some((size, align)) = facts.get(expression) else {
            bad.push(format!(
                "  SIZE:       {expression} is recorded {}/{} and measures no/measurement",
                recorded.size, recorded.align
            ));
            continue;
        };
        checked += 1;
        if (size, align) != (&recorded.size, &recorded.align) {
            bad.push(format!(
                "  SIZE:       {expression} is recorded {}/{} and measures {size}/{align}",
                recorded.size, recorded.align
            ));
        }
    }
    writeln!(out, "ROSTER SIZES:    {} measured", facts.len())?;
    writeln!(
        out,
        "RECORDED ROWS:   {checked} checked     HEAD ROWS: {heads}     SIZE BAD: {}",
        bad.len()
    )?;
    for line in &bad {
        writeln!(out, "{line}")?;
    }
    if bad.is_empty() {
        Ok(Outcome::Clean)
    } else {
        Ok(Outcome::Findings)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Kind, Shape, capital_names, cargo_reason, collapse, is_lint_line, is_literal_value,
        is_position, join_continuations, make_fields_public, marker_of, mask_arms, printed_list,
        rule_id_at, section_at, split_top, states_a_number, strip_docs, under,
    };

    #[test]
    fn a_scratch_path_under_the_repository_is_refused() {
        assert!(
            under("/repo", "/repo"),
            "the repository root is inside itself"
        );
        assert!(
            under("/repo/x", "/repo"),
            "a child of the root is inside it"
        );
        assert!(
            !under("/repos", "/repo"),
            "a sibling name is not inside the root"
        );
    }

    #[test]
    fn a_marker_line_states_its_id_and_its_floor() {
        assert_eq!(
            marker_of("<!-- GUARD BLOCK id=pins rows>=13 -->"),
            Some(("pins", 13)),
            "the marker states the id and the floor"
        );
        assert_eq!(
            marker_of("<!-- GUARD BLOCK id=Pins rows>=13 -->"),
            None,
            "an upper-case id is no marker"
        );
    }

    #[test]
    fn a_whole_line_comment_leaves_the_roster() {
        assert_eq!(
            strip_docs("/// doc\npub struct A;\n    // note\n"),
            "pub struct A;\n",
            "substitution S1 removes every whole-line comment"
        );
    }

    #[test]
    fn a_continuation_line_folds_onto_the_line_that_opens_it() {
        assert_eq!(
            join_continuations("a -> b,\n    c\nd -> e"),
            "a -> b, c\nd -> e",
            "a comma at a line end folds the next line onto it"
        );
    }

    #[test]
    fn an_attribute_that_spans_lines_becomes_one_line() {
        assert_eq!(
            collapse("#[expect(\n    lint,\n)]"),
            "#[expect( lint, )]",
            "every line break of an attribute folds to one space"
        );
    }

    #[test]
    fn every_field_of_a_braced_body_becomes_public() {
        assert_eq!(
            make_fields_public("a: u8,\n    b: u16", Shape::Braced),
            "pub a: u8,\n    pub b: u16",
            "substitution S2 makes every field public"
        );
        assert_eq!(
            make_fields_public("u8", Shape::Tuple),
            "pub u8",
            "a tuple field becomes public too"
        );
    }

    #[test]
    fn an_enum_arm_name_is_not_a_type() {
        assert_eq!(
            mask_arms("enum A { Down, Up }"),
            "enum A { ,  }",
            "PG8 removes the leading identifier of every arm"
        );
    }

    #[test]
    fn a_name_after_a_path_separator_is_not_a_name_of_its_own() {
        let found = capital_names("Alpha core::Beta Gamma");
        assert!(found.contains("Alpha"), "a bare name is read");
        assert!(found.contains("Gamma"), "a second bare name is read");
        assert!(!found.contains("Beta"), "a name behind `::` is not read");
    }

    #[test]
    fn a_value_is_a_literal_only_when_it_states_no_call() {
        assert!(is_literal_value("48"), "an integer literal is a literal");
        assert!(
            is_literal_value("MAX_STRIPS * 2 * MAX_SLOTS"),
            "arithmetic over constants is a literal"
        );
        assert!(!is_literal_value("non_zero(1_920)"), "a call is no literal");
    }

    #[test]
    fn a_substitution_line_may_state_no_count() {
        assert!(
            !states_a_number("S1 line-comment  every comment goes, per WR14 and section 1.9"),
            "a rule id and a section number state no count"
        );
        assert!(
            states_a_number("S1 line-comment  33 blocks and 11 complete"),
            "a bare count is a finding"
        );
    }

    #[test]
    fn a_rule_id_and_a_section_number_are_read_whole() {
        assert_eq!(
            rule_id_at("WR14", 0),
            Some(4),
            "the letters and digits are read"
        );
        assert_eq!(
            rule_id_at("WR-14", 0),
            None,
            "a hyphen between the letters and the digits is no rule id"
        );
        assert_eq!(
            section_at("section 1.9 holds", 0),
            Some(11),
            "a section number is read"
        );
    }

    #[test]
    fn a_top_level_comma_divides_and_a_nested_one_does_not() {
        assert_eq!(
            split_top("a, b<c, d>, e", b','),
            vec!["a", " b<c, d>", " e"],
            "a comma inside brackets holds its piece together"
        );
    }

    #[test]
    fn a_printed_list_reads_as_the_prototype_prints_it() {
        assert_eq!(
            printed_list(&["S1".to_owned(), "S2".to_owned()]),
            "['S1', 'S2']",
            "the list reads as the prototype prints it"
        );
    }

    #[test]
    fn a_lint_diagnostic_states_a_bare_level_at_a_file_position() {
        assert!(
            is_lint_line("src/lib.rs:2:1: error: missing documentation for a function"),
            "a bare level at a file position is a lint"
        );
        assert!(
            is_lint_line("src/lib.rs:5:9: warning: unused variable: `mark`"),
            "a warning at a file position is a lint too"
        );
    }

    #[test]
    fn a_coded_error_at_a_file_position_is_no_lint() {
        assert!(
            !is_lint_line("src/lib.rs:2:23: error[E0308]: mismatched types"),
            "an error code makes the line a compiler error, not a lint"
        );
    }

    #[test]
    fn a_line_with_no_file_position_is_no_lint() {
        assert!(
            !is_lint_line("error: no matching package named `alpha` found"),
            "a resolution failure carries no file position"
        );
        assert!(
            !is_lint_line("error: could not compile `alpha` (lib) due to 1 previous error"),
            "the build summary carries no file position"
        );
    }

    #[test]
    fn a_file_position_ends_with_a_row_and_a_column() {
        assert!(
            is_position("src/lib.rs:2:1: "),
            "a path, a row, and a column are a position"
        );
        assert!(!is_position(""), "an empty head is no position");
        assert!(
            !is_position("src/lib.rs:two:1: "),
            "a row that states no number is no position"
        );
        assert!(
            !is_position(":2:1: "),
            "a position with no path is no position"
        );
        assert!(
            !is_position("src/lib.rs:2:1"),
            "a head that does not close the field is no position"
        );
    }

    #[test]
    fn the_cargo_reason_is_the_first_line_that_names_an_error() {
        assert_eq!(
            cargo_reason(
                "    Updating crates.io index\nerror: no matching package named `alpha` found\n\
                 error: could not compile `alpha`\n"
            ),
            "error: no matching package named `alpha` found",
            "the first line that names an error carries the cause"
        );
        assert_eq!(
            cargo_reason("    Blocking waiting for file lock\n"),
            "Blocking waiting for file lock",
            "a report with no error line falls back to its first line"
        );
        assert_eq!(
            cargo_reason(""),
            "cargo printed no reason",
            "an empty report states that cargo printed nothing"
        );
    }

    #[test]
    fn a_long_cargo_reason_is_bounded() {
        let long = format!("error: {}", "x".repeat(super::REASON_WIDTH));
        let bounded = cargo_reason(&long);
        assert!(bounded.ends_with(" ..."), "a cut reason ends with the mark");
        assert_eq!(
            bounded.chars().count(),
            super::REASON_WIDTH + 4,
            "the reason holds the width and the four marks of the cut"
        );
    }

    #[test]
    fn the_register_states_one_kind_per_block() {
        assert_eq!(
            super::DATA_BLOCKS.len(),
            9,
            "the register holds nine blocks"
        );
        assert!(
            super::DATA_BLOCKS
                .iter()
                .any(|block| block.kind == Kind::Rust),
            "the constant block is a Rust block"
        );
    }
}
