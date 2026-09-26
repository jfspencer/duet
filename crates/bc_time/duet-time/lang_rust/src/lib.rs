//! The Duet time kernel.
//!
//! It declares the beat domain and the audio domain, the position and span
//! types, the tempo map, and the one module of the workspace that holds an
//! `as` cast. Section 2 of `roadmap/duet-v1/architecture.md` states the
//! design.
//!
//! # The decimal path for a stored number
//!
//! No consumer writes an `as` cast: the lint policy denies one outside
//! `convert`, and `cargo xtask check-conversions` refuses one in every other
//! file. Three total paths cover every reader.
//!
//! 1. A tempo readout takes the two sides of a [`Ratio`] apart:
//!    `i32::try_from(rate.numerator())` and then `f64::from`, and the same
//!    pair for `rate.denominator().get()`. Each step is checked, and neither
//!    needs a suppression.
//! 2. A tick count takes [`convert::ticks_to_f64`], which refuses at the
//!    magnitude where an `f64` stops holding every integer.
//! 3. A stored float is already a [`Finite`], so [`Finite::get`] answers an
//!    `f64` and [`convert::finite_to_f32_saturating`] narrows it.

#![forbid(unsafe_code)]

pub mod convert;

mod bbt;
mod error;
mod finite;
mod position;
mod span;
mod tempo;
mod tuplet;
mod units;

/// A [`Finite`] constant from a float literal.
///
/// It is the supported way to write a `Finite` constant. The macro takes a
/// literal alone, so the assertion inside
/// [`Finite::from_finite_const`] runs at compile time and no binary carries
/// the panic. A run-time value has no literal form, so it cannot reach the
/// assertion through this macro.
///
/// # Examples
/// ```
/// use duet_time::{Finite, finite};
///
/// const MIX_SPLIT: Finite = finite!(0.45);
/// assert!(
///     MIX_SPLIT > Finite::ZERO,
///     "the macro builds the constant at compile time"
/// );
/// ```
#[macro_export]
macro_rules! finite {
    ($value:literal) => {
        $crate::Finite::from_finite_const($value)
    };
}

pub use bbt::Bbt;
pub use error::TimeError;
pub use finite::Finite;
pub use position::{Delta, Position, TimeDomain};
pub use span::Span;
pub use tempo::{Meter, MeterPoint, NoteValue, Ratio, Tempo, TempoMap, TempoMapEdit, TempoPoint};
pub use tuplet::{Tuplet, split_tuplet};
pub use units::{
    BarCount, ChannelCount, ChannelIndex, FrameCount, GainDb, I24, MAX_BOUND_PORTS, MAX_PARAMS,
    MAX_SENDS, MAX_SLOT_METERS, MAX_SLOTS, MAX_STRIPS, SUPERCLOCK_HZ, SampleClock, SampleRate,
    SchemaVersion, SuperClock, TICKS_PER_QUARTER, Ticks, UnixSeconds,
};
