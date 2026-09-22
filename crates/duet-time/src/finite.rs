//! The finite `f64` newtype that every stored float in the workspace uses.

use core::cmp::Ordering;
use core::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use crate::error::TimeError;

/// A finite `f64` with a canonical bit pattern.
///
/// Two invariants hold, and `new` is the only way in, on every path.
/// 1. The value is never a `NaN` and never an infinity.
/// 2. A negative zero becomes a positive zero.
///
/// `#[serde(try_from = "f64")]` routes deserialization through `TryFrom`, which
/// calls `new`. A hand-edited `NaN` in a canonical file is therefore refused
/// with a serde error, and a hand-edited negative zero becomes the canonical
/// zero. A derived `Deserialize` writes the inner field directly and breaks
/// both invariants.
///
/// The derived `Default` is `ZERO`. A derived `Default` on a one-field struct
/// writes `0.0_f64`, which is finite and which is already the canonical
/// positive zero, so the derive breaks neither invariant.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(try_from = "f64")]
pub struct Finite(f64);

impl Finite {
    /// Zero, which every default uses.
    pub const ZERO: Self = Self(0.0);

    /// Build a finite value, or `None` for a `NaN` or an infinity.
    ///
    /// A negative zero becomes a positive zero.
    #[must_use]
    pub fn new(value: f64) -> Option<Self> {
        value.is_finite().then_some(Self(value + 0.0))
    }

    /// Build a finite value at compile time, for the workspace constants.
    ///
    /// The supported call is the `finite!` macro, which takes a literal alone.
    /// This function is the expansion of that macro and not a call site of its
    /// own, so it carries `#[doc(hidden)]`. It stays `pub`, because the
    /// constants that expand the macro live in other crates.
    ///
    /// `Finite::new` returns an `Option` and `expect` is denied, so a `const`
    /// cannot go through it. This `const fn` asserts instead. In a `const`
    /// item the assertion runs at compile time, so a constant that is not
    /// finite fails the build and not the run. Adding a positive zero
    /// canonicalizes a negative zero and changes no other finite value.
    ///
    /// # Panics
    /// It panics when `value` is a `NaN` or an infinity. The macro binds the
    /// argument to a literal and every caller binds the result to a `const`
    /// item, so the assertion runs at compile time and no binary carries the
    /// panic.
    #[must_use]
    #[doc(hidden)]
    pub const fn from_finite_const(value: f64) -> Self {
        assert!(value.is_finite(), "a `Finite` constant must be finite");
        Self(value + 0.0)
    }

    /// The inner value. It is finite, and it is never a negative zero.
    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }
}

impl TryFrom<f64> for Finite {
    type Error = TimeError;

    /// Build a finite value, or refuse a `NaN` and an infinity.
    ///
    /// # Errors
    /// Returns `TimeError::NotFinite` when `value` is a `NaN` or an infinity.
    fn try_from(value: f64) -> Result<Self, TimeError> {
        Self::new(value).ok_or(TimeError::NotFinite)
    }
}

impl PartialEq for Finite {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for Finite {}

impl Hash for Finite {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl Ord for Finite {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl PartialOrd for Finite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
