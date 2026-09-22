//! The tuplet type and the rounding rule that splits a span into its parts.

use core::num::NonZeroU8;

use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::units::Ticks;

/// A tuplet: `count` notes written in the time of `over` of the same value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tuplet {
    /// How many notes the tuplet writes.
    count: NonZeroU8,
    /// How many notes of the same value the tuplet occupies.
    over: NonZeroU8,
}

impl Tuplet {
    /// A tuplet of `count` notes in the time of `over` notes.
    #[must_use]
    pub const fn new(count: NonZeroU8, over: NonZeroU8) -> Self {
        Self { count, over }
    }

    /// How many notes the tuplet writes.
    #[must_use]
    pub const fn count(self) -> NonZeroU8 {
        self.count
    }

    /// How many notes of the same value the tuplet occupies.
    #[must_use]
    pub const fn over(self) -> NonZeroU8 {
        self.over
    }

    /// The tick count of each note of this tuplet inside `span`.
    ///
    /// It is `split_tuplet(span, self.count())`, so the two answers can never
    /// differ. The rounding rule reads `count` alone: `over` names the written
    /// value of the group and moves no tick.
    #[must_use]
    pub fn split(self, span: Ticks) -> SmallVec<[Ticks; 13]> {
        split_tuplet(span, self.count)
    }
}

/// Split a span into `parts` tick counts that sum to the span exactly.
///
/// The last part absorbs the remainder. Architecture section 2.4 states the
/// rule. Each of the first `parts` minus one entries takes
/// `span.div_euclid(parts)` ticks. The last entry takes what is left.
///
/// The tick resolution B40 has no factor of seven, of eleven, or of thirteen.
/// A tuplet of one of those counts is not exact, and the rule decides where
/// the leftover ticks go.
///
/// `div_euclid` floors, so a negative span gives a quotient below the exact
/// value and the last part can carry the opposite sign.
/// `split_tuplet(Ticks::new(-1), 255)` answers 254 parts of -1 tick and one
/// last part of 253 ticks. The parts still sum to the span exactly.
///
/// The inline capacity is B113, the divisor maximum of the range B43. A tuplet
/// of that range needs no heap block. A count of 14 to 255 is above that range,
/// and the answer then takes a heap block.
#[must_use]
pub fn split_tuplet(span: Ticks, parts: NonZeroU8) -> SmallVec<[Ticks; 13]> {
    let count = parts.get();
    let each = Ticks::new(span.get().div_euclid(i64::from(count)));
    let mut split = SmallVec::with_capacity(usize::from(count));
    let mut consumed = Ticks::ZERO;
    for _ in 1..count {
        split.push(each);
        consumed = consumed.saturating_add(each);
    }
    split.push(span.saturating_sub(consumed));
    split
}
