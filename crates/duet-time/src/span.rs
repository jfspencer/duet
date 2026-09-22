//! The span type: a distance together with the position where it starts.

use serde::{Deserialize, Serialize};

use crate::position::{Delta, Position};
use crate::tempo::TempoMap;
use crate::units::{SuperClock, Ticks};

/// A distance together with the position where it starts.
///
/// A duration is not a scalar. Three beats has no length in samples until the
/// origin is known, because a tempo change can sit inside it. Both ends of the
/// distance therefore convert through the map in the same way.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span {
    /// The position the distance starts at.
    origin: Position,
    /// The signed distance from the origin.
    delta: Delta,
}

impl Span {
    /// A distance of `delta` that starts at `origin`.
    ///
    /// It stores both values and normalizes neither, because a normalization
    /// needs a map that this signature does not carry.
    #[must_use]
    pub const fn new(origin: Position, delta: Delta) -> Self {
        Self { origin, delta }
    }

    /// The position the distance starts at.
    #[must_use]
    pub const fn origin(self) -> Position {
        self.origin
    }

    /// The position at the end of the distance.
    #[must_use]
    pub fn end(self, map: &TempoMap) -> Position {
        self.origin.later(self, map)
    }

    /// The distance as a tick count, measured through its own origin.
    #[must_use]
    pub fn in_beats(self, map: &TempoMap) -> Ticks {
        match self.delta {
            Delta::Beats(ticks) => ticks,
            Delta::Audio(clock) => {
                let start = self.origin.in_audio(map);
                let finish = start.saturating_add(clock);
                map.ticks_at(finish).saturating_sub(map.ticks_at(start))
            },
        }
    }

    /// The distance as a superclock count, measured through its own origin.
    #[must_use]
    pub fn in_audio(self, map: &TempoMap) -> SuperClock {
        match self.delta {
            Delta::Beats(ticks) => {
                let start = self.origin.in_beats(map);
                let finish = start.saturating_add(ticks);
                map.superclock_at(finish)
                    .saturating_sub(map.superclock_at(start))
            },
            Delta::Audio(clock) => clock,
        }
    }
}
