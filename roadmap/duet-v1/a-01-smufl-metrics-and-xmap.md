---
id: A1
line: A
depends_on: [M2, T2]
write_scope:
  - crates/duet-engrave/Cargo.toml
  - crates/duet-engrave/src/lib.rs
  - crates/duet-engrave/src/smufl.rs
  - crates/duet-engrave/src/metrics.rs
  - crates/duet-engrave/src/placement.rs
  - crates/duet-engrave/src/map.rs
  - crates/duet-engrave/src/spacing.rs
  - crates/duet-engrave/src/system.rs
  - crates/duet-engrave/src/beam.rs
  - crates/duet-engrave/src/spanner.rs
  - crates/duet-engrave/src/lyric.rs
  - crates/duet-engrave/src/mark.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-engrave -E 'test(xmap)' --no-tests=fail"
---

# A1: The SMuFL metadata reader, the scale model, the placement types, and `SystemXMap`

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk opens line A over the crate `duet-engrave`. Chunk M2 creates the crate
skeleton in phase 2, so `crates/duet-engrave/Cargo.toml` and `crates/duet-engrave/src/lib.rs` exist
before this chunk starts and this chunk modifies both. Every other file of the write scope does not
exist yet, and this chunk creates it. The chunk reads the Bravura SMuFL metadata, declares the
placement value types, and declares `SystemXMap` with its two directions. It implements
architecture sections 10.5 and 15.6, design contract sections 2.1, 2.2, and 2.3, and the product
stories C-05, C-08, and X-01.

Section 13.2 gives this chunk the second duty of SM2: it creates every module file of line A at
every depth as a stub, so chunks A2, A3, and A4 modify a stub and create no source file.

## Files

- `crates/duet-engrave/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses.
- `crates/duet-engrave/src/lib.rs` — modify. Add one `mod` line per module file below.
- `crates/duet-engrave/src/smufl.rs` — create. The Bravura metadata reader.
- `crates/duet-engrave/src/metrics.rs` — create. `FontMetrics`, `Symbol`, `SizePx`, `LayoutOptions`.
- `crates/duet-engrave/src/placement.rs` — create. `QuadPlacement`, `PathPlacement`,
  `GlyphPlacement`, `NoteHit`, `SystemPlacement`, `SystemId`, `EngraveError`, `engrave`.
- `crates/duet-engrave/src/map.rs` — create. `SystemXMap` and `Knot`.
- `crates/duet-engrave/src/spacing.rs` — create as a stub. Chunk A2 fills it.
- `crates/duet-engrave/src/system.rs` — create as a stub. Chunk A2 fills it.
- `crates/duet-engrave/src/beam.rs` — create as a stub. Chunk A3 fills it.
- `crates/duet-engrave/src/spanner.rs` — create as a stub. Chunk A3 fills it.
- `crates/duet-engrave/src/lyric.rs` — create as a stub. Chunk A3 fills it.
- `crates/duet-engrave/src/mark.rs` — create as a stub. Chunk A3 fills it.
- `Cargo.lock` — modify. SM5 rule 2 puts it in the write scope of every chunk that writes a member
  manifest.

A stub holds the `//!` module documentation and nothing else, so it compiles under the full lint
set (SM2).

## Types and signatures

The member manifest gains these entries. The root `[workspace.dependencies]` already pins `serde`,
`serde_json`, and `thiserror` from chunk M0, and it carries the entry of each internal crate whose
skeleton a manifest chunk created (SM1 rule 2). SM4 forbids this chunk to edit the root manifest, so
a missing entry is a discrepancy that this chunk reports under SM0.

```toml
# crates/duet-engrave/Cargo.toml, [dependencies]
duet-time = { workspace = true }
duet-score = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
thiserror = { workspace = true }
```

Section 1.2 gives `duet-engrave` exactly those three third-party crates. Section 1.3 gives it the
two internal edges `duet-engrave -> duet-time` and `duet-engrave -> duet-score`.

### From architecture section 15.6, module `duet_engrave::metrics`

```rust
/// A size in pixels at the scale the engrave step was given.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizePx { width: f32, height: f32 }

/// One SMuFL symbol the font supplies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Symbol { NoteheadBlack, NoteheadHalf, NoteheadWhole, Flag8Up, Flag8Down, ClefG, ClefF, ClefC, AccidentalSharp, AccidentalFlat, AccidentalNatural, RestQuarter, RestHalf, RestWhole, DynamicPiano, DynamicForte, ArticAccent, ArticStaccato, Fermata }

/// The SMuFL metadata this build read from the shipped font.
#[derive(Debug, Clone, PartialEq)]
pub struct FontMetrics { staff_space: f32, advances: BTreeMap<Symbol, f32>, anchors: BTreeMap<Symbol, (f32, f32)> }

/// What a caller asks the engraver for.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LayoutOptions { pixels_per_staff_space: f32, page_width: f32, systems_visible: NonZeroU16, show_lyrics: bool }
```

Section 15.6 states the rule for `LayoutOptions::pixels_per_staff_space`: the value is
`ZoomStep::staff_space` for the mode's own step, converted with
`duet_time::convert::finite_to_f32_saturating`. B137 is the map, so this field states no scale of
its own.

### From architecture section 15.6, module `duet_engrave::placement`

```rust
/// A stable identifier for one engraved system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SystemId(u64);

/// One axis-aligned rectangle to paint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct QuadPlacement { x: f32, y: f32, width: f32, height: f32 }

/// One path to paint, as its control points in pixels.
#[derive(Debug, Clone, PartialEq)]
pub struct PathPlacement { points: Vec<(f32, f32)>, width: f32, filled: bool }

/// One glyph to paint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GlyphPlacement { symbol: Symbol, x: f32, y: f32, scale: f32 }

/// One hit rectangle, so a click resolves to a note with no arithmetic.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteHit { note: NoteId, x: f32, y: f32, width: f32, height: f32 }

/// Every way the engraver refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EngraveError { Metrics(Symbol), Overflow { system: SystemId }, Score(ScoreError) }
```

`NoteId` and `ScoreError` come from the crate `duet-score`, paths `duet_score::NoteId` and
`duet_score::ScoreError` (section 1.5).

### From architecture section 10.5, module `duet_engrave::placement`

```rust
/// What the paint element receives. Every position is absolute within the
/// system, in pixels, at the scale the engrave step was given.
#[derive(Debug, Clone)]
pub struct SystemPlacement {
    id: SystemId,
    tick_range: (Ticks, Ticks),
    size: SizePx,
    /// Axis-aligned rectangles.
    quads: Vec<QuadPlacement>,
    /// Slurs, ties, hairpins, and beams.
    paths: Vec<PathPlacement>,
    /// SMuFL symbols.
    glyphs: Vec<GlyphPlacement>,
    /// One rectangle per note, for hit testing.
    hits: Vec<NoteHit>,
    /// The monotone map from ticks to x for this system.
    map: SystemXMap,
}

/// Lay out a score.
///
/// # Errors
/// Returns `EngraveError::Metrics` when the font metadata lacks a glyph, and
/// `EngraveError::Overflow` when a system cannot fit the page width.
pub fn engrave(score: &Score, map: &TempoMap, options: &LayoutOptions, metrics: &FontMetrics)
    -> Result<Vec<SystemPlacement>, EngraveError>;
```

`Ticks`, `TempoMap`, and `Score` come from `duet_time::Ticks`, `duet_time::TempoMap`, and
`duet_score::Score` (section 1.5).

### From architecture section 10.5, module `duet_engrave::map`

```rust
/// A monotone, strictly increasing, piecewise-linear map from a beat
/// position to an x offset inside one system.
#[derive(Debug, Clone, PartialEq)]
pub struct SystemXMap { knots: Vec<Knot> }

/// One knot. `x` is in pixels at the scale the engrave step was given.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knot { at: Ticks, x: f32 }

impl SystemXMap {
    /// The x offset of a beat position, by linear interpolation.
    pub fn x_for_beat(&self, at: Ticks) -> f32;

    /// The beat position at an x offset. The inverse exists because the map
    /// is strictly increasing.
    pub fn beat_for_x(&self, x: f32) -> Ticks;
}
```

### Module `duet_engrave::smufl`

Section 1.5 places no type in this module, so the module holds functions alone. Design contract 2.2
names the three tables the reader keeps: `engravingDefaults`, `glyphAdvanceWidths`, and
`glyphsWithAnchors`. Appendix B.3 states that the `smufl` crate is not a dependency and that this
chunk writes the reader.

```rust
/// Read a Bravura SMuFL metadata document into the metrics this build uses.
///
/// # Errors
/// Returns `EngraveError::Metrics` for a symbol the document does not carry.
pub fn read_metadata(bytes: &[u8], staff_space: f32) -> Result<FontMetrics, EngraveError>;
```

## Steps

1. Read `crates/duet-engrave/Cargo.toml` and `crates/duet-engrave/src/lib.rs`. Confirm that chunk M2
   created both, that the manifest carries `[lints] workspace = true`, a `description`, and no
   `[dependencies]` section, and that `lib.rs` carries the `//!` crate documentation and
   `#![forbid(unsafe_code)]`. Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` carries `serde`,
   `serde_json`, `thiserror`, `duet-time`, and `duet-score`. SM4 forbids this chunk to edit that
   file, so report a discrepancy and stop if an entry is absent.
3. Create the ten module files as stubs. Each one holds one `//!` line and nothing else. Example for
   `beam.rs`:
   ```rust
   //! Beam grouping and beam slope. Chunk A3 fills this module.
   ```
4. Add one `mod` line per module file to `src/lib.rs`, in alphabetical order, each with its own
   `///` documentation line. Run `cargo check -p duet-engrave`. Expect a clean build.
5. Add the five `{ workspace = true }` entries to `crates/duet-engrave/Cargo.toml`. Run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3). Expect a clean build and a
   changed lock file.
6. Write the failing test `xmap_x_for_beat_interpolates_between_knots` in `src/map.rs`, inside a
   `#[cfg(test)] mod tests`. Run
   `cargo nextest run -p duet-engrave -E 'test(xmap)' --no-tests=fail` and confirm that the run
   fails to compile, because `SystemXMap` does not exist yet.
7. Declare `Knot` and `SystemXMap` in `src/map.rs` with the shapes above, plus a checked
   constructor. The constructor refuses a knot list that is not strictly increasing in `at` and in
   `x`, because the inverse needs that property:
   ```rust
   /// Build a map from its knots.
   ///
   /// # Errors
   /// Returns `EngraveError::Overflow` when the knot list is not strictly
   /// increasing in `at` and in `x`, because `beat_for_x` is the inverse and
   /// only a strictly increasing map has one.
   pub fn new(system: SystemId, knots: Vec<Knot>) -> Result<Self, EngraveError>;
   ```
8. Implement `x_for_beat` and `beat_for_x` by linear interpolation between the two knots that
   bracket the argument. Use `slice::binary_search_by` and `slice::get` for every lookup, because
   `clippy::indexing_slicing` is denied. Clamp an argument below the first knot to the first knot,
   and an argument above the last knot to the last knot.
9. Run `cargo nextest run -p duet-engrave -E 'test(xmap)' --no-tests=fail` and confirm that it
   passes.
10. Write the remaining `xmap` tests of the Tests section. Run the same command and confirm that
    every one passes.
11. Declare `Symbol`, `SizePx`, `FontMetrics`, and `LayoutOptions` in `src/metrics.rs`. Add an
    accessor per private field, each one `const` where `clippy::missing_const_for_fn` asks for it
    and each one `#[must_use]`.
12. Write the failing test `metrics_reader_keeps_every_engraving_default` in `src/smufl.rs`. Run
    `cargo nextest run -p duet-engrave -E 'test(metrics_reader)' --no-tests=fail` and confirm that
    it fails.
13. Implement `read_metadata` in `src/smufl.rs` over `serde_json`. Parse `engravingDefaults`,
    `glyphAdvanceWidths`, and `glyphsWithAnchors`. Map each SMuFL glyph name of design contract 2.2
    to its `Symbol` arm with an exhaustive `match`, because `clippy::wildcard_enum_match_arm` is
    denied. Return `EngraveError::Metrics(symbol)` for a symbol the document does not carry.
14. Run the same command and confirm that it passes.
15. Declare `SystemId`, `QuadPlacement`, `PathPlacement`, `GlyphPlacement`, `NoteHit`,
    `SystemPlacement`, and `EngraveError` in `src/placement.rs`.
16. Implement `engrave` in `src/placement.rs` for one system with no line break: it builds the staff
    lines as quads, the clef, key signature, and time signature as glyphs, one glyph per note head,
    one `NoteHit` per note, and one `SystemXMap` over the note onsets. Chunk A2 adds the horizontal
    spacing rule and the system break, and chunk A3 adds beams, spanners, lyrics, and marks.
17. Run `cargo nextest run -p duet-engrave --no-tests=fail` and confirm that every test passes.
18. Run `cargo clippy -p duet-engrave --all-targets --locked -- -D warnings` once, as the one
    targeted diagnostic the Constraints allow, and repair every finding.
19. Commit on the branch `chunk/a1-smufl-metrics-and-xmap`. The native git hook runs
    `scripts/dod.sh`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.

`crates/duet-engrave/src/map.rs`

- `xmap_x_for_beat_interpolates_between_knots` — builds a map with three knots and asserts that a
  beat position between two knots answers the linear interpolation of the two x values.
- `xmap_beat_for_x_inverts_x_for_beat` — asserts that `beat_for_x(x_for_beat(at))` equals `at` for
  every knot position and for three positions between knots.
- `xmap_clamps_outside_the_knot_range` — asserts that a beat before the first knot answers the first
  knot's x, and that a beat after the last knot answers the last knot's x.
- `xmap_new_refuses_a_non_monotone_knot_list` — asserts that `SystemXMap::new` returns
  `EngraveError::Overflow` for a knot list whose `x` values fall.
- `xmap_new_refuses_a_repeated_position` — asserts the same refusal for two knots at one `at`,
  because a repeated position makes the inverse ambiguous.

`crates/duet-engrave/src/smufl.rs`

- `metrics_reader_keeps_every_engraving_default` — reads a fixture document that holds the fourteen
  `engravingDefaults` keys of design contract 2.2 and asserts each value.
- `metrics_reader_refuses_a_missing_glyph` — asserts that a document with no `noteheadBlack` advance
  returns `EngraveError::Metrics(Symbol::NoteheadBlack)`.
- `metrics_reader_reads_the_stem_anchors` — asserts that `stemUpSE` and `stemDownNW` reach
  `FontMetrics::anchors` for `Symbol::NoteheadBlack`.

`crates/duet-engrave/src/placement.rs`

- `placement_engraves_five_staff_lines_per_staff` — asserts that a one-staff score produces five
  quads whose heights equal `staffLineThickness` times the staff space, and that the line centres
  sit one staff space apart (design contract 2.3).
- `placement_reports_one_hit_per_note` — asserts that the `hits` length equals the note count.
- `placement_orders_quads_then_paths_then_glyphs` — asserts the fixed per-system order of section
  10.4.

The fixture SMuFL document is a `const` string in the test module, so the test needs no file on
disk.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-engrave -E 'test(xmap)' --no-tests=fail
cargo nextest run -p duet-engrave --no-tests=fail
cargo clippy -p duet-engrave --all-targets --locked -- -D warnings
```

The first command fails before this chunk, because the crate holds no `xmap` test, and
`--no-tests=fail` makes an empty match a failure (SM3 rule 1). It passes after this chunk. The
second command reports every test of the crate as passed. The third command prints no warning.

The work lands as one commit on the branch `chunk/a1-smufl-metrics-and-xmap`, with a conventional
subject such as `feat(engrave): read SMuFL metrics and map beats to x offsets`.

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
