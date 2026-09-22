# ADR-0004: The audio backend and the threading contract

## Status

Proposed.

## Context

Ardour's whole engine model rests on one rule: the backend owns the thread and calls one `process_callback(nframes)`; nothing else pulls (research section 5). That single cycle is the reference point for latency compensation, for capture alignment, and for the graph run.

Three facts from the research constrain the Rust answer.

1. **cpal has no duplex stream in 0.18.2.** Input and output are two callbacks with two clocks, and the device can refuse a fixed buffer size (survey). The duplex interface merged after that release (platform, cpal facts).
2. **`arc-swap` lets the audio thread free memory.** The reader can hold the last reference to a retired value and then run its destructor inside the callback (critic 5.1).
3. **The no-allocation rule has no mechanical guard here.** `GlobalAlloc` is an unsafe trait, `unsafe_code` is denied at the workspace level, and every member crate inherits the policy (critic 5.4).

Two more facts shape it. `[profile.release]` sets `panic = "abort"` with `overflow-checks = true`, so one overflow in the audio path loses a take. The product budget B1 is a small graph.

The operator then set the platforms: PipeWire on Linux, one Ubuntu minimum, one macOS minimum, no PulseAudio or JACK client code, no deprecated interface, and no legacy support. Specification section 11.3 and section 11.4 carry both versions. `roadmap/duet-v1/research/linux-macos-platform.md` verified what cpal 0.18.2 gives on each one, and this record is edited to those facts.

## Decision

Every number this record relies on is a row of specification section 1.6, cited by its id. Every
rule is cited by its id. This record carries decisions and consequences only.

### The backend trait

1. **One duplex `process` call per cycle, and one `Cycle` handle carries the frames.** One cycle
   gives one reference point for latency compensation and for capture alignment. Specification
   section 5.1 declares the trait.
2. **The block size varies on every cycle.** Every internal buffer is sized once to the reported
   maximum, off the audio thread, and the process body reads the frame count each cycle.
   Specification section 5.2 holds the rule.
3. **The dummy backend is the first implementation.** It is what makes a headless test, a
   deterministic engine test, and a gate with no audio server possible. Specification section 5.3
   holds each mode and what it is for.
4. **cpal satisfies the trait by making the output callback the process cycle.** The input callback
   writes into a ring that the cycle drains, and a shortfall is zero filled and counted.
   Specification section 5.4 holds the bridge.
4a. **cpal is pinned with `default-features = false, features = ["pipewire"]`.** Default features
   are empty in 0.18.2, and the `pipewire` feature is the only host feature this product wants.
4b. **The backend selects its host by identifier and never calls `default_host()`.** On Linux
   `default_host()` falls through to two hosts the operator ruled out. Linux selects the PipeWire
   host; macOS selects the CoreAudio host.
4c. **There is no ALSA host fallback.** When no PipeWire socket exists the backend reports the
   cause, the window opens, and the status bar names it. cpal's own source documents unreliable
   period sizes and continuous xruns on the ALSA host over the pipewire-alsa plugin. A fallback
   would trade a clear failure for a dropout the user cannot diagnose.
4d. **Each constraint the PipeWire host carries has a stated answer, and no answer is a fallback to
   another host.** Specification section 5.4 holds the table of constraints and answers, and
   specification section 12.4 holds the message the user reads.
5. **Version one refuses a record pass across two different devices.** With one device there is one
   clock. With two devices a drift-corrected vocal take is a silent quality loss the user did not
   ask for. The user picks a duplex device, or an Aggregate Device on macOS.
5a. **Every way the engine can fail to run carries its own cause.** Revision 5 collapsed three
   causes into one payload-free variant, so a user whose device open timed out read that PipeWire
   was missing. The no-fallback decision of 4c exists to give a clear failure, and one message for
   three causes erases it. Specification section 12.4 gives each cause its own message.
6. **The backend measures drift over the rolling window B70 and reports it.** A magnitude over the
   B70 threshold raises a fault. That measurement is the evidence that decides when to build the
   native backends.
6a. **`LatencyReport` comes from a loopback calibration that the user runs once per device, on both
   platforms.** cpal exposes callback timestamps, not a latency query, and a timestamp gives the
   callback's own jitter rather than the path through the converter and the analogue stage. The
   PipeWire reported delay is rejected for the same reason. Specification section 5.4 holds the
   measurement, its median over B74, and the key it is stored under.
6b. **With no calibration the backend applies B72, the observable latency, and guesses nothing.**
   B73 is the residual, and it is always **late**, never early. Specification section 5.4 states the
   arithmetic against B82 and the reason the sign is known. A take recorded on a default carries
   `TakeFlag::Uncalibrated`, the take list marks it, and the Record top bar offers a `Calibrate`
   action. A later calibration does not move an existing take.
6c. **A capture shortfall is reported, not only counted.** The fault travels the fault queue, and
   the user message names the silent frames.
6d. **The alignment test runs with no device.** The dummy backend reports a configured latency, the
   test records a synthetic impulse, and it asserts the region position inside B71.
6e. **Every latency subtraction is checked, and a failed subtraction is a fact.** `unwrap_or(0)` is
   forbidden here: a zero capture latency is a claim about the device, and a floor would assert that
   claim after a failed subtraction. A failure discards the calibration and raises
   `EngineFault::CalibrationInvalid`. Specification section 5.4 declares the split.
6f. **The calibration is bounded in time**, at B20 for the run and B21 for each pass. On expiry the
   run aborts, the device keeps its default, and `EngineFault::CalibrationTimeout` reaches the user.
7. **The native CoreAudio and ALSA backends are separate crates behind the same trait, and the drift
   measurement at B70 is what starts them.** Decision 6 states that condition. Until the measurement
   shows that the two-callback shape costs more than a native duplex backend saves, neither crate is
   in this plan. Specification section 1.3 and specification section 5.4 hold the same condition.

### The processor chain and the graph

8. **The chain is a typed fixed skeleton with bounded user slots.** The field order is the signal
   order. `SlotKind` is a closed enum, because plugins are out of scope, so dispatch is static and
   no `Box<dyn Trait>` appears on the audio thread.
8a. **Topology and mutable state are two types.** A published value is read through a shared
   reference and a processor needs `&mut self`, so a chain that held both could not run.
   Specification section 5.5 declares each type and names the mechanism that publishes it.
8b. **The runner signature is the rule.** Specification section 5.6 declares it.
8c. **State migrates in place, inside the cycle, with no allocation and no free.** A `Generation`
   counter marks each publication, and a slot that did not change keeps its state. **Audio-owned
   state holds no heap**, which is what keeps every migration a copy: PG26 refuses a heap-owning
   field inside an audio-owned declaration, and specification section 5.7 holds the block that rule
   reads. Specification section 5.5 holds the migration cases.
8d. **Parameter values never travel in the topology.** They travel in `ParamSnapshot`, addressed by
   `ParamId`, so a fader move publishes no topology and resets no filter.
8e. **A slot state holds no audio buffer, and one `BufferPool` per open stream holds every long
   buffer.** `GraphState` owns the pool on the audio thread, and neither published type holds one:
   `ChainTopology` is per track, so a pool field there would give one pool per track, and both
   published types derive comparison traits that a pool cannot. **The pool is built once and it
   never travels**, because every number that sizes it is fixed, so a topology change writes a slot
   index and allocates nothing. It sits inside a `PoolHandle`, a newtype with a hand-written
   `Debug`, because `missing_debug_implementations` is denied workspace-wide, `GraphState` derives
   `Debug`, and `basedrop` 0.1.3 implements `Debug` for neither of its smart pointers; that is a
   measured fact now, so no chunk may delete the wrapper. There is no free list and no run-time
   allocator.
8e.1. **The chain-state array is handed over per position, and the audio thread never resizes it
   and never copies one chain's state.** **No thread but the audio thread ever reads a
   `ChainState`**, which is why a migration is a plan and never a copy. Specification section 5.5
   holds the mechanism, section 5.6 holds the drain, and section 5.7 holds the three audio rules.
   Revision 21 restated about twenty lines of that mechanism here, against DR6, while decisions 15
   and 16c of this record applied DR6 correctly (critic N21-13).
8e.2. **One migration allocates one ring pair per ADDED chain and none for a kept chain.**
   Specification section 5.5 states the formula, names every refusal, and `configure` evaluates it
   before it allocates.
8e.3. **The handoff ring holds B105 and the audio thread drains it to empty.** What makes a plan's
   index sound is the apply-in-full rule of the drain, not the capacity; the capacity carries a
   different property, which is that a free slot proves adoption and lets the builder refuse to
   build against a stale count. Revision 21 wrote the number in words where B105 owns it, and DR3
   exempts no such form (critic N21-14). Specification section 5.6 holds the drain rule and the
   generation proof, and section 5.5 step 2 holds the slot test.
8f. **`SlotKind` holds no `PitchCorrect`.** PR 8.2 rules pitch correction out of version one, and
   `clippy::wildcard_enum_match_arm` is denied, so a variant no chunk builds would force a dead arm
   in every match.
9. **The graph runs in one topological pass on the audio thread.** The order is computed off the
   audio thread. A parallel runner enters the plan on one measurement only: the single-threaded
   runner must use more than half of B7.
10. **A topology change builds a new immutable `GraphChain` off the audio thread and publishes it.**
   A connection change never edits a live graph.

### The threading contract

11. **The audio thread runs under TH1, and this record adds no word to it.** Specification
   section 5.7 states TH1 and names the thread that owns each other duty.
11a. **No cross-boundary wait and no large allocation sits on the thread that paints, inside an
   IMMEDIATE verb.** TH3 carries the exact wording and this clause cites it rather than restating
   it; revision 15 dropped the verb qualifier here (concern N15-11). One thread
   owns `configure`, ring allocation, the published topology, the handoff producer, and the
   `basedrop` collector; it paints nothing and it runs no deferred verb. A verb answers at once and
   the completion arrives as an event. TH3 carries both halves of the rule, and specification
   section 5.5 holds the mechanism, its thread, its bounds, and its attempt count.
11b. **The collector has a period and a thread that no long verb can hold.** Specification section
   5.7 TH5 names both, and section 5.5 states that one drain releases a whole nest of deferred
   values.
12. **`triple_buffer` carries every snapshot the audio thread reads.** The reader borrows and never
   owns, so the writer thread drops every retired value. `arc-swap` is not a dependency of any Duet
   crate.
12a. **Rings are allocated when a track enters the playable set, and released when it leaves.** A
   ring is B53 per channel. `Play` allocates nothing, so no transport action carries an allocation
   pause. **Ring allocation has exactly one owner**, `GraphConfigurator::configure` on the engine
   handoff thread, under B27; the engine disk thread allocates none and releases none (TH4, critic
   C15-1). **Two enums name that refusal, and one DECLARED conversion joins them.**
   `configure` returns `ConfigError`, and `EngineError` wraps it with `#[from]`.
   `GatewayError` declares the same four budget names with the same payloads, because the gateway
   raises them from its own TH8 arithmetic before any `configure` runs.
   **`impl From<ConfigError> for GatewayError` is the one bridge**, it lives in `duet-engine`
   because section 1.3 gives `duet-command` no edge to `duet-engine`, and the `impl-sites` block
   carries its placement. **A gateway caller sees a `GatewayError` and never a `ConfigError`**, and
   specification section 12.4 gives `fault_text.rs` one table per key. Revision 6 wrote both names
   with no conversion and a reader could not tell which one `configure` returns; revisions 16 and 17
   restored that state while this clause asserted no second variant existed (critic C16-16, C17-4).
   Specification section 5.5 holds the per-state table and every refusal.
12a1. **The strip count and the parameter count each carry a bound and a refusal, and each caller
   names its own crate's error.** The two fixed arrays are sized at B86 and B90. `duet-session`
   names no engine type, because `ConfigError` wraps the mix error through `From`. Specification
   section 5.5 holds both rules and specification section 12.4 holds both messages.
12b. **Every value the audio thread publishes satisfies TH9.** On every audio-to-user-interface
   path the audio thread is the writer, and decision 12 makes the writer the dropper, so a
   published value that owned a heap allocation would make the audio thread call `free` once per
   frame. Specification section 5.8 states how many such paths there are, section 5.9 declares each
   published type, and PG13 checks the derive (critic C22I-W8).
   **The application reads a meter through `duet-core`, never through an engine type**:
   specification section 15.14 declares the reader that resolves a held mark, and specification
   section 7.3 holds the meter path and the reset generations. This record states no second copy of
   either (DR6).
13. **`rtrb` is the one ring buffer crate, and every consumer gets its own ring.** The MIDI thread
   writes each record twice, once for the monitor voice and once for note entry. A duplicated record
   is cheaper than a second thread, and the two consumers then have independent backpressure.
   Specification section 5.8 holds every boundary and the bound of each one.
13a. **The MIDI-to-sound path never crosses the GPUI thread, in every mode.** B9 covers exactly that
   path.
13b. **`MidiRecord` and `NoteEntry` are declared in `duet-command`, and each one carries a
   `PortSlot`.** `crates/duet` never names one: the core drains the note queue and emits an event.
   **Neither record carries a `MidiPortId`.** That identity owns a heap allocation, so it would
   refuse the `Copy` derive and put an allocation and a free on the B9 path, which TH10 forbids.
   `PortSlot` is the fixed-size handle that `MidiPortMap` mints, and the map turns it back into a
   name off the audio thread. Specification section 8.1 declares both types.
13c. **The MIDI thread is the one owner of `MidiPortMap`, and the presence path is bounded end to
   end.** A `HotplugEvent` carries a whole `MidiPortInfo`, so no other thread needs the map and no
   shared reference crosses a boundary. The path runs through two bounded queues, at B87 and at
   B100, and **`duet-core` drains the second one**; specification section 8.1 holds every step
   and names the producer and the consumer of each queue. On overflow either queue keeps the newest
   value and sets a resync flag, because a presence event is a state and a note is an event.
13d. **A bound MIDI input that leaves during a record pass never cancels the take.** The audio path
   has `DeviceLost`, and the MIDI path now has the same shape. Specification section 8.2 holds the
   steps and the owner of each one.
14. **One primitive carries each boundary, and the overflow action decides which one.** `rtrb`
   carries every boundary whose producer only pushes, and `crossbeam_queue::ArrayQueue` carries
   every boundary that must evict with `force_push`. A drop counter is the backpressure signal on
   both. Specification section 5.8 holds each boundary, its primitive, its bound, and its overflow
   rule.
15. **`basedrop` carries any owned value the audio thread must release** (TH5). Specification
   section 5.7 names the thread its collector runs on.
16. **Structural traffic and high-rate traffic use different mechanisms.** Commands and events are
   bounded, ordered, and versioned; an overflow forces a resynchronize, never a silent drop. The
   playhead and the meters are latest-value snapshots. Each publication has exactly one read end,
   `duet-core` owns it, and the host entity reads both once per frame before any child view
   renders. Specification section 10.2 holds that frame order.
17. **Arithmetic on the audio path is checked or saturating**, so one overflow cannot abort the
   process.
17a. **Every cross-boundary wait has a timeout**, and specification section 5.12 holds the table.
   Each row names a B id and the action on expiry.
17b. **Every audio file read and write goes through `duet-media`.** `duet-engine` names no format
   crate. A take promotes to the larger container at B67.
17c. **The engine disk thread serves the engine only** (TH4). Every other file operation in the
   product runs off the thread that paints, and specification section 5.7 names each thread that
   may perform one; this record states no second copy of that table (DR6). Revision 22 wrote that
   every other file operation runs on a `JobRunner` thread, which forbade the `duet-quit-save`
   thread the same revision added and would have put the quit-path `fsync` back on the thread that
   paints (critic C22I-W9).
18. **Verification of decision 11 is by the named means of specification section 5.7**, and
   this record claims no more. The soak test runs for B77 over the dummy backend.

### The platform

19. **The gate runs on one Linux runner and one macOS runner, with the dummy backend and no daemon.**
   Specification section 11.6 holds every rule that keeps the gate free of hardware, and
   specification section 11.5 holds the build prerequisites of each platform.
20. **A separate `audio-smoke` job proves the real backend once per run**, on the Linux runner
   only, because a macOS runner offers no virtual audio device. It **waits for the PipeWire socket under
   B85 and fails the job on expiry**, and its test is `#[ignore]` by default, so the gate never runs
   it. The workflow belongs to the first manifest chunk that runs after the cpal backend lands,
   because a filter with no match is a pass nobody earned. Specification section 11.6 holds the
   steps.
21. **Wayland is the Linux display target.** `gpui-kit` 0.6.4 exposes no feature that disables the
   `x11` back end of its `gpui-pre` dependency, so the binary links both and the release notes state
   that X11 is untested. Specification section 11.3 holds the decision.
22. **The macOS minimum and the Ubuntu minimum are decisions of this plan**, and specification
   section 11.4 and section 11.3 state each version and name the places that carry it. DR6 asks for
   the citation alone, so **no runner label appears anywhere in this record**, and no version of a
   PLATFORM does either (critic C-12, concern N-4). A version PIN of a third-party crate is DR3
   exemption 1, a fact about an artifact rather than a budget this design chooses, and this record
   names some of them; **the pins are the fact and no sentence counts them** (concern N15-10, critic
   N21-12). Revision 15 wrote the absolute and broke it four lines later, and revision 21 wrote
   "four" over three crates.

## Consequences

Easier:

- One cycle gives one reference point, so capture alignment and latency compensation are exact.
- The dummy backend makes every engine test, every export test, and the headless host possible with
  no hardware. It is also what lets the gate run on a runner with no audio server.
- A closed processor enum removes heap allocation and dynamic dispatch from the audio thread by
  construction.
- A `triple_buffer` reader cannot free, so the hardest real-time bug class is removed by type.
- A varying block size in the dummy backend finds a fixed-size assumption in the test suite.
- One pool per open stream, owned by the thread that reads it and rebuilt never, removes the
  second writer, the per-track duplication, and the doubled transient that three earlier revisions
  carried.
- A migration that hands over a plan rather than a set keeps every kept chain's filter history and
  meter hold by construction, so a user who adds one send resets nothing else.
- One thread that paints nothing owns every allocation of a topology change, so an immediate verb
  answers at once whatever the project size is.
- Naming the host by identifier makes the Linux audio path a decision a reader can check, rather
  than whatever `default_host()` found.

Harder:

- The cpal backend carries a ring bridge and a shortfall counter that a true duplex backend would
  not need.
- A two-device record pass is refused in version one, and the native backends are the answer.
- `triple_buffer` publishes by value, so a tempo-map edit copies the map. The map is small.
- The no-allocation rule holds by review, a soak test, and a profiler. This record claims no more.
- The crates specification section 1.2 marks impure carry the real-time discipline, and each one has a single named job. Specification
  section 1.2 lists them in the `duet-engine` row.
- The loopback calibration is a setup step the user must run, and a user who skips it records with
  an overestimate. The flag, the take marker, and the top bar alert make the cost visible.
- A track holds every idle processor state whether or not it uses them. That is what removes
  allocation from the topology change.
- One more thread exists, with its own loop, its own deadline, and its own retry. That thread is a
  cost this record accepts, because the alternative put a large allocation on the thread that
  paints. Specification section 5.7 holds the thread table and states how many threads the engine
  and the backend own; this record states no count of its own (DR6, critic N22I-7).
- One handoff is in flight at a time, so a burst of topology verbs lands one audio cycle apart.
  `DuetCore::pending_configure` keeps the latest and re-sends it, so nothing is lost and the user
  sees the document change at once.
- The PipeWire host gives one sample rate, so a project at another rate needs a user decision that a
  multi-rate server would not have asked for.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| cpal first, with no backend trait | No dummy backend means no headless test, no deterministic engine test, and no gate on a runner with no audio server. |
| `default_host()` on Linux | It falls through to PulseAudio and then to the ALSA host, and the operator ruled both out. A silent fallback is also a dropout the user cannot diagnose. |
| The cpal ALSA host over the pipewire-alsa plugin | cpal's own source documents unreliable period sizes and continuous xruns on that path. |
| JACK as the first backend | It makes a server a runtime prerequisite that the minimum Ubuntu release does not ship, and the operator ruled out JACK client code. |
| The `pipewire` crate as the audio backend, in place of cpal | cpal's PipeWire host wraps the same server behind a safe API this plan already depends on, so a second binding buys nothing. **The stream path of the `pipewire` crate is safe Rust**, which `research/linux-macos-platform.md` records. **The two research files AGREE** (critic N21-16): `research/crate-survey.md` now records the caller path as safe, and `research/linux-macos-platform.md` is the file written against the crate's own stream API. Revision 17 said both sources refute the unsafe claim while one source still stated it, and revision 21 recorded a disagreement that the survey had already corrected. The decision does not rest on the disagreement: cpal's PipeWire host already wraps the same server, so a second binding buys nothing whichever file is right. The crate still serves the MIDI registry listener. |
| Two independent callbacks exposed to the engine | There is no cycle start, so latency compensation has no reference point and capture alignment becomes a guess. |
| `arc-swap` for the audio snapshots | The audio thread can hold the last reference and run the destructor inside the callback. |
| `arc-swap` plus `basedrop` for the snapshots | It works, and it needs a second smart pointer through every published type. `triple_buffer` removes the ownership from the reader altogether. |
| `ringbuf` instead of `rtrb` | Both fit. The smaller surface is the smaller review surface. |
| A work-stealing graph executor in version one | The stated budget is small. A pool adds thread wake latency and a class of bugs that graph does not pay for. |
| `Arc<Mutex<EngineState>>` shared with the user interface | A lock on the audio thread breaks the real-time rule outright. |
| GPUI's `spawn_realtime` and `Priority::RealtimeAudio` for the engine threads | It would make `duet-engine` depend on `gpui-kit`, which stops the headless host from running the engine. |
| Publish the processors inside the chain, as the first draft did | A published value is read through a shared reference and a processor needs `&mut self`, so it could not run. |
| Put the `BufferPool` inside `ChainTopology`, as revision 4 did | `ChainTopology` is per track, so the project would hold one pool per track, and the type derives `Clone` and `Eq` that a pool cannot. |
| A free list inside the pool | `configure` would assign a slot off the audio thread while migration returned one on it, which gives one list two writers. The only safe mechanisms are a lock or an atomic, and decision 11 forbids both. |
| Reset every processor on a topology change | A fader move or a new send would then click. The generation check plus the migration cases of section 5.5 keep an unchanged slot untouched. |
| Derive `LatencyReport` from cpal callback timestamps, or from the PipeWire reported delay | A timestamp gives the callback's own jitter, not the path through the converter and the analogue stage. |
| Resample a take to correct a two-device drift | A drift-corrected vocal take is a silent quality loss the user did not ask for. |
| `unwrap_or(0)` on the calibration split | A zero capture latency is a claim about the device, and every take would then be misaligned with no signal that anything went wrong. |
| Keep the audio buffers inside the slot states | A long delay line per slot on every track is hundreds of megabytes, and a topology change would either free on the audio thread or keep every buffer forever. |
| One MIDI ring with two consumers | `rtrb` is strictly single producer and single consumer, so this does not compile. |
| An in-house counting allocator to prove the no-allocation rule | It needs `unsafe impl GlobalAlloc`, which the workspace policy denies. |
| Resample on playback when the project rate and the server rate differ | It puts a resampler on the real-time path for a case the user can fix once, in the server or in the project. |
