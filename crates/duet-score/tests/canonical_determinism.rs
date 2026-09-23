//! The determinism property of the canonical document.
//!
//! Section 3.6 of `roadmap/duet-v1/architecture.md` states that the same score
//! produces the same bytes, that `score/notes.jsonl` sorts by
//! `(staff, voice, onset, pitch, id)`, and that `score/spanners.jsonl` sorts by
//! `(from, kind, id)`. The insert order therefore never reaches a file.
//!
//! Every fixture of this target gives the elements identifiers whose order
//! contradicts the sort key, so a writer that walks the identifier map rather
//! than the sort key produces a different block.

#[cfg(test)]
mod tests {
    use duet_score::{
        Clef, Clipboard, Duration, Note, NoteId, NoteValue, PartName, Pitch, Score, ScoreCommand,
        ScoreEvent, Spanner, SpannerId, SpannerKind, StaffId, Step, VoiceId, VoiceType, read,
        write,
    };
    use duet_time::Ticks;
    use serde_json::Value;

    /// The tick count of one quarter note.
    const QUARTER_TICKS: i64 = 1_920;

    /// The identifiers of the four fixture notes, in the order they are built.
    ///
    /// The first identifier carries the last onset, so the identifier order is
    /// the reverse of the `(staff, voice, onset, pitch, id)` order.
    const NOTE_IDS: [u64; 4] = [200, 201, 202, 203];

    /// The identifiers of the four fixture spanners, in the order they are
    /// built.
    const SPANNER_IDS: [u64; 4] = [300, 301, 302, 303];

    /// The note identifiers that `notes.jsonl` must carry, top line first.
    ///
    /// The four notes share one staff and one voice, so the onset decides, and
    /// the onsets descend with the identifier.
    const NOTES_IN_FILE_ORDER: [u64; 4] = [203, 202, 201, 200];

    /// The spanner identifiers that `spanners.jsonl` must carry, top line
    /// first.
    ///
    /// The key is `(from, kind, id)`. Spanner 300 starts at note 202 and every
    /// other spanner starts at note 203. `SpannerKind::Slur` precedes
    /// `SpannerKind::Crescendo`, so spanner 302 comes last, and the identifier
    /// breaks the tie between spanner 301 and spanner 303.
    const SPANNERS_IN_FILE_ORDER: [u64; 4] = [300, 301, 303, 302];

    /// A score with one part and one staff, and the staff and the voice of it.
    struct Stage {
        /// The score under test.
        score: Score,
        /// The staff that the stage added.
        staff: StaffId,
        /// The voice that the staff holds.
        voice: VoiceId,
    }

    /// A score with one part, one staff, one voice, and no note.
    ///
    /// Two calls mint the same identifiers, so two stages differ in nothing.
    fn stage() -> Stage {
        let mut score = Score::new();
        let part_result = score
            .apply(ScoreCommand::AddPart {
                voice_type: VoiceType::Soprano,
                name: PartName::new("Soprano 1").expect("a name with text"),
            })
            .expect("AddPart is accepted");
        let part = part_result
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::PartAdded(part) = *event {
                    Some(part)
                } else {
                    None
                }
            })
            .expect("an accepted AddPart reports PartAdded");
        let staff_result = score
            .apply(ScoreCommand::AddStaff {
                part,
                clef: Clef::Treble,
            })
            .expect("AddStaff is accepted");
        let staff = staff_result
            .events
            .iter()
            .find_map(|event| {
                if let ScoreEvent::StaffAdded(staff) = *event {
                    Some(staff)
                } else {
                    None
                }
            })
            .expect("an accepted AddStaff reports StaffAdded");
        let voice = *score
            .staves()
            .get(&staff)
            .expect("the score holds the staff it just added")
            .voices()
            .first()
            .expect("AddStaff gives the staff one voice");
        Stage {
            score,
            staff,
            voice,
        }
    }

    /// One quarter note of `step` in octave four.
    fn note(id: u64, staff: StaffId, voice: VoiceId, beats: i64, step: Step) -> Note {
        Note::new(
            NoteId::new(id),
            staff,
            voice,
            Ticks::new(beats.saturating_mul(QUARTER_TICKS)),
            Duration::new(NoteValue::Quarter, 0, None),
            Pitch::new(4, step, 0),
        )
    }

    /// The four fixture notes, in identifier order.
    ///
    /// Note 200 sits last on the timeline and note 203 sits first, so the
    /// identifier order and the onset order disagree at every position.
    fn fixture_notes(staff: StaffId, voice: VoiceId) -> [Note; 4] {
        [
            note(NOTE_IDS[0], staff, voice, 3, Step::F),
            note(NOTE_IDS[1], staff, voice, 2, Step::E),
            note(NOTE_IDS[2], staff, voice, 1, Step::D),
            note(NOTE_IDS[3], staff, voice, 0, Step::C),
        ]
    }

    /// The four fixture spanners, in identifier order.
    ///
    /// Spanner 301 and spanner 303 share a start note and a kind, so only the
    /// identifier separates them.
    fn fixture_spanners() -> [Spanner; 4] {
        [
            Spanner::new(
                SpannerId::new(SPANNER_IDS[0]),
                SpannerKind::Slur,
                NoteId::new(202),
                NoteId::new(201),
            ),
            Spanner::new(
                SpannerId::new(SPANNER_IDS[1]),
                SpannerKind::Slur,
                NoteId::new(203),
                NoteId::new(202),
            ),
            Spanner::new(
                SpannerId::new(SPANNER_IDS[2]),
                SpannerKind::Crescendo,
                NoteId::new(203),
                NoteId::new(201),
            ),
            Spanner::new(
                SpannerId::new(SPANNER_IDS[3]),
                SpannerKind::Slur,
                NoteId::new(203),
                NoteId::new(200),
            ),
        ]
    }

    /// A score that holds every fixture element, taken in the given order.
    ///
    /// A paste keeps the identifier that the clipboard carries when the score
    /// holds no such element, so two orders build one content under one set of
    /// identifiers. The clipboard origin and the paste tick are both zero, so
    /// no onset moves.
    fn score_in_order(note_order: [usize; 4], spanner_order: [usize; 4]) -> Score {
        let mut staged = stage();
        let notes = fixture_notes(staged.staff, staged.voice);
        let spanners = fixture_spanners();
        let clipboard = Clipboard::new(
            Ticks::ZERO,
            note_order
                .iter()
                .map(|place| notes[*place].clone())
                .collect(),
            Vec::new(),
            spanner_order
                .iter()
                .map(|place| spanners[*place].clone())
                .collect(),
            Vec::new(),
        );
        staged
            .score
            .apply(ScoreCommand::Paste {
                clipboard: Box::new(clipboard),
                at: Ticks::ZERO,
                staff: staged.staff,
                voice: staged.voice,
            })
            .expect("a paste into an empty staff is accepted");
        staged.score
    }

    /// The block as text. Every canonical block is UTF-8.
    fn text(block: &[u8]) -> &str {
        core::str::from_utf8(block).expect("a canonical block is UTF-8")
    }

    /// The identifier counter that one meta block states.
    fn counter_of(meta: &[u8]) -> u64 {
        let document: Value =
            serde_json::from_str(text(meta)).expect("meta.json is one JSON object");
        document
            .get("next_id")
            .and_then(Value::as_u64)
            .expect("meta.json carries the identifier counter")
    }

    /// The `id` field of every record of one JSON Lines block, in file order.
    fn identifiers_in_file_order(block: &[u8]) -> Vec<u64> {
        text(block)
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(|line| {
                let record: Value =
                    serde_json::from_str(line).expect("a JSON Lines record is one JSON object");
                record
                    .get("id")
                    .and_then(Value::as_u64)
                    .expect("every canonical record carries its identifier")
            })
            .collect()
    }

    #[test]
    fn canonical_determinism() {
        let ascending = score_in_order([0, 1, 2, 3], [0, 1, 2, 3]);
        let first = write(&ascending).expect("a score writes");
        let second = write(&ascending).expect("a score writes a second time");
        assert_eq!(
            first.meta(),
            second.meta(),
            "two writes of one score give one meta.json block"
        );
        assert_eq!(
            first.notes(),
            second.notes(),
            "two writes of one score give one notes.jsonl block"
        );
        assert_eq!(
            first.spanners(),
            second.spanners(),
            "two writes of one score give one spanners.jsonl block"
        );

        let shuffled = score_in_order([3, 0, 2, 1], [2, 0, 3, 1]);
        assert_eq!(
            shuffled, ascending,
            "the two insert orders build one content"
        );
        assert_eq!(
            shuffled.revision(),
            ascending.revision(),
            "the two fixtures run one command sequence, so they stand at one revision"
        );
        let third = write(&shuffled).expect("the score of the second insert order writes");
        assert_eq!(
            counter_of(third.meta()),
            counter_of(first.meta()),
            "meta.json carries the identifier counter and the revision, and content equality reads neither, so the premise of this test needs both pinned: the two fixtures mint one set of identifiers"
        );
        assert_eq!(
            third.meta(),
            first.meta(),
            "the insert order does not reach meta.json"
        );
        assert_eq!(
            third.notes(),
            first.notes(),
            "the insert order does not reach notes.jsonl"
        );
        assert_eq!(
            third.spanners(),
            first.spanners(),
            "the insert order does not reach spanners.jsonl"
        );

        let read_back = read(
            text(first.meta()),
            text(first.notes()),
            text(first.spanners()),
        )
        .expect("the written document reads back");
        assert_eq!(
            read_back, ascending,
            "a read of the written blocks gives a score equal to the written score"
        );
    }

    #[test]
    fn canonical_sorts_by_the_two_keys_of_section_3_6() {
        let score = score_in_order([0, 1, 2, 3], [0, 1, 2, 3]);
        let document = write(&score).expect("a score writes");
        assert_eq!(
            identifiers_in_file_order(document.notes()),
            NOTES_IN_FILE_ORDER.to_vec(),
            "notes.jsonl sorts by (staff, voice, onset, pitch, id) and not by the identifier map"
        );
        assert_eq!(
            identifiers_in_file_order(document.spanners()),
            SPANNERS_IN_FILE_ORDER.to_vec(),
            "spanners.jsonl sorts by (from, kind, id), and the identifier breaks the tie"
        );
    }
}
