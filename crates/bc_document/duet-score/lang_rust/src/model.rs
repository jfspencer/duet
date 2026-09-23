//! The entities, the value objects, and the aggregate root of the score.
//!
//! `Score` is one aggregate root and no smaller consistency boundary exists.
//! A tie crosses a measure, a slur crosses a staff, and a lyric verse runs the
//! length of a part, so a command applies to the root. Sections 3.1 and 3.3 of
//! `roadmap/duet-v1/architecture.md` state the design.

use core::num::{NonZeroU8, NonZeroU16};
use std::collections::BTreeMap;

use duet_time::{
    Bbt, Meter, MeterPoint, NoteValue, SchemaVersion, TempoMap, TempoMapEdit, Ticks, Tuplet,
};
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::ids::{
    LyricText, MarkId, MeasureId, NoteId, PartId, PartName, RehearsalText, Revision, SpannerId,
    StaffId, VerseNumber, VoiceId,
};

/// The current schema number of the canonical document (B42).
pub const SCHEMA: SchemaVersion = SchemaVersion::new(1);

/// The non-zero byte that `value` spells, or one where `value` is zero.
///
/// `NonZeroU8::new` answers an `Option` and `expect` is denied, so the match
/// here reads the option in a `const` context. It checks nothing: a zero takes
/// the second arm and the caller reads one. Each caller passes a literal, and
/// the test of that literal is the guard.
const fn non_zero_u8(value: u8) -> NonZeroU8 {
    match NonZeroU8::new(value) {
        Some(checked) => checked,
        None => NonZeroU8::MIN,
    }
}

/// The meter of the one measure that an empty score holds.
const FOUR_FOUR: Meter = Meter::new(non_zero_u8(4), NoteValue::Quarter);

/// The key of the one measure that an empty score holds.
const C_MAJOR: KeySignature = KeySignature::new(0, true);

/// The four vocal parts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VoiceType {
    /// The highest of the four parts.
    Soprano,
    /// The second part from the top.
    Alto,
    /// The third part from the top.
    Tenor,
    /// The lowest of the four parts.
    Bass,
}

/// One named part. A score holds many parts of each voice type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Part {
    /// The identifier that the score minted for this part.
    id: PartId,
    /// The vocal range of this part.
    voice_type: VoiceType,
    /// 1 for "Soprano 1", 2 for "Soprano 2".
    ordinal: NonZeroU16,
    /// The name that the user reads and edits.
    name: PartName,
    /// The staves of this part, from the top down.
    staves: Vec<StaffId>,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Part {
    /// A part with no staff and no unknown field.
    #[must_use]
    pub const fn new(
        id: PartId,
        voice_type: VoiceType,
        ordinal: NonZeroU16,
        name: PartName,
    ) -> Self {
        Self {
            id,
            voice_type,
            ordinal,
            name,
            staves: Vec::new(),
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this part.
    #[must_use]
    pub const fn id(&self) -> PartId {
        self.id
    }

    /// The vocal range of this part.
    #[must_use]
    pub const fn voice_type(&self) -> VoiceType {
        self.voice_type
    }

    /// The place of this part inside its voice type.
    #[must_use]
    pub const fn ordinal(&self) -> NonZeroU16 {
        self.ordinal
    }

    /// The name that the user reads and edits.
    #[must_use]
    pub const fn name(&self) -> &PartName {
        &self.name
    }

    /// The staves of this part, from the top down.
    #[must_use]
    pub fn staves(&self) -> &[StaffId] {
        &self.staves
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// Add `staff` at the foot of the staves of this part.
    pub(crate) fn push_staff(&mut self, staff: StaffId) {
        self.staves.push(staff);
    }

    /// Take `staff` out of the staves of this part.
    pub(crate) fn drop_staff(&mut self, staff: StaffId) {
        self.staves.retain(|held| *held != staff);
    }
}

/// One staff with a clef and a written transposition.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Staff {
    /// The identifier that the score minted for this staff.
    id: StaffId,
    /// The clef that this staff opens with.
    clef: Clef,
    /// How far the written pitch sits above the sounding pitch.
    transpose_semitones: i8,
    /// The voices of this staff, from the top down.
    voices: Vec<VoiceId>,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Staff {
    /// A staff with no voice and no unknown field.
    #[must_use]
    pub const fn new(id: StaffId, clef: Clef, transpose_semitones: i8) -> Self {
        Self {
            id,
            clef,
            transpose_semitones,
            voices: Vec::new(),
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this staff.
    #[must_use]
    pub const fn id(&self) -> StaffId {
        self.id
    }

    /// The clef that this staff opens with.
    #[must_use]
    pub const fn clef(&self) -> Clef {
        self.clef
    }

    /// How far the written pitch sits above the sounding pitch.
    #[must_use]
    pub const fn transpose_semitones(&self) -> i8 {
        self.transpose_semitones
    }

    /// The voices of this staff, from the top down.
    #[must_use]
    pub fn voices(&self) -> &[VoiceId] {
        &self.voices
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// Add `voice` at the foot of the voices of this staff.
    pub(crate) fn push_voice(&mut self, voice: VoiceId) {
        self.voices.push(voice);
    }

    /// Take the clef that this staff opens with.
    pub(crate) const fn set_clef(&mut self, clef: Clef) {
        self.clef = clef;
    }
}

/// One voice inside one staff.
///
/// It carried `Copy` until revision 10, and property 4 of section 3.6 gives
/// every `score/meta.json` record the `extra` bag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Voice {
    /// The identifier that the score minted for this voice.
    id: VoiceId,
    /// The place of this voice inside its staff.
    ordinal: NonZeroU8,
    /// Whether the stems of this voice point up.
    stem_up: bool,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Voice {
    /// A voice with no unknown field.
    #[must_use]
    pub const fn new(id: VoiceId, ordinal: NonZeroU8, stem_up: bool) -> Self {
        Self {
            id,
            ordinal,
            stem_up,
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this voice.
    #[must_use]
    pub const fn id(&self) -> VoiceId {
        self.id
    }

    /// The place of this voice inside its staff.
    #[must_use]
    pub const fn ordinal(&self) -> NonZeroU8 {
        self.ordinal
    }

    /// Whether the stems of this voice point up.
    #[must_use]
    pub const fn stem_up(&self) -> bool {
        self.stem_up
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }
}

/// A bar on the shared timeline. Every staff shares the same measures.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Measure {
    /// The identifier that the score minted for this measure.
    id: MeasureId,
    /// The onset of this measure in ticks from score zero.
    start: Ticks,
    /// The time signature that this measure carries.
    meter: Meter,
    /// The key signature that this measure carries.
    key: KeySignature,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Measure {
    /// A measure with no unknown field.
    #[must_use]
    pub const fn new(id: MeasureId, start: Ticks, meter: Meter, key: KeySignature) -> Self {
        Self {
            id,
            start,
            meter,
            key,
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this measure.
    #[must_use]
    pub const fn id(&self) -> MeasureId {
        self.id
    }

    /// The onset of this measure in ticks from score zero.
    #[must_use]
    pub const fn start(&self) -> Ticks {
        self.start
    }

    /// The time signature that this measure carries.
    #[must_use]
    pub const fn meter(&self) -> Meter {
        self.meter
    }

    /// The key signature that this measure carries.
    #[must_use]
    pub const fn key(&self) -> KeySignature {
        self.key
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// Take a new onset in ticks from score zero.
    pub(crate) const fn set_start(&mut self, start: Ticks) {
        self.start = start;
    }

    /// Take a new time signature.
    pub(crate) const fn set_meter(&mut self, meter: Meter) {
        self.meter = meter;
    }

    /// Take a new key signature.
    pub(crate) const fn set_key(&mut self, key: KeySignature) {
        self.key = key;
    }
}

/// A diatonic step.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Step {
    /// The step C.
    C,
    /// The step D.
    D,
    /// The step E.
    E,
    /// The step F.
    F,
    /// The step G.
    G,
    /// The step A.
    A,
    /// The step B.
    B,
}

/// How a respell chooses an accidental.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Accidental {
    /// No alteration.
    Natural,
    /// One semitone up.
    Sharp,
    /// One semitone down.
    Flat,
    /// Two semitones up.
    DoubleSharp,
    /// Two semitones down.
    DoubleFlat,
}

/// How far an octave spanner shifts, in octaves and in direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum OctaveShift {
    /// One octave up.
    Up8,
    /// One octave down.
    Down8,
    /// Two octaves up.
    Up15,
    /// Two octaves down.
    Down15,
}

/// A key signature. `fifths` is negative for flats and positive for sharps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeySignature {
    /// The steps around the circle of fifths from C.
    fifths: i8,
    /// Whether the key is major.
    major: bool,
}

impl KeySignature {
    /// The key that `fifths` and `major` name.
    #[must_use]
    pub const fn new(fifths: i8, major: bool) -> Self {
        Self { fifths, major }
    }

    /// The steps around the circle of fifths from C.
    #[must_use]
    pub const fn fifths(self) -> i8 {
        self.fifths
    }

    /// Whether the key is major.
    #[must_use]
    pub const fn major(self) -> bool {
        self.major
    }
}

/// A written pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Pitch {
    /// The octave, where 4 holds middle C.
    octave: i8,
    /// The diatonic step inside the octave.
    step: Step,
    /// -2 is a double flat, 2 is a double sharp.
    alter: i8,
}

impl Pitch {
    /// The pitch that `octave`, `step`, and `alter` name.
    #[must_use]
    pub const fn new(octave: i8, step: Step, alter: i8) -> Self {
        Self {
            octave,
            step,
            alter,
        }
    }

    /// The octave, where 4 holds middle C.
    #[must_use]
    pub const fn octave(self) -> i8 {
        self.octave
    }

    /// The diatonic step inside the octave.
    #[must_use]
    pub const fn step(self) -> Step {
        self.step
    }

    /// How many semitones the accidental moves the step.
    #[must_use]
    pub const fn alter(self) -> i8 {
        self.alter
    }
}

/// A written duration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Duration {
    /// The note value that the head and the flags spell.
    value: NoteValue,
    /// How many dots follow the head.
    dots: u8,
    /// The tuplet that holds this duration, when one does.
    tuplet: Option<Tuplet>,
}

impl Duration {
    /// The duration that `value`, `dots`, and `tuplet` spell.
    #[must_use]
    pub const fn new(value: NoteValue, dots: u8, tuplet: Option<Tuplet>) -> Self {
        Self {
            value,
            dots,
            tuplet,
        }
    }

    /// The note value that the head and the flags spell.
    #[must_use]
    pub const fn value(self) -> NoteValue {
        self.value
    }

    /// How many dots follow the head.
    #[must_use]
    pub const fn dots(self) -> u8 {
        self.dots
    }

    /// The tuplet that holds this duration, when one does.
    #[must_use]
    pub const fn tuplet(self) -> Option<Tuplet> {
        self.tuplet
    }

    /// The tick count of this duration.
    ///
    /// The base is `NoteValue::ticks`. Each dot adds one half of the term
    /// before it, and a shift right by one takes that half. A term that
    /// reaches zero ends the sum, so a dot beyond that point adds nothing and
    /// the answer of 13 dots is the answer of 255 dots. A duration that holds
    /// no tuplet is exact.
    ///
    /// A tuplet writes `count` notes in the time of `over` notes. The dotted
    /// value first spans `over` written notes, and the answer is the first part
    /// of the `duet_time::split_tuplet` division of that span into `count`
    /// parts. That rule gives the remainder to the last part, and the tick
    /// resolution B40 has no factor of seven, of eleven, or of thirteen, so a
    /// group of one of those counts does not always divide evenly: `count`
    /// copies of this answer then sum to less than the span of the group. A
    /// caller that needs the whole span calls `split_tuplet` itself and adds
    /// the parts. This function reads one note, and only `Score` knows which
    /// member of the group that note is.
    ///
    /// A `count` above the tick span of the group answers zero ticks, because
    /// the division then leaves no whole tick for a part. 255 notes in the time
    /// of one thirty-second note is such a group.
    ///
    /// Every step saturates, so no input wraps, and no input reaches a bound
    /// either: the base is one whole note at most, a dot run stays below twice
    /// the base, and `over` holds one byte, so the widest group spans fewer
    /// than four million ticks.
    #[must_use]
    pub fn ticks(self) -> Ticks {
        let base = self.value.ticks().get();
        let mut term = base;
        let mut total = base;
        for _ in 0..self.dots {
            term >>= 1;
            if term == 0 {
                break;
            }
            total = total.saturating_add(term);
        }
        let Some(tuplet) = self.tuplet else {
            return Ticks::new(total);
        };
        let group = total.saturating_mul(i64::from(tuplet.over().get()));
        Ticks::new(group.div_euclid(i64::from(tuplet.count().get())))
    }
}

/// Whether a note starts or ends a tie.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct TieState {
    /// Whether a tie starts at this note.
    starts: bool,
    /// Whether a tie stops at this note.
    stops: bool,
}

impl TieState {
    /// The tie state that `starts` and `stops` name.
    #[must_use]
    pub const fn new(starts: bool, stops: bool) -> Self {
        Self { starts, stops }
    }

    /// Whether a tie starts at this note.
    #[must_use]
    pub const fn starts(self) -> bool {
        self.starts
    }

    /// Whether a tie stops at this note.
    #[must_use]
    pub const fn stops(self) -> bool {
        self.stops
    }
}

/// One note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Note {
    /// The identifier that the score minted for this note.
    id: NoteId,
    /// The staff that holds this note.
    staff: StaffId,
    /// The voice that holds this note.
    voice: VoiceId,
    /// The onset in ticks from score zero, not from a measure start.
    onset: Ticks,
    /// The written duration of this note.
    duration: Duration,
    /// The written pitch of this note.
    pitch: Pitch,
    /// Whether a tie starts or stops at this note.
    tie: TieState,
    /// The articulation marks on this note.
    articulations: SmallVec<[Articulation; 2]>,
    /// The lyric syllables on this note, one for each verse.
    lyrics: SmallVec<[Lyric; 2]>,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Note {
    /// A note with no tie, no articulation, no lyric, and no unknown field.
    #[must_use]
    pub fn new(
        id: NoteId,
        staff: StaffId,
        voice: VoiceId,
        onset: Ticks,
        duration: Duration,
        pitch: Pitch,
    ) -> Self {
        Self {
            id,
            staff,
            voice,
            onset,
            duration,
            pitch,
            tie: TieState::new(false, false),
            articulations: SmallVec::new(),
            lyrics: SmallVec::new(),
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this note.
    #[must_use]
    pub const fn id(&self) -> NoteId {
        self.id
    }

    /// The staff that holds this note.
    #[must_use]
    pub const fn staff(&self) -> StaffId {
        self.staff
    }

    /// The voice that holds this note.
    #[must_use]
    pub const fn voice(&self) -> VoiceId {
        self.voice
    }

    /// The onset in ticks from score zero, not from a measure start.
    #[must_use]
    pub const fn onset(&self) -> Ticks {
        self.onset
    }

    /// The written duration of this note.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// The written pitch of this note.
    #[must_use]
    pub const fn pitch(&self) -> Pitch {
        self.pitch
    }

    /// Whether a tie starts or stops at this note.
    #[must_use]
    pub const fn tie(&self) -> TieState {
        self.tie
    }

    /// The articulation marks on this note.
    #[must_use]
    pub fn articulations(&self) -> &[Articulation] {
        &self.articulations
    }

    /// The lyric syllables on this note, one for each verse.
    #[must_use]
    pub fn lyrics(&self) -> &[Lyric] {
        &self.lyrics
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// This note under the identifier `id`.
    ///
    /// `Duplicate` mints a copy, and `Paste` mints one where the score already
    /// holds the identifier that the clipboard carries.
    #[must_use]
    pub(crate) const fn with_id(mut self, id: NoteId) -> Self {
        self.id = id;
        self
    }

    /// Take a new staff and a new voice.
    pub(crate) const fn set_place(&mut self, staff: StaffId, voice: VoiceId) {
        self.staff = staff;
        self.voice = voice;
    }

    /// Take a new onset in ticks from score zero.
    pub(crate) const fn set_onset(&mut self, onset: Ticks) {
        self.onset = onset;
    }

    /// Take a new written duration.
    pub(crate) const fn set_duration(&mut self, duration: Duration) {
        self.duration = duration;
    }

    /// Take a new written pitch.
    pub(crate) const fn set_pitch(&mut self, pitch: Pitch) {
        self.pitch = pitch;
    }

    /// Take a new tie state.
    pub(crate) const fn set_tie(&mut self, tie: TieState) {
        self.tie = tie;
    }

    /// Take a new list of articulation marks.
    pub(crate) fn set_articulations(&mut self, articulations: SmallVec<[Articulation; 2]>) {
        self.articulations = articulations;
    }

    /// Put `text` in `verse`, over the syllable that the verse already holds.
    ///
    /// The list stays ordered by verse, because the engraver stacks verse one
    /// above verse two.
    pub(crate) fn set_lyric(&mut self, verse: VerseNumber, text: LyricText) {
        let syllable = Lyric::new(verse, text, false);
        if let Some(held) = self.lyrics.iter_mut().find(|held| held.verse() == verse) {
            *held = syllable;
            return;
        }
        self.lyrics.push(syllable);
        self.lyrics.sort_by_key(Lyric::verse);
    }
}

/// One rest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Rest {
    /// The identifier that the score minted for this rest.
    id: NoteId,
    /// The staff that holds this rest.
    staff: StaffId,
    /// The voice that holds this rest.
    voice: VoiceId,
    /// The onset in ticks from score zero, not from a measure start.
    onset: Ticks,
    /// The written duration of this rest.
    duration: Duration,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Rest {
    /// A rest with no unknown field.
    #[must_use]
    pub const fn new(
        id: NoteId,
        staff: StaffId,
        voice: VoiceId,
        onset: Ticks,
        duration: Duration,
    ) -> Self {
        Self {
            id,
            staff,
            voice,
            onset,
            duration,
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this rest.
    #[must_use]
    pub const fn id(&self) -> NoteId {
        self.id
    }

    /// The staff that holds this rest.
    #[must_use]
    pub const fn staff(&self) -> StaffId {
        self.staff
    }

    /// The voice that holds this rest.
    #[must_use]
    pub const fn voice(&self) -> VoiceId {
        self.voice
    }

    /// The onset in ticks from score zero, not from a measure start.
    #[must_use]
    pub const fn onset(&self) -> Ticks {
        self.onset
    }

    /// The written duration of this rest.
    #[must_use]
    pub const fn duration(&self) -> Duration {
        self.duration
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// This rest under the identifier `id`.
    #[must_use]
    pub(crate) const fn with_id(mut self, id: NoteId) -> Self {
        self.id = id;
        self
    }

    /// Take a new staff and a new voice.
    pub(crate) const fn set_place(&mut self, staff: StaffId, voice: VoiceId) {
        self.staff = staff;
        self.voice = voice;
    }

    /// Take a new onset in ticks from score zero.
    pub(crate) const fn set_onset(&mut self, onset: Ticks) {
        self.onset = onset;
    }
}

/// A mark that spans more than one note.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Spanner {
    /// The identifier that the score minted for this spanner.
    id: SpannerId,
    /// What this spanner draws.
    kind: SpannerKind,
    /// The note that this spanner starts at.
    from: NoteId,
    /// The note that this spanner stops at.
    to: NoteId,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl Spanner {
    /// A spanner with no unknown field.
    #[must_use]
    pub const fn new(id: SpannerId, kind: SpannerKind, from: NoteId, to: NoteId) -> Self {
        Self {
            id,
            kind,
            from,
            to,
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this spanner.
    #[must_use]
    pub const fn id(&self) -> SpannerId {
        self.id
    }

    /// What this spanner draws.
    #[must_use]
    pub const fn kind(&self) -> SpannerKind {
        self.kind
    }

    /// The note that this spanner starts at.
    #[must_use]
    pub const fn from(&self) -> NoteId {
        self.from
    }

    /// The note that this spanner stops at.
    #[must_use]
    pub const fn to(&self) -> NoteId {
        self.to
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// This spanner under the identifier `id`, between `from` and `to`.
    ///
    /// A duplicate and a paste both remap the two endpoints, so the three
    /// values change together.
    #[must_use]
    pub(crate) const fn with_ids(mut self, id: SpannerId, from: NoteId, to: NoteId) -> Self {
        self.id = id;
        self.from = from;
        self.to = to;
        self
    }
}

/// The spanner kinds that v1 draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum SpannerKind {
    /// A slur over the notes it holds.
    Slur,
    /// A hairpin that grows.
    Crescendo,
    /// A hairpin that falls.
    Diminuendo,
    /// An octave line over the notes it holds.
    Octave(OctaveShift),
}

/// A mark on the score that is not a note and not a spanner.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScoreMark {
    /// The identifier that the score minted for this mark.
    id: MarkId,
    /// The onset in ticks from score zero.
    at: Ticks,
    /// What this mark means.
    kind: MarkKind,
    /// Fields from a newer schema that this build does not model.
    #[serde(flatten)]
    extra: BTreeMap<String, serde_json::Value>,
}

impl ScoreMark {
    /// A score mark with no unknown field.
    #[must_use]
    pub const fn new(id: MarkId, at: Ticks, kind: MarkKind) -> Self {
        Self {
            id,
            at,
            kind,
            extra: BTreeMap::new(),
        }
    }

    /// The identifier of this mark.
    #[must_use]
    pub const fn id(&self) -> MarkId {
        self.id
    }

    /// The onset in ticks from score zero.
    #[must_use]
    pub const fn at(&self) -> Ticks {
        self.at
    }

    /// What this mark means.
    #[must_use]
    pub const fn kind(&self) -> &MarkKind {
        &self.kind
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// This mark under the identifier `id`.
    #[must_use]
    pub(crate) const fn with_id(mut self, id: MarkId) -> Self {
        self.id = id;
        self
    }

    /// Take a new onset in ticks from score zero.
    pub(crate) const fn set_at(&mut self, at: Ticks) {
        self.at = at;
    }
}

/// What a score mark means.
#[expect(
    variant_size_differences,
    reason = "`Repeat` carries one byte and `Rehearsal` carries one pointer, so the ratio test \
              refuses every shape that stores text, a boxed arm included"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarkKind {
    /// Force a new system at this point.
    SystemBreak,
    /// A repeat barline. `Start` and `End` pair by position order.
    Repeat(RepeatSide),
    /// A rehearsal mark with its printed text.
    Rehearsal(RehearsalText),
}

/// Which side of a repeat a barline carries.
///
/// It derives `Eq`, because every field supplies it and VR1 makes that derive
/// the compiler's demand. It derives no `Hash` and no order, because VR1 names
/// no map, no set, and no sort over a repeat side.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepeatSide {
    /// The barline opens a repeat.
    Start,
    /// The barline closes a repeat.
    End,
}

/// A clef, as the staff line it centres on and its octave transposition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Clef {
    /// The G clef on the second line.
    Treble,
    /// The F clef on the fourth line.
    Bass,
    /// The C clef on the third line.
    Alto,
    /// The C clef on the fourth line.
    Tenor,
    /// The G clef on the second line, sounding one octave down.
    TrebleOctaveDown,
    /// The percussion clef, which carries no pitch.
    Percussion,
}

/// A printed dynamic mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Dynamic {
    /// As soft as the score asks for.
    Pppp,
    /// Softer than `Pp`.
    Ppp,
    /// Softer than `P`.
    Pp,
    /// Soft.
    P,
    /// Between `P` and `Mf`.
    Mp,
    /// Between `Mp` and `F`.
    Mf,
    /// Loud.
    F,
    /// Louder than `F`.
    Ff,
    /// Louder than `Ff`.
    Fff,
    /// As loud as the score asks for.
    Ffff,
    /// One accented note.
    Sf,
    /// One strongly accented note.
    Sfz,
    /// A loud attack that falls to soft at once.
    Fp,
}

/// A printed articulation mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Articulation {
    /// Short and detached.
    Staccato,
    /// Shorter than `Staccato`.
    Staccatissimo,
    /// Held for its full value.
    Tenuto,
    /// Attacked.
    Accent,
    /// Attacked and shortened.
    Marcato,
    /// Held past its value, at the will of the conductor.
    Fermata,
    /// A breath before the next note.
    Breath,
}

/// One lyric syllable on one note, in one verse.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Lyric {
    /// The verse that this syllable belongs to.
    verse: VerseNumber,
    /// The printed text of this syllable.
    text: LyricText,
    /// Whether a hyphen joins this syllable to the next one.
    hyphenated: bool,
}

impl Lyric {
    /// The syllable that `verse`, `text`, and `hyphenated` name.
    #[must_use]
    pub const fn new(verse: VerseNumber, text: LyricText, hyphenated: bool) -> Self {
        Self {
            verse,
            text,
            hyphenated,
        }
    }

    /// The verse that this syllable belongs to.
    #[must_use]
    pub const fn verse(&self) -> VerseNumber {
        self.verse
    }

    /// The printed text of this syllable.
    #[must_use]
    pub const fn text(&self) -> &LyricText {
        &self.text
    }

    /// Whether a hyphen joins this syllable to the next one.
    #[must_use]
    pub const fn hyphenated(&self) -> bool {
        self.hyphenated
    }
}

/// How much memory one transaction's inverse holds.
///
/// It is declared here, because `Applied` carries one and `duet-core` depends
/// on `duet-score`. A declaration in `duet-core` is the Cargo cycle at the
/// base of the design.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct InverseCost {
    /// How many commands the inverse holds.
    commands: u32,
    /// How many bytes the inverse holds.
    bytes: u32,
}

impl InverseCost {
    /// The cost of an inverse of `commands` commands and `bytes` bytes.
    #[must_use]
    pub const fn new(commands: u32, bytes: u32) -> Self {
        Self { commands, bytes }
    }

    /// How many commands the inverse holds.
    #[must_use]
    pub const fn commands(self) -> u32 {
        self.commands
    }

    /// How many bytes the inverse holds.
    #[must_use]
    pub const fn bytes(self) -> u32 {
        self.bytes
    }
}

/// The score aggregate root. Section 3.1 states the boundary and section 3.4
/// states the one way in.
///
/// **The aggregate owns the tempo map**, because `score/meta.json` stores it
/// and `write` takes the score alone (section 3.6).
#[derive(Debug, Clone)]
pub struct Score {
    /// The parts of the score, by identifier.
    parts: BTreeMap<PartId, Part>,
    /// The staves of the score, by identifier.
    staves: BTreeMap<StaffId, Staff>,
    /// The voices of the score, by identifier.
    voices: BTreeMap<VoiceId, Voice>,
    /// The measures of the shared timeline, by identifier.
    measures: BTreeMap<MeasureId, Measure>,
    /// The notes of the score, by identifier.
    notes: BTreeMap<NoteId, Note>,
    /// The rests of the score, by identifier.
    rests: BTreeMap<NoteId, Rest>,
    /// The spanners of the score, by identifier.
    spanners: BTreeMap<SpannerId, Spanner>,
    /// The marks of the score, by identifier.
    marks: BTreeMap<MarkId, ScoreMark>,
    /// The tempo and meter map of the score.
    tempo_map: TempoMap,
    /// The revision counter, which increases on every applied command.
    revision: Revision,
    /// The schema number that this score was read under.
    schema: SchemaVersion,
    /// Fields from a newer schema that this build does not model.
    extra: BTreeMap<String, serde_json::Value>,
    /// The counter that every identifier of this score comes from.
    next_id: u64,
}

/// Two scores are equal when their content is equal.
///
/// The comparison reads the eight entity maps, the tempo map, the schema, and
/// the bag of unknown fields. It skips `revision` and `next_id`, which count
/// what the score has done and never state what the score holds. An undo
/// restores the content and lowers neither counter, so a derived equality would
/// report every undone score as a different score and `Score::apply` could
/// carry no inverse that a test can check.
impl PartialEq for Score {
    fn eq(&self, other: &Self) -> bool {
        self.parts == other.parts
            && self.staves == other.staves
            && self.voices == other.voices
            && self.measures == other.measures
            && self.notes == other.notes
            && self.rests == other.rests
            && self.spanners == other.spanners
            && self.marks == other.marks
            && self.tempo_map == other.tempo_map
            && self.schema == other.schema
            && self.extra == other.extra
    }
}

/// Content equality is reflexive, symmetric, and transitive, because every
/// field that `PartialEq` reads supplies `Eq`.
impl Eq for Score {}

/// The tempo map of a new score: one meter entry of four four at tick zero.
///
/// Section 3.1 gives the first measure a meter, and `TempoMap::meter_at` is the
/// second statement of the same fact. A new score makes both statements, so no
/// query falls back to an empty map. One entry at tick zero and at
/// `Bbt::ORIGIN` meets the first-point rule of `TempoMapEdit::finish`, and
/// `the_tempo_map_of_a_new_score_states_four_four` proves that the fallback
/// path stays unused.
fn four_four_at_origin() -> TempoMap {
    TempoMapEdit::new()
        .push_meter(MeterPoint::new(Ticks::ZERO, Bbt::ORIGIN, FOUR_FOUR))
        .finish()
        .unwrap_or_default()
}

/// The canonical sort key of one note: the key of section 3.6.
///
/// `score/notes.jsonl` sorts by `(staff, voice, onset, pitch, id)`, and
/// `Clipboard::notes` takes the same order, so one rule serves the writer and
/// the clipboard and the two can never disagree.
pub(crate) const fn note_order_key(note: &Note) -> (StaffId, VoiceId, Ticks, Pitch, NoteId) {
    (
        note.staff(),
        note.voice(),
        note.onset(),
        note.pitch(),
        note.id(),
    )
}

/// The canonical sort key of one rest: the note key without the pitch.
pub(crate) const fn rest_order_key(rest: &Rest) -> (StaffId, VoiceId, Ticks, NoteId) {
    (rest.staff(), rest.voice(), rest.onset(), rest.id())
}

/// The canonical sort key of one spanner: the key of section 3.6.
///
/// `score/spanners.jsonl` sorts by `(from, kind, id)`, and a clipboard takes
/// the same order.
pub(crate) const fn spanner_order_key(spanner: &Spanner) -> (NoteId, SpannerKind, SpannerId) {
    (spanner.from(), spanner.kind(), spanner.id())
}

impl Score {
    /// An empty score with one measure of four-four in C major.
    ///
    /// The measure starts at tick zero, the tempo map states four four at tick
    /// zero, the revision is zero, and the schema is `SCHEMA`.
    #[must_use]
    pub fn new() -> Self {
        let mut score = Self {
            parts: BTreeMap::new(),
            staves: BTreeMap::new(),
            voices: BTreeMap::new(),
            measures: BTreeMap::new(),
            notes: BTreeMap::new(),
            rests: BTreeMap::new(),
            spanners: BTreeMap::new(),
            marks: BTreeMap::new(),
            tempo_map: four_four_at_origin(),
            revision: Revision::ZERO,
            schema: SCHEMA,
            extra: BTreeMap::new(),
            next_id: 0,
        };
        let first = Measure::new(
            MeasureId::new(score.mint_id()),
            Ticks::ZERO,
            FOUR_FOUR,
            C_MAJOR,
        );
        score.measures.insert(first.id(), first);
        score
    }

    /// The next unused identifier of this score.
    ///
    /// The counter holds at `u64::MAX`. A score that mints that many
    /// identifiers is outside every bound this plan states.
    pub(crate) const fn mint_id(&mut self) -> u64 {
        let minted = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        minted
    }

    /// The counter that the next mint reads.
    ///
    /// `score/meta.json` stores it beside the revision, because a reload that
    /// restarts the counter would let a later mint alias a live identifier. The
    /// canonical writer and the transaction test are the two readers, and both
    /// live in this crate.
    #[must_use]
    pub(crate) const fn next_id(&self) -> u64 {
        self.next_id
    }

    /// The parts of the score, by identifier.
    #[must_use]
    pub const fn parts(&self) -> &BTreeMap<PartId, Part> {
        &self.parts
    }

    /// The staves of the score, by identifier.
    #[must_use]
    pub const fn staves(&self) -> &BTreeMap<StaffId, Staff> {
        &self.staves
    }

    /// The voices of the score, by identifier.
    #[must_use]
    pub const fn voices(&self) -> &BTreeMap<VoiceId, Voice> {
        &self.voices
    }

    /// The measures of the shared timeline, by identifier.
    #[must_use]
    pub const fn measures(&self) -> &BTreeMap<MeasureId, Measure> {
        &self.measures
    }

    /// The notes of the score, by identifier.
    #[must_use]
    pub const fn notes(&self) -> &BTreeMap<NoteId, Note> {
        &self.notes
    }

    /// The rests of the score, by identifier.
    #[must_use]
    pub const fn rests(&self) -> &BTreeMap<NoteId, Rest> {
        &self.rests
    }

    /// The spanners of the score, by identifier.
    #[must_use]
    pub const fn spanners(&self) -> &BTreeMap<SpannerId, Spanner> {
        &self.spanners
    }

    /// The marks of the score, by identifier.
    #[must_use]
    pub const fn marks(&self) -> &BTreeMap<MarkId, ScoreMark> {
        &self.marks
    }

    /// The tempo and meter map of the score.
    #[must_use]
    pub const fn tempo_map(&self) -> &TempoMap {
        &self.tempo_map
    }

    /// The revision counter. It increases on every successful command.
    #[must_use]
    pub const fn revision(&self) -> Revision {
        self.revision
    }

    /// The schema number that this score was read under.
    #[must_use]
    pub const fn schema(&self) -> SchemaVersion {
        self.schema
    }

    /// Fields from a newer schema that this build does not model.
    #[must_use]
    pub const fn extra(&self) -> &BTreeMap<String, serde_json::Value> {
        &self.extra
    }

    /// The parts of the score, for a change.
    ///
    /// Every mutator below is `pub(crate)`, because section 3.4 makes
    /// `Score::apply` the one way into the aggregate. `apply` lives in the
    /// sibling module `apply`, so it reaches no private field of this one.
    pub(crate) const fn parts_mut(&mut self) -> &mut BTreeMap<PartId, Part> {
        &mut self.parts
    }

    /// The staves of the score, for a change.
    pub(crate) const fn staves_mut(&mut self) -> &mut BTreeMap<StaffId, Staff> {
        &mut self.staves
    }

    /// The voices of the score, for a change.
    pub(crate) const fn voices_mut(&mut self) -> &mut BTreeMap<VoiceId, Voice> {
        &mut self.voices
    }

    /// The measures of the score, for a change.
    pub(crate) const fn measures_mut(&mut self) -> &mut BTreeMap<MeasureId, Measure> {
        &mut self.measures
    }

    /// The notes of the score, for a change.
    pub(crate) const fn notes_mut(&mut self) -> &mut BTreeMap<NoteId, Note> {
        &mut self.notes
    }

    /// The rests of the score, for a change.
    pub(crate) const fn rests_mut(&mut self) -> &mut BTreeMap<NoteId, Rest> {
        &mut self.rests
    }

    /// The spanners of the score, for a change.
    pub(crate) const fn spanners_mut(&mut self) -> &mut BTreeMap<SpannerId, Spanner> {
        &mut self.spanners
    }

    /// The marks of the score, for a change.
    pub(crate) const fn marks_mut(&mut self) -> &mut BTreeMap<MarkId, ScoreMark> {
        &mut self.marks
    }

    /// Replace the tempo and meter map.
    ///
    /// The caller builds the new map through `TempoMapEdit` and its `finish`
    /// gate, so no unchecked map reaches the aggregate.
    pub(crate) fn set_tempo_map(&mut self, map: TempoMap) {
        self.tempo_map = map;
    }

    /// Step the revision counter by one.
    ///
    /// `Score::apply` calls it once, after a command is accepted and applied.
    pub(crate) const fn bump_revision(&mut self) {
        self.revision = self.revision.next();
    }

    /// Take the revision counter that `score/meta.json` carried.
    ///
    /// The canonical reader is the one caller. Every other path steps the
    /// counter through `bump_revision`.
    pub(crate) const fn set_revision(&mut self, revision: Revision) {
        self.revision = revision;
    }

    /// Take the identifier counter that `score/meta.json` carried.
    ///
    /// The canonical reader is the one caller. Every other path takes the next
    /// number through `mint_id`.
    pub(crate) const fn set_next_id(&mut self, next_id: u64) {
        self.next_id = next_id;
    }

    /// The bag of unknown fields, for a change.
    pub(crate) const fn extra_mut(&mut self) -> &mut BTreeMap<String, serde_json::Value> {
        &mut self.extra
    }
}

impl Default for Score {
    /// An empty score with one measure of four-four in C major.
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use core::num::NonZeroU8;

    use duet_time::{NoteValue, Ticks, Tuplet, split_tuplet};

    use super::{C_MAJOR, Duration, FOUR_FOUR, SCHEMA, Score};
    use crate::ids::Revision;

    /// A tuplet of `count` notes in the time of `over` notes.
    fn tuplet(count: u8, over: u8) -> Tuplet {
        Tuplet::new(
            NonZeroU8::new(count).expect("a non-zero count"),
            NonZeroU8::new(over).expect("a non-zero over"),
        )
    }

    #[test]
    fn a_plain_duration_is_its_note_value() {
        let quarter = Duration::new(NoteValue::Quarter, 0, None);
        assert_eq!(
            quarter.ticks(),
            NoteValue::Quarter.ticks(),
            "a duration with no dot and no tuplet is the note value itself"
        );
    }

    #[test]
    fn one_dot_adds_one_half_of_the_value() {
        let dotted = Duration::new(NoteValue::Half, 1, None);
        let half = NoteValue::Half.ticks().get();
        assert_eq!(
            dotted.ticks(),
            Ticks::new(half + (half >> 1)),
            "one dot adds one half of the note value"
        );
    }

    #[test]
    fn two_dots_add_one_half_and_one_quarter() {
        let dotted = Duration::new(NoteValue::Half, 2, None);
        let half = NoteValue::Half.ticks().get();
        assert_eq!(
            dotted.ticks(),
            Ticks::new(half + (half >> 1) + (half >> 2)),
            "the second dot adds one half of the first dot"
        );
    }

    #[test]
    fn a_dot_that_adds_nothing_ends_the_sum() {
        let thirteen = Duration::new(NoteValue::Whole, 13, None);
        let every = Duration::new(NoteValue::Whole, u8::MAX, None);
        assert_eq!(
            thirteen.ticks(),
            every.ticks(),
            "a dot below one tick adds nothing, so more dots change nothing"
        );
    }

    #[test]
    fn a_dot_run_stays_below_twice_the_value() {
        let every = Duration::new(NoteValue::Whole, u8::MAX, None);
        let whole = NoteValue::Whole.ticks().get();
        assert!(
            every.ticks().get() < whole * 2,
            "the dot series converges below twice the note value"
        );
    }

    #[test]
    fn a_triplet_takes_two_thirds_of_the_value() {
        let triplet = Duration::new(NoteValue::Quarter, 0, Some(tuplet(3, 2)));
        let quarter = NoteValue::Quarter.ticks().get();
        assert_eq!(
            triplet.ticks().get() * 3,
            quarter * 2,
            "three notes of the triplet fill the time of two plain notes"
        );
    }

    #[test]
    fn a_tuplet_of_a_dotted_value_reads_the_dots_first() {
        let dotted_triplet = Duration::new(NoteValue::Quarter, 1, Some(tuplet(3, 2)));
        let plain_triplet = Duration::new(NoteValue::Quarter, 0, Some(tuplet(3, 2)));
        assert!(
            dotted_triplet.ticks() > plain_triplet.ticks(),
            "the dot grows the written value that the tuplet then divides"
        );
    }

    #[test]
    fn a_tuplet_answers_the_first_part_of_the_kernel_split() {
        for count in [1_u8, 2, 3, 5, 7, 11, 13, 64, 255] {
            for over in [1_u8, 2, 4, 255] {
                let written = Duration::new(NoteValue::Quarter, 1, None);
                let group = written.ticks().get() * i64::from(over);
                let parts = split_tuplet(
                    Ticks::new(group),
                    NonZeroU8::new(count).expect("a non-zero count"),
                );
                let first = parts
                    .first()
                    .copied()
                    .expect("a split answers one part for each note of the group");
                let member = Duration::new(NoteValue::Quarter, 1, Some(tuplet(count, over)));
                assert_eq!(
                    member.ticks(),
                    first,
                    "a tuplet of {count} notes in the time of {over} answers the first part of the kernel split"
                );
            }
        }
    }

    #[test]
    fn a_tuplet_of_more_parts_than_ticks_answers_zero() {
        let dense = Duration::new(NoteValue::ThirtySecond, 0, Some(tuplet(255, 1)));
        assert_eq!(
            dense.ticks(),
            Ticks::ZERO,
            "255 notes in the time of one thirty-second note leave no whole tick for one part"
        );
    }

    #[test]
    fn seven_parts_of_a_seven_tuplet_sum_below_the_span() {
        let written = Duration::new(NoteValue::Quarter, 0, None);
        let member = Duration::new(NoteValue::Quarter, 0, Some(tuplet(7, 4)));
        assert!(
            member.ticks().get() * 7 < written.ticks().get() * 4,
            "the tick resolution has no factor of seven, so the last member of the group takes the ticks that the seven equal parts leave"
        );
    }

    #[test]
    fn the_opening_meter_holds_four_quarter_beats() {
        assert_eq!(
            FOUR_FOUR.beats_per_bar().get(),
            4,
            "`non_zero_u8` checks nothing, so the literal of the opening meter is pinned here"
        );
        assert_eq!(
            FOUR_FOUR.beat_unit(),
            NoteValue::Quarter,
            "the opening meter counts quarter notes"
        );
    }

    #[test]
    fn an_empty_score_holds_one_measure() {
        let score = Score::new();
        assert_eq!(
            score.measures().len(),
            1,
            "an empty score opens with one measure"
        );
    }

    #[test]
    fn the_first_measure_is_four_four_in_c_major() {
        let score = Score::new();
        let first = score.measures().values().next().expect("the first measure");
        assert_eq!(
            first.start(),
            Ticks::ZERO,
            "the first measure starts at score zero"
        );
        assert_eq!(
            first.meter(),
            FOUR_FOUR,
            "an empty score opens in four-four"
        );
        assert_eq!(first.key(), C_MAJOR, "an empty score opens in C major");
    }

    #[test]
    fn the_tempo_map_of_a_new_score_states_four_four() {
        let score = Score::new();
        let point = score
            .tempo_map()
            .meter_at(Ticks::ZERO)
            .expect("a new score seeds one meter entry at tick zero");
        assert_eq!(
            point.meter(),
            FOUR_FOUR,
            "the tempo map and the first measure state one meter"
        );
        assert_eq!(
            point.ticks(),
            Ticks::ZERO,
            "the meter entry sits at score zero"
        );
    }

    #[test]
    fn an_empty_score_holds_no_other_entity() {
        let score = Score::new();
        assert!(score.parts().is_empty(), "an empty score holds no part");
        assert!(score.staves().is_empty(), "an empty score holds no staff");
        assert!(score.voices().is_empty(), "an empty score holds no voice");
        assert!(score.notes().is_empty(), "an empty score holds no note");
        assert!(score.rests().is_empty(), "an empty score holds no rest");
        assert!(
            score.spanners().is_empty(),
            "an empty score holds no spanner"
        );
        assert!(score.marks().is_empty(), "an empty score holds no mark");
        assert!(
            score.extra().is_empty(),
            "an empty score holds no unknown field"
        );
    }

    #[test]
    fn an_empty_score_opens_at_revision_zero_under_the_current_schema() {
        let score = Score::new();
        assert_eq!(score.revision(), Revision::ZERO, "no command has run yet");
        assert_eq!(
            score.schema(),
            SCHEMA,
            "a new score carries the current schema"
        );
    }

    #[test]
    fn the_default_score_is_the_new_score() {
        assert_eq!(
            Score::default(),
            Score::new(),
            "`Default` answers what `new` builds"
        );
    }

    #[test]
    fn the_counter_never_mints_one_identifier_twice() {
        let mut score = Score::new();
        let first = score.mint_id();
        let second = score.mint_id();
        assert_ne!(first, second, "the counter answers a new number every time");
        assert!(
            first > 0,
            "the first measure already took the identifier at zero"
        );
    }
}
