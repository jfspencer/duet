# Duet v1 design contract

Author: Designer. Date: 2026-09-20. Status: normative for v1.

This document is the visual and interaction contract for `crates/duet`.
An engineer builds from this document plus two skills: `gpui-kit` for the
component and element APIs, and `gpui-kit-design-guides` for the house design
rules. Load both before you write a view.

Abbreviations, expanded at first use:
- **sp** is one staff space. It is the distance between two adjacent staff
  lines. Every notation measurement in this document is in sp.
- **SMuFL** is the Standard Music Font Layout specification.
- **LUFS** is Loudness Units relative to Full Scale.
- **dBTP** is decibels True Peak.
- **LRA** is Loudness Range.
- **RMS** is Root Mean Square.

## 0. Rules that bind every section

1. **No literal color, radius, or font size in a view.** A view reads a color
   from `cx.theme()` or from the Duet token global that section 7 defines. A
   view reads a radius from `cx.theme().radius`. A literal is a review defect.
2. **No raw `px(...)` for product space.** Shell layout uses the rem helpers
   (`p_2`, `gap_3`, `h_8`, `text_sm`) and the component `Size` API. The pixel
   values in this document state the intended result at the default base font
   of 16 px. Section 9 names the audited exceptions.
3. **Staff geometry is the one exception to the rem rule.** The staff scales
   with its own zoom value, not with the interface rem. Music size is a
   document property, so a reader can enlarge the interface and keep the score
   size, or the reverse. The staff element receives `sp` as a parameter.
4. **Every state in section 8 ships with the surface.** A surface without its
   load, empty, error, and no-device state is incomplete.
5. **Color never carries meaning alone.** Section 9 lists the second cue for
   every colored state.
6. **Reduced motion is a first-class path.** `gpui_base::init` reads the system
   preference into `App::reduce_motion`. Section 6 gives the reduced path for
   every animation.

---

## 1. The shell

### 1.1 Window

- Minimum window size: 1024 x 700 px. Below that width the sidebar collapses to
  an icon rail. Below 900 px width the window refuses to shrink further.
- Default window size: 1440 x 900 px.
- Breakpoints, by window width:
  - **wide**, 1600 px and above: sidebar 280 px, inspector visible, every top
    bar group visible.
  - **standard**, 1200 px to 1599 px: sidebar 240 px, inspector visible in Mix
    and Master only.
  - **narrow**, 1024 px to 1199 px: sidebar collapses to a 48 px icon rail, the
    inspector becomes a `Sheet`, the top bar moves its low-frequency groups
    into one overflow `DropdownMenu`.
- The shell is the same in all four modes. Only the work area and the top bar
  change. Global navigation stays stable, as the design guide requires.

### 1.2 Vertical band order

From the top of the window to the bottom:

| Band | Height | Component |
|---|---|---|
| Title bar | 36 px | `TitleBar` |
| Top bar (mode options) | 44 px | custom `Toolbar`, built from `h_flex`, `ButtonGroup`, `Separator` |
| Body | fills | `h_resizable` with two or three panels |
| Transport bar | 56 px | custom `TransportBar`, built from `h_flex` |
| Status bar | 24 px | `StatusBar` |

A hairline of `cx.theme().border` sits on the lower edge of the title bar, the
lower edge of the top bar, and the upper edge of the transport bar. The status
bar owns its own upper hairline through `status_bar_border`. Two neighbor bands
never both draw the same separator.

### 1.3 Title bar

- Leading group, 12 px inset: the project name at 13 px semibold, then a
  `Badge` that shows the commit state. The badge reads `Saved`, `3 changes`, or
  `On commit a4f1c2`. A click on the badge opens the history `Sheet`.
- Center group: the mode switcher.
- Trailing group, 12 px inset: an agent state `Button` with a 16 px icon, then
  the window controls that the platform owns.

### 1.4 Mode switcher

- Component: `ToggleGroup` in the segmented style, on the `tab_bar_segmented`
  surface.
- Four segments in fixed order: `Compose`, `Record`, `Mix`, `Master`.
- Geometry: total width 336 px, height 28 px, segment width 84 px, radius
  `cx.theme().radius`, inner inset 2 px.
- The selected segment gets the `tab_active` fill, `tab_active_foreground`
  text, and a 1 px `border`. Hover on an unselected segment raises it to
  `list_hover`. The selected state persists and does not depend on hover.
- Shortcuts: `cmd-1` to `cmd-4` on macOS, `ctrl-1` to `ctrl-4` on Linux. Each
  segment carries a tooltip that names its shortcut.
- The switcher never reorders. The order is the production order of the
  product, so the user reads a left-to-right progress line.

### 1.5 Top bar

- Height 44 px, horizontal inset 12 px, vertical center alignment.
- A group is an `h_flex` with an 8 px inner gap. Two groups are separated by a
  `Separator` of 1 px with a 12 px gap on each side.
- Every control in the bar uses the `small` size, which gives a 28 px frame.
- Overflow rule: when the sum of the group widths passes the available width,
  the bar removes groups from the trailing end and adds them to one overflow
  `DropdownMenu` with an icon of three dots. The removal order is fixed per
  mode and is listed in the mode sections. No action disappears without a menu
  path, as the design guide requires.

### 1.6 Body

- Component: `h_resizable` with `resizable_panel` children and a persisted
  `ResizableState`.
- Panels:
  1. **Sidebar** (`Sidebar` component): minimum 180 px, preferred 240 px,
     maximum 360 px. Collapses to a 48 px icon rail.
  2. **Work area**: `flex_1()` with `min_w_0()`. Minimum 560 px. This panel
     always wins the surplus space.
  3. **Inspector**: minimum 240 px, preferred 300 px, maximum 420 px. Hidden in
     Compose and Record by default. Visible in Mix and Master.
- Duet persists the split per project and per mode. On restore, Duet clamps
  each value against the current window width before it applies the value.
- Mix adds a `v_resizable` inside the work area. The top child is the linear
  timeline, minimum 200 px, preferred 45 percent. The bottom child is the
  mixer strip row, minimum 320 px, preferred 55 percent.

### 1.7 Sidebar: parts, tracks, and takes

- Component: `Tree` inside `Sidebar`, with `SidebarHeader` and `SidebarGroup`.
- Three levels: part, track, take.
- Row height 28 px. Indent step 16 px. Disclosure arrow 16 px in a fixed slot.
- Row layout, leading to trailing:
  1. Part color chip: 3 px wide, 20 px tall, radius 2 px, at the leading edge.
     The chip holds no text. The part row beside it starts with the part letter
     in a 16 px square: `S`, `A`, `T`, or `B`.
  2. Name: 13 px, one line, truncated at the end, with a tooltip that gives the
     full name.
  3. A fixed trailing lane of 84 px that holds three 24 px square buttons:
     arm, mute, solo. The lane is always present, so a row without buttons
     does not move the name of the row above it.
- Selection uses `list_active` with a 2 px leading bar in `sidebar_primary`.
  Hover uses `list_hover`. Focus draws a 2 px `ring` outline, inset 1 px.
- The icon rail at narrow width shows four part letters as 40 px square
  buttons. A click expands the sidebar back to its preferred width.

### 1.8 Transport bar

Three groups on one center line at 56 px height.

**Leading group**, 16 px inset:
- Record mode `Select`, width 132 px: `Layer takes`, `Replace`, `Sound on
  sound`.
- Punch `Button` with a toggle state, 28 px. Its pressed state shows the punch
  range in bars beside it, in 12 px tabular text.
- Loop `Button` with a toggle state, 28 px.

**Center group**, centered on the window:
- Five icon buttons, each 32 px square, 6 px gap: go to start, rewind, play or
  pause, stop, record. Every button has a tooltip with a shortcut.
- The record button paints the `duet.record.red` token as its icon fill. When a
  track is armed, the button gains a 2 px ring of the same color. During a
  record pass the button fills solid and its icon turns to a square.
- Time readout: a two-line block, 160 px wide, monospace, tabular numerals.
  Line one is the primary readout at 20 px. Line two is the secondary readout
  at 11 px in `muted_foreground`. A click swaps the two. The two formats are
  `bar.beat.tick` and `h:mm:ss.mmm`.

**Trailing group**, 16 px inset:
- Tempo: a `NumberInput` 72 px wide with the unit `BPM` as a suffix label.
- Time signature: a `Select` 72 px wide.
- Metronome: a 28 px toggle `Button`.
- Master output level: a 120 px wide horizontal `LevelMeter`, 10 px tall, with
  a peak number to its trailing side, 40 px, tabular.

### 1.9 Status bar

`StatusBar` at 24 px, 11 px text, `muted_foreground` by default.

- Leading: the audio device name, the sample rate, and the buffer size, joined
  by a middle dot. A click opens audio settings.
- Center: the MIDI device name, or the tag `No MIDI device`.
- Trailing: a compact engine load bar of 60 px, the disk state, and an agent
  state item. The agent item shows a 12 px `Spinner` while the agent works.
- A warning state in any item paints the text in `warning` and adds a 12 px
  icon. This is the single place where Duet reports a device problem, so no
  mode repeats a device banner except Record, which cannot proceed.

### 1.10 The four work areas

| Mode | Work area content | Inspector |
|---|---|---|
| Compose | Wrapped score. Systems stack down the page. | Hidden. `Sheet` on demand for note properties. |
| Record | Wrapped score plus one audio lane block under each system. | Hidden. `Sheet` on demand for take properties. |
| Mix | `v_resizable`: linear timeline above, mixer strip row below. | Visible. The selected strip's full chain. |
| Master | Linear timeline above, master chain below. | Visible. The loudness panel. |

### 1.11 Custom elements Duet builds

The audit confirms that GPUI Kit has no component for any item in this list.
Each item is a Duet element or a Duet view over `canvas()` and `PathBuilder`.

| Element | Purpose | Base API |
|---|---|---|
| `StaffSystem` | Paints one system: lines, glyphs, stems, beams, ties. | `canvas()`, `PathBuilder`, `Window::paint_glyph` |
| `SystemXMap` | Pure map from a beat position to an x offset in a system. | Plain Rust, in `duet-engrave` |
| `WaveformLane` | Paints one audio lane from cached peak paths. | `canvas()`, `PathBuilder`, `PathCache` pattern |
| `PlayheadLayer` | Paints the playhead only. Its own small view. | `canvas()`, `Window::request_animation_frame` |
| `LevelMeter` | Stateless: paints one `MeterReading` and a `held` flag as peak, RMS, and clip in one vertical or horizontal bar; `MeterLayer` owns the clip state and the numeric readout. | `canvas()` |
| `LufsMeter` | Momentary, short term, integrated, target, tolerance. | `canvas()` |
| `Fader` | Channel fader with a dB taper and detents. | `canvas()` plus GPUI drag listeners |
| `Knob` | Radial control with an arc and a pointer line. | `canvas()` |
| `AutomationLane` | Polyline and handles over a linear time axis. | `canvas()` |
| `Toolbar` | The top bar container with the overflow rule. | `h_flex`, `ButtonGroup`, `Separator` |
| `PunchRange` | The punch band and its two drag handles. | `canvas()` plus drag listeners |

Paint order rule from the audit: paint every path of a frame together, then
every glyph. A draw order that alternates path, quad, and glyph splits the
batch and costs a pass per switch.

---

## 2. Compose mode

### 2.1 The staff unit system

`sp` is the unit. Every other number derives from it.

- Default `sp` = 8.0 px. Zoom steps set `sp` to 6, 7, 8, 10, 12, and 16 px.
  Those steps read as 75, 87, 100, 125, 150, and 200 percent.
- Staff height = 4 sp. A staff has 5 lines.
- The SMuFL em square is 4 sp by specification. So a glyph paints with
  `font_size = 4.0 * sp`. At the default `sp` that is 32 px.
- Every derived value rounds to a device pixel only at paint time. Layout math
  stays in `f32` staff spaces inside `duet-engrave`.

### 2.2 Bravura as the glyph source

- Duet ships Bravura (SIL Open Font License) and `bravura_metadata.json`.
- Duet loads the font at start with `TextSystem::add_fonts`, from bytes
  embedded in the binary.
- Duet parses `bravura_metadata.json` at start and keeps three tables:
  `engravingDefaults`, `glyphAdvanceWidths`, and `glyphsWithAnchors`.
- **No stroke width, no anchor, and no advance is hard-coded.** The engraver
  reads each value from the parsed metadata. The audit names the absence of a
  SMuFL metadata reader as a risk, so the reader is a v1 deliverable in
  `duet-engrave`.
- The values below are the Bravura `engravingDefaults`. They are the expected
  result, not a substitute for the reader.

| Key | Value in sp |
|---|---|
| `staffLineThickness` | 0.13 |
| `stemThickness` | 0.12 |
| `beamThickness` | 0.50 |
| `beamSpacing` | 0.25 |
| `legerLineThickness` | 0.16 |
| `legerLineExtension` | 0.40 |
| `tieEndpointThickness` | 0.10 |
| `tieMidpointThickness` | 0.22 |
| `slurEndpointThickness` | 0.10 |
| `slurMidpointThickness` | 0.22 |
| `thinBarlineThickness` | 0.16 |
| `thickBarlineThickness` | 0.50 |
| `barlineSeparation` | 0.40 |
| `bracketThickness` | 0.50 |

- A stroke narrower than one device pixel paints at one device pixel. A staff
  line at `sp` = 6 px computes 0.78 px and paints at 1 px.
- Glyph codepoints in use: `gClef` U+E050, `fClef` U+E062, `brace` U+E000,
  `noteheadWhole` U+E0A2, `noteheadHalf` U+E0A3, `noteheadBlack` U+E0A4,
  `flag8thUp` U+E240, `flag8thDown` U+E241, `accidentalSharp` U+E262,
  `accidentalFlat` U+E260, `accidentalNatural` U+E261, `augmentationDot`
  U+E1E7, `restWhole` U+E4E3, `restQuarter` U+E4E5, `timeSig0` to `timeSig9`
  U+E080 to U+E089.

### 2.3 Staff render contract

**Lines.** Five lines, 1 sp apart, thickness `staffLineThickness`. The line
centers land on whole sp offsets from the top line.

**Grand staff.** The treble staff is above the bass staff. The gap from the
bottom line of the treble staff to the top line of the bass staff is 8 sp. A
`brace` glyph sits at the leading edge, 0.4 sp before the leading barline,
scaled to span both staves.

**System spacing.** The gap between the bottom line of one system and the top
line of the next is 12 sp in Compose mode, and never less than 9 sp after
collision correction. The gap grows to fit leger lines, lyrics, and dynamics.

**Margins.** A system starts 3 sp after the page leading edge and ends 3 sp
before the page trailing edge. The page has a 4 sp top inset and a 4 sp bottom
inset inside the scroll region.

**Clef.** The `gClef` baseline sits on the second line from the bottom of the
treble staff, which is 1 sp above the bottom line. The `fClef` two dots
straddle the fourth line from the bottom of the bass staff, which is 3 sp above
the bottom line. The clef leading edge sits 1.0 sp after the leading barline.
A clef gets 1.0 sp of clear space after its advance width.

**Key signature.** Accidentals start after the clef's clear space. Each sharp
advances 1.0 sp. Each flat advances 0.9 sp. The vertical order follows the
standard order of sharps and flats for the clef. The signature gets 1.0 sp of
clear space after its last accidental.

**Time signature.** Two glyphs stack. The numerator centers on the line 3 sp
above the bottom line. The denominator centers on the line 1 sp above the
bottom line. The pair's width is the wider of the two glyphs. The signature
gets 1.5 sp of clear space before the first note.

**Note heads.** A head paints as one glyph at `font_size = 4 sp`. `noteheadBlack`
is about 1.18 sp wide and 1.0 sp tall. A head on a line centers on that line. A
head in a space centers in that space. Pitch step size is 0.5 sp.

**Leger lines.** A leger line is 1 sp from its neighbor, at the same whole sp
steps as the staff lines. Its thickness is `legerLineThickness`. It extends
`legerLineExtension` past each edge of the head.

**Stems.** Stem thickness is `stemThickness`. The stem attaches at the
`stemUpSE` anchor for an up stem and the `stemDownNW` anchor for a down stem,
read from `glyphsWithAnchors`. Default stem length is 3.5 sp from the head
center. Rules, in order:
1. A note on or above the middle line gets a down stem. A note below the middle
   line gets an up stem.
2. In a chord, the note farthest from the middle line decides the direction.
3. When a note sits more than 1 sp outside the staff on the stem side, the stem
   extends until its free end reaches the middle line.
4. A beamed stem never falls below 2.5 sp, measured from the head center to the
   near edge of the nearest beam.
5. A whole note has no stem.

**Accidentals.** The accidental's trailing edge sits 0.16 sp before the head's
leading edge. In a chord, accidentals stack in columns from the trailing side
inward with a 0.16 sp gap between columns.

**Dots.** An augmentation dot sits 0.5 sp after the head's trailing edge, in
the space at the head's pitch. A note on a line moves its dot up 0.5 sp into
the space above. A second dot follows 0.35 sp later.

**Note spacing.** The engraver assigns each event a width from its duration:
`width(d) = 4.0 sp * (d / quarter) ^ 0.6`.
That gives a whole note 9.2 sp, a half 6.1 sp, a quarter 4.0 sp, an eighth
2.6 sp, and a sixteenth 1.7 sp. The minimum gap between two adjacent heads is
1.6 sp. The engraver then justifies each system so the last barline lands on
the trailing margin, unless the system is the last one and holds less than 60
percent of the available width.

**Barlines.** A thin barline is `thinBarlineThickness` and spans the full 4 sp
of each staff plus the 8 sp gap between them. A final barline is a thin line,
`barlineSeparation` of gap, then a thick line.

**Beams.**
1. Beam thickness is `beamThickness`. The gap between two beam edges is
   `beamSpacing`, so the pitch between two beam top edges is 0.75 sp.
2. The beam group follows the time signature's beat unit. In 4/4, eighths beam
   in groups of four across two beats. Sixteenths beam in groups of four inside
   one beat. A rest breaks the group.
3. The beam slant follows the pitch difference between the first and the last
   note of the group. The slant is capped at 2.0 sp and quantized to 0.25 sp
   steps.
4. A group whose inner notes all lie between the first and the last note keeps
   a straight beam. A group whose inner note passes outside gets a flat beam.
5. A secondary beam that covers fewer notes than the primary beam paints as a
   partial beam of 1.0 sp, on the side where the subdivision continues.

**Ties.**
1. A tie joins two heads of the same pitch. Its endpoint thickness is
   `tieEndpointThickness` and its midpoint thickness is `tieMidpointThickness`.
2. The tie starts 0.2 sp after the trailing edge of the first head and ends
   0.2 sp before the leading edge of the second head.
3. The tie curves away from the stem. A note with a down stem gets a tie above.
   A note with an up stem gets a tie below.
4. Tie height is 0.25 sp for a span below 3 sp, and grows to a maximum of
   0.5 sp at a span of 12 sp or more.
5. A tie that crosses a system break splits. The first half ends 1.0 sp after
   the last head of the system. The second half starts 1.0 sp before the first
   head of the next system.
6. The tie is a `PathBuilder::fill()` path of two cubic curves, not a stroke,
   because the thickness varies along the span.

### 2.4 Cursor, hover, and preview on the staff

**Pointer.** The default arrow cursor stays. The design guide reserves a
special cursor for a manipulation. Duet shows the target through paint, not
through a cursor change.

**Hover guides.** When the pointer sits over the staff region:
- A pitch guide paints as a 1 device pixel horizontal line at the snapped
  pitch, 2.5 sp wide, centered on the pointer x, in `duet.staff.grid`.
- A beat guide paints as a 1 device pixel vertical line at the snapped x, from
  the top line of the treble staff to the bottom line of the bass staff, in
  `duet.staff.grid`.
- A bar-and-beat tag follows the pointer 12 px above and 12 px trailing. It
  reads `12.3.0` in 11 px tabular text on a `popover` surface with 4 px inset.

**Snap.** Pitch snaps to the nearest 0.5 sp, which is one diatonic step. Time
snaps to the grid value from the top bar. The default grid is one eighth of a
beat. A held `alt` key suspends the time snap.

**Insertion preview.** The preview paints the exact glyph that the click would
insert, in `duet.staff.preview`, which is `duet.staff.ink` at 40 percent alpha.
- With no duration key held, the preview shows the duration that the top bar
  selects.
- While `w` is held, the preview shows a whole note with no stem.
- While `h` is held, the preview shows a half note with a stem.
- While `e` is held, the preview shows an eighth note with a stem and a flag.
- The preview includes the stem, the flag, the leger lines, and any accidental
  that the key signature implies.
- The audit records a gap: GPUI reports no set of held letter keys. The staff
  view holds a focus handle and tracks `on_key_down` and `on_key_up` itself.
  The view keeps one `HeldDuration` field. On focus loss the view clears the
  field, so a released key outside the window cannot leave a stuck preview.
- A change of held key updates the preview on the next frame. The preview never
  animates its own change, because it tracks the pointer.

**Click.** A left click inserts the note. The new head animates as section 6.4
defines. A click on an existing head selects it instead.

### 2.5 Selected note

- The head, the stem, the flag, the beam, and the dots paint in
  `duet.note.selected`.
- A halo paints behind the note: a rounded rectangle at `duet.note.selected_halo`,
  outset 0.35 sp from the union of the head and its dots, radius
  `cx.theme().radius`.
- The halo is a shape, so selection does not depend on color alone.
- Multi-select: `shift` plus a click adds one note. `cmd` plus a click toggles
  one note. A drag on empty staff paints a marquee with `selection` fill at 20
  percent alpha and a 1 device pixel `ring` border.
- The selection persists through a scroll, a zoom, and a mode change.
- The keyboard note cursor paints as a 2 px vertical bar of 2 sp height in
  `duet.note.cursor`, at the insert x, centered on the cursor pitch. The bar
  does not blink, because a blink competes with the playhead.

### 2.6 Context menu

Component: `ContextMenu`. A right click opens it at the pointer. `shift-F10`
and the Menu key open it at the note cursor. Every item uses the same verb,
icon, and shortcut that the top bar uses.

Order, with a `Separator` between each numbered block:

1. **Pitch**
   - `Raise semitone` (alt-up)
   - `Lower semitone` (alt-down)
   - `Raise octave` (cmd-alt-up)
   - `Lower octave` (cmd-alt-down)
   - `Respell` submenu: `Sharp`, `Flat`, `Natural`
2. **Duration**
   - `Duration` submenu: `Whole`, `Half`, `Quarter`, `Eighth`, `Sixteenth`,
     `Thirty-second`
   - `Add dot` (period)
   - `Tuplet…` (cmd-t)
3. **Attach**
   - `Tie to next` (t)
   - `Slur to next` (s)
   - `Accidental` submenu
   - `Articulation` submenu: `Staccato`, `Tenuto`, `Accent`, `Marcato`,
     `Fermata`
   - `Dynamic` submenu: `ppp` to `fff`, then `Crescendo`, `Diminuendo`
   - `Lyric…` (cmd-l)
4. **Assign**
   - `Voice` submenu: `Voice 1` to `Voice 4`
   - `Part` submenu: `Soprano`, `Alto`, `Tenor`, `Bass`
5. **Edit**
   - `Cut` (cmd-x)
   - `Copy` (cmd-c)
   - `Paste` (cmd-v)
   - `Duplicate` (cmd-d)
6. **Delete** (Delete), with the `danger` treatment.

An item that does not apply to the selection is disabled, not hidden, and its
tooltip states the reason. `Tie to next` on the last note of the score is
disabled with the reason `There is no next note`.

### 2.7 Compose top bar

Nine groups, in this order. The overflow removes groups from the trailing end,
in the order 8, 7, 6, 5, 4.

1. **Duration.** `ToggleGroup` of six 28 px buttons. Icons are the Bravura
   glyphs `noteWhole`, `noteHalfUp`, `noteQuarterUp`, `note8thUp`,
   `note16thUp`, `note32ndUp`, each at 20 px. Shortcuts `1` to `6`. A held `w`,
   `h`, or `e` overrides the group for one click and flashes the matching
   button at `list_active` for the duration of the hold.
2. **Modifier.** `ButtonGroup` of four: `Dot`, `Double dot`, `Tie`, `Tuplet…`.
3. **Accidental.** `ToggleGroup` of five: double flat, flat, natural, sharp,
   double sharp. Icons are Bravura accidental glyphs at 20 px.
4. **Articulation.** `ButtonGroup` of five: staccato, tenuto, accent, marcato,
   fermata.
5. **Dynamic.** One `DropdownMenu` button labelled `Dynamic` with a Bravura
   `dynamicForte` icon. The menu lists `ppp` to `fff`, then the two hairpins.
6. **Insert.** One `DropdownMenu` button labelled `Insert…`. The menu lists
   `Measure`, `System break`, `Key signature…`, `Time signature…`, `Clef…`,
   `Repeat`, `Rehearsal mark`.
7. **Lyrics.** One toggle `Button` labelled `Lyrics`.
8. **Grid.** A `Select` 96 px wide: `1/1`, `1/2`, `1/4`, `1/8`, `1/16`, `Off`.
9. **View.** Zoom out `Button`, a `Select` 84 px wide with the six zoom steps,
   zoom in `Button`, then a `ToggleGroup` of `Page` and `Scroll`.

**Icon family exception.** Groups 1, 3, and 5 use Bravura glyphs, not the
product icon family. The reason is that the glyph is the object that the button
inserts. Every other icon in Duet comes from the one product icon family. This
exception is documented here and nowhere else.

---

## 3. Record mode

### 3.1 The wrapped timeline

Record keeps the Compose page. Under each system, Record inserts one lane block.
The lane block holds one lane per visible track of the shown part or parts.

**The x map is shared, not copied.** The engraver publishes a `SystemXMap`
per system. The map answers `x_for_beat(beat) -> f32` and
`beat_for_x(x) -> Ticks`. A lane converts a sample position to a beat through the
tempo map, then asks the same `SystemXMap` for x. A lane never computes its own
samples-per-pixel value. This rule is the whole reason a note and its audio
line up.

A lane paints only the beat range that its system covers. The lane clips at the
system's leading and trailing margins.

### 3.2 Lane geometry

| Value | Size |
|---|---|
| Lane height, compact | 48 px |
| Lane height, default | 72 px |
| Lane height, expanded | 120 px |
| Gap between lanes in one block | 4 px |
| Gap from the bass staff bottom line to the first lane | 6 sp |
| Gap from the last lane to the next system's top line | 8 sp |
| Lane header width | 140 px |
| Lane corner radius | `cx.theme().radius` |

The user sets the height per track in the sidebar. An alt-drag on the lane's
bottom edge resizes every lane of that track, in every system, at once. The
drag handle paints 2 px and hit-tests 8 px.

### 3.3 Waveform style

- Source data is a min and max peak pair per column, in the `duet-dsp` pyramid
  format, which `duet-core` hands to the lane. One
  peak file serves every zoom level, as the Ardour concept map records.
- The lane paints one vertical bar per device pixel column. The bar spans from
  the min value to the max value, mapped to the lane height with the zero line
  at the lane center.
- Fill: `duet.wave.fill`, which is the part color at 70 percent alpha in the
  light theme and 62 percent in the dark theme.
- RMS core: a second, narrower bar at the same column, at half the height of
  the peak bar, painted over the peak bar at 100 percent alpha in
  `duet.wave.rms`.
- Zero line: 1 device pixel at `duet.wave.zero`, across the full lane width.
- Region boundary: a 1 device pixel rounded rectangle at the part color, 45
  percent alpha, radius `cx.theme().radius`. A region name sits at the leading
  edge, 10 px, `muted_foreground`, on a 2 px inset.
- A column whose peaks are not built yet paints `duet.wave.absent` with the
  shimmer of section 8. The user interface thread never waits for a disk read.
- `RecordView` caches the built `Path<Pixels>` keyed by
  `(SourceHash, RegionId, SystemId, ZoomStep)` (ADR 0006 decision 6), and every
  `LaneView` reads it. A region that spans several systems has one path per system.
  `PathBuilder::build()` tessellates on the CPU on each call, so a scroll must
  reuse the path, not rebuild it.

### 3.4 Take layers

- The active take fills the lane.
- Other takes stack below the active take as 10 px strips, at 45 percent alpha,
  with a 2 px gap. Each strip paints a reduced waveform at 1/6 the detail.
- The stack shows at most four strips. A fifth and later take collapses into a
  `Badge` that reads `+3`. A click on the badge opens the take list `Sheet`.
- A strip paints 10 px and hit-tests 20 px, so the target meets the minimum.
- A click on a strip promotes that take. The promoted waveform cross-fades with
  the current one over 160 ms, `Easing::EaseInOut`. The strips reorder with a
  120 ms move.
- A comp mode toggle in the top bar expands every take to a full lane in a
  `v_flex`. In comp mode a drag across a take selects a range, and the selected
  range paints at 100 percent alpha while the rest drops to 35 percent.
- Every take carries a name and a time. A take with a name shows the name. A
  take without a name shows `Take 3 - 14:22`.

### 3.5 Record head

- A 2 px vertical line in `duet.record.red`, with a filled triangle at the top,
  12 px wide and 8 px tall, pointed down.
- The head spans the lane block and the system above it, so the user sees the
  note position and the audio position on one line.
- While a record pass runs, new audio grows from the head at 100 percent alpha.
  Audio already on disk drops to 70 percent alpha. The boundary is the head.
- The head is part of `PlayheadLayer`, not of `WaveformLane`. The audit states
  that `request_animation_frame` marks the whole view dirty, so the head lives
  in its own small child view.
- The head never eases. It reads the transport's sample position each frame and
  paints there.

### 3.6 Punch range

- The band spans the lane block and the system above it.
- Fill `duet.record.tint`, which is the record red at 10 percent alpha in light
  and 14 percent in dark.
- Edges: 2 px vertical lines at the record red, 60 percent alpha.
- Two drag handles, one per edge. Each paints 8 px wide and hit-tests 24 px
  wide. When the two handles are closer than 48 px, the shared middle splits
  evenly, so neither handle steals the other's target.
- The bar and beat of each edge show in the transport bar beside the punch
  toggle, as `Punch 9.1 to 13.1`.
- Keyboard path: `I` sets punch in at the playhead. `O` sets punch out. The
  transport `DropdownMenu` repeats both commands with their shortcuts.
- When punch is off, the band paints at 40 percent of its alpha and the handles
  hide. The range value persists.

### 3.7 Part and track selector

Two surfaces, one model.

**Sidebar tree.** Section 1.7 defines it. This is the full list and the place
where a user renames, reorders, and sets lane height.

**Lane header.** A 140 px block at the leading edge of each lane, inside the
system margin. From leading to trailing:
1. Part color bar, 3 px wide, full lane height.
2. Part letter in a 16 px square: `S`, `A`, `T`, `B`.
3. Track name, 11 px, truncated, with a tooltip.
4. A 72 px trailing lane with three 24 px buttons: arm, mute, solo.

The arm button is a 12 px circle. At rest it is a 1.5 px ring in
`muted_foreground`. When armed it is a filled circle in `duet.record.red` with
a ring that pulses, as section 6.5 defines. During a record pass the pulse
stops and the fill stays solid, because a state that blinks during a take is a
distraction.

**Show one or show all.** A `ToggleGroup` in the Record top bar with two
segments: `One track` and `All tracks`. In `One track`, each part shows only
its active track, and the lane block holds at most four lanes. In `All tracks`,
every track of every shown part gets a lane, and the lane block scrolls if it
passes 40 percent of the window height.

**Part filter.** A `ToggleGroup` of four segments, `S`, `A`, `T`, `B`, each 32
px wide, with the part color as a 3 px underline. A segment toggles its part.
At least one part stays on.

### 3.8 Record top bar

Groups, in order. Overflow removes from the trailing end in the order 6, 5, 4.

1. **Part filter.** The four-segment `ToggleGroup`.
2. **Track view.** `One track` and `All tracks`.
3. **Take.** `Comp mode` toggle, `New take` button, `Take list…` dropdown.
4. **Edit.** `Split` (s), `Trim start` (`[`), `Trim end` (`]`), `Ripple delete`.
5. **Monitor.** A `Select`: `Auto`, `Input`, `Disk`.
6. **View.** Lane height `Select`, zoom out, zoom `Select`, zoom in.

### 3.9 The Compose to Record transition

The user must not lose the score. The score does not move horizontally, and no
glyph re-lays out. The transition opens space and drops the lanes into it.

| Step | Property | From | To | Duration | Delay | Easing |
|---|---|---|---|---|---|---|
| 1 | Spacer height under each system | 12 sp | lane block height | 260 ms | 0 | `EaseOut` |
| 2 | Lane block opacity | 0.0 | 1.0 | 200 ms | 80 ms | `EaseOut` |
| 3 | Lane block y offset | +12 px | 0 px | 200 ms | 80 ms | `EaseOut` |
| 4 | Compose top bar groups 3 to 7, opacity | 1.0 | 0.0 | 100 ms | 0 | `EaseIn` |
| 5 | Record top bar groups, opacity | 0.0 | 1.0 | 140 ms | 100 ms | `EaseOut` |
| 6 | Record button ring radius | 0 px | 2 px | 160 ms | 120 ms | `EaseOut` |

- Steps 2 and 3 stagger by 24 ms per system, through `motion::stagger`. The
  total added delay caps at 96 ms, so the fourth system and every later system
  share the same delay.
- The score glyph layer does not animate at all. It keeps its exact position.
- The vertical scroll position holds the first visible system at the top of the
  viewport, so the user keeps their place while the page grows.
- The reverse transition, Record to Compose, runs the same table backwards with
  each duration cut to 0.75 of its value.
- Under reduced motion, step 1 takes 0 ms, steps 2 and 3 become an instant
  swap, and steps 4 and 5 take 80 ms of opacity only.

---

## 4. Mix mode

### 4.1 The Record to Mix transition

The notation collapses. Duet does not animate the unwrap geometry. A wrapped
page and a linear timeline have no shared path, so a geometric morph would read
as noise and would cost a full re-layout per frame. Duet cross-fades and keeps
one anchor.

| Step | Property | From | To | Duration | Delay | Easing |
|---|---|---|---|---|---|---|
| 1 | Staff glyph layer opacity | 1.0 | 0.0 | 140 ms | 0 | `EaseIn` |
| 2 | Wrapped view opacity | 1.0 | 0.0 | 160 ms | 0 | `EaseIn` |
| 3 | Linear timeline opacity | 0.0 | 1.0 | 200 ms | 60 ms | `EaseOut` |
| 4 | Linear timeline y offset | +16 px | 0 px | 200 ms | 60 ms | `EaseOut` |
| 5 | Mixer strip row y offset | +100% | 0 | 240 ms | 60 ms | `EaseOut` |

The anchor is the playhead. Duet scrolls the linear timeline so the playhead
lands on the same screen x that it held in the wrapped view, when the scroll
range allows. The user follows one object across the change.

### 4.2 Mixer strip

Width: 96 px default, 72 px compact, 120 px wide. The user sets the width once
for the whole row. Gap between strips 1 px, filled by a `border` hairline.
Strip background `cx.theme().group_box`. Selected strip background
`list_active` with a 2 px top bar in `primary`.

From the top of a strip to the bottom:

| Row | Height | Content |
|---|---|---|
| 1 | 4 px | Part color bar, full width |
| 2 | 22 px | Name, 12 px, one line, truncated, tooltip with the full name |
| 3 | 24 px | Input `Select` and a 20 px monitor toggle |
| 4 | 4 x 20 px, 2 px gap | Insert slots. Each is an `outline` Button |
| 5 | 4 x 22 px, 2 px gap | Sends |
| 6 | 44 px | Pan knob 28 px, value label 11 px under it |
| 7 | 220 px | Fader and meter |
| 8 | 28 px | Mute, solo, arm: three 28 px square buttons, 4 px gap |
| 9 | 24 px | Output `Select` |

Every strip repeats the same row heights, so the row boundaries form
continuous horizontal lines across the whole mixer. That is the alignment
spine the design guide requires.

### 4.3 Fader

- Travel 200 px inside the 220 px row, with a 10 px inset at each end.
- Track: 6 px wide, radius 3 px, fill `cx.theme().slider_bar`.
- Cap: 28 px wide, 14 px tall, radius `cx.theme().radius`, fill
  `cx.theme().slider_thumb`, 1 device pixel `border`. A 2 px center notch in
  `muted_foreground` marks the exact value line.
- Hit area: 28 x 28 px around the cap. The whole 200 px track also accepts a
  click, which jumps the cap to that value.
- Taper: -inf dB at the bottom, +6 dB at the top. Unity gain, 0 dB, sits at 72
  percent of the travel. The lower 8 percent of the travel maps to -inf.
- Ticks: 1 device pixel marks at +6, 0, -6, -12, -24, -48 dB, on the trailing
  side of the track, in `duet.meter.scale`. The 0 dB tick is 2 px and runs the
  full track width.
- Detent: the cap snaps to 0 dB within 2 px of travel. A `shift` drag removes
  the detent and gives a 5x finer ratio.
- A double-click returns the fader to 0 dB.
- Keyboard: up and down move 0.5 dB. `shift` gives 0.1 dB. Page up and page
  down move 3 dB. `Home` gives 0 dB.
- The dB value shows in an 11 px tabular label under the cap, always visible,
  right aligned to the fader column.

### 4.4 Level meter

- Width 14 px, height 200 px, beside the fader with a 6 px gap.
- Scale from -60 dB at the bottom to +6 dB at the top, non-linear. -20 dB lands
  at 50 percent of the height, so the useful vocal range gets half the meter.
- **RMS** paints as a solid bar of the full 14 px width, from the bottom to the
  current value. Its color follows the value: `duet.meter.low` below -12 dB,
  `duet.meter.mid` from -12 to -3 dB, `duet.meter.high` above -3 dB. The change
  is a hard boundary at the threshold, not a gradient, so a reader can name the
  zone.
- **Peak** paints as a 2 px horizontal cap line across the full width, in
  `duet.meter.peak_cap`. It holds for 1200 ms, then falls at 20 dB per second.
- **Clip** paints the top 6 px of the meter in `duet.meter.clip` and writes
  `CLIP` in 8 px caps beside the meter. The state holds until the user clicks
  the meter or presses the `Clear clip` command. The text is the second cue, so
  the state does not depend on color.
- A numeric peak readout, painted by `MeterLayer`, sits under the meter, 11 px tabular, 40 px wide, right
  aligned. It shows the highest peak since the last reset, as `-3.2`.
- Scale ticks paint on the trailing edge at 0, -6, -12, -24, -48 dB, 1 device
  pixel, in `duet.meter.scale`. Labels show at 0 and -24 only, at 8 px.
- Every strip meter paints in one canvas owned by a `MeterLayer` leaf view that
  spans the strip row. That leaf owns the single animation frame. The mixer view
  itself is never marked dirty by the meter frame.

### 4.5 Knob

- Outer diameter 28 px. The pan knob and the send knobs use 18 px.
- Arc: from 225 degrees to -45 degrees, a 270 degree sweep. Track stroke 2.5 px
  in `cx.theme().muted`. Value stroke 2.5 px in `cx.theme().primary`.
- A bipolar knob, such as pan, fills its value arc from the 90 degree top
  center, not from the start of the sweep.
- Pointer: a 2 px line from 40 percent to 90 percent of the radius, in
  `foreground`.
- Drag: vertical only. The ratio is one percent of the range per pixel.
  `shift` gives 0.2 percent per pixel. A horizontal drag does nothing, so a
  user cannot change two knobs at once by accident.
- A double-click returns the knob to its default.
- Keyboard: up and down move one percent. `shift` moves 0.2 percent. `Home`
  returns to the default.
- The value label sits under the knob, 11 px tabular. A pan value reads `L24`,
  `C`, or `R18`.
- Hit area 28 x 28 px, even for the 18 px knob.

### 4.6 Sends and buses

- A send row is 22 px tall. From leading to trailing: a 4 px pre or post chip,
  the bus name at 10 px in 40 px, an 18 px knob, and a 28 px tabular value.
- The chip is hollow for a pre-fader send and filled for a post-fader send. A
  tooltip states which one. The shape is the cue, not a color.
- An empty send row shows `Add send…` in `muted_foreground` and behaves as a
  `ghost` Button.
- Bus strips sit at the trailing end of the strip row, after a 16 px gap and a
  1 px `border` divider that runs the full row height.
- A bus strip shares the track strip geometry. Its part color bar shows
  `cx.theme().muted`. Its name shows in 11 px ALL CAPS, which the design guide
  allows for a very short section label.
- The master strip is the last bus strip. Its width is 120 px at every setting,
  and its name row shows `MASTER`.

### 4.7 Mute and solo

- **Mute on.** The `M` button fills `cx.theme().warning` with
  `warning_foreground` text. The strip name and the fader column drop to 55
  percent alpha. The meter keeps full alpha, because the input signal still
  exists and the user needs to see it.
- **Solo on.** The `S` button fills `cx.theme().primary` with
  `primary_foreground` text.
- **Implicit mute.** Every strip that is not soloed drops its name and fader
  column to 55 percent alpha, and its `M` button draws a 1 px `warning` outline
  with no fill. The outline separates an implicit mute from an explicit mute.
- **Solo active.** When any solo is on, the transport bar shows a `Clear solo`
  Button with the `warning` treatment. That is the global escape from a state a
  user can forget.
- Alt-click on `S` makes the solo exclusive. Alt-click on `M` clears every mute.
  Both commands also live in the strip context menu.

### 4.8 Linear timeline and automation lanes

- Ruler: 28 px tall, sticky at the top of the timeline panel. Bars above at 11
  px, clock time below at 9 px in `muted_foreground`. A bar line paints 1 device
  pixel at `border`, and a bar number paints every bar at the current zoom, or
  every fourth bar when the bar width falls below 40 px.
- Track row: 72 px tall, the same waveform contract as section 3.3.
- Track header: 140 px, the same content as the lane header of section 3.7,
  plus a disclosure arrow for the automation lanes.
- Automation lane: 40 px tall, one per open parameter, below its track row.
  - The lane paints four horizontal guide lines at `border` at 40 percent
    alpha, at 0, 25, 50, and 75 percent of its height.
  - The curve is a 2 px polyline in `duet.automation.line`.
  - A point is a 7 px circle with a 1 px `background` ring, so a point stays
    visible over the curve. Hit area 20 x 20 px.
  - A double-click adds a point. A drag moves a point. `Delete` removes the
    selected points.
  - A segment between two points carries a curve handle at its midpoint, a 5 px
    diamond. A vertical drag on the handle bends the segment.
  - The lane header is 140 px and holds a parameter `Select` and a `ghost`
    close Button.
- Region selection: a selected region paints a 2 px `primary` border and raises
  its fill alpha by 15 points.
- Snap: the same grid `Select` as Compose, shared between the two modes.

---

## 5. Master mode

### 5.1 The Mix to Master transition

One object survives the change: the master strip. The user follows it.

| Step | Property | From | To | Duration | Delay | Easing |
|---|---|---|---|---|---|---|
| 1 | Track strip row y offset | 0 | +100% | 200 ms | 0 | `EaseIn` |
| 2 | Track strip row opacity | 1.0 | 0.0 | 140 ms | 0 | `EaseIn` |
| 3 | Master strip width | 120 px | 160 px | 240 ms | 0 | `EaseInOut` |
| 4 | Master strip x | its Mix x | the leading edge of the chain row | 240 ms | 0 | `EaseInOut` |
| 5 | Master chain y offset | +40 px | 0 px | 240 ms | 60 ms | `EaseOut` |
| 6 | Master chain opacity | 0.0 | 1.0 | 200 ms | 60 ms | `EaseOut` |
| 7 | Loudness panel x offset | +32 px | 0 px | 240 ms | 100 ms | `EaseOut` |

The linear timeline above does not move and keeps its scroll position and its
zoom. Only the lower half changes.

### 5.2 Master chain layout

- The chain row sits below the timeline, inside the same `v_resizable`.
  Minimum height 260 px, preferred 300 px.
- Leading: the master strip at 160 px, the same rows as section 4.2 with the
  fader row grown to 260 px.
- Then the stage row: an `h_flex` with a 12 px gap and a horizontal scroll.
- Five stages in fixed order: `Gain`, `EQ`, `Compressor`, `Limiter`, `Dither`.
  The order is fixed by construction. A user cannot reorder the master chain,
  because the correct order is not a preference.
- A stage is a `group_box`, 220 px wide and 240 px tall, radius
  `cx.theme().radius`, 1 px `border`.
  - Header row, 24 px: the stage name at 12 px semibold, then a `Switch` that
    bypasses the stage.
  - Graph area, 120 px: the stage's own curve. The EQ shows a response curve
    over a log frequency axis. The compressor shows a transfer curve with a
    live dot at the current input level. The limiter shows a gain reduction
    bar, 8 px tall, that grows to the leading side.
  - Control grid, 80 px: two rows of up to three 28 px knobs with 11 px labels.
- A bypassed stage drops to 50 percent alpha and its header shows a `Bypassed`
  `Badge` in `muted`.
- Trailing: the loudness panel at 320 px, pinned to the trailing edge, outside
  the horizontal scroll.

### 5.3 LUFS and true-peak meter

> Correction note (2026-09-21, architecture B75, critic N22I-5): the loudness preset set is Streaming (-14 LUFS, -1 dBTP), Broadcast (-23 LUFS, -1 dBTP), and Custom, as product requirements section 11 Q7 decides. Where this section names other targets, B75 governs.

The loudness panel, 320 px wide, 16 px inset, 16 px inner gap.

**LUFS bar.**
- Horizontal, 260 px long, 20 px tall, radius `cx.theme().radius`.
- Scale from -36 LUFS at the leading end to 0 LUFS at the trailing end.
- The bar fills to the momentary value, in the same three-zone color law as the
  level meter, with the thresholds set from the target.
- The short-term value paints as a 2 px vertical cap line in
  `duet.meter.peak_cap`.
- The target marker paints as a 2 px vertical line in `duet.lufs.target`, with
  the number above it at 10 px tabular. The default target is -14 LUFS, and a
  `Select` offers -23, -16, -14, and a custom value.
- The tolerance band paints +/-1 LU around the target, filled
  `duet.lufs.tolerance`.
- Scale ticks at -36, -30, -24, -18, -12, -6, 0, 1 device pixel, with labels at
  -24, -14, and 0.

**Readouts.** Three rows under the bar, each 20 px, label 96 px, value right
aligned at 14 px tabular:
- `Momentary` (400 ms window)
- `Short term` (3 s window)
- `Integrated` (the whole program, gated by BS.1770)
A fourth row shows `Loudness range` in LU.

**True peak.** A vertical meter beside the bar, 14 px wide and 120 px tall.
- Scale from -12 dBTP at the bottom to +3 dBTP at the top.
- The zone above -1 dBTP paints `duet.meter.clip` at 25 percent alpha as a
  permanent background, so the reader sees the ceiling before a peak reaches
  it.
- The value bar and the hold cap follow the level meter contract of section 4.4.
- A readout under it gives the maximum true peak at 11 px tabular.

**Result line.** One row under the readouts, 24 px, with a `Badge`:
`Within target` in `success`, or `Outside target by 2.3 LU` in `warning`. The
text carries the number, so color is not the only cue.

### 5.4 Export dialog

Component: `Dialog`, width 560 px. Title `Export master`. The trigger is an
`Export…` primary Button in the Master top bar. The ellipsis marks a command
that needs more input, as the design guide requires.

Form, through the `form` module. Label column 132 px, control column fills.

| Row | Control | Notes |
|---|---|---|
| Format | `Select` | `WAV`, `RF64`, `FLAC` (architecture 7.5 and story MA-04; MP3 is not in v1, corrected 2026-09-21) |
| Sample rate | `Select` | `44100`, `48000`, `88200`, `96000` |
| Bit depth | `Select` | Disabled for FLAC above 24 bit |
| Channels | `RadioGroup` | `Stereo`, `Mono` |
| Range | `Select` | `Whole project`, `Loop range`, `Selection` |
| Normalize | `RadioGroup` | `None`, `Peak`, `Loudness` |
| Target | `NumberInput` | LUFS. Enabled only for `Loudness` |
| Ceiling | `NumberInput` | dBTP. Enabled for `Peak` and `Loudness` |
| File name | `Input` | The project name is the default |
| Folder | `Input` plus a `Browse…` Button | |

- A disabled row keeps its label at full contrast and drops its control to the
  disabled treatment. Its help text states the condition: `Set Normalize to
  Loudness to use a target`.
- A summary line sits above the footer, 12 px `muted_foreground`:
  `Soprano session.wav - 48 kHz, 24 bit, stereo - about 62 MB`.
- Footer: `Cancel` on the leading side, `Export` as the primary on the trailing
  side, 8 px gap.
- **During the export** the dialog does not close.
  - A `Progress` bar appears above the footer, 4 px tall, full width.
  - The progress label reads `Pass 1 of 2: measure loudness` and then
    `Pass 2 of 2: render`. The two-pass shape comes from the loudness
    normalization design in the concept map.
  - The primary Button label becomes `Export` with a 12 px `Spinner`, and the
    Button is disabled, so a second submission is impossible.
  - `Cancel` becomes `Stop`. `Stop` deletes the partial file.
  - The dialog cannot be dismissed by a click outside. Escape asks `Stop the
    export?` through an `AlertDialog`.
- On success the dialog closes and the loudness report opens.

### 5.5 Loudness report

Component: `Sheet` from the trailing edge, width 420 px. Title
`Loudness report`.

1. A header block: the file name at 14 px semibold, the format and the length
   at 12 px `muted_foreground`.
2. A `DescriptionList` with eight rows, label 140 px, value tabular:
   `Integrated`, `Loudness range`, `True peak`, `Target`, `Applied gain`,
   `Sample rate`, `Bit depth`, `Duration`.
3. A result `Badge` row, the same words as section 5.3.
4. A short-term loudness chart from the `plot` module, 388 px wide and 140 px
   tall. The x axis is time in minutes. The y axis is LUFS from -36 to 0. The
   target line paints in `duet.lufs.target`, and the tolerance band paints
   behind the curve.
5. Footer, 16 px horizontal and 12 px vertical inset: `Reveal file` as an
   outline Button, `Done` as the primary.

The report persists with the project under the analysis folder, so the user
opens it again from the export history.

---

## 6. Motion vocabulary

Every value below maps to `gpui_kit::base::motion` or to `Animation` from
`gpui-pre`. No view invents a duration.

### 6.1 Durations

| Name | Value | Use |
|---|---|---|
| `instant` | 0 ms | A state change with no spatial meaning |
| `micro` | 80 ms | Press feedback, hover fill, focus ring |
| `short` | 140 ms | Tooltip, popover, tab change, simple fade |
| `medium` | 220 ms | Panel open, sheet, cross-fade between modes |
| `long` | 320 ms | The largest mode transition |

No interface transition passes 320 ms. Duet publishes these five values as
constants in one module, and a view names the constant.

### 6.2 Easings

From `gpui_kit::base::motion::Easing`:

| Intent | Easing | Reason |
|---|---|---|
| Enter, appear, expand | `Easing::EaseOut` | Fast start, soft arrival |
| Exit, dismiss, collapse | `Easing::EaseIn` | Soft start, fast departure |
| Move an object that stays | `Easing::EaseInOut` | Symmetric travel |
| A continuous value | `Easing::Linear` | Playhead, meter, progress |

### 6.3 Springs

From `gpui_kit::base::motion::Spring::new(response).with_damping(ratio)`:

| Case | Response | Damping |
|---|---|---|
| Note insertion and selection halo | 220 ms | 0.85 |
| Fader cap snap to a detent | 160 ms | 0.90 |
| Panel resize release | 240 ms | 1.00 |
| Take strip reorder | 200 ms | 0.90 |

A damping of 1.00 gives no overshoot. A panel that overshoots reads as loose,
so the resize case stays critically damped.

### 6.4 Case by case

**Mode transitions.** Sections 3.9, 4.1, and 5.1 give the full tables. The
mechanism is `motion::transition` on each animated property, plus
`motion::presence` for the layer that enters or leaves, plus `motion::stagger`
at 24 ms per system with a 96 ms cap.

**Playhead.** No animation object. `PlayheadLayer` calls
`window.request_animation_frame()` in its `render` and reads the transport's
current sample position from an atomic value. It paints at that position. It
never interpolates and never eases. On stop it holds its position with no
animation. On a locate it jumps with no animation, because a playhead that
slides after a locate reports a position that is false.

**Note insertion.** `animate_keyframes` over 160 ms with `Easing::EaseOut`:
- scale 0.85 at 0 percent, 1.04 at 65 percent, 1.00 at 100 percent;
- alpha 0.40 at 0 percent, 1.00 at 55 percent, 1.00 at 100 percent.
The animation applies to the head, the stem, and the flag as one transform
group, so the parts do not separate. The beam and the tie do not animate; they
appear at the end of the insertion animation.

**Take arm.** At rest the ring is static. When armed and the transport is
stopped, the ring alpha pulses:
`Animation::new(Duration::from_millis(1000)).repeat().with_easing(pulsating_between(0.45, 1.0))`.
When the transport records, the pulse stops and the fill stays solid. A blink
during a take is a distraction and also hides the real state.

**Meter ballistics.** These are signal values, not interface animation. The
engine computes them and the view samples the engine once per frame.

| Meter | Attack | Release | Hold |
|---|---|---|---|
| Peak | 0 ms, instant rise | 20 dB per second | 1200 ms |
| RMS | 10 ms | 300 ms | none |
| Loudness momentary | 400 ms window | per BS.1770 | none |
| Loudness short term | 3000 ms window | per BS.1770 | none |
| Gain reduction | 0 ms | 300 ms | none |

A meter never uses an `Easing`. An eased meter lies about the signal.

**Hover and press.** A hover fill crosses over 80 ms with `Easing::EaseOut`. A
press state applies at 0 ms and releases over 80 ms. A focus ring applies at
0 ms, because a delayed focus ring reads as a dropped key press.

### 6.5 Reduced motion

`gpui_base::init` reads the system preference into `App::reduce_motion`. Duet
adds a Settings control with three values: `System`, `Full`, `Reduced`. When
the user picks `System`, Duet calls `apply_system_reduce_motion`.

Under reduced motion:
- Every movement and size transition takes 0 ms.
- Every opacity transition takes 80 ms, so a layer change is still legible.
- Every spring becomes an instant set.
- The take arm pulse stops. The armed state shows as a solid fill plus the word
  `ARM` when the header width allows.
- The playhead and every meter continue to update, because they are data, not
  decoration.
- The note insertion animation becomes an instant paint at full alpha.

No state in Duet needs motion to be understood. That is the test.

---

## 7. Color and theme tokens

### 7.1 The token layer

GPUI Component's `Theme` has no field for a staff or a waveform. Duet defines
those roles in its own token layer, as the design guide requires.

- Duet publishes a `DuetTokens` struct as a GPUI `Global`.
- Duet resolves `DuetTokens` once at start and again on every change of
  `cx.theme().mode`.
- A view reads `cx.global::<DuetTokens>().staff_ink`. No view holds an `Hsla`
  literal.
- A token whose value equals a GPUI Component role derives from that role and
  does not repeat its value.

### 7.2 The tokens

Values are `hsl(hue saturation lightness)`. An alpha suffix states a fraction
of the base token.

| Token | Light | Dark | Derives from |
|---|---|---|---|
| `duet.staff.paper` | hsl(0 0% 100%) | hsl(222 14% 11%) | `background` |
| `duet.staff.ink` | hsl(222 20% 16%) | hsl(210 14% 88%) | `foreground` |
| `duet.staff.ink_muted` | hsl(222 12% 45%) | hsl(210 10% 62%) | `muted_foreground` |
| `duet.staff.line` | hsl(222 14% 34%) | hsl(210 10% 72%) | own |
| `duet.staff.grid` | hsl(222 10% 80%) | hsl(216 12% 28%) | `border` |
| `duet.staff.preview` | `duet.staff.ink` at 40% | `duet.staff.ink` at 40% | own |
| `duet.note.selected` | hsl(214 90% 44%) | hsl(210 96% 66%) | `primary` |
| `duet.note.selected_halo` | `duet.note.selected` at 14% | `duet.note.selected` at 22% | own |
| `duet.note.cursor` | `duet.note.selected` | `duet.note.selected` | own |
| `duet.part.soprano` | hsl(292 58% 48%) | hsl(292 68% 70%) | own |
| `duet.part.alto` | hsl(190 74% 34%) | hsl(188 70% 58%) | own |
| `duet.part.tenor` | hsl(34 88% 44%) | hsl(36 92% 62%) | own |
| `duet.part.bass` | hsl(150 50% 32%) | hsl(150 52% 54%) | own |
| `duet.wave.fill` | part color at 70% | part color at 62% | part color |
| `duet.wave.rms` | part color at 100% | part color at 100% | part color |
| `duet.wave.zero` | hsl(222 10% 72%) | hsl(216 12% 34%) | `border` |
| `duet.wave.absent` | hsl(222 10% 92%) | hsl(216 12% 19%) | `skeleton` |
| `duet.record.red` | hsl(0 72% 45%) | hsl(0 78% 64%) | own |
| `duet.record.tint` | `duet.record.red` at 10% | `duet.record.red` at 14% | own |
| `duet.meter.low` | hsl(150 56% 36%) | hsl(150 56% 54%) | own |
| `duet.meter.mid` | hsl(42 92% 44%) | hsl(44 94% 60%) | own |
| `duet.meter.high` | hsl(14 84% 46%) | hsl(14 88% 62%) | own |
| `duet.meter.clip` | hsl(0 80% 48%) | hsl(0 84% 64%) | `danger` |
| `duet.meter.peak_cap` | hsl(222 20% 16%) | hsl(210 14% 92%) | `foreground` |
| `duet.meter.scale` | hsl(222 10% 56%) | hsl(216 10% 52%) | `muted_foreground` |
| `duet.playhead` | hsl(214 90% 44%) | hsl(210 96% 66%) | `primary` |
| `duet.automation.line` | hsl(280 58% 48%) | hsl(280 70% 70%) | `accent` |
| `duet.lufs.target` | hsl(200 80% 38%) | hsl(200 84% 64%) | `info` |
| `duet.lufs.tolerance` | `duet.lufs.target` at 12% | `duet.lufs.target` at 18% | own |

### 7.3 Rules for the tokens

1. **Record red is never decoration.** `duet.record.red` paints the record
   button, the record head, the armed ring, and the punch band. Nothing else.
2. **The part color identifies the part everywhere.** The sidebar chip, the
   lane header bar, the waveform fill, the strip color bar, and the part filter
   underline all use the same four values. A user learns four colors once.
3. **The part colors do not carry meaning alone.** Every part surface also
   shows the letter `S`, `A`, `T`, or `B`, or the full part name. The colors
   sit at hues 292, 190, 34, and 150, which separate under the common forms of
   color vision deficiency, but the letter is the contract.
4. **The staff ink is not the body text color.** `duet.staff.ink` is slightly
   softer than `foreground` in the dark theme, because a hairline staff line at
   full white glares. The contrast floor of section 9 still holds.
5. **The selection color and the playhead color are the same hue.** Both mean
   "the place you are". They never appear in the same region at the same time,
   because the playhead is in the lane and the selection is on the staff.
6. **Every token is verified in both themes.** A custom theme changes
   `background` and `foreground`, so `DuetTokens` re-resolves and the derived
   tokens follow. A token marked `own` is checked against the new background
   for contrast at resolve time, and Duet lightens or darkens it by up to 12
   lightness points to meet the floor.

---

## 8. Load, empty, error, and no-device states

Components: `Skeleton` and `Shimmer` for a load state, `Empty` for an empty
state, `Alert` for an inline error that blocks the task, `Notification` for an
asynchronous result, `AlertDialog` for a decision.

The rule for a load state: keep the real geometry. A skeleton occupies the
exact rectangle that the real content will occupy, so no row reflows on
arrival.

### 8.1 Compose

**Load.** The shell paints at once. The score region shows two skeleton
systems: five 1 px lines per staff at `skeleton`, the brace, and eight
skeleton note blocks per system at 0.8 sp square. A shimmer sweeps over the
region every 1200 ms. The top bar renders in full and is disabled. The
transport is disabled. The status bar reads `Open the score…`.

**Empty.** A new project holds one grand staff system with four empty measures,
the clefs, the key signature, and the time signature. Under the system, at 32 px
of clear space, a centered hint block:
- 14 px `muted_foreground`: `Click the staff to add a quarter note.`
- 13 px `muted_foreground`: `Hold w, h, or e for a whole, half, or eighth note.`
- One outline Button: `Connect a MIDI keyboard`.
The hint hides on the first note and does not return.

**Error.** The score file did not parse. The work area shows the `Empty`
component with a `danger` icon at 32 px:
- Title, 16 px: `This score did not open`
- Body, 13 px `muted_foreground`: the file name and the reason on one line each.
- Actions: `Open the last commit` as the primary, `Show the file…` as outline.
A `Copy the error` `ghost` Button sits under the body.

**No device.** Compose needs no audio device. The score still edits. The status
bar shows `No audio device` in `warning`. The transport play button is disabled
and its tooltip reads `Choose an audio device in Settings`. Note preview
through the internal voice stays disabled, and the top bar shows no change.
Duet does not open a banner, because the task is not blocked.

### 8.2 Record

**Load.** The systems paint at once from the Compose state. Each lane paints
its region border at once and fills with `duet.wave.absent`. A shimmer sweeps
each absent region every 1200 ms. Peaks replace the placeholder per range as
the analysis worker reports a range. The user interface thread never waits for
a disk read.

**Empty.** A track with no take shows a dashed 1 px `border` rectangle at 60
percent alpha, inset 6 px inside the lane. Centered inside it:
- 12 px `muted_foreground`: `Arm this track and press record.`
- One 24 px outline Button: `Arm`.

**Error.** A take's audio file is missing. The lane paints a 10 percent
`danger` diagonal hatch at a 45 degree angle and a 6 px pitch. Centered:
- 12 px `danger`: `The audio file is missing.`
- One 24 px outline Button: `Locate…`
The region border paints in `danger` at 60 percent alpha.

**No device.** Record cannot proceed. An `Alert` of the `warning` variant spans
the work area width, above the first system, 48 px tall:
- Text: `No audio input. Duet cannot record.`
- Action: `Choose input…` as the primary Button.
Every arm button is disabled and its tooltip repeats the reason. The transport
record button is disabled. Playback of existing takes is also disabled, and the
transport play button carries the same tooltip. The staff still edits.

### 8.3 Mix

**Load.** Each strip paints as a stack of `Skeleton` blocks at the exact row
heights of section 4.2. The strip count comes from the project state, so the
row does not reflow on arrival. The timeline above shows the absent waveform
treatment of section 8.2.

**Empty.** The project has no track. The work area shows the `Empty` component:
- Title: `No tracks yet`
- Body: `Record a part in Record mode.`
- Action: `Go to Record` as the primary Button, which switches the mode.
The mixer row shows only the master strip, at full function.

**Error.** A processor did not start. Its insert slot paints a 1 px `danger`
border and shows the processor name in `danger` with a 12 px warning icon. The
strip header shows a `danger` `Badge` that reads `1 error`. A tooltip on the
slot reads `This effect did not start. The signal passes through it.` The
signal path stays connected, so a failed effect never silences a track.

**No device.** Faders, sends, knobs, and automation all stay usable, because
they change stored state. Every meter paints its scale and a flat baseline at
its floor, plus a centered 10 px `muted_foreground` label `No output`. The
master strip shows a `warning` `Badge` that reads `No device`. The transport
play button is disabled with the standard tooltip.

### 8.4 Master

**Load.** The chain stages render with their headers and their control grids in
place, and their graph areas show a `Skeleton` block. The loudness readouts
show `--.-` in `muted_foreground`, and a shimmer sweeps the LUFS bar.

**Empty.** The timeline holds no audio. Every stage renders in the disabled
treatment. The loudness panel shows `--.-` with no shimmer. The `Export…`
Button is disabled and its tooltip reads `Record or import audio first.`

**Error.** The export did not finish. An `AlertDialog`:
- Title: `Export did not finish`
- Body: the reason, then the path of the partial file.
- Actions: `Delete the partial file` with the `danger` treatment, and
  `Try again` as the primary.
A failure during pass 1 states `Duet measured 0 seconds of audio.` A failure
during pass 2 states the disk error.

**No device.** Analysis and export run offline. The loudness panel shows a
`Badge` that reads `Offline analysis`, and the live meters show their scale
with no bar. The transport play button is disabled. **`Export…` stays enabled.**
No audio device is needed to render a file, and a design that blocks the export
would be wrong.

### 8.5 One rule across the four modes

Duet reports a device problem once, in the status bar. A mode adds a second
surface only when the mode's primary task cannot proceed. Only Record does
that.

---

## 9. Accessibility

### 9.1 Contrast

| Content | Floor | Measured against |
|---|---|---|
| Body text, labels, values | 7.0:1 | Its own surface |
| Staff ink, note glyphs | 7.0:1 | `duet.staff.paper` |
| Staff lines, region borders, meter ticks | 3.0:1 | Their own surface |
| Hover and selection fills | 3.0:1 | The rest state fill |
| Focus ring | 3.0:1 | Both the control and the surface behind it |
| Disabled text | 3.0:1 | Its own surface |
| Waveform fill boundary | 3.0:1 | The lane background |
| Record head, playhead | 4.5:1 | The waveform fill under it |

A token marked `own` in section 7 is checked at resolve time. When a custom
theme pushes a token under its floor, Duet shifts the token's lightness by up
to 12 points toward the required side. When 12 points are not enough, Duet logs
a warning that names the token and uses `foreground` or `danger` instead.

### 9.2 Target sizes

The paint rectangle and the hit rectangle are separate. The hit rectangle never
falls below 24 x 24 px.

| Control | Paint | Hit |
|---|---|---|
| Top bar Button | 28 x 28 px | 28 x 28 px |
| Transport Button | 32 x 32 px | 32 x 32 px |
| Sidebar row Button | 24 x 24 px | 24 x 28 px |
| Mode switcher segment | 84 x 28 px | 84 x 28 px |
| Strip mute, solo, arm | 28 x 28 px | 28 x 28 px |
| Knob, 28 px | 28 px circle | 28 x 28 px |
| Knob, 18 px | 18 px circle | 28 x 28 px |
| Fader cap | 28 x 14 px | 28 x 28 px |
| Take strip | full width x 10 px | full width x 20 px |
| Punch handle | 8 px wide | 24 px wide |
| Automation point | 7 px circle | 20 x 20 px |
| Lane resize edge | 2 px | 8 px tall |

Two exceptions are below 24 px in one axis and are documented here:
- The take strip hit height is 20 px. Four strips at 24 px would push the lane
  block past its budget. The take list `Sheet` gives the same commands with
  full-size rows, and the keyboard path selects a take with `alt` plus a
  number.
- The lane resize edge hit height is 8 px. It is a drag affordance, not a
  command. The sidebar gives the same value as a `Select` with a 28 px row.

When two punch handles come closer than 48 px, the shared middle splits evenly,
so neither handle takes the other's target.

### 9.3 Keyboard path through the staff

The staff is one focus stop. `Tab` enters it and `Tab` leaves it. Inside, a
note cursor moves.

| Key | Result |
|---|---|
| Left, Right | Previous or next event in the current voice |
| Up, Down | One diatonic step |
| alt-Up, alt-Down | One semitone |
| cmd-Up, cmd-Down | One octave |
| Home, End | First or last event of the measure |
| cmd-Home, cmd-End | First or last event of the score |
| shift plus any move | Extend the selection |
| `1` to `6` | Set the duration for the next insertion |
| Enter | Insert at the cursor with the current duration and pitch |
| Delete, Backspace | Delete the selection |
| `t` | Tie to the next note |
| `.` | Add a dot |
| shift-F10, Menu | Open the context menu at the cursor |
| Escape | Clear the selection. A second Escape returns focus to the region |

Every command in the top bar and in the context menu has a keyboard path. No
command lives only behind the pointer.

### 9.4 Keyboard path through the timeline

The timeline is one focus stop. A region cursor moves inside it.

| Key | Result |
|---|---|
| Left, Right | One grid step |
| cmd-Left, cmd-Right | One bar |
| Up, Down | Previous or next track |
| shift plus Left or Right | Extend the selection |
| Space | Play or stop |
| `s` | Split at the playhead |
| `[`, `]` | Trim the start or the end to the playhead |
| `i`, `o` | Set punch in or punch out at the playhead |
| Enter | Open the take list for the current track |
| Delete | Delete the selected region |
| shift-F10, Menu | Open the region context menu |

### 9.5 Focus order

`F6` cycles the major regions forward. `shift-F6` cycles backward. The order:

1. Title bar and mode switcher
2. Top bar, group by group, leading to trailing
3. Sidebar tree
4. Work area
5. Inspector, when visible
6. Transport bar
7. Status bar

Inside a region the focus order follows the visual order. A dialog and a sheet
trap focus through `focus_trap`. Escape dismisses the topmost dismissible layer
and returns focus to the trigger. Duet never stacks two modal layers.

The focus ring is 2 px in `cx.theme().ring`, outset 2 px from the control
frame, with a radius of `cx.theme().radius` plus 2 px. The staff draws its ring
around the whole score region, not around one note, because the note cursor is
the inner indicator.

A clipped region clips an outward focus ring. Every scroll region in Duet
reserves a 4 px inner inset on the leading and trailing edges for that reason.

### 9.6 Names and second cues

- Every icon-only Button carries a `Tooltip` and an accessible name. The
  tooltip names the shortcut.
- `gpui_base::macos_accessibility` publishes the names. The staff element
  publishes one name per system, such as `System 3, bars 9 to 12`. The note
  cursor publishes `G4 quarter note, bar 9, beat 1`.
- Every colored state carries a second cue:

| State | Color | Second cue |
|---|---|---|
| Part identity | Part hue | The letter `S`, `A`, `T`, `B` |
| Selected note | `duet.note.selected` | The halo shape |
| Armed track | `duet.record.red` | A filled ring, plus `ARM` above 96 px |
| Muted strip | `warning` fill | The letter `M`, plus 55 percent alpha |
| Implicit mute | `warning` outline | The outline shape, plus 55 percent alpha |
| Solo | `primary` fill | The letter `S` |
| Clip | `duet.meter.clip` | The word `CLIP` |
| Pre-fader send | none | A hollow chip |
| Post-fader send | none | A filled chip |
| Bypassed stage | 50 percent alpha | The `Bypassed` badge |
| Within target | `success` | The badge text and the number |

### 9.7 Text size and zoom

- The shell scales from `cx.theme().font_size` through the rem helpers. Duet is
  verified at base font 14, 16, and 20 px.
- At base font 20 px the sidebar preferred width grows to 300 px and the strip
  width grows to 112 px, so no label truncates that did not truncate at 16 px.
- The staff does not scale with rem. It scales with its own zoom, on
  `cmd-plus` and `cmd-minus`, through the six `sp` steps of section 2.1. This
  is the one documented exception, and section 0 states the reason.
- Every label is free to wrap or to truncate with a tooltip. No control takes
  its width from one English string.

---

## 10. Review checklist for this contract

Run every item before a surface is called complete.

1. The primary task of the mode is the largest thing on screen.
2. No view holds a color, a radius, or a font size literal.
3. Hover, focus, pressed, selected, disabled, load, and error each have a
   distinct treatment on every control.
4. The load, empty, error, and no-device states of section 8 all render.
5. Equal gaps are equal to the device pixel, and they resolve from one shared
   token.
6. Every command is reachable from the keyboard.
7. Escape dismisses the topmost layer and returns focus to the trigger.
8. Every colored state has its second cue from section 9.6.
9. The surface is checked in the light theme, the dark theme, and at base font
   14 and 20 px.
10. The surface is checked with reduced motion on.
11. The window is checked at 1024 x 700 px and at 1920 x 1200 px.
12. The staff is checked at `sp` = 6 px and `sp` = 16 px.

---

## Orchestrator decisions (2026-09-20)

The Automated Orchestrator decided the three points the Designer raised for the Product Manager.

| Point | Decision | Reason |
|---|---|---|
| Master chain order | Fixed for v1; not user-reorderable. | A fixed order is testable and matches the three loudness presets. |
| Export with no audio device | Enabled. The export pipeline is offline and needs no device. | A user with no interface can still finish a project. |
| Device banner | Only Record mode raises a second device banner. Every other mode reports the device state once in the status bar. | Record is the one mode where a lost device loses work. |

## Twelfth custom element: `StageCurve` (orchestrator decision, 2026-09-20)

Architecture section 10 adds one element to the eleven this contract names: `StageCurve`, one processor's response curve (an equalizer response, a compressor transfer curve with a live dot, a limiter gain-reduction bar). It serves stories M-03 and MA-02 and lives in `crates/duet/src/element/stage_curve.rs` (chunk K4). Visual rules: the curve uses the same canvas, ink, and grid tokens as `AutomationLane`; the live dot uses `duet.meter.peak_cap`; the height is 96 px in a strip inspector and 160 px in the master chain. The Designer reviews the element at the K4 design checkpoint.
