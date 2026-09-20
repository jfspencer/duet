//! `plan-db`: the single store tool every Hypervisor, Orchestrator, and
//! operating agent calls through `.claude/plan-coordination/db.sh <cmd> <plan-dir> ...`.
//!
//! # Engine
//!
//! LMDB through the `heed` binding. LMDB is a memory-mapped key/value store with
//! lock-free readers, a serialized-but-graceful single writer, and multi-PROCESS
//! locking built in. That is the whole reason it is here: one plan is operated
//! on by CONCURRENT orchestrators and hypervisors from several worktrees, and the
//! store must be reachable from every one of them. There are NO external lock
//! files; LMDB's own environment lock is relied on.
//!
//! # Location
//!
//! GLOBAL and plan-keyed, never inside a worktree:
//! `~/.claude/plan-dbs/<plan-key>/`. The plan key is derived deterministically
//! (see [`plan_key`]) so every actor resolves the SAME environment from any
//! worktree. `PLAN_DB_ROOT` overrides the parent directory for tests.
//!
//! # Key classes (mirrored exactly in `memory-agent.md`)
//!
//! * FIXED well-known keys (`current_context`, `control:signal`,
//!   `orch:<id>:status`, `orch:<id>:heartbeat`, `pacing:current`) are addressed
//!   directly with `get` / `put` / `del`.
//! * DYNAMIC entries (reports, findings, results, checkpoints, work-item events,
//!   fluid notes) are minted by `append` as `<simpleflake>-<suffix>`: unique
//!   without cross-writer coordination, short, and time-ordered.
//!
//! This is a CLI: writing results to stdout IS its contract.

use std::hash::{BuildHasher as _, RandomState};
use std::io::{self, Write as _};
use std::path::{Component, Path, PathBuf};
use std::process::{Command as Process, ExitCode, Stdio};
use std::time::{SystemTime, SystemTimeError, UNIX_EPOCH};
use std::{env, fs};

use clap::{Parser, Subcommand};
use heed::types::Str;
use heed::{Database, Env, EnvOpenOptions};
use serde::Serialize;

/// Virtual size of the memory map. LMDB reserves address space, not disk, so a
/// large value costs nothing until data is written.
const MAP_SIZE_BYTES: usize = 8 * 1024 * 1024 * 1024;

/// How many fresh flakes `append` mints before it gives up on a suffix.
const APPEND_RETRIES: usize = 8;

/// The seed value of `current_context` on a fresh store.
const FRESH_CONTEXT: &str = r#"{"resume_mode":"fresh","seq":0}"#;

/// The plan store CLI.
#[derive(Debug, Parser)]
#[command(name = "plan-db", version, about, disable_help_subcommand = true)]
struct Cli {
    /// The subcommand to run.
    #[command(subcommand)]
    command: Command,
}

/// Every subcommand takes the plan directory FIRST, as a repo-root-relative
/// roadmap path such as `roadmap/<plan>`.
#[derive(Debug, Subcommand)]
enum Command {
    /// Print the canonical global env path for a plan dir.
    Resolve {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
    },
    /// Open or create the env; seed `current_context` and `control:signal=run` if absent.
    Init {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
    },
    /// Print the raw value at a key (nothing if absent).
    Get {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
        /// The key to read.
        key: String,
    },
    /// Durably write a FIXED key.
    Put {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
        /// The key to write.
        key: String,
        /// The value to store.
        value: String,
    },
    /// Delete a key.
    Del {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
        /// The key to delete.
        key: String,
    },
    /// Print the keys with a prefix; `-v` emits one `{"key","value"}` JSON record per line.
    Scan {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
        /// The key prefix to match (empty matches every key).
        prefix: String,
        /// Emit values as compact JSON records.
        #[arg(short = 'v', long = "values")]
        values: bool,
    },
    /// Print `{key, bytes, approx_tokens}` for one key (NO value).
    Len {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
        /// The key to size.
        key: String,
    },
    /// Mint `<simpleflake>-<suffix>`, durably write, and print the key.
    Append {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
        /// A few words of context, sanitized to `[a-z0-9-]`.
        suffix: String,
        /// The value to store.
        value: String,
    },
    /// Print `{key, bytes, approx_tokens}` for ALL keys (optional prefix); NO values.
    Keys {
        /// Repo-root-relative plan directory (`roadmap/<plan>`).
        plan_dir: String,
        /// Optional key prefix.
        prefix: Option<String>,
    },
}

/// Every way a subcommand can fail.
#[derive(Debug, thiserror::Error)]
enum Error {
    /// The plan dir resolved to the repo root itself.
    #[error(
        "plan-dir {plan_dir:?} resolves to the repo ROOT itself ({root}); pass a repo-root-relative roadmap path like \"roadmap/<plan>\", never \".\" or the repo root"
    )]
    PlanDirIsRoot {
        /// The plan dir as given.
        plan_dir: String,
        /// The repo root.
        root: String,
    },
    /// The plan dir resolved outside the repo root.
    #[error(
        "plan-dir {plan_dir:?} (resolved {resolved}) is OUTSIDE the repo root {root}; pass a repo-root-relative roadmap path like \"roadmap/<plan>\""
    )]
    PlanDirOutside {
        /// The plan dir as given.
        plan_dir: String,
        /// Where it resolved to.
        resolved: String,
        /// The repo root.
        root: String,
    },
    /// A caller-supplied key or prefix left the ASCII key alphabet.
    #[error(
        "{label} {key:?} contains characters outside the key alphabet [A-Za-z0-9._:-]; keys must be ASCII so prefix scans stay correct"
    )]
    KeyAlphabet {
        /// Which argument failed.
        label: &'static str,
        /// The offending key.
        key: String,
    },
    /// `append` could not find a free key.
    #[error("append: could not mint a unique key for suffix {0:?} after retries")]
    AppendCollision(String),
    /// `HOME` is unset and `PLAN_DB_ROOT` is not given.
    #[error("neither PLAN_DB_ROOT nor HOME is set; cannot locate ~/.claude/plan-dbs")]
    NoHome,
    /// The LMDB layer failed.
    #[error("store: {0}")]
    Store(#[from] heed::Error),
    /// A filesystem or stdout failure.
    #[error("io: {0}")]
    Io(#[from] io::Error),
    /// JSON framing failed (cannot happen for string fields; kept honest).
    #[error("json: {0}")]
    Json(#[from] serde_json::Error),
    /// The system clock is before the Unix epoch.
    #[error("clock: {0}")]
    Clock(#[from] SystemTimeError),
}

/// The size annotation row `keys` and `len` emit. Never carries a value.
#[derive(Debug, Serialize)]
struct SizeRow<'a> {
    /// The key.
    key: &'a str,
    /// UTF-8 byte length of the value (0 when absent).
    bytes: usize,
    /// `ceil(bytes / 4)`: a cheap token estimate.
    approx_tokens: usize,
}

impl<'a> SizeRow<'a> {
    /// Build the row for a key and its optional value.
    fn new(key: &'a str, value: Option<&str>) -> Self {
        let bytes = value.map_or(0, str::len);
        Self {
            key,
            bytes,
            approx_tokens: bytes.div_ceil(4),
        }
    }
}

/// The record `scan -v` emits, one per line, so a multi-line value never
/// splits across output lines.
#[derive(Debug, Serialize)]
struct KeyValue<'a> {
    /// The key.
    key: &'a str,
    /// The raw value.
    value: &'a str,
}

/// Replace everything outside `[A-Za-z0-9._-]` with `-`, collapse runs, trim ends.
fn sanitize(input: &str) -> String {
    collapse_dashes(input.chars().map(|c| {
        if c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-') {
            c
        } else {
            '-'
        }
    }))
}

/// Lower-case and restrict to `[a-z0-9-]`, collapse runs, trim ends.
fn sanitize_suffix(input: &str) -> String {
    collapse_dashes(input.to_lowercase().chars().map(|c| {
        if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
            c
        } else {
            '-'
        }
    }))
}

/// Collapse runs of `-` and strip leading/trailing `-`.
fn collapse_dashes(chars: impl Iterator<Item = char>) -> String {
    let mut out = String::new();
    for c in chars {
        if c == '-' && out.ends_with('-') {
            continue;
        }
        out.push(c);
    }
    out.trim_matches('-').to_owned()
}

/// Reject a key or prefix that leaves the documented ASCII alphabet
/// `[A-Za-z0-9._:-]`. A code point above it would sort outside a prefix range
/// and be silently omitted from scans.
fn validate_key(label: &'static str, key: &str) -> Result<(), Error> {
    let ok = key
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-'));
    if ok {
        Ok(())
    } else {
        Err(Error::KeyAlphabet {
            label,
            key: key.to_owned(),
        })
    }
}

/// Lexically normalize a path: drop `.` segments and resolve `..` against the
/// preceding segment. Never touches the filesystem, so it agrees across
/// worktrees exactly like `path.resolve` does.
fn normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {},
            Component::ParentDir => {
                let popped = out.pop();
                if !popped {
                    out.push("..");
                }
            },
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                out.push(component.as_os_str());
            },
        }
    }
    out
}

/// The path as a string for messages.
fn display(path: &Path) -> String {
    path.display().to_string()
}

/// The stable project root shared by every worktree: the PARENT of
/// `git rev-parse --git-common-dir`. A linked worktree's `--show-toplevel`
/// differs per worktree; the common dir always points at the main checkout's
/// `.git`. `None` when the cwd is not inside a git repository.
fn repo_root() -> Option<PathBuf> {
    let output = Process::new("git")
        .args(["rev-parse", "--path-format=absolute", "--git-common-dir"])
        .stdin(Stdio::null())
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let common = String::from_utf8(output.stdout).ok()?;
    let common = normalize(Path::new(common.trim()));
    common.parent().map(Path::to_path_buf)
}

/// A file name as a sanitized string, or a fallback when the path has none.
fn leaf(path: &Path, fallback: &str) -> String {
    let name = path
        .file_name()
        .map(|n| sanitize(&n.to_string_lossy()))
        .unwrap_or_default();
    if name.is_empty() {
        fallback.to_owned()
    } else {
        name
    }
}

/// The deterministic plan key all actors agree on for a plan dir:
/// `<repo-basename>__<repo-relative-slug>`, where the slug is the plan dir's
/// path RELATIVE to the repo root with separators flattened to `-`
/// (`roadmap/alpha` -> `roadmap-alpha`). The FULL relative path is used so two
/// plans sharing a leaf name never collide onto one store.
///
/// A relative plan dir is interpreted relative to the REPO ROOT, not the cwd, so
/// `roadmap/<plan>` resolves to the same store from the main checkout and from
/// any worktree. The repo root itself and anything outside it are rejected.
fn plan_key(plan_dir: &str) -> Result<String, Error> {
    let Some(root) = repo_root() else {
        // Not a git repo (throwaway smoke dir): key on the plan dir's own parent
        // basename + leaf so the key is still deterministic for that path.
        let abs = normalize(&env::current_dir()?.join(plan_dir));
        let repo = abs
            .parent()
            .map_or_else(|| "repo".to_owned(), |p| leaf(p, "repo"));
        return Ok(format!("{repo}__{}", leaf(&abs, "plan")));
    };
    let abs = normalize(&root.join(plan_dir));
    let repo = leaf(&root, "repo");
    let Ok(rel) = abs.strip_prefix(&root) else {
        return Err(Error::PlanDirOutside {
            plan_dir: plan_dir.to_owned(),
            resolved: display(&abs),
            root: display(&root),
        });
    };
    if rel.as_os_str().is_empty() {
        return Err(Error::PlanDirIsRoot {
            plan_dir: plan_dir.to_owned(),
            root: display(&root),
        });
    }
    let joined = rel
        .components()
        .map(|c| c.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("-");
    let slug = sanitize(&joined);
    let slug = if slug.is_empty() {
        "plan".to_owned()
    } else {
        slug
    };
    Ok(format!("{repo}__{slug}"))
}

/// The parent directory of every plan env: `$PLAN_DB_ROOT`, else `~/.claude/plan-dbs`.
fn global_root() -> Result<PathBuf, Error> {
    if let Some(root) = env::var_os("PLAN_DB_ROOT") {
        return Ok(PathBuf::from(root));
    }
    let home = env::var_os("HOME").ok_or(Error::NoHome)?;
    Ok(PathBuf::from(home).join(".claude").join("plan-dbs"))
}

/// The canonical GLOBAL env directory for a plan dir.
fn env_path(plan_dir: &str) -> Result<PathBuf, Error> {
    Ok(global_root()?.join(plan_key(plan_dir)?))
}

/// Open the LMDB environment at `path`.
#[expect(
    unsafe_code,
    reason = "heed marks Env::open unsafe because opening one path twice in one process is undefined behaviour; this binary opens exactly one environment per process"
)]
fn open_lmdb(path: &Path) -> Result<Env, heed::Error> {
    let mut options = EnvOpenOptions::new();
    options.map_size(MAP_SIZE_BYTES).max_dbs(1);
    // SAFETY: `open_env` is the only caller and `main` runs it once, so this
    // process never holds two environments on the same path.
    unsafe { options.open(path) }
}

/// Open (creating if needed) the env and its unnamed database for a plan dir.
fn open_env(plan_dir: &str) -> Result<(Env, Database<Str, Str>), Error> {
    let path = env_path(plan_dir)?;
    fs::create_dir_all(&path)?;
    let env = open_lmdb(&path)?;
    let mut wtxn = env.write_txn()?;
    let db = env.create_database::<Str, Str>(&mut wtxn, None)?;
    wtxn.commit()?;
    Ok((env, db))
}

/// A borrowed walk over `(key, value)` pairs in key order.
type Entries<'txn> = Box<dyn Iterator<Item = heed::Result<(&'txn str, &'txn str)>> + 'txn>;

/// Every `(key, value)` pair whose key starts with `prefix`, in key order.
///
/// LMDB's prefix cursor needs a non-empty prefix; an empty prefix means the
/// whole keyspace, so that case walks the full range instead.
fn entries<'txn>(
    db: Database<Str, Str>,
    rtxn: &'txn heed::RoTxn<'_>,
    prefix: &'txn str,
) -> Result<Entries<'txn>, heed::Error> {
    if prefix.is_empty() {
        Ok(Box::new(db.iter(rtxn)?))
    } else {
        Ok(Box::new(db.prefix_iter(rtxn, prefix)?))
    }
}

/// Render `n` in base 36 with lower-case digits, like JavaScript's `toString(36)`.
fn to_base36(mut n: u128) -> String {
    if n == 0 {
        return "0".to_owned();
    }
    let mut digits = Vec::new();
    while n > 0 {
        let digit = u32::try_from(n % 36).unwrap_or(0);
        digits.push(char::from_digit(digit, 36).unwrap_or('0'));
        n /= 36;
    }
    digits.iter().rev().collect()
}

/// A time-ordered id with no shared counter: `(now_ms << 23) | 23 random bits`,
/// base-36 encoded. Same-millisecond collisions are bounded, not impossible, so
/// `append` re-mints on a clash instead of trusting the bound.
fn simpleflake() -> Result<String, Error> {
    let now_ms = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let random = RandomState::new().hash_one(now_ms) & ((1_u64 << 23) - 1);
    Ok(to_base36((now_ms << 23) | u128::from(random)))
}

/// Write one line to stdout.
fn emit(out: &mut impl io::Write, line: &str) -> Result<(), Error> {
    writeln!(out, "{line}")?;
    Ok(())
}

/// Run one subcommand against the store.
fn run(command: Command, out: &mut impl io::Write) -> Result<(), Error> {
    match command {
        Command::Resolve { plan_dir } => emit(out, &display(&env_path(&plan_dir)?)),
        Command::Init { plan_dir } => {
            let (env, db) = open_env(&plan_dir)?;
            let mut wtxn = env.write_txn()?;
            if db.get(&wtxn, "current_context")?.is_none() {
                db.put(&mut wtxn, "current_context", FRESH_CONTEXT)?;
            }
            if db.get(&wtxn, "control:signal")?.is_none() {
                db.put(&mut wtxn, "control:signal", "run")?;
            }
            wtxn.commit()?;
            let line = format!(
                "init: {} (current_context + control:signal ready)",
                display(&env_path(&plan_dir)?)
            );
            emit(out, &line)
        },
        Command::Get { plan_dir, key } => {
            validate_key("get key", &key)?;
            let (env, db) = open_env(&plan_dir)?;
            let rtxn = env.read_txn()?;
            if let Some(value) = db.get(&rtxn, &key)? {
                emit(out, value)?;
            }
            Ok(())
        },
        Command::Put {
            plan_dir,
            key,
            value,
        } => {
            validate_key("put key", &key)?;
            let (env, db) = open_env(&plan_dir)?;
            let mut wtxn = env.write_txn()?;
            db.put(&mut wtxn, &key, &value)?;
            wtxn.commit()?;
            Ok(())
        },
        Command::Del { plan_dir, key } => {
            validate_key("del key", &key)?;
            let (env, db) = open_env(&plan_dir)?;
            let mut wtxn = env.write_txn()?;
            db.delete(&mut wtxn, &key)?;
            wtxn.commit()?;
            Ok(())
        },
        Command::Scan {
            plan_dir,
            prefix,
            values,
        } => {
            validate_key("scan prefix", &prefix)?;
            let (env, db) = open_env(&plan_dir)?;
            let rtxn = env.read_txn()?;
            for entry in entries(db, &rtxn, &prefix)? {
                let (key, value) = entry?;
                if values {
                    emit(out, &serde_json::to_string(&KeyValue { key, value })?)?;
                } else {
                    emit(out, key)?;
                }
            }
            Ok(())
        },
        Command::Len { plan_dir, key } => {
            validate_key("len key", &key)?;
            let (env, db) = open_env(&plan_dir)?;
            let rtxn = env.read_txn()?;
            let value = db.get(&rtxn, &key)?;
            emit(out, &serde_json::to_string(&SizeRow::new(&key, value))?)
        },
        Command::Append {
            plan_dir,
            suffix,
            value,
        } => {
            let safe_suffix = sanitize_suffix(&suffix);
            let (env, db) = open_env(&plan_dir)?;
            let mut wtxn = env.write_txn()?;
            let mut minted = None;
            for _ in 0..APPEND_RETRIES {
                let candidate = format!("{}-{safe_suffix}", simpleflake()?);
                if db.get(&wtxn, &candidate)?.is_none() {
                    minted = Some(candidate);
                    break;
                }
            }
            let key = minted.ok_or(Error::AppendCollision(safe_suffix))?;
            db.put(&mut wtxn, &key, &value)?;
            wtxn.commit()?;
            emit(out, &key)
        },
        Command::Keys { plan_dir, prefix } => {
            let (env, db) = open_env(&plan_dir)?;
            let rtxn = env.read_txn()?;
            let prefix = prefix.unwrap_or_default();
            validate_key("keys prefix", &prefix)?;
            for entry in entries(db, &rtxn, &prefix)? {
                let (key, value) = entry?;
                emit(
                    out,
                    &serde_json::to_string(&SizeRow::new(key, Some(value)))?,
                )?;
            }
            Ok(())
        },
    }
}

/// Map a clap exit code onto `ExitCode`.
fn exit_code(code: i32) -> ExitCode {
    u8::try_from(code).map_or(ExitCode::FAILURE, ExitCode::from)
}

fn main() -> ExitCode {
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(error) => {
            // clap prints usage/help itself; a failure to print is not recoverable.
            return if error.print().is_ok() {
                exit_code(error.exit_code())
            } else {
                ExitCode::from(2)
            };
        },
    };
    let stdout = io::stdout();
    let mut out = stdout.lock();
    match run(cli.command, &mut out) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            let stderr = io::stderr();
            let mut err = stderr.lock();
            if writeln!(err, "plan-db: {error}").is_err() {
                return ExitCode::from(1);
            }
            ExitCode::from(1)
        },
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{normalize, sanitize, sanitize_suffix, to_base36, validate_key};

    #[test]
    fn sanitize_keeps_the_documented_alphabet() {
        assert_eq!(
            sanitize("roadmap/alpha beta"),
            "roadmap-alpha-beta",
            "separators become dashes"
        );
        assert_eq!(
            sanitize("--a..b_c--"),
            "a..b_c",
            "runs collapse and ends trim"
        );
        assert_eq!(sanitize(""), "", "empty stays empty");
    }

    #[test]
    fn suffix_is_lowercase_words() {
        assert_eq!(
            sanitize_suffix("Orch12 Report!"),
            "orch12-report",
            "lower-case and dashes"
        );
        assert_eq!(
            sanitize_suffix("finding_auth"),
            "finding-auth",
            "underscore is not in the suffix alphabet"
        );
    }

    #[test]
    fn keys_must_be_ascii() {
        assert!(
            validate_key("k", "work_item:12.a-b").is_ok(),
            "documented alphabet passes"
        );
        assert!(validate_key("k", "emoji-🧠").is_err(), "non-ASCII fails");
        assert!(validate_key("k", "has space").is_err(), "space fails");
    }

    #[test]
    fn base36_matches_javascript() {
        assert_eq!(to_base36(0), "0", "zero");
        assert_eq!(to_base36(35), "z", "last digit");
        assert_eq!(to_base36(36), "10", "carry");
        assert_eq!(
            to_base36(1_700_000_000_000),
            "loyw3v28",
            "a millisecond timestamp"
        );
    }

    #[test]
    fn normalize_is_lexical() {
        assert_eq!(
            normalize(Path::new("/a/b/../c/./d")),
            Path::new("/a/c/d"),
            "dot and dot-dot"
        );
        assert_eq!(normalize(Path::new("/a/..")), Path::new("/"), "pop to root");
    }
}
