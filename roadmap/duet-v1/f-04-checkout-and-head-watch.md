---
id: F4
line: F
depends_on: [F3, M7]
write_scope:
  - crates/duet-project/src/checkout.rs
  - crates/duet-project/src/headwatch.rs
parallelism: independent
completion: "cargo nextest run -p duet-project -E 'test(checkout)' --no-tests=fail passes; commit SHA on a branch chunk/f4-checkout-and-head-watch"
---

# F4: Checkout by manifest, the `HEAD` watch, and the external-git contract

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the checkout of architecture section 4.5 and the contract for a user who runs plain git of section 4.6. ADR 0003 clauses 4, 5, 7, 8, 9 and 12 decide the manifest lookup, the order of the steps, the `HEAD` watch, the record-pass refusal, the unsupported rewrite, and the undo-stack clear. It carries MUST story H-03 (check out an earlier commit) and the reconciliation half of A-02.

**This chunk writes no member manifest**, so it names neither `crates/duet-project/Cargo.toml` nor `Cargo.lock` in its write scope, and its Completion command carries no `--locked` flag (SM3 rule 4, SM5).

## Files

- `crates/duet-project/src/checkout.rs` — modify. Chunk F1 created the stub.
- `crates/duet-project/src/headwatch.rs` — modify.

## Types and signatures

### `duet_project::checkout` (architecture 4.5)

**A checkout of an earlier commit into the live project never touches the worktree first.** Six steps run in order.

1. Read the target tree into memory with `History::read_tree`.
2. Parse it into a candidate project state.
3. Look up every `SourceHash` in the candidate `media.manifest` against `media/`.
4. A missing hash is **not** a failure of the checkout. The region draws as absent and the core emits `CoreEvent::MissingSource { hash }`.
5. On success the core swaps the whole candidate state in one step, writes the files, and moves the branch reference.
6. A failure at step 2 changes nothing and emits `CoreEvent::CheckoutRejected { commit, error }`.

The open path refuses an object format this build cannot read: `ProjectOpen` calls `History::object_format` and returns `ProjectError::ObjectFormat` for anything but `HashKind::Sha1`.

### `duet_project::headwatch` (architecture 4.6)

The application watches `.git/HEAD` and `.git/refs`. Four rules run.

1. On an external `HEAD` move, the application stops the transport first. It then flushes any open take to disk and hashes it.
2. It then follows the rule of section 3.9. No unsaved work means reload. Unsaved work means commit to a new branch, discard, or cancel, and **commit to a new branch is the default**.
3. While a record pass runs, the application refuses every history verb that rewrites: `HistoryCheckout`, `HistoryBranch`, and any reset. The verb returns `GatewayError::RecordInProgress`.
4. A rewrite of history by an external tool while the project is open is unsupported. The application detects an unknown `HEAD` and offers a reload.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `History`, `TreeSnapshot`, `HeadState`, `HistoryError`, `ProjectError` | `duet-project` | chunks F1 and F3 |
| `ManifestEntry`, the store lookup | `duet-project` | chunk F2 |
| `CommitId`, `HashKind`, `GatewayError`, `CoreEvent`, `CommandError` | `duet-command` | `duet_command` |
| `SourceHash` | `duet-session` | `duet_session` |

`duet-project` emits no `CoreEvent` itself; it returns a result that `duet-core` turns into `CoreEvent::MissingSource` and `CoreEvent::CheckoutRejected`. This chunk therefore returns a checkout outcome value that names the missing hashes and the parse failure, and chunk I1 dispatches it.

## Steps

1. Read both files of the write scope. Confirm that chunk F1 left each one a stub, and that chunk F3 wrote the `History` trait and its gix implementation. Confirm that chunk M7 has landed. Report a discrepancy and stop.
2. Write the failing test `checkout_reads_the_tree_before_it_touches_the_worktree` in `src/checkout.rs`. Run `cargo nextest run -p duet-project -E 'test(checkout)' --no-tests=fail` and confirm that it fails to compile.
3. Write the checkout in `src/checkout.rs` as the six steps above. Step 1 and step 2 run entirely in memory. Run the test and confirm that it passes.
4. Write the failing test `checkout_rejects_a_parse_failure_and_changes_nothing`. Run it and confirm that it fails, then confirm that it passes.
5. Write the failing test `checkout_reports_a_missing_hash_and_still_succeeds`. Run it and confirm that it fails.
6. Implement step 3 and step 4: look up every `SourceHash` of the candidate manifest against `media/`, collect every missing hash into the outcome, and complete the checkout. A missing hash is never a failure. Run the test and confirm that it passes.
7. Implement step 5: swap the whole candidate state in one step, write the files through the chunk F1 atomic save, and move the branch reference.
8. Write the failing test `checkout_refuses_a_foreign_object_format`. Run it and confirm that it fails, then implement the `ProjectError::ObjectFormat` refusal on the open path and confirm that it passes.
9. Write the failing test `checkout_head_watch_stops_the_transport_first` in `src/headwatch.rs`. Run it and confirm that it fails.
10. Write the `HEAD` watch in `src/headwatch.rs`: watch `.git/HEAD` and `.git/refs`, and on an external move report the four steps of section 4.6 in order. The watcher reports an outcome value; it stops no transport itself, because `duet-core` owns the transport and `duet-project` touches no core state. Run the test and confirm that it passes.
11. Write the failing test `checkout_refuses_a_rewrite_while_a_record_pass_runs`. Run it and confirm that it fails, then implement the `GatewayError::RecordInProgress` refusal and confirm that it passes.
12. Write the failing test `checkout_detects_an_unknown_head`. Run it and confirm that it fails, then implement the unsupported-rewrite detection and confirm that it passes.
13. Run `cargo clippy -p duet-project --all-targets -- -D warnings`. Fix every finding in the code.
14. Commit on the branch `chunk/f4-checkout-and-head-watch`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end.

| Test | File | What it asserts |
|---|---|---|
| `checkout_reads_the_tree_before_it_touches_the_worktree` | `src/checkout.rs` | A `History` double that records every call shows `read_tree` before the first worktree write. Message: "the checkout reads the tree before it writes the worktree". |
| `checkout_rejects_a_parse_failure_and_changes_nothing` | `src/checkout.rs` | A tree whose `session.json` does not parse leaves every worktree file unchanged and returns a rejected outcome that carries the `CommandError`. Message: "a parse failure changes nothing". |
| `checkout_reports_a_missing_hash_and_still_succeeds` | `src/checkout.rs` | A candidate manifest that names one hash `media/` does not hold completes the checkout and reports that hash as missing. Message: "a missing hash is reported and the checkout still succeeds". |
| `checkout_swaps_the_state_in_one_step` | `src/checkout.rs` | Between the start and the end of the swap, no reader observes a mixture of the old and the new documents. Message: "the candidate state is swapped in one step". |
| `checkout_moves_the_branch_reference` | `src/checkout.rs` | After a successful checkout, `History::head` names the target commit. Message: "a successful checkout moves the branch reference". |
| `checkout_refuses_a_foreign_object_format` | `src/checkout.rs` | An open of a repository whose `object_format` is `HashKind::Sha256` returns `ProjectError::ObjectFormat`. Message: "an object format this build does not read is refused". |
| `checkout_refuses_a_rewrite_while_a_record_pass_runs` | `src/checkout.rs` | With a record pass in progress, `HistoryCheckout` and `HistoryBranch` each return `GatewayError::RecordInProgress`. The message names the verb: `"verb {verb} refused during a record pass"`. |
| `checkout_head_watch_stops_the_transport_first` | `src/headwatch.rs` | On an external `HEAD` move, the reported step order is stop the transport, flush and hash the open take, then ask the user. Message: "the HEAD watch stops the transport before it flushes". |
| `checkout_head_watch_defaults_to_commit_to_a_new_branch` | `src/headwatch.rs` | With unsaved work, the reported default answer is commit to a new branch. Message: "the default answer is commit to a new branch". |
| `checkout_head_watch_reloads_with_no_unsaved_work` | `src/headwatch.rs` | With no unsaved work, the reported answer is reload and no dialog is offered. Message: "no unsaved work means reload". |
| `checkout_detects_an_unknown_head` | `src/headwatch.rs` | A `HEAD` that names a commit the repository no longer holds reports an unknown head and offers a reload. Message: "an unknown HEAD is detected and offers a reload". |

## Verification

1. `cargo nextest run -p duet-project -E 'test(checkout)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-project --no-tests=fail` passes.
3. `cargo clippy -p duet-project --all-targets -- -D warnings` prints nothing.
4. `git status` inside `crates/duet-project` shows no change to `Cargo.toml` and no change to `Cargo.lock`, because this chunk adds no dependency.
5. One commit on the branch `chunk/f4-checkout-and-head-watch` passes the native git hook.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
