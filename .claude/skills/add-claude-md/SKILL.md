---
name: add-claude-md
description: Use when adding any rule to any CLAUDE.md file. Walks through the Constitution decision tree, prevents rule duplication, enforces Shape 1 (Hard Constraint) with Why/How to apply/Enforced by. Triggered by user requests like "add to CLAUDE.md", "remember this", "make this a rule", and whenever Claude is about to edit any CLAUDE.md file.
allowed-tools: Bash(grep:*) Bash(find:*) Grep Read Edit Write
length-exception: walks the Constitution decision tree with four worked examples (A/B/C/D) plus full Shape 1 drafting flow; the canonical entry point for every CLAUDE.md edit so the procedure must live in one file; ceiling raised to 240 lines
verified: 2026-09-20
verified-against:
  - CLAUDE.md
  - Cargo.toml
  - clippy.toml
  - .claude/skills/claude-md-audit/SKILL.md
review-cadence: on-architectural-change
---

# Add to CLAUDE.md

## Trigger

The user asks to add or change a rule in a CLAUDE.md file, or Claude is about to invoke `Edit` / `Write` on any CLAUDE.md file. Every such moment routes through the procedure below, including when the asker is the file owner or the change is "tiny."

## The Iron Law

```
EVERY ADDITION TO ANY CLAUDE.md FILE GOES THROUGH THE CONSTITUTION DECISION TREE.
NO EXCEPTIONS. NOT FOR THE OWNER, NOT FOR "QUICK NOTES," NOT FOR PHASE LOGS.
```

The Constitution is the decision tree in this skill (Q1 to Q4 below) plus Shape 1. No linter backstops it: nothing in this repo reads a CLAUDE.md file for register, length, duplication, or cross-references. The checks are manual, and the `claude-md-audit` skill is the periodic sweep that catches what slipped past this procedure.

Frontmatter every CLAUDE.md carries: `scope:`, `audience:` (`every-session` for root, `inside-this-directory` for everything else), `verified:` (an ISO date), `verified-against:` (a list of paths, lints, or tests the rules were checked against), `review-cadence:` (`quarterly` | `on-architectural-change` | `never`). A file over 150 lines carries a `length-exception:` line that says why.

## When to use

- User asks to add a rule to a CLAUDE.md file ("add to CLAUDE.md", "remember this", "make this a rule", "we should never do X again")
- User asks to update an existing CLAUDE.md rule
- Claude is about to invoke `Edit` or `Write` on any path matching `*CLAUDE.md`

## When NOT to use

- The rule is deterministically enforceable (a clippy lint, a `clippy.toml` ban, a `deny.toml` rule, a hook, a test); those edits do not touch CLAUDE.md. Use `failure-mode-author`.
- The user is updating a non-rule field (e.g., bumping `verified:` date after a routine review). Use the `claude-md-audit` skill instead.
- The user is asking how to do something procedural (e.g., "how do I write an integration test"); that's a skill addition, not a CLAUDE.md addition. Redirect to the appropriate skill (`test-author`, `gpui-kit`, `rust-expertise`); if none exists, hand-author a new SKILL.md by mirroring the format of an existing skill under `.claude/skills/`.

## Procedure

1. **Capture the rule in one sentence.** Ask the user (or, if internally triggered, formulate it directly): "Describe the rule you want to add in one sentence." The sentence must use "must" or "must not". If the user gives a paragraph, distill to one sentence and confirm before continuing.

2. **Q1 (Is this enforceable by tooling?).** Run the Constitution Q1 check:

   > Is this deterministically enforceable by clippy (a lint level in `[workspace.lints]` or a `clippy.toml` ban), rustfmt, cargo-deny, a Claude hook, or a test?

   - **If YES** → STOP. Do not add to CLAUDE.md. Redirect:
     - Banned API, macro, or method → `clippy.toml` `disallowed-*` with a `reason`
     - Pattern clippy already names → raise that lint to `deny` in the root `Cargo.toml`
     - Dependency, license, or source policy → `deny.toml`
     - Type contract → a newtype or sealed enum, proven by a `compile_fail` doctest
     - Edit-time or Bash-time rule → a `.claude/hooks/` hook
     - Behavior contract → a unit or integration test
     Tell the user: "This is deterministically enforceable. Adding it to CLAUDE.md accepts ~80% adherence when ~100% is available. Encode it as <specific tool> instead." Offer to draft it through `failure-mode-author`.
   - **If NO** → continue to step 3.

3. **Q2 (Procedure or invariant?).**

   > Is the rule a procedure (how to do X) or an invariant (X must not happen)?

   - **If PROCEDURE** → STOP. Redirect to skill creation. Procedures belong in `.claude/skills/`, not CLAUDE.md. Tell the user: "This is a procedure, it should live in a skill so Claude can invoke it on demand. Putting procedures in CLAUDE.md wastes context on every turn even when not authoring." Offer to hand-author a new SKILL.md by mirroring an existing skill under `.claude/skills/`.
   - **If INVARIANT** → continue to step 4.

4. **Q3 (Where does the invariant's blast radius end?).**

   > Where does the invariant's blast radius end?

   Match to the target file:
   - **Entire workspace** → root `/CLAUDE.md`
   - **Single crate** → the unit directory `crates/bc_<context>/<package>/CLAUDE.md` or `tools/bc_<context>/<package>/CLAUDE.md` (beside `lang_rust/`, never inside it)
   - **Dense invariants in one module tree** → `crates/bc_<context>/<package>/lang_rust/src/<module>/CLAUDE.md` (create the file when it doesn't exist, but only if there are ≥3 invariants; otherwise put it in the crate file)
   - **The agent system** (`.claude/agents`, hooks, plan store) → root `/CLAUDE.md` POINTER + the owning README or agent file
   - **Crosses many directories with no single home** → root `/CLAUDE.md` POINTER + a skill that owns the procedural detail

   State the proposed target file. Confirm with the user before proceeding.

5. **Q4 (Load-bearing check).**

   > Is the rule load-bearing? Would violation cause a real regression, security issue, data loss, or break a contract?

   Require a concrete answer: "Yes, because <past regression / specific constraint>" or "No, this is general best practice."

   - **If NO** → STOP. Reject the addition. Tell the user: "Generic engineering advice is noise. The model operates this way by default. Adding it consumes tokens on every turn without changing behavior. To preserve a personal preference, use auto-memory at `~/.claude/projects/<project>/memory/` instead; it persists per-developer and doesn't bloat the team's CLAUDE.md."
   - **If YES** → capture the Why (the past regression, dated PR, or load-bearing constraint). This becomes the `**Why:**` clause in Shape 1.

6. **Duplication check.** Grep across all CLAUDE.md files in the repo for similar content. The discovery command is the same one used by `claude-md-audit` (`find . -name 'CLAUDE.md' -not -path './target/*' -not -path './.claude/worktrees/*'`), piped through `xargs grep -l -i '<keyword from the rule>'`.

   Read every match. Ask:
   - Does a rule like this already exist? When yes:
     - When the existing rule is in the same target file → STOP. Edit the existing rule instead of duplicating.
     - When the existing rule is in a different file but applies to the same blast radius → STOP. Hoist or unify.
     - When the existing rule is narrower (e.g., applies only to one app, new rule applies to whole repo) → hoist the existing rule UP to the new target file, delete the narrower one.
     - When the existing rule is broader (e.g., applies to whole repo, new rule is a specific instance) → reference the broader rule by pointer instead of restating.

   Continue only when no overlap exists or the resolution is "hoist."

7. **Draft in Shape 1 (Hard Constraint).** Draft the rule using Shape 1 verbatim. The four required markers are `**Rule:**`, `**Why:**`, `**How to apply:**`, `**Enforced by:**`, each on its own line under an `## <terse rule name>` heading. The root `CLAUDE.md` carries worked instances; mirror their register.

   State the rule's scope explicitly in the `**Rule:**` line: name the class of files or call sites it governs (e.g. "every view under `crates/bc_app/duet/lang_rust/src/**`"), not a single example. The model follows instructions literally and will not widen a rule from one instance to its class, so a rule phrased around a lone example is read as binding only on that example.

   When the rule cannot be assigned an Enforced-by, that is a yellow flag. Most invariants have at least an advisory backstop: a test that exercises the happy path, a skill that documents the procedure, or a clippy lint that catches the common shape. When genuinely none exists, write `advisory` and flag for `claude-md-audit` to revisit.

   Show the draft to the user for approval before editing.

8. **Frontmatter freshness check.** Read the target CLAUDE.md's frontmatter. Verify it has:

   - `scope:`, present and correct
   - `audience:`, present (`every-session` for root, `inside-this-directory` for everything else)
   - `verified:`, will be updated in step 12
   - `verified-against:`, list of paths/rules/tests. Confirm the new rule's `Enforced by:` is captured here. Add when missing.
   - `review-cadence:`, present (`quarterly` | `on-architectural-change` | `never`)

   When the target file is brand new (step 4 created it), draft full frontmatter following the list in The Iron Law above.

9. **Manual review pass.** No linter exists; read the draft against these checks yourself:
   - Duplicate content (step 6) → return to step 6
   - Frontmatter fields missing or malformed → fix step 8 before continuing
   - Anti-patterns: a PR number, an absolute date in a rule body, motivational prose without a load-bearing Why, a code block longer than 5 lines, content derivable from `Cargo.toml` or a file listing → revise the draft
   - Register: ASD-STE100 Simplified Technical English (the `simplified-technical-english` skill)

   Do not proceed until every check is clean.

10. **Make the edit.** Use the `Edit` tool to insert the new rule in the correct section of the target file. Insertion order matters: group related rules together. Read the surrounding sections before choosing the insertion point.

    For brand-new files (rare; only when step 4 determined a new subsystem CLAUDE.md is needed), use `Write` with the full frontmatter + the new rule.

11. **Re-read the file.** Read the whole target file after the edit and repeat the step 9 checks on the result, not only on the draft. Fix every finding. Do not use `--no-verify` on commit; the native git hook runs `scripts/dod.sh`, which does not read CLAUDE.md, so this re-read is the only check the rule gets.

12. **Bump verified date.** Update the target file's frontmatter `verified:` field to today's date. Then verify by reading the file.

    When the rule added a new `Enforced by:` path that didn't exist in `verified-against:`, ensure step 8 captured it. Double-check.

## Examples

### Example A: Deterministic enforcement (REJECT, redirect to tooling)

> User: "Add a rule to CLAUDE.md: no `println!` in committed code."

**Procedure trace:**
- Step 1: Rule: "Committed code must not use `println!` or `eprintln!`."
- Step 2 (Q1): Enforceable by a clippy lint? YES (already enforced: `clippy::print_stdout` and `clippy::print_stderr` are `deny` in the root `Cargo.toml`, with `allow-print-in-tests = true` in `clippy.toml`).
- **STOP at step 2.** Tell the user: "Clippy already enforces this and `-D warnings` makes it a build error. Adding to CLAUDE.md would be a duplicate and accept ~80% adherence when the compiler already gives ~100%. No action needed."

### Example B: Procedure masquerading as invariant (REJECT, redirect to skill)

> User: "Add a rule to CLAUDE.md: when adding an integration test, wrap it in a `#[cfg(test)] mod tests`, give every test its own scratch directory, and clean it up."

**Procedure trace:**
- Step 1: Rule: "New integration tests must follow the scratch-directory pattern."
- Step 2 (Q1): The module wrapper is already enforced (`clippy::tests_outside_test_module` is `deny`). The scratch-directory discipline is not deterministically expressible. Mostly NO.
- Step 3 (Q2): This is a procedure ("how to add"), not an invariant ("X must not happen"). PROCEDURE.
- **STOP at step 3.** Tell the user: "This is a procedure, it belongs in a skill so it loads only when authoring a test. The `test-author` skill owns this. The root `CLAUDE.md` routing table already references the skill; do not duplicate the procedure."

### Example C: Motivational prose (REJECT, not load-bearing)

> User: "Add to CLAUDE.md: be careful and thoughtful when refactoring code."

**Procedure trace:**
- Step 1: Rule: "Refactors must be careful and thoughtful."
- Step 2 (Q1): Not enforceable by tooling.
- Step 3 (Q2): Not a procedure, not a falsifiable invariant. Borderline.
- Step 5 (Q4): Load-bearing? What past regression motivates it? User says "no specific one, it's good practice."
- **STOP at step 5.** Tell the user: "The model operates carefully by default; this rule consumes tokens every turn without changing behavior. When a specific class of refactor caused a regression (e.g., a `pub use` re-export removed without a workspace-wide grep), capture THAT specific rule with the regression as the Why. Otherwise this belongs in auto-memory or nowhere."

### Example D: Genuine load-bearing invariant (ACCEPT, write Shape 1)

> User: "Add a rule: every plan-dir argument to the plan store must be a repo-root-relative roadmap path, never `.`, so two actors in different worktrees resolve the same LMDB env."

**Procedure trace:**
- Step 1: Rule: "Every caller of `.claude/plan-coordination/db.sh` must pass a repo-root-relative plan dir (`roadmap/<plan>`), never `.` or an absolute worktree path."
- Step 2 (Q1): Could a tool enforce? `plan-db` rejects the repo root and paths outside it, so the two worst shapes fail closed. A worktree-local `.` still resolves to a different store than the main checkout's `.`, and no test can see which cwd an agent will type from. Not 100% deterministic.
- Step 3 (Q2): Invariant (X must not happen: "two actors on one plan silently use two stores").
- Step 4 (Q3): Blast radius = the agent system (every Hypervisor, Orchestrator, hook, and companion). Root `/CLAUDE.md` POINTER + the owning README `.claude/plan-coordination/README.md`.
- Step 5 (Q4): Load-bearing? YES. The store is the fleet's only coordination substrate; a split store means two orchestrators never see each other's work items and both dispatch the same slice.
- Step 6: Grep existing CLAUDE.md and the plan-coordination README → the README's plan-key section already states it. The canonical home is the README; root gets a one-line pointer if it lacks one.
- Step 7: Draft Shape 1. Why = the split-store failure; How = always `roadmap/<plan>`; Enforced by = `plan-db`'s root/outside rejection (`tools/bc_plan_store/plan-db/lang_rust/src/main.rs` `plan_key`) + `tools/bc_plan_store/plan-db/lang_rust/tests/roundtrip.rs` (advisory for the `.`-from-a-worktree case).
- Steps 8-12: Proceed normally.
- **Result:** Rule stays in `.claude/plan-coordination/README.md`, root `CLAUDE.md` carries the pointer. `verified:` bumped to today.

## Output format

When the procedure completes successfully, report to the user:

```
Added rule "<rule name>" to <target file>.
- Shape: Hard Constraint (Shape 1)
- Enforced by: <enforcement mechanism>
- Frontmatter verified: <today>
- Manual review (step 9 and 11): clean
```

When the procedure rejects an addition (Steps 2, 3, 5), report:

```
Did not add to CLAUDE.md. Reason: <Q2 deterministic / Q3 procedure / Q5 not load-bearing>.
Recommended path forward: <specific redirect: clippy lint or ban, deny.toml rule, test, skill, auto-memory, etc.>
```

## Related skills

- `claude-md-audit`, periodic health check that catches drift past this skill
- `failure-mode-author`, the redirect at Step 2 for anything a tool can enforce
- `test-author`, `gpui-kit`, `rust-expertise` own procedural detail; this skill redirects to them at Step 3
- `git-commit`, when the user is ready to commit the CLAUDE.md change
