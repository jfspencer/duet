//! The bar, beat, and tick address.

use core::cmp::Ordering;
use core::num::{NonZeroU16, NonZeroU32};

use serde::{Deserialize, Serialize};

/// A bar, beat, and tick address. Bars and beats are one-based.
///
/// It derives no `Ord`, because a bar marker can reset the address. `Ticks` is
/// the monotonic value and the only sort key on the timeline. A tempo map holds
/// a list whose addresses never reset, so the map orders them with
/// `lexicographic_cmp` and the type still carries no derive.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bbt {
    /// The one-based bar number.
    bar: NonZeroU32,
    /// The one-based beat inside the bar.
    beat: NonZeroU16,
    /// The tick inside the beat.
    tick: u16,
}

impl Bbt {
    /// Bar 1, beat 1, tick 0. It is the address of timeline zero.
    pub const ORIGIN: Self = Self {
        bar: NonZeroU32::MIN,
        beat: NonZeroU16::MIN,
        tick: 0,
    };

    /// The last bar an address can name, at its first beat and its first tick.
    ///
    /// `TempoMap::bbt_at` holds its answer here at the top of the range.
    pub const LAST: Self = Self {
        bar: NonZeroU32::MAX,
        beat: NonZeroU16::MIN,
        tick: 0,
    };

    /// The address at `bar`, `beat`, and `tick`.
    ///
    /// The type cannot tell whether beat 5 or tick 2000 is in range, because
    /// the range comes from a meter that the type does not hold.
    /// `TempoMap::ticks_at_bbt` holds the meter and makes that check.
    #[must_use]
    pub const fn new(bar: NonZeroU32, beat: NonZeroU16, tick: u16) -> Self {
        Self { bar, beat, tick }
    }

    /// The one-based bar number.
    #[must_use]
    pub const fn bar(self) -> NonZeroU32 {
        self.bar
    }

    /// The one-based beat inside the bar.
    #[must_use]
    pub const fn beat(self) -> NonZeroU16 {
        self.beat
    }

    /// The tick inside the beat.
    #[must_use]
    pub const fn tick(self) -> u16 {
        self.tick
    }

    /// The order of two addresses, by bar, then beat, then tick.
    #[must_use]
    pub fn lexicographic_cmp(self, other: Self) -> Ordering {
        (self.bar.get(), self.beat.get(), self.tick).cmp(&(
            other.bar.get(),
            other.beat.get(),
            other.tick,
        ))
    }
}
