---
id: F3
line: F
depends_on: [F2]
write_scope:
  - crates/bc_project/duet-project/lang_rust/src/history/gix_impl.rs
  - crates/bc_project/duet-project/lang_rust/src/history/budget.rs
  - crates/bc_project/duet-project/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-project -E 'test(history_budget)' --no-tests=fail passes; commit SHA on a branch chunk/f3-history-over-gix"
---

# F3: The `History` trait, the gix implementation, and the B19 budget

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the git history of architecture section 4.5: the `History` trait, its one gix implementation, `TrackedFile`, `TreeSnapshot`, `HeadState`, `HistoryError`, the object-format read path of section 4.7, and the B19 budget on a job thread. ADR 0003 clauses 10, 15 and 16 decide the object format, the one-trait boundary and the symbolic-reference rule. It carries MUST stories H-01 (commit from the user interface), H-02 (browse the history), H-04 (branch), and H-06 (agent history control).

**Every `gix` type stays inside `duet-project` behind one `History` trait.** No `gix` type appears in any other crate's public signature. Every `History` call runs on a job thread under B19, which TH3 requires.

## Files

- `crates/bc_project/duet-project/lang_rust/src/history/gix_impl.rs` — modify. Chunk F1 created the stub.
- `crates/bc_project/duet-project/lang_rust/src/history/budget.rs` — modify.
- `crates/bc_project/duet-project/lang_rust/Cargo.toml` — modify. Add the `gix` entry.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_project::history` (architecture 4.5)

```rust
/// The history operations the project needs. One implementation uses gix.
pub trait History {
    /// # Errors
    /// Returns `HistoryError::Write` when an object cannot be written.
    fn commit(&mut self, message: &str, files: &[TrackedFile]) -> Result<CommitId, HistoryError>;

    /// # Errors
    /// Returns `HistoryError::MissingRef` when the branch does not exist.
    fn branch(&mut self, name: &BranchName, from: CommitId) -> Result<(), HistoryError>;

    /// # Errors
    /// Returns `HistoryError::Read` when an object cannot be read.
    fn log(&self, start: CommitId, limit: u32) -> Result<Vec<CommitSummary>, HistoryError>;

    /// Read a commit's tracked files into memory. It touches no worktree file.
    ///
    /// # Errors
    /// Returns `HistoryError::Read` when an object cannot be read.
    fn read_tree(&self, commit: CommitId) -> Result<TreeSnapshot, HistoryError>;

    /// The commit that `HEAD` names now.
    ///
    /// # Errors
    /// Returns `HistoryError::Read` when `HEAD` cannot be resolved.
    fn head(&self) -> Result<HeadState, HistoryError>;

    /// The object format this repository uses.
    ///
    /// **The repository configuration is the authority, and an absent key is
    /// an answer.** This method reads `core.repositoryFormatVersion` first. At
    /// version 0 it returns `HashKind::Sha1` and reads no extension. At
    /// version 1 it reads `extensions.objectFormat` and maps the value.
    ///
    /// # Errors
    /// Returns `HistoryError::Read` when the configuration cannot be read, and
    /// when the repository declares a format version this build does not know.
    fn object_format(&self) -> Result<HashKind, HistoryError>;
}
```

### `duet_project::history::gix_impl` (architecture 15.12)

```rust
/// One file the history tracks, as its bundle-relative path and its bytes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedFile { path: Box<str>, bytes: Vec<u8> }

/// The tracked files of one commit, read into memory.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeSnapshot { commit: CommitId, files: Vec<TrackedFile> }

/// Where `HEAD` points now.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeadState { Branch { name: BranchName, commit: CommitId }, Detached(CommitId), Unborn }
```

A commit is one user action or one agent verb group. The implementation writes the tracked files, hashes them, writes the blobs, builds the tree, and commits. **It never runs the `git` binary.** `gix::ObjectId` is the type the implementation returns, and `duet-project` converts it to `CommitId` at its own boundary, because PL3 maps one name to one type in one crate.

### `duet_project::history::budget` (architecture 4.5, 5.12)

Every gix call runs on a job thread under B19. A call that passes the budget returns `HistoryError::Timeout`, and the user interface shows the history surface in a failed state with a retry action. A repository on a stalled network volume cannot hang a verb.

### The pointer table this chunk enforces (architecture 4.5)

| Pointer | Form | Reason |
|---|---|---|
| The project's current line of work | A branch name in `duet.toml` | A rebase or an amend rewrites every commit identifier on the branch. |
| The autosave target | The symbolic reference `refs/duet/autosave` | The same reason, and it must survive a branch switch. |
| A take's origin commit, shown in the take list | Not stored | It is computed from the manifest at read time. |
| A history entry the user pinned | A tag under `refs/tags/duet/<name>` | A tag survives a rewrite of the branch that created it. |
| A commit identifier inside a `Verb` call | A bare identifier | It is a transient argument. |

**No stored pointer is a bare commit identifier.**

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `CommitId`, `CommitDigest`, `HashKind`, `BranchName`, `CommitSummary` | `duet-command` | `duet_command` |
| `HistoryError`, `ProjectError` | `duet-project` | chunk F1 |

The object-format read path of section 4.7: the open path returns `ProjectError::ObjectFormat` for anything but `HashKind::Sha1`, because `CommitDigest` reads its length from that answer. **An absent `extensions.objectFormat` key is never a refusal.**

## Steps

1. Read every file of the write scope and `crates/bc_project/duet-project/lang_rust/Cargo.toml`. Confirm that chunk F1 left both files a stub and that chunk F2 has landed. Report a discrepancy and stop.
2. Add to `crates/bc_project/duet-project/lang_rust/Cargo.toml` the entry `gix = { workspace = true }`. Run `cargo build --workspace` and keep `Cargo.lock` for the same commit.
3. Write the failing test `history_budget_returns_timeout_past_b19` in `src/history/budget.rs`. Run `cargo nextest run -p duet-project -E 'test(history_budget)' --no-tests=fail` and confirm that it fails to compile.
4. Write `src/history/budget.rs`: one wrapper that runs a `History` call on a job thread inside B19 and returns `HistoryError::Timeout` on expiry. The wrapper takes the call as a closure and the budget as an argument, so a test drives it with plain numbers and no wall clock. Run the test and confirm that it passes.
5. Write `TrackedFile`, `TreeSnapshot` and `HeadState` in `src/history/gix_impl.rs`, with the `History` trait in `src/history.rs`.
6. Write the failing test `history_commit_writes_the_tracked_set`. Run it and confirm that it fails.
7. Implement `History::commit` over gix: write the tracked files, hash them, write the blobs, build the tree, and commit. Run no `git` binary. Run the test and confirm that it passes.
8. Implement `History::branch`, `History::log`, `History::read_tree` and `History::head`. `read_tree` reads a commit's tracked files into memory and touches no worktree file.
9. Write the failing test `history_object_format_answers_sha1_at_version_zero`. Run it and confirm that it fails.
10. Implement `History::object_format`: read `core.repositoryFormatVersion` first; at version 0 return `HashKind::Sha1` and read no extension; at version 1 read `extensions.objectFormat` and map the value; for a version this build does not know return `HistoryError::Read`. Run the test and confirm that it passes.
11. Write the failing test `history_bundle_opens_with_the_real_git_binary`. It writes a bundle with the real `git` binary, runs `git status` inside it, reads it with gix, and asserts both that `git` opens it and that the bytes match. Run it and confirm that it fails, then confirm that it passes.
12. Implement the pointer rules of the table above: `duet.toml` holds a branch name, the autosave target is `refs/duet/autosave`, and a pinned entry is a tag under `refs/tags/duet/<name>`.
13. Run `cargo clippy -p duet-project --all-targets -- -D warnings`. Fix every finding in the code.
14. Commit on the branch `chunk/f3-history-over-gix`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test writes inside this repository.

| Test | File | What it asserts |
|---|---|---|
| `history_budget_returns_timeout_past_b19` | `src/history/budget.rs` | A call that does not answer inside the budget returns `HistoryError::Timeout` and the caller is not blocked. Message: "a History call past B19 returns Timeout". |
| `history_budget_returns_the_value_inside_the_bound` | `src/history/budget.rs` | A call that answers inside the budget returns its value unchanged. Message: "a History call inside B19 returns its value". |
| `history_commit_writes_the_tracked_set` | `src/history/gix_impl.rs` | A commit of three `TrackedFile` values writes three blobs, one tree and one commit, and `read_tree` returns the same three files. Message: "a commit writes and reads back the tracked set". |
| `history_commit_runs_no_git_binary` | `src/history/gix_impl.rs` | With a `PATH` that holds no `git`, the commit still succeeds. Message: "the implementation runs no git binary". |
| `history_log_returns_the_summaries_in_order` | `src/history/gix_impl.rs` | Three commits produce three `CommitSummary` values, newest first, each with its own message and parents. Message: "the log returns the summaries newest first". |
| `history_head_reports_unborn_then_branch` | `src/history/gix_impl.rs` | A new repository reports `HeadState::Unborn`; after one commit it reports `HeadState::Branch { name, commit }`. Message: "head reports Unborn before the first commit". |
| `history_branch_refuses_a_missing_reference` | `src/history/gix_impl.rs` | `branch` from a commit that does not exist returns `HistoryError::MissingCommit`. Message: "a branch from an unknown commit is refused". |
| `history_object_format_answers_sha1_at_version_zero` | `src/history/gix_impl.rs` | A repository at `core.repositoryFormatVersion` 0 with no `extensions.objectFormat` key answers `HashKind::Sha1`. Message: "an absent object-format key answers Sha1 at version 0". |
| `history_object_format_reads_the_extension_at_version_one` | `src/history/gix_impl.rs` | A repository at format version 1 with `extensions.objectFormat = sha256` answers `HashKind::Sha256`. Message: "version 1 reads the object-format extension". |
| `history_bundle_opens_with_the_real_git_binary` | `src/history/gix_impl.rs` | `git status` inside a bundle this chunk created exits without an error, and gix reads the same bytes. Message: "plain git opens a bundle this build creates". |

## Verification

1. `cargo nextest run -p duet-project -E 'test(history_budget)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-project --no-tests=fail` passes.
3. `cargo clippy -p duet-project --all-targets -- -D warnings` prints nothing.
4. `cargo doc -p duet-project` is clean with `-D warnings`, and no public signature outside `duet-project` names a `gix` type.
5. `cargo machete` reports no unused dependency of `duet-project`.
6. One commit on the branch `chunk/f3-history-over-gix` passes the native git hook. The commit carries `crates/bc_project/duet-project/lang_rust/Cargo.toml` and `Cargo.lock` together.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives at `crates/bc_<context>/<crate>/lang_rust/` in the context that architecture section 1.2 names (ADR 0011), declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- After chunk M94 lands, every `.rs` file a commit writes carries one front-matter block (`cargo xtask check-ddd --write`), and the gate refuses a changed `.rs` file with none.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
