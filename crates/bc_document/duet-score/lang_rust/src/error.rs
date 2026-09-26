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
    ///
    /// It is the one arm that carries `#[from]`, which section 15.3 of
    /// `roadmap/duet-v1/architecture.md` does not print: `apply` and
    /// `canonical` call the kernel behind `?`, and the conversion keeps the
    /// call sites free of a hand-written `map_err` that could drop the source.
    /// `#[from]` adds a public `From<TimeError> for ScoreError`, so the code and
    /// the architecture declare two different public surfaces until one of them
    /// changes. `escalation:T2-15` holds that choice.
    #[error(transparent)]
    Time(#[from] TimeError),
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use duet_time::{SchemaVersion, Ticks, TimeError};

    use super::ScoreError;
    use crate::ids::{ElementRef, MarkId, MeasureId, NoteId, PartId, SpannerId, StaffId, VoiceId};

    /// How many arms `ScoreError` declares.
    const ARM_COUNT: usize = 11;

    /// One refusal and the values that its message must print.
    struct ArmCase {
        /// The refusal under test.
        refusal: ScoreError,
        /// The text that the message must hold.
        needles: Vec<String>,
    }

    /// The name of the arm that `refusal` took.
    ///
    /// The match names every arm, so a new arm of `ScoreError` stops this
    /// module from building until the fixture below covers it too.
    fn arm_name(refusal: &ScoreError) -> &'static str {
        match *refusal {
            ScoreError::MissingElement(_) => "MissingElement",
            ScoreError::MissingPart(_) => "MissingPart",
            ScoreError::MissingStaff(_) => "MissingStaff",
            ScoreError::MissingMeasure(_) => "MissingMeasure",
            ScoreError::OverlappingNote { .. } => "OverlappingNote",
            ScoreError::TieAcrossPitch { .. } => "TieAcrossPitch",
            ScoreError::MeasureNotEmpty(_) => "MeasureNotEmpty",
            ScoreError::Schema { .. } => "Schema",
            ScoreError::Parse(_) => "Parse",
            ScoreError::Serialize(_) => "Serialize",
            ScoreError::Time(_) => "Time",
        }
    }

    /// One case for each arm of `ScoreError`, with the text it must print.
    fn arm_cases() -> Vec<ArmCase> {
        vec![
            ArmCase {
                refusal: ScoreError::MissingElement(ElementRef::Spanner(SpannerId::new(11))),
                needles: vec!["spanner".to_owned(), "11".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::MissingElement(ElementRef::Mark(MarkId::new(12))),
                needles: vec!["mark".to_owned(), "12".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::MissingPart(PartId::new(13)),
                needles: vec!["13".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::MissingStaff(StaffId::new(14)),
                needles: vec!["14".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::MissingMeasure(MeasureId::new(15)),
                needles: vec!["15".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::OverlappingNote {
                    staff: StaffId::new(16),
                    voice: VoiceId::new(17),
                    onset: Ticks::new(18),
                },
                needles: vec!["16".to_owned(), "17".to_owned(), "18".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::TieAcrossPitch {
                    from: NoteId::new(19),
                    to: NoteId::new(20),
                },
                needles: vec!["19".to_owned(), "20".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::MeasureNotEmpty(MeasureId::new(21)),
                needles: vec!["21".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::Schema {
                    found: SchemaVersion::new(22),
                    expected: SchemaVersion::new(23),
                },
                needles: vec!["22".to_owned(), "23".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::Parse(Box::from("line 24 holds no object")),
                needles: vec!["line 24 holds no object".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::Serialize(Box::from("a float of no finite value")),
                needles: vec!["a float of no finite value".to_owned()],
            },
            ArmCase {
                refusal: ScoreError::from(TimeError::Overflow),
                needles: vec![TimeError::Overflow.to_string()],
            },
        ]
    }

    #[test]
    fn every_refusal_prints_its_own_values() {
        let cases = arm_cases();
        let covered: BTreeSet<&str> = cases.iter().map(|case| arm_name(&case.refusal)).collect();
        assert_eq!(
            covered.len(),
            ARM_COUNT,
            "the fixture covers every arm of ScoreError"
        );
        for case in &cases {
            let arm = arm_name(&case.refusal);
            let message = case.refusal.to_string();
            assert!(!message.is_empty(), "the message of {arm} holds text");
            for needle in &case.needles {
                assert!(
                    message.contains(needle.as_str()),
                    "the message of {arm} names {needle}, and it reads {message}"
                );
            }
        }
    }

    #[test]
    fn every_refusal_survives_the_transport() {
        for case in arm_cases() {
            let text = serde_json::to_string(&case.refusal).expect("a refusal serializes");
            let read: ScoreError = serde_json::from_str(&text).expect("a refusal reads back");
            assert_eq!(
                read,
                case.refusal,
                "{} survives the transport, and it reads {text}",
                arm_name(&case.refusal)
            );
        }
    }

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
