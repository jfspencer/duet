"""Closure guard: one Appendix C row per finding of one review (PG32).

**This is a REVIEW-TIME tool and no gate runs it** (critic C18-6). The
Architect runs it as the last step of every closure, once per review block,
before the document is frozen for the next review. Revision 18 called it a
prototype for a `cargo xtask check-closure` that no chunk could create: the
name appeared in no write scope, in no workflow, and nowhere in the
specification. The contract lives in architecture section 1.5, rule PG32,
which also states the one change that would make it a gate and who owns that
change. Section 1.9 records its probe PP32.

Revision 17 recorded the sixteenth review as thirteen Criticals and thirteen
Warnings. It returned seventeen and eighteen. Twelve ids had no row, no
citation and no fix, and the count sentence made the drop invisible: a reader
who audits the appendix reads thirteen, counts thirteen rows, and concludes the
review is closed (critic C17-1). The denominator was wrong, not one row.

usage: closure_check.py <architecture.md> <review.md> <block id> [repo-root]

`repo-root` is the repository that holds `.claude/plan-coordination/db.sh`,
which CL1c reads the stored review from. It is EXPLICIT and never an
environment variable, because a FROZEN copy of this plan sits outside the
repository and a guard that guessed would be a guard that guessed wrong
(critic N19-3, which refused a second environment seam).
`run_all_gates.py` passes its own `<repo-root>` argument through.

The rule has six parts.

CL1 the id list comes from `review_ids.py`, which reads the review's own
    headings. No id in this guard and no id in the document is typed.
CL2 every generated id has exactly one row of the registered block, and every
    row names a generated id. Both directions, so a row for a finding the
    review does not state is a failure as well.
CL3 the document holds the COUNT SENTENCE this guard builds from the generated
    counts, character for character. The sentence carries the three family
    counts and the total, so a truncated read changes the sentence and the
    guard refuses it.
CL4 every row carries a non-empty Finding cell and a State cell from the three
    words CLOSED, PARTIAL and OPEN, and a CLOSED row names a section that a
    heading of this document holds. Revision 18 checked the section cell of a
    CLOSED row for emptiness alone, so a row could be emptied of everything it
    says and stay green, and a CLOSED row could name a section number the
    document does not hold (critic C18-W1).
CL1c the review file matches an INDEPENDENT copy, taken from outside the
    Architect's write scope (critic C21-W1). CL1b reads a second SENTENCE of
    the same file, under the same hand, and the twenty-first Critic defeated
    the whole closure record with seven coordinated edits: it deleted one
    `## WARNING 6` section, retyped that review's own count line, dropped the
    matching row, retyped the document count sentence, retyped the digest
    sentence, lowered the `rows>=` marker, and edited one line of the
    `placement_check.py` register. Every gate stayed green. **Every file those
    seven edits touched belongs to one party**, so no rule that reads only
    those files can ever answer, and the remedy is a different writer and not
    a further rule.

    The Critic stores every review with `.claude/plan-coordination/db.sh
    append`, which mints a `<simpleflake>-<suffix>` key in the plan store. A
    DYNAMIC key is append-only and the store sits outside the repository, so
    the Architect cannot rewrite a stored review. The closure section records
    that key beside the digest, and this rule reads the stored copy back and
    compares its md5 with the file's.

    **The rule states its limit at its own site.** The seven reviews before
    revision 22 were appended to the store BY THE ARCHITECT during revision
    22, because the keys the earlier Critic runs printed were never recorded
    in this plan. For those seven the stored copy is a CHECKSUM and not a
    second party, and a coordinated edit that also re-appends would still
    pass. From revision 22 on the stored copy is the one the Critic appended
    and the key is the one the Critic printed, so the two writers differ.
    There is NO way to skip the store half: a flag that removed a check was
    the fail-open C22I-W4 deleted, and a run that reaches the end with that
    half unrun is a failure that says so.

CL5 the document states the REVIEW FILE'S OWN DIGEST beside the block, and the
    digest of the file this run was handed matches it.

    **CL5 is the rule that makes the other four mean something** (critic
    C18-1). Revision 18 proved `rows == generated(input)` and never
    `input == the review`: the Critic handed the guard a truncated copy of one
    review and it reported a complete closure over 20 of 26 findings, at
    production defaults, in one command. The denominator was taken on trust
    from the one party the appendix exists to check. The document already held
    the correct pattern one line away, because each closure section records the
    md5 of the frozen DOCUMENT that review read.

It exits 2 on a usage or input failure, 1 on a finding, and 0 when the block is
complete against the review.
"""

import hashlib
import os
import re
import subprocess
import sys

import placement_check
import review_ids

HERE = os.path.dirname(os.path.abspath(__file__))
PLAN = os.path.dirname(HERE)
REPO = os.path.dirname(os.path.dirname(PLAN))
# The plan directory as `db.sh` names it, which is the plan key and not a path
# this script resolves.
PLAN_NAME = "roadmap/" + os.path.basename(PLAN)
STORE_KEY = re.compile(
    r"The review file this block records is stored at plan-store key `([A-Za-z0-9._:-]+)`"
    r" \((closure-[A-Za-z0-9-]+)\)\."
)


STATES = ("CLOSED", "PARTIAL", "OPEN")


def count_sentence(criticals, warnings, concerns, total):
    """The one sentence the appendix must hold for this review (CL3)."""
    return (
        f"It returned {criticals} Criticals, {warnings} Warnings, and {concerns} Concerns,"
        f" and this block holds one row for each of the {total}."
    )


def digest_sentence(block_id, digest):
    """The one sentence that binds a block to the review file it records."""
    return f"The review file this block records has md5 `{digest}` ({block_id})."


def store_sentence(block_id, key):
    """The one sentence that binds a block to the stored copy (CL1c)."""
    return (
        "The review file this block records is stored at plan-store key"
        f" `{key}` ({block_id})."
    )


def store_digest(key, repo):
    """The md5 of the stored review at one plan-store key, or a reason.

    It calls `.claude/plan-coordination/db.sh get`, which is the ONE store
    tool of this repository, and it reads the raw bytes that command prints.
    A store that does not answer, and a key that holds nothing, are each a
    failure and never a skip: a guard that passes when its second source is
    unreachable is the fail-open this rule exists to remove.
    """
    launcher = os.path.join(repo, ".claude", "plan-coordination", "db.sh")
    if not os.path.isfile(launcher):
        return None, f"the store launcher {launcher} does not open"
    try:
        result = subprocess.run(
            ["bash", launcher, "get", PLAN_NAME, key],
            cwd=repo,
            capture_output=True,
            check=False,
        )
    except OSError as error:
        return None, f"the store launcher did not run: {error}"
    if result.returncode != 0:
        detail = result.stderr.decode("utf-8", "replace").strip().splitlines()
        return None, f"`db.sh get` exited {result.returncode}: {detail[0] if detail else ''}"
    if not result.stdout:
        return None, f"the store holds nothing at key `{key}`"
    return content_digest(result.stdout), None


def content_digest(data):
    """The md5 of one review's CONTENT, with trailing newlines removed.

    `db.sh append` takes the value as one argument and `db.sh get` prints it
    with one trailing newline, so a byte compare would answer the shell's
    quoting and not the review's content. Both sides are stripped, so the rule
    decides the text and nothing else.
    """
    return hashlib.md5(data.rstrip(b"\n")).hexdigest()


def digest_of(path):
    """The md5 of one file, as the document records it."""
    with open(path, "rb") as handle:
        return hashlib.md5(handle.read()).hexdigest()


def headings(source):
    """Every section number a heading of this document states."""
    found = set()
    for match in re.finditer(r"(?m)^#{2,4} ((?:[0-9]+\.[0-9]+[a-z]?)|(?:[A-Z]\.[0-9]+))", source):
        found.add(match.group(1))
    for match in re.finditer(r"(?m)^## ([0-9]+)\. ", source):
        found.add(match.group(1))
    return found


def generated(review):
    """`(ids, counts)` of one review, or `(None, reason)` on a parse failure."""
    with open(review, encoding="utf-8") as handle:
        text = handle.read()
    prefix = review_ids.prefix_of_path(review)
    if prefix is None:
        revision = review_ids.revision_of(text)
        if revision is None:
            return None, "the file name states no review and the title states no revision"
        prefix = f"C{revision}"
    parsed, reason = review_ids.ids_of(text, prefix)
    if parsed is None:
        return None, reason
    return parsed, None


def block_section(source, block_id):
    """The text of the section that holds one block, heading to heading.

    CL3 and CL5 are SUBSTRING tests, and revision 19 ran both over the whole
    document: the count sentence and the digest sentence moved into section
    1.1 and both runs stayed green, so a reader of the closure appendix saw
    neither (critic C19-W2). PG32 says the digest sits "beside the block", so
    the search window is the block's own section and nothing wider.
    """
    heading = placement_check.DATA_BLOCKS[block_id][0]
    found = list(re.finditer(r"(?m)^#### " + re.escape(heading) + r"\s*$", source))
    if len(found) != 1:
        return None
    # The window opens at the `###` heading above the block's own `####`
    # heading, because the appendix states the count and the digest in the
    # prose of that section and then carries the block under a sub-heading.
    above = list(re.finditer(r"(?m)^### ", source[: found[0].start()]))
    start = above[-1].start() if above else 0
    rest = source[found[0].end():]
    stop = re.search(r"(?m)^### ", rest)
    end = found[0].end() + (stop.start() if stop else len(rest))
    return source[start:end]


def audit(source, window, rows, ids, seen, known):
    """Every CL2, CL3 and CL4 failure of one block, as printable lines."""
    failures = []
    listed = []
    for row in rows:
        listed.append((row[0].strip().strip("`") if row else "", row))
    have = {identifier for identifier, _row in listed}
    # CL2 counts as well as compares (critic C19-W3). A duplicated row with
    # the state flipped printed `GENERATED: 20   ROWS: 21   CLOSURE BAD: 0`,
    # because the two numbers stood side by side and nothing compared them.
    # PG32 says ONE closure row per finding, so a second row for one id is a
    # failure of its own.
    counted = {}
    for identifier, _row in listed:
        counted[identifier] = counted.get(identifier, 0) + 1
    for identifier in sorted(name for name, times in counted.items() if times > 1):
        failures.append(
            f"CLOSURE:   {identifier}: the block holds {counted[identifier]} rows and"
            " PG32 states one closure row per finding"
        )
    for identifier in ids:
        if identifier not in have:
            failures.append(
                f"CLOSURE:   {identifier}: the review states it and the block holds no row"
            )
    for identifier, row in listed:
        if identifier not in set(ids):
            failures.append(
                f"CLOSURE:   {identifier}: the block holds a row and the review states no"
                " such finding"
            )
            continue
        finding = row[1].strip() if len(row) > 1 else ""
        state = row[2].strip().strip("*") if len(row) > 2 else ""
        section = row[3].strip() if len(row) > 3 else ""
        if not finding:
            failures.append(f"CLOSURE:   {identifier}: the row states no finding")
            continue
        if state.upper() not in STATES:
            failures.append(
                f"CLOSURE:   {identifier}: the state is `{state}` and the three states are"
                f" {', '.join(STATES)}"
            )
            continue
        if state.upper() == "CLOSED" and not section:
            failures.append(
                f"CLOSURE:   {identifier}: the row says CLOSED and names no section"
            )
            continue
        if state.upper() == "CLOSED":
            # A backticked span holds a crate version, a package name, or a
            # path, never a section reference, so it leaves the cell first.
            # `libpipewire-0.3-dev` and `0.7.19` are not sections.
            plain = re.sub(r"``.+?``|`[^`]*`", " ", section)
            cited = set(re.findall(r"(?<![\w.-])((?:[0-9]+\.[0-9]+[a-z]?)|(?:[A-Z]\.[0-9]+))(?![\w.-])", plain))
            unknown = sorted(cited - known)
            # CL4 asks for a section and not merely for the absence of a wrong
            # one (critic C19-W1). Revision 19 fired only `if cited and
            # unknown`, so a cell with no section-shaped token at all passed:
            # "fixed in the tool" was a green CLOSED row, and five real rows
            # of C.22 were already that shape. A CLOSED row must NAME at least
            # one heading of this document, and every named one must exist.
            if not cited:
                failures.append(
                    f"CLOSURE:   {identifier}: the row says CLOSED and its section cell"
                    " names no section of this document"
                )
                continue
            if unknown:
                failures.append(
                    f"CLOSURE:   {identifier}: the row names section {unknown[0]} and no"
                    " heading of this document states it"
                )
    wanted = count_sentence(
        len(seen["CRITICAL"]), len(seen["WARNING"]), len(seen["CONCERN"]), len(ids)
    )
    if wanted not in window:
        failures.append(
            "COUNT:     the document does not hold the count sentence this review"
            f" generates: {wanted}"
        )
    return failures


def sibling_keys(key, repo):
    """Every stored key that shares one key's suffix, or a reason (CL1c).

    **The Architect must not choose which stored copy the guard compares**
    (critic C22I-W5). The store refuses a REWRITE of an existing key and it
    does not refuse an APPEND, so a coordinated edit needed no rewrite at all:
    it appended a doctored review under a NEW key and retyped one sentence of
    the closure section, and both acts sat inside the Architect's own write
    scope. Revision 22's rule read the one key the document named, so it could
    not see the second copy.

    The guard reads EVERY copy of the suffix now. A second append under that
    suffix is a second copy with a different digest, and the rule fails on the
    difference rather than on the key the document happens to name.
    """
    launcher = os.path.join(repo, ".claude", "plan-coordination", "db.sh")
    if not os.path.isfile(launcher):
        return None, f"the store launcher {launcher} does not open"
    suffix = key.split("-", 1)[-1]
    try:
        result = subprocess.run(
            ["bash", launcher, "keys", PLAN_NAME],
            cwd=repo,
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError as error:
        return None, f"the store launcher did not run: {error}"
    if result.returncode != 0:
        return None, f"`db.sh keys` exited {result.returncode}"
    found = sorted(re.findall(r'"key":"([A-Za-z0-9._:-]+)"', result.stdout))
    matched = [name for name in found if name.split("-", 1)[-1] == suffix]
    if key not in matched:
        return None, f"the store holds no key `{key}`"
    return matched, None


def wanted_suffix(review):
    """The one suffix a `critic-spec-<rev>.md` review may be stored under.

    A rule that read the named key alone left the Architect a second move:
    append the doctored review under a NEW suffix and point one sentence at
    it. The review FILE NAME decides the suffix, `run_all_gates.py` derives
    that name from `reviews/`, and CL5 binds the file to its digest, so the
    suffix is outside the sentence the Architect writes (critic C22I-W5). A
    review whose name is not `critic-spec-<rev>.md` is a probe's throwaway
    file and this half declines it, which the CL1c site states as a limit.
    """
    match = re.fullmatch(r"critic-spec-(r[0-9][A-Za-z0-9-]*)\.md", os.path.basename(review))
    return f"review-{match.group(1)}" if match else None


def main(argv):
    """Run the four rules over one block and return an exit code."""
    if len(argv) not in (4, 5):
        sys.stdout.write(
            "usage: closure_check.py <architecture.md> <review.md> <block id> [repo-root]\n"
        )
        return 2
    document, review, block_id = argv[1], argv[2], argv[3]
    repo = os.path.abspath(argv[4]) if len(argv) == 5 else REPO
    for path in (document, review):
        if not os.path.isfile(path):
            sys.stdout.write(f"FAIL: cannot open {path}; the guard is fail-closed.\n")
            return 2
    if block_id not in placement_check.DATA_BLOCKS:
        sys.stdout.write(
            f"FAIL: `{block_id}` is no registered block; the guard is fail-closed.\n"
        )
        return 2
    with open(document, encoding="utf-8") as handle:
        source = handle.read()
    # A probe builds a throwaway document that holds one block, so the row
    # minimum of the REGISTER cannot apply to it. The register's minimum is a
    # DR7 rule about this specification, and PG27 inside `placement_check.py`
    # is the rule that enforces it. This guard reads the block and decides its
    # CONTENT, so it takes the marker's own minimum.
    rows, reason = placement_check.read_block(source, block_id, register=False)
    if reason is not None:
        sys.stdout.write(f"FAIL: the `{block_id}` block of this document: {reason}\n")
        return 2
    parsed, problem = generated(review)
    if parsed is None:
        sys.stdout.write(f"FAIL: {review}: {problem}; the guard is fail-closed.\n")
        return 2
    ids, seen, _stated = parsed
    wanted_digest = digest_of(review)
    with open(review, "rb") as handle:
        wanted_content = content_digest(handle.read())
    window = block_section(source, block_id)
    if window is None:
        sys.stdout.write(
            f"FAIL: the `{block_id}` block has no section of its own; the guard is"
            " fail-closed.\n"
        )
        return 2
    failures = audit(source, window, rows, ids, seen, headings(source))
    if digest_sentence(block_id, wanted_digest) not in window:
        failures.append(
            f"DIGEST:    the document states no md5 `{wanted_digest}` for {block_id}"
            " beside the block, so nothing binds this block to the file this run read"
        )
    store_state, key, copies = "skipped", None, None
    found_key = STORE_KEY.search(window)
    if found_key is None or found_key.group(2) != block_id:
        failures.append(
            f"STORE:     the section states no plan-store key for {block_id}, so nothing"
            " binds this block to a copy outside the Architect's write scope (CL1c)"
        )
        store_state = "absent"
    else:
        key = found_key.group(1)
        suffix = wanted_suffix(review)
        if suffix is not None and key.split("-", 1)[-1] != suffix:
            failures.append(
                f"STORE:     the section names key `{key}`, whose suffix is not"
                f" `{suffix}`; the review file name decides the suffix and the Architect"
                " does not (CL1c)"
            )
            store_state = "wrong-suffix"
        else:
            copies, reason = sibling_keys(key, repo)
            if copies is None:
                failures.append(f"STORE:     key `{key}`: {reason}; CL1c is fail-closed")
                store_state = "unreachable"
    if copies:
        digests = {}
        for name in copies:
            stored, reason = store_digest(name, repo)
            if stored is None:
                failures.append(f"STORE:     key `{name}`: {reason}; CL1c is fail-closed")
                store_state = "unreachable"
                break
            digests[name] = stored
        else:
            wrong = sorted(name for name, value in digests.items() if value != wanted_content)
            if wrong:
                failures.append(
                    f"STORE:     {len(wrong)} of {len(copies)} stored copies of this review"
                    f" differ from the file this run read, `{wanted_content}`; the first is"
                    f" key `{wrong[0]}` at md5 `{digests[wrong[0]]}` (CL1c)"
                )
                store_state = "differs"
            else:
                store_state = "matches"
    if store_state == "skipped":
        failures.append(
            "STORE:     the store half of CL1c did not run, and a rule that can be skipped"
            " is the fail-open this rule exists to remove (CL1c)"
        )
    sys.stdout.write(
        f"REVIEW: {os.path.basename(review)}   BLOCK: {block_id}"
        f"   GENERATED: {len(ids)}   ROWS: {len(rows)}   STORE: {store_state}"
        f"   STORE COPIES: {len(copies or ())}"
        f"   CLOSURE BAD: {len(failures)}\n"
    )
    for line in failures:
        sys.stdout.write("  " + line + "\n")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
