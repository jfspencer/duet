//! The nightly soak target: the four kernel properties at the soak case count.
//!
//! The four properties are the four properties of `tests/kernel.rs` that take
//! a proptest strategy:
//!
//! 1. The `Finite` invariants.
//! 2. `muldiv` against a 128-bit reference.
//! 3. The 24-bit round trip.
//! 4. The tuplet sum.
//!
//! The gate runs each one at the gate case count of B78. This target runs each
//! one at the soak case count of B78. Every test here carries an `ignore`
//! attribute, so the gate skips them. Section 14 of
//! `roadmap/duet-v1/architecture.md` gives this target to the nightly
//! `soak.yml` workflow, which chunk M7 writes.

#[cfg(test)]
mod tests {
    use core::num::{NonZeroI64, NonZeroU8};

    use duet_time::convert::{Rounding, i24_to_f32, muldiv, unit_to_i24};
    use duet_time::{Finite, I24, Ticks, TimeError, split_tuplet};
    use proptest::prelude::{
        ProptestConfig, Strategy as _, any, prop_assert, prop_assert_eq, prop_assume, proptest,
    };

    /// The proptest case count of the nightly soak run (B78).
    const SOAK_CASES: u32 = 1_000_000;

    /// The proptest configuration of every property in this target.
    fn soak() -> ProptestConfig {
        ProptestConfig {
            cases: SOAK_CASES,
            ..ProptestConfig::default()
        }
    }

    /// The rounding rule of the kernel, computed toward zero and then adjusted.
    ///
    /// It returns `None` when the exact result is outside the 64-bit range.
    fn reference(value: i64, numerator: i64, denominator: i64, rounding: Rounding) -> Option<i64> {
        let product = i128::from(value) * i128::from(numerator);
        let divisor = i128::from(denominator);
        let truncated = product.checked_div(divisor)?;
        let remainder = product.checked_rem(divisor)?;
        let exact = remainder == 0;
        let negative = (product < 0) != (divisor < 0);
        let floor = if exact || !negative {
            truncated
        } else {
            truncated - 1
        };
        let ceiling = if exact || negative {
            truncated
        } else {
            truncated + 1
        };
        let away = if negative { floor } else { ceiling };
        let chosen = match rounding {
            Rounding::Down => floor,
            Rounding::Up => ceiling,
            Rounding::Nearest => {
                let doubled = remainder.abs() * 2;
                if doubled < divisor.abs() {
                    truncated
                } else {
                    away
                }
            },
        };
        i64::try_from(chosen).ok()
    }

    /// Fold an arbitrary integer into the 24-bit sample range.
    fn narrow_to_i24(raw: i32) -> i32 {
        let span = i64::from(I24::MAX.get()) - i64::from(I24::MIN.get()) + 1;
        let folded = i64::from(raw).rem_euclid(span) + i64::from(I24::MIN.get());
        i32::try_from(folded).expect("the folded value fits an i32")
    }

    proptest! {
        #![proptest_config(soak())]

        #[test]
        #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
        fn proptest_large_finite_properties(left in any::<f64>(), right in any::<f64>()) {
            prop_assert_eq!(
                Finite::new(left).is_some(),
                left.is_finite(),
                "Finite::new accepts a value exactly when the value is finite"
            );
            if let (Some(first), Some(second)) = (Finite::new(left), Finite::new(right)) {
                prop_assert_eq!(
                    first == second,
                    (left + 0.0).to_bits() == (right + 0.0).to_bits(),
                    "equality follows the canonical bits"
                );
                prop_assert_eq!(
                    first.cmp(&second),
                    (left + 0.0).total_cmp(&(right + 0.0)),
                    "the order is total_cmp over the canonical value"
                );
            }
        }
    }

    proptest! {
        #![proptest_config(soak())]

        #[test]
        #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
        fn proptest_large_muldiv_matches_i128_reference(
            value in any::<i64>(),
            numerator in any::<i64>(),
            denominator in any::<i64>(),
        ) {
            prop_assume!(denominator != 0);
            let divisor = NonZeroI64::new(denominator).expect("the divisor is not zero");
            for rounding in [Rounding::Down, Rounding::Nearest, Rounding::Up] {
                let got = muldiv(value, numerator, divisor, rounding);
                if let Some(want) = reference(value, numerator, denominator, rounding) {
                    prop_assert_eq!(got, Ok(want), "muldiv matches the 128-bit reference");
                } else {
                    prop_assert_eq!(
                        got,
                        Err(TimeError::Overflow),
                        "muldiv reports an overflow outside the 64-bit range"
                    );
                }
            }
        }
    }

    proptest! {
        #![proptest_config(soak())]

        #[test]
        #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
        fn proptest_large_convert_i24_round_trip(raw in any::<i32>().prop_map(narrow_to_i24)) {
            let sample = I24::new(raw).expect("the folded value is a 24-bit sample");
            let back = unit_to_i24(i24_to_f32(sample));
            let drift = i64::from(back.get()) - i64::from(sample.get());
            prop_assert!(
                drift.abs() <= 1,
                "the 24-bit round trip stays inside one 24-bit step"
            );
            if i64::from(raw).abs() <= 4_194_304 {
                prop_assert_eq!(
                    back.get(),
                    raw,
                    "the 24-bit round trip is exact below a half-scale sample"
                );
            }
        }
    }

    proptest! {
        #![proptest_config(soak())]

        #[test]
        #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
        fn proptest_large_tuplet_parts_sum_to_span(span in 1_i64..=7_680, divisor in 2_u8..=13) {
            let parts = NonZeroU8::new(divisor).expect("the divisor is not zero");
            let split = split_tuplet(Ticks::new(span), parts);
            prop_assert_eq!(
                split.len(),
                usize::from(divisor),
                "the split holds one entry for each part"
            );
            let total: i64 = split.iter().map(|part| part.get()).sum();
            prop_assert_eq!(total, span, "the parts sum to the span exactly");
            let head = split.first().expect("the split holds at least one part");
            for part in split.iter().take(usize::from(divisor).saturating_sub(1)) {
                prop_assert_eq!(part, head, "every part but the last holds the same count");
            }
        }
    }
}
