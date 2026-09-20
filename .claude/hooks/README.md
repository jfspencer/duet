# Claude Code hooks

The native git hooks `.githooks/pre-commit` and `.githooks/pre-push` run `scripts/dod.sh`, the only Definition of Done (DoD) gate; `.githooks/commit-msg` rejects an AI-attribution trailer or marker in a commit message (root CLAUDE.md, Git workflow). Claude hooks gate edits and Bash calls, not commits. All hooks short-circuit cleanly when `jq` is missing.

Registered in `.claude/settings.json`. Codex reaches the same scripts through `.codex/hooks.json` (and `.codex/hooks/apply-patch-adapter.sh` for the edit hook).

All scripts use `#!/usr/bin/env bash` for portability.

## The commit gate is not here

`scripts/install-hooks.sh` points `core.hooksPath` at `.githooks/`. Both hooks exec `scripts/dod.sh`, which runs `cargo fmt --all --check`, `cargo clippy --workspace --all-targets --locked -- -D warnings`, `cargo doc` with `-D warnings`, `cargo nextest run` (or `cargo test`), `cargo test --doc`, `cargo deny check`, `cargo machete`, `cargo xtask sync-agents --check`, and `bash -n` over every shell hook. CI runs the same script.

No Claude hook runs the DoD. No `Stop` or `SubagentStop` hook exists. The rule for an agent: make the change, then `git commit`. The hook runs the gate and blocks a bad commit. Do not hand-run the gate as a pre-commit ritual. `scripts/dod.sh --plan` is a read-only dry run and is allowed.

`--no-verify` stays banned on `git commit` and on `git push`. `pre-git-destructive.sh` refuses `git commit --no-verify` inside Claude's Bash tool. Nothing inspects `git push`.

## Shared helpers

| File | Purpose |
|---|---|
| `_lib-tokenize.sh` | Quote-aware shell tokenizer (`__claude_hook_tokenize`) plus env-var stripper (`__claude_hook_strip_env_vars`). Sourced (not executed) by `pre-git-destructive.sh`. Centralizes the awk tokenizer so escaped-quote evasions (`git "commit"`) are handled identically across every gate that classifies a Bash command by its first tokens. Files prefixed with `_` are libraries, never registered as hooks. |
| `_lib-hypervisor.sh` | Identity gate + `db.sh` helpers for the two Hypervisor compaction hooks (`__hypervisor_session_dir`, `__hypervisor_is_hv`, `__hypervisor_plan_dir`, `__hypervisor_proj`, `__hypervisor_db`). Sourced (not executed). Markers live in a PER-SESSION subdir `<cwd>/.hypervisor/<session_id>/` so multiple Hypervisors in one cwd never collide. `__hypervisor_is_hv <session_id> [cwd]` is the positive gate: it acts only when BOTH the per-session subdir named by the hook's stdin `session_id` exists AND the OWNER recorded in its `role` matches that `session_id` (file only — NEVER the inheritable `$HYPERVISOR_PLAN_DIR` env); `__hypervisor_plan_dir <session_id> [cwd]` reads `<cwd>/.hypervisor/<session_id>/plan-dir` (file only). Returns nothing / false for any non-Hypervisor session, so the hooks are inert in ordinary developer sessions. |

## Active hooks (5)

No automated suite covers these hooks yet; `scripts/dod.sh` runs `bash -n` (and `shellcheck` when installed) over every script. A change to a hook is verified by hand with a real payload on stdin.

| Hook | Event | Mode |
|---|---|---|
| `pre-git-destructive.sh` | `PreToolUse` (Bash) | Blocking + Advisory |
| `post-edit-rustfmt.sh` | `PostToolUse` (Edit\|Write) | Advisory |
| `instructions-loaded-log.sh` | `InstructionsLoaded` | Observability (never blocks) |
| `pre-compact-hypervisor-checkpoint.sh` | `PreCompact` (all triggers) | Blocking on manual w/o checkpoint; advisory on auto |
| `session-start-hypervisor-reanchor.sh` | `SessionStart` (all sources; acts on `compact`/`resume`) | Context injection (never blocks) |

The two Hypervisor compaction hooks are no-ops outside a Hypervisor run — see below.

## Not ported from project-xavier

| Hook | Why |
|---|---|
| `post-edit-biome.sh`, `post-edit-ast-grep.sh` | TypeScript tooling. `post-edit-rustfmt.sh` is the Rust equivalent; clippy runs in the gate, not per edit (a workspace clippy pass is too slow for an edit hook). |
| `pre-edit-no-hardcoded-hex.sh`, `pre-edit-migration-idempotent.sh` | Svelte design tokens and SQL migrations do not exist here. GPUI theming holds by the `gpui-kit-design-guides` skill and review. |
| `beast-pre-training-gate.sh`, `beast-model-publish-gate.sh`, `session-start-native-freshness.sh` | Xavier-specific research and napi binding checks. |

## Hook details

### `post-edit-rustfmt.sh`
**Purpose:** Runs `rustfmt` (with the repo `rustfmt.toml`) on a `.rs` file the moment it is written, and `taplo fmt` on a `.toml` file when taplo is installed, so an agent does not need to remember to format.
**Trigger:** `PostToolUse` + `Edit|Write` (filters in-script by extension).
**Mode:** Advisory (`exit 0` always). `cargo fmt --all --check` in the gate is the blocking surface.


### `pre-git-destructive.sh`
**Purpose:** Blocks the destructive git operations root CLAUDE.md bans under Git workflow.
**Trigger:** `PreToolUse` + `Bash` (joins the existing Bash matcher block; short-circuits in-script on commands without `git`).
**Blocks (`exit 2`):**
- `git push --force` (or `-f`) WITHOUT `--force-with-lease`. The lease variant is allowed on personal branches; bare `--force` is too broad.
- `git commit --no-verify` as a real argv flag. **Token-level scan:** walks the command's tokens respecting shell quoting, so a commit message body that quotes the flag for documentation purposes (`git commit -m "see --no-verify docs"`) is NOT a false positive, while flag-after-message (`git commit -m "feat: x" --no-verify`) and flag-before-message (`git commit --no-verify -m "feat: x"`) are both blocked. The hook does not inspect `git push`, so `git push --no-verify` has no mechanical block; the ban holds by review.

**Advises (`exit 0` with stderr WARNING):**
- `git reset --hard`. Sometimes legitimate (post-merge cleanup); warns but does not block.

Reference: root `CLAUDE.md`, Git workflow.

### `instructions-loaded-log.sh`
**Purpose:** Pure observability. Logs the list of CLAUDE.md files auto-loaded into a session so "why didn't this rule fire?" can be answered by replaying the loaded set.
**Trigger:** `InstructionsLoaded` (fires once after Claude Code auto-loads the layered memory system; see https://code.claude.com/docs/en/hooks).
**Mode:** Observability (`exit 0` always). Defensive against schema renames: tries several JSON shapes (`.instructions[].path`, `.tool_input.instructions[].path`, `.files[].path`, `.hookSpecificOutput.files[].path`) and logs a placeholder if none yields a match.
**Log location:** `$XDG_STATE_HOME/claude/instructions-loaded/<project-slug>.log` (default `~/.local/state/claude/instructions-loaded/<project-slug>.log`). One tab-separated line per session: `<ISO timestamp>\t<comma-separated paths>`. The project slug is `basename "$(pwd)"`.
**Why per-project (not `/tmp/claude-instructions-loaded.log`):** the previous shared `/tmp` path interleaved every project on the machine into a single file, making per-session replay useless. Namespacing by the basename of cwd keeps each project's stream isolated.
**Atomicity:** Each line is a single `printf >> "$LOG_FILE"`. POSIX guarantees `write(2)` calls of `<= PIPE_BUF` (4096 bytes on Linux and macOS) to an `O_APPEND` file are atomic with respect to other appenders. A loaded-instruction line is timestamp + tab + a CSV of CLAUDE.md paths, well under 4KB even with the full layered set loaded; tmp+mv swap is unnecessary at this scale.

### `pre-compact-hypervisor-checkpoint.sh` and `session-start-hypervisor-reanchor.sh` (Hypervisor compaction)

**Purpose:** Implement the Hypervisor's *compact-and-continue* self-continuity (replaces the old "wind down at 60% and wait for an operator restart"). Design references: the on-disk wiring + file-ownership table in `.claude/plan-coordination/README.md` and the agent contract `.claude/agents/engineering/hypervisor.md`.

- **`pre-compact-hypervisor-checkpoint.sh` (Seam 2 — durability).** `PreCompact` (a hook cannot inject context, only block). On every compaction boundary it appends a `compaction-frame` audit row via `db.sh` (the *observable* marker the control loop watches to know a compaction happened) and verifies a fresh `current_context` checkpoint exists. **Never blocks on `trigger=auto`** (blocking auto-compaction risks a hard context death) — it warns only. **Blocks (`exit 2`) on `trigger=manual`** when there is no fresh checkpoint, forcing a CHECKPOINT before a deliberate `/compact`.
- **`session-start-hypervisor-reanchor.sh` (Seam 3 — re-anchor + role re-assertion).** `SessionStart`, acting on `source` `compact` (after auto/manual compaction) and `resume`. Emits `hookSpecificOutput.additionalContext` whose FIRST line re-asserts **"YOU ARE THE HYPERVISOR"** and then re-hydrates the session from the LMDB store (resume snapshot pointer, `control:signal`, pacing, open-front counts, the keys index). This is the belt-and-braces guard against losing the Hypervisor role across a compaction; the agent's system prompt is the primary guarantee, and the control loop's own observable re-hydration is the final backstop.

**Mode / safety:** Both `source` `_lib-hypervisor.sh` and **self-gate to a complete no-op** (`pre-compact` `exit 0`; `session-start` prints `{}`) unless this session is POSITIVELY identified as the Hypervisor by an OWNER-MATCH on the non-inheritable marker `<session>/.hypervisor/role` (it holds `hypervisor <owning-session-id>`, stamped by INITIALIZE from the statusline-written `.hypervisor/session-id`, removed at HALT): the hook acts only when its own stdin `session_id` equals that owner — NEVER from the inheritable `HYPERVISOR_PLAN_DIR` env. So a dispatched child (different `session_id`, and given its own dir by the Child-spawn contract) is **never** mistaken for the Hypervisor, and a crashed Hypervisor's marker (a dead owner id no live session matches) is harmlessly inert — no liveness epoch needed. `session-start` additionally acts only on `source ∈ {compact, resume}` (a worker's `startup` is ignored). The gate is checked *before* invoking the store binary, so the hooks are inert and cheap for every ordinary developer session and every worker. They also no-op (never failing a compaction) when `jq`/`db.sh` is absent, and every store call is wrapped in a 20s `timeout`; the Hypervisor Bootstrap Gate (D18, check C) asserts the tooling is present for an actual run, and `autoCompactEnabled` must stay true (the trigger).
**Registration:** matcher-less in `.claude/settings.json`, so `PreCompact` covers both triggers and `SessionStart` reliably covers the `compact` source.
