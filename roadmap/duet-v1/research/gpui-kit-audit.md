# GPUI Kit 0.6.4 capability audit

Audit date: 2026-09-20. Source of truth: the crate sources under `~/.cargo/registry/src/*/` for `gpui-kit-0.6.4`, `gpui-component-0.6.4`, `gpui-base-0.6.4`, `gpui-pre-0.3.5`, `gpui-pre-apple-0.3.5`, `gpui-pre-linux-0.3.5`, `gpui-pre-wgpu-0.3.5`, `gpui-pre-scheduler-0.3.5`. Line numbers refer to those files. Nothing here is from documentation alone.

## 1. Custom vector drawing: YES

- `canvas()` and `Canvas<T>` in `gpui-pre-0.3.5/src/elements/canvas.rs:10`. The paint callback gets `Bounds<Pixels>`, `&mut Window`, `&mut App`.
- `PathBuilder` in `gpui-pre-0.3.5/src/path_builder.rs:25`: `move_to`, `line_to`, `curve_to`, `cubic_bezier_to`, `arc_to`, `relative_arc_to`, `add_polygon`, `close`, `transform`, `translate`, `scale`, `rotate`, `dash_array`. Constructors `PathBuilder::fill()` and `PathBuilder::stroke(width)`. Re-exports lyon `FillOptions`, `FillRule`, `StrokeOptions`, `Transform`.
- Window paint primitives in `gpui-pre-0.3.5/src/window.rs`: `paint_path` (:4457), `paint_quad` (:4386), `paint_svg` (:4706), `paint_image` (:4776), `paint_glyph` (:4544), `paint_emoji`, `paint_underline`, `paint_strikethrough`, `paint_surface` (:4878), `paint_layer` (:4235).
- The `Element` trait is public in `gpui-pre-0.3.5/src/element.rs`.
- Cost: `Scene::batches` (`scene.rs:172`) groups adjacent paths into one `PrimitiveBatch::Paths`. The Metal renderer draws one batch in one pass (`metal_renderer.rs:750`, `:904`). The example `examples/paths_bench.rs` paints 2000 paths per frame.
- Reuse pattern: `PathCache` in `gpui-component-0.6.4/src/plot/path_cache.rs`.
- Caution: `BatchIterator::next` (`scene.rs:291`) splits a batch when the draw order alternates between paths, quads, and glyphs. Paint all staff paths together, then all glyphs.

## 2. Text and fonts: YES

- Runtime font load: `TextSystem::add_fonts(Vec<Cow<'static,[u8]>>)` at `text_system.rs:98`. Example in `examples/example_support/fonts.rs`. A SMuFL font such as Bravura loads from bytes.
- Single glyph at any position and size: `Window::paint_glyph(origin, font_id, glyph_id, font_size, color)` at `window.rs:4544`.
- Glyph ids and offsets: `WindowTextSystem::layout_line` (`text_system.rs:657`) returns `Arc<LineLayout>` with `ShapedRun { font_id, glyphs }` and `ShapedGlyph { id, position, index, is_emoji }`.
- Whole-line paint: `shape_line`, `shape_text`, `ShapedLine::paint`, `WrappedLine`. Metrics: `typographic_bounds`, `advance`. `TextRun`, `Font`, `StyledText`, `InteractiveText`.

## 3. Animation: YES

- `Animation` in `elements/animation.rs:15` with `repeat`, `repeat_synced`, `with_easing`. `AnimationExt::with_animation`, `with_animations`. Easing: `linear`, `ease_in_out`, `ease_out_quint`, `bounce`, `pulsating_between`. Springs in `src/spring.rs`.
- `Window::request_animation_frame()` at `window.rs:2606`, `on_next_frame` (:2586), `refresh` (:2248).
- `Transition` and `MotionValue<T>` in `gpui-base-0.6.4/src/motion.rs`, plus `motion/{easing,keyframes,presence,reveal,sequence,stagger,timing}.rs`. Reachable as `gpui_kit::base::motion`.
- A view drives 60 Hz with `window.request_animation_frame()` inside `render`. Proof: `examples/paths_bench.rs:61`. `WindowOptions::inactive_frame_interval` throttles an inactive window.

## 4. Mouse and keyboard: PARTIAL

- `MouseDownEvent { button, position, modifiers, click_count, first_mouse }` at `interactive.rs:148`. `KeyDownEvent { keystroke, is_held, prefer_character_input }`, `KeyUpEvent`, `ModifiersChangedEvent`, `ScrollWheelEvent`.
- `Keystroke { modifiers, key, key_char }`. `Modifiers { control, alt, shift, platform, function }`. `Window::modifiers()` at `window.rs:3121`, `Window::mouse_position()`.
- Listeners on `div`: `on_mouse_down`, `capture_any_mouse_down`, `on_mouse_down_out`, `on_mouse_move`, `on_scroll_wheel`, `on_key_down`, `on_key_up`, `on_modifiers_changed`. Drag: `on_drag`, `on_drag_move`, `DragMoveEvent<T>`, `drag_over`, `on_drop`, `can_drop`.
- GAP: no API reports the set of held letter keys. Only modifiers have a live query. The staff view must hold a focus handle and track `on_key_down` and `on_key_up` itself to know that `w` is held at click time.

## 5. Scroll and virtualization: PARTIAL

- `ScrollHandle` (`div.rs:4245`) with `offset`, `set_offset`, `max_offset`, `bounds`, `bounds_for_item`, `scroll_to_item`. `overflow_scroll`, `overflow_x_scroll`, `overflow_y_scroll`, `track_scroll`.
- `uniform_list`, `UniformListScrollHandle`. `v_virtual_list`, `h_virtual_list`, `VirtualList`, `VirtualListScrollHandle` in `gpui-base-0.6.4/src/virtual_list.rs`. `Scrollbar`, `auto_scroll.rs`, `scroll_bounce.rs`.
- Vertical slider: `Slider::vertical()` in `gpui-component-0.6.4/src/slider.rs:113`.
- GAP: no timeline, waveform, gantt, knob, or level meter component. Charts: area, bar, candlestick, line, pie, radar, sankey. Nearest meters: `Progress`, `ProgressCircle`.

## 6. Images and GPU: PARTIAL

- `img()`, `ImageSource` (`Resource`, `Render(Arc<RenderImage>)`, `Image`, `Custom`). `RenderImage` holds BGRA frames. Per-frame bitmap update: `paint_image` then `Window::drop_image` (:4894) for the old one, or the atlas grows without limit.
- `Surface` element and `paint_surface` are macOS only.
- Renderer: macOS `MetalRenderer` (`gpui-pre-apple-0.3.5/src/metal_renderer.rs:112`). Linux `WgpuRenderer` (`gpui-pre-wgpu-0.3.5/src/wgpu_renderer.rs`) for both X11 and Wayland. No Blade code.
- GAP: `Primitive` (`scene.rs:222`) is a closed enum. No custom shader hook, no external texture hand-off. On Linux there is no `Surface` element.

## 7. Native window interop: PARTIAL

- `impl HasWindowHandle for Window` (`window.rs:7199`) and `HasDisplayHandle`. macOS gives an `AppKitWindowHandle` from the NSView; Linux gives `WaylandWindowHandle` or `XcbWindowHandle`.
- `App::open_window` (`app.rs:1283`). `WindowKind`: `Normal`, `PopUp`, `AnchoredPopup`, `Floating`, `Dialog`, `LayerShell` (Wayland). macOS applies `NSFloatingWindowLevel`, `NSPopUpWindowLevel`, and `beginSheet:`. `PopupOptions.parent`.
- GAP: no public API adds a native child view, and no element reserves a rectangle for one. Not needed while plugins are out of scope.

## 8. Layout: YES

- `h_resizable`, `v_resizable`, `resizable_panel`, `ResizablePanelGroup`, `ResizableState`. `Tabs`, `Tab`, `TabBar`. `Sidebar`. `TitleBar`, `StatusBar`.
- Dock and panel system: `DockArea`, `Panel` trait, `DockAreaState`, `tab_group.rs`, `registry.rs`, `drag.rs` in `gpui-base-0.6.4/src/dock/`; `Dock`, `TabPanel`, `Panel` in `gpui-component-0.6.4/src/dock/`.
- `Collapsible`, `Accordion`, `AccordionItem`.

## 9. Menus: PARTIAL

- `PopupMenu`, `ContextMenu<E>` and `ContextMenuExt` (right click), `DropdownMenuPopover<T>`, `MenuItem`, `AppMenuBar`. `NativeMenu` with macOS and Windows back ends. macOS menu bar: `App::set_menus`, `set_dock_menu`, `Menu`, `MenuItem`; example `examples/set_menus.rs`. `ButtonGroup`, `ToggleGroup`.
- GAP: no `Toolbar` component. Build one from `h_flex`, `ButtonGroup`, `Separator`.

## 10. Threads: YES

- `BackgroundExecutor` with `spawn`, `spawn_with_priority`, `timer`, `num_cpus`, `scoped`. `ForegroundExecutor` with `spawn`, `spawn_when_idle`, `block_on`, `block_with_timeout`. `App::background_spawn`, `AsyncApp::background_executor`.
- A real thread pool: `PlatformScheduler` over `PlatformDispatcher`; macOS posts to Grand Central Dispatch queues.
- Real-time audio is a first-class case: `Priority::RealtimeAudio` (`gpui-pre-scheduler-0.3.5/src/scheduler.rs:32`) starts a dedicated OS thread. `BackgroundExecutor::spawn_realtime`, `PlatformDispatcher::spawn_realtime`; the macOS body calls `set_audio_thread_priority()`. Also `spawn_dedicated` and `DedicatedExecutor`.
- Runtime: neither tokio nor smol. GPUI has its own executor on `async-task`. Channel crates in the tree: `futures`, `postage`, `async-channel`, `flume`.

## 11. Platform split: YES

- Features of `gpui-pre-0.3.5`: `default = ["font-kit", "wayland", "x11", "windows-manifest"]`, plus `screen-capture`, `test-support`, `inspector`, `profiler`, `bench`, `leak-detection`, `stacker`.
- `cfg(target_os)` gates the source. `gpui_platform::current_platform(headless)` picks the platform. Linux: `gpui_linux::current_platform` calls `guess_compositor()` and selects Wayland, X11, or headless.

## 12. Tests: PARTIAL

- `#[gpui_kit::test]` via `gpui-kit-0.6.4/src/lib.rs:95`. `TestAppContext`, `VisualTestContext` with `simulate_click`, `simulate_keystrokes`, `simulate_input`, `simulate_event`, `simulate_resize`, `draw`.
- Kit helpers: `TestWindowExt` (`gpui-kit-0.6.4/src/test.rs:25`): `find`, `try_find`, `within`, `render_frame`, `click`, `click_at`, `right_click`, `double_click`, `hover`, `scroll`, `drag`, `drag_to`, `press`, `input`. `ElementSnapshot`. Guide: `gpui-kit-0.6.4/TESTING.md`.
- `Window::painted_quads()` returns the quads of the last frame. `Window::render_to_image` and `capture_screenshot` exist on macOS only, and those tests are `#[ignore]` by default.
- GAP: no `painted_paths()` and no `painted_glyphs()`. A custom element's path and glyph output is not assertable. Mitigation: keep layout math in pure crates and test it there; the GPUI element paints a placement list.

## Skill defect

`.claude/skills/gpui-kit/SKILL.md` links about 15 files under `references/gpui/` that do not exist in this checkout. The directory holds only `coding-guides.md`, `conventions.md`, `recipes.md`, and `usage.md`.

## Highest-risk gaps

Music notation:
1. No test hook for painted paths or glyphs.
2. Batch splits when the draw order alternates primitive kinds.
3. No SMuFL metadata reader; anchors and engraving defaults come from our own parse of the font metadata file.
4. `PathBuilder::build()` tessellates on the CPU each call. Cache the built `Path<Pixels>` per symbol.
5. No text-on-path and no path boolean operations.

Waveform timelines and meters at 60 Hz:
1. No waveform, timeline, knob, or level meter component.
2. `request_animation_frame` marks the whole view dirty. Keep the playhead in a small child view or its own canvas.
3. A per-frame `RenderImage` gets a new atlas entry each frame; call `drop_image` on the old one.
4. Pre-tessellate and cache waveform peak paths per zoom level.
5. A zoomed virtual list needs our own width model.
