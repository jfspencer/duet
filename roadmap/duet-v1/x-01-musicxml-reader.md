---
id: X1
line: X
depends_on: [M4, T2, T4]
write_scope:
  - crates/bc_notation/duet-interchange/lang_rust/Cargo.toml
  - crates/bc_notation/duet-interchange/lang_rust/src/lib.rs
  - crates/bc_notation/duet-interchange/lang_rust/src/musicxml.rs
  - crates/bc_notation/duet-interchange/lang_rust/src/smf.rs
  - crates/bc_notation/duet-interchange/lang_rust/src/musicxml/read.rs
  - crates/bc_notation/duet-interchange/lang_rust/src/musicxml/write.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-interchange -E 'test(read_musicxml)' --no-tests=fail"
---

# X1: The MusicXML reader for the modelled subset, with the unmapped bucket

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk opens line X over the crate `duet-interchange`. The line was `B` until
revision 10, and section 13.2 states why it is `X` now: chunk `B1`, `B2`, and `B3` collided with the
budget ids B1, B2, and B3 of section 1.6. Chunk M4 creates the crate skeleton in phase 4, so
`crates/bc_notation/duet-interchange/lang_rust/Cargo.toml` and `crates/bc_notation/duet-interchange/lang_rust/src/lib.rs` exist before this chunk
starts and this chunk modifies both. Every other file of the write scope does not exist yet, and this
chunk creates it. The chunk declares `MusicXmlImport`, `UnmappedElements`, and `InterchangeError`, and
it reads a MusicXML document into a `Score`. It implements architecture sections 3.6, 3.7, and 15.7
and ADR `adr/0002-storage-format.md`, and it carries the reader half of the product story A-03 and
of PR 11 Q9.

Section 13.2 gives this chunk the second duty of SM2: it creates every module file of line X at every
depth as a stub, so chunks X2 and X3 modify a stub and create no source file.

Section 1.2 states the one invariant this crate protects: a foreign schema never reaches the
canonical model.

## Files

- `crates/bc_notation/duet-interchange/lang_rust/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk
  uses.
- `crates/bc_notation/duet-interchange/lang_rust/src/lib.rs` — modify. Add the two `mod` lines, `UnmappedElements`, and
  `InterchangeError`.
- `crates/bc_notation/duet-interchange/lang_rust/src/musicxml.rs` — create. The sibling file of the `musicxml/` directory,
  which `clippy::mod_module_files` requires (SM2). It holds the two `mod` lines, `MusicXmlImport`,
  `read_musicxml`, and `write_musicxml`.
- `crates/bc_notation/duet-interchange/lang_rust/src/smf.rs` — create as a stub. Chunk X3 fills it.
- `crates/bc_notation/duet-interchange/lang_rust/src/musicxml/read.rs` — create and fill. The streaming reader.
- `crates/bc_notation/duet-interchange/lang_rust/src/musicxml/write.rs` — create as a stub. Chunk X2 fills it.
- `Cargo.lock` — modify. SM5 rule 2 puts it in the write scope of every chunk that writes a member
  manifest.

## Types and signatures

### Manifest

```toml
# crates/bc_notation/duet-interchange/lang_rust/Cargo.toml, [dependencies]
duet-time = { workspace = true }
duet-score = { workspace = true }
duet-command = { workspace = true }
quick-xml = { workspace = true }
thiserror = { workspace = true }
```

Section 1.2 gives `duet-interchange` the third-party set `quick-xml`, `midly`, and `thiserror`.
`midly` reaches only chunk X3, so this chunk adds no `midly` entry and `cargo machete` passes. Chunk
M4 pins `quick-xml` at 0.42.0 in phase 4, which Appendix B.3 states. Section 1.3 gives the three
internal edges `duet-interchange -> duet-time`, `duet-interchange -> duet-score`, and
`duet-interchange -> duet-command`. Section 13.4 states the link "T4 before X1", with the reason
"`MusicXmlImport` and `SmfImport` each carry `Vec<ImportWarning>`, which is the `duet-interchange ->
duet-command` edge of section 1.3, and X1 adds the entry", and the link "T2 before X1", with the
reason "The reader builds a `Score`". SM4 forbids this chunk to edit the root manifest, so a missing
pin is a discrepancy that this chunk reports under SM0.

### From architecture section 15.7, module `duet_interchange` root

```rust
/// Every element a MusicXML import could not model, keyed by its parent path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnmappedElements { entries: BTreeMap<Box<str>, Vec<Box<str>>> }

/// Every way an interchange read or write refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InterchangeError { Parse(Box<str>), Unsupported(Box<str>), Serialize(Box<str>), Score(ScoreError) }
```

### From architecture section 3.7, module `duet_interchange::musicxml`

```rust
/// Read a MusicXML document into a score.
///
/// # Errors
/// Returns `InterchangeError` for malformed XML or an unsupported version.
pub fn read_musicxml(bytes: &[u8]) -> Result<MusicXmlImport, InterchangeError>;

/// The result of a MusicXML import.
#[derive(Debug, Clone)]
pub struct MusicXmlImport {
    pub score: Score,
    pub unmapped: UnmappedElements,
    pub warnings: Vec<ImportWarning>,
}

/// Write a score as MusicXML, and return the unmapped elements to their place.
///
/// # Errors
/// Returns `InterchangeError::Serialize` when a value has no MusicXML form.
pub fn write_musicxml(score: &Score, unmapped: &UnmappedElements) -> Result<Vec<u8>, InterchangeError>;
```

This chunk declares `write_musicxml` and returns `InterchangeError::Unsupported` from it, so the
signature exists for chunk X2 to fill and no caller of phase 4 reaches a missing item. Chunk X2
replaces the body and changes no signature.

### Consumed types

| Type | Crate and path |
|---|---|
| `Score`, `ScoreError`, `ScoreCommand`, `Part`, `Staff`, `Voice`, `Measure`, `Note`, `Rest`, `Spanner`, `SpannerKind`, `Lyric`, `LyricText`, `VerseNumber`, `Dynamic`, `Articulation`, `Clef`, `KeySignature`, `TieState`, `PartName`, `PartId`, `StaffId`, `VoiceId`, `MeasureId`, `NoteId`, `SpannerId` | `duet-score`, sections 3.3 and 15.3 |
| `ImportWarning` | `duet-command`, path `duet_command::ImportWarning` |
| `Ticks`, `TempoMap`, `NoteValue`, `Meter`, `Tempo` | `duet-time` |

From architecture section 15.5:

```rust
/// One unknown key a reader kept, with the path that carried it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportWarning { path: Box<str>, key: Box<str>, detail: Box<str> }
```

### Module `duet_interchange::musicxml::read`

Section 3.7 states the subset: parts, staves, measures, notes, rests, key and time signatures, ties,
slurs, dynamics, and lyrics. On import the reader keeps every unmodelled element in a lossless bucket,
keyed by the parent element path, so an export round trip returns them.

```rust
/// Read one `note` element and every child it carries.
///
/// # Errors
/// Returns `InterchangeError::Parse` for a malformed child, and
/// `InterchangeError::Score` when the note breaks a score invariant.
#[expect(
    too_many_lines,
    reason = "one match over the eleven child elements MusicXML 4 allows inside `note`, in \
              schema order, with the order test inside the same match"
)]
pub fn read_note(
    reader: &mut Reader<&[u8]>,
    context: &mut ReadContext,
) -> Result<(), InterchangeError>;

/// What the reader carries across one document.
///
/// It holds the score under construction, the unmapped bucket, the warning
/// list, and the identifier maps that turn a MusicXML name into a
/// `duet-score` identifier.
#[derive(Debug, Default)]
pub struct ReadContext {
    score: Score,
    unmapped: UnmappedElements,
    warnings: Vec<ImportWarning>,
    parts: BTreeMap<Box<str>, PartId>,
    staves: BTreeMap<(PartId, u8), StaffId>,
    voices: BTreeMap<(StaffId, u8), VoiceId>,
}

/// Read the whole document.
///
/// # Errors
/// Returns `InterchangeError::Parse` for malformed XML, and
/// `InterchangeError::Unsupported` for a MusicXML version this build does
/// not read.
pub fn read_document(bytes: &[u8]) -> Result<ReadContext, InterchangeError>;
```

`Reader` is `quick_xml::Reader`. `ReadContext` is a module type of `duet-interchange` that no other
crate names; report the discrepancy to the Architect if `cargo xtask check-placement` refuses it.

The `#[expect]` attribute on `read_note` is the Appendix B.1 row
`duet-interchange::musicxml::read::read_note`, and the reason text above is that row's text, copied.
The eleven child elements MusicXML 4 allows inside `note`, in schema order, are `grace`, `pitch`,
`rest`, `duration`, `tie`, `voice`, `type`, `dot`, `accidental`, `stem`, and `lyric`. Name every one
in the match, because `clippy::wildcard_enum_match_arm` is denied, and route every other child into
`UnmappedElements`.

Section 3.6 property 4 gives the warning rule: a reader keeps an unknown key in the `extra` bag as an
`ImportWarning`, and the command-line interface prints it.

## Steps

1. Read `crates/bc_notation/duet-interchange/lang_rust/Cargo.toml` and `crates/bc_notation/duet-interchange/lang_rust/src/lib.rs`. Confirm that
   chunk M4 created both, that the manifest carries `[lints] workspace = true`, a `description`, and
   no `[dependencies]` section, and that `lib.rs` carries the `//!` crate documentation and
   `#![forbid(unsafe_code)]`. Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `quick-xml` at 0.42.0 and
   carries the `duet-time`, `duet-score`, and `duet-command` entries. Report a discrepancy and stop if
   an entry is absent.
3. Create the four module files. Fill `musicxml.rs` and `musicxml/read.rs`. Leave `smf.rs` and
   `musicxml/write.rs` at one `//!` line each, except for the `write_musicxml` signature, which
   `musicxml.rs` declares and which answers `InterchangeError::Unsupported` until chunk X2.
4. Add the two `mod` lines to `src/lib.rs` and declare `UnmappedElements` and `InterchangeError`.
5. Add the five `{ workspace = true }` entries to `crates/bc_notation/duet-interchange/lang_rust/Cargo.toml`. Run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3).
6. Write the failing test `read_musicxml_builds_a_one_part_score` in `src/musicxml/read.rs`, inside a
   `#[cfg(test)] mod tests`. Section 13.2 gives this chunk no `tests/` file, so every test of this
   chunk lives in the same file as the code it covers. Run
   `cargo nextest run -p duet-interchange -E 'test(read_musicxml)' --no-tests=fail` and confirm that
   the run fails to compile.
7. Declare `ReadContext`. Implement `read_document` as a streaming pass over
   `quick_xml::Reader::read_event_into`. Refuse a `score-partwise` version the build does not read
   with `InterchangeError::Unsupported`.
8. Implement `read_note` with the eleven-arm match and the `#[expect]` attribute. Route every
   unmodelled child into `UnmappedElements::entries`, keyed by the parent element path.
9. Run the same command and confirm that the test passes.
10. Write the remaining `read_musicxml` tests of the Tests section. Run the same command and confirm
    that every one passes.
11. Run `cargo nextest run -p duet-interchange --no-tests=fail` and confirm that every test passes.
12. Run `cargo clippy -p duet-interchange --all-targets --locked -- -D warnings` once and repair
    every finding. Confirm that the `#[expect]` attribute on `read_note` is fulfilled, because
    `-D warnings` turns an unfulfilled expectation into an error.
13. Run `cargo deny check` once and confirm that `quick-xml` passes the licence policy. Record the
    output in the commit body.
14. Commit on the branch `chunk/x1-musicxml-reader`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, because section 13.2 gives this
chunk no `tests/` file. Every assert carries a message. Every fixture document is a `const` string in
the test module, so the crate stays pure and opens no file; section 1.4 names `duet-interchange` a
pure crate for that reason.

`crates/bc_notation/duet-interchange/lang_rust/src/musicxml/read.rs`

- `read_musicxml_builds_a_one_part_score` — reads a document with one part, one measure, and four
  quarter notes, and asserts one `Part`, one `Staff`, one `Measure`, and four `Note` values.
- `read_musicxml_reads_a_grand_staff` — asserts two `Staff` values under one `Part`, with the treble
  staff first.
- `read_musicxml_reads_key_and_time_signatures` — asserts a key of four sharps and a meter of 3/4.
- `read_musicxml_reads_rests` — asserts one `Rest` with the duration the document states.
- `read_musicxml_reads_ties_and_slurs` — asserts one `TieState` pair and one `Spanner` of the slur
  kind.
- `read_musicxml_reads_dynamics_and_lyrics` — asserts one `Dynamic::F` and two `Lyric` syllables in
  two verses.
- `read_musicxml_keeps_an_unmapped_element` — reads a document with a `<harmony>` element inside a
  measure and asserts that `UnmappedElements::entries` holds it under the measure's path.
- `read_musicxml_reports_an_unknown_key_as_a_warning` — asserts one `ImportWarning` whose `path`,
  `key`, and `detail` name the element the reader kept.
- `read_musicxml_refuses_malformed_xml` — asserts `InterchangeError::Parse`.
- `read_musicxml_refuses_an_unsupported_version` — asserts `InterchangeError::Unsupported`.
- `read_musicxml_refuses_a_note_that_breaks_an_invariant` — reads two notes at one onset in one voice
  and asserts `InterchangeError::Score(ScoreError::OverlappingNote { .. })`, which proves that a
  foreign schema never reaches the canonical model.
- `read_musicxml_note_children_are_in_schema_order` — reads a document whose `note` children are out
  of schema order and asserts one `ImportWarning` that names the order, which is the order test the
  `#[expect]` reason cites.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-interchange -E 'test(read_musicxml)' --no-tests=fail
cargo nextest run -p duet-interchange --no-tests=fail
cargo clippy -p duet-interchange --all-targets --locked -- -D warnings
cargo deny check
cargo machete
```

The first command fails before this chunk, because the crate holds no `read_musicxml` test, and
`--no-tests=fail` makes an empty match a failure (SM3 rule 1). It passes after this chunk and it
reports twelve tests. The second command reports every test of the crate as passed. The third command
prints no warning, and the one `#[expect]` site is the Appendix B.1 row for `read_note`. The fourth
command reports no licence failure and no advisory. The fifth command reports no unused dependency.

The work lands as one commit on the branch `chunk/x1-musicxml-reader`, with a conventional subject
such as `feat(interchange): read MusicXML into the score, with a lossless unmapped bucket`.

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
