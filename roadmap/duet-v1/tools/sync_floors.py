"""Write every guard-block floor from the row count the block itself holds.

PG27b refuses a floor below the row count, so a floor is a measurement and
never a hand number (critic C20-W10, C20-N5). Three files hold a copy of each
number: the `rows>=` marker inside `architecture.md`, the `DATA_BLOCKS`
register of `placement_check.py`, and the `DATA_BLOCKS` register of
`roster_compile.sh`. This script counts the rows of each registered block and
writes all three, so the three copies cannot drift and no reviewer has to run
one guard to learn what the other would say.

usage: sync_floors.py <architecture.md>

It exits 2 on a usage or input failure, and 0 when every floor is written. It
prints one line per block whose floor it changed and a final count.
"""

import re
import sys

import placement_check

MARKER = re.compile(r"(?m)^<!-- GUARD BLOCK id=([a-z0-9-]+) rows>=([0-9]+) -->$")
HERE = __file__.rsplit("/", 1)[0]


def counted(source, block_id):
    """The row count of one block, read with the register test switched off."""
    rows, reason = placement_check.read_block(source, block_id, register=False)
    if rows is None:
        return None, reason
    return len(rows), None


def write_register(path, counts):
    """Rewrite one `DATA_BLOCKS` register in place, and report the changes."""
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    changed = []
    for block_id, count in counts.items():
        pattern = re.compile(
            r'("' + re.escape(block_id) + r'": \("[^"]*", "[a-z]+", )([0-9]+)\)'
        )
        match = pattern.search(text)
        if match is None:
            continue
        if match.group(2) != str(count):
            changed.append(f"{path}: {block_id} {match.group(2)} -> {count}")
        text = pattern.sub(lambda m: m.group(1) + str(count) + ")", text, count=1)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    return changed


def main(argv):
    """Write every floor and return an exit code."""
    if len(argv) != 2:
        sys.stdout.write("usage: sync_floors.py <architecture.md>\n")
        return 2
    document = argv[1]
    try:
        with open(document, encoding="utf-8") as handle:
            source = handle.read()
    except OSError:
        sys.stdout.write(f"FAIL: cannot open {document}; the guard is fail-closed.\n")
        return 2

    counts, bad = {}, []
    for block_id in placement_check.DATA_BLOCKS:
        count, reason = counted(source, block_id)
        if count is None:
            bad.append(f"FAIL: the `{block_id}` block does not read: {reason}\n")
            continue
        counts[block_id] = count
    if bad:
        for line in bad:
            sys.stdout.write(line)
        return 2

    changed = []

    def replace(match):
        block_id = match.group(1)
        if block_id not in counts:
            return match.group(0)
        if match.group(2) != str(counts[block_id]):
            changed.append(f"{document}: {block_id} {match.group(2)} -> {counts[block_id]}")
        return f"<!-- GUARD BLOCK id={block_id} rows>={counts[block_id]} -->"

    source = MARKER.sub(replace, source)
    with open(document, "w", encoding="utf-8") as handle:
        handle.write(source)

    changed += write_register(f"{HERE}/placement_check.py", counts)
    changed += write_register(f"{HERE}/roster_compile.sh", counts)
    for line in changed:
        sys.stdout.write(line + "\n")
    sys.stdout.write(f"FLOORS WRITTEN: {len(counts)}     CHANGED: {len(changed)}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
