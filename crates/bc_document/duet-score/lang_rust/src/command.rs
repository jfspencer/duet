//! The command vocabulary of the score aggregate.
//!
//! `ScoreCommand` is the one intent type that reaches the aggregate root,
//! `Selection` names what an edit verb acts on, and `ScoreSelector` names what
//! a read-only query asks for. Sections 3.4 and 15.3 of
//! `roadmap/duet-v1/architecture.md` state the design.

use core::num::NonZeroU16;

use duet_time::{Meter, Ticks};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::ids::{
    LyricText, MarkId, MeasureId, NoteId, PartId, PartName, SpannerId, StaffId, VerseNumber,
    VoiceId,
};
use crate::model::{
    Accidental, Articulation, Clef, Duration, Dynamic, KeySignature, MarkKind, Note, Pitch, Rest,
    ScoreMark, Spanner, SpannerKind, TieState, VoiceType,
};

/// What a command acts on. One selection type serves every edit verb.
///
/// It derives `Eq`, because every arm payload supplies it. It derives no
/// `Hash` and no order, because VR1 names no map, no set, and no sort over a
/// selection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Selection {
    /// These notes and rests.
    Notes(Vec<NoteId>),
    /// These spanners.
    Spanners(Vec<SpannerId>),
    /// These score marks.
    Marks(Vec<MarkId>),
    /// Every element of one staff inside a tick range.
    Range {
        /// The staff that holds the elements.
        staff: StaffId,
        /// The tick where the range starts.
        from: Ticks,
        /// The tick where the range ends.
        to: Ticks,
    },
}

/// One intent against the score.
///
/// There is no `Cut` arm. A cut is `Score::copy` and then
/// `ScoreCommand::Remove`, in one undo transaction (section 3.4).
///
/// It derives `Eq`, because every arm payload supplies it. It derives no
/// `Hash` and no order, because VR1 names no map, no set, and no sort over a
/// command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreCommand {
    /// Add one part with a voice type and a name.
    AddPart {
        /// The vocal range of the new part.
        voice_type: VoiceType,
        /// The name that the user reads and edits.
        name: PartName,
    },
    /// Remove one part and every staff that it holds.
    RemovePart {
        /// The part to remove.
        part: PartId,
    },
    /// Add one staff to one part.
    AddStaff {
        /// The part that takes the new staff.
        part: PartId,
        /// The clef that the new staff opens with.
        clef: Clef,
    },
    /// Remove one staff and every element that it holds.
    RemoveStaff {
        /// The staff to remove.
        staff: StaffId,
    },
    /// Put a clef change on one staff at one tick.
    SetClef {
        /// The staff that takes the clef change.
        staff: StaffId,
        /// The tick where the new clef starts.
        at: Ticks,
        /// The clef that sounds from that tick.
        clef: Clef,
    },
    /// Insert empty measures after one measure.
    InsertMeasures {
        /// The measure that the new measures follow.
        after: MeasureId,
        /// How many measures the command inserts.
        count: NonZeroU16,
    },
    /// Remove measures from one measure onward.
    RemoveMeasures {
        /// The first measure to remove.
        from: MeasureId,
        /// How many measures the command removes.
        count: NonZeroU16,
    },
    /// Change the key signature of one measure.
    SetKeySignature {
        /// The measure that takes the key signature.
        measure: MeasureId,
        /// The key signature that starts at that measure.
        key: KeySignature,
    },
    /// Change the time signature of one measure.
    SetTimeSignature {
        /// The measure that takes the time signature.
        measure: MeasureId,
        /// The meter that starts at that measure.
        meter: Meter,
    },
    /// Add one score mark at one tick.
    AddMark {
        /// The tick where the mark sits.
        at: Ticks,
        /// What the mark means.
        kind: MarkKind,
    },
    /// Remove one score mark.
    RemoveMark {
        /// The mark to remove.
        mark: MarkId,
    },
    /// Insert one note.
    InsertNote {
        /// The staff that takes the note.
        staff: StaffId,
        /// The voice that takes the note.
        voice: VoiceId,
        /// The onset in ticks from score zero, not from a measure start.
        onset: Ticks,
        /// The written pitch of the note.
        pitch: Pitch,
        /// The written duration of the note.
        duration: Duration,
    },
    /// Insert one rest.
    InsertRest {
        /// The staff that takes the rest.
        staff: StaffId,
        /// The voice that takes the rest.
        voice: VoiceId,
        /// The onset in ticks from score zero, not from a measure start.
        onset: Ticks,
        /// The written duration of the rest.
        duration: Duration,
    },
    /// Change the pitch of every named note.
    SetPitch {
        /// The notes that the command changes.
        notes: Vec<NoteId>,
        /// How the command expresses the new pitch.
        pitch: PitchEdit,
    },
    /// Change the duration of every named note.
    SetDuration {
        /// The notes that the command changes.
        notes: Vec<NoteId>,
        /// The written duration that every note takes.
        duration: Duration,
    },
    /// Change the tie of one note.
    SetTie {
        /// The note that the command changes.
        note: NoteId,
        /// Whether the note starts a tie, ends one, or does both.
        tie: TieState,
    },
    /// Replace the articulations of every named note.
    SetArticulations {
        /// The notes that the command changes.
        notes: Vec<NoteId>,
        /// The articulations that every note takes.
        articulations: SmallVec<[Articulation; 2]>,
    },
    /// Put one lyric syllable on one note, in one verse.
    SetLyric {
        /// The note that takes the syllable.
        note: NoteId,
        /// The verse that holds the syllable.
        verse: VerseNumber,
        /// The syllable text.
        text: LyricText,
    },
    /// Put a dynamic mark on one staff at one tick.
    SetDynamic {
        /// The staff that takes the dynamic mark.
        staff: StaffId,
        /// The tick where the dynamic mark sits.
        onset: Ticks,
        /// The printed dynamic.
        dynamic: Dynamic,
    },
    /// Add one spanner between two notes.
    AddSpanner {
        /// What the spanner draws.
        kind: SpannerKind,
        /// The note where the spanner starts.
        from: NoteId,
        /// The note where the spanner ends.
        to: NoteId,
    },
    /// Remove one spanner.
    RemoveSpanner {
        /// The spanner to remove.
        spanner: SpannerId,
    },
    /// Move every named note into one voice.
    SetVoice {
        /// The notes that the command moves.
        notes: Vec<NoteId>,
        /// The voice that takes the notes.
        voice: VoiceId,
    },
    /// Move every named note into one staff of one part.
    SetPart {
        /// The notes that the command moves.
        notes: Vec<NoteId>,
        /// The part that takes the notes.
        part: PartId,
        /// The staff of that part that takes the notes.
        staff: StaffId,
    },
    /// Move a selection along the timeline, and to another staff.
    Move {
        /// What the command moves.
        selection: Selection,
        /// How far the selection moves. A negative value moves it earlier.
        by: Ticks,
        /// The staff that takes the selection, or `None` to keep the staff.
        to_staff: Option<StaffId>,
    },
    /// Copy a selection to another tick and keep the original.
    Duplicate {
        /// What the command copies.
        selection: Selection,
        /// The tick where the copy starts.
        at: Ticks,
    },
    /// Remove a selection.
    Remove {
        /// What the command removes.
        selection: Selection,
    },
    /// Insert a detached copy at one tick, in one staff and voice.
    ///
    /// The clipboard is boxed, as section 3.4 declares it. It is the one arm
    /// that holds two vectors, and the box keeps both out of every
    /// `ScoreCommand` value.
    ///
    /// **It keeps the identifier of every element whose identifier the score
    /// does not already hold**, and mints a new one for the rest. A cut and a
    /// paste of one selection therefore return the score to its earlier
    /// content, and every reference into that selection survives. It is the one
    /// arm that restores an element under the identifier it had, so the inverse
    /// of a `Remove` is a `Paste`.
    ///
    /// **A mark and a spanner ignore the staff and the voice.** A mark names a
    /// tick and a spanner names two notes, so neither holds a staff of its own,
    /// and the two identifiers of this arm reach the notes and the rests alone.
    Paste {
        /// The detached copy that the command inserts.
        clipboard: Box<Clipboard>,
        /// The tick where the paste starts.
        at: Ticks,
        /// The staff that takes the paste.
        staff: StaffId,
        /// The voice that takes the paste.
        voice: VoiceId,
    },
}

/// How a pitch edit is expressed. The context menu offers all four.
///
/// It derives `Copy`, because every field is `Copy` and VR5 makes the derive
/// the compiler's demand below the size bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PitchEdit {
    /// Take this written pitch, whatever the note carries now.
    Absolute(Pitch),
    /// Move the pitch by this many semitones.
    BySemitones(i8),
    /// Move the pitch by this many octaves.
    ByOctaves(i8),
    /// Keep the sounding pitch and write it with this accidental.
    Respell(Accidental),
}

/// A detached copy of a selection. A copy and a cut each produce one.
///
/// It carries all four content kinds, because `ScoreCommand::Paste` is the one
/// arm that restores an element under the identifier it had. `Remove` inverts
/// to a `Paste` of the clipboard that `Score::copy` produced before the
/// removal, so a cut of a rest, of a spanner, or of a mark is undone as
/// exactly as a cut of a note.
///
/// It derives `Eq`, because every field supplies it. It derives no `Copy`,
/// because it holds four vectors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clipboard {
    /// The tick that every onset in this copy is measured from.
    origin: Ticks,
    /// The notes of this copy.
    notes: Vec<Note>,
    /// The rests of this copy.
    rests: Vec<Rest>,
    /// The spanners of this copy.
    spanners: Vec<Spanner>,
    /// The score marks of this copy.
    marks: Vec<ScoreMark>,
}

impl Clipboard {
    /// A detached copy of the given notes, rests, spanners, and marks.
    #[must_use]
    pub const fn new(
        origin: Ticks,
        notes: Vec<Note>,
        rests: Vec<Rest>,
        spanners: Vec<Spanner>,
        marks: Vec<ScoreMark>,
    ) -> Self {
        Self {
            origin,
            notes,
            rests,
            spanners,
            marks,
        }
    }

    /// The tick that every onset in this copy is measured from.
    #[must_use]
    pub const fn origin(&self) -> Ticks {
        self.origin
    }

    /// The notes of this copy.
    #[must_use]
    pub fn notes(&self) -> &[Note] {
        &self.notes
    }

    /// The rests of this copy.
    #[must_use]
    pub fn rests(&self) -> &[Rest] {
        &self.rests
    }

    /// The spanners of this copy.
    #[must_use]
    pub fn spanners(&self) -> &[Spanner] {
        &self.spanners
    }

    /// The score marks of this copy.
    #[must_use]
    pub fn marks(&self) -> &[ScoreMark] {
        &self.marks
    }

    /// Whether this copy holds no element at all.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.notes.is_empty()
            && self.rests.is_empty()
            && self.spanners.is_empty()
            && self.marks.is_empty()
    }
}

/// What a read-only score query asks for.
///
/// It derives `Copy`, because every field is `Copy` and VR5 makes the derive
/// the compiler's demand below the size bound.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreSelector {
    /// One part and everything below it.
    Part(PartId),
    /// One staff and everything below it.
    Staff(StaffId),
    /// A run of measures across every staff.
    Measures {
        /// The first measure of the run.
        from: MeasureId,
        /// How many measures the run holds.
        count: NonZeroU16,
    },
    /// Every element of one staff inside a tick range.
    Range {
        /// The staff that holds the elements.
        staff: StaffId,
        /// The tick where the range starts.
        from: Ticks,
        /// The tick where the range ends.
        to: Ticks,
    },
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;
    use core::num::{NonZeroU8, NonZeroU16};

    use duet_time::{Meter, NoteValue, Ticks, Tuplet};
    use serde::Serialize;
    use serde::de::DeserializeOwned;
    use smallvec::SmallVec;

    use super::{Clipboard, PitchEdit, ScoreCommand, Selection};
    use crate::ids::{
        LyricText, MarkId, MeasureId, NoteId, PartId, PartName, RehearsalText, SpannerId, StaffId,
        VerseNumber, VoiceId,
    };
    use crate::model::{
        Accidental, Articulation, Clef, Duration, Dynamic, KeySignature, MarkKind, Note, Pitch,
        RepeatSide, Rest, ScoreMark, Spanner, SpannerKind, Step, TieState, VoiceType,
    };

    /// The tick count of one quarter note.
    const QUARTER_TICKS: i64 = 1_920;

    /// A part identifier that no fixture of this module puts into a score.
    const ABSENT_PART: PartId = PartId::new(9_000);

    /// A staff identifier that no fixture of this module puts into a score.
    const ABSENT_STAFF: StaffId = StaffId::new(9_001);

    /// A measure identifier that no fixture of this module puts into a score.
    const ABSENT_MEASURE: MeasureId = MeasureId::new(9_002);

    /// A note identifier that no fixture of this module puts into a score.
    const ABSENT_NOTE: NoteId = NoteId::new(9_003);

    /// A spanner identifier that no fixture of this module puts into a score.
    const ABSENT_SPANNER: SpannerId = SpannerId::new(9_004);

    /// A mark identifier that no fixture of this module puts into a score.
    const ABSENT_MARK: MarkId = MarkId::new(9_005);

    /// A voice identifier that no fixture of this module puts into a score.
    const ABSENT_VOICE: VoiceId = VoiceId::new(9_006);

    /// A rest identifier that no fixture of this module puts into a score.
    const ABSENT_REST: NoteId = NoteId::new(9_007);

    /// A run of one measure.
    const ONE_MEASURE: NonZeroU16 = NonZeroU16::MIN;

    /// A quarter note with no dot and no tuplet.
    fn quarter() -> Duration {
        Duration::new(NoteValue::Quarter, 0, None)
    }

    /// A half note with no dot and no tuplet.
    fn half() -> Duration {
        Duration::new(NoteValue::Half, 0, None)
    }

    /// The pitch of `step` in octave four, with no alteration.
    fn natural(step: Step) -> Pitch {
        Pitch::new(4, step, 0)
    }

    /// A meter of three quarter beats in one bar.
    fn three_four() -> Meter {
        Meter::new(
            NonZeroU8::new(3).expect("three is not zero"),
            NoteValue::Quarter,
        )
    }

    /// The first verse.
    fn verse_one() -> VerseNumber {
        VerseNumber::new(NonZeroU8::MIN)
    }

    /// The lyric syllable that `text` spells.
    fn lyric(text: &str) -> LyricText {
        LyricText::new(text).expect("a syllable with text")
    }

    /// An `AddPart` command for a soprano part that `name` names.
    fn add_part(name: &str) -> ScoreCommand {
        ScoreCommand::AddPart {
            voice_type: VoiceType::Soprano,
            name: PartName::new(name).expect("a name with text"),
        }
    }

    /// A tuplet of three notes in the time of two.
    fn triplet() -> Tuplet {
        Tuplet::new(
            NonZeroU8::new(3).expect("three is not zero"),
            NonZeroU8::new(2).expect("two is not zero"),
        )
    }

    /// Assert that `value` comes back equal from its own JSON form.
    fn assert_round_trip<Value>(value: &Value, kind: &str)
    where
        Value: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let text = serde_json::to_string(value).expect("a vocabulary value serializes");
        let read: Value = serde_json::from_str(&text).expect("a vocabulary value reads back");
        assert_eq!(read, *value, "{kind} survives the transport: {text}");
    }

    /// Every arm of `Selection`.
    fn every_selection() -> Vec<Selection> {
        vec![
            Selection::Notes(vec![ABSENT_NOTE, ABSENT_REST]),
            Selection::Spanners(vec![ABSENT_SPANNER]),
            Selection::Marks(vec![ABSENT_MARK]),
            Selection::Range {
                staff: ABSENT_STAFF,
                from: Ticks::ZERO,
                to: Ticks::new(QUARTER_TICKS),
            },
        ]
    }

    /// A clipboard that holds one element of each content kind.
    fn full_clipboard() -> Clipboard {
        let note = Note::new(
            ABSENT_NOTE,
            ABSENT_STAFF,
            ABSENT_VOICE,
            Ticks::ZERO,
            Duration::new(NoteValue::Eighth, 1, Some(triplet())),
            natural(Step::C),
        );
        let rest = Rest::new(
            ABSENT_REST,
            ABSENT_STAFF,
            ABSENT_VOICE,
            Ticks::new(QUARTER_TICKS),
            quarter(),
        );
        let spanner = Spanner::new(ABSENT_SPANNER, SpannerKind::Slur, ABSENT_NOTE, ABSENT_REST);
        let mark = ScoreMark::new(
            ABSENT_MARK,
            Ticks::new(QUARTER_TICKS),
            MarkKind::Rehearsal(RehearsalText::new("A").expect("a rehearsal text with text")),
        );
        Clipboard::new(
            Ticks::ZERO,
            vec![note],
            vec![rest],
            vec![spanner],
            vec![mark],
        )
    }

    /// One command of every arm that names a part, a staff, or a measure.
    fn structure_commands() -> Vec<ScoreCommand> {
        vec![
            add_part("Soprano 1"),
            ScoreCommand::RemovePart { part: ABSENT_PART },
            ScoreCommand::AddStaff {
                part: ABSENT_PART,
                clef: Clef::Percussion,
            },
            ScoreCommand::RemoveStaff {
                staff: ABSENT_STAFF,
            },
            ScoreCommand::SetClef {
                staff: ABSENT_STAFF,
                at: Ticks::new(QUARTER_TICKS),
                clef: Clef::TrebleOctaveDown,
            },
            ScoreCommand::InsertMeasures {
                after: ABSENT_MEASURE,
                count: ONE_MEASURE,
            },
            ScoreCommand::RemoveMeasures {
                from: ABSENT_MEASURE,
                count: ONE_MEASURE,
            },
            ScoreCommand::SetKeySignature {
                measure: ABSENT_MEASURE,
                key: KeySignature::new(-3, false),
            },
            ScoreCommand::SetTimeSignature {
                measure: ABSENT_MEASURE,
                meter: three_four(),
            },
        ]
    }

    /// One command of every arm that names a note, a rest, a mark, or a spanner.
    fn content_commands() -> Vec<ScoreCommand> {
        vec![
            ScoreCommand::AddMark {
                at: Ticks::ZERO,
                kind: MarkKind::Repeat(RepeatSide::End),
            },
            ScoreCommand::RemoveMark { mark: ABSENT_MARK },
            ScoreCommand::InsertNote {
                staff: ABSENT_STAFF,
                voice: ABSENT_VOICE,
                onset: Ticks::ZERO,
                pitch: natural(Step::B),
                duration: quarter(),
            },
            ScoreCommand::InsertRest {
                staff: ABSENT_STAFF,
                voice: ABSENT_VOICE,
                onset: Ticks::new(QUARTER_TICKS),
                duration: half(),
            },
            ScoreCommand::SetPitch {
                notes: vec![ABSENT_NOTE],
                pitch: PitchEdit::Respell(Accidental::DoubleFlat),
            },
            ScoreCommand::SetDuration {
                notes: vec![ABSENT_NOTE],
                duration: Duration::new(NoteValue::Sixteenth, 2, Some(triplet())),
            },
            ScoreCommand::SetTie {
                note: ABSENT_NOTE,
                tie: TieState::new(true, true),
            },
            ScoreCommand::SetArticulations {
                notes: vec![ABSENT_NOTE],
                articulations: SmallVec::from_slice(&[
                    Articulation::Staccato,
                    Articulation::Fermata,
                ]),
            },
            ScoreCommand::SetLyric {
                note: ABSENT_NOTE,
                verse: verse_one(),
                text: lyric("la"),
            },
            ScoreCommand::SetDynamic {
                staff: ABSENT_STAFF,
                onset: Ticks::ZERO,
                dynamic: Dynamic::Sfz,
            },
            ScoreCommand::AddSpanner {
                kind: SpannerKind::Slur,
                from: ABSENT_NOTE,
                to: ABSENT_REST,
            },
            ScoreCommand::RemoveSpanner {
                spanner: ABSENT_SPANNER,
            },
        ]
    }

    /// One command of every arm that moves, copies, or removes a selection.
    fn edit_commands() -> Vec<ScoreCommand> {
        vec![
            ScoreCommand::SetVoice {
                notes: vec![ABSENT_NOTE],
                voice: ABSENT_VOICE,
            },
            ScoreCommand::SetPart {
                notes: vec![ABSENT_NOTE],
                part: ABSENT_PART,
                staff: ABSENT_STAFF,
            },
            ScoreCommand::Move {
                selection: Selection::Notes(vec![ABSENT_NOTE]),
                by: Ticks::new(-QUARTER_TICKS),
                to_staff: Some(ABSENT_STAFF),
            },
            ScoreCommand::Move {
                selection: Selection::Marks(vec![ABSENT_MARK]),
                by: Ticks::new(QUARTER_TICKS),
                to_staff: None,
            },
            ScoreCommand::Duplicate {
                selection: Selection::Spanners(vec![ABSENT_SPANNER]),
                at: Ticks::new(QUARTER_TICKS),
            },
            ScoreCommand::Remove {
                selection: Selection::Notes(vec![ABSENT_NOTE, ABSENT_REST]),
            },
            ScoreCommand::Paste {
                clipboard: Box::new(full_clipboard()),
                at: Ticks::new(QUARTER_TICKS),
                staff: ABSENT_STAFF,
                voice: ABSENT_VOICE,
            },
        ]
    }

    #[test]
    fn a_selection_survives_the_transport() {
        for selection in every_selection() {
            assert_round_trip(&selection, "a selection");
        }
    }

    #[test]
    fn a_command_survives_the_transport() {
        for command in structure_commands()
            .into_iter()
            .chain(content_commands())
            .chain(edit_commands())
        {
            assert_round_trip(&command, "a command");
        }
    }

    /// One note at the given onset, with no tie and no articulation.
    fn note_at(id: u64, onset: i64) -> Note {
        Note::new(
            NoteId::new(id),
            StaffId::new(1),
            VoiceId::new(1),
            Ticks::new(onset),
            Duration::new(NoteValue::Quarter, 0, None),
            Pitch::new(4, Step::C, 0),
        )
    }

    /// A clipboard that holds two notes and nothing else.
    fn sut() -> Clipboard {
        Clipboard::new(
            Ticks::new(480),
            vec![note_at(1, 480), note_at(2, 960)],
            Vec::new(),
            Vec::new(),
            Vec::new(),
        )
    }

    #[test]
    fn a_clipboard_answers_what_it_took() {
        let clipboard = sut();
        assert_eq!(
            clipboard.origin(),
            Ticks::new(480),
            "the clipboard answers the origin the caller gave"
        );
        assert_eq!(clipboard.notes().len(), 2, "the clipboard holds both notes");
        assert!(clipboard.rests().is_empty(), "the clipboard holds no rest");
        assert!(
            clipboard.spanners().is_empty(),
            "the clipboard holds no spanner"
        );
        assert!(clipboard.marks().is_empty(), "the clipboard holds no mark");
        assert!(
            !clipboard.is_empty(),
            "a clipboard that holds a note is not empty"
        );
    }

    #[test]
    fn a_paste_carries_the_clipboard_behind_a_box() {
        let command = ScoreCommand::Paste {
            clipboard: Box::new(sut()),
            at: Ticks::new(1920),
            staff: StaffId::new(1),
            voice: VoiceId::new(1),
        };
        let round_trip = serde_json::to_string(&command).expect("a command serializes");
        let read: ScoreCommand = serde_json::from_str(&round_trip).expect("a command reads back");
        assert_eq!(read, command, "the paste survives the transport");
    }

    #[test]
    fn a_move_takes_a_negative_tick_count() {
        let command = ScoreCommand::Move {
            selection: Selection::Notes(vec![NoteId::new(1)]),
            by: Ticks::new(-480),
            to_staff: None,
        };
        let round_trip = serde_json::to_string(&command).expect("a command serializes");
        let read: ScoreCommand = serde_json::from_str(&round_trip).expect("a command reads back");
        assert_eq!(read, command, "a move earlier survives the transport");
    }
}
