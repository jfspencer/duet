"""One recorded-fragment oracle, shared by every probe harness.

Three harnesses compare a section 1.9 `Recorded result` cell with their own
runs, and revision 18 gave two of them their own filter. The narrower one
skipped a counter line such as `ROSTER CRATES:   17`, so one cell was compared
by one rule and one by another (critic C18-N6). One filter, one place.

A recorded fragment is guard OUTPUT when it opens with a head a guard prints:
a `FAIL` line, a `usage:` line, an upper-case label followed by a colon, or a
compiler line, which always carries `: error`.
"""

import re
import sys

OUTPUT_HEAD = re.compile(r"^(FAIL|usage:|[A-Z][A-Z0-9 ]*:)")
FRAGMENT = re.compile(r"``(.+?)``|`([^`]+)`", re.DOTALL)


def flatten(text):
    """One line with every whitespace run collapsed, for a text compare."""
    return " ".join(text.split())


def is_output(piece):
    """Whether one fragment records a line of guard output."""
    return bool(OUTPUT_HEAD.match(piece)) or ": error" in piece


def recorded_fragments(cell):
    """Every fragment of one cell that records a line of guard output."""
    found = []
    for double, single in FRAGMENT.findall(cell):
        piece = flatten(double or single)
        if is_output(piece):
            found.append(piece)
    return found


def missing(fragment, produced):
    """Whether one recorded fragment is absent from the produced text.

    `...` inside a cell stands for text the row does not repeat, such as a
    temporary path, so a fragment matches when each of its parts appears in
    order. Every other character must match, which is what makes a stale
    number red.
    """
    position = 0
    for part in fragment.split("..."):
        part = part.strip()
        if not part:
            continue
        found = produced.find(part, position)
        if found < 0:
            return True
        position = found + len(part)
    return False


def cell_of(source, read_block, probe):
    """The `Recorded result` cell of the section 1.9 row that covers one probe."""
    rows, reason = read_block(source, "probe-table")
    if reason is not None:
        return None
    for row in rows:
        if len(row) >= 5 and re.search(r"\b" + re.escape(probe) + r"\b", row[2]):
            return row[4]
    return None


def main(argv):
    """Refuse a direct run and name the harnesses that import this module.

    An exit code of 0 with no output is indistinguishable from a passing
    check, and a status line that records `exit 0` for it asserts nothing
    (critic C20-N6). This module is a library, so a direct run is a usage
    failure and says so.
    """
    del argv
    sys.stdout.write(
        "FAIL: probe_fragments.py is a library; run probe_run.py,"
        " probe_roster.sh, or probe_closure.py.\n"
    )
    return 2


if __name__ == "__main__":
    sys.exit(main(sys.argv))
