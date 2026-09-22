# ADR-0006: The wrapped timeline for Compose and Record

## Status

Proposed.

## Context

In Compose the user reads a score that wraps into systems. In Record an audio lane appears under each line of the score, so the user sees notes and audio together (shared brief).

One fact decides the whole surface. **Note placement is not linear in time.** An engraver places a note by its duration class and by optical rules. A half note is therefore not twice the width of a quarter note. Equal time spans therefore get unequal widths (critic 7.1).

Ardour's waveform rule is the opposite. Its header states that x equals zero is `region->start()` samples into the source and x equals N is `N * samples_per_pixel` further on (research section 9). That rule cannot hold under a staff.

Three more facts follow from it.

1. A tempo change inside a system does not move x at all, because placement follows duration, not elapsed time. Only the sample axis changes, so samples per pixel differs on each side of the tempo point (critic 7.2).
2. A take that crosses a line break must not split as a region. A model split would let a layout pass edit user data (critic 7.3).
3. Zoom in a wrapped view is staff size, not samples per pixel. A zoom step relayouts the score and moves every line break. A pixel cache would be rebuilt on every step, and it would never pay for itself (critic 7.4).

The GPUI Kit audit adds two constraints. No waveform or timeline component exists (audit 5). `VirtualList` needs our own width model (audit, highest-risk gaps).

## Decision

Every number this record relies on is a row of specification section 1.6, cited by its id. Every
rule is cited by its id. This record carries decisions and consequences only.

1. **Compose and Record align the audio lane with the notes, not with a uniform time axis.** The product requirement is that the user sees notes and audio together. A waveform that does not sit under its note is worse than no waveform.
2. **The engraver produces one `SystemXMap` per system, beside the glyph placements.** It is the one function that turns a beat position into an x offset inside a system, and the one function that turns an x offset back. `SystemXMap` and `Knot` are declared once, in specification section 10.5, and design contract 3.1 uses the same two method names, so no document carries a variant of them.

3. **A sample reaches x through composed maps, never through a stored scale.** The device edge gives a superclock, the `TempoMap` gives ticks, and the `SystemXMap` gives x. **No samples-per-pixel value is ever cached, for a system or for a segment.**
4. **`SystemXMap::beat_for_x` is the hit-test path.** A click in the staff or in the lane resolves to a beat position with the same function, so the two surfaces cannot disagree.
5. **Layout produces derived paint segments. The model never splits, and a segment carries no scale of its own.** A region keeps the invariant of specification section 6.1 over an immutable source. A take that crosses a line break becomes several segments in one pass. A take that starts before the system start clips at the boundary.

The first draft gave a segment a tick range, an x range, and a frame range. Two endpoints encode one linear map from x to frames inside the segment, which is the per-system samples-per-pixel defect at a finer grain. This decision removes it.

**`TakeSegment` is declared once, in specification section 10.5.** `crates/duet` is the lowest crate that reaches both the region identity and the system map, so the type lives there. Every x comes from `SystemXMap::x_for_beat`, and every frame comes from the `TempoMap` composed with the region start. A tempo change inside a segment is therefore exact with no knot and no split, and the frame lookup runs per column rather than per endpoint.

A test covers it. It builds one system with a tempo change in its middle and one take that spans the change. It asserts that the frame at each sampled column matches an independent reference from the tempo map.

6. **Two caches exist, and they are different things.**
   - **A pixel bitmap cache is rejected for the wrapped view.** A zoom step relayouts the score and moves every line break. A bitmap would be rebuilt on every step, and it would never pay for itself. The audit also warns that a per-frame `RenderImage` grows the texture atlas without limit.
   - **A tessellated `Path<Pixels>` cache is adopted.** The audit records that `PathBuilder::build()` tessellates on the central processor on each call and advises a cached path. Design contract 3.3 requires the cache. The first draft rejected the bitmap and the path together, which contradicted both.

   **The cache key carries the system**, because a region that runs past one system appears on several systems and each appearance has its own path. A key without the system would return the first system's shape for every lane. Specification section 10.5 holds both keys and every document that states them, so no document holds a variant.

   **The cache has one owner per VIEW that paints a timeline, and there are THREE** (critic C21-W6). `RecordView` owns the wrapped one, `MixView` owns a linear one, and `MasterView` owns a linear one, which is what decision 8 states. **Compose owns none.** Specification section 10.2 gives Compose to `ComposeView`, which owns no `LaneView` and paints no waveform, so Compose holds no tessellated path to cache; it also makes `MixView` and `MasterView` siblings with no edge, so a shared cache between them cannot be built. Revision 21 wrote that `MixView` owns it in Mix and Master, which decision 8 contradicts four decisions later and which DR1 forbids, and a reader who followed this sentence gave Master no cache at all. The type is `PathCache`, declared in `crates/duet`.

   **The bound is derived from the visible working set, not chosen.** At the product budget B1, with the systems that fit one window at the contract 1.1 default of B91 by B92 and the stacked take strips of contract 3.4, the visible set is B62. A cache smaller than its working set evicts and rebuilds every frame, which is the tessellation cost the cache exists to remove. The cache therefore holds B61, evicted least recently used. Both halves of B61 are needed: a count alone does not bound memory.

   It is invalidated when the region changes, when the engrave revision changes, or when the zoom step changes. A scroll reuses the path; a zoom rebuilds it.

   **On a cache miss the lane draws, it never waits.** It paints the absent treatment of design contract 3.3 and requests a build on a background task. The user interface thread never waits for a tessellation or for a disk read, which is the rule Ardour states for its own waveform cache.
7. **The wrapped view caches a peak pyramid per source, under the path cache.** `duet-dsp` owns the format, because `duet-engine` writes it during a record pass and `duet-analysis` reads it afterwards. A path is built from the pyramid and then cached. Specification section 5.10 holds the level layout at B68 and the file that holds it.
8. **Mix and Master use a linear timeline with one samples-per-pixel scalar, and **each view owns its OWN path cache**: `MixView` holds one and `MasterView` holds one, with the same key type. A GPUI child reaches a sibling only through its parent, and the two are siblings under `DuetApp`, so one shared cache would need a third owner and a notification path between two views that never render in the same frame. Specification section 10.5 states the arithmetic and B61 bounds each cache (critic C16-15, C17-3).** There is no system in a linear timeline, so a `SystemId` in that key would be a constant that wastes a slot per region. **`ZoomScalar` is a declared `duet-command` type**, which `ZoomStep::scalar` produces and which no file persists, so the two modes read one stored zoom value per mode and cannot disagree with it. **No pixel bitmap cache exists in any mode.** Specification section 10.2 declares the scalar, and specification section 10.5 holds both keys in one table.
9. **Virtualization is a `gpui_kit::component::VirtualList` over systems, owned by the work-area view.** Each system reports its own height and width from the size the layout output carries, which is the width model the audit says we must supply. Specification section 10.3 names the two owners, the item, the identity source, and the two chunks that write it; this record cites that section rather than repeating it (DR6).
10. **Every axis-aligned rectangle is a quad, and the draw order per system is quads, then paths, then glyphs.** Specification section 10.4 holds the mark-by-mark table and the two gains that follow.
11. **The paint element computes nothing.** Its input is an absolute placement list in pixels, and `paint` adds only the element bounds origin.

## Consequences

Easier:

- A waveform sits exactly under the note it belongs to, at every tempo and at every staff size. That is the feature.
- Hit testing in the staff and in the lane uses one function, so a click cannot select one thing and show another.
- A line-break change edits no user data, because a segment is derived and a region is not.
- The layout output is a pure value, so `duet-engrave` carries the assertions and no screenshot is needed.
- The peak pyramid survives a zoom step, so a zoom does not rebuild a cache.

Harder:

- The waveform stretches and compresses across a system. It is correct for alignment and unusual to look at. The design review of rung three judges it, and the Designer states the visual treatment.
- Scroll and zoom are per system, not one scalar. The `VirtualList` needs a width and a height per system from the layout output.
- A path is built once per segment per zoom step and then cached, rather than blitted from a cached image. The `criterion` bench holds the build to the frame budget, and B61 keeps the cache memory known.
- A segment holds a shared reference to the system's map, so a layout result is shared and not copied per segment. The map is immutable once the engrave step publishes it.
- Two draw paths exist, one wrapped and one linear. Each has its own cache policy, and the specification names both so that neither is used in the wrong mode.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| A proportional staff, so that x is linear in time and the lane is trivial | Proportional notation is hard to sight-read. The product's first user is a singer reading a page. |
| A uniform samples-per-pixel lane under a conventionally spaced staff | The waveform and the notes agree only at the system start. That defeats the feature that the whole Record mode exists for. |
| Two independent timelines side by side | The product requirement is that the user sees notes and audio together on one line. |
| One samples-per-pixel value per system | A tempo change inside a system makes it wrong on one side of the tempo point. The two composed maps are correct everywhere. |
| A per-segment x range and frame range, as the first draft used | Two endpoints encode one linear map inside the segment, which is the same defect at a finer grain. A segment now holds a beat span and the map; specification section 10.5 declares the whole field list, and this record states none of it (concern N15-12). |
| Break a segment at every knot and every tempo point, so the endpoints are exact | It works, and it makes the segment count depend on the tempo map rather than on the line breaks. Holding the map and looking up per column is simpler and cannot go stale. |
| A path bound smaller than the visible working set, with no memory bound | It would evict and rebuild every frame, and a count alone leaves memory unbounded. |
| One cache per `LaneView` | A cache on every lane gives no global bound and no shared reuse between two lanes over one source. |
| Reject the tessellated path cache with the pixel cache | The audit says `PathBuilder::build()` tessellates on each call, and design contract 3.3 requires the cache. Only the bitmap cache is wrong here. |
| Split a region at a line break in the model | A layout pass would edit user data, and a change of staff size would rewrite the arrangement. Specification section 6.1 holds the region invariant, and a derived segment cannot touch it. |
| A pixel cache in Compose and Record, as Ardour's `WaveViewCache` does | Ardour's cache works because zoom is one scalar and a scroll is a blit. Here a zoom step relayouts the score and moves every line break, so the cache would be rebuilt on every step. |
| Draw every mark as a path, including staff lines | `painted_quads()` is the only assertion hook GPUI offers, and the batch splits when the draw order alternates between primitive kinds. Quads give a test hook and fewer splits. |
| Let the element compute a position from a tick value | Any arithmetic inside `paint` is untestable forever, because no API returns painted paths or painted glyphs. |
