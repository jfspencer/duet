//! Every refusal the time kernel can answer with.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Every way the time kernel refuses.
///
/// It derives serde, because `ScoreError::Time` and `SessionError::Time` wrap
/// it and both reach the transport inside `GatewayError`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum TimeError {
    /// A result is outside the 64-bit range.
    #[error("the result is outside the 64-bit range")]
    Overflow,
    /// A value has no exact form in the target type.
    #[error("the value has no exact form in the target type")]
    NotRepresentable,
    /// A float is a `NaN` or an infinity.
    #[error("the value is not a finite number")]
    NotFinite,
    /// A point list is not in tick order.
    #[error("the point list is not in tick order")]
    UnorderedMap,
    /// A point list has no point at tick zero.
    #[error("the point list has no point at tick zero")]
    NoFirstPoint,
    /// A bar, beat, and tick address is outside the tempo map.
    #[error("the bar, beat, and tick address is outside the tempo map")]
    BbtOutOfRange,
}
