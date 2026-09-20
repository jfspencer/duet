---
name: resolve-pr-comments
description: Use when the user wants to address, fix, and resolve PR review comments (bot or human reviewers). Fetches comments, plans fixes, implements them, runs verification, replies, and resolves threads.
allowed-tools: Bash(gh:*) Bash(git:*) Bash(cargo:*) Bash(grep:*) Grep Read Edit
length-exception: six-step PR-review workflow with GitHub API calls and GraphQL resolution must remain in one file because reviewers and CI both reference it; ceiling raised to 180 lines
disable-model-invocation: true
verified: 2026-09-20
verified-against:
  - .github/PULL_REQUEST_TEMPLATE.md
  - scripts/dod.sh
review-cadence: on-architectural-change
---

# Resolve PR Review Comments

## Overview

Systematic workflow for addressing PR review comments. Fetches unresolved comments, researches each one, plans and implements fixes, verifies the build, replies to each comment, and resolves the threads.

## When to Use

- User says "address PR comments", "fix review comments", "resolve review comments"
- After a PR review with actionable feedback
- When a review bot or a human reviewer leaves suggestions

## When NOT to use

- Drive-by code review of someone else's PR. Use the `Engineering Critic` agent for that workflow.
- Addressing your own self-review notes before submission. Edit the diff and amend via `git-commit`.
- Opening a new PR. Use `create-pr`.
- Verifying that all comment-driven fixes pass the DoD. Use `verification-before-completion` after the fixes land.

## The Workflow

### Step 1: Discovery

1. **Find the PR** for the current branch:
   ```bash
   gh pr list --head "$(git branch --show-current)" --json number,title,url
   ```

2. **Fetch review comments** (both inline and top-level):
   ```bash
   # Inline review comments
   gh api repos/{owner}/{repo}/pulls/{number}/comments --paginate

   # Top-level review bodies
   gh api repos/{owner}/{repo}/pulls/{number}/reviews --paginate
   ```

3. **Filter to actionable comments.** Focus on:
   - Human reviewer comments with requested changes
   - Review-bot comments that name a concrete defect (any `*[bot]` login, e.g. `github-advanced-security[bot]`)
   - Skip: approval-only reviews, bot comments that are purely informational

4. **If no actionable comments exist**, report that and stop.

### Step 2: Research and Plan

**Enter plan mode.** For each comment:

1. **Read the affected file** at the referenced lines
2. **Understand the concern**: is it a bug, style issue, performance problem, or architecture suggestion?
3. **Evaluate validity**, since not every bot suggestion is correct. Assess whether:
   - The concern is real and applies to this codebase
   - The suggested fix is the right approach (or if a better one exists)
   - The fix aligns with existing codebase patterns
4. **Search for related patterns** in the codebase (e.g., how similar concerns are handled elsewhere)
5. **Design the fix** with minimal, targeted changes

Write the plan covering all comments, then exit plan mode for approval.

### Step 3: Implementation

For each comment, apply the fix:

- Make targeted edits to the affected files
- Follow existing codebase patterns (search before inventing)
- Do not introduce unrelated changes

### Step 4: Verification

Commit the fixes (Step 6) and let `.githooks/pre-commit` run `scripts/dod.sh`. Every gate must pass. If one fails, run the one targeted command it names (`cargo nextest run -p <crate> <filter>`, `cargo clippy -p <crate> --all-targets -- -D warnings`), fix, and commit again. Do not hand-run the whole gate first.

### Step 5: Reply and Resolve

1. **Reply to each comment** explaining what was fixed:
   ```bash
   gh api repos/{owner}/{repo}/pulls/{number}/comments \
     -f body="Fixed. {brief explanation of change}" \
     -F in_reply_to={comment_id}
   ```

2. **Resolve each review thread** via GraphQL:
   ```bash
   # First, get thread IDs
   gh api graphql -f query='{
     repository(owner: "{owner}", name: "{repo}") {
       pullRequest(number: {number}) {
         reviewThreads(first: 50) {
           nodes {
             id
             isResolved
             comments(first: 1) {
               nodes { body, author { login } }
             }
           }
         }
       }
     }
   }'

   # Then resolve each unresolved thread that was addressed
   gh api graphql -f query='mutation {
     resolveReviewThread(input: {threadId: "{thread_id}"}) {
       thread { isResolved }
     }
   }'
   ```

3. **Only resolve threads you actually fixed.** If a comment was invalid or deferred, reply explaining why but leave it unresolved for the reviewer.

### Step 6: Commit and Push

Use plain `git`; this repo has no stacking CLI (never invoke `gt`).

Ask the user if they want to commit and push. If yes:

1. **Commit** with a descriptive message:
   ```
   fix: address PR review feedback

   - {brief description of fix 1}
   - {brief description of fix 2}
   ```

   `git commit -m "<message>"`. Add a new commit rather than amending, because review threads anchor to commits and amending a pushed commit orphans them. Use `git commit --amend` only if the user explicitly asked to amend a prior *unpushed* commit.

2. **Push** to the remote branch:

   ```bash
   git push
   ```

   If this branch has other PRs stacked on top of it, rebase those children onto the updated branch (`git rebase --onto`) before they are pushed, so the fixes don't reappear as duplicates in their diffs.

3. **Re-request review** if the PR had prior approvals that may be invalidated:
   ```bash
   gh pr edit {number} --add-reviewer {reviewer_login}
   ```

## Decision Framework

| Comment Type | Action |
|---|---|
| Valid bug (memory leak, race condition, etc.) | Fix it |
| Valid consistency issue (naming, patterns) | Fix it |
| Style preference with no codebase precedent | Reply explaining current convention, ask reviewer |
| Incorrect suggestion (bot hallucination, wrong context) | Reply explaining why it doesn't apply, leave unresolved |
| Architecture suggestion (large scope) | Reply acknowledging, create GitHub issue for follow-up, leave unresolved |

## Key Principles

- **Evaluate every comment independently.** Bot suggestions can be wrong. Human suggestions can be wrong. Think critically.
- **Search the codebase first.** Match existing patterns rather than blindly applying suggestions.
- **Minimal fixes.** Address exactly what the comment raises. Don't refactor surrounding code.
- **Reply concisely.** "Fixed. The scratch directory is now removed in a `Drop` impl so a failed assertion cannot leak it." is better than a paragraph.
- **Never resolve without addressing.** Either fix it, explain why it's not needed, or acknowledge it as a follow-up.
