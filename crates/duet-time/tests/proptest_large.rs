//! The nightly soak target: six kernel properties at the soak case count.
//!
//! `tests/kernel.rs` proves each of the six at the gate case count, and this
//! target proves it again at the soak case count:
//!
//! 1. The `Finite` invariants.
//! 2. `muldiv` against a 128-bit reference.
//! 3. The 24-bit round trip.
//! 4. The `superclock_at` and `ticks_at` monotonicity.
//! 5. The `bbt_at` and `ticks_at_bbt` round trip.
//! 6. The `bbt_at` monotonicity under `Bbt::lexicographic_cmp`.
//!
//! The three map properties draw over a wide range, so a longer run reaches
//! inputs that the gate run does not. The tuplet sum is not one of the six: the
//! whole domain of that rule is 99,840 pairs, and
//! `tuplet_parts_sum_to_span` in `tests/kernel.rs` enumerates every one of them
//! at the gate.
//!
//! The gate runs each property at the gate case count of B78. This target runs
//! each one at the soak case count of B78. Every test here carries an `ignore`
//! attribute, so the gate skips them. Section 14 of
//! `roadmap/duet-v1/architecture.md` gives this target to the nightly
//! `soak.yml` workflow, which chunk M7 writes.

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;
    use core::num::{NonZeroI64, NonZeroU8, NonZeroU16, NonZeroU32};

    use duet_time::convert::{Rounding, i24_to_f32, muldiv, unit_to_i24};
    use duet_time::{
        Bbt, Finite, I24, Meter, MeterPoint, NoteValue, Ratio, SuperClock, Tempo, TempoMap,
        TempoMapEdit, TempoPoint, Ticks, TimeError,
    };
    use proptest::prelude::{
        ProptestConfig, Strategy as _, any, prop_assert, prop_assert_eq, prop_assert_ne,
        prop_assume, proptest,
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

    /// The tick count of one bar of four quarter notes.
    const BAR_TICKS: i64 = 7_680;

    /// The tick count of one bar of three quarter notes.
    const THREE_FOUR_BAR_TICKS: i64 = 5_760;

    /// Superclock ticks of one tick at 120 quarter notes per minute.
    const CLOCK_AT_120: i64 = 73_500;

    /// Superclock ticks of one tick at 60 quarter notes per minute.
    const CLOCK_AT_60: i64 = 147_000;

    /// A non-zero 64-bit value for a test fixture.
    fn nonzero_i64(value: i64) -> NonZeroI64 {
        NonZeroI64::new(value).expect("the fixture value is not zero")
    }

    /// A non-zero 8-bit value for a test fixture.
    fn nonzero_u8(value: u8) -> NonZeroU8 {
        NonZeroU8::new(value).expect("the fixture value is not zero")
    }

    /// The address at `bar`, `beat`, and `tick`.
    fn bbt(bar: u32, beat: u16, tick: u16) -> Bbt {
        Bbt::new(
            NonZeroU32::new(bar).expect("a bar number is not zero"),
            NonZeroU16::new(beat).expect("a beat number is not zero"),
            tick,
        )
    }

    /// A tempo entry of `beats` quarter notes per minute.
    fn tempo_point(ticks: i64, clock: i64, beats: i64) -> TempoPoint {
        let rate = Ratio::new(beats, nonzero_i64(1)).expect("the rate reduces inside the range");
        TempoPoint::new(
            Ticks::new(ticks),
            SuperClock::new(clock),
            Tempo::new(rate, NoteValue::Quarter, false).expect("the rate is positive"),
        )
    }

    /// A meter entry of `beats_per_bar` quarter notes.
    fn meter_point(ticks: i64, address: Bbt, beats_per_bar: u8) -> MeterPoint {
        MeterPoint::new(
            Ticks::new(ticks),
            address,
            Meter::new(nonzero_u8(beats_per_bar), NoteValue::Quarter),
        )
    }

    /// A map of three tempo entries whose stored clocks match the arithmetic.
    ///
    /// It is the fixture of `superclock_at_is_monotonic` in `tests/kernel.rs`,
    /// so the soak run and the gate run prove one property of one map.
    fn three_tempo_map() -> TempoMap {
        let second_clock = BAR_TICKS.saturating_mul(CLOCK_AT_120);
        let third_clock = second_clock.saturating_add(BAR_TICKS.saturating_mul(CLOCK_AT_60));
        TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(BAR_TICKS, second_clock, 60))
            .push_tempo(tempo_point(BAR_TICKS.saturating_mul(2), third_clock, 90))
            .finish()
            .expect("the three entries rise in every view")
    }

    /// A map of three meter entries, two of which start inside a bar.
    ///
    /// It is the fixture of `bbt_round_trips_over_a_multi_meter_map` in
    /// `tests/kernel.rs`, and it holds two segment boundaries.
    fn three_meter_map() -> TempoMap {
        let second = BAR_TICKS.saturating_mul(4).saturating_add(1_920);
        let third = second.saturating_add(THREE_FOUR_BAR_TICKS.saturating_mul(3));
        TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(second, bbt(5, 2, 0), 3))
            .push_meter(meter_point(third, bbt(8, 2, 0), 5))
            .finish()
            .expect("each entry states the address the entry before it produces")
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

        /// The strategy draws two arbitrary counts with `any::<i64>()`.
        #[test]
        #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
        fn proptest_large_tempo_queries_are_monotonic(
            first in any::<i64>(),
            second in any::<i64>(),
        ) {
            let map = three_tempo_map();
            let low = first.min(second);
            let high = first.max(second);
            prop_assert!(
                map.superclock_at(Ticks::new(low)) <= map.superclock_at(Ticks::new(high)),
                "a larger tick count never gives a smaller superclock count"
            );
            prop_assert!(
                map.ticks_at(SuperClock::new(low)) <= map.ticks_at(SuperClock::new(high)),
                "a larger superclock count never gives a smaller tick count"
            );
        }
    }

    proptest! {
        #![proptest_config(soak())]

        /// The strategy draws a tick count from `0..=1_000_000_000`.
        #[test]
        #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
        fn proptest_large_bbt_round_trips_over_a_multi_meter_map(
            ticks in 0_i64..=1_000_000_000,
        ) {
            let map = three_meter_map();
            prop_assert_eq!(
                map.ticks_at_bbt(map.bbt_at(Ticks::new(ticks))),
                Ok(Ticks::new(ticks)),
                "an address names the tick it came from on a map of three meter entries"
            );
        }
    }

    proptest! {
        #![proptest_config(soak())]

        /// The strategy draws two arbitrary tick counts with `any::<i64>()`.
        #[test]
        #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
        fn proptest_large_bbt_at_is_monotonic(first in any::<i64>(), second in any::<i64>()) {
            let map = three_meter_map();
            let low = Ticks::new(first.min(second));
            let high = Ticks::new(first.max(second));
            prop_assert_ne!(
                map.bbt_at(low).lexicographic_cmp(map.bbt_at(high)),
                Ordering::Greater,
                "the address never falls as the tick rises, over the whole 64-bit range"
            );
        }
    }
}
