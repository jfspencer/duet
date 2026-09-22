---
id: I1
line: I
depends_on: [M7, C1, F1, F3]
write_scope:
  - crates/duet-core/Cargo.toml
  - crates/duet-core/src/lib.rs
  - crates/duet-core/src/gateway.rs
  - crates/duet-core/src/aggregate.rs
  - crates/duet-core/src/undo.rs
  - crates/duet-core/src/job_runner.rs
  - crates/duet-core/src/channel.rs
  - crates/duet-core/src/channel/input.rs
  - crates/duet-core/src/channel/event.rs
  - crates/duet-core/src/channel/resync.rs
  - crates/duet-core/src/snapshot.rs
  - crates/duet-core/src/job.rs
  - crates/duet-core/src/midi_entry.rs
  - crates/duet-core/src/reader.rs
  - crates/duet-core/src/quit.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-core -E 'test(dispatch)' --no-tests=fail passes; cargo clippy -p duet-core --all-targets -- -D warnings is clean; commit SHA on a branch chunk/i1-gateway-dispatch"
---

# I1: Verb dispatch, the aggregate, the undo stack, and the job runner

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk builds the first half of `duet-core`, which architecture section 9 calls the
implementation of the agent gateway. It delivers `Gateway::call` for the project, transport, score,
take, track, mix, view, and history verb groups of section 9.1, the four documents behind an `Arc`
of section 9.6, the undo stack of section 4.9, the `JobRunner` of section 9.6, and
`DuetCore::drain_note_entries` of section 8.3. It implements architecture sections 4.9, 8.3, 9.1,
9.6, 12.1, 15.14, Appendix A row 1, and ADR `adr/0005-agent-gateway.md`. It carries product stories
A-01, A-04, C-02, C-03, C-04, C-07, C-08, C-11, C-13, C-21, H-01, H-02, H-06, R-02, R-05, R-06,
R-09, R-11, X-08, X-09 and X-10 in the "Runtime or verb" column of architecture section 13.2.

This chunk is the FIRST chunk of line I, so SM2 makes it create every module file the line will ever
need, at every depth. Section 13.2 names the whole list in the Writes cell, and this chunk creates
each one.

## Files

- `crates/duet-core/Cargo.toml` — modify. M7 created the skeleton with no `[dependencies]` section.
  **This chunk adds every `{ workspace = true }` entry the crate needs, including `async-channel`,
  `futures`, `crossbeam-queue` and `arrayvec`** (SM1): architecture section 15.14 declares `DuetCore`
  as ONE struct whose fields name all four, and section 13.2 gives this chunk the file that holds the
  struct. Appendix B.3 names I1 in its "Chunk that needs it" column for all four rows; it named I2
  until this revision, and the chunk author of line I reported it.
- `crates/duet-core/src/lib.rs` — modify. M7 wrote the `//!` crate doc and `#![forbid(unsafe_code)]`.
- `crates/duet-core/src/gateway.rs` — create. `Gateway::call` and every dispatch arm.
- `crates/duet-core/src/aggregate.rs` — create. `DuetCore`, its documents, `drain_note_entries`.
- `crates/duet-core/src/undo.rs` — create. `Transaction` and `UndoStack`.
- `crates/duet-core/src/job_runner.rs` — create. `JobRunner` and `JobWorker`.
- `crates/duet-core/src/channel.rs` — create. The `mod` lines of the `channel` directory.
- `crates/duet-core/src/channel/input.rs` — create. `CoreLink` and `CoreReceiver`.
- `crates/duet-core/src/channel/event.rs` — create. `ClientLink` and `CoreClient`.
- `crates/duet-core/src/channel/resync.rs` — create. Stub.
- `crates/duet-core/src/snapshot.rs` — create. Stub.
- `crates/duet-core/src/job.rs` — create. `JobRegistry` and `JobState` handling.
- `crates/duet-core/src/midi_entry.rs` — create. Stub.
- `crates/duet-core/src/reader.rs` — create. Stub.
- `crates/duet-core/src/quit.rs` — create. Stub.
- `Cargo.lock` — modify. SM5 rule 2 binds every chunk that writes a member manifest.

**Which files hold code and which hold documentation alone.** SM2 asks for a stub, and section 13.0
states one exception: a file a later chunk REPLACES is a seam implementation and not a stub, because
it must be reachable from the call graph on the day it lands. Four files take that exception here,
because section 15.14 declares `DuetCore` as ONE struct and chunk I2 cannot edit `aggregate.rs`.

| File | Shape this chunk gives it | Who replaces the body |
|---|---|---|
| `channel.rs` | `mod` lines only | nobody |
| `channel/input.rs` | Seam: `CoreLink`, `CoreReceiver`, and the B28 channel builder | I2 |
| `channel/event.rs` | Seam: `ClientLink`, `CoreClient`, `register_client`, `release_client` | I2 |
| `channel/resync.rs` | Documentation stub | I2 |
| `job.rs` | Seam: `JobRegistry` with its three fields and an empty registry | I3 |
| `snapshot.rs`, `midi_entry.rs`, `reader.rs`, `quit.rs` | Documentation stub | I3, I3, I3, I2 |

## Types and signatures

### Declared by this chunk, in `crates/duet-core/src/aggregate.rs`

Copied from architecture section 15.14. Every field keeps its declared name and its declared type.

```rust
/// The one writer of project state. Section 5.7 gives it the core thread.
///
/// Each document sits behind an `Arc`, so a client reads a snapshot with no
/// lock and the core replaces the whole value with `Arc::make_mut`
/// (Appendix A).
#[derive(Debug)]
pub struct DuetCore {
    score: Arc<Score>,
    session: Arc<Session>,
    mix: Arc<MixState>,
    view: Arc<ViewState>,
    version: Version,
    undo: UndoStack,
    jobs: JobRunner,
    engine: EngineState,
    resets: Arc<ResetGenerations>,
    notes: Arc<ArrayQueue<NoteEntry>>,
    hotplug: Arc<ArrayQueue<HotplugEvent>>,
    entry_dropped: Arc<AtomicU32>,
    pending_configure: Option<Box<ConfigureRequest>>,
    bundle: Option<PathBuf>,
    inputs: CoreReceiver<CoreInput>,
    engine_events: CoreReceiver<EngineEvent>,
    configure: SyncSender<ConfigureCommand>,
    clients: ArrayVec<ClientLink, MAX_CLIENTS>,
    tempo: Input<TempoMap>,
    params: Input<ParamSnapshot>,
    plan: Input<PlaybackPlan>,
    take_flags: ArrayVec<TrackFlags, MAX_DISK_TRACKS>,
}
```

`drain_note_entries` is a plain method with no framework type in its signature, which section 13.2
states and which section 1.3 rule 6 needs. Chunk K1 registers the call and chunk I3 writes the map
from one `NoteEntry` to one score command.

```rust
impl DuetCore {
    /// Drain the note-entry queue and return every entry the queue held.
    ///
    /// `crates/duet` calls it once per frame from `Window::on_next_frame`
    /// (section 8.3, section 13.2). It names no framework type, so
    /// `duet-core` carries no `gpui-kit` edge.
    pub fn drain_note_entries(&mut self) -> Vec<NoteEntry>;
}
```

### Declared by this chunk, in `crates/duet-core/src/undo.rs`

Copied from architecture section 4.9.

```rust
/// One undo step. It is a named group of edits with their inverses.
#[derive(Debug, Clone)]
pub struct Transaction {
    name: TransactionName,
    commands: Vec<EditCommand>,
    inverse: Vec<EditCommand>,
    cost: InverseCost,
    sources: SmallVec<[SourceHash; 4]>,
}

/// The in-session undo stack, owned by `DuetCore`.
#[derive(Debug, Default)]
pub struct UndoStack { done: VecDeque<Transaction>, undone: VecDeque<Transaction>, total: InverseCost }
```

Section 15.14 declares the transaction name, and section 4.9 gives the three bounds B63, B64 and
B65 and the `CoreEvent::UndoHorizonMoved { dropped, oldest_name }` report.

```rust
/// The name of one undo transaction, as the History surface shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionName(Box<str>);
```

### Declared by this chunk, in `crates/duet-core/src/job_runner.rs` and `src/job.rs`

Copied from architecture section 9.6.

```rust
/// The worker pool for every deferred verb.
#[derive(Debug)]
pub struct JobRunner {
    workers: Vec<JoinHandle<()>>,
    queue: Sender<JobId>,
    registry: Arc<JobRegistry>,
    core: CoreLink,
}

/// One `JobRunner` worker's own state. B37 of them exist.
#[derive(Debug)]
pub struct JobWorker {
    worker: u16,
    queue: CoreReceiver<JobId>,
    registry: Arc<JobRegistry>,
    core: CoreLink,
}

/// The registry the core owns. `JobStatus` and `JobCancel` read and write it.
#[derive(Debug, Default)]
pub struct JobRegistry {
    states: Mutex<BTreeMap<JobId, JobState>>,
    cancels: Mutex<BTreeMap<JobId, Arc<AtomicBool>>>,
    next: AtomicU64,
}
```

### Declared by this chunk, in `crates/duet-core/src/channel/input.rs` and `src/channel/event.rs`

Copied from architecture section 15.14.

```rust
/// The write end of the B28 core input channel, as every producer holds it.
#[derive(Debug, Clone)]
pub struct CoreLink { inputs: Sender<CoreInput> }

/// The three ends one CLIENT holds, as that client holds them.
#[derive(Debug)]
pub struct CoreClient {
    link: CoreLink,
    events: CoreReceiver<CoreEvent>,
    snapshots: CoreReceiver<Snapshot>,
}

/// The two write ends of one client, as `DuetCore` holds them.
#[derive(Debug)]
pub struct ClientLink {
    client: u64,
    events: Sender<CoreEvent>,
    snapshots: Sender<Snapshot>,
}

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

`CoreReceiver<T>` is this crate's name for the read end of an `async_channel` at the bounds B28,
B29, B30, B36 and B138 that architecture section 5.8 states. Declare it in `src/channel/input.rs`.

### Declared by this chunk, in `crates/duet-core/src/gateway.rs`

Copied from architecture section 15.14.

```rust
/// Every way the core refuses. It wraps the gateway refusal.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum CoreError { Gateway(GatewayError), NothingToUndo, NothingToRedo, ChannelClosed, ClientBudgetExceeded }
```

### Consumed from other crates

| Type | Crate | Declaring section |
|---|---|---|
| `Verb`, `VerbOutcome`, `VerbCost`, `Gateway` | `duet-command` | 9.1 |
| `GatewayError`, `GatewayRequest`, `CommandError` | `duet-command` | 15.5 |
| `CoreInput`, `CoreEvent`, `Snapshot`, `VerbData`, `DomainEvent` | `duet-command` | 15.5 |
| `JobId`, `JobState`, `Version`, `CommitId`, `CommitSummary` | `duet-command` | 15.5, 9.6 |
| `NoteEntry`, `HotplugEvent`, `EngineState`, `EngineFault` | `duet-command` | 8.3, 12.4 |
| `ViewState`, `ViewDocument`, `Mode`, `ZoomStep` | `duet-command` | 10.2 |
| `Score`, `ScoreCommand`, `Selection`, `EditCommand`, `InverseCost` | `duet-score` | 15.3 |
| `Session`, `SessionCommand`, `SourceHash`, `TrackFlags` | `duet-session` | 15.4, 6.4 |
| `MixState`, `ResetGenerations` | `duet-session`, `duet-dsp` | 15.4, 15.2 |
| `ConfigureCommand`, `ConfigureRequest`, `EngineEvent` | `duet-engine` | 5.5, 15.10 |
| `TempoMap` | `duet-time` | 2.9 |
| `ParamSnapshot`, `PlaybackPlan` | `duet-engine` | 15.10 |
| `MAX_CLIENTS` (B141), `MAX_DISK_TRACKS` (B146) | `duet-time` | 1.6 |

## Steps

1. Read `crates/duet-core/Cargo.toml` and `crates/duet-core/src/lib.rs`. Confirm that M7 wrote the
   package fields, `description`, `[lints] workspace = true`, the `//!` crate doc and
   `#![forbid(unsafe_code)]`, and that no `[dependencies]` section exists. Read the root
   `Cargo.toml` and confirm that `[workspace.dependencies]` carries one
   `duet-<crate> = { path = "crates/duet-<crate>" }` entry per internal crate step 2 names, which
   the manifest chunk of each crate's phase wrote (SM1 rule 4), and that it pins `async-channel`,
   `futures`, `crossbeam-queue` and `arrayvec`. Stop and report if the crate is absent, if it
   already holds code, or if a root entry is missing; SM4 forbids this chunk to edit the root
   manifest.
2. Add the `[dependencies]` section to `crates/duet-core/Cargo.toml`. Add one
   `{ workspace = true }` entry per crate this chunk names: the internal crates `duet-time`,
   `duet-dsp`, `duet-score`, `duet-session`, `duet-command`, `duet-interchange`, `duet-project`,
   `duet-engine`, `duet-midi`, `duet-analysis`, `duet-media`, and the third-party crates
   `smallvec`, `async-channel`, `crossbeam-queue`, `futures`, `triple_buffer`, `arrayvec`,
   `thiserror`, `tracing`, which architecture section 1.2 lists for this crate. Add no
   `duet-export` entry: chunk I4 adds it with the three arms that need it.
3. Run `cargo build --workspace` and commit the `Cargo.lock` it produces, per SM5 rule 3.
4. Create every module file of the Files table. Give each one its `//!` module documentation.
   Add one `mod` line per file to `src/lib.rs`, and one `mod` line per child to `src/channel.rs`.
5. Write the failing test `dispatch_refuses_every_verb_without_a_project` in
   `src/gateway.rs`, inside a `#[cfg(test)] mod tests`. It builds a `DuetCore` with no bundle,
   calls `Gateway::call` with `Verb::Save`, and asserts `GatewayError::NoProject`.
6. Run `cargo nextest run -p duet-core -E 'test(dispatch)' --no-tests=fail` and confirm that it
   fails, because no type compiles yet.
7. Declare `CoreError` in `src/gateway.rs` with the five arms of section 15.14. Derive
   `thiserror::Error` and write one `#[error("...")]` message per arm in Simplified Technical
   English.
8. Declare `CoreReceiver<T>`, `CoreLink`, `ClientLink` and `CoreClient` in the two channel files,
   with the field names section 15.14 states. Build each channel with `async_channel::bounded` at
   the bound its section 5.8 row names.
9. Declare `DuetCore` in `src/aggregate.rs`, field for field, from the block above. Declare
   `Transaction`, `UndoStack` and `TransactionName` in `src/undo.rs`. Declare `JobRunner` and
   `JobWorker` in `src/job_runner.rs` and `JobRegistry` in `src/job.rs`.
10. Implement `Gateway for DuetCore` in `src/gateway.rs`. Write one match arm per `Verb` variant of
    section 9.1. `clippy::wildcard_enum_match_arm` is denied, so the match names every arm.
11. Answer `GatewayError::NotYetImplemented` in exactly three arms: `ExportAudio`, `MasterMeasure`
    and `Gc`. Section 13.2 states that split, and chunk I4 delivers the three.
12. Implement `Verb::cost` dispatch: an `Immediate` verb runs on the calling thread inside B5, and a
    `Deferred` verb enqueues a `JobId` on the B36 queue and answers `VerbOutcome::Started { job }`.
    A full queue answers `GatewayError::JobQueueFull { user_half }`, and `Verb::is_user_started`
    decides which half section 9.6 reserves.
13. Implement the snapshot hand-off of section 9.6: a job takes `Arc::clone` of each document, and
    an edit that lands while a snapshot is outstanding calls `Arc::make_mut`.
14. Implement `UndoStack` push, undo and redo. Drop the oldest whole transaction when B63, B64 or
    B65 is passed, and emit `CoreEvent::UndoHorizonMoved { dropped, oldest_name }`.
15. Implement `DuetCore::drain_note_entries`. Drain `notes`, read and reset `entry_dropped`, and
    return the entries. The method names no framework type.
16. Write the remaining tests of the Tests section.
17. Run `cargo nextest run -p duet-core -E 'test(dispatch)' --no-tests=fail` and confirm that it
    passes.
18. Run `cargo clippy -p duet-core --all-targets -- -D warnings` and confirm that it is clean.
19. Commit on a branch named `chunk/i1-gateway-dispatch`. The native git hook runs the gate.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. Every assert carries a message.

| Test | File | What it asserts |
|---|---|---|
| `dispatch_refuses_every_verb_without_a_project` | `src/gateway.rs` | `Verb::Save` with no open bundle answers `GatewayError::NoProject`. |
| `dispatch_answers_not_yet_implemented_for_exactly_three_verbs` | `src/gateway.rs` | It drives every `Verb` arm and counts the arms that answer `GatewayError::NotYetImplemented`. The count is three, and the three are `ExportAudio`, `MasterMeasure` and `Gc` (section 13.2). |
| `dispatch_marks_every_deferred_verb_started` | `src/gateway.rs` | Every verb whose `cost` is `VerbCost::Deferred` answers `VerbOutcome::Started { job }` and the registry holds `JobState::Queued` for that `JobId`. |
| `dispatch_refuses_a_full_user_half_of_the_job_queue` | `src/gateway.rs` | A user-started verb on a full reserved half answers `GatewayError::JobQueueFull { user_half: true }` (section 9.6). |
| `dispatch_bumps_the_version_on_every_changed_outcome` | `src/gateway.rs` | A `VerbOutcome::Changed` carries a `Version` strictly above the one before it. |
| `undo_stack_drops_whole_transactions_at_the_horizon` | `src/undo.rs` | A stack past B63 drops the oldest whole transaction and never part of one, and it reports `dropped` and `oldest_name`. |
| `undo_stack_refuses_an_empty_transaction` | `src/undo.rs` | A transaction that collects no command never enters the stack (section 4.9). |
| `note_entry_drain_returns_every_queued_entry` | `src/aggregate.rs` | `drain_note_entries` returns each `NoteEntry` once, in queue order, and leaves the queue empty. |
| `note_entry_drain_reports_the_dropped_count` | `src/aggregate.rs` | A `force_push` past B31 raises the shared counter, and the drain reads and clears it (section 8.3). |

`dispatch_answers_not_yet_implemented_for_exactly_three_verbs` is the test chunk I4 later mirrors
with `no_verb_is_unimplemented`. Section 13.2 states that pair, and critic C16-10 is the reason the
arm stays declared.

## Verification

1. `cargo nextest run -p duet-core -E 'test(dispatch)' --no-tests=fail` prints a pass for every
   `dispatch` test and no `no tests to run` line.
2. `cargo nextest run -p duet-core --no-tests=fail` passes, so the undo and note-entry tests pass
   too.
3. `cargo clippy -p duet-core --all-targets -- -D warnings` prints no warning.
4. `cargo build --workspace` leaves `Cargo.lock` unchanged after the commit.
5. One commit on a branch named `chunk/i1-gateway-dispatch`. The native git hook runs
   `scripts/dod.sh` and the commit lands only when the gate passes. The `verification-before-completion`
   skill states the rule: quote the command output before any claim of success.

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
