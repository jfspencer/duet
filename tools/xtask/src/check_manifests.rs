//! Refuse a member manifest with no `[lints] workspace = true` line and no
//! non-empty `description`.
//!
//! `clippy::cargo_common_metadata` cannot fire under `publish = false`, so no
//! lint covers either condition. A member that omits the lint table sits
//! silently outside the whole policy, and a member that omits the description
//! gives a reader no statement of what it holds.

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::Context as _;
use serde_json::Value;

use crate::Outcome;

/// One condition the guard asserts over a member manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Condition {
    /// The manifest carries `[lints] workspace = true`.
    Lints,
    /// The manifest carries a non-empty `description`.
    Description,
}

impl Condition {
    /// The report text for a member that fails this condition.
    const fn report(self) -> &'static str {
        match self {
            Self::Lints => "no `[lints] workspace = true`",
            Self::Description => "no non-empty `description`",
        }
    }
}

/// Run the manifest guard over every member `cargo metadata --no-deps` reports.
///
/// # Errors
/// Returns an error when `cargo metadata` cannot run or when its output is not
/// the expected JSON.
pub(crate) fn run(root: &Path) -> anyhow::Result<Outcome> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    let manifests = match member_manifests(root)? {
        Ok(manifests) => manifests,
        Err(reason) => {
            writeln!(out, "FAIL: {reason}")?;
            return Ok(Outcome::FailClosed);
        },
    };
    let mut findings = 0_usize;
    for manifest in manifests {
        let Ok(text) = fs::read_to_string(&manifest) else {
            writeln!(out, "FAIL: cannot read {}", manifest.display())?;
            return Ok(Outcome::FailClosed);
        };
        let table = match toml::from_str::<toml::Table>(&text) {
            Ok(table) => table,
            Err(error) => {
                writeln!(out, "FAIL: cannot parse {}: {error}", manifest.display())?;
                return Ok(Outcome::FailClosed);
            },
        };
        let failed = failed_conditions(&table);
        if !failed.is_empty() {
            findings += 1;
            let reasons: Vec<&str> = failed.iter().map(|condition| condition.report()).collect();
            writeln!(
                out,
                "FINDING: {}: {}",
                manifest.display(),
                reasons.join(" and ")
            )?;
        }
    }
    Ok(if findings == 0 {
        Outcome::Clean
    } else {
        Outcome::Findings
    })
}

/// The conditions one member manifest fails, in report order.
fn failed_conditions(manifest: &toml::Table) -> Vec<Condition> {
    let mut failed = Vec::new();
    let lints = manifest
        .get("lints")
        .and_then(|lints| lints.get("workspace"))
        .and_then(toml::Value::as_bool);
    if lints != Some(true) {
        failed.push(Condition::Lints);
    }
    let description = manifest
        .get("package")
        .and_then(|package| package.get("description"))
        .and_then(toml::Value::as_str);
    if description.is_none_or(|text| text.trim().is_empty()) {
        failed.push(Condition::Description);
    }
    failed
}

/// The manifest path of every workspace member, in the order cargo reports it.
///
/// The inner `Err` holds the reason the guard cannot decide.
///
/// # Errors
/// Returns an error when `cargo metadata` cannot run.
fn member_manifests(root: &Path) -> anyhow::Result<Result<Vec<PathBuf>, String>> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .current_dir(root)
        .output()
        .context("cannot run `cargo metadata`")?;
    if !output.status.success() {
        return Ok(Err(format!(
            "`cargo metadata` exited with {}",
            output.status
        )));
    }
    let Ok(text) = String::from_utf8(output.stdout) else {
        return Ok(Err(
            "`cargo metadata` wrote output that is not UTF-8".to_owned()
        ));
    };
    let Ok(metadata) = serde_json::from_str::<Value>(&text) else {
        return Ok(Err(
            "`cargo metadata` wrote output that is not JSON".to_owned()
        ));
    };
    let Some(packages) = metadata.get("packages").and_then(Value::as_array) else {
        return Ok(Err("`cargo metadata` wrote no `packages` array".to_owned()));
    };
    let members: Vec<&str> = metadata
        .get("workspace_members")
        .and_then(Value::as_array)
        .map(|ids| ids.iter().filter_map(Value::as_str).collect())
        .unwrap_or_default();
    let mut manifests = Vec::new();
    for package in packages {
        let Some(id) = package.get("id").and_then(Value::as_str) else {
            return Ok(Err(
                "`cargo metadata` wrote a package with no `id`".to_owned()
            ));
        };
        if !members.contains(&id) {
            continue;
        }
        let Some(path) = package.get("manifest_path").and_then(Value::as_str) else {
            return Ok(Err(format!(
                "`cargo metadata` wrote no `manifest_path` for {id}"
            )));
        };
        manifests.push(PathBuf::from(path));
    }
    if manifests.is_empty() {
        return Ok(Err(
            "`cargo metadata` reported no workspace member".to_owned()
        ));
    }
    Ok(Ok(manifests))
}
