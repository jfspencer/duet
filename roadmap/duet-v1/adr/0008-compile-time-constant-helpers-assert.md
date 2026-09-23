# ADR-0008: A compile-time constant helper asserts, so a bad constant fails the build

## Status

Accepted. It repairs the `non_zero` body that specification section 1.6 mandates, and it covers the
second helper that chunk T1 added.

## Context

`clippy::unwrap_used` and `clippy::expect_used` are denied, so a `const` item cannot go through
`Option::expect`. The workspace therefore carries compile-time helper functions that take a literal
and return a checked newtype.

Two helpers of this shape existed after chunk T1, and they answered a bad input in opposite ways.

1. `Finite::from_finite_const` asserts. In a `const` item the assertion is evaluated at compile
   time, so a value that is not finite FAILS THE BUILD and no binary carries the panic.
   Specification section 12.3 names this house form 5.
2. `duet-time::units::non_zero` matches on `NonZeroI64::new` and answers `None => NonZeroI64::MIN`,
   which is `i64::MIN`. A zero constant therefore SHIPS, as the most negative integer. Chunk T1 then
   added `duet-time::units::non_zero_u32` with the same wrong arm.

Both `TICKS_PER_QUARTER` and `SUPERCLOCK_HZ` are positive divisors everywhere in `duet-time`. A
negative value for either would invert the whole timeline, and `SampleRate::SUPPORTED` would carry a
rate of one rather than a refusal.

## Decision

1. **Every compile-time constant helper of this workspace asserts first.** The body opens with
   `assert!(value != 0, "a non-zero constant must not be zero");` and then matches as before.
   `clippy::missing_assert_message` is denied, so the assertion carries its message.
2. **The `None` arm stays, and the specification states why.** A match over an `Option` must be
   total. The assertion above it makes the arm unreachable, so the value it names can no longer
   reach a binary.
3. **Both `duet-time::units::non_zero` and `duet-time::units::non_zero_u32` take the shape.** The
   32-bit form is identical with `NonZeroU32`.
4. **Each helper carries a `# Panics` section in the crate**, which house form 5 requires. Section
   12.3 now names three sites and no longer one. **REVIEW holds this rule, and no lint does.**
   `clippy::missing_panics_doc` fires on an EXPORTED item alone, and both `NonZero` helpers are
   private to `duet-time::units`, so the lint reads neither. Revision 24 of the specification named
   that lint as the enforcer at its section 1.6 site, which was false; the requirement is real and
   its stated enforcer was not.
5. **The condition of house form 5 binds every site**: a `const` item is the only caller. A
   `const fn` is an ordinary function at run time, and `[profile.release]` sets `panic = "abort"`,
   so one run-time call with a refused value would end the process. The `finite!` macro and the
   `#[doc(hidden)]` marker hold that condition for `Finite`; the two `NonZero` helpers are private
   to `duet-time::units` and every caller in that module is a `const` item.
6. **The compile-time refusal carries NO committed test, and this decision ACCEPTS that debt.**
   Rust cannot declare a function that only a `const` item may call, so no test file can assert that
   the build fails. Chunk M91 step 17 is the proof: the engineer changes one constant to
   `non_zero(0)`, runs `cargo check -p duet-time`, observes the refusal, reverts the edit, and
   quotes the compiler message in the chunk report. **The extension of
   `constants_hold_their_literal_values` that M91 step 14 describes is a READER for the
   `non_zero_u32` path and not a regression test for this repair**: it is green on the pre-fix code
   AND on the repaired code, because `NonZeroU32::MIN` is 1 and every literal of
   `SampleRate::SUPPORTED` is already non-zero. **A test that is green on every state of the code
   under repair can never go red for that repair**, and the phrase "green on the pre-fix code" reads
   as if the repaired code were the state that turns it red.
   Follow-up item FU-4 of the plan store (`fluid:next_steps_from_T1`) proposes the mechanical guard
   for the condition of decision 5. **This act does not promote FU-4**; the item stays in the store
   and this decision is the site that names the gap.

Chunk M91 lands the code. Specification section 1.6 holds the mandated body, character for
character, and section 12.3 holds the house form.

## Consequences

Easier:

- A wrong constant is a build failure with a message, in every crate that takes the form. A reviewer
  needs no test to know that.
- The workspace has one answer to a refused compile-time value, so a reader of one helper knows all
  three.
- The existing test that asserts the literal values of both constants keeps its purpose and gains a
  second line of defence it does not have to carry alone.

Harder:

- The body is one line longer and it keeps an unreachable arm, so a reader has to be told why the
  arm is there. The specification tells the reader at the site.
- Form 5 now has three sites rather than one, so the condition it carries has to hold at three
  places. Follow-up item FU-4 of the plan store proposes a mechanical guard for that condition, and
  this record does not decide it.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| Keep the fallback arm and add a unit test that asserts each literal | A test proves the two constants of today. It proves nothing about the next constant an author adds, and the arm is the thing that lets a wrong one ship. |
| `assert!(false, ...)` inside the `None` arm | `clippy::assertions_on_constants` denies it, and the arm would still have to produce a value. |
| `NonZeroI64::new_unchecked` | It is `unsafe`, which the workspace denies with no exception. |
| A macro that refuses a zero literal at the call site | A macro cannot evaluate an arithmetic expression the way a `const fn` can, so it would accept a literal and refuse a constant expression. The assertion covers both. |
| Return `Option<NonZeroI64>` and let each `const` item unwrap it | `expect` is denied, so a `const` item cannot unwrap. |
