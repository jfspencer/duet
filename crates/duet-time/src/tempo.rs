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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ratio {
    /// The numerator, which carries the sign of the ratio.
    numerator: i64,
    /// The denominator, which is always positive.
    denominator: NonZeroI64,
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
        let numerator_magnitude = numerator.unsigned_abs();
        let denominator_magnitude = denominator.get().unsigned_abs();
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
/// `value` and a zero numerator reduces to zero over one.
const fn greatest_common_divisor(left: u64, right: u64) -> u64 {
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
fn signed_magnitude(magnitude: u64, negative: bool) -> Result<i64, TimeError> {
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tempo {
    /// The rate in beats per minute.
    beats_per_minute: Ratio,
    /// The note value of one beat.
    beat_unit: NoteValue,
    /// The ramp marker of the entry.
    ramped: bool,
}

impl Tempo {
    /// A tempo of `beats_per_minute` over `beat_unit`.
    ///
    /// # Errors
    /// Returns `TimeError::NotRepresentable` when the rate is zero or
    /// negative. The scale divides by the rate, so a zero rate names no
    /// segment and a negative rate runs the timeline backwards. This
    /// constructor is the one gate, so no query repeats the test.
    pub const fn new(
        beats_per_minute: Ratio,
        beat_unit: NoteValue,
        ramped: bool,
    ) -> Result<Self, TimeError> {
        if beats_per_minute.numerator() <= 0 {
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
/// It carries its own position in all three views, so a lookup never calls
/// back into the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TempoPoint {
    /// The tick position of the entry.
    ticks: Ticks,
    /// The superclock position of the entry.
    clock: SuperClock,
    /// The address of the entry.
    bbt: Bbt,
    /// The tempo the entry starts.
    tempo: Tempo,
}

impl TempoPoint {
    /// A tempo entry at `ticks`, `clock`, and `bbt`.
    #[must_use]
    pub const fn new(ticks: Ticks, clock: SuperClock, bbt: Bbt, tempo: Tempo) -> Self {
        Self {
            ticks,
            clock,
            bbt,
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

    /// The address of the entry.
    #[must_use]
    pub const fn bbt(self) -> Bbt {
        self.bbt
    }

    /// The tempo the entry starts.
    #[must_use]
    pub const fn tempo(self) -> Tempo {
        self.tempo
    }
}

/// One meter entry, with the same three views as a tempo entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeterPoint {
    /// The tick position of the entry.
    ticks: Ticks,
    /// The superclock position of the entry.
    clock: SuperClock,
    /// The address of the entry.
    bbt: Bbt,
    /// The meter the entry starts.
    meter: Meter,
}

impl MeterPoint {
    /// A meter entry at `ticks`, `clock`, and `bbt`.
    #[must_use]
    pub const fn new(ticks: Ticks, clock: SuperClock, bbt: Bbt, meter: Meter) -> Self {
        Self {
            ticks,
            clock,
            bbt,
            meter,
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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TempoMap {
    /// The tempo entries, ordered by tick.
    tempos: Vec<TempoPoint>,
    /// The meter entries, ordered by tick.
    meters: Vec<MeterPoint>,
}

impl TempoMap {
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
    /// address holds at `Bbt::ORIGIN` below the first bar and at `Bbt::LAST`
    /// above the last bar. An empty map answers `Bbt::ORIGIN`.
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
    /// meter, or when the tick is at or above one beat. Returns
    /// `TimeError::Overflow` when the tick count leaves the 64-bit range.
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

    /// This builder with every tempo entry at `ticks` removed.
    #[must_use]
    pub fn remove_tempo_at(mut self, ticks: Ticks) -> Self {
        self.tempos.retain(|point| point.ticks() != ticks);
        self
    }

    /// This builder with every meter entry at `ticks` removed.
    #[must_use]
    pub fn remove_meter_at(mut self, ticks: Ticks) -> Self {
        self.meters.retain(|point| point.ticks() != ticks);
        self
    }

    /// The validated map.
    ///
    /// The tempo list is validated first, then the meter list. Inside one list
    /// the first-point rule is checked before the order rule. An empty list is
    /// legal, and the two lists are validated apart.
    ///
    /// # Errors
    /// Returns `TimeError::NoFirstPoint` when a list is not empty and its first
    /// entry does not sit at tick zero, at superclock zero, and at
    /// `Bbt::ORIGIN`. Returns `TimeError::UnorderedMap` when the ticks or the
    /// addresses of a list do not rise, or when a superclock value falls.
    pub fn finish(self) -> Result<TempoMap, TimeError> {
        validate_list(&self.tempos, |point| PointViews {
            ticks: point.ticks(),
            clock: point.clock(),
            bbt: point.bbt(),
        })?;
        validate_list(&self.meters, |point| PointViews {
            ticks: point.ticks(),
            clock: point.clock(),
            bbt: point.bbt(),
        })?;
        Ok(TempoMap {
            tempos: self.tempos,
            meters: self.meters,
        })
    }
}

/// The three views of one map entry that the validation reads.
#[derive(Debug, Clone, Copy)]
struct PointViews {
    /// The tick position of the entry.
    ticks: Ticks,
    /// The superclock position of the entry.
    clock: SuperClock,
    /// The address of the entry.
    bbt: Bbt,
}

/// Refuse a point list that starts off the origin or that is out of order.
///
/// `views` reads the three positions of one entry, so the tempo list and the
/// meter list share one rule.
///
/// The tick and the address must rise strictly: a duplicate tick names an
/// entry that no query can reach, and the address lookup of `ticks_at_bbt` is
/// wrong on an unordered list. The superclock value must not fall: at an
/// extreme rate two neighbouring ticks round to one superclock tick, and the
/// lookup takes the last entry of an equal run.
///
/// # Errors
/// Returns `TimeError::NoFirstPoint` or `TimeError::UnorderedMap`.
fn validate_list<P>(points: &[P], views: impl Fn(&P) -> PointViews) -> Result<(), TimeError> {
    let Some(first) = points.first() else {
        return Ok(());
    };
    let start = views(first);
    if start.ticks != Ticks::ZERO || start.clock != SuperClock::ZERO || start.bbt != Bbt::ORIGIN {
        return Err(TimeError::NoFirstPoint);
    }
    for pair in points.windows(2) {
        let [previous, next] = pair else {
            continue;
        };
        let earlier = views(previous);
        let later = views(next);
        let rises = later.ticks > earlier.ticks
            && later.clock >= earlier.clock
            && later.bbt.lexicographic_cmp(earlier.bbt) == Ordering::Greater;
        if !rises {
            return Err(TimeError::UnorderedMap);
        }
    }
    Ok(())
}

/// The floor of `value` times `numerator` over `denominator`.
///
/// It answers `None` when an intermediate or the result leaves the range.
/// `checked_div_euclid` answers `None` for a zero divisor, so the function
/// needs no precondition and carries no panic primitive.
fn scale_floor(value: i64, numerator: i128, denominator: i128) -> Option<i64> {
    let scaled = i128::from(value).checked_mul(numerator)?;
    let quotient = scaled.checked_div_euclid(denominator)?;
    i64::try_from(quotient).ok()
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

/// The address of `ticks`, held inside the range a `Bbt` can name.
///
/// The whole address holds, not the bar alone. A query below the first bar
/// answers `Bbt::ORIGIN`, and a query above the last bar answers `Bbt::LAST`.
fn clamped_address(anchor: MeterPoint, ticks: Ticks) -> Bbt {
    let limit = if ticks.get() < 0 {
        Bbt::ORIGIN
    } else {
        Bbt::LAST
    };
    let Some(address) = bar_address(anchor, ticks) else {
        return limit;
    };
    let (Ok(bar_value), Ok(beat_value), Ok(tick_value)) = (
        u32::try_from(address.bar),
        u16::try_from(address.beat_offset.saturating_add(1)),
        u16::try_from(address.tick_in_beat),
    ) else {
        return limit;
    };
    let (Some(bar), Some(beat)) = (NonZeroU32::new(bar_value), NonZeroU16::new(beat_value)) else {
        return limit;
    };
    Bbt::new(bar, beat, tick_value)
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
