---
id: C2
line: C
depends_on: [C1, N1]
write_scope:
  - crates/duet-engine/src/disk/reader.rs
  - crates/duet-engine/src/disk/writer.rs
  - crates/duet-engine/src/disk/ring.rs
  - crates/duet-engine/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-engine -E 'test(disk_ring) + test(capture_done)' --no-tests=fail passes; commit SHA on a branch chunk/c2-disk-thread-and-rings"
---

# C2: The ring types, the engine disk thread, and the capture report

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the disk path of `duet-engine`: the `rtrb` ring TYPES, the engine disk thread with `EngineDisk` as its own root, `DiskTake` and `DiskSource` with the take writer, the source reader and the peak builder of architecture section 5.10, the disk reader and the disk writer, the `Output<PlaybackPlan>` read end and the B33 consumer, `EngineLink` and the B138 channel with its B147 pending list, and the one `CaptureInfo` per armed track that the thread reports on it (sections 5.8, 5.10, 6.6, 15.10). **It allocates no ring**: it consumes the ends that `configure` hands it, and chunk C3 builds the allocation with `GraphConfigurator` (TH4). ADR 0004 governs the threading contract. It carries MUST stories R-07 and R-08 (the disk writer half) and X-02 (the refill half). Sections 12.1 and 12.4 decide the error path.

`duet-media` supplies `SourceReader` and `TakeWriter`, and chunk N1 wrote both. Link `N1 before C2` of section 13.4 states the reason: the disk path reads and writes through `duet-media`.

## Files

- `crates/duet-engine/src/disk/reader.rs` — modify. Chunk C1 created the stub.
- `crates/duet-engine/src/disk/writer.rs` — modify.
- `crates/duet-engine/src/disk/ring.rs` — modify.
- `crates/duet-engine/Cargo.toml` — modify. Add the `rtrb`, `triple_buffer`, `basedrop`, and `async-channel` entries.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_engine::disk::ring` (architecture 15.10, 5.8)

```rust
/// A request from the audio thread for more playback frames.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefillRequest { track: TrackId, channel: ChannelIndex, from: SampleClock, frames: FrameCount }

/// The disk reader's per-track playback path.
/// **Audio-owned** (section 5.7).
pub struct DiskReader {
    track: TrackId,
    channels: ChannelIndex,
    /// The playback ring, channel major, B53 long per channel.
    samples: Owned<Consumer<f32>>,
    position: SampleClock,
}

impl core::fmt::Debug for DiskReader {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DiskReader").finish_non_exhaustive()
    }
}

/// The disk writer's per-track capture path. It follows the `DiskReader`
/// rule: the ring end sits inside a `basedrop::Owned` and the `Debug` impl is
/// written by hand.
/// **Audio-owned** (section 5.7).
pub struct DiskWriter {
    track: TrackId,
    take: TakeId,
    channels: ChannelIndex,
    /// The capture ring, channel major, B53 long per channel.
    samples: Owned<Producer<f32>>,
    written_frames: u64,
}

impl core::fmt::Debug for DiskWriter {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DiskWriter").finish_non_exhaustive()
    }
}

/// The write end of the engine-to-core event channel at B138.
#[derive(Debug, Clone)]
pub struct EngineLink { events: Sender<EngineEvent> }

/// One span of one source on the timeline, as the DISK thread reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlaybackSpan {
    source: SourceHash,
    /// The first timeline frame this span covers.
    at: SampleClock,
    /// The first frame inside the source.
    offset: u64,
    frames: FrameCount,
    /// The region gain, already combined with the take gain.
    gain: Finite,
    /// The fade lengths in frames. The disk reader applies both before the
    /// ring, which is why no chain stage carries a region envelope.
    fade_in: FrameCount,
    fade_out: FrameCount,
}

/// Every span one track plays, in timeline order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackPlayback { track: TrackId, spans: Box<[PlaybackSpan]> }

/// The whole playable set, as the disk thread reads it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaybackPlan { generation: Generation, tracks: Box<[TrackPlayback]> }
```

`Owned<T>` is `basedrop::Owned`, `Producer<T>` and `Consumer<T>` are `rtrb`, and `Sender<T>` is `async_channel::Sender`. `basedrop` 0.1.3 implements `Debug` for neither `Owned<T>` nor `Shared<T>`, and `missing_debug_implementations` is denied, so each declaration above writes `Debug` by hand. **No chunk may delete a wrapper.**

### `duet_engine::disk` root (architecture 15.10, 6.6)

```rust
/// The engine disk thread's own state, and the ROOT of that thread.
///
/// **It is not audio-owned.** This thread allocates, it opens files, and it
/// blocks on `fsync`, which is why B117 and B118 bound its two file calls and
/// why the ring ends it holds carry no `basedrop` wrapper: a free on this
/// thread is legal and a free on the audio thread is not (TH1, TH4).
pub struct EngineDisk {
    /// The read end of the published playback plan (section 5.8, 5.10).
    plan: Output<PlaybackPlan>,
    /// One record per armed track, at B146.
    takes: ArrayVec<DiskTake, MAX_DISK_TRACKS>,
    /// One record per playing track, at B146.
    sources: ArrayVec<DiskSource, MAX_DISK_TRACKS>,
    /// The single consumer of the B33 refill ring from the audio thread.
    requests: Consumer<RefillRequest>,
    /// The disk half of every playback ring, at B140.
    fills: ArrayVec<Producer<f32>, MAX_DISK_RINGS>,
    /// The disk half of every capture ring, at B140.
    drains: ArrayVec<Consumer<f32>, MAX_DISK_RINGS>,
    /// The write end of the B138 engine event channel. Every `CaptureInfo`
    /// this thread reports travels on it (section 6.6).
    events: EngineLink,
    /// Every event `try_send` refused, at B147. This thread retries them at
    /// the head of its next pass, exactly as `GraphConfigurator` does.
    pending_events: ArrayVec<EngineEvent, MAX_PENDING_EVENTS>,
}

/// The engine disk thread's record for ONE armed track.
pub struct DiskTake {
    track: TrackId,
    /// The take file this pass appends to. B118 bounds one append and its
    /// flush, and `TakeWriter::finish` writes the container header, which is
    /// the step that makes a partial take readable (section 9.5 step 3).
    writer: TakeWriter,
    /// The peak pyramid this pass builds beside the take (section 5.10).
    peaks: PyramidBuilder,
    /// What this pass has recorded so far. It becomes the one `CaptureInfo`
    /// this thread sends when the pass ends (section 6.6).
    info: CaptureInfo,
}

/// The engine disk thread's record for ONE playing track.
pub struct DiskSource {
    track: TrackId,
    /// The source file the refill path reads. B117 bounds one block read.
    reader: SourceReader,
    /// Where the last refill of this track ended.
    position: SampleClock,
}
```

Each of `EngineDisk`, `DiskTake` and `DiskSource` writes `Debug` by hand with `finish_non_exhaustive`, for the reason `DiskReader` states.

### `duet_engine` crate root, from section 6.6

```rust
/// What one capture pass produced, for ONE track.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CaptureInfo { track: TrackId, start: SuperClock, frames: u64, loop_offset: u64, underruns: u32 }
```

`CaptureInfo` carries **no** `TakeFlags`. Section 6.4 names five observers of a flag, and four of them are the backend, the audio thread and `duet-midi`; the CORE assembles `TakeFlags` in `DuetCore::take_flags`, which chunk I2 writes.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `SourceReader`, `TakeWriter`, `MediaError` | `duet-media` | `duet_media` |
| `PyramidBuilder`, `PeakBin`, `PyramidHeader` | `duet-dsp` | `duet_dsp::peaks` |
| `SourceHash`, `TrackId`, `TakeId` | `duet-session` | `duet_session` |
| `SampleClock`, `SuperClock`, `FrameCount`, `ChannelIndex`, `Finite` | `duet-time` | `duet_time` |
| `EngineEvent`, `EngineFault` | `duet-engine` | `duet_engine` (chunk C1 and this chunk) |
| `MAX_DISK_TRACKS` (B146), `MAX_DISK_RINGS` (B140), `MAX_PENDING_EVENTS` (B147) | `duet-time` | `duet_time` |

`EngineEvent` is declared by this chunk in `src/disk/ring.rs` beside `EngineLink`, because the two ends and the message belong to one carrier row (TH13). Chunk C3 adds the arms its own thread sends:

```rust
/// One fact the engine reports on the structural channel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EngineEvent {
    Fault(EngineFault),
    StateChanged(EngineState),
    Latency(LatencyReport),
    Drift(DriftReport),
    CaptureDone(CaptureInfo),
    GraphConfigured { generation: Generation },
    ConfigureRefused(ConfigError),
    StreamClosed { outcome: Result<(), BackendError> },
    HandoffStopped,
}
```

Declare the whole enum here, with `ConfigError` declared as the section 15.10 shape in `chain/configure.rs`, so the carrier row is complete in one chunk and chunk C3 fills the producer of each remaining arm.

## Steps

1. Read every file of the write scope. Confirm that chunk C1 left each one a stub with a `//!` line and nothing else, and that `crates/duet-engine/Cargo.toml` holds the entries C1 added. Report a discrepancy and stop.
2. Add to `crates/duet-engine/Cargo.toml` the entries `rtrb`, `triple_buffer`, `basedrop`, `async-channel` and `crossbeam-queue`, each `{ workspace = true }`. Run `cargo build --workspace` and keep `Cargo.lock` for the same commit.
3. Write the failing test `disk_ring_refill_reads_the_span_the_plan_names` in `src/disk/ring.rs`. Run `cargo nextest run -p duet-engine -E 'test(disk_ring)' --no-tests=fail` and confirm that it fails to compile.
4. Write `RefillRequest`, `PlaybackSpan`, `TrackPlayback`, `PlaybackPlan`, `DiskReader`, `DiskWriter`, `EngineLink` and `EngineEvent` in `src/disk/ring.rs`, with the declarations above and the hand-written `Debug` impls. Run the test and confirm that it passes.
5. Write the failing test `disk_ring_holds_no_bare_ring_end`. It is a type-level test: a `compile_fail` doctest on `DiskReader` that shows a bare `rtrb::Consumer<f32>` field is refused by PG26. Run `cargo test -p duet-engine --doc` and confirm the expected result.
6. Write `EngineDisk`, `DiskTake` and `DiskSource` in `src/disk/reader.rs` and `src/disk/writer.rs`: the reader half in `reader.rs` and the writer half in `writer.rs`, with the shared root `EngineDisk` in `reader.rs` and re-exported from `src/disk.rs`.
7. Implement the refill path of section 5.10 step 3. The thread reads the plan once per B14 cycle, before it serves any request. For each `RefillRequest` it finds the span that covers `from`, opens `media/<hh>/<hash>.wav` for that span's `source` through `duet_media::SourceReader`, seeks to `offset` plus the distance from `at`, reads, applies the span `gain` and the two fades, and pushes the frames into the ring. A position no span covers reads as silence.
8. Bound every media call. One `SourceReader::read_block` runs inside B117; on expiry the read is abandoned and the ring stays short, and the cycle that meets the short ring returns `CycleOutcome::PlaybackStarved`. One `TakeWriter::append` and its flush run inside B118; on expiry the pass ends, `EngineFault::CaptureStalled { track }` is raised, and the take keeps every sample already flushed. One `PyramidBuilder` append runs inside B118; on expiry the peak write is abandoned for the pass, no fault is raised, and the take is not marked.
9. Implement the capture drain. The thread drains each capture ring and appends through `duet_media::TakeWriter`, and it flushes on every drain. It builds peak bins as it writes, through `duet_dsp::peaks::PyramidBuilder`, and appends them to `derived/peaks/<uuid>.peaks`.
10. **This thread drains no fault queue.** The engine handoff thread owns that drain (section 12.4 step 2, chunk C3).
11. Write the failing test `capture_done_reports_one_info_per_armed_track`. Run `cargo nextest run -p duet-engine -E 'test(capture_done)' --no-tests=fail` and confirm that it fails.
12. Implement the end-of-pass report. The thread sends one `EngineEvent::CaptureDone(CaptureInfo)` per armed track on the B138 channel through `EngineLink`. The send is a `try_send`; a refused event goes into `pending_events`, which is a LIST at B147 and never one slot, and the thread retries at the head of its next pass. On a full list the thread performs one blocking send of the oldest event, which is legal on this thread because it already blocks on a file. Run the test and confirm that it passes.
13. Write the failing test `capture_done_carries_no_take_flags`, then confirm by construction that `CaptureInfo` holds the five fields above and no flag set.
14. Run `cargo clippy -p duet-engine --all-targets -- -D warnings`. Fix every finding in the code.
15. Commit on the branch `chunk/c2-disk-thread-and-rings`. The native git hook runs `scripts/dod.sh`.

## Tests

| Test | File and place | What it asserts |
|---|---|---|
| `disk_ring_refill_reads_the_span_the_plan_names` | `src/disk/ring.rs`, `#[cfg(test)] mod tests` | A plan with two spans over one track answers a `RefillRequest` at a position inside the second span with the second span's source and offset. Message: "the refill resolves the span that covers the position". |
| `disk_ring_gap_reads_as_silence` | `src/disk/ring.rs` | A `RefillRequest` at a position no span covers pushes only zero samples. Message: "a gap on the timeline reads as silence". |
| `disk_ring_applies_the_span_gain_and_both_fades` | `src/disk/ring.rs` | The first frame of a span with a fade in is below the span gain, the middle frame equals the span gain, and the last frame of a fade out is below it. Message: "the disk reader applies the gain and the two fades before the ring". |
| `disk_ring_holds_no_bare_ring_end` | `src/disk/ring.rs`, a `compile_fail` doctest on `DiskReader` | A `DiskReader` whose `samples` field is a bare `rtrb::Consumer<f32>` does not build. Message: the doctest itself is the proof. |
| `disk_read_expiry_leaves_the_ring_short` | `src/disk/reader.rs` | A `SourceReader` double that never answers inside B117 leaves the ring short and raises no fault. Message: "a read past B117 leaves the ring short and raises no fault". |
| `disk_write_expiry_raises_capture_stalled` | `src/disk/writer.rs` | A `TakeWriter` double that never answers inside B118 ends the pass and sends `EngineFault::CaptureStalled { track }`. Message: "an append past B118 ends the pass and reports CaptureStalled". |
| `capture_done_reports_one_info_per_armed_track` | `src/disk/writer.rs` | With three armed tracks, the thread sends three `EngineEvent::CaptureDone` values and each one names a distinct `TrackId`. Message: "one CaptureDone per armed track, each naming its own track". |
| `capture_done_retries_a_refused_send` | `src/disk/writer.rs` | With a B138 channel the test keeps full for two passes, no `CaptureDone` is lost: `pending_events` holds each refused event and the next pass sends it. Message: "a refused CaptureDone waits in the pending list and is sent later". |
| `capture_done_reports_zero_frames_after_a_count_in_stop` | `src/disk/writer.rs` | A pass stopped during the count-in reports `CaptureInfo::frames` of zero. Message: "a pass that captured nothing reports zero frames". |

Every test builds its System Under Test with one `fn sut(...)` that takes the plan, the media doubles, and the channel ends as explicit arguments. A test that touches the filesystem creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test sleeps; each one waits on a returned value or on a channel.

## Verification

1. `cargo nextest run -p duet-engine -E 'test(disk_ring) + test(capture_done)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of either name exists.
2. `cargo nextest run -p duet-engine --no-tests=fail` passes.
3. `cargo test -p duet-engine --doc` passes, and the `compile_fail` doctest fails to compile as it must.
4. `cargo clippy -p duet-engine --all-targets -- -D warnings` prints nothing.
5. `cargo machete` reports no unused dependency of `duet-engine`.
6. One commit on the branch `chunk/c2-disk-thread-and-rings` passes the native git hook. The commit carries `crates/duet-engine/Cargo.toml` and `Cargo.lock` together.

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
