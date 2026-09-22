---
id: I2
line: I
depends_on: [M8, I1]
write_scope:
  - crates/duet-core/src/channel/input.rs
  - crates/duet-core/src/channel/event.rs
  - crates/duet-core/src/channel/resync.rs
  - crates/duet-core/src/quit.rs
  - crates/duet-core/src/job_runner.rs
  - crates/duet-core/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-core -E 'test(resync) + test(quit_path) + test(quit_save_thread) + test(worker_stopped) + test(take_flag_merge) + test(client_budget)' --no-tests=fail passes; cargo clippy -p duet-core --all-targets -- -D warnings is clean; commit SHA on a branch chunk/i2-structural-channels"
---

# I2: The structural channels, the client registry, and the quit path

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk fills the channel modules that chunk I1 created as seams. It delivers the
structural seam of architecture section 5.8, the resynchronize path with its hysteresis, the client
registry with its B141 refusal, the five-source `TakeFlags` merge of section 6.4, and the whole
nine-step quit path of section 9.5 rule 8. It implements architecture sections 5.8, 6.4, 9.5, 9.6,
12.4 step 3, 15.14, and Appendix A. It carries product stories A-02, C-04, R-13, X-12 and X-09.

This chunk modifies five files and creates none. SM2 gave every file to chunk I1.

## Files

- `crates/duet-core/src/channel/input.rs` — modify. The B28 receive loop and the `CoreInput` arms.
- `crates/duet-core/src/channel/event.rs` — modify. `register_client`, `release_client`, the B29 and
  B30 send rules, and the `TakeFlags` merge.
- `crates/duet-core/src/channel/resync.rs` — modify. The snapshot path and the B39 hysteresis.
- `crates/duet-core/src/quit.rs` — modify. `QuitSave` and the nine steps.
- `crates/duet-core/src/job_runner.rs` — modify. `JobWorker` and `JobRunner::shut_down`.
- `crates/duet-core/Cargo.toml` — modify. No new entry is expected; see step 2. Appendix B.3 names
  chunk I1 in its "Chunk that needs it" column for `async-channel`, `futures` and `crossbeam-queue`,
  so this chunk adds none of the three.
- `Cargo.lock` — modify, only when step 2 changes the member manifest (SM5 rule 2).

## Types and signatures

### Consumed, declared by chunk I1 in `duet-core`

`DuetCore`, `CoreLink`, `ClientLink`, `CoreClient`, `CoreReceiver<T>`, `JobRunner`, `JobWorker`,
`JobRegistry`, `CoreError`. Architecture section 15.14 holds every declaration. This chunk adds no
field to `DuetCore`: section 15.14 declares one struct and chunk I1 writes it whole.

### Declared by this chunk, in `crates/duet-core/src/quit.rs`

Copied from architecture section 15.14.

```rust
/// The `duet-quit-save` thread's own state (TH12).
#[derive(Debug)]
pub struct QuitSave {
    /// Where the project lives.
    bundle: PathBuf,
    /// The byte form of every dirty document, in `BundleDocument::paths`
    /// order.
    documents: Vec<Vec<u8>>,
    /// The write end of the B28 core input channel.
    core: CoreLink,
}
```

### Implemented by this chunk, in `crates/duet-core/src/channel/event.rs`

Copied from architecture section 15.14.

```rust
impl DuetCore {
    /// Register one client and return the ends that client holds.
    ///
    /// # Errors
    /// Returns `CoreError::ClientBudgetExceeded` when B141 clients are
    /// already registered.
    pub fn register_client(&mut self, client: u64) -> Result<CoreClient, CoreError>;

    /// Forget one client and drop its two write ends, which closes both
    /// channels and ends that client's drain task.
    pub fn release_client(&mut self, client: u64);
}
```

### Consumed from `duet-command`

| Type | Declaring section |
|---|---|
| `CoreInput`, with the arms `Call`, `FilesChanged`, `SetEntryContext`, `Resynchronize`, `Shutdown`, `WorkerStopped`, `QuitSaveDone` | 15.5 |
| `CoreEvent`, with `ClientDegraded`, `EngineFault`, `EngineStateChanged`, `DocumentReloaded` | 15.5 |
| `Snapshot { version, score, session, mix, view, engine }` | 15.5 |
| `TakeFlag`, `TakeFlags`, `TrackFlags` | 6.4, 15.4 |
| `CommandError` | 15.5 |

### Consumed from `duet-engine`

| Type | Declaring section |
|---|---|
| `EngineEvent`, with `Fault`, `StateChanged`, `Latency`, `Drift`, `CaptureDone`, `GraphConfigured`, `ConfigureRefused`, `StreamClosed`, `HandoffStopped` | 15.10 |
| `ConfigureCommand`, with `Apply`, `Open`, `Close`, `Reset`, `Stop` | 5.5 |
| `CaptureInfo { track, start, frames, loop_offset, underruns }` | 6.6 |

## Steps

1. Read every file of the write scope. Confirm that chunk I1 created each one and that the channel
   files hold the seam declarations and no behaviour. Stop and report a discrepancy.
2. Read `crates/duet-core/Cargo.toml`. Chunk I1 added `async-channel`, `futures`,
   `crossbeam-queue`, `arrayvec` and `triple_buffer`, because section 15.14 declares one `DuetCore`
   struct whose fields name all five, and Appendix B.3 names I1 for the first three. Add an entry only when a step below names a crate the manifest
   lacks. Run `cargo build --workspace` and commit `Cargo.lock` when the manifest changed.
3. Write the failing test `client_budget_refuses_the_ninth_client` in `src/channel/event.rs`. It
   registers B141 clients and asserts `CoreError::ClientBudgetExceeded` for the next one.
4. Run `cargo nextest run -p duet-core -E 'test(client_budget)' --no-tests=fail` and confirm that it
   fails.
5. Implement `register_client`. Build one B29 `CoreEvent` channel and one B30 `Snapshot` channel per
   client, keep the `ClientLink`, and return the `CoreClient`. Refuse past B141.
6. Implement `release_client`. Remove the `ClientLink` and drop its two write ends, which closes
   both channels and ends that client's drain task.
7. Implement the core input loop in `src/channel/input.rs`. Select over `DuetCore::inputs` and
   `DuetCore::engine_events` with one `futures` select, so one loop wakes on either channel and
   measures every bound of section 9.5 rule 8 on its own passes.
8. Implement each `CoreInput` arm. `Call` reaches `Gateway::call`, which chunk I1 wrote.
   `FilesChanged` runs the reconciliation of section 3.9. `SetEntryContext` records the caret and
   the duration that section 8.3 needs. `Resynchronize` calls the path of step 10.
9. Implement the `CoreEvent` send rule of section 5.8: `try_send` on the B29 channel, and on a full
   channel mark the client stale and stop every event to it until the snapshot is taken.
10. Implement the resynchronize path in `src/channel/resync.rs`. Build one `Snapshot` from the four
    documents in the byte form `BundleDocument::to_bytes` produces. Send it on that client's B30
    channel. Answer a second request inside B39 with the same snapshot. Past B39 emit
    `CoreEvent::ClientDegraded { client }`.
11. Implement the `TakeFlags` merge of section 6.4 in `src/channel/event.rs`. The core accumulates
    one `TrackFlags` per armed track in `DuetCore::take_flags` while a pass runs, from five
    observers:

    | Flag | Source the core reads |
    |---|---|
    | `Uncalibrated` | `Calibration` project state, plus `EngineEvent::Latency` |
    | `HadShortfall` | `EngineFault::CaptureShortfall { track, frames }` |
    | `HadOverrun` | `EngineFault::CaptureOverflow { track, frames }` and `EngineFault::CaptureStalled { track }` |
    | `HadDrift` | `EngineEvent::Drift`, which carries a `DriftReport` |
    | `MidiPortLost` | `HotplugEvent` on the B100 queue |

    At the end of the pass the disk thread reports one `CaptureInfo` per armed track on the B138
    channel. The core then applies the `SessionCommand` for the new take, with the flags it
    assembled, in the same undo transaction as the regions.
12. Implement `JobWorker` in `src/job_runner.rs`. Each worker takes a `JobId` from the B36 queue,
    runs the deferred verb against an `Arc` snapshot, and writes progress into the registry.
13. Implement `JobRunner::shut_down`. Cancel every queued job and drop the queue sender. Each worker
    then sees a closed channel and sends one `CoreInput::WorkerStopped { worker }` as its last act.
14. Implement the nine steps of section 9.5 rule 8 in `src/quit.rs`, in this order. Each step has its
    own bound, and the core measures every one on its own input loop with `std::time::Instant`.

    1. Cancel the `CancellationToken`, so no new agent verb enters.
    2. Send `CoreInput::Shutdown` and wait B23 for every in-flight verb to reply.
    3. Send `TransportCommand::Stop`. Wait B131 for one `EngineEvent::CaptureDone` per armed track
       and commit each take. On expiry mark every unfinished take with `TakeFlag::HadOverrun` and
       record `EngineFault::CaptureStalled`.
    4. Send `ConfigureCommand::Close` and wait B116 for `EngineEvent::StreamClosed`. On expiry
       record `EngineFault::DeviceStalled` and go on.
    5. Call `JobRunner::shut_down` and wait B132 for one `CoreInput::WorkerStopped` per B37 worker.
       Nothing joins.
    6. Build the byte form of every dirty document, move it and the writer-lock handle onto one
       `std::thread` named `duet-quit-save`, and wait B134 for `CoreInput::QuitSaveDone`. That
       thread runs the atomic save of section 4.10, releases the writer lock, and sends the message.
    7. Move the `Runtime` onto a detached `std::thread` and call `shutdown_timeout` with B24 there.
    8. Send `ConfigureCommand::Reset`, then `ConfigureCommand::Stop`, then wait B119 for
       `EngineEvent::HandoffStopped`. Neither send carries a wait of its own; both are one
       `try_send` on the B109 channel.
    9. Let GPUI quit. The main thread never blocks.

15. Write the remaining tests of the Tests section.
16. Run the Completion command and confirm that it passes.
17. Run `cargo clippy -p duet-core --all-targets -- -D warnings` and confirm that it is clean.
18. Commit on a branch named `chunk/i2-structural-channels`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. Every assert carries a message.
Every bound is driven with a plain number, so no test reads a wall clock.

| Test | File | What it asserts |
|---|---|---|
| `client_budget_refuses_the_ninth_client` | `src/channel/event.rs` | B141 clients register, and the next one answers `CoreError::ClientBudgetExceeded`. |
| `client_budget_frees_a_slot_on_release` | `src/channel/event.rs` | `release_client` frees one slot, and the next `register_client` succeeds. |
| `resync_answers_a_second_request_inside_the_window_with_one_snapshot` | `src/channel/resync.rs` | Two requests inside B39 produce one `Snapshot` value, not two. |
| `resync_declares_a_client_degraded_past_the_rate` | `src/channel/resync.rs` | Three requests in ten seconds emit `CoreEvent::ClientDegraded { client }`. |
| `resync_snapshot_carries_the_session_and_the_mix` | `src/channel/resync.rs` | The `Snapshot` holds the score, the session, the mix, the view and the engine state (section 15.5, critic W15). |
| `take_flag_merge_collects_all_five_sources` | `src/channel/event.rs` | One pass that raises all five observations yields a `TakeFlags` that contains all five flags, on the track that `CaptureInfo::track` names. |
| `take_flag_merge_names_its_own_track` | `src/channel/event.rs` | Two armed tracks accumulate two `TrackFlags` values, and neither track receives the other's flag. |
| `worker_stopped_ends_the_wait_when_the_set_empties` | `src/job_runner.rs` | B37 `CoreInput::WorkerStopped` messages end the step 5 wait before B132 elapses. |
| `worker_stopped_on_expiry_leaves_no_partial_file` | `src/job_runner.rs` | A worker that never reports leaves the wait at B132, and the temporary directory rule of section 9.6 holds. |
| `quit_path_runs_the_nine_steps_in_order` | `src/quit.rs` | The recorded step order is 1 to 9 exactly, and steps 1 and 9 perform no cross-boundary wait. |
| `quit_path_bounds_each_step_on_its_own_budget` | `src/quit.rs` | Step 2 reads B23, step 3 reads B131, step 4 reads B116, step 5 reads B132, step 6 reads B134, step 7 reads B24 and step 8 reads B119. No two steps share one bound. |
| `quit_path_sends_reset_before_stop_at_step_eight` | `src/quit.rs` | Step 8 sends `ConfigureCommand::Reset`, then `ConfigureCommand::Stop`, and only then waits. |
| `quit_save_thread_reports_its_outcome` | `src/quit.rs` | The `duet-quit-save` thread sends `CoreInput::QuitSaveDone { outcome }` and the core records a refusal before the process exits. |
| `quit_save_thread_writes_beside_the_final_name` | `src/quit.rs` | The save writes each temporary file in the same directory as its final name and records the set in `state/save.commit` (section 4.10, section 9.5 step 6). |

## Verification

1. `cargo nextest run -p duet-core -E 'test(resync) + test(quit_path) + test(quit_save_thread) + test(worker_stopped) + test(take_flag_merge) + test(client_budget)' --no-tests=fail` passes and selects at
   least one test for every one of the six terms.
2. `cargo nextest run -p duet-core --no-tests=fail` passes, so chunk I1's tests still pass.
3. `cargo clippy -p duet-core --all-targets -- -D warnings` prints no warning.
4. One commit on a branch named `chunk/i2-structural-channels`. The native git hook runs
   `scripts/dod.sh`. Quote the command output before any claim of success, per the
   `verification-before-completion` skill.

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
