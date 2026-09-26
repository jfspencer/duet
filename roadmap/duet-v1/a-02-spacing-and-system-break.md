---
id: A2
line: A
depends_on: [M3, A1]
write_scope:
  - crates/bc_notation/duet-engrave/lang_rust/src/spacing.rs
  - crates/bc_notation/duet-engrave/lang_rust/src/system.rs
parallelism: independent
completion: "cargo nextest run -p duet-engrave -E 'test(spacing)' --no-tests=fail"
---

# A2: Horizontal spacing and the system break

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk A1 created `spacing.rs` and `system.rs` as stubs in phase 2, so both files exist
and hold a `//!` line alone. This chunk fills both. It gives each event its width from its duration,
it justifies each system to the trailing margin, and it breaks the score into systems at the page
width. It implements architecture sections 10.5 and 15.6 and design contract section 2.3, and it
carries the product stories C-05, C-06, X-03, and X-04.

The chunk writes no manifest, so `Cargo.lock` is not in the write scope (SM5 rule 2). The crate
already holds every dependency this chunk needs, from chunk A1.

## Files

- `crates/bc_notation/duet-engrave/lang_rust/src/spacing.rs` — modify. The column widths, the minimum gap, and the
  justification.
- `crates/bc_notation/duet-engrave/lang_rust/src/system.rs` — modify. The system break and the vertical system order.

## Types and signatures

Section 1.5 places no new type in either module, so both modules hold functions alone. A chunk that
needs a new type reports the discrepancy instead of inventing it (SM0).

### Consumed types

| Type | Crate and path |
|---|---|
| `SystemPlacement`, `SystemId`, `SystemXMap`, `Knot`, `EngraveError`, `LayoutOptions`, `FontMetrics`, `SizePx`, `QuadPlacement`, `GlyphPlacement`, `NoteHit` | `duet-engrave`, chunk A1, modules `placement`, `map`, and `metrics` |
| `Score`, `NoteId`, `ScoreError`, `Duration` | `duet-score`, paths `duet_score::Score`, `duet_score::NoteId`, `duet_score::ScoreError`, `duet_score::Duration` |
| `Ticks`, `TempoMap`, `NoteValue` | `duet-time`, paths `duet_time::Ticks`, `duet_time::TempoMap`, `duet_time::NoteValue` |

### Module `duet_engrave::spacing`

```rust
/// One engraved column: every event that starts at one tick position.
#[derive(Debug, Clone, PartialEq)]
pub struct Column { at: Ticks, width: f32, glyphs: Vec<GlyphPlacement>, hits: Vec<NoteHit> }

/// The width one duration asks for, in staff spaces.
///
/// Design contract 2.3 gives `width(d) = 4.0 sp * (d / quarter) ^ 0.6`, so a
/// whole note is 9.2 sp, a half is 6.1 sp, a quarter is 4.0 sp, an eighth is
/// 2.6 sp, and a sixteenth is 1.7 sp.
#[must_use]
pub fn duration_width(duration: Duration) -> f32;

/// Place every column of one system, and return the placed columns with the
/// map that reads them.
///
/// # Errors
/// Returns `EngraveError::Overflow` when the placed columns pass the page
/// width that `LayoutOptions` states.
#[expect(
    too_many_lines,
    reason = "one pass over every column of one system, carrying four running values: the last \
              column, the accumulated width, the open tuplet, and the open beam group"
)]
pub fn place_columns(
    score: &Score,
    system: SystemId,
    range: (Ticks, Ticks),
    options: &LayoutOptions,
    metrics: &FontMetrics,
) -> Result<(Vec<Column>, SystemXMap), EngraveError>;

/// Stretch the placed columns so the last barline lands on the trailing
/// margin.
///
/// Design contract 2.3 leaves the last system unjustified when it holds less
/// than 60 percent of the available width.
pub fn justify(columns: &mut [Column], map: &mut SystemXMap, available: f32, last: bool);
```

`Column` is a module type of `duet-engrave` that no other crate names, so it stays inside the crate
and `SystemPlacement` remains the one value the paint element receives (section 10.5).

The `#[expect(too_many_lines, ...)]` attribute on `place_columns` is the Appendix B.1 row
`duet-engrave::spacing::place_columns`, and the reason text above is that row's text, copied.

### Module `duet_engrave::system`

```rust
/// The clear space above and below each system, in staff spaces.
///
/// Design contract 2.3 gives 12 sp in Compose mode, and never less than 9 sp
/// after collision correction.
pub const SYSTEM_GAP_SP: f32 = 12.0;

/// The floor the collision correction may reach.
pub const SYSTEM_GAP_MIN_SP: f32 = 9.0;

/// The clear space at the leading and the trailing edge of a system.
pub const SYSTEM_MARGIN_SP: f32 = 3.0;

/// Break the score into systems at the page width `LayoutOptions` states.
///
/// Each system covers one tick range, and the ranges are contiguous and
/// ordered. The returned list is the input of `engrave`.
///
/// # Errors
/// Returns `EngraveError::Overflow` when one measure alone cannot fit the
/// page width, because a break inside a measure is not a layout this design
/// draws.
pub fn break_into_systems(
    score: &Score,
    map: &TempoMap,
    options: &LayoutOptions,
    metrics: &FontMetrics,
) -> Result<Vec<(SystemId, (Ticks, Ticks))>, EngraveError>;

/// The size and the origin of one system, after the break.
///
/// The height grows to fit leger lines, lyrics, and dynamics, which design
/// contract 2.3 requires.
#[must_use]
pub fn system_size(columns: &[Column], options: &LayoutOptions, metrics: &FontMetrics) -> SizePx;
```

`engrave` in `placement.rs` calls `break_into_systems`, then `place_columns` and `justify` per
system, then `system_size`. Chunk A1 declared `engrave`, and this chunk changes no signature of it.

## Steps

1. Read `crates/bc_notation/duet-engrave/lang_rust/src/spacing.rs` and `crates/bc_notation/duet-engrave/lang_rust/src/system.rs`. Confirm that
   each one holds a `//!` line and nothing else. Report a discrepancy and stop if the state differs.
2. Write the failing test `spacing_width_follows_the_duration_curve` in `src/spacing.rs`, inside a
   `#[cfg(test)] mod tests`. Run
   `cargo nextest run -p duet-engrave -E 'test(spacing)' --no-tests=fail` and confirm that the run
   fails to compile.
3. Implement `duration_width`. Convert the duration to a fraction of a quarter note through
   `duet_time::NoteValue`, raise it to the power 0.6 with `f32::powf`, and multiply by 4.0. Use
   `duet_time::convert` for every narrowing, because `as` is denied outside that module.
4. Run the same command and confirm that the test passes.
5. Write the failing tests `spacing_keeps_the_minimum_gap` and
   `spacing_justifies_to_the_trailing_margin`. Run the same command and confirm that both fail.
6. Declare `Column`. Implement `place_columns`. It walks the score events of the tick range in
   order, groups every event that starts at one tick into one `Column`, takes the column width as
   the larger of `duration_width` and the glyph advance plus the 1.6 staff-space minimum gap, and
   accumulates the x offsets into a `SystemXMap`. It returns `EngraveError::Overflow` when the
   accumulated width passes the page width.
7. Implement `justify`. It spreads the residual width across the gaps in proportion to each gap's
   own width and rewrites the map knots. It returns with no change when `last` is true and the
   accumulated width is below 60 percent of `available`.
8. Run the same command and confirm that every `spacing` test passes.
9. Write the failing tests `system_breaks_at_the_page_width` and
   `system_height_grows_for_leger_lines` in `src/system.rs`. Run
   `cargo nextest run -p duet-engrave -E 'test(system)' --no-tests=fail` and confirm that both fail.
10. Declare the three constants. Implement `break_into_systems` and `system_size`.
11. Change `engrave` in `src/placement.rs` to call the two modules. `placement.rs` is not in this
    chunk's write scope, so the call site must already exist from chunk A1. Report a discrepancy and
    stop if `engrave` does not already call `spacing::place_columns` and `system::break_into_systems`
    behind the stub.
12. Run `cargo nextest run -p duet-engrave --no-tests=fail` and confirm that every test passes.
13. Run `cargo clippy -p duet-engrave --all-targets --locked -- -D warnings` once and repair every
    finding.
14. Commit on the branch `chunk/a2-spacing-and-system-break`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.

`crates/bc_notation/duet-engrave/lang_rust/src/spacing.rs`

- `spacing_width_follows_the_duration_curve` — asserts the five widths design contract 2.3 prints:
  9.2 sp for a whole note, 6.1 sp for a half, 4.0 sp for a quarter, 2.6 sp for an eighth, and 1.7 sp
  for a sixteenth, each within 0.05 sp.
- `spacing_keeps_the_minimum_gap` — places two adjacent sixteenth notes and asserts that the gap
  between the two head centres is at or above 1.6 staff spaces.
- `spacing_justifies_to_the_trailing_margin` — asserts that the last knot of the map equals the
  page width less the trailing margin after `justify`.
- `spacing_leaves_a_short_last_system_unjustified` — asserts that `justify` with `last` true and a
  fill below 60 percent changes no knot.
- `spacing_refuses_a_column_run_past_the_page_width` — asserts that `place_columns` returns
  `EngraveError::Overflow` for a measure whose columns pass the page width.
- `spacing_is_monotone` — asserts that the returned `SystemXMap` is strictly increasing, which is
  the property `SystemXMap::new` enforces.

`crates/bc_notation/duet-engrave/lang_rust/src/system.rs`

- `system_breaks_at_the_page_width` — engraves twelve measures at a page width that fits four, and
  asserts three systems with contiguous, ordered tick ranges.
- `system_ranges_are_contiguous` — asserts that each system's start tick equals the previous
  system's end tick.
- `system_height_grows_for_leger_lines` — asserts that a system with a note five leger lines above
  the staff reports a larger `SizePx::height` than the same system with no leger line.
- `system_refuses_a_measure_wider_than_the_page` — asserts `EngraveError::Overflow`.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-engrave -E 'test(spacing)' --no-tests=fail
cargo nextest run -p duet-engrave --no-tests=fail
cargo clippy -p duet-engrave --all-targets --locked -- -D warnings
```

The first command fails before this chunk, because the crate holds no `spacing` test. It passes
after this chunk. The second command reports every test of the crate as passed, and the count grows
by the ten tests above. The third command prints no warning, and the one `#[expect]` site is the
Appendix B.1 row for `place_columns`.

The work lands as one commit on the branch `chunk/a2-spacing-and-system-break`, with a conventional
subject such as `feat(engrave): space the columns and break the score into systems`.

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
