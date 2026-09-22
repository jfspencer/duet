---
id: M93
line: M
depends_on: [M0, M90]
write_scope:
  - .github/workflows/ci.yml
parallelism: serial-only: SM4 makes `.github/workflows/ci.yml` a policy file, so the Orchestrator executes this chunk and no engineer writes that file beside it.
completion: "the plan-lint job RUNS and is green on a PULL REQUEST whose base is chunk/m93-plan-lint-trigger and whose one changed path is the root Cargo.toml, which gh pr checks reports; the same shaped pull request against main reports NO plan-lint job before this chunk; cargo xtask check-plan-graph roadmap/duet-v1 --check-manifest exits 0; commit SHA on a branch chunk/m93-plan-lint-trigger"
---

# M93: The plan-lint trigger covers the four policy files that decide the PG25 oracle

Verify the current state of the file in the write scope; report a discrepancy and stop, instead of
proceeding.

This is a REPAIR chunk under rule SM9 of architecture section 13.0. It implements architecture
section 1.9 (rule PG25 and its stated inputs) and section 14 (the `plan-lint` row and the paragraph
that states the roster trigger gap). It writes one policy file and no crate source.

**Rule PG25 reads five repository inputs, and no job runs when any one of them changes.** The roster
compile copies the repository's own `[workspace.lints]` table, `.cargo/config.toml`, `clippy.toml`,
`rust-toolchain.toml` and `Cargo.lock` into the scratch workspace, then runs
`cargo clippy --workspace --all-targets -- -D warnings` over the generated roster.
`tools/xtask/src/check_roster.rs` states the list in its own crate doc, lines 6 to 10; lines 2934 to
2937 name the four files it copies; and `lint_table` at line 2767 reads the `[workspace.lints]` table
of the root `Cargo.toml` and fails closed when it is absent. A change to any one of the five changes
what PG25 decides.

**`cargo xtask check-roster` runs in exactly one place**, the `plan-lint` job of
`.github/workflows/ci.yml`, and it runs in no gate: section 14 states the reason, which is a measured
4.0 GB of build output and several minutes. The `changes` job gates `plan-lint` on a pattern that
matches `^roadmap/` and `^tools/xtask/` alone. **A commit that edits the root `Cargo.toml`,
`clippy.toml`, `rust-toolchain.toml` or `.cargo/config.toml` therefore runs no roster in the gate and
none in CI.**

**This chunk takes FOUR of the five inputs and it excludes `Cargo.lock`, which is an operator
decision with a stated cost reason.** The four are policy files that this repository edits rarely and
deliberately, so the widened trigger fires on a rare commit and costs one job. `Cargo.lock` changes
on every dependency edit, so a `Cargo.lock` trigger would put a 4.0 GB compile on every manifest
chunk and on every pin. That is the exact cost architecture section 14 names when it refuses a roster
line in the gate. The heading "What this chunk does NOT cover" below states the residual.

**The Orchestrator executes this chunk directly.** `.github/workflows/ci.yml` is a policy file under
SM4, and CLAUDE.md makes a gate edit an adjudication. No engineer may edit one.

## Files

| Path | Action |
|---|---|
| `.github/workflows/ci.yml` | modify (the `changes` job output name, the grep pattern, and the `plan-lint` condition) |

## Types and signatures

This chunk declares no Rust type and changes no Rust file. It edits one workflow file.

## Steps

1. Read `.github/workflows/ci.yml`. Confirm that the `changes` job declares the output `roadmap` at
   line 60, that the filter step holds
   `if printf '%s\n' "$changed" | grep -qE '^roadmap/|^tools/xtask/'; then`, that the two `echo`
   lines write `roadmap=true` and `roadmap=false`, and that the `plan-lint` job carries
   `if: needs.changes.outputs.roadmap == 'true'`. Report a discrepancy and stop.

2. Prove the gap before the repair, and OPEN A PULL REQUEST to do it. `.github/workflows/ci.yml`
   lines 3 to 6 trigger on `pull_request` and on `push` to `main` alone, so a push to a feature
   branch starts NO run of any kind and a `gh run list` over such a branch is an empty list that
   proves nothing. **The event this proof needs is `pull_request`.**

   **A scratch branch is OUTSIDE this chunk's write scope by construction.** `write_scope` binds
   every commit the chunk LANDS. A branch that is opened, read and closed with no merge lands
   nothing, so the whitespace commit on the root `Cargo.toml` below is not a write this chunk makes.
   The pull request closes with no merge.

   1. Branch off `main`, change one whitespace character of the root `Cargo.toml`, and commit.
   2. Open a pull request whose BASE is `main`. The `changes` job then reads
      `origin/main...HEAD`, so the changed set is the root `Cargo.toml` and nothing else.
   3. Run `gh pr checks <the pull request>` and confirm that the `plan-lint` job DID NOT RUN. **A
      job whose `if:` is false is SKIPPED and not absent**, so `gh pr checks` may list it with a
      `skipped` conclusion, or may omit it. Either reading is the expected result; a `success` or a
      `fail` conclusion is NOT. Record the output in the chunk report.
   4. Close the pull request without a merge and delete the scratch branch. **Nothing of this step
      is ever merged.**

3. Widen the grep pattern. **Edit the pattern IN PLACE and leave the line's indentation untouched.**
   Replace the single-quoted argument of `grep -qE` on the filter line with exactly this text, which
   is the whole new pattern and nothing else:

   ```
   '^roadmap/|^tools/xtask/|^Cargo\.toml$|^clippy\.toml$|^rust-toolchain\.toml$|^\.cargo/config\.toml$'
   ```

   **Do NOT retype the whole line from a rendered block.** That line carries ten leading spaces
   inside a `run: |` block scalar, a markdown reader strips a fence's own indent from the block it
   renders, and a line that reaches the file with fewer than ten spaces ends the scalar early and
   breaks the YAML. The fence above therefore holds the pattern alone, at no meaningful indent, and
   the line keeps the indentation chunk M0 gave it.

   The four new alternatives are ANCHORED at both ends. `^Cargo\.toml$` matches the root manifest and
   no member manifest, because every member manifest is `crates/<name>/Cargo.toml` or
   `tools/<name>/Cargo.toml`. `^clippy\.toml$` and `^rust-toolchain\.toml$` each match the one file
   of that name at the repository root. `^\.cargo/config\.toml$` matches the one cargo config, and
   the leading dot is escaped like every other. The dot is escaped in each alternative, so no pattern
   matches a path such as `CargoXtoml`. **`Cargo.lock` is NOT in the pattern**, and the heading
   "What this chunk does NOT cover" states why.

4. Rename the job output, because the name `roadmap` no longer states what the output means. The
   `changes` job output line becomes:

   ```yaml
      plan_inputs: ${{ steps.filter.outputs.plan_inputs }}
   ```

   The two `echo` lines become `echo "plan_inputs=true" >> "$GITHUB_OUTPUT"` and
   `echo "plan_inputs=false" >> "$GITHUB_OUTPUT"`. The `plan-lint` condition becomes
   `if: needs.changes.outputs.plan_inputs == 'true'`. **Search the whole file for
   `outputs.roadmap` and confirm that no other job reads it**; at this revision the `plan-lint` job
   is the one reader inside `ci.yml`. **One DOCUMENT reader lives outside the file**:
   `roadmap/duet-v1/m-00-manifest-phase-0.md` step 5 of its `ci.yml` block quotes the old output
   name, because M0 is the chunk of record for the `plan-lint` job. The Architect owns that file
   under SM9 and has already written the note that names this chunk and the new name; confirm that
   the note is there, and report it to the Orchestrator and stop if it is not. **Do not edit any
   file under `roadmap/`**: this chunk's write scope is `.github/workflows/ci.yml` alone.

5. Leave every other line of the `changes` job alone. The `-z` and `--no-renames` diff form, its
   comment block, the `base` fallback chain, and the `changed="roadmap/"` fallback for a repository
   with no parent commit all stay as chunk M0 wrote them. **The fallback value stays `roadmap/`**,
   which still matches the widened pattern and still means "run the job when neither ref resolves".

6. Read the file once with a YAML parser and confirm that it parses. Run
   `python3 -c "import sys,yaml;yaml.safe_load(open('.github/workflows/ci.yml'))"` if the `yaml`
   module is available; otherwise use `gh workflow view` after the push. **Do not run `bash -n` on
   this file**: it is YAML, and the repository gate runs `bash -n` over shell files alone.

7. Prove the repair, with a pull request STACKED on this chunk's own branch. **`write_scope` binds
   every commit this chunk LANDS**, so the commit on `chunk/m93-plan-lint-trigger` carries
   `.github/workflows/ci.yml` and nothing else, and a second path on that branch is a breach the
   Orchestrator returns. **A scratch branch that is opened, read and closed with no merge lands
   nothing**, so the whitespace commit on the root `Cargo.toml` below is outside the write scope by
   construction, and its pull request closes with no merge. A pull
   request whose base is `main` and whose head is `chunk/m93-plan-lint-trigger` therefore carries
   `.github/workflows/ci.yml` alone, which matches no alternative of the pattern, so that pull
   request runs no `plan-lint` job and proves nothing. The proof is a second, throwaway pull
   request.

   1. Commit the step 3 and step 4 edits on `chunk/m93-plan-lint-trigger` and push that branch.
   2. Branch off `chunk/m93-plan-lint-trigger`, change one whitespace character of the root
      `Cargo.toml`, and commit.
   3. Open a pull request whose BASE is `chunk/m93-plan-lint-trigger` and whose head is that scratch
      branch. The `changes` job reads `origin/chunk/m93-plan-lint-trigger...HEAD`, so **the changed
      set is the root `Cargo.toml` and nothing else**, and the workflow file the run uses is the one
      on the head branch, which carries the widened pattern.
   4. Run `gh pr checks <the scratch pull request>` and confirm that the `plan-lint` job RAN and
      reported a `success` conclusion. **A `skipped` conclusion is a FAILURE of this step**, because
      a skipped job is the state step 2 records before the repair. Record the run id in the chunk
      report. **`^Cargo\.toml$` is the alternative under
      test**, and it is the only one that can have matched: the changed set holds one path, and that
      path opens with neither `roadmap/` nor `tools/xtask/` and is neither `clippy.toml`,
      `rust-toolchain.toml` nor `.cargo/config.toml`.
   5. Close the scratch pull request without a merge and delete the scratch branch.

   **A run that reports no `plan-lint` job, or reports it as `skipped`, is a failure of this
   chunk**, and the chunk reports it instead of proceeding.

   **The second form of this proof was considered and refused.** It waits for the
   `chunk/m93-plan-lint-trigger` pull request to merge, then opens one scratch pull request off
   `main` whose one changed path is the root `Cargo.toml`. It is simpler to read, and CLAUDE.md
   gives the merge to the operator, so a Completion command that waits for a merge cannot close
   inside the chunk. The stacked form closes inside the chunk and reads the same alternative.

8. Run `cargo xtask check-plan-graph roadmap/duet-v1 --check-manifest` and confirm exit 0. This
   chunk writes no chunk front matter, so the generated manifest is unchanged.

9. Commit on a branch `chunk/m93-plan-lint-trigger` and let the native git hook run
   `scripts/dod.sh`.

## Tests

This chunk writes no Rust test. Its check is the run of step 7 (SM3): a `plan-lint` job that does
not run before the chunk and does run after it, over a pull request whose one changed path is the
root `Cargo.toml`. Step 2 records the negative half against base `main`, step 7 records the positive
half against base `chunk/m93-plan-lint-trigger`, and both belong in the chunk report. **The negative
half is a job that DID NOT RUN, which reads as a `skipped` conclusion or as an omission**; the
positive half is a `success` conclusion. A `skipped` conclusion on the positive half is a failure.

**Both halves are a PULL REQUEST and neither is a branch push.** `.github/workflows/ci.yml` lines 3
to 6 trigger on `pull_request` and on `push` to `main` alone, so a push to a feature branch starts no
run and an empty `gh run list` over such a branch is a green this chunk did not earn. The two
pull requests are throwaway; neither one merges.

**A local grep is NOT the check.** A grep proves that the pattern text changed; it proves nothing
about the job that reads it. The run is the oracle, and the two `gh pr checks` outputs are the
evidence.

## What this chunk does NOT cover

**`Cargo.lock` is the one PG25 input this chunk leaves out, and the reason is cost.** The roster
compile copies the lock file into its scratch workspace, so the lock fixes every external crate
version the roster builds against and a change to it can change what PG25 decides. The trigger still
leaves it out. `Cargo.lock` changes on every dependency edit: every manifest chunk writes it, and
SM5 puts it in the write scope of every chunk that adds a dependency to a member manifest. A
`Cargo.lock` alternative in the pattern would therefore put a 4.0 GB compile and several minutes on
each of those commits, which is the exact cost architecture section 14 names when it refuses a roster
line in the gate.

**The cover for the gap is review.** A commit that moves a pinned version is a commit a reviewer
reads for that reason, and `cargo deny check` runs in the gate over the same lock file. Architecture
section 14 records this limit beside the interim rule, so the gap lives in the specification and not
in a report alone. **A lock-only change, such as a bare `cargo update`, runs no roster**, and the
party that makes one runs
`cargo xtask check-roster roadmap/duet-v1/architecture.md "${TMPDIR:-/tmp}/duet-roster" .` by hand.

## Verification

```
gh pr checks <the step 2 pull request, base main>
gh pr checks <the step 7 pull request, base chunk/m93-plan-lint-trigger>
cargo xtask check-plan-graph roadmap/duet-v1 --check-manifest
cargo xtask check-placement roadmap/duet-v1/architecture.md
```

The first command shows that the `plan-lint` job DID NOT RUN, as a `skipped` conclusion or as an
omission, which is the gap before the repair. The second names a `plan-lint` job with a `success`
conclusion, over a pull request whose one changed path is the root `Cargo.toml`. Both pull requests
close without a merge, so neither scratch commit lands and neither is a write this chunk makes. The third and the fourth commands each exit
0. Then commit on a branch `chunk/m93-plan-lint-trigger`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh`
  and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command
  is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site
  `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture
  Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is
  denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice
  indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only
  inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are
  required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every
  `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the
  root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no
  pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with
  Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this
  repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match
  what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
