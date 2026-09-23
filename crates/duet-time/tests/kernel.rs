//! The kernel contract: the `Finite` invariants, the conversion round trips,
//! and the workspace constants.

#[cfg(test)]
mod tests {
    use core::cmp::Ordering;
    use core::num::NonZeroI64;
    use std::hash::{BuildHasher as _, RandomState};

    use core::num::{NonZeroU8, NonZeroU16, NonZeroU32};

    use duet_time::convert::{
        Rounding, Unit, f64_to_f32, finite_to_f32_saturating, i16_to_f32, i24_to_f32, i32_to_f32,
        muldiv, sample_clock_to_superclock, samples_to_superclock, superclock_to_sample_clock,
        superclock_to_samples, ticks_to_f64, unit_to_i24, unit_to_i32,
    };
    use duet_time::{
        BarCount, Bbt, ChannelCount, ChannelIndex, Delta, Finite, FrameCount, GainDb, I24,
        MAX_BOUND_PORTS, MAX_PARAMS, MAX_SENDS, MAX_SLOT_METERS, MAX_SLOTS, MAX_STRIPS, Meter,
        MeterPoint, NoteValue, Position, Ratio, SUPERCLOCK_HZ, SampleClock, SampleRate,
        SchemaVersion, Span, SuperClock, TICKS_PER_QUARTER, Tempo, TempoMap, TempoMapEdit,
        TempoPoint, Ticks, TimeDomain, TimeError, Tuplet, UnixSeconds, finite, split_tuplet,
    };
    use proptest::prelude::{
        ProptestConfig, Strategy as _, any, prop_assert, prop_assert_eq, prop_assert_ne,
        prop_assume, proptest,
    };

    /// The proptest case count of the gate (B78).
    const GATE_CASES: u32 = 1_000;

    /// The proptest configuration of every property in this file.
    ///
    /// `tests/proptest_large.rs` proves six of these properties again at the
    /// soak case count of the same budget row: the `Finite` invariants,
    /// `muldiv`, the 24-bit round trip, the two tempo queries, the bar round
    /// trip, and the bar order.
    fn gate() -> ProptestConfig {
        ProptestConfig {
            cases: GATE_CASES,
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

    proptest! {
        #![proptest_config(gate())]

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
        #![proptest_config(gate())]

        #[test]
        fn finite_equality_matches_bits(left in any::<f64>(), right in any::<f64>()) {
            if let (Some(first), Some(second)) = (Finite::new(left), Finite::new(right)) {
                prop_assert_eq!(
                    first == second,
                    (left + 0.0).to_bits() == (right + 0.0).to_bits(),
                    "equality follows the canonical bits"
                );
            }
        }
    }

    proptest! {
        #![proptest_config(gate())]

        #[test]
        fn finite_order_matches_total_cmp(left in any::<f64>(), right in any::<f64>()) {
            if let (Some(first), Some(second)) = (Finite::new(left), Finite::new(right)) {
                prop_assert_eq!(
                    first.cmp(&second),
                    (left + 0.0).total_cmp(&(right + 0.0)),
                    "the order is total_cmp over the canonical value"
                );
            }
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
        #![proptest_config(gate())]

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

    /// The 64-bit boundary values that a uniform strategy cannot reach.
    ///
    /// `any::<i64>()` is uniform over the whole range, so each of these has a
    /// probability near two to the power minus 64 per draw and no gate run
    /// reaches one. `muldiv` holds a branch for a negative divisor, and
    /// `i64::MIN` is the value that branch is written for.
    const I64_BOUNDARIES: [i64; 6] = [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX];

    /// Assert `muldiv` against the 128-bit reference in every rounding mode.
    fn assert_muldiv_matches_reference(value: i64, numerator: i64, divisor: NonZeroI64) {
        let denominator = divisor.get();
        for rounding in [Rounding::Down, Rounding::Nearest, Rounding::Up] {
            let got = muldiv(value, numerator, divisor, rounding);
            match reference(value, numerator, denominator, rounding) {
                Some(want) => assert_eq!(
                    got,
                    Ok(want),
                    "muldiv matches the reference at {value} times {numerator} over {denominator} under {rounding:?}"
                ),
                None => assert_eq!(
                    got,
                    Err(TimeError::Overflow),
                    "muldiv reports an overflow at {value} times {numerator} over {denominator} under {rounding:?}"
                ),
            }
        }
    }

    /// Assert `muldiv` over every boundary denominator of one pair.
    fn assert_muldiv_over_the_boundary_denominators(value: i64, numerator: i64) {
        for denominator in I64_BOUNDARIES {
            let Some(divisor) = NonZeroI64::new(denominator) else {
                continue;
            };
            assert_muldiv_matches_reference(value, numerator, divisor);
        }
    }

    #[test]
    fn muldiv_matches_the_reference_at_every_boundary() {
        for value in I64_BOUNDARIES {
            for numerator in I64_BOUNDARIES {
                assert_muldiv_over_the_boundary_denominators(value, numerator);
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
            assert_eq!(
                scaled, want,
                "both scales are powers of two, so the 16-bit round trip of {sample} is exact"
            );
        }
    }

    proptest! {
        #![proptest_config(gate())]

        #[test]
        fn convert_i24_round_trip(raw in any::<i32>().prop_map(narrow_to_i24)) {
            let sample = I24::new(raw).expect("the folded value is a 24-bit sample");
            prop_assert_eq!(
                unit_to_i24(i24_to_f32(sample)).get(),
                raw,
                "the 24-bit round trip answers itself"
            );
        }
    }

    /// The 24-bit boundary samples, each with the answer of one round trip.
    ///
    /// Both scales are two to the power 23, so every pair is an identity
    /// (ADR-0007). The table stays beside the exhaustive test of this pair,
    /// because it names the values a reader checks first.
    const I24_BOUNDARY_ROUND_TRIPS: [(i32, i32); 8] = [
        (-8_388_608, -8_388_608),
        (-8_388_607, -8_388_607),
        (-4_194_304, -4_194_304),
        (-1, -1),
        (0, 0),
        (1, 1),
        (4_194_304, 4_194_304),
        (8_388_607, 8_388_607),
    ];

    #[test]
    fn convert_i24_round_trip_at_the_boundaries() {
        assert_eq!(
            I24::MIN.get(),
            -8_388_608,
            "the table opens at the lowest sample the type holds"
        );
        assert_eq!(
            I24::MAX.get(),
            8_388_607,
            "the table closes at the highest sample the type holds"
        );
        for (raw, want) in I24_BOUNDARY_ROUND_TRIPS {
            let sample = I24::new(raw).expect("a boundary value is a 24-bit sample");
            assert_eq!(
                unit_to_i24(i24_to_f32(sample)).get(),
                want,
                "the 24-bit round trip of {raw} answers {want}"
            );
        }
    }

    #[test]
    fn convert_i24_round_trip_is_lossless_over_every_sample() {
        for raw in I24::MIN.get()..=I24::MAX.get() {
            let Some(sample) = I24::new(raw) else {
                panic!("every value of the range is a 24-bit sample");
            };
            assert_eq!(
                unit_to_i24(i24_to_f32(sample)).get(),
                raw,
                "the 24-bit round trip of {raw} answers itself"
            );
        }
    }

    /// The drift bound of the 32-bit round trip, in one derived term.
    ///
    /// An `f32` carries a 24-bit significand, so the widening of an `i32`
    /// moves the value by at most one half of a unit in the last place. The
    /// widest binade an `i32` reaches is two to the power 30 to two to the
    /// power 31, where a unit in the last place is 128, so the half is 64.
    /// Both scales are powers of two, so neither scaling adds a term, and the
    /// rounding to an integer adds none either, because the product of an
    /// exact `f32` and two to the power 31 is an integer (ADR-0007).
    const I32_ROUND_TRIP_DRIFT: i64 = 64;

    /// The drift of one 32-bit round trip through the unit range.
    fn i32_round_trip_drift(sample: i32) -> i64 {
        i64::from(unit_to_i32(i32_to_f32(sample))) - i64::from(sample)
    }

    /// The four 32-bit samples that attain the drift bound, each with the
    /// signed drift of one round trip.
    ///
    /// An exhaustive scan of all 4,294,967,296 values finds a maximum absolute
    /// drift of exactly 64, and many samples attain it. ADR-0007 decision 4
    /// names these four. The proptest above draws 1,000 cases, so it reaches
    /// none of them; this table is the committed record of the measurement.
    const I32_DRIFT_BOUND_SAMPLES: [(i32, i64); 4] = [
        (-2_147_483_584, -64),
        (1_073_741_888, -64),
        (1_195_673_408, -64),
        (1_946_040_768, 64),
    ];

    #[test]
    fn convert_i32_round_trip_at_the_boundaries() {
        for sample in [i32::MIN, i32::MIN + 1, -1, 0, 1, i32::MAX] {
            assert!(
                i32_round_trip_drift(sample).abs() <= I32_ROUND_TRIP_DRIFT,
                "the 32-bit round trip of {sample} stays inside the bound {I32_ROUND_TRIP_DRIFT}"
            );
        }
        for (sample, want) in I32_DRIFT_BOUND_SAMPLES {
            assert_eq!(
                i32_round_trip_drift(sample),
                want,
                "the 32-bit round trip of {sample} drifts by exactly {want}"
            );
            assert_eq!(
                want.abs(),
                I32_ROUND_TRIP_DRIFT,
                "the sample {sample} attains the bound {I32_ROUND_TRIP_DRIFT}"
            );
        }
    }

    proptest! {
        #![proptest_config(gate())]

        /// The strategy draws an arbitrary sample with `any::<i32>()`.
        #[test]
        fn convert_i32_round_trip(sample in any::<i32>()) {
            prop_assert!(
                i32_round_trip_drift(sample).abs() <= I32_ROUND_TRIP_DRIFT,
                "the 32-bit round trip stays inside the bound {}",
                I32_ROUND_TRIP_DRIFT
            );
            if i64::from(sample).abs() < 16_777_216 {
                prop_assert_eq!(
                    i32_round_trip_drift(sample),
                    0,
                    "the 32-bit round trip is exact below a magnitude of 2^24"
                );
            }
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
        assert_eq!(MAX_STRIPS, 48, "MAX_STRIPS holds the value of row B86");
        assert_eq!(MAX_PARAMS, 2_048, "MAX_PARAMS holds the value of row B90");
        assert_eq!(MAX_SLOTS, 8, "MAX_SLOTS holds the value of row B45");
        assert_eq!(MAX_SENDS, 8, "MAX_SENDS holds the value of row B46");
        assert_eq!(
            MAX_BOUND_PORTS, 16,
            "MAX_BOUND_PORTS holds the value of row B139"
        );
        assert_eq!(
            MAX_SLOT_METERS, 768,
            "MAX_SLOT_METERS holds the value of row B133"
        );
        assert_eq!(
            MAX_SLOT_METERS,
            MAX_STRIPS * 2 * MAX_SLOTS,
            "MAX_SLOT_METERS is arithmetic over three constants and never a literal"
        );
        assert_eq!(
            SampleRate::SUPPORTED.len(),
            6,
            "SampleRate::SUPPORTED holds the six rates of section 2.1"
        );
        for rate in SampleRate::SUPPORTED {
            assert_ne!(
                rate.get().get(),
                0,
                "the type of the rate field makes a zero rate unrepresentable"
            );
        }
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

    /// A tempo entry with the two views the map reads.
    fn tempo_point(ticks: i64, clock: i64, beats: i64) -> TempoPoint {
        TempoPoint::new(
            Ticks::new(ticks),
            SuperClock::new(clock),
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

    /// Superclock ticks of one tick at 240 quarter notes per minute.
    const CLOCK_AT_240: i64 = 36_750;

    /// The tick count of one bar of three quarter notes.
    const THREE_FOUR_BAR_TICKS: i64 = 5_760;

    /// The first tick of the last bar a `Bbt` can name, under 4/4 at the
    /// origin. It is `(u32::MAX - 1)` bars of `BAR_TICKS` ticks.
    const LAST_BAR_TICKS: i64 = 32_985_348_817_920;

    /// A rate at which one tick scales to no superclock tick at all.
    ///
    /// The floor of the tempo scale answers zero when the beat side is above
    /// the minute side, which needs a rate above `8_820_000` quarter notes per
    /// minute. It is the rate at which two adjacent entries may share one
    /// superclock value.
    const EXTREME_RATE: i64 = 9_000_000;

    /// A meter entry with the two views the map reads.
    fn meter_point(ticks: i64, address: Bbt, beats_per_bar: u8) -> MeterPoint {
        MeterPoint::new(
            Ticks::new(ticks),
            address,
            Meter::new(nonzero_u8(beats_per_bar), NoteValue::Quarter),
        )
    }

    /// A map of one tempo entry at the origin.
    fn one_tempo_map(tempo: Tempo) -> TempoMap {
        TempoMapEdit::new()
            .push_tempo(TempoPoint::new(Ticks::ZERO, SuperClock::ZERO, tempo))
            .finish()
            .expect("a single entry at the origin is a valid map")
    }

    /// The stored clock of the second entry of `three_tempo_map`.
    fn second_tempo_clock() -> i64 {
        BAR_TICKS.saturating_mul(CLOCK_AT_120)
    }

    /// A map of three tempo entries whose stored clocks match the arithmetic.
    fn three_tempo_map() -> TempoMap {
        let second_clock = second_tempo_clock();
        let third_clock = second_clock.saturating_add(BAR_TICKS.saturating_mul(CLOCK_AT_60));
        TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(BAR_TICKS, second_clock, 60))
            .push_tempo(tempo_point(BAR_TICKS.saturating_mul(2), third_clock, 90))
            .finish()
            .expect("the three entries rise in every view")
    }

    /// A map of one meter entry of four quarter notes at the origin.
    fn quarter_meter_map() -> TempoMap {
        TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
            .finish()
            .expect("a single meter entry at the origin is a valid map")
    }

    /// A map that holds a tempo list and a meter list together.
    ///
    /// The load path reads both lists, so a round trip needs a map that holds
    /// both. It takes the three tempo entries and the three meter entries of
    /// the two other fixtures.
    fn tempo_and_meter_map() -> TempoMap {
        let meters = three_meter_map();
        let mut edit = three_tempo_map().edit();
        for point in meters.meters() {
            edit = edit.push_meter(*point);
        }
        edit.finish()
            .expect("the two lists are validated apart, and each one is consistent")
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
        #![proptest_config(gate())]

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
        #![proptest_config(gate())]

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
        #![proptest_config(gate())]

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
        #![proptest_config(gate())]

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
        #![proptest_config(gate())]

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
        #![proptest_config(gate())]

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
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(start, bbt(5, 2, 0), 3))
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
            bbt(u32::MAX, 4, 1_919),
            "a tick beyond the last bar holds at the LAST address the governing meter names"
        );
        assert_eq!(
            Bbt::LAST.lexicographic_cmp(map.bbt_at(Ticks::new(i64::MAX))),
            Ordering::Less,
            "the first address of the last bar is below the clamp, so it is not the clamp target"
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
    fn ticks_at_bbt_answers_the_far_bar_of_a_consistent_map() {
        let start = BAR_TICKS.saturating_mul(3_999_999_999);
        let map = TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(start, bbt(4_000_000_000, 1, 0), 4))
            .finish()
            .expect("the second entry states the address the first entry produces");
        assert_eq!(
            map.ticks_at_bbt(bbt(u32::MAX, 1, 0)),
            Ok(Ticks::new(
                start.saturating_add(BAR_TICKS.saturating_mul(294_967_295))
            )),
            "the last bar a `Bbt` names is inside the 64-bit range on a consistent map"
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
            .push_tempo(tempo_point(0, 500, 120))
            .finish();
        assert_eq!(
            off_clock.err(),
            Some(TimeError::NoFirstPoint),
            "a first tempo entry off superclock zero is refused"
        );
        let off_address = TempoMapEdit::new()
            .push_meter(meter_point(0, bbt(2, 1, 0), 4))
            .finish();
        assert_eq!(
            off_address.err(),
            Some(TimeError::NoFirstPoint),
            "a first meter entry off the origin address is refused"
        );
    }

    #[test]
    fn tempo_map_refuses_a_duplicate_tick() {
        let second_clock = second_tempo_clock();
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(BAR_TICKS, second_clock, 90))
            .push_tempo(tempo_point(BAR_TICKS, second_clock, 60))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::UnorderedMap),
            "a duplicate tick names an entry no query can reach, and the cross-check accepts a tick delta of zero"
        );
    }

    #[test]
    fn meter_map_refuses_an_address_that_does_not_rise() {
        let finished = TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(BAR_TICKS, bbt(2, 1, 0), 3))
            .push_meter(meter_point(BAR_TICKS, bbt(2, 1, 0), 5))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::UnorderedMap),
            "two meter entries at one tick hold one address, so the address does not rise, and the cross-check accepts the pair"
        );
    }

    #[test]
    fn tempo_map_accepts_a_non_decreasing_clock() {
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, EXTREME_RATE))
            .push_tempo(tempo_point(1, 0, EXTREME_RATE))
            .finish();
        assert!(
            finished.is_ok(),
            "at a rate where one tick scales to no superclock tick, two equal superclock values are accepted"
        );
    }

    #[test]
    fn tempo_map_refuses_an_unsorted_meter_list() {
        let unordered = TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(BAR_TICKS, bbt(2, 1, 0), 3))
            .push_meter(meter_point(
                BAR_TICKS.saturating_sub(1),
                bbt(1, 3, 1_919),
                3,
            ))
            .finish();
        assert_eq!(
            unordered.err(),
            Some(TimeError::UnorderedMap),
            "the meter list carries the order rule of the tempo list, and the cross-check accepts the third entry"
        );
        let off_origin = TempoMapEdit::new()
            .push_meter(meter_point(BAR_TICKS, Bbt::ORIGIN, 4))
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
            .push_tempo(tempo_point(0, 0, 120))
            .finish();
        assert!(
            tempo_only.is_ok(),
            "an empty meter list beside a tempo list is accepted"
        );
        let meter_only = TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
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
            .remove_tempo_at(Ticks::new(BAR_TICKS.saturating_mul(2)))
            .finish()
            .expect("the remaining entries still rise and still agree with the arithmetic");
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
    fn span_end_is_the_position_the_arithmetic_names() {
        let map = one_tempo_map(quarter_tempo(120));
        let origin_ticks = 1_920_i64;
        let delta_ticks = 960_i64;
        let end_ticks = origin_ticks.saturating_add(delta_ticks);
        let beat_origin = Position::Beats(Ticks::new(origin_ticks));
        let audio_origin =
            Position::Audio(SuperClock::new(origin_ticks.saturating_mul(CLOCK_AT_120)));
        let beat_delta = Delta::Beats(Ticks::new(delta_ticks));
        let audio_delta = Delta::Audio(SuperClock::new(delta_ticks.saturating_mul(CLOCK_AT_120)));
        let beat_end = Position::Beats(Ticks::new(end_ticks));
        let audio_end = Position::Audio(SuperClock::new(end_ticks.saturating_mul(CLOCK_AT_120)));
        for (origin, delta, want) in [
            (beat_origin, beat_delta, beat_end),
            (beat_origin, audio_delta, beat_end),
            (audio_origin, beat_delta, audio_end),
            (audio_origin, audio_delta, audio_end),
        ] {
            assert_eq!(
                Span::new(origin, delta).end(&map),
                want,
                "the end is the tick or the clock the map names for {origin:?} and {delta:?}"
            );
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
        let second_clock = second_tempo_clock();
        let third_clock = second_clock.saturating_sub(3_840_i64.saturating_mul(CLOCK_AT_90));
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(7_680, second_clock, 90))
            .push_tempo(tempo_point(3_840, third_clock, 60))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::UnorderedMap),
            "finish refuses a tempo list whose ticks fall, and the cross-check accepts the third clock"
        );
    }

    #[test]
    fn tempo_map_refuses_no_first_point() {
        let finished = TempoMapEdit::new()
            .push_tempo(tempo_point(1_920, 0, 120))
            .finish();
        assert_eq!(
            finished.err(),
            Some(TimeError::NoFirstPoint),
            "finish refuses a tempo list whose first entry is off tick zero"
        );
    }

    /// A consistent map of three meter entries, two of which start mid-bar.
    ///
    /// Each entry states the address the entry before it produces at that
    /// tick, so `finish` accepts it, and none of the three starts a bar.
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

    #[test]
    fn finish_refuses_a_tempo_clock_the_rate_denies() {
        let refused = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(1_920, 1, 120))
            .finish();
        assert_eq!(
            refused.err(),
            Some(TimeError::UnorderedMap),
            "a stored clock of 1 where the rate of the entry before it produces 141_120_000 is refused"
        );
        let repaired = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(
                1_920,
                1_920_i64.saturating_mul(CLOCK_AT_120),
                120,
            ))
            .finish()
            .expect("the stored clock is the clock the rate produces");
        assert!(
            repaired.superclock_at(Ticks::new(1_920)) > repaired.superclock_at(Ticks::new(1_919)),
            "the query rises across the segment boundary of a map that finish accepts"
        );
    }

    #[test]
    fn finish_refuses_a_meter_address_the_bar_arithmetic_denies() {
        let refused = TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 4))
            .push_meter(meter_point(15_360, bbt(2, 1, 0), 4))
            .finish();
        assert_eq!(
            refused.err(),
            Some(TimeError::UnorderedMap),
            "a second entry that claims bar 2 where the first entry produces bar 3 is refused"
        );
    }

    #[test]
    fn finish_refuses_a_meter_address_the_entry_meter_cannot_name() {
        let refused = TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 7))
            .push_meter(meter_point(11_520, bbt(1, 7, 0), 4))
            .finish();
        assert_eq!(
            refused.err(),
            Some(TimeError::UnorderedMap),
            "beat 7 is the address the first meter produces, and a meter of four beats does not name it"
        );
    }

    #[test]
    fn finish_accepts_a_meter_change_on_a_barline() {
        let map = TempoMapEdit::new()
            .push_meter(meter_point(0, Bbt::ORIGIN, 7))
            .push_meter(meter_point(13_440, bbt(2, 1, 0), 4))
            .finish()
            .expect("a change on a barline stores beat 1, which every meter names");
        assert_eq!(
            map.ticks_at_bbt(bbt(2, 1, 0)),
            Ok(Ticks::new(13_440)),
            "the address that the second entry stores names the tick of that entry"
        );
        assert_eq!(
            map.bbt_at(Ticks::new(13_440)),
            bbt(2, 1, 0),
            "the query answers the address that the entry stores"
        );
    }

    #[test]
    fn finish_accepts_a_meter_entry_that_starts_inside_a_bar() {
        let map = three_meter_map();
        assert_eq!(
            map.meters().len(),
            3,
            "a meter entry needs no bar alignment, only an address the arithmetic produces"
        );
    }

    proptest! {
        #![proptest_config(gate())]

        /// The strategy draws a tick count from `0..=200_000`.
        #[test]
        fn bbt_round_trips_over_a_multi_meter_map(ticks in 0_i64..=200_000) {
            let map = three_meter_map();
            prop_assert_eq!(
                map.ticks_at_bbt(map.bbt_at(Ticks::new(ticks))),
                Ok(Ticks::new(ticks)),
                "an address names the tick it came from on a map of three meter entries"
            );
        }
    }

    #[test]
    fn bbt_at_rises_at_the_top_of_the_range() {
        let map = quarter_meter_map();
        for (probe, want) in [
            (LAST_BAR_TICKS, bbt(u32::MAX, 1, 0)),
            (LAST_BAR_TICKS.saturating_add(1_920), bbt(u32::MAX, 2, 0)),
            (
                LAST_BAR_TICKS.saturating_add(BAR_TICKS),
                bbt(u32::MAX, 4, 1_919),
            ),
        ] {
            assert_eq!(
                map.bbt_at(Ticks::new(probe)),
                want,
                "the address at tick {probe} is the one the meter names"
            );
        }
        let rising = [
            LAST_BAR_TICKS,
            LAST_BAR_TICKS.saturating_add(1_920),
            LAST_BAR_TICKS.saturating_add(BAR_TICKS),
        ];
        for pair in rising.windows(2) {
            let [low, high] = pair else { continue };
            assert_ne!(
                map.bbt_at(Ticks::new(*low))
                    .lexicographic_cmp(map.bbt_at(Ticks::new(*high))),
                Ordering::Greater,
                "the address at tick {high} is not below the address at tick {low}"
            );
        }
    }

    proptest! {
        #![proptest_config(gate())]

        /// The strategy draws two arbitrary tick counts with `any::<i64>()`.
        #[test]
        fn bbt_at_is_monotonic(first in any::<i64>(), second in any::<i64>()) {
            let low = Ticks::new(first.min(second));
            let high = Ticks::new(first.max(second));
            for map in [quarter_meter_map(), three_meter_map()] {
                prop_assert_ne!(
                    map.bbt_at(low).lexicographic_cmp(map.bbt_at(high)),
                    Ordering::Greater,
                    "the address never falls as the tick rises, over the whole 64-bit range and across a segment boundary"
                );
            }
        }
    }

    #[test]
    fn tempo_scale_reduces_before_the_multiply() {
        let rate = Ratio::new(520_833_333_333_333_333, nonzero_i64(59_049_180_329))
            .expect("the rate reduces inside the range");
        let tempo = Tempo::new(rate, NoteValue::Quarter, false).expect("the rate is positive");
        let map = one_tempo_map(tempo);
        assert_eq!(
            map.superclock_at(Ticks::new(1_000_000_000_000_000_000))
                .get(),
            999_962_439_363_417_600,
            "a large rate denominator no longer drives the intermediate out of the 128-bit range"
        );
        assert_eq!(
            map.superclock_at(Ticks::new(100_000_000_000_000_000)).get(),
            99_996_243_936_341_760,
            "the answer that already fitted the intermediate is unchanged"
        );
    }

    #[test]
    fn tempo_new_tests_both_sides_of_the_rate() {
        let stored: Ratio = serde_json::from_str(r#"{"numerator":120,"denominator":-1}"#)
            .expect("the constructor moves the sign to the numerator");
        assert_eq!(
            stored.denominator().get(),
            1,
            "the stored denominator is positive on the load path"
        );
        assert_eq!(stored.numerator(), -120, "the sign moved to the numerator");
        assert_eq!(
            Tempo::new(stored, NoteValue::Quarter, false).err(),
            Some(TimeError::NotRepresentable),
            "a negative rate names no segment"
        );
    }

    /// Assert that a serde refusal carries the message of one `TimeError`.
    ///
    /// `serde_json` renders a `TryFrom` error as its own message, so the text
    /// names the variant. A test that asserts `is_err` alone stays green when a
    /// field is renamed and the parse fails for another reason.
    fn assert_refusal_names<T>(parsed: &Result<T, serde_json::Error>, want: TimeError, label: &str)
    where
        T: core::fmt::Debug,
    {
        let error = parsed
            .as_ref()
            .err()
            .unwrap_or_else(|| panic!("the load path refuses {label}"));
        assert!(
            error.to_string().contains(&want.to_string()),
            "the refusal of {label} names {want:?} and reads: {error}"
        );
    }

    #[test]
    fn deserialization_refuses_a_zero_tempo_rate() {
        let parsed: Result<Tempo, serde_json::Error> = serde_json::from_str(
            r#"{"beats_per_minute":{"numerator":0,"denominator":1},"beat_unit":"Quarter","ramped":false}"#,
        );
        assert_refusal_names(
            &parsed,
            TimeError::NotRepresentable,
            "a stored rate of zero",
        );
    }

    #[test]
    fn deserialization_canonicalizes_a_reducible_ratio() {
        let parsed: Ratio = serde_json::from_str(r#"{"numerator":2,"denominator":4}"#)
            .expect("two over four is a legal ratio");
        assert_eq!(parsed.numerator(), 1, "the numerator is reduced");
        assert_eq!(parsed.denominator().get(), 2, "the denominator is reduced");
        assert_eq!(
            parsed,
            Ratio::new(1, nonzero_i64(2)).expect("one over two is a legal ratio"),
            "two ratios of one value are one value on the load path too"
        );
    }

    #[test]
    fn deserialization_refuses_a_negative_tempo_denominator() {
        let parsed: Result<Tempo, serde_json::Error> = serde_json::from_str(
            r#"{"beats_per_minute":{"numerator":120,"denominator":-1},"beat_unit":"Quarter","ramped":false}"#,
        );
        assert_refusal_names(
            &parsed,
            TimeError::NotRepresentable,
            "a stored rate that runs the timeline backwards",
        );
    }

    #[test]
    fn deserialization_refuses_a_sample_outside_the_24_bit_range() {
        let parsed: Result<I24, serde_json::Error> = serde_json::from_str("2000000000");
        assert_refusal_names(
            &parsed,
            TimeError::NotRepresentable,
            "a stored sample far outside the 24-bit range",
        );
        let inside: I24 = serde_json::from_str("8388607").expect("the highest sample is in range");
        assert_eq!(inside, I24::MAX, "a sample inside the range still parses");
    }

    #[test]
    fn deserialization_refuses_a_map_the_builder_refuses() {
        let document = r#"{"tempos":[],"meters":[{"ticks":5000,"bbt":{"bar":9,"beat":1,"tick":0},"meter":{"beats_per_bar":4,"beat_unit":"Quarter"}}]}"#;
        let parsed: Result<TempoMap, serde_json::Error> = serde_json::from_str(document);
        assert_refusal_names(
            &parsed,
            TimeError::NoFirstPoint,
            "a stored meter list whose first entry is off tick zero",
        );
        let unnameable = r#"{"tempos":[],"meters":[{"ticks":0,"bbt":{"bar":1,"beat":1,"tick":0},"meter":{"beats_per_bar":7,"beat_unit":"Quarter"}},{"ticks":11520,"bbt":{"bar":1,"beat":7,"tick":0},"meter":{"beats_per_bar":4,"beat_unit":"Quarter"}}]}"#;
        let refused: Result<TempoMap, serde_json::Error> = serde_json::from_str(unnameable);
        assert_refusal_names(
            &refused,
            TimeError::UnorderedMap,
            "a stored meter entry whose own meter cannot name its address",
        );
    }

    #[test]
    fn tempo_map_json_round_trips() {
        let map = tempo_and_meter_map();
        assert_eq!(
            (map.tempos().len(), map.meters().len()),
            (3, 3),
            "the round trip reads a map that holds both lists"
        );
        let document = serde_json::to_string(&map).expect("the map serializes");
        let parsed: TempoMap = serde_json::from_str(&document).expect("a valid map parses back");
        assert_eq!(parsed, map, "a map the builder accepts survives the file");
    }

    #[test]
    fn tempo_map_reads_its_own_entries() {
        let map = three_tempo_map();
        assert_eq!(
            map.tempos().len(),
            3,
            "the read surface answers every tempo entry"
        );
        assert_eq!(
            map.meters(),
            &[],
            "an empty meter list reads as an empty slice"
        );
        assert_eq!(
            map.tempo_at(Ticks::new(BAR_TICKS)).map(TempoPoint::ticks),
            Some(Ticks::new(BAR_TICKS)),
            "the lookup answers the entry that sits at the tick"
        );
        assert_eq!(
            map.tempo_at(Ticks::new(BAR_TICKS.saturating_sub(1)))
                .map(TempoPoint::ticks),
            Some(Ticks::ZERO),
            "the lookup answers the last entry at or before the tick"
        );
        assert_eq!(
            TempoMap::default().tempo_at(Ticks::ZERO),
            None,
            "an empty list has no governing entry"
        );
        let meters = three_meter_map();
        assert_eq!(
            meters
                .meter_at(Ticks::new(BAR_TICKS))
                .map(|point| point.meter().beats_per_bar().get()),
            Some(4),
            "the meter lookup answers the governing meter"
        );
        assert_eq!(
            TempoMap::default().meter_at(Ticks::ZERO),
            None,
            "an empty meter list has no governing entry"
        );
    }

    #[test]
    fn tempo_map_edit_inserts_a_tempo_change_in_the_middle() {
        let edited = three_tempo_map()
            .edit()
            .insert_tempo(tempo_point(3_840, 0, 240))
            .recompute_cached_views()
            .finish()
            .expect("the recompute states the clock that the rate before each entry produces");
        assert_eq!(
            edited.tempos().len(),
            4,
            "the new entry sits between the first entry and the second"
        );
        let at_insert = 3_840_i64.saturating_mul(CLOCK_AT_120);
        let at_second = at_insert.saturating_add(3_840_i64.saturating_mul(CLOCK_AT_240));
        let at_third = at_second.saturating_add(BAR_TICKS.saturating_mul(CLOCK_AT_60));
        for (probe, want) in [
            (3_840, at_insert),
            (BAR_TICKS, at_second),
            (BAR_TICKS.saturating_mul(2), at_third),
        ] {
            assert_eq!(
                edited.superclock_at(Ticks::new(probe)).get(),
                want,
                "the map answers the clock that the rates before tick {probe} produce"
            );
        }
    }

    #[test]
    fn tempo_map_edit_replaces_an_entry_at_the_same_tick() {
        let edit = three_tempo_map().edit().insert_tempo(tempo_point(
            BAR_TICKS,
            second_tempo_clock(),
            240,
        ));
        assert_eq!(
            edit.tempos().len(),
            3,
            "an insert at a tick the list holds replaces that entry"
        );
        assert_eq!(
            edit.tempos()
                .get(1)
                .map(|point| point.tempo().beats_per_minute().numerator()),
            Some(240),
            "the entry at that tick carries the new rate"
        );
        let finished = edit
            .recompute_cached_views()
            .finish()
            .expect("the replaced entry keeps the tick order of the list");
        assert_eq!(
            finished
                .superclock_at(Ticks::new(BAR_TICKS.saturating_add(1_920)))
                .get(),
            second_tempo_clock().saturating_add(1_920_i64.saturating_mul(CLOCK_AT_240)),
            "a tick after the replaced entry reads the new rate"
        );
    }

    #[test]
    fn tempo_map_edit_inserts_a_meter_change_in_the_middle() {
        let edit = three_meter_map().edit().insert_meter(meter_point(
            BAR_TICKS.saturating_mul(2),
            Bbt::ORIGIN,
            3,
        ));
        assert_eq!(
            edit.meters().len(),
            4,
            "the new entry sits between the first entry and the second"
        );
        let edited = edit
            .recompute_cached_views()
            .finish()
            .expect("the recompute states the address that the meter before each entry produces");
        assert_eq!(
            edited.bbt_at(Ticks::new(BAR_TICKS.saturating_mul(2))),
            bbt(3, 1, 0),
            "the new entry starts at bar 3, which the first meter names"
        );
        assert_eq!(
            edited.ticks_at_bbt(bbt(6, 1, 0)),
            Ok(Ticks::new(
                BAR_TICKS
                    .saturating_mul(2)
                    .saturating_add(THREE_FOUR_BAR_TICKS.saturating_mul(3))
            )),
            "the address round trips on the map that the insert and the recompute leave"
        );
    }

    #[test]
    fn sample_rate_lists_the_set_that_is_supported_tests() {
        assert_eq!(
            SampleRate::SUPPORTED.map(|rate| rate.get().get()),
            [44_100, 48_000, 88_200, 96_000, 176_400, 192_000],
            "the list holds the six rates of section 2.1"
        );
        for rate in SampleRate::SUPPORTED {
            assert!(
                rate.is_supported(),
                "every listed rate answers as supported, at {} samples per second",
                rate.get().get()
            );
        }
        assert!(
            !SampleRate::SUPPORTED.contains(&SampleRate::new(nonzero_u32(44_056))),
            "a rate the kernel does not name is outside the list"
        );
    }

    #[test]
    fn sample_compares_with_the_declared_partial_order() {
        let zero = I24::new(0).expect("zero is a 24-bit sample");
        assert!(I24::MIN < zero, "the lowest 24-bit sample is below zero");
        assert!(zero < I24::MAX, "zero is below the highest 24-bit sample");
        assert!(
            I24::MIN < I24::MAX,
            "the sample order is the order of the sample values"
        );
    }

    #[test]
    fn tuplet_splits_a_span_with_its_own_count() {
        let triplet = Tuplet::new(nonzero_u8(3), nonzero_u8(2));
        let parts = triplet.split(Ticks::new(BAR_TICKS));
        assert_eq!(
            parts.as_slice(),
            split_tuplet(Ticks::new(BAR_TICKS), nonzero_u8(3)).as_slice(),
            "the method answers what the function answers for the same count"
        );
        assert_eq!(parts.len(), 3, "a triplet splits a span into three parts");
        let total: i64 = parts.iter().map(|part| part.get()).sum();
        assert_eq!(total, BAR_TICKS, "the parts sum to the span exactly");
    }

    #[test]
    fn split_tuplet_floors_a_negative_span() {
        let parts = split_tuplet(Ticks::new(-1), nonzero_u8(255));
        assert_eq!(parts.len(), 255, "the split holds one entry for each part");
        let total: i64 = parts.iter().map(|part| part.get()).sum();
        assert_eq!(total, -1, "the parts sum to the span exactly");
        assert_eq!(
            parts.first().copied().map(Ticks::get),
            Some(-1),
            "every part but the last takes the floor of the quotient"
        );
        assert_eq!(
            parts.last().copied().map(Ticks::get),
            Some(253),
            "the last part absorbs the remainder and carries the opposite sign"
        );
    }

    #[test]
    fn span_reads_its_own_delta() {
        for delta in [
            Delta::Beats(Ticks::new(960)),
            Delta::Audio(SuperClock::new(CLOCK_AT_120)),
        ] {
            let span = Span::new(Position::Beats(Ticks::ZERO), delta);
            assert_eq!(
                span.delta(),
                delta,
                "the span answers the delta it was built with for {delta:?}"
            );
        }
    }

    #[test]
    fn finite_macro_builds_a_constant() {
        const MIX_SPLIT: Finite = finite!(0.45);
        assert_eq!(
            MIX_SPLIT,
            Finite::new(0.45).expect("the literal is finite"),
            "the macro and the run-time constructor agree"
        );
    }

    #[test]
    fn unit_to_i24_keeps_the_sign_at_both_bounds() {
        for (value, want) in [(-1.0_f32, I24::MIN.get()), (1.0_f32, I24::MAX.get())] {
            assert_eq!(
                unit_to_i24(Unit::clamped(value)).get(),
                want,
                "the full-scale sample {value} answers a value of its own sign"
            );
        }
        assert!(
            unit_to_i24(Unit::clamped(-0.5)).get() < 0,
            "a negative sample never answers a positive full scale"
        );
    }

    #[test]
    fn sample_clock_round_trips_through_the_typed_pair() {
        let rate = SampleRate::new(nonzero_u32(48_000));
        let clock = SuperClock::new(SUPERCLOCK_HZ.get());
        let frames = superclock_to_sample_clock(clock, rate, Rounding::Nearest)
            .expect("one second of superclock ticks is one second of frames");
        assert_eq!(
            frames,
            SampleClock::new(48_000),
            "one second is 48000 frames at 48 kHz"
        );
        assert_eq!(
            sample_clock_to_superclock(frames, rate),
            Ok(clock),
            "the typed pair round trips through the device type"
        );
    }

    #[test]
    fn sample_rate_names_the_six_supported_rates() {
        for rate in [44_100_u32, 48_000, 88_200, 96_000, 176_400, 192_000] {
            assert!(
                SampleRate::new(nonzero_u32(rate)).is_supported(),
                "the kernel names {rate} as supported"
            );
            assert_eq!(
                SUPERCLOCK_HZ.get().rem_euclid(i64::from(rate)),
                0,
                "the superclock rate divides exactly by {rate}"
            );
        }
        assert!(
            !SampleRate::new(nonzero_u32(44_056)).is_supported(),
            "a rate the kernel does not name is not supported"
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

    #[test]
    fn tuplet_holds_its_two_counts() {
        let triplet = Tuplet::new(nonzero_u8(3), nonzero_u8(2));
        assert_eq!(triplet.count().get(), 3, "a triplet writes three notes");
        assert_eq!(
            triplet.over().get(),
            2,
            "a triplet occupies two notes of the same value"
        );
        assert_ne!(
            triplet,
            Tuplet::new(nonzero_u8(2), nonzero_u8(3)),
            "the two counts name different tuplets"
        );
    }

    #[test]
    fn unordered_map_names_both_of_its_causes() {
        let message =
            "the point list is not in tick order, or a point disagrees with the map arithmetic";
        let second_clock = second_tempo_clock();
        let out_of_order = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(BAR_TICKS, second_clock, 90))
            .push_tempo(tempo_point(
                BAR_TICKS.saturating_sub(1),
                second_clock.saturating_sub(CLOCK_AT_90),
                60,
            ))
            .finish()
            .expect_err("a tempo list whose ticks fall is refused");
        assert_eq!(
            out_of_order.to_string(),
            message,
            "the message names the tick-order cause"
        );
        let disagreeing = TempoMapEdit::new()
            .push_tempo(tempo_point(0, 0, 120))
            .push_tempo(tempo_point(1_920, 1, 120))
            .finish()
            .expect_err("a stored clock that the rate before it denies is refused");
        assert_eq!(
            disagreeing.to_string(),
            message,
            "the same message names the arithmetic cause"
        );
        assert_eq!(
            out_of_order, disagreeing,
            "one variant carries both causes, so one message states both"
        );
    }

    #[test]
    fn removing_a_middle_tempo_entry_needs_the_recompute() {
        let map = three_tempo_map();
        let middle = Ticks::new(BAR_TICKS);
        assert_eq!(
            map.edit().remove_tempo_at(middle).finish().err(),
            Some(TimeError::UnorderedMap),
            "a removal alone leaves the later entries at the clock of the map they came from"
        );
        let trimmed = map
            .edit()
            .remove_tempo_at(middle)
            .recompute_cached_views()
            .finish()
            .expect("the recompute states the clock the remaining rates produce");
        assert_eq!(
            trimmed.tempos().len(),
            2,
            "the middle entry is gone from the list"
        );
        let last_tick = BAR_TICKS.saturating_mul(2);
        assert_eq!(
            trimmed.superclock_at(Ticks::new(last_tick)).get(),
            last_tick.saturating_mul(CLOCK_AT_120),
            "the second entry sits where the rate of the first entry carries it"
        );
        assert_eq!(
            trimmed
                .superclock_at(Ticks::new(last_tick.saturating_add(1_920)))
                .get(),
            last_tick
                .saturating_mul(CLOCK_AT_120)
                .saturating_add(1_920_i64.saturating_mul(CLOCK_AT_90)),
            "a tick after the second entry reads the rate of that entry"
        );
    }

    #[test]
    fn removing_a_middle_meter_entry_needs_the_recompute() {
        let map = three_meter_map();
        let middle = Ticks::new(BAR_TICKS.saturating_mul(4).saturating_add(1_920));
        assert_eq!(
            map.edit().remove_meter_at(middle).finish().err(),
            Some(TimeError::UnorderedMap),
            "a removal alone leaves the later entries at the address of the map they came from"
        );
        let trimmed = map
            .edit()
            .remove_meter_at(middle)
            .recompute_cached_views()
            .finish()
            .expect("the recompute states the address the remaining meters produce");
        assert_eq!(
            trimmed.meters().len(),
            2,
            "the middle entry is gone from the list"
        );
        let last_tick = middle
            .get()
            .saturating_add(THREE_FOUR_BAR_TICKS.saturating_mul(3));
        assert_eq!(
            trimmed.bbt_at(Ticks::new(last_tick)),
            bbt(7, 3, 0),
            "the four-four meter of the first entry names tick {last_tick} as bar 7 beat 3"
        );
        assert_eq!(
            trimmed.ticks_at_bbt(bbt(7, 3, 0)),
            Ok(Ticks::new(last_tick)),
            "the address round trips on the map the recompute leaves"
        );
    }

    #[test]
    fn finite_to_f32_saturates_above_the_f32_range() {
        let inside = Finite::new(1.5).expect("the sample is finite");
        assert_eq!(
            finite_to_f32_saturating(inside).to_bits(),
            1.5_f32.to_bits(),
            "a value the f32 range holds narrows exactly"
        );
        for (raw, positive) in [(f64::MAX, true), (f64::MIN, false)] {
            let value = Finite::new(raw).expect("a bound of the f64 range is finite");
            let narrowed = finite_to_f32_saturating(value);
            assert!(
                narrowed.is_infinite(),
                "a magnitude above the f32 range saturates to an infinity for {raw}"
            );
            assert!(!narrowed.is_nan(), "the narrowing answers no NaN for {raw}");
            assert_eq!(
                narrowed.is_sign_positive(),
                positive,
                "the saturation keeps the sign of {raw}"
            );
        }
    }

    #[test]
    fn from_finite_const_canonicalizes_a_negative_zero() {
        const NEGATIVE_ZERO: Finite = finite!(-0.0);
        assert_eq!(
            NEGATIVE_ZERO,
            Finite::ZERO,
            "the compile-time constructor writes the canonical zero"
        );
        assert!(
            NEGATIVE_ZERO.get().is_sign_positive(),
            "the stored constant carries a positive sign"
        );
    }

    #[test]
    fn checked_from_i64_refuses_both_bounds() {
        for bound in [i64::MIN, i64::MAX] {
            assert_eq!(
                Ticks::checked_from_i64(bound).err(),
                Some(TimeError::Overflow),
                "a tick magnitude of {bound} reads as a saturated one, so it is refused"
            );
            assert_eq!(
                SuperClock::checked_from_i64(bound).err(),
                Some(TimeError::Overflow),
                "a superclock magnitude of {bound} reads as a saturated one, so it is refused"
            );
        }
        assert_eq!(
            Ticks::checked_from_i64(1_920),
            Ok(Ticks::new(1_920)),
            "a tick magnitude inside the range is accepted"
        );
        assert_eq!(
            SuperClock::checked_from_i64(i64::MIN.saturating_add(1)),
            Ok(SuperClock::new(i64::MIN.saturating_add(1))),
            "the magnitude beside the lower bound is accepted"
        );
    }

    #[test]
    fn channel_count_refuses_zero() {
        assert_eq!(
            ChannelCount::new(0),
            None,
            "a chunk size of zero would end the process, so the type holds no zero"
        );
        let stereo = ChannelCount::new(2).expect("two channels is a count");
        assert_eq!(
            stereo.get().get(),
            2,
            "the count answers the value it was built with"
        );
    }

    #[test]
    fn unit_new_refuses_a_value_outside_the_range() {
        for raw in [-1.0_f32, -0.5, 0.0, 0.5, 1.0] {
            assert!(Unit::new(raw).is_some(), "the unit range holds {raw}");
        }
        for raw in [-1.5_f32, 1.5, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(Unit::new(raw).is_none(), "the unit range refuses {raw}");
        }
        assert_eq!(
            Unit::new(0.25).map(Unit::get).map(f32::to_bits),
            Some(0.25_f32.to_bits()),
            "the constructor stores the value it was given"
        );
    }

    #[test]
    fn the_small_newtypes_carry_their_value() {
        assert_eq!(
            FrameCount::new(512).get(),
            512,
            "a frame count answers its value"
        );
        assert_eq!(
            FrameCount::default().get(),
            0,
            "no frames is a count and not an absence"
        );
        assert_eq!(BarCount::new(2).get(), 2, "a bar count answers its value");
        assert_eq!(
            SampleClock::new(-48_000).get(),
            -48_000,
            "a sample counter answers its value"
        );
        assert_eq!(
            ChannelIndex::new(1).get(),
            1,
            "a channel index answers its value"
        );
        assert_eq!(
            UnixSeconds::new(-5).get(),
            -5,
            "an instant before the epoch answers its value"
        );
        assert_eq!(
            SchemaVersion::new(3).get(),
            3,
            "a schema number answers its value"
        );
        assert!(
            SchemaVersion::new(2) < SchemaVersion::new(3),
            "the order lets a reader migrate an older document and refuse a newer one"
        );
        let gain = GainDb::new(Finite::new(-6.0).expect("the gain is finite"));
        assert_eq!(
            gain.get().get().to_bits(),
            (-6.0_f64).to_bits(),
            "a gain answers the decibels it was built with"
        );
    }

    /// Assert that one value returns equal from a JSON round trip.
    fn assert_json_round_trip<T>(value: &T, label: &str)
    where
        T: serde::Serialize + serde::de::DeserializeOwned + PartialEq + core::fmt::Debug,
    {
        let text = serde_json::to_string(value).expect("the value serializes");
        let back: T = serde_json::from_str(&text).expect("the text deserializes");
        assert_eq!(&back, value, "a JSON round trip returns an equal {label}");
    }

    #[test]
    fn the_small_newtypes_round_trip_through_json() {
        assert_json_round_trip(&Ticks::new(-1_920), "tick magnitude");
        assert_json_round_trip(&SuperClock::new(73_500), "superclock magnitude");
        assert_json_round_trip(&FrameCount::new(512), "frame count");
        assert_json_round_trip(&BarCount::new(2), "bar count");
        assert_json_round_trip(&SampleClock::new(48_000), "sample counter");
        assert_json_round_trip(&ChannelIndex::new(1), "channel index");
        assert_json_round_trip(&UnixSeconds::new(-5), "instant");
        assert_json_round_trip(&SchemaVersion::new(3), "schema number");
        assert_json_round_trip(&SampleRate::new(nonzero_u32(48_000)), "sample rate");
        assert_json_round_trip(
            &ChannelCount::new(2).expect("two channels is a count"),
            "channel count",
        );
        assert_json_round_trip(
            &GainDb::new(Finite::new(-6.0).expect("the gain is finite")),
            "gain",
        );
        assert_json_round_trip(&Tuplet::new(nonzero_u8(3), nonzero_u8(2)), "tuplet");
    }
}
