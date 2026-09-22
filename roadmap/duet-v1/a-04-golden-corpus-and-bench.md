---
id: A4
line: A
depends_on: [A3]
write_scope:
  - crates/duet-engrave/tests/golden.rs
  - crates/duet-engrave/tests/golden/
  - crates/duet-engrave/benches/
  - crates/duet-engrave/benches/layout.rs
  - crates/duet-engrave/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-engrave --test golden --no-tests=fail"
---

# A4: The golden corpus, the comparator, and the criterion bench

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Phase 5 carries no manifest chunk, so this chunk depends on chunk A3 alone (section
13.3). Chunk M4 pinned `criterion` in phase 4, which Appendix B.3 states for this reason: A4 runs in
phase 5 and phases 5 and 6 have no manifest chunk, so SM1 rule 1 gives the pin to M4. This chunk
adds the golden corpus of B79 scores, the field-by-field comparator, and the criterion bench over
the layout pass. It implements architecture sections 10.7 and 14 rung one item 4.

None of the files in the write scope exists yet. SM2 covers `src/` alone, so an integration test
file under `tests/` is its own crate root, needs no `mod` line, and is created by the chunk that
writes it. This chunk therefore creates `tests/golden.rs` although chunk A1 created no stub for it.

## Files

- `crates/duet-engrave/tests/golden.rs` — create. The comparator and the twelve corpus cases.
- `crates/duet-engrave/tests/golden/` — create. Twelve input scores and twelve expected placement
  documents, as JSON. The file names are `case-01.score.json` to `case-12.score.json` and
  `case-01.expected.json` to `case-12.expected.json`.
- `crates/duet-engrave/benches/` — create. The bench directory.
- `crates/duet-engrave/benches/layout.rs` — create. The criterion bench over `engrave`.
- `crates/duet-engrave/Cargo.toml` — modify. Add the `criterion` dev-dependency and the `[[bench]]`
  target.
- `Cargo.lock` — modify. SM5 rule 2 puts it in the write scope of every chunk that writes a member
  manifest.

## Types and signatures

### Manifest

```toml
# crates/duet-engrave/Cargo.toml
[dev-dependencies]
criterion = { workspace = true }

[[bench]]
name = "layout"
harness = false
```

Appendix B.3 pins `criterion` at 0.7.0 and Appendix B.5 states its feature requirement: the bench
harness with no plotting back end, so the tree stays small and `cargo deny` stays quiet. SM4 forbids
this chunk to edit the root manifest, so a missing pin is a discrepancy that this chunk reports
under SM0.

### Consumed types

| Type | Crate and path |
|---|---|
| `engrave`, `SystemPlacement`, `SystemId`, `SizePx`, `QuadPlacement`, `PathPlacement`, `GlyphPlacement`, `NoteHit`, `LayoutOptions`, `FontMetrics`, `EngraveError` | `duet-engrave`, chunk A1, modules `placement` and `metrics` |
| `Score` | `duet-score`, path `duet_score::Score` |
| `TempoMap`, `Ticks` | `duet-time`, paths `duet_time::TempoMap`, `duet_time::Ticks` |

### The comparator, in `crates/duet-engrave/tests/golden.rs`

Section 10.7 states the rule: the golden corpus B79 lives in `crates/duet-engrave/tests/golden/`,
and a comparator checks every field with a stated tolerance.

```rust
/// The tolerance every position and every size is compared with, in pixels.
///
/// A device pixel is the smallest difference a reader can see, so a
/// difference below this value is not a layout change.
const POSITION_TOLERANCE_PX: f32 = 0.01;

/// Compare one engraved system with the document the corpus holds.
///
/// It reports the first field that differs, with the case name, the system
/// index, the list name, the entry index, and the two values, so a failure
/// names one field and never a whole document.
fn compare_system(case: &str, index: usize, got: &SystemPlacement, want: &SystemPlacement)
    -> Result<(), String>;
```

`SystemPlacement` derives `Debug` and `Clone` and no serde (section 10.5), so the expected document
is read into a plain mirror struct of the test module and the comparator reads the accessors of the
real type. The mirror is a test type and it never leaves `tests/golden.rs`.

### The bench, in `crates/duet-engrave/benches/layout.rs`

Section 14 states that `cargo bench -p duet-engrave` reports the layout cost at B1 against B4, and
section 10.7 states that the bench covers the layout pass and the peak path build, against B3 and
B4. B1 is 8 parts, 32 tracks, and 300 bars. B3 is 2.0 ms for the layout and the paint of one system.
B4 is 8.0 ms for a whole visible frame at B1.

```rust
/// Bench the whole layout pass over a score at B1.
fn bench_layout_at_b1(c: &mut Criterion);

/// Bench one system alone, which is what B3 bounds.
fn bench_one_system(c: &mut Criterion);

criterion_group!(benches, bench_layout_at_b1, bench_one_system);
criterion_main!(benches);
```

A bench is a report and never a gate. Section 14 states it: a regression is reviewed, not blocked,
because a bench is not stable enough to gate a merge.

## Steps

1. Read `crates/duet-engrave/Cargo.toml`. Confirm that chunks A1 to A3 left it with the five
   `{ workspace = true }` entries and no `[dev-dependencies]` section. Report a discrepancy and stop
   if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `criterion` at 0.7.0
   with the feature set Appendix B.5 states. Report a discrepancy and stop if the pin is absent.
3. Create `crates/duet-engrave/tests/golden/` and write the twelve input scores. Each one covers one
   notation case, and the twelve together cover the whole engraver. The cases are:
   1. one treble staff, four quarter notes;
   2. a grand staff with a brace;
   3. a key signature of four sharps and a key signature of four flats;
   4. a time signature change inside the score;
   5. eight eighth notes that beam in two groups;
   6. a beam group with a rest inside it;
   7. a chord with stacked accidentals;
   8. leger lines five steps above and five steps below the staff;
   9. a tie that crosses a system break;
   10. a slur over six notes with a hairpin under it;
   11. two verses of lyrics;
   12. a repeat barline pair and a rehearsal mark.
4. Write `crates/duet-engrave/tests/golden.rs` with a `#[cfg(test)] mod tests`, because
   `clippy::tests_outside_test_module` is denied. Write the twelve test functions and the comparator
   first, with no expected document on disk. Run
   `cargo nextest run -p duet-engrave --test golden --no-tests=fail` and confirm that every case
   fails, because no expected document exists.
5. Run the engraver over each input score once, and write the result to the matching
   `case-NN.expected.json`. Read each document by hand against design contract 2.3 before you commit
   it, because a golden file that nobody read is a record of a defect.
6. Run `cargo nextest run -p duet-engrave --test golden --no-tests=fail` and confirm that every case
   passes.
7. Add the `[dev-dependencies]` section and the `[[bench]]` target to
   `crates/duet-engrave/Cargo.toml`.
8. Write `crates/duet-engrave/benches/layout.rs` with the two bench functions above.
9. Run `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3). Expect a clean build and
   a changed lock file.
10. Run `cargo bench -p duet-engrave` once and record the two reported times in the commit body,
    beside B3 and B4.
11. Run `cargo nextest run -p duet-engrave --no-tests=fail` and confirm that every test passes.
12. Run `cargo clippy -p duet-engrave --all-targets --locked -- -D warnings` once and repair every
    finding. A test module may unwrap, expect, print, and index, which `clippy.toml` allows, so no
    `#[expect]` enters this chunk.
13. Commit on the branch `chunk/a4-golden-corpus-and-bench`.

## Tests

The twelve cases live in `crates/duet-engrave/tests/golden.rs`, inside a `#[cfg(test)] mod tests`.
Every assert carries a message that names the case, the system, the list, and the entry.

- `golden_single_staff_quarter_notes`
- `golden_grand_staff_with_brace`
- `golden_key_signatures_sharp_and_flat`
- `golden_time_signature_change`
- `golden_eighth_note_beam_groups`
- `golden_rest_breaks_the_beam_group`
- `golden_chord_accidental_columns`
- `golden_leger_lines_above_and_below`
- `golden_tie_across_a_system_break`
- `golden_slur_with_hairpin`
- `golden_two_lyric_verses`
- `golden_repeat_barlines_and_rehearsal_mark`

Each one reads its input score, calls `engrave` with the corpus `LayoutOptions` and the shipped
Bravura metrics, and asserts that every system equals the expected document field by field, within
`POSITION_TOLERANCE_PX`.

Two more tests guard the corpus itself.

- `golden_corpus_holds_twelve_cases` — reads the directory and asserts that it holds twelve input
  files and twelve expected files, which is B79. The test fails when somebody adds a case and
  forgets its pair.
- `golden_comparator_reports_the_first_difference` — perturbs one x value by 1.0 pixel and asserts
  that the comparator returns an error whose message names that list and that entry index.

No property test enters this chunk. The proptest dev-dependency belongs to `duet-time` and
`duet-interchange`, which Appendix B.3 records.

The bench carries no assert, because it is a report and not a gate.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-engrave --test golden --no-tests=fail
cargo nextest run -p duet-engrave --no-tests=fail
cargo bench -p duet-engrave
cargo clippy -p duet-engrave --all-targets --locked -- -D warnings
cargo machete
```

The first command fails before this chunk, because the target `golden` does not exist, and
`--no-tests=fail` makes an empty match a failure (SM3 rule 1). It passes after this chunk and it
reports fourteen tests. The second command reports every test of the crate as passed. The third
command prints the two layout times; record them beside B3 and B4. The fourth command prints no
warning. The fifth command reports no unused dependency, because the bench uses `criterion` in the
same commit that adds it (SM1).

The work lands as one commit on the branch `chunk/a4-golden-corpus-and-bench`, with a conventional
subject such as `test(engrave): add the golden corpus, the comparator, and the layout bench`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs
  `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted
  diagnostic command is allowed after a hook failure. Never `--no-verify`.
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
