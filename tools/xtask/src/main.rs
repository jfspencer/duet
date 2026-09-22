//! Repository automation.
//!
//! `cargo xtask sync-agents [--check]` generates one Codex TOML agent under
//! `.codex/agents/` and one OpenCode Markdown agent under `.opencode/agents/`
//! for every canonical Claude agent under `.claude/agents/`. The Claude tree is
//! the source; the two mirrors are generated and must never be edited by hand.
//! `--check` reports drift and exits 1 without writing.
//!
//! `cargo xtask check-manifests` and `cargo xtask check-plan-graph <plan-dir>`
//! are guards. Each one exits 0 when it finds nothing, 1 when it finds at least
//! one breach, and 2 when it cannot decide.

#![forbid(unsafe_code)]

mod check_manifests;
mod check_plan_graph;
mod sync_agents;

use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

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
    /// Refuse a member manifest with no `[lints] workspace = true` line and
    /// no non-empty `description`.
    CheckManifests,
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
    let Some(root) = repo_root() else {
        return report_failure(&anyhow::anyhow!("cannot locate the repository root"));
    };
    let result = match cli.command {
        Command::SyncAgents { check } => sync_agents::run(&root, check).map(sync_outcome),
        Command::CheckManifests => check_manifests::run(&root),
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

/// Print an error to stderr and return the failure exit code.
fn report_failure(error: &anyhow::Error) -> ExitCode {
    let stderr = io::stderr();
    let mut err = stderr.lock();
    if writeln!(err, "xtask: {error:#}").is_err() {
        return ExitCode::FAILURE;
    }
    ExitCode::FAILURE
}
