---
id: N3
line: N
depends_on: [N2]
write_scope:
  - crates/bc_audio/duet-media/lang_rust/src/flac.rs
  - crates/bc_audio/duet-media/lang_rust/src/decode.rs
  - crates/bc_audio/duet-media/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-media -E 'test(flac) + test(decode)' --no-tests=fail"
---

# N3: The FLAC encoder over flacenc and the decode path over symphonia

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Phase 5 carries no manifest chunk, so this chunk depends on chunk N2 alone (section
13.3). Chunk N1 created `crates/bc_audio/duet-media/lang_rust/src/flac.rs` and `crates/bc_audio/duet-media/lang_rust/src/decode.rs` as
stubs in phase 3, so each one holds a `//!` line alone. This chunk fills both. It encodes FLAC for
the render path that chunk H3 calls, and it decodes an imported file in a format Duet does not write.
It implements architecture sections 1.2, 7.5, and 15.9, and it carries the FLAC half of the product
story MA-04.

This chunk adds two dependencies, so SM1 puts `crates/bc_audio/duet-media/lang_rust/Cargo.toml` in its write scope and
SM5 rule 2 puts `Cargo.lock` there beside it. Section 13.2 names both files in this chunk's Writes
cell.

## Files

- `crates/bc_audio/duet-media/lang_rust/src/flac.rs` — modify. The FLAC encoder.
- `crates/bc_audio/duet-media/lang_rust/src/decode.rs` — modify. The decode path for an imported file.
- `crates/bc_audio/duet-media/lang_rust/Cargo.toml` — modify. Add `flacenc` and `symphonia`.
- `Cargo.lock` — modify.

## Types and signatures

### Manifest

```toml
# crates/bc_audio/duet-media/lang_rust/Cargo.toml, [dependencies], added by this chunk
flacenc = { workspace = true }
symphonia = { workspace = true }
```

Chunk M4 pins both in phase 4, which section 13.1 states. Appendix B.5 states the `symphonia` feature
requirement: MP3 and AAC decoding, which the default set omits, beside FLAC and OGG. Chunk M4
resolves the exact feature names against the pinned version and records them, so this chunk reads the
resolved line and adds no feature of its own. SM4 forbids this chunk to edit the root manifest, so a
missing pin is a discrepancy that this chunk reports under SM0.

Section 1.2 gives the reason for each crate. `flacenc` is the only maintained pure-Rust FLAC encoder.
`symphonia` decodes MP3, AAC, FLAC, and OGG, which a user may drag into a project.

### Consumed types

| Type | Crate and path |
|---|---|
| `MediaError` | `duet-media`, chunk N1, crate root |
| `SampleRate`, `ChannelIndex`, `I24` | `duet-time`, paths `duet_time::SampleRate`, `duet_time::ChannelIndex`, `duet_time::I24` |

From architecture section 15.1:

```rust
/// A 24-bit signed sample, held in the low three bytes of an `i32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct I24(i32);
```

### Module `duet_media::flac`

Section 1.5 places no type in this module, so the module holds functions alone. The module is the one
place that names `flacenc`.

Section 7.5 states the rule for the format: FLAC takes integer formats only, and a float render
refuses FLAC with `ExportError::FormatMismatch`. `ExportError` is a `duet-export` type, and this
crate carries no `duet-export` edge, so this module answers `MediaError::Format` and chunk H3
converts at its own boundary.

```rust
/// Encode one whole source as FLAC.
///
/// The input is integer samples, because FLAC holds no float format. A
/// caller with a float render converts first, and chunk H3 answers a float
/// request with `ExportError::FormatMismatch` before it reaches this
/// function (section 7.5).
///
/// # Errors
/// Returns `MediaError::Open` when the target cannot be created,
/// `MediaError::Io` when a write fails, and `MediaError::Format` when the
/// bit depth is one FLAC does not hold.
pub fn encode(
    path: &Path,
    samples: &[I24],
    rate: SampleRate,
    channels: ChannelIndex,
    bits_per_sample: u8,
) -> Result<u64, MediaError>;

/// The bit depths this build encodes.
pub const SUPPORTED_BIT_DEPTHS: [u8; 3] = [16, 20, 24];
```

### Module `duet_media::decode`

Section 1.5 places no type in this module either. The module is the one place that names `symphonia`.

```rust
/// What one decoded import holds.
#[derive(Debug, Clone, PartialEq)]
pub struct DecodedAudio { samples: Vec<f32>, rate: SampleRate, channels: ChannelIndex, frames: u64 }

/// Decode one imported audio file into interleaved `f32` samples.
///
/// It is how a user brings an MP3 or an AAC reference track into a project
/// (section 1.2). The result is written back as a WAV or an RF64 take
/// through chunk N1 and chunk N2, so nothing outside this module ever holds
/// a foreign container.
///
/// # Errors
/// Returns `MediaError::Open` when the path cannot be opened,
/// `MediaError::Format` for a container or a codec this build does not
/// decode, `MediaError::Rate` for a sample rate outside the range the
/// project holds, and `MediaError::Io` when a read fails.
pub fn decode_file(path: &Path) -> Result<DecodedAudio, MediaError>;

/// Decode one imported file in blocks, and hand each block to `sink`.
///
/// A long import must not hold the whole file in memory, so the block form
/// is the one the import job calls and `decode_file` is the one a test
/// calls.
///
/// # Errors
/// Returns the same set `decode_file` returns, and it stops at the first
/// refusal the sink answers.
pub fn decode_blocks(
    path: &Path,
    sink: &mut dyn FnMut(&[f32]) -> Result<(), MediaError>,
) -> Result<(SampleRate, ChannelIndex, u64), MediaError>;
```

`DecodedAudio` is a module type of `duet-media` that no other crate names, exactly as `WavSink` of
chunk N1 and `Rf64Sink` of chunk N2 are; report the discrepancy to the Architect if
`cargo xtask check-placement` refuses it.

## Steps

1. Read `crates/bc_audio/duet-media/lang_rust/src/flac.rs` and `crates/bc_audio/duet-media/lang_rust/src/decode.rs`. Confirm that each one
   holds a `//!` line and nothing else. Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `flacenc` and
   `symphonia`, and that the `symphonia` line carries the decoder features Appendix B.5 names. Report
   a discrepancy and stop if either one is absent.
3. Write the failing test `flac_round_trips_through_the_decoder` in `src/flac.rs`, inside a
   `#[cfg(test)] mod tests`. Section 13.2 gives this chunk no `tests/` file, so every test of this
   chunk lives in the same file as the code it covers. Run
   `cargo nextest run -p duet-media -E 'test(flac)' --no-tests=fail` and confirm that the run fails
   to compile.
4. Add the two entries to `crates/bc_audio/duet-media/lang_rust/Cargo.toml`. Run `cargo build --workspace`, which
   settles `Cargo.lock` (SM5 rule 3).
5. Declare `SUPPORTED_BIT_DEPTHS` and implement `encode` over `flacenc`. Refuse a bit depth the
   array does not hold with `MediaError::Format`. Convert every `I24` to the encoder's own sample
   type through `duet_time::convert`, because `as` is denied outside that module.
6. Run the same command and confirm that the test passes.
7. Write the failing test `decode_reads_a_flac_file` in `src/decode.rs`. Run
   `cargo nextest run -p duet-media -E 'test(decode)' --no-tests=fail` and confirm that it fails.
8. Declare `DecodedAudio` and implement `decode_blocks` over `symphonia`. Name every `symphonia`
   error arm this build handles, because `clippy::wildcard_enum_match_arm` is denied, and map each
   one to its `MediaError` arm with a named `map_err` that keeps the source text.
9. Implement `decode_file` over `decode_blocks`, so one decode path exists and the whole-file form is
   a caller of the block form.
10. Run the same command and confirm that every `decode` test passes.
11. Run `cargo nextest run -p duet-media -E 'test(flac) + test(decode)' --no-tests=fail` and confirm
    that both terms pass together.
12. Run `cargo nextest run -p duet-media --no-tests=fail` and confirm that every test passes.
13. Run `cargo clippy -p duet-media --all-targets --locked -- -D warnings` once and repair every
    finding.
14. Run `cargo deny check` once and confirm that `flacenc` and `symphonia` pass the licence policy.
    Record the output in the commit body.
15. Commit on the branch `chunk/n3-flac-encode-and-decode`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, because section 13.2 gives this
chunk no `tests/` file. Every assert carries a message. Every test owns one scratch directory, which
it builds from `std::env::temp_dir` plus a unique name and removes at the end.

`crates/bc_audio/duet-media/lang_rust/src/flac.rs`

- `flac_round_trips_through_the_decoder` — encodes 4800 stereo frames at 24 bits, decodes the result
  with `crate::decode::decode_file`, and asserts exact sample equality, because FLAC is lossless.
- `flac_encodes_every_supported_bit_depth` — asserts a readable file at 16, 20, and 24 bits.
- `flac_refuses_an_unsupported_bit_depth` — asserts `MediaError::Format` at 32 bits, which is the
  refusal section 7.5 calls `ExportError::FormatMismatch` at the export boundary.
- `flac_is_smaller_than_the_wav_source` — encodes a sine and asserts a byte count below the WAV byte
  count of the same frames, which is the whole reason the format is offered.
- `flac_refuses_an_unwritable_path` — asserts `MediaError::Open`.
- `flac_encode_is_deterministic` — encodes one buffer twice and asserts equal bytes, which the
  deterministic render of section 1.2 requires.

`crates/bc_audio/duet-media/lang_rust/src/decode.rs`

- `decode_reads_a_flac_file` — encodes with `crate::flac::encode`, decodes, and asserts the rate, the
  channel count, and the frame count.
- `decode_reads_a_wav_file` — writes with `crate::wav`, decodes, and asserts exact sample equality.
- `decode_blocks_never_holds_the_whole_file` — decodes a ten-minute file in blocks and asserts that
  no single block passes one second of frames, which is the property the block form exists for.
- `decode_refuses_an_unknown_container` — asserts `MediaError::Format` for a file of random bytes.
- `decode_refuses_a_missing_file` — asserts `MediaError::Open`.
- `decode_stops_at_the_first_sink_refusal` — returns a `MediaError::Io` from the sink on the third
  block and asserts that the decode stops with that error and reads no fourth block.

No test of this chunk needs an MP3 or an AAC fixture: the two codecs are a feature of the pinned
`symphonia` build, which chunk M4 records with `cargo tree -e features -i symphonia`, and a
proprietary fixture is not a file this repository ships.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-media -E 'test(flac) + test(decode)' --no-tests=fail
cargo nextest run -p duet-media --no-tests=fail
cargo clippy -p duet-media --all-targets --locked -- -D warnings
cargo deny check
cargo machete
```

The first command fails before this chunk, because the crate holds neither test term. Chunks N1 and
N2 wrote their tests as `wav_*` and `rf64_*` for that reason (SM3 rule 2). The command passes after
this chunk and it reports twelve tests. The second command reports every test of the crate as passed.
The third command prints no warning, and this chunk carries no `#[expect]` site. The fourth command
reports no licence failure and no advisory. The fifth command reports no unused dependency, because
this chunk adds the two entries in the same commit as the code that uses them (SM1).

The work lands as one commit on the branch `chunk/n3-flac-encode-and-decode`, with a conventional
subject such as `feat(media): encode FLAC and decode an imported file`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs
  `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted
  diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site
  `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture
  Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is
  denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice
  indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only
  inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are
  required on every item.
- A new crate lives at `crates/bc_<context>/<crate>/lang_rust/` in the context that architecture section 1.2 names (ADR 0011), declares `[lints] workspace = true`, inherits every
  `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the
  root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- After chunk M94 lands, every `.rs` file a commit writes carries one front-matter block (`cargo xtask check-ddd --write`), and the gate refuses a changed `.rs` file with none.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no
  pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with
  Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this
  repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match
  what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
