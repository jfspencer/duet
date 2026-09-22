---
id: C4
line: C
depends_on: [C3, D1, M7]
write_scope:
  - crates/duet-engine/src/cpal/host.rs
  - crates/duet-engine/src/cpal/stream.rs
  - crates/duet-engine/src/cpal/calibrate.rs
  - crates/duet-engine/tests/alignment.rs
  - crates/duet-engine/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "macOS, on macos-26: cargo nextest run -p duet-engine -E 'test(calibration_split)' --no-tests=fail passes. Linux, on ubuntu-26.04: cargo nextest run -p duet-engine -E 'test(calibration_split) + test(host_select)' --no-tests=fail passes; commit SHA on a branch chunk/c4-cpal-backend-and-calibration"
---

# C4: The cpal backend, the host selection, the calibration split, and the alignment test

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the one real backend: cpal 0.18.2 with `default-features = false, features = ["pipewire"]`, the PipeWire host selection by identifier on Linux and the CoreAudio host on macOS, the input ring bridge with `CpalBridge` as its declared pair of ends, the calibration split, the drift report, the `pipewire_smoke` test, and the alignment test. Architecture sections 5.4, 5.12 and 11 decide it, ADR 0004 clauses 4, 4a, 4b, 4c, 4d, 5, 5a, 6, 6a to 6f decide the backend and the calibration, and `roadmap/duet-v1/research/linux-macos-platform.md` supplies every verified platform fact. It carries MUST story R-13 (the audio device disappears) and the backend half of R-04 and R-07.

**This chunk is platform specific, so it names a per-platform command and the runner of each** (SM3 rule 3). `host_select` is the PipeWire host selection and it carries `#[cfg(target_os = "linux")]`, so it exists on one platform alone.

## Files

- `crates/duet-engine/src/cpal/host.rs` — modify. Chunk C1 created the stub.
- `crates/duet-engine/src/cpal/stream.rs` — modify.
- `crates/duet-engine/src/cpal/calibrate.rs` — modify.
- `crates/duet-engine/tests/alignment.rs` — create. SM2 covers `src/` only, so this chunk creates the integration test file and names it in its own write scope.
- `crates/duet-engine/Cargo.toml` — modify. Add the `cpal` entry.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_engine::cpal::host` (architecture 5.4, 11.1)

The consumer line the root manifest pins is `cpal = { version = "0.18.2", default-features = false, features = ["pipewire"] }`. `duet-engine` carries no `cfg(target_os)` of its own, because cpal covers both hosts behind one API; the host **selection** is data.

| Platform | Host | When the host is absent |
|---|---|---|
| Linux | `HostId::PipeWire` | `BackendError::NoServer`, which the core maps to `EngineState::NoServer { detail }`. |
| macOS | `HostId::CoreAudio` | `BackendError::NoServer`, which cannot happen on a supported macOS. |

**The host is selected by identifier, never by `default_host()`.** There is no ALSA host fallback.

```rust
/// The one real backend of version one.
///
/// It selects its host by identifier. On Linux the identifier is
/// `HostId::PipeWire` and on macOS it is `HostId::CoreAudio`, which is the
/// table section 5.4 states; `default_host()` is never called, because on
/// Linux it falls through to two hosts the operator ruled out.
pub struct CpalBackend { /* fields the implementation needs */ }

impl AudioBackend for CpalBackend {
    fn name(&self) -> BackendName;
    fn devices(&self) -> Result<Vec<DeviceInfo>, BackendError>;
    fn supported_sample_rates(&self, device: &DeviceId) -> Result<Vec<SampleRate>, BackendError>;
    fn supported_block_sizes(&self, device: &DeviceId) -> Result<BlockSizeRange, BackendError>;
    fn open(&mut self, request: &StreamRequest, process: Box<dyn AudioProcess>)
        -> Result<StreamInfo, BackendError>;
    fn start(&mut self) -> Result<(), BackendError>;
    fn stop(&mut self) -> Result<(), BackendError>;
    fn latency(&self) -> Result<LatencyReport, BackendError>;
    fn clock(&self) -> SampleClock;
}
```

The host selection function carries a doc comment that records the pin: socket detection without an environment variable landed after cpal 0.18.2, so the backend sets no environment variable and accepts the pinned behaviour.

### `duet_engine::cpal::stream` (architecture 5.4, 5.8, 15.10)

cpal has no duplex stream in 0.18.2, so input and output are two callbacks with two clocks. **The output callback is the process cycle.** The input callback writes each channel into an `rtrb` ring, and the output callback drains up to `cycle.frames()` frames from that ring into the capture buffers before each cycle.

```rust
/// The INPUT callback's end of the cpal input-to-output sample ring
/// (section 5.4, section 5.8).
/// **Audio-owned** (section 5.7).
pub struct CpalInput {
    /// Which input channel this end carries.
    channel: ChannelIndex,
    /// The input callback's half of the ring.
    write: Owned<Producer<f32>>,
}

impl core::fmt::Debug for CpalInput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CpalInput").finish_non_exhaustive()
    }
}

/// The output callback's half of the same ring (section 5.4, section 5.8).
/// **Audio-owned** (section 5.7).
pub struct CpalOutput {
    /// Which input channel this end carries.
    channel: ChannelIndex,
    /// The output callback's half of the ring.
    read: Owned<Consumer<f32>>,
}

impl core::fmt::Debug for CpalOutput {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("CpalOutput").finish_non_exhaustive()
    }
}

/// The declared pair of ends of the cpal input bridge.
///
/// `cpal::Device::build_input_stream` and `build_output_stream` each take
/// their own `FnMut` closure and each closure owns what it captures, and an
/// `rtrb` end is `Send` and not `Sync`, so one value cannot sit in both.
/// **One of each exists per input channel**, which is the count the section
/// 5.8 row states.
#[derive(Debug)]
pub struct CpalBridge {
    inputs: ArrayVec<CpalInput, MAX_DISK_RINGS>,
    outputs: ArrayVec<CpalOutput, MAX_DISK_RINGS>,
}
```

Each end sits inside a `basedrop::Owned`, so the free happens on the collector thread (TH5). Both callbacks are real-time callbacks the backend owns, and TH1 binds each word for word.

The cpal error callback holds one `basedrop::Shared<FaultQueue>` clone and nothing else (TH11). It classifies the `ErrorKind` cpal hands it and pushes one `EngineFault`. An xrun arrives as `ErrorKind::Xrun` and pushes `EngineFault::PlaybackStarved`. **This push is not latched and it cannot be**: the callback holds no `ChainState`, so it has no per-track run to open. B34 contains a device that xruns repeatedly: the queue refuses the newest record and the drain mints `EngineFault::FaultsDropped`.

The PipeWire host constraints and their answers (section 5.4):

| Fact | What the backend does |
|---|---|
| `BufferSize::Fixed(n)` is validated against the server quantum range and set as `node.latency`; an out-of-range value returns `UnsupportedConfig` | The backend requests the configured block. On `UnsupportedConfig` it falls back to `BufferSize::Default`, reads the real block from `StreamInfo`, and emits `EngineFault::BlockSizeAdjusted { requested, actual }`. |
| The host offers one sample rate, the graph rate | `supported_sample_rates` returns that one value. When the project rate differs, `open` returns `BackendError::RateUnavailable { project, device }`. |
| An xrun arrives as `ErrorKind::Xrun` | The error callback pushes `EngineFault::PlaybackStarved` into the B34 `FaultQueue`. |

### `duet_engine::cpal::calibrate` (architecture 5.4, 15.10)

```rust
/// Split a measured round trip into a capture part and a playback part.
///
/// # Errors
/// Returns `BackendError::CalibrationInvalid` when the measured round trip
/// is below the playback part, which means the measurement is not a
/// physical round trip.
pub fn split(round_trip: u32, block: u32, output_buffer: u32)
    -> Result<LatencyReport, BackendError>
{
    let playback = block.checked_add(output_buffer)
        .ok_or(BackendError::CalibrationInvalid)?;
    let capture = round_trip.checked_sub(playback)
        .ok_or(BackendError::CalibrationInvalid)?;
    Ok(LatencyReport::new(capture, playback, block))
}

/// The report to use when no calibration exists.
///
/// It reports the observable part and nothing else: one block plus the
/// reported output buffer as playback latency, and zero capture latency.
/// It guesses no round trip, so it cannot be too large.
#[must_use]
pub fn observable_only(block: u32, output_buffer: u32) -> LatencyReport;
```

`unwrap_or(0)` is not allowed here. A `None` discards the calibration, falls back to `CalibrationSource::Default`, and raises `EngineFault::CalibrationInvalid`.

The measurement plays a short chirp, records the input, and finds the offset by cross-correlation in `duet_dsp::align`. It runs B74 passes and takes the median. The whole run is bounded at B20 and each pass at B21; on expiry the run aborts, the device keeps `CalibrationSource::Default`, and the backend raises `EngineFault::CalibrationTimeout`.

The drift measurement runs over the window B70 even in the one-device case, and a magnitude over the B70 threshold raises `EngineFault::ClockDrift { parts_per_million }`.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `align` cross-correlation | `duet-dsp` | `duet_dsp::align` |
| `Calibration`, `CalibrationSource`, `DeviceKey` | `duet-session` | `duet_session` |
| `EngineFault`, `EngineState`, `ServerDetail`, `DeviceLabel` | `duet-command` | `duet_command` |
| `AudioBackend`, `AudioProcess`, `LatencyReport`, `DriftReport`, `BackendError`, `StreamRequest`, `StreamInfo`, `DeviceInfo`, `DeviceId`, `BlockSizeRange`, `BackendName` | `duet-engine` | chunk C1 |
| `FaultQueue`, `EngineProcess` | `duet-engine` | chunk C3 |

Link `D1 before C4` of section 13.4 states the reason for the `duet-dsp` edge: the loopback calibration uses `duet_dsp::align`.

## Steps

1. Read every file of the write scope and `crates/duet-engine/Cargo.toml`. Confirm the stubs chunk C1 created, the ring types chunk C2 wrote, and the `FaultQueue` and `EngineProcess` chunk C3 wrote. Confirm that chunk M7 has landed. Report a discrepancy and stop.
2. Add to `crates/duet-engine/Cargo.toml` the entry `cpal = { workspace = true }`. The root pin is `cpal = { version = "0.18.2", default-features = false, features = ["pipewire"] }`, which chunk M4 wrote. Run `cargo build --workspace` on both platforms and keep `Cargo.lock` for the same commit.
3. Write the failing test `calibration_split_refuses_a_round_trip_below_the_playback_part` in `src/cpal/calibrate.rs`. Run `cargo nextest run -p duet-engine -E 'test(calibration_split)' --no-tests=fail` and confirm that it fails to compile.
4. Write `split` and `observable_only` in `src/cpal/calibrate.rs`, exactly as the declarations above. Run the test and confirm that it passes.
5. Write the failing test `calibration_split_returns_the_two_parts`. Run it and confirm that it fails, then implement and confirm that it passes.
6. Write the loopback measurement in `src/cpal/calibrate.rs`: the chirp, the cross-correlation through `duet_dsp::align`, the median over B74 passes, the B20 run bound and the B21 pass bound, and the two faults `EngineFault::CalibrationInvalid` and `EngineFault::CalibrationTimeout`.
7. Write the drift measurement over the B70 window, with `DriftReport` and the `EngineFault::ClockDrift` threshold.
8. Write the failing test `host_select_picks_pipewire_by_identifier` in `src/cpal/host.rs`, with `#[cfg(target_os = "linux")]`. Run `cargo nextest run -p duet-engine -E 'test(host_select)' --no-tests=fail` on Linux and confirm that it fails.
9. Write `CpalBackend` and the host selection in `src/cpal/host.rs`. On Linux it selects `HostId::PipeWire`; on macOS it selects `HostId::CoreAudio`. `default_host()` is never called. When no PipeWire socket exists the backend returns `BackendError::NoServer(ServerDetail::SocketMissing)` and never falls back to the ALSA host. Run the Linux command and confirm that it passes.
10. Write `CpalInput`, `CpalOutput` and `CpalBridge` in `src/cpal/stream.rs`, with the hand-written `Debug` impls. Implement the two-callback bridge: the input callback writes each channel into its ring, and the output callback drains up to `cycle.frames()` frames before it runs the process cycle. A cycle whose input ring holds fewer frames than `cycle.frames()` zero fills the difference and adds the zero-filled frames to `ChainState::shortfall`.
11. Implement the cpal error callback under TH11: one `basedrop::Shared<FaultQueue>` clone, one `ErrorKind` classification, one `EngineFault` push, and no other value of the engine.
12. Implement the three PipeWire host answers of the table above, including the `BufferSize::Fixed` fallback with `EngineFault::BlockSizeAdjusted` and the `BackendError::RateUnavailable` refusal.
13. Write the `pipewire_smoke` test in `src/cpal/stream.rs`, inside a `#[cfg(test)] mod tests`. It carries `#[ignore = "needs a running PipeWire daemon; the audio-smoke.yml workflow runs it"]`. It opens one PipeWire stream through the real backend, runs one cycle, asserts `CycleOutcome::Ran`, and closes the stream. Chunk M8 writes `audio-smoke.yml`, which selects it.
14. Write `crates/duet-engine/tests/alignment.rs`. The whole file body sits inside `#[cfg(test)] mod tests`. The test runs the dummy backend with a configured `LatencyReport`, records a synthetic impulse, commits the take, and asserts that the region position is inside B71.
15. Run `cargo clippy -p duet-engine --all-targets -- -D warnings` on macOS and on Linux. Fix every finding in the code.
16. Commit on the branch `chunk/c4-cpal-backend-and-calibration`. The native git hook runs `scripts/dod.sh`.

## Tests

| Test | File and place | Platform | What it asserts |
|---|---|---|---|
| `calibration_split_refuses_a_round_trip_below_the_playback_part` | `src/cpal/calibrate.rs`, `#[cfg(test)] mod tests` | Both | `split(1, 128, 256)` returns `Err(BackendError::CalibrationInvalid)` and the process does not end. Message: "a round trip of one frame is not a physical round trip". |
| `calibration_split_returns_the_two_parts` | `src/cpal/calibrate.rs` | Both | For a round trip of 700, a block of 128 and an output buffer of 256, `capture` is 316 and `playback` is 384. Message: "the split returns the capture part and the playback part". |
| `calibration_split_observable_only_guesses_no_round_trip` | `src/cpal/calibrate.rs` | Both | `observable_only(128, 256)` reports zero capture latency and a playback latency of 384. Message: "the observable report guesses no round trip". |
| `calibration_run_takes_the_median_of_b74_passes` | `src/cpal/calibrate.rs` | Both | One outlying pass among B74 does not move the result. Message: "the run takes the median, so one bad pass does not set the value". |
| `calibration_run_expiry_keeps_the_default` | `src/cpal/calibrate.rs` | Both | A run that passes B20 aborts, the device keeps `CalibrationSource::Default`, and `EngineFault::CalibrationTimeout` is raised. Message: "a calibration past B20 keeps the default and reports a timeout". |
| `host_select_picks_pipewire_by_identifier` | `src/cpal/host.rs`, `#[cfg(target_os = "linux")]` | Linux | The backend asks cpal for `HostId::PipeWire` and never calls `default_host()`. Message: "the Linux backend selects the PipeWire host by identifier". |
| `host_select_reports_no_server_with_no_socket` | `src/cpal/host.rs`, `#[cfg(target_os = "linux")]` | Linux | With no PipeWire socket the backend returns `BackendError::NoServer(ServerDetail::SocketMissing)` and selects no other host. Message: "no PipeWire socket reports NoServer and never falls back to ALSA". |
| `cpal_bridge_zero_fills_a_short_input_ring` | `src/cpal/stream.rs` | Both | A cycle whose input ring holds fewer frames than `cycle.frames()` zero fills the difference and adds the frames to the shortfall run. Message: "a short input ring is zero filled and counted". |
| `cpal_block_size_fallback_reports_the_adjustment` | `src/cpal/stream.rs` | Both | An `UnsupportedConfig` on `BufferSize::Fixed` makes the backend fall back to `BufferSize::Default` and emit `EngineFault::BlockSizeAdjusted { requested, actual }`. Message: "a refused block size falls back and reports the adjustment". |
| `pipewire_smoke` | `src/cpal/stream.rs`, `#[ignore = "needs a running PipeWire daemon; the audio-smoke.yml workflow runs it"]` | Linux | One PipeWire stream opens through the real backend, one cycle returns `CycleOutcome::Ran`, and the stream closes. Message: "one real PipeWire cycle runs and the stream closes". |
| `alignment` | `crates/duet-engine/tests/alignment.rs`, inside `#[cfg(test)] mod tests` | Both | With a configured dummy latency, a synthetic impulse commits to a region whose position is inside B71 of the expected frame. Message: "the committed region lands inside the B71 tolerance". |

Every assert carries a message. The `MidiPresence` and `MidiStream` doubles of section 11.6 are not needed here; a test that needs a real device carries `#[ignore]` with a reason string that names the device it needs, and the gate never runs it.

## Verification

1. **macOS, on `macos-26`:** `cargo nextest run -p duet-engine -E 'test(calibration_split)' --no-tests=fail` passes. It fails before this chunk, because no test of that name exists.
2. **Linux, on `ubuntu-26.04`:** `cargo nextest run -p duet-engine -E 'test(calibration_split) + test(host_select)' --no-tests=fail` passes. `host_select` carries `#[cfg(target_os = "linux")]`, so a single command over both platforms would let a macOS run pass on `calibration_split` alone and prove nothing about the Linux half.
3. `cargo nextest run -p duet-engine --no-tests=fail` passes on both runners, and it runs no ignored test.
4. `cargo clippy -p duet-engine --all-targets -- -D warnings` prints nothing on both platforms.
5. `cargo machete` reports no unused dependency of `duet-engine`.
6. One commit on the branch `chunk/c4-cpal-backend-and-calibration` passes the native git hook. The commit carries `crates/duet-engine/Cargo.toml` and `Cargo.lock` together.

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
