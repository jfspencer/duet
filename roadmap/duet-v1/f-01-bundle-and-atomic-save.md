---
id: F1
line: F
depends_on: [T4, M4]
write_scope:
  - crates/duet-project/Cargo.toml
  - crates/duet-project/src/lib.rs
  - crates/duet-project/src/bundle.rs
  - crates/duet-project/src/save.rs
  - crates/duet-project/src/view.rs
  - crates/duet-project/src/gitconfig.rs
  - crates/duet-project/src/store.rs
  - crates/duet-project/src/history.rs
  - crates/duet-project/src/checkout.rs
  - crates/duet-project/src/headwatch.rs
  - crates/duet-project/src/watch.rs
  - crates/duet-project/src/lock.rs
  - crates/duet-project/src/templates.rs
  - crates/duet-project/src/recent.rs
  - crates/duet-project/src/platform_macos.rs
  - crates/duet-project/src/platform_linux.rs
  - crates/duet-project/src/store/manifest.rs
  - crates/duet-project/src/store/content.rs
  - crates/duet-project/src/store/gc.rs
  - crates/duet-project/src/history/gix_impl.rs
  - crates/duet-project/src/history/budget.rs
  - crates/duet-project/src/watch/debounce.rs
  - crates/duet-project/src/watch/ledger.rs
  - crates/duet-project/src/templates/default.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-project -E 'test(atomic_save) + test(view_json)' --no-tests=fail passes; commit SHA on a branch chunk/f1-bundle-and-atomic-save"
---

# F1: Bundle create, the five-step atomic save, `view.json`, and the templates

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the first layer of `duet-project`: bundle creation with the directory layout of architecture section 4.1, the tracked set of section 4.2, the five-step atomic save and the crash matrix of section 4.10, `state/view.json` through `BundleDocument` (sections 1.8 and 10.2), the repository configuration of section 4.7, and the two templates of PR 11 Q5. It also creates every module file of line F as a stub (SM2). ADR 0003 clauses 1, 1a, 10, 13 and 13a decide the tracked set, the view state, the object format, and the two-phase marker. It carries MUST stories A-03 (text-readable score), C-01 (the templates half of the start surface), C-02 (create a project), C-03 (open a project), C-04 (save and dirty state), and X-08 (context and layout persistence). The crate skeleton (`Cargo.toml` package fields, `[lints] workspace = true`, `src/lib.rs` with `#![forbid(unsafe_code)]`) already exists, because chunk M4 created it.

`duet-project` declares no document. `duet-command` declares `BundleDocument` and implements it for `Score`, `Session`, `MixState` and `ViewState`.

## Files

- `crates/duet-project/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).
- `crates/duet-project/src/lib.rs` — modify. Add every `mod` line of the line.
- `crates/duet-project/src/bundle.rs` — create.
- `crates/duet-project/src/save.rs` — create.
- `crates/duet-project/src/view.rs` — create.
- `crates/duet-project/src/gitconfig.rs` — create.
- `crates/duet-project/src/templates.rs` — create.
- `crates/duet-project/src/templates/default.rs` — create.
- `crates/duet-project/src/store.rs` — create as a stub.
- `crates/duet-project/src/store/manifest.rs` — create as a stub.
- `crates/duet-project/src/store/content.rs` — create as a stub.
- `crates/duet-project/src/store/gc.rs` — create as a stub.
- `crates/duet-project/src/history.rs` — create as a stub.
- `crates/duet-project/src/history/gix_impl.rs` — create as a stub.
- `crates/duet-project/src/history/budget.rs` — create as a stub.
- `crates/duet-project/src/checkout.rs` — create as a stub.
- `crates/duet-project/src/headwatch.rs` — create as a stub.
- `crates/duet-project/src/watch.rs` — create as a stub.
- `crates/duet-project/src/watch/debounce.rs` — create as a stub.
- `crates/duet-project/src/watch/ledger.rs` — create as a stub.
- `crates/duet-project/src/lock.rs` — create as a stub.
- `crates/duet-project/src/recent.rs` — create as a stub.
- `crates/duet-project/src/platform_macos.rs` — create, with content.
- `crates/duet-project/src/platform_linux.rs` — create, with content.

A stub holds the `//!` module documentation and nothing else (SM2). A later chunk of line F modifies a stub and creates no file.

**The two platform modules are not stubs, and this chunk states why.** Section 11.1 gives `duet-project` two platform facilities: the runtime socket directory and the process liveness check of section 3.8. No later chunk of line F names either file in its Writes column, and SM2 lets no chunk create a file outside its own write scope, so this chunk writes both modules in full. Chunk F5 then consumes the liveness check through the trait this chunk declares.

## Types and signatures

### The bundle layout this chunk creates (architecture 4.1)

```text
<name>.duet/
├── .git/                      gix repository; text only
├── .gitattributes             written at creation
├── .gitignore                 written at creation
├── duet.toml                  bundle identity and schema number
├── score/{meta.json,notes.jsonl,spanners.jsonl}
├── session/{session.json,takes.jsonl,regions.jsonl,locations.jsonl}
├── mix/{strips.json,automation.jsonl}
├── media.manifest
├── media/{<hh>/<hash>.wav,incoming/,dead/}
├── derived/{peaks/<hash>.peaks,analysis/<hash>.pitch}
├── state/{save.commit,view.json,duet.lock,agent.sock.path,.tmp/<job>/}
└── exports/.tmp/<job>/
```

Tracked: `duet.toml`, `score/`, `session/`, `mix/`, `media.manifest`, `.gitattributes`, `.gitignore`. Not tracked: `media/`, `derived/`, `state/`, `exports/`. `.gitignore` covers `media/`, `derived/`, `state/`, and `exports/`.

`derived/` is a cache in full; a user may delete it and every file rebuilds. `state/` is **not** a cache: it holds what belongs to this machine and this run.

### `duet_project::gitconfig` (architecture 4.7)

The bundle writes these two files at creation, byte for byte:

```text
# .gitattributes
* -text
*.json   text eol=lf
*.jsonl  text eol=lf
*.toml   text eol=lf
media.manifest text eol=lf
```

```text
# repository-local config
core.autocrlf=false
core.safecrlf=false
```

**Bundle creation writes no `extensions.objectFormat` key.** That key is a repository-format-version-1 extension, and at format version 0 plain `git` refuses to open a repository that carries one. `git init` writes no such key, SHA-1 is the default, and a version-0 repository states it by omission. `duet.toml` records the value as a mirror a reader can see without a git tool, and it decides nothing.

### `duet_project::save` (architecture 4.10)

The five steps, with every `fsync` named:

1. Write every temporary file. `fsync` each file.
2. `fsync` every directory that received a temporary file.
3. Write `state/save.commit`, which lists each temporary name and its final name. `fsync` the marker file, then `fsync` `state/`.
4. Rename each file. `fsync` every directory that a rename touched.
5. Delete `state/save.commit`. `fsync` `state/`.

The crash matrix, which the recovery at the next start implements:

| Crash point | State on disk | Recovery at the next start |
|---|---|---|
| During step 1 or step 2, before the marker | Temporary files, no marker | The old state is intact and durable. The start deletes every temporary file whose name matches the save pattern. The unsaved edit is gone. |
| During step 3, marker partly written | A marker that does not parse | The start deletes the marker and every temporary file. The old state stands, and the application tells the user that the last save did not finish. |
| After step 3, during step 4 | A valid marker, some files renamed | The start reads the marker and completes every rename whose temporary file still exists. The result is the new state. |
| After step 4, before step 5 | A valid marker, every file renamed, every directory synced | The start reads the marker, finds no temporary file, and deletes the marker. The result is the new state. |
| After step 5 | No marker | Nothing to do. |

**The marker lives in `state/`, not in `derived/`.** `.gitignore` covers `state/`, so `git clean -xdf` deletes the marker and the temporary files together and the bundle returns to the old state, which is the answer the recovery gives.

### `duet_project::view` (architecture 4.1, 9.4, 10.2)

`state/view.json` holds a `ViewDocument`, which `duet-command` declares:

```rust
// in duet-command
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewDocument { schema: SchemaVersion, view: ViewState }
```

`state/view.json` carries `VIEW_SCHEMA` (B84). `ViewState::tracked()` returns false, so git does not track the file and the save path reads the `BundleDocument::tracked` method rather than matching a path at four call sites. The file watcher of chunk F5 does not watch it: this process is its one writer, and `Verb::ViewSet` is the one path that changes it.

### `duet_project` crate root (architecture 15.12)

```rust
/// Every way the project refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProjectError {
    Open(Box<str>),
    Io(Box<str>),
    LockLost,
    HashReachable { hash: SourceHash },
    /// The repository uses an object format this build does not read.
    ObjectFormat(Box<str>),
    History(HistoryError),
    Command(CommandError),
    Media(MediaError),
}

/// Every way the history refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum HistoryError {
    Read(Box<str>),
    Write(Box<str>),
    MissingRef(BranchName),
    MissingCommit(CommitId),
    Timeout,
}
```

`ProjectError` wraps `HistoryError` with `#[from]` (section 12.1). `duet-project` implements `From<ProjectError> for UpstreamFailure` at its own boundary, with `FailureSurface::Project` and the matching `FailureCode`, which the orphan rule allows because the source type is local here.

### The trait this chunk consumes (architecture 1.8)

```rust
// in duet-command
pub trait BundleDocument: Sized {
    fn paths() -> &'static [&'static str];
    fn to_bytes(&self) -> Result<Vec<Vec<u8>>, CommandError>;
    fn from_bytes(blocks: &[Vec<u8>]) -> Result<Self, CommandError>;
    fn warnings(&self) -> &[ImportWarning];
    fn tracked() -> bool;
}
```

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `BundleDocument`, `ViewDocument`, `ViewState`, `CommandError`, `ImportWarning`, `SchemaVersion`, `BranchName`, `CommitId`, `CreateRequest` | `duet-command` | `duet_command` |
| `SourceHash`, `AudioContainer`, `Session`, `MixState` | `duet-session` | `duet_session` |
| `MediaError` | `duet-media` | `duet_media` |
| `VIEW_SCHEMA` (B84), `SESSION_SCHEMA` and `MIX_SCHEMA` (B89) | `duet-time` | `duet_time` |

Link `T4 before F1` of section 13.4 states the reason: the save path writes through `BundleDocument`, and `view.json` is a `ViewDocument`.

## Steps

1. Read `crates/duet-project/Cargo.toml` and `crates/duet-project/src/lib.rs`. Confirm the M4 skeleton: the `*.workspace = true` package fields, a `description`, `[lints] workspace = true`, no `[dependencies]` section, and a `src/lib.rs` that holds the `//!` crate documentation and `#![forbid(unsafe_code)]`. Report a discrepancy and stop.
2. Create every module file of the write scope as a stub. Each stub holds one `//!` line and nothing else. Add one `mod` line per stub to `src/lib.rs`, and one `mod` line per child stub to its parent module file. The `platform_macos` and `platform_linux` `mod` lines sit behind `#[cfg(target_os = "macos")]` and `#[cfg(target_os = "linux")]` in the crate root, which is the one place a `cfg(target_os)` branch appears in this crate (section 11.2 rule 1).
3. Run `cargo check -p duet-project`. Confirm that the crate builds on macOS and on Linux with the stub tree.
4. Add to `crates/duet-project/Cargo.toml` the entries this chunk uses: `duet-time`, `duet-session`, `duet-command`, `duet-media`, `serde`, `serde_json`, `thiserror`, and `tracing`, each `{ workspace = true }`. Run `cargo build --workspace` and commit `Cargo.lock` with the manifest.
5. Write `ProjectError` and `HistoryError` in `src/lib.rs`, with `#[from]` on each wrapping arm, and the `From<ProjectError> for UpstreamFailure` impl.
6. Write the failing test `atomic_save_completes_a_rename_the_marker_names` in `crates/duet-project/src/save.rs`. Run `cargo nextest run -p duet-project -E 'test(atomic_save)' --no-tests=fail` and confirm that it fails to compile.
7. Write the five-step save in `src/save.rs`, with every `fsync` the list above names. A single file is written as a temporary file in the same directory, then `fsync` of the file, then `rename`, then `fsync` of the directory. Run the test and confirm that it passes.
8. Write the failing test for each remaining row of the crash matrix, one test per row. Run each one and confirm that it fails, then implement the recovery in `src/save.rs` and confirm that each one passes.
9. Write `src/bundle.rs`: bundle creation writes the whole directory layout, `duet.toml`, `.gitattributes`, `.gitignore`, and `media/README.txt`, which states in one line that `git clean -xdf` deletes every take. It initialises the gix repository and writes no `extensions.objectFormat` key.
10. Write `src/gitconfig.rs`: the `.gitattributes` bytes and the two repository-local configuration keys, exactly as the blocks above give them.
11. Write the failing test `view_json_round_trips_through_bundle_document` in `src/view.rs`. Run `cargo nextest run -p duet-project -E 'test(view_json)' --no-tests=fail` and confirm that it fails.
12. Write `src/view.rs`: read and write `state/view.json` through `ViewDocument` and `BundleDocument`. A `ViewDocument` whose schema differs from `VIEW_SCHEMA` returns `CommandError::Schema`. The save path reads `ViewState::tracked()`, which answers false, so `view.json` never enters a commit. Run the test and confirm that it passes.
13. Write `src/templates.rs` and `src/templates/default.rs`: the two templates of PR 11 Q5, "Solo voice" and "SATB". Each template builds one project from a `CreateRequest`, with the key, the meter and the tempo the create form supplies.
14. Declare the platform trait in `src/lib.rs`: one method that answers the runtime socket directory and one that answers the liveness of a process identifier with a start time. The liveness answer has three values: live, dead, and unknown.
15. Write `src/platform_macos.rs`: the runtime socket directory, and a liveness check that calls `sysctl` with `KERN_PROC_PID`, read only. Write `src/platform_linux.rs`: the runtime socket directory, and a liveness check that reads `/proc/<pid>/stat`, read only. Each module states its fallback: `std::env::temp_dir()` for the directory, and an unknown answer that forces the retry path for the liveness check (section 11.1 rule 3).
16. Write the failing test `platform_liveness_answers_unknown_for_a_refused_query`, then implement and confirm that it passes.
17. Run `cargo clippy -p duet-project --all-targets -- -D warnings`. Fix every finding in the code.
18. Commit on the branch `chunk/f1-bundle-and-atomic-save`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test that touches the filesystem creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test uses a fixed path.

| Test | File | What it asserts |
|---|---|---|
| `atomic_save_writes_every_file_and_deletes_the_marker` | `src/save.rs` | After a complete save, every final file holds the new bytes and `state/save.commit` is gone. Message: "a complete save leaves the new state and no marker". |
| `atomic_save_recovers_with_no_marker` | `src/save.rs` | A crash before step 3 leaves the old state, and the recovery deletes every temporary file. Message: "a crash before the marker keeps the old state". |
| `atomic_save_recovers_from_an_unparsable_marker` | `src/save.rs` | A marker that does not parse is deleted with every temporary file, and the old state stands. Message: "an unparsable marker is deleted and the old state stands". |
| `atomic_save_completes_a_rename_the_marker_names` | `src/save.rs` | A crash inside step 4 leaves a valid marker and some renamed files; the recovery completes every rename whose temporary file still exists, and the result is the new state. Message: "the recovery completes every rename the marker names". |
| `atomic_save_deletes_a_marker_with_no_temporary_file` | `src/save.rs` | A crash between step 4 and step 5 leaves a valid marker and no temporary file; the recovery deletes the marker and the result is the new state. Message: "a marker with no temporary file is deleted and the new state stands". |
| `atomic_save_never_leaves_a_mixture` | `src/save.rs` | A table-driven loop over the five crash points asserts that the result is the old state or the new state and never a mixture. The message names the case: `"crash point {point}"`. |
| `view_json_round_trips_through_bundle_document` | `src/view.rs` | A `ViewState` written to `state/view.json` and read back equals the value written. Message: "view.json round trips through BundleDocument". |
| `view_json_refuses_a_foreign_schema` | `src/view.rs` | A `view.json` whose schema is not `VIEW_SCHEMA` returns `CommandError::Schema`. Message: "a view.json of another schema is refused". |
| `view_json_is_not_tracked` | `src/view.rs` | `ViewState::tracked()` is false, and the save path puts no `view.json` path in the tracked set. Message: "git does not track view.json". |
| `bundle_create_writes_the_whole_layout` | `src/bundle.rs` | Every directory and every file of the section 4.1 layout exists after creation, and `media/README.txt` holds the one-line warning. Message: "bundle creation writes the whole layout". |
| `bundle_create_writes_no_object_format_key` | `src/bundle.rs` | The repository configuration holds no `extensions.objectFormat` key and reports format version 0. Message: "a new bundle carries no object-format extension". |
| `bundle_gitignore_covers_the_untracked_set` | `src/bundle.rs` | `.gitignore` covers `media/`, `derived/`, `state/`, and `exports/`. Message: "gitignore covers every untracked directory". |
| `templates_build_solo_voice_and_satb` | `src/templates/default.rs` | Each template builds a project whose part count and staff set match its name, with the key, the meter and the tempo the request carried. Message: "each template builds the parts its name states". |
| `platform_liveness_answers_unknown_for_a_refused_query` | `src/platform_macos.rs` and `src/platform_linux.rs`, each behind its own `cfg` | A liveness query the platform refuses answers unknown and never answers dead. Message: "a refused liveness query answers unknown". |
| `platform_socket_directory_falls_back_to_temp_dir` | `src/platform_macos.rs` and `src/platform_linux.rs` | With no runtime directory, the module answers `std::env::temp_dir()`. Message: "the socket directory falls back to the temporary directory". |

## Verification

1. `cargo nextest run -p duet-project -E 'test(atomic_save) + test(view_json)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of either name exists.
2. `cargo nextest run -p duet-project --no-tests=fail` passes.
3. `cargo clippy -p duet-project --all-targets -- -D warnings` prints nothing.
4. `cargo doc -p duet-project` is clean with `-D warnings`.
5. `cargo machete` reports no unused dependency of `duet-project`.
6. One commit on the branch `chunk/f1-bundle-and-atomic-save` passes the native git hook. The commit carries `crates/duet-project/Cargo.toml` and `Cargo.lock` together.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
