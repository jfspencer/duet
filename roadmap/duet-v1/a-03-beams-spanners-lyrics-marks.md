---
id: A3
line: A
depends_on: [M4, A2]
write_scope:
  - crates/duet-engrave/src/beam.rs
  - crates/duet-engrave/src/spanner.rs
  - crates/duet-engrave/src/lyric.rs
  - crates/duet-engrave/src/mark.rs
parallelism: independent
completion: "cargo nextest run -p duet-engrave -E 'test(beam)' --no-tests=fail"
---

# A3: Beams, ties, slurs, dynamics, lyrics, and marks

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk A1 created the four files as stubs in phase 2, so each one holds a `//!` line
alone. This chunk fills all four. It groups and slopes the beams, it draws the ties and the slurs as
filled paths, it places the dynamics and the articulations, it places the lyric syllables under the
staff, and it places the rehearsal marks and the repeat barlines. It implements architecture
sections 10.4, 10.5, and 15.6 and design contract sections 2.3 and 2.6, and it carries the product
story C-19 and the placement half of C-18.

The chunk writes no manifest, so `Cargo.lock` is not in the write scope (SM5 rule 2).

## Files

- `crates/duet-engrave/src/beam.rs` — modify. The beam group, the beam slant, and the partial beam.
- `crates/duet-engrave/src/spanner.rs` — modify. Ties, slurs, and hairpins as filled paths.
- `crates/duet-engrave/src/lyric.rs` — modify. Lyric syllables, one row per verse.
- `crates/duet-engrave/src/mark.rs` — modify. Dynamics, articulations, rehearsal marks, and repeats.

## Types and signatures

Section 1.5 places no new type in these four modules, so each one holds functions alone. A chunk
that needs a new type reports the discrepancy instead of inventing it (SM0).

### Consumed types

| Type | Crate and path |
|---|---|
| `PathPlacement`, `GlyphPlacement`, `QuadPlacement`, `Symbol`, `FontMetrics`, `LayoutOptions`, `EngraveError`, `SystemXMap` | `duet-engrave`, chunk A1, modules `placement`, `metrics`, and `map` |
| `Column` | `duet-engrave`, chunk A2, module `spacing` |
| `Note`, `NoteId`, `Spanner`, `SpannerId`, `SpannerKind`, `Lyric`, `LyricText`, `VerseNumber`, `Dynamic`, `Articulation`, `ScoreMark`, `MarkKind`, `RepeatSide`, `RehearsalText`, `Score` | `duet-score`, section 15.3 and section 3.3 |
| `Ticks` | `duet-time`, path `duet_time::Ticks` |

### Module `duet_engrave::beam`

Design contract 2.3 states the beam rules: the thickness is `beamThickness`, the gap between two
beam edges is `beamSpacing`, so the pitch between two beam top edges is 0.75 staff spaces; the group
follows the beat unit of the time signature; a rest breaks the group; the slant follows the pitch
difference between the first and the last note, capped at 2.0 staff spaces and quantized to 0.25
staff-space steps; a group whose inner note passes outside the first and the last note gets a flat
beam; a secondary beam that covers fewer notes than the primary beam paints as a partial beam of
1.0 staff space.

```rust
/// The cap on a beam slant, in staff spaces.
pub const BEAM_SLANT_CAP_SP: f32 = 2.0;

/// The quantization step of a beam slant, in staff spaces.
pub const BEAM_SLANT_STEP_SP: f32 = 0.25;

/// The length of a partial beam, in staff spaces.
pub const PARTIAL_BEAM_SP: f32 = 1.0;

/// Group the beamable notes of one system and give each group its slope.
///
/// The result is one entry per beam, from the primary beam down to the last
/// subdivision, in paint order.
///
/// # Errors
/// Returns `EngraveError::Metrics` when the font metadata lacks the stem
/// anchor a group needs.
#[expect(
    too_many_lines,
    reason = "one decision table over the seven beam rules of section 10.5, in the order that \
              section states; every arm is present and `wildcard_enum_match_arm` proves it"
)]
pub fn group_and_slope(
    score: &Score,
    columns: &[Column],
    map: &SystemXMap,
    options: &LayoutOptions,
    metrics: &FontMetrics,
) -> Result<Vec<QuadPlacement>, EngraveError>;
```

A beam is an axis-aligned rectangle only when its slant is zero, so a sloped beam is a
`PathPlacement`. Section 10.4 states the split: paths carry slurs, ties, hairpins, and beams. The
function therefore returns both lists:

```rust
/// The beams of one system, split by the section 10.4 primitive rule.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Beams { flat: Vec<QuadPlacement>, sloped: Vec<PathPlacement> }
```

`Beams` is a module type of `duet-engrave` that no other crate names, and `SystemPlacement` stays
the one value the paint element receives (section 10.5). `group_and_slope` returns
`Result<Beams, EngraveError>`.

The `#[expect(too_many_lines, ...)]` attribute is the Appendix B.1 row
`duet-engrave::beam::group_and_slope`, and the reason text above is that row's text, copied.

### Module `duet_engrave::spanner`

Design contract 2.3 states the tie rules, and rule 6 states that a tie is a `PathBuilder::fill()`
path of two cubic curves and never a stroke, because the thickness varies along the span.

```rust
/// The tie height at a short span, in staff spaces.
pub const TIE_HEIGHT_MIN_SP: f32 = 0.25;

/// The tie height at a span of `TIE_HEIGHT_FULL_SPAN_SP` or more.
pub const TIE_HEIGHT_MAX_SP: f32 = 0.5;

/// The span at which a tie reaches `TIE_HEIGHT_MAX_SP`, in staff spaces.
pub const TIE_HEIGHT_FULL_SPAN_SP: f32 = 12.0;

/// The clear space a tie leaves at each head, in staff spaces.
pub const TIE_INSET_SP: f32 = 0.2;

/// The overhang of a tie half at a system break, in staff spaces.
pub const TIE_BREAK_OVERHANG_SP: f32 = 1.0;

/// Place every tie, slur, and hairpin of one system as a filled path.
///
/// A spanner that crosses the system break splits into two halves, which
/// design contract 2.3 rule 5 states.
///
/// # Errors
/// Returns `EngraveError::Score` when a spanner names a note the score does
/// not hold.
pub fn place_spanners(
    score: &Score,
    system_range: (Ticks, Ticks),
    columns: &[Column],
    map: &SystemXMap,
    options: &LayoutOptions,
    metrics: &FontMetrics,
) -> Result<Vec<PathPlacement>, EngraveError>;
```

### Module `duet_engrave::lyric`

```rust
/// Place one row of lyric syllables per verse, under the staff.
///
/// The row is present only when `LayoutOptions::show_lyrics` is true, so a
/// caller that hides the lyrics pays no layout cost.
///
/// # Errors
/// Returns `EngraveError::Score` when a lyric names a note the score does not
/// hold.
pub fn place_lyrics(
    score: &Score,
    columns: &[Column],
    options: &LayoutOptions,
    metrics: &FontMetrics,
) -> Result<Vec<GlyphPlacement>, EngraveError>;
```

A lyric syllable is text and not a SMuFL glyph, so the placement carries the baseline and the
leading edge of each syllable and the element paints the text. `GlyphPlacement` carries a `Symbol`,
so the lyric row returns its own module type:

```rust
/// One lyric syllable, placed. It carries text, so it is not a
/// `GlyphPlacement`.
#[derive(Debug, Clone, PartialEq)]
pub struct LyricPlacement { note: NoteId, verse: VerseNumber, text: LyricText, x: f32, baseline: f32 }
```

`place_lyrics` returns `Result<Vec<LyricPlacement>, EngraveError>`. `SystemPlacement` gains no
field in this chunk, because section 10.5 declares its seven fields and SM0 forbids a shape this
document does not state. The lyric row therefore reaches the element through the `glyphs` list as
the syllable's leading glyph plus the text the score already carries; report the discrepancy to the
Architect if the element needs a separate list.

### Module `duet_engrave::mark`

```rust
/// Place every dynamic mark, articulation, rehearsal mark, and repeat
/// barline of one system.
///
/// # Errors
/// Returns `EngraveError::Metrics` when the font metadata lacks the glyph a
/// mark needs.
pub fn place_marks(
    score: &Score,
    system_range: (Ticks, Ticks),
    columns: &[Column],
    map: &SystemXMap,
    options: &LayoutOptions,
    metrics: &FontMetrics,
) -> Result<(Vec<GlyphPlacement>, Vec<QuadPlacement>), EngraveError>;
```

The glyph list carries the dynamics and the articulations. The quad list carries the repeat
barlines, because design contract 2.3 makes every axis-aligned rectangle a quad (section 10.4).

## Steps

1. Read the four files. Confirm that each one holds a `//!` line and nothing else. Report a
   discrepancy and stop if the state differs.
2. Write the failing test `beam_groups_follow_the_beat_unit` in `src/beam.rs`, inside a
   `#[cfg(test)] mod tests`. Run `cargo nextest run -p duet-engrave -E 'test(beam)' --no-tests=fail`
   and confirm that the run fails to compile.
3. Declare the three beam constants and the `Beams` type. Implement `group_and_slope` as one
   decision table over the five beam rules of design contract 2.3, in the order that section states.
   Match every `duet_score::Duration` arm, because `clippy::wildcard_enum_match_arm` is denied.
4. Run the same command and confirm that the test passes.
5. Write the remaining `beam` tests of the Tests section. Run the same command and confirm that
   every one passes.
6. Write the failing test `spanner_tie_is_a_filled_path` in `src/spanner.rs`. Run
   `cargo nextest run -p duet-engrave -E 'test(spanner)' --no-tests=fail` and confirm that it fails.
7. Declare the five tie constants. Implement `place_spanners`. Build each tie and each slur as two
   cubic curves with `PathPlacement::filled` set to true. Build a hairpin as two straight segments
   with `filled` set to false.
8. Run the same command and confirm that every `spanner` test passes.
9. Write the failing test `lyric_places_one_row_per_verse` in `src/lyric.rs`. Run
   `cargo nextest run -p duet-engrave -E 'test(lyric)' --no-tests=fail` and confirm that it fails.
10. Declare `LyricPlacement`. Implement `place_lyrics`. Return an empty list at once when
    `LayoutOptions::show_lyrics` is false.
11. Run the same command and confirm that every `lyric` test passes.
12. Write the failing test `mark_places_a_dynamic_under_the_staff` in `src/mark.rs`. Run
    `cargo nextest run -p duet-engrave -E 'test(mark)' --no-tests=fail` and confirm that it fails.
13. Implement `place_marks`. Match every `duet_score::Dynamic`, `duet_score::Articulation`, and
    `duet_score::MarkKind` arm.
14. Run `cargo nextest run -p duet-engrave --no-tests=fail` and confirm that every test passes.
15. Run `cargo clippy -p duet-engrave --all-targets --locked -- -D warnings` once and repair every
    finding.
16. Commit on the branch `chunk/a3-beams-spanners-lyrics-marks`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.

`crates/duet-engrave/src/beam.rs`

- `beam_groups_follow_the_beat_unit` — asserts that eight eighth notes in 4/4 give two groups of
  four, and that sixteen sixteenth notes give four groups of four.
- `beam_rest_breaks_the_group` — asserts that a rest in the middle of four eighth notes gives two
  groups of two.
- `beam_slant_is_capped_and_quantized` — asserts that a pitch difference of ten staff steps gives a
  slant of 2.0 staff spaces, and that every returned slant is a whole multiple of 0.25.
- `beam_inner_note_outside_gives_a_flat_beam` — asserts a slant of zero for a group whose middle
  note sits above the first and the last note.
- `beam_secondary_beam_paints_partial` — asserts that a group of one eighth and two sixteenths gives
  one primary beam over all three and one partial beam of 1.0 staff space.
- `beam_pitch_between_two_beams_is_three_quarters` — asserts that the distance between two beam top
  edges equals `beamThickness` plus `beamSpacing`, which design contract 2.3 states is 0.75 staff
  spaces for Bravura.

`crates/duet-engrave/src/spanner.rs`

- `spanner_tie_is_a_filled_path` — asserts that a tie returns one `PathPlacement` with `filled` true
  and eight control points, which is two cubic curves.
- `spanner_tie_height_grows_with_the_span` — asserts 0.25 staff spaces at a span of 2 staff spaces
  and 0.5 staff spaces at a span of 12 staff spaces.
- `spanner_tie_curves_away_from_the_stem` — asserts that a down-stem note gives a tie above the head
  and that an up-stem note gives a tie below it.
- `spanner_tie_splits_at_the_system_break` — asserts two halves, and that the first half ends 1.0
  staff space after the last head of the system.
- `spanner_refuses_a_missing_note` — asserts `EngraveError::Score`.

`crates/duet-engrave/src/lyric.rs`

- `lyric_places_one_row_per_verse` — asserts that two verses give two distinct baselines, and that
  the second baseline sits below the first.
- `lyric_aligns_each_syllable_with_its_note` — asserts that each syllable's x equals the head's x
  less half of the syllable's own width.
- `lyric_hidden_option_places_nothing` — asserts an empty list when `show_lyrics` is false.

`crates/duet-engrave/src/mark.rs`

- `mark_places_a_dynamic_under_the_staff` — asserts that a `Dynamic::F` gives one `GlyphPlacement`
  with `Symbol::DynamicForte` below the bottom staff line.
- `mark_places_an_articulation_on_the_head_side` — asserts that a staccato sits opposite the stem.
- `mark_repeat_barline_is_a_quad_pair` — asserts one thin quad and one thick quad for a
  `RepeatSide::Start`, with `barlineSeparation` between them.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-engrave -E 'test(beam)' --no-tests=fail
cargo nextest run -p duet-engrave --no-tests=fail
cargo clippy -p duet-engrave --all-targets --locked -- -D warnings
```

The first command fails before this chunk, because the crate holds no `beam` test. It passes after
this chunk. The second command reports every test of the crate as passed. The third command prints
no warning, and the one `#[expect]` site is the Appendix B.1 row for `group_and_slope`.

The work lands as one commit on the branch `chunk/a3-beams-spanners-lyrics-marks`, with a
conventional subject such as `feat(engrave): place beams, spanners, lyrics, and marks`.

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
