# Specification review, revision 9: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is the ninth pass. Revisions 1 to 8 each returned NOT READY.

Sources read in full: `roadmap/duet-v1/architecture.md` (7701 lines), the six ADRs,
`research/linux-macos-platform.md`, `product-requirements.md` section 8, `design-contract.md`
sections 0, 1, 3.1, 3.3, 4.3, 4.4 and the `StageCurve` appendix, `CLAUDE.md`, the root `Cargo.toml`,
`.cargo/config.toml`, `deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/` directory. I ran both guards on the real inputs. I ran all thirty recorded probes.
I added six probes of my own to the conversion guard and eight to the placement guard. I ran one
compiler run in a throwaway cargo crate. I ran nine mechanical passes. I wrote no repository file.
I ran no other git command.

**Revision 9 keeps every promise it makes about the guards.** All thirty recorded probes reproduce
the recorded exit code and the recorded message, including the two new rules PG4b and PG18 and the
two new conversion rules CG3b and CG4b. The baseline run matches section 1.9 line for line, and
`UNDECLARED: 0` is true: the 1.5 table places 358 names and the document declares every one. The
deletion of ID1 is the right call and it closes the revision-8 Critical by construction.

It does not close. The deletion of ID1 moved 186 undeclared names into 358 real declarations, and
**no guard rule reads any derive except `Copy`, and no guard rule reads the section 1.3 edge list
against a declaration.** Four Criticals and sixteen Warnings block it. Every one of the four
Criticals is a declaration this document states and a compiler refuses.

---

## 1. Closure check on R1 to R12 and C1 to C16

| Id | Finding | State | Section and reason |
|---|---|---|---|
| R1 | `CommitId` is a git object identifier and ID1 gave it a `u64` | **CLOSED** | 3.5 deletes ID1, section 15 declares every placed name, `CommitId(ObjectId)` and `ObjectId([u8; 20])` sit in 15.5, PG4b exists and PP4b goes red. I measured `UNDECLARED: 0`. |
| R2 | PG10b and PG14 drop `AtomicU32` and seven more types | **CLOSED** | The drop list is gone. The 1.9 external table carries 40 rows, PG18 refuses a name it omits, and PP10's second shape plants `reset: AtomicU32` in `MeterState` and goes red. |
| R3 | CG3b removes a span that holds a real cast | **CLOSED** | `type_path` is strict and symmetric on both sides of the `as`. CP3b now plants a span with no bracket. My revision-8 shape `a<b && c as u32 > d` is caught. |
| R4 | CG4 blanks the rest of a file after an unterminated `use` | **CLOSED** | CG4b exits 2 with the recorded message. CG4 requires statement position, and the macro-pattern shape blanks nothing. CG4extra confirms the later cast is found. |
| R5 | B86 refuses a project that three MUST stories build | **CLOSED** | B86 is 48 and its arithmetic is exact: 32 tracks, 8 part buses, 4 reverb, 2 delay, 1 monitor, 1 master. `MeterSnapshot` at 1.6 kilobytes follows. |
| R6 | The over-mark reset cannot serve its own verb | **PARTIAL** | 7.3 gives a per-strip generation array and covers `hold`. No type stores the last-seen generation, and the stopped-engine path asks `duet-core` to write a buffer the audio thread owns. See W2. |
| R7 | `ViewState::default` does not exist | **PARTIAL** | `first_open` and B91 to B95 exist. `from_finite_const` cannot be a `const fn`, `ModeView` has no constructor, B91 to B95 are not constants, and ADR 0005 16c still says `default`. See W3, W4, W11. |
| R8 | The threading contract omits three threads | **CLOSED** | 5.7 carries ten rows with an owner and a "Never does" cell each, TH10 binds the MIDI thread, and 8.3 cites TH10 rather than TH1. |
| R9 | Four chunks modify `top_bar.rs` and none owns it | **OPEN** | 13.2 still reads "K2, K3, K4, and K5 each add their own group to the file K1 created". `ModeToolbar` lives in one doc comment. See W5. |
| R10 | The section 1.5 denominator is stale in all three numbers | **CLOSED** | The three counts are gone from 1.5. Section 1.9 prints them, and my baseline run reproduces every line. |
| R11 | `MixState::validate` cannot name the strip refusal | **CLOSED** | 15.4 declares `MixError::StripBudgetExceeded`, 15.10 declares `ConfigError::Mix(MixError)`, 5.5 names both callers, and ADR 0004 12a1 agrees. |
| R12 | Three ADR decisions state a replaced rule | **PARTIAL** | ADR 0004 6b, ADR 0004 12a1, and ADR 0005 9 are all rewritten. ADR 0005 16c and ADR 0004 12b are stale now. See W11. |
| C1 | Four 1.9 rows name the wrong crate | **CLOSED** | The justified-unknown table is one row, and the guard prints `UNKNOWN: 1  JUSTIFIED: 1`. |
| C2 | PG17 checks one direction only | **CLOSED** | The limit sentence sits beside the table, in PG15's shape. |
| C3 | A negated edge claim gives a false failure | **CLOSED** | `edge_claims` reads both sides of the verb. I probed "never writes" and "writes no": both pass. The positive control still fails. |
| C4 | CP1 and CP3 do not reproduce on macOS | **CLOSED** | The guard canonicalizes the root before `relpath`. Both probes printed the recorded relative path in a macOS temporary directory. |
| C5 | VR1 has three unnamed derives and one false reason | **PARTIAL** | The three named types are fixed and `PartialOrd` is out of the rule. Section 15 added eight more types that derive `Eq`, `Hash`, or `Ord` with no VR1 row. See W16. |
| C6 | The header says Revision 7 | **CLOSED** | Line 3 reads Revision 9. |
| C7 | Two backend crates are in the graph and in no chunk | **CLOSED** | 1.3 drops both and states the measurement that would start them. |
| C8 | `acceptance.md` has no chunk | **CLOSED** | Rung three holds the seven judgements and names no file. |
| C9 | The `Clear clip` command has no action | **CLOSED** | `ClearClip` is in the `actions!` list, in the 1.5 table, and in 15.16. |
| C10 | `EngineFault::ClockDrift` has two shapes | **CLOSED** | 5.4 uses the declared struct form and states why. |
| C11 | The 12.4 refusal table lists two `EngineState` variants | **CLOSED** | Both moved to the engine-state sentence. |
| C12 | Four more ADR statements are stale | **PARTIAL** | ADR 0002 3b and 3c, ADR 0005 2a, and ADR 0001 are fixed. ADR 0002 decision 2 still states an `extra` bag on every record, which decision 3c contradicts. See W8, W11. |
| C13 | `quick-xml` has a second manifest owner | **CLOSED** | B.3 names M2. |
| C14 | The `StageCurve` appendix names `duet.meter.peak` | **CLOSED** | The appendix names `duet.meter.peak_cap`. |
| C15 | Virtualization has no type, owner, or chunk | **OPEN** | `VirtualList` appears only in the 1.3 framework block. Appendix C cites a paragraph in 10.3 that does not exist. See W13. |
| C16 | ADR 0006 keys the Mix path cache by a scalar | **OPEN** | `ZoomScalar` is an undeclared, unplaced name in Appendix A and ADR 0006. See W14. |

**Count: 20 CLOSED, 5 PARTIAL, 3 OPEN.**

### 1.1 The baseline run, as I measured it

```
DOCUMENT:        roadmap/duet-v1/architecture.md
FRAMEWORK NAMES: 51    NAME MAP: 12
CANDIDATE TYPES: 360   DECLARED: 358
TABLE NAMES:     358     UNDECLARED: 0
EXTERNAL ROWS:   40     EXTERNAL MISSING: 0
PLACED:          360
UNPLACED:        0     DUPLICATED:   0
MISCLAIMED:      0     FRAMEWORK MISUSE: 0
COPY MISSING:    0     COPY IMPOSSIBLE:  0
COPY UNDECIDED:  0     UNKNOWN: 1     JUSTIFIED: 1     UNJUSTIFIED: 0
EDGES PARSED:    54    EDGE CLAIMS BAD: 0
DEP ROWS:        16    DEP MISSING: 0
SNAPSHOTS:       2     SNAPSHOT BAD: 0
REGISTER BAD:    0
TESTS SELECTED:  5     TEST ROWS BAD: 0
EXIT=0
```

Every line matches section 1.9. The conversion guard on the repository tree gives
`MEMBERS: 3   FILES: 6   FINDINGS: 0`, exit 0, through a canonical root and through a symbolic
link alike. The tree holds exactly six `.rs` files outside `target/`, so the denominator is whole.

### 1.2 The thirty recorded probes

Every probe reproduced the recorded exit code and the recorded message.

| Probe | Result | Probe | Result |
|---|---|---|---|
| PP1 | exit 2, `usage: placement_check.py <architecture.md>` | PP11 | exit 1, `EDGE MISS: duet-time -> duet-project` |
| PP2 | exit 2, `FAIL: cannot open ...; the guard is fail-closed.` | PP12 | exit 1, `DEP MISS: duet-score uses serde through Accidental` |
| PP3 | exit 1, `CANDIDATE TYPES: 0   DECLARED: 0` | PP13 | exit 1, `SNAPSHOT: MeterSnapshot: the declaration derives no Copy` |
| PP4 | exit 1, `UNPLACED: ProbeUnplaced` | PP14 | exit 1, `COPY UNDECIDED: 1` |
| PP4b | exit 1, `UNDECLARED: 1`, `UNDECLARED: VoiceId ...` | PP15 | exit 1, `REGISTER: duet-command declares MidiMessage in 8.3` |
| PP5 | exit 1, `DUPLICATED: Knot` | PP16 | exit 1, `TEST MISS: soak: no row in the selected-test table` |
| PP6 | exit 1, `MISCLAIMED: Pixels claimed by duet-command` | PP17 | exit 1, `UNJUSTIFIED: 1` |
| PP7 | exit 1, `FRAMEWORK: duet-command::ModeView holds Px` | PP18 | exit 1, `EXTERNAL: Rest names Value, which no 1.9 row decides` |
| PP8 | exit 1, `UNPLACED: Reverb` | CP1 | exit 1, `CAST: m/benches/bench.rs` |
| PP9 | exit 1, `COPY: duet-score::Spanner needs Copy or an #[expect]` | CP2 | exit 0, `FINDINGS: 0` |
| PP10 | exit 1, both shapes: `MidiRecord` over `MidiPortId` and `MeterState` over `AtomicU32` | CP3 | exit 1, `CAST: m/src/lib.rs` |
| PP10b | exit 1, `NOT DECIDED: duet-session::InputSelection derives Copy over DeviceKey` | CP3b | exit 1, `FINDINGS: 1` |
| CP4 | exit 0, `FINDINGS: 0` | CP5 | exit 0, `FINDINGS: 0` |
| CP4b | exit 2, ``FAIL: m/src/lib.rs: a `use` at line 10 has no `;``` | CP6 | exit 1, `FINDINGS: 2`, one per form |
| CP7 | exit 0, `MEMBERS: 2   FILES: 3   FINDINGS: 0`, direct and through a link | CP8 | exit 2, the fail-closed line |

Both stated extra checks hold. CG5 reports two findings over a file that holds one qualified path
and two real casts. CG4 reports the cast that follows a `macro_rules!` pattern which holds `use`.

### 1.3 My own fourteen probes

| Probe | Shape | Result | Caught |
|---|---|---|---|
| MINE-P1 | `duet-engrave` holds a `duet-session` type, an edge 1.3 omits | exit 0 | **No (S2)** |
| MINE-P2 | A `Default` derive over `Finite`, which derives no `Default` | exit 0 | **No (S1)** |
| MINE-P3 | A `Serialize` derive over a type that derives none | exit 0 | **No (S4)** |
| MINE-P4 | A type name that lives only in a doc comment | exit 0 | No, and 1.5 states the limit (W5) |
| MINE-P5 | An unnecessary `missing_copy_implementations` expectation | exit 0 | **No (S3)** |
| MINE-P6 | A 1.2 row that omits a crate used in a function body | exit 0 | No, and PG12 states the limit |
| MINE-P7 | An `Eq` and `Hash` derive that VR1 names no use for | exit 0 | **No (W16)** |
| MINE-P8 | One name placed in two crate rows at once | exit 1, `REGISTER` | Yes, by side effect |
| MINE-P9 | A field type no block declares and no table names | exit 1 | Yes, PG18 and PG10b |
| MINE-P10 | An edge claim with a verb outside the closed list | exit 0 | No, and PG11 states the limit |
| MINE-C1 | A cast between two strings that form `/*` and `*/` | exit 0 | **No (W10)** |
| MINE-C2 | A cast after a `//` inside a string on one line | exit 0 | **No (W10)** |
| MINE-C4 | A member with no `.rs` file, to test the denominator | exit 2 | Yes, `cargo metadata` refuses first |
| MINE-C5 | An injected root that points at a clean sub-workspace | exit 0 | No (Concern C4) |

### 1.4 The nine mechanical passes

1. **A type declared twice: none.** `DUPLICATED: 0` over 360 candidates.
2. **A B id with no row: none.** 95 rows, and every citation resolves. The only row nothing else
   cites is B62, which ADR 0006 carries.
3. **A number outside 1.6: none.** Sixteen unit-bearing literals remain, and every one is a type
   size in bytes, which DR3 exempts.
4. **A rule id nothing states: none.** DR1 to DR6, PL1 to PL3, PG1 to PG18 with PG4b and PG10b,
   PP1 to PP18 with PP4b and PP10b, CG1 to CG8 with CG3b and CG4b, CP the same, VR1 to VR6, TH1 to
   TH10, and SM0 to SM7 all appear.
5. **Chunk coverage: exact.** 55 chunks in 13.3, 55 rows in 13.1 and 13.2, no phase mismatch, and
   no chunk in one table and not the other.
6. **SM6 holds.** No line runs two chunks in one phase. The trunk chunks T1 to T4 are four lines of
   one chunk each, which 13.1 states.
7. **The plan graph is acyclic and forward.** I expanded every cross-line link of 13.4 against the
   13.3 phases. No link runs backward. One same-phase link, M0 before T1, which 13.4 states.
8. **Write scopes are disjoint per phase.** The only shared names are `Cargo.lock`, `src/lib.rs`,
   and `tests/`. SM5 rule 4 resolves the first; the other two are relative to two crate directories.
9. **MUST story coverage is exact.** 64 MUST rows in the product requirements, 64 rows in 13.2, and
   the two sets are identical. Every filtered `nextest` command carries `--no-tests=fail`.

---

## 2. New findings in revision 9

### CRITICAL: S1. Ten declarations derive `Default` over `Finite`, and section 10.2 states that `Finite` derives no `Default`

**OBSERVATION.** Section 2.6a declares `Finite` with `#[derive(Debug, Clone, Copy, Serialize,
Deserialize)]` and five hand-written impls. It carries no `Default`. Section 10.2 states the
consequence as a fact: "`Finite` derives no `Default`, so the derive chain stops there"
(`architecture.md:4779`).

Ten declarations derive `Default` over a `Finite` field.

```
architecture.md:3755   #[derive(Debug, Clone, Copy, Default, PartialEq)]  pub struct MeterReading { peak: Finite, ... }
architecture.md:6174   #[derive(Debug, Clone, Copy, PartialEq, Default)]  pub struct GainState { current: Finite, ... }
architecture.md:6186   #[derive(Debug, Clone, Copy, PartialEq, Default)]  pub struct BiquadState { b0: Finite, ... }
```

The full set is `MeterReading` (7.3) and, in 15.2, `GainState`, `MeterState`, `BiquadState`,
`CompressorState`, `GateState`, `DeEsserState`, `DelayState`, `ReverbState`, and `EqualizerState`,
which reaches `Finite` through `[BiquadState; 4]`. `GateState` also derives `Default` over
`FrameCount`, which derives no `Default` either.

**CLAIM.** Ten of the eleven `duet-dsp` state types do not compile. This is the same class as the
revision-8 Critical R7, and section 10.2 names the exact fact that makes it true.

**ARGUMENT.** A derived `Default` on a struct requires `Default` on every field type. `Finite` has
one constructor, `Finite::new`, which returns an `Option`, so a derive cannot build one. The design
chose that on purpose: ADR 0001 decision 3b states that the constructor is the one way in.

The chunk that is hit is C3, which writes `ChainState`, and D1 and D3, which write every state
type. Each one reads the block in section 15.2 as "the declaration of record" (section 15 preamble)
and writes it verbatim. The compiler then refuses, and the chunk stops under SM0 with no stated
answer.

The deeper defect is the reason it reached revision 9 at all. **Every guard rule reads the `Copy`
derive and no other.** PG9, PG10, PG10b, PG14, and PG13 all read `Copy`. PG18 reads a name and not
a derive. Section 15 added 358 declarations with five derive families, and the guard family checks
one of them.

**EVIDENCE.** `architecture.md:1243` to `:1257` (`Finite`), `:4779` (the sentence), `:3755`,
`:6174` to `:6207` (the nine `duet-dsp` blocks). MINE-P2: I added
`#[derive(Debug, Clone, Copy, PartialEq, Default)] pub struct ProbeDefault { value: Finite }` and
placed it in the 1.5 table. The guard exited 0 and every counter stayed at zero. My own derive
closure pass over all 358 declarations reports nine direct failures and one transitive failure.

**WHAT SHOULD CHANGE.** Give `Finite` a `Default` impl that returns `ZERO`, or give each of the ten
types a named constructor, as `ViewState::first_open` already does. State which one, once. Then add
a guard rule that closes the derive family: for each of `Default`, `Serialize`, `Deserialize`,
`Eq`, `Hash`, and `Ord`, a declaration that derives the trait must reach no field type that does
not. Give it a probe that plants this exact shape.

### CRITICAL: S2. Sixteen field references cross a crate edge that section 1.3 does not carry, and five of them are Cargo cycles

**OBSERVATION.** Section 1.3 states "This list is the whole internal graph". PL1 reads "A type
crosses a crate edge only when the list above carries that edge". I compared every field type of
every declaration against the list.

| Declaration | Crate | Field type | Lives in | Cycle |
|---|---|---|---|---|
| `Applied.inverse_cost` | `duet-score` | `InverseCost` | `duet-core` | **Yes** |
| `GatewayError::Engine` | `duet-command` | `EngineError` | `duet-engine` | **Yes** |
| `GatewayError::Project` | `duet-command` | `ProjectError` | `duet-project` | **Yes** |
| `GatewayError::Export` | `duet-command` | `ExportError` | `duet-export` | **Yes** |
| `DomainEvent::Transport` | `duet-command` | `TransportCommand` | `duet-engine` | **Yes** |
| `GatewayError::Interchange` | `duet-command` | `InterchangeError` | `duet-interchange` | No |
| `VerbData::Loudness` | `duet-command` | `LoudnessReport` | `duet-analysis` | No |
| `Track.part`, `Track.staff` | `duet-session` | `PartId`, `StaffId` | `duet-score` | No |
| `SessionCommand::TrackAdd` | `duet-session` | `PartId`, `StaffId` | `duet-score` | No |
| `BusRole::Part` | `duet-session` | `PartId` | `duet-score` | No |
| `PitchTrack.source` | `duet-analysis` | `SourceHash` | `duet-session` | No |
| `MusicXmlImport.warnings` | `duet-interchange` | `ImportWarning` | `duet-command` | No |
| `SmfImport.warnings` | `duet-interchange` | `ImportWarning` | `duet-command` | No |
| `ProjectError::Media` | `duet-project` | `MediaError` | `duet-media` | No |

**CLAIM.** The workspace does not build. Five references make a Cargo cycle, and eleven more name a
crate the manifest will not carry.

**ARGUMENT.** Section 1.3 lists `duet-session -> duet-time` and nothing else, and rule 4 of the
same section states "`duet-session` names no engine type". `Track` is the central session entity,
and section 6.5 states that "A `Track` names exactly one `PartId` and one `StaffId`". So the design
needs the edge `duet-session -> duet-score`, and the graph omits it.

The five cycles are worse, because no manifest edit fixes them. `duet-core -> duet-score` exists,
so `duet-score -> duet-core` closes a loop; `InverseCost` must move to `duet-score` or lower.
`duet-engine -> duet-command`, `duet-project -> duet-command`, and `duet-export -> duet-command`
all exist, so `GatewayError` cannot hold any of their error types. That is blocker V1 reopened, and
ADR 0005's own rejected-alternatives table names it: "The command types in `duet-command`, with the
aggregate below it | Cargo refuses the cycle."

The guard is blind here by construction. PG11 reads prose sentences that join two crate names with
one of nine verbs. It never reads a declaration. PG12 reads a declaration and maps it to a
**third-party** crate through the 1.2 name map. No rule maps a declaration to an internal crate.
Revision 9 added 358 declarations to that blind spot.

**EVIDENCE.** `architecture.md:200` to `:218` (the edge list), `:249` (rule 4), `:1655`
(`Applied`), `:6622` to `:6642` (`GatewayError`), `:6506` (`DomainEvent`), `:6514` to `:6523`
(`VerbData`), `:1694`, `:1733`, `:3434` to `:3436` (`duet-session`), `:6706` (`PitchTrack`),
`:1963`, `:4231` (`duet-interchange`), `:6919` to `:6931` (`ProjectError`). MINE-P1: I added
`probe_strip: StripId` to `duet-engrave::FontMetrics`, which no edge reaches. The guard exited 0.

**WHAT SHOULD CHANGE.** Decide each of the sixteen. Move `InverseCost` into `duet-score`. Replace
the four `GatewayError` arms that name an upper crate with a boxed string or a `duet-command` error
of its own, and state the conversion site. Add `duet-session -> duet-score`,
`duet-analysis -> duet-session`, `duet-interchange -> duet-command`, and
`duet-project -> duet-media` to the 1.3 list, and rewrite rule 4 and the 1.2 rows to match. Then
add a guard rule that reads every field type of every declaration against the 1.3 list, with a
probe that plants one absent edge.

### CRITICAL: S3. Three `#[expect(missing_copy_implementations)]` sites in a binary crate are build errors, and I proved it with the compiler

**OBSERVATION.** Section 15.16 opens: "Every type here is `pub(crate)`. `crates/duet` is a binary,
so `unreachable_pub` makes that the only correct visibility". Three declarations in that section
carry a `missing_copy_implementations` expectation.

```
architecture.md:7111   #[expect(missing_copy_implementations, ...)]  pub(crate) struct PunchRange
architecture.md:7118   #[expect(missing_copy_implementations, ...)]  pub(crate) struct LevelMeter
architecture.md:7137   #[expect(missing_copy_implementations, ...)]  pub(crate) struct LufsMeter
```

**CLAIM.** `missing_copy_implementations` does not fire on a type that is not externally reachable,
so each expectation is unfulfilled, and `-D warnings` turns an unfulfilled expectation into an
error. The three declarations do not build.

**ARGUMENT.** The rustc lint checks a publicly reachable type. A `pub(crate)` type in a binary
crate is reachable from nothing outside the crate. `unfulfilled_lint_expectations` is a rustc lint
at `warn`, and `.cargo/config.toml` adds `-D warnings`, which is the exact mechanism VR5 relies on
in the other direction.

I did not reason about this. I built a throwaway binary crate with `#![deny(warnings)]`,
`#![warn(missing_copy_implementations)]`, and one `pub(crate)` struct whose fields are all `Copy`,
carrying the same expectation. `cargo build` printed:

```
error: this lint expectation is unfulfilled
 --> src/main.rs:7:5
  = note: `#[deny(unfulfilled_lint_expectations)]` implied by `#[deny(warnings)]`
```

VR5's sentence is correct for a library and wrong for this crate, and the document applies it to
both. Chunk K3, K4, and K5 each write one of the three files and each one stops.

**EVIDENCE.** `architecture.md:6979` (the visibility sentence), `:7111`, `:7118`, `:7137`,
`:1810` (the VR5 mechanism sentence), the compiler run above.

**WHAT SHOULD CHANGE.** Delete the three expectations and state at VR5's site that the rule binds a
library crate, because the lint reads reachability. Then give VR5 the size answer for an
application element, which is the real question those three reasons were written to answer.

### CRITICAL: S4. Three `Serialize` derives reach a type that derives none, and two of them break the VR2 compile-time assertion

**OBSERVATION.** Three declarations derive `Serialize` and `Deserialize` over a field whose type
derives neither.

| Declaration | Section | Field | Field type declared with |
|---|---|---|---|
| `Snapshot.score` | 15.5 | `Box<CanonicalDocument>` | `#[derive(Debug, Clone, PartialEq)]` (15.3) |
| `VerbData::Loudness` | 15.5 | `Box<LoudnessReport>` | `#[derive(Debug, Clone, Copy, PartialEq)]` (15.8) |
| `JobState::Failed` | 9.6 | `GatewayError` | `#[derive(Debug, Clone, PartialEq, Error)]` (15.5) |

**CLAIM.** Three declarations do not compile, and the assertion that section 3.5 built to catch
this exact class cannot compile either.

**ARGUMENT.** Section 3.5 holds VR2 with a compile-time block that calls
`assert_plain_data::<T>()` for eight types, and the bound is
`T: Serialize + DeserializeOwned + Send + 'static`. `Snapshot` and `VerbData` are two of the eight.
So the block fails to build for two reasons at once: the derive on each type fails, and the
assertion over it fails.

`JobState` is worse, because it crosses the transport twice. `DomainEvent::Job { job, state }` and
`VerbData::Job { job, state }` both carry it, and the Model Context Protocol tool schema is
generated from the `serde` derive (section 9.2). A `GatewayError` that carries `EngineError`,
`ProjectError`, and six more enums has no serde derive anywhere in its chain.

This is the same blind spot as S1, in a second derive family, and it reaches the one mechanism the
document names as its own guard.

**EVIDENCE.** `architecture.md:6510` (`Snapshot`), `:6521` (`VerbData::Loudness`), `:4540`
(`JobState::Failed`), `:6317` (`CanonicalDocument`), `:6710` (`LoudnessReport`), `:6621`
(`GatewayError`), `:1841` to `:1855` (the VR2 block). MINE-P3: I planted a `Serialize` derive over
`FontMetrics`, which derives none. The guard exited 0.

**WHAT SHOULD CHANGE.** Add `Serialize` and `Deserialize` to `CanonicalDocument` and
`LoudnessReport`. Decide what `JobState::Failed` carries across the transport: a `GatewayError`
with a full serde chain, or a stable code plus a message. State it once. Then extend the derive
closure rule of S1 to `Serialize` and `Deserialize`, and give it a probe.

### WARNING: W1. B90 is arithmetically false at B86, and `MAX_PARAMS` carries no refusal at all

**OBSERVATION.** B90 reads: "512. `MAX_PARAMS`, the automatable parameters one project holds. It
sizes `ParamSnapshot` at B86 strips times one trim, one polarity, one fader, one pan, and B45 slot
parameters" (`architecture.md:583`).

B86 is 48 and B45 is 8. The row's own formula gives 48 times 12, which is 576.

**CLAIM.** The value is 512 and the stated derivation gives at least 576. The real floor is higher,
and no error variant refuses the surplus.

**ARGUMENT.** At revision 8 B86 was 41, and 41 times 12 is 492, which fits 512. Revision 9 raised
B86 to 48 to close R5 and did not re-derive B90. The direction of the error is silent.

The true floor is higher than 576. `ChainTopology` holds `pre_fader` and `post_fader`, each an
`ArrayVec` at B45, so a chain has 16 slots and not 8. `SlotConfig` carries a
`ParamRange { first, count }`, so one slot addresses many parameters: `CompressorState` holds six
`Finite` values. At one parameter per `Finite`, a project at B1 needs several thousand.

The consequence is the exact shape of Q5 and R5, with the loud half removed. `ParamSnapshot` holds
`[Finite; MAX_PARAMS]` and the audio thread addresses it by `ParamId`. A `ParamId` at or above 512
has no cell. `configure` refuses a strip past B86 and a ring past B57; it refuses no parameter.
`ConfigError` carries no `ParamBudgetExceeded`, and `MixError` carries none either. A fader ride on
the 513th parameter is a silent loss in the mix.

**EVIDENCE.** `architecture.md:583` (B90), `:574` (B86), `:533` (B45), `:2784` to `:2786`
(two slot lists), `:6401` (`SlotConfig`), `:6191` (`CompressorState`), `:6793` (`ParamSnapshot`),
`:6820` to `:6829` (`ConfigError`), `:6444` to `:6454` (`MixError`).

**WHAT SHOULD CHANGE.** Recompute B90 from the real slot count and the real parameter count per
slot kind, and write the arithmetic into the row. Add `ConfigError::ParamBudgetExceeded` and
`MixError::ParamBudgetExceeded`, with a 12.4 message, in the shape the strip budget already has.
Then state whether `ParamSnapshot` stays `Copy` at its new size, because VR5's 32-byte bound gives
no answer for a four-kilobyte value.

### WARNING: W2. The per-strip reset generation has nowhere to store the last-seen value, and the stopped-engine path writes a buffer the audio thread owns

**OBSERVATION.** Section 7.3 states the mechanism. "The audio thread reads its own strip's entry
once per cycle with a relaxed load and clears that strip's `OverMark` when the value **changed**"
(`architecture.md:3774`).

`MeterState` is the audio thread's per-strip meter state.

```
architecture.md:6179   pub struct MeterState { reading: MeterReading, over: OverMark, decay: Finite }
```

**CLAIM.** "Changed" needs a last-seen value, and no declared type holds one. The
`[AtomicU32; MAX_STRIPS]` array also has no declared owner and no declared type.

**ARGUMENT.** A relaxed load gives the current generation. To see a change the audio thread must
compare it with the value it read on the previous cycle. `MeterState` has three fields and none of
them is a generation. `ChainState` holds `meter: MeterState` and nothing else for the meter.
`GraphState` holds `chains`, `pool`, `generation`, and `pool_generation`, and the last two are the
topology and pool counters that section 5.6 defines.

The array itself is worse. Section 5.8 names `[AtomicU32; MAX_STRIPS]` as a row, and section 15.10
declares no field of any engine type that holds it. An atomic array shared between the core thread
and the audio thread needs an owner and a lifetime, and this design gives it neither.

The second half is a contradiction, not a gap. Section 7.3 states: "In those five states
**`duet-core` clears the held marks in the published snapshot itself**" (`:3784`). Section 5.8
places `MeterSnapshot` on an `Audio -> UI` row through `triple_buffer`, and section 5.8's own rule
is that "A `triple_buffer` reader borrows and never owns". The audio thread is the writer.
`duet-core` is not on the path at all: Appendix A names the frame driver as the one observer.
`duet-core` cannot write that value.

**EVIDENCE.** `architecture.md:3768` to `:3787`, `:3173` (the 5.8 row), `:6179` (`MeterState`),
`:2812` to `:2821` (`ChainState`), `:3025` to `:3032` (`GraphState`), `:3143` to `:3147` (the
high-rate table), `:3149` to `:3153` (the borrow rule), `:7172` (Appendix A).

**WHAT SHOULD CHANGE.** Add a `reset_seen: Generation` field to `MeterState`, and name the type
that owns the atomic array together with the field of `GraphState` or `ChainState` that holds it.
Then replace the stopped-engine sentence: either the mixer view hides the latch while
`EngineState` is not `Running`, or `duet-core` sends a `CoreEvent` that the frame driver reads.
State which, and put the row in section 5.8.

### WARNING: W3. `from_finite_const` cannot be a `const fn`, and B91 to B95 are not workspace constants

**OBSERVATION.** Section 10.2 declares the constructor.

```
architecture.md:4774   pub const fn from_finite_const(value: f64) -> Finite;
```

Its doc comment states the body: "This `const fn` matches instead, and the fallback arm is `ZERO`".
The match is over `Finite::new`.

```
architecture.md:1250   pub fn new(value: f64) -> Option<Finite>;
```

**CLAIM.** A `const fn` cannot call a function that is not a `const fn`. The declared constructor
cannot be written.

**ARGUMENT.** Section 1.6 carries the working form next door: `non_zero` is a `const fn` that
matches on `NonZeroI64::new`, which the standard library declares `const`. `Finite::new` is not
declared `const` in section 2.6a, and it cannot be one while it stays the one guarded constructor,
because a `const fn` cannot run the negative-zero canonicalization with a float comparison on a
stable toolchain without care that no section states.

The second half is the same paragraph. Section 10.2 states: "chunk K1 replaces them with the
constants of section 1.6, which is what DR3 requires" (`:4791`). The section 1.6 constant block
declares twelve constants, and none of them is B91, B92, B93, B94, or B95. K1 therefore has no
constant to substitute, and DR3's own rule that "a literal appears in no other Rust block" leaves
the five numbers with no home.

**EVIDENCE.** `architecture.md:4766` to `:4792`, `:1247` to `:1257`, `:587` to `:636` (the constant
block), `:36` to `:38` (DR3).

**WHAT SHOULD CHANGE.** Declare `Finite::new` as a `const fn`, or replace `from_finite_const` with
a `Finite::ZERO`-based constant plus a run-time constructor, and state which. Add the five
constants to the section 1.6 block beside `MAX_STRIPS`, in `duet-command`.

### WARNING: W4. `ViewState::first_open` leaves `per_mode` empty, and `ModeView` has no first-open constructor

**OBSERVATION.** Section 10.2 states the value field by field. "`per_mode` is **empty**, so each
mode takes its own first-open layout the first time the user enters it. The window is B91 wide and
B92 high at B93 from the display origin, **the sidebar is B94 wide, and the body split is B95**"
(`architecture.md:4787` to `:4790`).

`sidebar_width` and `body_split` are fields of `ModeView`, not of `ViewState`.

**CLAIM.** The paragraph names two values that `first_open` does not set, and the type that would
hold them has no constructor and can derive no `Default`.

**ARGUMENT.** `ModeView` holds `sidebar_width: LogicalPx`, `inspector_width: LogicalPx`,
`body_split: Finite`, `mix_vertical_split: Finite`, `zoom: ZoomStep`, `scroll: ScrollOffset`,
`selection: Selection`, and `shown_tracks: TrackFilter`. A derived `Default` needs `Default` on all
eight. `Finite` has none, `Selection` is an enum with no default arm, and `TrackFilter` is another.
So the derive chain stops in the same place it stopped for `ViewState`, and R7's fix did not follow
it down one level.

Two more values have no first-open row at all. `inspector_width` and `mix_vertical_split` carry no
B id, and design contract 1.6 states a preferred inspector width and a 45 percent Mix split. Chunk
K6 writes `ViewStateStore` and chunk F1 writes the persistence, and neither has a value to write.

**EVIDENCE.** `architecture.md:4714` to `:4735` (`ModeView` and `ViewState`), `:4776` to `:4791`,
`:578` to `:582` (B91 to B95), `design-contract.md:113` to `:130` (section 1.6).

**WHAT SHOULD CHANGE.** Declare `ModeView::first_open(mode: Mode) -> ModeView` beside
`ViewState::first_open`, and state its value for all eight fields. Add a B row for the inspector
width and for the Mix vertical split. Then rewrite the 10.2 paragraph so that it names only the
fields `ViewState::first_open` sets.

### WARNING: W5. R9 is not closed. `ModeToolbar` exists in one doc comment, and no chunk owns the per-mode toolbar

**OBSERVATION.** Appendix C row R9 reads: "13.2 the `ModeToolbar` seam; the four `toolbar.rs` stubs
in K1's scope" (`architecture.md:7664`).

The name `ModeToolbar` appears twice in the whole plan: in that row, and in one doc comment.

```
architecture.md:7065   /// The top bar shell. It hosts one `ModeToolbar` per mode (section 13.2).
```

Section 13.2 states no `ModeToolbar` seam. It carries the revision-8 sentence unchanged: "`TopBar`
shows a per-mode group set, so K2, K3, K4, and K5 each add their own group to the file K1 created"
(`:5717`). K1's write scope holds one `toolbar.rs` stub, not four, and that file is
`crates/duet/src/element/toolbar.rs`, which chunk K6 fills in phase 12.

**CLAIM.** The defect R9 named is unchanged, the closure row cites a seam that no section holds,
and the type it names is declared nowhere. Appendix C's own rule makes that a defect.

**ARGUMENT.** Appendix C opens with "A row that cites a section which does not hold the mechanism
is a defect that returns to the Architect" (`:7346`). This row cites two things: a seam in 13.2 and
four stubs in K1's scope. Neither exists.

The plan consequence is unchanged. K2 to K5 must edit `crates/duet/src/shell/top_bar.rs`, which
sits in K1's write scope and in no other. SM0 makes each of the four stop.

There is a second ordering defect underneath. `Toolbar` is the element that carries the overflow
rule of contract 1.5, and K6 fills it in phase 12. K2 runs in phase 8, K3 in 9, K4 in 10, and K5 in
11. Each one adds a top-bar group to a container that is still a stub.

`ModeToolbar` is also a name that section 1.5 does not place and section 15 does not declare. The
guard cannot see it: the candidate set comes from fenced Rust blocks, and section 1.5 states that
limit. MINE-P4 confirms it: I planted a second ghost name in the same comment and the guard exited
0.

**EVIDENCE.** `architecture.md:7065`, `:7664`, `:5717`, `:5686` to `:5691` (the K rows), `:4895`
(the `Toolbar` row), `:5808` to `:5814` (the phases), `:7346` (the Appendix C rule).

**WHAT SHOULD CHANGE.** Declare `ModeToolbar` in section 15.16 and place it in section 1.5. Create
four stubs, `crates/duet/src/shell/toolbar_{compose,record,mix,master}.rs`, in K1's write scope and
name one in each of K2, K3, K4, and K5. Move the `Toolbar` element from K6 to K1, because four
later chunks build on it. Then rewrite the 13.2 sentence and the Appendix C row.

### WARNING: W6. `MidiPortMap` has two owners again, because the presence listener must mint the value it pushes

**OBSERVATION.** Section 5.7 gives the platform presence listener its own thread row: it does
"Anything but classify a port and push one `HotplugEvent` at B87" (`architecture.md:3067`).

Section 8.1 gives the map one owner. "**The MIDI thread is the one owner of `MidiPortMap`, and no
other thread holds a reference** (TH10, section 5.7). `insert` and `retire` take `&mut self` ... all
three run on the MIDI thread" (`:4002`).

Section 8.1 also states what the event carries. "A `HotplugEvent` that the PipeWire registry or the
CoreMIDI callback produced carries a `MidiPortInfo` that `insert` minted" (`:4015`).

**CLAIM.** The listener thread is the producer of `HotplugEvent`, and `HotplugEvent` carries a
`MidiPortInfo`, and only `insert` mints one, and `insert` runs on the MIDI thread. Two threads need
`&mut MidiPortMap`.

**ARGUMENT.** Follow the value. `HotplugEvent::PortAdded(MidiPortInfo)` is the declared shape.
`MidiPortInfo { slot: PortSlot, id: MidiPortId, is_source: bool }` carries a `PortSlot`, which the
map mints and which `insert` is the one source of. The queue at B87 has one producer, which section
8.1 step 1 names as the listener thread, and one consumer, which is `duet-core`.

So the listener must either call `insert`, which TH10 forbids, or push something the map has not
seen, which the declared `HotplugEvent` cannot carry. This is critic Q6 with the owner moved rather
than removed, and the new 5.7 row is what makes it visible.

Elastic and Message Driven both suffer. A shared `&mut` across that boundary needs a lock, and TH10
forbids a block on the MIDI thread. There is no third mechanism in section 5.8.

**EVIDENCE.** `architecture.md:3067` (the 5.7 row), `:3088` to `:3091` (TH10), `:3922` to `:3926`
(`MidiPortInfo`, `HotplugEvent`), `:3989` (`insert`), `:4002` to `:4007`, `:4015` to `:4019`,
`:4030` to `:4034` (the four steps), `:3174` (the 5.8 row).

**WHAT SHOULD CHANGE.** Make the listener push a `PlatformPort` and give the queue that payload.
Let the MIDI thread drain it, call `insert`, and publish a second bounded queue of `HotplugEvent`
to the core. State both rows in section 5.8 with their own bounds. Then rewrite 8.1 step 1 and the
5.7 listener row so that the producer and the map owner are one thread.

### WARNING: W7. Appendix B.1 lists three `missing_copy_implementations` sites and the document holds eight

**OBSERVATION.** VR5 reads: "A type that must not be `Copy` carries a single-site
`#[expect(missing_copy_implementations, reason = "...")]`, and **Appendix B.1 lists every such
site**" (`architecture.md:1801`). B.1 closes with "Fifteen sites are predicted in total"
(`:7235`).

The document carries eight such expectations: `ManifestEntry` (4.3), `SlotState` (5.5), `Transport`
(5.9), `Source` (6.1), `ConfigError` (15.10), `PunchRange`, `LevelMeter`, and `LufsMeter` (15.16).
B.1 lists three.

**CLAIM.** Five suppressions are absent from the list that VR5 makes authoritative, and the stated
total is wrong by five.

**ARGUMENT.** B.1 states the consequence itself: "A suppression outside this list is a plan defect
and returns to the Architect" (`:7197`). So the chunks that write `SlotState` and `ConfigError`,
which are C1 and C3, hit a plan defect by the document's own rule, and both stop under SM0.

The count is also load-bearing for the Orchestrator. B.1 opens with "Each row needs a decision from
the Orchestrator before the chunk that contains it is dispatched" (`:7191`). Five decisions are not
on the list to be made. Three of the five are S3 and do not belong in the code at all.

Nothing checks this. PG9 skips any declaration that carries the attribute and never compares the
set with B.1.

**EVIDENCE.** `architecture.md:1801` to `:1808` (VR5), `:2178`, `:2824`, `:3201`, `:3374`, `:6814`,
`:7111`, `:7118`, `:7137` (the eight sites), `:7222` to `:7235` (B.1).

**WHAT SHOULD CHANGE.** Add `SlotState` and `ConfigError` to B.1 with their reason text, delete the
three application sites under S3, and set the total. Then give the placement guard a rule that
compares every `#[expect(missing_copy_implementations)]` site against the B.1 table, with a probe
that plants a site the table omits.

### WARNING: W8. Property 4 of section 3.6 requires an `extra` bag that four score types cannot hold

**OBSERVATION.** Section 3.6 property 4 reads: "**Every record** of `score/meta.json`,
`score/notes.jsonl`, and `score/spanners.jsonl` carries `#[serde(flatten)] extra: BTreeMap<String,
serde_json::Value>`" (`architecture.md:1891`).

`score/spanners.jsonl` holds `Spanner`.

```
architecture.md:1521   /// A mark that spans more than one note. It is 32 bytes, so VR5 gives it `Copy`.
architecture.md:1523   pub struct Spanner { id: SpannerId, kind: SpannerKind, from: NoteId, to: NoteId }
```

`score/meta.json` holds `Part`, `Staff`, `Voice`, `Measure`, `ScoreMark`, and the tempo map. None
of the six carries an `extra` field. `Rest` (15.3) carries one with no `#[serde(flatten)]`.

**CLAIM.** One type of eight implements the property. `Spanner` cannot implement it at all, because
a `BTreeMap` is not `Copy` and VR5 gives the type `Copy` at 32 bytes.

**ARGUMENT.** The property is the reason `duet-score` depends on `serde_json`, and ADR 0002
decision 4c builds the `ImportWarning` path on it. Without the bag, an unknown field on a spanner
line is a parse error and not a kept key, so the round trip that ADR 0002 promises fails for the
file the property names.

`Rest` fails differently and silently. Without `#[serde(flatten)]`, the writer emits a nested
`"extra": {}` key on every rest line, and the deterministic-output property of 3.6 then puts that
key in the git diff of every save. A reader that meets an unknown top-level key gets a serde error.

**EVIDENCE.** `architecture.md:1891` to `:1895`, `:1521` to `:1523`, `:1444` to `:1459` (`Part`,
`Staff`, `Measure`), `:6279` to `:6287` (`Rest`), `:6290` to `:6291` (`Voice`),
`adr/0002-storage-format.md:28` and `:49`.

**WHAT SHOULD CHANGE.** State which score types carry the bag and which do not, and give the reason
for each exclusion. If `Spanner` needs one, remove its `Copy` derive and move it to the VR5
expectation list. Add `#[serde(flatten)]` to `Rest`. Then rewrite ADR 0002 decision 2, which still
says every record carries one and which decision 3c in the same record contradicts.

### WARNING: W9. `ObjectId` collides with `gix::ObjectId` inside `duet-project`, and `duet.toml` is not the authority on the object format

**OBSERVATION.** Section 15.5 declares the type and its policy.

```
architecture.md:6474   pub struct ObjectId([u8; 20]);
```

Its doc comment reads: "an object identifier is the SHA-1 digest that `gix` writes by default.
`duet.toml` records the object format, and this build refuses a SHA-256 repository at open with
`ProjectError::ObjectFormat`" (`:6463` to `:6466`).

**CLAIM.** PL3 forbids the name, and `duet.toml` cannot answer the question the refusal asks.

**ARGUMENT.** PL3 reads "A name maps to one type in one crate", and its own example is
`MeterPoint` against `MeterTap`: "Two types with one name would collide inside `duet-engine`"
(`:237` to `:240`). `duet-project` depends on `gix` and on `duet-command`, and `gix::ObjectId` is
the type its `History` implementation returns. So the collision is in exactly the crate PL3 was
written for, and the document renamed a type once to avoid the same shape.

The object-format half is a correctness defect. The authority for a repository's hash function is
`extensions.objectFormat` in `.git/config`, which git and gix both read. `duet.toml` is a Duet file
that Duet writes at creation. Section 4.6 makes a user who runs plain `git` a supported path, and
`git` can convert a repository without touching `duet.toml`. The two then disagree, and the check
passes on a repository whose object identifiers are 32 bytes.

Section 4.5 also contradicts the field. "`duet.toml` holds a branch name and nothing else about
history" (`:2301`). Section 4.1 describes the file as "bundle identity and schema number". Neither
carries an object format, and section 4.7, which lists every repository setting written at
creation, does not set one.

**EVIDENCE.** `architecture.md:237` to `:240` (PL3), `:2063`, `:2301`, `:2330` to `:2349` (4.7),
`:6460` to `:6478`, `:6925` to `:6927` (`ProjectError::ObjectFormat`),
`adr/0003-history-model.md:49`.

**WHAT SHOULD CHANGE.** Rename the type, for example to `GitObjectId`, and state the reason beside
PL3's `MeterTap` precedent. Make the refusal read `extensions.objectFormat` from the repository
through the `History` trait, and add the method to the trait in section 4.5. Then state in 4.7 that
bundle creation writes the SHA-1 object format explicitly, so a new bundle never depends on the
default of the installed git.

### WARNING: W10. The conversion guard drops a cast that sits between two string literals, and no rule or probe covers the shape

**OBSERVATION.** CG2 reads: "It removes every line comment, every block comment, and **every
string literal** before it matches" (`architecture.md:1065`). `scrub` removes block comments
first, then line comments, then strings (`conversion_check.py:103` to `:109`).

**CLAIM.** A `/*` inside a string literal opens a comment the guard believes in, and every cast up
to the next `*/` disappears. A `//` inside a string does the same to the rest of that line.

**ARGUMENT.** The order is the defect. A block-comment regex that runs before the string pass
cannot know that its opening token is inside a literal. MINE-C1, in a throwaway cargo workspace:

```rust
pub fn f(x: u64) -> u32 { let s = "/*"; let n = x as u32; let t = "*/"; let _ = (s, t); n }
```

gives `MEMBERS: 1   FILES: 1   FINDINGS: 0` and exit 0. MINE-C2:

```rust
pub fn f(x: u64) -> u32 { let s = "// not a comment"; let n = x as u32; let _ = s; n }
```

gives the same. My control, the same file with the strings removed, gives `FINDINGS: 1`.

**The gate as a whole does not fail open, and I say so plainly.** The root `Cargo.toml` sets
`as_conversions = "deny"` and `scripts/dod.sh` runs clippy with `-D warnings`, so clippy refuses
the same cast. The defect is that CG2 claims a property the machine does not hold, and that
CG3b's own sentence, "An exclusion never hides a cast, and it prefers a false red to a miss", is
false for two shapes that no rule names. Revision 9 closed exactly this class twice, as CG3b and
CG4b, and left the third instance of it in the function both of them run after.

**EVIDENCE.** `architecture.md:1065` to `:1069` (CG2), `:1074` to `:1081` (CG3b),
`conversion_check.py:103` to `:109`, MINE-C1 and MINE-C2 above.

**WHAT SHOULD CHANGE.** Scan the file once, left to right, with one state machine over four states:
code, line comment, block comment, and string. Then give CG2 the "does not cover" sentence that
every other rule carries, and replace CP2 with a probe that plants a `/*` and a `//` inside a
string beside a real cast.

### WARNING: W11. DR6 is stated and not applied. Two ADR decisions state a rule the architecture replaced

**OBSERVATION.** DR6 reads: "A decision record cites a section; it never carries a second copy of
one" (`architecture.md:65`). Two decisions carry a second copy, and both copies are stale.

1. **ADR 0005 decision 16c**: "`ProjectOpen` opens with `ViewState::default` after a
   `CommandError::Schema`" (`adr/0005:74`). Architecture 10.2 names
   `ViewState::first_open()`, and ADR 0002 decision 3c already uses the new name.
2. **ADR 0004 decision 12b**: "the reset travels the other way as **an** `AtomicU32` counter that
   the audio thread reads once per cycle" (`adr/0004:157`). Architecture 7.3 replaced the single
   counter with `[AtomicU32; MAX_STRIPS]`, one generation per strip, which is the whole fix for R6.

**CLAIM.** DR6 closes a defect class by rule and the revision did not apply the rule. An
implementer who reads ADR 0004 12b writes the defect R6 named.

**ARGUMENT.** DR1 makes a revision rewrite the sentence it changes. Both sentences changed in the
architecture and neither changed in its record, which is the shape R12 and C12 named one revision
ago. The document even states the reason: "the ADRs went eight revisions with no guard and
collected nine statements the architecture had replaced".

DR6's stronger half is also unapplied. ADR 0003 decision 13 restates all five save steps. ADR 0005
decision 15 restates the `JobRunner` worker model. ADR 0004 decision 12b restates TH9. Each one is
a second copy that DR6 forbids, and each one is a place the next revision can go stale.

Both guards read one file, and that file is still the only one either guard reads.

**EVIDENCE.** `architecture.md:65` to `:69` (DR6), `:29` to `:30` (DR1), `:3772` to `:3779`,
`:4762`; `adr/0005-agent-gateway.md:73` to `:76`; `adr/0004-backend-and-threading-contract.md:151`
to `:159`; `adr/0003-history-model.md:46`.

**WHAT SHOULD CHANGE.** Rewrite the two stale decisions to cite section 10.2 and section 7.3 rather
than restate them. Then run one pass over all six records against DR6 and delete every restated
rule, because DR6 exists to make that pass unnecessary next time.

### WARNING: W12. The first-open window is smaller than the minimum window the design contract states

**OBSERVATION.** Section 1.6 of the architecture sets the first-open geometry.

```
architecture.md:578   | B91 | 960 logical pixels | The first-open window width | 10.2 |
architecture.md:579   | B92 | 640 logical pixels | The first-open window height | 10.2 |
architecture.md:581   | B94 | 260 logical pixels | The first-open sidebar width | 10.2 |
```

Design contract 1.1 states: "Minimum window size: 1024 x 700 px ... Default window size:
1440 x 900 px" (`design-contract.md:46` to `:48`).

**CLAIM.** Duet opens at a size its own normative design contract refuses, and the sidebar width
matches no breakpoint the contract defines.

**ARGUMENT.** B92 at 640 is 60 pixels below the contract's minimum height, and B91 at 960 is below
the contract's minimum width and below the 900-pixel floor at which the window "refuses to shrink
further". A first open therefore lands in a state the contract says cannot exist, and the narrow
breakpoint rules would apply from the first frame.

B94 at 260 matches nothing. Contract 1.1 gives three breakpoints with sidebar widths of 280, 240,
and a 48-pixel rail. Contract 1.6 gives the sidebar a preferred width of 240.

The paragraph that introduces these numbers also states a fact I checked and found false:
"`crates/duet/src/main.rs` already carries those numbers as literals in `main_window_options`"
(`architecture.md:4790`). The function carries 960, 640, and 120. It carries no 260 and no 0.62,
and a window-options function is not where a sidebar width could live.

**EVIDENCE.** `architecture.md:578` to `:582`, `:4787` to `:4792`, `design-contract.md:44` to
`:58`, `:113` to `:130`, `crates/duet/src/main.rs:35` to `:47`.

**WHAT SHOULD CHANGE.** Set B91 and B92 to the contract's default of 1440 by 900, and set B94 to a
value the contract's breakpoint table holds. State that the current `main.rs` literals are the
skeleton's and that K1 replaces them. Then correct the sentence that claims main.rs carries all
five numbers.

### WARNING: W13. C15 is not closed. `VirtualList` has no paragraph, no owner, and no chunk

**OBSERVATION.** Appendix C row C15 reads: "Virtualization has no type, owner, or chunk | 10.3, the
`VirtualList` paragraph; K2 and K3" (`architecture.md:7682`).

The token `VirtualList` appears twice in the whole document: inside the 1.3 framework-name block
that the guard reads, and in that Appendix C row. Section 10.3 holds no `VirtualList` paragraph.

ADR 0006 decision 9 reads: "**Virtualization is a `gpui_kit::VirtualList` over systems, owned by
the work-area view.** ... Specification section 10.3 names the owner and the chunk"
(`adr/0006:65`).

**CLAIM.** Two documents cite a paragraph in section 10.3 that does not exist. Section 10.7 asserts
the behaviour as a required test with no element behind it.

**ARGUMENT.** Section 10.7 rung three item 4 reads "Virtualization: the correct systems render at a
given scroll offset" (`:5092`). That is a required user-interface integration test. SM7 binds a
selected test to a chunk that writes it, and no Writes column in section 13.2 names a virtualization
file, a `VirtualList` use, or the width model ADR 0006 decision 9 says the audit requires.

The element list of section 10.3 holds twelve rows and none is a virtual list. The 10.3 paragraph
below it states "The linear timeline of Mix and Master is drawn by `AutomationLane` plus a ruler",
which is the only paragraph in the section about a scrolling surface.

Appendix C's own rule applies: a row that cites a section which does not hold the mechanism is a
defect.

**EVIDENCE.** `architecture.md:274` (the framework block), `:4882` to `:4906` (the 10.3 table and
its paragraphs), `:5092` (the required test), `:7682`, `adr/0006-wrapped-timeline.md:65`.

**WHAT SHOULD CHANGE.** Add a paragraph to 10.3 that names the owner of the `VirtualList`, the
width model it reads from `SizePx`, and the chunk that writes it. Add that file to the Writes
column of K2 and K3. Alternatively, delete decision 9 from ADR 0006 and state that version one
renders every visible system with no virtualization, and delete test 4 with it.

### WARNING: W14. C16 is not closed. `ZoomScalar` is an undeclared, unplaced type name in two documents

**OBSERVATION.** Appendix A gives the path cache two keys: "`(SourceHash, RegionId, SystemId,
ZoomStep)` in Compose and Record and `(SourceHash, RegionId, ZoomScalar)` in Mix and Master"
(`architecture.md:7163`). ADR 0006 decision 8 carries the same second key.

Section 10.5 states one key and claims agreement: "The path cache key is `(SourceHash, RegionId,
SystemId, ZoomStep)`, and **three documents now say so**. ADR 0006 decision 6, Appendix A, and
design contract 3.3 carry the same four parts" (`:5016`).

Appendix C row C16 cites "10.5, `ZoomScalar`". The token does not appear in section 10.5.

**CLAIM.** `ZoomScalar` is a type name that section 1.5 places nowhere and section 15 declares
nowhere. The 10.5 claim of three-document agreement is about one key while two of the three
documents now carry two.

**ARGUMENT.** PG4b now proves that the 1.5 table and the declarations are one set. That proof holds
only for names inside a fenced Rust block, and 1.5 states the limit. `ZoomScalar` sits in a table
cell and in an ADR, so the guard cannot see it, and the chunk that writes `PathCache`, which is K3,
has a key part with no type.

`ZoomStep(u8)` is declared and is the persisted zoom value for all four modes: section 3.6 states
that "`state/view.json` persists the `ZoomStep`". A second, undeclared scalar for Mix and Master
either duplicates it or contradicts it, and no section says which.

**EVIDENCE.** `architecture.md:4697` (`ZoomStep`), `:1916`, `:5016` to `:5020`, `:7163`, `:7683`,
`adr/0006-wrapped-timeline.md:64`, `design-contract.md:562` to `:566`.

**WHAT SHOULD CHANGE.** Use `ZoomStep` for both keys and delete `ZoomScalar` from Appendix A and
ADR 0006, or declare it in section 15.16 and place it in section 1.5. Then rewrite the 10.5
sentence so that it states how many keys exist before it counts the documents that agree.

### WARNING: W15. A resynchronized client receives no session and no mix state

**OBSERVATION.** Section 5.8 gives the resynchronize path its own channel. "A snapshot travels its
own channel at B30, one per client ... The core stops every event to a stale client until the
snapshot is taken" (`architecture.md:3130` to `:3132`).

```
architecture.md:6510   pub struct Snapshot { version: Version, score: Box<CanonicalDocument>, view: Box<ViewState>, engine: EngineState }
```

**CLAIM.** `Snapshot` carries the score, the view, and the engine state. It carries neither
`Session` nor `MixState`, so a client that dropped events never recovers its track list, its take
list, its region list, or its strips.

**ARGUMENT.** The whole reason for the path is a gap. Section 5.8 states that a client "that sees a
gap, or that the core marked stale, sends `CoreInput::Resynchronize`", and B29 makes the gap real:
on a full `CoreEvent` channel "the core marks the client stale". Every `SessionEvent` that fell in
that window is gone, and nothing replaces it.

The consequence is visible and wrong. The Record work area draws one lane per track, and the Mix
work area draws one strip per strip. Both read session state. After one resynchronize they draw the
state as it stood before the gap, with no marker and no fault, until the user restarts.

The mechanism to fix it already exists. `duet-command` implements `BundleDocument` for `Session`
and `MixState` (section 1.8), so both have a canonical byte form, exactly as `CanonicalDocument`
gives one for `Score`. The type omits them.

The declaration also does not build, which S4 covers: `CanonicalDocument` derives no `Serialize`.

**EVIDENCE.** `architecture.md:3122` (B29 and the stale rule), `:3128` to `:3137`, `:6510`,
`:696` to `:697` (the four implementations), `:4644` to `:4645` (the two work areas).

**WHAT SHOULD CHANGE.** Add a session block and a mix block to `Snapshot`, in the byte form that
`BundleDocument::to_bytes` already produces. State the size of a whole snapshot at B1 and give it a
B row, because B30 holds one per client.

### WARNING: W16. VR1's table is stale against section 15, and nine types derive `Eq`, `Hash`, or `Ord` with no stated use

**OBSERVATION.** VR1 reads: "`Eq`, `Hash`, and `Ord` appear only where this rule names the use"
(`architecture.md:1759`). I compared every declaration against the VR1 table and against row 1,
which covers every identifier newtype.

| Type | Crate | Derives | VR1 row |
|---|---|---|---|
| `Symbol` | `duet-engrave` | `Eq`, `Hash`, `Ord` | none |
| `ElementRef` | `duet-score` | `Eq`, `Hash`, `Ord` | none |
| `Revision` | `duet-score` | `Eq`, `Hash`, `Ord` | none |
| `RepeatSide` | `duet-score` | `Eq`, `Hash`, `Ord` | none |
| `Layer` | `duet-session` | `Eq`, `Hash`, `Ord` | none |
| `Version` | `duet-command` | `Eq`, `Hash`, `Ord` | none |
| `Velocity` | `duet-command` | `Eq`, `Ord` | none |
| `NodeIndex` | `duet-engine` | `Eq`, `Hash`, `Ord` | none |
| `MenuPath` | `crates/duet` | `Eq`, `Hash`, `Ord` | row 1, by the identifier sentence |

**CLAIM.** VR1's sentence is false for eight types. Revision 9 closed C5 by repairing three types
and then added eight more of the same class in section 15.

**ARGUMENT.** One of the eight has a real, stated use that the table should name: `FontMetrics`
holds `BTreeMap<Symbol, f32>` and `BTreeMap<Symbol, (f32, f32)>`, so `Symbol` needs `Ord`. The
other seven have no stated use anywhere in the document, and VR1's own reason for existing is that
a trait a type does not need points the demand upward and breaks the enum above it, which is what
made revision 4 fail to compile.

Nothing checks it. My derive closure pass reports zero failures for `Eq`, `Hash`, and `Ord`, so the
derives are consistent; they are simply unjustified. VR1 has no guard rule and no probe, unlike
VR5, which PG9 and PG10 hold.

**EVIDENCE.** `architecture.md:1759` to `:1784` (VR1), `:6657` (`Symbol`), `:6662`
(`FontMetrics`), `:6294` (`ElementRef`), `:6298` (`Revision`), `:6258` (`RepeatSide`), `:6388`
(`Layer`), `:6489` (`Version`), `:4129` (`Velocity`), `:6773` (`NodeIndex`), `:7008` (`MenuPath`).
MINE-P7: I added `Eq` and `Hash` to `Revision` and the guard exited 0.

**WHAT SHOULD CHANGE.** Add a row for `Symbol` that names the `FontMetrics` map. Remove `Eq`,
`Hash`, and `Ord` from the seven types that have no use, or add a row for each with the use. Then
give VR1 a guard rule in PG9's shape, with a probe that plants an unnamed derive.

---

## 3. Concerns

### CONCERN: C1. Line B's chunk ids collide with the budget ids B1, B2, and B3

Section 13.2 names the interchange line's chunks B1, B2, and B3. Section 1.6 names the product
budget B1, the frame budget B2, and the system paint budget B3. Appendix B.3 reads "| `quick-xml` |
... | B1 | M2 |" in a column headed "Chunk that needs it", and B.4 reads "chunk B3 needs `midly`".
Section 13.0 renamed the media line from M to N for the same reason (critic N20), and the rename
stopped one letter short.

### CONCERN: C2. The placement guard still carries a hardcoded candidate drop list that the document does not state

Section 1.9 states "There is no drop list" for PG18, and the guard holds that: the external table
decides every name a declaration reaches. A second, separate list of 52 names still filters the
candidate set before PG4 runs (`placement_check.py:44` to `:53`). It is not the same set as the 40
rows of the 1.9 table: it holds `ExitCode`, `Sync`, and `Sized`, and it omits `AtomicUsize` and
`AtomicI32`. No section states it and no probe covers it.

### CONCERN: C3. The conversion guard takes a root argument, which is an ungated scope seam

`main` reads `root = argv[1] if len(argv) == 2 else "."` (`conversion_check.py:223`). MINE-C5: I
built a workspace with a cast and a clean sub-workspace inside it. The guard at the real root gave
`FINDINGS: 1` and exit 1; the same guard at the injected root gave `FINDINGS: 0` and exit 0.
Chunk M0 ports the guard to `cargo xtask check-conversions`, and no section says whether the
subcommand accepts a path.

### CONCERN: C4. Section 12.1 says `EngineError` wraps three enums and section 15.10 declares five

12.1 reads "`EngineError` wraps the other three with `#[from]`" (`architecture.md:5266`). 15.10
declares `EngineError { Backend, Config, Transport, Media, Dsp }`. The two extra arms are correct,
because `duet-engine` carries both edges; the sentence is a revision behind.

### CONCERN: C5. `ParamSnapshot` derives `Copy` at about four kilobytes with no VR5 answer

`ParamSnapshot { values: [Finite; MAX_PARAMS], generation: Generation }` is 512 times eight bytes.
VR5 mandates `Copy` at 32 bytes or less and requires an expectation for "a type that must not be
`Copy`". It gives no answer for a large type that must be `Copy`, and TH9 does not bind this path
because the core is the writer. The row is not wrong; the rule states no reason for it.

### CONCERN: C6. `Rest` names `Value` and `Note` names `serde_json::Value` in one crate

Two spellings of one type sit twelve hundred lines apart in `duet-score`. PG12 proves the
`serde_json` dependency through `Note`, so the guard is quiet, and a reader of 15.3 cannot tell
which `Value` the field holds.

### CONCERN: C7. Phase 13 claims a width of one and lists no chunk

The 13.3 row for phase 13 reads "The acceptance run of section 14" with a width of 1, and the cell
holds no chunk id. Every other phase row lists the chunks that give it its width.

### CONCERN: C8. `mix_vertical_split` and `inspector_width` have no first-open value and no B row

`ModeView` holds both. Section 10.2's first-open paragraph names the sidebar and the body split and
neither of these. Design contract 1.6 states a preferred inspector width of 300 pixels and a Mix
split of 45 percent, and neither number is a B row.

---

## 4. Reactive assessment

- **Responsive: PASS.** 95 budgets and eleven timeouts are real, cited by id, and correct in every
  derived row except B90 (W1). B72 and B73 state a one-signed error that matches B82 exactly. The
  timeout table of B.5 names a mechanism and a crate feature for every bound. The `JobRunner`, the
  reserved slots, the `Arc` snapshot, and the one frame driver per work area keep every slow verb
  and every meter off the frame path.
- **Resilient: FAIL.** Four declaration sets do not compile, and one of them is the compile-time
  assertion the design built to catch its own class (S1, S2, S3, S4). The save path, the crash
  matrix, the four-source live set, the writer lock with its liveness check, the typed error rule,
  and the MIDI port-loss path are all correct and complete. The failure is that the guard family
  reads one derive and one prose form, and section 15 added 358 declarations outside both.
- **Elastic: PARTIAL.** Every channel, ring, queue, store, and cache carries a bound with an id.
  B86 now carries the strips the product builds. One bound is wrong in the silent direction: B90
  is below its own stated derivation and no error refuses the surplus (W1).
- **Message Driven: PARTIAL.** One vocabulary, one writer, versioned documents, published
  snapshots, and no shared mutable state across the user-interface seam. Three paths are wrong:
  `MidiPortMap` needs two owners again (W6), the meter reset has no storage and no delivery while
  the engine is stopped (W2), and a resynchronized client receives no session state (W15).

---

## 5. Plan graph check on section 13

1. **The graph is acyclic and every link runs forward.** I expanded every cross-line link of 13.4
   against the 13.3 phases by machine. No link runs backward.
2. **One same-phase link, and it is the stated exception.** M0 before T1.
3. **Every chunk has a phase and a row.** 55 chunks, 0 gaps, 0 phase mismatches.
4. **SM6 holds in every line.** No line runs two chunks in one phase.
5. **Write scopes are disjoint per phase.** The only shared names are `Cargo.lock`, `src/lib.rs`,
   and `tests/`, and SM5 rule 4 resolves the first.
6. **Every chunk has a runnable Completion command, and every filtered one carries
   `--no-tests=fail`.**
7. **SM7 holds.** PG16 passes, and all five selected tests resolve to a chunk in an earlier phase.
8. **The manifest seam holds.** Each member manifest has one writer per phase. M3 pins `criterion`
   before A4 in phase 5, and M7 owns `audio-smoke.yml` after C4 in phase 6.
9. **MUST story coverage is exact.** 64 of 64, and the two sets are identical.
10. **No line chunk writes a policy file.** M0, M6, and M7 carry them all, and all three go to the
    Orchestrator.
11. **SM0 matches the repository.** `crates/duet/src/` holds `main.rs` and `app.rs`, which is what
    K1 expects. `scripts/dod.sh` runs the ten steps rung one names and does not run
    `check-conversions`, which is what M0 adds.
12. **Two ordering defects.** K2 to K5 must write a file no chunk gives them (W5), and the
    `Toolbar` element lands four phases after the first chunk that builds on it (W5).

---

## 6. Consistency across the documents

1. **Closed.** Contract 3.1, ADR 0006 decision 2, and architecture 10.5 agree on
   `beat_for_x(x) -> Ticks` and `x_for_beat(beat) -> f32`.
2. **Open.** Contract 3.3 and architecture 10.5 carry one four-part path-cache key. Appendix A and
   ADR 0006 decision 8 carry a second key over an undeclared `ZoomScalar` (W14).
3. **Closed.** ADR 0003 and architecture 4.5 agree on the tracked set, the content store, the four
   live-set sources, the five save steps, the crash matrix, and the symbolic pointers.
4. **Closed.** ADR 0001 and architecture 2 agree on B40, B41, the two kernel newtypes, the absent
   order on `Position` and `Delta`, and the six named methods.
5. **Closed.** The platform research, section 5.4, and section 11 agree on cpal 0.18.2 with
   `default-features = false, features = ["pipewire"]`, on `HostId::PipeWire` and
   `HostId::CoreAudio`, on no ALSA fallback, and on `ubuntu-26.04` and `macos-26`.
6. **Closed.** Product requirements section 8 holds 64 MUST rows and 13.2 holds 64. The sets are
   identical, and every SHOULD story the paragraph below names is a SHOULD in the requirements.
7. **Closed.** `deny.toml` carries every licence B.4 calls present and does not carry `Unlicense`,
   which B.4 asks M0 to add for `midly`.
8. **Closed.** The root `Cargo.toml` sets `panic = "abort"`, `overflow-checks = true`, and
   `as_conversions = "deny"`, which is what sections 2.2 and 2.3 state.
9. **Closed.** Contract 4.4 and architecture 7.3 agree on the latch, the `Clear clip` command, and
   the numeric peak readout since the last reset. `ClearClip` is a real action.
10. **Closed.** The `StageCurve` appendix names `duet.meter.peak_cap`, which contract 7.2 holds.
11. **Open.** ADR 0005 16c names `ViewState::default` and ADR 0004 12b names one reset counter
    (W11). ADR 0002 decision 2 contradicts decision 3c in the same record (W8). ADR 0006 decision 9
    cites a section 10.3 paragraph that does not exist (W13).
12. **Open.** Design contract 1.1 states a minimum window of 1024 by 700 and architecture B91 and
    B92 open at 960 by 640 (W12).

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

Revision 9 is the strongest revision of the nine on the axis it set out to fix. The deletion of ID1
is right, and the mechanism that replaces it works: section 15 declares all 358 placed names, PG4b
proves the two sets are identical on every run, and the baseline prints `UNDECLARED: 0`. I ran all
thirty recorded probes and every exit code and every message reproduced, including the four new
rules. Twenty of the twenty-eight revision-8 findings close by mechanism, and I proved each closure
with a planted defect rather than with the text. The plan graph passes nine mechanical tests.

Four Criticals block it, and they share one cause. **The guard family reads one derive and one
prose form, and revision 9 moved 186 undeclared names into 358 full declarations inside that blind
spot.** PG9, PG10, PG10b, PG13, and PG14 all read `Copy`. PG11 reads a prose sentence, never a
declaration. So ten `Default` derives over a type that has none, three `Serialize` derives over
types that have none, three lint expectations that a compiler run proves are build errors, and
sixteen field references across crate edges that section 1.3 forbids all pass in silence. Five of
those sixteen are Cargo cycles, and one of them, `duet-score` naming `InverseCost` in `duet-core`,
is blocker V1 reopened at the base of the design.

The single biggest risk is the shape of the fix, not the count of the findings. **Revision 9 closed
a rule that decided a shape by replacing it with 358 declarations, and it added no rule that reads
those declarations.** The guard grew two rules, PG4b and PG18, and both ask whether a name is
declared. Neither asks whether the declaration can be written. The next step is one rule per derive
family and one rule that reads every field type against the section 1.3 edge list, each with its
own probe. Both are cheap: my own passes over the whole document ran in under a second and found
every one of the four Criticals.

Sixteen Warnings sit under the Criticals, in four shapes. Four are an arithmetic or a mechanism
that the R5 and R6 fixes left behind: B90 is false at the new B86 with no refusal at all, the
per-strip reset has no storage, `from_finite_const` cannot be a `const fn`, and `ModeView` has no
first-open constructor. Four are a closure claim that its own cited section does not hold: R9, C15,
C16, and the VR1 table. Four are a machine or a list narrower than its sentence: CG2 drops a cast
between two strings, B.1 lists three of eight suppressions, and VR1 and property 4 of section 3.6
are each false for most of the types they name. Four are cross-document drift: two stale ADR
decisions, a window smaller than the contract's minimum, an `ObjectId` that collides with the gix
type of the same name, and a snapshot that omits half the project.

The weakest Reactive property is Resilient, and this time it fails rather than partly holds. Four
sets of declarations that this document calls "the declaration of record" cannot be compiled, and
one of the four breaks the compile-time assertion the design built to catch its own class.

### Findings that block plan authoring

1. S1 (Critical). Ten declarations derive `Default` over `Finite` and `FrameCount`, which have none.
2. S2 (Critical). Sixteen field references cross an absent crate edge, and five make a Cargo cycle.
3. S3 (Critical). Three `missing_copy_implementations` expectations in a binary crate are build errors.
4. S4 (Critical). Three `Serialize` derives reach a type with none, and the VR2 assertion fails.
5. W1 (Warning). B90 is false at B86 of 48, and `MAX_PARAMS` carries no refusal.
6. W2 (Warning). The per-strip reset has no stored last-seen value and no path while the engine stops.
7. W3 (Warning). `from_finite_const` cannot be a `const fn`, and B91 to B95 are not constants.
8. W4 (Warning). `first_open` leaves `per_mode` empty and `ModeView` has no constructor.
9. W5 (Warning). R9 is open: `ModeToolbar` has no declaration and no chunk owns the per-mode toolbar.
10. W6 (Warning). The presence listener must mint a `MidiPortInfo` that only the MIDI thread may mint.
11. W7 (Warning). Appendix B.1 lists three suppression sites and the document holds eight.
12. W8 (Warning). Property 4 of 3.6 requires an `extra` bag that `Spanner` cannot hold.
13. W9 (Warning). `ObjectId` collides with `gix::ObjectId`, and `duet.toml` is not the format authority.
14. W10 (Warning). The conversion guard drops a cast between two strings and after a `//` in a string.
15. W11 (Warning). ADR 0005 16c and ADR 0004 12b state a rule the architecture replaced.
16. W12 (Warning). The first-open window is below the design contract's stated minimum.
17. W13 (Warning). C15 is open: `VirtualList` has no paragraph, no owner, and no chunk.
18. W14 (Warning). C16 is open: `ZoomScalar` is undeclared and unplaced.
19. W15 (Warning). `Snapshot` carries no session and no mix state.
20. W16 (Warning). VR1's table is stale: eight types derive `Eq`, `Hash`, or `Ord` with no use.

### Concerns, each with a disposition for the orchestrator

| Id | Concern | Disposition |
|---|---|---|
| C1 | Chunk ids B1, B2, B3 collide with budget ids B1, B2, B3 | Rename the interchange line to X or to another free letter, as line M became line N |
| C2 | The guard carries an unstated candidate drop list of 52 names | State the list beside PG4, or build the candidate filter from the 1.9 external table |
| C3 | The conversion guard takes an injectable root | State whether `cargo xtask check-conversions` accepts a path; if it does, add a probe that plants a decoy workspace |
| C4 | 12.1 says `EngineError` wraps three enums and 15.10 declares five | Change "the other three" to name the five arms |
| C5 | `ParamSnapshot` derives `Copy` at about four kilobytes | Add a VR5 sentence for a large type that must be `Copy`, and name this site |
| C6 | `Rest` names `Value` and `Note` names `serde_json::Value` | Use one spelling in `duet-score` |
| C7 | Phase 13 claims a width of one and lists no chunk | Set the width to zero, or name the chunk that runs the acceptance |
| C8 | `mix_vertical_split` and `inspector_width` have no first-open value | Add two B rows from design contract 1.6 and name them in the 10.2 paragraph |
