---
id: F5
line: F
depends_on: [F4, M8]
write_scope:
  - crates/bc_project/duet-project/lang_rust/src/watch/debounce.rs
  - crates/bc_project/duet-project/lang_rust/src/watch/ledger.rs
  - crates/bc_project/duet-project/lang_rust/src/lock.rs
  - crates/bc_project/duet-project/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-project -E 'test(writer_ledger) + test(writer_lock)' --no-tests=fail passes; commit SHA on a branch chunk/f5-watcher-and-writer-lock"
---

# F5: The file watcher, the debouncer, the writer ledger, and the writer lock

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the file watch of architecture sections 3.9 and 9.4, the `WriterLedger` of section 15.12, and the four-step writer-lock protocol of section 3.8 with the platform liveness check of section 11.1. It carries MUST story A-02 (live edit with the window open, the writer-lock half) and the reconciliation path every external edit takes.

The watcher runs `notify` 8.2.0 with `notify-debouncer-full` 0.7.0 on the thread that `notify` creates. The 9.0 release candidate is rejected; the pair must match, and the gate does not run a release candidate.

## Files

- `crates/bc_project/duet-project/lang_rust/src/watch/debounce.rs` — modify. Chunk F1 created the stub.
- `crates/bc_project/duet-project/lang_rust/src/watch/ledger.rs` — modify.
- `crates/bc_project/duet-project/lang_rust/src/lock.rs` — modify.
- `crates/bc_project/duet-project/lang_rust/Cargo.toml` — modify. Add the `notify` and `notify-debouncer-full` entries.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_project::watch::ledger` (architecture 3.9, 15.12)

```rust
/// The content hash of every file the project writer last wrote.
#[derive(Debug, Clone, Default)]
pub struct WriterLedger { entries: BTreeMap<Box<str>, [u8; 32]> }
```

The project writer records the content hash of every file it writes. The watcher drops an event whose current hash matches the ledger, which stops a watch storm from our own saves. The `WriterLedger` also records the hash of the bytes a save wrote, so a re-run of a deferred `Save` never leaves a ledger entry that names bytes the document no longer holds (section 9.6, TH8).

### `duet_project::watch::debounce` (architecture 3.9, 9.4)

The six steps of the reconciliation:

1. `duet-project` watches the bundle with `notify` and `notify-debouncer-full`, under the debounce window B26.
2. The project writer records the content hash of every file it writes in a `WriterLedger`. The watcher drops an event whose current hash matches the ledger.
3. A surviving batch reaches `duet-core` as `CoreInput::FilesChanged(Vec<PathBuf>)`.
4. The core parses the changed files into a candidate document. A parse error emits `CoreEvent::ExternalEditRejected { path, error }` and changes nothing.
5. With no unsaved edit, the core swaps the document, increases the version, and emits `CoreEvent::DocumentReloaded { version }`.
6. With an unsaved edit, the core does not merge. It emits `CoreEvent::ExternalEditConflict { paths }`, and the default answer is commit to a new branch.

The watcher watches `score/`, `session/`, `mix/`, `.git/HEAD`, and `.git/refs`. **It does not watch `state/view.json`.** That file is untracked, this process is its one writer, and `Verb::ViewSet` is the one path that changes it.

The file-watch thread owns the `notify` thread and its debouncer, and it does nothing else: it parses no document and touches no core state. It sends one `CoreInput::FilesChanged` and nothing else, on the B28 channel through the `CoreLink` clone the core hands it (section 5.7 thread table, TH13).

### `duet_project::lock` (architecture 3.8)

The writer-lock protocol is four steps and two checks.

1. **The window process binds its socket first, then writes the lock.** `state/duet.lock` is created with `create_new` and holds the process identifier, the start time of that process, and the socket path. A lock therefore never names a socket that does not exist.
2. **A second writer that finds the lock reads it and tries the socket**, under B15. When the socket answers, the second writer forwards its verbs and never writes a file.
3. **When the socket does not answer, the second writer checks liveness.** It reads `/proc/<pid>/stat` on Linux and calls `sysctl` with `KERN_PROC_PID` on macOS, both read-only and both in the platform module of `duet-project`. A live process with a dead socket is a start in progress, so the second writer waits.
4. **The retry rule is B25.** After the last attempt with a live process, the command exits with `GatewayError::ProjectBusy` and a message that names the process identifier. With a dead process, the second writer removes the lock and takes it.

The recorded start time defends against a reused process identifier. A live process whose start time differs from the record is a different process, so the lock is stale.

**What the running application does when its own lock disappears.** `duet-project` watches `state/duet.lock`. When the file is gone, or its content no longer matches this process, three steps run in order.

1. Every write stops at once. The save path, the disk writer, and every job thread refuse with `ProjectError::LockLost`.
2. The transport stops, and any open take is flushed and hashed into `media/incoming/`, which needs no lock because the name is a fresh identifier.
3. `CoreEvent::LockLost` reaches the user. The window shows a blocking dialog with `Re-take the lock` and `Close without saving`.

The liveness check is one of the two platform-specific facilities of this crate (section 11.1). Chunk F1 declared the platform trait and wrote both platform modules in full, and the `mod` lines sit behind `#[cfg(target_os = ...)]` in the crate root. This chunk calls the trait and edits neither platform module. The fallback is a liveness check that returns "unknown" and forces the retry path.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `CoreInput`, `CoreEvent`, `GatewayError`, `CommandError` | `duet-command` | `duet_command` |
| `CoreLink` | `duet-core` | the core hands the watcher one clone; this chunk takes the send end as a trait-object argument, so `duet-project` names no `duet-core` type |
| `ProjectError` | `duet-project` | chunk F1 |

`duet-project` carries no edge to `duet-core` (section 1.3), so the debouncer takes its send end as a closure argument that the core supplies at section 9.4.

## Steps

1. Read every file of the write scope and `crates/bc_project/duet-project/lang_rust/Cargo.toml`. Confirm that chunk F1 left each one a stub and that chunk F4 has landed. Confirm that chunk M8 has landed. Report a discrepancy and stop.
2. Add to `crates/bc_project/duet-project/lang_rust/Cargo.toml` the entries `notify = { workspace = true }` and `notify-debouncer-full = { workspace = true }`. Run `cargo build --workspace` and keep `Cargo.lock` for the same commit.
3. Write the failing test `writer_ledger_drops_our_own_save` in `src/watch/ledger.rs`. Run `cargo nextest run -p duet-project -E 'test(writer_ledger)' --no-tests=fail` and confirm that it fails to compile.
4. Write `WriterLedger` in `src/watch/ledger.rs` with the declaration above, plus the record and the compare operations. Run the test and confirm that it passes.
5. Write the failing test `writer_ledger_keeps_a_foreign_edit`. Run it and confirm that it fails, then confirm that it passes.
6. Write `src/watch/debounce.rs`: start `notify` with `notify-debouncer-full` on the thread `notify` creates, close each batch at B26, filter every event whose current hash matches the ledger, and send one `CoreInput::FilesChanged` per surviving batch. The watch set is `score/`, `session/`, `mix/`, `.git/HEAD`, and `.git/refs`, and it excludes `state/view.json`.
7. Write the failing test `writer_lock_takes_a_stale_lock_from_a_dead_process` in `src/lock.rs`. Run `cargo nextest run -p duet-project -E 'test(writer_lock)' --no-tests=fail` and confirm that it fails.
8. Write `src/lock.rs`: create `state/duet.lock` with `create_new`, hold the process identifier, the start time and the socket path, and implement the four steps and the two checks above. The liveness check calls the platform module, which returns a live answer, a dead answer, or an unknown answer. Run the test and confirm that it passes.
9. Write the failing test `writer_lock_refuses_a_live_process_with_project_busy`. Run it and confirm that it fails, then implement the B25 retry and the `GatewayError::ProjectBusy` refusal and confirm that it passes.
10. Write the failing test `writer_lock_lost_stops_every_write`. Run it and confirm that it fails.
11. Implement the lock-lost path: every write refuses with `ProjectError::LockLost`, and the outcome value reports that the transport must stop and the open take must be flushed and hashed into `media/incoming/`. `duet-project` touches no core state, so it reports and never acts on the transport. Run the test and confirm that it passes.
12. Call the platform liveness check through the trait chunk F1 declared. **`crates/bc_project/duet-project/lang_rust/src/platform_macos.rs` and `crates/bc_project/duet-project/lang_rust/src/platform_linux.rs` are outside this chunk's write scope**, and chunk F1 wrote both modules in full, so this chunk consumes the trait and edits neither file. When either module holds no liveness check, report the discrepancy and stop.
13. Run `cargo clippy -p duet-project --all-targets -- -D warnings` on macOS and on Linux. Fix every finding in the code.
14. Commit on the branch `chunk/f5-watcher-and-writer-lock`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test sleeps; each one waits on a channel or on a returned value.

| Test | File | What it asserts |
|---|---|---|
| `writer_ledger_drops_our_own_save` | `src/watch/ledger.rs` | An event for a file whose current hash equals the ledger entry produces no `CoreInput::FilesChanged`. Message: "the watcher drops an event our own save produced". |
| `writer_ledger_keeps_a_foreign_edit` | `src/watch/ledger.rs` | An event for a file whose current hash differs from the ledger entry survives the filter. Message: "the watcher keeps a foreign edit". |
| `writer_ledger_keeps_an_unknown_path` | `src/watch/ledger.rs` | An event for a path the ledger does not hold survives the filter. Message: "an unknown path is a foreign edit". |
| `writer_ledger_records_the_bytes_a_save_wrote` | `src/watch/ledger.rs` | After a save, the ledger entry for each written path equals the hash of the bytes the save wrote. Message: "the ledger records the bytes the save wrote". |
| `writer_ledger_batch_closes_at_b26` | `src/watch/debounce.rs` | Three events inside the debounce window produce one `CoreInput::FilesChanged` that names three paths. Message: "one batch per B26 window". |
| `writer_ledger_never_watches_view_json` | `src/watch/debounce.rs` | The watch set holds `score/`, `session/`, `mix/`, `.git/HEAD` and `.git/refs`, and it holds no path under `state/`. Message: "the watcher does not watch state/view.json". |
| `writer_lock_names_a_socket_that_exists` | `src/lock.rs` | The lock file is written after the socket is bound, so the path it names exists. Message: "a lock never names a socket that does not exist". |
| `writer_lock_takes_a_stale_lock_from_a_dead_process` | `src/lock.rs` | With a liveness check that answers dead, the second writer removes the lock and takes it. Message: "a lock from a dead process is stale and is taken". |
| `writer_lock_refuses_a_live_process_with_project_busy` | `src/lock.rs` | With a liveness check that answers live and a socket that never answers, the second writer retries B25 times and then returns `GatewayError::ProjectBusy` with the process identifier. Message: "a live process with a dead socket ends in ProjectBusy". |
| `writer_lock_treats_a_different_start_time_as_stale` | `src/lock.rs` | A live process whose start time differs from the record is a different process, so the lock is stale. Message: "a reused process identifier does not hold a lock". |
| `writer_lock_unknown_liveness_forces_the_retry_path` | `src/lock.rs` | A liveness check that answers unknown takes the retry path and never takes the lock at once. Message: "an unknown liveness answer forces the retry". |
| `writer_lock_lost_stops_every_write` | `src/lock.rs` | After the lock file disappears, the save path, the disk writer and a job thread each return `ProjectError::LockLost`. The message names the writer: `"writer {writer} refuses after the lock is lost"`. |

## Verification

1. `cargo nextest run -p duet-project -E 'test(writer_ledger) + test(writer_lock)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of either name exists.
2. `cargo nextest run -p duet-project --no-tests=fail` passes.
3. `cargo clippy -p duet-project --all-targets -- -D warnings` prints nothing on both platforms.
4. `cargo machete` reports no unused dependency of `duet-project`.
5. `cargo deny check` passes with the `notify` and `notify-debouncer-full` pins.
6. One commit on the branch `chunk/f5-watcher-and-writer-lock` passes the native git hook. The commit carries `crates/bc_project/duet-project/lang_rust/Cargo.toml` and `Cargo.lock` together.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives at `crates/bc_<context>/<crate>/lang_rust/` in the context that architecture section 1.2 names (ADR 0011), declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- After chunk M94 lands, every `.rs` file a commit writes carries one front-matter block (`cargo xtask check-ddd --write`), and the gate refuses a changed `.rs` file with none.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
