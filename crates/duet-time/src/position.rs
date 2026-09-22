//! The position type, the domain tag, and the signed delta.

use serde::{Deserialize, Serialize};

use crate::span::Span;
use crate::tempo::TempoMap;
use crate::units::{SuperClock, Ticks};

/// Which unit a value counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeDomain {
    /// The beat domain, counted in ticks.
    Beats,
    /// The audio domain, counted in superclock ticks.
    Audio,
}

/// A point on the timeline, measured from timeline zero.
///
/// Equality is structural: the same domain and the same value. Two positions
/// in different domains are never equal, which is the correct answer for a
/// cache key and for a dirty check.
///
/// This type implements no `PartialOrd` and no `Ord`. Order is a tempo-map
/// query, because two positions in different domains relate only through the
/// map. Code sorts with `slice.sort_by(|a, b| map.cmp(*a, *b))`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Position {
    /// A point in the beat domain.
    Beats(Ticks),
    /// A point in the audio domain.
    Audio(SuperClock),
}

/// A signed magnitude in one domain. Equality is structural, as above.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Delta {
    /// A magnitude in the beat domain.
    Beats(Ticks),
    /// A magnitude in the audio domain.
    Audio(SuperClock),
}

impl Position {
    /// The domain this position counts in.
    #[must_use]
    pub const fn domain(self) -> TimeDomain {
        match self {
            Self::Beats(_) => TimeDomain::Beats,
            Self::Audio(_) => TimeDomain::Audio,
        }
    }

    /// The position as a tick count.
    #[must_use]
    pub fn in_beats(self, map: &TempoMap) -> Ticks {
        match self {
            Self::Beats(ticks) => ticks,
            Self::Audio(clock) => map.ticks_at(clock),
        }
    }

    /// The position as a superclock count.
    #[must_use]
    pub fn in_audio(self, map: &TempoMap) -> SuperClock {
        match self {
            Self::Beats(ticks) => map.superclock_at(ticks),
            Self::Audio(clock) => clock,
        }
    }

    /// The span from this position to `other`.
    ///
    /// The origin of the span is this position, and the delta counts in this
    /// position's own domain. The delta is signed, so an `other` before this
    /// position gives a negative delta.
    #[must_use]
    pub fn distance(self, other: Self, map: &TempoMap) -> Span {
        let delta = match self {
            Self::Beats(ticks) => Delta::Beats(other.in_beats(map).saturating_sub(ticks)),
            Self::Audio(clock) => Delta::Audio(other.in_audio(map).saturating_sub(clock)),
        };
        Span::new(self, delta)
    }

    /// The position `span` later, in this position's own domain.
    ///
    /// The span length is signed, so a negative span moves earlier.
    #[must_use]
    pub fn later(self, span: Span, map: &TempoMap) -> Self {
        match self {
            Self::Beats(ticks) => Self::Beats(ticks.saturating_add(span.in_beats(map))),
            Self::Audio(clock) => Self::Audio(clock.saturating_add(span.in_audio(map))),
        }
    }

    /// The position `span` earlier, in this position's own domain.
    ///
    /// The span length is signed, so a negative span moves later.
    #[must_use]
    pub fn earlier(self, span: Span, map: &TempoMap) -> Self {
        match self {
            Self::Beats(ticks) => Self::Beats(ticks.saturating_sub(span.in_beats(map))),
            Self::Audio(clock) => Self::Audio(clock.saturating_sub(span.in_audio(map))),
        }
    }
}
