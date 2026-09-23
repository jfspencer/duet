//! Every refusal the score aggregate answers with.

use duet_time::{SchemaVersion, Ticks, TimeError};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::ids::{ElementRef, MeasureId, NoteId, PartId, StaffId, VoiceId};

/// Every way the score aggregate refuses.
///
/// It derives serde, because `GatewayError::Score` wraps it and every refusal
/// crosses the transport. Section 15.5 of `roadmap/duet-v1/architecture.md`
/// states the transport rule.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum ScoreError {
    /// A command names an element that the score does not hold.
    #[error("the score holds no {0}")]
    MissingElement(ElementRef),
    /// A command names a part that the score does not hold.
    #[error("the score holds no part with identifier {}", .0.get())]
    MissingPart(PartId),
    /// A command names a staff that the score does not hold.
    #[error("the score holds no staff with identifier {}", .0.get())]
    MissingStaff(StaffId),
    /// A command names a measure that the score does not hold.
    #[error("the score holds no measure with identifier {}", .0.get())]
    MissingMeasure(MeasureId),
    /// A note already sounds at that onset in that staff and voice.
    #[error(
        "a note already sounds in staff {} voice {} at tick {}",
        staff.get(),
        voice.get(),
        onset.get()
    )]
    OverlappingNote {
        /// The staff of the note that the command inserts.
        staff: StaffId,
        /// The voice of the note that the command inserts.
        voice: VoiceId,
        /// The onset of the note that the command inserts.
        onset: Ticks,
    },
    /// A tie holds one pitch, and the two notes carry different pitches.
    #[error(
        "a tie holds one pitch, and note {} and note {} carry different pitches",
        from.get(),
        to.get()
    )]
    TieAcrossPitch {
        /// The note that starts the tie.
        from: NoteId,
        /// The note that stops the tie.
        to: NoteId,
    },
    /// A measure holds a note or a rest, so a remove refuses it.
    #[error("measure {} holds a note or a rest", .0.get())]
    MeasureNotEmpty(MeasureId),
    /// A document states a schema number that this build does not read.
    #[error(
        "the document states schema {} and this build reads schema {}",
        found.get(),
        expected.get()
    )]
    Schema {
        /// The schema number that the document states.
        found: SchemaVersion,
        /// The schema number that this build reads.
        expected: SchemaVersion,
    },
    /// A document holds text that the reader cannot parse.
    #[error("the document text is malformed: {0}")]
    Parse(Box<str>),
    /// A value has no form that the writer can put on disk.
    #[error("the value has no form that the writer can put on disk: {0}")]
    Serialize(Box<str>),
    /// The time kernel refused a conversion or a map query.
    #[error(transparent)]
    Time(#[from] TimeError),
}

#[cfg(test)]
mod tests {
    use duet_time::{SchemaVersion, TimeError};

    use super::ScoreError;
    use crate::ids::{ElementRef, MeasureId, NoteId, PartId};

    #[test]
    fn missing_element_names_the_element() {
        let refusal = ScoreError::MissingElement(ElementRef::Note(NoteId::new(42)));
        assert_eq!(
            refusal.to_string(),
            "the score holds no note 42",
            "the message names the element the command asked for"
        );
    }

    #[test]
    fn missing_part_names_the_identifier() {
        let refusal = ScoreError::MissingPart(PartId::new(7));
        assert_eq!(
            refusal.to_string(),
            "the score holds no part with identifier 7",
            "the message names the part the command asked for"
        );
    }

    #[test]
    fn schema_names_both_numbers() {
        let refusal = ScoreError::Schema {
            found: SchemaVersion::new(9),
            expected: SchemaVersion::new(1),
        };
        assert_eq!(
            refusal.to_string(),
            "the document states schema 9 and this build reads schema 1",
            "the message tells the reader which build reads the document"
        );
    }

    #[test]
    fn time_refusal_keeps_the_kernel_message() {
        let refusal = ScoreError::from(TimeError::Overflow);
        assert_eq!(
            refusal.to_string(),
            TimeError::Overflow.to_string(),
            "a transparent variant repeats the message of the source"
        );
    }

    #[test]
    fn measure_not_empty_names_the_measure() {
        let refusal = ScoreError::MeasureNotEmpty(MeasureId::new(2));
        assert_eq!(
            refusal.to_string(),
            "measure 2 holds a note or a rest",
            "the message names the measure that the remove refused"
        );
    }
}
