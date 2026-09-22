//! The kernel contract: the `Finite` invariants, the conversion round trips,
//! and the workspace constants.

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;
    use core::num::NonZeroI64;
    use std::hash::{BuildHasher as _, RandomState};

    use core::num::{NonZeroU8, NonZeroU16, NonZeroU32};

    use duet_time::convert::{
        Rounding, f64_to_f32, i16_to_f32, i24_to_f32, i32_to_f32, muldiv, samples_to_superclock,
        superclock_to_samples, ticks_to_f64, unit_to_i24, unit_to_i32,
    };
    use duet_time::{
        Bbt, Delta, Finite, I24, Meter, MeterPoint, NoteValue, Position, Ratio, SUPERCLOCK_HZ,
        SampleRate, Span, SuperClock, TICKS_PER_QUARTER, Tempo, TempoMap, TempoMapEdit, TempoPoint,
        Ticks, TimeDomain, TimeError, split_tuplet,
    };
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

    /// A non-zero 64-bit value for a test fixture.
    fn nonzero_i64(value: i64) -> NonZeroI64 {
        NonZeroI64::new(value).expect("the fixture value is not zero")
    }

    /// A non-zero 32-bit value for a test fixture.
    fn nonzero_u32(value: u32) -> NonZeroU32 {
        NonZeroU32::new(value).expect("the fixture value is not zero")
    }

    /// A non-zero 16-bit value for a test fixture.
    fn nonzero_u16(value: u16) -> NonZeroU16 {
        NonZeroU16::new(value).expect("the fixture value is not zero")
    }

    /// A non-zero 8-bit value for a test fixture.
    fn nonzero_u8(value: u8) -> NonZeroU8 {
        NonZeroU8::new(value).expect("the fixture value is not zero")
    }

    /// The address at `bar`, `beat`, and `tick`.
    fn bbt(bar: u32, beat: u16, tick: u16) -> Bbt {
        Bbt::new(nonzero_u32(bar), nonzero_u16(beat), tick)
    }

    /// A tempo of `beats` quarter notes per minute.
    fn quarter_tempo(beats: i64) -> Tempo {
        let rate = Ratio::new(beats, nonzero_i64(1)).expect("the rate reduces inside the range");
        Tempo::new(rate, NoteValue::Quarter, false).expect("the rate is positive")
    }

    /// A tempo entry with the three views the map reads.
    fn tempo_point(ticks: i64, clock: i64, address: Bbt, beats: i64) -> TempoPoint {
        TempoPoint::new(
            Ticks::new(ticks),
            SuperClock::new(clock),
            address,
            quarter_tempo(beats),
        )
    }

    /// The six note values, in their derived order.
    const NOTE_VALUES: [NoteValue; 6] = [
        NoteValue::Whole,
        NoteValue::Half,
        NoteValue::Quarter,
        NoteValue::Eighth,
        NoteValue::Sixteenth,
        NoteValue::ThirtySecond,
    ];

    /// The tick count of one bar of four quarter notes.
    const BAR_TICKS: i64 = 7_680;

    /// Superclock ticks of one tick at 120 quarter notes per minute.
    const CLOCK_AT_120: i64 = 73_500;

    /// Superclock ticks of one tick at 60 quarter notes per minute.
    const CLOCK_AT_60: i64 = 147_000;

    /// Superclock ticks of one tick at 90 quarter notes per minute.
    const CLOCK_AT_90: i64 = 98_000;

    /// A meter entry with the three views the map reads.
    fn meter_point(ticks: i64, clock: i64, address: Bbt, beats_per_bar: u8) -> MeterPoint {
        MeterPoint::new(
            Ticks::new(ticks),
            SuperClock::new(clock),
            address,
            Meter::new(nonzero_u8(beats_per_bar), NoteValue::Quarter),
        )
    }

    /// A map of one tempo entry at the origin.
    fn one_tempo_map(tempo: Tempo) -> TempoMap {
        TempoMapEdit::new()
            .push_tempo(TempoPoint::new(
                Ticks::ZERO,
                SuperClock::ZERO,
                Bbt::ORIGIN,
                tempo,
            ))
            .finish()
            .expect("a single entry at the origin is a valid map")
    }

    /// A map of three tempo entries whose stored clocks match the arithmetic.
    fn three_tempo_map() -> TempoMap {
        let second_clock = BAR_TICKS.saturating_mul(CLOCK_AT_120);
        let third_clock = second_clock.saturating_add(BAR_TICKS.saturating_mul(CLOCK_AT_60));
        TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, Bbt::ORIGIN, 120))
            .push_tempo(tempo_point(BAR_TICKS, second_clock, bbt(2, 1, 0), 60))
            .push_tempo(tempo_point(
                BAR_TICKS.saturating_mul(2),
                third_clock,
                bbt(3, 1, 0),
                90,
            ))
            .finish()
            .expect("the three entries rise in every view")
    }

    /// A map of one meter entry of four quarter notes at the origin.
    fn quarter_meter_map() -> TempoMap {
        TempoMapEdit::new()
            .push_meter(meter_point(0, 0, Bbt::ORIGIN, 4))
            .finish()
            .expect("a single meter entry at the origin is a valid map")
    }

    #[test]
    fn note_value_ticks_are_exact() {
        let quarter = TICKS_PER_QUARTER.get();
        for (value, want) in [
            (NoteValue::Whole, quarter.saturating_mul(4)),
            (NoteValue::Half, quarter.saturating_mul(2)),
            (NoteValue::Quarter, quarter),
            (NoteValue::Eighth, quarter.div_euclid(2)),
            (NoteValue::Sixteenth, quarter.div_euclid(4)),
            (NoteValue::ThirtySecond, quarter.div_euclid(8)),
        ] {
            assert_eq!(
                value.ticks().get(),
                want,
                "the tick count of {value:?} is the derived multiple of the quarter note"
            );
        }
        assert_eq!(
            NoteValue::Whole.ticks().get(),
            NoteValue::ThirtySecond.ticks().get().saturating_mul(32),
            "a whole note holds 32 thirty-second notes"
        );
    }

    #[test]
    fn note_value_ticks_fall_with_the_derived_order() {
        for (earlier, later) in NOTE_VALUES.iter().zip(NOTE_VALUES.iter().skip(1)) {
            assert!(
                earlier < later,
                "the derived order puts {earlier:?} before {later:?}"
            );
            assert!(
                earlier.ticks().get() > later.ticks().get(),
                "a later note value is a shorter note than {earlier:?}"
            );
        }
    }

    #[test]
    fn ratio_new_reduces_and_normalizes_sign() {
        let half = Ratio::new(1, nonzero_i64(2)).expect("one over two is canonical");
        let two_quarters = Ratio::new(2, nonzero_i64(4)).expect("two over four reduces");
        assert_eq!(two_quarters, half, "two over four reduces to one over two");
        let hasher = RandomState::new();
        assert_eq!(
            hasher.hash_one(two_quarters),
            hasher.hash_one(half),
            "two equal ratios hash equal"
        );
        let moved_sign = Ratio::new(1, nonzero_i64(-2)).expect("a negative denominator is legal");
        assert_eq!(
            moved_sign.numerator(),
            -1,
            "the sign moves to the numerator"
        );
        assert_eq!(
            moved_sign.denominator().get(),
            2,
            "the stored denominator is positive"
        );
    }

    #[test]
    fn ratio_new_refuses_an_unrepresentable_sign() {
        assert_eq!(
            Ratio::new(i64::MIN, nonzero_i64(1)).err(),
            Some(TimeError::Overflow),
            "a numerator magnitude of two to the power 63 has no positive form"
        );
        assert_eq!(
            Ratio::new(1, NonZeroI64::MIN).err(),
            Some(TimeError::Overflow),
            "a denominator magnitude of two to the power 63 has no positive form"
        );
    }

    #[test]
    fn tempo_new_refuses_a_non_positive_rate() {
        let zero = Ratio::new(0, nonzero_i64(1)).expect("zero over one is canonical");
        assert_eq!(
            Tempo::new(zero, NoteValue::Quarter, false).err(),
            Some(TimeError::NotRepresentable),
            "a zero rate names no segment"
        );
        let negative = Ratio::new(-120, nonzero_i64(1)).expect("a negative rate is a ratio");
        assert_eq!(
            Tempo::new(negative, NoteValue::Quarter, false).err(),
            Some(TimeError::NotRepresentable),
            "a negative rate would run the timeline backwards"
        );
    }

    proptest! {
        /// The strategy draws a rate from `1..=1_000_000` over `1..=1_000_000`,
        /// a tick delta from `-1_000_000_000..=1_000_000_000`, and a note value
        /// index from `0..6`.
        #[test]
        fn tempo_scale_matches_the_exact_rational(
            numerator in 1_i64..=1_000_000,
            denominator in 1_i64..=1_000_000,
            delta in -1_000_000_000_i64..=1_000_000_000,
            unit_index in 0_usize..6,
        ) {
            let unit = NOTE_VALUES[unit_index];
            let rate = Ratio::new(numerator, nonzero_i64(denominator))
                .expect("the drawn rate reduces inside the range");
            let tempo = Tempo::new(rate, unit, false).expect("the drawn rate is positive");
            let map = one_tempo_map(tempo);
            let superclocks_per_minute = i128::from(SUPERCLOCK_HZ.get()) * 60;
            let above = i128::from(delta)
                * superclocks_per_minute
                * i128::from(rate.denominator().get());
            let below = i128::from(rate.numerator()) * i128::from(unit.ticks().get());
            let want = above.div_euclid(below);
            if let Ok(inside) = i64::try_from(want) {
                prop_assert_eq!(
                    map.superclock_at(Ticks::new(delta)).get(),
                    inside,
                    "the scale is the floor of the exact rational value"
                );
            }
        }
    }

    #[test]
    fn tempo_scale_saturates_instead_of_wrapping() {
        let map = one_tempo_map(quarter_tempo(1));
        assert_eq!(
            map.superclock_at(Ticks::new(2_000_000_000_000)).get(),
            i64::MAX,
            "a product above the range holds at the upper bound"
        );
        assert_eq!(
            map.superclock_at(Ticks::new(-2_000_000_000_000)).get(),
            i64::MIN,
            "a product below the range holds at the lower bound"
        );
    }

    proptest! {
        /// The strategy draws two arbitrary tick counts with `any::<i64>()`.
        #[test]
        fn superclock_at_is_monotonic(first in any::<i64>(), second in any::<i64>()) {
            let map = three_tempo_map();
            let low = Ticks::new(first.min(second));
            let high = Ticks::new(first.max(second));
            prop_assert!(
                map.superclock_at(low) <= map.superclock_at(high),
                "a larger tick count never gives a smaller superclock count"
            );
        }
    }

    proptest! {
        /// The strategy draws two arbitrary superclock counts with `any::<i64>()`.
        #[test]
        fn ticks_at_is_monotonic(first in any::<i64>(), second in any::<i64>()) {
            let map = three_tempo_map();
            let low = SuperClock::new(first.min(second));
            let high = SuperClock::new(first.max(second));
            prop_assert!(
                map.ticks_at(low) <= map.ticks_at(high),
                "a larger superclock count never gives a smaller tick count"
            );
        }
    }

    proptest! {
        /// The strategy draws a rate from `1..=100_000` over `1..=1_000` and a
        /// tick count from `-1_000_000..=1_000_000`.
        #[test]
        fn superclock_at_round_trips_within_one_tick(
            numerator in 1_i64..=100_000,
            denominator in 1_i64..=1_000,
            ticks in -1_000_000_i64..=1_000_000,
        ) {
            let rate = Ratio::new(numerator, nonzero_i64(denominator))
                .expect("the drawn rate reduces inside the range");
            let tempo = Tempo::new(rate, NoteValue::Quarter, false)
                .expect("the drawn rate is positive");
            let map = one_tempo_map(tempo);
            let back = map.ticks_at(map.superclock_at(Ticks::new(ticks))).get();
            prop_assert!(
                back == ticks || back == ticks - 1,
                "the round trip loses at most one tick to the floor"
            );
        }
    }

    #[test]
    fn tempo_map_treats_a_ramp_as_a_step() {
        let rate = Ratio::new(120, nonzero_i64(1)).expect("120 over one is canonical");
        let stepped = one_tempo_map(
            Tempo::new(rate, NoteValue::Quarter, false).expect("the rate is positive"),
        );
        let ramped = one_tempo_map(
            Tempo::new(rate, NoteValue::Quarter, true).expect("the rate is positive"),
        );
        for probe in [-BAR_TICKS, 0, 1, 1_920, BAR_TICKS, 1_000_000] {
            assert_eq!(
                ramped.superclock_at(Ticks::new(probe)),
                stepped.superclock_at(Ticks::new(probe)),
                "a ramped entry governs its segment at a constant rate at tick {probe}"
            );
        }
    }

    #[test]
    fn tempo_map_anchors_on_the_last_point_at_or_before() {
        let map = three_tempo_map();
        let second_clock = BAR_TICKS.saturating_mul(CLOCK_AT_120);
        let third_clock = second_clock.saturating_add(BAR_TICKS.saturating_mul(CLOCK_AT_60));
        for (probe, want) in [
            (BAR_TICKS - 1, (BAR_TICKS - 1).saturating_mul(CLOCK_AT_120)),
            (BAR_TICKS, second_clock),
            (BAR_TICKS + 1, second_clock.saturating_add(CLOCK_AT_60)),
            (
                BAR_TICKS.saturating_mul(2) - 1,
                second_clock.saturating_add((BAR_TICKS - 1).saturating_mul(CLOCK_AT_60)),
            ),
            (BAR_TICKS.saturating_mul(2), third_clock),
            (
                BAR_TICKS.saturating_mul(2) + 1,
                third_clock.saturating_add(CLOCK_AT_90),
            ),
        ] {
            assert_eq!(
                map.superclock_at(Ticks::new(probe)).get(),
                want,
                "tick {probe} reads the entry that governs it"
            );
        }
    }

    #[test]
    fn tempo_map_extrapolates_before_the_first_point() {
        let map = three_tempo_map();
        assert_eq!(
            map.superclock_at(Ticks::new(-1_920)).get(),
            (-1_920_i64).saturating_mul(CLOCK_AT_120),
            "a tick below the first entry uses the first entry's rate"
        );
        assert!(
            map.superclock_at(Ticks::new(-1_920)).get() < 0,
            "a tick below timeline zero gives a negative superclock count"
        );
    }

    proptest! {
        /// The strategy draws a tick count from `0..=1_000_000_000`.
        #[test]
        fn bbt_round_trips_for_every_tick(ticks in 0_i64..=1_000_000_000) {
            let map = quarter_meter_map();
            prop_assert_eq!(
                map.ticks_at_bbt(map.bbt_at(Ticks::new(ticks))),
                Ok(Ticks::new(ticks)),
                "an address names the tick it came from"
            );
        }
    }

    proptest! {
        /// The strategy draws an arbitrary tick count with `any::<i64>()`.
        #[test]
        fn bbt_at_fields_stay_in_range(ticks in any::<i64>()) {
            let map = quarter_meter_map();
            let address = map.bbt_at(Ticks::new(ticks));
            prop_assert!(
                address.beat().get() <= 4,
                "the beat stays at or below the governing beats per bar"
            );
            prop_assert!(
                i64::from(address.tick()) < NoteValue::Quarter.ticks().get(),
                "the tick stays below one beat"
            );
        }
    }

    #[test]
    fn bbt_at_normalizes_a_mid_bar_meter_point() {
        let start = BAR_TICKS.saturating_mul(4).saturating_add(1_920);
        let map = TempoMapEdit::new()
            .push_meter(meter_point(0, 0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(
                start,
                start.saturating_mul(CLOCK_AT_120),
                bbt(5, 2, 0),
                3,
            ))
            .finish()
            .expect("the two meter entries rise in every view");
        for (probe, want) in [
            (start, bbt(5, 2, 0)),
            (start.saturating_add(1_920), bbt(5, 3, 0)),
            (start.saturating_add(3_840), bbt(6, 1, 0)),
        ] {
            assert_eq!(
                map.bbt_at(Ticks::new(probe)),
                want,
                "a meter entry inside a bar needs no bar alignment at tick {probe}"
            );
        }
    }

    #[test]
    fn bbt_at_clamps_below_the_origin() {
        let map = quarter_meter_map();
        assert_eq!(
            map.bbt_at(Ticks::new(-1)),
            Bbt::ORIGIN,
            "a tick below timeline zero holds the whole address at the origin"
        );
    }

    #[test]
    fn bbt_at_clamps_above_the_last_bar() {
        let map = quarter_meter_map();
        assert_eq!(
            map.bbt_at(Ticks::new(i64::MAX)),
            Bbt::LAST,
            "a tick beyond the last bar holds the whole address at the last bar"
        );
    }

    #[test]
    fn ticks_at_bbt_refuses_a_beat_above_the_meter() {
        let map = quarter_meter_map();
        assert_eq!(
            map.ticks_at_bbt(bbt(1, 5, 0)).err(),
            Some(TimeError::BbtOutOfRange),
            "beat 5 of a bar of four beats names no tick"
        );
    }

    #[test]
    fn ticks_at_bbt_refuses_a_tick_above_the_beat() {
        let map = quarter_meter_map();
        assert_eq!(
            map.ticks_at_bbt(bbt(1, 1, 1_920)).err(),
            Some(TimeError::BbtOutOfRange),
            "the first tick of the next beat names no tick of this beat"
        );
    }

    #[test]
    fn ticks_at_bbt_refuses_an_overflowing_bar() {
        let far = 9_223_372_036_854_000_000_i64;
        let map = TempoMapEdit::new()
            .push_meter(meter_point(0, 0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(far, 1, bbt(4_000_000_000, 1, 0), 4))
            .finish()
            .expect("the two meter entries rise in every view");
        assert_eq!(
            map.ticks_at_bbt(bbt(u32::MAX, 1, 0)).err(),
            Some(TimeError::Overflow),
            "an address whose tick count leaves the 64-bit range is an overflow"
        );
    }

    #[test]
    fn empty_map_is_the_origin() {
        let map = TempoMap::default();
        assert_eq!(
            map.superclock_at(Ticks::new(1_920)),
            SuperClock::ZERO,
            "an empty map answers the origin in the audio domain"
        );
        assert_eq!(
            map.ticks_at(SuperClock::new(1_000)),
            Ticks::ZERO,
            "an empty map answers the origin in the beat domain"
        );
        assert_eq!(
            map.bbt_at(Ticks::new(1_920)),
            Bbt::ORIGIN,
            "an empty map answers the origin address"
        );
        assert_eq!(
            map.ticks_at_bbt(Bbt::ORIGIN),
            Ok(Ticks::ZERO),
            "an empty map states the origin pair"
        );
        assert_eq!(
            map.ticks_at_bbt(bbt(2, 1, 0)).err(),
            Some(TimeError::BbtOutOfRange),
            "an empty map states no other address"
        );
        assert_eq!(
            map.convert(Position::Beats(Ticks::new(1_920)), TimeDomain::Audio),
            Position::Audio(SuperClock::ZERO),
            "a cross-domain conversion on an empty map answers zero"
        );
        assert!(
            map.same_instant(
                Position::Beats(Ticks::new(1_920)),
                Position::Audio(SuperClock::ZERO)
            ),
            "a beat position on an empty map equals the audio origin"
        );
        assert_eq!(
            map.cmp(
                Position::Beats(Ticks::new(1_920)),
                Position::Audio(SuperClock::new(5))
            ),
            Ordering::Less,
            "an empty map converts the beat side to zero and leaves the audio side"
        );
    }

    #[test]
    fn empty_map_keeps_a_same_domain_answer() {
        let map = TempoMap::default();
        let position = Position::Beats(Ticks::new(1_920));
        assert_eq!(
            map.convert(position, TimeDomain::Beats),
            position,
            "a conversion to its own domain is the identity on an empty map"
        );
        assert_eq!(
            map.cmp(position, Position::Beats(Ticks::new(960))),
            Ordering::Greater,
            "two beat positions order by their tick counts on an empty map"
        );
    }

    #[test]
    fn convert_to_the_same_domain_is_the_identity() {
        let map = three_tempo_map();
        for position in [
            Position::Beats(Ticks::new(1_921)),
            Position::Audio(SuperClock::new(7)),
        ] {
            assert_eq!(
                map.convert(position, position.domain()),
                position,
                "a conversion to its own domain loses no value for {position:?}"
            );
        }
    }

    #[test]
    fn cmp_orders_across_domains_in_the_audio_domain() {
        let map = one_tempo_map(quarter_tempo(120));
        let beats = Position::Beats(Ticks::new(1_920));
        let exact = Position::Audio(SuperClock::new(1_920_i64.saturating_mul(CLOCK_AT_120)));
        assert_eq!(
            map.cmp(beats, exact),
            Ordering::Equal,
            "a beat position equals the audio position it converts to"
        );
        assert!(
            map.same_instant(beats, exact),
            "equality agrees with the order"
        );
        let earlier = Position::Audio(SuperClock::new(1_920_i64.saturating_mul(CLOCK_AT_120) - 1));
        assert_eq!(
            map.cmp(beats, earlier),
            Ordering::Greater,
            "a beat position is after a smaller audio position"
        );
        assert!(
            !map.same_instant(beats, earlier),
            "equality agrees with the order for an unequal pair"
        );
    }

    #[test]
    fn tempo_map_same_instant_does_not_shadow_partial_eq() {
        let map = one_tempo_map(quarter_tempo(120));
        let copy = map.clone();
        assert!(map == copy, "the derived PartialEq compares two maps");
        assert!(
            map.same_instant(
                Position::Beats(Ticks::ZERO),
                Position::Audio(SuperClock::ZERO)
            ),
            "the inherent same_instant compares two positions through the map"
        );
    }

    #[test]
    fn tempo_map_refuses_a_first_point_off_the_origin() {
        let off_clock = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 500, Bbt::ORIGIN, 120))
            .finish();
        assert_eq!(
            off_clock.err(),
            Some(TimeError::NoFirstPoint),
            "a first entry off superclock zero is refused"
        );
        let off_address = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, bbt(2, 1, 0), 120))
            .finish();
        assert_eq!(
            off_address.err(),
            Some(TimeError::NoFirstPoint),
            "a first entry off the origin address is refused"
        );
    }

    #[test]
    fn tempo_map_refuses_a_duplicate_tick() {
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, Bbt::ORIGIN, 120))
            .push_tempo(tempo_point(BAR_TICKS, 1_000, bbt(2, 1, 0), 90))
            .push_tempo(tempo_point(BAR_TICKS, 2_000, bbt(3, 1, 0), 60))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::UnorderedMap),
            "a duplicate tick names an entry no query can reach"
        );
    }

    #[test]
    fn tempo_map_refuses_an_unordered_bbt() {
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, Bbt::ORIGIN, 120))
            .push_tempo(tempo_point(BAR_TICKS, 1_000, bbt(5, 1, 0), 90))
            .push_tempo(tempo_point(
                BAR_TICKS.saturating_mul(2),
                2_000,
                bbt(3, 1, 0),
                60,
            ))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::UnorderedMap),
            "a list sorted by tick but not by address is refused"
        );
    }

    #[test]
    fn tempo_map_accepts_a_non_decreasing_clock() {
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, Bbt::ORIGIN, 120))
            .push_tempo(tempo_point(1, 0, bbt(1, 1, 1), 120))
            .finish();
        assert!(
            finished.is_ok(),
            "two entries whose superclock values are equal are accepted"
        );
    }

    #[test]
    fn tempo_map_refuses_an_unsorted_meter_list() {
        let unordered = TempoMapEdit::new()
            .push_meter(meter_point(0, 0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(BAR_TICKS, 1_000, bbt(2, 1, 0), 3))
            .push_meter(meter_point(
                BAR_TICKS.saturating_sub(1),
                2_000,
                bbt(3, 1, 0),
                3,
            ))
            .finish();
        assert_eq!(
            unordered.err(),
            Some(TimeError::UnorderedMap),
            "the meter list carries the order rule of the tempo list"
        );
        let off_origin = TempoMapEdit::new()
            .push_meter(meter_point(BAR_TICKS, 0, Bbt::ORIGIN, 4))
            .finish();
        assert_eq!(
            off_origin.err(),
            Some(TimeError::NoFirstPoint),
            "the meter list carries the first-point rule of the tempo list"
        );
    }

    #[test]
    fn tempo_map_validates_the_two_lists_apart() {
        let tempo_only = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, Bbt::ORIGIN, 120))
            .finish();
        assert!(
            tempo_only.is_ok(),
            "an empty meter list beside a tempo list is accepted"
        );
        let meter_only = TempoMapEdit::new()
            .push_meter(meter_point(0, 0, Bbt::ORIGIN, 4))
            .finish();
        assert!(
            meter_only.is_ok(),
            "an empty tempo list beside a meter list is accepted"
        );
    }

    #[test]
    fn tempo_map_edit_seeds_from_the_map() {
        let map = three_tempo_map();
        assert_eq!(
            map.edit().finish(),
            Ok(map.clone()),
            "a builder seeded from a map finishes as an equal map"
        );
        let trimmed = map
            .edit()
            .remove_tempo_at(Ticks::new(BAR_TICKS))
            .finish()
            .expect("the remaining entries still rise in every view");
        assert_ne!(
            trimmed, map,
            "a removed entry leaves a map that differs from the original"
        );
    }

    #[test]
    fn span_zero_delta_has_zero_length() {
        let map = three_tempo_map();
        let origins = [
            Position::Beats(Ticks::new(1_920)),
            Position::Audio(SuperClock::new(CLOCK_AT_120)),
        ];
        let deltas = [Delta::Beats(Ticks::ZERO), Delta::Audio(SuperClock::ZERO)];
        for origin in origins {
            for delta in deltas {
                let span = Span::new(origin, delta);
                assert_eq!(
                    span.in_beats(&map),
                    Ticks::ZERO,
                    "a zero delta has no length in ticks for {origin:?} and {delta:?}"
                );
                assert_eq!(
                    span.in_audio(&map),
                    SuperClock::ZERO,
                    "a zero delta has no length in superclocks for {origin:?} and {delta:?}"
                );
            }
        }
    }

    #[test]
    fn span_end_matches_origin_later() {
        let map = three_tempo_map();
        let origins = [
            Position::Beats(Ticks::new(1_920)),
            Position::Audio(SuperClock::new(CLOCK_AT_120)),
        ];
        let deltas = [
            Delta::Beats(Ticks::new(960)),
            Delta::Audio(SuperClock::new(CLOCK_AT_120)),
        ];
        for origin in origins {
            for delta in deltas {
                let span = Span::new(origin, delta);
                assert_eq!(
                    span.end(&map),
                    span.origin().later(span, &map),
                    "the end of a span is its origin moved later for {origin:?} and {delta:?}"
                );
            }
        }
    }

    #[test]
    fn position_earlier_and_later_are_inverse() {
        let map = three_tempo_map();
        let origins = [
            Position::Beats(Ticks::new(1_920)),
            Position::Audio(SuperClock::new(CLOCK_AT_120)),
        ];
        let deltas = [
            Delta::Beats(Ticks::new(960)),
            Delta::Audio(SuperClock::new(CLOCK_AT_120)),
        ];
        for origin in origins {
            for delta in deltas {
                let span = Span::new(origin, delta);
                assert_eq!(
                    origin.earlier(span, &map).later(span, &map),
                    origin,
                    "a move earlier and back returns the position for {delta:?}"
                );
            }
        }
    }

    #[test]
    fn position_distance_keeps_the_receiver_domain() {
        let map = three_tempo_map();
        let first = Position::Beats(Ticks::new(1_920));
        let second = Position::Audio(SuperClock::new(CLOCK_AT_120.saturating_mul(3_840)));
        let span = first.distance(second, &map);
        assert_eq!(
            span.origin(),
            first,
            "the origin of the span is the receiver"
        );
        assert_eq!(
            span,
            Span::new(first, Delta::Beats(Ticks::new(1_920))),
            "the delta counts in the receiver's own domain"
        );
    }

    #[test]
    fn tempo_map_refuses_unsorted() {
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, Bbt::ORIGIN, 120))
            .push_tempo(tempo_point(7_680, 2_000, bbt(2, 1, 0), 90))
            .push_tempo(tempo_point(3_840, 3_000, bbt(3, 1, 0), 60))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::UnorderedMap),
            "finish refuses a tempo list whose ticks fall"
        );
    }

    #[test]
    fn tempo_map_refuses_no_first_point() {
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(1_920, 0, Bbt::ORIGIN, 120))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::NoFirstPoint),
            "finish refuses a tempo list whose first entry is off tick zero"
        );
    }

    /// Assert the rounding rule of section 2.4 over one span and one divisor.
    fn assert_tuplet_split(span: i64, divisor: u8) {
        let parts = NonZeroU8::new(divisor).expect("the divisor is not zero");
        let split = split_tuplet(Ticks::new(span), parts);
        assert_eq!(
            split.len(),
            usize::from(divisor),
            "the split holds one entry for each part, at {divisor} over {span}"
        );
        let total: i64 = split.iter().map(|part| part.get()).sum();
        assert_eq!(
            total, span,
            "the parts sum to the span exactly, at {divisor} over {span}"
        );
        let head = split.first().expect("the split holds at least one part");
        for part in split.iter().take(usize::from(divisor).saturating_sub(1)) {
            assert_eq!(
                part, head,
                "every part but the last holds the same count, at {divisor} over {span}"
            );
        }
    }

    #[test]
    fn tuplet_parts_sum_to_span() {
        for divisor in 2_u8..=13 {
            for span in 1_i64..=7_680 {
                assert_tuplet_split(span, divisor);
            }
        }
    }
}
