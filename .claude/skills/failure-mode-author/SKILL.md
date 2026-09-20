---
name: failure-mode-author
description: Use right after hitting a defect a tool should have caught — a bug, incident, regression, flake, or a Critic/audit finding — to turn it into a tested mechanical guard plus a durable ledger entry. Walks the guard-layer decision tree (workspace lint, clippy.toml ban, cargo-deny rule, type-level fixture, unit or property test, Claude hook), proves the guard red-on-repro and green-on-fix, then writes an FM-NNNN entry in docs/failure-modes/. NOT for authoring ordinary features, tests, or CLAUDE.md rules, and not for one-off failures a guard cannot express.
allowed-tools: Bash Read Write Edit Grep
verified: 2026-09-20
verified-against:
  - Cargo.toml
  - clippy.toml
  - deny.toml
  - scripts/dod.sh
  - .claude/hooks/pre-git-destructive.sh
review-cadence: on-architectural-change
---

# Failure-mode author

## Trigger

Invoke the moment a defect surfaces that a tool could have caught before a human
did: a shipped bug, an incident, a regression, a load-induced flake, or a Critic
or audit finding. The output is a mechanical guard that fails on recurrence plus
a failure-mode ledger entry (`docs/failure-modes/FM-NNNN-slug.md`) recording the
mode and the guard. Also invoke when an operator states an invariant the codebase
must never violate, to codify it before it is breached.

## When NOT to use

- Authoring ordinary features or tests. Those route to `rust-expertise`,
  `gpui-kit`, and `test-author`.
- Adding a CLAUDE.md rule. That routes through `add-claude-md`; an FM entry is a
  ledger record, not a CLAUDE.md edit.
- A genuinely one-off failure whose recurrence no tool can express (a
  third-party outage, a one-time data-entry mistake). Record it as an incident
  note, not a failure mode.
- A guard candidate that still has existing violations. Fix the violations first,
  or the guard cannot be seeded clean.

## The ritual

1. **Reproduce minimally.** Reduce the defect to the smallest artifact that
   exhibits it: a failing test, a scratch file that trips the pattern, a command
   whose exit code flips. This artifact is the red-on-repro evidence.
2. **Walk the decision tree** (below) and pick the cheapest layer that FULLY
   covers the mode. Layers compose; prefer one strong layer over two weak ones.
3. **Author the guard plus its test.** Every guard ships with a proof: a test
   carries the failing case, a `clippy.toml` ban or a workspace lint is proven
   zero-violation across the workspace, a `deny.toml` rule is proven against
   `cargo deny check`.
4. **Prove red-on-repro, green-on-fix.** Run the guard's own check against the
   repro (it must fail) and against the fixed tree (it must pass). Quote both
   outputs. For a lint or ban, the check is
   `cargo clippy --workspace --all-targets -- -D warnings`. For a test, the
   check is `cargo nextest run -p <crate> <filter>`. For a `deny.toml` rule, the
   check is `cargo deny check`. For a hook, the check is the hook script fed a
   real payload on stdin and its exit code.
5. **Write the FM entry** in `docs/failure-modes/FM-NNNN-slug.md` (create the
   directory and its `README.md` index on the first entry): what happened,
   blast radius, guard layer and why it fully covers, resolving artifact links.
6. **Set the back-pointer.** In a clippy `reason` string, a `deny.toml`
   `reason`, a test name, or a hook header comment, cite the FM slug. The ledger
   links to the guard and the guard links back.
7. **Cite every artifact in the PR, then commit.** The native git hook runs the
   Definition of Done at commit time. Do not hand-run the gates before the
   commit.

## Guard-layer decision tree

Pick the first layer that fully covers the mode.

| Mode shape | Guard layer |
|---|---|
| Banned API, method, macro, or trait | `clippy.toml` `disallowed-methods` / `disallowed-macros` / `disallowed-types`, each with a `reason` |
| A pattern clippy already names | raise that lint to `deny` in `[workspace.lints.clippy]` (root `Cargo.toml`) |
| Banned or constrained dependency, license, or source | `deny.toml` `[bans] deny`, `[licenses]`, `[sources]` |
| Type-level invariant | a newtype or sealed enum, plus a `compile_fail` doctest that proves the wrong shape does not compile |
| Behavioral / algebraic property | a unit test in `#[cfg(test)] mod tests`, or a table-driven loop over cases; a `proptest` dependency only after `cargo deny check` accepts it |
| Cross-process or CLI contract | an integration test under `tests/` that spawns the binary (`tools/plan-db/tests/roundtrip.rs` is the reference) |
| Process / workflow rule | a `.claude/hooks/` hook, registered in `.claude/settings.json` and `.codex/hooks.json` |

Notes on the layers:

- The lint policy is declarative and one-way: `[workspace.lints]`, `clippy.toml`,
  and `deny.toml`. A candidate ban with an existing violation is fixed trivially
  or dropped, never suppressed with an inline `#[allow]`; `allow_attributes` is
  denied, and `#[expect(lint, reason = "...")]` is the only accepted exception.
  `.cargo/config.toml` sets `-D warnings`, so a new `warn`-level lint blocks the
  gate the same as a `deny`.
- A guard that lives in a test must have a positive control: the test fails on
  the repro. A test nobody has watched go red is not a guard.
- A Claude hook gates an edit or a Bash call, not a commit. The native git hook
  `.githooks/pre-commit` runs `scripts/dod.sh` and is the only commit gate.

## Cross-linking

The ledger and the guards form a closed loop: `docs/failure-modes/README.md`
indexes the entries, each entry links to its guard artifact, and each guard
carries a back-pointer (a clippy or cargo-deny `reason`, a test name, a hook
header) to its FM slug. An entry is FIXED spec: edit it only when the guard changes, and record a
superseded guard with a supersession note rather than deleting it.
