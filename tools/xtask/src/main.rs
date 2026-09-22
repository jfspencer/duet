//! Repository automation.
//!
//! `cargo xtask sync-agents [--check]` generates one Codex TOML agent under
//! `.codex/agents/` and one OpenCode Markdown agent under `.opencode/agents/`
//! for every canonical Claude agent under `.claude/agents/`. The Claude tree is
//! the source; the two mirrors are generated and must never be edited by hand.
//! `--check` reports drift and exits 1 without writing.
//!
//! `cargo xtask check-conversions`, `cargo xtask check-manifests`, and
//! `cargo xtask check-plan-graph <plan-dir>` are guards. Each one exits 0 when
//! it finds nothing, 1 when it finds at least one breach, and 2 when it cannot
//! decide.

#![forbid(unsafe_code)]

mod check_conversions;
mod check_manifests;
mod check_plan_graph;
mod sync_agents;

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::{Command as Process, ExitCode};

use clap::{Parser, Subcommand};

/// The automation entry point.
#[derive(Debug, Parser)]
#[command(name = "xtask", about, disable_help_subcommand = true)]
struct Cli {
    /// The task to run.
    #[command(subcommand)]
    command: Command,
}

/// Available tasks.
#[derive(Debug, Subcommand)]
enum Command {
    /// Regenerate the Codex and OpenCode agent mirrors from `.claude/agents`.
    SyncAgents {
        /// Report drift and exit 1 instead of writing files.
        #[arg(long)]
        check: bool,
    },
    /// Refuse a member manifest with no `[lints] workspace = true` line, or
    /// with no non-empty `description`.
    CheckManifests,
    /// Refuse a cast and a cast suppression outside the one conversion file.
    CheckConversions,
    /// Refuse a chunk file whose front-matter breaks a section 13 rule.
    CheckPlanGraph {
        /// The plan directory that holds the chunk files.
        plan_dir: PathBuf,
        /// Write `plan-graph.md` from the front-matter instead of a report
        /// alone.
        #[arg(long)]
        write_manifest: bool,
    },
}

/// What one guard run produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// No finding. The caller returns `ExitCode::SUCCESS`.
    Clean,
    /// At least one finding. The caller returns `ExitCode::from(1)`.
    Findings,
    /// The guard could not decide. The caller returns `ExitCode::from(2)`.
    FailClosed,
}

/// The repository root: two levels above this crate's manifest directory.
fn repo_root() -> Option<PathBuf> {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .map(Path::to_path_buf)
}

/// The workspace root that holds the current directory.
///
/// A guard reads the workspace it runs inside, which a probe builds in a
/// temporary directory, so the root cannot come from a path this binary
/// compiled in. The current directory is the answer when cargo names no
/// workspace root, which leaves the guard to print its own fail-closed line.
fn guard_root() -> Option<PathBuf> {
    let here = std::env::current_dir().ok()?;
    let named = cargo_metadata(&here)
        .and_then(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .and_then(|parsed| {
            parsed
                .get("workspace_root")
                .and_then(serde_json::Value::as_str)
                .map(PathBuf::from)
        });
    Some(named.unwrap_or(here))
}

/// The raw `cargo metadata --no-deps` document for one directory, or `None`.
///
/// The call carries no `--manifest-path`, so cargo walks up from the given
/// directory to the workspace root that holds it.
pub(crate) fn cargo_metadata(here: &Path) -> Option<String> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Process::new(cargo)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(here)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            let code = u8::try_from(error.exit_code()).unwrap_or(2);
            return if error.print().is_ok() {
                ExitCode::from(code)
            } else {
                ExitCode::from(2)
            };
        },
    };
    let result = match cli.command {
        Command::SyncAgents { check } => {
            let Some(root) = repo_root() else {
                return report_failure(&anyhow::anyhow!("cannot locate the repository root"));
            };
            sync_agents::run(&root, check).map(sync_outcome)
        },
        Command::CheckManifests => {
            let Some(root) = guard_root() else {
                return report_failure(&anyhow::anyhow!("cannot read the current directory"));
            };
            check_manifests::run(&root)
        },
        Command::CheckConversions => {
            let Some(root) = guard_root() else {
                return report_failure(&anyhow::anyhow!("cannot read the current directory"));
            };
            check_conversions::run(&root)
        },
        Command::CheckPlanGraph {
            plan_dir,
            write_manifest,
        } => check_plan_graph::run(&plan_dir, write_manifest),
    };
    match result {
        Ok(outcome) => exit_code(outcome),
        Err(error) => report_failure(&error),
    }
}

/// Map a `sync-agents` outcome onto the shared guard outcome.
const fn sync_outcome(outcome: sync_agents::Outcome) -> Outcome {
    match outcome {
        sync_agents::Outcome::Clean => Outcome::Clean,
        sync_agents::Outcome::Drift => Outcome::Findings,
    }
}

/// The exit code one outcome produces.
fn exit_code(outcome: Outcome) -> ExitCode {
    match outcome {
        Outcome::Clean => ExitCode::SUCCESS,
        Outcome::Findings => ExitCode::from(1),
        Outcome::FailClosed => ExitCode::from(2),
    }
}

/// Print an error to stderr and return the fail-closed exit code.
///
/// Every error that reaches this function names a condition the task could not
/// decide: an unreadable file, a file that is not UTF-8, an unreadable
/// directory, a missing cargo binary, or a closed output stream. Each one is
/// fail-closed, so the code is 2 and never the 1 a breach carries. A write
/// that itself fails leaves the plain failure code, because the reason can no
/// longer reach the operator.
fn report_failure(error: &anyhow::Error) -> ExitCode {
    let stderr = io::stderr();
    let mut err = stderr.lock();
    if writeln!(err, "FAIL: {error:#}").is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::from(2)
}
