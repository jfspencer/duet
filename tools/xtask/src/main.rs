//! Repository automation.
//!
//! `cargo xtask sync-agents [--check]` generates one Codex TOML agent under
//! `.codex/agents/` and one OpenCode Markdown agent under `.opencode/agents/`
//! for every canonical Claude agent under `.claude/agents/`. The Claude tree is
//! the source; the two mirrors are generated and must never be edited by hand.
//! `--check` reports drift and exits 1 without writing.
//!
//! `cargo xtask check-conversions [--appendix <path>]`,
//! `cargo xtask check-manifests`,
//! `cargo xtask check-plan-graph <plan-dir>`, and
//! `cargo xtask check-closure <document> <review> <block>`, and
//! `cargo xtask check-roster <document> <scratch> <repo> [--generate-only]` are
//! guards. Each one exits 0 when it finds nothing, 1 when it finds at least one
//! breach, and 2 when it cannot decide. `--appendix` turns on rule `CG9`, which
//! binds each Appendix B.1 reason cell of the named document to the `reason =`
//! string of the one exempt conversion file; the guard skips that rule and
//! prints the skip when the caller names no document.
//!
//! Three rules find a root, and each task takes exactly one of them. A task
//! that rewrites this repository takes [`repo_root`], the directory two levels
//! above the compiled-in `CARGO_MANIFEST_DIR`; `sync-agents` is that task and
//! the only one. A guard that reads the workspace it runs inside takes
//! [`guard_root`], the workspace root `cargo metadata` names from the current
//! directory; `check-manifests` and `check-conversions` are those guards. A
//! guard that reads a named document or a named directory takes the path the
//! operator states on the command line; `check-plan-graph`, `check-closure`,
//! `check-roster`, and `check-placement` are those guards.
//!
//! A guard that a probe must reach never takes the compiled-in root. A probe
//! builds a throwaway workspace in a temporary directory and spawns the guard
//! over it, so a guard that took the compiled-in root would read this
//! repository and report on the wrong tree. `check-closure` reads
//! `crate::repo_root` inside one helper, and for one file alone: the plan-store
//! launcher `.claude/plan-coordination/db.sh`, which belongs to this repository
//! and never to the tree under review.

#![forbid(unsafe_code)]

mod check_closure;
mod check_conversions;
mod check_manifests;
mod check_placement;
mod check_plan_graph;
mod check_roster;
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
    CheckConversions {
        /// The architecture document whose Appendix B.1 reason cells bind the
        /// `reason =` strings of the one exempt file (`CG9`).
        #[arg(long)]
        appendix: Option<PathBuf>,
    },
    /// Refuse a closure block that its own review file does not support.
    CheckClosure {
        /// The architecture document to read.
        document: PathBuf,
        /// The review file the block closes.
        review: PathBuf,
        /// The closure block identifier.
        block: String,
    },
    /// Compile the section 15 roster in a scratch workspace outside the
    /// repository.
    CheckRoster {
        /// The architecture document to read.
        document: PathBuf,
        /// A scratch directory outside the repository.
        scratch: PathBuf,
        /// The repository root.
        repo: PathBuf,
        /// Write the scratch workspace and skip every cargo command.
        #[arg(long)]
        generate_only: bool,
    },
    /// Refuse a declared type that the section 1.5 table does not place.
    CheckPlacement {
        /// The architecture document to read.
        document: PathBuf,
    },
    /// Refuse a chunk file whose front-matter breaks a section 13 rule.
    CheckPlanGraph {
        /// The plan directory that holds the chunk files.
        plan_dir: PathBuf,
        /// Write `plan-graph.md` from the front-matter instead of a report
        /// alone.
        #[arg(long)]
        write_manifest: bool,
        /// Report `plan-graph.md` as a finding when the front-matter no longer
        /// generates it.
        #[arg(long)]
        check_manifest: bool,
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
        Command::CheckConversions { appendix } => {
            let Some(root) = guard_root() else {
                return report_failure(&anyhow::anyhow!("cannot read the current directory"));
            };
            check_conversions::run(&root, appendix.as_deref())
        },
        Command::CheckClosure {
            document,
            review,
            block,
        } => check_closure::run(&document, &review, &block),
        Command::CheckRoster {
            document,
            scratch,
            repo,
            generate_only,
        } => check_roster::run(&document, &scratch, &repo, generate_only),
        Command::CheckPlacement { document } => check_placement::run(&document),
        Command::CheckPlanGraph {
            plan_dir,
            write_manifest,
            check_manifest,
        } => {
            let Some(mode) = check_plan_graph::ManifestMode::select(write_manifest, check_manifest)
            else {
                return report_failure(&anyhow::anyhow!(
                    "give --write-manifest or --check-manifest, not both"
                ));
            };
            check_plan_graph::run(&plan_dir, mode)
        },
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

/// Print an error to stdout and return the fail-closed exit code.
///
/// Every error that reaches this function names a condition the task could not
/// decide: an unreadable file, a file that is not UTF-8, an unreadable
/// directory, a missing cargo binary, or a closed output stream. Each one is
/// fail-closed, so the code is 2 and never the 1 a breach carries. The code
/// stays 2 when the write of the `FAIL:` line itself fails, because a stream
/// that refuses the fail-closed line is the most fail-closed state of all, and
/// 1 would tell the operator that the input broke a rule.
///
/// The line goes to stdout, where every guard prints its own `FAIL:` line, so
/// one redirection carries the whole report.
fn report_failure(error: &anyhow::Error) -> ExitCode {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let _unwritten = writeln!(out, "FAIL: {error:#}");
    ExitCode::from(2)
}
