---
id: J3
line: J
depends_on: [J2, H4, I4]
write_scope:
  - crates/bc_gateway/duet-agent/lang_rust/tests/export_end_to_end.rs
parallelism: independent
completion: "cargo nextest run -p duet-agent --test export_end_to_end --no-tests=fail passes; cargo clippy -p duet-agent --all-targets -- -D warnings is clean; commit SHA on a branch chunk/j3-export-end-to-end"
---

# J3: The headless export end-to-end test

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk writes the second half of the headless product path. It proves that an agent
measures the master output and then exports a file, with no window and no device. It implements
architecture sections 7.4, 7.5, 9.1, 9.3, 9.6 and 14 rung two. It carries product stories MA-01,
MA-03 and MA-04, which are MUST.

**The headless product path is TWO tests, and the phase numbers are the reason** (critic C16-9).
Chunk J1 wrote `end_to_end`, which asserts what phase 8 delivers. The export half needs chunk I4 in
phase 10, chunk H3 in phase 9 and chunk H4 in phase 10, so a single test would fail every commit of
phases 8 and 9, and CLAUDE.md forbids `--no-verify`. This chunk is the one chunk that writes the
export half, and it lands in phase 11.

**Neither test is `#[ignore]`**, because each one runs green in every phase from the one that writes
it. SM2 allows both files, because an integration test under `tests/` is its own crate root and
needs no `mod` line anywhere.

## Files

- `crates/bc_gateway/duet-agent/lang_rust/tests/export_end_to_end.rs` — create. One integration test, wrapped in a
  `#[cfg(test)] mod tests` because `clippy::tests_outside_test_module` is denied.

This chunk writes no member manifest and no lock file. Every dependency it needs is already an entry
that chunk J1 added.

## Types and signatures

### Consumed from `duet-command`

Copied from architecture section 9.1 and 15.5.

```rust
pub enum Verb {
    // ...
    MasterMeasure(Box<MeasureRequest>),
    ExportAudio(Box<AudioExportRequest>),
    JobStatus { job: JobId },
    // ...
}

/// Measure the master output without writing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureRequest { span: Option<Span>, normalization: Normalization }

/// Render the mix to one audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioExportRequest { path: PathBuf, spec: ExportSpec, overwrite: bool, part: Option<PartId> }

/// What a loudness measurement produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoudnessReport { integrated_lufs: Finite, short_term_lufs: Finite, momentary_lufs: Finite, true_peak_dbtp: Finite, range_lu: Finite }

/// What a job reports while it runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running { done: u32, total: u32 },
    Done,
    Cancelled,
    Failed(GatewayError),
}

/// What a verb produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbOutcome {
    Changed { version: Version, events: Vec<DomainEvent> },
    Data(Box<VerbData>),
    Started { job: JobId },
}
```

### The two assertions the architecture names

Architecture section 13.2 gives this chunk the B75 and B76 assertions.

| Bound | What this test asserts |
|---|---|
| B75 | The Streaming preset target, which is -14 LUFS with a -1 dBTP ceiling (design contract 5.3 correction note, product requirements section 11 Q7). The measured integrated loudness of the exported file sits inside the target tolerance. |
| B76 | The true-peak result stays at or below the ceiling the preset names. |

Read the live value of each bound from architecture section 1.6 before the assertion is written. The
test names the constant and never a literal, which is rule DR3.

### The presets, read at run time

Chunk H4 wrote the Streaming, Broadcast and Custom presets. Chunk I4's `ExportAudio` arm reads the
preset list at run time, so this test names a preset by its identifier and never by a number.

## Steps

1. Read `crates/bc_gateway/duet-agent/lang_rust/tests/`. Confirm that `end_to_end.rs` exists and that
   `export_end_to_end.rs` does not. Stop and report a discrepancy.
2. Confirm that chunk I4 landed: run
   `cargo nextest run -p duet-core -E 'test(export_arm) + test(gc_arm)' --no-tests=fail` and confirm
   that it passes. A red run means this chunk starts too early.
3. Create `crates/bc_gateway/duet-agent/lang_rust/tests/export_end_to_end.rs` with a `//!` file doc and a
   `#[cfg(test)] mod tests`.
4. Write the test body as a failing test first. Build a headless host with the dummy backend, exactly
   as `end_to_end.rs` does. Create a project from the SATB template, add one track, arm it, and
   record eight bars from the dummy backend.
5. Run `cargo nextest run -p duet-agent --test export_end_to_end --no-tests=fail` and confirm that
   it fails.
6. Add the measure half. Call `Verb::MasterMeasure` with the Streaming normalization. Assert that
   the outcome is `VerbOutcome::Started { job }`.
7. Poll `Verb::JobStatus { job }` and assert the progress sequence: at least one
   `JobState::Running { done, total }` with `done` below `total`, then `JobState::Done`.
8. Assert the completion value. The measure answers a `VerbData::Loudness(Box<LoudnessReport>)` with
   five finite fields.
9. Add the export half. Call `Verb::ExportAudio` with an `AudioExportRequest` whose `spec` names a
   48 kHz 24-bit WAV container and the Streaming preset, with `part: None` and `overwrite: false`.
10. Poll `Verb::JobStatus { job }` to `JobState::Done`, then assert that the file exists at the
    requested path and that no file remains under `exports/.tmp/`.
11. Read the written file header and assert the sample rate is 48 000 and the bit depth is 24.
12. Measure the written file and assert B75 and B76: the integrated loudness sits inside the preset
    tolerance, and the true peak stays at or below the preset ceiling.
13. Assert that the run opened no window: the test never calls `gpui_kit::application()` and the
    binary links GPUI without starting it (section 9.3).
14. Run the Completion command and confirm that it passes.
15. Run `cargo clippy -p duet-agent --all-targets -- -D warnings` and confirm that it is clean.
16. Commit on a branch named `chunk/j3-export-end-to-end`.

## Tests

One integration test file, wrapped in a `#[cfg(test)] mod tests`. Every assert carries a message.
The test uses its own scratch directory, per the `test-author` skill.

| Test | File | What it asserts |
|---|---|---|
| `export_end_to_end` | `crates/bc_gateway/duet-agent/lang_rust/tests/export_end_to_end.rs` | The whole path of the Steps section: create, arm, record eight bars, `MasterMeasure` with its progress and its completion, then a 48 kHz 24-bit WAV export with the Streaming preset. It asserts the B75 loudness result and the B76 true-peak result on the written file, that the file exists at the requested path, and that `exports/.tmp/` is empty. |

The test carries no `#[ignore]`. `scripts/dod.sh` runs it at every commit from phase 11 onward.

Architecture section 14 rung two selects this test by name, and SM7 binds the name to this chunk:
every test name a rung-two command or a workflow command names in a filter is written by exactly one
chunk, and that chunk lands at or before the phase of the command that selects it.

## Verification

1. `cargo nextest run -p duet-agent --test export_end_to_end --no-tests=fail` passes and prints no
   `no tests to run` line.
2. `cargo nextest run -p duet-agent --no-tests=fail` passes, so `end_to_end` still passes.
3. `cargo clippy -p duet-agent --all-targets -- -D warnings` prints no warning.
4. The run passes on `macos-26` and on `ubuntu-26.04`, because the dummy backend needs no device.
5. One commit on a branch named `chunk/j3-export-end-to-end`. The native git hook runs
   `scripts/dod.sh`. Quote the command output before any claim of success, per the
   `verification-before-completion` skill.

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
