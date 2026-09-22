---
id: I3
line: I
depends_on: [I2, C3, D3, G4]
write_scope:
  - crates/duet-core/src/snapshot.rs
  - crates/duet-core/src/job.rs
  - crates/duet-core/src/midi_entry.rs
  - crates/duet-core/src/reader.rs
  - crates/duet-core/benches/
  - crates/duet-core/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-core -E 'test(snapshot) + test(job_registry) + test(playback_plan) + test(slot_measure)' --no-tests=fail passes; cargo clippy -p duet-core --all-targets -- -D warnings is clean; commit SHA on a branch chunk/i3-snapshots-and-readers"
---

# I3: Snapshot publication, the job registry, and the two frame readers

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk delivers the publication half of `duet-core`: snapshot publication with the
`Arc::make_mut` bench, the job registry with its B127 removal rule, the `PlaybackPlan` build on a
job thread inside B130 with the one `Input<PlaybackPlan>` write the core performs, the note-entry
map as a plain method with no framework type, the peak read task, and `MeterReader` and
`TransportReader` with the per-strip over-mark compare. It implements architecture sections 5.8,
5.10, 5.12, 7.3, 8.3, 9.6, 10.2, 15.14 and Appendix A. It carries product stories C-10, M-03, M-05,
X-01, X-02, X-09 and R-15 in the "Runtime or verb" column of architecture section 13.2.

This chunk owns a bench, so it owns the member manifest and the `benches/` directory. Architecture
section 13.2 states that rule, and SM6 keeps chunk I4 one phase later so the manifest has one writer
per phase.

## Files

- `crates/duet-core/src/snapshot.rs` — modify. Snapshot publication and the version counter read.
- `crates/duet-core/src/job.rs` — modify. `JobRegistry`, the progress path, and the B127 sweep.
- `crates/duet-core/src/midi_entry.rs` — modify. One `NoteEntry` to one `ScoreCommand`.
- `crates/duet-core/src/reader.rs` — modify. `MeterReader` and `TransportReader`.
- `crates/duet-core/benches/make_mut.rs` — create. The `criterion` bench of section 5.12.
- `crates/duet-core/Cargo.toml` — modify. The `criterion` dev-dependency, the `triple_buffer` entry
  if chunk I1 did not need it, and the `[[bench]]` target with `harness = false`.
- `Cargo.lock` — modify (SM5 rule 2).

## Types and signatures

### Declared by this chunk, in `crates/duet-core/src/reader.rs`

Copied from architecture section 15.14.

```rust
/// The one read end of the meter publication.
#[derive(Debug)]
pub struct MeterReader {
    output: Output<MeterSnapshot>,
    slots: Output<SlotMeterSnapshot>,
    resets: Arc<ResetGenerations>,
    latest: MeterView,
    latest_slots: [SlotMeasure; MAX_SLOT_METERS],
    live_slots: u16,
    caps: [Finite; MAX_STRIPS],
    holds: [Finite; MAX_STRIPS],
}

impl MeterReader {
    /// Read the publication, resolve every held mark, and age every peak cap
    /// into the slot.
    pub fn poll(&mut self, elapsed_ms: Finite);

    /// The meter values of this frame, as the last `poll` resolved them.
    #[must_use]
    pub const fn latest(&self) -> MeterView;

    /// The measurement at one cell of the section 7.3 formula, or `None`
    /// when the cell is at or above the number the topology fills.
    #[must_use]
    pub fn slot_measure(&self, cell: u16) -> Option<SlotMeasure>;
}

/// The one read end of the transport publication.
#[derive(Debug)]
pub struct TransportReader {
    output: Output<TransportSnapshot>,
    latest: TransportView,
}

impl TransportReader {
    /// Read the publication into the slot, once per frame.
    pub fn poll(&mut self);

    /// The transport of this frame.
    #[must_use]
    pub const fn latest(&self) -> TransportView;
}
```

### The one cell formula, copied from architecture section 7.3

```text
cell = strip * 2 * MAX_SLOTS + position * MAX_SLOTS + slot
```

`strip` is the strip's `ChainIndex`, `position` is 0 for a pre-fader slot and 1 for a post-fader
slot, and `slot` is the slot's index in that strip's own list. Both ends compute it and neither
stores it. B133 is `MAX_SLOT_METERS` and it is the product of the three bounds.

### Consumed from `duet-dsp`, `duet-command` and `duet-engine`

| Type | Crate | Declaring section |
|---|---|---|
| `MeterReading`, `MeterView`, `SlotMeasure`, `HeldMarks`, `OverMark`, `ResetGenerations` | `duet-dsp` | 7.3, 15.2 |
| `meter_law::db_to_fraction` | `duet-dsp` | 7.3 |
| `Pyramid`, `PyramidHeader` | `duet-dsp` | 15.2 |
| `TransportView`, `NoteEntry`, `JobId`, `JobState`, `GatewayError` | `duet-command` | 15.5, 8.3, 9.6 |
| `MeterSnapshot`, `SlotMeterSnapshot`, `TransportSnapshot`, `PlaybackPlan`, `Generation` | `duet-engine` | 5.9, 15.10 |
| `ScoreCommand::InsertNote`, `Revision` | `duet-score` | 15.3 |
| `Finite`, `MAX_STRIPS` (B86), `MAX_SLOTS` (B45), `MAX_SLOT_METERS` (B133) | `duet-time` | 1.6, 2.6a |

### The note-entry map, in `crates/duet-core/src/midi_entry.rs`

The map names no framework type. Architecture section 13.2 splits the drain across the crate
boundary: chunk I1 wrote `DuetCore::drain_note_entries`, this chunk writes the map from one
`NoteEntry` to one score command, and chunk K1 registers the call in `Window::on_next_frame`.

```rust
/// Map one played note to one score command, at the caret the last
/// `CoreInput::SetEntryContext` named (section 8.3).
///
/// It names no framework type, so `duet-core` carries no `gpui-kit` edge
/// (section 1.3 rule 6, section 13.2).
pub fn note_entry_to_command(entry: NoteEntry, caret: Position, duration: NoteValue) -> ScoreCommand;
```

## Steps

1. Read every file of the write scope. Confirm that chunk I1 created `snapshot.rs`, `job.rs`,
   `midi_entry.rs` and `reader.rs`, and that `job.rs` holds the seam declaration of `JobRegistry`
   and no behaviour. Stop and report a discrepancy.
2. Add `criterion` as a dev-dependency with `{ workspace = true }`, and add the `[[bench]]` target
   with `harness = false` and `name = "make_mut"`. Add `triple_buffer` if the manifest lacks it.
   Run `cargo build --workspace` and commit the `Cargo.lock` it produces.
3. Write the failing test `slot_measure_answers_one_cell_of_the_formula` in `src/reader.rs`. It
   fills a `SlotMeterSnapshot`, polls the reader, and asserts the value at one computed cell.
4. Run `cargo nextest run -p duet-core -E 'test(slot_measure)' --no-tests=fail` and confirm that it
   fails.
5. Implement `MeterReader::poll`. Read both `Output` values into the two foreground-only slots.
   Resolve each strip's over mark: the mark paints when `over[i]` is set and `latched[i]` is at or
   above `ResetGenerations[i]` (section 7.3).
6. Implement the peak cap rule of section 7.3 inside `poll`. A poll that reads a peak above the cap
   sets the cap and re-arms the hold at B111. A poll that does not subtracts `elapsed_ms` from the
   hold, and it falls the cap at B112 once the hold reaches zero. `elapsed_ms` is the one clock.
7. Implement `MeterReader::latest`, `MeterReader::slot_measure` and the whole `TransportReader`.
   `slot_measure` answers `None` for a cell at or above `live_slots`. It is the one producer of
   `StageCurve::measure`.
8. Implement snapshot publication in `src/snapshot.rs`: each document behind an `Arc`, a snapshot is
   an `Arc::clone`, and an edit applied while a snapshot is outstanding calls `Arc::make_mut`.
9. Write the `criterion` bench in `crates/duet-core/benches/make_mut.rs`. It measures one
   `Arc::make_mut` clone on the B1 fixture and reports against B5.
10. Implement `JobRegistry` in `src/job.rs`. Mint a `JobId` from `next`, write
    `JobState::Running { done, total }`, and emit `CoreEvent::JobProgress`. A worker that reaches a
    terminal state emits the last progress event and then removes its own entry from both maps.
11. Implement the B127 window. A terminal entry stays readable for B127, and the core sweeps every
    entry whose terminal moment is older than B127 on each pass of its own input loop. A `JobStatus`
    for an absent identifier answers `GatewayError::UnknownJob`.
12. Implement the `PlaybackPlan` build in `src/job.rs`. A job thread builds the plan inside B130 and
    hands it to the core; the core performs the one `write` on `DuetCore::plan`, because `Input` is
    the write end and one owner holds it.
13. Implement `note_entry_to_command` in `src/midi_entry.rs`, with the caret and the duration the
    last `CoreInput::SetEntryContext` named.
14. Implement the peak read task in `src/snapshot.rs`. It reads one `Pyramid` and emits
    `CoreEvent::PeaksReady(SourceHash)`. `LaneView` opens no file.
15. Write the remaining tests of the Tests section.
16. Run the Completion command and confirm that it passes.
17. Run `cargo bench -p duet-core --bench make_mut -- --test` and confirm that the bench builds and
    runs one iteration.
18. Run `cargo clippy -p duet-core --all-targets -- -D warnings` and confirm that it is clean.
19. Commit on a branch named `chunk/i3-snapshots-and-readers`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file. Every assert carries a message.

| Test | File | What it asserts |
|---|---|---|
| `slot_measure_answers_one_cell_of_the_formula` | `src/reader.rs` | A value written at `strip * 2 * MAX_SLOTS + position * MAX_SLOTS + slot` reads back at the same cell. |
| `slot_measure_answers_none_past_the_live_count` | `src/reader.rs` | A cell at or above `live_slots` answers `None`. |
| `slot_measure_carries_input_and_reduction_apart` | `src/reader.rs` | `SlotMeasure::input` and `SlotMeasure::reduction` are two fields and one value never fills both (critic C21-7). |
| `snapshot_clone_leaves_the_document_shared` | `src/snapshot.rs` | An `Arc::clone` shares the pointer, and the strong count rises by one. |
| `snapshot_make_mut_clones_once_per_outstanding_snapshot` | `src/snapshot.rs` | One edit after one outstanding snapshot clones once; the next edit clones nothing. |
| `snapshot_reader_holds_the_cap_then_falls` | `src/reader.rs` | A poll that reads a peak sets the cap; polls inside B111 hold it; a poll past B111 falls it at B112. Every millisecond value is a plain number. |
| `snapshot_reader_resolves_the_over_mark_per_strip` | `src/reader.rs` | A mark paints on the strip whose `latched` entry is at or above its reset generation, and on no other strip. |
| `job_registry_removes_a_terminal_entry_after_the_window` | `src/job.rs` | A terminal entry stays readable for B127 and then leaves both maps. |
| `job_registry_answers_unknown_job_after_the_sweep` | `src/job.rs` | A `JobStatus` for a swept identifier answers `GatewayError::UnknownJob`. |
| `job_registry_cancels_between_units_of_work` | `src/job.rs` | `JobCancel` sets the flag, the worker reads it between units, and no write is cut in half. |
| `playback_plan_is_written_once_by_the_core` | `src/job.rs` | The job thread returns a plan and performs no `Input::write`; the core performs exactly one write. |
| `playback_plan_build_reports_inside_the_budget` | `src/job.rs` | The build reports progress inside B130 with a plain frame count and no wall clock. |
| `note_entry_maps_to_one_insert_note_at_the_caret` | `src/midi_entry.rs` | One `NoteEntry` yields one `ScoreCommand::InsertNote` at the caret with the reported duration. |

## Verification

1. `cargo nextest run -p duet-core -E 'test(snapshot) + test(job_registry) + test(playback_plan) + test(slot_measure)' --no-tests=fail` passes and selects at least
   one test per term.
2. `cargo nextest run -p duet-core --no-tests=fail` passes.
3. `cargo bench -p duet-core --bench make_mut -- --test` builds and runs.
4. `cargo clippy -p duet-core --all-targets -- -D warnings` prints no warning.
5. One commit on a branch named `chunk/i3-snapshots-and-readers`. The native git hook runs
   `scripts/dod.sh`.

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
