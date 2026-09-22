//! The kernel contract: the `Finite` invariants, the conversion round trips,
//! and the workspace constants.

#[cfg(test)]
mod tests {
    use core::num::NonZeroI64;
    use std::hash::{BuildHasher as _, RandomState};

    use core::num::NonZeroU32;

    use duet_time::convert::{
        Rounding, f64_to_f32, i16_to_f32, i24_to_f32, i32_to_f32, muldiv, samples_to_superclock,
        superclock_to_samples, ticks_to_f64, unit_to_i24, unit_to_i32,
    };
    use duet_time::{Finite, I24, SUPERCLOCK_HZ, SampleRate, TICKS_PER_QUARTER, Ticks, TimeError};
    use proptest::prelude::{
        Strategy as _, any, prop_assert, prop_assert_eq, prop_assume, proptest,
    };

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

    proptest! {
        #[test]
        fn finite_rejects_non_finite(raw in any::<f64>()) {
            prop_assert_eq!(
                Finite::new(raw).is_some(),
                raw.is_finite(),
                "Finite::new accepts a value exactly when the value is finite"
            );
            prop_assert_eq!(Finite::new(f64::NAN).is_none(), true, "a NaN is refused");
            prop_assert_eq!(
                Finite::new(f64::INFINITY).is_none(),
                true,
                "a positive infinity is refused"
            );
            prop_assert_eq!(
                Finite::new(f64::NEG_INFINITY).is_none(),
                true,
                "a negative infinity is refused"
            );
        }
    }

    #[test]
    fn finite_canonicalizes_negative_zero() {
        let negative = Finite::new(-0.0).expect("a negative zero is finite");
        let positive = Finite::new(0.0).expect("a positive zero is finite");
        assert_eq!(negative, positive, "a negative zero equals a positive zero");
        let hasher = RandomState::new();
        assert_eq!(
            hasher.hash_one(negative),
            hasher.hash_one(positive),
            "a negative zero hashes as a positive zero"
        );
        assert!(
            negative.get().is_sign_positive(),
            "the stored zero carries a positive sign"
        );
    }

    proptest! {
        #[test]
        fn finite_equality_matches_bits(left in any::<f64>(), right in any::<f64>()) {
            prop_assume!(left.is_finite() && right.is_finite());
            let first = Finite::new(left).expect("the left input is finite");
            let second = Finite::new(right).expect("the right input is finite");
            prop_assert_eq!(
                first == second,
                (left + 0.0).to_bits() == (right + 0.0).to_bits(),
                "equality follows the canonical bits"
            );
        }
    }

    proptest! {
        #[test]
        fn finite_order_matches_total_cmp(left in any::<f64>(), right in any::<f64>()) {
            prop_assume!(left.is_finite() && right.is_finite());
            let first = Finite::new(left).expect("the left input is finite");
            let second = Finite::new(right).expect("the right input is finite");
            prop_assert_eq!(
                first.cmp(&second),
                (left + 0.0).total_cmp(&(right + 0.0)),
                "the order is total_cmp over the canonical value"
            );
        }
    }

    #[test]
    fn finite_json_round_trip() {
        for raw in [0.0_f64, -0.0, 1.5, -2.5, f64::MAX, f64::MIN] {
            let value = Finite::new(raw).expect("the sample is finite");
            let text = serde_json::to_string(&value).expect("the value serializes");
            let back: Finite = serde_json::from_str(&text).expect("the text deserializes");
            assert_eq!(
                back, value,
                "a JSON round trip returns an equal value for {raw}"
            );
        }
        assert!(
            serde_json::from_str::<Finite>("NaN").is_err(),
            "a NaN token is refused"
        );
        assert!(
            serde_json::from_str::<Finite>("null").is_err(),
            "a null is refused"
        );
        let canonical: Finite = serde_json::from_str("-0.0").expect("a negative zero deserializes");
        assert_eq!(
            canonical,
            Finite::ZERO,
            "a negative zero deserializes to the canonical zero"
        );
    }

    proptest! {
        #[test]
        fn muldiv_matches_i128_reference(
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

    /// Fold an arbitrary integer into the 24-bit sample range.
    fn narrow_to_i24(raw: i32) -> i32 {
        let span = i64::from(I24::MAX.get()) - i64::from(I24::MIN.get()) + 1;
        let folded = i64::from(raw).rem_euclid(span) + i64::from(I24::MIN.get());
        i32::try_from(folded).expect("the folded value fits an i32")
    }

    #[test]
    fn convert_i16_round_trip() {
        for sample in i16::MIN..=i16::MAX {
            let scaled = i64::from(unit_to_i32(i16_to_f32(sample)));
            let want = i64::from(sample) * 65_536;
            assert!(
                (scaled - want).abs() <= 1,
                "the 16-bit round trip stays inside one 32-bit step for {sample}"
            );
        }
    }

    proptest! {
        #[test]
        fn convert_i24_round_trip(raw in any::<i32>().prop_map(narrow_to_i24)) {
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

    #[test]
    fn convert_i32_round_trip() {
        for sample in [i32::MIN, -1, 0, 1, i32::MAX] {
            let back = i64::from(unit_to_i32(i32_to_f32(sample)));
            let drift = back - i64::from(sample);
            assert!(
                drift.abs() <= 256,
                "the 32-bit round trip stays below the 24th bit for {sample}"
            );
        }
    }

    #[test]
    fn convert_f64_out_of_range_errors() {
        for raw in [
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::MAX,
            f64::MIN,
        ] {
            assert_eq!(
                f64_to_f32(raw).err(),
                Some(TimeError::NotRepresentable),
                "f64_to_f32 refuses {raw}"
            );
        }
        assert!(
            f64_to_f32(1.5).is_ok(),
            "f64_to_f32 accepts a value inside the f32 range"
        );
        for raw in [
            9_007_199_254_740_992_i64,
            -9_007_199_254_740_992,
            i64::MAX,
            i64::MIN,
        ] {
            assert_eq!(
                ticks_to_f64(Ticks::new(raw)).err(),
                Some(TimeError::NotRepresentable),
                "ticks_to_f64 refuses {raw}"
            );
        }
        assert!(
            ticks_to_f64(Ticks::new(9_007_199_254_740_991)).is_ok(),
            "ticks_to_f64 accepts the largest exact tick count"
        );
    }

    #[test]
    fn clock_round_trip_every_rate() {
        for hz in [44_100_u32, 48_000, 88_200, 96_000, 176_400, 192_000] {
            let rate = SampleRate::new(NonZeroU32::new(hz).expect("a supported rate is not zero"));
            assert_eq!(
                SUPERCLOCK_HZ.get().rem_euclid(i64::from(hz)),
                0,
                "the superclock rate divides exactly by {hz} samples per second"
            );
            for samples in [0_i64, 1, -1, 4_096, -4_096, 1_000_000, -1_000_000] {
                let clock = samples_to_superclock(samples, rate)
                    .expect("the superclock count fits the 64-bit range");
                let back = superclock_to_samples(clock, rate, Rounding::Nearest)
                    .expect("the sample count fits the 64-bit range");
                assert_eq!(
                    back, samples,
                    "the clock round trip returns the sample count at {hz} samples per second"
                );
            }
        }
    }

    #[test]
    fn constants_hold_their_literal_values() {
        assert_eq!(
            TICKS_PER_QUARTER.get(),
            1_920,
            "TICKS_PER_QUARTER holds its literal value"
        );
        assert_eq!(
            SUPERCLOCK_HZ.get(),
            282_240_000,
            "SUPERCLOCK_HZ holds its literal value"
        );
    }
}
