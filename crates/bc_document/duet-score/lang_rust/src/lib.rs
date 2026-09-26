//! The Duet score aggregate.
//!
//! It holds the score root, its entities and value objects, the command and
//! event vocabulary, and the canonical document. It names no file format
//! parser. Section 3 of `roadmap/duet-v1/architecture.md` states the design.

#![forbid(unsafe_code)]

mod apply;
mod canonical;
mod command;
mod error;
mod event;
mod ids;
mod model;

pub use apply::Applied;
pub use canonical::{CanonicalDocument, read, write};
pub use command::{Clipboard, PitchEdit, ScoreCommand, ScoreSelector, Selection};
/// The time-kernel types that a consumer of this crate reads.
///
/// A signature of this crate names each of them, or an exported type answers
/// one: `TempoMap::tempos` is the one path to a `Tempo`, and no signature of
/// `duet-score` names that type. A consumer therefore reads every one of them
/// through this crate and needs no direct dependency on `duet-time`.
pub use duet_time::{Meter, NoteValue, SchemaVersion, Tempo, TempoMap, Ticks, TimeError, Tuplet};
pub use error::ScoreError;
pub use event::ScoreEvent;
pub use ids::{
    ElementRef, LyricText, MarkId, MeasureId, NoteId, PartId, PartName, RehearsalText, Revision,
    SpannerId, StaffId, VerseNumber, VoiceId,
};
pub use model::{
    Accidental, Articulation, Clef, Duration, Dynamic, InverseCost, KeySignature, Lyric, MarkKind,
    Measure, Note, OctaveShift, Part, Pitch, RepeatSide, Rest, SCHEMA, Score, ScoreMark, Spanner,
    SpannerKind, Staff, Step, TieState, Voice, VoiceType,
};
