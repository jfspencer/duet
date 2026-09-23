//! The canonical document of the score: the reader and the writer.
//!
//! Section 3.6 of `roadmap/duet-v1/architecture.md` states the four
//! properties, the three files, and the two sort keys. `score/meta.json` is
//! one JSON object, `score/notes.jsonl` holds one note or one rest per line in
//! `(staff, voice, onset, pitch, id)` order, and `score/spanners.jsonl` holds
//! one spanner per line in `(from, kind, id)` order.
//!
//! Five rules of this module answer what section 3.6 leaves open.
//!
//! 1. **A note stands before a rest** at one `(staff, voice, onset)`, and two
//!    rests then stand in identifier order. The reader of a diff sees the
//!    sounding event first, and the rule is total, so each line holds one
//!    place and the output stays unique.
//! 2. **`score/meta.json` carries `revision` and `next_id`** beside `schema`,
//!    and `read` restores both. Section 3.2 makes the identifier counter a
//!    counter that the document persists, so no mint after a reload aliases a
//!    live identifier. The stored counter is a claim and not a fact: `read`
//!    raises it above every identifier the document holds, whatever the
//!    document states. Decision 4b of ADR 0002 makes a hand edit a product
//!    path, and a counter at or below a live identifier would make the next
//!    mint replace a live entity.
//! 3. **Every block ends with a newline**: the meta object and each record of
//!    both JSON Lines files. One rule serves all three files.
//! 4. **An unknown key reaches the `extra` bag of the record that carried it,
//!    and no other bag.** The `ImportWarning` channel of edge rule 2 is a
//!    `duet-command` type, and that crate sits above this one, so the `read`
//!    signature of section 3.6 carries no warning channel. The bag is the
//!    whole report at this boundary (`escalation:T2-2`).
//! 5. **One identifier names one record of one keyspace.** `Score` mints each
//!    identifier once, so only a hand edit can give one number to two records.
//!    Such a document would put two entities behind one reference, and a map
//!    insert at a live key replaces the entity that stood there, so the second
//!    record would delete the first. `read` refuses it as malformed text, over
//!    every keyspace of the document: the five lists of `score/meta.json`,
//!    `score/notes.jsonl`, where the notes and the rests answer to one
//!    identifier type, and `score/spanners.jsonl`.
//!
//! The schema gate is a comparison, not an equality: a number above `SCHEMA`
//! is `ScoreError::Schema`, and a number below it is the score of a build that
//! no release carried, so the read runs no migration step and the score comes
//! back under `SCHEMA`.

use std::collections::BTreeMap;

use duet_time::{SchemaVersion, TempoMap, Ticks};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::error::ScoreError;
use crate::ids::{MarkId, MeasureId, NoteId, PartId, Revision, SpannerId, StaffId, VoiceId};
use crate::model::{
    Measure, Note, Part, Pitch, Rest, SCHEMA, Score, ScoreMark, Spanner, Staff, Voice,
    note_order_key, rest_order_key, spanner_order_key,
};

/// The part of the document that the parts of `score/meta.json` come from.
const PARTS_LIST: &str = "the parts list of score/meta.json";

/// The part of the document that the staves of `score/meta.json` come from.
const STAVES_LIST: &str = "the staves list of score/meta.json";

/// The part of the document that the voices of `score/meta.json` come from.
const VOICES_LIST: &str = "the voices list of score/meta.json";

/// The part of the document that the measures of `score/meta.json` come from.
const MEASURES_LIST: &str = "the measures list of score/meta.json";

/// The part of the document that the marks of `score/meta.json` come from.
const MARKS_LIST: &str = "the marks list of score/meta.json";

/// The file that holds one note or one rest per line.
const NOTES_FILE: &str = "score/notes.jsonl";

/// The file that holds one spanner per line.
const SPANNERS_FILE: &str = "score/spanners.jsonl";

/// The byte blocks of one canonical score document, in `paths` order.
///
/// It derives serde, because `Snapshot` carries one across the transport and
/// the compile-time assertion of section 3.5 covers `Snapshot`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalDocument {
    /// The bytes of `score/meta.json`.
    meta: Vec<u8>,
    /// The bytes of `score/notes.jsonl`.
    notes: Vec<u8>,
    /// The bytes of `score/spanners.jsonl`.
    spanners: Vec<u8>,
}

impl CanonicalDocument {
    /// The bytes of `score/meta.json`.
    #[must_use]
    pub fn meta(&self) -> &[u8] {
        &self.meta
    }

    /// The bytes of `score/notes.jsonl`.
    #[must_use]
    pub fn notes(&self) -> &[u8] {
        &self.notes
    }

    /// The bytes of `score/spanners.jsonl`.
    #[must_use]
    pub fn spanners(&self) -> &[u8] {
        &self.spanners
    }
}

/// Read a canonical document.
///
/// The read runs no migration step. A document above `SCHEMA` is refused, and a
/// document below it is the score of a build that no release carried, so the
/// score comes back under `SCHEMA` with its content as it stands. A document
/// that lacks a field this build reads is malformed text, not an older schema.
///
/// The reader checks the SHAPE of the document and not its references: a hand
/// edit that names a staff, a voice, or a note the document does not carry
/// reads back as an aggregate that no command of `apply` would build.
/// `escalation:T2-2` records the gap.
///
/// # Errors
/// Returns `ScoreError::Schema` for a document above the schema that this
/// build reads, and `ScoreError::Parse` for malformed text, which includes a
/// document that gives one identifier to two records of one keyspace (rule 5).
pub fn read(meta: &str, notes: &str, spanners: &str) -> Result<Score, ScoreError> {
    let found = schema_of(meta)?;
    if found > SCHEMA {
        return Err(ScoreError::Schema {
            found,
            expected: SCHEMA,
        });
    }
    let stored: Meta = from_json(meta)?;
    let mut score = stored.into_score()?;
    for line in text_records(notes) {
        let record: TimedRecord = from_json(line)?;
        let id = record.id();
        if score.notes().contains_key(&id) || score.rests().contains_key(&id) {
            return Err(repeated_id(NOTES_FILE, id.get()));
        }
        match record {
            TimedRecord::Note(note) => {
                score.notes_mut().insert(id, note);
            },
            TimedRecord::Rest(rest) => {
                score.rests_mut().insert(id, rest);
            },
        }
    }
    for line in text_records(spanners) {
        let spanner: Spanner = from_json(line)?;
        let id = spanner.id();
        if score.spanners().contains_key(&id) {
            return Err(repeated_id(SPANNERS_FILE, id.get()));
        }
        score.spanners_mut().insert(id, spanner);
    }
    if let Some(number) = greatest_identifier(&score) {
        score.reserve_id(number);
    }
    Ok(score)
}

/// Write a canonical document with a deterministic byte order.
///
/// # Errors
/// Returns `ScoreError::Serialize` when a field cannot be written.
pub fn write(score: &Score) -> Result<CanonicalDocument, ScoreError> {
    Ok(CanonicalDocument {
        meta: meta_block(score)?,
        notes: notes_block(score)?,
        spanners: spanners_block(score)?,
    })
}

/// The stored form of `score/meta.json`.
///
/// The declaration order is the byte order of the block, which property 2 of
/// section 3.6 fixes as the struct order: the schema, the five entity lists
/// and the tempo map of the section 3.6 table, and then the two counters.
#[derive(Debug, Serialize, Deserialize)]
struct Meta {
    /// The schema number that the document states.
    schema: SchemaVersion,
    /// The parts of the score, in identifier order.
    parts: Vec<Part>,
    /// The staves of the score, in identifier order.
    staves: Vec<Staff>,
    /// The voices of the score, in identifier order.
    voices: Vec<Voice>,
    /// The measures of the shared timeline, in identifier order.
    measures: Vec<Measure>,
    /// The marks of the score, in identifier order.
    marks: Vec<ScoreMark>,
    /// The tempo and meter map of the score.
    tempo_map: TempoMap,
    /// The revision counter of the score.
    revision: Revision,
    /// The counter that every identifier of the score comes from.
    next_id: u64,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Meta {
    /// The meta block of `score`.
    fn of(score: &Score) -> Self {
        Self {
            schema: score.schema(),
            parts: score.parts().values().cloned().collect(),
            staves: score.staves().values().cloned().collect(),
            voices: score.voices().values().cloned().collect(),
            measures: score.measures().values().cloned().collect(),
            marks: score.marks().values().cloned().collect(),
            tempo_map: score.tempo_map().clone(),
            revision: score.revision(),
            next_id: score.next_id(),
            extra: score.extra().clone(),
        }
    }

    /// The score that this block states, with no note, rest, or spanner.
    ///
    /// # Errors
    /// Returns `ScoreError::Parse` when two records of one list carry one
    /// identifier (rule 5).
    fn into_score(self) -> Result<Score, ScoreError> {
        let mut score = Score::new();
        *score.parts_mut() = keyed(self.parts, Part::id, PartId::get, PARTS_LIST)?;
        *score.staves_mut() = keyed(self.staves, Staff::id, StaffId::get, STAVES_LIST)?;
        *score.voices_mut() = keyed(self.voices, Voice::id, VoiceId::get, VOICES_LIST)?;
        *score.measures_mut() = keyed(self.measures, Measure::id, MeasureId::get, MEASURES_LIST)?;
        *score.marks_mut() = keyed(self.marks, ScoreMark::id, MarkId::get, MARKS_LIST)?;
        score.set_tempo_map(self.tempo_map);
        score.set_revision(self.revision);
        score.set_next_id(self.next_id);
        *score.extra_mut() = self.extra;
        Ok(score)
    }
}

/// The schema number of one `score/meta.json`, read before the block is.
///
/// A document above the schema of this build holds fields that `Meta` cannot
/// take, so the refusal of property 4 reads the number on its own.
#[derive(Debug, Clone, Copy, Deserialize)]
struct SchemaProbe {
    /// The schema number that the document states.
    schema: SchemaVersion,
}

/// One line of `score/notes.jsonl`.
///
/// The `record` tag names the line for a reader and for a hand edit, and it
/// makes the two shapes total: a line is a note or a rest, and neither shape
/// stands in for the other.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "record", rename_all = "lowercase")]
enum TimedRecord {
    /// One sounding note.
    Note(Note),
    /// One silence.
    Rest(Rest),
}

impl TimedRecord {
    /// The identifier that this record carries.
    const fn id(&self) -> NoteId {
        match *self {
            Self::Note(ref note) => note.id(),
            Self::Rest(ref rest) => rest.id(),
        }
    }

    /// The place of this record in `score/notes.jsonl`.
    const fn place(&self) -> NoteFilePlace {
        match *self {
            Self::Note(ref note) => NoteFilePlace::of_note(note),
            Self::Rest(ref rest) => NoteFilePlace::of_rest(rest),
        }
    }
}

/// Whether one line of `score/notes.jsonl` sounds.
///
/// The declaration order is the file order at one place on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum NoteFileRank {
    /// The line holds a note.
    Note,
    /// The line holds a rest.
    Rest,
}

/// The place of one line in `score/notes.jsonl`.
///
/// The declaration order is the comparison order, and the identifier at the
/// foot breaks every tie, so the key is total over the two shapes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct NoteFilePlace {
    /// The staff that holds the line.
    staff: StaffId,
    /// The voice that holds the line.
    voice: VoiceId,
    /// The onset in ticks from score zero.
    onset: Ticks,
    /// Whether the line sounds.
    rank: NoteFileRank,
    /// The written pitch, which a rest carries no value for.
    pitch: Option<Pitch>,
    /// The identifier of the note or the rest.
    id: NoteId,
}

impl NoteFilePlace {
    /// The place of `note`, from the key of section 3.6.
    const fn of_note(note: &Note) -> Self {
        let (staff, voice, onset, pitch, id) = note_order_key(note);
        Self {
            staff,
            voice,
            onset,
            rank: NoteFileRank::Note,
            pitch: Some(pitch),
            id,
        }
    }

    /// The place of `rest`, from the key of section 3.6 without the pitch.
    const fn of_rest(rest: &Rest) -> Self {
        let (staff, voice, onset, id) = rest_order_key(rest);
        Self {
            staff,
            voice,
            onset,
            rank: NoteFileRank::Rest,
            pitch: None,
            id,
        }
    }
}

/// The records of `list`, by the identifier that each one carries.
///
/// `place` names the part of the document that the list came from, for the
/// refusal of a repeated identifier.
///
/// # Errors
/// Returns `ScoreError::Parse` when two records carry one identifier (rule 5).
fn keyed<Id, Record>(
    list: Vec<Record>,
    id: impl Fn(&Record) -> Id,
    number: impl Fn(Id) -> u64,
    place: &str,
) -> Result<BTreeMap<Id, Record>, ScoreError>
where
    Id: Ord + Copy,
{
    let mut held = BTreeMap::new();
    for record in list {
        let key = id(&record);
        if held.insert(key, record).is_some() {
            return Err(repeated_id(place, number(key)));
        }
    }
    Ok(held)
}

/// The greatest number that any identifier of `score` carries.
///
/// Every map of the aggregate sorts by its own identifier, so the last key of
/// each one is the greatest of that keyspace and the greatest of the eight is
/// the greatest of the score. The answer is `None` for a score that holds
/// nothing at all.
fn greatest_identifier(score: &Score) -> Option<u64> {
    [
        score.parts().keys().next_back().copied().map(PartId::get),
        score.staves().keys().next_back().copied().map(StaffId::get),
        score.voices().keys().next_back().copied().map(VoiceId::get),
        score
            .measures()
            .keys()
            .next_back()
            .copied()
            .map(MeasureId::get),
        score.notes().keys().next_back().copied().map(NoteId::get),
        score.rests().keys().next_back().copied().map(NoteId::get),
        score
            .spanners()
            .keys()
            .next_back()
            .copied()
            .map(SpannerId::get),
        score.marks().keys().next_back().copied().map(MarkId::get),
    ]
    .into_iter()
    .flatten()
    .max()
}

/// The refusal of a document that names one identifier twice in `place`.
///
/// `ScoreError::Parse` is the arm that the roster holds for a document the
/// reader cannot take, and a repeated identifier is such a document: the second
/// record would hide the first behind one reference (rule 5 of this module).
fn repeated_id(place: &str, number: u64) -> ScoreError {
    ScoreError::Parse(format!("two records of {place} carry identifier {number}").into_boxed_str())
}

/// The schema number that `meta` states.
fn schema_of(meta: &str) -> Result<SchemaVersion, ScoreError> {
    let probe: SchemaProbe = from_json(meta)?;
    Ok(probe.schema)
}

/// Every line of one JSON Lines block that holds a record.
fn text_records(block: &str) -> impl Iterator<Item = &str> {
    block.lines().filter(|line| !line.trim().is_empty())
}

/// The bytes of `score/meta.json`.
fn meta_block(score: &Score) -> Result<Vec<u8>, ScoreError> {
    let mut text = to_json(&Meta::of(score))?;
    text.push('\n');
    Ok(text.into_bytes())
}

/// The bytes of `score/notes.jsonl`.
fn notes_block(score: &Score) -> Result<Vec<u8>, ScoreError> {
    let notes = score.notes().values().cloned().map(TimedRecord::Note);
    let rests = score.rests().values().cloned().map(TimedRecord::Rest);
    let mut records: Vec<TimedRecord> = notes.chain(rests).collect();
    records.sort_by_key(TimedRecord::place);
    json_lines(&records)
}

/// The bytes of `score/spanners.jsonl`.
fn spanners_block(score: &Score) -> Result<Vec<u8>, ScoreError> {
    let mut records: Vec<Spanner> = score.spanners().values().cloned().collect();
    records.sort_by_key(spanner_order_key);
    json_lines(&records)
}

/// The JSON Lines block of `records`: one record per line, each line ended.
fn json_lines<Record: Serialize>(records: &[Record]) -> Result<Vec<u8>, ScoreError> {
    let mut text = String::new();
    for record in records {
        let line = to_json(record)?;
        text.push_str(&line);
        text.push('\n');
    }
    Ok(text.into_bytes())
}

/// The record that `text` states, or the refusal of a malformed text.
fn from_json<Record: DeserializeOwned>(text: &str) -> Result<Record, ScoreError> {
    serde_json::from_str(text)
        .map_err(|error| ScoreError::Parse(error.to_string().into_boxed_str()))
}

/// The JSON text of `record`, or the refusal of a value with no written form.
fn to_json<Record: Serialize>(record: &Record) -> Result<String, ScoreError> {
    serde_json::to_string(record)
        .map_err(|error| ScoreError::Serialize(error.to_string().into_boxed_str()))
}

#[cfg(test)]
mod tests {
    use duet_time::{NoteValue, SchemaVersion, Ticks};
    use serde_json::{Map, Value};

    use super::{read, write};
    use crate::apply::Applied;
    use crate::command::ScoreCommand;
    use crate::error::ScoreError;
    use crate::event::ScoreEvent;
    use crate::ids::{NoteId, PartId, PartName, StaffId};
    use crate::model::{
        Clef, Duration, MarkKind, Pitch, Rest, SCHEMA, Score, SpannerKind, Step, VoiceType,
    };

    /// The tick count of one quarter note.
    const QUARTER_TICKS: i64 = 1_920;

    /// The rest that stands first of the two rests at one place.
    const LOWER_REST_ID: u64 = 900;

    /// The rest that stands second of the two rests at one place.
    const HIGHER_REST_ID: u64 = 901;

    /// A key that no schema of this build declares, put into `score/meta.json`.
    const UNKNOWN_META_KEY: &str = "duet_unknown_meta_key";

    /// A key that no schema of this build declares, put into one note record.
    const UNKNOWN_NOTE_KEY: &str = "duet_unknown_note_key";

    /// The value that both unknown keys carry.
    const UNKNOWN_VALUE: &str = "a hand edit put this here";

    /// The field that carries the non-finite float token.
    const NAN_PROBE_KEY: &str = "duet_nan_probe";

    /// The string that the fixture swaps for a bare `NaN` token.
    ///
    /// `serde_json` writes no non-finite float, so the fixture writes a string
    /// that no other part of the document holds and then replaces it in the
    /// text.
    const NAN_SENTINEL: &str = "__duet_nan_sentinel__";

    /// The part that `PartAdded` names in `applied`.
    fn part_of(applied: &Applied) -> PartId {
        applied
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::PartAdded(part) = *event {
                    Some(part)
                } else {
                    None
                }
            })
            .expect("an accepted AddPart reports PartAdded")
    }

    /// The staff that `StaffAdded` names in `applied`.
    fn staff_of(applied: &Applied) -> StaffId {
        applied
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::StaffAdded(staff) = *event {
                    Some(staff)
                } else {
                    None
                }
            })
            .expect("an accepted AddStaff reports StaffAdded")
    }

    /// A score with one part, one staff, one voice, and two quarter notes.
    fn sut() -> Score {
        let mut score = Score::new();
        let part_result = score
            .apply(ScoreCommand::AddPart {
                voice_type: VoiceType::Soprano,
                name: PartName::new("Soprano 1").expect("a name with text"),
            })
            .expect("AddPart is accepted");
        let part = part_of(&part_result);
        let staff_result = score
            .apply(ScoreCommand::AddStaff {
                part,
                clef: Clef::Treble,
            })
            .expect("AddStaff is accepted");
        let staff = staff_of(&staff_result);
        let voice = *score
            .staves()
            .get(&staff)
            .expect("the score holds the staff it just added")
            .voices()
            .first()
            .expect("AddStaff gives the staff one voice");
        for (onset, step) in [(QUARTER_TICKS, Step::D), (0, Step::C)] {
            score
                .apply(ScoreCommand::InsertNote {
                    staff,
                    voice,
                    onset: Ticks::new(onset),
                    pitch: Pitch::new(4, step, 0),
                    duration: Duration::new(NoteValue::Quarter, 0, None),
                })
                .expect("an InsertNote at a free onset is accepted");
        }
        score
    }

    /// A score that holds one record of every kind the document carries.
    ///
    /// The eight keyspaces of the aggregate reach the document through the five
    /// lists of `score/meta.json`, the two shapes of `score/notes.jsonl`, and
    /// `score/spanners.jsonl`, so one fixture exercises the repeated-identifier
    /// rule over every one of them.
    fn sut_with_every_kind() -> Score {
        let mut score = sut();
        let mut notes = score.notes().keys().copied();
        let from = notes.next().expect("the fixture score holds two notes");
        let to = notes.next().expect("the fixture score holds two notes");
        score
            .apply(ScoreCommand::AddSpanner {
                kind: SpannerKind::Slur,
                from,
                to,
            })
            .expect("an AddSpanner between two notes the score holds is accepted");
        score
            .apply(ScoreCommand::AddMark {
                at: Ticks::ZERO,
                kind: MarkKind::SystemBreak,
            })
            .expect("an AddMark is accepted");
        score
    }

    /// The greatest number that any identifier of `score` carries.
    ///
    /// The helper reads the eight maps on its own, so the expectation of the
    /// counter test does not come from the code under test.
    fn greatest_identifier(score: &Score) -> u64 {
        let parts = score.parts().keys().map(|id| id.get());
        let staves = score.staves().keys().map(|id| id.get());
        let voices = score.voices().keys().map(|id| id.get());
        let measures = score.measures().keys().map(|id| id.get());
        let notes = score.notes().keys().map(|id| id.get());
        let rests = score.rests().keys().map(|id| id.get());
        let spanners = score.spanners().keys().map(|id| id.get());
        let marks = score.marks().keys().map(|id| id.get());
        parts
            .chain(staves)
            .chain(voices)
            .chain(measures)
            .chain(notes)
            .chain(rests)
            .chain(spanners)
            .chain(marks)
            .max()
            .unwrap_or(0)
    }

    /// The meta block of `score` with the entries of `list` doubled.
    ///
    /// The answer is the text of the document and the number that two entries
    /// of the list then carry.
    fn meta_with_a_repeated_entry(meta: &str, list: &str) -> (String, u64) {
        let mut object = meta_object(meta);
        let entries = object
            .get(list)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_else(|| panic!("meta.json carries the {list} list"));
        let first = entries
            .first()
            .cloned()
            .unwrap_or_else(|| panic!("the fixture score fills the {list} list"));
        let number = first
            .get("id")
            .and_then(Value::as_u64)
            .unwrap_or_else(|| panic!("every record of the {list} list carries its identifier"));
        let mut doubled = entries;
        doubled.push(first);
        object.insert(list.to_owned(), Value::Array(doubled));
        (object_text(object), number)
    }

    /// The block as an owned string. Every canonical block is UTF-8.
    fn text(block: &[u8]) -> String {
        core::str::from_utf8(block)
            .expect("a canonical block is UTF-8")
            .to_owned()
    }

    /// The three blocks of the written document, as text.
    fn blocks(score: &Score) -> (String, String, String) {
        let document = write(score).expect("a score writes");
        (
            text(document.meta()),
            text(document.notes()),
            text(document.spanners()),
        )
    }

    /// The meta block as a JSON object.
    fn meta_object(meta: &str) -> Map<String, Value> {
        serde_json::from_str(meta).expect("meta.json is one JSON object")
    }

    /// The JSON text of one object.
    fn object_text(object: Map<String, Value>) -> String {
        serde_json::to_string(&Value::Object(object)).expect("a JSON object writes")
    }

    /// The lines of one JSON Lines block, with no empty line.
    fn records(block: &str) -> Vec<String> {
        block
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(str::to_owned)
            .collect()
    }

    /// The block that `lines` spells, with the line ending of `original`.
    fn rejoin(lines: &[String], original: &str) -> String {
        let body = lines.join("\n");
        if original.ends_with('\n') {
            let mut ended = body;
            ended.push('\n');
            ended
        } else {
            body
        }
    }

    /// The `id` field of every record of one JSON Lines block, in file order.
    fn identifiers(block: &str) -> Vec<u64> {
        records(block)
            .iter()
            .map(|line| {
                let record: Map<String, Value> =
                    serde_json::from_str(line).expect("a JSON Lines record is one JSON object");
                record
                    .get("id")
                    .and_then(Value::as_u64)
                    .expect("every canonical record carries its identifier")
            })
            .collect()
    }

    #[test]
    fn canonical_schema_refuses_newer() {
        let score = sut();
        let (meta, notes, spanners) = blocks(&score);
        let newer = SchemaVersion::new(SCHEMA.get().saturating_add(1));
        let mut object = meta_object(&meta);
        object.insert("schema".to_owned(), Value::from(newer.get()));
        let outcome = read(&object_text(object), &notes, &spanners);
        assert_eq!(
            outcome,
            Err(ScoreError::Schema {
                found: newer,
                expected: SCHEMA,
            }),
            "a document above the schema this build reads is refused, and the refusal names both numbers"
        );
    }

    #[test]
    fn canonical_schema_does_not_refuse_older() {
        let score = sut();
        let (meta, notes, spanners) = blocks(&score);
        let older = SchemaVersion::new(SCHEMA.get().saturating_sub(1));
        let mut object = meta_object(&meta);
        object.insert("schema".to_owned(), Value::from(older.get()));
        let read_back = read(&object_text(object), &notes, &spanners)
            .expect("a document below the schema this build reads is not refused");
        assert_eq!(
            read_back.schema(),
            SCHEMA,
            "the schema gate compares with a greater-than test, and the read normalizes an older document to SCHEMA rather than refusing it or keeping the stored number"
        );
    }

    #[test]
    fn canonical_parse_refuses_a_nan_token() {
        let score = sut();
        let (meta, notes, spanners) = blocks(&score);
        let mut object = meta_object(&meta);
        object.insert(NAN_PROBE_KEY.to_owned(), Value::from(NAN_SENTINEL));
        let quoted_sentinel = Value::from(NAN_SENTINEL).to_string();
        let broken = object_text(object).replace(&quoted_sentinel, "NaN");
        assert!(
            broken.contains("NaN"),
            "the fixture puts a non-finite float token into the document text"
        );
        let refusal = read(&broken, &notes, &spanners);
        assert!(
            matches!(refusal, Err(ScoreError::Parse(_))),
            "a non-finite float token never comes back from the writer, so the reader refuses it as malformed text"
        );
    }

    #[test]
    fn canonical_reader_keeps_unknown_keys() {
        let score = sut();
        let (meta, notes, spanners) = blocks(&score);
        let mut object = meta_object(&meta);
        object.insert(UNKNOWN_META_KEY.to_owned(), Value::from(UNKNOWN_VALUE));
        let mut lines = records(&notes);
        assert!(
            !lines.is_empty(),
            "the fixture score writes at least one note record"
        );
        let mut record: Map<String, Value> =
            serde_json::from_str(&lines[0]).expect("a notes.jsonl line is one JSON object");
        let carrier = NoteId::new(
            record
                .get("id")
                .and_then(Value::as_u64)
                .expect("a note record carries its identifier"),
        );
        record.insert(UNKNOWN_NOTE_KEY.to_owned(), Value::from(UNKNOWN_VALUE));
        lines[0] = object_text(record);
        let patched_notes = rejoin(&lines, &notes);

        let read_back = read(&object_text(object), &patched_notes, &spanners)
            .expect("a document with an unknown key reads back");

        assert_eq!(
            read_back.extra().get(UNKNOWN_META_KEY),
            Some(&Value::from(UNKNOWN_VALUE)),
            "an unknown key of meta.json lands in the score's own bag"
        );
        assert_eq!(
            read_back.extra().len(),
            1,
            "the unknown key of meta.json lands exactly once, and no known field joins it"
        );
        assert!(
            !read_back.extra().contains_key(UNKNOWN_NOTE_KEY),
            "an unknown key of a note record does not also reach the score's own bag"
        );

        let note = read_back
            .notes()
            .get(&carrier)
            .expect("the score holds the note the fixture patched");
        assert_eq!(
            note.extra().get(UNKNOWN_NOTE_KEY),
            Some(&Value::from(UNKNOWN_VALUE)),
            "an unknown key of a note record lands in that record's own bag"
        );
        assert_eq!(
            note.extra().len(),
            1,
            "the unknown key of a note record lands exactly once"
        );
        for (id, other) in read_back.notes() {
            assert!(
                *id == carrier || other.extra().is_empty(),
                "no other note record takes the unknown key"
            );
        }
    }

    #[test]
    fn canonical_files_hold_one_object_and_one_record_per_line() {
        let score = sut();
        let (meta, notes, spanners) = blocks(&score);
        let document: Value = serde_json::from_str(&meta).expect("meta.json parses");
        assert!(document.is_object(), "meta.json is one JSON object");

        let note_records = records(&notes);
        assert_eq!(
            note_records.len(),
            score.notes().len() + score.rests().len(),
            "notes.jsonl holds one line for each note and each rest"
        );
        for line in &note_records {
            let record: Value =
                serde_json::from_str(line).expect("a notes.jsonl line parses on its own");
            assert!(
                record.is_object(),
                "every notes.jsonl line is one JSON object"
            );
        }

        let spanner_records = records(&spanners);
        assert_eq!(
            spanner_records.len(),
            score.spanners().len(),
            "spanners.jsonl holds one line for each spanner"
        );
        for line in &spanner_records {
            let record: Value =
                serde_json::from_str(line).expect("a spanners.jsonl line parses on its own");
            assert!(
                record.is_object(),
                "every spanners.jsonl line is one JSON object"
            );
        }
    }

    #[test]
    fn canonical_round_trip_keeps_the_counters() {
        let score = sut();
        let (meta, notes, spanners) = blocks(&score);
        let read_back = read(&meta, &notes, &spanners).expect("the written document reads back");
        assert_eq!(
            read_back.next_id(),
            score.next_id(),
            "meta.json carries the identifier counter, so no mint after a reload aliases a live identifier"
        );
        assert_eq!(
            read_back.revision(),
            score.revision(),
            "meta.json carries the revision, so a reload keeps the count of applied commands"
        );
        assert_eq!(
            read_back, score,
            "content equality survives the round trip, whatever the two counters hold"
        );
    }

    #[test]
    fn canonical_puts_a_note_before_a_rest_and_two_rests_in_identifier_order() {
        let mut score = sut();
        let carrier = score
            .notes()
            .values()
            .find(|note| note.onset() == Ticks::new(QUARTER_TICKS))
            .expect("the fixture score holds a note at the second quarter")
            .clone();
        let opening = score
            .notes()
            .values()
            .find(|note| note.onset() == Ticks::ZERO)
            .expect("the fixture score holds a note at score zero")
            .id();
        for planted in [HIGHER_REST_ID, LOWER_REST_ID] {
            let rest = NoteId::new(planted);
            score.rests_mut().insert(
                rest,
                Rest::new(
                    rest,
                    carrier.staff(),
                    carrier.voice(),
                    carrier.onset(),
                    carrier.duration(),
                ),
            );
        }
        let (_, notes, _) = blocks(&score);
        assert_eq!(
            identifiers(&notes),
            vec![
                opening.get(),
                carrier.id().get(),
                LOWER_REST_ID,
                HIGHER_REST_ID
            ],
            "a note stands before a rest at one (staff, voice, onset), and two rests there stand in identifier order"
        );
    }

    #[test]
    fn canonical_read_raises_the_counter_above_every_identifier() {
        let score = sut();
        let (meta, notes, spanners) = blocks(&score);
        let mut object = meta_object(&meta);
        object.insert("next_id".to_owned(), Value::from(0_u64));

        let mut read_back = read(&object_text(object), &notes, &spanners)
            .expect("a document that states a low counter reads back");

        let greatest = greatest_identifier(&read_back);
        assert!(
            read_back.next_id() > greatest,
            "the reader raises the counter above the greatest identifier the document holds, and it stands at {} against {greatest}",
            read_back.next_id()
        );
        let parts_before = read_back.parts().len();
        read_back
            .apply(ScoreCommand::AddPart {
                voice_type: VoiceType::Alto,
                name: PartName::new("Alto 1").expect("a name with text"),
            })
            .expect("an AddPart is accepted");
        assert_eq!(
            read_back.parts().len(),
            parts_before + 1,
            "a mint after the read takes a free number, so it adds a part rather than replacing a live one"
        );
    }

    #[test]
    fn canonical_read_refuses_a_repeated_identifier_in_every_list_of_meta() {
        let score = sut_with_every_kind();
        let (meta, notes, spanners) = blocks(&score);
        for list in ["parts", "staves", "voices", "measures", "marks"] {
            let (broken, number) = meta_with_a_repeated_entry(&meta, list);

            let refusal = read(&broken, &notes, &spanners);

            let Err(ScoreError::Parse(ref message)) = refusal else {
                panic!(
                    "two records of the {list} list under one identifier are refused as malformed text, and the reader answered {refusal:?}"
                );
            };
            assert!(
                message.contains(&number.to_string()),
                "the refusal of the {list} list names the repeated number, and it reads {message}"
            );
        }
    }

    #[test]
    fn canonical_read_refuses_a_repeated_spanner_identifier() {
        let score = sut_with_every_kind();
        let (meta, notes, spanners) = blocks(&score);
        let mut lines = records(&spanners);
        let first = lines
            .first()
            .cloned()
            .expect("the fixture score writes one spanner line");
        let number = serde_json::from_str::<Map<String, Value>>(&first)
            .expect("a spanners.jsonl line is one JSON object")
            .get("id")
            .and_then(Value::as_u64)
            .expect("every spanner record carries its identifier");
        lines.push(first);
        let doubled = rejoin(&lines, &spanners);

        let refusal = read(&meta, &notes, &doubled);

        let Err(ScoreError::Parse(ref message)) = refusal else {
            panic!(
                "two lines of score/spanners.jsonl under one identifier are refused as malformed text, and the reader answered {refusal:?}"
            );
        };
        assert!(
            message.contains(&number.to_string()),
            "the refusal of a repeated spanner identifier names the number, and it reads {message}"
        );
    }

    #[test]
    fn canonical_read_refuses_one_identifier_in_two_maps() {
        let mut score = sut();
        let carrier = score
            .notes()
            .values()
            .next()
            .expect("the fixture score holds a note")
            .clone();
        score.rests_mut().insert(
            carrier.id(),
            Rest::new(
                carrier.id(),
                carrier.staff(),
                carrier.voice(),
                Ticks::new(QUARTER_TICKS * 2),
                carrier.duration(),
            ),
        );
        let (meta, notes, spanners) = blocks(&score);
        assert_eq!(
            records(&notes).len(),
            3,
            "the fixture writes one rest line beside the two note lines"
        );

        let refusal = read(&meta, &notes, &spanners);

        let Err(ScoreError::Parse(ref message)) = refusal else {
            panic!(
                "a repeated identifier is refused as malformed text, and the reader answered {refusal:?}"
            );
        };
        assert!(
            message.contains(&carrier.id().get().to_string()),
            "the refusal of a repeated identifier names the number, and it reads {message}"
        );
    }
}
