---
id: D3
line: D
depends_on: [M3, D2]
write_scope:
  - crates/bc_audio/duet-dsp/lang_rust/src/voice.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/slot.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/pool.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/delay.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/reverb.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/format.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/builder.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/peaks/reader.rs
  - crates/bc_audio/duet-dsp/lang_rust/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-dsp -E 'test(pyramid) + test(monitor_voice) + test(buffer_pool)' --no-tests=fail"
---

# D3: The monitor voice, the delay and reverb kernels, `SlotState`, the buffer pool, and the pyramid builder

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk D1 created every file of this write scope in phase 1. `voice.rs`, `slot.rs`,
`pool.rs`, `dynamics/delay.rs`, `dynamics/reverb.rs`, and `peaks/builder.rs` hold a `//!` line alone,
and `peaks/format.rs` and `peaks/reader.rs` already hold the read half of the peak pyramid. This
chunk fills the six stubs and adds the write half of the pyramid. It declares `SlotState` over all
EIGHT `SlotKind` arms, which is what `clippy::wildcard_enum_match_arm` needs at the first match site
(critic C22I-6). It implements architecture sections 5.5, 5.10, 8.3, and 15.2, and it carries the
kernel half of the product stories C-22 and M-04.

The chunk writes the member manifest, so `Cargo.lock` is in the write scope (SM5 rule 2).

## The `arrayvec` entry

`PyramidBuilder::staging` is an `ArrayVec<PeakBin, PEAK_STAGING_BINS>` (section 5.10), so this chunk
uses the crate `arrayvec`. **This chunk adds the `arrayvec = { workspace = true }` entry to
`crates/bc_audio/duet-dsp/lang_rust/Cargo.toml`** (SM1), and `Cargo.lock` goes in the same commit (SM5 rule 2).
**Chunk M3 pins `arrayvec` in the root `[workspace.dependencies]` in phase 3**, one step before this
chunk: architecture section 13.1 ownership decision 6 and Appendix B.3 both state the owner. The pin
was M4's until this revision, which is one phase too late; the chunk author of line D reported it
and the Architect moved it.

Read the root `Cargo.toml` and `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml` first. Proceed when
`[workspace.dependencies]` pins `arrayvec`. Report the discrepancy to the Architect and stop when
the pin is absent (SM0), because SM4 forbids this chunk to edit the root manifest.

## Files

- `crates/bc_audio/duet-dsp/lang_rust/src/voice.rs` — modify. The monitor voice.
- `crates/bc_audio/duet-dsp/lang_rust/src/slot.rs` — modify. `SlotState` over the eight `SlotKind` arms.
- `crates/bc_audio/duet-dsp/lang_rust/src/pool.rs` — modify. `BufferPool`, `PoolBuffer`, and `PoolClass`. **`PoolSlot`
  is chunk D1's, in `src/buffer.rs`**; this chunk reads it and declares it nowhere.
- `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml` — modify. Add the `arrayvec = { workspace = true }` entry.
- `Cargo.lock` — modify. `cargo build --workspace` settles it (SM5 rule 3).
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/delay.rs` — modify. `DelayState` and the delay kernel.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/reverb.rs` — modify. `ReverbState` and the reverb kernel.
- `crates/bc_audio/duet-dsp/lang_rust/src/peaks/format.rs` — modify. Add the write half of the level layout.
- `crates/bc_audio/duet-dsp/lang_rust/src/peaks/builder.rs` — modify. `PyramidBuilder`.
- `crates/bc_audio/duet-dsp/lang_rust/src/peaks/reader.rs` — modify. Add the header read that the builder writes.

## Types and signatures

### Consumed types

| Type | Crate and path |
|---|---|
| `PeakBin`, `PyramidHeader`, `Pyramid`, `BiquadState`, `EqualizerState`, `DspError`, `SlotMeasure`, `PEAK_STAGING_BINS` | `duet-dsp`, chunk D1 |
| `CompressorState`, `GateState`, `DeEsserState`, `LimiterState` | `duet-dsp`, chunk D2, module `dynamics` |
| `SlotKind` | `duet-session`, path `duet_session::SlotKind`. **This crate has no `duet-session` edge** (section 1.3), so `SlotState` names the eight arms by its own arm names and never the foreign enum. |
| `Finite`, `ChannelIndex`, `SampleRate`, `MAX_STRIPS` | `duet-time` |

`SlotKind` is `{ HighPass, Equalizer, Compressor, Gate, DeEsser, Limiter, Delay, Reverb }` (section
5.5). `SlotState` carries one arm per kind, with the same names.

### From architecture section 15.2, module `duet_dsp::dynamics::delay`

```rust
/// The running state of one delay line. It holds a pool handle, never a buffer.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DelayState { buffer: Option<PoolSlot>, write_head: u32, delay_frames: u32, feedback: Finite, mix: Finite }
```

### From architecture section 15.2, module `duet_dsp::dynamics::reverb`

```rust
/// The running state of one reverb. It holds a pool handle, never a buffer.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ReverbState { buffer: Option<PoolSlot>, write_head: u32, decay: Finite, damping: Finite, mix: Finite }
```

### From architecture section 5.5, module `duet_dsp::slot`

```rust
#[expect(
    variant_size_differences,
    reason = "audio-owned state stays inline; the arm spread is intended and bounded by B51"
)]
#[expect(
    missing_copy_implementations,
    reason = "one slot state is B51 bytes and it is the identity the audio thread owns for the \
              life of the chain, so a `Copy` derive would hide a copy of that size at every \
              migration site"
)]
#[derive(Debug)]
/// **Audio-owned** (section 5.7).
pub enum SlotState {
    /// One biquad section. A high-pass needs no state of its own, so the
    /// arm holds the `duet-dsp` biquad directly and no seventh state type
    /// exists to keep in step with it (PR M-03).
    HighPass(BiquadState),
    Equalizer(EqualizerState),
    Compressor(CompressorState),
    Gate(GateState),
    DeEsser(DeEsserState),
    /// The look-ahead limiter of the master chain. PR MA-02 makes it a MUST
    /// and contract 5.2 draws its gain reduction bar.
    Limiter(LimiterState),
    /// The state holds coefficients and a pool handle, never the buffer.
    Delay(DelayState),
    /// The same shape. PR M-04 makes a reverb bus a MUST.
    Reverb(ReverbState),
}
```

Both `#[expect]` attributes are Appendix B.1 rows for the site `duet-dsp::slot::SlotState`, and each
reason text above is that row's text, copied. B51 is 232 bytes for one `SlotState` with no buffer.

### From architecture section 5.5, module `duet_dsp::pool`

```rust
/// A preallocated set of long audio buffers. One pool serves one project.
///
/// It derives `Debug` and nothing else. No published type holds one, so no
/// container is ever forced to derive `Clone` or `PartialEq` over it.
///
/// **Every allocation happens off the audio thread**, at `configure` time.
/// The audio thread receives the whole pool through the B105 handoff queue
/// and never grows it, never shrinks it, and never drops it; `basedrop`
/// defers the free to the collector thread (section 5.5).
#[derive(Debug)]
pub struct BufferPool {
    /// One delay line per B47 slot, each one B49 bytes.
    delays: Box<[PoolBuffer]>,
    /// One reverb tail per B48 slot, each one B50 bytes.
    reverbs: Box<[PoolBuffer]>,
    /// One look-ahead line per B120 slot, each one B121 bytes.
    limiters: Box<[PoolBuffer]>,
    /// Bit k is set when slot k is free. B47 plus B48 plus B120 sits below
    /// the bit width of a `u64`, so one word covers the whole pool.
    free: u64,
}

/// One long audio buffer of the pool, with the channel count it was sized for.
#[derive(Debug)]
pub struct PoolBuffer {
    samples: Box<[f32]>,
    channels: ChannelIndex,
}

```

**`PoolSlot` is declared by chunk D1, in `crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs`.** `LimiterState`,
`DelayState`, and `ReverbState` each hold an `Option<PoolSlot>` and chunk D2 writes `LimiterState`
in phase 2, so the index type lands in phase 1 and the pool that hands it out lands here, in phase
3. Architecture sections 13.2 and 15.2 state the split. This chunk imports the type and declares it
nowhere; a second declaration is a discrepancy that stops the chunk (SM0).

From architecture section 1.6, on `BufferPool`:

```rust
impl BufferPool {
    /// Delay buffers in one pool (B47).
    pub const DELAY_SLOTS: usize = 8;
    /// Reverb buffers in one pool (B48).
    pub const REVERB_SLOTS: usize = 4;
    /// Look-ahead buffers in one pool (B120).
    pub const LIMITER_SLOTS: usize = 9;
}
```

The pool needs a claim path and a release path, and both run off the audio thread at `configure`
time:

```rust
/// Build one pool for a project at `rate` and `channels`.
///
/// It allocates every buffer, so no allocation happens on the audio thread.
pub fn new(rate: SampleRate, channels: ChannelIndex) -> Self;

/// Take a free slot of the class the kind needs.
///
/// # Errors
/// Returns `DspError::PoolSlotMissing` when the class holds no free slot.
pub fn claim(&mut self, kind: PoolClass) -> Result<PoolSlot, DspError>;

/// Return a slot to its class.
pub fn release(&mut self, slot: PoolSlot);

/// Borrow the samples of one slot.
///
/// # Errors
/// Returns `DspError::PoolSlotMissing` for a handle the pool does not hold.
pub fn samples_mut(&mut self, slot: PoolSlot) -> Result<&mut [f32], DspError>;

/// Which class of the pool a slot belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolClass { Delay, Reverb, Limiter }
```

`PoolClass` is a module type of `duet-dsp` that no other crate names. `PoolExhausted { kind }` is a
`duet-engine` refusal and it sits outside this crate, so the pool answers with
`DspError::PoolSlotMissing` and `configure` converts.

### From architecture section 5.10, module `duet_dsp::peaks::builder`

```rust
/// Build a pyramid one block at a time, during a record pass or after it.
///
/// **Nothing accumulates for the length of the take** (critic C20-W4). A
/// closed bin is written to the file and forgotten.
#[derive(Debug)]
pub struct PyramidBuilder {
    header: PyramidHeader,
    /// The bin each level is filling now. B68 gives the level layout.
    open: Box<[PeakBin]>,
    /// How many frames each level's open bin has absorbed.
    filled: Box<[u32]>,
    /// The bins this append closed, and nothing older.
    ///
    /// It is FIXED at B123 bins and it is drained to the file on every
    /// `append`, so the builder's residency is B123 times the six bytes of a
    /// `PeakBin` whatever the length of the take is. A full staging buffer
    /// inside one `append` is a flush and never a growth.
    staging: ArrayVec<PeakBin, PEAK_STAGING_BINS>,
}
```

```rust
/// Start a builder over one source.
pub fn new(header: PyramidHeader) -> PyramidBuilder;

/// Absorb one block and drain every bin the block closed into `out`.
///
/// # Errors
/// Returns `DspError::BlockTooLong` when the block length is not a whole
/// number of frames for the channel count the header states.
pub fn append(&mut self, block: &[f32], out: &mut Vec<u8>) -> Result<(), DspError>;

/// Close every open bin and write the trailing bytes.
///
/// # Errors
/// Returns `DspError::PyramidFormat` when the frame count the header states
/// and the frame count the builder absorbed differ.
pub fn finish(self, out: &mut Vec<u8>) -> Result<(), DspError>;
```

### Module `duet_dsp::voice`

Section 8.3 states the shape: the monitor voice is one small synthesized vowel-like tone that
follows the dynamic (PR 11 Q4). It lives in `duet-dsp::voice`, so the engine and the Compose monitor
share one implementation. Section 1.5 places no type in this module, so the module holds functions
and one running state that no other crate names.

```rust
/// The running state of the monitor voice.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VoiceState { phase: Finite, level: GainState, note: Option<u8> }

/// Start a note at `note` and `velocity`.
pub fn note_on(state: &mut VoiceState, note: u8, velocity: u8);

/// Release the sounding note, if the number matches.
pub fn note_off(state: &mut VoiceState, note: u8);

/// Release every sounding note at once, which section 8.2 step 1 needs when
/// a port departs.
pub fn all_notes_off(state: &mut VoiceState);

/// Add the voice into the block. It allocates nothing.
pub fn render(state: &mut VoiceState, block: &mut [f32], rate: SampleRate);
```

`VoiceState` is not in the section 1.5 register, so report the discrepancy to the Architect if
`cargo xtask check-placement` refuses it; the alternative is a state the caller passes as plain
arguments.

## Steps

1. Read the eight files. Confirm the state each one is in, as the opening paragraph states. Report a
   discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `arrayvec`, which chunk
   M3 wrote in this phase. Report a discrepancy and stop if the pin is absent. Add
   `arrayvec = { workspace = true }` to `crates/bc_audio/duet-dsp/lang_rust/Cargo.toml` and run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3).
3. Write the failing test `buffer_pool_claims_and_releases_each_class` in `src/pool.rs`, inside a
   `#[cfg(test)] mod tests`. Run
   `cargo nextest run -p duet-dsp -E 'test(buffer_pool)' --no-tests=fail` and confirm that the run
   fails to compile.
4. Declare `PoolBuffer`, `PoolClass`, and `BufferPool` with the three class constants. Import
   `PoolSlot` from `crate::buffer`, which chunk D1 declared in phase 1.
   Implement `new`, `claim`, `release`, and `samples_mut`. The free set is one `u64` word, so a claim
   is one `trailing_zeros` call and one bit clear.
5. Run the same command and confirm that every `buffer_pool` test passes.
6. Declare `DelayState` and implement its kernel in `src/dynamics/delay.rs`. Declare `ReverbState`
   and implement its kernel in `src/dynamics/reverb.rs`. Each `process` takes the pool buffer as an
   argument, exactly as the limiter of chunk D2 does, so no kernel reaches the pool.
7. Declare `SlotState` in `src/slot.rs` with both `#[expect]` attributes and the eight arms.
   Implement one `process` that matches every arm and calls the kernel of that arm:
   ```rust
   /// Run one slot over the block and return the measurement the strip
   /// publishes for it.
   ///
   /// # Errors
   /// Returns `DspError::PoolSlotMissing` when an arm that needs a pool
   /// buffer holds no handle.
   pub fn process(
       state: &mut SlotState,
       block: &mut [f32],
       pool_buffer: Option<&mut [f32]>,
       rate: SampleRate,
   ) -> Result<SlotMeasure, DspError>;
   ```
   Name every arm, because `clippy::wildcard_enum_match_arm` is denied.
8. Write the failing test `monitor_voice_sounds_a_note_on` in `src/voice.rs`. Run
   `cargo nextest run -p duet-dsp -E 'test(monitor_voice)' --no-tests=fail` and confirm that it
   fails.
9. Declare `VoiceState` and implement the four voice functions.
10. Run the same command and confirm that every `monitor_voice` test passes.
11. Write the failing tests `pyramid_builder_closes_a_bin_per_level` and
    `pyramid_builder_holds_a_fixed_staging_buffer` in `src/peaks/builder.rs`. Run
    `cargo nextest run -p duet-dsp -E 'test(pyramid)' --no-tests=fail` and confirm that both fail.
12. Add the write half of the level layout to `src/peaks/format.rs`: the byte encode of one
    `PeakBin` and of one `PyramidHeader`, each one little-endian, and the byte offset of the first
    bin of each level.
13. Declare `PyramidBuilder` and implement `new`, `append`, and `finish`. Drain `staging` into `out`
    whenever it reaches `PEAK_STAGING_BINS` and at the end of every `append`, so the residency stays
    at B123 bins.
14. Add the matching header read to `src/peaks/reader.rs`, so a file the builder wrote loads through
    `Pyramid::load_level`.
15. Run the same command and confirm that every `pyramid` test passes.
16. Run `cargo nextest run -p duet-dsp --no-tests=fail` and confirm that every test passes.
17. Run `cargo clippy -p duet-dsp --all-targets --locked -- -D warnings` once and repair every
    finding. Confirm that the two `#[expect]` attributes on `SlotState` are fulfilled, because
    `-D warnings` turns an unfulfilled expectation into an error.
18. Commit on the branch `chunk/d3-voice-slots-and-pyramid`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.

`crates/bc_audio/duet-dsp/lang_rust/src/pool.rs`

- `buffer_pool_claims_and_releases_each_class` — asserts that a claim of each class gives a distinct
  handle, and that a release makes the handle available again.
- `buffer_pool_refuses_a_ninth_delay` — asserts `DspError::PoolSlotMissing` after `DELAY_SLOTS`
  claims.
- `buffer_pool_limiter_class_is_its_own` — asserts that nine limiter claims succeed while the delay
  class stays full, which is the B120 class critic C20-N3 added.
- `buffer_pool_samples_refuse_a_foreign_handle` — asserts `DspError::PoolSlotMissing`.

`crates/bc_audio/duet-dsp/lang_rust/src/slot.rs`

- `slot_state_processes_every_arm` — builds one `SlotState` per arm and asserts that `process`
  answers a `SlotMeasure` for each one, which proves the match is exhaustive.
- `slot_state_refuses_a_missing_pool_buffer` — asserts `DspError::PoolSlotMissing` for the `Delay`,
  `Reverb`, and `Limiter` arms with no buffer.
- `slot_state_size_is_within_b51` — asserts `size_of::<SlotState>()` is at or below 232 bytes, which
  is B51 and which the `variant_size_differences` expectation cites.

`crates/bc_audio/duet-dsp/lang_rust/src/voice.rs`

- `monitor_voice_sounds_a_note_on` — asserts that the rendered block is not silent after `note_on`.
- `monitor_voice_stops_after_note_off` — asserts silence after the release time.
- `monitor_voice_all_notes_off_silences_at_once` — asserts silence on the next block, which section
  8.2 step 1 needs.
- `monitor_voice_follows_the_velocity` — asserts a larger peak at velocity 127 than at velocity 40.

`crates/bc_audio/duet-dsp/lang_rust/src/dynamics/delay.rs`

- `delay_repeats_after_the_delay_frames` — asserts that an impulse reappears at `delay_frames`.
- `delay_feedback_decays` — asserts that each repeat is smaller than the one before it.

`crates/bc_audio/duet-dsp/lang_rust/src/dynamics/reverb.rs`

- `reverb_tail_decays_to_silence` — asserts that the tail falls below 0.001 inside the decay time.
- `reverb_damping_cuts_the_high_band` — asserts less high-band energy at a larger damping value.

`crates/bc_audio/duet-dsp/lang_rust/src/peaks/builder.rs`

- `pyramid_builder_closes_a_bin_per_level` — appends 4096 frames and asserts the bin count of each
  level against the B68 layout.
- `pyramid_builder_holds_a_fixed_staging_buffer` — appends one hour of frames in blocks and asserts
  that `staging` never passes `PEAK_STAGING_BINS`, which is B123.
- `pyramid_builder_finish_refuses_a_frame_count_mismatch` — asserts `DspError::PyramidFormat`.
- `pyramid_round_trips_through_the_reader` — writes a pyramid with the builder, reads level 2 with
  `Pyramid::load_level`, and asserts that every bin equals the value the builder wrote.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-dsp -E 'test(pyramid) + test(monitor_voice) + test(buffer_pool)' --no-tests=fail
cargo nextest run -p duet-dsp --no-tests=fail
cargo clippy -p duet-dsp --all-targets --locked -- -D warnings
```

The first command fails before this chunk, because the crate holds none of the three test terms.
Chunk D1 wrote its peak tests as `peak_format_*` and `level_read_*` for that reason (SM3 rule 2). The
command passes after this chunk and it reports twelve tests. The second command reports every test
of the crate as passed. The third command prints no warning, and the two `#[expect]` sites are the
Appendix B.1 rows for `duet-dsp::slot::SlotState`.

The work lands as one commit on the branch `chunk/d3-voice-slots-and-pyramid`, with a conventional
subject such as `feat(dsp): add the monitor voice, the slot states, the pool, and the pyramid
builder`.

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
