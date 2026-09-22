//! The unit newtypes of the time kernel, and the limits every crate reads.

use core::num::{NonZeroI64, NonZeroU16, NonZeroU32};

use serde::{Deserialize, Serialize};

use crate::error::TimeError;
use crate::finite::Finite;

/// The lowest sample value an `I24` holds.
const MIN_SAMPLE: i32 = -8_388_608;

/// The highest sample value an `I24` holds.
const MAX_SAMPLE: i32 = 8_388_607;

/// A non-zero constant, built at compile time.
///
/// `NonZeroI64::new` returns an `Option`, and `expect` is denied. A `const fn`
/// match gives the same compile-time check with a total fallback arm.
const fn non_zero(value: i64) -> NonZeroI64 {
    match NonZeroI64::new(value) {
        Some(checked) => checked,
        None => NonZeroI64::MIN,
    }
}

/// Ticks per quarter note in the beat domain (B40).
pub const TICKS_PER_QUARTER: NonZeroI64 = non_zero(1_920);

/// Ticks per second in the audio domain (B41).
pub const SUPERCLOCK_HZ: NonZeroI64 = non_zero(282_240_000);

/// Strips one project may hold, and the meter array length (B86).
pub const MAX_STRIPS: usize = 48;

/// Automatable parameters one project holds (B90).
pub const MAX_PARAMS: usize = 2_048;

/// Pre-fader and post-fader slots per chain (B45).
pub const MAX_SLOTS: usize = 8;

/// Aux sends per chain (B46).
pub const MAX_SENDS: usize = 8;

/// Per-slot meter cells one slot meter snapshot carries (B133).
///
/// It is arithmetic over three constants and never a literal, so a change to
/// any one of them moves it.
pub const MAX_SLOT_METERS: usize = MAX_STRIPS * 2 * MAX_SLOTS;

/// The MIDI input ports one engine binds at once (B139).
pub const MAX_BOUND_PORTS: usize = 16;

/// A magnitude in the beat domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ticks(i64);

impl Ticks {
    /// A magnitude of zero ticks.
    pub const ZERO: Self = Self(0);

    /// A magnitude of `value` ticks.
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// The magnitude as a plain integer.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    /// The sum of two magnitudes, held at the 64-bit bound.
    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// The difference of two magnitudes, held at the 64-bit bound.
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// A magnitude from an imported or parsed integer.
    ///
    /// # Errors
    /// Returns `TimeError::Overflow` for the two 64-bit bounds. `saturating_add`
    /// and `saturating_sub` answer with a bound when they saturate, so an
    /// imported magnitude at a bound cannot be told apart from a saturated one.
    pub const fn checked_from_i64(value: i64) -> Result<Self, TimeError> {
        match value {
            i64::MIN | i64::MAX => Err(TimeError::Overflow),
            magnitude => Ok(Self(magnitude)),
        }
    }
}

/// A magnitude in the audio domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SuperClock(i64);

impl SuperClock {
    /// A magnitude of zero superclock ticks.
    pub const ZERO: Self = Self(0);

    /// A magnitude of `value` superclock ticks.
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// The magnitude as a plain integer.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }

    /// The sum of two magnitudes, held at the 64-bit bound.
    #[must_use]
    pub const fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    /// The difference of two magnitudes, held at the 64-bit bound.
    #[must_use]
    pub const fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    /// A magnitude from an imported or parsed integer.
    ///
    /// # Errors
    /// Returns `TimeError::Overflow` for the two 64-bit bounds, for the reason
    /// `Ticks::checked_from_i64` states.
    pub const fn checked_from_i64(value: i64) -> Result<Self, TimeError> {
        match value {
            i64::MIN | i64::MAX => Err(TimeError::Overflow),
            magnitude => Ok(Self(magnitude)),
        }
    }
}

/// A device sample rate. The type makes a zero divisor impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleRate(NonZeroU32);

impl SampleRate {
    /// A rate of `value` samples per second.
    #[must_use]
    pub const fn new(value: NonZeroU32) -> Self {
        Self(value)
    }

    /// The rate as a non-zero number of samples per second.
    #[must_use]
    pub const fn get(self) -> NonZeroU32 {
        self.0
    }
}

/// A sample counter at the device edge, in frames since the stream opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleClock(i64);

impl SampleClock {
    /// A counter at `value` frames.
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// The counter as a plain integer.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// A count of audio frames.
///
/// It derives `Default`, because zero frames is a count and not an absence.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub struct FrameCount(u32);

impl FrameCount {
    /// A count of `value` frames.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// The count as a plain integer.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A count of bars, which the count-in uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BarCount(u16);

impl BarCount {
    /// A count of `value` bars.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// The count as a plain integer.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// A channel index inside one stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelIndex(u16);

impl ChannelIndex {
    /// The channel at position `value`.
    #[must_use]
    pub const fn new(value: u16) -> Self {
        Self(value)
    }

    /// The position as a plain integer.
    #[must_use]
    pub const fn get(self) -> u16 {
        self.0
    }
}

/// A count of channels that cannot be zero.
///
/// It exists so that `chunks_exact_mut` cannot panic. That method panics on a
/// chunk size of zero, and `panic = "abort"` makes such a panic a lost take, so
/// the audio path takes the count from a type that has no zero. `new` is the
/// only way in, and it answers `None` on a zero, so every caller decides once
/// and off the audio thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelCount(NonZeroU16);

impl ChannelCount {
    /// One count, or `None` when `value` is zero.
    #[must_use]
    pub const fn new(value: u16) -> Option<Self> {
        match NonZeroU16::new(value) {
            Some(count) => Some(Self(count)),
            None => None,
        }
    }

    /// The count as a non-zero number.
    ///
    /// A chunk size is a `usize`, so a caller writes
    /// `usize::from(count.get().get())`, which is total and never zero.
    #[must_use]
    pub const fn get(self) -> NonZeroU16 {
        self.0
    }
}

/// Seconds since the Unix epoch. A calibration records when it was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UnixSeconds(i64);

impl UnixSeconds {
    /// The instant `value` seconds after the Unix epoch.
    #[must_use]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// The instant as a plain integer.
    #[must_use]
    pub const fn get(self) -> i64 {
        self.0
    }
}

/// A document schema number.
///
/// It derives `Ord`, because a reader migrates an older document and refuses a
/// newer one, and neither decision can be written without an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SchemaVersion(u32);

impl SchemaVersion {
    /// Build a version at compile time, for the workspace constants.
    #[must_use]
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    /// The version as a plain integer.
    #[must_use]
    pub const fn get(self) -> u32 {
        self.0
    }
}

/// A gain in decibels. Every stored gain on a region and on a strip is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct GainDb(Finite);

impl GainDb {
    /// A gain of `value` decibels.
    #[must_use]
    pub const fn new(value: Finite) -> Self {
        Self(value)
    }

    /// The gain as a finite number of decibels.
    #[must_use]
    pub const fn get(self) -> Finite {
        self.0
    }
}

/// A 24-bit signed sample, held in the low three bytes of an `i32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Serialize, Deserialize)]
pub struct I24(i32);

impl I24 {
    /// The highest 24-bit sample.
    pub const MAX: Self = Self(MAX_SAMPLE);

    /// The lowest 24-bit sample.
    pub const MIN: Self = Self(MIN_SAMPLE);

    /// One sample, or `None` when `value` is outside the 24-bit range.
    #[must_use]
    pub const fn new(value: i32) -> Option<Self> {
        match value {
            MIN_SAMPLE..=MAX_SAMPLE => Some(Self(value)),
            _ => None,
        }
    }

    /// The sample as a plain integer, in the 24-bit range.
    #[must_use]
    pub const fn get(self) -> i32 {
        self.0
    }
}
