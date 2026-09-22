---
id: C3
line: C
depends_on: [C2, D2, D3]
write_scope:
  - crates/duet-engine/src/chain/topology.rs
  - crates/duet-engine/src/chain/state.rs
  - crates/duet-engine/src/chain/graph.rs
  - crates/duet-engine/src/chain/migrate.rs
  - crates/duet-engine/src/chain/configure.rs
  - crates/duet-engine/src/mix/strip.rs
  - crates/duet-engine/src/mix/bus.rs
  - crates/duet-engine/src/mix/solo.rs
  - crates/duet-engine/tests/soak.rs
  - crates/duet-engine/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-engine -E 'test(chain_migrate) + test(solo_in_place) + test(slot_meters) + test(fault_run) + test(handoff_stop) + test(port_gone_silence)' --no-tests=fail passes; commit SHA on a branch chunk/c3-chain-graph-and-publications"
---

# C3: The chain, the graph runner, the handoff thread, and the three publications

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the centre of `duet-engine`: `ChainTopology`, `ChainState`, `PoolHandle`, `ChainSlot`, `ChainSource`, `ChainLayout`, and `GraphConfigurator` with the engine handoff thread, its `basedrop` collector at B108 and its B95 retry; the per-position migration of architecture section 5.5; the drain rule and the generation proof of section 5.6; the mixer runtime with solo in place of section 7.1; the graph runner of section 5.6; the THREE audio publications with `TransportSnapshot`, `MeterSnapshot` and `SlotMeterSnapshot` of sections 5.8, 5.9 and 7.3; the `FaultRun` latch with its B136 cap; `EngineProcess::sounding` with the `MidiMessage::PortGone` drain, the note-off synthesis of section 8.2 step 1 and B139; the B138 send from this thread with its B147 pending list; and `ConfigureCommand::Stop`, which is the one message that ends the thread's loop. It also writes the `soak` test. ADR 0004 clauses 8, 8a to 8e.2 govern the chain and the graph. It carries MUST stories C-22 (playback), M-02 (track strips and part bus strips), M-03 (the vocal tool set runtime), M-04 (reverb and delay buses), M-05 (metering and the over latch), M-06 (solo and mute), R-04 (the monitor path), X-02 (the transport publication), and SHOULD story R-10 (the layered mode in the runtime).

**Five selected tests and not two.** This chunk names the most mechanisms of any chunk of the plan, and SM3 asks one command to fail before the chunk and pass after it.

**This chunk owns item 14 of architecture section 10.7 rung three.** Rung three holds fourteen numbered behaviours. Items 1 to 13 are user-interface behaviours and the chunks of line K write them. Item 14 is the generation skew window: it pushes a handoff at generation N plus 1, runs one audio cycle with the published `GraphChain` still at N, and asserts one `EngineFault::HandoffGenerationSkew { published, adopted }` and no panic; it then publishes N plus 1, runs a second cycle, and asserts no second fault and a graph that runs every strip. The mechanism is the engine handoff and no view takes part, so the test lives here, in `crates/duet-engine/src/chain/graph.rs`, and `graph_runner_reports_generation_skew_once` is its name. The count word of rung three said twelve until this revision, and the chunk author of line K reported the difference.

## Files

- `crates/duet-engine/src/chain/topology.rs` — modify. Chunk C1 created the stub.
- `crates/duet-engine/src/chain/state.rs` — modify.
- `crates/duet-engine/src/chain/graph.rs` — modify.
- `crates/duet-engine/src/chain/migrate.rs` — modify.
- `crates/duet-engine/src/chain/configure.rs` — modify.
- `crates/duet-engine/src/mix/strip.rs` — modify.
- `crates/duet-engine/src/mix/bus.rs` — modify.
- `crates/duet-engine/src/mix/solo.rs` — modify.
- `crates/duet-engine/tests/soak.rs` — create. SM2 covers `src/` only, so the chunk that writes an integration test creates the file and names it in its own write scope.
- `crates/duet-engine/Cargo.toml` — modify.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_engine::chain::topology` (architecture 5.5, 15.10)

```rust
/// The immutable shape of one track chain. It is published, never mutated.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainTopology {
    strip: StripId,
    track: Option<TrackId>,
    channels: ChannelConfig,
    polarity: bool,
    pre_fader: ArrayVec<SlotSpec, MAX_SLOTS>,
    post_fader: ArrayVec<SlotSpec, MAX_SLOTS>,
    sends: ArrayVec<SendSpec, MAX_SENDS>,
    meter_tap: MeterTap,
    output: StripTarget,
    /// Computed off the audio thread from every strip's mute and solo flag
    /// (section 7.1). The audio thread reads it and never computes it.
    audible: AudibleState,
}

/// One slot's declaration: what it is, which parameters address it, and
/// which pool buffer it uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotSpec { slot: SlotId, kind: SlotKind, params: ParamIdRange, buffer: Option<PoolSlot> }

/// One send as the runtime reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SendSpec { id: SendId, target: NodeIndex, level: ParamId, tap: SendTap }
```

### `duet_engine::chain::state` (architecture 5.5, 15.10)

```rust
/// The mutable running state of one chain. The audio thread owns it and no
/// other thread ever holds a reference to it while a cycle runs.
/// **Audio-owned** (section 5.7).
#[derive(Debug)]
pub struct ChainState {
    strip: StripId,
    writer: Option<DiskWriter>,
    reader: Option<DiskReader>,
    trim: GainState,
    pre_fader: ArrayVec<SlotState, MAX_SLOTS>,
    fader: GainState,
    post_fader: ArrayVec<SlotState, MAX_SLOTS>,
    meter: MeterState,
    /// The meter-reset generation this strip last acted on.
    reset_seen: Generation,
    /// The `PlaybackStarved` run of this strip.
    starved: FaultRun,
    /// The `CaptureOverflow` run of this strip.
    overflow: FaultRun,
    /// The `CaptureShortfall` run of this strip.
    shortfall: FaultRun,
}

/// One per-condition fault run of one chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaultRun { open: bool, frames: u32 }

/// A position in the audio thread's chain-state array.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ChainIndex(u16);

/// Where one position of the new chain-state array gets its state.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChainSource {
    /// `configure` built this position's state off the audio thread.
    Adopt,
    /// The state moves out of this position of the array the audio thread
    /// holds now.
    Keep(ChainIndex),
}

/// One position of the audio thread's chain-state array.
/// **Audio-owned** (section 5.7).
#[derive(Debug)]
pub struct ChainSlot { source: ChainSource, state: Option<ChainState>, reported: bool }

/// The audio thread's owned chain-state array, behind a hand-written `Debug`.
/// **Audio-owned** (section 5.7).
pub struct ChainSetHandle(Owned<Box<[ChainSlot]>>);

/// The audio thread's owned pool, behind a hand-written `Debug`.
/// **Audio-owned** (section 5.7).
pub struct PoolHandle(Owned<BufferPool>);

/// The audio thread's read end of the B105 handoff ring, behind a
/// hand-written `Debug`.
/// **Audio-owned** (section 5.7).
pub struct HandoffReader { consumer: Owned<Consumer<GraphHandoff>> }

/// One owned handoff from the engine handoff thread to the audio thread, at
/// B105.
/// **Audio-owned** (section 5.7).
#[derive(Debug)]
pub struct GraphHandoff {
    /// The generation this handoff belongs to.
    generation: Generation,
    /// The new array. It holds no pool: one pool serves the open stream.
    slots: ChainSetHandle,
}
```

`ChainSetHandle`, `PoolHandle` and `HandoffReader` each write `Debug` by hand with `finish_non_exhaustive`, because `basedrop` 0.1.3 implements `Debug` for neither `Owned<T>` nor `Shared<T>` and `missing_debug_implementations` is denied. **No chunk may delete a wrapper.**

### `duet_engine::chain::configure` (architecture 5.5, 15.10)

```rust
/// The strip order one chain-state array holds, one `StripId` per position,
/// and the ring pairs that array holds in total.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChainLayout { strips: Box<[StripId]>, pairs: u32 }

/// Everything `configure` reads. The core builds it on the core thread.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigureRequest {
    chains: Box<[ChainTopology]>,
    order: Box<[NodeIndex]>,
    stream: StreamInfo,
    generation: Generation,
}

/// What one `configure` produced.
#[derive(Debug)]
pub struct Configured { topology: GraphChain, handoff: GraphHandoff, layout: ChainLayout }

/// What the core asks the engine handoff thread to do, at B109.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigureCommand {
    Apply(Box<ConfigureRequest>),
    Open(Box<StreamRequest>),
    Close,
    Reset,
    Stop,
}

/// The engine handoff thread's own state. One exists per engine.
pub struct GraphConfigurator {
    /// **The one owner of the audio backend.**
    backend: Box<dyn AudioBackend>,
    /// The `basedrop` collector. One pass of the loop drains it (TH5).
    collector: Collector,
    /// The B34 fault queue this thread drains once per pass (section 12.4).
    faults: Shared<FaultQueue>,
    /// The read end of the configure channel at B109.
    commands: Receiver<ConfigureCommand>,
    /// The one write end of the B105 handoff ring.
    handoffs: Producer<GraphHandoff>,
    /// The one write end of the published topology (section 5.8).
    topology: Input<GraphChain>,
    /// The layout the last accepted handoff left on the audio thread.
    layout: ChainLayout,
    /// The topology change this thread has not built yet, or `None`.
    pending_request: Option<ConfigureRequest>,
    /// The configuration this thread built and has not pushed, or `None`.
    handoff_retry: Option<Configured>,
    /// Consecutive passes on which the B105 slot was not free, against B95.
    attempts: u8,
    /// The write end of the engine-to-core event channel at B138.
    events: EngineLink,
    /// Every event `try_send` refused, at B147.
    pending_events: ArrayVec<EngineEvent, MAX_PENDING_EVENTS>,
}

impl GraphConfigurator {
    /// Build everything one topology change needs, on the engine handoff
    /// thread.
    ///
    /// # Errors
    /// Returns `ConfigError::ChannelMismatch` when a stage cannot accept the
    /// count that the stage before it produces, `ConfigError::PoolExhausted`
    /// past B47, B48, or B120, `ConfigError::RingBudgetExceeded` when the
    /// migration peak passes B57, `ConfigError::StripBudgetExceeded` past
    /// B86, `ConfigError::ParamBudgetExceeded` past B90,
    /// `ConfigError::HandoffRetryPending` when `handoff_retry` is already
    /// `Some`, and `ConfigError::Mix`. Every refusal is computed before any
    /// allocation.
    pub fn configure(&mut self, request: &ConfigureRequest)
        -> Result<Configured, ConfigError>;

    /// Push the pending handoff at B105 and publish its topology, or leave
    /// it in `handoff_retry` for the next attempt.
    ///
    /// # Errors
    /// Returns `ConfigError::PoolHandoffBusy` on a full ring at the B95
    /// attempt, and `ConfigError::NothingToHandOff` when `handoff_retry` is
    /// `None`.
    pub fn hand_off(&mut self) -> Result<Option<Generation>, ConfigError>;
}

/// Every way `configure` refuses, before audio flows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ConfigError {
    ChannelMismatch { stage: SlotId, produced: u16, accepted: u16 },
    PoolExhausted { kind: SlotKind },
    RingBudgetExceeded { requested_bytes: u64 },
    StripBudgetExceeded { requested: u32 },
    ParamBudgetExceeded { requested: u32 },
    PoolHandoffBusy,
    HandoffRetryPending,
    NothingToHandOff,
    Mix(MixError),
}
```

The `From<ConfigError> for GatewayError` impl of section 15.10 lives in `duet-engine`, because the orphan rule allows `impl From<Local> for Foreign` and `duet-command` carries no edge to `duet-engine`. Write it exactly as section 15.10 declares it.

### `duet_engine::chain::graph` (architecture 5.6, 15.10)

```rust
/// A pre-computed, immutable topology for the whole graph.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphChain {
    chains: Box<[ChainTopology]>,
    /// Node indexes in topological order.
    order: Box<[NodeIndex]>,
    generation: Generation,
    handoff_generation: Generation,
}

/// Every strip's meter-reset generation, shared by the core and the audio
/// thread.
#[derive(Debug)]
pub struct ResetGenerations { entries: [AtomicU32; MAX_STRIPS] }

impl ResetGenerations {
    /// Every strip's generation at zero.
    pub const fn new() -> Self {
        Self { entries: [const { AtomicU32::new(0) }; MAX_STRIPS] }
    }
}

impl Default for ResetGenerations {
    fn default() -> Self { Self::new() }
}

/// The audio thread's own mutable state for the whole graph.
/// **Audio-owned** (section 5.7).
#[derive(Debug)]
pub struct GraphState {
    chains: ChainSetHandle,
    pool: PoolHandle,
    generation: Generation,
    handoff_generation: Generation,
    handoffs: HandoffReader,
    resets: Arc<ResetGenerations>,
}

/// How a chain runs. One implementation exists in version one.
pub trait GraphRunner {
    fn run(&mut self, topology: &GraphChain, state: &mut GraphState, cycle: &mut Cycle<'_>)
        -> CycleOutcome;
}

/// The one `GraphRunner` of version one: one topological pass on the audio
/// thread (section 5.6).
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ChainRunner;

impl GraphRunner for ChainRunner {
    fn run(&mut self, topology: &GraphChain, state: &mut GraphState, cycle: &mut Cycle<'_>)
        -> CycleOutcome;
}

/// The note numbers sounding on one `PortSlot`.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SoundingNotes { bits: [u64; 2] }

/// Every parameter value the audio thread reads, addressed by `ParamId`.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParamSnapshot { values: [Finite; MAX_PARAMS], generation: Generation }

/// The B34 fault queue and its drop counter, as one shared allocation.
#[derive(Debug)]
pub struct FaultQueue { queue: ArrayQueue<EngineFault>, dropped: AtomicU32 }

/// The real-time body the backend owns, and the ROOT of the audio thread.
/// **Audio-owned** (section 5.7).
pub struct EngineProcess {
    graph: GraphState,
    topology: Owned<Output<GraphChain>>,
    tempo: Owned<Output<TempoMap>>,
    params: Owned<Output<ParamSnapshot>>,
    midi: Owned<Consumer<MidiRecord>>,
    refills: Owned<Producer<RefillRequest>>,
    faults: Shared<FaultQueue>,
    transport: Transport,
    transport_out: Owned<Input<TransportSnapshot>>,
    meters: Owned<Input<MeterSnapshot>>,
    slot_meters: Owned<Input<SlotMeterSnapshot>>,
    /// The notes sounding on each bound port, at B139.
    sounding: [SoundingNotes; MAX_BOUND_PORTS],
    runner: ChainRunner,
}

impl AudioProcess for EngineProcess {
    fn process(&mut self, cycle: &mut Cycle<'_>) -> CycleOutcome;
}
```

`EngineProcess` writes `Debug` by hand with `finish_non_exhaustive`. **Every heap handle sits inside a `basedrop` wrapper.** PG26, PG26b, PG26c, PG26e and PG26f each walk from `EngineProcess`.

### `duet_engine::mix::strip` (architecture 5.9, 7.3)

```rust
/// Every strip's meter values for one frame, and its held over marks.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeterSnapshot {
    readings: [MeterReading; MAX_STRIPS],
    over: [OverMark; MAX_STRIPS],
    live: u16,
    latched: [Generation; MAX_STRIPS],
}

/// Every configured slot's live measurement for one frame.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SlotMeterSnapshot {
    measures: [SlotMeasure; MAX_SLOT_METERS],
    live: u16,
    generation: Generation,
}

impl SlotMeterSnapshot {
    /// Every cell silent, which is the value the publication carries before
    /// the first cycle writes one.
    #[must_use]
    pub const fn silent() -> Self;
}
```

The one cell index of section 7.3, which both ends compute and neither stores:

```text
cell = strip * 2 * MAX_SLOTS + position * MAX_SLOTS + slot
```

### `duet_engine::mix::solo` (architecture 7.1)

```rust
/// What the runtime computes from the mute and solo flags, once per
/// topology change, off the audio thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudibleState { explicit_mute: bool, implicit_mute: bool }
```

### `duet_engine::chain::migrate` (architecture 5.5)

`migrate_state` carries the one Appendix B.1 complexity suppression of this chunk:

```rust
#[expect(
    too_many_lines,
    reason = "one match over the four migration cases of section 5.5, each of which reads both \
              generations once"
)]
```

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `BufferPool`, `PoolBuffer`, `PoolSlot`, `SlotState`, `GainState`, `MeterState`, `MeterReading`, `SlotMeasure`, `OverMark`, `BiquadState` | `duet-dsp` | `duet_dsp` |
| `SlotKind`, `SlotId`, `SendId`, `StripId`, `StripTarget`, `SendTap`, `MeterTap`, `ParamIdRange`, `ParamId`, `MixError`, `TrackId` | `duet-session` | `duet_session` |
| `MidiRecord`, `MidiMessage`, `BoundPort`, `EngineFault`, `FaultCode`, `TempoMap` | `duet-command` and `duet-time` | `duet_command`, `duet_time` |
| `MAX_STRIPS` (B86), `MAX_SLOTS` (B45), `MAX_SENDS` (B46), `MAX_PARAMS` (B90), `MAX_SLOT_METERS` (B133), `MAX_BOUND_PORTS` (B139), `MAX_PENDING_EVENTS` (B147) | `duet-time` | `duet_time` |

## Steps

1. Read every file of the write scope and `crates/duet-engine/Cargo.toml`. Confirm the stubs chunk C1 created and the ring and disk types chunk C2 wrote. Report a discrepancy and stop.
2. Add to `crates/duet-engine/Cargo.toml` any entry this chunk adds that the manifest does not hold, each `{ workspace = true }`. Run `cargo build --workspace` and keep `Cargo.lock` for the same commit.
3. Write the failing test `chain_migrate_keeps_the_state_of_a_kept_chain` in `src/chain/migrate.rs`. Run `cargo nextest run -p duet-engine -E 'test(chain_migrate)' --no-tests=fail` and confirm that it fails to compile.
4. Write `ChainTopology`, `SlotSpec` and `SendSpec` in `src/chain/topology.rs`, and `ChainState`, `FaultRun`, `ChainIndex`, `ChainSource`, `ChainSlot`, `ChainSetHandle`, `PoolHandle`, `HandoffReader` and `GraphHandoff` in `src/chain/state.rs`, with the declarations above.
5. Write `migrate_state` in `src/chain/migrate.rs` as the four cases of section 5.5 rules 1 to 4, plus rule 0, the per-position move. For each `Keep(k)` position the audio thread writes `new[i].state = old[k].state.take()`, which MOVES the kept state and leaves `None` behind. Nothing is copied, nothing is allocated, nothing is freed. Run the test and confirm that it passes.
6. Write `ChainLayout`, `ConfigureRequest`, `Configured`, `ConfigureCommand`, `GraphConfigurator`, `ConfigError` and the `From<ConfigError> for GatewayError` impl in `src/chain/configure.rs`.
7. Implement `GraphConfigurator::configure`. It reads four things and nothing else: `request`, `self.layout`, `self.handoff_retry`, and the free-slot count of `self.handoffs`. It refuses first and allocates second: `PoolExhausted` past B47, B48 or B120, which are THREE pool classes and not two; `StripBudgetExceeded` past B86; `ParamBudgetExceeded` past B90; `RingBudgetExceeded` past B57, computed from the B56 formula `peak = B52 + B54 * (ChainLayout::pairs + added pairs)`. It then allocates one ring pair per ADDED chain and none for a kept chain, writes every `PoolSlot` into the matching `SlotSpec`, and builds one `Box<[ChainSlot]>` of the new length inside one `basedrop::Owned`.
8. Implement `GraphConfigurator::hand_off`. It takes no argument. `Ok(None)` is a deferred attempt and not an error. The push happens before the publication, so the audio thread never sees a raised `handoff_generation` with no handoff behind it. `attempts` counts consecutive passes against B95; past B95 the thread reports `EngineFault::HandoffBusy`, drops `pending_request` into `basedrop`, and stops building until a `ConfigureCommand::Reset` arrives.
9. Implement the loop of the engine handoff thread in the fixed order of section 5.5: drain the collector with `Collector::collect`; serve at most one `ConfigureCommand`, draining B109 so the last `Apply` wins; park on a bounded receive of B108. Implement the five steps of one topology change and the three outcomes of step 5.
10. Implement the B34 fault drain on this thread (section 12.4 step 2). It sends one `EngineEvent::Fault` per record on the B138 channel through `GraphConfigurator::events`, and it swaps the drop counter to zero and mints `EngineFault::FaultsDropped { count }` when the swap returns a value above zero. A refused `try_send` waits in `pending_events` at B147; while the list is not empty this thread drains no further fault, so the backpressure reaches B34.
11. Write the failing test `handoff_stop_ends_the_loop_and_reports`. Run `cargo nextest run -p duet-engine -E 'test(handoff_stop)' --no-tests=fail` and confirm that it fails.
12. Implement `ConfigureCommand::Stop`. It is the one message that ends this thread's loop: the thread runs `Collector::collect`, then `Collector::try_cleanup`, sends `EngineEvent::HandoffStopped` on the B138 channel with a BLOCKING send, drops `backend`, and returns. Run the test and confirm that it passes.
13. Write `GraphChain`, `ResetGenerations`, `GraphState`, `GraphRunner`, `ChainRunner`, `SoundingNotes`, `ParamSnapshot`, `FaultQueue` and `EngineProcess` in `src/chain/graph.rs`.
14. Implement the drain rule of section 5.6 at the top of every cycle, in three steps: drain the B105 ring to empty and apply each handoff IN FULL before the next pop; prove the adopted generation against the published generation and report `EngineFault::HandoffGenerationSkew { published, adopted }` once per change; compare the topology generation and run the in-place slot migration. An empty position is `EngineFault::ChainSlotVacant { index }`, latched by `ChainSlot::reported`, and never a panic.
15. Write the failing test `fault_run_reports_once_per_run_and_at_the_cap`. Run `cargo nextest run -p duet-engine -E 'test(fault_run)' --no-tests=fail` and confirm that it fails.
16. Implement the `FaultRun` latch. `GraphRunner::run` owns all four transitions: it opens a run on the first cycle a condition holds, it adds that cycle's frame count on every cycle the run stays open, it pushes NOTHING while a run is open, and it closes the run and pushes ONE `EngineFault` on the first cycle the condition stops. A run also closes at B136 frames and then re-opens. Run the test and confirm that it passes.
17. Write the failing test `port_gone_silence_clears_the_bitset_in_one_cycle`. Run `cargo nextest run -p duet-engine -E 'test(port_gone_silence)' --no-tests=fail` and confirm that it fails.
18. Implement `EngineProcess::sounding` and the `MidiMessage::PortGone` drain of section 8.2 step 1. The cycle that reads a `PortGone` record synthesizes a note-off for every set bit of that `BoundPort` slot and clears the bitset with two stores. It allocates nothing. Run the test and confirm that it passes.
19. Write the failing test `slot_meters_fill_the_cell_the_formula_names`. Run `cargo nextest run -p duet-engine -E 'test(slot_meters)' --no-tests=fail` and confirm that it fails.
20. Write `MeterSnapshot`, `SlotMeterSnapshot` and `SlotMeterSnapshot::silent` in `src/mix/strip.rs`, and implement the three audio publications: the runner writes `transport_out`, `meters` and `slot_meters` once per cycle through the `triple_buffer::Input` ends `EngineProcess` holds. Every published type is `Copy` plain data with fixed-size fields and no heap field (TH9). Run the test and confirm that it passes.
21. Write the failing test `solo_in_place_mutes_every_strip_that_feeds_nothing_soloed`. Run `cargo nextest run -p duet-engine -E 'test(solo_in_place)' --no-tests=fail` and confirm that it fails.
22. Write `AudibleState` in `src/mix/solo.rs` and the three rules of section 7.1, computed once per topology change and off the audio thread. Write the bus runtime in `src/mix/bus.rs`: a bus is a strip with no disk reader and no disk writer, and the master is a bus with a true-peak meter. Run the test and confirm that it passes.
23. Write `crates/duet-engine/tests/soak.rs`. The whole file body sits inside `#[cfg(test)] mod tests`. The test is named `soak`, it carries `#[ignore = "long run; the nightly soak.yml workflow runs it"]`, and it runs the dummy backend in `Freewheel` for B77 cycles with a varying block size. It asserts zero `PlaybackStarved` and zero `CaptureOverflow` outcomes, and it records the longest media read and the longest media append of the run against B117 and B118.
24. Read every `zip` of the crate against the form-1 rule of section 5.11: the form is legal only where a type already proves the two slices equal, and the module that writes it names the proof in a doc comment beside it.
25. Run `cargo clippy -p duet-engine --all-targets -- -D warnings`. Fix every finding in the code.
26. Commit on the branch `chunk/c3-chain-graph-and-publications`. The native git hook runs `scripts/dod.sh`.

## Tests

| Test | File and place | What it asserts |
|---|---|---|
| `chain_migrate_keeps_the_state_of_a_kept_chain` | `src/chain/migrate.rs`, `#[cfg(test)] mod tests` | After a migration that adds one strip, the filter history, the meter hold and the gain ramp of every kept chain are the same bytes. Message: "a kept chain's state moves and is never rebuilt". |
| `chain_migrate_resets_a_slot_whose_kind_changed` | `src/chain/migrate.rs` | Rule 2: a slot whose `SlotKind` changed holds the default state of the new kind, and the pool buffer it takes is cleared. Message: "a kind change resets the slot state and clears its buffer". |
| `chain_migrate_leaves_a_vacant_position_silent` | `src/chain/migrate.rs` | A `Keep(k)` whose index is outside the old array leaves `state` at `None`, the strip is silent for the cycle, and one `EngineFault::ChainSlotVacant { index }` is pushed and then latched. Message: "a vacant position is silent, reported once, and never a panic". |
| `chain_migrate_allocates_no_ring_for_a_kept_chain` | `src/chain/migrate.rs` | `ChainLayout::pairs` grows by the added pairs alone. Message: "a migration allocates one ring pair per added chain and none for a kept chain". |
| `solo_in_place_mutes_every_strip_that_feeds_nothing_soloed` | `src/mix/solo.rs` | With one soloed track strip, `implicit_mute` is false on that strip, on the bus it feeds and on the master, and true on every other strip. Message: "solo in place mutes only what feeds nothing soloed". |
| `solo_in_place_keeps_an_explicit_mute` | `src/mix/solo.rs` | A soloed strip that is also muted is silent, and both states are readable. Message: "an explicit mute survives a solo". |
| `slot_meters_fill_the_cell_the_formula_names` | `src/mix/strip.rs` | For a topology with two strips and three slots each, the cell index of every measurement equals `strip * 2 * MAX_SLOTS + position * MAX_SLOTS + slot`. Message: "each slot measurement lands in the cell the formula names". |
| `slot_meters_report_input_and_reduction_apart` | `src/mix/strip.rs` | A compressor slot reports an `input` above zero and a `reduction` at or below zero, and a high-pass slot reports `Finite::ZERO` for both. Message: "a slot measure carries two quantities and not one". |
| `fault_run_reports_once_per_run_and_at_the_cap` | `src/chain/state.rs` | A condition that holds for 200 cycles pushes exactly one `EngineFault` when it stops, whose `frames` is the whole run; a condition that never stops pushes one fault per B136 frames. Message: "one fault per run, and one per B136 frames while the run stays open". |
| `handoff_stop_ends_the_loop_and_reports` | `src/chain/configure.rs` | `ConfigureCommand::Stop` makes the loop run `collect`, then `try_cleanup`, then send `EngineEvent::HandoffStopped`, and then return. Message: "Stop ends the loop and reports HandoffStopped". |
| `handoff_retry_waits_and_caps_at_b95` | `src/chain/configure.rs` | With the B105 slot held full, `hand_off` answers `Ok(None)` on attempts one and two and reports `EngineFault::HandoffBusy` past B95. Message: "the handoff retry waits on its own thread and caps at B95". |
| `configure_refuses_before_it_allocates` | `src/chain/configure.rs` | A request past B86, past B90, past B57 or past a pool class answers the matching `ConfigError` and `ChainLayout::pairs` does not move. Message: "configure refuses on arithmetic before it allocates a byte". |
| `port_gone_silence_clears_the_bitset_in_one_cycle` | `src/chain/graph.rs` | A `MidiMessage::PortGone` record on the B32 ring makes the cycle that reads it synthesize a note-off for every set bit of that `BoundPort` and clear the bitset. Message: "a departed port is silent inside one cycle". |
| `graph_runner_reports_generation_skew_once` | `src/chain/graph.rs` | A handoff adopted against an earlier published topology reports `EngineFault::HandoffGenerationSkew { published, adopted }` once, and the next cycle agrees and runs every strip. **This is item 14 of architecture section 10.7 rung three**, which this chunk owns. Message: "a generation skew costs one cycle and one fault". |
| `soak` | `crates/duet-engine/tests/soak.rs`, inside `#[cfg(test)] mod tests`, `#[ignore = "long run; the nightly soak.yml workflow runs it"]` | Over B77 cycles of the dummy backend in `Freewheel` with a varying block, zero `PlaybackStarved` and zero `CaptureOverflow` outcomes occur, and the longest media read and append sit inside B117 and B118. Message: "the soak run starves no cycle and overruns no capture". |

Every assert carries a message. Every test builds its System Under Test with one `fn sut(...)` that takes the topology, the channel ends and the backend double as explicit arguments. No test shares mutable state, and no test sleeps.

## Verification

1. `cargo nextest run -p duet-engine -E 'test(chain_migrate) + test(solo_in_place) + test(slot_meters) + test(fault_run) + test(handoff_stop) + test(port_gone_silence)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of any of the six names exists.
2. `cargo nextest run -p duet-engine --no-tests=fail` passes, and it runs no ignored test, so `soak` does not run in the gate.
3. `cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(soak)' --no-tests=fail` passes on a developer machine. The nightly `soak.yml` workflow, which chunk M7 writes, runs the same command.
4. `cargo clippy -p duet-engine --all-targets -- -D warnings` prints nothing.
5. `cargo doc -p duet-engine` is clean with `-D warnings`.
6. One commit on the branch `chunk/c3-chain-graph-and-publications` passes the native git hook. The commit carries `crates/duet-engine/Cargo.toml` and `Cargo.lock` together.

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
