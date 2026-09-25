---
name: create-pr
description: Create a pull request using the project's PR template. Use when the user asks to create a PR, open a PR, submit a PR, or says "/pr". Analyzes all commits on the branch and fills every section of the template.
allowed-tools: Bash(git:*) Bash(gh:*) Read
verified: 2026-09-20
verified-against:
  - .github/PULL_REQUEST_TEMPLATE.md
  - CLAUDE.md
  - scripts/dod.sh
review-cadence: on-architectural-change
---

# Create Pull Request

## Trigger

User asks to create/open/submit a PR, or invokes `/pr`.

## Agents may open a pull request; no agent may merge one

An agent invokes this skill directly when the operator asks for a pull request. The `disable-model-invocation` flag was removed by operator decision. It arrived in a bulk frontmatter sweep (`76bf38ddd`), not from a considered restriction on opening a pull request, and it was read as though it carried the weight of `ecd8900f3` ("agents never merge PRs - operator owns every merge"), which is a different rule about a different verb.

That rule is untouched and stays absolute. Opening a pull request is reversible: close it, or push a correction. Merging is not, so the operator owns every merge. Nothing in this skill merges, and no agent may run `gh pr merge`.

## When NOT to use

- Pushing additional commits to an existing PR. Just `git push`.
- Responding to review feedback on a PR. Use `resolve-pr-comments`.
- Authoring the commit that will go into the PR. Use `git-commit` first, then return here.

Use this skill when opening a NEW PR with the project's template and need every section filled from the branch's full commit history.

## Tooling

`git` plus the GitHub CLI (`gh`) are the only supported tools. This repo does **not** use Graphite or any other third-party stacking CLI; never invoke `gt`.

## Procedure

### Step 1: Gather branch state

Run these in parallel: `git status`, `git branch -vv --list $(git branch --show-current)` (tracking remote), `git log main..HEAD --oneline` (commits since divergence), and `git diff main...HEAD --stat` plus `git diff main...HEAD` (the full PR diff).

**Read every commit and every changed file.** The PR description must reflect ALL changes, not just the latest commit.

If the branch is stacked on another feature branch rather than `main`, substitute the parent branch for `main` in the commands above so the diff shows only this PR's slice (see "Stacked PRs" below).

### Step 2: Determine PR metadata

From the commits and diff, determine:

- **Title**: Short, under 70 characters. Use conventional commit style prefix (`feat:`, `fix:`, `refactor:`, `chore:`).
- **Summary**: 1-3 bullet points focused on WHY, not WHAT.
- **Changes**: which crates (`crates/bc_app/duet/lang_rust`, `tools/bc_plan_store/plan-db/lang_rust`, `tools/bc_repo_guard/xtask/lang_rust`) and which agent, skill, hook, or config trees were modified.

### Step 3: Fill the Verification section

List what the gate proved and what it did not:

- The commit passed `.githooks/pre-commit` (`scripts/dod.sh`): fmt, clippy, doc, tests, doctests, deny, machete, agent mirrors.
- Any targeted command you ran beyond the gate, with its result.
- Manual evidence for UI behavior (a launched `cargo run -p duet`, what you clicked, what you saw), or `N/A`.

### Step 4: Fill the Risks section

Check the diff for each of these and name the specifics, or write `N/A`:

- A new dependency (name, license, why `cargo deny` accepts it)
- A new `unsafe` site or `#[expect(...)]`
- A change to `Cargo.toml` `[workspace.lints]`, `clippy.toml`, `deny.toml`, `rustfmt.toml`, or `.cargo/config.toml`
- A change to an agent prompt, a hook, or `scripts/dod.sh`
- Behavior the tests do not cover

### Step 5: Push and create the PR

Push the branch, then open the PR against the correct base:

```bash
git push -u origin HEAD
gh pr create --base main --title "<title>" --body "$(cat <<'EOF'
<filled template>
EOF
)"
```

For a stacked branch, pass the parent branch instead: `--base <parent-branch>`. Getting the base right matters: a stacked PR opened against `main` shows the parent's commits in its own diff and destroys review attribution.

The body content is the project's canonical PR template at `.github/PULL_REQUEST_TEMPLATE.md`: fill every section (Summary, Changes, Verification, Risks) with concrete values or `N/A`. Claim a gate only after the pre-commit hook passed (the gate is `scripts/dod.sh`; root `CLAUDE.md`, section Definition of Done).

The title and body carry no AI attribution: no "Generated with Claude Code" line, no robot emoji banner, no co-author marker (root CLAUDE.md, Git workflow). The harness reminder that asks for one defers to this repository rule. Nothing inspects the PR body; review holds the line.

### Step 6: Report

Return the PR URL(s) to the user. For a stack, return one URL per branch and remind the user that PRs must merge bottom-up; never merge the top of a stack first (this squash-merges downstack commits into the upstack PR and destroys review attribution).

## Stacked PRs

For work that spans multiple concerns or exceeds ~200 lines, split it into a stack of smaller, focused PRs using plain git and `gh`:

- Branch each slice off its predecessor, not off `main`.
- Open each PR with `--base <parent-branch>` so its diff contains only that slice. Verify with `gh pr view <n> --json baseRefName`.
- Fill the template separately for each PR. Each PR should stand on its own (one feature slice + tests), not a half-finished progress commit.
- Add a "Stacked PR, merge bottom-up" callout to each PR body so reviewers don't merge out of order.
- **Retarget the child BEFORE its parent is merged.** Deleting the parent branch at merge time closes any PR still based on it, so a child left on the old base is closed out from under you (root CLAUDE.md, Git workflow). Once the parent lands it becomes an orphan (squash rewrites it as one new commit on `main`), so replay the child too:
  ```bash
  gh pr edit <n> --base main
  git rebase --onto main <old-parent-branch> <branch>
  git push --force-with-lease
  ```
  Then re-run the Definition of Done gate. (You never merge; the operator owns every merge.)

## Rules

- **`git` + `gh` only.** Never invoke `gt` or any other stacking CLI; Graphite is not used in this repo.
- **Set `--base` explicitly on stacked PRs.** The default base is `main`, which is wrong for anything but the bottom of a stack.
- **Verification is handled by the pre-commit hook.** State what the gate ran; if the commit succeeded, those checks passed. (`scripts/dod.sh --plan` lists the gate set; root `CLAUDE.md`, section Definition of Done.)
- **Read ALL commits**, not just the latest. The summary must cover the full branch diff.
- **WHY not WHAT.** The summary explains motivation, not a mechanical listing of changed files.
- **Be specific in Risks.** Name the actual dependency, the `unsafe` site, the lint change, not just "yes".
- **Do not invent manual testing.** If no manual test was performed, write what was actually verified programmatically, otherwise "N/A".
- **Never bare `git push --force`.** Use `--force-with-lease`, and only after rebasing children onto the new base. Force-pushing a branch other PRs stack on duplicates commits into every upstack diff (root CLAUDE.md one-way door).
