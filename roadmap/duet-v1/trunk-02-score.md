---
id: T2
line: trunk
depends_on: [M1, T1]
write_scope:
  - crates/bc_document/duet-score/lang_rust/Cargo.toml
  - crates/bc_document/duet-score/lang_rust/src/lib.rs
  - crates/bc_document/duet-score/lang_rust/src/model.rs
  - crates/bc_document/duet-score/lang_rust/src/ids.rs
  - crates/bc_document/duet-score/lang_rust/src/command.rs
  - crates/bc_document/duet-score/lang_rust/src/event.rs
  - crates/bc_document/duet-score/lang_rust/src/apply.rs
  - crates/bc_document/duet-score/lang_rust/src/canonical.rs
  - crates/bc_document/duet-score/lang_rust/src/error.rs
  - crates/bc_document/duet-score/lang_rust/tests/canonical_determinism.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-score --no-tests=fail passes; cargo clippy -p duet-score --all-targets -- -D warnings is clean; commit SHA on a branch chunk/t2-score"
---

# T2: The score aggregate

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk builds `duet-score`: the score aggregate root, its entities and value objects,
`ScoreCommand`, `ScoreEvent`, `Applied`, the transactional `apply`, and the canonical reader and
writer. It implements architecture sections 3.1, 3.2, 3.3, 3.4, 3.6, and 15.3, and ADR
`adr/0002-storage-format.md`. Section 13.1 states the goal, the write scope, and the Completion
command. Section 14 selects `canonical_determinism` for rung-two command 4, so this chunk writes
that integration target.

Section 13.4 puts `T1 before T2`, because every declaration here names `Ticks`, `Position`, or
`SchemaVersion`. Chunk M1 creates the `crates/bc_document/duet-score/lang_rust` skeleton (SM1 rule 2), so the crate root
and the member manifest already exist. Dispatch: **Duet Engineer**.

## Files

| Path | Action |
|---|---|
| `crates/bc_document/duet-score/lang_rust/Cargo.toml` | modify (add `[dependencies]`) |
| `crates/bc_document/duet-score/lang_rust/src/lib.rs` | modify (add the `mod` and `pub use` lines) |
| `crates/bc_document/duet-score/lang_rust/src/ids.rs` | create |
| `crates/bc_document/duet-score/lang_rust/src/model.rs` | create |
| `crates/bc_document/duet-score/lang_rust/src/command.rs` | create |
| `crates/bc_document/duet-score/lang_rust/src/event.rs` | create |
| `crates/bc_document/duet-score/lang_rust/src/apply.rs` | create |
| `crates/bc_document/duet-score/lang_rust/src/canonical.rs` | create |
| `crates/bc_document/duet-score/lang_rust/src/error.rs` | create |
| `crates/bc_document/duet-score/lang_rust/tests/canonical_determinism.rs` | create |
| `Cargo.lock` | modify (SM5 rule 2) |

## Types and signatures

Every signature below is copied from the architecture section the heading names.

### `duet_score::ids` (section 3.2, section 15.3)

```rust
/// A stable identifier for a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NoteId(u64);
```

`PartId`, `StaffId`, `VoiceId`, `MeasureId`, `SpannerId`, and `MarkId` each carry the same shape and
the same derive set (section 15.3). Section 3.2 states that a generational arena index is wrong
here, because the identifier must survive a save, a git commit, and a hand edit.

```rust
/// A part name the user reads and edits. It is never empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartName(Box<str>);

/// One syllable of a lyric. It is never empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LyricText(Box<str>);

/// The printed text of a rehearsal mark. It is never empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RehearsalText(Box<str>);

/// A verse number, one based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VerseNumber(NonZeroU8);

/// The revision counter of one score. It increases on every applied command.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision(u64);

/// A reference to one element of the score, whatever its kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementRef { Note(NoteId), Spanner(SpannerId), Mark(MarkId) }
```

### `duet_score::model` (section 3.3, section 15.3)

```rust
/// The four vocal parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceType { Soprano, Alto, Tenor, Bass }

/// One named part. A score holds many parts of each voice type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    id: PartId,
    voice_type: VoiceType,
    /// 1 for "Soprano 1", 2 for "Soprano 2".
    ordinal: NonZeroU16,
    name: PartName,
    staves: Vec<StaffId>,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// One staff with a clef and a written transposition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Staff {
    id: StaffId,
    clef: Clef,
    transpose_semitones: i8,
    voices: Vec<VoiceId>,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// One voice inside one staff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voice {
    id: VoiceId,
    ordinal: NonZeroU8,
    stem_up: bool,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// A bar on the shared timeline. Every staff shares the same measures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Measure {
    id: MeasureId,
    start: Ticks,
    meter: Meter,
    key: KeySignature,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// A diatonic step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Step { C, D, E, F, G, A, B }

/// How a respell chooses an accidental.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accidental { Natural, Sharp, Flat, DoubleSharp, DoubleFlat }

/// How far an octave spanner shifts, in octaves and in direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OctaveShift { Up8, Down8, Up15, Down15 }

/// A key signature. `fifths` is negative for flats and positive for sharps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySignature { fifths: i8, major: bool }

/// A written pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Pitch {
    octave: i8,
    step: Step,
    /// -2 is a double flat, 2 is a double sharp.
    alter: i8,
}

/// A written duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Duration { value: NoteValue, dots: u8, tuplet: Option<Tuplet> }

impl Duration {
    /// The exact tick count of this duration.
    pub fn ticks(self) -> Ticks;
}

/// Whether a note starts or ends a tie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TieState { starts: bool, stops: bool }

/// One note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    id: NoteId,
    staff: StaffId,
    voice: VoiceId,
    /// The onset in ticks from score zero, not from a measure start.
    onset: Ticks,
    duration: Duration,
    pitch: Pitch,
    tie: TieState,
    articulations: SmallVec<[Articulation; 2]>,
    lyrics: SmallVec<[Lyric; 2]>,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// One rest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rest {
    id: NoteId,
    staff: StaffId,
    voice: VoiceId,
    onset: Ticks,
    duration: Duration,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// A mark that spans more than one note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spanner {
    id: SpannerId,
    kind: SpannerKind,
    from: NoteId,
    to: NoteId,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// The spanner kinds that v1 draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpannerKind { Slur, Crescendo, Diminuendo, Octave(OctaveShift) }

/// A mark on the score that is not a note and not a spanner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreMark {
    id: MarkId,
    at: Ticks,
    kind: MarkKind,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

/// What a score mark means.
#[expect(
    variant_size_differences,
    reason = "`Repeat` carries one byte and `Rehearsal` carries one pointer, so the ratio test \
              refuses every shape that stores text, a boxed arm included"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkKind {
    /// Force a new system at this point.
    SystemBreak,
    /// A repeat barline. `Start` and `End` pair by position order.
    Repeat(RepeatSide),
    /// A rehearsal mark with its printed text.
    Rehearsal(RehearsalText),
}

/// Which side of a repeat a barline carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatSide { Start, End }

/// A clef, as the staff line it centres on and its octave transposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clef { Treble, Bass, Alto, Tenor, TrebleOctaveDown, Percussion }

/// A printed dynamic mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dynamic { Pppp, Ppp, Pp, P, Mp, Mf, F, Ff, Fff, Ffff, Sf, Sfz, Fp }

/// A printed articulation mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Articulation { Staccato, Staccatissimo, Tenuto, Accent, Marcato, Fermata, Breath }

/// One lyric syllable on one note, in one verse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lyric { verse: VerseNumber, text: LyricText, hyphenated: bool }

/// The score aggregate root. Section 3.1 states the boundary and section 3.4
/// states the one way in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Score {
    parts: BTreeMap<PartId, Part>,
    staves: BTreeMap<StaffId, Staff>,
    voices: BTreeMap<VoiceId, Voice>,
    measures: BTreeMap<MeasureId, Measure>,
    notes: BTreeMap<NoteId, Note>,
    rests: BTreeMap<NoteId, Rest>,
    spanners: BTreeMap<SpannerId, Spanner>,
    marks: BTreeMap<MarkId, ScoreMark>,
    tempo_map: TempoMap,
    revision: Revision,
    schema: SchemaVersion,
    /// Fields from a newer schema that this build does not model.
    extra: BTreeMap<String, serde_json::Value>,
}

/// How much memory one transaction's inverse holds.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InverseCost { commands: u32, bytes: u32 }
```

The `MarkKind` `#[expect]` is one of the three `variant_size_differences` sites of Appendix B.1. The
reason text above is the exact text that appendix records.

The `duet-score` constant of section 1.6 lives in `model.rs`.

```rust
/// The current schema number of the canonical document (B42).
pub const SCHEMA: SchemaVersion = SchemaVersion::new(1);
```

### `duet_score::command` (section 3.4, section 15.3)

```rust
/// What a command acts on. One selection type serves every edit verb.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selection {
    Notes(Vec<NoteId>),
    Spanners(Vec<SpannerId>),
    Marks(Vec<MarkId>),
    /// Every element of one staff inside a tick range.
    Range { staff: StaffId, from: Ticks, to: Ticks },
}

/// One intent against the score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreCommand {
    AddPart { voice_type: VoiceType, name: PartName },
    RemovePart { part: PartId },
    AddStaff { part: PartId, clef: Clef },
    RemoveStaff { staff: StaffId },
    SetClef { staff: StaffId, at: Ticks, clef: Clef },
    InsertMeasures { after: MeasureId, count: NonZeroU16 },
    RemoveMeasures { from: MeasureId, count: NonZeroU16 },
    SetKeySignature { measure: MeasureId, key: KeySignature },
    SetTimeSignature { measure: MeasureId, meter: Meter },
    AddMark { at: Ticks, kind: MarkKind },
    RemoveMark { mark: MarkId },
    InsertNote { staff: StaffId, voice: VoiceId, onset: Ticks, pitch: Pitch, duration: Duration },
    InsertRest { staff: StaffId, voice: VoiceId, onset: Ticks, duration: Duration },
    SetPitch { notes: Vec<NoteId>, pitch: PitchEdit },
    SetDuration { notes: Vec<NoteId>, duration: Duration },
    SetTie { note: NoteId, tie: TieState },
    SetArticulations { notes: Vec<NoteId>, articulations: SmallVec<[Articulation; 2]> },
    SetLyric { note: NoteId, verse: VerseNumber, text: LyricText },
    SetDynamic { staff: StaffId, onset: Ticks, dynamic: Dynamic },
    AddSpanner { kind: SpannerKind, from: NoteId, to: NoteId },
    RemoveSpanner { spanner: SpannerId },
    SetVoice { notes: Vec<NoteId>, voice: VoiceId },
    SetPart { notes: Vec<NoteId>, part: PartId, staff: StaffId },
    Move { selection: Selection, by: Ticks, to_staff: Option<StaffId> },
    Duplicate { selection: Selection, at: Ticks },
    Remove { selection: Selection },
    Paste { clipboard: Box<Clipboard>, at: Ticks, staff: StaffId, voice: VoiceId },
}

/// How a pitch edit is expressed. The context menu offers all four.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PitchEdit {
    Absolute(Pitch),
    BySemitones(i8),
    ByOctaves(i8),
    Respell(Accidental),
}

/// A detached copy of a selection. `Cut` and `Copy` produce one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clipboard { origin: Ticks, notes: Vec<Note>, spanners: Vec<Spanner> }

/// What a read-only score query asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreSelector {
    Part(PartId),
    Staff(StaffId),
    Measures { from: MeasureId, count: NonZeroU16 },
    Range { staff: StaffId, from: Ticks, to: Ticks },
}
```

Section 3.4 states that `Cut` is not a command: it is `Score::copy` and then
`ScoreCommand::Remove`, in one undo transaction.

### `duet_score::event` (section 3.4)

```rust
/// One fact about a score change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreEvent {
    PartAdded(PartId),
    PartRemoved(PartId),
    StaffAdded(StaffId),
    StaffRemoved(StaffId),
    NoteInserted(NoteId),
    NoteChanged(NoteId),
    ElementsRemoved(Selection),
    ElementsMoved(Selection),
    SpannerAdded(SpannerId),
    SpannerRemoved(SpannerId),
    MarkAdded(MarkId),
    MarkRemoved(MarkId),
    MeasuresChanged { from: MeasureId, count: u16 },
    SignatureChanged(MeasureId),
    ClefChanged(StaffId),
}
```

### `duet_score::apply` (section 3.4)

```rust
/// What one command produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Applied {
    /// The facts a view needs.
    pub events: Vec<ScoreEvent>,
    /// The commands that undo this one, in application order.
    pub inverse: Vec<ScoreCommand>,
    /// How many commands and how many bytes the inverse holds.
    pub inverse_cost: InverseCost,
}

impl Score {
    /// Apply one command. The score is unchanged when the result is an error.
    ///
    /// # Errors
    /// Returns `ScoreError` when the command names a missing element or
    /// breaks a notation invariant.
    pub fn apply(&mut self, command: ScoreCommand) -> Result<Applied, ScoreError>;

    /// Copy a selection without a change. `Cut` is `copy` then `Remove`.
    ///
    /// # Errors
    /// Returns `ScoreError::MissingElement` when the selection names one.
    pub fn copy(&self, selection: &Selection) -> Result<Clipboard, ScoreError>;

    /// The revision counter. It increases on every successful command.
    pub fn revision(&self) -> Revision;
}
```

`Applied` is one of the three stated `VR7` exemptions, so its three fields are public (section 3.5).
`apply` is transactional: it validates first and mutates second, so a rejected command leaves no
partial change.

### `duet_score::canonical` (section 3.6, section 15.3)

```rust
/// The byte blocks of one canonical score document, in `paths` order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalDocument { meta: Vec<u8>, notes: Vec<u8>, spanners: Vec<u8> }

/// Read a canonical document and migrate it to the current schema.
///
/// # Errors
/// Returns `ScoreError::Schema` for an unknown version, and
/// `ScoreError::Parse` for malformed text.
pub fn read(meta: &str, notes: &str, spanners: &str) -> Result<Score, ScoreError>;

/// Write a canonical document with a deterministic byte order.
///
/// # Errors
/// Returns `ScoreError::Serialize` when a field cannot be written.
pub fn write(score: &Score) -> Result<CanonicalDocument, ScoreError>;
```

Section 3.6 states the three files and the two sort keys: `score/notes.jsonl` sorts by
`(staff, voice, onset, pitch, id)`, and `score/spanners.jsonl` sorts by `(from, kind, id)`. Every
field of both keys derives `Ord` under VR1, and the trailing identifier breaks a tie.

### `duet_score::error` (section 15.3)

```rust
/// Every way the score aggregate refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum ScoreError {
    MissingElement(ElementRef),
    MissingPart(PartId),
    MissingStaff(StaffId),
    MissingMeasure(MeasureId),
    OverlappingNote { staff: StaffId, voice: VoiceId, onset: Ticks },
    TieAcrossPitch { from: NoteId, to: NoteId },
    MeasureNotEmpty(MeasureId),
    Schema { found: SchemaVersion, expected: SchemaVersion },
    Parse(Box<str>),
    Serialize(Box<str>),
    Time(TimeError),
}
```

## Steps

1. Read `crates/bc_document/duet-score/lang_rust/Cargo.toml` and `crates/bc_document/duet-score/lang_rust/src/lib.rs`. Confirm that M1 created
   both and that the manifest holds no `[dependencies]` section. Confirm that `crates/bc_time/duet-time/lang_rust`
   holds every type this chunk names. Report a discrepancy and stop if any one is false.
2. Add the dependency entries to `crates/bc_document/duet-score/lang_rust/Cargo.toml`. The section 1.2 row for
   `duet-score` names four third-party crates, and section 1.3 gives the one internal edge.

   ```toml
   [dependencies]
   duet-time = { workspace = true }
   serde = { workspace = true }
   serde_json = { workspace = true }
   smallvec = { workspace = true }
   thiserror = { workspace = true }
   ```

3. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains the five edges.
4. Create `src/error.rs` with `ScoreError`, then `src/ids.rs` with the seven identifier newtypes,
   the three string newtypes, `VerseNumber`, `Revision`, and `ElementRef`. Add both `mod` lines and
   the `pub use` lines to `src/lib.rs`. Run `cargo check -p duet-score`. Expected result: it
   succeeds.
5. Create `src/model.rs` with every entity, every value object, `Score`, `InverseCost`, and the
   `SCHEMA` constant. Give every record of the eight that section 3.6 property 4 names the
   `#[serde(flatten)] extra: BTreeMap<String, serde_json::Value>` field: `Part`, `Staff`, `Voice`,
   `Measure`, `ScoreMark`, `Note`, `Rest`, and `Spanner`. Add the `mod` line. Run
   `cargo check -p duet-score`. Expected result: it succeeds.
6. Create `src/command.rs` and `src/event.rs` with the four command types and `ScoreEvent`. Add both
   `mod` lines. Run `cargo check -p duet-score`. Expected result: it succeeds.
7. Write the failing invariant tests in a `#[cfg(test)] mod tests` block at the foot of
   `src/apply.rs`. They cover the four refusals `OverlappingNote`, `TieAcrossPitch`,
   `MeasureNotEmpty`, and `MissingElement`, and the transactional property. Run
   `cargo nextest run -p duet-score -E 'test(apply)' --no-tests=fail`. Expected result: the run
   fails, because `apply.rs` holds no implementation.
8. Create `src/apply.rs` with `Applied`, `Score::apply`, `Score::copy`, and `Score::revision`. Every
   arm validates first and mutates second, and every arm builds the inverse command list and the
   `InverseCost`. `clippy::wildcard_enum_match_arm` is denied, so the match over `ScoreCommand`
   names every arm. Run the same command. Expected result: the run passes.
9. Write the failing determinism test in `crates/bc_document/duet-score/lang_rust/tests/canonical_determinism.rs`. It
   builds one score, calls `write` twice, and asserts that the three byte blocks are equal. It then
   inserts the same notes in a different order, calls `write`, and asserts that the bytes still
   match. It then calls `read` on the output and asserts that the score is equal. Run
   `cargo nextest run -p duet-score --test canonical_determinism --no-tests=fail`. Expected result:
   the run fails.
10. Create `src/canonical.rs` with `CanonicalDocument`, `read`, and `write`. The writer sorts every
    map key, writes fields in struct order, prints every float through `Finite`, and writes one
    record per line for `notes.jsonl` and `spanners.jsonl`. The reader keeps every unknown key in an
    `extra` bag and reports each one. Run the same command. Expected result: the run passes.
11. Write the failing schema tests in `src/canonical.rs`. One asserts that a document whose
    `SchemaVersion` is above `SCHEMA` gives `ScoreError::Schema`. One asserts that a NaN token in a
    float field gives `ScoreError::Parse`. Run
    `cargo nextest run -p duet-score -E 'test(schema)' --no-tests=fail`. Expected result: the run
    fails, then passes after the reader gains both checks.
12. Run `cargo build --workspace` again, then `git add` the write scope and `git commit`. The native
    hook runs `scripts/dod.sh`.

## Tests

Every assert carries a message. The unit tests live in a `#[cfg(test)] mod tests` block in the same
file. `tests/canonical_determinism.rs` is an integration target and wraps its tests in a
`#[cfg(test)] mod tests` block. Author guidance: the `test-author` skill.

| Test | What it asserts | Where it lives |
|---|---|---|
| `apply_rejects_overlapping_note` | An insert over an existing note in the same staff and voice gives `ScoreError::OverlappingNote` | `src/apply.rs` |
| `apply_rejects_tie_across_pitch` | A tie between two different pitches gives `ScoreError::TieAcrossPitch` | `src/apply.rs` |
| `apply_rejects_remove_of_filled_measure` | A measure remove over a measure that holds a note gives `ScoreError::MeasureNotEmpty` | `src/apply.rs` |
| `apply_leaves_no_partial_change` | The score compares equal to its own clone after every refusal | `src/apply.rs` |
| `apply_inverse_restores_the_score` | Applying `Applied::inverse` in order restores the score to its earlier value | `src/apply.rs` |
| `apply_increases_the_revision` | `Score::revision` is greater after a successful command and equal after a refusal | `src/apply.rs` |
| `copy_then_remove_is_a_cut` | `Score::copy` returns the selected notes and leaves the score unchanged | `src/apply.rs` |
| `canonical_schema_refuses_newer` | A document above `SCHEMA` gives `ScoreError::Schema` | `src/canonical.rs` |
| `canonical_reader_keeps_unknown_keys` | An unknown key lands in the `extra` bag and is reported once | `src/canonical.rs` |
| `canonical_determinism` | Two writes of one score give equal bytes, an insert order change gives equal bytes, and a read of the output gives an equal score | `tests/canonical_determinism.rs` |

Section 14 selects `canonical_determinism` for rung-two command 4, and the selected-test table names
chunk T2 as its writer.

## Verification

```
cargo nextest run -p duet-score --no-tests=fail
cargo nextest run -p duet-score --test canonical_determinism --no-tests=fail
cargo clippy -p duet-score --all-targets -- -D warnings
```

Expected output: the first command reports every unit test and both integration tests as passed. The
second command reports the determinism test as passed and reports no filter miss. The third command
prints no warning.

Then commit on a branch named `chunk/t2-score`. The native git hook runs `scripts/dod.sh`, and the
commit lands only when every gate passes.

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
