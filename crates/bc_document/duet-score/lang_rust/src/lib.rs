//! The Duet score aggregate.
//!
//! It holds the score root, its entities and value objects, the command and
//! event vocabulary, and the canonical document. It names no file format
//! parser. Section 3 of `roadmap/duet-v1/architecture.md` states the design.

#![forbid(unsafe_code)]

mod error;
mod ids;
mod model;

pub use duet_time::{Meter, NoteValue, Tempo};
pub use error::ScoreError;
pub use ids::{
    ElementRef, LyricText, MarkId, MeasureId, NoteId, PartId, PartName, RehearsalText, Revision,
    SpannerId, StaffId, VerseNumber, VoiceId,
};
pub use model::{
    Accidental, Articulation, Clef, Duration, Dynamic, InverseCost, KeySignature, Lyric, MarkKind,
    Measure, Note, OctaveShift, Part, Pitch, RepeatSide, Rest, SCHEMA, Score, ScoreMark, Spanner,
    SpannerKind, Staff, Step, TieState, Voice, VoiceType,
};
