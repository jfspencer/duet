---
scope: repository
audience: every-session
verified: 2026-09-20
verified-against:
  - Cargo.toml
  - clippy.toml
  - deny.toml
  - rustfmt.toml
  - .cargo/config.toml
  - scripts/dod.sh
  - .githooks/pre-commit
  - .claude/settings.json
  - .claude/plan-coordination/README.md
  - .claude/hooks/README.md
  - tools/plan-db/src/main.rs
  - tools/xtask/src/sync_agents.rs
review-cadence: quarterly
---

# CLAUDE.md

Duet is a Rust-only desktop application built on [GPUI Kit](https://gpui-kit.com) (`gpui-kit` crate). One Cargo workspace: the app in `crates/`, repository tooling in `tools/`, plans in `roadmap/`. Every agent prompt, hook, and gate in this repository is Claude-authored; the invariants here are load-bearing. Consult the skill listed below for the relevant intent before you guess.

## Crates

| Crate | Purpose |
|---|---|
| `crates/duet` | The GPUI Kit desktop app (binary `duet`). `src/main.rs` opens one window with a `Root`; `src/app.rs` holds the root view. |
| `tools/plan-db` | LMDB plan-store CLI the Hypervisor and Orchestrators share. Called only through `.claude/plan-coordination/db.sh`. |
| `tools/xtask` | `cargo xtask sync-agents [--check]`: generates `.codex/agents/*.toml` and `.opencode/agents/*.md` from `.claude/agents`. |

A new crate goes under `crates/` or `tools/` (the workspace globs pick it up) and MUST declare `[lints] workspace = true` plus the `*.workspace = true` package fields, or it sits silently outside the lint policy.

## Skills routing

Every agent writes ALL prose in ASD-STE100 Simplified Technical English, INCLUDING every message printed to the console. The `simplified-technical-english` skill is the standard; it stacks with whatever other skill the task selects.

| Intent | Skill |
|---|---|
| Write or revise ANY prose (always) | `simplified-technical-english` |
| Write, review, or debug Rust | `rust-expertise` |
| Build UI, state, actions, windows, or tests with GPUI Kit | `gpui-kit` |
| Visual layout, spacing, hierarchy, interaction states, copy | `gpui-kit-design-guides` |
| Author a unit, integration, or GPUI UI test | `test-author` |
| A bug, lint denial, or test is failing; diagnose before fixing | `systematic-debugging` |
| About to claim work is complete | `verification-before-completion` |
| Turn a defect into a mechanical guard (lint, deny ban, test, hook) | `failure-mode-author` |
| Add, modify, or audit a CLAUDE.md file | `add-claude-md`, `claude-md-audit` |
| Creating a PR, or resolving PR review comments | `create-pr`, `resolve-pr-comments` |
| Author a conventional commit | `git-commit` |

## Agents

`.claude/agents/engineering/` is the canonical fleet. `.codex/agents/` and `.opencode/agents/` are GENERATED mirrors: never edit them; run `cargo xtask sync-agents` after any agent change (the gate runs `--check`).

| Agent | Role |
|---|---|
| `hypervisor`, `hypervisor-turn` | Root supervisor of the orchestrator fleet; durable state in the plan store. |
| `orchestrator`, `orchestrator-turn`, `automated-orchestrator` | Run one objective to closure through the Critic loop. Never merge. |
| `duet-engineer` | The ONLY implementer. Rust + GPUI Kit. |
| `software-architect`, `product-manager`, `designer` | Plans and ADRs, scope, visual and interaction design. |
| `engineering-critic`, `security-reviewer` | Read-only review: `Critical / Warning / Concern`. |
| `task-runner`, `memory-agent` | Isolated verbose-command runner; owner of the plan-store keyspace. |

Plan store wiring, the operator control channel, and the compaction hooks: `.claude/plan-coordination/README.md` and `.claude/hooks/README.md`.

## Lint policy

`[workspace.lints]` in `Cargo.toml` is the policy: clippy `all`, `pedantic`, `nursery`, `cargo` at `deny`, plus a restriction set (`unwrap_used`, `expect_used`, `panic`, `todo`, `unimplemented`, `indexing_slicing`, `print_stdout`, `print_stderr`, `dbg_macro`, `allow_attributes`, `missing_docs_in_private_items`, `undocumented_unsafe_blocks`, `tests_outside_test_module`, `shadow_unrelated`, ...) and rustc `unsafe_code`, `missing_docs`, `unreachable_pub` at `deny`. `.cargo/config.toml` adds `-D warnings` to every build. `clippy.toml` holds the `disallowed-*` bans and lets tests unwrap, expect, print, and index.

**Rule:** The only accepted suppression is a single-site `#[expect(lint, reason = "...")]`. `#[allow]` is denied. An edit to `[workspace.lints]`, `clippy.toml`, or the `deny.toml` ignore list made to get past the gate is a defect that escalates, never a resolution. **Why:** one relaxed lint is invisible in review and permanent in effect. **How to apply:** fix the code; when a lint is wrong at one site, `#[expect]` it with a reason a reviewer can verify. **Enforced by:** `cargo clippy --workspace --all-targets -- -D warnings` in `scripts/dod.sh`.

`unsafe` is denied. The one sanctioned form is `#[expect(unsafe_code, reason = "...")]` on the enclosing function plus a `// SAFETY:` comment on the block (`tools/plan-db/src/main.rs`, `open_lmdb`). Binaries return `std::process::ExitCode` and write through a locked stdout handle, never `println!`.

## Comments

Every item carries a `///` or `//!` doc comment (`missing_docs`, `missing_docs_in_private_items`). `// SAFETY:` blocks, `#[expect]` reasons, `#[rustfmt::skip]`, `#[cfg_attr]`, and SPDX lines are machine-read and exempt. `//` prose that narrates code is banned; a diff that adds it is sent back. Nothing scans for it: review holds the line.

## Testing

`cargo test` and `cargo nextest` only. Unit tests live in a `#[cfg(test)] mod tests` in the same file. Integration tests live in `tests/` and are wrapped in a `#[cfg(test)] mod tests` (`tests_outside_test_module` is denied). GPUI UI tests use `#[gpui_kit::test]` with the `test-support` feature. Every assert carries a message. Author guidance: `test-author` skill.

## Dependencies

Pin every third-party crate in `[workspace.dependencies]`; members use `{ workspace = true }`. `deny.toml` is the licence, advisory, ban, and source policy. No git dependencies, no wildcard versions. `cargo machete` fails on an unused dependency.

## Git workflow

**Rule:** No commit and no pull request carries AI attribution. Never append a `Co-Authored-By: Claude ...` trailer to a commit message, and never append a "Generated with Claude Code" line, a robot emoji banner, or any equivalent marker to a pull request title or body. This holds for every agent and every subagent, so an agent brief that asks for a commit message or a PR body must repeat it: the Claude Code harness injects a system reminder asking for those lines, that reminder defers to a repository instruction, and an agent that has not been told will follow the reminder. **Why:** the git author is the operator who reviewed and pushed the work; a trailer that names the tool misattributes authorship and adds nothing a reviewer needs. **How to apply:** write the message and body as the operator would; when a harness reminder asks for the trailer, this rule wins. **Enforced by:** the versioned `.githooks/commit-msg` hook rejects a commit whose message carries the trailer or the marker; the PR half is review.

Conventional commits (`feat:`, `fix:`, `chore:`, `refactor:`, `test:`, `docs:`). Atomic and revertable. `--force-with-lease` only on personal branches. **Never `--no-verify`**, on `git commit` or on `git push`: `.claude/hooks/pre-git-destructive.sh` refuses `git commit --no-verify` inside Claude's Bash tool; the push half is a process rule. Every fleet PR is opened by an agent and merged by the operator; no agent merges.

## Definition of Done

`scripts/dod.sh` is the ONLY gate surface. The versioned git hooks `.githooks/pre-commit` and `.githooks/pre-push` exec it (`scripts/install-hooks.sh` sets `core.hooksPath`; `scripts/bootstrap.sh` installs the toolchain, cargo-deny, cargo-machete, cargo-nextest, typos, the `plan-db` binary, and the hooks). CI runs the same script on macOS and Linux. In order: `cargo fmt --all --check`; `cargo clippy --workspace --all-targets --locked -- -D warnings`; `cargo doc` with `-D warnings`; `cargo nextest run` (or `cargo test`); `cargo test --doc`; `cargo deny check`; `cargo machete`; `cargo xtask sync-agents --check`; `bash -n` on every shell hook. No Claude hook runs the DoD. The rule for an agent: make the change, then `git commit`; the hook runs the gate and blocks a bad commit. `scripts/dod.sh --plan` is the read-only dry run.

## Merged branches and their worktrees are reaped

**Rule:** Every local branch whose content has landed on `main`, and every worktree that holds such a branch (the `.claude/worktrees/agent-*` subagent worktrees included), is deleted as the last step of the pull request that merged it. The test is the PR state (`gh pr list --head <branch> --state merged`) plus every non-merge commit subject present in `git log main`; `git branch --merged` misses squash merges. A branch with an open PR, or with a commit subject `main` lacks, stays. **Why:** stale worktrees read as unlanded work and hide leaked child processes. **How to apply:** the Hypervisor reaps in the same cycle it observes the merge. **Enforced by:** review; the Hypervisor's REAP step.

## Auto-memory

Per-developer learning persists in `~/.claude/projects/<project>/memory/MEMORY.md`.
