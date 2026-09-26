//! The stable identifiers and the short text values of the score.
//!
//! Every entity carries a `u64` identifier that the score mints from one
//! counter and persists. A generational arena index is smaller and it is wrong
//! here, because the identifier must survive a save, a git commit, and a hand
//! edit by a terminal agent. Section 3.2 of `roadmap/duet-v1/architecture.md`
//! states the rule.

use core::fmt;
use core::num::NonZeroU8;

use serde::{Deserialize, Serialize};

use crate::error::ScoreError;

/// A stable identifier for one note.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct NoteId(u64);

impl NoteId {
    /// The identifier that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The identifier as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A stable identifier for one part.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PartId(u64);

impl PartId {
    /// The identifier that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The identifier as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A stable identifier for one staff.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct StaffId(u64);

impl StaffId {
    /// The identifier that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The identifier as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A stable identifier for one voice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VoiceId(u64);

impl VoiceId {
    /// The identifier that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The identifier as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A stable identifier for one measure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MeasureId(u64);

impl MeasureId {
    /// The identifier that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The identifier as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A stable identifier for one spanner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SpannerId(u64);

impl SpannerId {
    /// The identifier that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The identifier as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A stable identifier for one score mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MarkId(u64);

impl MarkId {
    /// The identifier that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The identifier as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// A part name the user reads and edits. It is never empty.
///
/// `#[serde(try_from = "Box<str>")]` routes deserialization through `new`, for
/// the reason section 2.6a of `roadmap/duet-v1/architecture.md` states for
/// `Finite`: a derived `Deserialize` writes the inner field directly, and a
/// hand edit then walks past the one invariant the type carries.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "Box<str>")]
pub struct PartName(Box<str>);

impl PartName {
    /// The part name that `value` spells, or `None` for an empty string.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        (!value.is_empty()).then(|| Self(Box::from(value)))
    }

    /// The name as text.
    #[must_use]
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl TryFrom<Box<str>> for PartName {
    type Error = ScoreError;

    /// The part name that `value` spells.
    ///
    /// # Errors
    /// Returns `ScoreError::Parse` for an empty string.
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::new(&value).ok_or_else(|| ScoreError::Parse(Box::from("a part name is never empty")))
    }
}

/// One syllable of a lyric. It is never empty.
///
/// `#[serde(try_from = "Box<str>")]` holds the invariant on the
/// deserialization path, for the reason `PartName` states.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "Box<str>")]
pub struct LyricText(Box<str>);

impl LyricText {
    /// The syllable that `value` spells, or `None` for an empty string.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        (!value.is_empty()).then(|| Self(Box::from(value)))
    }

    /// The syllable as text.
    #[must_use]
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl TryFrom<Box<str>> for LyricText {
    type Error = ScoreError;

    /// The syllable that `value` spells.
    ///
    /// # Errors
    /// Returns `ScoreError::Parse` for an empty string.
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::new(&value)
            .ok_or_else(|| ScoreError::Parse(Box::from("a lyric syllable is never empty")))
    }
}

/// The printed text of a rehearsal mark. It is never empty.
///
/// `#[serde(try_from = "Box<str>")]` holds the invariant on the
/// deserialization path, for the reason `PartName` states.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "Box<str>")]
pub struct RehearsalText(Box<str>);

impl RehearsalText {
    /// The rehearsal text that `value` spells, or `None` for an empty string.
    #[must_use]
    pub fn new(value: &str) -> Option<Self> {
        (!value.is_empty()).then(|| Self(Box::from(value)))
    }

    /// The rehearsal text as text.
    #[must_use]
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl TryFrom<Box<str>> for RehearsalText {
    type Error = ScoreError;

    /// The rehearsal text that `value` spells.
    ///
    /// # Errors
    /// Returns `ScoreError::Parse` for an empty string.
    fn try_from(value: Box<str>) -> Result<Self, Self::Error> {
        Self::new(&value)
            .ok_or_else(|| ScoreError::Parse(Box::from("a rehearsal mark text is never empty")))
    }
}

/// A verse number, one based.
///
/// `Lyric` sorts by verse, so the engraver stacks verse one above verse two
/// (section 10.5 of `roadmap/duet-v1/architecture.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct VerseNumber(NonZeroU8);

impl VerseNumber {
    /// The verse that `value` numbers.
    #[must_use]
    pub const fn new(value: NonZeroU8) -> Self {
        Self(value)
    }

    /// The verse number as a non-zero integer.
    #[must_use]
    pub const fn get(self) -> NonZeroU8 {
        self.0
    }
}

/// The revision counter of one score. It increases on every applied command.
///
/// It derives `Eq`, which VR1 makes the compiler's demand over a `u64`. It
/// derives no order and no `Hash`. The engrave task compares the revision of
/// its own result with the current one and drops a stale one (section 10.5 of
/// `roadmap/duet-v1/architecture.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Revision(u64);

impl Revision {
    /// The first revision of a score that no command has changed.
    pub const ZERO: Self = Self(0);

    /// The revision that the number `value` names.
    #[must_use]
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    /// The revision as a plain integer.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }

    /// The revision that follows this one.
    ///
    /// The counter holds at `u64::MAX`. A score that applies that many
    /// commands is outside every bound this plan states.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

/// A reference to one element of the score, whatever its kind.
///
/// It derives `Eq`, because every arm payload supplies it. It derives no
/// `Hash` and no order, because VR1 names no map, no set, and no sort over an
/// element reference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementRef {
    /// The element is a note or a rest.
    Note(NoteId),
    /// The element is a spanner.
    Spanner(SpannerId),
    /// The element is a score mark.
    Mark(MarkId),
}

/// The kind and the number of the element, for example `note 42`.
///
/// `ScoreError::MissingElement` carries an `ElementRef` and its message reads
/// this text, so a refusal names the element that the command asked for.
impl fmt::Display for ElementRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::Note(note) => write!(f, "note {}", note.get()),
            Self::Spanner(spanner) => write!(f, "spanner {}", spanner.get()),
            Self::Mark(mark) => write!(f, "mark {}", mark.get()),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroU8;

    use super::{
        ElementRef, LyricText, MarkId, NoteId, PartName, RehearsalText, Revision, SpannerId,
        VerseNumber,
    };

    #[test]
    fn a_text_value_refuses_an_empty_string() {
        assert!(PartName::new("").is_none(), "a part name is never empty");
        assert!(
            LyricText::new("").is_none(),
            "a lyric syllable is never empty"
        );
        assert!(
            RehearsalText::new("").is_none(),
            "a rehearsal text is never empty"
        );
    }

    #[test]
    fn a_text_value_keeps_the_text_it_took() {
        let name = PartName::new("Soprano 1").expect("a name with text");
        assert_eq!(
            name.get(),
            "Soprano 1",
            "the name answers the text the caller gave"
        );
    }

    #[test]
    fn deserialization_refuses_an_empty_string() {
        let refusal = serde_json::from_str::<PartName>("\"\"");
        assert!(
            refusal.is_err(),
            "a hand edit cannot walk past the empty-string rule"
        );
    }

    #[test]
    fn deserialization_takes_a_name_with_text() {
        let name = serde_json::from_str::<PartName>("\"Alto 2\"").expect("a name with text");
        assert_eq!(name.get(), "Alto 2", "the reader answers the stored text");
    }

    #[test]
    fn a_revision_increases_by_one() {
        assert_eq!(
            Revision::ZERO.next(),
            Revision::new(1),
            "the counter steps by one"
        );
    }

    #[test]
    fn a_revision_holds_at_the_bound() {
        let last = Revision::new(u64::MAX);
        assert_eq!(last.next(), last, "the counter holds at the 64-bit bound");
    }

    #[test]
    fn an_element_reference_prints_its_kind_and_its_number() {
        assert_eq!(
            ElementRef::Note(NoteId::new(42)).to_string(),
            "note 42",
            "a note reference names the kind and the number"
        );
        assert_eq!(
            ElementRef::Spanner(SpannerId::new(7)).to_string(),
            "spanner 7",
            "a spanner reference names the kind and the number"
        );
        assert_eq!(
            ElementRef::Mark(MarkId::new(3)).to_string(),
            "mark 3",
            "a mark reference names the kind and the number"
        );
    }

    #[test]
    fn a_verse_number_answers_its_own_value() {
        let verse = VerseNumber::new(NonZeroU8::MIN);
        assert_eq!(
            verse.get(),
            NonZeroU8::MIN,
            "the verse answers the number it took"
        );
    }
}
