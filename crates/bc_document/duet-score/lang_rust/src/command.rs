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
    Accidental, Articulation, Clef, Duration, Dynamic, KeySignature, MarkKind, Note, Pitch,
    Spanner, SpannerKind, TieState, VoiceType,
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
/// It derives `Eq`, because every field supplies it. It derives no `Copy`,
/// because it holds two vectors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Clipboard {
    /// The tick that every onset in this copy is measured from.
    origin: Ticks,
    /// The notes of this copy.
    notes: Vec<Note>,
    /// The spanners of this copy.
    spanners: Vec<Spanner>,
}

impl Clipboard {
    /// A detached copy of the given notes and spanners.
    #[must_use]
    pub const fn new(origin: Ticks, notes: Vec<Note>, spanners: Vec<Spanner>) -> Self {
        Self {
            origin,
            notes,
            spanners,
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

    /// The spanners of this copy.
    #[must_use]
    pub fn spanners(&self) -> &[Spanner] {
        &self.spanners
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
    use duet_time::{NoteValue, Ticks};

    use super::{Clipboard, ScoreCommand, Selection};
    use crate::ids::{NoteId, StaffId, VoiceId};
    use crate::model::{Duration, Note, Pitch, Step};

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

    /// A clipboard that holds two notes and no spanner.
    fn sut() -> Clipboard {
        Clipboard::new(
            Ticks::new(480),
            vec![note_at(1, 480), note_at(2, 960)],
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
        assert!(
            clipboard.spanners().is_empty(),
            "the clipboard holds no spanner"
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
