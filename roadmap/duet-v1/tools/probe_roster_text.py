"""Compare the recorded text of the section 1.9 PG25 row with a roster run.

`probe_roster.sh` runs the compiler, so `probe_roster.sh` owns this compare.
Revision 17 gave the recorded-text machine to `probe_run.py` and exempted
`PP25` by name, which is the one row whose false record caused critic CR-13
and critic C16-1. A reviewer then read `RECORDED FRAGMENTS BAD: 0` and
believed every recorded line in section 1.9 came from a run; thirteen of them
did not have to (critic C17-7).

usage: probe_roster_text.py <architecture.md> <output directory>

It reads every `.out` file the harness wrote, takes every fragment of the PG25
`Recorded result` cell that records a line of output, and prints the compared
count and the count that no run produced. It exits 1 when one is missing.

**It also compares the "planted gate defects" table** (critic C19-W8). That
table records one compiler line per plant and revision 19 left it registered
nowhere and compared by nothing, so it carried a recorded result no run had
produced: it stated `duet-engine/src/lib.rs:177:22` while the real compile
printed `:178:22`, which is the CR-13 and C16-W2 class alive in the one
section that exists to remove it. The table is a registered block now, and
every fragment of every row is compared against the run of its own shape.
"""

import os
import re
import sys

import placement_check
from probe_fragments import cell_of, flatten, missing, recorded_fragments


def main(argv):
    """Compare the PG25 cell with the run and return an exit code."""
    if len(argv) != 3:
        sys.stdout.write("usage: probe_roster_text.py <architecture.md> <output directory>\n")
        return 2
    document, directory = argv[1], argv[2]
    if not os.path.isfile(document) or not os.path.isdir(directory):
        sys.stdout.write("FAIL: cannot open the document or the output directory.\n")
        return 2
    with open(document, encoding="utf-8") as handle:
        source = handle.read()
    rows, reason = placement_check.read_block(source, "probe-table")
    if reason is not None:
        sys.stdout.write(f"FAIL: the section 1.9 probe table does not read: {reason}\n")
        return 2
    cell = cell_of(source, placement_check.read_block, "PP25")
    if not cell:
        sys.stdout.write("FAIL: the probe table carries no PG25 row; the harness is fail-closed.\n")
        return 2
    # One `.out` file per shape, so a fragment is compared against the shape
    # that produced it before the pool is asked. Revision 18 pooled every
    # shape, which is the defect the same revision removed from
    # `probe_run.py` (critic C18-W5).
    own, shapes = {}, []
    for name in sorted(os.listdir(directory)):
        if not name.endswith(".out"):
            continue
        with open(os.path.join(directory, name), encoding="utf-8", errors="replace") as handle:
            own[name[: -len(".out")]] = handle.read()
        shapes.append(name[: -len(".out")])
    pool = flatten("\n".join(own.values()))
    bad_transcript = []
    # The section 1.9 roster TRANSCRIPT, line by line (critic C18-2). It is the
    # fenced block that follows the anchor sentence, and it records the run
    # this script performs, so it belongs to this comparer.
    anchor = "The roster compile on this revision prints"
    transcript = []
    if anchor not in source:
        sys.stdout.write(
            f"FAIL: the document states no `{anchor}` sentence, so the transcript"
            " compare has no input; the harness is fail-closed (critic C19-1).\n"
        )
        return 2
    if True:
        rest = source[source.index(anchor):]
        opened = rest.index("```") + 3
        closed = rest.index("```", opened)
        transcript = [
            flatten(line)
            for line in rest[opened:closed].strip().splitlines()
            if flatten(line) and flatten(line) != "EXIT=0"
        ]
    base = own.get("base", "")
    for line in transcript:
        if missing(line, flatten(base)):
            bad_transcript.append((line, "the baseline roster run does not print it"))
    # The planted gate-defect table (critic C19-W8). Each row records one
    # compiler line, and the harness runs one shape per row, so the same
    # attribution rule applies: the shape whose own output holds the line is
    # the answer, and a line no shape holds is a line no run produced.
    gate_rows, gate_reason = placement_check.read_block(source, "gate-defects")
    if gate_reason is not None:
        sys.stdout.write(
            f"FAIL: the `gate-defects` block of this document: {gate_reason};"
            " the harness is fail-closed.\n"
        )
        return 2
    gate_fragments = []
    for row in gate_rows:
        if len(row) >= 3:
            gate_fragments.extend(recorded_fragments(row[2]))
    bad_gate = []
    for piece in gate_fragments:
        holders = [name for name, text in own.items() if not missing(piece, flatten(text))]
        if holders:
            continue
        bad_gate.append(
            (piece, "no run produced it" if missing(piece, pool) else
             "a run of ANOTHER shape produced it and no shape of this row did")
        )
    gate_floor = len(gate_rows)
    gate_short = len(gate_fragments) < gate_floor

    fragments = recorded_fragments(cell)
    bad = []
    for piece in fragments:
        # The cell records one line per named shape, so the shape whose own
        # output holds the line is the attribution. A line no shape holds is a
        # line no run produced.
        holders = [name for name, text in own.items() if not missing(piece, flatten(text))]
        if holders:
            continue
        if missing(piece, pool):
            bad.append((piece, "no run produced it"))
        else:
            bad.append((piece, "a run of ANOTHER shape produced it and no shape of this row did"))
    # The floor (critic C18-W4, C19-1). "Compared 0 and found 0" and "compared
    # 8 and found 0" print almost the same line and mean opposite things, and
    # the filter that selects an output fragment is two literal tests. One
    # recorded line per planted shape is the floor, and the shape count is the
    # source; the baseline run is not a planted shape.
    #
    # Revision 19 subtracted a hard-coded 10 from that count, which put the
    # floor at 4 against 14 planted shapes: six recorded lines could leave the
    # PG25 cell and the harness stayed green. The two sibling harnesses,
    # `probe_run.py` and `probe_closure.py`, each use one line per shape with
    # no discount, and this one now does the same.
    floor = len([name for name in shapes if name != "base"])
    short = len(fragments) < floor
    sys.stdout.write(
        f"ROSTER FRAGMENTS: {len(fragments)}     FLOOR: {floor}"
        f"     FRAGMENTS BAD: {len(bad)}"
        f"     TRANSCRIPT LINES: {len(transcript)}     TRANSCRIPT BAD: {len(bad_transcript)}\n"
    )
    sys.stdout.write(
        f"GATE FRAGMENTS:   {len(gate_fragments)}     FLOOR: {gate_floor}"
        f"     GATE BAD: {len(bad_gate)}\n"
    )
    for piece, reason in bad_gate:
        sys.stdout.write(f"TEXT:  gate-defects: {reason}: {piece}\n")
    if gate_short:
        sys.stdout.write(
            f"TEXT:  gate-defects: the table records {len(gate_fragments)} output lines"
            f" and the floor is {gate_floor}; the harness is fail-closed.\n"
        )
    for line, reason in bad_transcript:
        sys.stdout.write(f"TEXT:  1.9 transcript: {reason}: {line}\n")
    for piece, reason in bad:
        sys.stdout.write(f"TEXT:  PP25: {reason}: {piece}\n")
    if short:
        sys.stdout.write(
            f"TEXT:  PP25: the cell records {len(fragments)} output lines and the floor"
            f" is {floor}; the harness is fail-closed.\n"
        )
    return 1 if bad or short or bad_transcript or bad_gate or gate_short else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
