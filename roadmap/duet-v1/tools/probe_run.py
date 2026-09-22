"""Run every placement-guard probe of architecture section 1.9.

Four revisions recorded a probe result that no run had produced, and the
revision-12 review found two of them inside one cell (critic CR-3, CR-13). A
recorded result is a measurement, so a measurement needs a machine. This
harness is that machine: it plants each probe's own defect in a throwaway copy
of the document, runs `placement_check.py` against the copy, and prints the
exit code and the lines the probe's row records.

usage: probe_run.py <architecture.md> [probe id ...]

It exits 0 when every probe is correct by DR5: the baseline run is green and
every planted run is red. It exits 1 when a planted run is not red, and 2 on a
usage or input failure. `PROBES=1` is not a mode; every run is the whole set
unless the caller names probe ids.

**PP25 is not here.** It needs a compiler and a scratch Cargo workspace, so
`probe_roster.sh` runs it and section 1.9 says at the PG25 site that a roster
probe needs its own run (critic CR-13). The conversion probes are not here
either: each one needs a throwaway Cargo workspace and `probe_conversion.py`
builds one per shape.
"""

import os
import re
import subprocess
import sys
import tempfile

import placement_check
from probe_fragments import flatten, missing, recorded_fragments

HERE = os.path.dirname(os.path.abspath(__file__))
GUARD = os.path.join(HERE, "placement_check.py")


def swap(old, new):
    """One mutation that replaces exactly one occurrence of `old`."""

    def apply(text):
        if text.count(old) != 1:
            raise LookupError(f"the anchor appears {text.count(old)} times: {old[:70]}")
        return text.replace(old, new, 1)

    return apply


def drop_block(block_id):
    """Delete a registered block's fence or table body, keeping the heading."""

    def apply(text):
        return cut_block(text, block_id, keep_rows=0)

    return apply


def empty_block(block_id):
    """Cut a registered block to one row below its stated minimum.

    A block whose minimum is one has no shape between whole and absent, so
    the cut is to zero rows there and the guard prints the same named line
    (DR7).
    """
    _heading, _kind, minimum = placement_check.DATA_BLOCKS[block_id]

    def apply(text):
        return cut_block(text, block_id, keep_rows=max(minimum - 1, 0))

    return apply


def drop_marker(block_id):
    """Delete a registered block's marker line and leave the block in place."""

    def apply(text):
        heading, _kind, minimum = placement_check.DATA_BLOCKS[block_id]
        return swap(f"<!-- GUARD BLOCK id={block_id} rows>={minimum} -->\n", "")(text)

    return apply


def rename_heading(block_id):
    """Rename a registered block's heading and leave the block in place."""

    def apply(text):
        heading = placement_check.DATA_BLOCKS[block_id][0]
        return swap(f"#### {heading}\n", f"#### {heading} probe\n")(text)

    return apply


def block_span(text, block_id):
    """The `(start, end)` of one registered block's body, marker included."""
    heading, kind, _minimum = placement_check.DATA_BLOCKS[block_id]
    head = re.search(r"(?m)^#### " + re.escape(heading) + r"\s*$", text)
    if not head:
        raise LookupError(f"no heading for {block_id}")
    rest = text[head.end() :]
    stop = re.search(r"(?m)^#{1,4} ", rest)
    region_end = head.end() + (stop.start() if stop else len(rest))
    mark = placement_check.MARKER.search(text, head.end(), region_end)
    if not mark:
        raise LookupError(f"no marker for {block_id}")
    body = text[mark.end() + 1 : region_end]
    if kind == "table":
        header = re.match(r"\|[^\n]*\|[ \t]*\n\|[-\s|:]+\|[ \t]*\n", body)
        lead = mark.end() + 1 + header.end()
        rows = []
        for line in body[header.end() :].splitlines():
            if not line.startswith("|"):
                break
            rows.append(line)
        return lead, lead + sum(len(row) + 1 for row in rows), rows
    fence = re.match(r"```[a-z]*\n(.*?)^```", body, re.S | re.M)
    lead = mark.end() + 1 + body.index("\n", 0) + 1
    rows = fence.group(1).splitlines()
    return lead, lead + len(fence.group(1)), rows


def cut_block(text, block_id, keep_rows):
    """Replace a registered block's rows with the first `keep_rows` of them."""
    start, end, rows = block_span(text, block_id)
    kept = "".join(row + "\n" for row in rows[:keep_rows])
    return text[:start] + kept + text[end:]


def add_rust(text, code):
    """Append one declaration to the section 15.1 Rust block."""
    anchor = "pub struct GainDb(Finite);"
    return swap(anchor, anchor + "\n\n" + code)(text)


PROBES = [
    ("PP1", "no document argument", None, 2, ["usage:"]),
    ("PP2", "a path that does not exist", "MISSING", 2, ["FAIL: cannot open"]),
    (
        "PP3",
        "every ```rust fence renamed",
        lambda text: text.replace("```rust\n", "```rusty\n"),
        2,
        ["FAIL: the `constants`"],
    ),
    (
        "PP4a",
        "a declared struct ProbeUnplaced with no section 1.5 row",
        lambda text: add_rust(text, "/// A probe.\npub struct ProbeUnplaced { value: u8 }"),
        1,
        ["UNPLACED:"],
    ),
    ("PP4-droplist", "the candidate drop list deleted under a kept heading", drop_block("drop-list"), 2, ["FAIL: the `drop-list`"]),
    ("PP4-primtraits", "the primitive trait-set block deleted", drop_block("primitive-traits"), 2, ["FAIL: the `primitive-traits`"]),
    ("PP4-primsizes", "the primitive size block deleted", drop_block("primitive-sizes"), 2, ["FAIL: the `primitive-sizes`"]),
    (
        "PP4b",
        "the VoiceId declaration removed from section 15.3",
        swap("pub struct VoiceId(u64);", ""),
        1,
        ["UNDECLARED:"],
    ),
    (
        "PP5",
        "a second declaration of Knot",
        lambda text: add_rust(text, "/// A probe.\npub struct Knot { at: Ticks, x: f32 }"),
        1,
        ["DUPLICATED:"],
    ),
    (
        "PP6",
        "Pixels added to the duet-command row of section 1.5",
        swap("| `duet-command` | `Verb`", "| `duet-command` | `Pixels`, `Verb`"),
        1,
        ["MISCLAIMED:"],
    ),
    (
        "PP7",
        "sidebar_width retyped to gpui_kit::Px in ModeView",
        swap("pub struct ModeView {\n    sidebar_width: LogicalPx,", "pub struct ModeView {\n    sidebar_width: gpui_kit::Px,"),
        1,
        ["FRAMEWORK:"],
    ),
    (
        "PP8",
        "SlotSpec::kind retyped to the arm name Reverb",
        swap("pub struct SlotSpec { slot: SlotId, kind: SlotKind,", "pub struct SlotSpec { slot: SlotId, kind: Reverb,"),
        1,
        ["UNPLACED:"],
    ),
    (
        "PP9",
        "the Copy derive removed from ExportSpec",
        swap(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\npub struct ExportSpec {",
            "#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]\npub struct ExportSpec {",
        ),
        1,
        ["COPY:"],
    ),
    (
        "PP10",
        "port: MidiPortId in MidiRecord and reset: AtomicU32 in MeterState",
        lambda text: swap(
            "pub struct MidiRecord { port: BoundPort,", "pub struct MidiRecord { port: MidiPortId,"
        )(
            swap(
                "pub struct MeterState { reading: MeterReading, over: OverMark, decay: Finite }",
                "pub struct MeterState { reading: MeterReading, over: OverMark, decay: Finite, reset: AtomicU32 }",
            )(text)
        ),
        1,
        ["NOT COPY:"],
    ),
    (
        "PP10b",
        "the DeviceKey declaration removed",
        lambda text: re.sub(r"pub struct DeviceKey\([^)]*\);", "", text, count=1),
        1,
        ["NOT DECIDED:"],
    ),
    (
        "PP11",
        "a sentence that `duet-time` writes `duet-project`",
        swap(
            "### 1.4 Pure crates\n",
            "### 1.4 Pure crates\n\nA probe sentence: `duet-time` writes `duet-project` state.\n",
        ),
        1,
        ["EDGE MISS:"],
    ),
    (
        "PP12",
        "`serde` removed from the duet-score row of section 1.2",
        swap(
            "| `duet-score` | The score aggregate accepts no command that breaks a notation invariant | Yes | `serde`, `serde_json`",
            "| `duet-score` | The score aggregate accepts no command that breaks a notation invariant | Yes | `serde_json`",
        ),
        1,
        ["DEP MISS:"],
    ),
    (
        "PP13",
        "the Copy derive removed from MeterSnapshot",
        swap(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct MeterSnapshot {",
            "#[derive(Debug, Clone, PartialEq, Eq)]\npub struct MeterSnapshot {",
        ),
        1,
        ["SNAPSHOT:"],
    ),
    (
        "PP14",
        "probe: AnyView added to Toolbar, which derives Copy in an application row",
        swap(
            "pub(crate) struct Toolbar { groups: ToolbarGroups }",
            "pub(crate) struct Toolbar { groups: ToolbarGroups, probe: AnyView }",
        ),
        1,
        ["COPY UNDECIDED:"],
    ),
    (
        "PP15",
        "8.3 removed from the Declared in cell of the duet-command row",
        swap("| 1.8, 3.5, 7.4, 8.1, 8.3, 8.4, 9.1, 9.5, 9.6, 10.2, 12.4, 15.5 |",
             "| 1.8, 3.5, 7.4, 8.1, 8.4, 9.1, 9.5, 9.6, 10.2, 12.4, 15.5 |"),
        1,
        ["REGISTER:"],
    ),
    (
        "PP16",
        "the `soak` row of the selected-test table renamed to `soakx`",
        swap("| `soak` | C3 | 6 |", "| `soakx` | C3 | 6 |"),
        1,
        ["TEST MISS:"],
    ),
    (
        "PP17",
        "the SharedString name removed from the justified-unknown table",
        swap("`Pixels`, `Runtime`, `SharedString`, `Subscription`,", "`Pixels`, `Runtime`, `Subscription`,"),
        1,
        ["UNJUSTIFIED:"],
    ),
    (
        "PP18",
        "the BTreeMap name removed from its external-verdict row",
        swap("| `Vec`, `VecDeque`, `BTreeMap`, `BTreeSet` | `not Copy` |", "| `Vec`, `VecDeque`, `BTreeSet` | `not Copy` |"),
        1,
        ["EXTERNAL:"],
    ),
    (
        "PP19",
        "the Default derive removed from Finite in section 2.6a",
        swap(
            "#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]\n#[serde(try_from = \"f64\")]\npub struct Finite(f64);",
            "#[derive(Debug, Clone, Copy, Serialize, Deserialize)]\n#[serde(try_from = \"f64\")]\npub struct Finite(f64);",
        ),
        1,
        ["CLOSURE:"],
    ),
    (
        "PP20",
        "probe_strip: StripId added to duet-engrave::FontMetrics",
        swap("pub struct FontMetrics { staff_space: f32,", "pub struct FontMetrics { probe_strip: StripId, staff_space: f32,"),
        1,
        ["REACH:"],
    ),
    (
        "PP20b",
        "MAX_STRIPS moved under a `// duet-project` comment",
        swap(
            "// duet-time\n/// Strips one project may hold, and the meter array length (B86).\npub const MAX_STRIPS: usize = 48;\n",
            "// duet-project\n/// Strips one project may hold, and the meter array length (B86).\npub const MAX_STRIPS: usize = 48;\n// duet-time\n",
        ),
        1,
        ["CONST:", "LIMIT:"],
    ),
    (
        "PP21a",
        "the duet-session::track::Source expectation site renamed in its B.1 row",
        swap("| `duet-session::track::Source` |", "| `duet-session::track::Sourcex` |"),
        1,
        ["B.1:"],
    ),
    (
        "PP21b",
        "the duet-score::mark::MarkKind site renamed in the variant_size_differences table",
        swap("| `duet-score::mark::MarkKind` |", "| `duet-score::mark::MarkKindx` |"),
        1,
        ["B.1:"],
    ),
    (
        "PP21c",
        "the SlotState expect reason states a size as a literal",
        swap(
            'reason = "one slot state is B51 bytes and it is the identity the audio thread owns for the \\',
            'reason = "one slot state is 80 bytes and it is the identity the audio thread owns for the \\',
        ),
        1,
        ["REASON:"],
    ),
    (
        "PP21d",
        "a doc comment states a size as a literal",
        swap(
            "/// nothing (TH9). One write is a memory copy of the size the PG24 table",
            "/// nothing (TH9). One write is a memory copy of 9000 bytes, and the PG24 table",
        ),
        1,
        ["REASON:"],
    ),
    (
        "PP21e",
        "a missing_copy_implementations expectation on a Drop type",
        swap(
            "#[derive(Debug)]\npub struct HotplugSubscription {",
            '#[expect(missing_copy_implementations, reason = "a probe")]\n#[derive(Debug)]\npub struct HotplugSubscription {',
        ),
        1,
        ["B.1:"],
    ),
    (
        "PP22",
        "the NoteId name changed to NoteIdx in its section 3.5 VR1 row",
        swap("| `NoteId`, `PartId`, `StaffId`,", "| `NoteIdx`, `PartId`, `StaffId`,"),
        1,
        ["VR1:"],
    ),
    (
        "PP23",
        "the Eq derive removed from Revision in section 15.3",
        swap(
            "#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]\npub struct Revision(u64);",
            "#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]\npub struct Revision(u64);",
        ),
        1,
        ["EQ:"],
    ),
    (
        "PP24",
        "the variant_size_differences expectation removed from SlotState in section 5.5",
        swap(
            '#[expect(\n    variant_size_differences,\n    reason = "audio-owned state stays inline; the arm spread is intended and bounded by B51"\n)]\n',
            "",
        ),
        1,
        ["VARIANT:"],
    ),
    (
        "PP26-a",
        "Equalizer(Box<EqualizerState>) in place of the inline arm, the revision-11 shape",
        swap("    Equalizer(EqualizerState),", "    Equalizer(Box<EqualizerState>),"),
        1,
        ["HEAP:"],
    ),
    (
        "PP26-b",
        "probe: Arc<Generation> added to ChainState, a heap handle that never grows",
        swap("pub struct ChainState {\n    /// The identity of this chain, which `ChainTopology::strip` names.", "pub struct ChainState {\n    probe: Arc<Generation>,\n    track: TrackId,"),
        1,
        ["HEAP:"],
    ),
    (
        "PP26-c",
        "the basedrop::Owned wrapper removed from DiskWriter.samples, which leaves a bare rtrb end",
        swap("    samples: Owned<Producer<f32>>,\n    written_frames: u64,",
             "    samples: Producer<f32>,\n    written_frames: u64,"),
        1,
        ["HEAP:"],
    ),
    (
        "PP26-d",
        "the basedrop::Owned wrapper removed from EngineProcess.refills",
        swap("    refills: Owned<Producer<RefillRequest>>,", "    refills: Producer<RefillRequest>,"),
        1,
        ["HEAP:"],
    ),
    (
        "PP26-e",
        "the one exemption row removed, which leaves the Arc on GraphState.resets bare",
        swap("GraphState.resets  Arc  outlives-cycle\n", ""),
        2,
        ["FAIL: the `audio-exempt`"],
    ),
    (
        "PP26b-owned",
        "probe_grow: Owned<Vec<u8>> added to DiskReader, which is the Critic shape MINE-A1",
        swap("pub struct DiskReader {\n    track: TrackId,", "pub struct DiskReader {\n    probe_grow: Owned<Vec<u8>>,\n    track: TrackId,"),
        1,
        ["GROW:"],
    ),
    (
        "PP26b-shared",
        "probe_shared: Shared<Vec<u8>> added to ChainState",
        swap("pub struct ChainState {\n    /// The identity of this chain, which `ChainTopology::strip` names.", "pub struct ChainState {\n    probe_shared: Shared<Vec<u8>>,\n    track: TrackId,"),
        1,
        ["GROW:"],
    ),
    (
        "PP26b-fixed",
        "CONTROL: probe_fixed: Owned<Box<[f32]>> added to ChainState, which is legal and stays green",
        swap("pub struct ChainState {\n    /// The identity of this chain, which `ChainTopology::strip` names.", "pub struct ChainState {\n    probe_fixed: Owned<Box<[f32]>>,\n    track: TrackId,"),
        0,
        [],
    ),
    (
        "PP26c-mutex",
        "probe_lock: Mutex<u32> added to ChainState, which is the Critic shape MINE-A4",
        swap("pub struct ChainState {\n    /// The identity of this chain, which `ChainTopology::strip` names.", "pub struct ChainState {\n    probe_lock: Mutex<u32>,\n    track: TrackId,"),
        1,
        ["LOCK:"],
    ),
    (
        "PP26c-rwlock",
        "probe_rw: RwLock<u32> added to ChainState, which is the Critic shape MINE-A6",
        swap("pub struct ChainState {\n    /// The identity of this chain, which `ChainTopology::strip` names.", "pub struct ChainState {\n    probe_rw: RwLock<u32>,\n    track: TrackId,"),
        1,
        ["LOCK:"],
    ),
    (
        "PP26c-parking",
        "probe_pl: parking_lot::Mutex<u32> added to ChainState, which proves the last path segment is the name",
        swap("pub struct ChainState {\n    /// The identity of this chain, which `ChainTopology::strip` names.", "pub struct ChainState {\n    probe_pl: parking_lot::Mutex<u32>,\n    track: TrackId,"),
        1,
        ["LOCK:"],
    ),
    (
        "PP26-f",
        "a bare Box<[u8]> in place of the Arc on the one exempt field, which a path-only row let through",
        swap("    resets: Arc<ResetGenerations>,\n}\n\n/// How a chain runs.", "    resets: Box<[u8]>,\n}\n\n/// How a chain runs."),
        1,
        ["HEAP:"],
    ),
    (
        "PP26b-below",
        "probe: Vec<u8> inside ResetGenerations, one step BELOW the one exempt field",
        swap("pub struct ResetGenerations { entries:", "pub struct ResetGenerations { probe: Vec<u8>, entries:"),
        1,
        ["GROW:"],
    ),
    (
        "PP26c-below",
        "probe: Mutex<u32> inside ResetGenerations, one step BELOW the one exempt field",
        swap("pub struct ResetGenerations { entries:", "pub struct ResetGenerations { probe: Mutex<u32>, entries:"),
        1,
        ["LOCK:"],
    ),
    (
        "PP26d-swap",
        "Transport swapped for ChannelConfig in the audio-owned block, a clean one-for-one swap",
        swap("\nTransport\nTransportState\n", "\nChannelConfig\nTransportState\n"),
        1,
        ["ROOT:"],
    ),
    (
        "PP26d-marker",
        "the audio-owned marker removed from the Transport declaration",
        swap("/// **Audio-owned** (section 5.7).\npub struct Transport {",
             "pub struct Transport {"),
        1,
        ["ROOT:"],
    ),
    (
        "PP26e-swap",
        "ChainState swapped for ChannelConfig in the block AND its marker moved, which is"
        " the Critic's four-line coordinated edit at a reachable root",
        lambda text: swap(
            "/// **Audio-owned** (section 5.7).\n#[derive(Debug)]\npub struct ChainState {",
            "#[derive(Debug)]\npub struct ChainState {",
        )(
            swap("\nChainState\n", "\nChannelConfig\n")(
                swap(
                    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct ChannelConfig {",
                    "/// **Audio-owned** (section 5.7).\n"
                    "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct ChannelConfig {",
                )(text)
            )
        ),
        1,
        ["CLOSURE:    "],
    ),
    (
        "PP26e-leaf",
        "the BufferPool row removed from the reachable-leaf block",
        swap("BufferPool      the pool itself", "BufferPoolx     the pool itself"),
        1,
        ["CLOSURE:    "],
    ),
    (
        "PP31-link",
        "J1 removed from phase 8, so the J1 before K1 link names a chunk no phase holds",
        swap("| 8 | M8 | `.github/workflows/audio-smoke.yml` | F5, H2, I2, J1 | 4 |",
             "| 8 | M8 | `.github/workflows/audio-smoke.yml` | F5, H2, I2 | 3 |"),
        1,
        ["LINK:"],
    ),
    (
        "PP31-line",
        "K2 moved into phase 11 beside K3, which SM6 forbids",
        swap("| 10 | none | none | H4, I4, K2 | 3 |\n| 11 | none | none | J3, K3 | 2 |",
             "| 10 | none | none | H4, I4 | 2 |\n| 11 | none | none | J3, K2, K3 | 3 |"),
        1,
        ["LINK:"],
    ),
    (
        "PP26f-swap",
        "the Critic's four-line coordinated edit at Transport, which PG26d and PG26e both pass",
        lambda text: swap(
            "/// **Audio-owned** (section 5.7).\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct ChannelConfig {",
            "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct ChannelConfig {",
        )(
            swap("\nTransport\nTransportState\n", "\nChannelConfig\nTransportState\n")(
                swap(
                    "/// **Audio-owned** (section 5.7).\npub struct Transport {",
                    "pub struct Transport {",
                )(
                    swap(
                        "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct ChannelConfig {",
                        "/// **Audio-owned** (section 5.7).\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct ChannelConfig {",
                    )(text)
                )
            )
        ),
        1,
        ["ASSERTED:   "],
    ),
    (
        "PP26f-row",
        "one audio-asserted row deleted",
        swap("MidiRecord       argument;", "MidiRecordx      argument;"),
        1,
        ["ASSERTED:   "],
    ),
    (
        "PP31b-pair",
        "the E1 D2 row deleted from the phase-pair-exempt block",
        swap("E1  D2  E1 adds the `duet-dsp` entry", "E1x D2  E1 adds the `duet-dsp` entry"),
        1,
        ["PAIR:       "],
    ),
    (
        "PP33-decl",
        "the finite_to_f32_saturating DECLARATION deleted, with the list and the B.1 row kept",
        swap("#[must_use]\npub fn finite_to_f32_saturating(value: Finite) -> f32;",
             "#[must_use]\npub fn finite_to_f32_saturating_absent(value: Finite) -> f32;"),
        1,
        ["SUPPRESS:   "],
    ),
    (
        "PP33-word",
        "the count word of the section 2.3 list changed from Seven to Six",
        swap("Seven functions carry a suppression", "Six functions carry a suppression"),
        1,
        ["SUPPRESS:   "],
    ),
    (
        "PP37-neighbour",
        "the B120 row changed from 9 slots to 8 slots, which is the value of a constant beside it",
        swap("| B120 | 9 slots |", "| B120 | 8 slots |"),
        1,
        ["VALUE:      "],
    ),
    (
        "PP37",
        "the B113 row changed from 13 elements to 12 elements, with the declaration kept",
        swap("| B113 | 13 elements |", "| B113 | 12 elements |"),
        1,
        ["VALUE:      "],
    ),
    (
        "PP33-b1count",
        "the Appendix B.1 complexity count word changed from Six to Nine",
        swap("Six complexity suppressions.", "Nine complexity suppressions."),
        1,
        ["SUPPRESS:   "],
    ),
    (
        "PP34-count",
        "the SM6 cost bullet restored to the revision-17 numbers",
        swap("Sixteen phases replace ten", "Thirteen phases replace ten"),
        1,
        ["TAIL:       "],
    ),
    (
        "PP31b-reason",
        "one phase-pair exemption reason that names neither chunk of its own pair",
        swap(
            "E1  D2  E1 adds the `duet-dsp` entry and names `SampleSource` and `Pyramid`,"
            " which D1 wrote in phase 1; D2 writes the dynamics kernels and E1 names none"
            " of them",
            "E1  D2  a reason that names neither chunk of its own pair",
        ),
        1,
        ["TAIL:       "],
    ),
    (
        "PP35-sequence",
        "the per-phase Cargo.lock writer sequence moved by one at phase four",
        swap(
            "writer counts, phases 0 to 15: 2, 3, 4, 5, 7,",
            "writer counts, phases 0 to 15: 2, 3, 4, 5, 6,",
        ),
        1,
        ["LOCK SEQ:   "],
    ),
    (
        "PP35-writer",
        "one chunk's Writes cell emptied of Cargo.lock, which moves the derived count",
        swap(
            "`crates/duet-engrave/benches/`, `crates/duet-engrave/Cargo.toml`, `Cargo.lock`",
            "`crates/duet-engrave/benches/`, `crates/duet-engrave/Cargo.toml`",
        ),
        1,
        ["LOCK SEQ:   "],
    ),
    (
        "PP36-owner",
        "one Appendix B.5 pin owner returned to the revision-19 chunk",
        swap(
            'features = ["pipewire"]` | M4 |',
            'features = ["pipewire"]` | M3 |',
        ),
        1,
        ["PIN OWNER:  "],
    ),
    (
        "PP36-pin",
        "one manifest pin list emptied of the name an appendix row owns",
        swap("`coremidi`**, **`pipewire`**, `gix`,", "`coremidi`**, **`pipewire`**,"),
        1,
        ["PIN OWNER:  "],
    ),
    (
        "PP33-row",
        "the finite_to_f32_saturating row deleted from the b1-convert block",
        swap("| `finite_to_f32_saturating` | `as_conversions`",
             "| `finite_to_f32_saturatingx` | `as_conversions`"),
        1,
        ["SUPPRESS:   "],
    ),
    (
        "PP33-list",
        "ticks_to_f64 removed from the section 2.3 list",
        swap("`f64_to_f32`, `ticks_to_f64`, and `finite_to_f32_saturating`",
             "`f64_to_f32`, and `finite_to_f32_saturating`"),
        1,
        ["SUPPRESS:   "],
    ),
    (
        "PP34-tail",
        "K6 moved from phase 14 into phase 15",
        swap("| 14 | none | none | K6 | 1 |\n| 15 | none | none | none;",
             "| 14 | none | none | none | 0 |\n| 15 | none | none | K6;"),
        1,
        ["TAIL:       "],
    ),
    (
        "PP30a",
        "10.2 removed from the Used by cell of B99",
        swap("the contract row that sets the value | 1.6, 1.9, 10.2, C.9, C.10 |",
             "the contract row that sets the value | 1.6, 1.9, C.9, C.10 |"),
        1,
        ["USED BY:"],
    ),
    (
        "PP30b",
        "9.9 added to the Used by cell of B99",
        swap("the contract row that sets the value | 1.6, 1.9, 10.2, C.9, C.10 |",
             "the contract row that sets the value | 1.6, 1.9, 9.9, 10.2, C.9, C.10 |"),
        1,
        ["USED BY:"],
    ),
    (
        "PP4-drop",
        "a declared struct ProbeUnplaced whose name is then added to the candidate drop list",
        lambda text: swap("CoreReceiver SyncSender OneshotSender OneshotReceiver\n```", "CoreReceiver SyncSender OneshotSender OneshotReceiver ProbeUnplaced\n```")(
            add_rust(text, "/// A probe.\npub struct ProbeUnplaced { value: u8 }")
        ),
        1,
        ["MEMBER:"],
    ),
    (
        "PP28",
        "the MAX_PARAMS owner changed to duet-engine in the shared-limit block",
        swap("MAX_PARAMS duet-time duet-session duet-engine duet", "MAX_PARAMS duet-engine duet-session duet-engine duet"),
        1,
        ["LIMIT:"],
    ),
    (
        "PP29a",
        "the PG13 rule id renamed to PG43 in the probe table",
        swap("| PG13 | An audio-published type that is not `Copy` |", "| PG43 | An audio-published type that is not `Copy` |"),
        1,
        ["PROBE:"],
    ),
    (
        "PP40",
        "chunk A2's Writes cell moved from crates/duet-engrave to crates/duet-dsp",
        swap("| A2 | 3 | Horizontal spacing and the system break | `crates/duet-engrave/src/{spacing,system}.rs` |",
             "| A2 | 3 | Horizontal spacing and the system break | `crates/duet-dsp/src/{spacing,system}.rs` |"),
        1,
        ["CHUNK CRATE:"],
    ),
    (
        "PP40-owner",
        "the A duet-engrave row of the line-map block renamed",
        swap("A duet-engrave\nX duet-interchange", "A duet-engravex\nX duet-interchange"),
        1,
        ["CHUNK CRATE:"],
    ),
    (
        "PP40-unmapped",
        "one added chunk row whose line prefix the line-map block does not carry",
        swap("| A4 | 5 | The golden corpus",
             "| Z9 | 5 | A probe chunk | `crates/duet-dsp/src/probe.rs` |"
             " `cargo nextest run -p duet-dsp -E 'test(probe)' --no-tests=fail` |\n"
             "| A4 | 5 | The golden corpus"),
        1,
        ["CHUNK CRATE:"],
    ),
    (
        "PP38",
        "the two escaped pipes of chunk M0's write scope unescaped, which is the revision-21 shape",
        swap("loses the `\\|\\| printf` fallback", "loses the `|| printf` fallback"),
        1,
        ["RAGGED:     "],
    ),
    (
        "PP38-closure",
        "one cell deleted from a closure row of Appendix C",
        swap("| N21I-1 | the document header states revision 20 | **CLOSED** |",
             "| N21I-1 | the document header states revision 20 |"),
        1,
        ["RAGGED:     "],
    ),
    (
        "PP39",
        "the section 1.7 placement-guard range cut back to PG36",
        swap("| PG1 to PG41, with PG4b,", "| PG1 to PG36, with PG4b,"),
        1,
        ["INDEX:      "],
    ),
    (
        "PP39-stray",
        "PG44 added to the section 1.7 placement-guard range",
        swap("PG27b, and PG31b | The placement-guard rules |",
             "PG27b, PG31b, and PG44 | The placement-guard rules |"),
        1,
        ["INDEX:      "],
    ),
    (
        "PP41",
        "the thread cell of the EngineEvent carrier row renamed to a thread the 5.7 table omits",
        swap("| Engine handoff and Engine disk -> Core |",
             "| Engine handoffs and Engine disk -> Core |"),
        1,
        ["CARRIER:"],
    ),
    (
        "PP41-nocarrier",
        "a Sender cell that names a real field which holds no carrier end",
        swap("| `TempoMap` | `DuetCore.tempo` | `triple_buffer` | `EngineProcess.tempo` |",
             "| `TempoMap` | `DuetCore.tempo` and `DuetCore.version` | `triple_buffer` | `EngineProcess.tempo` |"),
        1,
        ["CARRIER:"],
    ),
    (
        "PP41-swapped",
        "the Sender cell and the Receiver cell of one carrier row swapped",
        swap("| `ParamSnapshot` | `DuetCore.params` | `triple_buffer` | `EngineProcess.params` |",
             "| `ParamSnapshot` | `EngineProcess.params` | `triple_buffer` | `DuetCore.params` |"),
        1,
        ["CARRIER:"],
    ),
    (
        "PP41-orphan",
        "a second Input<TempoMap> field added to DuetCore, which no carrier row names",
        swap("    /// Where the project lives, or `None` before the first open.\n    bundle: Option<PathBuf>,",
             "    probe: Input<TempoMap>,\n    /// Where the project lives, or `None` before the first open.\n    bundle: Option<PathBuf>,"),
        1,
        ["CARRIER:"],
    ),
    (
        "PP29b",
        "the PP11 cell of the PG11 row renamed to PP45",
        swap("| PG11 | An edge claim the 1.3 list does not carry | PP11 |", "| PG11 | An edge claim the 1.3 list does not carry | PP45 |"),
        1,
        ["PROBE:"],
    ),
]


def run_guard(path):
    """`(exit code, stdout)` of one guard run over one document path."""
    result = subprocess.run(
        [sys.executable, GUARD] + ([path] if path is not None else []),
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode, result.stdout


def shown(text, wanted):
    """The first line of the run that carries each wanted token."""
    lines = []
    for token in wanted:
        for line in text.splitlines():
            if token in line:
                lines.append(line.strip())
                break
    return lines


def recorded_cells(source):
    """Every probe id mapped to the `Recorded result` cell of its 1.9 row.

    PP25 is not here: `probe_roster.sh` runs it, and section 1.9 says so at
    the PG25 site.
    """
    rows, reason = placement_check.read_block(source, "probe-table")
    if reason is not None:
        return None
    cells = {}
    for row in rows:
        if len(row) < 5:
            continue
        for probe in re.findall(r"\bPP\d+[a-z]?\b", row[2]):
            cells[probe] = row[4]
    return cells


def recorded_for(cells, probe_id):
    """The probe id of the 1.9 row that covers one shape, or None.

    A row covers several shapes: the PG4 row records five and the PG27 row
    records one shape per registered block. The lookup therefore takes the
    longest recorded id that is a prefix of this shape's id.
    """
    if probe_id in cells:
        return probe_id
    best = None
    for probe in cells:
        if probe_id.startswith(probe) and (best is None or len(probe) > len(best)):
            best = probe
    return best


# The probe ids ANOTHER harness owns. PP25 needs a compiler and a scratch Cargo
# workspace, so `probe_roster.sh` runs it and `probe_roster_text.py` compares
# its recorded text. PP32 needs a review file, so `probe_closure.py` runs it
# and compares its own recorded lines. **Neither row is exempt from a
# compare**, which is the whole of critic C17-7: revision 17 skipped PP25 here
# and gave it to nobody, so the one row whose false record caused CR-13 and
# C16-1 was compared with nothing. Each id in this set names the harness that
# owns it in the rule's own site in section 1.5.
ROSTER_PROBES = {"PP25", "PP32"}


# The fragment oracle lives in `probe_fragments.py`, and every harness imports
# the same one. Revision 18 gave two harnesses their own filter and the
# narrower one skipped a counter line, so one cell was compared by one rule
# and one by another (critic C18-N6).


def fragment_missing(fragment, produced):
    """Whether one recorded fragment is absent from the produced text."""
    return missing(fragment, produced)


def text_failures(cells, produced):
    """Every recorded fragment that no run of the whole set produced.

    **It compares a fragment against its OWN row's output first** (critic
    N17-6). Revision 17 pooled every shape's output and asked only whether the
    fragment appeared somewhere in it, so a recorded line moved from the row
    that produces it to a row that does not was green. The pool is the
    fallback now, and a fragment that only the pool satisfies is reported with
    that reason rather than passed.

    **The rule's own limit, stated here**: a shape id is not always a prefix
    of its row's probe id, so a shape the lookup cannot attribute contributes
    to the pool alone. The row count and the compared count both print, so a
    compare that stops matching is visible in the denominator.

    Revision 16 decided pass or fail from the exit code and the LINE COUNT
    alone, so any text passed and one recorded cell was already stale: the
    PG27 row recorded a minimum of 33 while the guard's registered minimum
    was 37. A recorded result is a measurement, so the machine compares the
    measurement with the run.
    """
    failures, compared = [], 0
    pool = flatten("\n".join(produced.values()))
    own = {}
    for shape, text in produced.items():
        row = recorded_for(cells, shape)
        if row:
            own[row] = own.get(row, "") + "\n" + text
    for probe in sorted(cells):
        if probe in ROSTER_PROBES:
            continue
        mine = flatten(own.get(probe, ""))
        for fragment in recorded_fragments(cells[probe]):
            compared += 1
            if not fragment_missing(fragment, mine):
                continue
            if fragment_missing(fragment, pool):
                failures.append((probe, fragment, "no run produced it"))
            else:
                failures.append(
                    (probe, fragment, "a run of ANOTHER row produced it and this row did not")
                )
    return compared, failures


# PP27, fifth shape. One row per registered block that names no referent
# (WR-18). The row is data here for the reason the drop list is data in the
# document: a set inside the machine is a set no reader can check.
STRAY_ROWS = {
    "crate-table": "| `duet-probe` | A probe | Yes | `serde` |",
    "name-map": "ProbeName        probe-crate",
    "edge-list": "duet-probe           -> duet-time",
    "framework-types": "Ticks",
    "ownership-table": "| `duet-probe` | `ProbeMember` | 15.1 |",
    "drop-list": "Ticks",
    "budget-table": "| B999 | 1 | A probe | 5.5 |",
    "constants": "// duet-probe",
    "shared-limits": "MAX_PROBE duet-time duet-core",
    "probe-table": "| PG99 | A probe | PP99 | A probe | `exit 1` |",
    "rule-blocks": "| PG99 | `drop-list` |",
    "block-members": "probe-block         crate-name",
    "external-verdicts": "| `Ticks` | `Copy` | none | A probe |",
    "external-paths": "ProbeName     std::probe::ProbeName",
    "pins": "probe-crate    1.0.0",
    "recorded-sizes": "ProbeName                          8   8",
    "substitutions": "probe probe-name         a probe",
    "drop-impls": "ProbeName",
    "impl-sites": "ProbeName inherent",
    "heap-names": "Generation",
    "grow-names": "Generation",
    "lock-names": "Generation",
    "primitive-traits": "probe : Copy Default",
    "primitive-sizes": "probe 1 1",
    "justified-unknowns": "| A probe | 1 | `ProbeName` | A probe |",
    "vr1-table": "| `ProbeName` | A probe |",
    "audio-owned": "ProbeName",
    "audio-exempt": "ProbeName.probe  probe",
    "audio-reachable-leaf": "ProbeName       a probe leaf",
    "audio-asserted": "ProbeName       a probe root",
    "closure-r16": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r17": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r18": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r19": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r20": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r21-inner": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r21": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r22-inner": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "closure-r23-inner": "| PROBE | A probe | **CLOSED** | 1.9 |",
    "gate-defects": "| A probe | `ProbeName`, section 1.9 | `exit 1`; a probe |",
    "fault-messages": "| `ProbeFault` | `A probe line.` |",
    "line-map": "Z duet-probe",
    "phase-pair-exempt": "Z9  Y9  a probe pair",
    "b1-convert": "| `probe_only_here` | \"a probe\" |",
    "b1-complexity": "| `duet-probe::probe::probe_site` | \"a probe\" |",
    "b1-copy": "| `duet-probe::probe::ProbeName` | \"a probe\" |",
    "b1-variant": "| `duet-probe::probe::ProbeName` | \"a probe\" |",
    "phase-table": "| 14 | Z9 | none | none | 0 |",
    "selected-tests": "| probe_test | Z9 | 0 | `probe.rs` | Plain | A probe |",
    "snapshot-table": "| Audio | `duet-core` | `ProbeName` | Once per frame |",
    "carrier-table": (
        "| `ProbeName` | `ProbeName.probe` | `rtrb` at B1 | `ProbeName.probe` |"
        " Audio -> Core | none |"
    ),
}


def add_row(block_id, row):
    """Append one row with no referent to a registered block (PP27, WR-18)."""

    def apply(text):
        start, end, rows = block_span(text, block_id)
        return text[:end] + row + "\n" + text[end:]

    return apply


def duplicate_last_row(block_id):
    """PP27b. One extra row, with the floor left where it was.

    This is the revision-20 drift shape, measured: `LimiterState` entered the
    `audio-owned` block, the row count went to 38, and the `rows>=37` floor
    stayed, so one deletion passed as a clean run (critic C20-W10). The row is
    a COPY of a row the block already holds, so every membership rule still
    passes and the floor is the one thing that answers.
    """

    def apply(text):
        start, end, rows = block_span(text, block_id)
        last = text[:end].rstrip("\n").rsplit("\n", 1)[-1]
        return text[:end] + last + "\n" + text[end:]

    return apply


# The one block PP27b cannot probe by duplication. `block-members` maps each
# registered block to ONE membership kind, and a duplicated row makes that
# block carry two kinds, which is an exit-2 input failure that answers before
# PG27b can. Every other block takes the shape, and PG27b itself reads every
# block including this one, so the rule is unprobed at one site and enforced at
# all of them.
FLOOR_EXEMPT = {"block-members"}


def block_probes():
    """PP27 and PP27b. Six hostile shapes for every registered block."""
    shapes = []
    for block_id in placement_check.DATA_BLOCKS:
        shapes.append((f"PP27:{block_id}:deleted", drop_block(block_id)))
        shapes.append((f"PP27:{block_id}:emptied", empty_block(block_id)))
        shapes.append((f"PP27:{block_id}:unmarked", drop_marker(block_id)))
        shapes.append((f"PP27:{block_id}:renamed", rename_heading(block_id)))
        shapes.append((f"PP27:{block_id}:stray", add_row(block_id, STRAY_ROWS[block_id])))
        if block_id not in FLOOR_EXEMPT:
            shapes.append((f"PP27b:{block_id}:floor", duplicate_last_row(block_id)))
    return shapes


def main(argv):
    """Run the named probes, or every probe, and return an exit code."""
    if len(argv) < 2:
        sys.stdout.write("usage: probe_run.py <architecture.md> [probe id ...]\n")
        return 2
    document = argv[1]
    wanted = set(argv[2:])
    # A caller that names a probe this harness does not hold gets an error and
    # never a green run over an empty set (critic WARNING on the unknown id).
    known = {probe[0] for probe in PROBES} | {shape[0] for shape in block_probes()} | {"PP27"}
    unknown = sorted(name for name in wanted if name not in known)
    if unknown:
        sys.stdout.write(f"FAIL: no probe named {unknown[0]}; the harness is fail-closed.\n")
        return 2
    try:
        with open(document, encoding="utf-8") as handle:
            source = handle.read()
    except OSError:
        sys.stdout.write(f"FAIL: cannot open {document}\n")
        return 2

    code, text = run_guard(document)
    sys.stdout.write(f"{'BASE':22s} exit {code}  {'the unmodified document'}\n")
    if code != 0:
        sys.stdout.write("FAIL: the baseline run is not green, so no probe is meaningful.\n")
        return 1

    cells = recorded_cells(source)
    if cells is None:
        sys.stdout.write(
            "FAIL: the section 1.9 probe table does not read, so no recorded text can be"
            " compared; the harness is fail-closed.\n"
        )
        return 2

    bad = 0
    produced = {}
    with tempfile.TemporaryDirectory() as scratch:
        copy = os.path.join(scratch, "architecture.md")
        for probe_id, shape, mutate, expected, tokens in PROBES:
            if wanted and probe_id not in wanted:
                continue
            if mutate is None:
                code, text = run_guard(None)
            elif mutate == "MISSING":
                code, text = run_guard(os.path.join(scratch, "absent.md"))
            else:
                try:
                    with open(copy, "w", encoding="utf-8") as handle:
                        handle.write(mutate(source))
                except LookupError as broken:
                    sys.stdout.write(f"{probe_id:22s} PLANT FAILED  {broken}\n")
                    bad += 1
                    continue
                code, text = run_guard(copy)
            lines = shown(text, tokens)
            produced[probe_id] = text
            ok = code == expected and len(lines) == len(tokens)
            bad += 0 if ok else 1
            sys.stdout.write(
                f"{probe_id:22s} exit {code}  {'OK ' if ok else 'BAD'}  {shape}\n"
            )
            for line in lines:
                sys.stdout.write(f"{'':22s}       {line}\n")

        for probe_id, mutate in block_probes():
            if wanted and "PP27" not in wanted and probe_id not in wanted:
                continue
            with open(copy, "w", encoding="utf-8") as handle:
                handle.write(mutate(source))
            code, text = run_guard(copy)
            first = next(
                (
                    line
                    for line in text.splitlines()
                    if line.startswith("FAIL")
                    or line.strip().startswith("MEMBER:")
                    or line.strip().startswith("FLOOR:")
                ),
                "",
            )
            produced[probe_id] = text
            if probe_id.endswith(":stray"):
                # A row with no referent is red by membership, and a row that
                # also breaks its own parser is red at exit 2. Both are red.
                ok = code != 0 and bool(first)
            elif probe_id.endswith(":floor"):
                # PG27b is a FINDING and not a fail-closed input failure, so
                # the run exits 1 and names the block whose floor lags.
                ok = code == 1 and first.strip().startswith("FLOOR:")
            else:
                ok = code == 2 and first.startswith("FAIL")
            bad += 0 if ok else 1
            sys.stdout.write(f"{probe_id:44s} exit {code}  {'OK ' if ok else 'BAD'}  {first[:96]}\n")

    compared, stale = (0, []) if wanted else text_failures(cells, produced)
    for probe, fragment, reason in stale:
        sys.stdout.write(f"TEXT:  {probe}: {reason}: {fragment}\n")
    # The denominator floor (critic N17-5). Revision 17 printed the failure
    # count and never the compared count, so a regular expression that stopped
    # matching would read the same and the run would stay green. One recorded
    # line per row is the floor, and the row count is the source.
    floor = 0 if wanted else len([probe for probe in cells if probe not in ROSTER_PROBES])
    short = not wanted and compared < floor
    if short:
        sys.stdout.write(
            f"TEXT:  the harness compared {compared} recorded fragments and the"
            f" section 1.9 table holds {floor} rows it owns; the floor is one"
            " fragment per row and the harness is fail-closed.\n"
        )
    sys.stdout.write(
        f"PROBES BAD: {bad}     RECORDED FRAGMENTS: {compared}"
        f"     FLOOR: {floor}     RECORDED FRAGMENTS BAD: {len(stale)}\n"
    )
    return 1 if bad or stale or short else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
