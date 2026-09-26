//! The event vocabulary of the score aggregate.
//!
//! `ScoreEvent` states one fact about a change that a command already made. A
//! view reads the facts and refreshes what they name. Section 3.4 of
//! `roadmap/duet-v1/architecture.md` states the design.

use serde::{Deserialize, Serialize};

use crate::command::Selection;
use crate::ids::{MarkId, MeasureId, NoteId, PartId, SpannerId, StaffId};

/// One fact about a score change.
///
/// It derives `Eq`, because every arm payload supplies it. It derives no
/// `Hash` and no order, because VR1 names no map, no set, and no sort over an
/// event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScoreEvent {
    /// The score gained one part.
    PartAdded(PartId),
    /// The score lost one part.
    PartRemoved(PartId),
    /// The score gained one staff.
    StaffAdded(StaffId),
    /// The score lost one staff.
    StaffRemoved(StaffId),
    /// The score gained one note or one rest.
    NoteInserted(NoteId),
    /// One note or one rest carries a new value.
    NoteChanged(NoteId),
    /// The score lost these elements.
    ElementsRemoved(Selection),
    /// These elements sit at a new tick, at a new staff, or at both.
    ElementsMoved(Selection),
    /// The score gained one spanner.
    SpannerAdded(SpannerId),
    /// The score lost one spanner.
    SpannerRemoved(SpannerId),
    /// The score gained one mark.
    MarkAdded(MarkId),
    /// The score lost one mark.
    MarkRemoved(MarkId),
    /// A run of measures came or went, so every later measure moved.
    MeasuresChanged {
        /// The first measure of the run.
        from: MeasureId,
        /// How many measures the run holds.
        count: u16,
    },
    /// One measure carries a new key signature, a new meter, or both.
    SignatureChanged(MeasureId),
    /// One staff carries a new clef.
    ClefChanged(StaffId),
}

#[cfg(test)]
mod tests {
    use core::fmt::Debug;

    use duet_time::Ticks;
    use serde::Serialize;
    use serde::de::DeserializeOwned;

    use super::ScoreEvent;
    use crate::command::Selection;
    use crate::ids::{MarkId, MeasureId, NoteId, PartId, SpannerId, StaffId};

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

    /// Assert that `value` comes back equal from its own JSON form.
    fn assert_round_trip<Value>(value: &Value, kind: &str)
    where
        Value: Debug + PartialEq + Serialize + DeserializeOwned,
    {
        let text = serde_json::to_string(value).expect("a vocabulary value serializes");
        let read: Value = serde_json::from_str(&text).expect("a vocabulary value reads back");
        assert_eq!(read, *value, "{kind} survives the transport: {text}");
    }

    /// Every arm of `ScoreEvent`.
    fn every_event() -> Vec<ScoreEvent> {
        vec![
            ScoreEvent::PartAdded(ABSENT_PART),
            ScoreEvent::PartRemoved(ABSENT_PART),
            ScoreEvent::StaffAdded(ABSENT_STAFF),
            ScoreEvent::StaffRemoved(ABSENT_STAFF),
            ScoreEvent::NoteInserted(ABSENT_NOTE),
            ScoreEvent::NoteChanged(ABSENT_NOTE),
            ScoreEvent::ElementsRemoved(Selection::Notes(vec![ABSENT_NOTE])),
            ScoreEvent::ElementsMoved(Selection::Range {
                staff: ABSENT_STAFF,
                from: Ticks::ZERO,
                to: Ticks::new(QUARTER_TICKS),
            }),
            ScoreEvent::SpannerAdded(ABSENT_SPANNER),
            ScoreEvent::SpannerRemoved(ABSENT_SPANNER),
            ScoreEvent::MarkAdded(ABSENT_MARK),
            ScoreEvent::MarkRemoved(ABSENT_MARK),
            ScoreEvent::MeasuresChanged {
                from: ABSENT_MEASURE,
                count: 2,
            },
            ScoreEvent::SignatureChanged(ABSENT_MEASURE),
            ScoreEvent::ClefChanged(ABSENT_STAFF),
        ]
    }

    #[test]
    fn an_event_survives_the_transport() {
        for event in every_event() {
            assert_round_trip(&event, "an event");
        }
    }
}
