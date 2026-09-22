"""Run every conversion-guard probe of architecture section 1.9.

Each conversion probe needs a workspace and not a document, so this harness
builds one throwaway Cargo workspace per shape in a temporary directory, runs
`conversion_check.py` from inside it, and prints the exit code and the lines
the probe's row records. No probe here reads this repository and no probe here
reads the architecture document (section 1.9).

usage: probe_conversion.py <architecture.md> [probe id ...]

**It reads the section 1.9 `CP` rows and compares their recorded text with
its own runs** (critic C17-7). Revision 17 gave that machine to the placement
harness alone, so the twelve `CP` cells and the `PP25` cell were compared with
nothing and a Critic changed a recorded FAIL line to name a crate no run ever
printed with a green result. A recorded result is a measurement, so every
measurement needs a machine.

It exits 0 when every shape gives the exit code its row records and every
recorded fragment appears in the run of its own row, 1 when one does not, and
2 on a usage failure.
"""

import os
import re
import subprocess
import sys
import tempfile

import placement_check

HERE = os.path.dirname(os.path.abspath(__file__))
GUARD = os.path.join(HERE, "conversion_check.py")

MANIFEST = """[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
publish = false

[lib]
name = "{ident}"
path = "src/lib.rs"
"""

WORKSPACE = """[workspace]
resolver = "3"
members = [{members}]
"""


def write(path, text):
    """Write one file and make its parent directory."""
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)


def member(root, name, files):
    """One workspace member with the files a shape needs."""
    write(os.path.join(root, name, "Cargo.toml"), MANIFEST.format(name=name, ident=name.replace("-", "_")))
    for relative, text in files.items():
        write(os.path.join(root, name, relative), text)


def member_at(root, relative, package, files):
    """One workspace member at a nested path, with its own package name.

    `member` takes one name and uses it for both the directory and the
    package. A CG1b shape needs a member under `crates/`, so the directory and
    the package name are two arguments here.
    """
    write(
        os.path.join(root, relative, "Cargo.toml"),
        MANIFEST.format(name=package, ident=package.replace("-", "_")),
    )
    for name, text in files.items():
        write(os.path.join(root, relative, name), text)


def workspace(root, members):
    """The workspace manifest over the named members."""
    write(os.path.join(root, "Cargo.toml"), WORKSPACE.format(
        members=", ".join(f'"{name}"' for name in members)
    ))


CAST = "//! A probe.\n\n/// A probe.\npub fn probe(x: u64) -> u32 {\n    let n = x as u32;\n    n\n}\n"


def shape_cp1a(root):
    """A cast in a `benches/` file, which the target walk still reaches."""
    member(root, "m", {"src/lib.rs": "//! A probe.\n", "benches/bench.rs": CAST})
    write(os.path.join(root, "m", "Cargo.toml"),
          MANIFEST.format(name="m", ident="m") + '\n[[bench]]\nname = "bench"\npath = "benches/bench.rs"\nharness = false\n')
    workspace(root, ["m"])


def shape_cp1b(root):
    """A cast in a source module named `target`, which is not a build tree."""
    member(root, "m", {"src/lib.rs": "//! A probe.\npub mod target;\n",
                       "src/target/mod.rs": "//! A probe.\npub mod inner;\n",
                       "src/target/inner.rs": CAST})
    workspace(root, ["m"])


def shape_cp1c(root):
    """A cast in a file whose extension is uppercase."""
    member(root, "m", {"src/lib.rs": "//! A probe.\n", "src/Other.RS": CAST})
    workspace(root, ["m"])


def shape_cp2(root):
    """Every false-hit form: comments, strings, a raw string, a lifetime."""
    body = (
        "//! A probe as a doc comment.\n\n"
        "/* a block /* nested */ as comment */\n"
        "/// One as doc line.\n"
        "pub fn probe<'as_life>(s: &'as_life str) -> usize {\n"
        '    let plain = "as a string";\n'
        '    let raw = r##"say "as" here and x as u32"##;\n'
        "    let ch = 'a';\n"
        "    let _ = ch;\n"
        "    plain.len() + raw.len() + s.len()\n"
        "}\n"
    )
    member(root, "m", {"src/lib.rs": body})
    workspace(root, ["m"])


def two_line(before):
    """A file whose second or third line holds a real cast."""
    return "//! A probe.\n" + before + "/// A probe.\npub fn probe(x: u64) -> u32 { let n = x as u32; n }\n"


def shape_cp2b(index):
    """One of the five CG2b shapes, each with a real cast after it."""
    leads = [
        'const A: &str = "/*"; const B: &str = "*/";\n',
        'const A: &str = "// not a comment";\n',
        'const A: &str = br#"a " b"#.len().to_string().leak();\n',
        'const A: &str = cr#"a " b"#.count_bytes().to_string().leak();\n',
        'const A: &str = rb#"a " b"#.len().to_string().leak();\n',
    ]

    def build(root):
        member(root, "m", {"src/lib.rs": two_line(leads[index])})
        workspace(root, ["m"])

    return build


def shape_cp3(root):
    """One bare cast in `src/lib.rs`."""
    member(root, "m", {"src/lib.rs": CAST})
    workspace(root, ["m"])


def shape_cp3b(root):
    """A comparison pair that reads as a qualified path span."""
    body = (
        "//! A probe.\n\n/// A probe.\n"
        "pub fn probe(a: u32, b: u32, c: u64, d: u32) -> bool {\n"
        "    a < b && c as u32 > d\n}\n"
    )
    member(root, "m", {"src/lib.rs": body})
    workspace(root, ["m"])


def shape_cp4(root):
    """Both `use` forms plus a `use` token inside a macro pattern."""
    body = (
        "//! A probe.\n"
        "use core::fmt::Write;\n"
        "pub use core::fmt::Debug;\n"
        "macro_rules! m { (use $i:ident) => {} }\n"
        "/// A probe.\n"
        "pub fn probe(out: &mut String) { let _ = out.write_str(\"x\"); }\n"
    )
    member(root, "m", {"src/lib.rs": body})
    workspace(root, ["m"])


def shape_cp4b(root):
    """A trailing `use` with no semicolon, after a real cast."""
    body = CAST + "\n\n\n\nuse core::fmt::Write\n"
    member(root, "m", {"src/lib.rs": body})
    workspace(root, ["m"])


def shape_cp5(root):
    """Two qualified path spans and no cast."""
    body = (
        "//! A probe.\n\n/// A probe.\n"
        "pub fn probe(v: i128) -> i64 {\n"
        "    let _ = <u32 as Default>::default();\n"
        "    <i64 as TryFrom<i128>>::try_from(v).unwrap_or(0)\n}\n"
    )
    member(root, "m", {"src/lib.rs": body})
    workspace(root, ["m"])


def shape_cp6(root):
    """Five suppression forms in one file, and no cast."""
    body = (
        "#![allow( clippy::as_conversions )]\n"
        "//! A probe.\n\n"
        "#[expect( clippy::as_conversions, reason = \"probe\")]\n"
        "/// One.\npub fn one() {}\n"
        "#[expect(\n    clippy::as_conversions,\n    reason = \"probe\"\n)]\n"
        "/// Two.\npub fn two() {}\n"
        "#[cfg_attr(test, allow(clippy::as_conversions))]\n"
        "/// Three.\npub fn three() {}\n"
        "#[allow(clippy::as_conversions)]\n"
        "/// Four.\npub fn four() {}\n"
    )
    member(root, "m", {"src/lib.rs": body})
    workspace(root, ["m"])


def shape_cp7a(root):
    """The sanctioned cast inside the one exempt file."""
    member(root, "m", {"src/lib.rs": "//! A probe.\n"})
    member(os.path.join(root, "crates"), "duet-time", {"src/lib.rs": "//! A probe.\npub mod convert;\n",
                                                       "src/convert.rs": CAST})
    workspace(root, ["m", "crates/duet-time"])


def shape_cp7b(root):
    """The exempt path made a symbolic link to another member's live module."""
    member(root, "m", {"src/lib.rs": "//! A probe.\n"})
    member(root, "n", {"src/lib.rs": "//! A probe.\npub mod hot;\n", "src/hot.rs": CAST})
    member(os.path.join(root, "crates"), "duet-time", {"src/lib.rs": "//! A probe.\n"})
    os.symlink(os.path.join(root, "n", "src", "hot.rs"),
               os.path.join(root, "crates", "duet-time", "src", "convert.rs"))
    workspace(root, ["m", "n", "crates/duet-time"])


def shape_cp8a(root):
    """A `Cargo.toml` that does not parse."""
    member(root, "m", {"src/lib.rs": CAST})
    write(os.path.join(root, "Cargo.toml"), "[workspace\nmembers = [")


def shape_cp8b(root):
    """A member file that is not UTF-8, beside a real cast."""
    member(root, "m", {"src/lib.rs": CAST})
    with open(os.path.join(root, "m", "src", "other.rs"), "wb") as handle:
        handle.write(b"//! \xc3\x28 probe\n")
    workspace(root, ["m"])


def shape_cp8c(root):
    """A member file at mode 000, beside a real cast."""
    member(root, "m", {"src/lib.rs": CAST, "src/locked.rs": "//! A probe.\n"})
    os.chmod(os.path.join(root, "m", "src", "locked.rs"), 0)
    workspace(root, ["m"])


def shape_cp1b_exclude(root):
    """The Critic's own shape: five crates, one excluded, a real cast in it.

    Revision 16 answered this shape with `exit 0` at the production default
    floors, which is the vacuous green CG1b exists to stop (critic C16-2).
    Every member sits under `crates/`, so the derived member set holds all
    five and the scan covers four.
    """
    for name in ("duet-a", "duet-b", "duet-c", "duet-d"):
        member_at(root, os.path.join("crates", name), name, {"src/lib.rs": "//! A probe.\n"})
    member_at(root, os.path.join("crates", "duet-dsp"), "duet-dsp", {"src/lib.rs": CAST})
    path = os.path.join(root, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(
            '[workspace]\nresolver = "3"\nexclude = ["crates/duet-dsp"]\n'
            'members = ["crates/duet-a", "crates/duet-b", "crates/duet-c", "crates/duet-d"]\n'
        )


def shape_cp1b_empty(root):
    """A workspace with no member at all."""
    path = os.path.join(root, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write('[workspace]\nresolver = "2"\nmembers = []\n')


def shape_cp1b_target(root):
    """The Critic's shape: a member at `crates/target/` with a real cast.

    Revision 17 pruned any directory named `target` at any depth, so this
    member never entered the expected set and one `exclude` line gave a clean
    run over a crate holding a cast (critic C17-6).
    """
    member_at(root, os.path.join("crates", "duet-a"), "duet-a", {"src/lib.rs": "//! A probe.\n"})
    member_at(root, os.path.join("crates", "target"), "duet-target", {"src/lib.rs": CAST})
    path = os.path.join(root, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(
            '[workspace]\nresolver = "3"\nexclude = ["crates/target"]\n'
            'members = ["crates/*"]\n'
        )


def shape_cp1b_dot(root):
    """The Critic's shape: a member whose directory name opens with a dot.

    Cargo's glob matches a leading dot and Python's does not, so revision 18
    could not see `crates/.fixture` and one `exclude` line gave a clean run
    over a crate holding a cast (critic C18-4).
    """
    member_at(root, os.path.join("crates", "duet-a"), "duet-a", {"src/lib.rs": "//! A probe.\n"})
    member_at(root, os.path.join("crates", ".fixture"), "fixture", {"src/lib.rs": CAST})
    path = os.path.join(root, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(
            '[workspace]\nresolver = "3"\nexclude = ["crates/.fixture"]\n'
            'members = ["crates/*"]\n'
        )


def shape_cp1b_deep(root):
    """A manifest that lists a member by full path, with a sibling excluded.

    Revision 18 expanded the parent of each entry alone, so an entry at depth
    two left depth one unreached (critic C18-W8).
    """
    member_at(
        root, os.path.join("crates", "group", "a"), "duet-a", {"src/lib.rs": "//! A probe.\n"}
    )
    member_at(root, os.path.join("crates", "standalone"), "duet-s", {"src/lib.rs": CAST})
    path = os.path.join(root, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(
            '[workspace]\nresolver = "3"\nexclude = ["crates/standalone"]\n'
            'members = ["crates/group/a"]\n'
        )


def shape_cp1b_top(root):
    """A member with NO slash, and a sibling of it at the top level excluded.

    `sibling_patterns` ran `range(1, len(parts))`, which yields no pattern at
    depth zero, so `members = ["alpha"]` with `exclude = ["beta"]` and a real
    cast in `beta` printed `MEMBERS: 1   FILES: 1   FINDINGS: 0` and exited 0
    (critic C19-W10). It is the fourth instance of the class after C15-4,
    C16-2, C18-4 and C18-W8, and it is latent in this plan because the
    manifest uses `crates/*` and `tools/*`.
    """
    member_at(root, "alpha", "duet-alpha", {"src/lib.rs": "//! A probe.\n"})
    member_at(root, "beta", "duet-beta", {"src/lib.rs": CAST})
    path = os.path.join(root, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(
            '[workspace]\nresolver = "3"\nexclude = ["beta"]\nmembers = ["alpha"]\n'
        )


def shape_cp1b_nested(root):
    """A nested fixture crate the `crates/*` glob does not make a member.

    Revision 17 walked both trees at any depth and reported a coverage failure
    over every `trybuild` fixture and every compile-fail case (critic C17-W7).
    The glob is the source now, so this shape is GREEN and it is the control
    that proves the rule stopped over-approximating.
    """
    member_at(root, os.path.join("crates", "duet-a"), "duet-a", {"src/lib.rs": "//! A probe.\n"})
    member_at(
        root,
        os.path.join("crates", "duet-a", "tests", "fixture"),
        "fixture",
        {"src/lib.rs": "//! A probe.\n"},
    )
    path = os.path.join(root, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write('[workspace]\nresolver = "3"\nmembers = ["crates/*"]\n')


def shape_cp1b_file(root):
    """A member whose one target sits outside `src/`, with a cast in `src/`.

    CG1 walks from each target's own directory, so the whole of `src/` is
    outside the scan and the derived file set is the only thing that sees it.
    """
    name = os.path.join("crates", "duet-e")
    member_at(root, name, "duet-e", {"other/lib.rs": "//! A probe.\n", "src/hidden.rs": CAST})
    path = os.path.join(root, name, "Cargo.toml")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(
            '[package]\nname = "duet-e"\nversion = "0.1.0"\nedition = "2024"\n'
            'publish = false\nautolib = false\n\n'
            '[lib]\nname = "duet_e"\npath = "other/lib.rs"\n'
        )
    workspace(root, ["crates/duet-e"])


SHAPES = [
    ("CP1-a", "a cast in a benches file", shape_cp1a, 1, ["CAST", "MEMBERS"]),
    ("CP1-b", "a cast in a source module named target", shape_cp1b, 1, ["CAST", "MEMBERS"]),
    ("CP1-c", "a cast in a file with an uppercase extension", shape_cp1c, 1, ["CAST", "MEMBERS"]),
    ("CP1-d", "one path argument", "ARGUMENT", 2, ["usage:"]),
    ("CP1b-exclude", "the Critic shape: five crates, one excluded, a real cast in it", shape_cp1b_exclude, 2, ["FAIL:"]),
    ("CP1b-empty", "a workspace with no member at all", shape_cp1b_empty, 2, ["FAIL:"]),
    ("CP1b-file", "a member whose one target sits outside src, with a cast in src", shape_cp1b_file, 2, ["FAIL:"]),
    ("CP1b-target", "the Critic shape: a member at crates/target, excluded, with a real cast", shape_cp1b_target, 2, ["FAIL:"]),
    ("CP1b-nested", "a nested fixture crate the glob does not reach, which is the green control", shape_cp1b_nested, 0, ["MEMBERS"]),
    ("CP1b-dot", "the Critic shape: a member whose name opens with a dot, excluded, with a real cast", shape_cp1b_dot, 2, ["FAIL:"]),
    ("CP1b-deep", "a member listed by full path with an excluded sibling one level up", shape_cp1b_deep, 2, ["FAIL:"]),
    ("CP1b-top", "the Critic shape: a member with no slash and an excluded top-level sibling", shape_cp1b_top, 2, ["FAIL:"]),
    ("CP2", "every false-hit form and no cast", shape_cp2, 0, ["MEMBERS"]),
    ("CP2b1", "two strings that spell a comment pair", shape_cp2b(0), 1, ["CAST", "MEMBERS"]),
    ("CP2b2", "a string that spells a line comment", shape_cp2b(1), 1, ["CAST", "MEMBERS"]),
    ("CP2b3", "a `br` raw string with a bare quotation mark", shape_cp2b(2), 1, ["CAST", "MEMBERS"]),
    ("CP2b4", "a `cr` raw string with a bare quotation mark", shape_cp2b(3), 1, ["CAST", "MEMBERS"]),
    ("CP2b5", "an `rb` raw string with a bare quotation mark", shape_cp2b(4), 1, ["CAST", "MEMBERS"]),
    ("CP3", "one bare cast", shape_cp3, 1, ["CAST", "MEMBERS"]),
    ("CP3b", "a comparison pair that holds a cast", shape_cp3b, 1, ["CAST", "MEMBERS"]),
    ("CP4", "both use forms and a macro pattern", shape_cp4, 0, ["MEMBERS"]),
    ("CP4b", "a use item with no terminating semicolon", shape_cp4b, 2, ["FAIL"]),
    ("CP5", "two qualified path spans", shape_cp5, 0, ["MEMBERS"]),
    ("CP6", "five suppression forms", shape_cp6, 1, ["SUPPRESSION", "MEMBERS"]),
    ("CP7a", "the sanctioned cast in the exempt file", shape_cp7a, 0, ["MEMBERS"]),
    ("CP7b", "the exempt path as a link to a live module", shape_cp7b, 1, ["CAST", "MEMBERS"]),
    ("CP8a", "a manifest that does not parse", shape_cp8a, 2, ["FAIL"]),
    ("CP8b", "a member file that is not UTF-8", shape_cp8b, 2, ["FAIL"]),
    ("CP8c", "a member file the guard cannot open", shape_cp8c, 2, ["FAIL"]),
]


def run(root, argument=False):
    """`(exit code, stdout)` of one guard run from inside one workspace.

    **No shape sets any environment variable.** Revision 16 read
    `CONVERSION_MEMBER_FLOOR` and `CONVERSION_FILE_FLOOR` at the production
    entry point, so one assignment disabled the guard and no line of the
    specification named the seam (critic C16-2). CG1b derives its expected
    sets from the tree, so a throwaway workspace needs no override: a shape
    that keeps its members outside `crates/` and `tools/` derives an empty
    expected set and meets the zero test alone.
    """
    result = subprocess.run(
        [sys.executable, GUARD] + (["probe"] if argument else []),
        cwd=root,
        capture_output=True,
        text=True,
        check=False,
    )
    return result.returncode, result.stdout





# ONE fragment oracle, imported and never copied (critic C18-N6, C19-W6).
# Revision 19 left this harness with its own `OUTPUT_HEAD`, which differed
# from `probe_fragments` by one character class, so one cell was compared by
# one rule and one by another.
from probe_fragments import flatten, recorded_fragments  # noqa: E402


def recorded_cells(document):
    """Every `CP` probe id mapped to the `Recorded result` cell of its row."""
    with open(document, encoding="utf-8") as handle:
        source = handle.read()
    rows, reason = placement_check.read_block(source, "probe-table")
    if reason is not None:
        return None
    cells = {}
    for row in rows:
        if len(row) < 5:
            continue
        for probe in re.findall(r"\bCP\d+[a-z]?\b", row[2]):
            cells[probe] = row[4]
    return cells


def row_of(cells, probe_id):
    """The `CP` row that covers one shape id, by longest prefix."""
    if probe_id in cells:
        return probe_id
    best = None
    for probe in cells:
        if probe_id.startswith(probe) and (best is None or len(probe) > len(best)):
            best = probe
    return best


def fragment_missing(fragment, produced):
    """Whether one recorded fragment is absent from the produced text."""
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


def main(argv):
    """Build each shape, run the guard, and return an exit code."""
    if len(argv) < 2:
        sys.stdout.write("usage: probe_conversion.py <architecture.md> [probe id ...]\n")
        return 2
    document = argv[1]
    if not os.path.isfile(document):
        sys.stdout.write(f"FAIL: cannot open {document}; the harness is fail-closed.\n")
        return 2
    cells = recorded_cells(document)
    if cells is None:
        sys.stdout.write(
            "FAIL: the section 1.9 probe table does not read, so no recorded text can be"
            " compared; the harness is fail-closed.\n"
        )
        return 2
    wanted = set(argv[2:])
    produced = {}
    bad = 0
    for probe_id, description, build, expected, tokens in SHAPES:
        if wanted and probe_id not in wanted:
            continue
        with tempfile.TemporaryDirectory() as scratch:
            if build == "ARGUMENT":
                shape_cp3(scratch)
                code, text = run(scratch, argument=True)
            else:
                build(scratch)
                code, text = run(scratch)
            lines = []
            for token in tokens:
                for line in text.splitlines():
                    if token in line:
                        lines.append(line.strip().replace(scratch, "<root>"))
                        break
            row = row_of(cells, probe_id)
            if row:
                produced[row] = produced.get(row, "") + "\n" + text.replace(scratch, "<root>")
            ok = code == expected and len(lines) == len(tokens)
            bad += 0 if ok else 1
            sys.stdout.write(f"{probe_id:8s} exit {code}  {'OK ' if ok else 'BAD'}  {description}\n")
            for line in lines:
                sys.stdout.write(f"{'':8s}        {line}\n")
            if not ok:
                sys.stdout.write(f"{'':8s}        expected exit {expected}; output was:\n{text}\n")
    stale, compared = [], 0
    if not wanted:
        for probe in sorted(cells):
            pool = flatten(produced.get(probe, ""))
            for fragment in recorded_fragments(cells[probe]):
                compared += 1
                if fragment_missing(fragment, pool):
                    stale.append((probe, fragment))
    for probe, fragment in stale:
        sys.stdout.write(
            f"TEXT:  {probe}: section 1.9 records a line no run of that row produced:"
            f" {fragment}\n"
        )
    # A FLOOR, exactly as `probe_run.py` and `probe_closure.py` print one
    # (critic C17-7, C19-W6). One recorded line per shape, with no discount:
    # a cell emptied of its recorded lines is a cell that asserts nothing, and
    # a compared count with no denominator cannot see that.
    floor = 0 if wanted else len(SHAPES)
    short = compared < floor
    if short:
        sys.stdout.write(
            f"TEXT:  the CP cells record {compared} output lines and the floor is"
            f" {floor}; the harness is fail-closed.\n"
        )
    sys.stdout.write(
        f"CONVERSION PROBES BAD: {bad}     RECORDED FRAGMENTS: {compared}"
        f"     FLOOR: {floor}     FRAGMENTS BAD: {len(stale) + (1 if short else 0)}\n"
    )
    return 1 if bad or stale or short else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
