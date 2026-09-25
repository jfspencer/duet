---
id: D2
line: D
depends_on: [M2, D1, M94]
write_scope:
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/compressor.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/gate.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/deesser.rs
  - crates/bc_audio/duet-dsp/lang_rust/src/dynamics/limiter.rs
parallelism: independent
completion: "cargo nextest run -p duet-dsp -E 'test(dynamics)' --no-tests=fail"
---

# D2: The compressor, the gate, the de-esser, and the true-peak limiter

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. Chunk D1 created the four files as stubs in phase 1, so each one holds a `//!` line
alone. This chunk fills all four. It declares `CompressorState`, `GateState`, `DeEsserState`, and
`LimiterState` and it writes the kernel of each one. The limiter kernel is the ONE implementation
that the live master chain and the section 7.4 render both call, so the monitor and the file agree
(PR MA-02, critic C19-11). The chunk implements architecture sections 5.5, 7.1, and 15.2, and it
carries the kernel half of the product stories M-03 and MA-02.

The chunk writes no manifest, so `Cargo.lock` is not in the write scope (SM5 rule 2).

## The `PoolSlot` import

`LimiterState` holds `buffer: Option<PoolSlot>` (section 15.2). `PoolSlot` is a `duet-dsp` type that
section 5.5 declares, and **chunk D1 declares it in `crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs` in phase 1**, one
phase before this chunk. Import it with `use crate::buffer::PoolSlot;` in `limiter.rs`. Section 13.2
gave the type to chunk D3, in phase 3, until this revision; the chunk author of line D reported that
this chunk could not then compile, and the Architect moved the declaration to D1. Chunk D3 still
builds `BufferPool` itself, in `crates/bc_audio/duet-dsp/lang_rust/src/pool.rs`, in phase 3.

Read `crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs` first. **A missing declaration is an SM0 stop**: report the
discrepancy to the Architect and write nothing. Do not declare `PoolSlot` here. `buffer.rs` is
outside this chunk's write scope, and a second declaration would break the one-owner rule of
section 1.5.

## Files

- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/compressor.rs` — modify. `CompressorState` and the compressor
  kernel.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/gate.rs` — modify. `GateState` and the gate kernel.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/deesser.rs` — modify. `DeEsserState` and the de-esser kernel.
- `crates/bc_audio/duet-dsp/lang_rust/src/dynamics/limiter.rs` — modify. `LimiterState` and the true-peak limiter
  kernel.

## Types and signatures

### Consumed types

| Type | Crate and path |
|---|---|
| `BiquadState`, `SlotMeasure`, `DspError` | `duet-dsp`, chunk D1, modules `filter`, `meter`, and the crate root |
| `PoolSlot` | `duet-dsp`, chunk D1, module `buffer`. See "The `PoolSlot` import" above. |
| `Finite`, `FrameCount`, `SampleRate` | `duet-time`, paths `duet_time::Finite`, `duet_time::FrameCount`, `duet_time::SampleRate` |

### From architecture section 15.2, module `duet_dsp::dynamics::compressor`

```rust
/// The running state of one compressor.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CompressorState { envelope: Finite, gain: Finite, threshold: Finite, ratio: Finite, attack: Finite, release: Finite }
```

### From architecture section 15.2, module `duet_dsp::dynamics::gate`

```rust
/// The running state of one gate.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct GateState { envelope: Finite, open: bool, threshold: Finite, hold_frames: FrameCount }
```

### From architecture section 15.2, module `duet_dsp::dynamics::deesser`

```rust
/// The running state of one de-esser.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeEsserState { detector: BiquadState, envelope: Finite, threshold: Finite }
```

### From architecture section 15.2, module `duet_dsp::dynamics::limiter`

```rust
/// The running state of one true-peak limiter.
///
/// **It is the LIVE limiter of the master chain** (contract 5.2, PR MA-02).
/// The same kernel runs offline in the section 7.4 render, so one
/// implementation serves the monitor and the file and the user hears what
/// the export writes. It holds a pool handle for its look-ahead delay and
/// never a buffer, exactly as `DelayState` does, so the audio thread
/// allocates nothing when the master chain changes. **The handle names a
/// slot of the `BufferPool::limiters` class**, which is B120 slots of B121
/// bytes; a delay slot would be B49 and far above the need (section 5.5,
/// critic C20-N3).
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LimiterState { buffer: Option<PoolSlot>, write_head: u32, envelope: Finite, ceiling: Finite, release: Finite }
```

### The four kernels

Section 1.5 places no other type in these modules, so each one holds its state type plus functions.
B96 gives the parameter count of each kind: 4 for a compressor, 3 for a gate, 3 for a de-esser, and
3 for a limiter, which are a ceiling, a release, and a look-ahead. Each kernel reads its parameters
by offset from the range that `SlotConfig::params` records, so the setter takes an offset and a
value.

```rust
// duet_dsp::dynamics::compressor
/// Set one parameter of the compressor, by its offset in the B96 range.
///
/// # Errors
/// Returns `DspError::BlockTooLong` for an offset at or above the B96 count
/// of this kind, because the caller addressed a parameter this kind does not
/// hold.
pub fn set_param(state: &mut CompressorState, offset: u16, value: Finite) -> Result<(), DspError>;

/// Run the compressor over the block, in place, and return the measurement
/// the strip publishes for this slot.
///
/// It allocates nothing, it locks nothing, and it takes no branch on a value
/// the audio thread cannot see, which TH1 requires.
pub fn process(state: &mut CompressorState, block: &mut [f32], rate: SampleRate) -> SlotMeasure;
```

`gate`, `deesser`, and `limiter` each declare the same pair over their own state type. The limiter
pair carries one more argument, because the look-ahead lives in the pool:

```rust
// duet_dsp::dynamics::limiter
/// Run the true-peak limiter over the block, in place, and return the
/// measurement the strip publishes.
///
/// `look_ahead` is the pool buffer that `LimiterState::buffer` names. The
/// caller resolves the handle, so this kernel never reaches the pool and the
/// offline render of section 7.4 calls the same function with its own
/// buffer.
///
/// # Errors
/// Returns `DspError::PoolSlotMissing` when `LimiterState::buffer` is `None`,
/// because a limiter with no look-ahead cannot bound a true peak.
pub fn process(
    state: &mut LimiterState,
    block: &mut [f32],
    look_ahead: &mut [f32],
    rate: SampleRate,
) -> Result<SlotMeasure, DspError>;
```

`SlotMeasure` carries two quantities, which section 7.3 states: `input` is the level the slot saw at
its input, on the B106 scale, and `reduction` is the decibels of gain the slot removed this frame,
at or below zero. A gate publishes `Finite::ZERO` for `reduction` while it is open.

B122 is the look-ahead of the true-peak limiter, at 5 ms. It is under two B7 cycles, so the master
chain adds no visible latency to a monitor mix.

## Steps

1. Read the four files. Confirm that each one holds a `//!` line and nothing else. Report a
   discrepancy and stop if the state differs.
2. Read `crates/bc_audio/duet-dsp/lang_rust/src/buffer.rs` and confirm that it declares `PoolSlot`, which chunk D1
   wrote in phase 1. Report the discrepancy and stop if the type is absent (SM0).
3. Write the failing test `dynamics_compressor_reduces_above_the_threshold` in
   `src/dynamics/compressor.rs`, inside a `#[cfg(test)] mod tests`. Run
   `cargo nextest run -p duet-dsp -E 'test(dynamics)' --no-tests=fail` and confirm that the run
   fails to compile.
4. Declare `CompressorState`. Implement `set_param` and `process`. The detector is a peak envelope
   with the attack and the release coefficients that the two parameters set. The gain is the
   reduction the ratio and the threshold ask for, in decibels, applied through the linear value the
   `duet_time` conversion supplies.
5. Run the same command and confirm that the test passes.
6. Write the failing test `dynamics_gate_closes_below_the_threshold`. Run the same command and
   confirm that it fails.
7. Declare `GateState`. Implement `set_param` and `process`. The gate opens on the first sample at
   or above the threshold, and it stays open for `hold_frames` after the last one.
8. Run the same command and confirm that the test passes.
9. Write the failing test `dynamics_deesser_cuts_only_the_sibilant_band`. Run the same command and
   confirm that it fails.
10. Declare `DeEsserState`. Implement `set_param` and `process`. The detector is the `BiquadState`
    of chunk D1, set to a peaking band at the sibilant frequency, and the reduction applies to the
    band alone.
11. Run the same command and confirm that the test passes.
12. Write the failing tests `dynamics_limiter_holds_the_ceiling` and
    `dynamics_limiter_refuses_a_missing_pool_slot`. Run the same command and confirm that both fail.
13. Declare `LimiterState`. Implement `set_param` and `process`. The kernel writes each input sample
    into the look-ahead buffer at `write_head`, reads the sample B122 frames behind it, and applies
    the gain the largest true peak inside the window asks for. Use `slice::get` and
    `slice::get_mut` for every buffer access, because `clippy::indexing_slicing` is denied.
14. Run the same command and confirm that every `dynamics` test passes.
15. Run `cargo nextest run -p duet-dsp --no-tests=fail` and confirm that every test passes.
16. Run `cargo clippy -p duet-dsp --all-targets --locked -- -D warnings` once and repair every
    finding.
17. Commit on the branch `chunk/d2-dynamics-kernels`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and every assert carries a message.
Every test name of this chunk starts with `dynamics_`, which is the term the Completion command
selects and a term chunk D1 produced in no test name (SM3 rule 2).

`crates/bc_audio/duet-dsp/lang_rust/src/dynamics/compressor.rs`

- `dynamics_compressor_reduces_above_the_threshold` — asserts that a block 6 decibels above the
  threshold at a ratio of 2 to 1 leaves the output 3 decibels above it, within 0.2 decibels.
- `dynamics_compressor_passes_below_the_threshold` — asserts that a block below the threshold
  changes by less than 0.01 decibels, and that the returned `reduction` is zero.
- `dynamics_compressor_attack_is_not_instant` — asserts that the first sample after a step reads a
  smaller reduction than the sample one attack time later.
- `dynamics_compressor_refuses_an_offset_past_b96` — asserts `DspError::BlockTooLong` at offset 4,
  which is the B96 count of this kind.
- `dynamics_compressor_reports_its_input_level` — asserts that `SlotMeasure::input` equals the block
  peak on the B106 scale.

`crates/bc_audio/duet-dsp/lang_rust/src/dynamics/gate.rs`

- `dynamics_gate_closes_below_the_threshold` — asserts that the output falls to silence after the
  hold time.
- `dynamics_gate_holds_open_for_the_hold_frames` — asserts that the output is unchanged for exactly
  `hold_frames` after the last loud sample.
- `dynamics_gate_refuses_an_offset_past_b96` — asserts `DspError::BlockTooLong` at offset 3.

`crates/bc_audio/duet-dsp/lang_rust/src/dynamics/deesser.rs`

- `dynamics_deesser_cuts_only_the_sibilant_band` — asserts that a two-tone block loses level at the
  sibilant tone and keeps it at the low tone, within 0.2 decibels.
- `dynamics_deesser_passes_a_clean_block` — asserts no change for a block with no sibilant energy.

`crates/bc_audio/duet-dsp/lang_rust/src/dynamics/limiter.rs`

- `dynamics_limiter_holds_the_ceiling` — asserts that no output sample passes the ceiling for an
  input 12 decibels above it.
- `dynamics_limiter_look_ahead_catches_a_transient` — asserts that a one-sample transient is bounded
  by the ceiling, which is the property the look-ahead exists for.
- `dynamics_limiter_refuses_a_missing_pool_slot` — asserts `DspError::PoolSlotMissing` when
  `LimiterState::buffer` is `None`.
- `dynamics_limiter_reports_its_gain_reduction` — asserts that `SlotMeasure::reduction` is at or
  below zero and equals the decibels the kernel removed, within 0.2 decibels.
- `dynamics_limiter_is_the_one_kernel` — asserts that two runs of `process` over one block with one
  state give equal output, which is the determinism the offline render of section 7.4 needs.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-dsp -E 'test(dynamics)' --no-tests=fail
cargo nextest run -p duet-dsp --no-tests=fail
cargo clippy -p duet-dsp --all-targets --locked -- -D warnings
```

The first command fails before this chunk, because the crate holds no `dynamics` test. It passes
after this chunk and it reports fifteen tests. The second command reports every test of the crate as
passed. The third command prints no warning, and this chunk carries no `#[expect]` site.

The work lands as one commit on the branch `chunk/d2-dynamics-kernels`, with a conventional subject
such as `feat(dsp): add the compressor, gate, de-esser, and true-peak limiter`.

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
