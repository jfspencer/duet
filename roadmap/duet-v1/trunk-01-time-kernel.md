---
id: T1
line: trunk
depends_on: [M0]
write_scope:
  - crates/duet-time/Cargo.toml
  - crates/duet-time/src/lib.rs
  - crates/duet-time/src/units.rs
  - crates/duet-time/src/convert.rs
  - crates/duet-time/src/position.rs
  - crates/duet-time/src/span.rs
  - crates/duet-time/src/finite.rs
  - crates/duet-time/src/bbt.rs
  - crates/duet-time/src/tempo.rs
  - crates/duet-time/src/tuplet.rs
  - crates/duet-time/src/error.rs
  - crates/duet-time/tests/kernel.rs
  - crates/duet-time/tests/proptest_large.rs
  - Cargo.lock
parallelism: serial-only: T1 is the only line chunk of phase 0, and section 13.4 puts M0 before it.
completion: "cargo nextest run -p duet-time --no-tests=fail passes; cargo clippy -p duet-time --all-targets -- -D warnings is clean; commit SHA on a branch chunk/t1-time-kernel"
---

# T1: The time kernel

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk builds `duet-time`, the one crate every other crate of the plan reaches. It delivers the
two unit types, the one conversion module, the `Finite` newtype, the five kernel counts,
`SchemaVersion`, `Ratio`, `NoteValue`, `Tuplet`, the tempo map, the tuplet rounding rule, and the
nightly proptest target. It implements architecture sections 2.1, 2.2, 2.3, 2.4, 2.5, 2.6, 2.6a,
2.7, 2.8, 2.9, 2.10, the `duet-time` block of section 1.6, section 15.1, and ADR
`adr/0001-time-kernel-representation.md`. Section 13.1 states the goal, the write scope, and the
Completion command. Section 14 selects `proptest_large` for the `soak.yml` workflow, so this chunk
writes that target and marks it `#[ignore]`.

Chunk M0 creates the `crates/duet-time` skeleton (SM1 rule 2), so the crate root and the member
manifest already exist. This chunk adds the module files, the dependency entries, and the code.
Dispatch: **Duet Engineer**.

## Files

| Path | Action |
|---|---|
| `crates/duet-time/Cargo.toml` | modify (add `[dependencies]` and `[dev-dependencies]`) |
| `crates/duet-time/src/lib.rs` | modify (add the `mod` and `pub use` lines) |
| `crates/duet-time/src/units.rs` | create |
| `crates/duet-time/src/convert.rs` | create |
| `crates/duet-time/src/position.rs` | create |
| `crates/duet-time/src/span.rs` | create |
| `crates/duet-time/src/finite.rs` | create |
| `crates/duet-time/src/bbt.rs` | create |
| `crates/duet-time/src/tempo.rs` | create |
| `crates/duet-time/src/tuplet.rs` | create |
| `crates/duet-time/src/error.rs` | create |
| `crates/duet-time/tests/kernel.rs` | create |
| `crates/duet-time/tests/proptest_large.rs` | create |
| `Cargo.lock` | modify (SM5 rule 2) |

## Types and signatures

Every signature below is copied from the architecture section the heading names. A chunk that needs
a different shape reports the discrepancy instead of inventing one (SM0).

### `duet_time::units` (section 2.1, section 1.6, section 15.1)

```rust
/// A magnitude in the beat domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Ticks(i64);

/// A magnitude in the audio domain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SuperClock(i64);

/// A device sample rate. The type makes a zero divisor impossible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleRate(NonZeroU32);

/// A sample counter at the device edge, in frames since the stream opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SampleClock(i64);

/// A count of audio frames.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FrameCount(u32);

/// A count of bars, which the count-in uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BarCount(u16);

/// A channel index inside one stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelIndex(u16);

/// A count of channels that cannot be zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ChannelCount(NonZeroU16);

impl ChannelCount {
    /// One count, or `None` when `value` is zero.
    pub const fn new(value: u16) -> Option<Self>;

    /// The count as a non-zero number. A chunk size is a `usize`, so a caller
    /// writes `usize::from(count.get().get())`, which is total and never
    /// zero (section 5.11, critic C20-W2).
    pub const fn get(self) -> NonZeroU16;
}

/// Seconds since the Unix epoch. A calibration records when it was measured.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct UnixSeconds(i64);

/// A document schema number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct SchemaVersion(u32);

impl SchemaVersion {
    /// Build a version at compile time, for the constants of section 1.6.
    pub const fn new(value: u32) -> Self;
}

/// A gain in decibels. Every stored gain on a region and on a strip is one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct GainDb(Finite);

/// A 24-bit signed sample, held in the low three bytes of an `i32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct I24(i32);
```

`GainDb` and `I24` each derive the order. Architecture section 15.1 states the reason: a caller that
sorts a gain list or a sample list can reach the order no other way, because no later chunk writes
`duet-time`. `Finite` carries a total order through `f64::total_cmp`, and the inner value of `I24`
is an `i32`, so each derive agrees with the type below it (chunk T1, 2026-09-22).

The constants below are the `duet-time` block of section 1.6. They live in `units.rs`.

```rust
/// A non-zero constant, built at compile time.
///
/// `NonZeroI64::new` returns an `Option`, and `expect` is denied. A `const fn`
/// match gives the same compile-time check with a total fallback arm.
const fn non_zero(value: i64) -> NonZeroI64 {
    match NonZeroI64::new(value) {
        Some(non_zero) => non_zero,
        None => NonZeroI64::MIN,
    }
}

/// Ticks per quarter note in the beat domain (B40).
pub const TICKS_PER_QUARTER: NonZeroI64 = non_zero(1_920);
/// Ticks per second in the audio domain (B41).
pub const SUPERCLOCK_HZ: NonZeroI64 = non_zero(282_240_000);
/// Strips one project may hold, and the meter array length (B86).
pub const MAX_STRIPS: usize = 48;
/// Automatable parameters one project holds (B90).
pub const MAX_PARAMS: usize = 2_048;
/// Pre-fader and post-fader slots per chain (B45).
pub const MAX_SLOTS: usize = 8;
/// Aux sends per chain (B46).
pub const MAX_SENDS: usize = 8;
/// Per-slot meter cells one `SlotMeterSnapshot` carries. It is arithmetic
/// over three constants and never a literal, so a change to any one of them
/// moves it and DR3 holds (B133, section 7.3).
pub const MAX_SLOT_METERS: usize = MAX_STRIPS * 2 * MAX_SLOTS;
/// The MIDI input ports one engine binds at once (B139).
pub const MAX_BOUND_PORTS: usize = 16;
```

### `duet_time::error` (section 15.1)

```rust
/// Every way the time kernel refuses.
///
/// It derives serde, because `ScoreError::Time` and `SessionError::Time`
/// wrap it and both reach the transport inside `GatewayError` (section 15.5).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum TimeError {
    Overflow,
    NotRepresentable,
    NotFinite,
    UnorderedMap,
    NoFirstPoint,
    BbtOutOfRange,
}
```

### `duet_time::finite` (section 2.6a)

```rust
/// A finite `f64` with a canonical bit pattern.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(try_from = "f64")]
pub struct Finite(f64);

impl Finite {
    /// Build a finite value, or `None` for a NaN or an infinity.
    /// A negative zero becomes a positive zero.
    pub fn new(value: f64) -> Option<Self> {
        value.is_finite().then_some(Self(value + 0.0))
    }

    /// Build a finite value at compile time, for the constants of section 1.6.
    ///
    /// # Panics
    /// It panics when `value` is a NaN or an infinity. House form 5 binds
    /// every caller to a `const` item, so the assertion runs at compile time
    /// and no binary carries the panic (critic W-9).
    pub const fn from_finite_const(value: f64) -> Self {
        assert!(value.is_finite(), "a `Finite` constant must be finite");
        Self(value + 0.0)
    }

    /// The inner value. It is finite, and it is never a negative zero.
    pub const fn get(self) -> f64 { self.0 }

    /// Zero, which every default uses.
    pub const ZERO: Self = Self(0.0);
}

impl TryFrom<f64> for Finite {
    type Error = TimeError;
    fn try_from(value: f64) -> Result<Self, TimeError> {
        Self::new(value).ok_or(TimeError::NotFinite)
    }
}

impl PartialEq for Finite {
    fn eq(&self, other: &Self) -> bool { self.0.to_bits() == other.0.to_bits() }
}
impl Eq for Finite {}
impl Hash for Finite {
    fn hash<H: Hasher>(&self, state: &mut H) { self.0.to_bits().hash(state); }
}
impl Ord for Finite {
    fn cmp(&self, other: &Self) -> Ordering { self.0.total_cmp(&other.0) }
}
impl PartialOrd for Finite {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { Some(self.cmp(other)) }
}
```

The five trait impls are hand written. A derive of any one of them over an `f64` field does not
compile. Section 2.6a states the reason for `total_cmp`.

### `duet_time::convert` (section 2.3)

```rust
/// A sample value in the closed range -1.0 to 1.0.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct Unit(f32);

impl Unit {
    /// Clamp a float into the unit range. A non-finite value becomes zero.
    pub fn clamped(value: f32) -> Self;

    /// The inner value, always in the closed range -1.0 to 1.0.
    pub fn get(self) -> f32;
}

/// How an integer conversion resolves a remainder.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding { Down, Nearest, Up }

/// Multiply and then divide with a 128-bit intermediate.
///
/// # Errors
/// Returns `TimeError::Overflow` when the result leaves the `i64` range.
pub fn muldiv(value: i64, numerator: i64, denominator: NonZeroI64, rounding: Rounding)
    -> Result<i64, TimeError>;

/// A 16-bit sample as a float. Lossless; uses `f32::from`, no suppression.
pub fn i16_to_f32(sample: i16) -> Unit;

/// A 24-bit sample as a float. Lossless; 24 bits fit the f32 mantissa.
pub fn i24_to_f32(sample: I24) -> Unit;

/// A 32-bit sample as a float. Lossy below the 24th bit.
pub fn i32_to_f32(sample: i32) -> Unit;

/// A unit sample as a 24-bit integer. The type carries the range.
pub fn unit_to_i24(value: Unit) -> I24;

/// A unit sample as a 32-bit integer. The type carries the range.
pub fn unit_to_i32(value: Unit) -> i32;

/// A double as a single.
///
/// # Errors
/// Returns `TimeError::NotRepresentable` when the value is not finite, or
/// when its magnitude leaves the `f32` range. The check runs before the cast.
pub fn f64_to_f32(value: f64) -> Result<f32, TimeError>;

/// A tick count as a double.
///
/// # Errors
/// Returns `TimeError::NotRepresentable` at or above the magnitude B44,
/// where an `f64` stops holding every integer.
pub fn ticks_to_f64(ticks: Ticks) -> Result<f64, TimeError>;

/// A superclock count as a sample count at a rate.
///
/// # Errors
/// Returns `TimeError::Overflow` when the result leaves the `i64` range.
pub fn superclock_to_samples(clock: SuperClock, rate: SampleRate, rounding: Rounding)
    -> Result<i64, TimeError>;

/// A sample count at a rate as a superclock count.
///
/// # Errors
/// Returns `TimeError::Overflow` when the result leaves the `i64` range.
pub fn samples_to_superclock(samples: i64, rate: SampleRate) -> Result<SuperClock, TimeError>;

/// A finite double as a single, with no error path.
#[must_use]
pub fn finite_to_f32_saturating(value: Finite) -> f32;
```

Exactly seven functions of this module carry a suppression: `i24_to_f32`, `i32_to_f32`,
`unit_to_i24`, `unit_to_i32`, `f64_to_f32`, `ticks_to_f64`, and `finite_to_f32_saturating`.
Appendix B.1 gives the lint list and the exact reason text of each one. `i16_to_f32` uses
`f32::from` and carries none. `convert.rs` is the one file in the whole workspace that may hold an
`#[expect(clippy::as_conversions, ...)]`.

### `duet_time::position` (section 2.5, section 2.7)

```rust
/// Which unit a value counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeDomain { Beats, Audio }

/// A point on the timeline, measured from timeline zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Position {
    Beats(Ticks),
    Audio(SuperClock),
}

/// A signed magnitude in one domain. Equality is structural, as above.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Delta {
    Beats(Ticks),
    Audio(SuperClock),
}

impl Position {
    pub fn domain(self) -> TimeDomain;
    pub fn in_beats(self, map: &TempoMap) -> Ticks;
    pub fn in_audio(self, map: &TempoMap) -> SuperClock;
    pub fn distance(self, other: Self, map: &TempoMap) -> Span;
    pub fn earlier(self, span: Span, map: &TempoMap) -> Self;
    pub fn later(self, span: Span, map: &TempoMap) -> Self;
}
```

`Position` and `Delta` implement no `PartialOrd` and no `Ord`. Section 2.5 states the reason.

### `duet_time::span` (section 2.6)

```rust
/// A distance together with the position where it starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Span { origin: Position, delta: Delta }

impl Span {
    pub fn new(origin: Position, delta: Delta) -> Self;
    pub fn origin(self) -> Position;
    pub fn end(self, map: &TempoMap) -> Position;
    pub fn in_beats(self, map: &TempoMap) -> Ticks;
    pub fn in_audio(self, map: &TempoMap) -> SuperClock;
}
```

### `duet_time::bbt` (section 2.8)

```rust
/// A bar, beat, and tick address. Bars and beats are one-based.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Bbt { bar: NonZeroU32, beat: NonZeroU16, tick: u16 }
```

### `duet_time::tempo` (section 2.9, section 2.7, section 15.1)

```rust
/// An exact ratio. The tempo holds one so that no float enters the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ratio { numerator: i64, denominator: NonZeroI64 }

/// A written note value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum NoteValue { Whole, Half, Quarter, Eighth, Sixteenth, ThirtySecond }

/// A tempo, held as an exact ratio so that no float enters the kernel.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Tempo { beats_per_minute: Ratio, beat_unit: NoteValue, ramped: bool }

/// A time signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Meter { beats_per_bar: NonZeroU8, beat_unit: NoteValue }

/// One tempo entry. It carries its own position in the two views the beat
/// and audio queries read, so a lookup never calls back into the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TempoPoint { ticks: Ticks, clock: SuperClock, tempo: Tempo }

/// One meter entry, with the two views the bar queries read and the same
/// serde rule.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeterPoint { ticks: Ticks, bbt: Bbt, meter: Meter }

/// A sorted tempo and meter map.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TempoMap { tempos: Vec<TempoPoint>, meters: Vec<MeterPoint> }

/// A checked builder for a tempo map. It validates the sort order and the
/// first-point rule, and `finish` produces the map.
#[derive(Debug, Clone, Default)]
pub struct TempoMapEdit { tempos: Vec<TempoPoint>, meters: Vec<MeterPoint> }

impl TempoMap {
    pub fn superclock_at(&self, ticks: Ticks) -> SuperClock;
    pub fn ticks_at(&self, clock: SuperClock) -> Ticks;
    pub fn bbt_at(&self, ticks: Ticks) -> Bbt;
    pub fn ticks_at_bbt(&self, bbt: Bbt) -> Result<Ticks, TimeError>;
    pub fn convert(&self, position: Position, target: TimeDomain) -> Position;

    /// A builder that checks the sort order and the first-point rule.
    pub fn edit(&self) -> TempoMapEdit;

    /// Order two positions, across domains when they differ.
    pub fn cmp(&self, left: Position, right: Position) -> core::cmp::Ordering;

    /// Test two positions for equality, across domains when they differ.
    pub fn same_instant(&self, left: Position, right: Position) -> bool;
}
```

The method is named `same_instant` and not `eq`, because `TempoMap` derives `PartialEq` and
`clippy::same_name_method` is denied. Architecture section 2.7 states the reason in full.

`TempoMapEdit::finish` returns `Result<TempoMap, TimeError>`. It returns `TimeError::UnorderedMap`
for a point list that is not sorted, and `TimeError::NoFirstPoint` for a list whose first point does
not sit at the origin in the tick, the clock, and the address (section 15.1, section 2.9). The
chunk sentence named the tick alone. A first point at tick zero with a non-zero clock makes
`ticks_at(SuperClock::ZERO)` answer a non-zero tick, so the map would be inconsistent from its
first entry, and architecture section 2.5 measures a `Position` from timeline zero in BOTH domains.
The rule refuses more lists and accepts none the chunk refuses, and it uses the declared variant
(chunk T1, 2026-09-22).

### `duet_time::tuplet` (section 2.4)

```rust
/// A tuplet: `count` notes written in the time of `over` of the same value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Tuplet { count: NonZeroU8, over: NonZeroU8 }

/// Split a span into `parts` tick counts that sum to the span exactly.
///
/// The last part absorbs the remainder.
pub fn split_tuplet(span: Ticks, parts: NonZeroU8) -> SmallVec<[Ticks; 13]>;
```

The inline capacity is B113, which is 13, the divisor maximum of B43. Section 2.4 states the
rounding rule: each of the first `parts - 1` entries takes `span.div_euclid(parts)` ticks, and the
last entry takes the remainder.

## Steps

1. Read `crates/duet-time/Cargo.toml` and `crates/duet-time/src/lib.rs`. Confirm that M0 created
   both, that the manifest holds `[lints] workspace = true`, a `description`, and every
   `*.workspace = true` package field, and that it holds no `[dependencies]` section. Confirm that
   `src/lib.rs` holds the `//!` crate documentation and `#![forbid(unsafe_code)]`. Report a
   discrepancy and stop if any one of these is false.
2. Add the dependency entries to `crates/duet-time/Cargo.toml`, in one commit with the code that
   uses them (SM1). The section 1.2 row for `duet-time` names three crates, and Appendix B.3 names
   the dev-dependency.

   ```toml
   [dependencies]
   serde = { workspace = true }
   smallvec = { workspace = true }
   thiserror = { workspace = true }

   [dev-dependencies]
   proptest = { workspace = true }
   ```

3. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains the four edges.
4. Create `src/error.rs` with `TimeError`. Add `mod error;` and `pub use error::TimeError;` to
   `src/lib.rs`. Run `cargo check -p duet-time`. Expected result: it succeeds.
5. Write the failing `Finite` proptest in `crates/duet-time/tests/kernel.rs`. It asserts the five
   properties of section 2.6a. Run `cargo nextest run -p duet-time -E 'test(finite)' --no-tests=fail`.
   Expected result: the run fails, because `finite.rs` does not exist.
6. Create `src/finite.rs` with the `Finite` declaration, the two constructors, `get`, `ZERO`, and the
   five hand-written trait impls of section 2.6a. Add `mod finite;` and `pub use finite::Finite;` to
   `src/lib.rs`. Run the same command. Expected result: the run passes.
7. Create `src/units.rs` with the twelve unit types, the two `impl` blocks, and the constant block of
   section 1.6. Add `mod units;` and the `pub use` line to `src/lib.rs`. `GainDb` names `Finite`, so
   `units.rs` follows `finite.rs`. Run `cargo check -p duet-time`. Expected result: it succeeds.
8. Write the failing `muldiv` proptest in `tests/kernel.rs`. It compares `muldiv` with an `i128`
   reference over every rounding mode. Run
   `cargo nextest run -p duet-time -E 'test(muldiv)' --no-tests=fail`. Expected result: the run fails.
9. Create `src/convert.rs` with `Unit`, `Rounding`, and the eleven functions of section 2.3. Write
   each of the seven suppressed sites with one `#[expect(...)]` attribute that names every lint of
   that site and carries the exact reason text of Appendix B.1. Add `pub mod convert;` to
   `src/lib.rs`. Run the same command. Expected result: the run passes.
10. Run `cargo xtask check-conversions`. Expected result: exit 0. The guard proves that no cast and
    no cast suppression sits outside `crates/duet-time/src/convert.rs` (CG3, CG6, CG7).
11. Write the failing round-trip tests for `i16_to_f32`, `i24_to_f32`, `i32_to_f32`, `unit_to_i24`,
    and `unit_to_i32` in `tests/kernel.rs`, and the failing range tests for `f64_to_f32` and
    `ticks_to_f64`. Run `cargo nextest run -p duet-time --no-tests=fail`. Expected result: the new
    tests fail.
12. Complete the seven function bodies so that every property holds. Run the same command. Expected
    result: every test passes.
13. Create `src/bbt.rs` with `Bbt`, then `src/tempo.rs` with `Ratio`, `NoteValue`, `Tempo`, `Meter`,
    `TempoPoint`, `MeterPoint`, `TempoMap`, `TempoMapEdit`, and the `TempoMap` methods. Add both
    `mod` lines. Every method that crosses a domain takes `&TempoMap` as an argument (section 2.10).
    Run `cargo check -p duet-time`. Expected result: it succeeds.
14. Create `src/position.rs` with `TimeDomain`, `Position`, `Delta`, and the six `Position` methods,
    then `src/span.rs` with `Span` and its five methods. Add both `mod` lines. Run
    `cargo check -p duet-time`. Expected result: it succeeds.
15. Write the failing tempo-map tests in `tests/kernel.rs`: a clock round trip at 44100, 48000,
    88200, 96000, 176400, and 192000 samples per second, a `TempoMapEdit::finish` refusal for an
    unsorted list, and a refusal for a list with no point at tick zero. Run
    `cargo nextest run -p duet-time --no-tests=fail`. Expected result: the new tests fail.
16. Complete the tempo-map bodies. Run the same command. Expected result: every test passes.
17. Write the failing tuplet sum test in `tests/kernel.rs`. It walks every divisor from 2 to 13 and
    every span from 1 to 7680 ticks (B43), and it asserts that the parts sum to the span. Run
    `cargo nextest run -p duet-time -E 'test(tuplet)' --no-tests=fail`. Expected result: the run
    fails.
18. Create `src/tuplet.rs` with `Tuplet` and `split_tuplet`. Add `mod tuplet;` and the `pub use`
    line. Run the same command. Expected result: the run passes.
19. Create `crates/duet-time/tests/proptest_large.rs`. It holds the same four proptest properties as
    `tests/kernel.rs` at 1,000,000 cases (B78), and every test in it carries `#[ignore]`. Section 14
    selects this target for `soak.yml`, which chunk M7 writes. Run
    `cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large)' --no-tests=fail`.
    Expected result: the run passes.
20. Run `cargo build --workspace` again, then `git add` the write scope and `git commit`. The native
    hook runs `scripts/dod.sh`.

## Tests

Every assert carries a message. `tests/kernel.rs` and `tests/proptest_large.rs` are integration
targets, so each file wraps its tests in a `#[cfg(test)] mod tests` block
(`clippy::tests_outside_test_module` is denied). Author guidance: the `test-author` skill.

| Test | What it asserts | Where it lives |
|---|---|---|
| `finite_rejects_non_finite` | `Finite::new` returns `None` for every NaN and for both infinities | `tests/kernel.rs` |
| `finite_canonicalizes_negative_zero` | `Finite::new(-0.0)` and `Finite::new(0.0)` are equal and hash equal | `tests/kernel.rs` |
| `finite_equality_matches_bits` | For two finite inputs, equality holds exactly when the bits match after the negative-zero rule | `tests/kernel.rs` |
| `finite_order_matches_total_cmp` | `x.cmp(&y)` equals `a.total_cmp(&b)` for every finite pair | `tests/kernel.rs` |
| `finite_json_round_trip` | A `serde_json` round trip returns an equal value; a `NaN` token, a `null`, and a `-0.0` give a rejection or the canonical value | `tests/kernel.rs` |
| `muldiv_matches_i128_reference` | `muldiv` equals an `i128` reference for every rounding mode and never overflows | `tests/kernel.rs` |
| `convert_i16_round_trip` | `i16_to_f32` then `unit_to_i32` keeps the value inside the stated bound | `tests/kernel.rs` |
| `convert_i24_round_trip` | `i24_to_f32` then `unit_to_i24` drifts by at most one 24-bit step over the whole range, and returns the same `I24` for every sample of magnitude 2^22 or less | `tests/kernel.rs` |
| `convert_i32_round_trip` | `i32_to_f32` then `unit_to_i32` keeps the value inside the stated bound | `tests/kernel.rs` |
| `convert_f64_out_of_range_errors` | `f64_to_f32` and `ticks_to_f64` return an error for every out-of-range input | `tests/kernel.rs` |
| `clock_round_trip_every_rate` | `samples_to_superclock` then `superclock_to_samples` returns the same sample count at each of the six supported rates | `tests/kernel.rs` |
| `tempo_map_refuses_unsorted` | `TempoMapEdit::finish` returns `TimeError::UnorderedMap` | `tests/kernel.rs` |
| `tempo_map_refuses_no_first_point` | `TempoMapEdit::finish` returns `TimeError::NoFirstPoint` | `tests/kernel.rs` |
| `tuplet_parts_sum_to_span` | For every divisor 2 to 13 and every span 1 to 7680 ticks, the parts sum to the span exactly | `tests/kernel.rs` |
| `proptest_large_*` | The same four properties at 1,000,000 cases; each test carries `#[ignore]` | `tests/proptest_large.rs` |

Proptest strategies: `any::<f64>()` for the `Finite` properties, `any::<i64>()` three times for
`muldiv`, `any::<i32>()` narrowed to the `I24` range with `prop_map` for the 24-bit round trip, and
`1_i64..=7_680` with `2_u8..=13` for the tuplet sum.

**The 24-bit round trip drifts by one step, and Appendix B.1 is the reason.** The two reason texts
of that appendix are mandatory character for character, and they fix both scale factors. The
`i24_to_f32` text says the result "is in the unit range", which holds only for a divisor of 2^23,
because a divisor of 2^23 - 1 sends `I24::MIN` outside the range and forces a clamp. The
`unit_to_i24` text names the scale factor 2^23 - 1 and says the product "fits `I24` with no clamp",
which holds only for that factor, because 2^23 sends a unit sample of 1.0 one step above `I24::MAX`.
The two factors therefore differ by one part in 8,388,608, and no rounding mode makes the round trip
exact above a half-scale sample. The chunk row above states the property that holds. A change to
either factor needs a change to Appendix B.1 first, which is an Architect decision and not an
implementation decision (T1, 2026-09-22).

## Verification

```
cargo nextest run -p duet-time --no-tests=fail
cargo clippy -p duet-time --all-targets -- -D warnings
cargo xtask check-conversions
```

Expected output: the first command reports every test of `tests/kernel.rs` as passed and reports no
filter miss. The second command prints no warning. The third command exits 0.

Then commit on a branch named `chunk/t1-time-kernel`. The native git hook runs `scripts/dod.sh`, and
the commit lands only when every gate passes.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
