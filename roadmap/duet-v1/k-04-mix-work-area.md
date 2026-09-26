---
id: K4
line: K
depends_on: [K3, C3, D2, I3]
write_scope:
  - crates/bc_app/duet/lang_rust/src/element/level_meter.rs
  - crates/bc_app/duet/lang_rust/src/element/fader.rs
  - crates/bc_app/duet/lang_rust/src/element/knob.rs
  - crates/bc_app/duet/lang_rust/src/element/automation_lane.rs
  - crates/bc_app/duet/lang_rust/src/element/stage_curve.rs
  - crates/bc_app/duet/lang_rust/src/mix/view.rs
  - crates/bc_app/duet/lang_rust/src/mix/strip.rs
  - crates/bc_app/duet/lang_rust/src/mix/meter_layer.rs
  - crates/bc_app/duet/lang_rust/src/mix/automation.rs
  - crates/bc_app/duet/lang_rust/src/shell/toolbar_mix.rs
  - crates/bc_app/duet/lang_rust/benches/
  - crates/bc_app/duet/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: "serial-only: phase 12 holds one chunk, because line K has six chunks and no cross-line dependency exists after phase 11 (section 13.3)"
completion: "cargo nextest run -p duet -E 'test(mix_view) + test(stage_curve)' --no-tests=fail passes; cargo clippy -p duet --all-targets -- -D warnings is clean; commit SHA on a branch chunk/k4-mix-work-area"
---

# K4: The Mix work area, the five mixer elements, and the frame-cost bench

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk fills the Mix work area that chunk K1 created as a seam. It delivers
`LevelMeter`, `Fader`, `Knob`, `AutomationLane`, `StageCurve`, `MixView`, `MeterLayer`, the cached
sibling strips, mute and solo, the arm control on the strip, the reverb and delay bus strips, the
linear ruler, the Mix `ModeToolbar`, and the frame-cost bench. It implements architecture sections
7.3, 10.2, 10.3, 10.4, 10.5, 10.7 and 15.16, design contract sections 4.1 to 4.8, 8.3 and the
`StageCurve` appendix. It carries product stories M-02, M-03, M-04, M-05, M-06, M-07, R-05 and X-05.

**Three chunks of line K own `crates/bc_app/duet/lang_rust/Cargo.toml`, `crates/bc_app/duet/lang_rust/benches/`, and `Cargo.lock`, one
per phase.** Chunk K1 added the internal entries in phase 9, this chunk adds the Mix frame-cost
bench in phase 12, and chunk K5 adds the Master frame-cost bench in phase 13. SM6 keeps the three in
three phases, so the manifest has one writer per phase and the seam is serial (section 13.2, critic
C16-W10).

## Files

- `crates/bc_app/duet/lang_rust/src/element/level_meter.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/element/fader.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/element/knob.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/element/automation_lane.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/element/stage_curve.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/mix/view.rs` — modify. Chunk K1 left a seam view.
- `crates/bc_app/duet/lang_rust/src/mix/strip.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/mix/meter_layer.rs` — modify. Chunk K1 left a driver seam.
- `crates/bc_app/duet/lang_rust/src/mix/automation.rs` — modify. Documentation stub.
- `crates/bc_app/duet/lang_rust/src/shell/toolbar_mix.rs` — modify. Replace one `render` body and one `group_count`
  body. Edit `top_bar.rs` not at all.
- `crates/bc_app/duet/lang_rust/benches/mix_frame.rs` — create. The `criterion` bench of section 10.2.
- `crates/bc_app/duet/lang_rust/Cargo.toml` — modify. The `criterion` dev-dependency and the `[[bench]]` target with
  `harness = false`.
- `Cargo.lock` — modify (SM5 rule 2).

## Types and signatures

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/mix/`

Copied from architecture section 15.16.

```rust
/// The Mix work area.
#[derive(Debug)]
pub(crate) struct MixView {
    strips: Vec<Entity<StripView>>,
    meters: Entity<MeterLayer>,
    /// Recomputed when the strip list or the layout changes, not per frame
    /// (Appendix A).
    slots: Vec<MeterSlot>,
    cache: PathCache,
    /// The one owner of the linear path build task (section 10.5).
    build: Option<Task<()>>,
    split: Finite,
    zoom: ZoomScalar,
    state: WorkAreaState,
    focus: FocusHandle,
    subscriptions: Vec<Subscription>,
}

/// One channel strip inside the Mix work area.
#[derive(Debug)]
pub(crate) struct StripView {
    strip: StripId,
    kind: StripKind,
    name: StripName,
    slots: Vec<SlotConfig>,
    /// The parameter a drag is moving now, and the value it started at.
    dragging: Option<ParamId>,
    drag_origin: Finite,
    muted: bool,
    soloed: bool,
    armed: bool,
    focus: FocusHandle,
}

/// The one frame driver of Mix. It paints every strip meter in one canvas.
#[derive(Debug)]
pub(crate) struct MeterLayer {
    core: Entity<CoreHost>,
    /// Where each strip's meter sits, published by `MixView`.
    slots: Vec<MeterSlot>,
    /// The value the last frame painted.
    shown: MeterView,
}

/// Where one strip's meter sits in the mixer row, for `MeterLayer`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct MeterSlot { strip: StripId, x: f32, width: f32 }
```

**The START condition is `FrameDemand::input_live` or `FrameDemand::transport_moves`, and the STOP
condition is both of them false with `FrameDemand::meter_decays` false.** An armed strip or a
monitored track therefore keeps the loop alive whatever the transport does. **The driver holds no
copy of the armed set**: it calls `CoreHost::frame_demand` in `render`, so `DuetCore` stays the one
owner. `MixView` calls `cx.notify` on this entity when the armed set changes, which is a
notification and not a value.

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/element/`

Copied from architecture section 15.16.

```rust
/// One meter bar, painted from a reading its parent hands it.
///
/// **It is stateless.** It owns no reading, no clip latch, and no frame.
/// `held` is the resolved over mark that `MeterReader` already decided.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoElement)]
pub(crate) struct LevelMeter {
    strip: StripId,
    reading: MeterReading,
    held: bool,
}

/// One channel fader with a decibel taper and detents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoElement)]
pub(crate) struct Fader { parameter: ParamId, value: Finite, strip: StripId }

/// One radial control with an arc and a pointer line.
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoElement)]
pub(crate) struct Knob { parameter: ParamId, value: Finite, range: ParamRange }

/// A polyline and its handles over a linear time axis.
#[derive(Debug, IntoElement)]
pub(crate) struct AutomationLane { parameter: ParamId, points: Arc<CurvePoints>, zoom: ZoomStep }

/// One processor's response curve, with a live dot where the contract asks.
#[derive(Debug, IntoElement)]
pub(crate) struct StageCurve { slot: SlotId, kind: SlotKind, params: Arc<[Finite]>, cell: u16, measure: Option<SlotMeasure> }
```

`LevelMeter`, `Fader` and `Knob` derive `Copy`. **No type in `crates/duet` carries a
`missing_copy_implementations` expectation**: the lint reads reachability from a library root, so it
cannot fire inside a binary, and `-D warnings` then turns an unfulfilled expectation into a build
error (section 15.16, critic S3).

`LevelMeter` **carries no scale field**. Design contract 4.4 fixes the scale, B106 carries its
numbers, and `duet_dsp::meter_law::db_to_fraction` applies it.

`Knob::range` is the parameter's own value range, which `duet_session::param_range` answers. Design
contract 4.5 moves the value by one percent of that range per pixel.

`StageCurve::cell` is the section 7.3 cell index of the slot, and `measure` is what
`CoreHost::slot_measure` answered for it this frame. **The two quantities are separate fields of
`SlotMeasure`**, because contract 5.2 draws the compressor dot at the INPUT level and the limiter
bar at the gain REDUCTION, and one scalar cannot carry both.

### Consumed from other crates

| Type | Crate | Declaring section |
|---|---|---|
| `MeterView`, `MeterReading`, `SlotMeasure`, `HeldMarks` | `duet-dsp` | 7.3 |
| `meter_law::db_to_fraction`, `fader_fraction_to_db`, `fader_db_to_fraction` | `duet-dsp` | 7.3, B106, B143 to B145 |
| `StripId`, `StripKind`, `StripName`, `SlotConfig`, `SlotId`, `SlotKind`, `ParamId`, `ParamRange`, `param_range`, `BusRole`, `CurvePoints` | `duet-session` | 15.4 |
| `Verb::MixSetParam`, `MixSetMute`, `MixSetSolo`, `MixClearSolo`, `MixAddBus`, `MixWriteCurve`, `MeterReset` | `duet-command` | 9.1 |
| `ZoomScalar`, `ZoomStep`, `Finite` | `duet-command`, `duet-time` | 10.2, 2.6a |
| `CoreHost`, `FrameDemand`, `PathCache`, `PathKey`, `WorkAreaState`, `DuetTokens` | this crate, chunks K1 and K3 | 15.16 |

**`crates/duet` names no `MeterSnapshot`.** `duet-core` holds the `Output<MeterSnapshot>` and
`MeterReader` answers in `MeterView` (section 7.3, section 1.3 rule 6, critic C16-W6).

### The mixer strip, copied from design contract 4.2

Nine rows, top to bottom, at 96 px default width: 1, 4 px part colour bar; 2, 22 px name; 3, 24 px
input `Select` plus a 20 px monitor toggle; 4, four insert slots of 20 px; 5, four send rows of
22 px; 6, 44 px pan knob of 28 px; 7, 220 px fader and meter; 8, 28 px mute, solo and arm; 9, 24 px
output `Select`. Every strip repeats the same row heights, so the row boundaries form continuous
horizontal lines across the mixer.

### The meter, copied from design contract 4.4

| Item | Contract value |
|---|---|
| Size | 14 px wide, 200 px tall, beside the fader with a 6 px gap |
| Scale | -60 dB at the bottom to +6 dB at the top, non-linear, -20 dB at 50 percent of the height (B106) |
| Zones | `duet.meter.low` below -12 dB, `duet.meter.mid` from -12 to -3 dB, `duet.meter.high` above -3 dB, as a hard boundary (B114, B115) |
| Peak cap | a 2 px line in `duet.meter.peak_cap`; it holds for B111 and then falls at B112 |
| Clip | the top 6 px in `duet.meter.clip` plus the word `CLIP` beside the meter; it holds until the user clicks the meter or runs `Clear clip` |
| Numeric readout | painted by `MeterLayer`, 11 px tabular, 40 px wide, right aligned |
| Animation frame | every strip meter paints in one canvas that `MeterLayer` owns; the mixer view itself is never marked dirty by the meter frame |

## Steps

1. Read every file of the write scope. Confirm that chunk K1 created each one, that `mix/view.rs`
   holds the seam view and `mix/meter_layer.rs` holds the driver seam, and that
   `crates/bc_app/duet/lang_rust/benches/` does not exist. Stop and report a discrepancy.
2. Add `criterion` as a dev-dependency with `{ workspace = true }`, and add the `[[bench]]` target
   with `harness = false` and `name = "mix_frame"`. Run `cargo build --workspace` and commit the
   `Cargo.lock` it produces.
3. Write the failing test `mix_view_meter_layer_owns_every_bar` in `src/mix/meter_layer.rs`, as a
   `#[gpui_kit::test]` with the signature `fn mix_view_meter_layer_owns_every_bar(cx: &mut TestAppContext)`
   and the first statement `cx.update(gpui_kit::init);`.
4. Run `cargo nextest run -p duet -E 'test(mix_view)' --no-tests=fail` and confirm that it fails.
5. Write `LevelMeter` in `src/element/level_meter.rs`. It is stateless: it paints the reading and
   the resolved mark its caller hands it, through `duet_dsp::meter_law::db_to_fraction`. It is not
   the mixer strip row; it serves the single meters outside that canvas.
6. Write `Fader` in `src/element/fader.rs` to contract 4.3: a 200 px travel, unity gain at 72 percent
   of the travel, a detent at 0 dB within 2 px, a `shift` drag with a 5x finer ratio, and a
   double-click that returns to 0 dB. Read the taper from `duet_dsp::meter_law`.
7. Write `Knob` in `src/element/knob.rs` to contract 4.5: a 270 degree sweep, a vertical-only drag at
   one percent of `range` per pixel, `shift` at 0.2 percent, and a horizontal drag that does nothing.
8. Write `AutomationLane` in `src/element/automation_lane.rs` to contract 4.8: four guide lines, a
   2 px polyline in `duet.automation.line`, a 7 px point circle with a 20 x 20 px hit area, a
   double-click that adds a point, and a 5 px diamond curve handle at each segment midpoint.
9. Write `StageCurve` in `src/element/stage_curve.rs`. It paints an equalizer response, a compressor
   transfer curve with a live dot at `SlotMeasure::input`, or a limiter gain-reduction bar at
   `SlotMeasure::reduction`. `params` is an `Arc<[Finite]>` that the owner fills when a parameter
   changes, not per frame, so a build is one pointer copy and one polyline over stored samples. The
   element is 96 px in a strip inspector and 160 px in the master chain.
10. Write `MeterLayer` in `src/mix/meter_layer.rs`. Paint every strip meter, every peak cap, every
    `CLIP` text and every numeric readout from one `CoreHost::meters` call. Read
    `CoreHost::frame_demand` in `render` and request an animation frame while any term is set.
11. Write `MixView` in `src/mix/view.rs`. Build the `v_resizable` of contract 1.6 with the linear
    timeline above and the mixer strip row below. Publish `slots` as a plain `Vec<MeterSlot>`
    whenever the strip list or the layout changes, not per frame. Build each strip as an
    `Entity::cached` child.
12. Own the linear path cache in `MixView::cache` and the build task in `MixView::build`. The linear
    key is `PathKey::Linear { source, region, zoom }`, which chunk K3 declared. `MixView` reads no
    other view's cache.
13. Write `StripView` in `src/mix/strip.rs` to contract 4.2, with the nine rows, mute and solo of
    contract 4.7, the arm control of product story R-05, the send rows of contract 4.6, and the
    reverb and delay bus strips of product story M-04.
14. Write `src/mix/automation.rs` with the curve edit of product story M-07, which dispatches
    `Verb::MixWriteCurve`.
15. Write the linear ruler of product story X-05 inside `MixView` from `h_flex` and quads, per
    contract 4.8. It is not a thirteenth element.
16. Write `src/shell/toolbar_mix.rs`. Design contract section 4 names no Mix top bar section, so the
    group set comes from product stories M-03 and M-04 and from contract 4.8: a snap grid `Select`
    shared with Compose, a `Clear solo` control that contract 4.7 puts in the transport bar, and the
    view controls of contract 4.8. Set `group_count` to the number of groups `render` returns.
17. Write the `criterion` bench in `crates/bc_app/duet/lang_rust/benches/mix_frame.rs`. It measures one Mix frame at
    B1, with the cached siblings and without them, and reports both against B4. The same run records
    what one frame leaves for an immediate verb at B5.
18. Write every test of the Tests section.
19. Run the Completion command and confirm that it passes.
20. Run `cargo clippy -p duet --all-targets -- -D warnings` and confirm that it is clean.
21. Commit on a branch named `chunk/k4-mix-work-area`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. A user-interface test is a
`#[gpui_kit::test]` with the signature `fn name(cx: &mut TestAppContext)` and the first statement
`cx.update(gpui_kit::init);`. Every assert carries a message.

### Rung-two element tests, for the five elements this chunk fills

Each of the five carries the three-part test of architecture section 10.7 rung two: the requested
layout size, the painted quad set through `Window::painted_quads()`, and a pointer event that
resolves to the element's own domain identifier.

| Test | File | What it asserts |
|---|---|---|
| `mix_view_level_meter_requests_its_layout_size` | `src/element/level_meter.rs` | 14 px wide and 200 px tall (contract 4.4). |
| `mix_view_level_meter_paints_the_bar_and_the_cap` | `src/element/level_meter.rs` | `Window::painted_quads()` holds one bar quad, one 2 px cap quad, and one clip quad when `held` is set. |
| `mix_view_level_meter_resolves_a_click_to_its_strip` | `src/element/level_meter.rs` | A click resolves to the element's own `StripId`. |
| `mix_view_fader_requests_its_layout_size` | `src/element/fader.rs` | The 220 px row with a 200 px travel (contract 4.3). |
| `mix_view_fader_paints_the_track_the_cap_and_the_ticks` | `src/element/fader.rs` | The quad set holds the track, the cap, and one tick per contract 4.3 value, with the 0 dB tick at 2 px. |
| `mix_view_fader_resolves_a_drag_to_its_parameter` | `src/element/fader.rs` | A drag resolves to the element's own `ParamId` and `StripId`. |
| `stage_curve_requests_its_layout_size` | `src/element/stage_curve.rs` | 96 px in a strip inspector and 160 px in the master chain. |
| `stage_curve_paints_a_gain_reduction_bar_for_a_limiter` | `src/element/stage_curve.rs` | A `SlotKind::Limiter` paints an 8 px bar that grows to the leading side, from `SlotMeasure::reduction`. |
| `stage_curve_resolves_a_click_to_its_slot` | `src/element/stage_curve.rs` | A click resolves to the element's own `SlotId`. |
| `mix_view_knob_requests_its_layout_size` | `src/element/knob.rs` | 28 px outer diameter, with a 28 x 28 px hit area even for the 18 px knob. |
| `mix_view_knob_paints_the_track_and_the_value_arc` | `src/element/knob.rs` | The quad and path set holds the track stroke, the value stroke, and the pointer line. |
| `mix_view_knob_ignores_a_horizontal_drag` | `src/element/knob.rs` | A horizontal drag changes no value, and a vertical drag moves one percent of `range` per pixel (contract 4.5). |
| `mix_view_automation_lane_requests_its_layout_size` | `src/element/automation_lane.rs` | 40 px per open parameter (contract 4.8). |
| `mix_view_automation_lane_paints_four_guide_lines` | `src/element/automation_lane.rs` | The quad set holds four guide quads at 0, 25, 50 and 75 percent of the height. |
| `mix_view_automation_lane_resolves_a_point_click` | `src/element/automation_lane.rs` | A click inside the 20 x 20 px hit area resolves to that point and to no other. |

### Rung-three user-interface behaviours this chunk owns

Architecture section 10.7 rung three items 7, 9, 11 and 13, for the Mix work area.

| Test | Rung-three item | File | What it asserts |
|---|---|---|---|
| `mix_view_has_one_frame_driver` | 7 | `src/mix/meter_layer.rs` | Over ten frames exactly one entity of the Mix work area requests an animation frame, and that entity is `MeterLayer`. |
| `mix_view_clip_latch_lights_then_clears` | 9 | `src/mix/meter_layer.rs` | It publishes a snapshot whose `over` bit and `latched` entry name strip `i`, runs one frame, and asserts that `CLIP` paints on strip `i` and on no other. It then dispatches `ClearClip`, runs ten frames against the same stale snapshot with no audio cycle, and asserts that `CLIP` paints nowhere. It then publishes a snapshot whose `latched` entry carries the new reset generation and asserts that `CLIP` paints again. |
| `mix_view_meter_layer_owns_every_bar` | 11 | `src/mix/meter_layer.rs` | `MeterLayer` paints every live bar and every held over mark, and `LevelMeter` paints the bar and the resolved mark a caller hands it and holds neither value. |
| `mix_view_meter_driver_starts_and_stops` | 13 | `src/mix/meter_layer.rs` | It arms one track with the transport stopped, publishes a rising `MeterSnapshot` on each of ten frames, and asserts a different painted value on each one. It then disarms the track, runs frames until the meter reads silence with no held mark, and asserts that the driver requests no further animation frame. A third half sets one track to `MonitorMode::Always` with no strip armed and asserts the same liveness, then sets `MonitorMode::Off` and asserts the loop ends. |

### The remaining tests of this chunk

| Test | File | What it asserts |
|---|---|---|
| `mix_view_caches_every_sibling_strip` | `src/mix/view.rs` | Each strip child is built with `Entity::cached`, and a meter frame re-renders no `StripView`. |
| `mix_view_publishes_slots_on_a_layout_change_only` | `src/mix/view.rs` | `slots` is recomputed when the strip list or the layout changes, and never per frame. |
| `mix_view_holds_its_own_linear_cache` | `src/mix/view.rs` | `MixView::cache` is keyed by `PathKey::Linear` and reads no other view's cache. |
| `mix_view_holds_the_path_build_task` | `src/mix/view.rs` | `MixView::build` holds the `Task`, so the work is not cancelled at the end of the statement that started it (critic C16-W5). |
| `mix_view_strip_shows_the_nine_rows` | `src/mix/strip.rs` | The strip lays out the nine rows of contract 4.2 at the stated heights, so the row boundaries line up across the mixer. |
| `mix_view_solo_drops_the_implicit_mute_alpha` | `src/mix/strip.rs` | An implicit mute drops the name and fader column to 55 percent alpha and draws a `warning` outline on `M` with no fill (contract 4.7). |
| `mix_view_clear_solo_appears_while_a_solo_holds` | `src/mix/strip.rs` | Any solo shows the `Clear solo` control, and the control dispatches `Verb::MixClearSolo`. |
| `mix_view_adds_a_reverb_bus_with_a_role` | `src/mix/strip.rs` | `Verb::MixAddBus` carries a `BusRole`, so a reverb bus and a delay bus are one verb with a role (PR M-04). |
| `mix_view_automation_writes_one_curve` | `src/mix/automation.rs` | A point edit dispatches one `Verb::MixWriteCurve` per coherent change (PR M-07). |
| `mix_view_toolbar_returns_its_group_set` | `src/shell/toolbar_mix.rs` | `group_count` matches the group count `render` returns, and the removal order is fixed. |
| `mix_view_renders_the_work_area_state_above_its_content` | `src/mix/view.rs` | Each `WorkAreaState` arm other than `Ready` renders the contract 8.3 surface. |

**The frame budget is a bench, not an assertion.** No test asserts a frame cost. The `criterion`
bench reports against B4 and B5 (section 10.7).

## Verification

1. `cargo nextest run -p duet -E 'test(mix_view) + test(stage_curve)' --no-tests=fail` passes and
   selects at least one test per term.
2. `cargo nextest run -p duet --no-tests=fail` passes.
3. `cargo bench -p duet --bench mix_frame -- --test` builds and runs one iteration.
4. `cargo clippy -p duet --all-targets -- -D warnings` prints no warning.
5. `git status` shows no change under `crates/bc_app/duet/lang_rust/src/shell/top_bar.rs`, `crates/bc_app/duet/lang_rust/src/compose/`
   or `crates/bc_app/duet/lang_rust/src/record/`.
6. One commit on a branch named `chunk/k4-mix-work-area`. The native git hook runs `scripts/dod.sh`.

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
