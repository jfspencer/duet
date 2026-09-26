---
id: N2
line: N
depends_on: [M4, N1]
write_scope:
  - crates/bc_audio/duet-media/lang_rust/src/rf64.rs
  - crates/bc_audio/duet-media/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-media -E 'test(rf64)' --no-tests=fail"
---

# N2: The RF64 path over bwavfile and the promotion rule

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk N1 created `crates/bc_audio/duet-media/lang_rust/src/rf64.rs` as a stub in phase 3, so the file holds
a `//!` line alone. This chunk fills it. It reads and writes RF64 through `bwavfile`, and it adds the
one promotion the writer performs when a take passes the container limit. It implements architecture
sections 1.2, 7.5, and 15.9, and it carries the container half of the product story MA-04.

This chunk adds one dependency, so SM1 puts `crates/bc_audio/duet-media/lang_rust/Cargo.toml` in its write scope and
SM5 rule 2 puts `Cargo.lock` there beside it. Section 13.2 names both files in this chunk's Writes
cell.

## Files

- `crates/bc_audio/duet-media/lang_rust/src/rf64.rs` — modify. The RF64 read path, the RF64 write path, and the
  promotion.
- `crates/bc_audio/duet-media/lang_rust/Cargo.toml` — modify. Add `bwavfile`.
- `Cargo.lock` — modify.

`crates/bc_audio/duet-media/lang_rust/src/lib.rs` is outside this chunk's write scope, so `TakeWriter::append` must
already call the promotion path behind a branch that chunk N1 wrote. Read that call site first, and
report a discrepancy and stop if it is absent.

## Types and signatures

### Manifest

```toml
# crates/bc_audio/duet-media/lang_rust/Cargo.toml, [dependencies], added by this chunk
bwavfile = { workspace = true }
```

Chunk M4 pins `bwavfile` in phase 4, which section 13.1 states. Section 1.2 gives the reason:
`hound` has no RF64 support and fails above the container limit. SM4 forbids this chunk to edit the
root manifest, so a missing pin is a discrepancy that this chunk reports under SM0.

### Consumed types

| Type | Crate and path |
|---|---|
| `SourceReader`, `TakeWriter`, `MediaError` | `duet-media`, chunk N1, crate root |
| `AudioContainer` | `duet-session`, path `duet_session::AudioContainer` |
| `SampleRate`, `ChannelIndex` | `duet-time` |

From architecture section 4.3:

```rust
/// The container a captured source is stored in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AudioContainer { Wav, Rf64 }
```

### The promotion rule

Section 1.2 states it in full. The disk writer opens a take as WAV through `hound`. It counts
written bytes. At the promotion threshold B67 it closes the WAV, reopens the take as RF64 through
`bwavfile`, and copies the written frames. A take that starts above the limit opens as RF64 at once.
`ManifestEntry::container` records which one. B67 is 3.9 GiB, against a 4 GiB container limit.

Section 7.5 states the render half: the render promotes when the predicted size passes the limit, and
the spec records the promotion. The predicted size is `frames * channels * bytes_per_sample + 4096`,
and the render computes it before the first write, so the promotion is a decision and not a
recovery.

### Module `duet_media::rf64`

Section 1.5 places no type in this module, so the module holds functions alone. The module is the one
place that names `bwavfile`, so a container change touches one file.

```rust
/// The byte count at which a WAV take becomes an RF64 take (B67).
pub const PROMOTION_THRESHOLD_BYTES: u64 = 4_187_593_113;

/// The predicted size of one render, in bytes.
///
/// Section 7.5 gives the formula: `frames * channels * bytes_per_sample +
/// 4096`. A render calls it before the first write, so a promotion is a
/// decision and not a recovery.
#[must_use]
pub fn predicted_bytes(frames: u64, channels: ChannelIndex, bytes_per_sample: u32) -> u64;

/// Whether a writer at `written_bytes` must promote before its next append.
#[must_use]
pub const fn needs_promotion(written_bytes: u64, next_block_bytes: u64) -> bool;

/// Read the header of one RF64 file.
///
/// # Errors
/// Returns `MediaError::Format` for a file that is not RF64, and
/// `MediaError::Rate` for a sample rate outside the range the project holds.
pub fn read_header(path: &Path) -> Result<(SampleRate, ChannelIndex, u64), MediaError>;

/// Read `out.len()` samples from `at`, and return the frame count.
///
/// # Errors
/// Returns `MediaError::Io` when the read fails.
pub fn read_frames(path: &Path, at: u64, out: &mut [f32]) -> Result<usize, MediaError>;

/// The open RF64 file one `TakeWriter` appends to.
#[derive(Debug)]
pub struct Rf64Sink { writer: bwavfile::WaveWriter<BufWriter<File>>, written_bytes: u64 }

/// Create one RF64 file for a take that starts above the limit.
///
/// # Errors
/// Returns `MediaError::Open` when the file cannot be created.
pub fn create(path: &Path, rate: SampleRate, channels: ChannelIndex) -> Result<Rf64Sink, MediaError>;

/// Close the WAV file at `path`, reopen the take as RF64, and copy every
/// written frame.
///
/// It is the one promotion a take performs, and `TakeWriter::container`
/// changes once, at B67 (section 15.9).
///
/// # Errors
/// Returns `MediaError::Io` when the copy fails, and
/// `MediaError::ContainerLimit { bytes }` when the source file is already
/// above the RF64 limit, which no take of this product can reach.
pub fn promote(path: &Path, written_frames: u64, rate: SampleRate, channels: ChannelIndex)
    -> Result<Rf64Sink, MediaError>;

/// Append one block and flush.
///
/// # Errors
/// Returns `MediaError::Io` when the write or the flush fails.
pub fn append(sink: &mut Rf64Sink, block: &[f32]) -> Result<(), MediaError>;

/// Write the real header and close.
///
/// # Errors
/// Returns `MediaError::Io` when the header write fails.
pub fn finish(sink: Rf64Sink) -> Result<u64, MediaError>;
```

`Rf64Sink` is a module type of `duet-media` that no other crate names, exactly as `WavSink` of chunk
N1 is; report the discrepancy to the Architect if `cargo xtask check-placement` refuses it.

## Steps

1. Read `crates/bc_audio/duet-media/lang_rust/src/rf64.rs`. Confirm that it holds a `//!` line and nothing else. Report
   a discrepancy and stop if the state differs.
2. Read `crates/bc_audio/duet-media/lang_rust/src/lib.rs` and confirm that `TakeWriter::append` already branches on the
   promotion condition and calls into `crate::rf64`. Report a discrepancy and stop if the call site
   is absent, because `lib.rs` is outside this chunk's write scope.
3. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `bwavfile`. Report a
   discrepancy and stop if the pin is absent.
4. Write the failing test `rf64_predicted_bytes_follows_the_formula` in `src/rf64.rs`, inside a
   `#[cfg(test)] mod tests`. Section 13.2 gives this chunk no `tests/` file, so every test of this
   chunk lives in this file. Run `cargo nextest run -p duet-media -E 'test(rf64)' --no-tests=fail`
   and confirm that the run fails to compile.
5. Add the `bwavfile` entry to `crates/bc_audio/duet-media/lang_rust/Cargo.toml`. Run `cargo build --workspace`, which
   settles `Cargo.lock` (SM5 rule 3).
6. Declare `PROMOTION_THRESHOLD_BYTES` and implement `predicted_bytes` and `needs_promotion`. Use
   `u64::checked_mul` and `u64::checked_add`, and answer the threshold on an overflow, because a
   product that overflows is above every limit.
7. Run the same command and confirm that the test passes.
8. Write the failing tests `rf64_round_trips_one_block` and `rf64_promotion_keeps_every_frame`. Run
   the same command and confirm that both fail.
9. Implement `read_header`, `read_frames`, `create`, `append`, and `finish` over `bwavfile`.
10. Implement `promote`. It closes the WAV sink, reads every written frame back through
    `crate::wav::read_frames` in blocks, writes each block into a new RF64 file beside the old one,
    renames the new file over the old one, and returns the open RF64 sink. The rename is the atomic
    step, so a crash during the copy leaves the WAV take readable.
11. Run the same command and confirm that every `rf64` test passes.
12. Run `cargo nextest run -p duet-media --no-tests=fail` and confirm that every test passes.
13. Run `cargo clippy -p duet-media --all-targets --locked -- -D warnings` once and repair every
    finding.
14. Run `cargo deny check` once and confirm that `bwavfile` passes the licence policy. Record the
    output in the commit body.
15. Commit on the branch `chunk/n2-rf64-promotion`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in `crates/bc_audio/duet-media/lang_rust/src/rf64.rs`, because section
13.2 gives this chunk no `tests/` file. Every assert carries a message. Every test owns one scratch
directory, which it builds from `std::env::temp_dir` plus a unique name and removes at the end.

The four tests that need a file above the container limit do not write one: a 3.9 GiB fixture would
cost more than the gate allows. Each one drives `needs_promotion` and `promote` with a small file and
a written-byte count the test sets by hand, which is the value `TakeWriter` carries.

- `rf64_predicted_bytes_follows_the_formula` — asserts that 48,000 frames of stereo at three bytes a
  sample give 288,000 plus 4096 bytes, and that an overflowing product answers the threshold.
- `rf64_needs_promotion_at_the_threshold` — asserts false one byte below B67 and true at B67, and
  true when the next block would cross it.
- `rf64_round_trips_one_block` — writes 4800 stereo frames as RF64, finishes, reopens, reads them
  back, and asserts sample equality within one least significant bit of 24 bits.
- `rf64_promotion_keeps_every_frame` — writes 4800 frames as WAV, calls `promote`, reads the result
  back through the RF64 reader, and asserts that every sample matches.
- `rf64_promotion_is_atomic_on_failure` — makes the target directory read only, calls `promote`,
  asserts `MediaError::Io`, and asserts that the original WAV take still reads every frame.
- `rf64_reader_reports_the_container` — asserts that a promoted take reads back as
  `AudioContainer::Rf64`, which `ManifestEntry::container` records.
- `rf64_open_refuses_a_plain_wav_file` — asserts `MediaError::Format` when `rf64::read_header` opens
  a file chunk N1 wrote.
- `rf64_promotion_runs_once` — calls `promote` twice on one take and asserts that the second call
  answers `MediaError::Format`, because the file is already RF64 and the container changes once
  (section 15.9).

## Verification

Passing looks like this.

```
cargo nextest run -p duet-media -E 'test(rf64)' --no-tests=fail
cargo nextest run -p duet-media --no-tests=fail
cargo clippy -p duet-media --all-targets --locked -- -D warnings
cargo deny check
cargo machete
```

The first command fails before this chunk, because the crate holds no `rf64` test. Chunk N1 wrote its
tests as `wav_*` for that reason (SM3 rule 2). The command passes after this chunk and it reports
eight tests. The second command reports every test of the crate as passed. The third command prints
no warning, and this chunk carries no `#[expect]` site. The fourth command reports no licence failure
and no advisory. The fifth command reports no unused dependency, because this chunk adds the entry in
the same commit as the code that uses it (SM1).

The work lands as one commit on the branch `chunk/n2-rf64-promotion`, with a conventional subject
such as `feat(media): promote a take to RF64 at the container limit`.

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
