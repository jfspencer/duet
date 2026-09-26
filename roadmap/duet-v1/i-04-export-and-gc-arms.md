---
id: I4
line: I
depends_on: [I3, F2, H2, H3]
write_scope:
  - crates/bc_gateway/duet-core/lang_rust/src/gateway.rs
  - crates/bc_gateway/duet-core/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-core -E 'test(export_arm) + test(gc_arm)' --no-tests=fail passes; cargo clippy -p duet-core --all-targets -- -D warnings is clean; commit SHA on a branch chunk/i4-export-and-gc-arms"
---

# I4: The `ExportAudio`, `MasterMeasure`, and `Gc` dispatch arms

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk I1 answered exactly three verbs with `GatewayError::NotYetImplemented`, because
each one needs a crate that phase 7 had not built. This chunk delivers the three arms with the TH8
re-validation of architecture section 9.6, and it adds the test that asserts no verb answers
`NotYetImplemented`. It implements architecture sections 9.1, 9.6 (TH8), 12.1 and 13.2. It carries
product stories MA-01, MA-03, MA-04, MA-05 and X-13.

**This chunk keeps `GatewayError::NotYetImplemented` and deletes no variant.** Architecture line 1.5
places `GatewayError` in `duet-command`, and chunk T4 is the one chunk that writes that crate, seven
phases earlier. A deletion would make this chunk edit a file outside its own line and outside its
own write scope, which is the contract that lets phases run in parallel (critic C16-10).

## Files

- `crates/bc_gateway/duet-core/lang_rust/src/gateway.rs` — modify. The three arms and the guard test.
- `crates/bc_gateway/duet-core/lang_rust/Cargo.toml` — modify. The `duet-export` entry.
- `Cargo.lock` — modify (SM5 rule 2).

## Types and signatures

### Consumed from `duet-command`

Copied from architecture section 9.1.

```rust
pub enum Verb {
    // ...
    Gc { purge: Box<[SourceHash]> },
    MasterMeasure(Box<MeasureRequest>),
    ExportAudio(Box<AudioExportRequest>),
    // ...
}
```

```rust
/// Measure the master output without writing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureRequest { span: Option<Span>, normalization: Normalization }

/// Render the mix to one audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioExportRequest { path: PathBuf, spec: ExportSpec, overwrite: bool, part: Option<PartId> }
```

`part` is the per-part choice of product story MA-05. `None` renders the whole mix to `path`, and
`Some(part)` renders that part alone. Architecture section 15.5 states that rule.

### Consumed from `duet-export`, which this chunk first names

| Item | Chunk that wrote it | Declaring section |
|---|---|---|
| The measure job | H2 | 7.4 |
| The encode stage over `duet-media`, with the size promotion rule | H3 | 7.4, 7.5 |
| The Streaming, Broadcast and Custom presets, read at run time | H4 | 7.4 |

### Consumed from `duet-project`

| Item | Chunk that wrote it | Declaring section |
|---|---|---|
| The reachability walk, `gc` and `--purge` | F2 | 4.4 |

### The TH8 re-validation table, copied from architecture section 9.6

| Verb | What the core re-checks before it commits |
|---|---|
| `Gc` | Every candidate hash against the live set as it stands now. A take recorded while the walk ran is therefore safe. |

`MasterMeasure` and `ExportAudio` write no document, so TH8 asks each one only that its target still
exists when the worker returns.

## Steps

1. Read `crates/bc_gateway/duet-core/lang_rust/src/gateway.rs`. Confirm that exactly three arms answer
   `GatewayError::NotYetImplemented` and that chunk I1's test
   `dispatch_answers_not_yet_implemented_for_exactly_three_verbs` passes. Stop and report a
   discrepancy.
2. Add the `duet-export` entry to `crates/bc_gateway/duet-core/lang_rust/Cargo.toml` with `{ workspace = true }`. SM1
   makes the chunk that uses a dependency add the entry in the same commit as the code that uses it.
   Run `cargo build --workspace` and commit the `Cargo.lock` it produces.
3. Write the failing test `no_verb_is_unimplemented` in `src/gateway.rs`. It drives every `Verb` arm
   and asserts that none answers `GatewayError::NotYetImplemented`. Architecture section 13.2 names
   this test.
4. Run `cargo nextest run -p duet-core -E 'test(no_verb_is_unimplemented)' --no-tests=fail` and
   confirm that it fails on three arms.
5. Implement the `Gc` arm. `Verb::cost` reports `VerbCost::Deferred`, so the arm enqueues a job. The
   worker runs the reachability walk of section 4.4 against an `Arc` snapshot. The core then
   re-checks every candidate hash against the live set as it stands now, before it purges.
6. Implement the `MasterMeasure` arm. The arm enqueues a job that calls the measure job of
   `duet-export`. The job reports progress as `CoreEvent::JobProgress`, and `JobCancel` stops it at
   the next progress point. The result is a `VerbData::Loudness(Box<LoudnessReport>)`.
7. Implement the `ExportAudio` arm. The arm enqueues a job that calls the encode stage of
   `duet-export`. It reads the preset set at run time, so it names no preset type. It renders one
   file per request, and it names the part in the job state when `part` is `Some`.
8. Apply the cleanup contract of section 9.6 to both writing arms: a render writes through
   `exports/.tmp/<job>/`, the worker deletes its own temporary directory in its `Drop`, and a
   finished render is renamed into place as the last step.
9. Run `cargo nextest run -p duet-core -E 'test(no_verb_is_unimplemented)' --no-tests=fail` and
   confirm that it passes.
10. Write the remaining tests of the Tests section.
11. Run the Completion command and confirm that it passes.
12. Run `cargo clippy -p duet-core --all-targets -- -D warnings` and confirm that it is clean.
13. Commit on a branch named `chunk/i4-export-and-gc-arms`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in `crates/bc_gateway/duet-core/lang_rust/src/gateway.rs`. Every assert
carries a message.

| Test | What it asserts |
|---|---|
| `no_verb_is_unimplemented` | It drives every `Verb` arm and asserts that none answers `GatewayError::NotYetImplemented`. The variant stays declared, and this test is the guard that no verb uses it (section 13.2, critic C16-10). |
| `export_arm_starts_a_job_and_names_it` | `Verb::ExportAudio` answers `VerbOutcome::Started { job }`, and `JobStatus` for that identifier answers `JobState::Queued` or `JobState::Running`. |
| `export_arm_renders_one_part_when_the_request_names_one` | An `AudioExportRequest` with `part: Some(part)` renders that part alone, and the job state names the part it renders now (PR MA-05). |
| `export_arm_writes_through_the_temporary_directory` | The render writes under `exports/.tmp/<job>/` and renames into place as the last step, so no truncated output file is reachable (section 9.6). |
| `export_arm_deletes_its_output_on_a_cancel` | A cancelled render deletes its temporary directory and any part file before it sets `JobState::Cancelled`. |
| `export_arm_measures_before_it_writes` | `Verb::MasterMeasure` answers `VerbData::Loudness` and writes no file. |
| `gc_arm_revalidates_every_candidate_hash` | A take recorded while the walk ran stays in the live set, and the purge skips its hash (TH8, section 9.6). |
| `gc_arm_refuses_a_purge_of_a_reachable_source` | A hash the reachability walk still reaches is never purged, whatever the request names. |

## Verification

1. `cargo nextest run -p duet-core -E 'test(export_arm) + test(gc_arm)' --no-tests=fail` passes and
   selects at least one test per term.
2. `cargo nextest run -p duet-core --no-tests=fail` passes, including `no_verb_is_unimplemented` and
   chunk I1's `dispatch_answers_not_yet_implemented_for_exactly_three_verbs`. The second test now
   asserts a count of zero refusals at run time, so update its name and its assertion only if
   chunk I1 wrote it as a count that this chunk makes false; report the change.
3. `cargo clippy -p duet-core --all-targets -- -D warnings` prints no warning.
4. One commit on a branch named `chunk/i4-export-and-gc-arms`. The native git hook runs
   `scripts/dod.sh`.

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
