---
id: K2
line: K
depends_on: [K1, A1]
write_scope:
  - crates/duet/src/element/staff_system.rs
  - crates/duet/src/compose/view.rs
  - crates/duet/src/compose/caret.rs
  - crates/duet/src/compose/menu.rs
  - crates/duet/src/compose/duration.rs
  - crates/duet/src/shell/toolbar_compose.rs
parallelism: independent
completion: "cargo nextest run -p duet -E 'test(compose)' --no-tests=fail passes; cargo clippy -p duet --all-targets -- -D warnings is clean; commit SHA on a branch chunk/k2-compose-work-area"
---

# K2: The Compose work area, the staff element, and the duration selector

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk fills the Compose work area that chunk K1 created as a seam. It delivers
`StaffSystem`, `ComposeView` with its `VirtualList`, the cached sibling systems, the duration
selector, the note context menu, and the Compose `ModeToolbar`. It implements architecture sections
10.2, 10.3, 10.4, 10.5, 10.6, 10.7 and 15.16, design contract sections 2.1, 2.2, 2.4, 2.5, 2.6 and
2.7, and ADR `adr/0006-wrapped-timeline.md`. It carries product stories C-05, C-06, C-08, C-09,
C-10, C-11, C-12, C-13, C-14, C-15, C-16, C-17, C-18, C-19, C-20 and X-03.

**C-14 belongs to K2, and to K2 alone** (critic C16-W8). The caret, the keyboard note entry, and the
note drag all live in `src/compose/{view,caret}.rs`, which is this chunk's write scope.

This chunk modifies six files and creates none. SM2 gave every file to chunk K1.

## Files

- `crates/duet/src/element/staff_system.rs` — modify. Chunk K1 left a documentation stub.
- `crates/duet/src/compose/view.rs` — modify. Chunk K1 left a seam view.
- `crates/duet/src/compose/caret.rs` — modify. Documentation stub.
- `crates/duet/src/compose/menu.rs` — modify. Documentation stub.
- `crates/duet/src/compose/duration.rs` — modify. Documentation stub.
- `crates/duet/src/shell/toolbar_compose.rs` — modify. Chunk K1 left the whole `impl ModeToolbar`
  block with an empty body. This chunk replaces one `render` body and one `group_count` body, and it
  edits `top_bar.rs` NOT at all.

This chunk writes no member manifest and no lock file.

## Types and signatures

### Declared by this chunk, in `crates/duet/src/compose/view.rs`

Copied from architecture section 15.16.

```rust
/// The Compose work area.
/// `VirtualListScrollHandle` carries no `Debug`, and
/// `missing_debug_implementations` is denied, so this view writes the impl by
/// hand, exactly as `PoolHandle` does (section 5.5).
pub(crate) struct ComposeView {
    systems: Vec<Arc<SystemPlacement>>,
    children: Vec<Entity<SystemView>>,
    playhead: Entity<PlayheadLayer>,
    scroll: VirtualListScrollHandle,
    sizes: Vec<SizePx>,
    selection: Selection,
    caret: Position,
    duration: DurationSelector,
    engraved: Revision,
    engrave: Option<Task<()>>,
    state: WorkAreaState,
    focus: FocusHandle,
    subscriptions: Vec<Subscription>,
}
impl core::fmt::Debug for ComposeView {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("ComposeView").finish_non_exhaustive()
    }
}

/// One engraved system inside the Compose work area.
#[derive(Debug)]
pub(crate) struct SystemView {
    system: SystemId,
    placement: Arc<SystemPlacement>,
    hits: Arc<[NoteHit]>,
    focus: FocusHandle,
}
```

### Declared by this chunk, in `crates/duet/src/element/staff_system.rs`

Copied from architecture section 15.16.

```rust
/// Paints one system: lines, glyphs, stems, beams, and ties.
#[derive(Debug, IntoElement)]
pub(crate) struct StaffSystem { placement: Arc<SystemPlacement>, system: SystemId }
```

### Declared by this chunk, in `crates/duet/src/compose/duration.rs`

Copied from architecture section 10.6.

```rust
/// Which duration letters are held now, in the ORDER they went down, over
/// the six letters of PR 11 Q1.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct HeldDuration(ArrayVec<NoteValue, 6>);

/// The duration the next click inserts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct DurationSelector {
    /// Set by a number key `1` to `6`, and by the top bar toggle group.
    sticky: NoteValue,
    /// Which duration letters are held now.
    held: HeldDuration,
}

impl DurationSelector {
    /// The duration a click inserts now. A held letter wins over the sticky
    /// value, and **the letter that went down LAST wins** over one held
    /// before it, which is the product criterion (PR 11 Q1, critic C19-W17).
    pub(crate) fn pending(&self) -> NoteValue;

    /// Whether a letter is held, so the view can show the override state.
    pub(crate) fn is_overridden(&self) -> bool;

    /// Clear every held letter. Focus loss and window deactivation call it.
    pub(crate) fn release_all(&mut self);
}
```

**It is a stack and not a bitset.** A key-down pushes, a key-up removes that one letter wherever it
sits, and `release_all` empties the stack. Six letters exist, so the inline capacity is six and the
type allocates nothing.

### Consumed from `duet-engrave`, which chunk A1 wrote

Copied from architecture section 10.5.

```rust
/// What the paint element receives. Every position is absolute within the
/// system, in pixels, at the scale the engrave step was given.
#[derive(Debug, Clone)]
pub struct SystemPlacement {
    id: SystemId,
    tick_range: (Ticks, Ticks),
    size: SizePx,
    quads: Vec<QuadPlacement>,
    paths: Vec<PathPlacement>,
    glyphs: Vec<GlyphPlacement>,
    hits: Vec<NoteHit>,
    map: SystemXMap,
}

/// Lay out a score.
///
/// # Errors
/// Returns `EngraveError::Metrics` when the font metadata lacks a glyph, and
/// `EngraveError::Overflow` when a system cannot fit the page width.
pub fn engrave(score: &Score, map: &TempoMap, options: &LayoutOptions, metrics: &FontMetrics)
    -> Result<Vec<SystemPlacement>, EngraveError>;

impl SystemXMap {
    /// The x offset of a beat position, by linear interpolation.
    pub fn x_for_beat(&self, at: Ticks) -> f32;

    /// The beat position at an x offset. The inverse exists because the map
    /// is strictly increasing.
    pub fn beat_for_x(&self, x: f32) -> Ticks;
}
```

### The Compose top bar, copied from design contract 2.7

Nine groups, in this order. The overflow removes groups from the trailing end in the order 8, 7, 6,
5, 4.

| Group | Contract 2.7 content |
|---|---|
| 1 Duration | `ToggleGroup` of six buttons, Bravura glyphs, shortcuts `1` to `6`. A held `w`, `h` or `e` overrides the group for one click and flashes the matching button for the hold. |
| 2 Modifier | `ButtonGroup` of four: `Dot`, `Double dot`, `Tie`, `Tuplet…`. |
| 3 Accidental | `ToggleGroup` of five, as Bravura glyphs. |
| 4 Articulation | `ButtonGroup` of five: staccato, tenuto, accent, marcato, fermata. |
| 5 Dynamic | One `DropdownMenu` button; the menu lists `ppp` to `fff` then the two hairpins. |
| 6 Insert | One `DropdownMenu` button: `Measure`, `System break`, `Key signature…`, `Time signature…`, `Clef…`, `Repeat`, `Rehearsal mark`. |
| 7 Lyrics | One toggle `Button`. |
| 8 Grid | A `Select`: `1/1`, `1/2`, `1/4`, `1/8`, `1/16`, `Off`. |
| 9 View | Zoom out `Button`, a `Select` with the six zoom steps, zoom in `Button`, then a `ToggleGroup` of `Page` and `Scroll`. |

### The zoom map, copied from architecture section 10.2

`LayoutOptions::pixels_per_staff_space` takes its value from `ZoomStep::staff_space`, which is the
one map from a zoom step to a staff size (B137). This chunk invents no zoom mapping of its own.

## Steps

1. Read every file of the write scope. Confirm that chunk K1 created each one, that
   `src/compose/view.rs` holds the seam view, and that `src/shell/toolbar_compose.rs` holds the whole
   `impl ModeToolbar` block with an empty body. Stop and report a discrepancy.
2. Write the failing test `compose_hit_test_resolves_a_click_to_a_note` in
   `src/element/staff_system.rs`, as a `#[gpui_kit::test]` with the signature
   `fn compose_hit_test_resolves_a_click_to_a_note(cx: &mut TestAppContext)` and the first statement
   `cx.update(gpui_kit::init);`.
3. Run `cargo nextest run -p duet -E 'test(compose)' --no-tests=fail` and confirm that it fails.
4. Write `StaffSystem` in `src/element/staff_system.rs`. It adds only the element bounds origin to
   each placement and performs no other arithmetic. Paint order per system is quads, then paths,
   then glyphs, which section 10.4 states and which keeps the batch from splitting.
5. Paint every axis-aligned rectangle as a quad: staff lines, ledger lines, bar lines and stems.
   Paint slurs, ties, hairpins and beams as paths. Paint note heads, clefs, accidentals, rests and
   dynamic marks as glyphs with `Window::paint_glyph`.
6. Read every colour from `cx.global::<DuetTokens>()`. No view holds an `Hsla` literal, which is
   contract 7.1 rule 3.
7. Write `ComposeView` in `src/compose/view.rs`. Replace the seam struct with the declaration above.
   Own one `gpui_kit::component::VirtualList`: the item is one system, the item count is the length
   of the `Vec<SystemPlacement>`, each item reports its own width and height from
   `SystemPlacement::size`, and the identity is `SystemId` and never the list index.
8. Build each system child as an `Entity::cached` child, so a `PlayheadLayer` frame does not
   re-render every `SystemView` (section 10.2).
9. Run the engrave step on `cx.background_spawn`, keyed by `Score::revision()`. The result lands
   through `cx.update`, and a result whose revision no longer matches is dropped. Hold the `Task` in
   `ComposeView::engrave`, so a dropped view cancels the work.
10. Write `src/compose/duration.rs` with `HeldDuration` and `DurationSelector`. `ComposeView` holds
    a `FocusHandle`, tracks `on_key_down` and `on_key_up`, and calls `release_all` on `on_focus_out`
    and on a window deactivation.
11. Write `src/compose/caret.rs` with the caret, the keyboard note entry of contract 9.3, and the
    note drag of product story C-14. The keyboard note cursor paints as a 2 px vertical bar of 2 sp
    height in `duet.note.cursor`, and the bar does not blink.
12. Write `src/compose/menu.rs` with the note context menu of contract 2.6, in its six blocks with a
    `Separator` between each block. An item that does not apply is disabled, not hidden, and its
    tooltip states the reason. Write the MIDI import preview of product story C-12 in the same file:
    one row per source track with a part target per row, an accept action that dispatches ONE verb
    so the write is one undo step, and a progress surface that reads `CoreEvent::JobProgress`.
13. Write `src/shell/toolbar_compose.rs`. Replace `group_count` with nine and replace `render` with
    the nine groups of contract 2.7, in the fixed removal order 8, 7, 6, 5, 4. Edit `top_bar.rs` not
    at all.
14. Write every test of the Tests section.
15. Run the Completion command and confirm that it passes.
16. Run `cargo clippy -p duet --all-targets -- -D warnings` and confirm that it is clean.
17. Commit on a branch named `chunk/k2-compose-work-area`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. A user-interface test is a
`#[gpui_kit::test]` with the signature `fn name(cx: &mut TestAppContext)` and the first statement
`cx.update(gpui_kit::init);`. Every assert carries a message.

### Rung-two element test, for `StaffSystem`

| Test | File | What it asserts |
|---|---|---|
| `compose_staff_system_requests_its_layout_size` | `src/element/staff_system.rs` | The element requests the width and the height `SystemPlacement::size` states. |
| `compose_staff_system_paints_the_expected_quads` | `src/element/staff_system.rs` | `Window::painted_quads()` holds one quad per staff line, ledger line, bar line and stem of a fixed `SystemPlacement`, and no quad for a slur or a glyph. |
| `compose_hit_test_resolves_a_click_to_a_note` | `src/element/staff_system.rs` | A click at a point resolves to the correct `NoteId`. |

### Rung-three user-interface behaviours this chunk owns

Architecture section 10.7 rung three items 1, 2, 3, 4 and 5.

| Test | Rung-three item | File | What it asserts |
|---|---|---|---|
| `compose_held_key_state_machine_covers_six_letters` | 1 | `src/compose/duration.rs` | Each of `w`, `h`, `q`, `e`, `s` and `t` sets the pending duration while it is down, and the letter that went down last wins over one held before it. |
| `compose_lost_key_up_returns_to_the_quarter_note` | 2 | `src/compose/duration.rs` | The window deactivates while `w` is held, `release_all` runs, and the pending duration returns to the quarter note. |
| `compose_hit_test_maps_x_to_the_correct_beat` | 3 | `src/compose/view.rs` | A click resolves to the correct pitch and the correct time through `SystemXMap::beat_for_x`. |
| `compose_virtual_list_renders_the_systems_at_a_scroll_offset` | 4 | `src/compose/view.rs` | At a given scroll offset the list renders exactly the systems that the offset and the item sizes select, and no other. |
| `compose_element_identity_comes_from_the_domain` | 5 | `src/compose/view.rs` | Every `ElementId` of a note comes from `NoteId` and every system id comes from `SystemId`, never a list index. An insert at the front leaves every other identity unchanged. |

### The remaining tests of this chunk

| Test | File | What it asserts |
|---|---|---|
| `compose_view_drops_a_stale_engrave_result` | `src/compose/view.rs` | A result whose `Revision` is behind the current one is dropped and never painted (section 10.5). |
| `compose_view_caches_every_sibling_system` | `src/compose/view.rs` | Each system child is built with `Entity::cached`, and a frame that notifies the playhead re-renders no `SystemView`. |
| `compose_view_renders_the_work_area_state_above_its_content` | `src/compose/view.rs` | Each `WorkAreaState` arm other than `Ready` renders the contract section 8.1 surface instead of the score. |
| `compose_caret_moves_by_one_diatonic_step` | `src/compose/caret.rs` | The Up and Down keys move the note cursor one diatonic step, and alt-Up and alt-Down move one semitone (contract 9.3). |
| `compose_note_drag_returns_on_a_release_outside_the_staff` | `src/compose/caret.rs` | A drag released outside the staff returns the note to its first pitch (PR C-14). |
| `compose_menu_disables_an_item_that_does_not_apply` | `src/compose/menu.rs` | `Tie to next` on the last note is disabled, not hidden, and its tooltip reads the reason (contract 2.6). |
| `compose_menu_import_accepts_in_one_undo_step` | `src/compose/menu.rs` | The import preview accept action dispatches ONE verb, so the whole import is one undo step (PR C-12). |
| `compose_toolbar_answers_nine_groups` | `src/shell/toolbar_compose.rs` | `group_count` answers nine and `render` returns the nine groups of contract 2.7 in the stated order. |
| `compose_toolbar_removes_from_the_trailing_end` | `src/shell/toolbar_compose.rs` | At a `visible` count of four the overflow holds groups 8, 7, 6 and 5, in that removal order. |
| `compose_toolbar_shows_the_value_at_the_insert_point` | `src/shell/toolbar_compose.rs` | Each control shows the value that applies at the insert point (PR C-15). |

**Path output and glyph output are not assertable.** The audit records that no `painted_paths()` and
no `painted_glyphs()` exist. Every layout assertion therefore lives in `duet-engrave`, which chunk
A1 and chunk A4 own, and this chunk asserts the plumbing alone (section 10.7 rung two).

## Verification

1. `cargo nextest run -p duet -E 'test(compose)' --no-tests=fail` passes and prints no
   `no tests to run` line.
2. `cargo nextest run -p duet --no-tests=fail` passes, so chunk K1's tests still pass.
3. `cargo clippy -p duet --all-targets -- -D warnings` prints no warning.
4. `git status` shows no change under `crates/duet/src/shell/top_bar.rs`.
5. One commit on a branch named `chunk/k2-compose-work-area`. The native git hook runs
   `scripts/dod.sh`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
