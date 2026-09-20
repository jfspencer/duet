---
name: systematic-debugging
description: Use when encountering any bug, test failure, build error, or unexpected behavior, before proposing fixes. Especially when under time pressure, when a previous fix didn't work, or when "just one quick fix" seems obvious.
allowed-tools: Bash Grep Read Edit
verified: 2026-09-20
verified-against:
  - CLAUDE.md
  - scripts/dod.sh
  - clippy.toml
review-cadence: quarterly
---

# Systematic Debugging

## Overview

Random fixes waste time and create new bugs. Quick patches mask underlying issues.

**Core principle:** ALWAYS find root cause before attempting fixes. Symptom fixes are failure.

## The Iron Law

```
NO FIXES WITHOUT ROOT CAUSE INVESTIGATION FIRST
```

If Step 1 is not yet complete, fixes cannot be proposed.

## When to Use

Use for ANY technical issue:
- Test failures (`cargo nextest run`, `cargo test --doc`)
- Build and lint failures (`cargo clippy -D warnings`, `cargo doc -D warnings`, `cargo deny check`)
- Unexpected behavior
- Performance problems (a dropped frame, a blocked render)
- GPUI entity, subscription, or focus errors

**Use this ESPECIALLY when:**
- Under time pressure
- "Just one quick fix" seems obvious
- You've already tried multiple fixes
- Previous fix didn't work

## When NOT to use

- Writing a new feature where no bug exists. Use the appropriate reference skill (`rust-expertise`, `gpui-kit`, `gpui-kit-design-guides`, `test-author`).
- Trivial syntax errors with an obvious one-line fix (missing import, typo, unclosed brace). Fix it directly.
- Architecture or "where does this live" questions. Read the root `CLAUDE.md` workspace map and the `roadmap/` plan.
- Verification of a completed change. Use `verification-before-completion`.

## The Four Steps

### Step 1: Root Cause Investigation

**BEFORE attempting ANY fix:**

1. **Read Error Messages Carefully**
   - Read the full diagnostic; note file paths, line numbers, the lint name in `-D clippy::<name>`, the `--explain` code.
   - For a panic: run with `RUST_BACKTRACE=1` and read the whole backtrace; for an `anyhow` chain, print `{:#}` to see every cause.

2. **Reproduce Consistently**
   - Can you trigger it reliably?

3. **Check Recent Changes**
   - `git diff` and recent commits
   - New dependencies, `Cargo.lock` changes, feature-flag changes
   - `pub use` re-export or module moves that may have broken paths

4. **Gather Evidence in Multi-Component Systems**

   Common boundaries to check:
   - View -> entity -> background task (state owned by the wrong entity, a `notify` on the wrong one)
   - Crate boundaries across the workspace (`crates/duet`, `tools/plan-db`, `tools/xtask`)
   - Process boundaries (the plan store is written by other processes)

5. **Trace Data Flow**
   - Where does the bad value originate?
   - Fix at source, not at symptom

### Step 2: Pattern Analysis

1. **Find Working Examples**, locate similar working code in the codebase.
2. **Compare Against References**, read the reference implementation COMPLETELY.
3. **Identify Differences**, list every difference, however small.
4. **Understand Dependencies**, check `Cargo.toml` (`[workspace.dependencies]`, features) and `cargo tree -p <crate>`.

### Step 3: Hypothesis and Testing

1. **Form Single Hypothesis**: "I think X is the root cause because Y".
2. **Test Minimally**, make the SMALLEST possible change.
3. **Verify Before Continuing**: did it work? If not, form NEW hypothesis.
4. **When You Don't Know**, say so. Escalate to the Orchestrator or ask the human.

### Step 4: Implementation

1. **Create Failing Test Case** in a `#[cfg(test)] mod tests` (or a `tests/*.rs` file); see `test-author`.
2. **Implement Single Fix**: ONE change at a time.
3. **Verify Fix**, run `cargo nextest run -p <crate> <filter>` and `cargo clippy -p <crate> --all-targets -- -D warnings` while iterating. When claiming the fix complete, commit and let `.githooks/pre-commit` run `scripts/dod.sh`; see `verification-before-completion`.
4. **If 3+ Fixes Failed**, question architecture. STOP and discuss with the human.

## Investigation Checklist

| Symptom | First Check |
|---------|-------------|
| Compile error after refactor | Grep for old paths, check `pub use` re-exports and `mod` declarations |
| Clippy denial | Read the lint name; fix the code, never `#[allow]`. `#[expect(lint, reason = "...")]` only when the lint is provably wrong at that site |
| Test hangs or is flaky | Look for shared state between tests (a static, a fixed temp path, a port); every test gets its own scratch dir |
| `cargo deny` advisory | `cargo tree -i <crate>` to find who pulls it; upgrade the parent. A new `ignore` entry is a defect |
| GPUI view does not update | The mutation happened on a different entity than the one that called `cx.notify()`, or `notify` was never called |
| GPUI panic in `render` | State mutated or an entity read re-entrantly inside `render`; move the mutation to an event handler or a task |
| `rustfmt` diff in the hook | Run `cargo fmt --all` and re-stage |

## Red Flags, STOP and Follow Process

Any of the following thoughts is a STOP signal:
- "Quick fix for now, investigate later"
- "Just try changing X and see if it works"
- "It's probably X, let me fix that"
- **"One more fix attempt" (when already tried 2+)**

**ALL of these mean: STOP. Return to Step 1.**
