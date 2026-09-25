---
id: K1
line: K
depends_on: [I1, J1]
write_scope:
  - crates/bc_app/duet/lang_rust/src/main.rs
  - crates/bc_app/duet/lang_rust/src/app.rs
  - crates/bc_app/duet/lang_rust/src/element.rs
  - crates/bc_app/duet/lang_rust/src/tokens.rs
  - crates/bc_app/duet/lang_rust/src/shell.rs
  - crates/bc_app/duet/lang_rust/src/compose.rs
  - crates/bc_app/duet/lang_rust/src/record.rs
  - crates/bc_app/duet/lang_rust/src/mix.rs
  - crates/bc_app/duet/lang_rust/src/master.rs
  - crates/bc_app/duet/lang_rust/src/element/staff_system.rs
  - crates/bc_app/duet/lang_rust/src/element/waveform_lane.rs
  - crates/bc_app/duet/lang_rust/src/element/playhead_layer.rs
  - crates/bc_app/duet/lang_rust/src/element/punch_range.rs
  - crates/bc_app/duet/lang_rust/src/element/level_meter.rs
  - crates/bc_app/duet/lang_rust/src/element/fader.rs
  - crates/bc_app/duet/lang_rust/src/element/knob.rs
  - crates/bc_app/duet/lang_rust/src/element/automation_lane.rs
  - crates/bc_app/duet/lang_rust/src/element/stage_curve.rs
  - crates/bc_app/duet/lang_rust/src/element/lufs_meter.rs
  - crates/bc_app/duet/lang_rust/src/element/toolbar.rs
  - crates/bc_app/duet/lang_rust/src/shell/root.rs
  - crates/bc_app/duet/lang_rust/src/shell/core_host.rs
  - crates/bc_app/duet/lang_rust/src/shell/agent_bridge.rs
  - crates/bc_app/duet/lang_rust/src/shell/mode_switcher.rs
  - crates/bc_app/duet/lang_rust/src/shell/top_bar.rs
  - crates/bc_app/duet/lang_rust/src/shell/transport_bar.rs
  - crates/bc_app/duet/lang_rust/src/shell/menu.rs
  - crates/bc_app/duet/lang_rust/src/shell/fault_text.rs
  - crates/bc_app/duet/lang_rust/src/shell/states.rs
  - crates/bc_app/duet/lang_rust/src/shell/title_bar.rs
  - crates/bc_app/duet/lang_rust/src/shell/sidebar.rs
  - crates/bc_app/duet/lang_rust/src/shell/inspector.rs
  - crates/bc_app/duet/lang_rust/src/shell/status_bar.rs
  - crates/bc_app/duet/lang_rust/src/shell/history_sheet.rs
  - crates/bc_app/duet/lang_rust/src/shell/start.rs
  - crates/bc_app/duet/lang_rust/src/shell/view_state.rs
  - crates/bc_app/duet/lang_rust/src/shell/toolbar_compose.rs
  - crates/bc_app/duet/lang_rust/src/shell/toolbar_record.rs
  - crates/bc_app/duet/lang_rust/src/shell/toolbar_mix.rs
  - crates/bc_app/duet/lang_rust/src/shell/toolbar_master.rs
  - crates/bc_app/duet/lang_rust/src/compose/view.rs
  - crates/bc_app/duet/lang_rust/src/compose/caret.rs
  - crates/bc_app/duet/lang_rust/src/compose/menu.rs
  - crates/bc_app/duet/lang_rust/src/compose/duration.rs
  - crates/bc_app/duet/lang_rust/src/record/view.rs
  - crates/bc_app/duet/lang_rust/src/record/lane.rs
  - crates/bc_app/duet/lang_rust/src/record/cache.rs
  - crates/bc_app/duet/lang_rust/src/record/controls.rs
  - crates/bc_app/duet/lang_rust/src/mix/view.rs
  - crates/bc_app/duet/lang_rust/src/mix/strip.rs
  - crates/bc_app/duet/lang_rust/src/mix/meter_layer.rs
  - crates/bc_app/duet/lang_rust/src/mix/automation.rs
  - crates/bc_app/duet/lang_rust/src/master/view.rs
  - crates/bc_app/duet/lang_rust/src/master/export_dialog.rs
  - crates/bc_app/duet/lang_rust/src/master/report.rs
  - crates/bc_app/duet/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet -E 'test(shell) + test(view_round_trip)' --no-tests=fail passes; cargo clippy -p duet --all-targets -- -D warnings is clean; commit SHA on a branch chunk/k1-application-shell"
---

# K1: The application shell, the entity tree, and the token layer

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk replaces the skeleton application with the shell of architecture section 10.
It delivers the command-line front, the mode machine, the entity tree, `CoreHost` with the frame
pump of section 10.2, the note-entry drain registration, the theme observer that re-resolves
`DuetTokens`, `AgentBridge`, `TopBar` with the `Toolbar` element and its overflow rule,
`TransportBar`, `ModeSwitcher`, `MenuHost`, `fault_text`, `WorkAreaState`, and the `ViewState` round
trip. It implements architecture sections 9.3, 9.5, 10.1, 10.2, 10.3, 10.4, 10.7, 12.4, 15.16 and
Appendix A, design contract sections 0, 1.1 to 1.10, 7.1, 7.2, 7.3, 8 and 9, and ADR
`adr/0006-wrapped-timeline.md`. It carries product stories A-02, C-15, C-21, C-22, R-01, R-05, R-06,
R-13, M-01, X-06, X-07, X-08, X-09, X-10, X-11, X-12 and X-14.

This chunk is the FIRST chunk of line K, so SM2 makes it create every module file the whole line
will ever need, at every depth. The current `crates/duet` holds `src/main.rs` and `src/app.rs`, and
this chunk **removes `src/app.rs`**: `src/shell/root.rs` replaces it.

## Files

`crates/bc_app/duet/lang_rust/src/main.rs` — modify. `crates/bc_app/duet/lang_rust/src/app.rs` — remove. `crates/bc_app/duet/lang_rust/Cargo.toml` and
`Cargo.lock` — modify. Every other path of the write scope — create.

**Which files hold code and which hold documentation alone.** SM2 asks for a stub, and section 13.0
states one exception: *a seam implementation is not a stub; it is a file a later chunk REPLACES, and
it is reachable from the call graph on the day it lands.* Architecture section 15.16 declares
`DuetApp` with one `Entity<T>` field per work-area view, so the four work-area views and the three
frame drivers must exist at the end of phase 9. They take the seam exception.

| File set | Shape this chunk gives it | Who replaces the body |
|---|---|---|
| `main.rs`, `element.rs`, `tokens.rs`, `shell.rs`, `compose.rs`, `record.rs`, `mix.rs`, `master.rs` | Filled | nobody |
| `shell/{root,core_host,agent_bridge,mode_switcher,top_bar,transport_bar,menu,fault_text,states}.rs` | Filled | nobody |
| `element/toolbar.rs` | Filled: the `Toolbar` element and the overflow rule of contract 1.5 | nobody |
| `shell/{toolbar_compose,toolbar_record,toolbar_mix,toolbar_master}.rs` | Seam: the whole `impl ModeToolbar` block, `group_count` answers zero and `render` returns an empty `ToolbarGroups` | K2, K3, K4, K5 |
| `compose/view.rs`, `record/view.rs`, `mix/view.rs`, `master/view.rs` | Seam: the view struct with `state: WorkAreaState`, `focus: FocusHandle`, `subscriptions: Vec<Subscription>`, and a `Render` impl that paints the section 8 surface | K2, K3, K4, K5 |
| `element/playhead_layer.rs`, `mix/meter_layer.rs` | Seam: the driver struct with `core: Entity<CoreHost>` and `shown`, and a `Render` impl that paints nothing and requests no frame | K3, K4 |
| `element/{staff_system,waveform_lane,punch_range,level_meter,fader,knob,automation_lane,stage_curve,lufs_meter}.rs` | Documentation stub | K2, K3, K3, K4, K4, K4, K4, K4, K5 |
| `shell/{title_bar,sidebar,inspector,status_bar,history_sheet,start,view_state}.rs` | Documentation stub | K6 |
| `compose/{caret,menu,duration}.rs`, `record/{lane,cache,controls}.rs`, `mix/{strip,automation}.rs`, `master/{export_dialog,report}.rs` | Documentation stub | K2, K3, K4, K5 |

## Types and signatures

### The current state of the code, which this chunk replaces

`crates/bc_app/duet/lang_rust/src/main.rs` holds `init_tracing`, `main_window_options`, `open_main_window` and
`main`. `main` calls `gpui_kit::application().with_assets(gpui_kit::assets::Assets)` and then
`gpui_kit::init(app)` inside `run`, and it spawns `open_main_window` with a detached task.
`crates/bc_app/duet/lang_rust/src/app.rs` holds a `DuetApp` with one `clicks: u32` field and a counter button. That
`DuetApp` is a skeleton and this chunk replaces it with the declaration below.

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/shell/root.rs`

Copied from architecture section 15.16. Every type in `crates/duet` is `pub(crate)`.

```rust
/// The root view. It owns the mode and every child entity (section 10.2).
#[derive(Debug)]
pub(crate) struct DuetApp {
    mode: Mode,
    core: Entity<CoreHost>,
    agent: Entity<AgentBridge>,
    title_bar: Entity<TitleBar>,
    mode_switcher: Entity<ModeSwitcher>,
    top_bar: Entity<TopBar>,
    transport_bar: Entity<TransportBar>,
    sidebar: Entity<Sidebar>,
    inspector: Entity<Inspector>,
    status_bar: Entity<StatusBar>,
    history: Entity<HistorySheet>,
    start: Entity<StartView>,
    menu: Entity<MenuHost>,
    view_state: Entity<ViewStateStore>,
    compose: Entity<ComposeView>,
    record: Entity<RecordView>,
    mix: Entity<MixView>,
    master: Entity<MasterView>,
    focus: FocusHandle,
    subscriptions: Vec<Subscription>,
}

/// One fact `DuetApp` emits to its children.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AppEvent { ModeChanged(Mode), PendingDuration(NoteValue), ProjectOpened, ProjectClosed, SelectionChanged }

/// Every way the shell refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum AppError { Core(CoreError), Shutdown(ShutdownError), NoProject }
```

The mode actions, copied from architecture section 10.1 and 15.16:

```rust
pub use duet_command::Mode;

gpui_kit::actions!(duet, [EnterCompose, EnterRecord, EnterMix, EnterMaster, ClearClip]);
```

`ClearClip` dispatches `Verb::MeterReset { strip: None }`, so the command of design contract 4.4 and
a click on one strip's meter share one verb (critic C9). Every transition is legal except one: the
application refuses to leave Record while `RecordState::Recording` holds. The handler shows a
notification and changes nothing.

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/shell/core_host.rs`

Copied from architecture section 15.16.

```rust
/// Why a frame driver runs this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct FrameDemand {
    transport_moves: bool,
    input_live: bool,
    meter_decays: bool,
}

/// The entity that owns `DuetCore`, drains its event channel, and holds the
/// one read end of each audio publication.
#[derive(Debug)]
pub(crate) struct CoreHost {
    core: DuetCore,
    drain: Task<()>,
    version: Version,
    engine: EngineState,
    fault: Option<EngineFault>,
    meters: MeterReader,
    transport: TransportReader,
    client: CoreClient,
    pumping: bool,
    theme: Subscription,
}

impl CoreHost {
    /// Read both publications into their slots and re-arm the frame pump.
    pub(crate) fn poll_frame(&mut self, window: &mut Window, cx: &mut Context<Self>);

    /// Why a frame driver runs this frame, from the slots the last poll
    /// filled and the armed set `DuetCore` holds.
    #[must_use]
    pub(crate) fn frame_demand(&self) -> FrameDemand;

    /// The meter values of this frame, as `MeterReader::latest` answers them.
    #[must_use]
    pub(crate) fn meters(&self) -> MeterView;

    /// The transport of this frame, as `TransportReader::latest` answers it.
    #[must_use]
    pub(crate) fn transport(&self) -> TransportView;

    /// The measurement at one cell of the section 7.3 formula, or `None`
    /// when the cell is at or above the number the topology fills.
    #[must_use]
    pub(crate) fn slot_measure(&self, cell: u16) -> Option<SlotMeasure>;
}
```

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/shell/agent_bridge.rs`

Copied from architecture section 15.16.

```rust
/// One verb in flight from a tokio task to the gateway, with its reply end.
pub(crate) struct GatewayCall {
    request: GatewayRequest,
    /// The write end of the reply. `send` consumes it, so one reply exists.
    reply: OneshotSender<Result<VerbOutcome, GatewayError>>,
}

impl core::fmt::Debug for GatewayCall {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("GatewayCall").finish_non_exhaustive()
    }
}

/// The tokio side of one in-flight verb (section 9.5 rule 5).
pub(crate) struct AgentCall {
    request: GatewayRequest,
    /// The read end of the reply.
    reply: OneshotReceiver<Result<VerbOutcome, GatewayError>>,
}

impl core::fmt::Debug for AgentCall {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("AgentCall").finish_non_exhaustive()
    }
}

/// The entity that owns the tokio runtime and the shutdown choke point.
#[derive(Debug)]
pub(crate) struct AgentBridge {
    runtime: Option<Runtime>,
    cancel: CancellationToken,
    finished: bool,
    requests: Sender<GatewayCall>,
    inbox: CoreReceiver<GatewayCall>,
}

/// Every way the shutdown choke point refuses. `Drop` absorbs it.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub(crate) enum ShutdownError { RuntimeStuck, ReplyTimeout, AlreadyRun }
```

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/shell/states.rs`

Copied from architecture section 10.2.

```rust
/// What a work area shows instead of its content.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum WorkAreaState {
    Loading { what: SharedString, done: u32, total: u32 },
    Empty,
    Error { message: SharedString },
    NoDevice { state: EngineState },
    Ready,
}
```

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/tokens.rs`

Copied from architecture section 15.16. The struct declares 27 fields; design contract 7.2 states 29
token ids, and the remaining two are the derived pair `duet.wave.fill` and `duet.wave.rms`, which
`wave_fill` and `wave_rms` answer.

```rust
/// Every colour role design contract section 7 defines, resolved once.
#[derive(Debug, Clone)]
pub(crate) struct DuetTokens {
    staff_paper: Hsla,
    staff_ink: Hsla,
    staff_ink_muted: Hsla,
    staff_line: Hsla,
    staff_grid: Hsla,
    staff_preview: Hsla,
    note_selected: Hsla,
    note_selected_halo: Hsla,
    note_cursor: Hsla,
    part_soprano: Hsla,
    part_alto: Hsla,
    part_tenor: Hsla,
    part_bass: Hsla,
    wave_zero: Hsla,
    wave_absent: Hsla,
    record_red: Hsla,
    record_tint: Hsla,
    meter_low: Hsla,
    meter_mid: Hsla,
    meter_high: Hsla,
    meter_clip: Hsla,
    meter_peak_cap: Hsla,
    meter_scale: Hsla,
    playhead: Hsla,
    automation_line: Hsla,
    lufs_target: Hsla,
    lufs_tolerance: Hsla,
}

impl DuetTokens {
    /// Resolve every role against the theme in scope.
    pub(crate) fn resolve(cx: &App) -> Self;

    /// The colour of one part, which contract 7.3 rule 2 makes the one
    /// identity of that part across the sidebar, the lane, the waveform, the
    /// strip, and the filter.
    #[must_use]
    pub(crate) fn part(&self, voice: VoiceType) -> Hsla;

    /// The waveform fill of one part, which contract 7.2 derives from the
    /// part colour.
    #[must_use]
    pub(crate) fn wave_fill(&self, voice: VoiceType) -> Hsla;

    /// The waveform mean-square colour of one part, which is the part colour
    /// itself.
    #[must_use]
    pub(crate) fn wave_rms(&self, voice: VoiceType) -> Hsla;
}

impl Global for DuetTokens {}
```

**A contract token id is written in dotted form and this crate writes it in snake case.** The two are
one name: contract `duet.meter.peak_cap` is the `meter_peak_cap` field. A Rust field cannot carry a
dot.

`resolve` applies contract 7.3 rule 6 and contract 9.1: a role the contract marks `own` is checked
against the live background and moved by up to twelve lightness points to meet the contrast floor.
When twelve points are not enough it logs a warning that names the token and uses `foreground` or
`danger` instead.

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/element/toolbar.rs` and `src/shell/top_bar.rs`

Copied from architecture section 15.16.

```rust
/// One control inside a toolbar group.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ToolbarControl {
    Button { id: MenuPath, label: SharedString },
    Toggle { id: MenuPath, label: SharedString, on: bool },
    Select { id: MenuPath, label: SharedString, options: Vec<SharedString>, chosen: usize },
    Separator,
}

/// One group of the top bar: its label and the controls contract 1.5 gives it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolbarGroup {
    id: MenuPath,
    label: SharedString,
    controls: Vec<ToolbarControl>,
}

/// One mode's toolbar, as the groups the design contract names for that mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolbarGroups {
    mode: Mode,
    groups: Vec<ToolbarGroup>,
    visible: usize,
}

/// The top bar container with the overflow rule.
#[derive(Debug, Clone, PartialEq, Eq, IntoElement)]
pub(crate) struct Toolbar { groups: ToolbarGroups }

/// One mode's group set inside the top bar.
pub(crate) trait ModeToolbar: core::fmt::Debug {
    /// How many groups this mode has, in the fixed removal order of
    /// contract 1.5. The order is fixed per mode; the count that fits is not.
    fn group_count(&self) -> usize;

    /// Build this mode's groups, with `visible` of them on the bar and every
    /// later group in the overflow menu.
    fn render(&self, visible: usize, window: &mut Window, cx: &mut App) -> ToolbarGroups;
}

/// The Compose group set. Chunk K1 creates it and K2 fills `render`.
#[derive(Debug)]
pub(crate) struct ComposeToolbar;
/// The Record group set. Chunk K1 creates it and K3 fills `render`.
#[derive(Debug)]
pub(crate) struct RecordToolbar;
/// The Mix group set. Chunk K1 creates it and K4 fills `render`.
#[derive(Debug)]
pub(crate) struct MixToolbar;
/// The Master group set. Chunk K1 creates it and K5 fills `render`.
#[derive(Debug)]
pub(crate) struct MasterToolbar;

/// The top bar shell.
#[derive(Debug)]
pub(crate) struct TopBar {
    mode: Mode,
    toolbars: BTreeMap<Mode, Box<dyn ModeToolbar>>,
    available: LogicalPx,
    focus: FocusHandle,
}
```

**`TopBar` writes `top_bar.rs`, the registry, and the four implementations; none of K2 to K5 edits
`top_bar.rs`.** Each of the four compiles and runs at the end of this phase: `group_count` returns
zero and `render` returns an empty `ToolbarGroups`.

### Declared by this chunk, in `src/shell/{mode_switcher,transport_bar,menu}.rs`

Copied from architecture section 15.16.

```rust
/// The four mode buttons.
#[derive(Debug)]
pub(crate) struct ModeSwitcher { mode: Mode, focus: FocusHandle }

/// The transport controls and the record arm.
#[derive(Debug)]
pub(crate) struct TransportBar {
    transport: TransportView,
    armed: bool,
    punch: Option<Span>,
    loop_range: Option<Span>,
    focus: FocusHandle,
}

/// The platform menu bar on macOS and the in-window bar on Linux.
#[derive(Debug)]
pub(crate) struct MenuHost {
    paths: BTreeMap<MenuPath, ElementId>,
    mode: Mode,
    subscriptions: Vec<Subscription>,
}

/// One path through the menu model, from the root to the item.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct MenuPath(Box<str>);
```

### Consumed from `duet-core`, `duet-command` and `duet-agent`

| Type | Crate | Declaring section |
|---|---|---|
| `DuetCore`, `CoreClient`, `CoreLink`, `CoreReceiver`, `MeterReader`, `TransportReader`, `CoreError` | `duet-core` | 15.14 |
| `DuetCore::drain_note_entries` | `duet-core` | 13.2, 8.3 |
| `Verb`, `VerbOutcome`, `GatewayRequest`, `GatewayError`, `CoreEvent` | `duet-command` | 9.1, 15.5 |
| `Mode`, `ViewState`, `ViewDocument`, `ModeView`, `LogicalPx`, `ZoomStep`, `ZoomScalar`, `ScrollOffset`, `TrackFilter`, `WindowGeometry` | `duet-command` | 10.2 |
| `MeterView`, `TransportView`, `SlotMeasure` | `duet-dsp`, `duet-command` | 7.3, 15.5 |
| `EngineState`, `EngineFault`, `FaultCode`, `FailureSurface`, `UpstreamFailure` | `duet-command` | 12.4, 15.5 |
| The stdio server and the shutdown choke point | `duet-agent` | 9.2, 9.5 |

### Declared by this chunk, in `crates/bc_app/duet/lang_rust/src/shell/view_state.rs` support

`ProjectView` is the app-side view of the persisted state. Chunk K6 writes `ViewStateStore`; this
chunk writes `ProjectView` and the round-trip test, which architecture section 10.2 states.

```rust
/// The app-side view of the persisted state. It holds GPUI values and it
/// never crosses the transport.
#[derive(Debug, Clone)]
pub(crate) struct ProjectView { state: ViewState }

impl ProjectView {
    /// Read the state back from the core, through `Verb::ViewGet`.
    pub(crate) fn load(state: ViewState) -> Self;

    /// Clamp every stored length against the current window, then apply.
    #[must_use]
    pub(crate) fn clamped(&self, window: Bounds<Pixels>) -> Self;

    /// The value `Verb::ViewSet` carries back to the core.
    #[must_use]
    pub(crate) fn to_state(&self) -> ViewState;
}
```

## Steps

1. Read `crates/bc_app/duet/lang_rust/src/main.rs`, `crates/bc_app/duet/lang_rust/src/app.rs` and `crates/bc_app/duet/lang_rust/Cargo.toml`. Confirm
   that `main.rs` holds the four functions named above and that `app.rs` holds the counter skeleton.
   Confirm that the manifest already carries `gpui-kit`, `tracing` and `tracing-subscriber`, and the
   `gpui-kit` `test-support` dev-dependency. Stop and report a discrepancy.
2. Add the internal `{ workspace = true }` entries to `crates/bc_app/duet/lang_rust/Cargo.toml`: `duet-time`,
   `duet-dsp`, `duet-score`, `duet-session`, `duet-command`, `duet-engrave`, `duet-analysis`,
   `duet-core`, `duet-agent`. Add the third-party entries this chunk first uses: `clap`,
   `arrayvec`, `async-channel`, `futures`, `tokio`. Architecture section 1.2 lists exactly those for
   this crate.
3. Run `cargo build --workspace` and commit the `Cargo.lock` it produces (SM5 rule 3).
4. Create every file of the Files table. Give each one its `//!` module documentation. Add the `mod`
   lines: `src/main.rs` gains one per top-level module, `src/element.rs` gains eleven,
   `src/shell.rs` gains twenty, and `src/{compose,record,mix,master}.rs` gain four, four, four and
   three.
5. Write the failing test `shell_frame_order_reads_every_publication` in `src/shell/core_host.rs`,
   inside a `#[cfg(test)] mod tests`, as a `#[gpui_kit::test]` UI test. Its signature is
   `fn shell_frame_order_reads_every_publication(cx: &mut TestAppContext)` and its first statement is
   `cx.update(gpui_kit::init);`.
6. Run `cargo nextest run -p duet -E 'test(shell)' --no-tests=fail` and confirm that it fails.
7. Write `src/tokens.rs`. Resolve all 27 fields from `cx.theme()` and the contract 7.2 derivation
   table. Apply the contrast check of contract 7.3 rule 6 inside `resolve`. Write
   `impl Global for DuetTokens {}`.
8. Write `src/shell/core_host.rs`. `poll_frame` calls `MeterReader::poll` with the milliseconds
   since its own last call, then `TransportReader::poll`, then it stores `frame_demand`, and it
   re-arms itself with `Context::on_next_frame` while any driver is active.
9. Register the note-entry drain inside `poll_frame`. It is one more statement that calls
   `DuetCore::drain_note_entries`, which chunk I1 wrote. `Window::on_next_frame` runs at the start of
   a frame, before the render pass of the same frame, so the drain and the paint happen in one frame
   and B10 holds (section 8.3).
10. Hold the theme observer on `CoreHost::theme`. `cx.observe_global::<gpui_kit::component::Theme>`
    returns a `Subscription`, and the closure calls `DuetTokens::resolve` and `cx.set_global`.
    Appendix A rule 3 makes a dropped subscription a defect.
11. Write `src/shell/agent_bridge.rs`. Build the runtime with
    `Builder::new_multi_thread().worker_threads(2)`, which is B38. Hold the `CancellationToken`.
    Build the B35 `async_channel` and hold both ends. Write `shut_down` as the idempotent choke
    point with its four callers, and write the `Drop` impl that absorbs every failure through a
    `tracing::warn!` with no `?`, no `unwrap` and no `expect`.
12. Write `src/shell/root.rs`. Declare `DuetApp`, `AppEvent` and `AppError`. Build every child
    entity. `DuetApp::render` calls `self.core.update(cx, |host, cx| host.poll_frame(window, cx))`
    as its FIRST statement, before it builds any child element. `Entity::update` takes
    `impl FnOnce(&mut T, &mut Context<T>) -> R`, so the argument is a two-argument closure.
13. Build every heavy sibling subtree as an `Entity::cached` child: `MixView` builds each strip as
    `strip.clone().cached(strip_style())`, `RecordView` builds each lane the same way, and
    `ComposeView` builds each system the same way. `DuetApp` is never cached.
14. Write the mode machine of section 10.1. Declare the five actions with `gpui_kit::actions!`. Bind
    the keys with `cx.bind_keys` BEFORE `cx.set_menus`, which the `gpui-kit` coding guide requires.
    Refuse to leave Record while `RecordState::Recording` holds, show a notification, and change
    nothing.
15. Write `src/shell/menu.rs`. Build the macOS menu bar with `App::set_menus` and the Linux
    in-window `AppMenuBar` from one list of `(MenuPath, Action)` pairs. A verb with no menu item is
    allowed; a menu item with no verb is not.
16. Write `src/element/toolbar.rs` with the `Toolbar` element and the overflow rule of contract 1.5:
    when the sum of the group widths passes the available width, the bar removes groups from the
    trailing end and adds them to one overflow `DropdownMenu`. `DropdownMenu` is a trait, so the
    view calls `Button::dropdown_menu` and fills the `PopupMenu` the closure receives.
17. Write `src/shell/top_bar.rs`: the `BTreeMap<Mode, Box<dyn ModeToolbar>>` registry, the width
    measurement that decides `visible`, and the four `ModeToolbar` implementations, one per file
    under `src/shell/`. Each of the four answers `group_count` zero and returns an empty
    `ToolbarGroups`.
18. Write `src/shell/transport_bar.rs` to design contract 1.8, `src/shell/mode_switcher.rs` to
    contract 1.3 and 1.4, and `src/shell/states.rs` with `WorkAreaState` and the four surfaces of
    contract section 8.
19. Write `src/shell/fault_text.rs` with the four message tables of architecture section 12.4. Every
    `FaultCode` and `FailureSurface` arm is named, because `clippy::wildcard_enum_match_arm` is
    denied.
20. Write the four work-area view seams. Each one holds `state: WorkAreaState`, a `FocusHandle` and
    its subscriptions, and renders the matching contract section 8 surface. Write the two frame
    driver seams the same way.
21. Write `src/main.rs`. Keep `init_tracing`. Add the `clap` command-line front of section 9.3:
    no subcommand opens the window, `duet mcp` and `duet serve` open a bundle with the dummy backend
    and never call `gpui_kit::application()`, and `duet <verb>` takes the writer lock and applies the
    verb to the files. `main` returns `std::process::ExitCode`.
22. Delete `crates/bc_app/duet/lang_rust/src/app.rs` and its `mod app;` line.
23. Write the accessibility work of product story X-14 and design contract section 9: a focus ring on
    every focusable element, a keyboard path to every top-bar group and every overflow item, the
    contrast floor check inside `DuetTokens::resolve`, a `reduce_motion` branch on every animated
    surface, a text scale that follows `cx.theme().font_size`, and one accessible name per custom
    element.
24. Write every test of the Tests section.
25. Run the Completion command and confirm that it passes.
26. Run `cargo clippy -p duet --all-targets -- -D warnings` and confirm that it is clean.
27. Commit on a branch named `chunk/k1-application-shell`.

## Tests

`crates/duet` is a binary with no library target, so every test lives in a `#[cfg(test)] mod tests`
in the same file. A user-interface test is a `#[gpui_kit::test]` with the signature
`fn name(cx: &mut TestAppContext)` and the first statement `cx.update(gpui_kit::init);`. The
`gpui-kit` `test-support` dev-dependency is already in the manifest. Every assert carries a message.

### Rung-two element test, for the one element this chunk fills

Architecture section 10.7 rung two gives every GPUI element the same three-part test: the requested
layout size, the painted quad set through `Window::painted_quads()`, and a pointer event at a point
that resolves to the element's own domain identifier.

| Test | File | What it asserts |
|---|---|---|
| `shell_toolbar_requests_its_layout_size` | `src/element/toolbar.rs` | `Toolbar` requests the height contract 1.5 states and the width its groups need. |
| `shell_toolbar_paints_one_quad_per_group_separator` | `src/element/toolbar.rs` | `Window::painted_quads()` holds one separator quad between each pair of visible groups and none after the last. |
| `shell_toolbar_resolves_a_click_to_a_menu_path` | `src/element/toolbar.rs` | A click on one control resolves to that control's `MenuPath` and to no other. |

### Rung-three user-interface behaviours this chunk owns

Architecture section 10.7 rung three names the behaviours that no pure crate reaches. This chunk
owns items 6 and 12.

| Test | Rung-three item | File | What it asserts |
|---|---|---|---|
| `shell_keeps_every_subscription_after_a_re_render` | 6 | `src/shell/core_host.rs` | The view stores its `Subscription` and the event still arrives after a re-render. |
| `shell_theme_observer_re_resolves_the_tokens` | 6 | `src/shell/core_host.rs` | It resolves `DuetTokens`, changes `cx.theme().mode`, runs one frame, and asserts that `cx.global::<DuetTokens>()` answers the dark value. |
| `shell_frame_order_reads_every_publication` | 12 | `src/shell/core_host.rs` | It publishes one of EVERY audio-written publication of the section 5.8 snapshot table, which is a `MeterSnapshot`, a `TransportSnapshot` and a `SlotMeterSnapshot`, runs one frame, and asserts that every reader answered the values of that same frame. It then publishes a second set, runs one more frame, and asserts the same thing again. |

### The remaining tests of this chunk

| Test | File | What it asserts |
|---|---|---|
| `shell_render_polls_the_frame_before_any_child` | `src/shell/root.rs` | `DuetApp::render` calls `poll_frame` as its first statement, before it builds any child element (section 10.2 step 1). |
| `shell_refuses_to_leave_record_while_recording` | `src/shell/root.rs` | The mode action shows a notification and leaves `Mode::Record` in place. |
| `shell_mode_action_emits_mode_changed` | `src/shell/root.rs` | Each of the four mode actions emits `AppEvent::ModeChanged(Mode)` exactly once. |
| `shell_clear_clip_dispatches_one_meter_reset` | `src/shell/root.rs` | `ClearClip` dispatches `Verb::MeterReset { strip: None }`, and a click on one meter dispatches the same verb with that strip. |
| `shell_top_bar_moves_trailing_groups_into_the_overflow` | `src/shell/top_bar.rs` | At a width that fits three of five groups, `visible` is three and the overflow menu holds the last two, in the fixed removal order (contract 1.5). |
| `shell_top_bar_registry_holds_four_modes` | `src/shell/top_bar.rs` | The registry holds one entry per `Mode` arm, and each one answers `group_count` zero at this phase. |
| `shell_menu_item_has_a_verb` | `src/shell/menu.rs` | Every `(MenuPath, Action)` pair maps to one action that reaches one `Verb`. A menu item with no verb fails the test. |
| `shell_menu_binds_keys_before_it_sets_menus` | `src/shell/menu.rs` | The recorded call order is `cx.bind_keys` then `cx.set_menus`, so every item shows its shortcut. |
| `shell_states_render_every_surface` | `src/shell/states.rs` | Each of the five `WorkAreaState` arms renders its own surface, and only `Ready` renders the work-area content. |
| `shell_fault_text_names_every_code` | `src/shell/fault_text.rs` | Every `FaultCode` arm and every `FailureSurface` arm produces one message, and no arm falls to a wildcard. |
| `shell_agent_bridge_shut_down_is_idempotent` | `src/shell/agent_bridge.rs` | A second call answers `ShutdownError::AlreadyRun`, and the `Drop` path absorbs the refusal. |
| `shell_tokens_resolve_meets_the_contrast_floor` | `src/tokens.rs` | Every token the contract marks `own` meets the contrast floor of contract 9.1 after `resolve`, in light and in dark. |
| `shell_tokens_derive_the_two_waveform_roles` | `src/tokens.rs` | `wave_fill` and `wave_rms` derive from `part`, so 27 fields and two methods answer the 29 ids of contract 7.2. |
| `view_round_trip_preserves_every_mode_view` | `src/shell/view_state.rs` | A `ViewState` written through `Verb::ViewSet` and read back through `Verb::ViewGet` is equal, including one `ModeView` per `Mode` and the window geometry (PR X-08, section 10.2). |
| `view_round_trip_clamps_against_a_smaller_window` | `src/shell/view_state.rs` | `ProjectView::clamped` bounds every stored length against the current window, so a project saved on a large display opens usable on a small one (contract 1.6). |

## Verification

1. `cargo nextest run -p duet -E 'test(shell) + test(view_round_trip)' --no-tests=fail` passes and
   prints no `no tests to run` line.
2. `cargo nextest run -p duet --no-tests=fail` passes.
3. `cargo clippy -p duet --all-targets -- -D warnings` prints no warning.
4. `cargo build --workspace` leaves `Cargo.lock` unchanged after the commit.
5. `git status` shows `crates/bc_app/duet/lang_rust/src/app.rs` as deleted.
6. One commit on a branch named `chunk/k1-application-shell`. The native git hook runs
   `scripts/dod.sh`. Quote the command output before any claim of success, per the
   `verification-before-completion` skill.

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
