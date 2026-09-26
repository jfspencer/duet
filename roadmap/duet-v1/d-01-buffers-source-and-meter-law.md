---
id: D1
line: D
depends_on: [M1, T1]
write_scope:
  - crates/bc_audio/duet-dsp/lang_rust/Cargo.toml
  - crates/bc_audio/duet-dsp/lang_rust/src/lib.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/source.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/fft.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/align.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/gain.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/filter.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/meter.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/meter_law.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/slot.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/voice.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/pool.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/compressor.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/gate.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/deesser.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/limiter.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/delay.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/reverb.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/format.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/builder.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/reader.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-dsp -E 'test(align) + test(meter_law) + test(fader_taper)' --no-tests=fail"
---

# D1: Buffers, `SampleSource`, the transforms, the gain stages, and the meter law

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk opens line D over the crate `duet-dsp`. Chunk M1 creates the crate skeleton in
phase 1, so `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml` and `crates/bc_audio/duet-dsp/lang_rust/src/lib.rs` exist before this chunk
starts and this chunk modifies both. Every other file of the write scope does not exist yet, and
this chunk creates it. The chunk declares the block helpers, `SampleSource`, the transform module,
the loopback alignment, the gain and pan stages, the biquad filters, `OverMark` and the meter value
types, and the `meter_law` module with its five functions. It implements architecture sections 5.10,
7.3, 15.2, and 1.6, and it carries the model half of the product story M-05.

Section 13.2 gives this chunk the second duty of SM2: it creates every module file of line D at every
depth as a stub, so chunks D2 and D3 modify a stub and create no source file.

## Files

- `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses.
- `crates/bc_audio/duet-dsp/lang_rust/src/lib.rs` — modify. Add one `mod` line per top-level module, and the five
  shared constants of section 1.6 that sit at the crate root.
- `crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs` — create and fill. Block helpers that allocate nothing, and
  `PoolSlot`, the pool handle index.
- `crates/bc_audio/duet-dsp/lang_rust/src/source.rs` — create and fill. `SampleSource`.
- `crates/bc_audio/duet-dsp/lang_rust/src/fft.rs` — create and fill. The real transform over `realfft` and `rustfft`.
- `crates/bc_audio/duet-dsp/lang_rust/src/align.rs` — create and fill. The loopback cross-correlation.
- `crates/bc_audio/duet-dsp/lang_rust/src/gain.rs` — create and fill. `GainState`, the gain ramp, and the pan law.
- `crates/bc_audio/duet-dsp/lang_rust/src/filter.rs` — create and fill. `BiquadState` and `EqualizerState`.
- `crates/bc_audio/duet-dsp/lang_rust/src/meter.rs` — create and fill. `MeterLaw`, `MeterReading`, `MeterState`,
  `OverMark`, `HeldMarks`, `MeterView`, `SlotMeasure`.
- `crates/bc_audio/duet-dsp/lang_rust/src/meter_law.rs` — create and fill. The five scale functions and the eleven
  scale constants.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics.rs` — create. The sibling file of the `dynamics/` directory, which
  `clippy::mod_module_files` requires (SM2). It holds the six `mod` lines and nothing else.
- `crates/bc_audio/duet-dsp/lang_rust/src/slot.rs` — create as a stub. Chunk D3 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/voice.rs` — create as a stub. Chunk D3 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/peaks.rs` — create. The sibling file of the `peaks/` directory. It holds the
  three `mod` lines and nothing else.
- `crates/bc_audio/duet-dsp/lang_rust/src/pool.rs` — create as a stub. Chunk D3 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/compressor.rs` — create as a stub. Chunk D2 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/gate.rs` — create as a stub. Chunk D2 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/deesser.rs` — create as a stub. Chunk D2 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/limiter.rs` — create as a stub. Chunk D2 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/delay.rs` — create as a stub. Chunk D3 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/reverb.rs` — create as a stub. Chunk D3 fills it.
- `crates/bc_audio/duet-dsp/lang_rust/src/peaks/format.rs` — create and fill. `PeakBin`, `PyramidHeader`, and the level
  layout of B68.
- `crates/bc_audio/duet-dsp/lang_rust/src/peaks/builder.rs` — create as a stub. Chunk D3 fills it with `PyramidBuilder`.
- `crates/bc_audio/duet-dsp/lang_rust/src/peaks/reader.rs` — create and fill. `Pyramid` and `Pyramid::load_level`.
- `Cargo.lock` — modify. SM5 rule 2 puts it in the write scope of every chunk that writes a member
  manifest.

**Why this chunk fills `peaks/format.rs` and `peaks/reader.rs`.** Section 13.0, block
`phase-pair-exempt`, states that chunk E1 "names `SampleSource` and `Pyramid`, which D1 wrote in
phase 1". E1 runs in phase 2, one phase after this chunk, so the two pyramid read types are this
chunk's work. Chunk D3 then adds `PyramidBuilder` in `peaks/builder.rs` and the write half of the
level layout. Both chunks name `peaks/{format,builder,reader}.rs` in their Writes cell, and SM6 puts
them in two phases, so one writer holds each file at a time.

## Types and signatures

### Manifest

```toml
# crates/bc_audio/duet-dsp/lang_rust/Cargo.toml, [dependencies]
duet-time = { workspace = true }
serde = { workspace = true }
thiserror = { workspace = true }
rustfft = { workspace = true }
realfft = { workspace = true }
```

Section 1.2 gives `duet-dsp` the third-party set `serde`, `thiserror`, `arrayvec`, `rustfft`, and
`realfft`. `arrayvec` reaches only `PyramidBuilder::staging`, which chunk D3 writes, so this chunk
adds no `arrayvec` entry and `cargo machete` passes. Chunk M1 pins `rustfft` and `realfft` in phase
1 and chunk M0 pinned `serde` and `thiserror` in phase 0. Section 1.3 gives the one internal edge
`duet-dsp -> duet-time`. SM4 forbids this chunk to edit the root manifest, so a missing pin is a
discrepancy that this chunk reports under SM0.

### From architecture section 15.2, module `duet_dsp::source`

```rust
/// A readable block source of samples. `duet-media` implements it.
pub trait SampleSource: Send {
    /// The frame count of the whole source.
    fn frames(&self) -> u64;

    /// Read one block into `out`, and return how many frames it wrote.
    ///
    /// # Errors
    /// Returns `DspError::SourceRead` when the source cannot answer.
    fn read(&mut self, at: u64, out: &mut [f32]) -> Result<usize, DspError>;
}
```

### From architecture section 15.2, module `duet_dsp::gain`

```rust
/// The running state of one gain stage.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GainState { current: Finite, target: Finite, step: Finite }
```

### From architecture section 15.2, module `duet_dsp::filter`

```rust
/// The running state of one equalizer, as four biquad sections.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct EqualizerState { sections: [BiquadState; 4] }

/// One biquad section: its coefficients and its two delay samples.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct BiquadState { b0: Finite, b1: Finite, b2: Finite, a1: Finite, a2: Finite, z1: Finite, z2: Finite }
```

### From architecture section 7.3, module `duet_dsp::meter`

```rust
/// The three meter laws that version one shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeterLaw { Peak, Rms, LufsTruePeak }

/// One strip's meter values for one frame.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct MeterReading { peak: Finite, rms: Finite, true_peak: Finite, hold: Finite }

impl MeterReading {
    /// Every level at zero. `MeterView::silent` fills its array with it.
    pub const SILENT: Self = Self {
        peak: Finite::ZERO,
        rms: Finite::ZERO,
        true_peak: Finite::ZERO,
        hold: Finite::ZERO,
    };
}

/// One slot's live measurement for one frame.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SlotMeasure { input: Finite, reduction: Finite }

impl SlotMeasure {
    /// No input and no reduction.
    pub const SILENT: Self = Self { input: Finite::ZERO, reduction: Finite::ZERO };
}

/// The held over marks of every strip, as a bitset.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeldMarks(u64);

/// Every strip's meter values for one frame, resolved for painting.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeterView {
    readings: [MeterReading; MAX_STRIPS],
    caps: [Finite; MAX_STRIPS],
    held: HeldMarks,
    live: u16,
}

impl MeterView {
    /// Every strip silent, no cap raised, and no mark held.
    #[must_use]
    pub const fn silent() -> Self {
        Self {
            readings: [MeterReading::SILENT; MAX_STRIPS],
            caps: [Finite::ZERO; MAX_STRIPS],
            held: HeldMarks(0),
            live: 0,
        }
    }
}

/// The held peak-over flag of one strip.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct OverMark(bool);
```

From architecture section 15.2, in the same module:

```rust
/// The running state of one meter tap.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MeterState { reading: MeterReading, over: OverMark, decay: Finite }
```

`MAX_STRIPS` is B86 and it sits at the root of `duet-time` (section 1.6), path
`duet_time::MAX_STRIPS`. `Finite` is `duet_time::Finite`.

### From architecture section 7.3, module `duet_dsp::meter_law`

```rust
/// The fraction of a meter's height that one decibel value reaches.
#[must_use]
pub fn db_to_fraction(db: Finite) -> Finite;

/// The fraction of the loudness bar that one integrated loudness reaches.
#[must_use]
pub fn lufs_to_fraction(lufs: Finite) -> Finite;

/// The fraction of the true-peak bar that one true-peak value reaches.
#[must_use]
pub fn true_peak_to_fraction(peak: Finite) -> Finite;

/// The gain one point of the fader travel sets, in decibels.
#[must_use]
pub fn fader_fraction_to_db(fraction: Finite) -> Finite;

/// The point of the fader travel that one gain in decibels sits at.
#[must_use]
pub fn fader_db_to_fraction(db: Finite) -> Finite;
```

### From architecture section 1.6, the `duet-dsp` constant block

```rust
/// The decibel value at the bottom of a meter (B106).
pub const METER_FLOOR_DB: Finite = Finite::from_finite_const(-60.0);
/// The decibel value at the top of a meter (B106).
pub const METER_CEILING_DB: Finite = Finite::from_finite_const(6.0);
/// The decibel value that sits at `METER_MID_FRACTION` of the height (B106).
pub const METER_MID_DB: Finite = Finite::from_finite_const(-20.0);
/// Where `METER_MID_DB` sits, as a fraction of the height (B106).
pub const METER_MID_FRACTION: Finite = Finite::from_finite_const(0.5);
/// The loudness value at the bottom of the LUFS bar (B143).
pub const LUFS_FLOOR: Finite = Finite::from_finite_const(-36.0);
/// The loudness value at the top of the LUFS bar (B143).
pub const LUFS_CEILING: Finite = Finite::from_finite_const(0.0);
/// The half width of the painted tolerance band, in loudness units (B143).
pub const LUFS_TOLERANCE: Finite = Finite::from_finite_const(1.0);
/// The true-peak value at the bottom of the true-peak bar (B144).
pub const TRUE_PEAK_FLOOR: Finite = Finite::from_finite_const(-12.0);
/// The true-peak value at the top of the true-peak bar (B144).
pub const TRUE_PEAK_CEILING: Finite = Finite::from_finite_const(3.0);
/// Where unity gain sits on the fader travel, as a fraction (B145).
pub const FADER_UNITY_FRACTION: Finite = Finite::from_finite_const(0.72);
/// The fraction below which the fader answers silence (B145).
pub const FADER_SILENCE_FRACTION: Finite = Finite::from_finite_const(0.08);
/// The gain at `FADER_SILENCE_FRACTION` of the travel (B145).
pub const FADER_SILENCE_DB: Finite = Finite::from_finite_const(-96.0);
/// The gain at the top of the fader travel (B145).
pub const FADER_CEILING_DB: Finite = Finite::from_finite_const(6.0);
```

Section 1.6 places the constants at the crate root, so `lib.rs` declares them and `meter_law.rs`
reads them.

### From architecture section 5.10, module `duet_dsp::peaks::format`

```rust
/// One bin of a peak pyramid. The level layout is B68. It is six bytes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeakBin { min: i16, max: i16, rms: u16 }

/// The on-disk header of a `.peaks` file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PyramidHeader {
    magic: [u8; 4],
    format_version: u16,
    channels: NonZeroU8,
    levels: NonZeroU8,
    frames: u64,
}
```

B68 states the level layout: level k covers 2 to the power of k plus 6 frames per bin.

### From architecture section 15.2, module `duet_dsp::peaks::reader`

```rust
/// ONE level of a peak pyramid over one source, held in memory for the lane
/// to paint.
#[derive(Debug, Clone)]
pub struct Pyramid { header: PyramidHeader, level: u8, from_bin: u64, bins: Box<[PeakBin]> }
```

Section 5.10 states the read rule: `Pyramid::load_level` reads the header, seeks to the level the
zoom step names, and reads the span the lane draws. B124 bounds the result at 2 MB.

```rust
/// Read one level of a pyramid, over the span the caller draws.
///
/// # Errors
/// Returns `DspError::PyramidFormat` for a bad magic value, a format version
/// this build does not read, or a level the header does not hold. Returns
/// `DspError::SourceRead` when the reader cannot answer.
pub fn load_level(
    source: &mut dyn SampleSource,
    level: u8,
    from_bin: u64,
    bins: u32,
) -> Result<Pyramid, DspError>;
```

### From architecture section 5.5, module `duet_dsp::buffer`

`PoolSlot` is a `u16` index into the buffer pool. **Chunk D3 builds `BufferPool` itself in phase 3,
and this chunk declares the index alone**, because `LimiterState`, `DelayState`, and `ReverbState`
each hold an `Option<PoolSlot>` and chunk D2 writes `LimiterState` in phase 2. Architecture sections
13.2 and 15.2 state the split.

```rust
/// A handle into the pool. A `SlotSpec` holds one; it never holds a buffer.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PoolSlot(u16);
```

### From architecture section 15.2, module `duet_dsp` root

```rust
/// Every way a signal block refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum DspError { SourceRead, BlockTooLong, PoolSlotMissing, PyramidFormat }
```

### Modules with no declared type

Section 1.5 places no type in `fft` or `align`, so each one holds functions alone. `buffer` holds
`PoolSlot` and the three block helpers below. A chunk that needs a new type reports the discrepancy
instead of inventing it (SM0).

```rust
// duet_dsp::buffer
/// Write silence over the block. It allocates nothing.
pub fn silence(block: &mut [f32]);

/// Add `src` into `dst`, sample by sample.
///
/// # Errors
/// Returns `DspError::BlockTooLong` when the two lengths differ.
pub fn add_into(dst: &mut [f32], src: &[f32]) -> Result<(), DspError>;

/// Copy `src` over `dst`, sample by sample.
///
/// # Errors
/// Returns `DspError::BlockTooLong` when the two lengths differ.
pub fn copy_into(dst: &mut [f32], src: &[f32]) -> Result<(), DspError>;
```

```rust
// duet_dsp::fft
/// The real forward transform of one block.
///
/// `duet-analysis` reaches the transforms through this module, which section
/// 1.3 states, so `rustfft` and `realfft` sit in `duet-dsp` alone.
///
/// # Errors
/// Returns `DspError::BlockTooLong` when the input length and the output
/// length do not match the planned size.
pub fn forward(
    planner: &mut RealFftPlanner<f32>,
    input: &mut [f32],
    output: &mut [Complex<f32>],
) -> Result<(), DspError>;

/// The real inverse transform of one spectrum.
///
/// # Errors
/// Returns `DspError::BlockTooLong` for the same reason `forward` does.
pub fn inverse(
    planner: &mut RealFftPlanner<f32>,
    input: &mut [Complex<f32>],
    output: &mut [f32],
) -> Result<(), DspError>;
```

```rust
// duet_dsp::align
/// The lag at which the captured block best matches the reference, in
/// frames.
///
/// Section 5.4 uses it for the loopback calibration: the application plays a
/// short chirp, records the input, and finds the offset by cross-correlation
/// in `duet_dsp::align`. Chunk C4 calls it.
///
/// # Errors
/// Returns `DspError::BlockTooLong` when `max_lag` is at or above the
/// captured length.
pub fn best_offset(reference: &[f32], captured: &[f32], max_lag: u32) -> Result<u32, DspError>;
```

```rust
// duet_dsp::gain
/// Advance the gain ramp and apply it to the block. It allocates nothing.
pub fn apply(state: &mut GainState, block: &mut [f32]);

/// Set a new target. The ramp reaches it over `frames` frames, so a fader
/// move makes no step in the signal.
pub fn set_target(state: &mut GainState, target: Finite, frames: u32);

/// Apply a constant-power pan to one stereo pair.
pub fn pan(left: &mut [f32], right: &mut [f32], position: Finite);
```

```rust
// duet_dsp::filter
/// Set the coefficients of one section to a high-pass at `frequency`.
pub fn set_high_pass(state: &mut BiquadState, frequency: Finite, rate: SampleRate);

/// Set the coefficients of one section to a peaking band.
pub fn set_peaking(state: &mut BiquadState, frequency: Finite, gain_db: Finite, q: Finite, rate: SampleRate);

/// Run one section over the block, in place. It allocates nothing.
pub fn process(state: &mut BiquadState, block: &mut [f32]);

/// Run the four sections of one equalizer over the block, in place.
pub fn process_equalizer(state: &mut EqualizerState, block: &mut [f32]);
```

```rust
// duet_dsp::meter
/// Measure one block and fold the result into the running state.
///
/// It sets `OverMark` on the first sample at or above full scale and never
/// clears it; only a reset generation change clears it (section 7.3).
pub fn measure(state: &mut MeterState, block: &[f32], law: MeterLaw);
```

`SampleRate` is `duet_time::SampleRate`.

## Steps

1. Read `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml` and `crates/bc_audio/duet-dsp/lang_rust/src/lib.rs`. Confirm that chunk M1 created
   both, that the manifest carries `[lints] workspace = true`, a `description`, and no
   `[dependencies]` section, and that `lib.rs` carries the `//!` crate documentation and
   `#![forbid(unsafe_code)]`. Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` carries `serde`, `thiserror`,
   `rustfft`, `realfft`, and `duet-time`. Report a discrepancy and stop if an entry is absent.
3. Create the twenty-two module files. Fill the ones this chunk owns and leave the ten stubs at
   one `//!` line each: `slot.rs`, `voice.rs`, `pool.rs`, `dynamics/compressor.rs`,
   `dynamics/gate.rs`, `dynamics/deesser.rs`, `dynamics/limiter.rs`, `dynamics/delay.rs`,
   `dynamics/reverb.rs`, and `peaks/builder.rs`.
4. Add the `mod` lines to `src/lib.rs`, the fourteen constants, and the `DspError` enum. Add the six
   `mod` lines to `src/dynamics.rs` and the three `mod` lines to `src/peaks.rs`.
5. Add the five `{ workspace = true }` entries to `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml`. Run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3).
6. Write the failing tests `meter_law_floor_reaches_zero`, `meter_law_ceiling_reaches_one`, and
   `meter_law_mid_reaches_the_mid_fraction` in `src/meter_law.rs`, inside a `#[cfg(test)] mod tests`.
   Run `cargo nextest run -p duet-dsp -E 'test(meter_law)' --no-tests=fail` and confirm that the run
   fails to compile.
7. Implement `db_to_fraction`, `lufs_to_fraction`, and `true_peak_to_fraction`. Each one clamps to
   the unit range. `db_to_fraction` is piecewise linear between the three B106 points, and the two
   bar functions are linear between their two points.
8. Run the same command and confirm that every `meter_law` test passes.
9. Write the failing tests `fader_taper_unity_sits_at_the_unity_fraction` and
   `fader_taper_round_trips` in `src/meter_law.rs`. Run
   `cargo nextest run -p duet-dsp -E 'test(fader_taper)' --no-tests=fail` and confirm that both fail.
10. Implement `fader_fraction_to_db` and `fader_db_to_fraction`. The map is linear in decibels
    between the three B145 points, and a fraction below `FADER_SILENCE_FRACTION` answers a gain of
    zero.
11. Run the same command and confirm that both tests pass.
12. Write the failing tests `align_finds_a_known_offset` and `align_refuses_a_lag_past_the_capture`
    in `src/align.rs`. Run `cargo nextest run -p duet-dsp -E 'test(align)' --no-tests=fail` and
    confirm that both fail.
13. Implement `forward` and `inverse` in `src/fft.rs` over `realfft`. Implement `best_offset` in
    `src/align.rs` as a cross-correlation through the transform, and take the lag of the largest
    value.
14. Run the same command and confirm that both tests pass.
15. Implement `silence`, `add_into`, and `copy_into` in `src/buffer.rs`. Use iterator zips and never
    an index, because `clippy::indexing_slicing` is denied. Declare `PoolSlot` in the same file,
    with the four derives section 5.5 states and an accessor that answers the inner index.
16. Declare `SampleSource` in `src/source.rs`.
17. Declare `GainState` and implement `apply`, `set_target`, and `pan` in `src/gain.rs`.
18. Declare `BiquadState` and `EqualizerState` and implement the four filter functions in
    `src/filter.rs`.
19. Declare `MeterLaw`, `MeterReading`, `MeterState`, `SlotMeasure`, `HeldMarks`, `MeterView`, and
    `OverMark` and implement `measure` in `src/meter.rs`.
20. Declare `PeakBin` and `PyramidHeader` in `src/peaks/format.rs`, with the B68 level layout as a
    `const fn frames_per_bin(level: u8) -> u64`.
21. Declare `Pyramid` and implement `load_level` in `src/peaks/reader.rs`. Refuse a request whose
    bin count would pass B124 with `DspError::PyramidFormat`.
22. Run `cargo nextest run -p duet-dsp --no-tests=fail` and confirm that every test passes.
23. Run `cargo clippy -p duet-dsp --all-targets --locked -- -D warnings` once and repair every
    finding.
24. Record the feature resolution of `rustfft` and `realfft`. Run `cargo tree -e features -i rustfft`
    and `cargo tree -e features -i realfft`, and paste both outputs into the commit body.
    **This chunk owns the record, and chunk M1 does not.** Appendix B.5 of
    `roadmap/duet-v1/architecture.md` states the rule: a `[workspace.dependencies]` entry that no
    member declares reaches neither the resolved graph nor `Cargo.lock`, so `cargo tree -i` exits 101
    at the manifest chunk. Step 5 of this chunk adds both `{ workspace = true }` entries to
    `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml`, so this commit is the first one that puts either crate in the
    graph. Chunk M1 confirmed each feature name with `cargo info`, and both pins take the default
    feature set. **Confirm that each tree shows the default set**; report a discrepancy to the
    Architect and stop if either one shows a feature the root pin does not state.
25. Commit on the branch `chunk/d1-buffers-source-and-meter-law`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.
No test name of this chunk holds the word `pyramid`, `dynamics`, `monitor_voice`, or `buffer_pool`,
because chunks D2 and D3 select those terms and a filterset union must not match a test an earlier
chunk wrote (SM3 rule 2).

`crates/bc_audio/duet-dsp/lang_rust/src/meter_law.rs`

- `meter_law_floor_reaches_zero` — asserts `db_to_fraction(METER_FLOOR_DB)` equals zero.
- `meter_law_ceiling_reaches_one` — asserts `db_to_fraction(METER_CEILING_DB)` equals one.
- `meter_law_mid_reaches_the_mid_fraction` — asserts `db_to_fraction(METER_MID_DB)` equals
  `METER_MID_FRACTION`.
- `meter_law_clamps_outside_the_range` — asserts zero below the floor and one above the ceiling.
- `meter_law_is_monotone` — asserts that the fraction never falls over 200 steps from the floor to
  the ceiling.
- `meter_law_lufs_floor_and_ceiling` — asserts zero at `LUFS_FLOOR` and one at `LUFS_CEILING`.
- `meter_law_true_peak_zero_is_inside_the_bar` — asserts that a true peak of zero answers a fraction
  strictly between zero and one, because `TRUE_PEAK_CEILING` sits above zero.
- `fader_taper_unity_sits_at_the_unity_fraction` — asserts that
  `fader_fraction_to_db(FADER_UNITY_FRACTION)` equals zero decibels.
- `fader_taper_top_reaches_the_ceiling` — asserts `FADER_CEILING_DB` at a fraction of one.
- `fader_taper_below_silence_answers_no_gain` — asserts a gain of zero below
  `FADER_SILENCE_FRACTION`.
- `fader_taper_round_trips` — asserts that `fader_db_to_fraction(fader_fraction_to_db(f))` equals
  `f` within 0.001 for forty fractions between `FADER_SILENCE_FRACTION` and one.

`crates/bc_audio/duet-dsp/lang_rust/src/align.rs`

- `align_finds_a_known_offset` — builds a chirp, delays a copy by 137 frames, and asserts that
  `best_offset` returns 137.
- `align_finds_a_zero_offset` — asserts 0 for two identical blocks.
- `align_refuses_a_lag_past_the_capture` — asserts `DspError::BlockTooLong`.
- `align_is_stable_under_noise` — adds noise at 20 decibels below the chirp and asserts the same
  offset.

`crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs`

- `buffer_silence_clears_every_sample` — asserts that every sample reads zero.
- `buffer_add_into_refuses_a_length_mismatch` — asserts `DspError::BlockTooLong`.
- `pool_slot_index_round_trips` — builds a `PoolSlot`, reads the index back, and asserts equality.
  The name holds no term that a later chunk of line D selects: chunk D3 selects `buffer_pool`.

`crates/bc_audio/duet-dsp/lang_rust/src/gain.rs`

- `gain_ramp_reaches_the_target` — asserts that the state reaches the target after the stated frame
  count and never passes it.
- `gain_pan_holds_constant_power` — asserts that the sum of the squares of the two channel gains
  stays within 0.001 of one across eleven pan positions.

`crates/bc_audio/duet-dsp/lang_rust/src/filter.rs`

- `filter_high_pass_removes_direct_current` — asserts that a constant block falls below 0.001 after
  one second of frames.
- `filter_peaking_lifts_its_own_band` — asserts a larger output at the centre frequency than at one
  octave below it.

`crates/bc_audio/duet-dsp/lang_rust/src/meter.rs`

- `meter_peak_follows_the_largest_sample` — asserts the peak value of a known block.
- `meter_over_mark_latches_and_never_clears` — asserts that one sample at full scale sets the mark,
  and that ten silent blocks after it leave the mark set.
- `meter_view_silent_holds_no_mark` — asserts that `MeterView::silent` reports no held mark and a
  live count of zero.

`crates/bc_audio/duet-dsp/lang_rust/src/peaks/format.rs`

- `peak_format_bin_is_six_bytes` — asserts `size_of::<PeakBin>()` equals 6.
- `peak_format_level_layout_follows_b68` — asserts that level 0 covers 64 frames per bin and that
  level 4 covers 1024.

`crates/bc_audio/duet-dsp/lang_rust/src/peaks/reader.rs`

- `level_read_returns_the_requested_span` — asserts the bin count and the first bin value.
- `level_read_refuses_a_bad_magic` — asserts `DspError::PyramidFormat`.
- `level_read_refuses_a_span_past_the_budget` — asserts `DspError::PyramidFormat` for a request that
  would pass B124.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-dsp -E 'test(align) + test(meter_law) + test(fader_taper)' --no-tests=fail
cargo nextest run -p duet-dsp --no-tests=fail
cargo clippy -p duet-dsp --all-targets --locked -- -D warnings
cargo machete
cargo tree -e features -i rustfft
cargo tree -e features -i realfft
```

The first command fails before this chunk, because the crate holds none of the three test terms. It
passes after this chunk and it reports fifteen tests. The second command reports every test of the
crate as passed. The third command prints no warning, and this chunk carries no `#[expect]` site.
The fourth command reports no unused dependency. The last two commands each print the default
feature set of their crate, and the commit body carries both; each one exits 101 before step 5, and
step 24 states why that record belongs to this chunk and not to chunk M1.

The work lands as one commit on the branch `chunk/d1-buffers-source-and-meter-law`, with a
conventional subject such as `feat(dsp): add the block helpers, the transforms, and the meter law`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs
  `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted
  diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site
  `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture
  Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is
  denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice
  indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only
  inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are
  required on every item.
- A new crate lives at `crates/bc_<context>/<crate>/lang_rust/` in the context that architecture section 1.2 names (ADR 0011), declares `[lints] workspace = true`, inherits every
  `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the
  root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- After chunk M94 lands, every `.rs` file a commit writes carries one front-matter block (`cargo xtask check-ddd --write`), and the gate refuses a changed `.rs` file with none.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no
  pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with
  Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this
  repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match
  what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
