---
id: X3
line: X
depends_on: [X2]
write_scope:
  - crates/bc_notation/duet-interchange/lang_rust/src/smf.rs
  - crates/bc_notation/duet-interchange/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-interchange -E 'test(smf_import)' --no-tests=fail"
---

# X3: Standard MIDI File import with the quantize options

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Phase 6 carries no manifest chunk, so this chunk depends on chunk X2 alone (section
13.3). Chunk X1 created `crates/bc_notation/duet-interchange/lang_rust/src/smf.rs` as a stub in phase 4, so the file holds
a `//!` line alone. This chunk fills it. It reads a Standard MIDI File into a `Score` and a
`TempoMap`, with the quantize option, the part map, and the tempo choice. One function serves the
command-line interface, the MCP verb, and the menu item (section 8.4). The chunk implements
architecture sections 8.4 and 15.7, and it carries the product story A-05, "MIDI file import by
agent", which is a MUST, and the model half of C-12, "MIDI file import in the user interface", which
is a SHOULD.

This chunk adds one dependency, so SM1 puts `crates/bc_notation/duet-interchange/lang_rust/Cargo.toml` in its write scope
and SM5 rule 2 puts `Cargo.lock` there beside it. Section 13.2 names both files in this chunk's
Writes cell.

## Files

- `crates/bc_notation/duet-interchange/lang_rust/src/smf.rs` — modify. The Standard MIDI File reader.
- `crates/bc_notation/duet-interchange/lang_rust/Cargo.toml` — modify. Add `midly`.
- `Cargo.lock` — modify.

## Types and signatures

### Manifest

```toml
# crates/bc_notation/duet-interchange/lang_rust/Cargo.toml, [dependencies], added by this chunk
midly = { workspace = true }
```

Chunk M4 pins `midly` in phase 4, which section 13.1 states. `midly` carries the Unlicense, which
`deny.toml` must allow; Appendix B.4 states that chunk M0 applies that edit, because `deny.toml` is a
policy file (SM4). Read `deny.toml` and confirm that `Unlicense` is on the allow list before you add
the entry. SM4 forbids this chunk to edit the root manifest and `deny.toml`, so a missing pin or a
missing licence entry is a discrepancy that this chunk reports under SM0.

### From architecture section 8.4, module `duet_interchange::smf`

```rust
/// Read a Standard MIDI File into a score and a tempo map.
///
/// # Errors
/// Returns `InterchangeError::Parse` for a malformed file, and
/// `InterchangeError::Unsupported` for a format this build does not read.
pub fn import_smf(bytes: &[u8], options: &SmfImportOptions)
    -> Result<SmfImport, InterchangeError>;

/// What an import produced.
#[derive(Debug, Clone)]
pub struct SmfImport { pub score: Score, pub tempo_map: TempoMap, pub warnings: Vec<ImportWarning> }
```

### Consumed types

| Type | Crate and path |
|---|---|
| `InterchangeError` | `duet-interchange`, chunk X1, crate root |
| `SmfImportOptions`, `PartMapping`, `TempoImport`, `ImportWarning` | `duet-command`, section 15.5 |
| `Score`, `ScoreError`, `PartId`, `StaffId`, `VoiceId`, `MeasureId`, `NoteId`, `Note`, `Pitch`, `Duration` | `duet-score` |
| `TempoMap`, `TempoMapEdit`, `Ticks`, `NoteValue`, `Tempo`, `Meter`, `TimeError` | `duet-time` |

From architecture section 8.4, declared in `duet-command`:

```rust
/// How to map a Standard MIDI File onto the score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmfImportOptions {
    /// Snap every onset and duration to this value. `None` keeps the raw ticks.
    quantize: Option<NoteValue>,
    /// Which track goes to which part.
    part_map: PartMapping,
    /// Take the tempo map from the file, or keep the current one.
    tempo: TempoImport,
}
```

Section 1.3 states the edge `duet-interchange -> duet-command`, with the reason "`MusicXmlImport` and
`SmfImport` each carry `Vec<ImportWarning>`". Chunk X1 added that entry in phase 4, so this chunk
adds it again for no crate and `cargo machete` passes on the one new entry alone.

### Module `duet_interchange::smf`

Section 1.5 places `SmfImport` in this crate and no other type, so the module holds `SmfImport`,
`import_smf`, and the module functions below.

```rust
/// The ticks of one Standard MIDI File tick, in `duet-time` ticks.
///
/// The file states its own division, and `TICKS_PER_QUARTER` is B40, at
/// 1920. The conversion is exact for every division that divides 1920, and
/// it rounds for every other one; a rounded onset produces one
/// `ImportWarning`.
///
/// # Errors
/// Returns `InterchangeError::Unsupported` for an SMPTE division, which this
/// build does not read.
pub fn ticks_per_file_tick(division: midly::Timing) -> Result<Ticks, InterchangeError>;

/// Snap one position to the quantize value.
///
/// Section 2.4 gives the tuplet rounding rule, and this function uses the
/// same `duet_time::Rounding` mode, so a quantized import and a tuplet split
/// round the same way.
#[must_use]
pub fn quantize_to(at: Ticks, value: NoteValue) -> Ticks;

/// Build the tempo map from the file's own tempo and meter events.
///
/// # Errors
/// Returns `InterchangeError::Parse` when the events break the first-point
/// rule or the sort order that `TempoMapEdit::finish` enforces.
pub fn read_tempo_map(smf: &midly::Smf<'_>) -> Result<TempoMap, InterchangeError>;
```

## Steps

1. Read `crates/bc_notation/duet-interchange/lang_rust/src/smf.rs`. Confirm that it holds a `//!` line and nothing else.
   Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml` and `deny.toml`. Confirm that `[workspace.dependencies]` pins `midly`
   and that the `deny.toml` allow list carries `Unlicense`. Report a discrepancy and stop if either
   one is absent.
3. Write the failing test `smf_import_reads_one_track` in `src/smf.rs`, inside a
   `#[cfg(test)] mod tests`. Section 13.2 gives this chunk no `tests/` file, so every test of this
   chunk lives in this file. Run
   `cargo nextest run -p duet-interchange -E 'test(smf_import)' --no-tests=fail` and confirm that the
   run fails to compile.
4. Add the `midly` entry to `crates/bc_notation/duet-interchange/lang_rust/Cargo.toml`. Run `cargo build --workspace`,
   which settles `Cargo.lock` (SM5 rule 3).
5. Implement `ticks_per_file_tick`. Refuse an SMPTE division with `InterchangeError::Unsupported`.
   Use `u32::checked_mul` and `u32::checked_div`, because `clippy::integer_division` is denied and
   `as` is denied outside `duet-time::convert`.
6. Implement `quantize_to` over `duet_time::Rounding`.
7. Implement `read_tempo_map`. Build a `TempoMapEdit`, push every tempo point and every meter point
   the file states, and call `finish`. Convert every `TimeError` into `InterchangeError::Parse` with
   a named `map_err` that keeps the source text, because `clippy::map_err_ignore` is denied.
8. Implement `import_smf`. It parses the bytes with `midly::Smf::parse`, it maps each track to a part
   through `PartMapping`, it pairs every note-on with its note-off, it quantizes when the option
   asks, and it builds the `Score` through `duet_score::Score::apply`, so no foreign schema reaches
   the canonical model. It honours `TempoImport`: it returns the file's map or the caller's current
   map.
9. Run the same command and confirm that the test passes.
10. Write the remaining `smf_import` tests of the Tests section. Run the same command and confirm
    that every one passes.
11. Run `cargo nextest run -p duet-interchange --no-tests=fail` and confirm that every test passes,
    which includes the twelve `read_musicxml` tests of chunk X1 and the six round-trip tests of chunk
    X2.
12. Run `cargo clippy -p duet-interchange --all-targets --locked -- -D warnings` once and repair
    every finding.
13. Run `cargo deny check` once and confirm that `midly` passes the licence policy under the
    `Unlicense` entry. Record the output in the commit body.
14. Commit on the branch `chunk/x3-smf-import`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in `crates/bc_notation/duet-interchange/lang_rust/src/smf.rs`, because
section 13.2 gives this chunk no `tests/` file. Every assert carries a message. Every fixture file is
built in memory with `midly`, so the crate stays pure and opens no file; section 1.4 names
`duet-interchange` a pure crate for that reason.

- `smf_import_reads_one_track` — builds a format 0 file with four quarter notes and asserts four
  `Note` values at the onsets the file states.
- `smf_import_maps_each_track_to_its_part` — builds a format 1 file with four tracks and a
  `PartMapping` that names Soprano, Alto, Tenor, and Bass, and asserts one part per track.
- `smf_import_quantizes_to_the_named_value` — builds a file whose onsets sit 7 ticks late, imports
  with `quantize` set to an eighth note, and asserts that every onset lands on the eighth-note grid.
- `smf_import_keeps_raw_ticks_with_no_quantize` — asserts that the same file with `quantize` set to
  `None` keeps every onset as the file states it.
- `smf_import_takes_the_tempo_map_from_the_file` — asserts the tempo points the file carries when
  `TempoImport` asks for the file's map.
- `smf_import_keeps_the_current_tempo_map` — asserts that the returned map equals the caller's map
  when `TempoImport` asks to keep it.
- `smf_import_reads_a_division_that_divides_b40` — asserts an exact conversion for 96, 192, 384, and
  480 ticks per quarter, which all divide 1920.
- `smf_import_warns_on_a_rounded_division` — asserts one `ImportWarning` for a division of 100, which
  does not divide 1920.
- `smf_import_refuses_an_smpte_division` — asserts `InterchangeError::Unsupported`.
- `smf_import_refuses_malformed_bytes` — asserts `InterchangeError::Parse`.
- `smf_import_pairs_every_note_on_with_a_note_off` — builds a file with one unmatched note-on and
  asserts one `ImportWarning` and no note of zero length.
- `smf_import_refuses_an_overlapping_note` — builds two note-on events at one onset in one voice and
  asserts `InterchangeError::Score(ScoreError::OverlappingNote { .. })`, which proves that the import
  goes through `Score::apply`.
- `smf_import_is_deterministic` — imports one file twice and asserts an equal `Score` and an equal
  `TempoMap`.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-interchange -E 'test(smf_import)' --no-tests=fail
cargo nextest run -p duet-interchange --no-tests=fail
cargo clippy -p duet-interchange --all-targets --locked -- -D warnings
cargo deny check
cargo machete
```

The first command fails before this chunk, because the crate holds no `smf_import` test. Chunks X1
and X2 wrote their tests as `read_musicxml_*` and `round_trip_*` for that reason (SM3 rule 2). The
command passes after this chunk and it reports thirteen tests. The second command reports every test
of the crate as passed. The third command prints no warning, and this chunk carries no new
`#[expect]` site. The fourth command reports no licence failure and no advisory, which proves the
`Unlicense` entry chunk M0 added. The fifth command reports no unused dependency, because this chunk
adds the entry in the same commit as the code that uses it (SM1).

The work lands as one commit on the branch `chunk/x3-smf-import`, with a conventional subject such as
`feat(interchange): import a Standard MIDI File with quantize options`.

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
- A new crate lives at `crates/bc_<context>/<crate>/lang_rust/` in the context that architecture section 1.2 names (ADR 0011), declares `[lints] workspace = true`, inherits every
  `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the
  root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- After chunk M94 lands, every `.rs` file a commit writes carries one front-matter block (`cargo xtask check-ddd --write`), and the gate refuses a changed `.rs` file with none.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no
  pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with
  Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this
  repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match
  what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
