---
id: K3
line: K
depends_on: [K2, A1, C1, C3, D3, E2, I3]
write_scope:
  - crates/bc_app/duet/lang_rust/src/element/waveform_lane.rs
  - crates/bc_app/duet/lang_rust/src/element/playhead_layer.rs
  - crates/bc_app/duet/lang_rust/src/element/punch_range.rs
  - crates/bc_app/duet/lang_rust/src/record/view.rs
  - crates/bc_app/duet/lang_rust/src/record/lane.rs
  - crates/bc_app/duet/lang_rust/src/record/cache.rs
  - crates/bc_app/duet/lang_rust/src/record/controls.rs
  - crates/bc_app/duet/lang_rust/src/shell/toolbar_record.rs
parallelism: independent
completion: "cargo nextest run -p duet -E 'test(record_view) + test(path_cache)' --no-tests=fail passes; cargo clippy -p duet --all-targets -- -D warnings is clean; commit SHA on a branch chunk/k3-record-work-area"
---

# K3: The Record work area, the waveform lane, the playhead, and the path cache

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk fills the Record work area that chunk K1 created as a seam. It delivers
`WaveformLane`, `PlayheadLayer` with the start and stop conditions of architecture section 10.2,
`PunchRange`, `RecordView` with its `VirtualList`, the cached sibling lanes, `PathCache`,
`TakeSegment`, the pitch overlay, the input and monitor controls, the track filter, and the Record
`ModeToolbar`. It implements architecture sections 10.2, 10.3, 10.4, 10.5, 10.7 and 15.16, design
contract sections 3.1 to 3.9 and 8.2, and ADR `adr/0006-wrapped-timeline.md`. It carries product
stories R-03, R-04, R-07, R-08, R-09, R-10, R-11, R-12, R-15, X-01, X-02 and X-04.

This chunk modifies eight files and creates none. SM2 gave every file to chunk K1.

## Files

- `crates/bc_app/duet/lang_rust/src/element/waveform_lane.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/element/playhead_layer.rs` — modify. Chunk K1 left a driver seam.
- `crates/bc_app/duet/lang_rust/src/element/punch_range.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/record/view.rs` — modify. Chunk K1 left a seam view.
- `crates/bc_app/duet/lang_rust/src/record/lane.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/record/cache.rs` — modify. Documentation stub. `PathCache` is this chunk's file,
  and chunk K5 modifies no file of this chunk (section 10.5).
- `crates/bc_app/duet/lang_rust/src/record/controls.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/shell/toolbar_record.rs` — modify. Replace one `render` body and one
  `group_count` body. Edit `top_bar.rs` not at all.

This chunk writes no member manifest and no lock file.

## Types and signatures

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/record/view.rs` and `src/record/lane.rs`

Copied from architecture section 15.16.

```rust
/// The Record work area.
/// It writes `Debug` by hand for the reason `ComposeView` states.
pub(crate) struct RecordView {
    systems: Vec<Arc<SystemPlacement>>,
    lanes: Vec<Entity<LaneView>>,
    playhead: Entity<PlayheadLayer>,
    scroll: VirtualListScrollHandle,
    sizes: Vec<SizePx>,
    cache: PathCache,
    build: Option<Task<()>>,
    selected: Option<TrackId>,
    lane_heights: BTreeMap<TrackId, LogicalPx>,
    shown: TrackFilter,
    comp: bool,
    state: WorkAreaState,
    focus: FocusHandle,
    subscriptions: Vec<Subscription>,
}
impl core::fmt::Debug for RecordView {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("RecordView").finish_non_exhaustive()
    }
}

/// One track's audio lane inside the Record work area.
///
/// It opens no file. Every byte it paints arrives through `duet-core`
/// (section 1.3 rule 6).
#[derive(Debug)]
pub(crate) struct LaneView {
    track: TrackId,
    segments: Arc<[TakeSegment]>,
    pyramid: Option<Arc<Pyramid>>,
    punch: Option<Span>,
    height: LogicalPx,
    focus: FocusHandle,
}
```

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/record/cache.rs`

Copied from architecture section 10.5.

```rust
/// One painted piece of one region inside one system. Derived data.
#[derive(Debug, Clone)]
pub(crate) struct TakeSegment {
    region: RegionId,
    system: SystemId,
    /// The beat span this segment covers, clipped to the system.
    beats: (Ticks, Ticks),
    /// The map that turns a beat position into an x offset.
    map: Arc<SystemXMap>,
}

/// The tessellated path cache. One owner per mode, bounded by B61.
#[derive(Debug)]
pub(crate) struct PathCache {
    paths: BTreeMap<PathKey, Arc<Path<Pixels>>>,
    order: VecDeque<PathKey>,
    bytes: u64,
}

/// One path-cache key. Section 10.5 gives the two timeline shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum PathKey {
    /// The wrapped shape of Record.
    Wrapped { source: SourceHash, region: RegionId, system: SystemId, zoom: ZoomStep },
    /// The linear shape of Mix and Master.
    Linear { source: SourceHash, region: RegionId, zoom: ZoomScalar },
}
```

**The cache holds the value `PathBuilder::build()` returns**, which is a tessellated
`Path<Pixels>`, and never a `PathPlacement` point list. A `PathPlacement` is the cache INPUT.
`order` is the least-recently-used queue: the front is the next key the cache evicts when either
half of B61 would be passed.

**A miss paints nothing rather than tessellates on the frame path.** A `LaneView` that finds no
entry draws the lane background and requests the build, and the next frame paints the wave.

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/element/`

Copied from architecture section 15.16.

```rust
/// Paints one audio lane from the cached peak paths.
#[derive(Debug, IntoElement)]
pub(crate) struct WaveformLane { segments: Arc<[TakeSegment]>, track: TrackId }

/// The punch band and its two drag handles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoElement)]
pub(crate) struct PunchRange { span: Option<Span>, track: TrackId }

/// The one frame driver of Compose and Record. It paints the playhead only.
#[derive(Debug)]
pub(crate) struct PlayheadLayer {
    core: Entity<CoreHost>,
    /// The value the last frame painted.
    shown: TransportView,
}
```

**The START condition is `FrameDemand::transport_moves`, and the STOP condition is its negation with
the slot at rest.** The driver calls `Window::request_animation_frame` when the demand is set or
when the slot differs from `shown`. The loop restarts because a transport verb produces a
`CoreEvent`, the drain task notifies, and this view renders again.

`PlayheadLayer` reads the transport through `CoreHost::transport`, which returns a copy of the slot
the frame order already filled. **No `duet-engine` name, no `triple_buffer` name, and no publication
read end reaches this crate.**

### Consumed from other crates

| Type | Crate | Declaring section |
|---|---|---|
| `SystemPlacement`, `SystemXMap`, `NoteHit` | `duet-engrave` | 10.5 |
| `Pyramid`, `PyramidHeader`, `PeakBin` | `duet-dsp` | 15.2 |
| `TransportView`, `TrackFilter`, `LogicalPx`, `ZoomStep`, `ZoomScalar`, `Span` | `duet-command` | 10.2, 15.5 |
| `TrackId`, `RegionId`, `TakeId`, `SourceHash`, `MonitorMode`, `InputSelection` | `duet-session` | 15.4 |
| `Verb::TrackSetInput`, `Verb::TrackSetMonitor`, `Verb::TakeSplit`, `Verb::TakeTrim`, `SessionCommand::RegionSetStart`, `SessionCommand::TakeSetMuted` | `duet-command` | 9.1, 15.4 |
| `CoreHost`, `FrameDemand`, `WorkAreaState`, `DuetTokens` | this crate, chunk K1 | 15.16 |

**`crates/duet` calls no `duet-engine` function.** The input and monitor controls send
`Verb::TrackSetInput` and `Verb::TrackSetMonitor` through `duet-core`, and the engine answers them
from the device selection chunk C1 wrote and the monitor path chunk C3 wrote (section 1.3 rule 6).

### The lane geometry, copied from design contract 3.2, 3.3 and 3.7

| Item | Contract value |
|---|---|
| Lane height | compact 48 px, default 72 px, expanded 120 px |
| Gap between lanes in one block | 4 px |
| Lane header width | 140 px |
| Waveform fill | `duet.wave.fill`, the part colour at 70 percent alpha in light and 62 in dark |
| Mean-square core | a narrower bar at half the peak height, at 100 percent alpha, in `duet.wave.rms` |
| Zero line | 1 device pixel at `duet.wave.zero` across the full lane width |
| A column with no peaks yet | `duet.wave.absent` with the shimmer of contract section 8 |
| Record head | a 2 px line in `duet.record.red` with a filled triangle 12 px wide and 8 px tall. It is part of `PlayheadLayer`, not `WaveformLane` |
| Punch band | fill `duet.record.tint`, edges 2 px at the record red at 60 percent alpha, two handles 8 px wide with a 24 px hit width |

**The x map is shared, not copied.** A lane converts a sample position to a beat through the tempo
map, then asks the same `SystemXMap` for x. A lane never computes its own samples-per-pixel value.

## Steps

1. Read every file of the write scope. Confirm that chunk K1 created each one and that
   `playhead_layer.rs` holds the driver seam with no frame request. Stop and report a discrepancy.
2. Write the failing test `path_cache_evicts_the_least_recently_used_key` in
   `src/record/cache.rs`, inside a `#[cfg(test)] mod tests`.
3. Run `cargo nextest run -p duet -E 'test(path_cache)' --no-tests=fail` and confirm that it fails.
4. Write `TakeSegment`, `PathKey` and `PathCache` in `src/record/cache.rs`. Bound the cache by both
   halves of B61 and evict from the front of `order`.
5. Write the path build task. It runs on a GPUI background task, turns one `PathPlacement` into one
   `Path<Pixels>` with `PathBuilder::build()`, and hands the result back through `cx.update`. Hold
   the `Task` in `RecordView::build`, so a dropped view cancels the work. Nothing tessellates inside
   `render`.
6. Write `WaveformLane` in `src/element/waveform_lane.rs`. Paint one vertical bar per device pixel
   column from the min to the max value, with the zero line at the lane centre. Paint the
   mean-square core over it. A miss paints the lane background and requests the build.
7. Write `PlayheadLayer` in `src/element/playhead_layer.rs`. Read `CoreHost::transport` and
   `CoreHost::frame_demand`. Request an animation frame when `transport_moves` is set or when the
   slot differs from `shown`. Paint the playhead line and the record head only.
8. Write `PunchRange` in `src/element/punch_range.rs` to contract 3.6. When the two handles come
   closer than 48 px, split the shared middle evenly. When punch is off, paint the band at 40 percent
   of its alpha, hide the handles, and keep the range value.
9. Write `RecordView` in `src/record/view.rs`. Own one `VirtualList` with the same four facts as
   Compose: the item is one system, the count is the placement count, the size is
   `SystemPlacement::size`, and the identity is `SystemId`. Build each `LaneView` as an
   `Entity::cached` child.
10. Write `LaneView` in `src/record/lane.rs`. It opens no file; every byte arrives through
    `duet-core` as a `Pyramid` that `CoreEvent::PeaksReady` announced. Paint the take stack of
    contract 3.4: the active take fills the lane and other takes stack below as 10 px strips.
11. Write the pitch overlay of product story R-15 in `src/record/lane.rs`. Paint the pitch line over
    the waveform, and paint a span more than 25 cents from the notated pitch in the warning colour.
    One toggle hides it, and the toggle state persists.
12. Write the region drag of product story R-12 on `WaveformLane`: the region follows the pointer and
    shows its new start, and the drop lands on the nearest beat when snap is on and at the exact
    position when snap is off. It dispatches `SessionCommand::RegionSetStart`.
13. Write `src/record/controls.rs`: the input `Select`, the monitor toggle, the arm control, the
    track filter of product story R-03, and the take list of product story R-10 with a mute toggle
    per take, which dispatches `SessionCommand::TakeSetMuted`.
14. Write `src/shell/toolbar_record.rs`. Replace `group_count` with six and replace `render` with the
    six groups of contract 3.8, in the fixed removal order 6, 5, 4.
15. Write every test of the Tests section.
16. Run the Completion command and confirm that it passes.
17. Run `cargo clippy -p duet --all-targets -- -D warnings` and confirm that it is clean.
18. Commit on a branch named `chunk/k3-record-work-area`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. A user-interface test is a
`#[gpui_kit::test]` with the signature `fn name(cx: &mut TestAppContext)` and the first statement
`cx.update(gpui_kit::init);`. Every assert carries a message.

### Rung-two element tests, for the three elements this chunk fills

| Test | File | What it asserts |
|---|---|---|
| `record_view_waveform_lane_requests_its_layout_size` | `src/element/waveform_lane.rs` | The element requests the lane height the track sets and the width the system covers. |
| `record_view_waveform_lane_paints_the_expected_quads` | `src/element/waveform_lane.rs` | `Window::painted_quads()` holds the zero line quad and one column quad per device pixel of a fixed pyramid. |
| `record_view_waveform_lane_resolves_a_click_to_a_region` | `src/element/waveform_lane.rs` | A click at a point resolves to the correct `RegionId`. |
| `record_view_playhead_requests_its_layout_size` | `src/element/playhead_layer.rs` | The layer requests the height of the lane block and the system above it. |
| `record_view_playhead_paints_one_line_quad` | `src/element/playhead_layer.rs` | `Window::painted_quads()` holds exactly one playhead quad, and one more record-head quad while a pass runs. |
| `record_view_playhead_resolves_a_drag_to_a_position` | `src/element/playhead_layer.rs` | A drag on the stopped playhead resolves to the bar and beat the ruler shows. |
| `record_view_punch_range_requests_its_layout_size` | `src/element/punch_range.rs` | The band spans the lane block and the system above it. |
| `record_view_punch_range_paints_the_band_and_two_edges` | `src/element/punch_range.rs` | `Window::painted_quads()` holds one fill quad and two 2 px edge quads. |
| `record_view_punch_range_resolves_a_handle_drag` | `src/element/punch_range.rs` | A drag inside the 24 px hit width moves the matching handle and no other. |

### Rung-three user-interface behaviours this chunk owns

Architecture section 10.7 rung three items 4, 5 and 7, for the Record work area.

| Test | Rung-three item | File | What it asserts |
|---|---|---|---|
| `record_view_virtualizes_to_the_scroll_offset` | 4 | `src/record/view.rs` | At a given scroll offset the list renders exactly the systems the offset selects, and each lane block follows its own system. |
| `record_view_element_identity_comes_from_the_domain` | 5 | `src/record/view.rs` | Every lane `ElementId` comes from `TrackId` and every region identity from `RegionId`, never a list index. |
| `record_view_has_one_frame_driver` | 7 | `src/element/playhead_layer.rs` | Over ten frames exactly one entity of the Record work area requests an animation frame, and that entity is `PlayheadLayer`. |

### The remaining tests of this chunk

| Test | File | What it asserts |
|---|---|---|
| `path_cache_evicts_the_least_recently_used_key` | `src/record/cache.rs` | A cache at either half of B61 evicts the key at the front of `order` and no other. |
| `path_cache_keys_a_wrapped_region_per_system` | `src/record/cache.rs` | A region that runs past one system holds one entry per system, because `SystemId` is in the wrapped key. |
| `path_cache_holds_the_tessellated_value` | `src/record/cache.rs` | The stored value is an `Arc<Path<Pixels>>` and never a `PathPlacement`. |
| `path_cache_miss_paints_the_background_only` | `src/record/lane.rs` | A `LaneView` with no entry paints the lane background, requests the build, and tessellates nothing inside `render`. |
| `record_view_playhead_starts_and_stops_with_the_transport` | `src/element/playhead_layer.rs` | A rolling or locating transport keeps the loop alive; a still transport with a settled slot ends it; a `CoreEvent` from a transport verb restarts it. |
| `record_view_lane_opens_no_file` | `src/record/lane.rs` | `LaneView` reaches `duet-core` alone, and the crate carries no `duet-media` edge and no `duet-project` edge. |
| `record_view_take_stack_collapses_past_four_strips` | `src/record/lane.rs` | A fifth take collapses into a `Badge` that reads `+3`, and a click opens the take list `Sheet` (contract 3.4). |
| `record_view_pitch_overlay_marks_a_span_past_the_tolerance` | `src/record/lane.rs` | A sung pitch more than 25 cents from the notated pitch paints in the warning colour, and a span with no clear pitch paints no line and raises no error (PR R-15). |
| `record_view_region_drag_snaps_to_the_nearest_beat` | `src/element/waveform_lane.rs` | With snap on the drop lands on the nearest beat; with snap off it lands at the exact position (PR R-12). |
| `record_view_track_filter_shows_one_track_or_all` | `src/record/controls.rs` | `TrackFilter::Tracks` shows one lane per named track and `TrackFilter::All` shows one lane per track of the shown parts (PR R-03). |
| `record_view_take_list_mutes_one_pass` | `src/record/controls.rs` | The take list shows both passes of a doubled part and each one can be muted through `SessionCommand::TakeSetMuted` (PR R-10). |
| `record_view_controls_send_verbs_and_call_no_engine` | `src/record/controls.rs` | The input and monitor controls dispatch `Verb::TrackSetInput` and `Verb::TrackSetMonitor` and name no `duet-engine` type. |
| `record_view_toolbar_answers_six_groups` | `src/shell/toolbar_record.rs` | `group_count` answers six and `render` returns the six groups of contract 3.8 in the stated order. |

## Verification

1. `cargo nextest run -p duet -E 'test(record_view) + test(path_cache)' --no-tests=fail` passes and
   selects at least one test per term.
2. `cargo nextest run -p duet --no-tests=fail` passes, so chunk K1 and chunk K2 tests still pass.
3. `cargo clippy -p duet --all-targets -- -D warnings` prints no warning.
4. `git status` shows no change under `crates/bc_app/duet/lang_rust/src/shell/top_bar.rs` and no change under
   `crates/bc_app/duet/lang_rust/src/compose/`.
5. One commit on a branch named `chunk/k3-record-work-area`. The native git hook runs
   `scripts/dod.sh`.

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
