---
id: E2
line: E
depends_on: [M3, E1]
write_scope:
  - crates/bc_audio/duet-analysis/lang_rust/src/pyin/candidates.rs
  - crates/bc_audio/duet-analysis/lang_rust/src/pyin/decode.rs
  - crates/bc_audio/duet-analysis/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-analysis -E 'test(pyin)' --no-tests=fail"
---

# E2: pYIN, the per-frame candidates and the path decode

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk E1 created both files as stubs in phase 2, so each one holds a `//!` line alone.
This chunk fills both. It computes the pitch candidates of each analysis frame, it decodes one path
over the whole take, and it declares `PitchTrack`. The operator decision is final: pitch detection is
in-house pYIN over `rustfft`, which this crate reaches through `duet_dsp::fft`. The chunk implements
architecture sections 1.2, 1.3, and 15.8, and it carries the product story R-15, "Pitch track
overlay", which is a SHOULD.

The chunk writes the member manifest, so `Cargo.lock` is in the write scope (SM5 rule 2).

## The `duet-session` entry

`PitchTrack` names the `SourceHash` it measured (section 15.8), which is the `duet-analysis ->
duet-session` edge of section 1.3. Section 13.0, block `phase-pair-exempt`, states that chunk "E2
adds that entry one phase later for `SourceHash`". **This chunk adds
`duet-session = { workspace = true }` to `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml`** (SM1), and `Cargo.lock`
goes in the same commit (SM5 rule 2). Architecture section 13.2 names all four paths in the E2 Writes
cell. The cell named the two source files alone until this revision, against section 13.0's own
sentence; the chunk author of line E reported it and the Architect corrected the cell.

Chunk T3 created `duet-session` in phase 2, so the internal root entry
`duet-session = { path = "crates/bc_document/duet-session/lang_rust" }` already exists (SM1 rule 4, written by chunk M2).
Read the root `Cargo.toml` first and report a discrepancy if that entry is absent, because SM4
forbids this chunk to edit the root manifest.

## Files

- `crates/bc_audio/duet-analysis/lang_rust/src/pyin/candidates.rs` — modify. The per-frame candidate set.
- `crates/bc_audio/duet-analysis/lang_rust/src/pyin/decode.rs` — modify. `PitchTrack` and the path decode.
- `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml` — modify. Add the `duet-session = { workspace = true }` entry.
- `Cargo.lock` — modify. `cargo build --workspace` settles it (SM5 rule 3).

## Types and signatures

### Consumed types

| Type | Crate and path |
|---|---|
| `AnalysisError` | `duet-analysis`, chunk E1, crate root |
| `SampleSource`, `DspError`, `fft::forward` | `duet-dsp`, chunk D1, paths `duet_dsp::SampleSource`, `duet_dsp::DspError`, `duet_dsp::fft::forward` |
| `SourceHash` | `duet-session`, path `duet_session::SourceHash`. See "The `duet-session` entry" above. |
| `Finite`, `SampleRate` | `duet-time`, paths `duet_time::Finite`, `duet_time::SampleRate` |

Section 1.3 states the placement: `duet-analysis` reaches the transforms through `duet_dsp::fft`, so
`rustfft` and `realfft` sit in `duet-dsp` alone and this crate names neither.

### From architecture section 15.8, module `duet_analysis::pyin::decode`

```rust
/// One pitch track over one source, at a fixed frame step.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PitchTrack { source: SourceHash, hop_frames: NonZeroU32, hz: Vec<Finite>, confidence: Vec<Finite> }
```

`hz` and `confidence` hold one value per analysis frame, and the two lists are the same length. A
frame with no voiced pitch carries `Finite::ZERO` in `hz` and a confidence at or below the voiced
threshold.

### Module `duet_analysis::pyin::candidates`

Section 1.5 places no type in this module, so the module holds functions and one module type that no
other crate names.

```rust
/// One pitch candidate of one analysis frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Candidate { hz: Finite, probability: Finite }

/// The candidates one analysis frame offers (B148).
///
/// The count is fixed, so the decode of `decode_path` walks a fixed state
/// set and the Viterbi cost is a product of two known numbers.
pub const CANDIDATES_PER_FRAME: usize = 12;

/// Compute the candidate set of one analysis frame.
///
/// It runs the cumulative mean normalized difference of YIN over the frame,
/// then it weighs every trough by the beta distribution that pYIN states, and
/// it keeps the `CANDIDATES_PER_FRAME` largest probabilities.
///
/// # Errors
/// Returns `AnalysisError::TooShort` when the frame is shorter than two
/// periods of the lowest pitch the range holds, and
/// `AnalysisError::Rate(rate)` for a sample rate the analysis window cannot
/// resolve.
pub fn frame_candidates(
    frame: &[f32],
    rate: SampleRate,
    out: &mut [Candidate; CANDIDATES_PER_FRAME],
) -> Result<(), AnalysisError>;
```

### Module `duet_analysis::pyin::decode`

```rust
/// Measure the pitch of one whole source.
///
/// It reads the source block by block, computes the candidate set of each
/// analysis frame, and decodes one path over the whole take. The result is a
/// `PitchTrack` at the hop the caller asked for.
///
/// # Errors
/// Returns `AnalysisError::SourceRead` when the source refuses,
/// `AnalysisError::TooShort` for a source below one analysis frame, and
/// `AnalysisError::Rate` for a sample rate the analysis cannot resolve.
pub fn measure(
    source: &mut dyn SampleSource,
    hash: SourceHash,
    rate: SampleRate,
    hop_frames: NonZeroU32,
) -> Result<PitchTrack, AnalysisError>;

/// Decode one path over the candidate sets of every frame.
///
/// The transition cost favours a small pitch change, so an octave error in
/// one frame does not survive the path.
#[expect(
    too_many_lines,
    cognitive_complexity,
    reason = "one Viterbi loop nest over the B148 candidate states, with the three state arrays \
              of section 15.8 live across every iteration"
)]
pub fn decode_path(frames: &[[Candidate; CANDIDATES_PER_FRAME]], out: &mut PitchTrack);
```

The `#[expect]` attribute is the Appendix B.1 row `duet-analysis::pyin::decode::decode_path`, and the
reason text above is that row's text, copied. **The row names two lints and one attribute carries
both**, which is what Appendix B.1 means by one site.

**B148 is the candidate count.** Section 1.6 gives B148 the value 12 candidates, which is the state
set one frame step of the decode walks, and section 15.8 names the three state arrays the loop nest
holds live. `CANDIDATES_PER_FRAME` is the module-private constant of `duet_analysis::pyin` that
carries that value, and its doc line names B148, so no loop bound writes a bare number (DR3). The
reason text cited B75 until this revision, and B75 is the loudness preset row; the chunk author of
line E reported it and the Architect minted B148 with the value this chunk already declared.

`PitchTrack` also answers the section 15.8 rule for B125: `duet-analysis` compares each measured `hz`
with the engraved note and reports every frame past B125, which is 25 cents, so the threshold is one
number in section 1.6 and never a literal at a draw site.

```rust
/// Every frame whose measured pitch is more than B125 from `expected_hz`.
///
/// B125 is 25 cents (PR R-15). The comparison lives here, so no draw site
/// carries the threshold.
#[must_use]
pub fn frames_off_pitch(track: &PitchTrack, expected_hz: Finite) -> Vec<u32>;
```

## Steps

1. Read both files. Confirm that each one holds a `//!` line and nothing else. Report a discrepancy
   and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` carries
   `duet-session = { path = "crates/bc_document/duet-session/lang_rust" }`, which chunk M2 wrote in phase 2. Report a
   discrepancy and stop if the entry is absent. Add `duet-session = { workspace = true }` to
   `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml` and run `cargo build --workspace`, which settles `Cargo.lock`
   (SM5 rule 3).
3. Write the failing test `pyin_candidates_find_a_pure_tone` in `src/pyin/candidates.rs`, inside a
   `#[cfg(test)] mod tests`. Run
   `cargo nextest run -p duet-analysis -E 'test(pyin)' --no-tests=fail` and confirm that the run
   fails to compile.
4. Declare `Candidate` and `CANDIDATES_PER_FRAME`. Implement `frame_candidates`. Compute the
   difference function through `duet_dsp::fft::forward`, normalize it, find every trough below the
   threshold, and weigh each one. Write the result into the caller's array, so the function
   allocates nothing per frame.
5. Run the same command and confirm that the test passes.
6. Write the failing tests `pyin_decode_follows_a_glide` and `pyin_decode_rejects_an_octave_error`
   in `src/pyin/decode.rs`. Run the same command and confirm that both fail.
7. Declare `PitchTrack` with an accessor per field, each one `#[must_use]`. Implement `decode_path`
   as one Viterbi loop nest over the fixed candidate state set, with the three state arrays live
   across every iteration. Carry the `#[expect]` attribute above.
8. Implement `measure`. It reads the source in blocks, it fills one candidate array per analysis
   frame, and it calls `decode_path` once at the end. It converts every `DspError` into
   `AnalysisError::SourceRead` with a named `map_err`, and it never drops the source error.
9. Run the same command and confirm that every `pyin` test passes.
10. Implement `frames_off_pitch`, with B125 as one named constant of the module, documented with its
    budget id.
11. Run `cargo nextest run -p duet-analysis --no-tests=fail` and confirm that every test passes.
12. Run `cargo clippy -p duet-analysis --all-targets --locked -- -D warnings` once and repair every
    finding. Confirm that the `#[expect]` attribute on `decode_path` is fulfilled, because
    `-D warnings` turns an unfulfilled expectation into an error.
13. Commit on the branch `chunk/e2-pyin-pitch-track`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.
A test double implements `duet_dsp::SampleSource` over a generated tone, so the crate stays pure and
opens no file.

`crates/bc_audio/duet-analysis/lang_rust/src/pyin/candidates.rs`

- `pyin_candidates_find_a_pure_tone` — feeds a 220 Hz sine at 48 kHz and asserts that the largest
  probability sits within 1 cent of 220 Hz.
- `pyin_candidates_rank_the_true_pitch_first` — feeds a sawtooth at 165 Hz and asserts that the
  first candidate is the fundamental and not the second harmonic.
- `pyin_candidates_report_low_probability_on_noise` — feeds white noise and asserts that every
  probability is below 0.3.
- `pyin_candidates_refuse_a_short_frame` — asserts `AnalysisError::TooShort`.
- `pyin_candidates_allocate_nothing_per_frame` — calls `frame_candidates` one thousand times over
  one array and asserts that the array length never changes, which is the property the fixed
  candidate count exists for.

`crates/bc_audio/duet-analysis/lang_rust/src/pyin/decode.rs`

- `pyin_decode_follows_a_glide` — feeds a tone that rises from 200 Hz to 400 Hz over one second and
  asserts that the decoded track rises with no step larger than 60 cents between two frames.
- `pyin_decode_rejects_an_octave_error` — plants a single frame whose largest candidate is one
  octave above the true pitch and asserts that the decoded value stays on the true pitch.
- `pyin_decode_track_lists_are_the_same_length` — asserts that `hz` and `confidence` hold one value
  per analysis frame.
- `pyin_measure_names_its_source` — asserts that the returned `PitchTrack` carries the `SourceHash`
  the caller passed.
- `pyin_measure_refuses_a_source_below_one_frame` — asserts `AnalysisError::TooShort`.
- `pyin_measure_wraps_a_source_failure` — asserts `AnalysisError::SourceRead(DspError::SourceRead)`.
- `pyin_frames_off_pitch_marks_past_b125` — builds a track 30 cents above the expected pitch and
  asserts that every frame is marked, then builds one 20 cents above and asserts that no frame is
  marked.
- `pyin_measure_is_deterministic` — measures one source twice and asserts equal tracks, which the
  offline and reproducible property of section 1.2 requires.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-analysis -E 'test(pyin)' --no-tests=fail
cargo nextest run -p duet-analysis --no-tests=fail
cargo clippy -p duet-analysis --all-targets --locked -- -D warnings
```

The first command fails before this chunk, because the crate holds no `pyin` test. Chunk E1 wrote
its tests as `pyramid_read_*` for that reason (SM3 rule 2). The command passes after this chunk and
it reports thirteen tests. The second command reports every test of the crate as passed. The third
command prints no warning, and the one new `#[expect]` site is the Appendix B.1 row for
`decode_path`.

The work lands as one commit on the branch `chunk/e2-pyin-pitch-track`, with a conventional subject
such as `feat(analysis): measure pitch with in-house pYIN`.

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
