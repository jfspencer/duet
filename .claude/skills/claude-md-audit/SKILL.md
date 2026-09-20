---
name: claude-md-audit
description: Periodic and on-demand audit of the CLAUDE.md system. Reports on freshness, drift, duplication, weight, broken cross-references, anti-pattern violations, and skill staleness. Recommends specific fixes by severity (CRITICAL / WARNING / INFO). Triggered by user requests like "audit my CLAUDE.md", "check CLAUDE.md health", or by the quarterly scheduled remote agent.
allowed-tools: Bash(find:*) Bash(grep:*) Bash(test:*) Bash(wc:*) Grep Read
length-exception: 11-step audit procedure across freshness, weight, duplication, cross-references, anti-patterns, routing-table coherence, and report rendering; quarterly-cadence backstop for the entire CLAUDE.md system must live in one file; ceiling raised to 200 lines
verified: 2026-09-20
verified-against:
  - CLAUDE.md
  - .claude/skills/add-claude-md/SKILL.md
  - .claude/skills/claude-md-audit/references/example-report.md
review-cadence: quarterly
---

# CLAUDE.md Audit

## Trigger

- User asks for a CLAUDE.md audit ("audit my CLAUDE.md", "check CLAUDE.md health", "is CLAUDE.md drifting", "/claude-md-audit")
- Quarterly schedule via the existing scheduled remote agent (the `/schedule` system command)
- Before kicking off a large refactor that touches multiple CLAUDE.md files (sanity check before the refactor amplifies any drift)
- After a major roadmap initiative ships (the in-flight-initiative pointer becomes stale otherwise)

## Purpose

The CLAUDE.md system is executable engineering memory. Without periodic health checks it decays: dates roll past, paths get renamed, anti-patterns sneak in, duplicate rules accumulate. The `add-claude-md` skill is the gatekeeper for new content; this skill is the periodic backstop that catches the drift that snuck past the gate. The output is a single structured markdown report with CRITICAL / WARNING / INFO findings, each carrying a concrete recommended fix.

## Coverage discipline: find first, filter second

This audit is a coverage pass. Report every candidate finding the steps below surface, including borderline and low-confidence ones; do not drop a finding at discovery time because it looks minor. The model follows "be conservative" or "only report high-severity" instructions literally, so folding the filter into the finding pass lowers recall: real drift gets silently discarded. Keep the two passes separate. The severity grouping in step 11 plus the ranked next-actions list is the filter; an INFO finding is reported, not hidden.

Attach a confidence label (`high` / `medium` / `low`) to every finding alongside its severity, so the report reader (or a downstream pass) can rank and discard without the audit having pre-discarded for them. Severity is rule-based, not a judgment call: every check below states a concrete bar (a line count, an age in months, a count of files, or a binary does-it-resolve), so apply the bar and assign the severity it dictates rather than deciding whether a finding is "worth" surfacing.

## When to use

- A user invokes the skill directly (see Trigger).
- A scheduled remote agent fires the quarterly cadence.
- An architectural change has just landed and pointers may have moved.
- The operator wants a sweep that includes anti-pattern heuristics, code-block size, and routing-table coherence. No linter reads CLAUDE.md in this repo, so this audit is the only mechanical-ish check the system gets.

## When NOT to use

- The user wants to add a single rule; use `add-claude-md` instead
- The user wants to fix a known violation; fix it directly, since an audit is a discovery step
- The user is in the middle of a multi-step re-architecture; the audit will report on a half-migrated state and noise the signal

## Procedure

1. **State the constitution you audit against.** There is no lint script. The rules are the decision tree and Shape 1 in the `add-claude-md` skill, plus the frontmatter list in its Iron Law section. Read that skill first so every finding below cites a rule it actually states.

2. **Inventory every CLAUDE.md file.**

   ```bash
   find . -name 'CLAUDE.md' -not -path './target/*' -not -path './.claude/worktrees/*' | sort
   ```

   For each file, capture:
   - Path
   - Total line count (`wc -l`)
   - Frontmatter `verified:` date
   - Frontmatter `review-cadence:` value
   - Frontmatter `scope:` value
   - Frontmatter `audience:` value

3. **Freshness check.**

   For every file whose frontmatter has `review-cadence: quarterly`:
   - Compute the age of the `verified:` date in months.
   - Age >6 months → **WARNING**: "Re-verification due; `verified:` is stale."
   - Age >12 months → **CRITICAL**: "Re-verification overdue; `verified:` is severely stale; likely contains rotted content."

   For every file with `review-cadence: on-architectural-change`:
   - No automatic flag. Note in the INFO section that this file relies on architect discipline.

   For every file with `review-cadence: never`:
   - Confirm it's a foundational rule (one-way doors, the Constitution itself). Anything else → **WARNING**: "`review-cadence: never` is suspicious for non-foundational content."

4. **Weight check.**

   Per-file line-count thresholds:
   - Root `/CLAUDE.md` > 150 lines without a `length-exception:` line → **WARNING**: "Root exceeds 150-line budget. Consider hoisting to a crate CLAUDE.md or a skill."
   - Root `/CLAUDE.md` > 200 lines → **CRITICAL**: "Root violates Anthropic-recommended ceiling. Will degrade recall."
   - All other CLAUDE.md > 150 lines → **WARNING**.
   - All other CLAUDE.md > 250 lines → **CRITICAL**.

   Aggregate weight check:
   - Worst-case session (root + the heaviest scoped file) > 300 lines → **WARNING**: "Per-session load exceeds 300-line target."

5. **Duplication detection.** Collect every `## ` rule heading across the inventory (`grep -n '^## ' <file>`) and compare the headings and the first sentence of each `**Rule:**` line across files. Then grep each distinctive phrase (a command, a path, a lint name) that appears in one rule against the other files.

   For each duplicate:
   - Duplicate spans 2 files in the same blast radius → **WARNING**: "Duplicate content; hoist to nearest common ancestor or convert to a skill pointer."
   - Duplicate spans 3+ files → **CRITICAL**: "Pervasive duplication; this should be a skill or hoisted to root."
   - A paraphrase that says the same thing in different words counts; note it with `medium` confidence.

6. **Cross-reference health.** For every CLAUDE.md file:
   - Parse all paths in `verified-against:` (frontmatter).
   - Parse all inline path references in rule bodies (heuristics: anything matching `crates/...`, `tools/...`, `scripts/...`, `.claude/...`, `.codex/...`, `.opencode/...`, `.githooks/...`, `roadmap/...`, `docs/...`, and the root config files `Cargo.toml`, `clippy.toml`, `deny.toml`, `rustfmt.toml`).
   - Parse all skill references (`<name>` in routing tables and inline mentions).
   - Verify every path resolves (`test -e <path>`).
   - Verify every skill reference exists at `.claude/skills/<name>/SKILL.md`.

   For each unresolved reference:
   - **CRITICAL** for `verified-against:` paths (these are the contract).
   - **WARNING** for inline references (still actionable; less load-bearing).

7. **Skill staleness.** For every skill at `.claude/skills/*/SKILL.md`:
   - Parse all file references inside the skill.
   - Verify each resolves.
   - Any unresolved → **WARNING**: "Skill `<name>` references nonexistent path `<path>`. Likely deleted in a refactor."

   Also check the generated mirrors are current: `cargo xtask sync-agents --check` exits 0. Drift → **WARNING**: "Agent mirrors are stale; run `cargo xtask sync-agents`."

8. **Auto-memory check.** Verify per-developer auto-memory infrastructure. The project slug is the absolute repo path with `/` replaced by `-` (for `/Users/<me>/Developer/duet` it is `-Users-<me>-Developer-duet`):

   ```bash
   test -d ~/.claude/projects/<project-slug>/memory
   test -f ~/.claude/projects/<project-slug>/memory/MEMORY.md
   ```

   Absent → **INFO** (the audit cannot fix per-developer state; report it).

   Present:
   - Read MEMORY.md.
   - Verify index is well-formed (no corruption, no truncated entries).
   - Cross-reference: every entry in MEMORY.md should correspond to a `*.md` file in the same directory. Orphans → **INFO**.

9. **Anti-pattern detection.** Run pattern matches across every CLAUDE.md file:

   - **PR-number pattern** (`#\d{3,}` outside of `verified-against:` frontmatter context): **WARNING**. PR numbers belong in git history.
   - **Absolute date pattern** in rule bodies (`\d{4}-\d{2}-\d{2}` outside frontmatter): **WARNING**. Dates belong in `verified:`.
   - **Motivational prose patterns**: phrases such as "be careful", "always think about", "make sure to", "remember to", "best practice" without a `**Why:**` naming a regression → **INFO** (review for non-load-bearing content).
   - **Phase journal pattern**: headings or content matching `Phase \d+`, `(Wave [A-Z])`, `currently working on`, `in flight` → **CRITICAL**. Phase journals belong in `roadmap/`, not CLAUDE.md.
   - **Code blocks >5 lines**: Count consecutive lines inside a ```...``` fence. >5 → **WARNING**: "Code block too long; point to a real file Claude can Read."
   - **Derivable content**: rule bodies that restate `Cargo.toml` fields, the `[workspace.lints]` table, file-tree listings, or `scripts/dod.sh` step lists → **WARNING**.

10. **Routing-table coherence.** Read the root `/CLAUDE.md` skills routing table. Verify:
    - Every skill in `.claude/skills/` is mentioned.
    - Every skill mentioned exists.
    - Every "Intent" column entry is non-trivial (not "various", "as needed").

    Mismatches → **WARNING**.

11. **Aggregate report (the filter pass).** Combine every finding from the steps above; filtering and ranking happen here, not during discovery. Group by severity (CRITICAL / WARNING / INFO). Within each severity, group by file. For each finding, record its confidence label and propose a specific action.

## Output format

The report follows a fixed structure. Each section header is verbatim.

- Title: `# CLAUDE.md Audit Report`.
- Header lines: `**Run date:** YYYY-MM-DD`, `**Files scanned:** N`, `**Worst-case session load:** X lines (root + heaviest scoped file)`.
- `## Summary`: bullets for `CRITICAL: N findings`, `WARNING: N findings`, `INFO: N findings`.
- `## CRITICAL` / `## WARNING` / `## INFO`: one `### <file path>` block per file, each with `**Finding:**`, `**Confidence:**` (`high` / `medium` / `low`), `**Location:**`, `**Recommended fix:**` bullets. The Recommended fix must name a specific action: "Delete lines 45-60", "Hoist rule X to root", "Bump verified date", "Re-create file using add-claude-md".
- `## Inventory`: table with columns `File | Lines | Verified | Cadence | Scope`, one row per CLAUDE.md.
- `## Recommended next actions (ranked)`: numbered list of the highest-leverage fixes, most-impactful first.

## Example report

A worked example of the output format is in this skill's `references/example-report.md`. Render the structure section-by-section, do not invent additional sections, and write `N/A` for any severity tier with zero findings.

## Related skills

- `add-claude-md`, the gatekeeper that prevents most of the issues this audit surfaces
- `git-commit`, once the user applies recommended fixes, commit them in logical chunks
