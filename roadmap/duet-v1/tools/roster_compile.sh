#!/usr/bin/env bash
# The roster compile: guard rule PG25, with probe PP25.
#
# Prototype for `cargo xtask check-roster <document>`, which chunk M0 ports to
# `tools/xtask/src/check_roster.rs`. The contract lives in architecture section
# 1.5, "The placement guard", rule PG25. Section 1.9 records the probe.
#
# The guard reads the document; the compiler reads the lint table. PG1 to PG24
# hold the first half and this rule holds the second. It writes a scratch Cargo
# workspace of one crate per section 1.5 crate row, gives each crate the section
# 1.3 edges as path dependencies, copies the repository's own `[workspace.lints]`
# table, `.cargo/config.toml`, `clippy.toml`, `rust-toolchain.toml`, and
# `Cargo.lock`, and runs the real lint invocation over it.
#
# usage: roster_compile.sh <architecture.md> <scratch-dir> <repo-root>
#
# It exits 2 on a usage or input failure, 1 on a finding, and 0 when the roster
# compiles clean and every recorded size is true.
set -uo pipefail

if [ "$#" -ne 3 ]; then
    printf 'usage: roster_compile.sh <architecture.md> <scratch-dir> <repo-root>\n'
    exit 2
fi

DOCUMENT="$1"
SCRATCH="$2"
REPO="$3"

if [ ! -f "$DOCUMENT" ]; then
    printf 'FAIL: cannot open %s; the guard is fail-closed.\n' "$DOCUMENT"
    exit 2
fi
if [ ! -f "$REPO/Cargo.toml" ]; then
    printf 'FAIL: %s holds no Cargo.toml; the guard is fail-closed.\n' "$REPO"
    exit 2
fi
case "$SCRATCH" in
    "$REPO"|"$REPO"/*)
        printf 'FAIL: the scratch workspace must sit outside the repository.\n'
        exit 2
        ;;
esac

mkdir -p "$SCRATCH" || exit 2
WORKSPACE="$SCRATCH/workspace"

# ONE scratch directory, ONE cargo target, and the target does not outlive the
# run that made it. A `target` for this workspace holds the whole `gpui`
# dependency tree, which is several gigabytes, and one run per revision filled
# the volume and stopped every shell on the machine.
#
# The owner of the target is the process that creates it. When a caller
# exports `ROSTER_TARGET_DIR`, that caller owns the target and deletes it;
# `probe_roster.sh` does exactly that, so its eleven shapes share one build
# rather than paying for eleven. When no caller does, this script owns the
# target and the trap below deletes it on every exit path.
# An exported path is VALIDATED before it is used, because nothing else does
# and the value reaches an `rm -rf` in `probe_roster.sh` (critic C21-W4).
# Both scripts already refuse a scratch directory inside the repository, and
# neither validated this one.
if [ -n "${ROSTER_TARGET_DIR:-}" ]; then
    case "$ROSTER_TARGET_DIR" in
        /*) ;;
        *)
            printf 'FAIL: ROSTER_TARGET_DIR must be an absolute path; the guard is fail-closed.\n'
            exit 2
            ;;
    esac
    # THE COMPARE IS ON THE RESOLVED PATH (critic C22I-W6). Revision 22 tested
    # the exported string against the repository string with a shell `case`,
    # so an absolute path OUTSIDE the repository that RESOLVED inside it
    # passed every arm and the value went on to an `rm -rf`. CG7 already
    # records that a symbolic link defeats a path exemption, and the
    # conversion guard resolves; these two scripts did not take that lesson.
    if [ -d "$ROSTER_TARGET_DIR" ]; then
        ROSTER_REAL=$(cd "$ROSTER_TARGET_DIR" && pwd -P)
    else
        ROSTER_REAL=$(cd "$(dirname "$ROSTER_TARGET_DIR")" 2>/dev/null && pwd -P)/$(basename "$ROSTER_TARGET_DIR")
    fi
    REPO_REAL=$(cd "$REPO" 2>/dev/null && pwd -P)
    case "$ROSTER_REAL" in
        "$REPO_REAL"|"$REPO_REAL"/*)
            printf 'FAIL: ROSTER_TARGET_DIR must sit outside %s; the guard is fail-closed.\n' "$REPO"
            exit 2
            ;;
    esac
    export CARGO_TARGET_DIR="$ROSTER_TARGET_DIR"
else
    export CARGO_TARGET_DIR="$SCRATCH/target"
    trap 'rm -rf "$CARGO_TARGET_DIR"' EXIT INT TERM
fi

cat > "$SCRATCH/roster_generate.py" <<'PYTHON_GENERATE'
"""The generator half of `roster_compile.sh`, which holds guard rule PG25.

It reads the architecture document and writes a scratch Cargo workspace: one
crate per section 1.5 crate row, with the section 1.3 edges as path
dependencies, so a cycle or a missing edge fails at `cargo`. It copies the
repository's `[workspace.lints]` table and relaxes the six lints of the stated
profile and nothing else. Section 1.5 states the rule and section 1.9 pairs it
with probe PP25.
"""

import os
import re
import shutil
import sys


def matching_brace(code, open_index):
    depth, index = 0, open_index
    while index < len(code):
        if code[index] == "{":
            depth += 1
        elif code[index] == "}":
            depth -= 1
            if depth == 0:
                return index
        index += 1
    return len(code)


def split_top(text, separator=","):
    depth, start, out = 0, 0, []
    for index, char in enumerate(text):
        if char in "({[<":
            depth += 1
        elif char in ")}]>":
            depth -= 1
        elif char == separator and depth == 0:
            out.append(text[start:index])
            start = index + 1
    out.append(text[start:])
    return [piece for piece in out if piece.strip()]


def rust_blocks(source):
    return [m.group(1) for m in re.finditer(r"```rust\n(.*?)```", source, re.S)]


# Every block this half reads, by the id the document's marker states (DR7,
# PG27). The minimum row count here and the minimum the marker states must be
# one number, exactly as the substitution list and `SUBSTITUTIONS` must be one
# set (critic CR-14, WR-3).
DATA_BLOCKS = {
    "ownership-table": ("The type ownership table", "table", 16),
    "edge-list": ("The internal edge list", "text", 18),
    "external-paths": ("Where every external name comes from", "text", 70),
    "pins": ("The external crate pins the roster compile uses", "text", 13),
    "constants": ("Every workspace constant", "rust", 122),
    "substitutions": ("Every substitution the roster compile applies", "text", 8),
    "recorded-sizes": ("Every size the guard records", "text", 51),
    "drop-impls": ("Every declaration with a hand-written Drop impl", "text", 3),
    "impl-sites": ("Every impl block the roster compiles", "text", 65),
}

MARKER = re.compile(r"(?m)^<!-- GUARD BLOCK id=([a-z0-9-]+) rows>=([0-9]+) -->$")


def read_block(source, block_id):
    """One registered block, as `(rows, reason)`. See DR7 and PG27."""
    heading, kind, minimum = DATA_BLOCKS[block_id]
    found = list(re.finditer(r"(?m)^#### " + re.escape(heading) + r"\s*$", source))
    if len(found) != 1:
        return None, f"the heading `#### {heading}` appears {len(found)} times"
    start = found[0].end()
    rest = source[start:]
    stop = re.search(r"(?m)^#{1,4} ", rest)
    region = rest[: stop.start()] if stop else rest
    marks = list(MARKER.finditer(region))
    if len(marks) != 1:
        return None, f"the marker line appears {len(marks)} times under the heading"
    mark = marks[0]
    if mark.group(1) != block_id:
        return None, f"the marker states id `{mark.group(1)}` and the register states `{block_id}`"
    if int(mark.group(2)) != minimum:
        return None, f"the marker states rows>={mark.group(2)} and the register states {minimum}"
    tail = region[mark.end() :]
    if not tail.startswith("\n"):
        return None, "the marker does not end its own line"
    tail = tail[1:]
    if kind == "table":
        head = re.match(r"\|[^\n]*\|[ \t]*\n\|[-\s|:]+\|[ \t]*\n", tail)
        if not head:
            return None, "no table header follows the marker"
        rows = []
        for line in tail[head.end() :].splitlines():
            if not line.startswith("|"):
                break
            rows.append([cell.strip() for cell in line.split("|")[1:-1]])
    else:
        fence = re.match(r"```([a-z]*)\n(.*?)^```", tail, re.S | re.M)
        if not fence:
            return None, "no fenced block follows the marker"
        wanted = "rust" if kind == "rust" else "text"
        if fence.group(1) != wanted:
            return None, f"the fence is `{fence.group(1)}` and `{wanted}` is required"
        rows = [line for line in fence.group(2).splitlines() if line.strip()]
    if len(rows) < minimum:
        return None, f"it holds {len(rows)} rows and the stated minimum is {minimum}"
    return rows, None


def read_blocks(source):
    """Every registered block, or a list of reasons."""
    parsed, failures = {}, []
    for block_id in DATA_BLOCKS:
        rows, reason = read_block(source, block_id)
        if reason is not None:
            failures.append((block_id, reason))
            continue
        parsed[block_id] = rows
    return parsed, failures


def fenced_map(blocks, block_id):
    out = {}
    for line in blocks[block_id]:
        parts = line.split()
        if len(parts) >= 2:
            out[parts[0]] = parts[1:]
    return out


def ownership_table(blocks):
    owner, order = {}, []
    for cells in blocks["ownership-table"]:
        if len(cells) < 3:
            continue
        crate = cells[0].strip("`")
        if crate not in order:
            order.append(crate)
        for name in re.findall(r"`([A-Za-z0-9_]+)`", cells[1]):
            owner[name] = crate
    return owner, order


def edge_list(blocks):
    edges = {}
    text = re.sub(r",\s*\n\s+", ", ", "\n".join(blocks["edge-list"]))
    for line in text.splitlines():
        if "->" not in line:
            continue
        left, right = line.split("->", 1)
        name = normalize(left.strip())
        edges[name] = [normalize(p.strip()) for p in right.split(",") if p.strip()]
    return edges


def normalize(crate):
    """The section 1.3 list writes `duet`; the section 1.5 table writes `crates/duet`."""
    return "crates/duet" if crate == "duet" else crate


ITEM = re.compile(
    r"((?:[ \t]*#\[[^\n]*\]\n)*)"
    r"[ \t]*(pub(?:\([a-z]+\))?\s+)?(struct|enum|trait)\s+([A-Z][A-Za-z0-9]*)(<[^>{(;]*>)?"
    r"(\s*:[^{;(]*)?"
)


def join_attributes(code):
    """Each attribute on one line, so an attribute run is a run of lines.

    A body line is then never part of an attribute run, which is the parse
    defect that made one declaration swallow the next.
    """
    out, index = [], 0
    while index < len(code):
        if code.startswith("#[", index):
            depth, end = 0, index + 1
            while end < len(code):
                if code[end] == "[":
                    depth += 1
                elif code[end] == "]":
                    depth -= 1
                    if depth == 0:
                        break
                end += 1
            joined = re.sub(r"\\\n\s*", "", code[index:end + 1])
            out.append(re.sub(r"\s*\n\s*", " ", joined))
            index = end + 1
            continue
        out.append(code[index])
        index += 1
    return "".join(out)


def strip_docs(text):
    """Every whole-line comment removed.

    The roster's prose is documentation, and a doc line that quotes an
    attribute would otherwise read as one. `/* private */` survives, because
    it is a block comment and it marks a body the roster does not spell out.
    """
    return re.sub(r"^[ \t]*//[^\n]*\n", "", text, flags=re.M)


def items(code):
    out = []
    for match in ITEM.finditer(code):
        attrs, _vis, kind, name, generics, bound = match.groups()
        generics = (generics or "") + (bound or "" if kind == "trait" else "")
        tail = code[match.end():]
        brace = tail.find("{")
        semi = tail.find(";")
        paren = tail.find("(")
        if kind in ("struct", "enum") and 0 <= paren < (brace if brace >= 0 else len(tail)) and (
            semi < 0 or paren < semi
        ):
            close = tail.find(")")
            out.append((name, attrs or "", kind, "tuple", tail[paren + 1:close], generics or ""))
        elif brace >= 0 and (semi < 0 or brace < semi):
            absolute = match.end() + brace
            close = matching_brace(code, absolute)
            out.append((name, attrs or "", kind, "braced", code[absolute + 1:close], generics or ""))
        else:
            out.append((name, attrs or "", kind, "unit", "", generics or ""))
    return out


IMPL = re.compile(
    r"\bimpl\b(?:\s*<[^>]*>)?\s+((?:[A-Za-z_][A-Za-z0-9_]*\s*::\s*)*[A-Z][A-Za-z0-9]*"
    r"(?:\s*<[^>]*>)?)\s+for\s+([A-Z][A-Za-z0-9]*)\s*\{"
)
INHERENT = re.compile(r"\bimpl\s+([A-Z][A-Za-z0-9]*)\s*\{")
BODILESS_FN = re.compile(r"\bfn\s+[a-z_][A-Za-z0-9_]*\s*(?:<[^>]*>)?\s*\([^{;]*\)\s*(?:->[^{;]*)?;")
BODILESS_CONST = re.compile(r"\bconst\s+[A-Z][A-Za-z0-9_]*\s*:[^=;{]*;")


def full_impls(code):
    """Every `impl` block whose every item carries a body, and every mixed one.

    Returns `(full, mixed)`. A MIXED block carries at least one body beside at
    least one bodiless signature. The seventeenth Critic planted three gate
    defects inside `impl Transport`, which carries a bodiless signature, and
    all three were green: the whole block leaves the roster, so a real body
    beside a signature escapes PG25 and no counter moves (critic N17-4). No
    block in this document is mixed today, so the rule is a guard against the
    day one is written rather than a fix for a live defect.
    """
    out, mixed = [], []
    for pattern, group in ((IMPL, 2), (INHERENT, 1)):
        for match in pattern.finditer(code):
            open_index = code.index("{", match.end() - 1)
            close = matching_brace(code, open_index)
            text = code[match.start():close + 1]
            bodiless = bool(BODILESS_FN.search(text) or BODILESS_CONST.search(text))
            has_body = bool(re.search(r"\)\s*(?:->[^;{]*)?\{", text[text.index("{") + 1:]))
            if bodiless and has_body:
                mixed.append((match.group(group), text))
                continue
            if bodiless:
                continue
            out.append((match.group(group), text))
    return out, mixed


CONST = re.compile(r"pub const ([A-Z][A-Z0-9_]*)\s*:\s*([A-Za-z0-9_:<> ]+?)\s*=\s*([^;]+);")


def constants(blocks):
    out = {}
    body = "\n".join(blocks["constants"])
    for match in CONST.finditer(body):
        value = match.group(3).strip()
        # An integer literal, or arithmetic over integer literals and other
        # constants of this same block. `MAX_SLOT_METERS` is
        # `MAX_STRIPS * 2 * MAX_SLOTS`, which DR3 prefers to a fourth literal,
        # and revision 21's filter took integer literals alone (critic C21-7).
        # Rust resolves constants in any order inside one crate, so the emitted
        # order does not matter.
        if not re.match(r"^[0-9_A-Z][0-9_A-Z* +-]*$", value):
            continue
        crates = re.findall(r"^// (duet-[a-z]+)", body[:match.start()], re.M)
        out[match.group(1)] = (
            crates[-1] if crates else "",
            f"pub const {match.group(1)}: {match.group(2).strip()} = {value};",
        )
    return out


def mask_arms(code):
    """Every enum arm's leading identifier removed.

    An arm name is not a type, so it must not make the roster compile import
    a name the crate never uses. This is the PG8 rule in the generator.
    """
    out = code
    for match in reversed(list(re.finditer(r"\benum\s+[A-Z][A-Za-z0-9]*\s*\{", out))):
        open_index = out.index("{", match.start())
        close_index = matching_brace(out, open_index)
        body = out[open_index + 1:close_index]
        masked = ",".join(
            re.sub(r"^(\s*)[A-Z][A-Za-z0-9]*", r"\1", arm, count=1) for arm in split_top(body)
        )
        out = out[:open_index + 1] + masked + out[close_index:]
    return out


def crate_ident(crate):
    return ("duet" if crate == "crates/duet" else crate).replace("-", "_")


def package_name(crate):
    return "duet" if crate == "crates/duet" else crate


def make_fields_public(body, shape):
    """Every field becomes `pub`.

    The roster carries no method and no function body, so a private field has
    no reader and `dead_code` reads every one of them as unused. Field
    visibility is section 5.1's rule and the chunk's business, never the
    roster's.
    """
    if shape == "tuple":
        return ", ".join("pub " + piece.strip() for piece in split_top(body))
    pieces = []
    for piece in split_top(body):
        text = piece.strip("\n")
        head = re.match(r"^((?:\s*#\[[^\]]*\]\s*)*)\s*", text)
        prefix = head.group(1) if head else ""
        rest = text[len(prefix):].strip()
        if re.match(r"^[a-z_][A-Za-z0-9_]*\s*:", rest):
            rest = "pub " + rest
        pieces.append(prefix + rest)
    return ",\n    ".join(pieces)


def emit_item(name, attrs, kind, shape, body, generics, private_marker):
    derives = re.search(r"#\[derive\(([^)]*)\)\]", attrs)
    dropped = set()
    if derives:
        names = [n.strip() for n in derives.group(1).split(",") if n.strip()]
        kept = [n for n in names if n not in ("Error", "IntoElement")]
        dropped = {n for n in names if n in ("Error", "IntoElement")}
        attrs = attrs.replace(derives.group(0), "#[derive(" + ", ".join(kept) + ")]" if kept else "")
    if kind != "trait" and body.strip():
        body = make_fields_public(body, shape)
    text = attrs.rstrip("\n")
    if text:
        text += "\n"
    if shape == "tuple":
        text += f"pub {kind} {name}{generics}({body});\n"
    elif shape == "unit":
        text += f"pub {kind} {name}{generics};\n"
    elif kind == "trait":
        text += f"pub {kind} {name}{generics} {{{body}}}\n"
    else:
        text += f"pub {kind} {name}{generics} {{\n    {body}\n}}\n"
    if "Error" in dropped:
        text += (
            f"impl core::fmt::Display for {name} {{\n"
            f"    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {{\n"
            f"        core::fmt::Debug::fmt(self, f)\n    }}\n}}\n"
            f"impl core::error::Error for {name} {{}}\n"
        )
    return text


PINS = {}

# The pins only the application row needs. Every other row would carry them as
# an unused dependency, and `crates/duet` is the one row that may name a
# framework type (section 1.3).
APP_ONLY_PINS = {"gpui-kit", "tokio", "tokio-util"}

# Every substitution this script applies, by the id section 1.9 gives it. The
# guard refuses to run when the two lists differ, so the count in the document
# and the count here are one number (critic WR-3).
SUBSTITUTIONS = {"S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8"}


def write_size_oracle(scratch, per_crate, bodies, owner, edges, paths, sizes, consts, marker):
    """The `roster-sizes` member, which measures every size this plan states.

    It prints one `<expression> <size> <align>` line per declaration that the
    roster spells out, and one per row of the section 1.9 size block that names
    a whole expression. A row marked `generic` names a container head and has
    no size of its own.
    """
    rows, imports = [], {}
    for name in sorted(bodies):
        _attrs, kind, shape, body, generics = bodies[name]
        if generics or kind == "trait":
            continue
        if kind == "struct" and shape == "braced" and not re.sub(
            r"/\*.*?\*/", "", body, flags=re.S
        ).strip():
            continue
        rows.append(name)
        imports.setdefault(owner[name], set()).add(name)
    for expression in sorted(sizes):
        if "generic" in sizes[expression]:
            continue
        rows.append(expression)
    def qualify(expression):
        return re.sub(
            r"(?<![:\w])([A-Za-z_][A-Za-z0-9_]*)",
            lambda m: paths[m.group(1)][0] if m.group(1) in paths and m.group(1) not in owner
            else m.group(1),
            expression,
        )

    spelled = {row: qualify(row) for row in rows}
    for row in rows:
        for token in re.findall(r"(?<![:\w])([A-Za-z_][A-Za-z0-9_]*)", row):
            if token in owner:
                imports.setdefault(owner[token], set()).add(token)
            elif token in consts and consts[token][0]:
                imports.setdefault(consts[token][0], set()).add(token)
    directory = os.path.join(scratch, "roster-sizes")
    os.makedirs(os.path.join(directory, "src"), exist_ok=True)
    lines = ["//! The size oracle of the roster compile.", "", 
             "use std::process::ExitCode;"]
    for crate in sorted(imports):
        names = sorted(imports[crate])
        if len(names) == 1:
            lines.append(f"use {crate_ident(crate)}::{names[0]};")
        else:
            lines.append(f"use {crate_ident(crate)}::{{{', '.join(names)}}};")
    lines += ["", "/// Every expression this workspace measures.",
              "const ROWS: &[(&str, usize, usize)] = &["]
    for expression in rows:
        form = spelled[expression]
        lines.append(f'    ("{expression}", size_of::<{form}>(), align_of::<{form}>()),')
    lines += ["];", "",
              "/// Print one line per row.",
              "///",
              "/// # Errors",
              "/// It returns the error the output handle reports.",
              "fn report(out: &mut impl std::io::Write) -> std::io::Result<()> {",
              "    for (name, size, align) in ROWS {",
              '        writeln!(out, "{name} {size} {align}")?;',
              "    }",
              "    Ok(())",
              "}", "",
              "/// Write every measured size to the locked output handle.",
              "fn main() -> ExitCode {",
              "    let stdout = std::io::stdout();",
              "    let mut out = stdout.lock();",
              "    if report(&mut out).is_err() {",
              "        return ExitCode::FAILURE;",
              "    }",
              "    ExitCode::SUCCESS",
              "}", ""]
    with open(os.path.join(directory, "src", "main.rs"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines))
    manifest = ["[package]", 'name = "roster-sizes"', 'version = "0.1.0"', 'edition = "2024"',
                'rust-version = "1.98"', 'license = "MIT"',
                'repository = "https://example.invalid/duet-roster"',
                'description = "The size oracle of the Duet roster compile."',
                'documentation = "https://example.invalid/duet-roster"',
                'readme = "../README.md"', 'keywords = ["duet"]',
                'categories = ["development-tools"]', "publish = false", "",
                "[[bin]]", 'name = "roster-sizes"', 'path = "src/main.rs"', "", "[dependencies]"]
    for crate in sorted(imports):
        if crate:
            manifest.append(f'{package_name(crate)} = {{ path = "../{package_name(crate)}" }}')
    for pin, spec in sorted(PINS.items()):
        # The same `-` rule the member manifests use (critic C20-W8).
        features = [token for token in spec[1:] if token != "-"]
        entry = f'{pin} = {{ version = "{spec[0]}"'
        if "-" in spec[1:]:
            entry += ", default-features = false"
        if features:
            entry += ", features = [" + ", ".join(f'"{f}"' for f in features) + "]"
        manifest.append(entry + " }")
    manifest += ["", "[lints]", "workspace = true", ""]
    with open(os.path.join(directory, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(manifest))


def main(argv):
    if len(argv) != 4:
        sys.stdout.write("usage: gen.py <architecture.md> <scratch-dir> <repo-root>\n")
        return 2
    doc_path, scratch, repo = argv[1], argv[2], argv[3]
    with open(doc_path, encoding="utf-8") as handle:
        source = handle.read()

    blocks, block_bad = read_blocks(source)
    for block_id, reason in block_bad:
        sys.stdout.write(
            f"FAIL: the `{block_id}` block of this document: {reason};"
            " the guard is fail-closed (DR7).\n"
        )
    if block_bad:
        return 2

    applied = fenced_map(blocks, "substitutions")
    if set(applied) != SUBSTITUTIONS:
        sys.stdout.write(
            "FAIL: the section 1.9 substitution block and the script disagree; "
            f"the block holds {sorted(applied)} and the script applies "
            f"{sorted(SUBSTITUTIONS)}.\n"
        )
        return 2
    # A rule id, a budget id, and a section number each carry a digit and none
    # of the three is a count, so all three leave the text before the search.
    # Anything that remains is a number only a run may state (critic WR-14).
    digits = []
    for line in blocks["substitutions"]:
        halves = line.split(None, 1)
        text = halves[1] if len(halves) > 1 else ""
        text = re.sub(r"\b[A-Z]{1,2}[0-9]+[a-z]?\b", "", text)
        text = re.sub(r"\bsection [0-9]+\.[0-9]+[a-z]?\b", "", text)
        if re.search(r"[0-9]", text):
            digits.append(line)
    if digits:
        sys.stdout.write(
            "FAIL: a substitution line states a number, and only a run may state one "
            f"(critic WR-14): {digits[0].strip()}\n"
        )
        return 2

    owner, crate_order = ownership_table(blocks)
    edges = edge_list(blocks)
    paths = fenced_map(blocks, "external-paths")
    pins = fenced_map(blocks, "pins")
    consts = constants(blocks)
    drop_names = [token for line in blocks["drop-impls"] for token in line.split()]
    unplaced = [name for name in drop_names if name not in owner]
    if unplaced:
        sys.stdout.write(
            f"FAIL: the Drop block names {unplaced[0]}, which section 1.5 places nowhere.\n"
        )
        return 2

    code_blocks = [join_attributes(strip_docs(block)) for block in rust_blocks(source)]
    bodies, seen = {}, []
    for code in code_blocks:
        for name, attrs, kind, shape, body, generics in items(code):
            if name in bodies or name not in owner:
                continue
            bodies[name] = (attrs, kind, shape, body, generics)
            seen.append(name)
    # An impl block usually lives in the crate that owns its target type. One
    # impl in this plan does not: `impl From<ConfigError> for GatewayError`
    # has a local source type and a foreign target, which Rust's orphan rule
    # allows and which section 1.3 places in `duet-engine`, because
    # `duet-command` carries no edge to `duet-engine` (critic C16-16, C17-4).
    # The `impl-sites` row states the crate with a trailing `in <crate>`, and
    # the block is the one source of that placement.
    impl_crate = {}
    for row in blocks["impl-sites"]:
        tokens = row.split()
        if len(tokens) >= 4 and tokens[-2] == "in":
            impl_crate[(tokens[0], tokens[1].rsplit("::", 1)[-1])] = tokens[-1]
    impls = {}
    all_impls = 0
    mixed_blocks = []
    for code in code_blocks:
        all_impls += len(IMPL.findall(code)) + len(INHERENT.findall(code))
        block_full, block_mixed = full_impls(code)
        mixed_blocks.extend(block_mixed)
        for name, text in block_full:
            if name in owner:
                impls.setdefault(name, []).append(text)

    marker = ""
    per_crate = {}
    for name in seen:
        attrs, kind, shape, body, generics = bodies[name]
        per_crate.setdefault(owner[name], []).append(
            emit_item(name, attrs, kind, shape, body, generics, marker)
        )
        for text in impls.get(name, []):
            trait_name = ""
            head = re.match(r"\s*impl(?:<[^>]*>)?\s+([A-Za-z0-9_:<>, ]+?)\s+for\s", text)
            if head:
                trait_name = head.group(1).strip()
            where = impl_crate.get(
                (name, trait_name.rsplit("::", 1)[-1]), owner[name]
            )
            per_crate.setdefault(where, []).append(text + "\n")
    # S8. A type with a hand-written `Drop` impl cannot implement `Copy`, so
    # `missing_copy_implementations` never fires on it and an expectation
    # there is a build error (critic WR-12). The roster emits the impl so that
    # it carries the same shape the crate will have; the BODY is the roster's
    # and never the chunk's, and `core::hint::black_box` keeps
    # `clippy::empty_drop` quiet without inventing a statement the chunk would
    # copy.
    for name in drop_names:
        per_crate.setdefault(owner[name], []).append(
            f"impl Drop for {name} {{\n"
            f"    fn drop(&mut self) {{\n"
            f"        core::hint::black_box(&*self);\n"
            f"    }}\n}}\n"
        )

    # A constant is declared in its owning crate and used from another, so the
    # generator needs the WHOLE used set before it writes the first crate.
    # Revision 12 kept every constant in the crate that used it, so a one-pass
    # emission was enough; `duet-time::limits` now owns the four shared limits
    # (critic CR-12).
    global_used = set()
    for texts in per_crate.values():
        joined = "".join(texts)
        global_used |= set(re.findall(r"(?<![:\w])([A-Z][A-Z0-9_]{2,})", joined))

    shutil.rmtree(scratch, ignore_errors=True)
    members = []
    for crate in crate_order:
        package = package_name(crate)
        members.append(package)
        directory = os.path.join(scratch, package)
        os.makedirs(os.path.join(directory, "src"), exist_ok=True)
        text = "".join(per_crate.get(crate, []))
        clean = mask_arms(re.sub(r"/\*.*?\*/", " ", text, flags=re.S))
        derived = set()
        for match in re.finditer(r"#\[derive\(([^)]*)\)\]", clean):
            derived |= {n.strip() for n in match.group(1).split(",")}
        outside = set(re.findall(r"(?<![:\w])([A-Z][A-Za-z0-9]*)", re.sub(r"#\[derive\([^)]*\)\]", " ", clean)))
        used = set(re.findall(r"(?<![:\w])([A-Z][A-Za-z0-9]*)", clean))
        used |= set(re.findall(r"(?<![:\w])([A-Z][A-Z0-9_]{2,})", text))
        wanted = []
        for dependency in edges.get(crate, []):
            names = sorted(token for token in used if owner.get(token) == dependency)
            if len(names) == 1:
                wanted.append(f"use {crate_ident(dependency)}::{names[0]};")
            elif names:
                wanted.append(f"use {crate_ident(dependency)}::{{{', '.join(names)}}};")
        for token in sorted(used):
            if token not in paths or owner.get(token) is not None:
                continue
            path = paths[token][0]
            # The WHOLE row after the name is the import, so a row may state
            # an alias: `CoreReceiver  async_channel::Receiver as
            # CoreReceiver` emits `use async_channel::Receiver as
            # CoreReceiver;`. PL3 needs it, because `Receiver` already names
            # `std::sync::mpsc::Receiver` (critic C22I-1, C22I-2).
            if token in outside or not path.startswith(("std::", "core::")):
                wanted.append("use " + " ".join(paths[token]) + ";")
        for token in sorted(used):
            if token in consts and consts[token][0] != crate:
                home = consts[token][0]
                if home and home != crate:
                    wanted.append(f"use {crate_ident(home)}::{token};")
        head = ["//! One crate of the scratch roster workspace.", ""]
        if crate == "crates/duet":
            head = [
                "//! One crate of the scratch roster workspace.",
                "#![expect(",
                "    missing_copy_implementations,",
                '    reason = "the roster compile builds the application row as a library and the \\',
                '              real crate is a binary, where the lint cannot fire (section 15.16)"',
                ")]",
                "#![expect(",
                "    missing_debug_implementations,",
                '    reason = "the five action types come from `gpui_kit::actions!`, which supplies \\',
                '              the derive the roster does not spell out (section 15.16)"',
                ")]",
                "",
            ]
        lines = head + sorted(set(wanted)) + [""]
        for token in sorted(global_used):
            if token in consts and consts[token][0] == crate:
                lines.append(consts[token][1])
        lines.append("")
        with open(os.path.join(directory, "src", "lib.rs"), "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + text)
        manifest = [
            "[package]",
            f'name = "{package}"',
            'version = "0.1.0"',
            'edition = "2024"',
            'rust-version = "1.98"',
            'license = "MIT"',
            'repository = "https://example.invalid/duet-roster"',
            'description = "One crate of the Duet roster compile."',
            'documentation = "https://example.invalid/duet-roster"',
            'readme = "../README.md"',
            'keywords = ["duet"]',
            'categories = ["development-tools"]',
            "publish = false",
            "",
            "[lib]",
            f'name = "{crate_ident(crate)}"',
            'path = "src/lib.rs"',
            "",
            "[dependencies]",
        ]
        for dependency in edges.get(crate, []):
            manifest.append(
                f'{package_name(dependency)} = {{ path = "../{package_name(dependency)}" }}'
            )
        for pin, spec in sorted(pins.items()):
            if crate != "crates/duet" and pin in APP_ONLY_PINS:
                continue
            # A leading `-` in the feature list is `default-features = false`
            # (critic C20-W8). Revision 20's line format could not express it,
            # so the roster resolved five crates with their default feature
            # set while Appendix B.5 required the opposite, and the resolution
            # the compile proved was not the resolution the plan wants.
            features = [token for token in spec[1:] if token != "-"]
            entry = f'{pin} = {{ version = "{spec[0]}"'
            if "-" in spec[1:]:
                entry += ", default-features = false"
            if features:
                entry += ", features = [" + ", ".join(f'"{f}"' for f in features) + "]"
            manifest.append(entry + " }")
        manifest += ["", "[lints]", "workspace = true", ""]
        with open(os.path.join(directory, "Cargo.toml"), "w", encoding="utf-8") as handle:
            handle.write("\n".join(manifest))

    PINS.update(pins)
    sizes = fenced_map(blocks, "recorded-sizes")
    write_size_oracle(scratch, per_crate, bodies, owner, edges, paths, sizes, consts, marker)
    members.append("roster-sizes")

    with open(os.path.join(repo, "Cargo.toml"), encoding="utf-8") as handle:
        repo_manifest = handle.read()
    lints = re.search(r"(\[workspace\.lints\.rust\]\n.*?)\n# -+\n# Profiles", repo_manifest, re.S)
    if not lints:
        sys.stdout.write("FAIL: the repository manifest holds no `[workspace.lints]` table.\n")
        return 2
    table = lints.group(1)
    for lint in ("missing_docs", "missing_docs_in_private_items", "must_use_candidate",
                 "missing_const_for_fn", "new_without_default", "missing_panics_doc",
                 "missing_errors_doc"):
        table = re.sub(r"^" + lint + r" = [^\n]*\n", "", table, flags=re.M)
    rust_relaxed = (
        "# The roster-compile profile, half one. The roster carries declarations\n"
        "# and not documentation.\n"
        'missing_docs = "allow"\n\n'
    )
    clippy_relaxed = (
        "\n# The roster-compile profile, half two. The roster carries declarations\n"
        "# and not documentation.\n"
        'missing_docs_in_private_items = "allow"\n'
        "# The roster carries no function body, so each lint below reads an impl\n"
        "# that the roster does not spell out.\n"
        'must_use_candidate = "allow"\n'
        'missing_const_for_fn = "allow"\n'
        'new_without_default = "allow"\n'
        'missing_panics_doc = "allow"\n'
        'missing_errors_doc = "allow"\n'
    )
    table = table.replace("[workspace.lints.rustdoc]", rust_relaxed + "[workspace.lints.rustdoc]")
    table = table.rstrip("\n") + "\n" + clippy_relaxed
    workspace = [
        "[workspace]",
        'resolver = "3"',
        "members = [" + ", ".join(f'"{m}"' for m in members) + "]",
        "",
        table,
    ]
    with open(os.path.join(scratch, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write("\n".join(workspace))
    os.makedirs(os.path.join(scratch, ".cargo"), exist_ok=True)
    for name, target in (
        (".cargo/config.toml", ".cargo/config.toml"),
        ("clippy.toml", "clippy.toml"),
        ("rust-toolchain.toml", "rust-toolchain.toml"),
        ("Cargo.lock", "Cargo.lock"),
    ):
        source_path = os.path.join(repo, name)
        if os.path.exists(source_path):
            shutil.copyfile(source_path, os.path.join(scratch, target))
    with open(os.path.join(scratch, "README.md"), "w", encoding="utf-8") as handle:
        handle.write("Scratch roster workspace.\n")
    # The denominator floor (critic C-3). Substitution S7 drops a declaration
    # section 1.5 does not place, so the roster's item count is PG4's output
    # and it can never sit below the number of names the section 1.5 table
    # places. A deleted declaration shrank the denominator in silence and the
    # run still exited 0.
    sys.stdout.write(f"ROSTER CRATES:   {len(members)}\n")
    sys.stdout.write(f"ROSTER ITEMS:    {len(seen)}     FLOOR: {len(owner)}\n")
    if len(seen) < len(owner):
        missing = sorted(set(owner) - set(seen))
        sys.stdout.write(
            f"FAIL: the roster holds {len(seen)} items and section 1.5 places"
            f" {len(owner)}; the first name with no declaration is {missing[0]}.\n"
        )
        return 1
    # The impl floor (concern N-3). Both counts come from one parse, so a
    # deleted `impl` shrank both and the run still exited 0. The `impl-sites`
    # block is the second source that makes a deletion a failure.
    # A MIXED impl block is refused (critic N17-4). `full_impls` drops a whole
    # block when any one item is a bodiless signature, so a real body beside a
    # signature leaves the roster and no counter moves. The refusal is stated
    # rather than the drop, so the day an author writes one the run says so.
    sys.stdout.write(f"ROSTER IMPL MIXED: {len(mixed_blocks)}\n")
    if mixed_blocks:
        for name, _text in sorted(mixed_blocks):
            sys.stdout.write(
                f"FAIL: the impl block for {name} carries a body beside a bodiless"
                " signature, so the whole block leaves the roster; write every item"
                " as a signature or write every item with a body.\n"
            )
        return 1
    impl_floor = len(blocks["impl-sites"])
    placed_impls = sum(len(v) for v in impls.values())
    sys.stdout.write(f"ROSTER IMPL BLOCKS: {all_impls}     FLOOR: {impl_floor}\n")
    sys.stdout.write(f"ROSTER IMPLS:    {placed_impls}\n")
    if all_impls != impl_floor:
        # The block lists a site per row, and the parse counts blocks. The
        # difference names how many rows the document no longer carries; it
        # cannot name which row, because a row is a site and not a name that
        # the parse returns. The block's own site states that limit.
        sys.stdout.write(
            f"FAIL: the roster parsed {all_impls} impl blocks and the `impl-sites`"
            f" block lists {impl_floor}; the two are one set and they differ by"
            f" {abs(impl_floor - all_impls)}.\n"
        )
        return 1
    sys.stdout.write(f"ROSTER DROPS:    {len(drop_names)}\n")
    sys.stdout.write(f"ROSTER CONSTS:   {len(consts)}\n")
    sys.stdout.write(f"ROSTER SUBS:     {len(SUBSTITUTIONS)}\n")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
PYTHON_GENERATE

cat > "$SCRATCH/roster_sizes.py" <<'PYTHON_SIZES'
"""The size half of `roster_compile.sh`, which holds guard rule PG25.

Every row of the section 1.9 block `Every size the guard records` that names a
whole expression is measured by the compiler on every run. A row the compiler
contradicts is a failure, so no size in this document can go stale, and every
`#[expect]` reason that states a size rests on a measured fact (critic C-4).
"""

import re
import sys


def recorded(path):
    """The section 1.9 size block, as expression -> (size, align, head)."""
    with open(path, encoding="utf-8") as handle:
        source = handle.read()
    block = re.search(
        r"#### Every size the guard records\n.*?```text\n(.*?)```", source, re.S
    )
    rows = {}
    if not block:
        return rows
    for line in block.group(1).splitlines():
        parts = line.split()
        if len(parts) >= 3:
            rows[parts[0]] = (parts[1], parts[2], "generic" in parts)
    return rows


def measured(path):
    """The oracle output, as expression -> (size, align)."""
    rows = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            parts = line.split()
            if len(parts) == 3:
                rows[parts[0]] = (parts[1], parts[2])
    return rows


def main(argv):
    """Compare every recorded size with the measured one."""
    if len(argv) != 3:
        sys.stdout.write("usage: roster_sizes.py <architecture.md> <sizes.txt>\n")
        return 2
    rows = recorded(argv[1])
    facts = measured(argv[2])
    if not rows or not facts:
        sys.stdout.write("FAIL: a size table is empty; the guard is fail-closed.\n")
        return 2
    checked, bad, heads = 0, [], 0
    for expression in sorted(rows):
        size, align, head = rows[expression]
        if head:
            heads += 1
            continue
        if expression not in facts:
            bad.append((expression, size, align, "no", "measurement"))
            continue
        checked += 1
        if facts[expression] != (size, align):
            bad.append((expression, size, align) + facts[expression])
    sys.stdout.write(f"ROSTER SIZES:    {len(facts)} measured\n")
    sys.stdout.write(f"RECORDED ROWS:   {checked} checked     HEAD ROWS: {heads}")
    sys.stdout.write(f"     SIZE BAD: {len(bad)}\n")
    for expression, size, align, real_size, real_align in bad:
        sys.stdout.write(
            f"  SIZE:       {expression} is recorded {size}/{align}"
            f" and measures {real_size}/{real_align}\n"
        )
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
PYTHON_SIZES

# The generator returns 2 on an input failure and 1 on a finding, such as the
# denominator floor of PG25 (critic C-3). The wrapper keeps both codes.
python3 "$SCRATCH/roster_generate.py" "$DOCUMENT" "$WORKSPACE" "$REPO"
GENERATED=$?
if [ "$GENERATED" != 0 ]; then
    exit "$GENERATED"
fi

BEFORE=$(grep -c '^\[\[package\]\]' "$REPO/Cargo.lock")
if ! (cd "$WORKSPACE" && cargo generate-lockfile --offline >/dev/null 2>&1 || cargo generate-lockfile >/dev/null 2>&1); then
    printf 'FAIL: the scratch workspace does not resolve; the guard is fail-closed.\n'
    exit 2
fi
AFTER=$(grep -c '^\[\[package\]\]' "$WORKSPACE/Cargo.lock")
printf 'ROSTER LOCK:     %s packages in the repository lock, %s after resolution\n' "$BEFORE" "$AFTER"

printf 'ROSTER LINT:     cargo clippy --workspace --all-targets -- -D warnings\n'
if ! (cd "$WORKSPACE" && cargo clippy --workspace --all-targets --message-format short -- -D warnings); then
    printf 'ROSTER CLIPPY:   FAIL\n'
    exit 1
fi
printf 'ROSTER CLIPPY:   clean\n'

if ! (cd "$WORKSPACE" && cargo run --quiet -p roster-sizes) > "$SCRATCH/sizes.txt"; then
    printf 'FAIL: the size oracle did not run; the guard is fail-closed.\n'
    exit 2
fi

python3 "$SCRATCH/roster_sizes.py" "$DOCUMENT" "$SCRATCH/sizes.txt"
exit "$?"
