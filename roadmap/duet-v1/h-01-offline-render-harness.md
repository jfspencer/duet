---
id: H1
line: H
depends_on: [C1, C3, M7]
write_scope:
  - crates/bc_audio/duet-export/lang_rust/Cargo.toml
  - crates/bc_audio/duet-export/lang_rust/src/lib.rs
  - crates/bc_audio/duet-export/lang_rust/src/render.rs
  - crates/bc_audio/duet-export/lang_rust/src/loudness.rs
  - crates/bc_audio/duet-export/lang_rust/src/encode.rs
  - crates/bc_audio/duet-export/lang_rust/src/preset.rs
  - crates/bc_audio/duet-export/lang_rust/src/loudness/analyse.rs
  - crates/bc_audio/duet-export/lang_rust/src/loudness/limit.rs
  - crates/bc_audio/duet-export/lang_rust/src/encode/wav.rs
  - crates/bc_audio/duet-export/lang_rust/src/encode/flac.rs
  - crates/bc_audio/duet-export/lang_rust/src/encode/promote.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-export -E 'test(offline_render)' --no-tests=fail passes; commit SHA on a branch chunk/h1-offline-render-harness"
---

# H1: The offline render harness over the dummy backend, and the resample stage

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the first layer of `duet-export`: the offline render harness that drives the dummy backend in `Freewheel`, the `ChannelMap`, `SilenceTrim` and `SampleRateConvert` stages of architecture section 7.4, `RenderJob`, `ExportError`, and the temporary-directory contract of section 9.6. It also creates every module file of line H as a stub (SM2). It carries MUST story MA-04 (export to WAV, RF64 and FLAC, the render half) and SHOULD stories MA-05 and MA-06 through the `ExportSpec` the dialog reads. The crate skeleton already exists, because chunk M7 created it.

Links `C1 before H1` and `C3 before H1` of section 13.4 state the reason: the offline render drives the dummy backend, and H1 renders the mix graph that C3 builds.

## Files

- `crates/bc_audio/duet-export/lang_rust/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).
- `crates/bc_audio/duet-export/lang_rust/src/lib.rs` — modify. Add every `mod` line of the line.
- `crates/bc_audio/duet-export/lang_rust/src/render.rs` — create.
- `crates/bc_audio/duet-export/lang_rust/src/loudness.rs` — create as a stub.
- `crates/bc_audio/duet-export/lang_rust/src/encode.rs` — create as a stub.
- `crates/bc_audio/duet-export/lang_rust/src/preset.rs` — create as a stub.
- `crates/bc_audio/duet-export/lang_rust/src/loudness/analyse.rs` — create as a stub.
- `crates/bc_audio/duet-export/lang_rust/src/loudness/limit.rs` — create as a stub.
- `crates/bc_audio/duet-export/lang_rust/src/encode/wav.rs` — create as a stub.
- `crates/bc_audio/duet-export/lang_rust/src/encode/flac.rs` — create as a stub.
- `crates/bc_audio/duet-export/lang_rust/src/encode/promote.rs` — create as a stub.

A stub holds the `//!` module documentation and nothing else (SM2). A later chunk of line H modifies a stub and creates no file.

## Types and signatures

### The pipeline this chunk starts (architecture 7.4)

```text
render the mix graph (dummy backend, freewheel)
  -> ChannelMap
  -> SilenceTrim
  -> SampleRateConvert (rubato)
  -> pass 1: LoudnessAnalyse (ebur128) and write an f32 intermediate file
  -> pass 2: read the intermediate -> Gain (computed) -> TruePeakLimiter
             -> Dither -> Encoder
```

This chunk builds the render and the first three stages. Chunk H2 builds the two passes, chunk H3 builds the encoder, and chunk H4 builds the presets.

### `duet_export` crate root (architecture 15.13)

```rust
/// One render in flight: what it renders, where it writes, and how far it got.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderJob { job: JobId, spec: ExportSpec, target: PathBuf, span: Option<Span>, done_frames: u64, total_frames: u64 }

/// Every way a render refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExportError {
    FormatMismatch { container: Container, format: SampleFormat },
    TargetExists(Box<str>),
    Cancelled,
    Media(MediaError),
    Analysis(AnalysisError),
    Engine(EngineError),
}
```

`ExportError` wraps three causes with `#[from]`. `duet-export` implements `From<ExportError> for UpstreamFailure` at its own boundary, with `FailureSurface::Export` and the matching `FailureCode`, which the orphan rule allows because the source type is local here.

### The record the render reads (architecture 7.4, declared in `duet-command`)

```rust
// in duet-command
/// One declarative record that fully determines a render (Ardour 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSpec {
    container: Container,
    sample_format: SampleFormat,
    sample_rate: SampleRate,
    dither: DitherKind,
    normalization: Normalization,
}

/// The output container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Container { Wav, Rf64, Flac }

/// The output sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat { Int16, Int24, Float32 }

/// Which dither a render applies before it narrows to an integer format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DitherKind { None, Rectangular, Triangular, Shaped }

/// How the render sets the level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Normalization {
    None,
    Peak { dbfs: Finite },
    Loudness { lufs: Finite, true_peak_dbtp: Finite },
}

/// Render the mix to one audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioExportRequest { path: PathBuf, spec: ExportSpec, overwrite: bool, part: Option<PartId> }
```

`AudioExportRequest::part` is the per-part choice PR MA-05 asks for. `None` renders the whole mix to `path`; `Some(part)` renders that part alone, and `RenderJob` writes one file per request.

### `duet_export::render` (architecture 7.4, 5.3, 9.6)

The render drives `DummyBackend` in `Freewheel` mode, so the cycle runs as fast as the process body allows and no device is needed. **The dummy backend varies the frame count on purpose**, over the whole range from 1 to `max_block` with a fixed seed, so the render meets the same variation every engine test meets (section 5.2).

The three stages this chunk writes:

- **`ChannelMap`** maps the graph output channels onto the requested channel layout.
- **`SilenceTrim`** removes leading and trailing silence from the rendered span.
- **`SampleRateConvert`** resamples through `rubato` when `ExportSpec::sample_rate` differs from the project rate. `rubato` stays a dependency of `duet-export` only; it resamples an export or a whole-project conversion, never a live take.

The cleanup contract of section 9.6 that this chunk implements:

1. **Every job that writes owns a temporary directory**, which is `exports/.tmp/<job>/` for a render.
2. **The worker deletes its own temporary directory and any partly written output file** before it sets `JobState::Cancelled` or `JobState::Failed`. The deletion runs in the worker's `Drop`, so a panic-free early return cannot skip it.
3. **A finished render is renamed into place as the last step.** A user therefore never finds a truncated file that looks like a finished render.
4. **The next start empties `exports/.tmp/` and `state/.tmp/`**, because no job survives a restart.
5. **The user sees one line.** After a cancel: `The export stopped. No file was written.` After a failure: `The export failed: <reason>. No file was written.`

Cancellation is a polled flag: the worker reads an `AtomicBool` between units of work. There is no forced stop, so no job is interrupted mid-write.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `AudioBackend`, `AudioProcess`, `DummyBackend`, `Cycle`, `CycleOutcome`, `GraphChain`, `GraphState`, `ChainRunner`, `EngineError` | `duet-engine` | chunks C1 and C3 |
| `ExportSpec`, `Container`, `SampleFormat`, `DitherKind`, `Normalization`, `AudioExportRequest`, `JobId`, `JobState`, `LoudnessReport` | `duet-command` | `duet_command` |
| `MediaError` | `duet-media` | `duet_media` |
| `AnalysisError` | `duet-analysis` | `duet_analysis` |
| `Span`, `Finite`, `SampleRate` | `duet-time` | `duet_time` |

## Steps

1. Read `crates/bc_audio/duet-export/lang_rust/Cargo.toml` and `crates/bc_audio/duet-export/lang_rust/src/lib.rs`. Confirm the M7 skeleton: the `*.workspace = true` package fields, a `description`, `[lints] workspace = true`, no `[dependencies]` section, and a `src/lib.rs` that holds the `//!` crate documentation and `#![forbid(unsafe_code)]`. Report a discrepancy and stop.
2. Create every module file of the write scope as a stub. Add one `mod` line per stub to `src/lib.rs`, and one `mod` line per child stub to its parent module file.
3. Run `cargo check -p duet-export`. Confirm that the crate builds with the stub tree.
4. Add to `crates/bc_audio/duet-export/lang_rust/Cargo.toml` the entries this chunk uses: `duet-time`, `duet-dsp`, `duet-analysis`, `duet-media`, `duet-command`, `duet-engine`, `rubato`, `thiserror`, and `tracing`, each `{ workspace = true }`. Run `cargo build --workspace` and commit `Cargo.lock` with the manifest.
5. Write `RenderJob` and `ExportError` in `src/lib.rs`, with `#[from]` on each wrapping arm, and the `From<ExportError> for UpstreamFailure` impl.
6. Write the failing test `offline_render_produces_the_expected_frame_count` in `src/render.rs`. Run `cargo nextest run -p duet-export -E 'test(offline_render)' --no-tests=fail` and confirm that it fails to compile.
7. Write the render harness in `src/render.rs`. It builds a `DummyBackend` in `Freewheel` mode, opens a stream with the graph `duet-engine` built, and runs the cycle until the span is rendered. It reads `cycle.frames()` each cycle and sizes no buffer from a constant. Run the test and confirm that it passes.
8. Write the failing test `offline_render_meets_a_varying_block_size`. Run it and confirm that it fails, then confirm that it passes over the fixed-seed range from 1 to `max_block`.
9. Write the `ChannelMap` stage. It maps the graph output channels onto the requested layout, and it refuses a layout the graph cannot produce.
10. Write the `SilenceTrim` stage. It removes leading and trailing silence from the rendered span, and it never trims inside the span.
11. Write the failing test `offline_render_resamples_when_the_rate_differs`. Run it and confirm that it fails.
12. Write the `SampleRateConvert` stage over `rubato`. It runs only when `ExportSpec::sample_rate` differs from the project rate, and it reports the conversion so chunk K5 can name it in the report line. Run the test and confirm that it passes.
13. Write the failing test `offline_render_deletes_its_temporary_directory_on_cancel`. Run it and confirm that it fails.
14. Implement the cleanup contract: the render owns `exports/.tmp/<job>/`, the deletion runs in the worker's `Drop`, the finished file is renamed into place as the last step, and a cancel or a failure leaves no partly written output file. Run the test and confirm that it passes.
15. Implement the polled cancel flag: the render reads an `AtomicBool` between units of work and returns `ExportError::Cancelled` when it is set. It never interrupts a write.
16. Write the failing test `offline_render_reports_progress`. Run it and confirm that it fails, then implement the `done_frames` and `total_frames` progress on `RenderJob` and confirm that it passes.
17. Run `cargo clippy -p duet-export --all-targets -- -D warnings`. Fix every finding in the code.
18. Commit on the branch `chunk/h1-offline-render-harness`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test that touches the filesystem creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test needs a device, because the dummy backend serves every export test (section 11.6 rule 1).

| Test | File | What it asserts |
|---|---|---|
| `offline_render_produces_the_expected_frame_count` | `src/render.rs` | A render of a one-second span at 48 kHz produces 48000 frames per channel. Message: "the render produces the frame count the span names". |
| `offline_render_meets_a_varying_block_size` | `src/render.rs` | Over the whole render, the set of observed frame counts holds more than one value and the maximum equals `max_block`. Message: "the render meets a varying block size". |
| `offline_render_channel_map_refuses_an_impossible_layout` | `src/render.rs` | A request for more channels than the graph produces returns an error and writes no file. Message: "the channel map refuses a layout the graph cannot produce". |
| `offline_render_silence_trim_keeps_the_middle` | `src/render.rs` | A span with silence at both ends and a tone in the middle loses both ends and keeps every tone frame. Message: "silence trim removes the ends and never the middle". |
| `offline_render_resamples_when_the_rate_differs` | `src/render.rs` | A render at 44100 from a 48000 project produces the expected frame count and reports the conversion. Message: "the render resamples and reports the conversion". |
| `offline_render_does_not_resample_at_the_project_rate` | `src/render.rs` | A render at the project rate runs no `rubato` stage. Message: "no resample runs at the project rate". |
| `offline_render_deletes_its_temporary_directory_on_cancel` | `src/render.rs` | After a cancel, `exports/.tmp/<job>/` is gone and no partly written output file exists. Message: "a cancel leaves no temporary directory and no partial file". |
| `offline_render_deletes_its_temporary_directory_on_failure` | `src/render.rs` | After a failure, the same two facts hold and the error is reported. Message: "a failure leaves no temporary directory and no partial file". |
| `offline_render_renames_into_place_last` | `src/render.rs` | The output path does not exist until the render is complete. Message: "the finished render is renamed into place as the last step". |
| `offline_render_reports_progress` | `src/render.rs` | `RenderJob::done_frames` rises during the render and equals `total_frames` at the end. Message: "the render reports progress as it runs". |
| `offline_render_cancel_never_interrupts_a_write` | `src/render.rs` | A cancel set during a write completes that write and then stops. Message: "a cancel is polled between units of work". |

## Verification

1. `cargo nextest run -p duet-export -E 'test(offline_render)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-export --no-tests=fail` passes.
3. `cargo clippy -p duet-export --all-targets -- -D warnings` prints nothing.
4. `cargo doc -p duet-export` is clean with `-D warnings`.
5. `cargo machete` reports no unused dependency of `duet-export`.
6. One commit on the branch `chunk/h1-offline-render-harness` passes the native git hook. The commit carries `crates/bc_audio/duet-export/lang_rust/Cargo.toml` and `Cargo.lock` together.

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
