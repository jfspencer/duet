---
id: H2
line: H
depends_on: [H1, D2, E3, M8]
write_scope:
  - crates/bc_audio/duet-export/lang_rust/src/loudness/analyse.rs
  - crates/bc_audio/duet-export/lang_rust/src/loudness/limit.rs
parallelism: independent
completion: "cargo nextest run -p duet-export -E 'test(two_pass) + test(cancel_cleanup)' --no-tests=fail passes; commit SHA on a branch chunk/h2-two-pass-loudness"
---

# H2: The two-pass loudness graph, the intermediate file, the limiter stage, and the measure job

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the two passes of architecture section 7.4: pass one measures loudness through `ebur128` and writes an f32 intermediate file, and pass two reads that file, applies the computed gain and the true-peak limiter, and hands the result to the encode stage. It also builds the intermediate directory and its cleanup contract (section 9.6) and the measure job that answers `Verb::MasterMeasure`. It carries MUST stories MA-01 (loudness target and true-peak ceiling), MA-02 (master chain, the limiter half) and MA-03 (measured report).

**The limiter kernel is the ONE implementation the live master chain and this render both call**, so the monitor and the file agree. Chunk D2 wrote that kernel, and this chunk calls it. Links `D2 before H2` and `E3 before H2` of section 13.4 state both reasons.

**This chunk writes no member manifest**, so it names neither `crates/bc_audio/duet-export/lang_rust/Cargo.toml` nor `Cargo.lock` in its write scope, and its Completion command carries no `--locked` flag (SM3 rule 4, SM5).

## Files

- `crates/bc_audio/duet-export/lang_rust/src/loudness/analyse.rs` — modify. Chunk H1 created the stub.
- `crates/bc_audio/duet-export/lang_rust/src/loudness/limit.rs` — modify.

## Types and signatures

### The two passes (architecture 7.4)

```text
  -> pass 1: LoudnessAnalyse (ebur128) and write an f32 intermediate file
  -> pass 2: read the intermediate -> Gain (computed) -> TruePeakLimiter
             -> Dither -> Encoder
```

**The intermediate file exists because pass one must finish before pass two knows the gain.** It has a named place and a named owner: it is `exports/.tmp/<job>/intermediate.f32`, and the running job owns the whole directory.

### `duet_export::loudness::analyse` (architecture 7.4, 15.5)

Pass one runs the render of chunk H1 once, feeds every frame to `ebur128` through `duet_analysis`, and writes the same frames to the intermediate file as `f32`. It produces one report:

```rust
// in duet-command
/// What a loudness measurement produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoudnessReport { integrated_lufs: Finite, short_term_lufs: Finite, momentary_lufs: Finite, true_peak_dbtp: Finite, range_lu: Finite }
```

`LoudnessReport` is a `duet-command` type, because `VerbData::Loudness` carries it and VR3 binds. This crate reads it through its `duet-command` edge.

The measure job answers `Verb::MasterMeasure` and writes no output file: it runs pass one, reports the `LoudnessReport`, and deletes its own temporary directory. `MeasureRequest` carries what it measures:

```rust
// in duet-command
/// Measure the master output without writing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureRequest { span: Option<Span>, normalization: Normalization }
```

### `duet_export::loudness::limit` (architecture 7.4)

Pass two reads the intermediate file, applies the gain that `Normalization` and the pass-one report determine, and then runs the true-peak limiter kernel of `duet-dsp`.

```rust
// in duet-command
/// How the render sets the level. Every level is a `Finite`, so a hand
/// edited file cannot carry a NaN into the limiter (VR4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Normalization {
    None,
    Peak { dbfs: Finite },
    Loudness { lufs: Finite, true_peak_dbtp: Finite },
}
```

B75 gives the two presets: Streaming at -14 LUFS and -1 dBTP, and Broadcast at -23 LUFS and -1 dBTP. Chunk H4 declares the preset set; this chunk takes the two numbers from the `Normalization` value it is given. `Custom` takes both numbers from the user through `Finite::new`, so the form rejects a non-finite entry before the verb is built.

`LimiterState` and the true-peak limiter kernel are `duet-dsp` types that chunk D2 wrote. The look-ahead is B122, and `BufferPool::LIMITER_SLOTS` is B120 with a look-ahead buffer of B121; the offline path takes its look-ahead buffer from its own allocation and never from the audio pool, because it runs on a job thread and not on the audio thread.

### The cleanup contract this chunk extends (architecture 9.6, 4.1)

| Store | Bound | Eviction rule |
|---|---|---|
| `exports/.tmp/` | One directory per running render job | The job deletes its own directory on success, on failure, and on cancel. The next start empties the parent, because no job survives a restart. |

The intermediate file is inside that directory, so one deletion removes it. **The worker deletes its own temporary directory and any partly written output file** before it sets `JobState::Cancelled` or `JobState::Failed`, and the deletion runs in the worker's `Drop`.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| The true-peak limiter kernel and `LimiterState` | `duet-dsp` | `duet_dsp::dynamics::limiter`, chunk D2 |
| The loudness reader over `ebur128` | `duet-analysis` | `duet_analysis::loudness`, chunk E3 |
| `LoudnessReport`, `Normalization`, `MeasureRequest`, `ExportSpec`, `JobState` | `duet-command` | `duet_command` |
| `RenderJob`, `ExportError`, the render harness and its three stages | `duet-export` | chunk H1 |
| `Finite`, `Span` | `duet-time` | `duet_time` |

## Steps

1. Read both files of the write scope. Confirm that chunk H1 left each one a stub and wrote the render harness, that chunk D2 wrote the true-peak limiter kernel, and that chunk E3 wrote the loudness reader. Confirm that chunk M8 has landed. Report a discrepancy and stop.
2. Write the failing test `two_pass_measures_before_it_applies_gain` in `src/loudness/analyse.rs`. Run `cargo nextest run -p duet-export -E 'test(two_pass)' --no-tests=fail` and confirm that it fails to compile.
3. Write pass one in `src/loudness/analyse.rs`. It runs the chunk H1 render once, feeds every frame to the `duet-analysis` loudness reader, writes the same frames to `exports/.tmp/<job>/intermediate.f32` as `f32`, and returns one `LoudnessReport`. Run the test and confirm that it passes.
4. Write the failing test `two_pass_writes_the_intermediate_in_the_job_directory`. Run it and confirm that it fails, then confirm that the intermediate file sits at `exports/.tmp/<job>/intermediate.f32` and nowhere else.
5. Write the failing test `two_pass_applies_the_computed_gain` in `src/loudness/limit.rs`. Run it and confirm that it fails.
6. Write pass two in `src/loudness/limit.rs`. It reads the intermediate file, computes the gain from the `Normalization` value and the pass-one report, applies it, and then runs the `duet-dsp` true-peak limiter kernel. Run the test and confirm that it passes.
7. Write the failing test `two_pass_holds_the_true_peak_ceiling`. Run it and confirm that it fails, then confirm that no output sample passes the requested true-peak ceiling.
8. Write the failing test `two_pass_calls_the_one_limiter_kernel`. Confirm by construction that this crate declares no limiter of its own and calls `duet_dsp` alone, so the live master chain and the render agree.
9. Write the failing test `two_pass_measure_job_writes_no_file`. Run it and confirm that it fails, then implement the measure job: it runs pass one, reports the `LoudnessReport`, writes no output file, and deletes its own temporary directory. Confirm that it passes.
10. Write the failing test `cancel_cleanup_removes_the_intermediate_file`. Run `cargo nextest run -p duet-export -E 'test(cancel_cleanup)' --no-tests=fail` and confirm that it fails.
11. Implement the cleanup contract over both passes: a cancel and a failure each delete `exports/.tmp/<job>/`, including the intermediate file, through the worker's `Drop`. Run the test and confirm that it passes.
12. Write the failing test `cancel_cleanup_reports_the_one_line`. Run it and confirm that it fails, then implement the two user lines of section 9.6 step 5 and confirm that it passes.
13. Run `cargo clippy -p duet-export --all-targets -- -D warnings`. Fix every finding in the code.
14. Commit on the branch `chunk/h2-two-pass-loudness`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. Each test creates its own scratch directory under `std::env::temp_dir()` with the process identifier and a nanosecond suffix, and removes it at the end. No test needs a device.

| Test | File | What it asserts |
|---|---|---|
| `two_pass_measures_before_it_applies_gain` | `src/loudness/analyse.rs` | A recorder that logs every stage shows the loudness measurement complete before the first gain sample. Message: "pass one finishes before pass two applies gain". |
| `two_pass_writes_the_intermediate_in_the_job_directory` | `src/loudness/analyse.rs` | The intermediate file is at `exports/.tmp/<job>/intermediate.f32` and no file is written outside that directory. Message: "the intermediate file sits in the job directory". |
| `two_pass_reports_every_loudness_field` | `src/loudness/analyse.rs` | The `LoudnessReport` carries a finite value in each of its five fields. Message: "the report carries five finite measurements". |
| `two_pass_measure_job_writes_no_file` | `src/loudness/analyse.rs` | A `MeasureRequest` run reports a `LoudnessReport`, writes no output file, and leaves no temporary directory. Message: "a measure job reports and writes no file". |
| `two_pass_applies_the_computed_gain` | `src/loudness/limit.rs` | A render whose measured integrated loudness is -20 LUFS with a target of -14 LUFS raises the level by 6 dB inside the tolerance B76. Message: "pass two applies the gain the measurement computes". |
| `two_pass_holds_the_true_peak_ceiling` | `src/loudness/limit.rs` | With a ceiling of -1 dBTP, no output sample measures above -1 dBTP. Message: "no sample passes the true-peak ceiling". |
| `two_pass_calls_the_one_limiter_kernel` | `src/loudness/limit.rs` | The render output and a live master chain run of the same input agree sample for sample. Message: "the render and the monitor call one limiter kernel". |
| `two_pass_normalization_none_changes_no_sample` | `src/loudness/limit.rs` | With `Normalization::None`, pass two writes the intermediate samples unchanged. Message: "no normalization changes no sample". |
| `two_pass_peak_normalization_reaches_the_target` | `src/loudness/limit.rs` | With `Normalization::Peak { dbfs }`, the highest output sample equals the target inside the tolerance. Message: "peak normalization reaches the requested level". |
| `cancel_cleanup_removes_the_intermediate_file` | `src/loudness/limit.rs` | After a cancel in pass two, `exports/.tmp/<job>/` is gone and the intermediate file with it. Message: "a cancel removes the job directory and the intermediate file". |
| `cancel_cleanup_removes_a_partly_written_output` | `src/loudness/limit.rs` | After a failure in pass two, no partly written output file exists at the target path. Message: "a failure leaves no partial output file". |
| `cancel_cleanup_reports_the_one_line` | `src/loudness/limit.rs` | A cancel reports `The export stopped. No file was written.` and a failure reports `The export failed: <reason>. No file was written.`. Message: "each outcome reports its one line". |

## Verification

1. `cargo nextest run -p duet-export -E 'test(two_pass) + test(cancel_cleanup)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of either name exists.
2. `cargo nextest run -p duet-export --no-tests=fail` passes.
3. `cargo clippy -p duet-export --all-targets -- -D warnings` prints nothing.
4. `git status` inside `crates/bc_audio/duet-export/lang_rust` shows no change to `Cargo.toml` and no change to `Cargo.lock`, because this chunk adds no dependency.
5. One commit on the branch `chunk/h2-two-pass-loudness` passes the native git hook.

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
