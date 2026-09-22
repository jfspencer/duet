"""Run every guard and every harness of this plan, in order, over ONE tree.

**Every recorded result in this plan must come from a run of the FINAL tree,
after the last edit.** Revision 20 ran a harness once, kept editing, and
recorded the earlier run, and the twentieth Critic measured both the stale
probe transcript and the stale roster transcript (critic C20-1, C20-2). That
defect class is already in the closure appendix three times: CR-13, C16-1 and
C18-2. The remedy is a machine and not a habit.

This script is that machine. It records the modification time of every file
under `roadmap/duet-v1/` before it starts, runs every guard and every harness
in a fixed order, and then reads those times again. **A file that changed
during the run makes the whole run a failure**, so a transcript this script
prints is a transcript of one tree and never of two.

usage: run_all_gates.py <architecture.md> <scratch-dir> <repo-root> [--fast]

`--fast` skips the two roster runs, which need a compiler and several
minutes. It is for an authoring loop and never for a recorded result, and the
final line says which mode ran.

**The closure gate list is DERIVED from `reviews/` and never typed** (critic
C21-W3). Revision 21 held it as `for revision in ("r16", ..., "r21-inner")`,
so a new review file got no closure gate and the run still printed
`GATES BAD: 0`; the twenty-first Critic copied one review to
`critic-spec-r22.md` and the harness was green over it. The script now reads
`reviews/critic-spec-*.md`, maps each file to the block `closure-<stem>`, and
**exits 2 when a review file has no registered block and when a registered
closure block has no review file**. Section 1.9's own words apply: a habit
cannot hold that rule and a script can.

**It refuses a document outside the plan directory** (critic N21-6). The
tree compare covers `roadmap/duet-v1/` and a document argument that points
elsewhere is a run of one tree against another. **It flushes stdout after
every gate** (critic N21-5), so a redirected run prints as it goes and a
reader can tell a slow gate from a hang.

Exit codes: 0 when every gate is green and nothing changed under the plan
directory; 1 when a gate is red; 2 on a usage failure, an input failure, or a
tree that changed while the run was in progress.
"""

import os
import re
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
PLAN = os.path.dirname(HERE)


def tree_times(root):
    """Every file under one directory, mapped to its modification time.

    `__pycache__` is excluded, because a Python run writes bytecode there and
    that write is not an edit to the plan (critic C20-N7). Every documented
    command sets `PYTHONDONTWRITEBYTECODE=1`, and this script sets it for
    every child it starts, so the directory should not appear at all.
    """
    found = {}
    for base, directories, files in os.walk(root):
        directories[:] = [name for name in directories if name != "__pycache__"]
        for name in files:
            path = os.path.join(base, name)
            try:
                found[path] = os.stat(path).st_mtime_ns
            except OSError:
                found[path] = None
    return found


def closure_pairs(reviews, document):
    """Every review file of `reviews/`, held against the registered blocks.

    It returns `(revisions, None)` or `(None, reason)`.

    **The reverse half is absolute**: a `closure-*` block with no review file
    is a block bound to nothing, and that is a refusal whatever the revision.

    **The forward half opens at the earliest REGISTERED revision.** Appendix C
    carries one section per review from C.1 and registers a block only for the
    recent ones, so the reviews of revisions 2 to 15 have rows and no block by
    design. A review at or above the earliest registered revision with no
    block of its own is a refusal, which is the hole probe G3 opened: one new
    `critic-spec-r22.md` in the tree and the harness was green with no closure
    gate for it (critic C21-W3).
    """
    if not os.path.isdir(reviews):
        return None, f"{reviews} is no directory, so the closure gate list is unknown"
    found = {}
    for name in sorted(os.listdir(reviews)):
        match = re.fullmatch(r"critic-spec-(r[0-9][A-Za-z0-9-]*)\.md", name)
        if match:
            found[match.group(1)] = name
    if not found:
        return None, f"{reviews} holds no `critic-spec-r<n>.md` file, so the denominator is zero"
    try:
        with open(document, encoding="utf-8") as handle:
            source = handle.read()
    except OSError as error:
        return None, f"cannot read {document}: {error}"
    registered = set(re.findall(r"<!--\s*GUARD BLOCK id=closure-([A-Za-z0-9-]+)", source))
    if not registered:
        return None, "the document registers no `closure-*` block, so the denominator is zero"
    numbers = [int(re.sub(r"[^0-9].*$", "", name[1:])) for name in registered]
    floor = min(numbers)
    for revision, name in sorted(found.items()):
        if int(re.sub(r"[^0-9].*$", "", revision[1:])) < floor:
            continue
        if revision not in registered:
            return None, (
                f"`reviews/{name}` has no registered `closure-{revision}` block, so no"
                f" closure gate would run for it; the earliest registered revision is {floor}"
            )
    for block in sorted(registered):
        if block not in found:
            return None, (
                f"the document registers `closure-{block}` and `reviews/` holds no"
                f" `critic-spec-{block}.md`, so that block is bound to no file"
            )
    return sorted(name for name in found if name in registered), None


def run(label, command, cwd):
    """One gate, as `(label, exit code, output)`."""
    environment = dict(os.environ)
    environment["PYTHONDONTWRITEBYTECODE"] = "1"
    started = time.monotonic()
    result = subprocess.run(
        command,
        cwd=cwd,
        capture_output=True,
        text=True,
        check=False,
        env=environment,
    )
    took = time.monotonic() - started
    return label, result.returncode, result.stdout + result.stderr, took


def main(argv):
    """Run every gate over one tree and return an exit code."""
    fast = "--fast" in argv
    argv = [item for item in argv if item != "--fast"]
    if len(argv) != 4:
        sys.stdout.write(
            "usage: run_all_gates.py <architecture.md> <scratch-dir> <repo-root>"
            " [--fast]\n"
        )
        return 2
    document, scratch, repo = os.path.abspath(argv[1]), argv[2], os.path.abspath(argv[3])
    for path in (document, os.path.join(repo, "Cargo.toml")):
        if not os.path.isfile(path):
            sys.stdout.write(f"FAIL: cannot open {path}; the harness is fail-closed.\n")
            return 2
    if os.path.dirname(document) != PLAN:
        sys.stdout.write(
            f"FAIL: the document must sit in {PLAN}; this run was handed {document}, so the"
            " tree this harness time-checks is not the tree it reads;"
            " the harness is fail-closed.\n"
        )
        return 2
    if os.path.abspath(scratch).startswith(os.path.abspath(repo) + os.sep):
        sys.stdout.write(
            "FAIL: the scratch directory must sit outside the repository;"
            " the harness is fail-closed.\n"
        )
        return 2
    os.makedirs(scratch, exist_ok=True)

    before = tree_times(PLAN)

    reviews = os.path.join(PLAN, "reviews")
    revisions, reason = closure_pairs(reviews, document)
    if revisions is None:
        sys.stdout.write(f"FAIL: {reason}; the harness is fail-closed.\n")
        return 2
    gates = [
        ("placement_check", ["python3", "tools/placement_check.py", document], PLAN),
        ("probe_run", ["python3", "tools/probe_run.py", document], PLAN),
        ("probe_closure", ["python3", "tools/probe_closure.py", document, repo], PLAN),
        (
            "conversion_check",
            ["python3", os.path.join(HERE, "conversion_check.py")],
            repo,
        ),
        (
            "probe_conversion",
            ["python3", os.path.join(HERE, "probe_conversion.py"), document],
            repo,
        ),
    ]
    for revision in revisions:
        review = os.path.join(reviews, f"critic-spec-{revision}.md")
        gates.append(
            (
                f"closure_check {revision}",
                [
                    "python3",
                    "tools/closure_check.py",
                    document,
                    review,
                    f"closure-{revision}",
                    repo,
                ],
                PLAN,
            )
        )
    if not fast:
        gates.append(
            (
                "roster_compile",
                [
                    "bash",
                    "tools/roster_compile.sh",
                    document,
                    os.path.join(scratch, "roster"),
                    repo,
                ],
                PLAN,
            )
        )
        gates.append(
            (
                "probe_roster",
                [
                    "bash",
                    "tools/probe_roster.sh",
                    document,
                    os.path.join(scratch, "probe"),
                    repo,
                ],
                PLAN,
            )
        )

    bad, transcript = 0, []
    for label, command, cwd in gates:
        name, code, text, took = run(label, command, cwd)
        transcript.append((name, code, text))
        state = "OK " if code == 0 else "RED"
        sys.stdout.write(f"=== {name:22s} exit {code}  {state}  {took:6.1f}s\n")
        sys.stdout.write(text if text.endswith("\n") or not text else text + "\n")
        # A redirected run prints nothing for six minutes without this, and a
        # reader cannot tell a slow gate from a hang (critic N21-5).
        sys.stdout.flush()
        if code != 0:
            bad += 1

    after = tree_times(PLAN)
    moved = sorted(
        path
        for path in set(before) | set(after)
        if before.get(path) != after.get(path)
    )
    if moved:
        sys.stdout.write(
            f"FAIL: {len(moved)} file(s) under {PLAN} changed while this run was in"
            " progress, so no line above is a measurement of one tree;"
            " the harness is fail-closed.\n"
        )
        for path in moved[:10]:
            sys.stdout.write(f"  MOVED: {path}\n")
        return 2

    mode = "FAST (no roster)" if fast else "FULL"
    sys.stdout.write(f"GATES RUN: {len(gates)}     GATES BAD: {bad}     MODE: {mode}\n")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
