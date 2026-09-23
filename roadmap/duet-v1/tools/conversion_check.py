"""Conversion guard for the Duet workspace.

Prototype for `cargo xtask check-conversions`, which chunk M0 ports to
`tools/xtask/src/check_conversions.rs`. The contract lives in architecture
section 2.3, and every rule there carries an id `CG<n>`. Section 1.9 pairs
each rule with its one probe `CP<n>` (DR5).

Clippy is the backstop: the root `Cargo.toml` sets `as_conversions = "deny"`
and `scripts/dod.sh` runs clippy with `-D warnings`. This guard is the
earlier, cheaper signal, and CG6 is the rule clippy cannot carry, because an
`#[expect]` silences clippy and does not silence a text scan.

CG1  the file set comes from the TARGET SOURCE PATHS of
     `cargo metadata --no-deps`: for every target of every workspace member
     the guard takes `src_path`, then it adds every `.rs` file under that
     path's parent directory that the same member owns. Revision 11 walked
     each manifest directory and dropped any path with a component named
     `target`, at any depth, so a source module named `target` left the
     scanned set and the run reported success over the files it did see
     (critic WR-8). The one build-directory filter that remains is ANCHORED:
     it compares canonical paths against the workspace root plus `target` and
     against each member root plus `target`, and against nothing else. A
     `.rs` extension matches without case, so `Other.RS` is scanned (critic
     concern 19). The guard takes NO path argument, because CG1 states that
     the subcommand takes none; a conversion probe changes directory into its
     own throwaway workspace (critic concern 18). **The rule's second stated
     limit**: a module reached by a `#[path = "..."]` attribute that points
     outside the target's own directory is outside the walked set, because
     the walk starts at each target's `src_path` and grows to the `.rs` files
     beside it. A cast in such a module exits 0 here, and
     `cargo clippy --all-targets` in `scripts/dod.sh` is the backstop that
     sees it (critic concern 8).
CG2  one lexer pass classifies every byte as code, string, character, line
     comment, or block comment before any rule scans the text. Only a byte
     classified as code reaches CG3. The pass reads all six string prefixes
     Rust writes, which are `b`, `c`, `r`, `br`, `cr`, and `rb`. A prefix that
     carries an `r` opens a RAW literal, which processes no escape and ends
     at a quotation mark followed by its own hash run.
CG2b a comment token inside a string or a character literal never opens a
     comment, and a quotation mark inside a comment never opens a string.
CG3  a whole-word `as` token outside the one conversion file is a failure.
CG3b an exclusion never hides a cast, and it prefers a false red to a miss:
     a `<... as ...>` span is exempt only when BOTH sides of `as` parse as a
     type path with no expression token. A comparison pair is never exempt.
     **The stated false-red classes**: a comparison pair that happens to read
     as a type path on both sides, and a RAW IDENTIFIER. `let r#as = 1u32;`
     reports two casts on one line, because the scan reads a whole-word `as`
     and the `r#` prefix does not change the word. Both are a red the author
     rewrites, never a cast the guard misses (critic concern 9).
CG4  a `use` item is excluded, as a whole item, across lines.
CG4b a `use` item in statement position with no terminating `;` is a
     failure, exit 2. A `use` token that is not in statement position, such
     as one inside a macro pattern, is not an item and blanks nothing.
CG5  a qualified path span is excluded when its `as` sits at the span's own
     depth with a type-shaped token on each side.
CG6  an attribute that suppresses `clippy::as_conversions` outside the one
     conversion file is a failure. The guard reads the WHOLE attribute span,
     outer and inner alike, and it accepts the `allow` spelling, the `expect`
     spelling, a `cfg_attr` wrapper, and arbitrary whitespace, a line break
     included, between every token. Revision 11 matched the literal
     `#!?\\[expect\\(clippy::as_conversions`, which one space after `(`, one
     line break, a `cfg_attr` wrapper, and the `allow` spelling each defeated
     (critic concern 1).
CG7  the one exempt file is matched on BOTH sides as a canonical path, and
     the canonical exempt file must sit inside the canonical `duet-time`
     member root. Revision 11 canonicalised the exempt path alone, so a
     symbolic link at `crates/duet-time/src/convert.rs` that pointed at a
     live module of another member exempted that module (critic concern 2).
CG1b a scan that misses a member this workspace holds, or a source file a
     scanned member holds. "Found 0 casts" and "scanned 0 files" print the
     same success, and one `exclude` line in the workspace manifest took a
     whole crate out of the scan while a real cast sat in it (critic C15-4).
     **The expected set is DERIVED and never a constant** (critic C16-2).
     Revision 16 used a count floor of three members and six files. A count
     floor bounds the scan by a number that does not track the thing it
     measures: section 13 creates fifteen more crates, so from the first new
     crate onward one `exclude` line removes any member and the remaining
     count still clears the floor. The Critic reproduced that vacuous green
     at the production default floors. Two environment variables also
     disabled the floor at the production entry point, and no line of the
     specification named them.
     The rule now has three parts and no constant.
     (1) **The zero test.** Zero members or zero files is exit 2, which is
     the fail-closed-on-zero rule PG12 states for a denominator.
     (2) **The derived member set.** Every directory under `crates/` and
     under `tools/` that holds a `Cargo.toml`, at any depth, is a member this
     workspace holds, because the root manifest globs both trees. The set is
     compared with the member set `cargo metadata` reports, and a member the
     scan does not cover is exit 2 WITH ITS PATH.
     (3) **The derived file set.** Every `.rs` file under each covered
     member's own `src/` directory is a file the scan must hold, and one it
     does not hold is exit 2 with its path. It catches a member whose one
     target sits outside `src/`, which leaves the whole of `src/` outside
     CG1's walk.
     **The rule's own limit, stated here**: it decides COVERAGE and never a
     verdict about a file it does scan, and it derives the member set from
     two directory names that the root manifest's own globs fix. A workspace
     that keeps a crate somewhere else is outside it, and `cargo clippy
     --all-targets` in `scripts/dod.sh` is the backstop. No environment
     variable changes any part of it.
CG8  a workspace that `cargo metadata` refuses, a member file that is not
     UTF-8, or a member file the guard cannot open. Each one prints one named
     line and exits 2. Revision 11 let the decode error escape, and the
     interpreter exited 1; it also reported an unopenable file as a FINDING,
     which is the same exit 1. A caller that reads the status alone could
     therefore not tell a fail-closed run from a findings run, while the
     finding's own text already said "the guard is fail-closed" (critic K-9).

**No rule here reads a document block.** The whole input of this guard is
`cargo metadata --no-deps` plus the bytes of the files it names, so the DR7
block register of section 1.9 carries no `CG` row and PG27 holds no rule of
this file. A `CG` rule that ever reads a block enters that register in the
same changeset.

**CG9 is the one rule this prototype does not implement.** The Rust port
compares each Appendix B.1 `b1-convert` reason cell with the `reason =` string
of that function in `crates/duet-time/src/convert.rs`, and the two sets are one
set in both directions. PG29 reads this file for rule IDS alone and never for
behaviour, so the name of the rule is the whole record this file carries, and
architecture section 1.9 states the divergence.
"""

import fnmatch
import json
import os
import re
import subprocess
import sys

# The one member that may hold a cast, and the one file inside it (CG7).
EXEMPT_MEMBER = ("crates", "duet-time")
EXEMPT_FILE = ("src", "convert.rs")

# A character literal, including every escape form Rust writes.
CHAR_LITERAL = re.compile(r"'(?:\\(?:x[0-9A-Fa-f]{2}|u\{[0-9A-Fa-f]{1,6}\}|.)|[^'\\])'")

IDENT_START = re.compile(r"[A-Za-z_]")

# Every prefix Rust writes before a string literal, longest first so `br`
# never matches as a bare `b`. A prefix that carries an `r` opens a raw
# literal, which processes no escape (CG2).
STRING_PREFIXES = ("br", "cr", "rb", "b", "c", "r")


def workspace(root):
    """The workspace one directory resolves, or None when it is bad (CG8).

    Returns `(workspace root, [(member root, [target source path, ...])])`.
    Every path is canonical. `cargo metadata` is the one source of the file
    set, and a workspace it refuses is a failure rather than an empty pass.
    """
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--no-deps", "--format-version", "1"],
            cwd=root,
            capture_output=True,
            text=True,
            check=False,
        )
    except OSError:
        return None
    if result.returncode != 0:
        return None
    try:
        parsed = json.loads(result.stdout)
        packages = parsed["packages"]
        top = os.path.realpath(parsed["workspace_root"])
    except (ValueError, KeyError, TypeError):
        return None
    members = []
    for package in packages:
        member = os.path.realpath(os.path.dirname(package["manifest_path"]))
        targets = [os.path.realpath(target["src_path"]) for target in package["targets"]]
        members.append((member, targets))
    return (top, members)


def under(parent, path):
    """Whether one path sits inside one directory. Both are canonical."""
    return path == parent or path.startswith(parent + os.sep)


def build_directories(top, members):
    """The anchored build directories CG1 excludes, as canonical paths.

    The Cargo build directory sits at the workspace root and at a member
    root, and nowhere else. Revision 11 tested every path component at every
    depth instead (critic WR-8).
    """
    found = [os.path.join(top, "target")]
    for member, _targets in members:
        found.append(os.path.join(member, "target"))
    return found


def owns(member, path, canonical, builds):
    """Whether one member owns one scanned file, outside every build tree.

    The file answers for its own path and for the path a symbolic link
    resolves to, so neither spelling can drop a file the member compiles.
    """
    if not (under(member, path) or under(member, canonical)):
        return False
    return not any(under(build, path) or under(build, canonical) for build in builds)


def source_files(top, members):
    """CG1. Every `.rs` file the target source paths of a member reach.

    A directory walk of the manifest directory cannot state which files a
    member owns, so revision 11 guessed with a component name and lost every
    file under a source module named `target` (critic WR-8). The set now
    starts at each target's own `src_path` and grows to the `.rs` files
    beside it. The extension test ignores case (critic concern 19).
    """
    builds = build_directories(top, members)
    found = set()
    for member, targets in members:
        for target in targets:
            if owns(member, target, target, builds):
                found.add(target)
            for base, _dirs, names in os.walk(os.path.dirname(target)):
                for name in names:
                    if not name.lower().endswith(".rs"):
                        continue
                    path = os.path.join(base, name)
                    canonical = os.path.realpath(path)
                    if owns(member, path, canonical, builds):
                        found.add(canonical)
    return sorted(found)


def exempt_path(top, members):
    """CG7. The one file that may hold a cast, canonical, or None.

    The exempt file must sit inside the canonical `duet-time` member root,
    and that member root must be one `cargo metadata` lists. A symbolic link
    at the exempt path that points outside the member therefore exempts
    nothing, and the module it points at keeps its own verdict (critic
    concern 2).
    """
    member = os.path.realpath(os.path.join(top, *EXEMPT_MEMBER))
    if not any(root == member for root, _targets in members):
        return None
    candidate = os.path.realpath(os.path.join(member, *EXEMPT_FILE))
    return candidate if under(member, candidate) else None


def lex(text):
    """CG2 and CG2b. Classify every byte, then blank everything but code.

    One left-to-right pass over five states: code, line comment, block
    comment, string, and character literal. The pass decides what a token
    means from the state it is already in, so a `/*` inside a string opens no
    comment and a `"` inside a comment opens no string. Revision 9 removed
    block comments, then line comments, then strings, each with its own
    regular expression, so two strings that spell `/*` and `*/` hid every
    cast between them (critic W10).

    A newline is kept, so a reported line number is the file's own. Every
    other byte outside code becomes a space, so a later offset stays put.
    """
    out = []
    index = 0
    length = len(text)
    while index < length:
        char = text[index]
        if IDENT_START.match(char) and (
            index == 0 or not re.match(r"[A-Za-z0-9_]", text[index - 1])
        ):
            end = prefixed_literal(text, index)
            if end is not None:
                out.append(blanked(text[index:end]))
                index = end
                continue
        if char == '"':
            probe = end_of_quoted(text, index)
            out.append(blanked(text[index:probe]))
            index = probe
            continue
        if char == "'":
            match = CHAR_LITERAL.match(text, index)
            if match:
                out.append(blanked(match.group(0)))
                index = match.end()
                continue
            # A lifetime, not a literal. It is code.
            out.append(char)
            index += 1
            continue
        if text.startswith("//", index):
            end = text.find("\n", index)
            end = length if end < 0 else end
            out.append(blanked(text[index:end]))
            index = end
            continue
        if text.startswith("/*", index):
            depth = 1
            probe = index + 2
            while probe < length and depth:
                if text.startswith("/*", probe):
                    depth += 1
                    probe += 2
                elif text.startswith("*/", probe):
                    depth -= 1
                    probe += 2
                else:
                    probe += 1
            out.append(blanked(text[index:probe]))
            index = probe
            continue
        out.append(char)
        index += 1
    return "".join(out)


def end_of_quoted(text, index):
    """The end of an ordinary string literal that opens at `index`.

    An ordinary literal processes `\\` escapes, so an escaped quotation mark
    does not close it. A raw literal processes none, which is why
    `prefixed_literal` scans for its own closer rather than calling this.
    """
    length = len(text)
    probe = index + 1
    while probe < length:
        if text[probe] == "\\":
            probe += 2
            continue
        if text[probe] == '"':
            probe += 1
            break
        probe += 1
    return probe


def prefixed_literal(text, index):
    """The end of a string literal that opens at `index`, or None (CG2).

    Rust writes the six prefixes of `STRING_PREFIXES` before a string, and a
    prefix that carries an `r` opens a RAW literal. A raw literal processes no
    escape and ends at a quotation mark followed by its own hash run, so its
    text may hold a bare quotation mark.

    Revision 10 read a bare `r` only, and it refused even that when an
    identifier byte sat in front. `br#"a " b"#` therefore opened an ORDINARY
    string at the first quotation mark, the raw text closed it early, and the
    next quotation mark opened a second false string that ran on and hid every
    cast inside it. That is a vacuous green, and CG3b states that this guard
    prefers a false red to a miss (critic W-3).

    A hash run needs an `r`, because `b#"` and `c#"` are not Rust.
    """
    length = len(text)
    for prefix in STRING_PREFIXES:
        if not text.startswith(prefix, index):
            continue
        probe = index + len(prefix)
        hashes = 0
        while probe < length and text[probe] == "#":
            hashes += 1
            probe += 1
        if probe >= length or text[probe] != '"':
            continue
        raw = "r" in prefix
        if hashes and not raw:
            continue
        if not raw:
            return end_of_quoted(text, probe)
        closer = '"' + "#" * hashes
        end = text.find(closer, probe + 1)
        return length if end < 0 else end + len(closer)
    return None


def blanked(span):
    """A span with every byte but a newline replaced by a space."""
    return "".join("\n" if char == "\n" else " " for char in span)


def blank_span(chars, start, end):
    """Replace a span with spaces, so every later offset stays put."""
    for index in range(start, end + 1):
        if chars[index] != "\n":
            chars[index] = " "


class Unterminated(Exception):
    """CG4b. A `use` token with no terminating semicolon."""

    def __init__(self, line):
        super().__init__(line)
        self.line = line


def drop_use_items(text):
    """CG4. Remove each `use` item as a whole, from `use` to its `;`.

    CG4b. A `use` with no terminating `;` raises rather than blanking to the
    end of the file. Revision 8 blanked, which disabled CG3 for the whole
    file and reported success it did not earn (critic R4).
    """
    chars = list(text)
    for match in re.finditer(r"\buse\b", text):
        line_start = text.rfind("\n", 0, match.start()) + 1
        lead = text[line_start : match.start()].strip()
        # A `use` item stands in statement position. `(use $i:ident)` inside a
        # macro pattern does not, and blanking from it would swallow the next
        # statement and every cast in it (critic R4).
        if lead not in ("", "pub") and not lead.endswith((";", "{", "}")):
            continue
        end = text.find(";", match.end())
        if end < 0:
            raise Unterminated(text[: match.start()].count("\n") + 1)
        blank_span(chars, match.start(), end)
    return "".join(chars)


TYPE_PATH = re.compile(r"^&?(?:mut\s+)?[A-Za-z_][A-Za-z0-9_]*(?:\s*::\s*[A-Za-z_][A-Za-z0-9_]*)*$")


def type_path(text):
    """Whether `text` is one type path and holds no expression token.

    CG3b prefers a false red to a miss, so the test is strict on both sides
    of the `as`. An expression token, a call, a field access, an operator, a
    literal, or a space between two identifiers all fail it (critic R3).
    """
    stripped = text.strip()
    if not stripped or not TYPE_PATH.match(stripped):
        return False
    return not re.search(r"[(){}\[\].,;+\-*/%!&|^<>=?\"\']|\b\d", stripped)


def qualified_path_span(text, open_index):
    """The end index of an exempt qualified path at `open_index`, or None.

    CG3b and CG5. The span is exempt only when it closes on a matching `>`,
    carries exactly one whole-word `as` at its own depth, and holds one
    expression-free type path on each side of that `as`. `a<b && c as u32 > d`
    fails the left-side test, so the cast inside it still reaches CG3.
    """
    depth, cursor, close = 0, open_index, -1
    while cursor < len(text):
        char = text[cursor]
        if char == "<":
            depth += 1
        elif char == ">":
            depth -= 1
            if depth == 0:
                close = cursor
                break
        elif char in ";{}()[]":
            return None
        cursor += 1
    if close < 0:
        return None
    inner = text[open_index + 1 : close]
    positions = [
        m
        for m in re.finditer(r"\bas\b", inner)
        if inner[: m.start()].count("<") == inner[: m.start()].count(">")
    ]
    if len(positions) != 1:
        return None
    before = inner[: positions[0].start()]
    after = inner[positions[0].end() :]
    # The trailing type may carry its own generic list; take the head.
    head = after.split("<", 1)[0] if "<" in after else after
    if "<" in after and not after.rstrip().endswith(">"):
        return None
    if not type_path(before) or not type_path(head):
        return None
    return close


def drop_qualified_paths(text):
    """CG5. Remove every balanced qualified path span that carries an `as`."""
    chars = list(text)
    position = 0
    while position < len(chars):
        if chars[position] != "<" or (position and chars[position - 1] in "-=<>"):
            position += 1
            continue
        close = qualified_path_span("".join(chars), position)
        if close is None:
            position += 1
            continue
        blank_span(chars, position, close)
        position = close + 1
    return "".join(chars)


SUPPRESSION = re.compile(r"\b(?:allow|expect)\s*\(\s*clippy\s*::\s*as_conversions\b", re.S)


def attribute_spans(text):
    """Every attribute of one lexed file, as `(start offset, inner text)`.

    An attribute opens at `#[` or at `#![`, with whitespace anywhere between
    the tokens, and it closes at its own matching bracket. CG6 reads the span
    rather than one literal prefix, so a line break and a `cfg_attr` wrapper
    each stay inside one attribute (critic concern 1).
    """
    spans = []
    for match in re.finditer(r"#\s*!?\s*\[", text):
        depth, cursor = 0, match.end() - 1
        while cursor < len(text):
            if text[cursor] == "[":
                depth += 1
            elif text[cursor] == "]":
                depth -= 1
                if depth == 0:
                    spans.append((match.start(), text[match.end() : cursor]))
                    break
            cursor += 1
    return spans


def suppression_lines(text):
    """CG6. The line of every `as_conversions` suppression one file carries.

    The rule accepts the `allow` spelling and the `expect` spelling, the
    outer form and the inner form, and a `cfg_attr` wrapper, because each one
    silences the lint and none of them silences this text scan.
    """
    return [
        text[:start].count("\n") + 1
        for start, body in attribute_spans(text)
        if SUPPRESSION.search(body)
    ]


def member_globs(top):
    """Every `members` glob of the root manifest, as a list of patterns.

    CG1b derived its trees from the two literals `crates` and `tools` in
    revision 17. One line added to the root manifest then took a whole tree
    outside the rule, and the two literals also claimed the globs reach any
    depth while `crates/*` reaches exactly one (critic C17-W6, C17-W7). The
    globs are the source now, and `exclude` is deliberately NOT read: an
    excluded member is exactly what this rule exists to see.
    """
    path = os.path.join(top, "Cargo.toml")
    try:
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
    except OSError:
        return None
    found = re.search(r"(?ms)^\s*members\s*=\s*\[(.*?)\]", text)
    if found is None:
        return None
    return re.findall(r'"([^"]+)"', found.group(1))


def build_roots(top, members):
    """The canonical build directories the walk must not enter.

    The prune is ANCHORED, exactly as `build_directories` anchors CG1's own
    filter. Revision 17 pruned any directory named `target` at any depth, so a
    member at `crates/target/` never entered the expected set and one
    `exclude` line gave a clean run over a crate holding a cast (critic
    C17-6). The same unanchored shape had already been condemned in the
    sibling function for the same reason (critic WR-8).
    """
    roots = {os.path.realpath(os.path.join(top, "target"))}
    for member, _targets in members:
        roots.add(os.path.realpath(os.path.join(member, "target")))
    return roots


def expand(top, pattern):
    """Every directory one `members` pattern matches, Cargo's way.

    **Python's `glob` does not match a leading dot and Cargo's does** (critic
    C18-4). Cargo expands a `members` glob with the Rust `glob` crate, whose
    default `MatchOptions` leaves `require_literal_leading_dot` false, so
    `crates/*` makes `crates/.fixture` a member. Revision 18 called
    `glob.glob`, so one directory name and one `exclude` line gave a clean run
    over a crate holding a cast. This walk uses `os.scandir`, which has no dot
    rule of its own, and `fnmatch`, which matches the segment as Cargo does.

    The class recurred three times because each revision wrote its own
    approximation of Cargo's member resolution. This function states the one
    assumption it makes and nothing more: a `members` entry is a path of
    segments, and a segment is matched literally or by `fnmatch`.
    """
    here = [os.path.realpath(top)]
    for segment in pattern.strip("/").split("/"):
        nxt = []
        for base in here:
            if not os.path.isdir(base):
                continue
            if any(mark in segment for mark in "*?["):
                for entry in os.scandir(base):
                    if entry.is_dir() and fnmatch.fnmatch(entry.name, segment):
                        nxt.append(os.path.realpath(entry.path))
            else:
                candidate = os.path.realpath(os.path.join(base, segment))
                if os.path.isdir(candidate):
                    nxt.append(candidate)
        here = nxt
    return here


def sibling_patterns(entry):
    """One `members` entry plus the sibling set at EVERY level of its path.

    A manifest that lists a member by its full path cannot hide the crate
    beside it. Revision 18 expanded the parent alone, so an entry at depth two
    left depth one unreached and a second manifest shape hid a whole crate
    (critic C18-W8).
    """
    found = {entry}
    parts = entry.strip("/").split("/")
    # Depth ZERO is a level too (critic C19-W10). `range(1, len(parts))` gave
    # a one-segment entry such as `alpha` no pattern at all, so a workspace
    # with `members = ["alpha"]` and `exclude = ["beta"]` hid a real cast in
    # `beta` and the run printed `MEMBERS: 1   FILES: 1   FINDINGS: 0`. The
    # top level is the sibling set of every entry.
    for depth in range(0, len(parts)):
        found.add(("/".join(parts[:depth]) + "/*").lstrip("/"))
    return found


def expected_members(top, members):
    """CG1b part 2. Every member this workspace holds, from the globs.

    A directory that one `members` pattern matches, or that sits beside one at
    any level of its path, and that holds a `Cargo.toml`, is a member. The set
    is DERIVED, so an `exclude` line that removes a crate from `cargo metadata`
    leaves that crate in this set and the run fails by name (critic C16-2).
    """
    globs = member_globs(top)
    if globs is None:
        return None
    builds = build_roots(top, members)
    patterns = set()
    for entry in globs:
        patterns |= sibling_patterns(entry)
    found = set()
    for pattern in sorted(patterns):
        for canonical in expand(top, pattern):
            if any(under(root, canonical) for root in builds):
                continue
            if os.path.isfile(os.path.join(canonical, "Cargo.toml")):
                found.add(canonical)
    return found


def expected_files(top, covered, members):
    """CG1b part 3. Every `.rs` file under each covered member's `src/`."""
    builds = build_roots(top, members)
    found = set()
    for root in sorted(covered):
        source = os.path.join(root, "src")
        if not os.path.isdir(source):
            continue
        for directory, children, names in os.walk(source):
            children[:] = [
                child
                for child in children
                if not any(
                    under(build, os.path.realpath(os.path.join(directory, child)))
                    for build in builds
                )
            ]
            for name in names:
                if name.lower().endswith(".rs"):
                    found.add(os.path.realpath(os.path.join(directory, name)))
    return found


def coverage_failures(top, members, scanned):
    """CG1b. Every coverage failure, as a list of printable lines."""
    covered = {root for root, _targets in members}
    missing = []
    if not members or not scanned:
        missing.append(
            f"FAIL: the scan covers {len(members)} members and {len(scanned)} files;"
            " a zero denominator is not a clean run and the guard is fail-closed."
        )
        return missing
    wanted = expected_members(top, members)
    if wanted is None:
        missing.append(
            "FAIL: the root manifest states no `members` array, so CG1b can derive"
            " no expected set; the guard is fail-closed."
        )
        return missing
    for root in sorted(wanted - covered):
        missing.append(
            f"FAIL: {os.path.relpath(root, top)} holds a Cargo.toml and the scan does not"
            " cover it; the guard is fail-closed."
        )
    held = set(scanned)
    for path in sorted(expected_files(top, covered, members) - held):
        missing.append(
            f"FAIL: {os.path.relpath(path, top)} sits under a scanned member's src and the"
            " scan does not hold it; the guard is fail-closed."
        )
    return missing


def main(argv):
    """Run the ten rules over the current workspace and return an exit code.

    CG1 states that `cargo xtask check-conversions` takes no path argument,
    so this prototype takes none either and any argument is a usage failure
    (critic concern 18). A conversion probe changes directory into its own
    throwaway workspace, and no probe reads this repository.
    """
    if len(argv) != 1:
        sys.stdout.write("usage: conversion_check.py\n")
        return 2
    root = os.getcwd()
    resolved = workspace(root)
    if resolved is None:
        sys.stdout.write(
            f"FAIL: `cargo metadata --no-deps` refused {root}; the guard is fail-closed.\n"
        )
        return 2
    top, members = resolved
    files = source_files(top, members)
    # CG1b. The coverage rule. "Found 0 casts" and "scanned 0 files" print the
    # same success, and one `exclude` line in the workspace manifest took a
    # whole crate out of the scan while a real cast sat in it (critic C15-4).
    # The expected sets are DERIVED from the tree, so a count that grows with
    # the plan cannot outrun them (critic C16-2).
    coverage = coverage_failures(top, members, files)
    if coverage:
        for line in coverage:
            sys.stdout.write(line + "\n")
        return 2
    exempt = exempt_path(top, members)  # CG7
    findings = []
    for path in files:
        canonical = os.path.realpath(path)
        shown = os.path.relpath(canonical, top)  # critic C4
        try:
            with open(path, encoding="utf-8") as handle:
                text = lex(handle.read())
        except UnicodeDecodeError:  # CG8
            sys.stdout.write(
                f"FAIL: {shown}: the bytes are not UTF-8; the guard is fail-closed.\n"
            )
            return 2
        except OSError:  # CG8
            sys.stdout.write(
                f"FAIL: {shown}: the file does not open; the guard is fail-closed.\n"
            )
            return 2
        for line in suppression_lines(text):  # CG6
            if exempt is None or canonical != exempt:
                findings.append((shown, "suppression", f"line {line}"))
        if exempt is not None and canonical == exempt:
            continue
        try:
            scanned = drop_qualified_paths(drop_use_items(text))
        except Unterminated as broken:  # CG4b
            sys.stdout.write(
                f"FAIL: {shown}: a `use` at line {broken.line} has no `;`; "
                "the guard is fail-closed.\n"
            )
            return 2
        for match in re.finditer(r"\bas\b", scanned):  # CG3
            line = scanned[: match.start()].count("\n") + 1
            findings.append((shown, "cast", f"line {line}"))
    for shown, kind, detail in findings:
        sys.stdout.write(f"  {kind.upper()}: {shown}: {detail}\n")
    sys.stdout.write(
        f"MEMBERS: {len(members)}   FILES: {len(files)}   FINDINGS: {len(findings)}"
        f"   EXPECTED MEMBERS: {len(expected_members(top, members) or ())}"
        f"   EXPECTED FILES: "
        f"{len(expected_files(top, {root for root, _t in members}, members))}\n"
    )
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
