---
id: M94
line: M
depends_on: [M90, M92, T2, D1, M2, M93]
write_scope:
  - tools/bc_repo_guard/xtask/lang_rust/src/main.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_ddd.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_ddd/
  - tools/bc_repo_guard/xtask/lang_rust/Cargo.toml
  - tools/bc_repo_guard/xtask/lang_rust/tests/ddd.rs
  - tools/bc_repo_guard/xtask/lang_rust/tests/fixtures/ddd/
  - .ddd/context-map.toml
  - scripts/dod.sh
  - scripts/ddd-fixtures.sh
  - CLAUDE.md
  - Cargo.lock
  - crates/bc_audio/duet-dsp/lang_rust/src/align.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/delay.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/reverb.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/fft.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/filter.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/gain.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/lib.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/meter.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/meter_law.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/builder.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/format.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/reader.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/pool.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/slot.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/source.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/voice.rs
  - crates/bc_document/duet-score/lang_rust/src/apply.rs
  - crates/bc_document/duet-score/lang_rust/src/canonical.rs
  - crates/bc_document/duet-score/lang_rust/src/command.rs
  - crates/bc_document/duet-score/lang_rust/src/error.rs
  - crates/bc_document/duet-score/lang_rust/src/event.rs
  - crates/bc_document/duet-score/lang_rust/src/ids.rs
  - crates/bc_document/duet-score/lang_rust/src/lib.rs
  - crates/bc_document/duet-score/lang_rust/src/model.rs
  - crates/bc_document/duet-score/lang_rust/tests/canonical_determinism.rs
  - crates/bc_time/duet-time/lang_rust/src/bbt.rs
  - crates/bc_time/duet-time/lang_rust/src/convert.rs
  - crates/bc_time/duet-time/lang_rust/src/error.rs
  - crates/bc_time/duet-time/lang_rust/src/finite.rs
  - crates/bc_time/duet-time/lang_rust/src/lib.rs
  - crates/bc_time/duet-time/lang_rust/src/position.rs
  - crates/bc_time/duet-time/lang_rust/src/span.rs
  - crates/bc_time/duet-time/lang_rust/src/tempo.rs
  - crates/bc_time/duet-time/lang_rust/src/tuplet.rs
  - crates/bc_time/duet-time/lang_rust/src/units.rs
  - crates/bc_time/duet-time/lang_rust/tests/kernel.rs
  - crates/bc_time/duet-time/lang_rust/tests/proptest_large.rs
  - crates/bc_app/duet/lang_rust/src/main.rs
  - crates/bc_app/duet/lang_rust/src/app.rs
  - tools/bc_plan_store/plan-db/lang_rust/src/main.rs
  - tools/bc_plan_store/plan-db/lang_rust/tests/roundtrip.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_closure.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_conversions.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_manifests.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_placement.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_plan_graph.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/check_roster.rs
  - tools/bc_repo_guard/xtask/lang_rust/src/sync_agents.rs
  - tools/bc_repo_guard/xtask/lang_rust/tests/probes.rs
  - crates/bc_document/duet-session/lang_rust/src/lib.rs
  - crates/bc_notation/duet-engrave/lang_rust/src/lib.rs
  - crates/bc_audio/duet-analysis/lang_rust/src/lib.rs
parallelism: serial-only: SM4 makes `scripts/dod.sh` and `CLAUDE.md` policy files, so the Orchestrator executes this chunk, and it must land before the phase-2 line chunks so that each of them writes front matter under the gate.
completion: "cargo xtask check-ddd exits 0 on the repository and prints non-zero PATHS, MEMBERS and BLOCKS counters; cargo nextest run -p xtask --test ddd --no-tests=fail passes; scripts/dod.sh runs the ddd step; cargo xtask check-plan-graph roadmap/duet-v1 --check-manifest exits 0; commit SHA on a branch chunk/m94-ddd-guard"
---

# M94: `cargo xtask check-ddd` makes the DDD path grammar and front matter a gate

Verify the current state of every file in the write scope; report a discrepancy and stop, instead of
proceeding.

This is a REPAIR chunk under rule SM9 of architecture section 13.0. It implements ADR 0011,
architecture section 1.2 ("The bounded context map"), and the `check-ddd` line of section 14 rung
one. The move of every crate to `crates/bc_<context>/<crate>/lang_rust/` already landed as a plan
act (section 13.1). Today nothing mechanical holds that layout except the `members` globs and PG40,
and nothing reads front matter at all. This chunk adds the guard.

**The normative sources live in the ultravisor repository, at commit `1391a1c4f`**, and this chunk
ports a SUBSET of them to Rust, because the gate runs no Bun:

- `packages/ddd-path/docs/grammar/README.md`: the path grammar. Sections 1, 2, 4, and 6 are the part
  this chunk ports.
- `packages/ddd-meta/docs/frontmatter/README.md` and `field-registry.md`: the front-matter format.
  The `block` carrier (`.rs`) and the `hash` carrier (`.toml`) are the part this chunk ports.
- `packages/ddd-path/src/parse.ts`: the `ANCHOR_BASENAMES` and `OUTSIDE_SEGMENTS` tables, which the
  port copies verbatim.

**A port with no parity check is a second copy of one truth, which the grammar forbids.** Two
committed fixtures bind the port to the upstream code, and a script regenerates them from an
ultravisor checkout (Steps 2 and 3).

**The Orchestrator executes this chunk.** `scripts/dod.sh` and `CLAUDE.md` are policy files under
SM4. The Duet Engineer may write the Rust half under a dispatch, with this file as the brief.

## Files

| Path | Action |
|---|---|
| `.ddd/context-map.toml` | create: the contexts, their crates, their subdomain, and the relationships |
| `tools/bc_repo_guard/xtask/lang_rust/src/check_ddd.rs` | create: the command, its counters, and the three checks |
| `tools/bc_repo_guard/xtask/lang_rust/src/check_ddd/` | create: `grammar.rs` (classify), `shell.rs` (the `.ddd/grammar.toml` decode), `front_matter.rs` (read, grade, write), `context_map.rs` |
| `tools/bc_repo_guard/xtask/lang_rust/src/main.rs` | modify: register `check-ddd` |
| `tools/bc_repo_guard/xtask/lang_rust/Cargo.toml` | modify only if a `{ workspace = true }` entry is missing (`toml` and `serde_json` are pinned already) |
| `tools/bc_repo_guard/xtask/lang_rust/tests/ddd.rs` | create: the parity tests and the red probes |
| `tools/bc_repo_guard/xtask/lang_rust/tests/fixtures/ddd/` | create: `paths.tsv`, `front-matter.tsv`, and their inputs |
| `scripts/ddd-fixtures.sh` | create: regenerates both fixtures from an ultravisor checkout given as its one argument |
| `scripts/dod.sh` | modify: the `ddd` step |
| `CLAUDE.md` | modify: the comment rule exempts the `---uv` block; the "Path grammar" rule names this guard |
| every other `.rs` path in the write scope | modify: add one front-matter block and nothing else |

## Types and signatures

```rust
/// One path's class, as the upstream `classify` answers it (grammar README section 6).
pub(crate) enum PathClass {
    Graded { context: Option<String>, language: Language, shell_role: ShellRole },
    Test { context: Option<String>, language: Option<Language>, kind: TestKind },
    Anchor { kind: AnchorKind },
    Outside { kind: OutsideKind },
    Ungraded { reason: UngradedReason, context: Option<String> },
    Malformed { reason: MalformedReason, segment: String },
}

pub(crate) fn classify(repo_relative_path: &str, grammar: &GrammarConfig) -> PathClass;
pub(crate) fn read_front_matter(path: &str, text: &str) -> Result<FrontMatterOutcome, FrontMatterError>;
pub(crate) fn upsert_front_matter(text: &str, carrier: Carrier, value: &FrontMatter) -> Result<String, FrontMatterWriteError>;
```

Every enum carries the upstream literal set in full, including the members this workspace never
produces (`Language` holds all six), because the fixtures compare the literal strings. Each enum
names its upstream set in its `///` doc.

## Steps

1. Read `.ddd/grammar.toml`, `Cargo.toml`, `scripts/dod.sh`, and the context map in
   `roadmap/duet-v1/architecture.md` section 1.2. Confirm that every workspace member sits at
   `(crates|tools)/bc_<context>/<package>/lang_rust/`. Report a discrepancy and stop.

2. **Write `scripts/ddd-fixtures.sh` and generate `paths.tsv`.** The script takes the path of an
   ultravisor checkout and the pinned commit, which it reads from the header of the committed
   fixture. It checks the pin out into a temporary worktree
   (`git -C <checkout> worktree add --detach <tmp> <pin>`), refuses when the pin is absent, writes the
   pin in the fixture header, and runs the upstream `classify` from `packages/ddd-migrate` of that
   worktree with Bun over two input sets. The pin moves only in a change that edits the header and
   regenerates both fixtures together:
   - every `git ls-files` path of this repository;
   - an adversarial set, committed as an input file, with at least one path for each
     `MalformedReason` the Rust shell can produce (`unknown_structural_kind`, `superseded_kind`,
     `unknown_language`, `unknown_test_kind`, `bad_name_shape`, `not_lowercase`, `leading_digit`,
     `reserved_word`, `tag_suffix_in_path`, `duplicate_kind`, `reserved_basename`,
     `token_below_shell`, `file_outside_shell`, `unnormalized_path`), one path for each step of the
     refusal precedence (grammar README section 2), `superseded_kind` below a shell and beside an
     `Outside` segment and an anchor basename, `reserved_basename` inside and outside an adopted
     subtree, a package segment that holds `_`, `lang_rust/build.rs`, `lang_rust/proptest-regressions/x.txt`,
     and `lang_rust/benches/x.rs`.
   Each fixture row is the path and the upstream result as canonical JSON.

3. **Generate `front-matter.tsv` the same way** from the upstream `parseFrontMatter` and
   `upsertFrontMatter`, over committed input files for the `block` and `hash` carriers: a clean block
   below `#![forbid(unsafe_code)]`, a block above `//!`, a misplaced block below `//!`, two blocks,
   an unterminated block, an empty payload, an unknown key, an unsorted `tags` array, a bad DATE,
   a `*/` inside a value, and a `/*` inside a value. Each row holds the upstream outcome (value or
   error tag and line) and, for a write case, the upstream output bytes.

4. **Port the grammar subset** in `check_ddd/grammar.rs` and `check_ddd/shell.rs`. The shell decode
   is strict: an unknown key is an error, and a shell run that declares a structural or superseded
   kind is an error (grammar README section 4). All four matcher forms decode, because the fixture
   holds the upstream answer for each. The reserved-word floor is the 52 words of section 1.

5. **Port the front-matter subset** in `check_ddd/front_matter.rs`: the preamble rule (a Rust `#![`
   inner attribute is preamble; a `//!` line STOPS the preamble), the 11-field registry, the IDENT,
   DATE and URI shapes, `tags` and `data` non-empty, unique and sorted, the two neutralised
   sequences, and the canonical writer in registry order. **This workspace adds one rule that
   upstream does not have: a `.rs` block never carries `subdomain`**, because the context map
   declares it once per context and a per-file copy has no parity check.

6. **Write `.ddd/context-map.toml`** with the nine product contexts of architecture section 1.2 and
   the two tool contexts (`plan_store` holds `plan-db`, `repo_guard` holds `xtask`, both `generic`).
   Each `[[relationship]]` names `downstream`, `upstream`, and one upstream `RelationshipPattern`
   literal. `check-ddd` then checks:
   - every crate of every context is a workspace member, and every member is in exactly one context;
   - every member manifest sits at `(crates|tools)/bc_<context>/<package>/lang_rust/Cargo.toml`, the
     unit segment equals `package.name`, and the name holds no `_`;
   - **the `bc_<context>` segment of every member path equals the context the file gives that
     crate**, so a `git mv` of a crate into another context directory is a finding;
   - the context graph over the `cargo metadata` edges is acyclic, and every cross-context edge has
     a declared relationship (`relationship-undeclared`, a finding);
   - a declared relationship with no edge yet is `relationship-unrealised`, which the run REPORTS
     and does not fail, because a later chunk builds the edge;
   - with `--architecture <path>`, the `context-map` block of that document names the same crate to
     context pairs as the file, **over the crates under `crates/` alone**. The block holds the
     crates of the section 1.2 crate table, which PG40 holds to that table, and the tool contexts
     `plan_store` and `repo_guard` live in the file alone. A pair in one source and not the other is
     a finding in each direction.

7. **The command.** `cargo xtask check-ddd [--architecture <path>] [--require-front-matter <path>...]`
   classifies every `git ls-files` path, reads front matter from every `.rs` and `.toml` file that
   carries a sentinel, grades `l` against `DddLayer` and `p` against `TacticalPattern`, and runs the
   context-map checks. `Malformed` is a finding; `Ungraded`, `unlayered`, and `unassigned` are
   green. **A path named by `--require-front-matter` that the grammar grades as `Graded`, `Test`,
   or an `Anchor` of kind `crate_root` and that carries no block is a finding.** The run prints
   `PATHS`, `MEMBERS`, `BLOCKS`, and `WITHOUT BLOCK` counters; `WITHOUT BLOCK` is a report and never
   a gate, because upstream bans a gate on an abstention count. A zero `PATHS` or `MEMBERS`
   denominator is fail-closed (exit 2). `--write <path> --l <layer> [--p <pattern>] [--tags <t>...]`
   writes one canonical block with the step 5 writer and changes nothing else in the file.

8. **Add the `ddd` step to `scripts/dod.sh`**, after `manifests`. It passes
   `--architecture roadmap/duet-v1/architecture.md` on EVERY run, because a move of a crate or an
   edit of `.ddd/context-map.toml` touches no file under `roadmap/`. It passes every changed `.rs`
   path of `CHANGED_PATHS` that exists in the working tree to `--require-front-matter`; a deleted
   path and the old side of a rename are skipped, because `--no-renames` lists both. **When
   `DENOMINATOR_UNKNOWN` is 1, the step passes `--require-front-matter-all`**, which requires a block
   on every tracked `.rs` file in the step 7 set (`Graded`, `Test`, and `Anchor` of kind
   `crate_root`). Between this chunk and chunk D2, such a run is red on the four D2 dynamics stubs,
   which is intended: the step cannot see which files changed, so it answers yes the
   way `touches` does, and the remedy for a red run is a branch with an upstream or `origin/main`. **So after
   this chunk lands, every chunk that writes a `.rs` file writes its block in the same commit, and
   the gate refuses a commit that does not.**

9. **Write the blocks** for every `.rs` path of the write scope with `cargo xtask check-ddd --write`.
   The three `lib.rs` skeletons that chunk M2 created are in the write scope: rule 4 of
   `check-plan-graph` allows that overlap, because T3, A1, and E1 depend on this chunk.
   Choose `l` from `domain`, `application`, `infrastructure`, `interface`, and `unlayered`, and `p`
   from the `TacticalPattern` set, by the design intent the file carries. Choose `unlayered` or
   `unassigned` when no term is honest (grammar README section 9); never invent a value to avoid an
   abstention. The four `duet-dsp` dynamics files that chunk D2 writes in this phase are outside the
   write scope, and D2 writes their blocks, because D2 depends on this chunk and the gate requires a
   block on every file D2 changes.

10. Edit `CLAUDE.md`: the comment rule lists the `/* ---uv ... --- */` block as machine-read, and
    the "Path grammar" rule names `cargo xtask check-ddd` in its `Enforced by` line.

11. Commit on `chunk/m94-ddd-guard`. The hook runs the gate, now with the `ddd` step.

## Tests

In `tests/ddd.rs` (the `test-author` skill), every assert with a message:

- `paths.tsv` parity: the port answers every row exactly as upstream did.
- `front-matter.tsv` parity: every read outcome and every written byte string.
- Each `MalformedReason` row of the adversarial set is red in the port, one test per reason.
- The writer is idempotent: a second `--write` with the same value changes no byte.
- Red probes over a scratch repository: a crate moved to a `bc_` directory that the file does not
  give it, a context-map pair in the file and not the architecture block, a pair in the block and
  not the file, a deleted `.rs` path passed to `--require-front-matter` (skipped, green), a member at `crates/duet-x/lang_rust/`, a unit segment that
  differs from `package.name`, a package name with `_`, a context cycle, an undeclared cross-context
  edge, a `subdomain` key in a `.rs` block, a `--require-front-matter` path with no block, an empty
  repository (exit 2), and an `--architecture` document whose block disagrees with the file.
- Each red probe fails before the rule exists and passes after; record the two runs in the commit
  body.

## What this chunk does NOT cover

- Languages other than Rust. The shell table declares Rust alone, so an adopted path of another
  language is `file_outside_shell` and the port needs no other shell.
- The `apostrophe` and `xml` carriers. No file this plan writes uses them. A `.md` file carries no
  block.
- Refutation oracles (grammar README section 7). The port grades vocabulary membership only.
- A gate on the `WITHOUT BLOCK` count. Files that no chunk touches after this one keep no block until
  a chunk writes them; the counter makes that visible.

## Verification

```sh
cargo xtask check-ddd --architecture roadmap/duet-v1/architecture.md   # exit 0, counters non-zero
cargo nextest run -p xtask --test ddd --no-tests=fail
cargo xtask check-plan-graph roadmap/duet-v1 --check-manifest
bash scripts/ddd-fixtures.sh ~/Developer/ultravisor && git diff --exit-code -- tools/bc_repo_guard/xtask/lang_rust/tests/fixtures/ddd/
```

## Constraints

- The cargo-only rule, the commit-gate rule, and the suppression rule of CLAUDE.md hold. The gate
  never runs Bun; only `scripts/ddd-fixtures.sh` does, by hand, when the pinned upstream commit
  changes.
- No new third-party crate. `toml`, `serde`, and `serde_json` are pinned already.
- No prose comments. A front-matter block is machine-read and is the one new comment form.
