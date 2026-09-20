//! Repository automation.
//!
//! `cargo xtask sync-agents [--check]` generates one Codex TOML agent under
//! `.codex/agents/` and one OpenCode Markdown agent under `.opencode/agents/`
//! for every canonical Claude agent under `.claude/agents/`. The Claude tree is
//! the source; the two mirrors are generated and must never be edited by hand.
//! `--check` reports drift and exits 1 without writing.

#![forbid(unsafe_code)]

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
        Command::SyncAgents { check } => sync_agents::run(&root, check),
    };
    match result {
        Ok(sync_agents::Outcome::Clean) => ExitCode::SUCCESS,
        Ok(sync_agents::Outcome::Drift) => ExitCode::from(1),
        Err(error) => report_failure(&error),
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
