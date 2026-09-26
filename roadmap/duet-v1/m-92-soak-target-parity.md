---
id: M92
line: M
depends_on: [M0, T1, M91]
write_scope:
  - crates/bc_time/duet-time/lang_rust/tests/proptest_large.rs
parallelism: independent
completion: "cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large_convert_i24_round_trip_is_lossless)' --no-tests=fail passes, and the same command fails before the chunk because no test of that name exists; the step-4 revert proof shows that same command RED against a scratch UNIT_TO_I24_SCALE of 8_388_607.0 and green again after the restore, and the chunk report quotes both outputs; cargo nextest run -p duet-time --no-tests=fail passes; cargo clippy -p duet-time --all-targets -- -D warnings is clean; cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large)' --no-tests=fail reports 6 tests run and 6 passed, which is the one command that EXECUTES the widened properties: property 1 with the three added constant refusals and property 6 over both meter fixtures; commit SHA on a branch chunk/m92-soak-target-parity"
---

# M92: The soak target asserts the lossless statement its headline claims

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This is a REPAIR chunk under rule SM9 of architecture section 13.0. It implements ADR-0007
decision 1, ADR-0007 decision 6, architecture section 2.3 (the round-trip table, row one), and
section 14 (the `soak.yml` row and the selected-test table). It writes one test file and it touches
no policy file and no crate source.

**Duet is a lossless editing system, and the soak target does not say so.**
`crates/bc_time/duet-time/lang_rust/tests/proptest_large.rs` opens with a module doc that claims parity with
`crates/bc_time/duet-time/lang_rust/tests/kernel.rs`: it names six properties and states that the gate proves each one
and that this target proves it again at the soak case count. The 24-bit property breaks that claim.
`kernel.rs` asserts `unit_to_i24(i24_to_f32(sample)).get() == raw` for every drawn sample. The soak
file asserts that the drift stays inside one least significant bit, and then asserts equality only
for a sample of magnitude 4,194,304 or below.

**A bound of one is green whenever the drift is zero, so the soak target can never go red for the
lossless gap that chunk T1 found.** A million cases of a weaker statement buy nothing over a
thousand cases of the same weaker statement. The half-scale branch is the second half of the same
defect: it asserted the exact property over the part of the range where the pre-repair scale factor
of 2^23 - 1 happened to be exact, and it left the part where that factor was wrong to the loose
bound.

An engineer executes this chunk. Chunk M91 repaired the code half, so the exact statement is already
true; this chunk makes the soak target assert it.

**This chunk CARRIES the `depends_on` edge to M91, and the guard accepts it.**
`tools/bc_repo_guard/xtask/lang_rust/src/check_plan_graph.rs` refuses a same-phase link only when the DEPENDENCY id does not
begin with `M`: the test is `if there == here && !dep.starts_with('M')`. `M91` begins with `M`, so
`depends_on: [M0, T1, M91]` is a clean run of both `cargo xtask check-plan-graph roadmap/duet-v1`
and `roadmap/duet-v1/tools/plan_graph_check.py`. **The narrower reading, that only the opening
manifest chunk of a phase may be a same-phase dependency, is WRONG**: the guard's own module doc
carries it and the guard's code does not, and an earlier revision of this chunk copied it. Code is
the source of truth.

**The edge is the order, and step 1 is the backup.** M92 asserts
`unit_to_i24(i24_to_f32(sample)).get() == raw` over the whole drawn range, and that statement is
true only at a scale factor of two to the power 23, which chunk M91 installs. Step 1 of this chunk
REREADS `crates/bc_time/duet-time/lang_rust/src/convert.rs` and STOPS when `UNIT_TO_I24_SCALE` is not `8_388_608.0`.
That check now backs up an encoded edge rather than standing in for a missing one, and it costs
nothing: it catches a tree that carries the edge and not the code, which no scheduler can see. The
two commits that landed the M91 repair are `de3ada1` and `0982f97`, and the constant is the check
that does not rest on a commit identifier.

## Files

| Path | Action |
|---|---|
| `crates/bc_time/duet-time/lang_rust/tests/proptest_large.rs` | modify (one module doc block, one renamed test, one repaired body, one new fixture helper, one widened property, three added assertions) |

## Types and signatures

This chunk declares no type and changes no signature. It reads two public functions of
`duet_time::convert`, which chunk T1 declared and chunk M91 repaired:

```rust
pub fn i24_to_f32(sample: I24) -> Unit;
pub fn unit_to_i24(value: Unit) -> I24;
```

## Steps

1. Read `crates/bc_time/duet-time/lang_rust/tests/proptest_large.rs` and `crates/bc_time/duet-time/lang_rust/tests/kernel.rs`. Confirm
   that `proptest_large_convert_i24_round_trip` binds `drift`, asserts `drift.abs() <= 1`, and
   carries an `if i64::from(raw).abs() <= 4_194_304` branch. Confirm that `convert_i24_round_trip`
   in `kernel.rs` asserts `prop_assert_eq!(unit_to_i24(i24_to_f32(sample)).get(), raw, "the 24-bit
   round trip answers itself")`. **Confirm that chunk M91 has landed**: read
   `crates/bc_time/duet-time/lang_rust/src/convert.rs` and confirm that it declares
   `const UNIT_TO_I24_SCALE: f64 = 8_388_608.0;`. Report a discrepancy and stop; a value of
   `8_388_607.0` means M91 is not on the branch and every assertion this chunk installs is red on
   correct work.

   **Confirm the two OTHER parity gaps this chunk repairs**, which step 5 closes. In
   `tests/kernel.rs`, `bbt_at_is_monotonic` loops `for map in [quarter_meter_map(),
   three_meter_map()]` and ends its message with "and across a segment boundary", while
   `proptest_large_bbt_at_is_monotonic` uses `three_meter_map()` alone. In the same file,
   `finite_rejects_non_finite` asserts three constant refusals over `f64::NAN`, `f64::INFINITY` and
   `f64::NEG_INFINITY` that `proptest_large_finite_properties` does not carry. Report a discrepancy
   and stop if either gap is absent, because step 5 then repairs something that is not there.

2. Confirm that the Completion filter matches nothing today. Run
   `cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large_convert_i24_round_trip_is_lossless)' --no-tests=fail`
   and confirm that it FAILS, because no test carries that name. Record the output in the chunk
   report. **The rename alone is NOT the proof**: step 4 is, and step 7 states why the chunk carries
   both.

3. Replace the whole `proptest!` block that holds the 24-bit property. The new block is exactly:

   ```rust
   proptest! {
       #![proptest_config(soak())]

       #[test]
       #[ignore = "the nightly soak target: only the soak.yml workflow runs it"]
       fn proptest_large_convert_i24_round_trip_is_lossless(
           raw in any::<i32>().prop_map(narrow_to_i24),
       ) {
           let sample = I24::new(raw).expect("the folded value is a 24-bit sample");
           prop_assert_eq!(
               unit_to_i24(i24_to_f32(sample)).get(),
               raw,
               "the 24-bit round trip answers itself"
           );
       }
   }
   ```

   The `back` binding, the `drift` binding, the `prop_assert!(drift.abs() <= 1, ...)` call, and the
   `if i64::from(raw).abs() <= 4_194_304 { ... }` branch all go. The assertion is the one
   `tests/kernel.rs` carries, character for character, including its message. **`prop_assert!` stays
   in the import list**, because `proptest_large_tempo_queries_are_monotonic` and
   `proptest_large_bbt_round_trips_over_a_multi_meter_map` still call it; confirm that with one
   search before the commit, and delete the import only if no caller is left.

4. Prove that the repaired test is red for the PROPERTY and not for the NAME. The rename makes the
   Completion command fail before step 3, and nothing more: an engineer who renames the test and
   keeps the `drift.abs() <= 1` body passes every command of this chunk. The repository already holds
   the answer, and ADR-0008 decision 5 with chunk M91 step 17 is the precedent: revert, observe,
   restore, and quote both outputs.

   1. In the working tree, change `UNIT_TO_I24_SCALE` in `crates/bc_time/duet-time/lang_rust/src/convert.rs` back to
      `8_388_607.0`, which is the pre-M91 value.
   2. Run
      `cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large_convert_i24_round_trip_is_lossless)' --no-tests=fail`
      and confirm that it goes RED. The failure names the assertion message
      `the 24-bit round trip answers itself`.
   3. Restore `UNIT_TO_I24_SCALE` to `8_388_608.0`, and confirm that
      `crates/bc_time/duet-time/lang_rust/src/convert.rs` matches its committed state again.
   4. Quote the RED output and the restored constant in the chunk report.

   **The revert is a scratch edit and it is NEVER committed.** `crates/bc_time/duet-time/lang_rust/src/convert.rs` is
   not in this chunk's write scope, and a commit that carries it is a write-scope breach the
   Orchestrator returns. The step exists because no `cargo` command over the committed tree can go
   red for this repair: chunk M91 already landed the lossless scale factor, so the weaker assertion
   and the exact assertion are both green on the tree this chunk starts from.

5. Close the two OTHER parity gaps of this file, because the module doc of step 6 claims that every
   property states EXACTLY what its gate tests state. Two of the six fall short today, and a doc
   that claims parity a file does not have is the defect this chunk exists to remove.

   **Property 6 asserts over ONE map where the gate asserts over TWO.**
   `crates/bc_time/duet-time/lang_rust/tests/kernel.rs` line 1537 loops `for map in [quarter_meter_map(),
   three_meter_map()]` and ends its message with "and across a segment boundary".
   `crates/bc_time/duet-time/lang_rust/tests/proptest_large.rs` uses `three_meter_map()` alone and carries the
   shorter message. Widen the soak property; do not weaken the doc.

   1. Add the `quarter_meter_map` helper beside `three_meter_map` in `proptest_large.rs`, copied
      from `tests/kernel.rs` line 627 with its `///` doc comment:

      ```rust
      /// A map of one meter entry of four quarter notes at the origin.
      fn quarter_meter_map() -> TempoMap {
          TempoMapEdit::new()
              .push_meter(meter_point(0, Bbt::ORIGIN, 4))
              .finish()
              .expect("a single meter entry at the origin is a valid map")
      }
      ```

      The file already imports `TempoMap`, `TempoMapEdit` and `Bbt`, and it already declares
      `meter_point`, so the helper needs no new import.

   2. Replace the body of `proptest_large_bbt_at_is_monotonic` with the gate body, at the soak case
      count:

      ```rust
      let low = Ticks::new(first.min(second));
      let high = Ticks::new(first.max(second));
      for map in [quarter_meter_map(), three_meter_map()] {
          prop_assert_ne!(
              map.bbt_at(low).lexicographic_cmp(map.bbt_at(high)),
              Ordering::Greater,
              "the address never falls as the tick rises, over the whole 64-bit range and across a segment boundary"
          );
      }
      ```

      The message is the gate message character for character.

   **Property 1 omits the three fixed refusals the gate asserts.**
   `finite_rejects_non_finite` in `tests/kernel.rs` asserts the drawn statement AND three constant
   statements: `Finite::new(f64::NAN).is_none()`, `Finite::new(f64::INFINITY).is_none()` and
   `Finite::new(f64::NEG_INFINITY).is_none()`. **Add the three assertions rather than hedge the doc
   line.** The drawn strategy reaches all three values, so the substance already holds, and the word
   EXACTLY does not; three constant assertions cost three `Option::is_none` calls per case, which is
   below the noise of one `f64` draw, and they make the doc line true rather than nearly true. Add
   them to `proptest_large_finite_properties`, ahead of the `if let` block, with the gate messages
   character for character:

   ```rust
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
   ```

   **The other three properties need no change**, and step 1 is where the engineer confirms it:
   property 2 compares `muldiv` against the same 128-bit reference, property 4 asserts the two
   monotonicity statements of `superclock_at_is_monotonic` and `ticks_at_is_monotonic` over
   `three_tempo_map()`, and property 5 asserts the same round trip as
   `bbt_round_trips_over_a_multi_meter_map` over a WIDER tick range, which is what a soak run is for.

   3. Run `cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large)' --no-tests=fail`
      and confirm that all six pass. **Record the wall-clock time of each property.** Property 6 now
      builds one extra `TempoMap` per case and runs one extra `bbt_at` pair, and the extra map holds
      ONE meter entry against the three of the map already in the loop, so the added half is the
      cheaper half and the body does at most twice the work it did. **If any property takes longer
      than sixty seconds, report it and stop** rather than lower the case count: `SOAK_CASES` is
      budget B78 and architecture section 14 binds it.

      **The measured margin is about twenty times, and the stop fires on the ENGINEER's local
      measurement.** A local run of this filter reports `6 tests run: 6 passed` in about three
      seconds, so the widening of property 1 and property 6 leaves the sixty-second stop far from
      the bound. `soak.yml` runs the same filter on a hosted runner, which is slower, so a local
      figure is a floor and not a ceiling; the stop exists to catch a widening that changes the
      order of magnitude, not to model the runner.

      **This command is in the `completion` field**, and it is the one command of this chunk that
      EXECUTES the widened properties. Every soak test carries `#[ignore]`, so a plain
      `cargo nextest run -p duet-time --no-tests=fail` skips all six.

6. Replace the module doc block. The new text is exactly:

   ```rust
   //! The nightly soak target: six kernel properties at the soak case count.
   //!
   //! Each property below states EXACTLY what the gate test or tests of
   //! `tests/kernel.rs` beside it state, and this target proves that same
   //! statement at the soak case count. Two of the six merge more than one
   //! gate test, so each line names every gate test it answers. **Property 5
   //! is the one that states MORE**, and its line says so: a soak property may
   //! draw a WIDER range than its gate test, because a stronger statement is
   //! still the gate statement, and it may never state a weaker one.
   //!
   //! 1. The `Finite` invariants: `finite_rejects_non_finite`,
   //!    `finite_equality_matches_bits` and `finite_order_matches_total_cmp`.
   //! 2. `muldiv` against a 128-bit reference:
   //!    `muldiv_matches_i128_reference`.
   //! 3. The 24-bit round trip, which is an EQUALITY over every drawn sample:
   //!    `convert_i24_round_trip`. Duet is a lossless editing system, and
   //!    ADR-0007 fixes both 24-bit scale factors at two to the power 23, so
   //!    `unit_to_i24(i24_to_f32(s))` answers `s`.
   //! 4. The `superclock_at` and `ticks_at` monotonicity:
   //!    `superclock_at_is_monotonic` and `ticks_at_is_monotonic`.
   //! 5. The `bbt_at` and `ticks_at_bbt` round trip:
   //!    `bbt_round_trips_over_a_multi_meter_map`. This one states MORE than
   //!    its gate test, by design: the same assertion and the same message
   //!    over a tick range of `0..=1_000_000_000` against the gate's
   //!    `0..=200_000`. A soak property may be STRONGER and never weaker.
   //! 6. The `bbt_at` monotonicity under `Bbt::lexicographic_cmp`, over both
   //!    meter fixtures: `bbt_at_is_monotonic`.
   //!
   //! A soak property never states a weaker bound than its gate twin. A weaker
   //! bound is green whenever the exact statement is green, so the longer run
   //! buys nothing at all. The form of this file before chunk M92 bounded the
   //! 24-bit drift at one least significant bit while `tests/kernel.rs`
   //! asserted equality, and it asserted equality only below half scale, so
   //! this target could not go red for the lossless gap that chunk T1 found.
   //!
   //! The three map properties draw over a wide range, so a longer run reaches
   //! inputs that the gate run does not. The tuplet sum is not one of the six:
   //! the whole domain of that rule is 99,840 pairs, and
   //! `tuplet_parts_sum_to_span` in `tests/kernel.rs` enumerates every one of
   //! them at the gate.
   //!
   //! The gate runs each property at the gate case count of B78. This target
   //! runs each one at the soak case count of B78. Every test here carries an
   //! `ignore` attribute, so the gate skips them. Section 14 of
   //! `roadmap/duet-v1/architecture.md` gives this target to the nightly
   //! `soak.yml` workflow, which chunk M7 writes.
   ```

7. Understand why the chunk carries BOTH a rename and a revert proof, because neither one is
   enough on its own. SM3 asks for one command that fails before the chunk and passes after it. The
   repaired assertion is TRUE on the code that chunk M91 already landed, and the old assertion is
   true on that code too, so no `cargo` command over the old name can separate the two states. The
   rename supplies the mechanical separator SM3 asks for, and the new name states the property the
   body now asserts. **The rename alone would be satisfied by a rename alone**: an engineer who
   renames the test and keeps the `drift.abs() <= 1` body passes every command of this chunk, so a
   Completion that rests on the name does not read the assertion it exists to install. **Step 4 is
   the half that reads the assertion**, and ADR-0008 decision 5 with chunk M91 step 17 is the
   precedent for it. **The `soak.yml` filter is `test(proptest_large)`**, which is a substring
   match, so the renamed test stays selected and the section 14 selected-test table needs no edit.
   Report both halves in the chunk report so no reader reads the rename as an accident.

8. Run `cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large_convert_i24_round_trip_is_lossless)' --no-tests=fail`
   and confirm that it PASSES. Record the wall-clock time. **If the run takes longer than sixty
   seconds, report it and stop** rather than lower the case count: `SOAK_CASES` is budget B78 and
   architecture section 14 binds it.

9. Run `cargo nextest run -p duet-time --no-tests=fail` and confirm that the gate suite is green.
   The soak tests carry `#[ignore]`, so this run skips them and proves that the file still compiles
   under the gate profile.

10. Run `cargo clippy -p duet-time --all-targets -- -D warnings` and confirm that it is clean. The
    chunk adds no `#[expect]` and removes none, so Appendix B.1 needs no edit.

11. Run `cargo test --doc -p duet-time` and confirm that it passes. The module doc block carries no
    code fence, so it adds no doc test.

12. Commit on a branch `chunk/m92-soak-target-parity` and let the native git hook run
    `scripts/dod.sh`.

## Tests

Every test of this chunk lives in `crates/bc_time/duet-time/lang_rust/tests/proptest_large.rs`, inside the existing
`#[cfg(test)] mod tests`. Every assert carries a message.

| Test | What it asserts | Where |
|---|---|---|
| `proptest_large_convert_i24_round_trip_is_lossless` | `unit_to_i24(i24_to_f32(sample)).get() == raw` over the strategy `any::<i32>().prop_map(narrow_to_i24)`, at the soak case count | `tests/proptest_large.rs`, inside `proptest!`, marked `#[ignore]` |
| `proptest_large_bbt_at_is_monotonic` | The address never falls as the tick rises, over BOTH meter fixtures, `quarter_meter_map()` and `three_meter_map()`. The message is the `bbt_at_is_monotonic` message of `tests/kernel.rs` | `tests/proptest_large.rs`, inside `proptest!`, marked `#[ignore]` |
| `proptest_large_finite_properties` | The drawn statements it already carried, plus the three constant refusals of `f64::NAN`, `f64::INFINITY` and `f64::NEG_INFINITY` that `finite_rejects_non_finite` asserts | `tests/proptest_large.rs`, inside `proptest!`, marked `#[ignore]` |

**This chunk writes no new property and it removes none.** It replaces one weaker statement with the
exact statement `convert_i24_round_trip` of `tests/kernel.rs` already carries, and it WIDENS two
others to the statements their gate tests already carry, so the count of soak properties stays at
six and the module doc keeps its list of six.

**The mechanical check is the pair of step 4 and step 8, and neither half stands alone.** Step 8
proves that a test of the new name exists and passes; step 4 proves that the same test goes RED when
the 24-bit scale factor moves back to `8_388_607.0`. A rename with the old body passes step 8 and
FAILS step 4, which is the property this chunk exists to install.

**THREE of the six properties change, and this chunk names all three.** Property 3 is the 24-bit
round trip, which step 3 replaces. Property 6 is the `bbt_at` monotonicity, which step 5 widens from
one meter fixture to both. Property 1 is the `Finite` invariants, which step 5 gives the three
constant refusals its gate test asserts. **Step 1 confirms each of the three gaps before step 3 and
step 5 repair them**, and it stops when a gap is absent.

**THREE properties are unchanged, and this chunk states why each one is already exact.** Property 2
compares `muldiv` against the same 128-bit reference as `muldiv_matches_i128_reference`. Property 4
asserts the two monotonicity statements of `superclock_at_is_monotonic` and `ticks_at_is_monotonic`
over `three_tempo_map()`, which is the same fixture. Property 5 asserts the round trip of
`bbt_round_trips_over_a_multi_meter_map` over a WIDER tick range, and a wider draw of one statement
is what a soak run is for. **Two of the six merge more than one gate test**, so the module doc names
every gate test rather than one twin.

**The SM0 stop stands for a mismatch this chunk does NOT name.** A gap in property 2, 4 or 5, or a
second gap in property 1, 3 or 6 beyond the three above, is a discrepancy: report it and stop,
instead of widening this chunk.

## Verification

```
cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large_convert_i24_round_trip_is_lossless)' --no-tests=fail
cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large)' --no-tests=fail
cargo nextest run -p duet-time --no-tests=fail
cargo clippy -p duet-time --all-targets -- -D warnings
cargo test --doc -p duet-time
```

The first command FAILS before step 3 with a no-match report, and it PASSES after step 3. **It also
goes RED under the step-4 revert**, against a scratch `UNIT_TO_I24_SCALE` of `8_388_607.0`, and the
failure names the message `the 24-bit round trip answers itself`; the revert is restored before the
commit and it never reaches the index. **Both outputs belong in the chunk report**: the name half
alone does not read the assertion.

**The second command is the one that EXECUTES the widened properties**, and it reports
`6 tests run: 6 passed`. Every soak test carries `#[ignore]`, so a plain `cargo nextest` run skips
all six and the first command selects property 3 alone; without this line the chunk would COMPILE
the widened property 6 and the three added property 1 refusals and run neither. `quarter_meter_map()`
is drawn against `bbt_at` at the soak case count here and nowhere else in the chunk.

The third command is green and skips every `#[ignore]` test. Clippy is clean and this chunk adds no
suppression. The doc test run is green. Then commit on a branch `chunk/m92-soak-target-parity`.

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
