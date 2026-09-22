"""Run PP32, the closure probe of architecture section 1.9.

PG32 needs two files: this document and a review file that sits outside the
repository. A probe that read a real review would depend on a path no future
reader has, so each shape here writes its OWN throwaway review and its own
throwaway copy of the document, plants one defect, and asserts the exit code
and the recorded line.

**It reads the section 1.9 PP32 row and compares every recorded output line
with its own runs** (critic C18-5). Revision 18 compared each shape's first
line with a literal inside this file's own `SHAPES` table and never opened the
document, so a false line planted in the PP32 cell was green in all three
harnesses. That is C17-7 with a new exempt row.

usage: probe_closure.py <architecture.md> <repo-root> [probe id ...]

`repo-root` is the repository that holds `.claude/plan-coordination/db.sh`.
It is EXPLICIT and never derived, because a FROZEN copy of this plan sits
outside the repository and a harness that guessed would report a store the
guard could not reach as a defect of the guard (critic C22I-1).
`run_all_gates.py` passes its own `<repo-root>` argument through.

It exits 0 when the baseline is green, every planted shape is red, and every
recorded output line of the PP32 cell appears in the run of its own shape. It
exits 1 when one is not, and 2 on a usage or input failure.
"""

import hashlib
import os
import subprocess
import sys
import tempfile

import closure_check
import placement_check
from probe_fragments import cell_of, flatten, missing, recorded_fragments

HERE = os.path.dirname(os.path.abspath(__file__))
GUARD = os.path.join(HERE, "closure_check.py")

# One throwaway review with two Criticals, one Warning and one Concern. The
# generator reads the headings, so the file needs no body.
REVIEW = """# Engineering Critic, specification review, revision 99

**Counts: 2 Criticals, 1 Warnings, 1 Concerns.**

## CRITICAL 1. A probe

## CRITICAL 2. A probe

## WARNING 1. A probe

## CONCERN 1. A probe
"""

SENTENCE = (
    "It returned 2 Criticals, 1 Warnings, and 1 Concerns,"
    " and this block holds one row for each of the 4."
)

# The throwaway document carries the one heading its rows cite and the digest
# sentence that binds the block to the review this harness writes. Both are
# rules of the guard, so the baseline has to satisfy them.
DIGEST = hashlib.md5(REVIEW.encode("utf-8")).hexdigest()
CONTENT_DIGEST = hashlib.md5(REVIEW.rstrip("\n").encode("utf-8")).hexdigest()


def store_the_review(repo, suffix, text=None):
    """Append one throwaway review to the plan store and return its key.

    CL1c compares a review file with the copy the store holds, so a probe of
    CL1c needs a real key in the real store. The harness appends its own
    throwaway review under the `probe-closure` suffix, which is one record of
    about 300 bytes per run in an append-only keyspace, and then plants two
    shapes against it. **A probe that mocked the store would prove the mock**,
    which is the class C18-5 already cost this plan one revision.

    **The suffix is unique to this run** (critic C22I-W5). CL1c reads every
    stored copy of one suffix now and fails when two copies differ, so a
    shape that appends a SECOND, doctored copy has to own its suffix or it
    would leave every later baseline red. The review file this harness writes
    is named `critic-spec-<rev>.md` for the same run token, because the file
    name is what decides the suffix a document may name.
    """
    launcher = os.path.join(repo, ".claude", "plan-coordination", "db.sh")
    if not os.path.isfile(launcher):
        return None, f"the store launcher {launcher} does not open"
    result = subprocess.run(
        ["bash", launcher, "append", closure_check.PLAN_NAME, suffix, text or REVIEW],
        cwd=repo,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode != 0:
        return None, result.stderr.strip() or f"`db.sh append` exited {result.returncode}"
    key = result.stdout.strip()
    return (key, None) if key else (None, "`db.sh append` printed no key")


# The key the store minted for this run, the run token that makes its suffix
# unique, and the key of the second, doctored copy the re-append shape writes.
# All three are built in `main` from the repository the caller names, so an
# import of this module writes nothing to the store.
STORE_KEY = None
RUN_TOKEN = None

# A review with DIFFERENT bytes and the same headings. The re-append shape
# stores it beside the real copy, so CL1c meets two copies of one suffix that
# disagree, which is the attack revision 22's rule could not see.
DOCTORED = REVIEW.replace("## CONCERN 1. A probe", "## CONCERN 1. A doctored probe")
DOCTORED_DIGEST = hashlib.md5(DOCTORED.rstrip("\n").encode("utf-8")).hexdigest()
REAPPEND_KEY = None


def block_text(store_key):
    """The throwaway block, which records the key the store minted."""
    return """### 1.9 A probe section

#### The revision-16 review, over the frozen document

""" + SENTENCE + """

The review file this block records has md5 `""" + DIGEST + """` (closure-r16).
The review file this block records is stored at plan-store key `""" + store_key + """` (closure-r16).

<!-- GUARD BLOCK id=closure-r16 rows>=4 -->
| Id | Finding | State | Section and mechanism |
|---|---|---|---|
| C99-1 | A probe | **CLOSED** | 1.9 |
| C99-2 | A probe | **CLOSED** | 1.9 |
| C99-W1 | A probe | **CLOSED** | 1.9 |
| N99-1 | A probe | **CLOSED** | 1.9 |
"""


# One live key of the plan store that holds a DIFFERENT review, which is the
# second CL1c shape: a key that resolves and that names the wrong file.
OTHER_KEY = "36304kihouge4-review-r21-inner"
OTHER_DIGEST = "18be2cdff62499156b1606119bb86a05"


def shape_nokey(block):
    """The plan-store key sentence deleted."""
    return "\n".join(
        line for line in block.splitlines()
        if "is stored at plan-store key" not in line
    ) + "\n"


def shape_otherkey(block):
    """The plan-store key changed to a live key of ANOTHER review's suffix.

    This is the coordinated edit CL1c exists to refuse: the Architect appends
    a doctored review under a suffix of its own and points one sentence of the
    closure section at it. The review FILE NAME decides the suffix, so the
    sentence cannot reach a key outside it (critic C22I-W5).
    """
    return block.replace("`" + STORE_KEY + "`", "`" + OTHER_KEY + "`")


def document_with(block):
    """One throwaway document that holds only the block PG32 reads."""
    return block


def shape_missing(block):
    """One generated id with no row."""
    return block.replace("| C99-2 | A probe | **CLOSED** | 1.9 |\n", "").replace(
        "rows>=4", "rows>=3"
    )


def shape_extra(block):
    """One row for a finding the review does not state."""
    return block.replace(
        "| N99-1 | A probe | **CLOSED** | 1.9 |",
        "| N99-1 | A probe | **CLOSED** | 1.9 |\n| C99-9 | A probe | **CLOSED** | 1.9 |",
    ).replace("rows>=4", "rows>=5")


def shape_count(block):
    """A count sentence the review does not generate."""
    return block.replace("It returned 2 Criticals", "It returned 1 Criticals")


def shape_nosection(block):
    """A CLOSED row that names no section."""
    return block.replace("| C99-1 | A probe | **CLOSED** | 1.9 |", "| C99-1 | A probe | **CLOSED** |  |")


def shape_digest(block):
    """A digest sentence that names a file this run did not read."""
    return block.replace(DIGEST, "0" * 32)


def shape_state(block):
    """A state word outside the three the guard accepts."""
    return block.replace("| C99-2 | A probe | **CLOSED** | 1.9 |", "| C99-2 | A probe | **PENDING** | 1.9 |")


def shape_blank(block):
    """A row emptied of everything it says."""
    return block.replace("| C99-W1 | A probe | **CLOSED** | 1.9 |", "| C99-W1 |  |  |  |")


def shape_section(block):
    """A CLOSED row that names a section no heading of the document states."""
    return block.replace("| N99-1 | A probe | **CLOSED** | 1.9 |", "| N99-1 | A probe | **CLOSED** | 99.99 |")


def shape_duplicate(block):
    """One row duplicated, which CL2's set compare could not see (C19-W3)."""
    row = "| C99-W1 | A probe | **CLOSED** | 1.9 |"
    return block.replace(row, row + "\n| C99-W1 | A probe | **OPEN** | 1.9 |")


def shape_prose(block):
    """A CLOSED row whose section cell names no section at all (C19-W1)."""
    return block.replace(
        "| C99-1 | A probe | **CLOSED** | 1.9 |",
        "| C99-1 | A probe | **CLOSED** | fixed in the tool |",
    )


def shape_moved(block):
    """The digest sentence moved out of the block's own section (C19-W2)."""
    line = [part for part in block.splitlines() if "md5" in part][0]
    return block.replace(line + "\n", "") + "\n### A section far away\n\n" + line + "\n"


def review_hidden(review):
    """A Warning the parser cannot see, beside a count sentence that states it.

    Revision 20 planted `**W2.**`, which is the nineteenth review's own shape.
    Revision 21 READS that shape, because `reviews/` is append-only and a
    stored review cannot be reformatted, so the plant moved to a shape no form
    of the parser recognises: a `### Warning two` sub-heading in prose.

    **The property under test is unchanged** (critic C19-2, C19-3). The count
    sentence states one Warning more than the scan finds, so CL1b refuses the
    run where CL1 alone would have generated a short list and let the appendix
    record a complete closure over that fraction. A fourth heading shape
    nobody declared is still a refused run.
    """
    return review.replace(
        "**Counts: 2 Criticals, 1 Warnings, 1 Concerns.**",
        "**Counts: 2 Criticals, 2 Warnings, 1 Concerns.**",
    ) + "\n### Warning two\n\nA probe.\n"


REVIEW_SHAPES = [
    ("PP32-format", "a Warning in a shape no form of the parser reads, which the count sentence contradicts",
     review_hidden, 2,
     "FAIL: ...: the review states 2 Warnings and the heading scan finds 1; a heading"
     " this parser cannot see is the one failure a second source exists to catch"),
]


SHAPES = [
    ("PP32-missing", "one generated id with no row", shape_missing, 1,
     "CLOSURE:   C99-2: the review states it and the block holds no row"),
    ("PP32-extra", "one row for a finding the review does not state", shape_extra, 1,
     "CLOSURE:   C99-9: the block holds a row and the review states no such finding"),
    ("PP32-count", "a count sentence the review does not generate", shape_count, 1,
     "COUNT:     the document does not hold the count sentence this review generates: "
     + SENTENCE),
    ("PP32-nosection", "a CLOSED row that names no section", shape_nosection, 1,
     "CLOSURE:   C99-1: the row says CLOSED and names no section"),
    ("PP32-digest", "a digest sentence that names a file this run did not read", shape_digest, 1,
     "DIGEST:    the document states no md5 `" + DIGEST + "` for closure-r16 beside the"
     " block, so nothing binds this block to the file this run read"),
    ("PP32-state", "a state word outside the three the guard accepts", shape_state, 1,
     "CLOSURE:   C99-2: the state is `PENDING` and the three states are CLOSED, PARTIAL, OPEN"),
    ("PP32-blank", "a row emptied of everything it says", shape_blank, 1,
     "CLOSURE:   C99-W1: the row states no finding"),
    ("PP32-section", "a CLOSED row naming a section no heading states", shape_section, 1,
     "CLOSURE:   N99-1: the row names section 99.99 and no heading of this document states it"),
    ("PP32-duplicate", "one row duplicated, with the state flipped", shape_duplicate, 1,
     "CLOSURE:   C99-W1: the block holds 2 rows and PG32 states one closure row per finding"),
    ("PP32-prose", "a CLOSED row whose section cell names no section at all", shape_prose, 1,
     "CLOSURE:   C99-1: the row says CLOSED and its section cell names no section of this"
     " document"),
    ("PP32-moved", "the digest sentence moved out of the block's own section", shape_moved, 1,
     "DIGEST:    the document states no md5 `" + DIGEST + "` for closure-r16 beside the"
     " block, so nothing binds this block to the file this run read"),
    ("PP32-nokey", "the plan-store key sentence deleted, which leaves CL1c with no second source",
     shape_nokey, 1,
     "STORE:     the section states no plan-store key for closure-r16, so nothing binds"
     " this block to a copy outside the Architect's write scope (CL1c)"),
]


def suffix_shapes():
    """The two CL1c shapes that need the run token (critic C22I-W5).

    `PP32-reappend` runs LAST and plants nothing in the document at all: the
    defect is a second, doctored copy of the review in the STORE, under the
    suffix the review file name decides. That is the coordinated edit the
    twenty-second review found, and it needs no rewrite of any stored key.
    """
    return [
        ("PP32-otherkey",
         "the plan-store key changed to a live key of another review's suffix",
         shape_otherkey, 1,
         "STORE:     the section names key `" + OTHER_KEY + "`, whose suffix is not"
         " `review-" + RUN_TOKEN + "`; the review file name decides the suffix and the"
         " Architect does not (CL1c)"),
        ("PP32-reappend",
         "a SECOND, doctored copy of the review appended under the same suffix",
         None, 1,
         "STORE:     1 of 2 stored copies of this review differ from the file this run"
         " read, `" + CONTENT_DIGEST + "`; the first is key `" + str(REAPPEND_KEY) + "`"
         " at md5 `" + DOCTORED_DIGEST + "` (CL1c)"),
    ]


def run(document, review, repo):
    """`(exit code, stdout)` of one guard run over one pair of files."""
    result = subprocess.run(
        [sys.executable, GUARD, document, review, "closure-r16", repo],
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode, result.stdout


def main(argv):
    """Run the baseline and every shape, and return an exit code."""
    global STORE_KEY, RUN_TOKEN, REAPPEND_KEY
    if len(argv) < 3:
        sys.stdout.write("usage: probe_closure.py <architecture.md> <repo-root> [probe id ...]\n")
        return 2
    if not os.path.isfile(argv[1]):
        sys.stdout.write(f"FAIL: cannot open {argv[1]}; the harness is fail-closed.\n")
        return 2
    repo = os.path.abspath(argv[2])
    RUN_TOKEN = "r99" + os.urandom(6).hex()
    STORE_KEY, store_reason = store_the_review(repo, "review-" + RUN_TOKEN)
    if STORE_KEY is None:
        sys.stdout.write(
            f"FAIL: the plan store did not take the throwaway review: {store_reason};"
            " CL1c cannot be probed and the harness is fail-closed.\n"
        )
        return 2
    block = block_text(STORE_KEY)
    with open(argv[1], encoding="utf-8") as handle:
        document = handle.read()
    cell = cell_of(document, placement_check.read_block, "PP32")
    if cell is None:
        sys.stdout.write(
            "FAIL: the section 1.9 probe table carries no PP32 row, so no recorded text"
            " can be compared; the harness is fail-closed.\n"
        )
        return 2
    wanted = set(argv[3:])
    produced = {}
    bad = 0
    with tempfile.TemporaryDirectory() as scratch:
        review = os.path.join(scratch, f"critic-spec-{RUN_TOKEN}.md")
        with open(review, "w", encoding="utf-8") as handle:
            handle.write(REVIEW)
        base = os.path.join(scratch, "base.md")
        with open(base, "w", encoding="utf-8") as handle:
            handle.write(document_with(block))
        code, text = run(base, review, repo)
        sys.stdout.write(f"{'BASE':16s} exit {code}  the unmodified block\n")
        if code != 0:
            sys.stdout.write("FAIL: the baseline run is not green, so no shape is meaningful.\n")
            sys.stdout.write(text)
            return 1
        for probe_id, description, mutate, expected, recorded in SHAPES:
            if wanted and probe_id not in wanted:
                continue
            copy = os.path.join(scratch, probe_id + ".md")
            with open(copy, "w", encoding="utf-8") as handle:
                handle.write(document_with(mutate(block)))
            code, text = run(copy, review, repo)
            line = next(
                (
                    one.strip()
                    for one in text.splitlines()
                    if one.strip().startswith(("CLOSURE:", "COUNT:", "DIGEST:", "STORE:"))
                ),
                "",
            )
            produced[probe_id] = text
            ok = code == expected and " ".join(line.split()) == " ".join(recorded.split())
            bad += 0 if ok else 1
            sys.stdout.write(
                f"{probe_id:16s} exit {code}  {'OK ' if ok else 'BAD'}  {description}\n"
            )
            sys.stdout.write(f"{'':16s}        {line}\n")
            if not ok:
                sys.stdout.write(f"{'':16s}        expected exit {expected}: {recorded}\n")
        for probe_id, description, mutate, expected, recorded in suffix_shapes()[:1]:
            if wanted and probe_id not in wanted:
                continue
            copy = os.path.join(scratch, probe_id + ".md")
            with open(copy, "w", encoding="utf-8") as handle:
                handle.write(document_with(mutate(block)))
            code, text = run(copy, review, repo)
            line = next(
                (one.strip() for one in text.splitlines() if one.strip().startswith("STORE:")),
                "",
            )
            produced[probe_id] = text
            ok = code == expected and " ".join(line.split()) == " ".join(recorded.split())
            bad += 0 if ok else 1
            sys.stdout.write(
                f"{probe_id:16s} exit {code}  {'OK ' if ok else 'BAD'}  {description}\n"
            )
            sys.stdout.write(f"{'':16s}        {line}\n")
            if not ok:
                sys.stdout.write(f"{'':16s}        expected exit {expected}: {recorded}\n")
        if not wanted or "PP32-reappend" in wanted:
            REAPPEND_KEY, reappend_reason = store_the_review(
                repo, "review-" + RUN_TOKEN, DOCTORED
            )
            if REAPPEND_KEY is None:
                sys.stdout.write(
                    f"FAIL: the plan store did not take the doctored copy:"
                    f" {reappend_reason}; the harness is fail-closed.\n"
                )
                return 2
            probe_id, description, _mutate, expected, recorded = suffix_shapes()[1]
            code, text = run(base, review, repo)
            line = next(
                (one.strip() for one in text.splitlines() if one.strip().startswith("STORE:")),
                "",
            )
            produced[probe_id] = text
            ok = code == expected and " ".join(line.split()) == " ".join(recorded.split())
            bad += 0 if ok else 1
            sys.stdout.write(
                f"{probe_id:16s} exit {code}  {'OK ' if ok else 'BAD'}  {description}\n"
            )
            sys.stdout.write(f"{'':16s}        {line}\n")
            if not ok:
                sys.stdout.write(f"{'':16s}        expected exit {expected}: {recorded}\n")
        for probe_id, description, mutate, expected, recorded in REVIEW_SHAPES:
            if wanted and probe_id not in wanted:
                continue
            copy = os.path.join(scratch, probe_id + "-review.md")
            with open(copy, "w", encoding="utf-8") as handle:
                handle.write(mutate(REVIEW))
            code, text = run(base, copy, repo)
            line = next(
                (one.strip() for one in text.splitlines() if one.strip().startswith("FAIL:")),
                "",
            )
            produced[probe_id] = text
            ok = code == expected and not missing(flatten(recorded), flatten(line))
            bad += 0 if ok else 1
            sys.stdout.write(
                f"{probe_id:16s} exit {code}  {'OK ' if ok else 'BAD'}  {description}\n"
            )
            sys.stdout.write(f"{'':16s}        {line}\n")
            if not ok:
                sys.stdout.write(f"{'':16s}        expected exit {expected}: {recorded}\n")
    fragments = recorded_fragments(cell)
    stale = []
    if not wanted:
        pool = flatten("\n".join(produced.values()))
        for piece in fragments:
            holders = [
                name for name, text in produced.items() if not missing(piece, flatten(text))
            ]
            if holders:
                continue
            reason = (
                "no run produced it"
                if missing(piece, pool)
                else "a run of ANOTHER shape produced it and no shape of this row did"
            )
            stale.append((piece, reason))
    # One recorded output line per shape is the floor, and the shape count is
    # the source (critic C18-W4).
    floor = 0 if wanted else len(SHAPES) + len(REVIEW_SHAPES) + len(suffix_shapes())
    short = not wanted and len(fragments) < floor
    for piece, reason in stale:
        sys.stdout.write(f"TEXT:  PP32: {reason}: {piece}\n")
    if short:
        sys.stdout.write(
            f"TEXT:  PP32: the cell records {len(fragments)} output lines and the floor"
            f" is {floor}; the harness is fail-closed.\n"
        )
    sys.stdout.write(
        f"CLOSURE PROBES BAD: {bad}     RECORDED FRAGMENTS: {len(fragments)}"
        f"     FLOOR: {floor}     FRAGMENTS BAD: {len(stale)}\n"
    )
    return 1 if bad or stale or short else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
