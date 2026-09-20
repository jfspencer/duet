---
name: verification-before-completion
description: Use when about to claim work is complete, fixed, or passing, before committing, creating PRs, or reporting success. Requires running verification commands and confirming output before making any success claims.
allowed-tools: Bash(cargo:*) Bash(git:*) Read
verified: 2026-09-20
verified-against:
  - scripts/dod.sh
  - .githooks/pre-commit
  - .githooks/pre-push
  - Cargo.toml
  - clippy.toml
  - deny.toml
  - CLAUDE.md
review-cadence: on-architectural-change
---

# Verification Before Completion

## Overview

A completion claim without verification is dishonesty, not efficiency.

**Core principle:** Evidence before claims, always.

## When NOT to use

- A single command's exit code mid-task. Run the command and read the output.
- A failure that has already surfaced. Use `systematic-debugging` to find the root cause, then return here to verify the fix.
- A draft commit or PR before the change is finished. The gate fires at commit time only.

Use this skill before a claim that work is complete, fixed, or passing. Use it before a commit, before a PR, and before a success report. An unverified success claim is the failure mode.

## The Iron Law

```
NO COMPLETION CLAIMS WITHOUT FRESH VERIFICATION EVIDENCE
```

Without verification evidence in this response, a passing claim is not permitted.

## The Gate Function

Before a status claim:

1. IDENTIFY: What evidence proves this claim?
2. RUN: Run the command that produces the evidence, or commit and let the hook run it.
3. READ: Read the full output. Check the exit code. Count the failures.
4. VERIFY: Does the output confirm the claim? If NO, state the actual status with evidence. If YES, state the claim WITH evidence.
5. ONLY THEN: Make the claim.

## Definition of Done

The native git hook `.githooks/pre-commit` is the ONLY Definition of Done (DoD) gate. It runs `scripts/dod.sh`. It fires on every `git commit`, for every committer, and `.githooks/pre-push` runs the same script again on push. No Claude hook runs the DoD.

`scripts/dod.sh` runs these steps, in order, and stops at the first failure:

1. `cargo fmt --all --check`
2. `cargo clippy --workspace --all-targets --locked -- -D warnings`
3. `cargo doc --workspace --no-deps --document-private-items --locked` under `RUSTDOCFLAGS=-D warnings`
4. `cargo nextest run --workspace --locked` (fallback: `cargo test --workspace --locked`)
5. `cargo test --doc --workspace --locked`
6. `cargo deny check` (advisories, licenses, bans, sources)
7. `cargo machete` (unused dependencies)
8. `cargo xtask sync-agents --check` (the Codex and OpenCode agent mirrors match `.claude/agents`)
9. `bash -n` on every shell hook and script (`shellcheck` when installed)
10. `typos` (advisory, when installed)

**The rule: make the change, then `git commit`.** The hook runs the gate and blocks a bad commit. Do not hand-run the DoD as a pre-commit ritual. Do not run `scripts/dod.sh`, `cargo clippy`, `cargo nextest run`, or `cargo fmt --all --check` before a commit to "check first". The hook output is the evidence. One targeted command to diagnose a failure that the hook reported is correct (`cargo nextest run -p <crate> <filter>`, `cargo clippy -p <crate> --all-targets -- -D warnings`). `scripts/dod.sh --plan` is a read-only dry run and is allowed.

`--no-verify` stays banned on `git commit` and on `git push`. `pre-git-destructive.sh` refuses `git commit --no-verify` inside Claude's Bash tool. Nothing inspects `git push`.

Strict style is machine-enforced by the gate. The authoritative rule sources are the `[workspace.lints]` table in the root `Cargo.toml`, `clippy.toml`, `deny.toml`, and `rustfmt.toml` (see root `CLAUDE.md`). Lowering a lint level, adding a `#[allow]`, or adding a `deny.toml` exception to make the gate pass is a defect, not a fix; the only accepted suppression is `#[expect(lint, reason = "...")]` at one site. CI (`.github/workflows/ci.yml`) re-runs `scripts/dod.sh` verbatim on macOS and Linux for every PR.

## Verification Requirements by Claim

| Claim | Required Evidence | NOT Sufficient |
|-------|-------------------|----------------|
| "The change is done" | A commit that passed the hook, quoted from the hook output | "I made the edit" |
| "It compiles" | The hook's `cargo clippy` step passed (clippy type-checks every target) | "I didn't add any type errors" |
| "Tests pass" | The hook's `cargo nextest run` and `cargo test --doc` steps with 0 failures, or one targeted `cargo nextest run -p <crate> <filter>` run to diagnose a reported failure | A previous run, "should pass" |
| "Lint is clean" | The hook's `cargo clippy ... -D warnings` and `cargo fmt --all --check` steps passed | "I followed the style" |
| "Docs build" | The hook's `cargo doc` step passed under `-D warnings` | "I documented everything" |
| "Dependencies are clean" | The hook's `cargo deny check` and `cargo machete` steps passed | "I only added one crate" |
| "Bug is fixed" | A test for the original symptom passes | "Code changed, assumed fixed" |
| "Refactor is safe" | A commit that passed the hook, plus a grep for old paths | "I updated the imports" |
| "Agent completed the work" | Independent verification of the agent's output | An agent report of "success" |

## Red Flags, STOP

Any of the following is a STOP signal:
- The words "should", "probably", or "seems to"
- Satisfaction before verification
- A completion report with no commit that passed the hook
- A hand-run DoD ritual (`scripts/dod.sh`, `cargo clippy`, `cargo nextest run`, `cargo fmt --all --check`) in place of a commit
- Trust in a subagent's success report without independent verification
- **ANY wording that implies success without verification**

## The Bottom Line

Commit the change. Read the hook output. THEN claim the result.

No shortcuts. Non-negotiable.
