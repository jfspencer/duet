---
id: E1
line: E
depends_on: [M2, D1, M94]
write_scope:
  - crates/bc_audio/duet-analysis/lang_rust/Cargo.toml
  - crates/bc_audio/duet-analysis/lang_rust/src/lib.rs
  - crates/bc_audio/duet-analysis/lang_rust/src/peaks.rs
  - crates/bc_audio/duet-analysis/lang_rust/src/pyin.rs
  - crates/bc_audio/duet-analysis/lang_rust/src/loudness.rs
  - crates/bc_audio/duet-analysis/lang_rust/src/pyin/candidates.rs
  - crates/bc_audio/duet-analysis/lang_rust/src/pyin/decode.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-analysis -E 'test(pyramid_read)' --no-tests=fail"
---

# E1: The pyramid read path over `SampleSource`

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk opens line E over the crate `duet-analysis`. Chunk M2 creates the crate
skeleton in phase 2, so `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml` and `crates/bc_audio/duet-analysis/lang_rust/src/lib.rs` exist
before this chunk starts and this chunk modifies both. Every other file of the write scope does not
exist yet, and this chunk creates it. The chunk declares `AnalysisError` and the peak read path that
turns a `SampleSource` into the `Pyramid` levels a lane draws. It implements architecture sections
5.10, 7.3, and 15.8.

Section 13.2 gives this chunk the second duty of SM2: it creates every module file of line E at
every depth as a stub, so chunks E2 and E3 modify a stub and create no source file.

**This chunk adds no `duet-session` entry and no `duet-command` entry.** Section 13.0, block
`phase-pair-exempt`, states it twice: "E1 adds NO `duet-session` entry, so nothing E1 writes needs
T3", and "E2 adds that entry one phase later for `SourceHash`". `PitchTrack` names a `SourceHash`, so
chunk E2 declares it and this chunk does not.

## Files

- `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses.
- `crates/bc_audio/duet-analysis/lang_rust/src/lib.rs` — modify. Add the three `mod` lines and `AnalysisError`.
- `crates/bc_audio/duet-analysis/lang_rust/src/peaks.rs` — create and fill. The pyramid read path.
- `crates/bc_audio/duet-analysis/lang_rust/src/pyin.rs` — create. The sibling file of the `pyin/` directory, which
  `clippy::mod_module_files` requires (SM2). It holds the two `mod` lines and nothing else.
- `crates/bc_audio/duet-analysis/lang_rust/src/loudness.rs` — create as a stub. Chunk E3 fills it.
- `crates/bc_audio/duet-analysis/lang_rust/src/pyin/candidates.rs` — create as a stub. Chunk E2 fills it.
- `crates/bc_audio/duet-analysis/lang_rust/src/pyin/decode.rs` — create as a stub. Chunk E2 fills it.
- `Cargo.lock` — modify. SM5 rule 2 puts it in the write scope of every chunk that writes a member
  manifest.

## Types and signatures

### Manifest

```toml
# crates/bc_audio/duet-analysis/lang_rust/Cargo.toml, [dependencies]
duet-time = { workspace = true }
duet-dsp = { workspace = true }
thiserror = { workspace = true }
```

Section 1.2 gives `duet-analysis` the third-party set `ebur128` and `thiserror`. `ebur128` reaches
only the loudness reader, which chunk E3 writes, so this chunk adds no `ebur128` entry and
`cargo machete` passes. Section 1.3 gives the crate four internal edges, and this chunk uses two of
them. SM4 forbids this chunk to edit the root manifest, so a missing pin is a discrepancy that this
chunk reports under SM0.

### From architecture section 15.8, module `duet_analysis` root

```rust
/// Every way a measurement refuses.
///
/// The expectation is the second of the two `variant_size_differences`
/// refusals in this plan (Appendix B.1). `Rate` carries one `SampleRate` and
/// `SourceRead` carries a fieldless enum, and the PG24 table prints both
/// sizes. The ratio test therefore refuses the difference between them, and
/// no shape that names a sample rate can satisfy it.
#[expect(
    variant_size_differences,
    reason = "the largest arm is the size the PG24 table prints for `SampleRate` and the next \
              is one arm tag, so the ratio test refuses every shape that names a sample rate"
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum AnalysisError { TooShort, SourceRead(DspError), Rate(SampleRate) }
```

The `#[expect]` attribute is the Appendix B.1 row `duet-analysis::AnalysisError`, and the reason text
above is that row's text, copied.

### Consumed types

| Type | Crate and path |
|---|---|
| `SampleSource`, `Pyramid`, `PyramidHeader`, `PeakBin`, `DspError` | `duet-dsp`, chunk D1, paths `duet_dsp::SampleSource`, `duet_dsp::Pyramid`, `duet_dsp::PyramidHeader`, `duet_dsp::PeakBin`, `duet_dsp::DspError` |
| `SampleRate` | `duet-time`, path `duet_time::SampleRate` |

Section 1.3 states the placement: `SampleSource`, `Pyramid`, `PyramidHeader`, and `PeakBin` are
`duet-dsp` types, `duet-media` implements the source, `duet-engine` writes a pyramid,
`duet-analysis` reads one, and `crates/duet` paints one.

### Module `duet_analysis::peaks`

Section 1.5 places no type in this module, so the module holds functions alone. A chunk that needs a
new type reports the discrepancy instead of inventing it (SM0).

```rust
/// The level whose bins cover at least one device pixel each, for a span of
/// `frames` frames drawn across `pixels` device pixels.
///
/// Section 5.10 states the rule: `LaneView` asks for the level whose bin
/// covers at least one device pixel, which is the level it would otherwise
/// average by hand. B68 gives the level layout.
///
/// # Errors
/// Returns `AnalysisError::TooShort` when `pixels` is zero, because a span
/// with no pixels names no level.
pub fn level_for_span(header: &PyramidHeader, frames: u64, pixels: u32) -> Result<u8, AnalysisError>;

/// Read the peak level that covers one span of one source.
///
/// It calls `duet_dsp::peaks::reader::load_level`, which reads the header,
/// seeks to the level, and reads the span the lane draws. B124 bounds the
/// result at 2 MB, so the draw path's residency does not grow with the
/// length of the take.
///
/// # Errors
/// Returns `AnalysisError::SourceRead` when the source refuses, and
/// `AnalysisError::TooShort` for a span the source does not hold.
pub fn read_span(
    source: &mut dyn SampleSource,
    from_frame: u64,
    frames: u64,
    pixels: u32,
) -> Result<Pyramid, AnalysisError>;

/// The minimum and the maximum of one bin, as a pair of unit values.
///
/// `LaneView` paints one vertical bar per device pixel column, from the min
/// value to the max value, which design contract 3.3 states. The conversion
/// from the stored `i16` pair to the unit range happens here, so no view
/// carries a scale of its own.
#[must_use]
pub fn bin_extent(bin: PeakBin) -> (Finite, Finite);
```

`Finite` is `duet_time::Finite`.

## Steps

1. Read `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml` and `crates/bc_audio/duet-analysis/lang_rust/src/lib.rs`. Confirm that chunk
   M2 created both, that the manifest carries `[lints] workspace = true`, a `description`, and no
   `[dependencies]` section, and that `lib.rs` carries the `//!` crate documentation and
   `#![forbid(unsafe_code)]`. Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` carries `thiserror`,
   `duet-time`, and `duet-dsp`. Report a discrepancy and stop if an entry is absent.
3. Create the five module files. Fill `peaks.rs` and `pyin.rs`. Leave `loudness.rs`,
   `pyin/candidates.rs`, and `pyin/decode.rs` at one `//!` line each.
4. Add the three `mod` lines to `src/lib.rs` and declare `AnalysisError` with its `#[expect]`
   attribute.
5. Add the three `{ workspace = true }` entries to `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml`. Run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3).
6. Write the failing test `pyramid_read_picks_the_level_for_the_span` in `src/peaks.rs`, inside a
   `#[cfg(test)] mod tests`. Run
   `cargo nextest run -p duet-analysis -E 'test(pyramid_read)' --no-tests=fail` and confirm that the
   run fails to compile.
7. Implement `level_for_span`. Compute the frames per pixel, and take the smallest level whose bin
   covers at least that many frames under B68. Use `u64::checked_div` and never the `/` operator,
   because `clippy::integer_division` is denied.
8. Run the same command and confirm that the test passes.
9. Write the remaining `pyramid_read` tests of the Tests section. Run the same command and confirm
   that every one passes.
10. Implement `read_span` and `bin_extent`. `read_span` converts every `DspError` into
    `AnalysisError::SourceRead` with a named `map_err`, and it never drops the source error, because
    `clippy::map_err_ignore` is denied.
11. Run `cargo nextest run -p duet-analysis --no-tests=fail` and confirm that every test passes.
12. Run `cargo clippy -p duet-analysis --all-targets --locked -- -D warnings` once and repair every
    finding. Confirm that the `#[expect]` attribute on `AnalysisError` is fulfilled, because
    `-D warnings` turns an unfulfilled expectation into an error.
13. Commit on the branch `chunk/e1-pyramid-read-path`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.
A test double implements `duet_dsp::SampleSource` over a byte vector, so the crate stays pure and
opens no file; section 1.4 names `duet-analysis` a pure crate for that reason.

`crates/bc_audio/duet-analysis/lang_rust/src/peaks.rs`

- `pyramid_read_picks_the_level_for_the_span` — asserts level 0 for 64 frames across 64 pixels, and
  level 4 for 65,536 frames across 64 pixels, against the B68 layout.
- `pyramid_read_level_never_passes_the_header` — asserts that a span longer than the source answers
  the last level the header holds, and never a level past it.
- `pyramid_read_refuses_a_zero_pixel_span` — asserts `AnalysisError::TooShort`.
- `pyramid_read_returns_the_requested_bin_count` — asserts that `read_span` answers one bin per
  device pixel, within one bin.
- `pyramid_read_wraps_a_source_failure` — asserts that a source double which refuses gives
  `AnalysisError::SourceRead(DspError::SourceRead)`, and that the wrapped value is the source's own.
- `pyramid_read_bin_extent_is_in_the_unit_range` — asserts that `bin_extent` answers a pair inside
  minus one to plus one for the extreme `i16` values, and that the min is at or below the max.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-analysis -E 'test(pyramid_read)' --no-tests=fail
cargo nextest run -p duet-analysis --no-tests=fail
cargo clippy -p duet-analysis --all-targets --locked -- -D warnings
cargo machete
```

The first command fails before this chunk, because the crate holds no `pyramid_read` test, and
`--no-tests=fail` makes an empty match a failure (SM3 rule 1). It passes after this chunk and it
reports six tests. The second command reports every test of the crate as passed. The third command
prints no warning, and the one `#[expect]` site is the Appendix B.1 row for `AnalysisError`. The
fourth command reports no unused dependency.

The work lands as one commit on the branch `chunk/e1-pyramid-read-path`, with a conventional subject
such as `feat(analysis): read one peak pyramid level per drawn span`.

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
