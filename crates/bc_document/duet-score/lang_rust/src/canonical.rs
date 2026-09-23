//! The canonical document of the score: the reader and the writer.
//!
//! Section 3.6 of `roadmap/duet-v1/architecture.md` states the four
//! properties, the three files, and the two sort keys. `score/meta.json` is
//! one JSON object, `score/notes.jsonl` holds one note or one rest per line in
//! `(staff, voice, onset, pitch, id)` order, and `score/spanners.jsonl` holds
//! one spanner per line in `(from, kind, id)` order.
//!
//! Four rules of this module answer what section 3.6 leaves open.
//!
//! 1. **A note stands before a rest** at one `(staff, voice, onset)`, and two
//!    rests then stand in identifier order. The reader of a diff sees the
//!    sounding event first, and the rule is total, so each line holds one
//!    place and the output stays unique.
//! 2. **`score/meta.json` carries `revision` and `next_id`** beside `schema`,
//!    and `read` restores both. Section 3.2 makes the identifier counter a
//!    counter that the document persists, so no mint after a reload aliases a
//!    live identifier.
//! 3. **Every block ends with a newline**: the meta object and each record of
//!    both JSON Lines files. One rule serves all three files.
//! 4. **An unknown key reaches the `extra` bag of the record that carried it,
//!    and no other bag.** The `ImportWarning` channel of edge rule 2 is a
//!    `duet-command` type, and that crate sits above this one, so the `read`
//!    signature of section 3.6 carries no warning channel. The bag is the
//!    whole report at this boundary (`escalation:T2-2`).
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
use crate::ids::{NoteId, Revision, StaffId, VoiceId};
use crate::model::{
    Measure, Note, Part, Pitch, Rest, SCHEMA, Score, ScoreMark, Spanner, Staff, Voice,
    note_order_key, rest_order_key, spanner_order_key,
};

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

/// Read a canonical document and migrate it to the current schema.
///
/// # Errors
/// Returns `ScoreError::Schema` for a document above the schema that this
/// build reads, and `ScoreError::Parse` for malformed text.
pub fn read(meta: &str, notes: &str, spanners: &str) -> Result<Score, ScoreError> {
    let found = schema_of(meta)?;
    if found > SCHEMA {
        return Err(ScoreError::Schema {
            found,
            expected: SCHEMA,
        });
    }
    let stored: Meta = from_json(meta)?;
    let mut score = stored.into_score();
    for line in text_records(notes) {
        let record: TimedRecord = from_json(line)?;
        match record {
            TimedRecord::Note(note) => {
                score.notes_mut().insert(note.id(), note);
            },
            TimedRecord::Rest(rest) => {
                score.rests_mut().insert(rest.id(), rest);
            },
        }
    }
    for line in text_records(spanners) {
        let spanner: Spanner = from_json(line)?;
        score.spanners_mut().insert(spanner.id(), spanner);
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
    fn into_score(self) -> Score {
        let mut score = Score::new();
        *score.parts_mut() = keyed(self.parts, Part::id);
        *score.staves_mut() = keyed(self.staves, Staff::id);
        *score.voices_mut() = keyed(self.voices, Voice::id);
        *score.measures_mut() = keyed(self.measures, Measure::id);
        *score.marks_mut() = keyed(self.marks, ScoreMark::id);
        score.set_tempo_map(self.tempo_map);
        score.set_revision(self.revision);
        score.set_next_id(self.next_id);
        *score.extra_mut() = self.extra;
        score
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
fn keyed<Id: Ord, Record>(list: Vec<Record>, id: impl Fn(&Record) -> Id) -> BTreeMap<Id, Record> {
    list.into_iter()
        .map(|record| (id(&record), record))
        .collect()
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
    use crate::model::{Clef, Duration, Pitch, Rest, SCHEMA, Score, Step, VoiceType};

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
        let outcome = read(&object_text(object), &notes, &spanners);
        assert!(
            !matches!(outcome, Err(ScoreError::Schema { .. })),
            "the schema gate compares with a greater-than test, so a document below the schema this build reads runs the migration chain rather than a refusal"
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
}
