# ADR-0007: Lossless sample conversion in `duet-time::convert`

## Status

Accepted. It repairs two rows of specification Appendix B.1 that chunk T1 proved wrong, and the
operator settled the substance on 2026-09-22.

## Context

Duet is a lossless editing system. The product opens a take, edits it, and writes it back, and a
user who does no processing expects the samples out to equal the samples in. `duet-time::convert`
is the one module of the workspace that narrows a number, so every sample of this product passes
through it.

Revision 23 of the specification fixed the decode divisor at 2^23 and the encode scale factor at
2^23 - 1. The composition of the two is `x * (1 - 2^-23)`, which is the identity for no sample of
magnitude 2^22 or above. Chunk T1 measured it: 8_388_607 answers 8_388_606, and -8_388_608 answers
-8_388_607. At unity gain a decode and a re-encode moved every sample above half scale by one least
significant bit, in the trunk crate of the product. The 32-bit pair carried the same defect, with
2^31 against 2^31 - 1.

Two more facts bear on the decision.

1. A 24-bit integer fits the f32 mantissa exactly. A 32-bit integer does not: the mantissa holds 24
   bits, so a sample of magnitude 2^24 or above rounds.
2. The encode body already matched on `I24::new` and answered `None => I24::MAX`. The 32-bit encode
   had no such match; it relied on the saturating behaviour of an `as` cast, which is a language
   rule and not a line of the body. Appendix B.1 opens with the rule that a suppression reason names
   an invariant a reader CHECKS in the body.

## Decision

1. **Both 24-bit scale factors are 2^23, and the 24-bit round trip is EXACT for all 16,777,216
   values.** `i24_to_f32` divides by 2^23 and `unit_to_i24` multiplies by 2^23. Both are powers of
   two, so neither scaling rounds.
2. **The one unit value of exactly +1.0 clamps, and the existing match arm is the clamp.** A scale
   factor of 2^23 sends +1.0 to 8_388_608.0, which is one above `I24::MAX`, and `I24::new` answers
   `None` there. The repair changes one constant and no control flow.
3. **Both 32-bit scale factors are 2^31, and the encode narrows with an explicit match.** The body
   rounds into an `i64`, which is exact over the whole product range, and narrows with
   `i32::try_from` in a match that mirrors the `I24::new` match. It relies on no saturating cast.
4. **The 32-bit round trip is EXACT at a magnitude of 2^24 or below and BOUNDED above it.** The
   bound is **one half of the f32 spacing at the top of the 32-bit range**. An `f32` carries a
   24-bit significand, so inside one binade every representable value sits one unit in the last
   place from the next. The widest binade an `i32` magnitude reaches is 2^30 up to 2^31, where that
   unit is 2^(30 - 23), which is 128; round to nearest moves a value by at most half of that, which
   is 64. The specification states the bound and claims no more, because the f32 mantissa cannot
   carry an i32 sample and no scale factor repairs that.

   **No sample is THE worst case.** An exhaustive scan of all 4,294,967,296 values finds a maximum
   of exactly 64, and many samples attain it: 1_073_741_888, 1_195_673_408 and 1_946_040_768 each
   drift by 64, and the first the ascending scan finds is -2_147_483_584. Revision 24 of the
   specification named one of them as the worst case, which is a singular claim the measurement does
   not support; this decision states the BOUND and gives a sample only as an example.
   **`i32::MIN` and `i32::MAX` each round trip exactly**: `i32::MIN` is -2^31, which an `f32` holds,
   and `i32::MAX` clamps back to itself through the match.

4a. **Both negative match arms are UNREACHABLE at these scale factors and both stay for totality.**
   A factor of 2^23 sends a unit value of -1.0 to exactly -8_388_608.0, which `I24::new` accepts,
   and 2^31 sends it to exactly -2_147_483_648.0, which `i32::try_from` accepts. A match over an
   `Option` and a match over a `Result` must each be total, which is the same reason ADR-0008 keeps
   the `None` arm of `non_zero`. **The 32-bit binding is named `_outside_the_range`**, because the
   name `_above_the_range` contradicts the `scaled < 0` guard beside it and a reader of a lossless
   kernel reads that arm as the one the clamp uses.
5. **`finite_to_f32_saturating` is not on the sample path, so this decision does not reach it.** An
   f64 to f32 narrowing drops mantissa bits by construction. Its reason text names the invariant its
   own body carries: every constructor of `Finite` refuses a NaN and an infinity. It names no
   `LogicalPx`, which is a `duet-command` type that `duet-time` cannot reach without an inversion of
   the specification section 1.3 crate graph.
6. **The required test of specification section 2.3 asserts the two statements separately.** The
   24-bit assert is equality over the whole range. The 32-bit assert is the bound of 64.

Chunk M91 lands the code. Specification section 2.3, Appendix B.1 `b1-convert`, and section 1.6 hold
the mandated text.

7. **A GUARD binds the appendix to the code, and chunk M90 adds it.** No rule compared an
   Appendix B.1 reason cell with the `reason =` string of the matching `#[expect]`, and that gap is
   the mechanism that let this defect live through a whole revision: the appendix stated one scale
   factor, `crates/duet-time/src/convert.rs` carried another, and every gate was green. Rule CG9 of
   specification section 2.3 makes the two one set in both directions and compares the text
   character for character. **Chunk M91 runs before chunk M90**, because M90's own gate is red until
   the differing reason strings are repaired; the `b1-convert` block holds seven rows, FIVE of them
   differ from the code today, and chunk M91 mandates all five.

   **CG9 is PATH CONDITIONAL, and that is part of the decision** (critic C2-1). `scripts/dod.sh`
   runs `cargo xtask check-conversions` on every commit with no path condition, so an unconditional
   CG9 would make every commit in this repository depend on one roadmap markdown file. The gate
   passes `--appendix roadmap/duet-v1/architecture.md` only when the change under test names a path
   that opens `roadmap/` or names `crates/duet-time/src/convert.rs`, and it reuses the three-clause
   denominator the plan step already computes. **Both halves of the binding are in the condition**,
   because a commit that edits the code half is exactly a commit that can break the binding.

   **The rule binds cell one and cell three, and cell two is UNBOUND** (critic C2-5). A `b1-convert`
   row names its lints in cell two; CG9 does not read that cell, so a function may carry a different
   lint with the exact cell-three reason and stay green.
   `clippy::unfulfilled_lint_expectation` is the partial enforcer, and review holds the rest.
   The compare reads the DECODED value of the string literal and not its raw source token.

## Consequences

Easier:

- A take that a user opens and writes back carries the same 24-bit samples. That is the product.
- One number, 2^23, serves both directions of the 24-bit pair, so a reader checks one fact instead
  of two.
- Every scale factor is a power of two, so a reviewer reasons about the mantissa alone.
- The 32-bit encode states its own clamp, so a reader needs no language reference.

Harder:

- A unit value of exactly +1.0 does not return itself. It clamps to `I24::MAX` or to `i32::MAX`, one
  least significant bit below full scale. Any symmetric integer scale has this property, and the
  specification states it rather than hiding it.
- The 32-bit round trip carries a stated bound and not an equality, so a test over that pair asserts
  a bound and reads less cleanly than the 24-bit one.
- The 32-bit encode body is longer by a match.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| Keep 2^23 - 1 as the encode factor and accept the one-bit error | It is a lossy editing system. The product's value is the opposite. |
| Change the DECODE divisor to 2^23 - 1 so the two agree there | The decode would then be inexact: 2^23 - 1 is not a power of two, so every sample would round. It also maps -8_388_608 outside the unit range. |
| Keep the saturating `as` cast in `unit_to_i32` and state the language rule in the reason | A reason must name an invariant a reader CHECKS in the body. A saturating cast is a fact about the compiler, and a reader has to leave the body to confirm it. |
| Use an f32 rather than an f64 intermediate in the encode | The product of a unit f32 and 2^31 does not fit the f32 mantissa, so the encode would round twice. The f64 holds every product of both pairs exactly. |
| Declare `LogicalPx` in `duet-time` so `finite_to_f32_saturating` can name it | Section 10.2 puts `LogicalPx` in `duet-command`, and section 1.3 puts `duet-command` above `duet-time`. The declaration would invert the crate graph for one doc sentence. |
| Narrow `finite_to_f32_saturating` to one caller's type | The function takes every `Finite` of every crate. A bound that names one caller is a promise the body cannot keep. |
