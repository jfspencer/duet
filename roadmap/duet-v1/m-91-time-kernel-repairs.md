---
id: M91
line: M
depends_on: [M0, T1]
write_scope:
  - crates/duet-time/src/convert.rs
  - crates/duet-time/src/units.rs
  - crates/duet-time/tests/kernel.rs
parallelism: independent
completion: "cargo nextest run -p duet-time -E 'test(lossless)' --no-tests=fail passes; cargo nextest run -p duet-time --no-tests=fail passes; cargo clippy -p duet-time --all-targets -- -D warnings is clean; commit SHA on a branch chunk/m91-time-kernel-repairs"
---

# M91: The lossless conversion pair and the two compile-time constant helpers

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This is a REPAIR chunk under rule SM9 of architecture section 13.0. It lands the code half of three
escalations that chunk T1 opened on 2026-09-22: T1-1, T1-2, and T1-3. It implements architecture
section 2.3 (the conversion declarations and the round-trip table), Appendix B.1 `b1-convert` items
1 to 3, section 1.6 (the `non_zero` block and the note under it), section 12.3 house form 5,
ADR-0007, and ADR-0008.

**Duet is a lossless editing system.** That is the product decision this chunk carries into the
trunk crate. Revision 23 paired a decode divisor of 2^23 with an encode scale factor of 2^23 - 1, so
a decode and a re-encode at unity gain moved every sample above half scale by one least significant
bit. ADR-0007 states the repair and the two bounds.

An engineer executes this chunk. It writes crate source alone and touches no policy file.

**This chunk runs FIRST among the phase-1 chunks that follow M1, and chunk M90 runs after it.**
Section 13.3 prints the order and section 13.0 rule SM1 states the rule. M90 adds rule CG9, which
compares each Appendix B.1 reason cell with the `reason =` string this chunk repairs, so a run of
M90 before this chunk would be red on its own gate. **FIVE of the seven cells differ from the code
today, and this chunk repairs all five**: steps 4, 5, 9, 10 and 13 carry them, one step per string,
and five is the size the Files row above states (critic C2-6). The other two cells already agree
with the code and this chunk leaves them alone. Both repair chunks complete before every line
chunk of phase 1, because this chunk changes the value that `unit_to_i24` and `unit_to_i32` RETURN
and chunks D1 and T2 consume `duet-time` in the same phase.

## Files

| Path | Action |
|---|---|
| `crates/duet-time/src/convert.rs` | modify (two scale constants, one body, FIVE `#[expect]` reasons, one doc block) |
| `crates/duet-time/src/units.rs` | modify (`non_zero` and `non_zero_u32`) |
| `crates/duet-time/tests/kernel.rs` | modify (one new test, one new constant value, three repaired tests) |

## Types and signatures

No signature changes. Every public item keeps its current shape. Two module constants of
`crates/duet-time/src/convert.rs` change value, and their documentation changes with them.

```rust
/// The scale that maps a unit sample onto the 24-bit range.
///
/// It is two to the power of 23, which is the divisor `I24_SCALE` uses, so the
/// 24-bit round trip is exact for all 16,777,216 values (ADR-0007).
const UNIT_TO_I24_SCALE: f64 = 8_388_608.0;

/// The scale that maps a unit sample onto the 32-bit range.
///
/// It is two to the power of 31, written as a product because each factor
/// prints exactly and the whole value does not. It is the divisor `I32_SCALE`
/// uses (ADR-0007).
const UNIT_TO_I32_SCALE: f64 = 32_768.0 * 65_536.0;
```

## Steps

1. Read `crates/duet-time/src/convert.rs`, `crates/duet-time/src/units.rs`, and
   `crates/duet-time/tests/kernel.rs`. Confirm that `I24_SCALE` is `8_388_608.0`, that
   `UNIT_TO_I24_SCALE` is `8_388_607.0`, that `UNIT_TO_I32_SCALE` is `2_147_483_647.0`, that
   `non_zero` and `non_zero_u32` each answer `None` with a `MIN` value, and that
   `I32_ROUND_TRIP_DRIFT` is 130. Report a discrepancy and stop.

2. Write the failing test for the exact 24-bit round trip. Add it to
   `crates/duet-time/tests/kernel.rs`, beside `convert_i24_round_trip_at_the_boundaries`. It walks
   every value from `I24::MIN` to `I24::MAX` and asserts equality. Name it
   `convert_i24_round_trip_is_lossless_over_every_sample`.

   ```rust
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
   ```

   **The loop bounds are `I24::MIN.get()` and `I24::MAX.get()`, which the block above already
   writes** (critic C1-6). `MIN_SAMPLE` and `MAX_SAMPLE` are private to `duet-time::units` and a
   test file is another crate, so neither name resolves here. `clippy.toml` sets
   `allow-panic-in-tests = true`, so the `else` arm is allowed. Run
   `cargo nextest run -p duet-time -E 'test(lossless)' --no-tests=fail` and confirm that it FAILS,
   **at the sample -8_388_608**, which is the first value of the loop. Under the pre-fix factor of
   2^23 - 1 that sample answers -8_388_607.

3. Change `UNIT_TO_I24_SCALE` to `8_388_608.0` and write the documentation the signature block
   above states. Run the test of step 2 and confirm that it passes. The encode body needs no other
   change: it already matches on `I24::new` and answers `None => I24::MAX`, which is the clamp for
   the one product a unit value of exactly +1.0 makes.

   **Rename the binding of the negative arm and state that the arm is unreachable** (critic C1-18).
   At a scale factor of 2^23 a unit value of -1.0 makes exactly -8_388_608.0, which `I24::new`
   ACCEPTS, so `None if scaled < 0 => I24::MIN` can no longer run. The arm stays, because a match
   over an `Option` must be total; that is the same reason architecture section 1.6 keeps the `None`
   arm of `non_zero`. The body becomes:

   ```rust
   pub fn unit_to_i24(value: Unit) -> I24 {
       let scaled = (f64::from(value.get()) * UNIT_TO_I24_SCALE).round() as i32;
       match I24::new(scaled) {
           Some(sample) => sample,
           None if scaled < 0 => I24::MIN,
           None => I24::MAX,
       }
   }
   ```

   The doc comment of the function states the unreachable arm in one sentence: a reader of a
   lossless kernel must not read the negative arm as the arm the clamp uses.

4. Repair the `#[expect]` reason of `unit_to_i24`. Appendix B.1 mandates the text character for
   character:

   ```rust
   #[expect(
       clippy::as_conversions,
       clippy::cast_possible_truncation,
       reason = "the `Unit` type carries the range -1.0 to 1.0 and the scale factor is 2^23, so the rounded product lies in -8_388_608.0 to 8_388_608.0, and `I24::new` refuses the one product above `I24::MAX`, which the match clamps"
   )]
   ```

5. Repair the `#[expect]` reason of `i24_to_f32` to the Appendix B.1 text:

   ```rust
   #[expect(
       clippy::as_conversions,
       clippy::cast_precision_loss,
       reason = "a 24-bit integer fits the f32 mantissa exactly and the divisor is 2^23, which is a power of two, so the conversion is lossless and the result is in the unit range"
   )]
   ```

6. Repair `I24_BOUNDARY_ROUND_TRIPS` in `crates/duet-time/tests/kernel.rs`. Every pair becomes an
   identity, and the doc comment states the new reason.

   ```rust
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
   ```

7. Repair `convert_i24_round_trip`, the proptest. The drift assertion and the half-scale branch both
   go, and one equality assertion replaces them.

   ```rust
           #[test]
           fn convert_i24_round_trip(raw in any::<i32>().prop_map(narrow_to_i24)) {
               let Some(sample) = I24::new(raw) else {
                   return Err(TestCaseError::fail("the folded value is a 24-bit sample"));
               };
               prop_assert_eq!(
                   unit_to_i24(i24_to_f32(sample)).get(),
                   raw,
                   "the 24-bit round trip answers itself"
               );
           }
   ```

   Keep the `expect` call of the current body if the test module already allows it; `clippy.toml`
   lets a test unwrap and expect. Run the whole `duet-time` suite and confirm that it passes.

8. Change `UNIT_TO_I32_SCALE` to `32_768.0 * 65_536.0` and write the documentation the signature
   block above states. Then repair the body of `unit_to_i32` so the clamp is a line a reader sees
   and not a property of the `as` cast:

   ```rust
   pub fn unit_to_i32(value: Unit) -> i32 {
       let scaled = (f64::from(value.get()) * UNIT_TO_I32_SCALE).round() as i64;
       match i32::try_from(scaled) {
           Ok(sample) => sample,
           Err(_outside_the_range) if scaled < 0 => i32::MIN,
           Err(_outside_the_range) => i32::MAX,
       }
   }
   ```

   The product of a unit value and 2^31 lies in -2_147_483_648.0 to 2_147_483_648.0, which every
   `i64` holds exactly, so the one cast is exact and `i32::try_from` is the checked narrowing. The
   match mirrors the `I24::new` match of `unit_to_i24`.

   **The binding is `_outside_the_range` and not `_above_the_range`** (critic C1-18). The name
   `_above_the_range` contradicts the guard beside it, `scaled < 0`, and a reader of a lossless
   kernel reads that arm as the one the clamp uses. **The negative arm is UNREACHABLE at a scale
   factor of 2^31**: a unit value of -1.0 makes exactly -2_147_483_648.0, which `i32::try_from`
   accepts. It stays for totality, exactly as the negative arm of `unit_to_i24` does, and the doc
   comment of the function states that in one sentence.

9. Repair the `#[expect]` reason of `unit_to_i32` to the Appendix B.1 text:

   ```rust
   #[expect(
       clippy::as_conversions,
       clippy::cast_possible_truncation,
       reason = "the `Unit` type carries the range -1.0 to 1.0 and the scale factor is 2^31, so the rounded product lies in -2_147_483_648.0 to 2_147_483_648.0 and the `as i64` cast is exact, and `i32::try_from` refuses the one product above `i32::MAX`, which the match clamps"
   )]
   ```

10. Repair the `#[expect]` reason of `i32_to_f32` to the Appendix B.1 text:

    ```rust
    #[expect(
        clippy::as_conversions,
        clippy::cast_precision_loss,
        reason = "a 32-bit integer rounds to the nearest f32, so the loss is below the 24th bit, and the divisor 2^31 is a power of two, which puts the result in the unit range and adds no further loss"
    )]
    ```

11. Repair `I32_ROUND_TRIP_DRIFT` and its derivation. The bound is 64 and no longer 130. **The
    bound is one half of the f32 spacing at the top of the 32-bit range, and no sample is the
    singular worst case**: an exhaustive scan of all 4,294,967,296 values finds a maximum of exactly
    64, and 1_073_741_888, 1_195_673_408 and 1_946_040_768 each attain it. `i32::MIN` and
    `i32::MAX` each round trip exactly.

    ```rust
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
    ```

12. Add the exactness half of the 32-bit statement to the proptest
    `convert_i32_round_trip`, beside the bound assertion it already carries.

    ```rust
                if i64::from(sample).abs() < 16_777_216 {
                    prop_assert_eq!(
                        i32_round_trip_drift(sample),
                        0,
                        "the 32-bit round trip is exact below a magnitude of 2^24"
                    );
                }
    ```

    Run the whole `duet-time` suite and confirm that it passes.

13. Repair the documentation and the `#[expect]` reason of `finite_to_f32_saturating`. The reason
    names no `LogicalPx`: that type is declared in `duet-command`, which architecture section 1.3
    puts above `duet-time`, and this function takes every `Finite` of every crate. Architecture
    Appendix B.1 item 3 states the decision. The mandated reason is:

    ```rust
    #[expect(
        clippy::as_conversions,
        clippy::cast_possible_truncation,
        reason = "every constructor of `Finite` refuses a NaN and an infinity, so the input is finite; the narrowing drops mantissa bits, and it saturates to an f32 infinity only above the f32 range, which the documentation of this function states"
    )]
    ```

    Rewrite the `///` block above it so it states the same two losses and names no caller type. It
    keeps the sentence that the function returns no `Result` and that `f64_to_f32` is the fallible
    form.

14. **The compile-time refusal carries NO committed test, and this step states the debt rather than
    claiming a test it does not have** (critic C1-5). Revision 24 opened this step with "Write the
    failing test", and the assertion it then described is GREEN on the pre-fix code: the current
    `non_zero_u32` answers `NonZeroU32::MIN`, which is 1, and every literal of
    `SampleRate::SUPPORTED` is already non-zero. A step whose first sentence promises a failing test
    and whose body describes a test that cannot fail is a step that teaches a reader to stop
    checking.

    **Extend `constants_hold_their_literal_values` anyway, and label what it is.** Add an assertion
    that `SampleRate::SUPPORTED.len()` is 6 and that every entry holds a non-zero rate, so the
    `non_zero_u32` path has a reader and a later edit that drops a rate is red. **This assertion is
    a reader and not a regression test for this repair**; write that in the chunk report and do not
    call it a proof of the fix. Run the suite and confirm that it passes.

    **The accepted debt, stated here.** Rust cannot declare a function that only a `const` item may
    call, so no test file can assert that the build fails. Step 17 is the proof, and it is a scratch
    edit the engineer makes, observes, and reverts. Follow-up item FU-4 of the plan store
    (`fluid:next_steps_from_T1`) proposes the mechanical guard for the condition that every caller
    of `Finite::from_finite_const`, `non_zero` and `non_zero_u32` binds the result to a `const`
    item. **FU-4 is NOT promoted into this act**; it stays in the plan store and this step is the
    site that names it. ADR-0008 records the same debt at its decision 5.

15. Repair `non_zero` in `crates/duet-time/src/units.rs`. Architecture section 1.6 mandates the body
    character for character, and section 12.3 house form 5 requires the `# Panics` section in the
    crate.

    ```rust
    /// A non-zero constant, built at compile time.
    ///
    /// `NonZeroI64::new` returns an `Option`, and `expect` is denied. House
    /// form 5 of architecture section 12.3 asserts instead: in a `const` item
    /// the assertion runs at compile time, so a zero fails the build and no
    /// binary carries the panic. The `None` arm stays, because a match over an
    /// `Option` must be total, and the assertion makes that arm unreachable.
    ///
    /// # Panics
    /// It panics when `value` is zero. Every caller binds the result to a
    /// `const` item, so the assertion runs at compile time.
    const fn non_zero(value: i64) -> NonZeroI64 {
        assert!(value != 0, "a non-zero constant must not be zero");
        match NonZeroI64::new(value) {
            Some(checked) => checked,
            None => NonZeroI64::MIN,
        }
    }
    ```

16. Repair `non_zero_u32` with the identical shape.

    ```rust
    /// A non-zero 32-bit constant, built at compile time.
    ///
    /// It is the 32-bit form of `non_zero`, and it states the same reason.
    ///
    /// # Panics
    /// It panics when `value` is zero. Every caller binds the result to a
    /// `const` item, so the assertion runs at compile time.
    const fn non_zero_u32(value: u32) -> NonZeroU32 {
        assert!(value != 0, "a non-zero constant must not be zero");
        match NonZeroU32::new(value) {
            Some(checked) => checked,
            None => NonZeroU32::MIN,
        }
    }
    ```

17. Prove the assertion at compile time, once, by hand. Change `TICKS_PER_QUARTER` to `non_zero(0)`
    in a scratch edit, run `cargo check -p duet-time`, and confirm that the build FAILS with the
    message `a non-zero constant must not be zero`. Restore the literal 1_920. Record the observed
    message in the chunk report. Do not commit the scratch edit.

18. Run the completion commands. Commit on a branch `chunk/m91-time-kernel-repairs` and let the
    native git hook run `scripts/dod.sh`.

## Tests

Every test lives in `crates/duet-time/tests/kernel.rs`, inside the existing `#[cfg(test)] mod tests`.
Every assert carries a message.

| Test | What it asserts | Where |
|---|---|---|
| `convert_i24_round_trip_is_lossless_over_every_sample` | `unit_to_i24(i24_to_f32(s)) == s` for each of the 16,777,216 values | `tests/kernel.rs`, a plain `#[test]` |
| `convert_i24_round_trip` | The same equality over the proptest strategy `any::<i32>().prop_map(narrow_to_i24)` | `tests/kernel.rs`, inside `proptest!` |
| `convert_i24_round_trip_at_the_boundaries` | The eight boundary pairs of `I24_BOUNDARY_ROUND_TRIPS`, each an identity | `tests/kernel.rs`, a plain `#[test]` |
| `convert_i32_round_trip` | The drift stays inside 64, and it is 0 below a magnitude of 2^24 | `tests/kernel.rs`, inside `proptest!` over `any::<i32>()` |
| `convert_i32_round_trip_at_the_boundaries` | The same bound at `i32::MIN`, `i32::MIN + 1`, -1, 0, 1, and `i32::MAX` | `tests/kernel.rs`, a plain `#[test]` |
| `constants_hold_their_literal_values` | `TICKS_PER_QUARTER` is 1_920, `SUPERCLOCK_HZ` is 282_240_000, and every `SampleRate::SUPPORTED` entry is non-zero. **It is a reader for the `non_zero_u32` path and it is GREEN on the pre-fix code**, so it is not a regression test for the step-15 and step-16 repair (critic C1-5) | `tests/kernel.rs`, a plain `#[test]` |

**The exhaustive test is a plain test and not an `#[ignore]` test.** It runs 16,777,216 iterations of
two arithmetic operations, which is inside the budget of a unit test on both platforms. Measure it
at step 2; if a run takes longer than ten seconds, report it and stop rather than mark it
`#[ignore]`, because architecture section 14 lists no new `#[ignore]` test for this chunk and SM7
binds the selected-test table.

**Step 17 is a compile-time proof and no test file holds it, and that is ACCEPTED DEBT.** Rust
cannot declare a function that only a `const` item may call, so no test can assert that the build
fails. The step is a scratch edit the engineer makes, observes, and reverts, and its evidence is the
observed compiler message in the chunk report. Follow-up item FU-4 of the plan store
(`fluid:next_steps_from_T1`) proposes the mechanical guard for the condition; **this act does not
promote it and this chunk does not carry it** (critic C1-5). ADR-0008 decision 5 states the same
debt.

## Verification

```
cargo nextest run -p duet-time -E 'test(lossless)' --no-tests=fail
cargo nextest run -p duet-time --no-tests=fail
cargo clippy -p duet-time --all-targets -- -D warnings
cargo test --doc -p duet-time
```

The first command fails before step 3 and passes after it. The whole suite is green. Clippy is clean
with no new suppression: every `#[expect]` this chunk writes replaces one that is already there, and
Appendix B.1 `b1-convert` holds all seven. Then commit on a branch `chunk/m91-time-kernel-repairs`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh`
  and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command
  is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site
  `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture
  Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is
  denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice
  indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only
  inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are
  required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every
  `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the
  root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no
  pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with
  Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this
  repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match
  what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
