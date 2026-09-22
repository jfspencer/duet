//! The Duet time kernel.
//!
//! It declares the beat domain and the audio domain, the position and span
//! types, the tempo map, and the one module of the workspace that holds an
//! `as` cast. Section 2 of `roadmap/duet-v1/architecture.md` states the
//! design.

#![forbid(unsafe_code)]

pub mod convert;

mod error;
mod finite;
mod units;

pub use error::TimeError;
pub use finite::Finite;
pub use units::{
    BarCount, ChannelCount, ChannelIndex, FrameCount, GainDb, I24, MAX_BOUND_PORTS, MAX_PARAMS,
    MAX_SENDS, MAX_SLOT_METERS, MAX_SLOTS, MAX_STRIPS, SUPERCLOCK_HZ, SampleClock, SampleRate,
    SchemaVersion, SuperClock, TICKS_PER_QUARTER, Ticks, UnixSeconds,
};
