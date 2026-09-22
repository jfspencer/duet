---
id: E3
line: E
depends_on: [M4, E2]
write_scope:
  - crates/duet-analysis/src/loudness.rs
  - crates/duet-analysis/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-analysis -E 'test(loudness_read)' --no-tests=fail"
---

# E3: The loudness reader over ebur128

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk E1 created `crates/duet-analysis/src/loudness.rs` as a stub in phase 2, so the file
holds a `//!` line alone. This chunk fills it. It measures the integrated loudness, the short-term
loudness, the momentary loudness, the true peak, and the loudness range of one source, and it answers
one `LoudnessReport`. Chunk H2 then builds the two-pass loudness graph on it, which section 13.4
states as the link "E3 before H2". The chunk implements architecture sections 1.2, 1.3, 7.4, and
15.5, and it carries the measurement half of the product stories MA-01 and MA-03.

This chunk adds two dependencies, so SM1 puts `crates/duet-analysis/Cargo.toml` in its write scope
and SM5 rule 2 puts `Cargo.lock` there beside it. Section 13.2 names both files in this chunk's
Writes cell.

## Files

- `crates/duet-analysis/src/loudness.rs` — modify. The loudness reader.
- `crates/duet-analysis/Cargo.toml` — modify. Add `ebur128` and `duet-command`.
- `Cargo.lock` — modify.

## Types and signatures

### Manifest

```toml
# crates/duet-analysis/Cargo.toml, [dependencies], added by this chunk
duet-command = { workspace = true }
ebur128 = { workspace = true }
```

Chunk M4 pins `ebur128` in phase 4, which section 13.1 states. Section 1.2 gives `duet-analysis` the
third-party set `ebur128` and `thiserror`, and chunk E1 added `thiserror`. Section 1.3 gives the edge
`duet-analysis -> duet-command`, and section 1.3 states its reason: "`LoudnessReport` crosses the
transport inside `VerbData`, so VR3 declares it in `duet-command` and the measurement crate consumes
it". Section 13.0, block `phase-pair-exempt`, states the same: "E3 adds `duet-command` for
`LoudnessReport`". SM4 forbids this chunk to edit the root manifest, so a missing pin is a
discrepancy that this chunk reports under SM0.

### From architecture section 15.5, consumed from `duet-command`

```rust
/// What a loudness measurement produced.
///
/// It is declared here and not in `duet-analysis`, because `VerbData` carries
/// it and VR3 binds. A `duet-command` edge to `duet-analysis` would instead
/// pull `ebur128` into the nine crates that depend on the vocabulary
/// (critic S2, S4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoudnessReport { integrated_lufs: Finite, short_term_lufs: Finite, momentary_lufs: Finite, true_peak_dbtp: Finite, range_lu: Finite }
```

The path is `duet_command::LoudnessReport`. This chunk declares no mirror of it, because section 1.5
places it in `duet-command` and PL3 gives one name to one type.

### Other consumed types

| Type | Crate and path |
|---|---|
| `AnalysisError` | `duet-analysis`, chunk E1, crate root |
| `SampleSource`, `DspError` | `duet-dsp`, chunk D1 |
| `Finite`, `SampleRate`, `ChannelIndex` | `duet-time` |

### Module `duet_analysis::loudness`

Section 1.5 places no type in this module, so the module holds functions alone. A chunk that needs a
new type reports the discrepancy instead of inventing it (SM0).

```rust
/// Measure the loudness of one whole source.
///
/// It reads the source block by block into one `ebur128::EbuR128` state and
/// answers the five values of `LoudnessReport`. The measurement is offline,
/// reproducible, and separate from the real-time path, which section 1.2
/// states is the one invariant this crate protects.
///
/// # Errors
/// Returns `AnalysisError::SourceRead` when the source refuses,
/// `AnalysisError::TooShort` for a source shorter than one 400 ms momentary
/// window, and `AnalysisError::Rate(rate)` for a sample rate `ebur128` does
/// not accept.
pub fn measure(
    source: &mut dyn SampleSource,
    rate: SampleRate,
    channels: ChannelIndex,
) -> Result<LoudnessReport, AnalysisError>;

/// The gain one render applies to reach `target_lufs`, in decibels.
///
/// The two-pass render of section 7.4 measures first and applies gain
/// second, which chunk H2 builds on this function. The value is the
/// difference between the target and the measured integrated loudness, so a
/// quiet source answers a positive gain.
#[must_use]
pub fn normalization_gain_db(report: LoudnessReport, target_lufs: Finite) -> Finite;

/// The block a streaming measurement reads at a time, in frames.
///
/// It is the 100 ms `ebur128` block at 48 kHz, so every window boundary the
/// standard names falls on a block boundary and no partial window is ever
/// measured twice.
pub const MEASURE_BLOCK_FRAMES: u32 = 4_800;
```

B75 gives the two loudness presets: Streaming is -14 LUFS and -1 dBTP, and Broadcast is -23 LUFS and
-1 dBTP. B76 gives the loudness tolerance the end-to-end test asserts, at 0.2 LU. Neither number
sits in this module: chunk H4 writes the preset set and section 14 asserts the tolerance, so this
module carries no target of its own.

## Steps

1. Read `crates/duet-analysis/src/loudness.rs`. Confirm that it holds a `//!` line and nothing else.
   Report a discrepancy and stop if the state differs.
2. Read `crates/duet-analysis/Cargo.toml`. Confirm that chunks E1 and E2 left it with the entries
   those chunks added and with no `ebur128` entry. Report a discrepancy and stop if the state
   differs.
3. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `ebur128` and carries the
   `duet-command` entry. Report a discrepancy and stop if either one is absent.
4. Write the failing test `loudness_read_measures_a_known_tone` in `src/loudness.rs`, inside a
   `#[cfg(test)] mod tests`. Run
   `cargo nextest run -p duet-analysis -E 'test(loudness_read)' --no-tests=fail` and confirm that the
   run fails to compile.
5. Add the two `{ workspace = true }` entries to `crates/duet-analysis/Cargo.toml`. Run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3).
6. Implement `measure`. Build one `ebur128::EbuR128` with the integrated, the short-term, the
   momentary, the true-peak, and the loudness-range modes. Read `MEASURE_BLOCK_FRAMES` frames at a
   time into one reused buffer, so the measurement allocates nothing after the first block. Convert
   every `DspError` into `AnalysisError::SourceRead` with a named `map_err`, and never drop the
   source error, because `clippy::map_err_ignore` is denied. Convert every `ebur128` refusal into
   `AnalysisError::Rate(rate)` or `AnalysisError::TooShort`, and name each arm, because
   `clippy::wildcard_enum_match_arm` is denied.
7. Run the same command and confirm that the test passes.
8. Write the remaining `loudness_read` tests of the Tests section. Run the same command and confirm
   that every one passes.
9. Implement `normalization_gain_db`. Use checked arithmetic on `Finite`, because a NaN reaches no
   taper (VR4).
10. Run `cargo nextest run -p duet-analysis --no-tests=fail` and confirm that every test passes.
11. Run `cargo clippy -p duet-analysis --all-targets --locked -- -D warnings` once and repair every
    finding.
12. Run `cargo deny check` once and confirm that `ebur128` passes the licence policy. Record the
    output in the commit body.
13. Commit on the branch `chunk/e3-loudness-reader`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.
A test double implements `duet_dsp::SampleSource` over a generated signal, so the crate stays pure
and opens no file; section 1.4 names `duet-analysis` a pure crate for that reason.

`crates/duet-analysis/src/loudness.rs`

- `loudness_read_measures_a_known_tone` — feeds a 1 kHz sine at minus 20 dBFS for ten seconds and
  asserts an integrated loudness of minus 20.0 LUFS within 0.2 LU, which is B76.
- `loudness_read_reports_the_true_peak` — feeds a signal whose inter-sample peak passes full scale
  and asserts a `true_peak_dbtp` above zero.
- `loudness_read_reports_a_loudness_range` — feeds ten seconds at minus 30 LUFS followed by ten
  seconds at minus 10 LUFS and asserts a `range_lu` near 20.
- `loudness_read_refuses_a_short_source` — asserts `AnalysisError::TooShort` for a source of 100 ms.
- `loudness_read_wraps_a_source_failure` — asserts `AnalysisError::SourceRead(DspError::SourceRead)`
  and that the wrapped value is the source's own.
- `loudness_read_is_deterministic` — measures one source twice and asserts an equal report, which
  the reproducible property of section 1.2 requires.
- `loudness_read_allocates_no_block_per_read` — reads a ten-minute source and asserts that the reused
  buffer length never changes.
- `loudness_read_gain_reaches_the_streaming_target` — asserts that `normalization_gain_db` of a
  minus 20 LUFS report against a minus 14 LUFS target answers plus 6.0 decibels within 0.01.
- `loudness_read_gain_reaches_the_broadcast_target` — asserts minus 3.0 decibels against a minus 23
  LUFS target, from the same report.

No property test enters this chunk, and no `#[expect]` site enters it either.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-analysis -E 'test(loudness_read)' --no-tests=fail
cargo nextest run -p duet-analysis --no-tests=fail
cargo clippy -p duet-analysis --all-targets --locked -- -D warnings
cargo deny check
cargo machete
```

The first command fails before this chunk, because the crate holds no `loudness_read` test. It
passes after this chunk and it reports nine tests. The second command reports every test of the crate
as passed. The third command prints no warning. The fourth command reports no licence failure and no
advisory. The fifth command reports no unused dependency, because this chunk adds the two entries in
the same commit as the code that uses them (SM1).

The work lands as one commit on the branch `chunk/e3-loudness-reader`, with a conventional subject
such as `feat(analysis): measure loudness and true peak over ebur128`.

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
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every
  `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the
  root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no
  pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with
  Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this
  repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match
  what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
