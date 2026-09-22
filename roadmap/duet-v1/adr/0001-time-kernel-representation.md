# ADR-0001: The time kernel representation

## Status

Proposed.

## Context

Duet holds notes and audio on one timeline. A note lives in musical time. A take lives in audio time. The two meet in every view, every region lookup, and every automation curve.

Ardour solved the same problem with two types (`/Users/james/Developer/ardour/libs/temporal/temporal/timeline.h`, research section 1). `timepos_t` is one integer plus a domain flag. `timecnt_t` is a distance that carries its own origin. Ardour stores audio time in superclocks rather than samples, so that a sample-rate change does not move existing material (`superclock.h`).

Three repository rules shape the answer.

1. `clippy::as_conversions` is denied, so a packed tag needs a suppression at every mask.
2. `clippy::integer_division` is denied, so every division is a method call.
3. `[profile.release]` sets `panic = "abort"` with `overflow-checks = true`, so one arithmetic overflow ends the process and loses a take.

The Engineering Critic raised four findings against the working hypothesis. A derived `Ord` gives a wrong cross-domain order (C1.1). "Audio clock" names no unit (C1.2). The tick resolution is not exact for a septuplet (C1.3). A packed tag costs more than it saves (C1.4). The revision-4 review added R0, R14, and R7 against the same kernel.

## Decision

Every number this record relies on is a row of specification section 1.6, cited by its id. Every
rule is cited by its id. This record carries decisions and consequences only.

1. **Musical time is `Ticks`, an `i64` newtype at B40 ticks per quarter note.** A triplet eighth and
   a quintuplet sixteenth are both exact.
2. **Audio time is `SuperClock`, an `i64` newtype at B41 ticks per second.** That rate divides
   exactly by every supported sample rate. A sample count appears only at the device edge.
3. **`Position` is an enum over the two time domains, and order is a `TempoMap` query.** A derived
   order would place every beat position before every audio position, whatever the real time.
   Specification section 2.5 holds the declarations and the reason for each derive.
3a. **`duet-time` owns `Finite`, and one guarded constructor is the only way in.** The guarded
   invariants make `to_bits` a total, injective key, which is what lets the type carry equality, a
   hash, and an order at all. **The order is `f64::total_cmp`, not the bit order.** A bit order puts
   every negative value above every positive one, so a gain curve would sort wrongly on disk.
   Specification section 2.6a holds the declaration, the invariants, and every impl.
3b. **`Finite` deserializes through that same constructor.** A derived `Deserialize` writes the
   inner field directly, so `Eq` and `Hash` would lie about a value that came from disk. `LogicalPx`
   inherits the invariants and `Unit` carries no serde derive at all. Specification section 2.6a
   holds the attribute and the reason.
3c. **A type derives what its own use needs, and the demand never points upward.** VR1 states which
   trait follows the fields and which one a caller chooses, and the VR1 table is the one list of
   those uses. **The rule is about the direction of the demand, not about a crate.** A
   `duet-command` type derives what its own use needs, exactly as a lower type does. Specification
   section 3.5 holds VR1. Revision 4 inverted the direction and the vocabulary crate did not
   compile.
3d. **`Finite` derives `Default`, and a compile-time constant goes through an asserting
   constructor.** The derived value is the canonical zero, so the derive breaks no invariant and the
   signal-block state types keep their own `Default` derive. Specification section 2.6a holds both
   forms, and specification section 1.6 declares every constant that the second form builds.
4. **`Span` is a `Delta` plus the `Position` where it starts.** A duration has no length in samples
   until the origin is known.
5. **`Position` implements no `Add` and no `Sub`.** Named methods replace them, and specification
   section 2.7 declares each one.
6. **Every conversion takes the map as an explicit argument.** No thread-local map exists anywhere
   in the workspace.
7. **`TempoMap` is a sorted point list, and each point carries every address it needs.** A lookup
   therefore never calls back into the map. Specification section 2.9 holds the declarations.
8. **A tuplet divides its span so that the last part absorbs the remainder.** One rule answers every
   divisor, so a finer tick resolution buys nothing. Specification section 2.4 holds the arithmetic
   and the sum property, and B43 is the range the test covers.
9. **Arithmetic on a time newtype is checked or saturating.** Specification section 2.2 holds the
   forms, and specification section 5.11 holds the ones the audio path uses.
10. **`duet-time::convert` is the one module in the workspace that may hold a cast, and
    `cargo xtask check-conversions` proves it under the CG rules specification section 1.7 indexes,
    one test per rule (DR5).** The guard needs no build, so it is
    the earlier signal, and clippy stays the backstop. **The guard carries no file-count floor.** A
    floor stopped chunk M0 from passing its own Completion command, and the file set the guard draws
    from `cargo metadata` makes a floor redundant. Specification section 2.3 holds every rule and
    every excluded form, and specification section 1.9 holds the one probe of each rule.
11. **A suppression is justified at its own site, by a checked type or by a checked range.** Every
    reason is therefore an invariant a reader checks in the body, not a promise a caller made.
    Specification section 2.3 states both forms, and specification Appendix B.1 holds every reason
    text.
12. **No constant uses `expect`.** Specification section 1.6 declares every workspace constant and
    the form that builds one with no panic primitive.
12a. **A time value never becomes a bare integer in a command.** Every command field and every
    automation curve point carries a time newtype. Specification section 3.4 and specification
    section 7.2 declare them.
13. **`Bbt` derives no `Ord`.** A bar marker can reset the address, so `Ticks` is the one sort key on
    the timeline. Specification section 2.8 holds the declaration.

## Consequences

Easier:

- A cross-domain comparison cannot be written by mistake. The compiler rejects it, where Ardour
  rejects it by convention.
- The vocabulary crate compiles, and it keeps compiling when a payload type is added. `Verb` asks
  its payloads for no `Hash` and no `Ord`.
- A sample-rate change moves nothing, because the audio unit is rate independent.
- A tuplet round-trips exactly over the whole test range, and a test proves the sum rule.
- Every conversion is visible in a signature, so a test needs no thread setup.
- An overflow cannot end the process, because no bare arithmetic exists on a time value.
- One audited module holds every lossy numeric conversion in the product, with round-trip proptests,
  and a guard that reads the whole workspace proves the module is the only one.
- A hand-edited canonical file cannot smuggle a NaN past the type system, because the constructor is
  on the deserialization path.

Harder:

- Many methods carry a `&TempoMap` parameter. That is the price of an explicit argument.
- `Position` is two words rather than one. It never appears in a per-sample loop, so nothing
  measurable is lost.
- `Position` can be a `HashMap` key, and it cannot be sorted by `sort()`. Code sorts through the
  map instead.
- Structural equality says two positions in different domains are never equal, even when they name
  the same instant. That is correct for a cache key and wrong for a timeline question, and the two
  method names keep the difference visible.
- Every arithmetic call site is longer. `a.saturating_add(b)` replaces `a + b`.
- `Finite` needs hand-written impls, which specification section 2.6a declares. A derive over an
  `f64` field does not compile, so the code was always going to be hand written.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| One `i64` with the tag packed into two spare bits, as Ardour does | Every mask and extract needs an `as` conversion, which the lint policy denies. Each one would need an `#[expect]` with its own invariant. The saving is in a type that never enters a per-sample loop. |
| `f64` seconds as the one unit | Float drift breaks an exact tuplet and an exact round trip. The research file records that Ardour converts with integer `muldiv`, never with a float. |
| A raw sample count as the audio unit | A project recorded at one rate and opened at another would shift every take. |
| Derive `Hash` on the vocabulary and push the requirement down | Revision 4 did that with `Eq` and `Hash` together. Each payload type then had to carry a trait it had no use for, and one missing derive broke the whole enum. `Eq` is no longer a choice, because the workspace denies `clippy::derive_partial_eq_without_eq`, and VR1 keeps `Hash` and `Ord` to the uses its own table names. |
| Hold a fixed-point integer in every vocabulary variant | It would remove the float, and it would put a scale decision in every parameter: a gain in millibels, a pan in thousandths, a curve value in an unknown unit. `Finite` keeps one number type and moves the guard into the constructor. |
| Let `Finite` keep `-0.0` distinct from `0.0` | `to_bits` would then give two keys for one number, so a map would hold two entries for one gain. |
| Let `Finite` derive `Deserialize` | The derive writes the inner field directly, so a hand-edited file could build a value the constructor would reject. |
| Derive no comparison trait at all, as the first draft did | `Verb` then cannot derive `PartialEq`, so the agent has no way to compare its last request with what `Status` reports. |
| Derive `Ord` on `Position` and accept the cross-domain order | The derive compares the discriminant first, so every beat position sorts before every audio position. The answer is plausible and wrong, and no test finds it until two domains meet in one list. |
| A thread-local tempo map, as Ardour's `TempoMap::use()` | It makes a conversion depend on hidden state and needs an assertion to catch misuse. The research file lists it under "concepts to reject". |
| Two separate types, `BeatPosition` and `AudioPosition`, with no shared `Position` | Every region, marker, and curve would need two code paths, so the product could not hold one timeline. |
| A finer tick resolution, to make a septuplet exact | It makes one divisor exact and leaves others inexact. The remainder rule solves every divisor at once and costs one method. |
