---
id: M0
line: M
depends_on: []
write_scope:
  - Cargo.toml
  - Cargo.lock
  - rust-toolchain.toml
  - clippy.toml
  - deny.toml
  - .cargo/config.toml
  - .gitignore
  - NOTICE
  - scripts/dod.sh
  - scripts/bootstrap.sh
  - .github/workflows/ci.yml
  - tools/xtask/Cargo.toml
  - tools/xtask/src/main.rs
  - tools/xtask/src/check_conversions.rs
  - tools/xtask/src/check_placement.rs
  - tools/xtask/src/check_roster.rs
  - tools/xtask/src/check_closure.rs
  - tools/xtask/src/check_manifests.rs
  - tools/xtask/src/check_plan_graph.rs
  - tools/xtask/tests/probes.rs
  - crates/duet/Cargo.toml
  - crates/duet/packaging/macos/Info.plist
  - crates/duet/assets/fonts/Bravura.otf
  - crates/duet/assets/fonts/bravura_metadata.json
  - crates/duet/assets/fonts/LICENSE.txt
  - crates/duet-time/Cargo.toml
  - crates/duet-time/src/lib.rs
  - .claude/skills/gpui-kit/SKILL.md
  - .claude/skills/gpui-kit/references/gpui/
parallelism: serial-only: SM1 runs the manifest chunk alone before every line chunk of its phase, and SM4 makes every policy file of this cell an Orchestrator adjudication.
completion: "cargo xtask check-conversions exits 0; cargo xtask check-manifests exits 0; cargo xtask check-plan-graph roadmap/duet-v1 exits 0; bash scripts/dod.sh --plan lists the check-conversions line and the check-manifests line; commit SHA on a branch chunk/m0-manifest-phase-0"
---

# M0: The phase-0 manifest and the repository policy set

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk opens phase 0. It pins the phase-0 dependencies, creates the `duet-time` crate skeleton,
adds the root path entry of that skeleton, and writes every repository file that the plan needs
before the first line chunk runs. **It does exactly four things under SM1**: it pins, it creates the
skeleton with no `[dependencies]` section, it adds one root path entry per skeleton (SM1 rule 4),
and it settles `Cargo.lock`. Every other file of the write scope is a repository file that SM1 rule
3 gives it. It implements architecture section 13.1 (the M0 row and the seven ownership decisions),
section 13.0
rules SM1, SM3, SM4, and SM5, section 1.9 (the guard and probe contract), section 2.3 (the
conversion guard), section 11.4, section 11.5, section 11.6, section 14 rung one, and Appendix B.3,
B.4, and B.5. It ports the fourteen plan tools of `roadmap/duet-v1/tools/` to six `cargo xtask`
subcommands and one probe target, so the guard rules move from a Python prototype to the repository
automation crate.

**The Orchestrator executes this chunk directly.** `scripts/dod.sh`, `scripts/bootstrap.sh`,
`deny.toml`, `.cargo/config.toml`, `.github/workflows/ci.yml`, `clippy.toml`, and
`.claude/skills/gpui-kit/SKILL.md` are policy files under SM4, and CLAUDE.md makes a gate edit an
adjudication. No engineer may edit one.

**The root `Cargo.toml` has two owners and the split is by table** (SM4). This chunk writes
`[workspace.dependencies]` and nothing else in that file. An edit to `[workspace.lints]` is a
defect that escalates, never a resolution.

## Files

| Path | Action |
|---|---|
| `Cargo.toml` | modify (`[workspace.dependencies]` only) |
| `Cargo.lock` | modify |
| `rust-toolchain.toml` | modify |
| `clippy.toml` | modify |
| `deny.toml` | modify |
| `.cargo/config.toml` | modify |
| `.gitignore` | modify |
| `NOTICE` | create |
| `scripts/dod.sh` | modify |
| `scripts/bootstrap.sh` | modify |
| `.github/workflows/ci.yml` | modify (the whole file) |
| `tools/xtask/Cargo.toml` | modify |
| `tools/xtask/src/main.rs` | modify |
| `tools/xtask/src/check_conversions.rs` | create |
| `tools/xtask/src/check_placement.rs` | create |
| `tools/xtask/src/check_roster.rs` | create |
| `tools/xtask/src/check_closure.rs` | create |
| `tools/xtask/src/check_manifests.rs` | create |
| `tools/xtask/src/check_plan_graph.rs` | create |
| `tools/xtask/tests/probes.rs` | create |
| `crates/duet/Cargo.toml` | modify |
| `crates/duet/packaging/macos/Info.plist` | modify |
| `crates/duet/assets/fonts/Bravura.otf` | create |
| `crates/duet/assets/fonts/bravura_metadata.json` | create |
| `crates/duet/assets/fonts/LICENSE.txt` | create |
| `crates/duet-time/Cargo.toml` | create |
| `crates/duet-time/src/lib.rs` | create |
| `.claude/skills/gpui-kit/SKILL.md` | modify |
| `.claude/skills/gpui-kit/references/gpui/` | create (step 25 restores the files this directory holds) |

## Types and signatures

### The `tools/xtask` command surface (section 2.3, section 14)

Section 2.3 names `cargo xtask check-conversions`. Section 14 names
`cargo xtask check-placement <document>`,
`cargo xtask check-roster <document> <scratch> <repo>`, and
`cargo xtask check-closure <document> <review> <block>`. Section 13.1 names
`cargo xtask check-manifests`. `roadmap/duet-v1/tools/plan_graph_check.py` carries the chunk-file
rules of section 13, and it becomes `cargo xtask check-plan-graph <plan-dir> [--write-manifest]`.
The `Command` enum of `tools/xtask/src/main.rs` gains six arms.

```rust
/// Available tasks.
#[derive(Debug, Subcommand)]
enum Command {
    /// Regenerate the Codex and OpenCode agent mirrors from `.claude/agents`.
    SyncAgents {
        /// Report drift and exit 1 instead of writing files.
        #[arg(long)]
        check: bool,
    },
    /// Refuse a cast and a cast suppression outside the one conversion file.
    CheckConversions,
    /// Refuse a member manifest with no `[lints] workspace = true` line and
    /// no non-empty `description`.
    CheckManifests,
    /// Refuse a declared type that the section 1.5 table does not place.
    CheckPlacement {
        /// The architecture document to read.
        document: PathBuf,
    },
    /// Compile the section 15 roster in a scratch workspace outside the
    /// repository.
    CheckRoster {
        /// The architecture document to read.
        document: PathBuf,
        /// A scratch directory outside the repository.
        scratch: PathBuf,
        /// The repository root.
        repo: PathBuf,
    },
    /// Refuse a chunk file whose front-matter breaks a section 13 rule.
    CheckPlanGraph {
        /// The plan directory that holds the chunk files.
        plan_dir: PathBuf,
        /// Write `plan-graph.md` from the front-matter instead of a report
        /// alone.
        #[arg(long)]
        write_manifest: bool,
    },
    /// Refuse a closure block that its own review file does not support.
    CheckClosure {
        /// The architecture document to read.
        document: PathBuf,
        /// The review file the block closes.
        review: PathBuf,
        /// The closure block identifier.
        block: String,
    },
}
```

Each check module exposes one entry point with the same shape. The three exit codes stay distinct:
0 is clean, 1 is findings, and 2 is fail-closed (CG8).

```rust
/// What one guard run produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Outcome {
    /// No finding. The caller returns `ExitCode::SUCCESS`.
    Clean,
    /// At least one finding. The caller returns `ExitCode::from(1)`.
    Findings,
    /// The guard could not decide. The caller returns `ExitCode::from(2)`.
    FailClosed,
}

/// Run the conversion guard over every member `cargo metadata` reports.
///
/// # Errors
/// Returns an error when `cargo metadata` fails, when a member file cannot be
/// read, and when the derived coverage sets do not match the reported member
/// set (CG1, CG1b, CG8).
pub(crate) fn run(root: &Path) -> anyhow::Result<Outcome>;
```

`anyhow` stays confined to `tools/xtask`, which the root manifest already pins.

### The internal path entry (SM1 rule 4)

```toml
duet-time = { path = "crates/duet-time" }
```

A member crate reaches an internal crate with `duet-<crate> = { workspace = true }`, and that entry
resolves only when the root table already carries the path entry. SM4 keeps the root manifest out of
every line chunk's reach, so the chunk that creates the skeleton is the one chunk that can add it.
Revision 23 gave the internal entries to no chunk at all, and five chunk authors each reported the
same hole.

### The `duet-time` crate skeleton (SM1 rule 2)

```toml
[package]
name = "duet-time"
description = "The Duet time kernel: beat and audio positions, the tempo map, and the one conversion module."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
publish.workspace = true

[lints]
workspace = true
```

The skeleton carries **no `[dependencies]` section at all** (SM1 rule 2). Chunk T1 adds the entries
in the same commit as the code that uses them.

```rust
//! The Duet time kernel.
//!
//! It declares the beat domain and the audio domain, the position and span
//! types, the tempo map, and the one module of the workspace that holds an
//! `as` cast. Section 2 of `roadmap/duet-v1/architecture.md` states the
//! design.

#![forbid(unsafe_code)]
```

## Steps

1. **Verify the prototype set and the review set before any other step.** Run
   `ls roadmap/duet-v1/tools/` and `ls roadmap/duet-v1/reviews/`. The tools directory holds
   fourteen files, and each one is a source this chunk ports or a library a port reads.

   | File | What it carries | Where it lands |
   |---|---|---|
   | `conversion_check.py` | CG1 to CG8, the twelve conversion rules (section 2.3) | `tools/xtask/src/check_conversions.rs` |
   | `placement_check.py` | Every PG rule but PG25 (section 1.9) | `tools/xtask/src/check_placement.rs` |
   | `roster_compile.sh` | PG25, the roster compile (section 1.9) | `tools/xtask/src/check_roster.rs` |
   | `closure_check.py` | PG32, the closure rules CL1 to CL5 (section 1.9) | `tools/xtask/src/check_closure.rs` |
   | `plan_graph_check.py` | The seven chunk-file rules of section 13 | `tools/xtask/src/check_plan_graph.rs` |
   | `review_ids.py` | The generated finding-id list of one review, which `closure_check.py` reads | A private module inside `check_closure.rs` |
   | `probe_fragments.py` | The one fragment oracle. It is a LIBRARY and never a check | A private test module inside `tools/xtask/tests/probes.rs` |
   | `probe_run.py` | Every placement probe but PP25, and six PP27 and PP27b shapes per registered block | `tools/xtask/tests/probes.rs` |
   | `probe_conversion.py` | CP1 to CP8, each in its own throwaway workspace | `tools/xtask/tests/probes.rs` |
   | `probe_roster.sh` | PP25 in four shapes, the mixed-impl shape, and six planted gate defects | `tools/xtask/tests/probes.rs` |
   | `probe_roster_text.py` | The recorded-text compare of the roster run | `tools/xtask/tests/probes.rs` |
   | `probe_closure.py` | PP32 in fifteen shapes | `tools/xtask/tests/probes.rs` |
   | `sync_floors.py` | The `rows>=` marker writer, which PG27b reads | `tools/xtask/src/check_placement.rs` |
   | `run_all_gates.py` | The ordered run of every guard and every harness | The `plan-lint` job of `ci.yml` |

   The reviews directory holds twenty-five files. **Nine of them carry a registered closure block**,
   and step 14 names each pair. Report a discrepancy and stop if a file above is absent, because a
   port needs a source and `check-closure` needs its review file.
2. Confirm the rest of the current state. `crates/duet`, `tools/plan-db`, and `tools/xtask` exist.
   `Cargo.toml` pins `serde`, `serde_json`, `thiserror`, `clap`, and `tracing` already, and pins no
   `smallvec`, no `proptest`, and no `md-5`. `deny.toml` allows MPL-2.0, CC0-1.0, Apache-2.0, and
   Apache-2.0 WITH LLVM-exception already, and allows no `Unlicense`. `crates/duet-time` does not
   exist. Report a discrepancy and stop if any one of these is false.
3. Add the three missing pins and the one internal path entry
   `duet-time = { path = "crates/duet-time" }` (SM1 rule 4) to `[workspace.dependencies]` of the
   root `Cargo.toml`. Appendix B.3 gives each version and Appendix B.5 gives the `smallvec`
   feature.

   ```toml
   smallvec = { version = "1.16.1", features = ["serde"] }
   proptest = "1.11.0"
   md-5 = "0.10.6"
   ```

   Confirm that `serde` keeps `features = ["derive"]` and that `clap` keeps the same (Appendix B.5,
   section 13.1 decision 4). Add no other entry, and edit no other table.
4. Create `crates/duet-time/Cargo.toml` and `crates/duet-time/src/lib.rs` with the two blocks the
   section above gives. The `crates/*` member glob picks the crate up, so the `members` array needs
   no edit.
5. Run `cargo build --workspace`. Expected result: the build succeeds and `Cargo.lock` gains the
   `duet-time` package entry. Commit `Cargo.lock` with the manifests (SM5 rule 3).
6. Add the `md-5` entry to `tools/xtask/Cargo.toml`, in the same commit as the
   `tools/xtask/src/check_closure.rs` code that uses it (SM1, critic C21-W5).

   ```toml
   md-5 = { workspace = true }
   ```

7. Port `roadmap/duet-v1/tools/conversion_check.py` to `tools/xtask/src/check_conversions.rs`. It
   holds the twelve rules CG1, CG1b, CG2, CG2b, CG3, CG3b, CG4, CG4b, CG5, CG6, CG7, and CG8. It
   takes no path argument, derives its member set from the `members` globs of the root manifest,
   derives its file set from each covered member's own `src/` directory, and prunes only the
   workspace root plus `target` and each member root plus `target` (section 2.3).
8. Port `roadmap/duet-v1/tools/placement_check.py` to `tools/xtask/src/check_placement.rs`, and
   `roadmap/duet-v1/tools/closure_check.py` to `tools/xtask/src/check_closure.rs`, and
   `roadmap/duet-v1/tools/roster_compile.sh` to `tools/xtask/src/check_roster.rs`. The port gives
   the placement rules and the roster rules **one Rust module and one block register**, which
   removes the duplicate `DATA_BLOCKS` table that section 1.9 records as a known defect. Port
   `roadmap/duet-v1/tools/review_ids.py` as a private module inside `check_closure.rs`: it generates
   the finding-id list from the review file, and the closure rules read that list rather than a
   typed one. Port `roadmap/duet-v1/tools/sync_floors.py` as a private module inside
   `check_placement.rs`, which is the one register of the `rows>=` markers.
9. Port `roadmap/duet-v1/tools/plan_graph_check.py` to `tools/xtask/src/check_plan_graph.rs`. It
   reads every `*.md` file of the plan directory whose front-matter carries `id`, `line`,
   `depends_on`, `write_scope`, `parallelism`, and `completion`, and it holds seven rules, each one
   fail-closed: every id is unique and every `depends_on` id exists; the graph is acyclic; the phase
   of each chunk is later than the phase of every dependency, with the stated M0 before T1 case; two
   chunks of one phase share no write-scope path, with `Cargo.lock` as the one exception (SM5 rule
   4); two chunks of one line share no phase (SM6); every chunk of section 13.3 has a file and every
   file has a row; and every serial link of section 13.4 appears as a `depends_on` edge (SM8).
   `--write-manifest` writes `plan-graph.md` from the front-matter. It exits 0 on success, 1 on a
   finding, and 2 on a read failure, which is the same three-code contract as every other guard.
10. Write `tools/xtask/src/check_manifests.rs`. It reads every member manifest that
    `cargo metadata --no-deps` reports and asserts two conditions: the manifest holds
    `[lints] workspace = true`, and the manifest holds a non-empty `description`.
    `clippy::cargo_common_metadata` cannot fire under `publish = false`, so no lint covers either
    half today (critic C20-W7).
11. Add the six `mod` lines and the six `Command` arms to `tools/xtask/src/main.rs`, and map each
    `Outcome` to its exit code.
12. Port every probe of the section 1.9 table to `tools/xtask/tests/probes.rs`. Each test plants its
    defect in an inline fixture, never in a file under `roadmap/`. Each conversion probe builds a
    throwaway cargo workspace in a per-test temporary directory and changes into it. No probe reads
    this repository. Run `cargo nextest run -p xtask --test probes --no-tests=fail`. Expected result:
    every probe passes, which means that every planted defect turns its rule red and the baseline run
    is green.
13. Edit `scripts/dod.sh`. Add a `check-conversions` line and a `check-manifests` line to the
    `GATES` array and to the run sequence. Replace `typos || printf ...` with a plain `typos` call,
    so that a finding fails the gate (critic C20-W6). Keep the `shellcheck` step and every other
    gate.
14. Edit `scripts/bootstrap.sh`. Install `libpipewire-0.3-dev`, `libasound2-dev`, and `pkg-config`
    on Linux (section 11.5). Pin `cargo-nextest` at 0.9.145, because the gate depends on
    `--no-tests=fail` (section 14). Install `shellcheck` (critic N21-18).
15. Edit `.github/workflows/ci.yml`, the whole file (section 13.1, critic C16-17, C17-5).
    1. Set the `dod` job matrix to `os: [macos-26, ubuntu-26.04]`.
    2. Add `libpipewire-0.3-dev` to the Linux package list, beside the `libasound2-dev` and
       `pkg-config` entries the file already carries.
    3. Install `shellcheck` in both jobs, and install `cargo-nextest@0.9.145` rather than the newest
       release.
    4. Add a `changes` job that runs
       `git diff --name-only "origin/${{ github.base_ref }}"...HEAD` and sets an output named
       `roadmap`, which is `true` when one path opens with `roadmap/`. **Chunk M93 renames that
       output to `plan_inputs` and widens the filter**; the note under step 5 gives the current
       name and the full pattern.
    5. Add a `plan-lint` job on `ubuntu-26.04` that declares `needs: changes` and
       `if: needs.changes.outputs.roadmap == 'true'`. It runs these commands, in this order.

       **Chunk M93 renames that output to `plan_inputs` and widens the filter**, so a party who
       re-derives this job from this brief reads the M93 body first. M93 sets the output name at
       every site, and its filter matches `^roadmap/`, `^tools/xtask/`, `^Cargo\.toml$`,
       `^clippy\.toml$`, `^rust-toolchain\.toml$` and `^\.cargo/config\.toml$`, because rule PG25
       reads the last four. This brief keeps the name M0 itself wrote, and this note is the pointer
       to the current one.

       ```
       cargo xtask check-placement roadmap/duet-v1/architecture.md
       cargo xtask check-roster roadmap/duet-v1/architecture.md "$RUNNER_TEMP/roster" .
       cargo xtask check-plan-graph roadmap/duet-v1
       cargo nextest run -p xtask --test probes --no-tests=fail
       ```

       **SUPERSEDED IN PART by escalation M0-2, which chunk M0 opened and the Architect resolved on
       2026-09-22.** This step first ordered nine `cargo xtask check-closure` lines into the job,
       one per registered Appendix C block. A hosted runner carries no plan store under
       `~/.claude/plan-dbs/`, so every one of the nine would have failed. Chunk M0 measured that
       state and removed the nine lines, and architecture section 1.5 rule CL1c and section 14 now
       state that `check-closure` is a review-time command that runs in no job. **The job runs no
       `check-closure` line.** Chunk M90 amends the job again: it adds the two `check-plan-graph`
       manifest lines and it leaves the roster line with no `--generate-only` flag.
    6. Add no `check-placement` line, no `check-roster` line, no `check-plan-graph` line, and no
       `check-closure` line to `scripts/dod.sh`.

       **SUPERSEDED IN PART by escalation T1-5, which chunk T1 opened and the Architect resolved on
       2026-09-22.** The reason this step gave is still the reason `check-roster` and
       `check-closure` stay out of the gate: a roster run costs a multi-gigabyte compile, and a
       closure run reads a store no runner holds. It was wrong for the three CHEAP plan guards.
       Nine T1 commits passed the local hook and the `plan-lint` job then refused on rule VR1, which
       cost one push and one round trip. **Chunk M90 adds a PATH-CONDITIONAL plan step to
       `scripts/dod.sh`** that runs `check-placement` and the two `check-plan-graph` forms, and only
       when the change under test names a path that opens `roadmap/`. Architecture section 14 rung
       one states the condition and the denominator.
16. Edit `clippy.toml`. Add the ten proper nouns of section 13.1 to `doc-valid-idents`:
    `PipeWire`, `CoreAudio`, `CoreMIDI`, `MusicXML`, `SMuFL`, `Bravura`, `Wayland`, `XWayland`,
    `APFS`, and `RF64`. Keep the `".."` entry, which keeps clippy's own default list.
17. Edit `deny.toml`. Add `"Unlicense"` to the `[licenses] allow` array. Appendix B.4 states that it
    is the one licence edit this plan requires, because `midly` carries it and chunk X3 needs
    `midly`. Run `cargo deny check`. Expected result: it passes. Add an advisory ignore only when
    this run reports a RUSTSEC identifier over the phase-0 tree, and record the identifier and the
    reason in the ignore list beside the existing entries.
18. Edit `.cargo/config.toml`. Add the macOS deployment target of section 11.4.

    ```toml
    [env]
    MACOSX_DEPLOYMENT_TARGET = "26.0"
    ```

19. Edit `crates/duet/packaging/macos/Info.plist`. Add the `LSMinimumSystemVersion` key with the
    value `26.0` (section 11.4).
20. Edit `rust-toolchain.toml`. Read the `rust-version` field of every crate this plan pins, and set
    `channel` to the highest of those values, or keep `1.98.1` when no pin asks for more. Record the
    resolved value and the crate that set it. **`[workspace.package] rust-version` and the
    `clippy.toml` `msrv` line must agree with the channel.** `clippy.toml` is in this write scope, so
    edit `msrv` here. SM1 confines the root-manifest edit to `[workspace.dependencies]`, so a
    `rust-version` bump is a discrepancy this chunk reports to the Architect instead of applying.
21. Edit `.gitignore`. Add a `__pycache__/` rule, because the plan tools are Python and no chunk
    owned that file (critic N19-7). The tree already holds
    `roadmap/duet-v1/tools/__pycache__/`, which a probe run produced, so the rule has a subject the
    moment it lands. Every harness of section 1.9 also sets `PYTHONDONTWRITEBYTECODE=1`, so a run
    writes no new bytecode into the tree it verifies (critic C20-N7).
22. Create `NOTICE`. It holds the Apache NOTICE text for the Apache-2.0 crates the workspace ships,
    and the SIL Open Font License 1.1 text for the Bravura font (Appendix B.4).
23. Add the Bravura font under `crates/duet/assets/fonts/`. Take `Bravura.otf` and
    `bravura_metadata.json` from the Steinberg Bravura release at
    `https://github.com/steinbergmedia/bravura`, and write the release's `OFL.txt` to
    `crates/duet/assets/fonts/LICENSE.txt`. A font file is data and not a crate, so it enters no
    `deny.toml` list.
24. Read `crates/duet/Cargo.toml` and confirm that it holds `[lints] workspace = true` and a
    non-empty `description`, which `cargo xtask check-manifests` now asserts. Section 13.1 names the
    path in this write scope and states no other edit for it. Make no other change; report a
    discrepancy to the Architect if the file needs one.
25. Repair `.claude/skills/gpui-kit/SKILL.md`. The file holds 23 links to 22 files under
    `references/gpui/`, and the directory does not exist. For each target, try
    `https://gpui-kit.com/llms.txt` first and restore the file under
    `.claude/skills/gpui-kit/references/gpui/`. Remove the link syntax of every target that source
    does not serve, and keep the topic name as plain text. Delete a bullet whose every link is
    removed. Run `grep -c "references/gpui/" .claude/skills/gpui-kit/SKILL.md`. Expected result: the
    count equals the number of files that now exist.
26. Run `cargo xtask check-conversions`. Expected result: exit 0. The current
    `crates/duet/src/main.rs` opens `use gpui_kit::{` on one line and carries `AppContext as _` on
    the next, so CG4 must join a `use` item across lines before CG3 reads the text (critic S10).
27. Run `cargo xtask check-manifests`. Expected result: exit 0 over `duet`, `duet-time`, `plan-db`,
    and `xtask`.
28. Run `cargo xtask check-plan-graph roadmap/duet-v1`. Expected result: exit 0 over every chunk
    file of the plan. The run proves that every `depends_on` edge is forward, that no two chunks of
    one phase share a write-scope path but `Cargo.lock`, and that every serial link of section 13.4
    is an edge (SM8).
29. Run `bash scripts/dod.sh --plan`. Expected result: the printed gate list holds the
    `check-conversions` line and the `check-manifests` line.
30. `git add` the write scope and `git commit`. The native hook runs `scripts/dod.sh`.

## Tests

`tools/xtask/tests/probes.rs` is the one test target this chunk writes. It is an integration target
and wraps its tests in a `#[cfg(test)] mod tests` block
(`clippy::tests_outside_test_module` is denied). Every assert carries a message. Section 14 names
`probes` in the selected-test table and names M0 as its writer, so SM7 holds. Author guidance: the
`test-author` skill.

| Test group | What each test asserts | Where it lives |
|---|---|---|
| `conversion_*` | One test per rule CG1, CG1b, CG2, CG2b, CG3, CG3b, CG4, CG4b, CG5, CG6, CG7, and CG8. Each one builds a throwaway workspace, plants the section 1.9 defect, runs the guard, and asserts the exit code and the printed line the probe row records | `tools/xtask/tests/probes.rs` |
| `placement_*` | One test per PG rule of the section 1.9 probe table, each with the planted defect and the recorded result that table states | `tools/xtask/tests/probes.rs` |
| `roster_*` | PP25 in all four shapes, the mixed-impl shape, and the six planted gate defects | `tools/xtask/tests/probes.rs` |
| `closure_*` | PP32 in all fifteen shapes, over a throwaway review and a throwaway block | `tools/xtask/tests/probes.rs` |
| `manifests_*` | A member with no `[lints] workspace = true` line gives exit 1 with the member path; a member with an empty `description` gives exit 1 with the member path; a workspace `cargo metadata` refuses gives exit 2 | `tools/xtask/tests/probes.rs` |
| `plan_graph_*` | One test per rule of `plan_graph_check.py`: an unknown `depends_on` id, a cycle, a backward link, a shared write-scope path inside one phase, two chunks of one line inside one phase, a chunk of section 13.3 with no file, and a section 13.4 link that no `depends_on` carries. Each one gives exit 1 with the named chunk id; a plan directory that does not open gives exit 2 | `tools/xtask/tests/probes.rs` |

A probe is correct when the baseline run is green and the planted run is red (DR5). No probe reads
this repository, and no probe reads a file under `roadmap/`.

## Verification

```
cargo xtask check-conversions
cargo xtask check-manifests
cargo xtask check-plan-graph roadmap/duet-v1
cargo nextest run -p xtask --test probes --no-tests=fail
cargo deny check
bash scripts/dod.sh --plan
```

Expected output: the first three commands exit 0 and print no finding. The fourth command reports
every probe as passed and reports no filter miss. The fifth command reports no advisory and no
licence failure. The sixth command prints a gate list that holds the `check-conversions` line and
the `check-manifests` line, and that holds no `check-placement`, `check-roster`, `check-plan-graph`,
or `check-closure` line, because those four run in the `plan-lint` job alone (section 14).

Then commit on a branch named `chunk/m0-manifest-phase-0`. The native git hook runs
`scripts/dod.sh`, and the commit lands only when every gate passes.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
