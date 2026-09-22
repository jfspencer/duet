#!/usr/bin/env python3
"""Check the chunk files of a roadmap plan and derive the plan-graph manifest.

Usage: plan_graph_check.py <plan-dir> [--write-manifest]

Reads every ``*.md`` file under the plan directory whose first line is ``---``
and whose front-matter carries ``id``, ``line``, ``depends_on``, ``write_scope``,
``parallelism``, and ``completion``. A file without that header is not a chunk.

Checks, each fail-closed:
1. Every id is unique and every ``depends_on`` id exists.
2. The dependency graph is acyclic.
3. The phase of each chunk (from section 13.3 of architecture.md) is later
   than the phase of every dependency; the one same-phase link allowed is from
   the phase's opening manifest chunk (section 13.3 orders it first).
4. Two chunks of one phase never share a write-scope path (prefix overlap),
   except ``Cargo.lock``, and except a crate skeleton (``Cargo.toml``,
   ``src/lib.rs``) shared between the phase's manifest chunk and a chunk that
   depends on it (SM1 and SM2: the manifest chunk creates it, the line chunk fills it).
5. Two chunks of one line never share a phase.
6. Every chunk of section 13.3 has a file, and every file has a row in 13.3.
7. Every serial link of section 13.4 appears as a ``depends_on`` edge.

With ``--write-manifest`` it writes ``plan-graph.md`` from the front-matter.
Exit 0 on success, 1 on a finding, 2 on a read failure.
"""

from __future__ import annotations

import re
import sys
from pathlib import Path


def fail(msg: str, code: int = 2) -> None:
    """Print a fail-closed line and exit."""
    print(f"FAIL: {msg}")
    sys.exit(code)


def parse_front_matter(text: str) -> dict | None:
    """Return the front-matter mapping, or None when the file is not a chunk."""
    lines = text.splitlines()
    if not lines or lines[0].strip() != "---":
        return None
    try:
        end = lines.index("---", 1)
    except ValueError:
        return None
    fm: dict = {}
    key = None
    for raw in lines[1:end]:
        line = raw.split("#", 1)[0].rstrip() if not raw.lstrip().startswith("-") else raw.rstrip()
        if not line.strip():
            continue
        if line.startswith("  - ") and key:
            fm.setdefault(key, []).append(line[4:].strip().split("#", 1)[0].strip())
            continue
        m = re.match(r"^([a-z_]+):\s*(.*)$", line)
        if not m:
            continue
        key, val = m.group(1), m.group(2).strip()
        if val.startswith("["):
            fm[key] = [v.strip() for v in val.strip("[]").split(",") if v.strip()]
        elif val == "":
            fm[key] = []
        else:
            fm[key] = val.strip('"')
    required = ("id", "line", "depends_on", "write_scope", "parallelism", "completion")
    if not all(k in fm for k in required):
        return None
    return fm


def read_phases(arch: str) -> dict[str, int]:
    """Read the section 13.3 phase table: chunk id to phase."""
    sec = arch.split("### 13.3", 1)
    if len(sec) < 2:
        fail("section 13.3 not found in architecture.md")
    body = sec[1].split("### 13.4", 1)[0]
    phases: dict[str, int] = {}
    for line in body.splitlines():
        m = re.match(r"^\|\s*(\d+)\s*\|\s*([^|]*)\|[^|]*\|\s*([^|]*)\|", line)
        if not m:
            continue
        phase = int(m.group(1))
        for cell in (m.group(2), m.group(3)):
            for cid in re.findall(r"\b([A-Z]\d+)\b", cell):
                phases[cid] = phase
    if not phases:
        fail("section 13.3 phase table parsed to zero rows")
    return phases


def read_links(arch: str) -> set[tuple[str, str]]:
    """Read section 13.4 links of the form 'X before Y' as (X, Y) edges."""
    sec = arch.split("### 13.4", 1)
    if len(sec) < 2:
        fail("section 13.4 not found in architecture.md")
    body = sec[1].split("\n## ", 1)[0]
    links: set[tuple[str, str]] = set()
    for line in body.splitlines():
        if not line.startswith("|"):
            continue
        cell = line.split("|")[1]
        m = re.match(r"\s*([A-Z]\d+(?:\s*,\s*[A-Z]\d+)*)\s+before\s+([A-Z]\d+(?:(?:\s*,\s*|\s+and\s+)[A-Z]\d+)*)", cell)
        if not m:
            continue
        srcs = re.findall(r"[A-Z]\d+", m.group(1))
        dsts = re.findall(r"[A-Z]\d+", m.group(2))
        for s in srcs:
            for d in dsts:
                links.add((s, d))
    return links


def overlap(a: str, b: str) -> bool:
    """True when one write-scope path is a prefix of the other."""
    a = a.rstrip("/")
    b = b.rstrip("/")
    return a == b or a.startswith(b + "/") or b.startswith(a + "/")


def main() -> None:
    """Run every check and optionally write the manifest."""
    if len(sys.argv) < 2:
        fail("usage: plan_graph_check.py <plan-dir> [--write-manifest]", 2)
    plan = Path(sys.argv[1])
    write = "--write-manifest" in sys.argv
    arch_path = plan / "architecture.md"
    if not arch_path.is_file():
        fail(f"cannot open {arch_path}")
    arch = arch_path.read_text(encoding="utf-8")
    phases = read_phases(arch)
    links = read_links(arch)

    chunks: dict[str, dict] = {}
    files: dict[str, Path] = {}
    for path in sorted(plan.glob("*.md")):
        fm = parse_front_matter(path.read_text(encoding="utf-8"))
        if fm is None:
            continue
        cid = fm["id"]
        if cid in chunks:
            fail(f"duplicate chunk id {cid} in {path.name} and {files[cid].name}", 1)
        chunks[cid] = fm
        files[cid] = path
    if not chunks:
        fail("no chunk file found", 1)

    bad = 0

    def finding(msg: str) -> None:
        nonlocal bad
        bad += 1
        print(f"FINDING: {msg}")

    for cid, fm in chunks.items():
        for dep in fm["depends_on"]:
            if dep not in chunks:
                finding(f"{cid} depends on {dep}, which has no chunk file")

    state: dict[str, int] = {}

    def visit(cid: str, stack: list[str]) -> None:
        if state.get(cid) == 1:
            finding("cycle: " + " -> ".join(stack + [cid]))
            return
        if state.get(cid) == 2:
            return
        state[cid] = 1
        for dep in chunks.get(cid, {}).get("depends_on", []):
            if dep in chunks:
                visit(dep, stack + [cid])
        state[cid] = 2

    for cid in chunks:
        visit(cid, [])

    for cid, fm in chunks.items():
        if cid not in phases:
            finding(f"{cid} has a file and no row in section 13.3")
    for cid in phases:
        if cid not in chunks:
            finding(f"{cid} has a row in section 13.3 and no chunk file")

    for cid, fm in chunks.items():
        for dep in fm["depends_on"]:
            if dep in phases and cid in phases:
                if phases[dep] > phases[cid]:
                    finding(f"{cid} (phase {phases[cid]}) depends on {dep} (phase {phases[dep]}), a backward link")
                if phases[dep] == phases[cid] and not dep.startswith("M"):
                    finding(f"{cid} and {dep} share phase {phases[cid]} with a link between them")

    by_phase: dict[int, list[str]] = {}
    for cid in chunks:
        if cid in phases:
            by_phase.setdefault(phases[cid], []).append(cid)
    for phase, ids in by_phase.items():
        for i, a in enumerate(ids):
            for b in ids[i + 1 :]:
                manifest_pair = (a.startswith("M") and a in chunks[b]["depends_on"]) or (
                    b.startswith("M") and b in chunks[a]["depends_on"]
                )
                for pa in chunks[a]["write_scope"]:
                    for pb in chunks[b]["write_scope"]:
                        if pa.endswith("Cargo.lock") and pb.endswith("Cargo.lock"):
                            continue
                        if manifest_pair and (pa.endswith("Cargo.toml") or pa.endswith("src/lib.rs")):
                            continue
                        if overlap(pa, pb):
                            finding(f"phase {phase}: {a} and {b} both write {pa} / {pb}")
        lines_seen: dict[str, str] = {}
        for cid in ids:
            ln = chunks[cid]["line"]
            if ln in ("M", "trunk"):
                continue
            if ln in lines_seen:
                finding(f"phase {phase}: line {ln} has two chunks, {lines_seen[ln]} and {cid}")
            lines_seen[ln] = cid

    for s, d in sorted(links):
        if d in chunks and s not in chunks[d]["depends_on"]:
            finding(f"section 13.4 link {s} before {d} is not a depends_on edge of {d}")

    print(f"CHUNKS: {len(chunks)}   PHASES: {len(by_phase)}   LINKS 13.4: {len(links)}   FINDINGS: {bad}")

    if write and bad == 0:
        out = [
            "# Plan graph: roadmap/duet-v1",
            "",
            "Derived from the chunk front-matter by `tools/plan_graph_check.py`. Edit the chunk files, then regenerate; never edit this file by hand.",
            "",
            "## Objective",
            "",
            "Duet v1: a vocal-first composition, record, mix, and master application on GPUI Kit, AI-first with a git-like history, on macOS 26 and Ubuntu 26.04. The measurable completion outcome is architecture.md section 14 (three rungs: `scripts/dod.sh` green on both platforms, the named test commands, and the human product review gate).",
            "",
            "## Phases",
            "",
            "| Phase | Chunks | Width |",
            "|---|---|---|",
        ]
        for phase in sorted(by_phase):
            ids = sorted(by_phase[phase])
            out.append(f"| {phase} | {', '.join(ids)} | {len(ids)} |")
        out += ["", "## Chunks", "", "| Id | Line | Phase | File | Depends on | Write scope |", "|---|---|---|---|---|---|"]
        for cid in sorted(chunks, key=lambda c: (phases.get(c, 99), c)):
            fm = chunks[cid]
            out.append(
                f"| {cid} | {fm['line']} | {phases.get(cid, '?')} | `{files[cid].name}` | {', '.join(fm['depends_on']) or 'none'} | {'; '.join(fm['write_scope'])} |"
            )
        out += ["", "## Parallelism", "", f"{len(by_phase)} phases; widest phase holds {max(len(v) for v in by_phase.values())} chunks. Every line is a serial chain (SM6); every cross-line link is a `depends_on` edge; `Cargo.lock` is the one shared write path and follows the regenerate rule of section 13.0.", ""]
        (plan / "plan-graph.md").write_text("\n".join(out), encoding="utf-8")
        print("MANIFEST: plan-graph.md written")

    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
