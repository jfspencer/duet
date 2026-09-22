---
id: X2
line: X
depends_on: [X1]
write_scope:
  - crates/duet-interchange/src/musicxml/write.rs
  - crates/duet-interchange/tests/round_trip.rs
  - crates/duet-interchange/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-interchange --test round_trip --no-tests=fail"
---

# X2: The MusicXML writer and the round-trip property test

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Phase 5 carries no manifest chunk, so this chunk depends on chunk X1 alone (section
13.3). Chunk X1 created `crates/duet-interchange/src/musicxml/write.rs` as a stub in phase 4 and
declared `write_musicxml` with a body that answers `InterchangeError::Unsupported`. This chunk fills
the module and replaces that body. It writes a score as MusicXML, it returns every unmapped element
to its place, and it proves the round trip with a property test. It implements architecture sections
3.7 and 15.7 and ADR `adr/0002-storage-format.md`, and it carries the writer half of the product
story A-03 and of PR 11 Q9.

PR 11 Q9 records that Duet does not print and does not write PDF. MusicXML export is the print path,
and the export surface says so.

This chunk adds one dev-dependency, so SM1 puts `crates/duet-interchange/Cargo.toml` in its write
scope and SM5 rule 2 puts `Cargo.lock` there beside it. Section 13.2 names both files in this chunk's
Writes cell.

## Files

- `crates/duet-interchange/src/musicxml/write.rs` — modify. The streaming writer.
- `crates/duet-interchange/tests/round_trip.rs` — create. The round-trip property test.
- `crates/duet-interchange/Cargo.toml` — modify. Add the `proptest` dev-dependency.
- `Cargo.lock` — modify.

SM2 covers `src/` alone, so an integration test file under `tests/` is its own crate root, needs no
`mod` line, and is created by the chunk that writes it. Section 13.2 names `tests/round_trip.rs` in
this chunk's Writes cell for that reason.

`crates/duet-interchange/src/musicxml.rs` is outside this chunk's write scope, so the
`write_musicxml` signature that chunk X1 declared stays as it is. This chunk changes the body of
`crate::musicxml::write` alone, which `write_musicxml` already calls.

## Types and signatures

### Manifest

```toml
# crates/duet-interchange/Cargo.toml, [dev-dependencies], added by this chunk
proptest = { workspace = true }
```

Appendix B.3 pins `proptest` at 1.11.0 and gives it the manifest owner M0 and the chunks T1 and X2.
SM4 forbids this chunk to edit the root manifest, so a missing pin is a discrepancy that this chunk
reports under SM0.

### From architecture section 3.7, module `duet_interchange::musicxml`

```rust
/// Write a score as MusicXML, and return the unmapped elements to their place.
///
/// # Errors
/// Returns `InterchangeError::Serialize` when a value has no MusicXML form.
pub fn write_musicxml(score: &Score, unmapped: &UnmappedElements) -> Result<Vec<u8>, InterchangeError>;
```

### Module `duet_interchange::musicxml::write`

Section 1.5 places no type in this module, so the module holds functions alone.

```rust
/// Write the whole document into `out`.
///
/// It writes `score-partwise` in schema order: `part-list` first, then one
/// `part` per part, then one `measure` per measure of that part. Every
/// element the bucket holds for a path is written back at that path, so the
/// round trip of section 3.7 returns it.
///
/// # Errors
/// Returns `InterchangeError::Serialize` when a value has no MusicXML form,
/// and when `quick_xml` refuses a write.
pub fn write_document(
    score: &Score,
    unmapped: &UnmappedElements,
    out: &mut Vec<u8>,
) -> Result<(), InterchangeError>;

/// Write one `note` element and every child the subset models.
///
/// The children are written in the schema order chunk X1's `read_note`
/// reads them in: `grace`, `pitch`, `rest`, `duration`, `tie`, `voice`,
/// `type`, `dot`, `accidental`, `stem`, `lyric`. The two functions share
/// that order, so a document this writer produces reads back with no
/// warning.
///
/// # Errors
/// Returns `InterchangeError::Serialize` for a value with no MusicXML form.
pub fn write_note(
    writer: &mut Writer<&mut Vec<u8>>,
    score: &Score,
    note: NoteId,
    unmapped: &UnmappedElements,
) -> Result<(), InterchangeError>;
```

`Writer` is `quick_xml::Writer`.

### Consumed types

| Type | Crate and path |
|---|---|
| `MusicXmlImport`, `UnmappedElements`, `InterchangeError`, `read_musicxml` | `duet-interchange`, chunk X1 |
| `Score`, `NoteId`, `ScoreError` | `duet-score` |
| `ImportWarning` | `duet-command`, path `duet_command::ImportWarning` |
| `Ticks`, `TempoMap`, `NoteValue` | `duet-time` |

## Steps

1. Read `crates/duet-interchange/src/musicxml/write.rs` and `crates/duet-interchange/src/musicxml.rs`.
   Confirm that `write.rs` holds a `//!` line alone and that `write_musicxml` answers
   `InterchangeError::Unsupported`. Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `proptest` at 1.11.0.
   Report a discrepancy and stop if the pin is absent.
3. Create `crates/duet-interchange/tests/round_trip.rs` with a `#[cfg(test)] mod tests`, because
   `clippy::tests_outside_test_module` is denied. Write the failing test
   `round_trip_keeps_every_modelled_value` inside it. Run
   `cargo nextest run -p duet-interchange --test round_trip --no-tests=fail` and confirm that the run
   fails, because `write_musicxml` answers `InterchangeError::Unsupported`.
4. Add the `[dev-dependencies]` section to `crates/duet-interchange/Cargo.toml`. Run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3).
5. Implement `write_document` in `src/musicxml/write.rs` over `quick_xml::Writer`. Write
   `part-list`, then one `part` per part, then one `measure` per measure. Write the key signature,
   the time signature, and the clef of each measure that changes one.
6. Implement `write_note` with the eleven children in the schema order chunk X1's `read_note` reads.
   Write every bucket entry of the note's own path after the modelled children, so the reader that
   opens the result finds them where it left them.
7. Replace the `write_musicxml` body in `src/musicxml.rs`. That file is outside this chunk's write
   scope, so confirm first that chunk X1 already wrote `write_musicxml` as a call into
   `crate::musicxml::write::write_document`. Report a discrepancy and stop if it did not.
8. Run the same command and confirm that `round_trip_keeps_every_modelled_value` passes.
9. Write the remaining tests of the Tests section, with the proptest strategies the section names.
   Run the same command and confirm that every one passes.
10. Run `cargo nextest run -p duet-interchange --no-tests=fail` and confirm that every test passes,
    which includes the twelve `read_musicxml` tests of chunk X1.
11. Run `cargo clippy -p duet-interchange --all-targets --locked -- -D warnings` once and repair
    every finding.
12. Commit on the branch `chunk/x2-musicxml-writer-round-trip`.

## Tests

The tests live in `crates/duet-interchange/tests/round_trip.rs`, wrapped in a
`#[cfg(test)] mod tests`. Every assert carries a message. The crate stays pure, so every fixture is
built in memory and no test opens a file.

### The proptest strategies

Each strategy is a named function of the test module, so a failure names the generator and a second
test can reuse it.

- `arb_pitch()` — a `duet_score::Pitch` over the seven steps, the five accidentals, and octaves 0 to
  8.
- `arb_duration()` — a `duet_score::Duration` over the six note values of design contract 2.7, with
  zero, one, or two dots.
- `arb_lyric()` — a `duet_score::Lyric` with a verse number of 1 to 4 and a syllable of one to eight
  ASCII letters, plus the hyphenated flag.
- `arb_note()` — one `duet_score::Note` from `arb_pitch`, `arb_duration`, an optional `Dynamic`, an
  optional `Articulation`, and an optional `arb_lyric`.
- `arb_measure()` — one measure of one to twelve values from `arb_note`, whose durations sum to the
  measure length.
- `arb_score()` — one `Score` of one or two parts, one or two staves per part, and one to eight
  measures from `arb_measure`, with a key signature of minus 7 to plus 7 and a meter from the eight
  meters the subset models.
- `arb_unmapped()` — an `UnmappedElements` of zero to six entries, each one a well-formed XML
  fragment under a path `arb_score` produces.

### The tests

- `round_trip_keeps_every_modelled_value` — a proptest over `arb_score`. It writes the score, reads
  the bytes back, and asserts that the returned `Score` equals the input. B78 gives the case counts:
  1,000 in the gate.
- `round_trip_returns_every_unmapped_element` — a proptest over `arb_score` and `arb_unmapped`. It
  writes, reads back, and asserts that the returned `UnmappedElements` equals the input, which is the
  lossless bucket property of section 3.7.
- `round_trip_produces_no_warning` — a proptest over `arb_score`. It asserts that
  `MusicXmlImport::warnings` is empty for a document this writer produced, which proves that the
  writer and the reader share one schema order.
- `round_trip_is_deterministic` — a proptest over `arb_score`. It writes the same score twice and
  asserts equal bytes.
- `round_trip_refuses_a_value_with_no_musicxml_form` — builds a score with a tuplet ratio MusicXML
  cannot express and asserts `InterchangeError::Serialize`.
- `round_trip_output_is_well_formed_xml` — a proptest over `arb_score`. It parses the output with
  `quick_xml::Reader` to the end and asserts no parse error.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-interchange --test round_trip --no-tests=fail
cargo nextest run -p duet-interchange --no-tests=fail
cargo clippy -p duet-interchange --all-targets --locked -- -D warnings
cargo machete
```

The first command fails before this chunk, because the target `round_trip` does not exist, and
`--no-tests=fail` makes an empty match a failure (SM3 rule 1). It passes after this chunk and it
reports six tests. The second command reports every test of the crate as passed, which is the twelve
of chunk X1 plus these six. The third command prints no warning, and this chunk carries no new
`#[expect]` site. The fourth command reports no unused dependency, because the test uses `proptest`
in the same commit that adds it (SM1).

The work lands as one commit on the branch `chunk/x2-musicxml-writer-round-trip`, with a conventional
subject such as `feat(interchange): write MusicXML and prove the round trip`.

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
