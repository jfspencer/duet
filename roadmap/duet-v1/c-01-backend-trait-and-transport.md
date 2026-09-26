---
id: C1
line: C
depends_on: [T4, M4]
write_scope:
  - crates/bc_audio/duet-engine/lang_rust/Cargo.toml
  - crates/bc_audio/duet-engine/lang_rust/src/lib.rs
  - crates/bc_audio/duet-engine/lang_rust/src/backend.rs
  - crates/bc_audio/duet-engine/lang_rust/src/dummy.rs
  - crates/bc_audio/duet-engine/lang_rust/src/transport.rs
  - crates/bc_audio/duet-engine/lang_rust/src/disk.rs
  - crates/bc_audio/duet-engine/lang_rust/src/chain.rs
  - crates/bc_audio/duet-engine/lang_rust/src/mix.rs
  - crates/bc_audio/duet-engine/lang_rust/src/cpal.rs
  - crates/bc_audio/duet-engine/lang_rust/src/audio.rs
  - crates/bc_audio/duet-engine/lang_rust/src/disk/reader.rs
  - crates/bc_audio/duet-engine/lang_rust/src/disk/writer.rs
  - crates/bc_audio/duet-engine/lang_rust/src/disk/ring.rs
  - crates/bc_audio/duet-engine/lang_rust/src/chain/topology.rs
  - crates/bc_audio/duet-engine/lang_rust/src/chain/state.rs
  - crates/bc_audio/duet-engine/lang_rust/src/chain/graph.rs
  - crates/bc_audio/duet-engine/lang_rust/src/chain/migrate.rs
  - crates/bc_audio/duet-engine/lang_rust/src/chain/configure.rs
  - crates/bc_audio/duet-engine/lang_rust/src/mix/strip.rs
  - crates/bc_audio/duet-engine/lang_rust/src/mix/bus.rs
  - crates/bc_audio/duet-engine/lang_rust/src/mix/solo.rs
  - crates/bc_audio/duet-engine/lang_rust/src/cpal/host.rs
  - crates/bc_audio/duet-engine/lang_rust/src/cpal/stream.rs
  - crates/bc_audio/duet-engine/lang_rust/src/cpal/calibrate.rs
  - crates/bc_audio/duet-engine/lang_rust/src/audio/README.md
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-engine -E 'test(transport_machine)' --no-tests=fail passes; commit SHA on a branch chunk/c1-backend-trait-and-transport"
---

# C1: The backend trait, the dummy backend, and the transport machine

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the first layer of `duet-engine`: the `AudioBackend` and `AudioProcess` traits of architecture section 5.1, the variable block-size rule of section 5.2, the `DummyBackend` of section 5.3, the device input selection of section 5.4, and the transport state machine of section 5.9. It also creates every module file of line C as a stub (SM2) and writes the forbidden-call list at `crates/bc_audio/duet-engine/lang_rust/src/audio/README.md` that section 5.7 requires. ADR 0004 clauses 1, 2, 3 and 5 decide the trait shape, the variable block, the dummy backend, and the one-device rule. It carries MUST stories C-22 (playback with a built-in voice, the transport half), R-04 (input selection, the device half), R-06 (count-in), R-07 and R-08 (the transport half), X-02 (the playhead, the transport half), and R-13 (the engine state half). Sections 12.1 and 12.3 decide the error enums and the no-panic forms. The crate skeleton (`Cargo.toml` package fields, `[lints] workspace = true`, `src/lib.rs` with `#![forbid(unsafe_code)]`) already exists, because chunk M4 created it.

## Files

- `crates/bc_audio/duet-engine/lang_rust/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).
- `crates/bc_audio/duet-engine/lang_rust/src/lib.rs` — modify. Add every `mod` line of the line.
- `crates/bc_audio/duet-engine/lang_rust/src/backend.rs` — create.
- `crates/bc_audio/duet-engine/lang_rust/src/dummy.rs` — create.
- `crates/bc_audio/duet-engine/lang_rust/src/transport.rs` — create.
- `crates/bc_audio/duet-engine/lang_rust/src/disk.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/chain.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/mix.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/cpal.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/audio.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/disk/reader.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/disk/writer.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/disk/ring.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/chain/topology.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/chain/state.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/chain/graph.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/chain/migrate.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/chain/configure.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/mix/strip.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/mix/bus.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/mix/solo.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/cpal/host.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/cpal/stream.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/cpal/calibrate.rs` — create as a stub.
- `crates/bc_audio/duet-engine/lang_rust/src/audio/README.md` — create.

A stub holds the `//!` module documentation and nothing else (SM2). A later chunk of line C modifies a stub and creates no file.

## Types and signatures

### `duet_engine::backend` (architecture 5.1, 15.10)

```rust
/// One audio device backend.
pub trait AudioBackend: Send {
    fn name(&self) -> BackendName;

    /// # Errors
    /// Returns `BackendError::Enumerate` when the platform refuses the query,
    /// and `BackendError::NoServer` when the platform audio server is absent.
    fn devices(&self) -> Result<Vec<DeviceInfo>, BackendError>;

    /// # Errors
    /// Returns `BackendError::Enumerate` when the device is gone.
    fn supported_sample_rates(&self, device: &DeviceId) -> Result<Vec<SampleRate>, BackendError>;

    /// # Errors
    /// Returns `BackendError::Enumerate` when the device is gone.
    fn supported_block_sizes(&self, device: &DeviceId) -> Result<BlockSizeRange, BackendError>;

    /// Open a duplex stream and take ownership of the process body.
    ///
    /// # Errors
    /// Returns `BackendError::Open` when the device refuses the request,
    /// `BackendError::RateUnavailable` when the device offers no supported
    /// rate, and `BackendError::OpenTimeout` past B18.
    fn open(&mut self, request: &StreamRequest, process: Box<dyn AudioProcess>)
        -> Result<StreamInfo, BackendError>;

    /// # Errors
    /// Returns `BackendError::Start` when the stream will not run.
    fn start(&mut self) -> Result<(), BackendError>;

    /// # Errors
    /// Returns `BackendError::Stop` when the stream will not stop.
    fn stop(&mut self) -> Result<(), BackendError>;

    /// The capture and playback latency in force.
    ///
    /// # Errors
    /// Returns `BackendError::NotOpen` before `open` succeeds.
    fn latency(&self) -> Result<LatencyReport, BackendError>;

    /// The device sample counter at the start of the current cycle.
    fn clock(&self) -> SampleClock;
}

/// The real-time body. The backend calls `process` on its own thread.
pub trait AudioProcess: Send {
    /// Run one cycle. The frame count varies between cycles.
    fn process(&mut self, cycle: &mut Cycle<'_>) -> CycleOutcome;
}

/// The buffers and the clock for one cycle.
/// **Audio-owned** (section 5.7).
#[derive(Debug)]
pub struct Cycle<'buffers> {
    input: &'buffers [f32],
    output: &'buffers mut [f32],
    channels: ChannelConfig,
    frames: FrameCount,
    start: SampleClock,
}

impl Cycle<'_> {
    /// How many frames this cycle carries. It varies.
    pub fn frames(&self) -> FrameCount;

    /// The device sample counter at the start of this cycle.
    pub fn start(&self) -> SampleClock;

    /// One capture channel, or `None` when the index is out of range.
    pub fn input(&self, channel: ChannelIndex) -> Option<&[f32]>;

    /// One playback channel, or `None` when the index is out of range.
    pub fn output(&mut self, channel: ChannelIndex) -> Option<&mut [f32]>;
}

/// What one cycle produced. Every variant is `Copy` and allocates nothing.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CycleOutcome { Ran, PlaybackStarved, CaptureOverflow, Faulted(FaultCode) }

/// The latency the backend reports, in frames.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LatencyReport { capture: u32, playback: u32, block: u32 }

/// Which backend a stream runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendName { Dummy, Cpal }

/// A device handle the platform host gave us.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeviceId(Box<str>);

/// What the host reports about one device.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeviceInfo { id: DeviceId, label: DeviceLabel, key: DeviceKey, inputs: u16, outputs: u16, default_rate: SampleRate }

/// The block sizes one device accepts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockSizeRange { min: FrameCount, max: FrameCount, fixed_only: bool }

/// What a caller asks the backend to open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamRequest { device: DeviceId, rate: SampleRate, block: FrameCount, inputs: u16, outputs: u16 }

/// What the backend opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StreamInfo { device: DeviceId, rate: SampleRate, max_block: FrameCount, inputs: u16, outputs: u16, output_buffer: FrameCount }

/// The channel counts one chain agreed at configure time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChannelConfig { input: ChannelCount, output: ChannelCount, sends: u16 }

/// A node index into the published topological order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NodeIndex(u16);

/// The measured relation between the capture clock and the playback clock.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DriftReport { frames: i64, window_seconds: u32, parts_per_million: i32 }

/// Every way a device refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum BackendError {
    Enumerate(Box<str>),
    NoServer(ServerDetail),
    Open(Box<str>),
    OpenTimeout(DeviceId),
    RateUnavailable { project: SampleRate, device: SampleRate },
    Start(Box<str>),
    Stop(Box<str>),
    NotOpen,
    CalibrationInvalid,
}
```

### `duet_engine::transport` (architecture 5.9, 15.10)

```rust
/// The transport state. Count-in and latency pre-roll are frame budgets, not
/// states, because a budget composes with every state (Ardour 4).
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportState { Stopped, Rolling, Locating { target: SuperClock } }

/// The record arm state. Three steps, as Ardour models them.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordState { Disabled, Enabled, Recording }

/// The whole transport.
#[expect(
    missing_copy_implementations,
    reason = "the live transport is an identity the audio thread owns, and the type is the size \
              the PG24 table prints for `Transport`"
)]
#[derive(Debug)]
/// **Audio-owned** (section 5.7).
pub struct Transport {
    state: TransportState,
    record: RecordState,
    position: SuperClock,
    count_in_remaining: FrameCount,
    preroll_remaining: FrameCount,
    loop_range: Option<Span>,
    punch_range: Option<Span>,
}

impl Transport {
    /// Apply one intent.
    ///
    /// # Errors
    /// Returns `TransportError::PunchAndLoop` when both ranges would be armed.
    pub fn apply(&mut self, command: TransportCommand) -> Result<TransportState, TransportError>;
}

/// What the audio thread publishes about the transport, once per cycle.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransportSnapshot {
    position: SuperClock,
    state: TransportState,
    record: RecordState,
    count_in_remaining: FrameCount,
    preroll_remaining: FrameCount,
    generation: Generation,
}

impl TransportSnapshot {
    /// The transport stopped at the start, which is the value the
    /// publication carries before the first cycle writes one.
    #[must_use]
    pub const fn stopped() -> Self;
}

/// Every way the transport machine refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TransportError { PunchAndLoop, LocateOutOfRange(SuperClock), RecordWhileStopped }
```

`Transport::apply` carries the one Appendix B.1 complexity suppression of this chunk:

```rust
#[expect(
    too_many_lines,
    cognitive_complexity,
    reason = "one match over the three `TransportState` arms times the three `RecordState` arms, \
              which is nine transitions; `wildcard_enum_match_arm` is denied, so a missing one is \
              a compile error"
)]
```

### `duet_engine` crate root (architecture 15.10)

```rust
/// Every way the engine refuses at run time. It wraps the other three.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EngineError { Backend(BackendError), Config(ConfigError), Transport(TransportError), Media(MediaError), Dsp(DspError) }
```

`EngineError` wraps five causes with `#[from]` (section 12.1). `ConfigError` is declared by chunk C3 in `chain::configure`, so this chunk declares `EngineError` with a `Config` arm over the `ConfigError` stub C3 fills. Declare `ConfigError` in `chain/configure.rs` as part of this chunk only when the compile needs it; otherwise C3 declares it and this chunk leaves the `EngineError::Config` arm out until C3 lands. Choose the second form: `EngineError` in this chunk holds `Backend`, `Transport`, `Media` and `Dsp`, and C3 adds the `Config` arm together with `ConfigError`.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `SampleClock`, `SuperClock`, `FrameCount`, `Span`, `SampleRate`, `ChannelIndex`, `ChannelCount` | `duet-time` | `duet_time` |
| `DeviceKey`, `Calibration`, `CalibrationSource`, `InputSelection`, `MonitorMode`, `TrackId` | `duet-session` | `duet_session` |
| `TransportCommand`, `DeviceLabel`, `ServerDetail`, `FaultCode`, `EngineFault`, `EngineState` | `duet-command` | `duet_command` |
| `MediaError` | `duet-media` | `duet_media` |
| `DspError` | `duet-dsp` | `duet_dsp` |

`Generation` is a `duet-engine` type that chunk C3 declares in `chain::graph`. This chunk declares it in `chain/graph.rs` beside its stub documentation, because `TransportSnapshot` names it and a stub cannot hold a field type:

```rust
/// A publication counter. The audio thread compares two of these once per
/// cycle and does nothing else with them.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Generation(u64);
```

## Steps

1. Read `crates/bc_audio/duet-engine/lang_rust/Cargo.toml` and `crates/bc_audio/duet-engine/lang_rust/src/lib.rs`. Confirm the M4 skeleton: the `*.workspace = true` package fields, a `description`, `[lints] workspace = true`, no `[dependencies]` section, and a `src/lib.rs` that holds the `//!` crate documentation and `#![forbid(unsafe_code)]`. Report a discrepancy and stop.
2. Create every module file of the write scope as a stub. Each stub holds one `//!` line and nothing else. Add one `mod` line per stub to `src/lib.rs`, and one `mod` line per child stub to its parent module file. The `cpal` module file is `src/cpal.rs` and its children are `src/cpal/{host,stream,calibrate}.rs`.
3. Run `cargo check -p duet-engine`. Confirm that the crate builds with the stub tree and that `clippy::mod_module_files` reports nothing.
4. Add to `crates/bc_audio/duet-engine/lang_rust/Cargo.toml` the dependency entries this chunk uses: `duet-time`, `duet-session`, `duet-command`, `duet-media`, `duet-dsp`, `serde`, `thiserror`, `tracing`, and `arrayvec`. Each entry is `{ workspace = true }`. Run `cargo build --workspace` and commit `Cargo.lock` with the manifest.
5. Write the failing test `transport_machine_refuses_punch_and_loop` in `src/transport.rs`. Run `cargo nextest run -p duet-engine -E 'test(transport_machine)' --no-tests=fail` and confirm that it fails to compile, because `Transport` does not exist.
6. Write `TransportState`, `RecordState`, `Transport`, `TransportSnapshot`, and `TransportError` in `src/transport.rs`, with the declarations above. Write `Transport::apply` as one match over the nine transitions, with the `#[expect]` attribute above it. Run the same command and confirm that it passes.
7. Write the failing test `transport_machine_counts_in_before_it_records`. Run it and confirm that it fails.
8. Implement the count-in budget and the pre-roll budget as frame counters that every state carries, as section 6.3 states: the writer captures nothing while `count_in_remaining` is above zero. Run the test and confirm that it passes.
9. Write `BackendName`, `DeviceId`, `DeviceInfo`, `BlockSizeRange`, `StreamRequest`, `StreamInfo`, `ChannelConfig`, `NodeIndex`, `Cycle`, `CycleOutcome`, `LatencyReport`, `DriftReport`, and `BackendError` in `src/backend.rs`, with the `AudioBackend` and `AudioProcess` traits. `Cycle::input` and `Cycle::output` each return `Option<&[f32]>` of exactly `cycle.frames()` frames, de-interleaved and channel major.
10. Write the failing test `dummy_backend_varies_the_block_size`. Run it and confirm that it fails.
11. Write `DummyBackend` in `src/dummy.rs` with the two modes of section 5.3, `RealTime` and `Freewheel`. The frame count of each cycle comes from a fixed-seed generator over the whole range from 1 to `max_block`, so every engine test meets the variation (section 5.2). The backend reports a configurable `LatencyReport`, which is what the alignment test of chunk C4 needs. Run the test and confirm that it passes.
12. Write the device input selection in `src/backend.rs`: the engine answers `InputSelection` and `MonitorMode` from the device list the backend reports, and `InputSelection::None` means the track plays back and records nothing (PR R-04, section 6.1).
13. Write `crates/bc_audio/duet-engine/lang_rust/src/audio/README.md`. It holds the forbidden-call list of section 5.7 verification means 1: no allocation, no free, no lock, no block, no string format, no log, no panic primitive, and TH9 on every published type. It names the six house forms of section 5.11 as the only forms the audio modules use.
14. Write `EngineError` in `src/lib.rs` with `#[from]` on each arm.
15. Run `cargo clippy -p duet-engine --all-targets -- -D warnings`. Fix every finding in the code, never in a policy file.
16. Commit on the branch `chunk/c1-backend-trait-and-transport`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test.

| Test | File | What it asserts |
|---|---|---|
| `transport_machine_refuses_punch_and_loop` | `src/transport.rs` | `Transport::apply` returns `Err(TransportError::PunchAndLoop)` when a loop range and a punch range would both be armed while the transport rolls. Message: "apply refuses a punch range beside a loop range". |
| `transport_machine_walks_nine_transitions` | `src/transport.rs` | A table-driven loop over the three `TransportState` arms times the three `RecordState` arms asserts the resulting state of each transition. The message names the case: `"transition from {state:?} and {record:?}"`. |
| `transport_machine_counts_in_before_it_records` | `src/transport.rs` | With a count-in of one bar, `RecordState` reaches `Recording` and `count_in_remaining` falls to zero over the expected frame count before the first captured frame. Message: "the count-in budget elapses before capture starts". |
| `transport_machine_locate_reports_the_target` | `src/transport.rs` | `TransportCommand::Locate` puts the machine in `TransportState::Locating { target }` with the target it was given. Message: "a locate carries its target". |
| `transport_snapshot_stopped_is_the_start` | `src/transport.rs` | `TransportSnapshot::stopped()` reports position zero, `TransportState::Stopped`, and `RecordState::Disabled`. Message: "the stopped snapshot is the publication default". |
| `dummy_backend_varies_the_block_size` | `src/dummy.rs` | Over 4096 cycles in `Freewheel` with a fixed seed, the set of observed frame counts holds more than one value, the minimum is 1, and the maximum is `max_block`. Message: "the dummy backend varies the block over the whole range". |
| `dummy_backend_reports_the_configured_latency` | `src/dummy.rs` | `AudioBackend::latency` returns the `LatencyReport` the test configured. Message: "the dummy backend reports the configured latency". |
| `cycle_input_and_output_answer_none_past_the_channel_count` | `src/backend.rs` | `Cycle::input` and `Cycle::output` each return `None` for a channel index at or above the configured count, and `Some` with exactly `frames()` samples below it. Message: "a channel index past the count answers None". |

Every assert carries a message. No test uses a shared temporary path, a fixed port, or a sleep. The System-Under-Test builder of each module is one `fn sut(...)` that takes the block range and the latency as explicit arguments.

## Verification

1. `cargo nextest run -p duet-engine -E 'test(transport_machine)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-engine --no-tests=fail` passes.
3. `cargo clippy -p duet-engine --all-targets -- -D warnings` prints nothing.
4. `cargo doc -p duet-engine` is clean with `-D warnings`.
5. `cargo machete` reports no unused dependency of `duet-engine`.
6. One commit on the branch `chunk/c1-backend-trait-and-transport` passes the native git hook.

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

### The audio-thread constraints (architecture 5.7)

- **TH1.** The audio thread never allocates, never frees, never locks, never blocks, never formats a string, never logs, and has no panic path.
- **TH2.** `DuetCore` is the one writer of project state, and it writes on the core thread only.
- **TH3.** No git walk, no file scan, no import parse, no render, no timed wait, and no allocation above B110 inside an immediate verb runs on the GPUI foreground thread.
- **TH4.** The engine disk thread serves the engine only. It moves samples between a ring and a take file and it writes peaks. It allocates no ring and it releases none. `GraphConfigurator::configure` on the engine handoff thread is the one owner of ring allocation.
- **TH5.** The `basedrop` collector runs on the engine handoff thread and drains once per B108. No deferred verb runs on that thread.
- **TH6.** No tokio task holds a GPUI entity handle.
- **TH7.** No runtime-bound tokio type crosses into GPUI code. The ban names a timer, a reactor-driven resource, and a future that awaits either one. The `Runtime` value and a `Handle` are exempt.
- **TH8.** Every deferred verb that writes re-validates on the core thread before it commits.
- **TH9.** A value the audio thread publishes owns no heap. Every type behind a `triple_buffer::Input` whose writer is the audio thread is `Copy`, holds only fixed-size fields, and holds no `Vec`, no `Box`, no `Arc`, and no `String`.
- **TH10.** The MIDI thread allocates nothing on the B9 path, never blocks, never reads a file, and never calls into `duet-core`.
- **TH11.** The cpal error callback is a real-time callback the backend owns, and TH1 binds it word for word. It holds one `basedrop::Shared<FaultQueue>` clone and nothing else.
- **TH12.** The `duet-quit-save` thread exists for one save and then ends.
- **TH13.** Every cross-thread duty this design states has one row of the `carrier-table` block of section 5.8: the message, the field that holds the write end, the carrier and its bound, the field that holds the read end, the thread of each end, and what a full carrier does.
- **PG26.** A type the audio thread owns names no heap-owning type outside a deferring wrapper.
- **PG26b.** A container that GROWS inside an audio-owned declaration is refused, at any depth and inside a deferring wrapper.
- **PG26c.** A LOCK inside an audio-owned declaration is refused, at any depth, inside a deferring wrapper and behind an exemption line alike.
- **PG26e.** The audio-owned root set is closed under reachability from `EngineProcess`.
- **PG26f.** Every asserted audio-owned root carries a third source, and the run prints `ASSERTED ROOTS`.
