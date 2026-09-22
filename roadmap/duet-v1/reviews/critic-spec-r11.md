# Specification review, revision 11: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-21. Mode: specification review, before plan authoring.
This is the eleventh pass. Revisions 1 to 10 each returned NOT READY.

## What I read and what I ran

Read in full: `roadmap/duet-v1/architecture.md` (9180 lines), the six ADRs,
`research/linux-macos-platform.md`, `product-requirements.md` section 8, `design-contract.md`
sections 1, 1.6, 3.1, 3.3, 4.4 and the `StageCurve` appendix, `CLAUDE.md`, `Cargo.toml`,
`.cargo/config.toml`, `clippy.toml`, `deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/`. I ran all three guards on the real inputs. All three reproduce the recorded
baseline, line for line. The roster compile runs clean at exit 0 in 1 minute 51 seconds.

I ran all 38 recorded probes. I added 9 probes to the placement guard, 14 to the conversion guard,
and 12 to the roster compile. I ran one compiler experiment against the roster with the six-lint
profile removed. I ran two compiler experiments outside the roster. I wrote no repository file and
I ran no other git command.

**Revision 11 is the strongest revision so far.** The roster compile is a real oracle. It found
four defects during the revision that no review had reached. 288 `Eq` derives now build clean under
the real lint table on the pinned toolchain, and the 11 declarations that keep `PartialEq` alone all
carry a float. The `Eq` class of C-1 is gone, and a compiler proved it.

It does not close. **Four Criticals block it.** Two are design faults the new guards cannot see,
because the guard substitutes an empty body for 44 declarations. One is a Reactive violation that
the C-2 fix introduced. One is a false recorded probe result in the table that DR5 calls the
contract of both guards.

---

## 1. Closure check on C-1 to C-4, W-1 to W-9, and K-1 to K-14

| Id | Finding | State | Section and reason |
|---|---|---|---|
| C-1 | 81 declarations trip `derive_partial_eq_without_eq` | **CLOSED** | 3.5 VR1 makes `Eq` follow the fields. The roster compile builds 288 `Eq` derives clean. The 11 types that keep `PartialEq` alone each hold an `f32` or an `f64`. |
| C-2 | `SlotState` does not build | **PARTIAL** | 5.5 boxes the equalizer arm and PG24 prints `VARIANT SPREAD BAD: 0`. The box puts an allocation and a free on the audio thread. See CR-2. |
| C-3 | Bundle creation writes a version-1-only git extension | **CLOSED** | 4.7 writes no `extensions.objectFormat` key. 4.5 reads `core.repositoryFormatVersion` first and answers `HashKind::Sha1` at version 0. |
| C-4 | Three of six stated type sizes are wrong | **PARTIAL** | The section 1.9 size block is guarded, and my wrong-row probe reds it. `MeterSnapshot` still states 1592 bytes and measures 1600. A false size inside an `#[expect]` reason passes all three guards. See WR-2. |
| W-1 | The stale snapshot restores every cleared over mark | **OPEN, inverted** | 7.3 now adds a mark only when the generation moves. The generation moves only on a reset, so no mark ever appears. See CR-1. |
| W-2 | Nothing wires `TopBar` to the four toolbar files | **PARTIAL** | 15.16 declares the trait and the registry. `ModeToolbar::render` returns a `Toolbar` that holds one `Mode` and nothing else. See WR-4. |
| W-3 | A raw byte string hides a real cast | **CLOSED** | CG2 reads all six prefixes. CP2b's five shapes each exit 1 with the recorded line. |
| W-4 | PG20 is blind to a trait-object field | **CLOSED** | 1.5 PG20 reads a `dyn` and an `impl` argument and names the four forms it declines. PP20 exits 1. |
| W-5 | The first-open geometry is the contract minimum | **CLOSED** | B91 and B92 are 1440 by 900, which contract 1.1 line 47 calls the default. B102, B103, and B104 carry the minimum and the floor. |
| W-6 | `LogicalPx` has no accessor | **CLOSED** | 10.2 declares `get` and `to_f32`. B.1 carries the `finite_to_f32_saturating` row and the count is 18. |
| W-7 | `LevelMeter` and `MeterLayer` both hold the clip state | **PARTIAL** | `LevelMeter` holds neither value now. It is still declared to paint the numeric peak readout, which is a live value. See WR-5. |
| W-8 | ADR 0004 13c contradicts section 8.1 | **CLOSED** | I checked all six records against the current text. Zero contradictions. The DR6 duplication class is a separate Concern. |
| W-9 | `from_finite_const` adds a panic primitive | **CLOSED** | 12.3 house form 5 states the condition. 2.6a carries the `# Panics` section. |
| K-1 | Section 15 declares no documentation | **CLOSED** | The section 15 preamble states the rule and names the two relaxed lints. |
| K-2 | `missing_const_for_fn` refuses the declared accessors | **PARTIAL** | The preamble tells the chunk to add `const` and `#[must_use]`. The PG25 profile relaxes four lints with a reason that is false. See WR-3. |
| K-3 | `ResetGenerations::new` trips `new_without_default` | **CLOSED** | 5.6 carries the hand `impl Default`, and the roster compiles it. |
| K-4 | A hidden table of 17 primitives | **CLOSED** | Both tables are section 1.9 blocks. PP4 shapes 3 and 4 each exit 1. |
| K-5 | Three wrong rows in the 1.9 `Traits` column | **CLOSED** | `Value`, `Ordering`, `Path`, `Cow`, and `Box` each carry a correct verdict. I proved `serde_json::Value` supplies `Eq` and `Hash` with a compiler run. |
| K-6 | PG19 reds a reference field | **CLOSED** | 1.5 PG19 states the class and the preference. |
| K-7 | PG19 passes a `Default` derive over a long array | **CLOSED** | 1.5 PG19 names the compiler as the backstop. |
| K-8 | CG5 reds a generic left side | **CLOSED** | 2.3 states it beside CG3b. |
| K-9 | A file that is not UTF-8 raises a traceback | **CLOSED** | CP8 shape 2 exits 2 with the stated line. |
| K-10 | A crate outside the members list is unscanned | **CLOSED** | 2.3 states the limit beside CG1. A second, unstated denominator hole exists. See WR-8. |
| K-11 | `ModeToolbar` is a twelfth element | **CLOSED** | 10.3 lists eleven elements plus `SystemXMap`. 15.16 makes `ModeToolbar` a trait. |
| K-12 | The cut point is fixed | **CLOSED** | `render` takes a `visible` count. The return type discards it. See WR-4. |
| K-13 | ADR 0006 gives Compose a path cache | **CLOSED** | ADR 0006 decision 6 and Appendix A both give Compose none. |
| K-14 | `?` does not compose to `GatewayError` | **CLOSED** | 12.1 states that `?` composes down the graph and never across it. |

**Count: 22 CLOSED, 4 PARTIAL, 1 OPEN.** Every PARTIAL and the one OPEN item names a new finding.

### 1.1 The baseline run of all three guards

The placement guard and the conversion guard each reproduce section 1.9 line for line. The roster
compile prints the recorded block:

```
ROSTER CRATES:   17
ROSTER ITEMS:    362
ROSTER IMPLS:    11
ROSTER CONSTS:   6
ROSTER LINT:     cargo clippy --workspace --all-targets -- -D warnings
ROSTER CLIPPY:   clean
ROSTER SIZES:    353 measured
RECORDED ROWS:   35 checked     HEAD ROWS: 12     SIZE BAD: 0
EXIT=0
```

The denominator holds. The 1.5 table places 372 names. The roster carries 362 structs and enums.
The 10 names it omits are exactly the 10 traits: `AudioBackend`, `AudioProcess`, `BundleDocument`,
`Gateway`, `GraphRunner`, `History`, `MidiPresence`, `MidiStream`, `ModeToolbar`, and
`SampleSource`.

### 1.2 The 38 recorded probes

36 of 38 reproduce the recorded exit code and the recorded message. **Two do not.** Both belong to
the placement guard, and both are stale against revision 11's own edits. See CR-3.

### 1.3 My own probes

I added 9 placement probes, 14 conversion probes, and 12 roster probes. 13 found a miss.

| Probe | Shape | Result | Caught |
|---|---|---|---|
| R1 | `Score` gains `Default`, `Hash`, `PartialOrd`, `Ord`, and serde over a private body | exit 0 | **No** (WR-1) |
| R2 | `MidiSink` gains `Default`, `Clone`, `PartialEq`, `Eq`, `Hash`, and serde over a private body | exit 0 | **No** (WR-1) |
| R3 | `#[allow(dead_code)]` on `Accidental` | exit 1 | Yes, two lint errors |
| R4 | An `f64` field inside `Eq`-deriving `KeySignature` | exit 1 | Yes |
| R5 | Every ` ```rust ` fence renamed | exit 1 | Yes, by the size oracle |
| R6 | The `Option<Span>` size row changed to 40 | exit 1 | Yes, `SIZE BAD: 1` |
| R7 | The `duet-analysis -> duet-session` edge removed | exit 1 | Yes, at `cargo` |
| R8 | `Ratio` given a generic parameter | exit 1 | Yes, loud; the head loses the parameter |
| R9 | A `Copy` derive over `DeviceLabel(Box<str>)` | exit 1 | Yes |
| R10 | "104 bytes" changed to "9000 bytes" in the `Transport` reason and in B.1 | exit 0 | **No** (WR-2) |
| R11 | A `BundleDocument` method that returns `duet_engine::MeterSnapshot` | exit 0 | **No**; no rule reads a trait |
| R12 | `impl Drop for Finite` with a `panic!` | exit 1 | Yes |
| XP4 | The `Value 72 8` size row changed to `Value 720 8` | exit 0 | **No** in placement; R6 shows PG25 catches it |
| XP4c | The `Option<PoolSlot> 4 2` row deleted | exit 0 | **No**; coverage falls by 4 types in silence |
| XP5b | A declared `ExitCode` struct with no 1.5 row | exit 0 | **No**; the drop list hides it |
| YC4 | A crate outside the `members` list | exit 0 | **No**; stated limit K-10 |
| YC5b | `#[expect(` and `clippy::as_conversions` on the next line | exit 0 | **No** (concern 1) |
| YC5a, YC5c, YC5d | One space, a `cfg_attr` wrapper, and the `allow` form | exit 0 each | **No** (concern 1) |
| YC7 | A cast in `m/src/target/mod.rs` | exit 0 | **No** (WR-8) |
| YC12 | The exempt path made a symbolic link to another member's live file | exit 0 | **No** (concern 2) |
| YC10 | A cast in `m/src/Other.RS` | exit 0 | **No**; low severity |
| T3a | The 1.2 crate-table heading renamed | exit 0 | **No**; `DEP ROWS: 0` (WR-10) |
| T3b | The 1.9 size-block heading renamed | exit 0 | **No**; coverage falls 315 to 202 |

The placement guard has no environment seam. `ROSTER_SIZES` only adds output lines. The roster
compile's `ROSTER_TARGET_DIR` only moves the build cache. Neither narrows a scanned set.

### 1.4 The mechanical passes

1. **Budgets.** 104 rows, B1 to B104, no gap and no duplicate. Every `B<number>` citation resolves.
   Five budgets are orphans: B42, B49, B50, B84, and B93. Fifteen "Used by" cells name a section
   that never cites the id.
2. **Rule ids.** Every family is complete: DR1 to DR6, PL1 to PL3, VR1 to VR6, TH1 to TH10, SM0 to
   SM7, 27 PG rules against 27 PP probes, and 11 CG rules against 11 CP probes. TH7 is stated
   twice, at line 3889 and line 5349, which DR4 forbids.
3. **Chunk coverage.** 55 chunks. Every chunk sits in exactly one phase. Every width cell matches.
4. **The plan graph.** 13.4 expands to 49 ordered pairs. No link runs backward. One same-phase
   pair, M0 before T1, which SM1 states.
5. **SM6.** No line runs two chunks in one phase.
6. **Write scopes.** No two line chunks in one phase write one path. `Cargo.lock` is the one shared
   name, and SM5 governs it.
7. **MUST stories.** 64 MUST rows in the requirements and 64 rows in 13.2. The two sets are equal.
   The two LATER stories appear nowhere in the architecture.
8. **ADR citations.** Every section citation and every B id resolves. Zero contradictions.
9. **Design contract.** Every checked fact matches, except two citations. See concerns 12 and 13.

---

## 2. New findings in revision 11

### CRITICAL: CR-1. The generation compare makes the `CLIP` indicator unreachable

**OBSERVATION.** Section 7.3 states the rule that closes W-1:

```
architecture.md:4619  **The generation compare is the rule, and it is stated once here.** Each frame `MeterLayer` reads
architecture.md:4620  the snapshot. **It adds a mark only when `snapshot.generation` differs from `seen`**; it then stores
architecture.md:4621  `snapshot.generation` in `seen`.
```

`Generation` is a publication counter (`architecture.md:3777`). `MeterSnapshot::generation` is
"the meter-reset generation the audio thread had applied when it wrote this value"
(`architecture.md:4064`). The value moves only when `Verb::MeterReset` increments an entry of
`ResetGenerations`.

**CLAIM.** No over mark ever reaches the user. The `CLIP` indicator never lights.

**ARGUMENT.** Trace the steady state. The audio thread publishes generation G on every cycle.
`seen` holds G. A singer clips. The audio thread sets `OverMark` on that strip. The generation does
not move, because no reset happened. The next frame reads generation G, finds `G == seen`, and adds
nothing. Every later frame does the same. The mark stays inside the snapshot and never enters
`shown`, which is the value `MeterLayer` paints (Appendix A, `architecture.md:8519`).

The one frame that adds anything is the frame after a reset, when the marks are empty by design.
Before the first reset both values are the initial generation, so the view adds nothing from the
first frame onward.

A second fault sits under the first. `ResetGenerations` holds `[AtomicU32; MAX_STRIPS]`, which is
48 independent counters (`architecture.md:3803`). `MeterSnapshot` holds **one** `Generation` field
(`architecture.md:4066`). `Verb::MeterReset { strip: Some(id) }` increments one entry. One scalar
cannot carry 48 independent counters, so a per-strip reset has no value to publish.

Design contract 4.4 states the requirement plainly: "**Clip** paints the top 6 px of the meter in
`duet.meter.clip` and writes `CLIP` in 8 px caps beside the meter. The state holds until the user
clicks the meter or presses the `Clear clip` command" (`design-contract.md:764`). PR M-05 makes it
a MUST.

Test 9 of section 10.7 cannot find this. It "asserts that no `CLIP` returns on any strip"
(`architecture.md:6103`). That is the one direction the broken rule satisfies. No test asserts that
a mark appears at all. A regression test with one oracle direction passes the inverted fix.

**EVIDENCE.** `architecture.md:4619` to `:4626` (the rule), `:3777` to `:3780` (`Generation`),
`:3803` (`ResetGenerations`), `:4055` to `:4070` (`MeterSnapshot`), `:8519` (Appendix A),
`:6100` to `:6104` (test 9); `design-contract.md:764` to `:767`.

**WHAT SHOULD CHANGE.** Invert the compare. `MeterLayer` holds the generation of the last reset the
core told it about, not the generation of the last snapshot it read. It adds every mark from a
snapshot whose generation is at or above that value, and it ignores a snapshot below it. Then give
`MeterSnapshot` a per-strip generation array, or narrow `ResetGenerations` to one counter and state
that a per-strip reset is not in version one. Then add a second test that asserts a mark **appears**
after a synthetic clip.

### CRITICAL: CR-2. The boxed `SlotState::Equalizer` arm allocates and frees on the audio thread

**OBSERVATION.** The C-2 fix boxes the equalizer arm (`architecture.md:3574`). Section 5.5 then
states how chain state migrates:

```
architecture.md:3690  The audio thread owns every `ChainState`. When a new `ChainTopology` arrives, the audio thread
architecture.md:3700  reconciles its own state in place, inside the cycle, with no allocation.
architecture.md:3694  2. A slot whose `SlotKind` changed resets to the default state for the new kind.
architecture.md:3696  3. A slot that the new topology drops has its state cleared in place.
```

**CLAIM.** Rule 2 allocates on the audio thread. Rule 3 frees on it. TH1 forbids both.

**ARGUMENT.** TH1 reads: "The audio thread never allocates, never frees, never locks, never blocks,
never formats a string, never logs, and has no panic path" (`architecture.md:3880`). ADR 0004
decision 8c repeats the claim: "**State migrates in place, inside the cycle, with no allocation.**"

Rule 2 builds the default state for the new kind. When the new kind is `Equalizer`, the value is
`SlotState::Equalizer(Box::new(EqualizerState::default()))`. That is one heap allocation, inside
`GraphRunner::run`, on the audio thread.

Rule 3 clears a dropped slot in place. When the old variant is `Equalizer(Box<EqualizerState>)`,
the clear drops the box. That is one `free`, on the audio thread. An `ArrayVec` that shrinks does
the same for every dropped element.

Appendix B.1 confirms where the code runs. It predicts a complexity suppression at
`duet-engine::chain::migrate::migrate_state`, with the reason "the four migration cases of section
5.5 form one match over two generations" (`architecture.md:8570`). Section 5.6 confirms the caller:
"At the top of every cycle the runner compares both generations. A topology change runs the
in-place migration of section 5.5" (`architecture.md:3848`).

Section 5.5's own sentence is self-contradictory: "The box is read once per migration and never on
the audio path, because `GraphState` owns one `SlotState` per slot for the life of the chain"
(`architecture.md:3566`). Migration **is** the audio path, and rules 2 and 3 replace the variant,
so the life of the chain is not the life of the box.

The engine's stated crate invariant is "The audio thread never allocates, frees, locks, or blocks"
(`architecture.md:131`). Revision 11 traded a build error for that invariant, and no guard reads a
thread rule.

**EVIDENCE.** `architecture.md:3568` to `:3584` (`SlotState`), `:3690` to `:3699` (the four
migration rules), `:3846` (the runner), `:3880` (TH1), `:131` (the crate invariant),
`:8570` (B.1); `adr/0004-backend-and-threading-contract.md` decision 8c.

**WHAT SHOULD CHANGE.** Take the equalizer state out of the enum. The `BufferPool` pattern already
exists for this: `configure` builds every equalizer state off the audio thread and hands the audio
thread a `PoolSlot` handle, exactly as it does for a delay buffer and a reverb buffer. Then
`SlotState::Equalizer` carries a handle, the enum needs no box, and the arm spread stays inside the
lint. State the new measured size at B51 and re-derive B55 and B56.

### CRITICAL: CR-3. Two recorded probe results are false, so PG21 and PG22 are unverified rules

**OBSERVATION.** DR5 reads: "A probe is correct when the baseline run is green and the planted run
is red" (`architecture.md:953`). Section 1.9 records a red result for every probe. I planted PP22
exactly as the cell writes it.

```
$ python3 placement_check.py <copy with the `Revision` name changed in its VR1 row>
VR1 ROWS:        68     VR1 BAD: 0
EQ MISSING:      0     EQ UNDECIDED: 0
exit 0
```

The recorded cell says `exit 1`; `VR1: duet-score::Revision derives Eq with no VR1 row`
(`architecture.md:992`).

**CLAIM.** PG22 has no probe that turns it red. PG21's probe cannot be planted at all. Both rules
are unverified, and two rows of the table that DR5 calls the contract of both guards are false.

**ARGUMENT.** `placement_check.py:143` reads `VR1_TRAITS = ("Hash", "Ord")`. Revision 11 narrowed
PG22 to those two traits and said so at the rule site (`architecture.md:564`). `Revision` derives
`PartialEq` and `Eq` and nothing else, so the rule never looks at it. The recorded message names
`Eq`, which the rule no longer reads.

PG22 still works. I planted a correct probe, a rename of the `NoteId` VR1 row, and the run exits 1
with `VR1: duet-score::NoteId derives Hash, Ord with no VR1 row`. The rule is alive and its
recorded probe is dead.

PP21 is worse. The cell says "The `ConfigError` name changed in its B.1 row"
(`architecture.md:991`). Appendix B.1 holds three rows, and none names `ConfigError`; C-4's own
disposition removed it (`architecture.md:8584`). No declaration carries a `ConfigError` expectation
either. The planted defect does not exist, and the recorded message needs a declaration the
document no longer holds.

This is the class revision 11 exists to remove. The tenth review found that a recorded number was
not a fact. Revision 11 answers with a compiler, and then records two probe results that its own
code does not produce. Chunk M0 ports each probe to one test in `tools/xtask`. An implementer who
writes the PP22 test from this table writes a test that fails, or writes one that asserts exit 0 and
locks the vacuous state into the gate.

**EVIDENCE.** `architecture.md:953` (DR5), `:564` (PG22's narrowed scope), `:991` and `:992` (the
two cells), `:8574` to `:8580` (Appendix B.1), `tools/placement_check.py:143`; the two runs above.

**WHAT SHOULD CHANGE.** Rewrite PP22 to plant a `Hash` or an `Ord` row. Rewrite PP21 to plant a
row against one of the three declarations B.1 holds. Then re-run every one of the 38 probes against
this revision and paste the real output, because two stale rows in one table means the table was not
re-run after the revision-11 edits.

### CRITICAL: CR-4. The audio-to-user-interface path names a crate and a dependency that `crates/duet` does not carry

**OBSERVATION.** Section 5.8 puts the two high-rate paths from the audio thread to the user
interface behind `triple_buffer::Output<TransportSnapshot>` and
`triple_buffer::Output<MeterSnapshot>` (`architecture.md:3951`). Section 10.2 states that
`MeterLayer` "paints every meter from one `MeterSnapshot` read" (`architecture.md:5532`). Section
7.3 gives `MeterLayer` a field `seen: Generation` (`architecture.md:4617`).

Section 1.5 places `MeterSnapshot`, `TransportSnapshot`, and `Generation` in `duet-engine`.
Section 1.3 gives `crates/duet` these edges, and no other:

```
duet   -> duet-time, duet-dsp, duet-score, duet-session, duet-command,
          duet-engrave, duet-analysis, duet-core, duet-agent
```

Section 1.2 gives the `duet` row these third-party dependencies: `gpui-kit`, `clap`, `tokio`,
`tracing`, and `tracing-subscriber`.

**CLAIM.** `crates/duet` cannot name `MeterSnapshot`, `TransportSnapshot`, or `Generation`, and it
cannot hold a `triple_buffer::Output`. Chunks K3, K4, and K5 do not build.

**ARGUMENT.** A Rust crate names a type through a direct dependency or through a re-export. Section
1.3 has no `duet -> duet-engine` edge. The document states every re-export it intends: "`duet-score`
re-exports `Tempo`, `Meter`, and `NoteValue` from `duet-time` under their own names"
(`architecture.md:260`). No sentence gives `duet-core` a re-export of the three engine types.

No `CoreEvent` carries a snapshot either. I read the whole enum (`architecture.md:7786` to `:7806`).
Section 5.8 rules the event path out on purpose: "No meter value travels as an event"
(`architecture.md:4640`).

The third-party half is independent of the re-export question. `Output<T>` is a `triple_buffer`
type. Whoever holds it depends on `triple_buffer`. The `duet` row of section 1.2 does not list it,
and `cargo machete` and the compiler both read that row.

No guard sees this. PG20 reads a field type against the section 1.3 graph, and `MeterLayer`,
`PlayheadLayer`, and `MasterMeterLayer` each carry `/* private */`, so the guard has no field to
read. That is the same blind spot as WR-1.

**EVIDENCE.** `architecture.md:216` to `:217` (the edge list), `:137` (the 1.2 row), `:349` (the
engine placement), `:3951` to `:3952` (5.8), `:4617` (`seen: Generation`), `:5533` (10.2),
`:8516` (Appendix A), `:7786` to `:7806` (`CoreEvent`).

**WHAT SHOULD CHANGE.** Pick one answer and state it once. Either add `duet-engine` and
`triple_buffer` to the `duet` row of section 1.2 and the edge to section 1.3, or declare the two
snapshot types and `Generation` in `duet-command`, which every side already reaches. The second
answer keeps section 1.3 rule 6 intact and costs nothing, because both types are plain `Copy` data
under TH9.

---

### WARNING: WR-1. The roster substitutes an empty body for 44 declarations, so four rules pass vacuously there

**OBSERVATION.** The PG25 site states substitution 2: "A body this document writes `/* private */`
becomes one public field of type `Box<()>`, because an empty body is fieldless and `Copy`, which is
an artifact of the roster and not of the design" (`architecture.md:616`).

44 declarations carry a private body. 25 of them are in `crates/duet`.

**CLAIM.** The stated reason names one artifact and hides eight. `Box<()>` supplies `Default`,
`PartialEq`, `Eq`, `Hash`, `PartialOrd`, `Ord`, `Serialize`, and `Deserialize` unconditionally. Every
derive over a private body is therefore satisfied by the substitute and not by the design.

**ARGUMENT.** I planted two shapes and ran the roster compile on each one.

```
R1: `Score` derives Default, Hash, PartialOrd, Ord, Serialize, Deserialize -> ROSTER CLIPPY: clean, exit 0
R2: `MidiSink` derives Default, Clone, PartialEq, Eq, Hash, Serialize, Deserialize -> ROSTER CLIPPY: clean, exit 0
```

Neither shape is buildable in the real crate. `MidiSink` holds a platform callback handle.

Three private-bodied types carry a real closure derive today: `Score`, `Session`, and `MixState`
each derive `PartialEq` and `Eq` (`architecture.md:7425`, `:7536`, `:7540`). No guard proves that
their fields supply either trait. PG19 and PG23 both state that they skip a private body
(`architecture.md:526`, `:574`). PG24 skips it too. PG25 answers with `Box<()>`.

CR-4 is one live instance of the same blind spot, in the reachability rule rather than the derive
rule.

**EVIDENCE.** `architecture.md:616` (the stated substitution), `:526` and `:574` (the two stated
skips), the 44 sites at `:3250`, `:3600`, `:4119`, `:4830`, `:5396`, `:5410`, `:5996`, `:7426`,
`:7537`, `:7541`, and 34 more; my two runs above.

**WHAT SHOULD CHANGE.** State the whole class at the PG25 site: the substitute supplies eight
closure traits and takes every private-bodied declaration out of PG19, PG23, PG24, and PG25. Then
close the gap for the three that matter. Give `Score`, `Session`, and `MixState` a field list in
section 15, or drop the derives and state why the aggregate needs no comparison.

### WARNING: WR-2. A false size inside an `#[expect]` reason passes all three guards

**OBSERVATION.** VR5's table heads one column "Measured size" and states: "**Every size in this
section is a measured value, and no size here is hand written**" (`architecture.md:2473`).
Appendix B.1 states: "**Every size in this appendix is a value PG24 computed and PG25 measured.** A
reason that states a number no guard printed is a plan defect" (`architecture.md:8586`).

I changed "the type is 104 bytes" to "the type is 9000 bytes" in the `Transport` expectation reason
and in its B.1 row, and I ran all three guards.

```
placement_check.py   EXPECTATIONS: 3   B.1 ROWS: 3   B.1 BAD: 0        exit 0
conversion_check.py  MEMBERS: 3   FILES: 6   FINDINGS: 0               exit 0
roster_compile.sh    RECORDED ROWS: 35 checked   SIZE BAD: 0           exit 0
```

**CLAIM.** No rule reads a size inside a reason string, inside VR5's table, or inside a doc
comment. C-4's closure covers the section 1.9 size block alone, which is 35 rows.

**ARGUMENT.** The size oracle compares the section 1.9 block against `size_of`. It reads nothing
else. PG21 compares names. I confirmed one live error of the same class: VR5 states
`MeterSnapshot` at 1592 bytes (`architecture.md:2485`) and the PG25 oracle measures **1600**. The
document therefore already carries a hand-written size that the compiler contradicts, under a
heading that says no size there is hand written.

The tenth review raised the same shape as MINE-P10. Appendix C closes it with "PG24 and PG25, which
measure every stated size (C-4)" (`architecture.md:9180` region). That sentence is false.

**EVIDENCE.** `architecture.md:2473` to `:2498` (VR5), `:2485` (the 1592 claim), `:8588` (B.1); the
three runs above; the oracle line `MeterSnapshot 1600 8`.

**WHAT SHOULD CHANGE.** Put every type that VR5 names and every type an `#[expect]` reason names
into the section 1.9 size block. Then make the oracle read the reason string, or delete the number
from the reason and cite the block row instead. Correct `MeterSnapshot` to 1600 now.

### WARNING: WR-3. The PG25 profile states five substitutions, applies at least eight, and gives a false reason for four relaxed lints

**OBSERVATION.** The PG25 site states: "**The five stated substitutions, because a roster is not a
crate.** Each one is mechanical and each one names what it gives up" (`architecture.md:613`). It
also states: "`clippy::must_use_candidate`, `clippy::missing_const_for_fn`,
`clippy::new_without_default`, and `clippy::missing_panics_doc`, because the roster carries no
function body and each of those four reads an impl the roster does not spell out"
(`architecture.md:608`).

**CLAIM.** The roster carries 11 function bodies. Three of the four relaxed lints fire on them. The
substitution list omits at least three transformations.

**ARGUMENT.** The generator prints `ROSTER IMPLS: 11`. I removed the six-lint profile from the
generated workspace and re-ran the real lint invocation:

```
duet-time/src/lib.rs:87:12: error: this method could have a `#[must_use]` attribute
duet-time/src/lib.rs:91:5:  error: docs for function which may panic missing `# Panics` section
duet-time/src/lib.rs:91:18: error: this method could have a `#[must_use]` attribute
duet-time/src/lib.rs:96:18: error: this method could have a `#[must_use]` attribute
```

Those three lines are `Finite::get`, `Finite::from_finite_const`, and `Finite::new`, which section
2.6a spells out in full with bodies. The stated reason is therefore not the true reason, and the
relaxation covers the one place the roster could have checked.

Three transformations are unstated.

1. **A generic parameter is dropped from the head.** `pub struct Cycle<'buffers> { /* private */ }`
   becomes `pub struct Cycle { pub opaque: Box<()> }`. A generic declaration with real fields fails
   loudly instead, which my R8 probe shows.
2. **Every `impl` block with one bodyless item is dropped.** The document holds 33 `impl` blocks and
   the roster compiles 11. 55 public function declarations reach the roster and none of the other 44
   is compiled or lint checked.
3. **`pub(crate)` becomes `pub`.** That is what forces the two crate-level `#[expect]` attributes on
   the application row, and the site states the consequence without the cause.

A fourth coupling is unstated: a declaration whose name the 1.5 table omits is dropped from the
roster in silence, so PG25's denominator is PG4's output.

**EVIDENCE.** `architecture.md:606` to `:625` (the profile and the substitutions), `:7204` (the
section 15 preamble claim "A block states no function body either"), `roster_compile.sh` functions
`items`, `full_impls`, and `emit_item`; the un-relaxed run above.

**WHAT SHOULD CHANGE.** Correct the stated reason: three of the four lints are relaxed because
substitution 1 removes the doc comment, not because the roster carries no body. Add the three
missing substitutions to the list and correct the count. Then state PG25's denominator rule: a
declaration the 1.5 table omits never reaches the compiler.

### WARNING: WR-4. `ModeToolbar::render` returns a value that cannot carry what the method computes

**OBSERVATION.** Section 15.16 declares the seam:

```rust
fn render(&self, visible: usize, window: &mut Window, cx: &mut App) -> Toolbar;
```

It declares the return type in the same section:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, IntoElement)]
pub(crate) struct Toolbar { mode: Mode }
```

`Mode` is a four-arm enum and `Toolbar` is one byte.

**CLAIM.** The `visible` count and every group the implementation builds are lost at the return.
The W-2 seam exists and carries no data.

**ARGUMENT.** `Toolbar` derives `Copy`, so it can hold no child element list. Its one field is the
mode, which `TopBar` already knows before it calls the registry. Whatever
`ComposeToolbar::render(visible, ...)` builds, the only value it can hand back is
`Toolbar { mode: Mode::Compose }`.

Two readings follow, and both are defects. If `Toolbar`'s own `RenderOnce` body rebuilds the groups
from the mode, then `ModeToolbar` and its `visible` parameter are dead, and K-12's fix is cosmetic.
If `ModeToolbar` is the real builder, then `Toolbar` needs the group list and the cut point, and its
declaration is wrong.

Chunks K2 to K5 each "replace one `render` body" (`architecture.md:6770`). Each one writes a body
whose output the type discards.

**EVIDENCE.** `architecture.md:8475` (the trait method), `:8452` (`Toolbar`), `:5553` (`Mode`),
`:6765` to `:6776` (the seam paragraph), `:5852` (the 10.3 row).

**WHAT SHOULD CHANGE.** Give `Toolbar` the fields it needs, or change the return type. The
smallest honest shape is a `Toolbar` that carries the mode and the visible count, plus a second
value for the overflow list. State which one carries the groups.

### WARNING: WR-5. `LevelMeter` is declared to paint a value it does not hold

**OBSERVATION.** Section 15.16 declares `LevelMeter { strip: StripId, scale: ParamRange }` and
describes it as "The static chrome of one strip meter: the scale ticks, the numeric readout, and the
frame" (`architecture.md:5846`). Appendix A says `LevelMeter` "paints the static chrome and holds
neither value" (`architecture.md:8520`).

Design contract 4.4 defines the readout: "A numeric peak readout sits under the meter, 11 px
tabular, 40 px wide, right aligned. It shows the highest peak since the last reset, as `-3.2`"
(`design-contract.md:769`).

**CLAIM.** The readout is a live value. `LevelMeter` carries no value, so it cannot paint the
readout. W-7's fix moved the bar and the clip state and left the readout behind.

**ARGUMENT.** "The highest peak since the last reset" is `MeterReading::hold`, which section 7.3
names in the same words. `MeterLayer` owns every `MeterReading`. `LevelMeter` holds a `StripId` and
a `ParamRange`, which is a decibel range and not a reading. A `RenderOnce` element with no value
cannot draw `-3.2`.

Test 11 of section 10.7 asserts the contradiction rather than the behaviour: it asserts that
`LevelMeter` "paints the scale, the readout, and the frame and holds neither value"
(`architecture.md:6107`). That assertion cannot be satisfied.

Design contract 1.11 still gives `LevelMeter` the purpose "Peak, RMS, and clip in one vertical or
horizontal bar" (`design-contract.md:211`). The architecture changed the element on purpose and the
contract did not follow, so two documents now describe one element differently.

**EVIDENCE.** `architecture.md:5846` to `:8433`, `:5846`, `:8520`, `:6105` to `:6108`;
`design-contract.md:211`, `:769`.

**WHAT SHOULD CHANGE.** Move the readout to `MeterLayer` with the bar and the mark, and correct the
three places that say `LevelMeter` paints it. Then raise a contract edit for design contract 1.11,
because the two documents disagree.

### WARNING: WR-6. The tempo map has no owner on the score persistence path

**OBSERVATION.** Section 3.6 lists what `score/meta.json` holds: "Schema, parts, staves, voices,
measures, marks, tempo map" (`architecture.md:2603`). Section 1.8 states that `duet-command`
implements `BundleDocument` for `Score`. Section 3.6 declares the two functions that touch those
files:

```rust
pub fn read(meta: &str, notes: &str, spanners: &str) -> Result<Score, ScoreError>;
pub fn write(score: &Score) -> Result<CanonicalDocument, ScoreError>;
```

Neither signature carries a `TempoMap`. Section 2.11 says `duet-core` owns the tempo map. Two other
signatures take it as a separate argument: `engrave(score, map, options, metrics)` and
`SmfImport { score, tempo_map, warnings }`.

**CLAIM.** Two answers exist and the document states neither. Both break something.

**ARGUMENT.** If `Score` holds the `TempoMap`, then `write(score)` is correct and the `Eq` derive on
`Score` does not compile. I built the declared shapes and ran the compiler:

```
error[E0369]: binary operation `==` cannot be applied to type `TempoMap`
error[E0277]: the trait bound `TempoMap: Eq` is not satisfied
```

`TempoMap` derives `Debug`, `Clone`, `Default`, `Serialize`, and `Deserialize`
(`architecture.md:1914`). `TempoPoint` and `MeterPoint` derive no `PartialEq` either.

If `Score` does not hold the map, then `write(score)` cannot produce the tempo map that
`score/meta.json` holds, and `read` cannot return it. The checkout path of section 4.5 step 2 then
loses the map on every checkout.

No guard can decide this, because `Score` carries `/* private */` (WR-1).

**EVIDENCE.** `architecture.md:2603` (the file table), `:2630` and `:2636` (the two signatures),
`:1906` to `:1915` (the three tempo types), `:7424` to `:7426` (`Score`), `:5096` (`SmfImport`),
`:5937` (`engrave`); the compiler run above.

**WHAT SHOULD CHANGE.** State the owner once. If `Score` holds the map, give `TempoMap`,
`TempoPoint`, and `MeterPoint` a `PartialEq` derive and an `Eq` derive, which every field supports.
If it does not, add the map to both signatures and say which type persists it.

### WARNING: WR-7. The pool handoff queue carries no budget id and no overflow rule

**OBSERVATION.** Section 5.5 step 3 reads: "The core wraps the new pool in a `PoolHandle` and pushes
it into a bounded handoff queue, capacity 2, core to audio. It then publishes the new `GraphChain`,
whose `pool_generation` counter has increased" (`architecture.md:3636`). Section 5.8 repeats the
literal: "`crossbeam::queue::ArrayQueue<PoolHandle>`, capacity 2".

**CLAIM.** The literal breaks DR3, and the design states no behaviour when the push fails.

**ARGUMENT.** DR3 says a number is stated once, as a B row, and it lists four exemptions: a version
pin, a platform or format constant, a type size, and a recorded guard count
(`architecture.md:40`). A queue capacity is none of the four. Every other queue in the same table
carries a B id: B28 to B35, B87, B100.

Every other queue also states its overflow action. B87 "keeps the newest and sets the resync flag".
B34 carries "an `AtomicU32` drop counter". The pool queue states nothing.

The failure is reachable. `ArrayQueue::push` returns the value when the queue is full. The audio
thread pops one handle per pool generation change, and it runs no cycle at all in
`EngineState::NoServer`, `OpenTimeout`, `NoDevice`, `RateUnavailable`, and `Faulted`. Three
`configure` calls while the engine is stopped fill the queue. The third push fails, the core still
publishes the increased `pool_generation`, and the audio thread then looks for a handle that never
arrives.

**EVIDENCE.** `architecture.md:40` (DR3), `:3636` (step 3), `:3963` (the 5.8 row), `:4609` (the
stopped-engine states), `:692` to `:797` (the B table).

**WHAT SHOULD CHANGE.** Give the capacity a B row. Then state what a failed push does. The simplest
correct answer refuses the topology change before it publishes the generation, with a named
`ConfigError` variant, which is the shape `PoolExhausted` and `RingBudgetExceeded` already use.

### WARNING: WR-8. A source directory named `target` leaves the conversion guard's file set in silence

**OBSERVATION.** CG1's stated limit names one denominator hole: a crate the `members` glob does not
reach (`architecture.md:1524`). `conversion_check.py:96` holds a second one:

```python
if os.sep + "target" + os.sep in base + os.sep:
```

**CLAIM.** Every `.rs` file under any path component named `target` leaves the scanned set, and the
run reports success over the files it did see.

**ARGUMENT.** I planted one cast in `m/src/target/mod.rs` and one in `m/src/notarget/mod.rs`.

```
target:    MEMBERS: 1   FILES: 1   FINDINGS: 0     exit 0
notarget:  CAST: m/src/notarget/mod.rs: line 1
           MEMBERS: 1   FILES: 2   FINDINGS: 1     exit 1
```

`FILES: 1` proves the guard never read the file. The filter is meant for the Cargo build directory,
which sits at the workspace root and at a member root. It tests any component at any depth.

`clippy::mod_module_files` is denied, so `src/target/mod.rs` itself cannot land. The compliant
layout is `src/target.rs` beside `src/target/*.rs`, and every file in that directory is hidden. A
`target` module is plausible in this plan: `Normalization` carries a loudness target and `LufsMeter`
carries a `target` field.

CG3 is not a backstop here, because CG3 never sees the bytes. Both rules go blind together.

**EVIDENCE.** `architecture.md:1524` (the stated limit), `tools/conversion_check.py:96`; the two
runs above.

**WHAT SHOULD CHANGE.** Anchor the filter to the build directory. Compare the path against the
member root plus `target`, and against the workspace root plus `target`. Then add a probe that
plants a cast in a source module named `target`.

### WARNING: WR-9. Section 14 wires no `check-roster` into CI and states three stale counts

**OBSERVATION.** Section 14 states that PG25 runs in the `plan-lint` job, and its property table
gives the command as "`cargo xtask check-placement ...`, then `cargo xtask check-roster ...`"
(`architecture.md:7001`). The workflow table four paragraphs later gives the same job a different
command:

```
| `ci.yml`, job `plan-lint` | `ubuntu-26.04`, on a `roadmap/**` change | `cargo xtask check-placement roadmap/duet-v1/architecture.md` | M0 |
```

**CLAIM.** Chunk M0 writes `ci.yml` from that table. PG25 would never run in CI.

**ARGUMENT.** The workflow table is the one row that names the file, the runner, the command, and
the writing chunk. It is the row an implementer copies. The property table above it is prose about
the job. Two rows disagree about one command, and the one that names the writing chunk omits the
rule revision 11 exists to add.

Three counts in the same section are stale. Line 7137 reads: "**The thirty-five guard probes are
part of rung two.** Twenty-four cover the placement guard and eleven cover the conversion guard, one
per rule (DR5)." The true counts are 38 and 27. Section 1.5 line 675 says "Twenty-seven tests in
`tools/xtask` hold the guard, one per rule". The two sections disagree about how many tests M0
writes. Line 1077 repeats "all thirty-five probes", and line 1073 writes "PP1 to PP22" where the
range is PP1 to PP24.

**EVIDENCE.** `architecture.md:7010` to `:7010` (the property table), `:7028` (the workflow table),
`:7137` to `:7138`, `:675`, `:1073`, `:1077`.

**WHAT SHOULD CHANGE.** Add `cargo xtask check-roster` to the workflow table row. Correct the four
counts. Then state whether M0 writes 35 tests or 38.

### WARNING: WR-10. PG12's denominator reaches zero in silence

**OBSERVATION.** PG12 reads the section 1.2 dependency column and fails a row that omits a crate its
own declarations prove it uses. I renamed the heading `### 1.2 The crate table`.

```
$ python3 placement_check.py <copy with the 1.2 heading renamed>
DEP ROWS:        0    DEP MISSING: 0
exit 0
```

**CLAIM.** "Found 0 violations" and "scanned 0 rows" print the same success. PG12 has no floor.

**ARGUMENT.** `main` carries a named `FAIL` line for six data blocks: the candidate set, the
framework block, the name map, the drop list, the primitive traits, and the primitive sizes. Each
one exits 1 when its source is absent. The 1.2 crate table has no such guard, although the name map
that lives in the same section does.

The section 1.9 size block has the same shape. I renamed its heading and the run printed
`SIZES DECIDED: 202     SIZE UNDECIDED: 160` and exited 0. Coverage fell from 315 to 202 in
silence. One deleted row does the same at a smaller scale: `Option<PoolSlot> 4 2` removed gives
`SIZES DECIDED: 311`, exit 0.

**EVIDENCE.** `tools/placement_check.py` `main`; the three runs above.

**WHAT SHOULD CHANGE.** Give every table the guard reads the same fail-closed treatment the six
blocks already have. A zero row count is a broken parse, not a clean run.

---

## 3. Concerns

1. **CG6 misses every attribute form this plan writes.** The pattern is the literal
   `#!?\[expect\(clippy::as_conversions`. One space after `(`, a line break, a `cfg_attr` wrapper,
   and the `allow` spelling each defeat it; I ran all four and each exits 0. Every `#[expect]` in
   `architecture.md` uses the multi-line form. CG6 is "the rule clippy cannot carry"
   (`architecture.md:1521`). CG3 is a real backstop for the harmful case, which I confirmed, so the
   tier is Concern and not Warning. CP6 plants only the one form that matches.
2. **A symbolic link at the exempt path exempts any member file.** CG7 canonicalises the exempt
   path, so the exemption follows the target. I made `crates/duet-time/src/convert.rs` a link to a
   live module of a second member that holds a bare cast, and the run exits 0 with `FINDINGS: 0`.
   CP7 plants the safe direction only.
3. **PG21 reads `missing_copy_implementations` alone.** The two new `variant_size_differences` rows
   of Appendix B.1 have no rule that holds the table and the code to one set. B.1 claims that PG25
   "holds this half of the table in both directions" (`architecture.md:8590`); PG25 reads the code
   and never the table.
4. **A declaration whose name is on the candidate drop list is invisible to PG4.** A planted
   `pub struct ExitCode` with no 1.5 row raises `DECLARED` and leaves `UNPLACED` at zero, exit 0.
   `Duration` is the one name that section 1.5 places and the drop list also carries. The overlap
   is unstated.
5. **VR1's prose contradicts the 1.9 `Value` row.** VR1 says "A type with an `f32`, an `f64`, or a
   `serde_json::Value` payload is outside the class" (`architecture.md:2376`). The 1.9 row says
   `Value` supplies `Eq` and `Hash`, and a compiler run confirms it. Eight score records derive `Eq`
   over a `BTreeMap<String, Value>`. An implementer who follows VR1's sentence removes eight
   derives and reopens C-1.
6. **The roster compile runs `cargo clippy` without `--locked`.** The script copies `Cargo.lock`
   and then lets cargo re-resolve; my runs print "Updating crates.io index" and "Locking 1 package".
   The claim that "every transitive version it resolves is the version the repository already
   holds" (`architecture.md:1173`) is approximate.
7. **Five orphan budgets.** B42, B49, B50, B84, and B93 are cited nowhere outside section 1.6.
8. **Fifteen stale "Used by" cells** in section 1.6 name a section that never cites the id. They
   include B1, B2, B19, B26, B42, B49, B50, B53, B71, B77, B84, B86, B89, B90, and B93.
9. **TH7 is stated twice**, at `architecture.md:3889` and `:5349`. DR4 forbids it.
10. **DR6 holds for contradictions and not for duplication.** The six records carry thirteen own
    copies of a number, three own copies of a list, and fourteen rules restated in the record's own
    words. One copy has already drifted: ADR 0004 decision 11 writes "never formats" where TH1
    writes "never formats a string".
11. **B95 cites a fact the contract does not carry.** B95 says the 0.62 split is "the score share
    above the take strips of contract 3.4". Design contract 3.4 describes 10 px take strips and
    states no split and no 0.62.
12. **Line 6750 cites contract 1.3 for `MenuHost`.** Design contract 1.3 is the title bar and names
    no menu. The nearest statement is contract 1.5.
13. **Design contract 1.11 and architecture 10.3 disagree about `LevelMeter`.** The contract needs an
    edit, which WR-5 names.
14. **Appendix B.5 omits the `Eq` derive of `ChainTopology`.** Line 8687 says it derives `Debug`,
    `Clone`, and `PartialEq`; line 3508 derives `Eq` as well.
15. **Chunk F1 names a directory, `src/templates/`, in its write scope.** SM2 says "A directory is
    not a write scope under this rule" (`architecture.md:6494`).
16. **Section 13.3 claim 2 omits the member manifest.** Line 6885 says the `src/lib.rs` is the only
    source file an `M` chunk shares with a line chunk. The member `Cargo.toml` is shared too, in
    thirty pairs.
17. **ADR 0001 and ADR 0005 number their decisions out of order.** ADR 0001 runs 3, 3a, 3b, 3d, 3c.
    ADR 0005 ends at decision 14.
18. **`conversion_check.py` takes an optional path argument** that CG1's own text says the
    subcommand must not accept. The xtask port must drop it.
19. **`conversion_check.py` misses an uppercase file extension.** A cast in `m/src/Other.RS` exits 0.
20. **The roster drops a generic parameter from a private-bodied head in silence.** `Cycle<'buffers>`
    becomes `Cycle`. A fielded generic fails loudly instead.

---

## Reactive Assessment

- **Responsive: PARTIAL.** Section 5.12 gives every cross-boundary wait a timeout and an action on
  expiry, and each one carries a B id. The audio thread now allocates and frees inside the cycle
  (CR-2), which is the one path with a hard 2.67 ms budget at B7.
- **Resilient: PARTIAL.** Every failure surface is a typed enum, no crate holds `unsafe`, and the
  fault queue carries a drop counter. The over-mark latch never reaches the user (CR-1), and the
  pool handoff queue has no overflow rule (WR-7).
- **Elastic: PASS.** Every channel, ring, and queue is bounded, and each bound is a B row with a
  named producer and consumer. The wrapped view is virtualized by `VirtualList`, and the path cache
  is bounded by B61. The one gap is the overflow action of the pool handoff queue.
- **Msg Driven: PASS.** Every thread seam is a push: `triple_buffer` for a latest value, `rtrb` and
  `ArrayQueue` for a stream, `async_channel` for structural traffic. The one poll is
  `MidirPresence`, and it carries a documented reason at its own site.

---

## Verdict

**NOT READY.** The engineering is close, and the roster compile is the right answer to the
revision-10 cause: a guard that reads one markdown file cannot see a lint table, and now a compiler
does. Four Criticals block. The single biggest risk is that the new oracle reads 44 declarations as
an empty struct, so the two design faults with the largest blast radius, the meter latch and the
audio-thread allocation, sit exactly where no guard looks. The weakest Reactive property is
**Resilient**: the design contains every failure it names, and the one user-visible safety signal,
the clip latch that tells a singer a take is ruined, can never light.

### The blocking list

1. **CR-1** The generation compare makes the `CLIP` indicator unreachable, and one scalar cannot
   publish 48 per-strip reset counters.
2. **CR-2** The boxed `SlotState::Equalizer` arm allocates and frees on the audio thread, against
   TH1 and ADR 0004 decision 8c.
3. **CR-3** PP22's recorded result is false and PP21 cannot be planted, so PG21 and PG22 are
   unverified rules.
4. **CR-4** `crates/duet` names three `duet-engine` types and a `triple_buffer` type with no edge
   and no dependency.
5. **WR-1** The `Box<()>` substitution takes 44 declarations out of PG19, PG23, PG24, and PG25.
6. **WR-2** A false size inside an `#[expect]` reason passes all three guards; `MeterSnapshot`
   states 1592 and measures 1600.
7. **WR-3** The PG25 profile states five substitutions, applies at least eight, and gives a false
   reason for four relaxed lints.
8. **WR-4** `ModeToolbar::render` returns a one-byte `Toolbar` that carries neither the visible
   count nor a group set.
9. **WR-5** `LevelMeter` is declared to paint the numeric peak readout and holds no value.
10. **WR-6** The tempo map has no owner on the score persistence path.
11. **WR-7** The pool handoff queue carries no budget id and no overflow rule.
12. **WR-8** A source directory named `target` leaves the conversion guard's file set in silence.
13. **WR-9** Section 14's workflow table wires no `check-roster` into CI and states three stale
    counts.
14. **WR-10** PG12's denominator, and the section 1.9 size block, each reach zero rows in silence.
