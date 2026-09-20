# Example: CLAUDE.md Audit Report (rendered)

Sample output the audit produces. Use as a layout reference only; the audit renders this from live scans.

```markdown
# CLAUDE.md Audit Report

**Run date:** 2026-09-20
**Files scanned:** 3
**Worst-case session load:** 190 lines (root + crates/duet)

## Summary

- CRITICAL: 1 finding
- WARNING: 4 findings
- INFO: 2 findings

## CRITICAL

### crates/duet/CLAUDE.md

- **Finding:** Contains 40 lines of phase progress notes (anti-pattern: phase journal).
- **Confidence:** high
- **Location:** Lines 22-62
- **Recommended fix:** Delete lines 22-62. Hoist any durable invariants to Shape 1 entries. Progress belongs in the plan store (`fluid:*` keys), not in a CLAUDE.md.

## WARNING

### /CLAUDE.md

- **Finding:** `verified:` is 7 months old; review-cadence is quarterly.
- **Confidence:** high
- **Location:** Frontmatter
- **Recommended fix:** Re-verify content, bump `verified:` to today.

### /CLAUDE.md

- **Finding:** Rule "no literal colors in views" duplicated in the coding-style section and the design-system section.
- **Confidence:** medium
- **Location:** Lines 48-53 and lines 91-96
- **Recommended fix:** Consolidate to one entry in the design-system section. Reference from coding style if relevant.

### tools/plan-db/CLAUDE.md

- **Finding:** Code block of 14 lines in §Key classes (anti-pattern: code block > 5 lines).
- **Confidence:** high
- **Location:** Lines 30-44
- **Recommended fix:** Replace with a 1-line pointer to `.claude/agents/engineering/memory-agent.md`, which holds the authoritative keyspace.

### .claude/skills/test-author/SKILL.md

- **Finding:** References `tools/plan-db/tests/lifecycle.rs`, which was renamed to `tools/plan-db/tests/roundtrip.rs`.
- **Confidence:** high
- **Location:** Line 47
- **Recommended fix:** Replace the path.

## INFO

### ~/.claude/projects/<project-slug>/memory/MEMORY.md

- **Finding:** Auto-memory index has 11 entries; 1 orphan file (`feedback_dead_link.md`) is unindexed.
- **Confidence:** high
- **Recommended fix:** Re-index or delete the orphan.

### tools/xtask/CLAUDE.md

- **Finding:** `review-cadence: on-architectural-change`, no automatic re-verification trigger.
- **Confidence:** medium
- **Recommended fix:** No automatic action. If the mirror format changed, schedule a manual review.

## Inventory

| File | Lines | Verified | Cadence | Scope |
|---|---|---|---|---|
| /CLAUDE.md | 142 | 2026-02-15 | quarterly | workspace |
| crates/duet/CLAUDE.md | 48 | 2026-09-11 | quarterly | crates/duet |
| tools/plan-db/CLAUDE.md | 31 | 2026-09-11 | on-architectural-change | tools/plan-db |

## Recommended next actions (ranked)

1. Delete the phase-journal block in `crates/duet/CLAUDE.md` (CRITICAL, single highest-impact fix).
2. Re-verify root `/CLAUDE.md` and bump `verified:`.
3. Consolidate the duplicate literal-color rule in `/CLAUDE.md`.
4. Replace the stale `test-author` reference path.
5. Trim the oversized code block in `tools/plan-db/CLAUDE.md`.
```
