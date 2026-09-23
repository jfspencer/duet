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
