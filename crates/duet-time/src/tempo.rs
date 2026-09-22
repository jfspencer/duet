//! The tempo map: the tempo and meter entries, and every query over them.

use core::cmp::Ordering;
use core::num::{NonZeroI64, NonZeroU8, NonZeroU16, NonZeroU32};

use serde::{Deserialize, Serialize};

use crate::bbt::Bbt;
use crate::error::TimeError;
use crate::position::{Position, TimeDomain};
use crate::units::{SUPERCLOCK_HZ, SuperClock, TICKS_PER_QUARTER, Ticks};

/// Ticks in one thirty-second note.
///
/// The tick resolution is 2^7 times 3 times 5, so the division by eight is
/// exact and the six note values scale from one base.
const TICKS_PER_THIRTY_SECOND: i64 = TICKS_PER_QUARTER.get().div_euclid(8);

/// Seconds in one minute.
const SECONDS_PER_MINUTE: i64 = 60;

/// Superclock ticks in one minute.
const SUPERCLOCKS_PER_MINUTE: i64 = SUPERCLOCK_HZ.get().saturating_mul(SECONDS_PER_MINUTE);

/// A written note value.
///
/// It derives `Ord` so that a caller can take the shortest held letter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NoteValue {
    /// A whole note.
    Whole,
    /// A half note.
    Half,
    /// A quarter note.
    Quarter,
    /// An eighth note.
    Eighth,
    /// A sixteenth note.
    Sixteenth,
    /// A thirty-second note.
    ThirtySecond,
}

impl NoteValue {
    /// The tick count of one note of this value.
    ///
    /// Every arm scales `TICKS_PER_THIRTY_SECOND`, so a change to the tick
    /// resolution moves every arm together and no arm holds a tick literal.
    #[must_use]
    pub const fn ticks(self) -> Ticks {
        Ticks::new(TICKS_PER_THIRTY_SECOND.saturating_mul(self.thirty_seconds()))
    }

    /// How many thirty-second notes this value holds.
    const fn thirty_seconds(self) -> i64 {
        match self {
            Self::Whole => 32,
            Self::Half => 16,
            Self::Quarter => 8,
            Self::Eighth => 4,
            Self::Sixteenth => 2,
            Self::ThirtySecond => 1,
        }
    }
}

/// An exact ratio. The tempo holds one so that no float enters the kernel.
///
/// `#[serde(try_from = "RatioFields")]` routes deserialization through `new`,
/// for the reason section 2.6a of `roadmap/duet-v1/architecture.md` states for
/// `Finite`: a derived `Deserialize` writes the two fields directly and breaks
/// the canonical form that the derived `Eq` and `Hash` read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "RatioFields")]
pub struct Ratio {
    /// The numerator, which carries the sign of the ratio.
    numerator: i64,
    /// The denominator, which is always positive.
    denominator: NonZeroI64,
}

/// The stored fields of a `Ratio`, read before the constructor runs.
///
/// It holds the fields of `Ratio` and no invariant, so the derived
/// `Deserialize` on it writes no bad value and `Ratio::try_from` is the gate.
#[derive(Debug, Clone, Copy, Deserialize)]
struct RatioFields {
    /// The stored numerator.
    numerator: i64,
    /// The stored denominator.
    denominator: NonZeroI64,
}

impl TryFrom<RatioFields> for Ratio {
    type Error = TimeError;

    /// The canonical ratio of the stored pair.
    ///
    /// # Errors
    /// Returns the error of `Ratio::new`.
    fn try_from(fields: RatioFields) -> Result<Self, TimeError> {
        Self::new(fields.numerator, fields.denominator)
    }
}

impl Ratio {
    /// The canonical form of `numerator` over `denominator`.
    ///
    /// The pair divides by its greatest common divisor and the sign moves to
    /// the numerator, so two ratios of one value are one value and the derived
    /// `Eq` and `Hash` agree with the arithmetic.
    ///
    /// # Errors
    /// Returns `TimeError::Overflow` when the magnitude of the numerator or of
    /// the denominator has no positive 64-bit form. The magnitude of
    /// `i64::MIN` is the one such value.
    pub fn new(numerator: i64, denominator: NonZeroI64) -> Result<Self, TimeError> {
        let numerator_magnitude = u128::from(numerator.unsigned_abs());
        let denominator_magnitude = u128::from(denominator.get().unsigned_abs());
        let divisor = greatest_common_divisor(numerator_magnitude, denominator_magnitude);
        let (Some(reduced_numerator), Some(reduced_denominator)) = (
            numerator_magnitude.checked_div(divisor),
            denominator_magnitude.checked_div(divisor),
        ) else {
            return Err(TimeError::Overflow);
        };
        let negative = (numerator < 0) != (denominator.get() < 0);
        let signed_numerator = signed_magnitude(reduced_numerator, negative)?;
        let signed_denominator = signed_magnitude(reduced_denominator, false)?;
        let Some(checked_denominator) = NonZeroI64::new(signed_denominator) else {
            return Err(TimeError::Overflow);
        };
        Ok(Self {
            numerator: signed_numerator,
            denominator: checked_denominator,
        })
    }

    /// The numerator, which carries the sign of the ratio.
    #[must_use]
    pub const fn numerator(self) -> i64 {
        self.numerator
    }

    /// The denominator, which is always positive.
    #[must_use]
    pub const fn denominator(self) -> NonZeroI64 {
        self.denominator
    }
}

/// The greatest common divisor of two magnitudes.
///
/// A zero pairs with any value, so `greatest_common_divisor(0, value)` is
/// `value` and a zero numerator reduces to zero over one. It reads 128-bit
/// magnitudes, because `reduced_scale` divides the two sides of the tempo
/// scale and each of those leaves the 64-bit range.
const fn greatest_common_divisor(left: u128, right: u128) -> u128 {
    let mut larger = left;
    let mut smaller = right;
    while smaller != 0 {
        let remainder = larger.rem_euclid(smaller);
        larger = smaller;
        smaller = remainder;
    }
    larger
}

/// The signed value of `magnitude`.
///
/// # Errors
/// Returns `TimeError::Overflow` when `magnitude` has no positive 64-bit form.
fn signed_magnitude(magnitude: u128, negative: bool) -> Result<i64, TimeError> {
    let Ok(value) = i64::try_from(magnitude) else {
        return Err(TimeError::Overflow);
    };
    if negative {
        value.checked_neg().ok_or(TimeError::Overflow)
    } else {
        Ok(value)
    }
}

/// A tempo, held as an exact ratio so that no float enters the kernel.
///
/// `#[serde(try_from = "TempoFields")]` routes deserialization through `new`,
/// so a stored rate that no constructor would pass is refused with a serde
/// error instead of reaching a query.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "TempoFields")]
pub struct Tempo {
    /// The rate in beats per minute.
    beats_per_minute: Ratio,
    /// The note value of one beat.
    beat_unit: NoteValue,
    /// The ramp marker of the entry.
    ramped: bool,
}

/// The stored fields of a `Tempo`, read before the constructor runs.
///
/// The rate is a `Ratio`, so it carries its own gate, and this struct adds
/// none of its own.
#[derive(Debug, Clone, Copy, Deserialize)]
struct TempoFields {
    /// The stored rate in beats per minute.
    beats_per_minute: Ratio,
    /// The stored note value of one beat.
    beat_unit: NoteValue,
    /// The stored ramp marker.
    ramped: bool,
}

impl TryFrom<TempoFields> for Tempo {
    type Error = TimeError;

    /// The tempo of the stored fields.
    ///
    /// # Errors
    /// Returns the error of `Tempo::new`.
    fn try_from(fields: TempoFields) -> Result<Self, TimeError> {
        Self::new(fields.beats_per_minute, fields.beat_unit, fields.ramped)
    }
}

impl Tempo {
    /// A tempo of `beats_per_minute` over `beat_unit`.
    ///
    /// # Errors
    /// Returns `TimeError::NotRepresentable` when the rate is zero or
    /// negative. The scale divides by the rate, so a zero rate names no
    /// segment and a negative rate runs the timeline backwards. Both sides of
    /// the rate carry the test, so this constructor is the one gate and it
    /// rests on no invariant of `Ratio::new`. No query repeats the test.
    pub const fn new(
        beats_per_minute: Ratio,
        beat_unit: NoteValue,
        ramped: bool,
    ) -> Result<Self, TimeError> {
        if beats_per_minute.numerator() <= 0 || beats_per_minute.denominator().get() <= 0 {
            return Err(TimeError::NotRepresentable);
        }
        Ok(Self {
            beats_per_minute,
            beat_unit,
            ramped,
        })
    }

    /// The rate in beats per minute.
    #[must_use]
    pub const fn beats_per_minute(self) -> Ratio {
        self.beats_per_minute
    }

    /// The note value of one beat.
    #[must_use]
    pub const fn beat_unit(self) -> NoteValue {
        self.beat_unit
    }

    /// The ramp marker of the entry.
    ///
    /// A ramped entry governs its segment as a step: `TempoMap::superclock_at`
    /// and `TempoMap::ticks_at` read the rate and the beat unit, and they do
    /// not read this marker. `Tempo` declares no end rate and no curve, so no
    /// interpolation rule exists yet.
    #[must_use]
    pub const fn ramped(self) -> bool {
        self.ramped
    }
}

/// A time signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Meter {
    /// The beats in one bar.
    beats_per_bar: NonZeroU8,
    /// The note value of one beat.
    beat_unit: NoteValue,
}

impl Meter {
    /// A meter of `beats_per_bar` beats of `beat_unit`.
    #[must_use]
    pub const fn new(beats_per_bar: NonZeroU8, beat_unit: NoteValue) -> Self {
        Self {
            beats_per_bar,
            beat_unit,
        }
    }

    /// The beats in one bar.
    #[must_use]
    pub const fn beats_per_bar(self) -> NonZeroU8 {
        self.beats_per_bar
    }

    /// The note value of one beat.
    #[must_use]
    pub const fn beat_unit(self) -> NoteValue {
        self.beat_unit
    }
}

/// One tempo entry.
///
/// It carries its own tick position and its own superclock position, so
/// `TempoMap::superclock_at` and `TempoMap::ticks_at` never call back into the
/// map. `TempoMapEdit::finish` gates both views: the tick rises strictly, the
/// clock never falls, and the clock is the value that the rate of the entry
/// before it produces.
///
/// The entry carries no address, because no query reads one. `bbt_at` and
/// `ticks_at_bbt` read the meter list alone. A stored address here could not
/// be gated at all on a map with an empty meter list, and the address rise made
/// the true address of every such entry illegal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TempoPoint {
    /// The tick position of the entry.
    ticks: Ticks,
    /// The superclock position of the entry.
    clock: SuperClock,
    /// The tempo the entry starts.
    tempo: Tempo,
}

impl TempoPoint {
    /// A tempo entry at `ticks` and `clock`.
    #[must_use]
    pub const fn new(ticks: Ticks, clock: SuperClock, tempo: Tempo) -> Self {
        Self {
            ticks,
            clock,
            tempo,
        }
    }

    /// The tick position of the entry.
    #[must_use]
    pub const fn ticks(self) -> Ticks {
        self.ticks
    }

    /// The superclock position of the entry.
    #[must_use]
    pub const fn clock(self) -> SuperClock {
        self.clock
    }

    /// The tempo the entry starts.
    #[must_use]
    pub const fn tempo(self) -> Tempo {
        self.tempo
    }
}

/// One meter entry.
///
/// It carries its own tick position and its own address, so `TempoMap::bbt_at`
/// and `TempoMap::ticks_at_bbt` never call back into the map.
/// `TempoMapEdit::finish` gates both views: the tick and the address rise, the
/// address is the value that the meter of the entry before it produces, and the
/// meter of the entry itself names that address.
///
/// The last rule holds an entry at a tick whose address its own meter can name:
/// the beat is at or below the beats of one bar, and the tick is below one
/// beat. A meter change on a barline always meets the rule, and a change at
/// beat 7 of a bar of seven beats to a meter of four beats does not.
///
/// The entry carries no superclock position, because no query reads one.
/// `superclock_at` and `ticks_at` read the tempo list alone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeterPoint {
    /// The tick position of the entry.
    ticks: Ticks,
    /// The address of the entry.
    bbt: Bbt,
    /// The meter the entry starts.
    meter: Meter,
}

impl MeterPoint {
    /// A meter entry at `ticks` and `bbt`.
    #[must_use]
    pub const fn new(ticks: Ticks, bbt: Bbt, meter: Meter) -> Self {
        Self { ticks, bbt, meter }
    }

    /// The tick position of the entry.
    #[must_use]
    pub const fn ticks(self) -> Ticks {
        self.ticks
    }

    /// The address of the entry.
    #[must_use]
    pub const fn bbt(self) -> Bbt {
        self.bbt
    }

    /// The meter the entry starts.
    #[must_use]
    pub const fn meter(self) -> Meter {
        self.meter
    }
}

/// A sorted tempo and meter map.
///
/// An empty map states one pair alone: the origin. Every cross-domain query on
/// it answers zero, and every same-domain query on it is the identity. A map
/// that `TempoMapEdit::finish` accepts and that holds a tempo entry is never
/// empty, so a project map always states a real tempo.
///
/// `#[serde(try_from = "TempoMapFields")]` runs the validation of
/// `TempoMapEdit::finish` on the deserialization path, so a stored document
/// meets the same gate as a builder and a hand edit cannot walk past it.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TempoMapFields")]
pub struct TempoMap {
    /// The tempo entries, ordered by tick.
    tempos: Vec<TempoPoint>,
    /// The meter entries, ordered by tick.
    meters: Vec<MeterPoint>,
}

/// The stored lists of a `TempoMap`, read before the validation runs.
///
/// The list invariants belong to the map, so this struct carries the two
/// lists and `TempoMap::try_from` carries the gate.
#[derive(Debug, Clone, Deserialize)]
struct TempoMapFields {
    /// The stored tempo entries.
    tempos: Vec<TempoPoint>,
    /// The stored meter entries.
    meters: Vec<MeterPoint>,
}

impl TryFrom<TempoMapFields> for TempoMap {
    type Error = TimeError;

    /// The validated map of the stored lists.
    ///
    /// # Errors
    /// Returns the error of `TempoMapEdit::finish`.
    fn try_from(fields: TempoMapFields) -> Result<Self, TimeError> {
        validate_map(&fields.tempos, &fields.meters)?;
        Ok(Self {
            tempos: fields.tempos,
            meters: fields.meters,
        })
    }
}

impl TempoMap {
    /// The tempo entries, ordered by tick.
    ///
    /// `duet-export` and `duet-interchange` write the map to another format,
    /// and `crates/duet` draws a tempo ruler. Each one reads the list here,
    /// because `duet-time` holds no `serde_json` edge.
    #[must_use]
    pub fn tempos(&self) -> &[TempoPoint] {
        &self.tempos
    }

    /// The meter entries, ordered by tick.
    ///
    /// `duet-engrave` draws barlines and beam groups from this list.
    #[must_use]
    pub fn meters(&self) -> &[MeterPoint] {
        &self.meters
    }

    /// The tempo entry that governs `ticks`, or `None` on an empty list.
    ///
    /// It reads the entry that `superclock_at` reads, so the two answers can
    /// never disagree. A tick below the first entry answers the first entry,
    /// which is the entry whose rate that query uses.
    #[must_use]
    pub fn tempo_at(&self, ticks: Ticks) -> Option<TempoPoint> {
        let index = self.tempos.partition_point(|point| point.ticks() <= ticks);
        self.tempos.get(index.saturating_sub(1)).copied()
    }

    /// The meter entry that governs `ticks`, or `None` on an empty list.
    ///
    /// It reads the entry that `bbt_at` reads, for the reason `tempo_at`
    /// states.
    #[must_use]
    pub fn meter_at(&self, ticks: Ticks) -> Option<MeterPoint> {
        let index = self.meters.partition_point(|point| point.ticks() <= ticks);
        self.meters.get(index.saturating_sub(1)).copied()
    }

    /// The superclock position of a tick position.
    ///
    /// The map reads the last tempo entry at or before `ticks` and scales the
    /// tick delta at that entry's rate. A tick below the first entry uses the
    /// first entry's rate, so the answer stays monotonic across timeline zero.
    /// A ramped entry governs its segment as a step, because `Tempo` declares
    /// no end rate and no curve. An empty map answers `SuperClock::ZERO`.
    #[must_use]
    pub fn superclock_at(&self, ticks: Ticks) -> SuperClock {
        let index = self.tempos.partition_point(|point| point.ticks() <= ticks);
        let Some(&anchor) = self.tempos.get(index.saturating_sub(1)) else {
            return SuperClock::ZERO;
        };
        let delta_ticks = ticks.saturating_sub(anchor.ticks());
        let delta_clock = superclocks_for_ticks(anchor.tempo(), delta_ticks.get());
        anchor.clock().saturating_add(SuperClock::new(delta_clock))
    }

    /// The tick position of a superclock position.
    ///
    /// It is the mirror of `superclock_at`, and it reads the same entry list.
    /// A ramped entry governs its segment as a step, for the same reason. An
    /// empty map answers `Ticks::ZERO`.
    #[must_use]
    pub fn ticks_at(&self, clock: SuperClock) -> Ticks {
        let index = self.tempos.partition_point(|point| point.clock() <= clock);
        let Some(&anchor) = self.tempos.get(index.saturating_sub(1)) else {
            return Ticks::ZERO;
        };
        let delta_clock = clock.saturating_sub(anchor.clock());
        let delta_ticks = ticks_for_superclocks(anchor.tempo(), delta_clock.get());
        anchor.ticks().saturating_add(Ticks::new(delta_ticks))
    }

    /// The address of a tick position.
    ///
    /// The map reads the last meter entry at or before `ticks`. The whole
    /// address holds at `Bbt::ORIGIN` below the first bar, and above the last
    /// bar it holds at the last address the governing meter names. An empty
    /// map answers `Bbt::ORIGIN`.
    ///
    /// The answer never falls as the tick rises, under
    /// `Bbt::lexicographic_cmp`, on any map that `TempoMapEdit::finish`
    /// accepts. Both clamps keep that order, and the meter cross-check of
    /// `finish` keeps it across a segment boundary.
    #[must_use]
    pub fn bbt_at(&self, ticks: Ticks) -> Bbt {
        let index = self.meters.partition_point(|point| point.ticks() <= ticks);
        let Some(&anchor) = self.meters.get(index.saturating_sub(1)) else {
            return Bbt::ORIGIN;
        };
        clamped_address(anchor, ticks)
    }

    /// The tick position of an address.
    ///
    /// # Errors
    /// Returns `TimeError::BbtOutOfRange` when the meter list is empty and the
    /// address is not `Bbt::ORIGIN`, when the beat is above the governing
    /// meter, or when the tick is at or above one beat.
    ///
    /// `TimeError::Overflow` is the total fallback of the tick arithmetic, and
    /// no map that `TempoMapEdit::finish` accepts can reach it. The meter
    /// cross-check of `finish` holds a consistent chain of meter entries below
    /// about 8.4e15 ticks: a bar number stays inside the 32-bit range, and one
    /// bar holds at most 255 beats of 7680 ticks. The arm stays, because it is
    /// what makes the function total on a list this signature cannot refuse.
    pub fn ticks_at_bbt(&self, bbt: Bbt) -> Result<Ticks, TimeError> {
        let index = self
            .meters
            .partition_point(|point| point.bbt().lexicographic_cmp(bbt).is_le());
        let Some(&anchor) = self.meters.get(index.saturating_sub(1)) else {
            return if bbt == Bbt::ORIGIN {
                Ok(Ticks::ZERO)
            } else {
                Err(TimeError::BbtOutOfRange)
            };
        };
        let meter = anchor.meter();
        if bbt.beat().get() > u16::from(meter.beats_per_bar().get()) {
            return Err(TimeError::BbtOutOfRange);
        }
        if i64::from(bbt.tick()) >= meter.beat_unit().ticks().get() {
            return Err(TimeError::BbtOutOfRange);
        }
        ticks_for_address(anchor, bbt).ok_or(TimeError::Overflow)
    }

    /// The position in the target domain.
    ///
    /// A position already in `target` returns unchanged, so a conversion to
    /// its own domain loses no tick and needs no entry.
    #[must_use]
    pub fn convert(&self, position: Position, target: TimeDomain) -> Position {
        match (position, target) {
            (Position::Beats(_), TimeDomain::Beats) | (Position::Audio(_), TimeDomain::Audio) => {
                position
            },
            (Position::Beats(ticks), TimeDomain::Audio) => {
                Position::Audio(self.superclock_at(ticks))
            },
            (Position::Audio(clock), TimeDomain::Beats) => Position::Beats(self.ticks_at(clock)),
        }
    }

    /// A builder seeded from this map.
    #[must_use]
    pub fn edit(&self) -> TempoMapEdit {
        TempoMapEdit {
            tempos: self.tempos.clone(),
            meters: self.meters.clone(),
        }
    }

    /// The order of two positions, across domains when they differ.
    ///
    /// Two positions of one domain compare by their own values. Two positions
    /// of different domains compare in the audio domain, which is the finer of
    /// the two, so the comparison loses at most one superclock tick.
    #[must_use]
    pub fn cmp(&self, left: Position, right: Position) -> Ordering {
        match (left, right) {
            (Position::Beats(first), Position::Beats(second)) => first.cmp(&second),
            (Position::Audio(first), Position::Audio(second)) => first.cmp(&second),
            (Position::Beats(_), Position::Audio(_)) | (Position::Audio(_), Position::Beats(_)) => {
                left.in_audio(self).cmp(&right.in_audio(self))
            },
        }
    }

    /// Whether two positions name one point of the timeline.
    ///
    /// This is the timeline question, and the derived `PartialEq` of
    /// `Position` is the structural question. The two answers differ: two
    /// positions of different domains are never structurally equal, and this
    /// method still reports the pair that names one instant. The two names
    /// therefore differ, so a reader can see which question a call asks.
    ///
    /// It reads `TempoMap::cmp`, so the order and the equality can never
    /// disagree.
    #[must_use]
    pub fn same_instant(&self, left: Position, right: Position) -> bool {
        self.cmp(left, right) == Ordering::Equal
    }
}

/// A checked builder for a tempo map.
///
/// It validates the order and the first-point rule, and `finish` produces the
/// map.
#[derive(Debug, Clone, Default)]
pub struct TempoMapEdit {
    /// The tempo entries the builder holds.
    tempos: Vec<TempoPoint>,
    /// The meter entries the builder holds.
    meters: Vec<MeterPoint>,
}

impl TempoMapEdit {
    /// An empty builder.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            tempos: Vec::new(),
            meters: Vec::new(),
        }
    }

    /// This builder with `point` added at the end of the tempo list.
    #[must_use]
    pub fn push_tempo(mut self, point: TempoPoint) -> Self {
        self.tempos.push(point);
        self
    }

    /// This builder with `point` added at the end of the meter list.
    #[must_use]
    pub fn push_meter(mut self, point: MeterPoint) -> Self {
        self.meters.push(point);
        self
    }

    /// The tempo entries the builder holds.
    ///
    /// A caller reads the list, decides what to change, and then calls one of
    /// the edit methods. Without this the caller must read `TempoMap::tempos`
    /// before it calls `TempoMap::edit`.
    #[must_use]
    pub fn tempos(&self) -> &[TempoPoint] {
        &self.tempos
    }

    /// The meter entries the builder holds.
    ///
    /// It answers the meter list, for the reason `tempos` states.
    #[must_use]
    pub fn meters(&self) -> &[MeterPoint] {
        &self.meters
    }

    /// This builder with `point` placed in the tempo list in tick order.
    ///
    /// An entry that already sits at that tick is replaced. "Add a tempo change
    /// at bar 12" is this method, and `push_tempo` appends instead.
    ///
    /// Every entry after the new one keeps the clock of the list it came from,
    /// so a caller calls `recompute_cached_views` before `finish`, the way a
    /// removal does.
    #[must_use]
    pub fn insert_tempo(mut self, point: TempoPoint) -> Self {
        insert_by_tick(&mut self.tempos, point, TempoPoint::ticks);
        self
    }

    /// This builder with `point` placed in the meter list in tick order.
    ///
    /// An entry that already sits at that tick is replaced, and every entry
    /// after the new one keeps the address of the list it came from, for the
    /// reason `insert_tempo` states.
    #[must_use]
    pub fn insert_meter(mut self, point: MeterPoint) -> Self {
        insert_by_tick(&mut self.meters, point, MeterPoint::ticks);
        self
    }

    /// This builder with every tempo entry at `ticks` removed.
    ///
    /// The entries after the removed one keep the clock of the list they came
    /// from, so a removal from the middle leaves a list that `finish` refuses.
    /// `recompute_cached_views` rebuilds those clocks.
    #[must_use]
    pub fn remove_tempo_at(mut self, ticks: Ticks) -> Self {
        self.tempos.retain(|point| point.ticks() != ticks);
        self
    }

    /// This builder with every meter entry at `ticks` removed.
    ///
    /// The entries after the removed one keep the address of the list they came
    /// from, for the reason `remove_tempo_at` states.
    #[must_use]
    pub fn remove_meter_at(mut self, ticks: Ticks) -> Self {
        self.meters.retain(|point| point.ticks() != ticks);
        self
    }

    /// This builder with every cached view the map arithmetic owns rebuilt.
    ///
    /// The tick axis is authoritative, and so is the tempo of a tempo entry and
    /// the meter of a meter entry. The first entry of a list is the anchor and
    /// keeps its stored view. Every later tempo entry takes the clock that the
    /// rate of the entry before it produces, and every later meter entry takes
    /// the address that the meter of the entry before it produces. An entry
    /// holds one cached view, and this method rebuilds it.
    ///
    /// A caller inserts an entry, or removes one, calls this, and then calls
    /// `finish`. `finish` itself rebuilds nothing, so a caller that edits the
    /// lists by hand and forgets this call is still refused.
    ///
    /// It is total: it uses the saturating scale the queries use, and it holds
    /// an address that leaves the range of a `Bbt` at the limit `bbt_at` holds
    /// it at. The PAIR of this method and `finish` is not total at the top of
    /// the address range. This method holds an address above the last bar at
    /// that clamp limit, and `finish` compares the stored address against the
    /// exact address, which no bar above `u32::MAX` has. A meter entry above
    /// bar 4294967295 therefore answers `TimeError::UnorderedMap` after the
    /// rebuild. A meter entry whose own meter cannot name the rebuilt address
    /// answers the same error, and a meter change on a barline never does.
    #[must_use]
    pub fn recompute_cached_views(mut self) -> Self {
        recompute_tempo_clocks(&mut self.tempos);
        recompute_meter_addresses(&mut self.meters);
        self
    }

    /// The validated map.
    ///
    /// The tempo list is validated first, then the meter list. Inside one list
    /// the first-point rule is checked before the order rule, and the order
    /// rule before the cross-check. An empty list is legal, and the two lists
    /// are validated apart.
    ///
    /// # Errors
    /// Returns `TimeError::NoFirstPoint` when a list is not empty and its first
    /// entry does not sit at tick zero and at the origin of its own second
    /// view: superclock zero for a tempo entry, and `Bbt::ORIGIN` for a meter
    /// entry. Returns `TimeError::UnorderedMap` when the ticks of a list do not
    /// rise, when a tempo clock falls, when a meter address does not rise, when
    /// a stored view disagrees with the arithmetic the map itself performs, or
    /// when a meter entry stores an address that its own meter cannot name.
    pub fn finish(self) -> Result<TempoMap, TimeError> {
        validate_map(&self.tempos, &self.meters)?;
        Ok(TempoMap {
            tempos: self.tempos,
            meters: self.meters,
        })
    }
}

/// Refuse a tempo list or a meter list that a query cannot answer.
///
/// The builder and the deserialization path share it, so a stored document
/// meets the gate that a builder meets.
///
/// # Errors
/// Returns `TimeError::NoFirstPoint` or `TimeError::UnorderedMap`.
fn validate_map(tempos: &[TempoPoint], meters: &[MeterPoint]) -> Result<(), TimeError> {
    validate_list(
        tempos,
        tempo_starts_at_origin,
        tempo_pair_rises,
        tempo_pair_agrees,
    )?;
    validate_list(
        meters,
        meter_starts_at_origin,
        meter_pair_rises,
        meter_pair_agrees,
    )
}

/// Whether the first tempo entry sits at the origin of both its views.
fn tempo_starts_at_origin(point: &TempoPoint) -> bool {
    point.ticks() == Ticks::ZERO && point.clock() == SuperClock::ZERO
}

/// Whether the first meter entry sits at the origin of both its views.
///
/// It also tests the stored address against the entry's own meter, so that no
/// meter entry escapes that rule. Every meter names `Bbt::ORIGIN`, so the test
/// refuses no list that the origin rule accepts.
fn meter_starts_at_origin(point: &MeterPoint) -> bool {
    point.ticks() == Ticks::ZERO && point.bbt() == Bbt::ORIGIN && meter_names_its_address(*point)
}

/// Whether the two views of a tempo pair rise.
///
/// The tick must rise strictly: a duplicate tick names an entry that no query
/// can reach. The superclock value must not fall: at an extreme rate two
/// neighbouring ticks round to one superclock tick, and the lookup takes the
/// last entry of an equal run.
fn tempo_pair_rises(previous: &TempoPoint, next: &TempoPoint) -> bool {
    next.ticks() > previous.ticks() && next.clock() >= previous.clock()
}

/// Whether the two views of a meter pair rise strictly.
///
/// A duplicate tick names an entry that no query can reach, and the address
/// lookup of `ticks_at_bbt` is wrong on a list whose addresses do not rise. The
/// cross-check implies the address rule on every pair whose ticks rise, because
/// a higher tick under one meter names a higher address, so the address rule is
/// defence in depth.
fn meter_pair_rises(previous: &MeterPoint, next: &MeterPoint) -> bool {
    next.ticks() > previous.ticks()
        && next.bbt().lexicographic_cmp(previous.bbt()) == Ordering::Greater
}

/// Whether the stored clock of `next` is the clock `previous` produces.
///
/// `superclock_at` reads the stored clock of the anchor and adds a scaled tick
/// delta to it. Without this rule the two halves of that answer can disagree
/// and the query falls as the tick rises.
fn tempo_pair_agrees(previous: &TempoPoint, next: &TempoPoint) -> bool {
    let delta_ticks = next.ticks().saturating_sub(previous.ticks());
    let delta_clock = superclocks_for_ticks(previous.tempo(), delta_ticks.get());
    next.clock()
        == previous
            .clock()
            .saturating_add(SuperClock::new(delta_clock))
}

/// Whether the stored address of `next` agrees with both meters of the pair.
///
/// `bbt_at` selects its anchor by tick and `ticks_at_bbt` selects its anchor by
/// address. Without the first rule the two selections name different segments
/// and the round trip moves the position by a whole bar. Without the second
/// rule `ticks_at_bbt` refuses the very address that the entry stores, and
/// `bbt_at` answers an address that the list does not hold.
///
/// It asks for no bar alignment: a meter entry may start inside a bar, at any
/// tick whose address its own meter names.
fn meter_pair_agrees(previous: &MeterPoint, next: &MeterPoint) -> bool {
    exact_address(*previous, next.ticks()) == Some(next.bbt()) && meter_names_its_address(*next)
}

/// Whether the meter of `point` names the address that `point` stores.
///
/// It is the pair of tests that `TempoMap::ticks_at_bbt` makes on a query: the
/// beat is at or below the beats of one bar, and the tick is below one beat.
fn meter_names_its_address(point: MeterPoint) -> bool {
    let meter = point.meter();
    point.bbt().beat().get() <= u16::from(meter.beats_per_bar().get())
        && i64::from(point.bbt().tick()) < meter.beat_unit().ticks().get()
}

/// Place `point` in `points` by tick, and replace an entry at the same tick.
///
/// The list stays ordered by tick, which is the order that `finish` demands and
/// the order that every query reads.
fn insert_by_tick<P: Copy>(points: &mut Vec<P>, point: P, ticks: impl Fn(P) -> Ticks) {
    let at = ticks(point);
    points.retain(|held| ticks(*held) != at);
    let index = points.partition_point(|held| ticks(*held) < at);
    points.insert(index, point);
}

/// Rebuild the clock of every tempo entry after the first.
///
/// Each entry takes the clock that `tempo_pair_agrees` demands of it, so a
/// list this leaves is a list the tempo cross-check accepts.
fn recompute_tempo_clocks(tempos: &mut [TempoPoint]) {
    let mut anchor: Option<TempoPoint> = None;
    for point in tempos {
        if let Some(previous) = anchor {
            let delta_ticks = point.ticks.saturating_sub(previous.ticks);
            let delta_clock = superclocks_for_ticks(previous.tempo, delta_ticks.get());
            point.clock = previous.clock.saturating_add(SuperClock::new(delta_clock));
        }
        anchor = Some(*point);
    }
}

/// Rebuild the address of every meter entry after the first.
///
/// Each entry takes the address that `meter_pair_agrees` demands of it, so a
/// list this leaves is a list the meter cross-check accepts whenever the
/// address of every entry fits the range a `Bbt` names and the meter of every
/// entry names its own new address.
fn recompute_meter_addresses(meters: &mut [MeterPoint]) {
    let mut anchor: Option<MeterPoint> = None;
    for point in meters {
        if let Some(previous) = anchor {
            point.bbt = clamped_address(previous, point.ticks);
        }
        anchor = Some(*point);
    }
}

/// Refuse a point list that starts off the origin, that is out of order, or
/// whose cached view disagrees with the map arithmetic.
///
/// `at_origin` tests the first entry, `rises` tests the order of an adjacent
/// pair, and `agrees` cross-checks that pair against the arithmetic. The tempo
/// list and the meter list share this walk, and each one states its own three
/// rules.
///
/// # Errors
/// Returns `TimeError::NoFirstPoint` or `TimeError::UnorderedMap`.
fn validate_list<P>(
    points: &[P],
    at_origin: impl Fn(&P) -> bool,
    rises: impl Fn(&P, &P) -> bool,
    agrees: impl Fn(&P, &P) -> bool,
) -> Result<(), TimeError> {
    let Some(first) = points.first() else {
        return Ok(());
    };
    if !at_origin(first) {
        return Err(TimeError::NoFirstPoint);
    }
    for pair in points.windows(2) {
        let [previous, next] = pair else {
            continue;
        };
        if !rises(previous, next) {
            return Err(TimeError::UnorderedMap);
        }
        if !agrees(previous, next) {
            return Err(TimeError::UnorderedMap);
        }
    }
    Ok(())
}

/// The floor of `value` times `numerator` over `denominator`.
///
/// The pair divides by its greatest common divisor before the multiply, so an
/// intermediate that would leave the 128-bit range no longer does when the
/// quotient itself is in range. Both sides of the tempo scale carry the tick
/// resolution as a factor, so the reduction covers the reachable class.
///
/// It answers `None` when an intermediate or the result still leaves the
/// range, which a coprime pair of large magnitudes can reach.
/// `checked_div_euclid` answers `None` for a zero divisor, so the function
/// needs no precondition and carries no panic primitive.
fn scale_floor(value: i64, numerator: i128, denominator: i128) -> Option<i64> {
    let (reduced_numerator, reduced_denominator) = reduced_scale(numerator, denominator);
    let scaled = i128::from(value).checked_mul(reduced_numerator)?;
    let quotient = scaled.checked_div_euclid(reduced_denominator)?;
    i64::try_from(quotient).ok()
}

/// The scale pair divided by the greatest common divisor of its magnitudes.
///
/// It answers the pair unchanged when the divisor has no signed 128-bit form
/// or when a division refuses, so the reduction never changes the value of the
/// fraction and never introduces a refusal of its own.
fn reduced_scale(numerator: i128, denominator: i128) -> (i128, i128) {
    let divisor = greatest_common_divisor(numerator.unsigned_abs(), denominator.unsigned_abs());
    let Ok(signed_divisor) = i128::try_from(divisor) else {
        return (numerator, denominator);
    };
    let (Some(reduced_numerator), Some(reduced_denominator)) = (
        numerator.checked_div(signed_divisor),
        denominator.checked_div(signed_divisor),
    ) else {
        return (numerator, denominator);
    };
    (reduced_numerator, reduced_denominator)
}

/// The same value as `scale_floor`, held at the 64-bit bound on a refusal.
fn scale_floor_saturating(value: i64, numerator: i128, denominator: i128) -> i64 {
    scale_floor(value, numerator, denominator)
        .unwrap_or_else(|| saturated_bound(value, numerator, denominator))
}

/// The 64-bit bound on the side that the sign of the scale names.
fn saturated_bound(value: i64, numerator: i128, denominator: i128) -> i64 {
    let sign = i128::from(value.signum())
        .saturating_mul(numerator.signum())
        .saturating_mul(denominator.signum());
    if sign < 0 { i64::MIN } else { i64::MAX }
}

/// The superclock delta of a tick delta under one tempo.
///
/// The exact value is the tick delta times the superclocks of one minute times
/// the rate denominator, over the rate numerator times the ticks of one beat.
/// The floor is the rounding rule, because it is monotonic over the whole
/// 64-bit range and a timeline holds negative positions.
///
/// The answer is held at the 64-bit bound when the exact value leaves the
/// `i64` range, and also when the reduced intermediate still leaves the
/// 128-bit range. The second case needs a rate whose two reduced sides are
/// both above about 1e19 together with a tick delta above about 1e18; no
/// session data reaches it, and the answer is a bound and not a wrong number.
fn superclocks_for_ticks(tempo: Tempo, delta_ticks: i64) -> i64 {
    let (minute_side, beat_side) = scale_operands(tempo);
    scale_floor_saturating(delta_ticks, minute_side, beat_side)
}

/// The tick delta of a superclock delta under one tempo.
///
/// It is the inverse of `superclocks_for_ticks`, with the two sides exchanged
/// and the same floor.
fn ticks_for_superclocks(tempo: Tempo, delta_clock: i64) -> i64 {
    let (minute_side, beat_side) = scale_operands(tempo);
    scale_floor_saturating(delta_clock, beat_side, minute_side)
}

/// The two sides of the tempo scale, each as an exact 128-bit value.
fn scale_operands(tempo: Tempo) -> (i128, i128) {
    let rate = tempo.beats_per_minute();
    let minute_side =
        i128::from(SUPERCLOCKS_PER_MINUTE).saturating_mul(i128::from(rate.denominator().get()));
    let beat_side =
        i128::from(rate.numerator()).saturating_mul(i128::from(tempo.beat_unit().ticks().get()));
    (minute_side, beat_side)
}

/// A normalized bar address before the one-based conversion.
#[derive(Debug, Clone, Copy)]
struct BarAddress {
    /// The one-based bar number, which may leave the 32-bit range.
    bar: i128,
    /// The zero-based beat inside the bar.
    beat_offset: i128,
    /// The tick inside the beat.
    tick_in_beat: i128,
}

/// The tick offset of an address inside its own bar.
fn beat_offset_ticks(address: Bbt, ticks_per_beat: i128) -> Option<i128> {
    i128::from(address.beat().get())
        .checked_sub(1)?
        .checked_mul(ticks_per_beat)?
        .checked_add(i128::from(address.tick()))
}

/// The normalized address of `ticks` under one meter entry.
///
/// The Euclidean pair answers a non-negative remainder for a positive divisor,
/// so the beat and the tick are in range for a query on either side of the
/// entry and a meter entry needs no bar alignment.
fn bar_address(anchor: MeterPoint, ticks: Ticks) -> Option<BarAddress> {
    let meter = anchor.meter();
    let ticks_per_beat = i128::from(meter.beat_unit().ticks().get());
    let ticks_per_bar = ticks_per_beat.checked_mul(i128::from(meter.beats_per_bar().get()))?;
    let anchor_bbt = anchor.bbt();
    let anchor_offset = beat_offset_ticks(anchor_bbt, ticks_per_beat)?;
    let travelled = i128::from(ticks.get()).checked_sub(i128::from(anchor.ticks().get()))?;
    let absolute = anchor_offset.checked_add(travelled)?;
    let bars = absolute.checked_div_euclid(ticks_per_bar)?;
    let remainder = absolute.checked_rem_euclid(ticks_per_bar)?;
    Some(BarAddress {
        bar: i128::from(anchor_bbt.bar().get()).checked_add(bars)?,
        beat_offset: remainder.checked_div_euclid(ticks_per_beat)?,
        tick_in_beat: remainder.checked_rem_euclid(ticks_per_beat)?,
    })
}

/// The address of `ticks` when every field fits the range a `Bbt` can name.
///
/// It answers `None` when the arithmetic or a field leaves that range, which
/// is the case a clamp then covers.
fn exact_address(anchor: MeterPoint, ticks: Ticks) -> Option<Bbt> {
    let address = bar_address(anchor, ticks)?;
    let (Ok(bar_value), Ok(beat_value), Ok(tick_value)) = (
        u32::try_from(address.bar),
        u16::try_from(address.beat_offset.saturating_add(1)),
        u16::try_from(address.tick_in_beat),
    ) else {
        return None;
    };
    let (Some(bar), Some(beat)) = (NonZeroU32::new(bar_value), NonZeroU16::new(beat_value)) else {
        return None;
    };
    Some(Bbt::new(bar, beat, tick_value))
}

/// The address a query holds at when it leaves the range a `Bbt` can name.
///
/// The lower limit is `Bbt::ORIGIN`, the first address that exists. The upper
/// limit is the LAST address the governing meter names: bar `u32::MAX`, its
/// last beat, and the last tick of that beat. The first address of that bar
/// would fall below an address a lower tick already answered, so `bbt_at`
/// would not rise with the tick.
fn clamp_limit(anchor: MeterPoint, ticks: Ticks) -> Bbt {
    if ticks.get() < 0 {
        return Bbt::ORIGIN;
    }
    let meter = anchor.meter();
    let last_tick =
        u16::try_from(meter.beat_unit().ticks().get().saturating_sub(1)).unwrap_or(u16::MAX);
    Bbt::new(
        NonZeroU32::MAX,
        NonZeroU16::from(meter.beats_per_bar()),
        last_tick,
    )
}

/// The address of `ticks`, held inside the range a `Bbt` can name.
///
/// The whole address holds, not the bar alone. A query below the first bar
/// answers `Bbt::ORIGIN`, and a query above the last bar answers the last
/// address the governing meter names.
fn clamped_address(anchor: MeterPoint, ticks: Ticks) -> Bbt {
    exact_address(anchor, ticks).unwrap_or_else(|| clamp_limit(anchor, ticks))
}

/// The tick position of `query` under one meter entry.
///
/// It answers `None` when a step leaves the 128-bit or the 64-bit range.
fn ticks_for_address(anchor: MeterPoint, query: Bbt) -> Option<Ticks> {
    let meter = anchor.meter();
    let ticks_per_beat = i128::from(meter.beat_unit().ticks().get());
    let ticks_per_bar = ticks_per_beat.checked_mul(i128::from(meter.beats_per_bar().get()))?;
    let anchor_bbt = anchor.bbt();
    let anchor_offset = beat_offset_ticks(anchor_bbt, ticks_per_beat)?;
    let bars = i128::from(query.bar().get()).checked_sub(i128::from(anchor_bbt.bar().get()))?;
    let query_offset = bars
        .checked_mul(ticks_per_bar)?
        .checked_add(beat_offset_ticks(query, ticks_per_beat)?)?;
    let answer = i128::from(anchor.ticks().get())
        .checked_add(query_offset)?
        .checked_sub(anchor_offset)?;
    i64::try_from(answer).ok().map(Ticks::new)
}
