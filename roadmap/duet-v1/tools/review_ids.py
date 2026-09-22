"""Generate the canonical finding-id list of one Engineering Critic review.

Appendix C of `architecture.md` carries one closure row per finding of each
review. Revision 17 recorded the sixteenth review as thirteen Criticals and
thirteen Warnings; it returned seventeen and eighteen, and twelve ids had no
row, no citation and no fix (critic C17-1). A truncated read of a review is the
one failure the closure appendix exists to prevent, so the id list is generated
from the review file and never typed.

usage: review_ids.py <review.md> [--prefix C17]

It prints one id per line in document order, then one summary line:

    CRITICALS: 17   WARNINGS: 18   CONCERNS: 11   TOTAL: 46

The prefix defaults to the review FILE NAME and never to the revision number
alone (critic N21-1). Two reviews read revision 21: `critic-spec-r21.md`,
the external one, and `critic-spec-r21-inner.md`, the inner one. Both titles
state "revision 21", so both generated `C21-n` and `N21-n`, and Appendix C
would have held two blocks with colliding ids that no parser could tell apart.

`critic-spec-r<n>.md` gives the prefix `C<n>`. `critic-spec-r<n>-<tag>.md`
gives `C<n><T>`, where `<T>` is the first letter of the tag in upper case, so
`critic-spec-r21-inner.md` gives `C21I`. A Critical becomes `<prefix>-<n>`, a
Warning `<prefix>-W<n>`, and a Concern `N<prefix without its leading C>-<n>`,
which is the shape Appendix C uses.

It exits 0 on a parse, 2 when the file does not open, when the title states no
revision and no prefix is given, or when a family is empty. **A review with no
Critical heading is a parse failure and not an empty list**: the appendix would
then record a complete closure over nothing, which is the vacuous green this
script exists to refuse.
"""

import os
import re
import sys

HEADING = re.compile(r"(?m)^##\s+(CRITICAL|WARNING|CONCERN)\s+(\d+)\b")
# The RECOVERED shapes, which the nineteenth review carries and which no
# revision can now change: `reviews/` is append-only, so a review already
# stored is read as it stands (critic C19-W1 to C19-W25, C19-N1 to C19-N9).
# A Warning written as `**W7.**` at the head of a line, and a Concern whose id
# opens a table row, are each a finding the canonical scan cannot see. The
# parser reads all three shapes and CL1b still holds the total against the
# review's own stated count, so a fourth shape nobody declared is still a
# refused run and never a short list.
RECOVERED_WARNING = re.compile(r"(?m)^\*\*W(\d+)\.")
RECOVERED_CONCERN = r"(?m)^\|\s*N{digits}-(\d+)\s*\|"
# Both CommonMark fence characters, and an UNCLOSED fence to the end of the
# file (critic C18-W6, C19-W4). Revision 19 stripped backtick fences alone, so
# a `~~~` fence that quoted `## CRITICAL 7` was counted as a heading and the
# parse printed `CRITICALS: 7` for a six-Critical review. A review of a review
# quotes headings, and `~~~` is standard.
FENCE = re.compile(r"(?ms)^(?:```|~~~).*?(?:^(?:```|~~~)|\Z)")
TITLE = re.compile(r"(?mi)^#\s.*\brevision\s+(\d+)\b")
FAMILIES = ("CRITICAL", "WARNING", "CONCERN")

# The review's OWN stated counts, which is the second source (critic C19-2).
# The heading scan and this sentence are written at different moments by the
# same reviewer, so a heading the scan cannot see -- a `### CRITICAL 11`, a
# `**W1.**`, a Concerns TABLE -- makes the two disagree and the run refuse.
COUNTS = re.compile(
    r"(?mi)^\*\*Counts:\s*(\d+)\s+Criticals?,\s*(\d+)\s+Warnings?,\s*(\d+)\s+Concerns?\.\*\*"
)
# The recovered count form. The nineteenth review closes with "This review
# holds 11 Criticals, 25 Warnings, and 9 Concerns." and predates the
# `**Counts: ...**` sentence, so the second source exists in that review under
# a different spelling. Both forms are read; neither is built by this parser.
COUNTS_PROSE = re.compile(
    r"(?mi)^This review holds\s*(\d+)\s+Criticals?,\s*(\d+)\s+Warnings?,"
    r"\s*and\s*(\d+)\s+Concerns?\."
)

# The first revision whose review must carry the count sentence. A review of
# an earlier revision predates the format, so the sentence is optional there
# and its absence is printed rather than hidden. The cut-off is a property of
# this plan and never a flag a runner passes, because a flag that removes a
# check is the defect C16-2 already cost this plan one revision.
COUNTS_REQUIRED_FROM = 20


# `critic-spec-r21.md` and `critic-spec-r21-inner.md`, which are one revision
# and two reviews (critic N21-1).
FILE_NAME = re.compile(r"^critic-spec-r(\d+)(?:-([a-z][a-z0-9]*))?\.md$")


def revision_of(text):
    """The revision number the review's own title states, or None."""
    found = TITLE.search(text)
    return found.group(1) if found else None


def prefix_of_path(path):
    """The id prefix one review file name gives, or None (critic N21-1).

    The file name is the one thing that separates two reviews of one
    revision, and the title is not: both titles say "revision 21". A name this
    parser cannot read returns None, and the caller falls back to the title,
    so a review stored under an older name still generates its own ids.
    """
    found = FILE_NAME.match(os.path.basename(path))
    if found is None:
        return None
    if found.group(2) is None:
        return f"C{found.group(1)}"
    return f"C{found.group(1)}{found.group(2)[0].upper()}"


def stated_counts(text):
    """The counts the review states about itself, or `None` (critic C19-2)."""
    stripped = FENCE.sub("", text)
    found = COUNTS.search(stripped) or COUNTS_PROSE.search(stripped)
    if not found:
        return None
    return {
        "CRITICAL": int(found.group(1)),
        "WARNING": int(found.group(2)),
        "CONCERN": int(found.group(3)),
    }


def ids_of(text, prefix):
    """Every finding id of one review, in document order.

    The parser reads `## CRITICAL n`, `## WARNING n` and `## CONCERN n` and
    nothing else. A review that repeats a number inside one family, or that
    numbers a family from something other than one, is a parse failure: both
    shapes make the generated list silently wrong, which is the class this
    script exists to refuse.

    **A fenced block is removed before the headings are read** (critic
    C18-W6). A review of a review quotes headings, and one quoted
    `## CRITICAL 8` inside a fence made a seven-Critical review generate eight
    ids. The only way to reach green was then to write an appendix row for a
    finding no review made, which is a fabricated closure.

    **An empty WARNING or CONCERN family is allowed** (critic C18-W6). A
    review with no Concern could not be recorded at all, so the guard put
    pressure on the reviewer to produce one; a guard that shapes the finding
    count of the review it audits has left its subject. A review with no
    heading of ANY family is still a parse failure, because a complete
    closure over nothing is the vacuous green this script exists to refuse.
    """
    stated = stated_counts(text)
    text = FENCE.sub("", text)
    found, seen = [], {family: [] for family in FAMILIES}
    for family, number in HEADING.findall(text):
        seen[family].append(int(number))
    for number in RECOVERED_WARNING.findall(text):
        seen["WARNING"].append(int(number))
    own_tail = prefix[1:] if prefix.startswith("C") else prefix
    for number in re.findall(RECOVERED_CONCERN.format(digits=re.escape(own_tail)), text):
        seen["CONCERN"].append(int(number))
    if not any(seen[family] for family in FAMILIES):
        return None, "the review states no CRITICAL, WARNING or CONCERN heading"
    for family in FAMILIES:
        numbers = seen[family]
        if not numbers:
            continue
        if sorted(numbers) != list(range(1, len(numbers) + 1)):
            return None, f"the {family} numbers are not 1 to {len(numbers)}: {numbers}"
    for number in sorted(seen["CRITICAL"]):
        found.append(f"{prefix}-{number}")
    for number in sorted(seen["WARNING"]):
        found.append(f"{prefix}-W{number}")
    for number in sorted(seen["CONCERN"]):
        found.append(f"N{own_tail}-{number}")
    # The second source (critic C19-2). The heading scan is ONE reading of the
    # review, and revision 19 shipped a review whose Warnings were `**Wn.**`
    # and whose Concerns were a table, so the scan found a fraction of the
    # findings and the appendix would have recorded a complete closure over
    # that fraction. The reviewer states the counts in a sentence the parser
    # does not build, and a disagreement is a refused run and never a short
    # list.
    revision = re.sub(r"[^0-9]", "", prefix)
    required = bool(revision) and int(revision) >= COUNTS_REQUIRED_FROM
    if stated is None:
        if required:
            return None, (
                "the review states no `**Counts: <n> Criticals, <n> Warnings,"
                " <n> Concerns.**` sentence, which every review from revision"
                f" {COUNTS_REQUIRED_FROM} carries"
            )
    else:
        for family in FAMILIES:
            if stated[family] != len(seen[family]):
                return None, (
                    f"the review states {stated[family]} {family.title()}s and the"
                    f" heading scan finds {len(seen[family])}; a heading this parser"
                    " cannot see is the one failure a second source exists to catch"
                )
    return (found, seen, stated), None


def main(argv):
    """Print the id list of one review and return an exit code."""
    if len(argv) not in (2, 4) or (len(argv) == 4 and argv[2] != "--prefix"):
        sys.stdout.write("usage: review_ids.py <review.md> [--prefix C17]\n")
        return 2
    path = argv[1]
    if not os.path.isfile(path):
        sys.stdout.write(f"FAIL: cannot open {path}; the generator is fail-closed.\n")
        return 2
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    prefix = argv[3] if len(argv) == 4 else None
    if prefix is None:
        prefix = prefix_of_path(path)
    if prefix is None:
        revision = revision_of(text)
        if revision is None:
            sys.stdout.write(
                "FAIL: the file name states no review, the title states no revision,"
                " and no --prefix was given; the generator is fail-closed.\n"
            )
            return 2
        prefix = f"C{revision}"
    parsed, reason = ids_of(text, prefix)
    if parsed is None:
        sys.stdout.write(f"FAIL: {path}: {reason}; the generator is fail-closed.\n")
        return 2
    found, seen, stated = parsed
    for identifier in found:
        sys.stdout.write(identifier + "\n")
    sys.stdout.write(
        f"CRITICALS: {len(seen['CRITICAL'])}   WARNINGS: {len(seen['WARNING'])}"
        f"   CONCERNS: {len(seen['CONCERN'])}   TOTAL: {len(found)}\n"
    )
    sys.stdout.write(
        "COUNT SENTENCE: agrees\n" if stated is not None
        else "COUNT SENTENCE: absent; this review predates the format\n"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
