# Chunk author brief: roadmap/duet-v1 (2026-09-20)

You author plan chunk files for the Duet v1 roadmap. A chunk file is the dispatch unit a fresh orchestrator executes without your context. Read, in this order: `/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/shared-brief.md` (product and operator decisions), `roadmap/duet-v1/architecture.md` section 13 in full, then every architecture section, ADR, design-contract section, and product-requirements story that your assigned chunks cite. `CLAUDE.md` holds the repository rules. Code is the source of truth: `crates/duet/src/`, `Cargo.toml`, `deny.toml`, `clippy.toml`, `scripts/dod.sh` are the current state; a chunk that assumes a file which does not exist yet must say "create".

## File name and location
One chunk is one file: `roadmap/duet-v1/<line>-<nn>-<slug>.md`, for example `trunk-01-time-kernel.md`, `a-02-spacing.md`, `x-01-musicxml-reader.md`, `m-03-manifest-phase-3.md`. The `line` is `m` for manifest chunks, `trunk` for T chunks, else the lower-case line letter. `nn` is two digits. Write only the files your assignment names.

## Front-matter (every file opens with this block; the scheduler reads it)
```yaml
---
id: A2                       # the chunk id from architecture section 13
line: A                      # trunk | M | A | C | D | E | F | G | H | I | J | K | N | X  (X is interchange; chunk ids never collide with budget ids)
depends_on: [T2, A1]         # chunk ids that must be DONE first; [] for M0
write_scope:                 # exact directories or files this chunk writes; nothing else
  - crates/duet-engrave/src/spacing.rs
  - crates/duet-engrave/src/system.rs
parallelism: independent     # or: serial-only: <one-line reason from section 13.4>
completion: "cargo nextest run -p duet-engrave --locked passes; cargo clippy -p duet-engrave --all-targets -- -D warnings is clean; commit SHA on a branch chunk/a2-spacing"
---
```
Rules: a line is an ordered chain, so every chunk of a line depends on the previous chunk of the same line, and two chunks of one line never share a phase. `depends_on` must list every predecessor that section 13.4 names for this chunk, plus the manifest chunk of the chunk's phase (section 13.3). The manifest chunk of a phase depends on every chunk of the prior phase. No non-M chunk touches the root `Cargo.toml`. The M chunk of a phase pins every dependency of the phase in the root `[workspace.dependencies]` and creates the skeleton of every crate whose first chunk runs in that phase (member `Cargo.toml` with the package fields and `[lints] workspace = true` and NO `[dependencies]`; `src/lib.rs` with the `//!` doc and `#![forbid(unsafe_code)]`). The first chunk of a line adds `{ workspace = true }` entries to its own member `Cargo.toml` together with the code that uses them (so `cargo machete` passes), writes under `src/`, `tests/`, and `benches/`, and adds the `mod` lines to `lib.rs`. `Cargo.lock` is in the `write_scope` of every M chunk and of every chunk that adds a dependency to a member manifest; a merge conflict on it is resolved with `git checkout --ours Cargo.lock && cargo build --workspace`, which is deterministic because every version is pinned. A `--locked` flag in a `completion` command is valid only after the chunk commits its lock file. K1 creates every file under `crates/duet/src/element/` as a stub (the `//!` doc only); K2 to K6 modify those stubs. `completion` is a command list a machine runs on macOS and Linux in CI; name the crate; a UI chunk names its `#[gpui_kit::test]` test module.

## Body (bite-sized, self-contained, no placeholders)
Sections, in this order:
1. `# <id>: <title>` and one paragraph of goal, with the architecture sections, ADRs, design-contract sections, and product stories this chunk implements, by number.
2. `## Files`: every path to create or modify, one line each, with "create" or "modify".
3. `## Types and signatures`: the exact Rust type and function signatures from the architecture, copied, with the module path. A chunk that defines a type shows its full definition. A chunk that consumes a type names the crate and path it comes from.
4. `## Steps`: numbered; one action per step; test first where a test can exist ("write the failing test", "run it and confirm it fails", "implement", "run it and confirm it passes"). Show the code a step adds when the step changes code. Give exact commands with the expected result.
5. `## Tests`: the named tests, what each asserts, and where it lives (same-file `#[cfg(test)] mod tests`, `tests/`, or `#[gpui_kit::test]`). Every assert carries a message. Property tests name the proptest strategy.
6. `## Verification`: what passing looks like: the commands, the expected output, and the commit on a branch named `chunk/<id-lowercase>-<slug>`.
7. `## Constraints`: copy this block verbatim into every chunk:
   - Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
   - The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
   - No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
   - `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
   - No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
   - A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
   - Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
   - Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
   - Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.

## M chunks
An M chunk (`m-<n>-manifest-phase-<n>.md`) writes root `Cargo.toml` (`[workspace.dependencies]` pins for the phase, from `roadmap/duet-v1/research/crate-survey.md` versions), the skeleton of every crate whose first chunk runs in the phase, and, for M0 only, `deny.toml` licence allow entries (MPL-2.0, CC0-1.0, Unlicense, Apache-2.0 and Apache-2.0 WITH LLVM-exception), the `NOTICE` file skeleton (Apache NOTICE text, the Bravura OFL-1.1 licence), the Bravura font asset under `crates/duet/assets/fonts/` with its `LICENSE.txt`, the `cargo xtask check-conversions` command in `tools/xtask` and its line in `scripts/dod.sh`, the `cargo xtask check-placement` and `cargo xtask check-closure` commands (ports of `roadmap/duet-v1/tools/placement_check.py` and `closure_check.py`, the latter over `roadmap/duet-v1/reviews/`) and their `plan-lint` job in `.github/workflows/ci.yml` that runs only on changes under `roadmap/` (never in `scripts/dod.sh`), the `dod` job of `ci.yml` moved to `ubuntu-26.04` and `macos-26` with `libpipewire-0.3-dev` installed, and the repair of the broken links in `.claude/skills/gpui-kit/SKILL.md` (about 15 links to `references/gpui/*.md` that do not exist: remove the table rows or restore the files from `https://gpui-kit.com/llms.txt`). The Orchestrator executes every M chunk directly because those are policy files; say so in the chunk body. An M chunk's `completion` is `bash scripts/dod.sh --plan` then a commit that passes the hook.

## Writing standard
ASD-STE100: active voice, one instruction per sentence, sentences under 25 words, no -ing verb forms, no idiom, no em dashes. Code identifiers, signatures, and commands are copied exactly.

## Report protocol
When done, write the list of files you wrote and a ten-line summary to a scratchpad file, then run from `/Users/james/Developer/duet`:
    .claude/plan-coordination/db.sh append roadmap/duet-v1 <SUFFIX> "$(cat <your-file>)"
Return only the summary, the printed key, and an importance flag. Do not run git.
