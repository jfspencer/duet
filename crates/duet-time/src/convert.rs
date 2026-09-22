//! The one module of the workspace that holds an `as` cast.
//!
//! Two rules keep each suppression verifiable at its own site. A function whose
//! correctness depends on the caller takes a checked newtype and never a raw
//! float. A function whose input range is unbounded returns `Result`, and it
//! checks the range before the cast.

use core::num::NonZeroI64;

use crate::error::TimeError;
use crate::finite::Finite;
use crate::units::{I24, SUPERCLOCK_HZ, SampleRate, SuperClock, Ticks};

/// The scale that maps a 16-bit sample into the unit range.
const I16_SCALE: f32 = 32_768.0;

/// The scale that maps a 24-bit sample into the unit range.
const I24_SCALE: f32 = 8_388_608.0;

/// The scale that maps a 32-bit sample into the unit range.
///
/// It is two to the power of 31, written as a product because each factor
/// prints exactly and the whole value does not.
const I32_SCALE: f32 = 32_768.0 * 65_536.0;

/// The scale that maps a unit sample onto the 24-bit range.
const UNIT_TO_I24_SCALE: f64 = 8_388_607.0;

/// The scale that maps a unit sample onto the 32-bit range.
const UNIT_TO_I32_SCALE: f64 = 2_147_483_647.0;

/// The magnitude at which an `f64` stops holding every integer (B44).
const EXACT_F64_INTEGER_BOUND: i64 = 9_007_199_254_740_992;

/// A sample value in the closed range -1.0 to 1.0.
///
/// Only `new` and `clamped` build one, so every consumer knows the range.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Unit(f32);

impl Unit {
    /// One sample, or `None` outside the unit range.
    ///
    /// A `NaN` and an infinity are both outside the range, so both answer
    /// `None`.
    #[must_use]
    pub fn new(value: f32) -> Option<Self> {
        (-1.0..=1.0).contains(&value).then_some(Self(value))
    }

    /// Clamp a float into the unit range. A non-finite value becomes zero.
    #[must_use]
    pub const fn clamped(value: f32) -> Self {
        if value.is_finite() {
            Self(value.clamp(-1.0, 1.0))
        } else {
            Self(0.0)
        }
    }

    /// The inner value, always in the closed range -1.0 to 1.0.
    #[must_use]
    pub const fn get(self) -> f32 {
        self.0
    }
}

/// How an integer conversion resolves a remainder.
///
/// `Down` rounds toward negative infinity. `Up` rounds toward positive
/// infinity. `Nearest` rounds to the nearest integer and resolves a tie away
/// from zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
    /// Round toward negative infinity.
    Down,
    /// Round to the nearest integer, and resolve a tie away from zero.
    Nearest,
    /// Round toward positive infinity.
    Up,
}

/// Multiply and then divide with a 128-bit intermediate.
///
/// # Errors
/// Returns `TimeError::Overflow` when the result leaves the `i64` range.
pub fn muldiv(
    value: i64,
    numerator: i64,
    denominator: NonZeroI64,
    rounding: Rounding,
) -> Result<i64, TimeError> {
    let product = i128::from(value) * i128::from(numerator);
    let divisor = i128::from(denominator.get());
    let (dividend, magnitude) = if divisor < 0 {
        (-product, -divisor)
    } else {
        (product, divisor)
    };
    let quotient = dividend.div_euclid(magnitude);
    let remainder = dividend.rem_euclid(magnitude);
    let step = match rounding {
        Rounding::Down => 0,
        Rounding::Up => i128::from(remainder > 0),
        Rounding::Nearest => {
            let doubled = remainder * 2;
            i128::from(doubled > magnitude || (doubled == magnitude && dividend > 0))
        },
    };
    i64::try_from(quotient + step)
        .ok()
        .ok_or(TimeError::Overflow)
}

/// A 16-bit sample as a float. Lossless; it uses `f32::from`, no suppression.
#[must_use]
pub fn i16_to_f32(sample: i16) -> Unit {
    Unit::clamped(f32::from(sample) / I16_SCALE)
}

/// A 24-bit sample as a float. Lossless; 24 bits fit the `f32` mantissa.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "a 24-bit integer fits the f32 mantissa exactly, so the conversion is lossless and the result is in the unit range"
)]
pub fn i24_to_f32(sample: I24) -> Unit {
    Unit::clamped(sample.get() as f32 / I24_SCALE)
}

/// A 32-bit sample as a float. Lossy below the 24th bit.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "the loss is below the 24th bit, and the divisor makes the result fall in the unit range; the round-trip proptest states the bound"
)]
pub fn i32_to_f32(sample: i32) -> Unit {
    Unit::clamped(sample as f32 / I32_SCALE)
}

/// A unit sample as a 24-bit integer. The type carries the range.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "the `Unit` type carries the range -1.0 to 1.0 and the scale factor is 2^23 - 1, which is `I24::MAX`, so the product lies in -8_388_607.0 to 8_388_607.0 and fits `I24` with no clamp"
)]
pub fn unit_to_i24(value: Unit) -> I24 {
    let scaled = (f64::from(value.get()) * UNIT_TO_I24_SCALE).round() as i32;
    I24::new(scaled).unwrap_or(I24::MAX)
}

/// A unit sample as a 32-bit integer. The type carries the range.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "the `Unit` type carries the range -1.0 to 1.0 and the scale factor is 2^31 - 1, which is `i32::MAX`, so the product lies in -2_147_483_647.0 to 2_147_483_647.0 and fits `i32` with no clamp"
)]
pub fn unit_to_i32(value: Unit) -> i32 {
    (f64::from(value.get()) * UNIT_TO_I32_SCALE).round() as i32
}

/// A double as a single.
///
/// # Errors
/// Returns `TimeError::NotRepresentable` when the value is not finite, or when
/// its magnitude leaves the `f32` range. The check runs before the cast.
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "the function returns an error for a non-finite value and for a magnitude outside the f32 range before it reaches this line"
)]
pub fn f64_to_f32(value: f64) -> Result<f32, TimeError> {
    if !value.is_finite() || value.abs() > f64::from(f32::MAX) {
        return Err(TimeError::NotRepresentable);
    }
    Ok(value as f32)
}

/// A tick count as a double.
///
/// # Errors
/// Returns `TimeError::NotRepresentable` at or above the magnitude B44, where
/// an `f64` stops holding every integer.
#[expect(
    clippy::as_conversions,
    clippy::cast_precision_loss,
    reason = "the function returns an error at or above 2^53 ticks before it reaches this line, so every integer below that bound converts to an f64 exactly"
)]
pub const fn ticks_to_f64(ticks: Ticks) -> Result<f64, TimeError> {
    let raw = ticks.get();
    if raw >= EXACT_F64_INTEGER_BOUND || raw <= -EXACT_F64_INTEGER_BOUND {
        return Err(TimeError::NotRepresentable);
    }
    Ok(raw as f64)
}

/// A superclock count as a sample count at a rate.
///
/// # Errors
/// Returns `TimeError::Overflow` when the result leaves the `i64` range.
pub fn superclock_to_samples(
    clock: SuperClock,
    rate: SampleRate,
    rounding: Rounding,
) -> Result<i64, TimeError> {
    muldiv(
        clock.get(),
        i64::from(rate.get().get()),
        SUPERCLOCK_HZ,
        rounding,
    )
}

/// A sample count at a rate as a superclock count.
///
/// The superclock rate divides exactly by every supported sample rate, so the
/// division leaves no remainder and the rounding mode never changes the answer.
///
/// # Errors
/// Returns `TimeError::Overflow` when the result leaves the `i64` range.
pub fn samples_to_superclock(samples: i64, rate: SampleRate) -> Result<SuperClock, TimeError> {
    let clock = muldiv(
        samples,
        SUPERCLOCK_HZ.get(),
        NonZeroI64::from(rate.get()),
        Rounding::Nearest,
    )?;
    Ok(SuperClock::new(clock))
}

/// A finite double as a single, with no error path.
///
/// It returns no `Result`, and that is the whole reason it exists. The caller
/// paints a window length in logical pixels and has no error surface to return
/// one on. A `Finite` is never a `NaN` and never an infinity, so the only loss
/// the narrowing can carry is a magnitude above the `f32` range, which
/// saturates to an `f32` infinity. A logical pixel length is far below that
/// magnitude, so the case is unreachable and the saturation is the documented
/// behaviour. `f64_to_f32` stays the fallible form for every caller that can
/// report.
#[must_use]
#[expect(
    clippy::as_conversions,
    clippy::cast_possible_truncation,
    reason = "the input is a `Finite`, so it is never a NaN and never an infinity; the narrowing saturates to an f32 infinity only above the f32 range, and `LogicalPx` carries a window length in logical pixels, which is far below it"
)]
pub const fn finite_to_f32_saturating(value: Finite) -> f32 {
    value.get() as f32
}
