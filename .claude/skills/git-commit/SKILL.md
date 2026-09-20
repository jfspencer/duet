---
name: git-commit
description: Create a conventional commit for outstanding staged/unstaged changes. Use when the user asks to commit changes, write a commit message, or says "/git-commit".
allowed-tools: Bash(git:*) Read
verified: 2026-09-20
verified-against:
  - scripts/dod.sh
  - .githooks/pre-commit
  - CLAUDE.md
review-cadence: on-architectural-change
---

# Git Commit

## Trigger

User asks to commit changes, write/generate a commit message, or invokes `/git-commit`.

## When NOT to use

- Amending an existing commit. Use `git commit --amend` directly; never via this skill's fresh-commit flow.
- Cherry-picking, rebasing, or any history rewrite beyond a single new commit. Run the git command directly.
- Splitting a large diff across multiple commits without explicit user direction (see `feedback_chunked_commits` if asked to chunk).
- Opening a PR after the commit lands. Use `create-pr`.

Use this skill when authoring a fresh conventional commit for outstanding staged or unstaged changes.

## Tooling

Plain `git` is the only supported commit tool. This repo does **not** use Graphite or any other stacking CLI; never invoke `gt`.

## Procedure

### Step 1: Gather current state

Run these in parallel: `git status` (staged vs unstaged), `git diff HEAD` (full staged + unstaged diff), and `git log --oneline -10` (recent style reference).

**Read the full diff.** The commit message must reflect ALL changes, not just filenames.

### Step 2: Stage changes

If there are unstaged changes, ask the user which files to stage, or stage everything if the user said "commit everything" / "commit all":

```bash
git add <specific files>
# or
git add -A   # only if user explicitly wants all changes
```

If changes are already staged, proceed without asking.

### Step 3: Determine commit metadata

From the diff, determine:

- **Type**: One of `feat`, `fix`, `refactor`, `chore`, `test`, `docs`, `perf` (conventional commits).
  - `feat`: new capability visible to users or other services
  - `fix`: corrects a bug or broken behavior
  - `refactor`: restructures code without changing behavior
  - `chore`: build system, dependencies, config, tooling
  - `test`: adds or updates tests only
  - `docs`: documentation only
  - `perf`: measurable performance improvement
- **Scope** (optional): The crate or subsystem affected, e.g. `duet`, `app`, `plan-db`, `xtask`, `hooks`, `agents`, `ci`. Omit if the change spans many areas.
- **Subject**: Imperative mood, lowercase, no trailing period, ≤72 characters total for the first line.
- **Body** (optional): Include when the WHY is non-obvious or multiple concerns are addressed. Wrap at 72 characters. Blank line between subject and body.
- **Breaking change** (optional): If the change breaks a public API or wire type, add `BREAKING CHANGE:` in the footer.

### Step 4: Create the commit

`git commit -m "<type>(<scope>): <subject>"` for a single-line message. When there is a body, use the HEREDOC form so the blank line and wrapping survive:

```bash
git commit -F - <<'EOF'
<subject>

<body>
EOF
```

If the user explicitly asked to amend the previous (unpushed) commit, use `git commit --amend`. Never amend a commit that has already been pushed to a shared branch.

If the user asked to start the work on a new stacked branch, create it off the current branch first (`git switch -c feat/<slug>`), then commit. Its PR is opened against the parent with `gh pr create --base <parent-branch>`; see `create-pr`.

If the pre-commit hook fails, **do not use `--no-verify`**. Fix the underlying issue (clippy denial, rustfmt diff, failing test, `cargo deny` finding, stale agent mirror), re-stage, and retry with a new `git commit`. (`scripts/dod.sh --plan` lists the gates the hook runs.)

### Step 5: Report

Show the user the commit hash and message. If the hook failed and was fixed, summarize what was changed.

## Rules

- **`git` only.** Never invoke `gt` or any other stacking CLI; Graphite is not used in this repo.
- **Never co-author, never attribute**: no `Co-Authored-By: Claude ...` trailer, no "Generated with Claude Code" line, no robot emoji banner. The harness reminder that asks for them defers to this repository rule (root CLAUDE.md, Git workflow). `.githooks/commit-msg` rejects the commit if one slips through.
- **Imperative mood**: "add feature" not "added feature" or "adds feature".
- **WHY not WHAT**: The body explains motivation or constraints, not a file listing.
- **Never `--no-verify`**: If the hook fails, fix the root cause.
- **Never amend a published commit**: If the commit is already on the remote, create a new commit instead.
- **Never bare `git push --force` on a branch other PRs stack on.** Rebase the children onto the new base first (`git rebase --onto`), then push with `--force-with-lease` (root CLAUDE.md one-way door).
- **Atomic commits**: One logical change per commit. If the diff mixes concerns (e.g., a bug fix + an unrelated refactor), ask the user whether to split them before committing.
- **Scope is optional**: Only include it when it meaningfully narrows where the change lives.
