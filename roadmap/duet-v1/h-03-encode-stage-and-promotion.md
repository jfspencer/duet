---
id: H3
line: H
depends_on: [H2, N3]
write_scope:
  - crates/bc_audio/duet-export/lang_rust/src/encode/wav.rs
  - crates/bc_audio/duet-export/lang_rust/src/encode/flac.rs
  - crates/bc_audio/duet-export/lang_rust/src/encode/promote.rs
  - crates/bc_audio/duet-export/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-export -E 'test(encode_stage)' --no-tests=fail passes; commit SHA on a branch chunk/h3-encode-stage-and-promotion"
---

# H3: The encode stage over `duet-media`, and the size promotion rule

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the encode stage of architecture sections 7.4 and 7.5: the WAV writer, the FLAC writer, the dither step that precedes each integer format, and the RF64 promotion rule that the predicted size decides. `duet-export` writes every render through `duet-media` and names no format crate of its own. It carries MUST story MA-04 (export to WAV, RF64 and FLAC) and SHOULD stories MA-05 (per-part export) and MA-06 (sample rate and bit depth choice), both through `ExportSpec` and `AudioExportRequest`.

Link `N3 before H3` of section 13.4 states the reason: the encode stage calls the FLAC encoder.

## Files

- `crates/bc_audio/duet-export/lang_rust/src/encode/wav.rs` — modify. Chunk H1 created the stub.
- `crates/bc_audio/duet-export/lang_rust/src/encode/flac.rs` — modify.
- `crates/bc_audio/duet-export/lang_rust/src/encode/promote.rs` — modify.
- `crates/bc_audio/duet-export/lang_rust/Cargo.toml` — modify, when this chunk adds an entry the manifest does not hold.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### The format table this chunk implements (architecture 7.5)

| Container | Crate inside `duet-media` | Rule |
|---|---|---|
| WAV | `hound` | The default for a render under the B67 container limit |
| RF64 | `bwavfile` | The render promotes when the predicted size passes the limit, and the spec records the promotion |
| FLAC | `flacenc` | Integer formats only; a float render refuses FLAC with `ExportError::FormatMismatch` |

**The predicted size is `frames * channels * bytes_per_sample + 4096`.** The render computes it before the first write, so the promotion is a decision and not a recovery. B67 is the threshold, at 3.9 GiB against a 4 GiB container limit.

### `duet_export::encode::promote` (architecture 7.5, 1.2)

```rust
/// Decide the container this render writes, before the first byte.
///
/// The predicted size is `frames * channels * bytes_per_sample + 4096`, and
/// the render promotes from `Container::Wav` to `Container::Rf64` when that
/// value passes B67. The decision is recorded on the spec, so the report
/// names it.
///
/// # Errors
/// Returns `ExportError::FormatMismatch` when the requested container cannot
/// hold the requested sample format, which today means FLAC with
/// `SampleFormat::Float32`.
pub fn decide_container(spec: &ExportSpec, frames: u64, channels: NonZeroU8)
    -> Result<Container, ExportError>;
```

### `duet_export::encode::wav` and `duet_export::encode::flac`

Both stages take the limited samples pass two produced, apply the dither step when the target is an integer format, and write through `duet-media`. **`duet-export` names no audio format crate of its own**: `duet_media::TakeWriter` and the `duet-media` encode path own `hound`, `bwavfile` and `flacenc`.

```rust
// in duet-command
/// Which dither a render applies before it narrows to an integer format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DitherKind { None, Rectangular, Triangular, Shaped }

/// The output sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat { Int16, Int24, Float32 }

/// The output container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Container { Wav, Rf64, Flac }
```

`DitherKind` narrows a render to an integer format and changes no live sample, which is why it is a field of `ExportSpec` and never a `SlotKind` the audio thread runs. A `SampleFormat::Float32` target runs no dither at all.

The per-part choice is a property of the request and not of the spec:

```rust
// in duet-command
/// Render the mix to one audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioExportRequest { path: PathBuf, spec: ExportSpec, overwrite: bool, part: Option<PartId> }
```

`RenderJob` writes one file per request, so a caller that wants one file per part sends one request per part.

### The refusal this chunk owns

```rust
// in duet-export, chunk H1
pub enum ExportError {
    FormatMismatch { container: Container, format: SampleFormat },
    TargetExists(Box<str>),
    Cancelled,
    Media(MediaError),
    Analysis(AnalysisError),
    Engine(EngineError),
}
```

`ExportError::TargetExists` answers a render whose target path exists and whose `AudioExportRequest::overwrite` is false. `ExportError::Media` wraps every `duet-media` refusal, including `MediaError::ContainerLimit { bytes }`, which the promotion rule exists to prevent.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| The FLAC encoder, the WAV and RF64 writers, `MediaError` | `duet-media` | `duet_media`, chunks N1, N2 and N3 |
| `ExportSpec`, `Container`, `SampleFormat`, `DitherKind`, `AudioExportRequest` | `duet-command` | `duet_command` |
| The limited samples of pass two | `duet-export` | chunk H2 |
| `ExportError`, `RenderJob` | `duet-export` | chunk H1 |

## Steps

1. Read every file of the write scope and `crates/bc_audio/duet-export/lang_rust/Cargo.toml`. Confirm that chunk H1 left each one a stub, that chunk H2 wrote the two passes, and that chunk N3 wrote the FLAC encoder and the decode path. Report a discrepancy and stop.
2. Confirm that `crates/bc_audio/duet-export/lang_rust/Cargo.toml` already holds the `duet-media` entry chunk H1 added. Add no format crate: `duet-export` names none. When the manifest needs a new entry, add it as `{ workspace = true }`, run `cargo build --workspace`, and keep `Cargo.lock` for the same commit; when it needs none, leave both files unchanged and say so in the commit message.
3. Write the failing test `encode_stage_writes_a_wav_under_the_limit` in `src/encode/wav.rs`. Run `cargo nextest run -p duet-export -E 'test(encode_stage)' --no-tests=fail` and confirm that it fails to compile.
4. Write `decide_container` in `src/encode/promote.rs`, with the predicted size `frames * channels * bytes_per_sample + 4096` and the B67 threshold. It runs before the first write. It returns `ExportError::FormatMismatch` for FLAC with `SampleFormat::Float32`.
5. Write the WAV stage in `src/encode/wav.rs`. It applies the dither step for an integer target, writes through `duet-media`, and uses the container `decide_container` chose. Run the test and confirm that it passes.
6. Write the failing test `encode_stage_promotes_to_rf64_past_b67`. Run it and confirm that it fails, then confirm that a predicted size past B67 selects `Container::Rf64` before the first byte and that the spec records the promotion.
7. Write the failing test `encode_stage_refuses_flac_with_a_float_format`. Run it and confirm that it fails, then confirm that it returns `ExportError::FormatMismatch { container, format }` and writes no file.
8. Write the FLAC stage in `src/encode/flac.rs`. It accepts `SampleFormat::Int16` and `SampleFormat::Int24` alone, applies the dither step, and writes through the `duet-media` FLAC encoder.
9. Write the failing test `encode_stage_applies_the_dither_before_it_narrows`. Run it and confirm that it fails, then implement the four `DitherKind` arms. `DitherKind::None` adds nothing, and a `SampleFormat::Float32` target runs no dither at all. Confirm that it passes.
10. Write the failing test `encode_stage_refuses_an_existing_target`. Run it and confirm that it fails, then implement the `ExportError::TargetExists` refusal for a request whose `overwrite` is false. Confirm that it passes.
11. Write the failing test `encode_stage_writes_one_file_per_request`. Run it and confirm that it fails, then confirm that a request with `part: Some(part)` writes one file that holds that part alone, and that a request with `part: None` writes the whole mix.
12. Confirm that the finished file is renamed into place as the last step, which chunk H1's cleanup contract requires, and that a failure inside the encode stage leaves no partial file at the target path.
13. Run `cargo clippy -p duet-export --all-targets -- -D warnings`. Fix every finding in the code.
14. Commit on the branch `chunk/h3-encode-stage-and-promotion`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test needs a device.

| Test | File | What it asserts |
|---|---|---|
| `encode_stage_writes_a_wav_under_the_limit` | `src/encode/wav.rs` | A render below B67 writes a WAV file that `duet-media` reads back with the expected frame count and sample rate. Message: "a render under the limit writes a readable WAV". |
| `encode_stage_promotes_to_rf64_past_b67` | `src/encode/promote.rs` | A predicted size past B67 selects `Container::Rf64` before the first byte, and the spec records the promotion. Message: "a render past B67 promotes to RF64 before it writes". |
| `encode_stage_predicts_the_size_from_the_formula` | `src/encode/promote.rs` | A table-driven loop over four frame counts asserts the predicted size equals `frames * channels * bytes_per_sample + 4096`. The message names the case: `"frames {frames}"`. |
| `encode_stage_refuses_flac_with_a_float_format` | `src/encode/promote.rs` | FLAC with `SampleFormat::Float32` returns `ExportError::FormatMismatch { container, format }` and writes no file. Message: "FLAC refuses a float render". |
| `encode_stage_writes_a_flac_for_an_integer_format` | `src/encode/flac.rs` | `SampleFormat::Int24` with `Container::Flac` writes a file that `duet-media` decodes to the expected frames. Message: "FLAC accepts an integer format". |
| `encode_stage_applies_the_dither_before_it_narrows` | `src/encode/wav.rs` | A table-driven loop over the four `DitherKind` arms asserts that each one changes the narrowed output as its name states, and that `None` adds nothing. The message names the case: `"dither {kind:?}"`. |
| `encode_stage_runs_no_dither_for_a_float_target` | `src/encode/wav.rs` | A `SampleFormat::Float32` target writes the limited samples unchanged. Message: "a float target runs no dither". |
| `encode_stage_refuses_an_existing_target` | `src/encode/wav.rs` | A request whose target exists and whose `overwrite` is false returns `ExportError::TargetExists` and changes nothing on disk. Message: "an existing target is refused when overwrite is false". |
| `encode_stage_writes_one_file_per_request` | `src/encode/wav.rs` | A request with `part: Some(part)` writes one file that holds that part alone, and a request with `part: None` writes the whole mix. Message: "one request writes one file". |
| `encode_stage_leaves_no_partial_file_on_a_failure` | `src/encode/wav.rs` | A `duet-media` failure inside the encode stage leaves no file at the target path. Message: "a failed encode leaves no partial file". |

## Verification

1. `cargo nextest run -p duet-export -E 'test(encode_stage)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-export --no-tests=fail` passes.
3. `cargo clippy -p duet-export --all-targets -- -D warnings` prints nothing.
4. `cargo machete` reports no unused dependency of `duet-export`, and the manifest names no audio format crate.
5. One commit on the branch `chunk/h3-encode-stage-and-promotion` passes the native git hook. When the commit changes `crates/bc_audio/duet-export/lang_rust/Cargo.toml`, it carries `Cargo.lock` with it.

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
