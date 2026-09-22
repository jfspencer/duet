# Specification review, revision 12: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-21. Mode: specification review, before plan authoring.
This is the twelfth pass. Revisions 1 to 11 each returned NOT READY.

## What I read and what I ran

Read in full: `roadmap/duet-v1/architecture.md` (9930 lines), the six records under
`roadmap/duet-v1/adr/`, the three prototypes under `roadmap/duet-v1/tools/`,
`research/linux-macos-platform.md`, `product-requirements.md` section 8, `design-contract.md`
sections 1, 1.1, 1.6, 1.11, 3.1, 3.3, 4.4 and the `StageCurve` appendix, `CLAUDE.md`, `Cargo.toml`,
`.cargo/config.toml`, `clippy.toml`, `deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`. I also
read the Architect's revision-12 report from the plan store.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/`. I ran `cargo` only inside that copy and inside the scratchpad. I wrote no
repository file and I ran no other git command.

**All three guards reproduce the recorded baseline, line for line.** The roster compile exits 0 in
50 seconds and measures 405 sizes.

I ran all 39 recorded probes. I added 9 probes to the placement guard, 7 to the conversion guard,
and 6 to the roster compile. I planted three defects the real gate catches and confirmed that PG25
catches each one. I ran one separate compiler experiment on a sanctioned suppression.

**Revision 12 is a large advance.** All 44 placeholder bodies are written. The compiler now reads
every field of every declaration, and the guard prints `PLACEHOLDERS: 0`. The clip latch works in
both directions. The audio thread allocates and frees nothing. The tempo map has an owner. The
conversion guard anchors its build-directory exclusion. Nineteen of the twenty Concerns close.

It does not close. **Four Criticals block it.** Two are design faults that the new bodies made
visible for the first time. One is the same false probe result that revision 11 failed on, in the
one row the guard cannot re-run. One is the new guard itself: PG26 reports success on three hostile
inputs, and it is the only mechanism that holds the revision-11 Critical.

---

## 1. Closure check on CR-1 to CR-4, WR-1 to WR-10, and the twenty Concerns

| Id | Finding | State | Section and reason |
|---|---|---|---|
| CR-1 | The generation compare makes `CLIP` unreachable | **CLOSED** | 7.3 states a per-strip compare over `MeterSnapshot::latched`. Four cases follow from one sentence and each one is correct. Test 9 asserts three states in order. |
| CR-2 | The boxed arm allocates and frees on the audio thread | **CLOSED** in the design | 15.2 and 5.5 keep every `SlotState` arm inline. Migration rule 2 is a write and rule 3 frees nothing. B51 is a measured 232 bytes. The guard that holds it is defective; see CR-14. |
| CR-3 | Two recorded probe results are false | **PARTIAL** | The 28 placement probes and the 11 conversion probes each reproduce their recorded exit code and counter. PP25 does not. See CR-13. |
| CR-4 | `crates/duet` names three engine types | **CLOSED** | 15.14 declares `MeterReader` and `TransportReader`. `REACH BAD: 0` and `DEP MISSING: 0`. The fix created a new fault; see CR-11. |
| WR-1 | The roster substitutes an empty body for 44 declarations | **CLOSED** | `PLACEHOLDERS: 0`. Substitution 2 is deleted. PG3 fails on a comment where a field belongs. |
| WR-2 | A false size inside a reason passes all three guards | **PARTIAL** | VR5 states no number and PG21 reds a literal in a reason. A size inside a declaration doc comment is still unread, and one is already stale. See WR-17. |
| WR-3 | The profile states five substitutions and applies eight | **PARTIAL** | Section 1.9 holds the block, the list is seven, and PG25 refuses an id mismatch. Two counts inside the block are false. See WR-16. |
| WR-4 | `ModeToolbar::render` returns a one-byte value | **CLOSED** | 15.16 declares `ToolbarGroups`, `ToolbarGroup`, and `ToolbarControl`. The return carries the mode, the groups, and the visible count. |
| WR-5 | `LevelMeter` paints a value it does not hold | **CLOSED** | 15.16 makes the element stateless. Design contract 1.11 and 4.4 carry the Orchestrator's two edits. A new fault sits in one field; see WR-14. |
| WR-6 | The tempo map has no owner | **CLOSED** | 15.3 gives `Score` a `tempo_map` field. `TempoMap`, `TempoPoint`, and `MeterPoint` each derive `Eq`, and the roster compiles all three. |
| WR-7 | The pool handoff queue has no budget id and no overflow rule | **PARTIAL** | B105 exists and `ConfigError::PoolHandoffBusy` refuses before the generation moves. The retry has no bound. See WR-18. |
| WR-8 | A source directory named `target` leaves the file set | **CLOSED** | CG1 takes the file set from the `cargo metadata` targets. My run of CP1 shape 2 reds with `FILES: 3`. |
| WR-9 | Section 14 wires no `check-roster` and states stale counts | **PARTIAL** | The `ci.yml` row runs both subcommands. Section 14 states no count. One count inside section 1.9 is stale. See concern 3. |
| WR-10 | PG12's denominator reaches zero in silence | **PARTIAL** | Nine tables fail closed on zero rows. The name map and the audio-owned block do not. See WR-19 and CR-14. |

**The twenty Concerns: 19 CLOSED, 1 PARTIAL.** I verified each one against the text or against a
run. Concern 1 (CG6 attribute forms), concern 2 (the symbolic-link exemption), concern 3 (PG21 over
the variant table), concern 18 (the path argument), and concern 19 (an uppercase extension) each
red in my own run of CP6, CP7, PP21b, CP1 shape 4, and CP1 shape 3. Concern 20 closes: the roster
emits `pub struct Cycle<'buffers>` with its lifetime. Concern 17 closes: every decision list in all
six records ascends. **Concern 8 is PARTIAL.** Two `Used by` cells still name a section that never
cites the id; see concern 4 below.

### 1.1 The baseline run of all three guards

The placement guard and the conversion guard each reproduce section 1.9 line for line. The roster
compile prints the recorded block, including `ROSTER ITEMS: 381`, `ROSTER SUBS: 7`,
`ROSTER SIZES: 405 measured`, and `SIZE BAD: 0`.

### 1.2 The 39 recorded probes

37 of 39 reproduce the recorded exit code and the recorded message. **Both failures belong to
PP25**, which is the one probe that needs a compiler. See CR-13.

### 1.3 Three planted defects that the real gate catches

I planted each defect in a copy and ran the roster compile. PG25 caught all three, and no
substitution hid any of them.

| Plant | Site | Result |
|---|---|---|
| `#[allow(dead_code)]` on a declaration | `Accidental`, section 3.3 | exit 1; `error: #[allow] attribute found` and `error: allow attribute without specifying a reason` |
| `.unwrap()` inside a spelled-out body | `Finite::new`, section 2.6a | exit 1; `error: used unwrap() on an Option value` |
| A bare `as` cast inside a spelled-out body | `Finite::get`, section 2.6a | exit 1; `error: using a potentially dangerous silent as conversion` |

PG25 also refuses a damaged substitution block. I renamed `S5` to `S9` and the guard exited 2 with
`FAIL: the section 1.9 substitution block and the script disagree`. I changed the recorded size row
`Value 72 8` to `Value 720 8` and the guard exited 1. Both mechanisms work.

### 1.4 The mechanical passes

1. **Budgets.** 105 ids, B1 to B105, with no gap and no duplicate row in section 1.6. Every
   `B<number>` citation resolves. **No budget is an orphan now**: B62 is cited in ADR 0006, which
   its `Used by` cell names.
2. **Rule ids.** Every family is complete: DR1 to DR6, PL1 to PL3, VR1 to VR6, TH1 to TH10, SM0 to
   SM7, 28 PG rules against 28 PP probes, and 11 CG rules against 11 CP probes. DR5 holds.
3. **Chunk coverage.** 55 chunks. Every chunk sits in exactly one phase. Every width cell matches
   the chunk count of its row.
4. **The plan graph.** 13.4 expands to 46 ordered pairs. No link runs backward. One same-phase
   pair, M0 before T1, which SM1 states.
5. **SM6.** No line runs two chunks in one phase. The trunk is four lines of one chunk each, so T2
   beside T3 in phase 1 is correct.
6. **Write scopes.** No two line chunks in one phase write one path. `Cargo.lock` is the one shared
   name, and SM5 rule 4 governs it.
7. **MUST stories.** 64 MUST rows in the requirements and 64 rows in 13.2. The two sets are equal.
8. **ADR citations.** Every section citation and every B id resolves. I found no contradiction
   against the current text.
9. **Design contract.** Contract 1.1 gives 1440 by 900 as the default, 1024 by 700 as the minimum,
   and 900 as the floor, which are B91, B92, B102, B103, and B104. Contract 1.6 gives 240, 300, and
   45 percent, which are B94, B97, and B98. Contract 1.11 and 4.4 carry the two edits the
   Orchestrator applied.

---

## 2. New findings in revision 12

### CRITICAL: CR-11. Two views each need the one read end of one publication

**OBSERVATION.** Section 5.8 lists two audio-to-user-interface paths, and each one is a single
value:

```
architecture.md:4138  | Audio | UI | `triple_buffer::Output<TransportSnapshot>` | Once per frame |
architecture.md:4139  | Audio | UI | `triple_buffer::Output<MeterSnapshot>` | Once per frame |
```

Section 15.14 gives each reader one `Output`:

```rust
pub struct MeterReader { output: Output<MeterSnapshot>, resets: Arc<ResetGenerations> }
pub struct TransportReader { output: Output<TransportSnapshot> }
```

Section 15.16 then gives each reader to **two** views:

```
architecture.md:9078  pub(crate) struct MeterLayer { meters: MeterReader, slots: ..., shown: MeterView }
architecture.md:9087  pub(crate) struct MasterMeterLayer { meters: MeterReader, shown: MeterView, ... }
architecture.md:9063  pub(crate) struct PlayheadLayer { transport: TransportReader, shown: TransportView }
```

`DuetApp` holds `mix: Entity<MixView>` and `master: Entity<MasterView>` at the same time.
`MixView` holds `meters: Entity<MeterLayer>` and `MasterView` holds
`meters: Entity<MasterMeterLayer>`. `ComposeView` and `RecordView` each hold their own
`playhead: Entity<PlayheadLayer>`.

**CLAIM.** The design needs two `Output<MeterSnapshot>` values and two `Output<TransportSnapshot>`
values. Each publication supplies exactly one, and the type cannot be duplicated.

**ARGUMENT.** `triple_buffer::triple_buffer()` returns one `Input` and one `Output`. The section 1.9
external table states the consequence: `Output` is "not Copy" and it "supplies none of the nine"
closure traits, so it is neither `Clone` nor `Copy`. Only one value exists, and only one owner can
hold it.

Appendix A states the same rule and then contradicts section 15.16:

```
architecture.md:9241  `duet_core::MeterReader`, which `Entity<MeterLayer>` holds. It owns the
                      `triple_buffer` output ... so it is the one owner of both values (critic CR-4)
```

`MasterMeterLayer` is a second owner. Section 10.2 makes that deliberate: the frame-driver table
gives Mix to `MeterLayer` and Master to `MasterMeterLayer`, and both paint meter values.

The transport half is the same shape at a larger scale. Section 10.2 gives Compose and Record one
`PlayheadLayer` each, and the Appendix A ownership tree gives each work-area view its own child
entity. Two `PlayheadLayer` values exist, and each one holds a `TransportReader`.

**No guard sees this.** PG13 reads the derive on the published type. PG20 reads the crate of a field
type. PG25 compiles a declaration, and a type may appear as a field of two structs. I confirmed that
in the generated roster: `duet/src/lib.rs` carries `MeterReader` in both `MeterLayer` and
`MasterMeterLayer`, and the roster exits 0. My own probe MY-P9 added a third `MeterReader` to
`StripView` and every counter stayed at zero.

This defect entered with the CR-4 fix. In revision 11 all three views carried `/* private */`, so no
reader could count the owners.

**EVIDENCE.** `architecture.md:4138` to `:4139`, `:8693` to `:8711`, `:9062` to `:9091`, `:9241`,
`:1140` (the `Output` row), `:5794` to `:5798` (the frame-driver table), `:8770` to `:8773`
(`DuetApp`), `:9014` and `:9043`; my run of MY-P9 and the generated roster source.

**WHAT SHOULD CHANGE.** Name one owner per publication and state how a second view reads it. The
smallest correct answer gives `CoreHost` the two readers, makes it call `read` once per frame, and
publishes the `MeterView` and the `TransportView` to the child views through the ordinary entity
path. Then correct section 10.2, because a frame driver that owns no reader is a different design.

### CRITICAL: CR-12. `duet-session` must enforce two budgets whose constants it cannot reach

**OBSERVATION.** Section 5.5 gives `duet-session` two refusals:

```
architecture.md:3867  `MixState::validate` and `SessionCommand::BusAdd` return
                      `MixError::StripBudgetExceeded { requested }` ... past B86
architecture.md:3898  `MixState::validate` and `SessionCommand::StripInsertSlot` return
                      `MixError::ParamBudgetExceeded { requested }` ... past B90
```

Section 7.1 declares the function in `duet-session`:
`pub fn validate(mix: &MixState) -> Result<(), MixError>;`. It takes one argument.

Section 1.6 declares both constants, and each one sits in a crate `duet-session` does not reach:

```
architecture.md:883   // duet-dsp     pub const MAX_STRIPS: usize = 48;
architecture.md:891   // duet-engine  pub const MAX_PARAMS: usize = 2_048;
```

Section 1.3 gives `duet-session` two edges only: `duet-session -> duet-time, duet-score`.

**CLAIM.** `MixState::validate` cannot compare against either budget. The `MAX_PARAMS` half cannot
be fixed by an edge, because that edge is a Cargo cycle.

**ARGUMENT.** DR3 states that section 1.6 declares every workspace constant, "so a literal appears
in no other Rust block". `duet-session` therefore cannot write 48 or 2048. It must name the
constant, and a Rust crate names a constant through a dependency.

`duet-session -> duet-dsp` does not exist today. It is acyclic, so it is available.
`duet-session -> duet-engine` is a cycle: section 1.3 already carries
`duet-engine -> duet-session`. That is blocker V1 and finding S2 in the same shape, one revision
after the document moved four types to remove them.

Section 1.6 itself records the use. The `Used by` cell of B86 names 15.4 and the cell of B90 names
15.4, and 15.4 is the `duet-session` roster. The column's own site calls that cell "a fact and not a
claim".

`MAX_STRIPS` moved to `duet-dsp` in this revision, for a correct reason that the CR-4 fix needed:
`MeterView` is a `duet-dsp` type. The move did not consider the third consumer.

**No guard sees this.** PG20 reads a field type, and a constant inside a function body is not a
field. PG11 reads a sentence that joins two crate names with one of nine verbs, and section 5.5
names a method rather than a crate. PG25 compiles a declaration and never a body. My probe MY-P6 put
a `PoolBuffer` field inside a `duet-session` declaration and PG20 reds it with `REACH BAD: 1`, which
shows that the rule works for a field and not for a constant.

**EVIDENCE.** `architecture.md:216` (the `duet-session` edges), `:799` and `:803` (the two `Used by`
cells), `:878` to `:891` (the constant block), `:3867`, `:3898`, `:4686` (`validate`), `:7936` to
`:7942` (`MixError`); my run of MY-P6.

**WHAT SHOULD CHANGE.** Decide where each budget lives, and make the deciding crate the lowest one
that every enforcer reaches. `duet-session` is the lowest crate that `duet-engine`, `duet-dsp`
consumers, and the application all reach for the strip count and the parameter count, so both
constants belong there or in `duet-time`. Then add a guard rule that reads a B id citation in a
crate-scoped sentence against the section 1.3 graph, because the field rule cannot see a constant.

### CRITICAL: CR-13. PP25's recorded result is false in both shapes

**OBSERVATION.** DR5 reads: "A probe is correct when the baseline run is green and the planted run
is red" (`architecture.md:978`). Section 1.9 records this for PG25:

```
architecture.md:1021  | PG25 | ... | PP25 | Two shapes, each in its own run: the `Eq` derive removed
  from `Revision`, and `Equalizer(EqualizerState)` in place of the boxed arm | `exit 1` both times;
  `duet-score/src/lib.rs:304:30: error: you are deriving `PartialEq` and can implement `Eq`` and
  `duet-dsp/src/lib.rs:12:5: error: enum variant is more than three times larger (224 bytes) ...`
```

**Shape 2 cannot be planted.** Revision 12 removed the boxed arm. `architecture.md:3686` reads
`Equalizer(EqualizerState)` already. The stated edit is a change of the text into the text it
already holds, so the run is the baseline and it exits 0.

**Shape 1 gives a different error.** I removed the `Eq` derive from `Revision` and ran the roster:

```
$ bash roster_compile.sh <copy> <scratch> <repo>
duet-score/src/lib.rs:322:5: error[E0277]: the trait bound `Revision: std::cmp::Eq` is not satisfied
error: could not compile `duet-score` (lib) due to 1 previous error
EXIT=1
```

`Score` derives `Eq` over a `Revision` field, so the type check fails before clippy reaches the
nursery lint. The recorded message names a lint the run never emits, at a line the run never names.

**CLAIM.** PG25 is an unverified rule. The one row of the contract table that needs a compiler is
the one row that was not re-run.

**ARGUMENT.** The Architect's report states "every probe result comes from a fresh run (28
placement, 11 conversion)". PP25 is neither placement nor conversion; it is the roster. The report's
own closure sentence for CR-3 lists eight rewritten cells and PP25 is not among them.

I found the plant that does produce the recorded message. I removed the
`#[expect(variant_size_differences, ...)]` attribute from `SlotState` and the roster printed
`duet-dsp/src/lib.rs:14:5: error: enum variant is more than three times larger (224 bytes) than the
next largest`, exit 1. That is PP24's plant, at line 14 rather than the recorded line 12.

Two more cells cite the wrong section. PP24 and PP26 each say "in section 15.2", and `SlotState` is
declared at `architecture.md:3684`, which is section 5.5. Section 15.2 declares its payload types
and not the enum.

This is the class revision 12 exists to remove. Chunk M0 ports each probe to one test in
`tools/xtask`. An implementer who writes the PP25 test from this table writes a test that fails, or
writes one that asserts exit 0 and locks a vacuous state into the `plan-lint` job.

**EVIDENCE.** `architecture.md:978` (DR5), `:1021` (the cell), `:1020` and `:1022` (the two section
citations), `:3684` to `:3694` (`SlotState`); my three roster runs above.

**WHAT SHOULD CHANGE.** Rewrite both PP25 shapes against this revision and paste the real output.
Shape 1 needs a type whose `Eq` no container demands, so the nursery lint fires. Shape 2 needs a
plant the document does not already hold. Correct the section citation in PP24 and PP26. Then state
at the PG25 site that a roster probe needs its own run, because the placement harness cannot
reproduce it.

### CRITICAL: CR-14. PG26 fails open, and its denominator is a hand list with two holes

**OBSERVATION.** PG26 is the one rule that holds TH1 against a declaration, and it is the whole
answer to the revision-11 Critical. Its site states a fail-closed promise:

```
architecture.md:4008  The guard is fail-closed when the block is absent.
```

`placement_check.py:352` reads the block with one regular expression:

```python
r"#### Every audio-owned declaration\n.*?```[a-z]*\n(.*?)```"
```

**CLAIM.** Three hostile inputs make PG26 report success it did not earn, and two audio-owned types
are outside its set on the unmodified document.

**ARGUMENT.** I ran four probes.

| Probe | Shape | Result | Caught |
|---|---|---|---|
| MY-P3 | The fenced block deleted, the heading kept | `AUDIO OWNED: 88`, exit 0 | **No** |
| MY-P4 | The block reduced to one harmless name | `AUDIO OWNED: 1`, exit 0 | **No** |
| MY-P5 | `SlotState` removed from the block, then `Equalizer(Box<EqualizerState>)` planted | `AUDIO OWNED: 21`, `HEAP IN AUDIO: 0`, exit 0 | **No** |
| MY-P1, MY-P2 | `probe: Vec<u8>` added to `ChainState`, and to `Transport` | `HEAP IN AUDIO: 0`, exit 0 each | **No** |

MY-P3 is the worst of the four. The regular expression is lazy up to the **next** fenced block, so
it skipped the deleted block and parsed a later Rust block. I confirmed the parsed set: it holds
`"the`, `#[derive(Debug,`, `Clone,`, `Disabled,` and 84 more source fragments. The guard then
scanned that set, found no heap name, and exited 0. **The denominator counter rose from 22 to 88, so
a reader sees more coverage and gets none.** The promise holds only when the heading goes too: I
removed both and the guard printed `FAIL: section 5.7 declares no audio-owned declaration block.`

MY-P5 is the direct attack on CR-2. One line removed from a fenced block in a markdown file takes
the revision-11 shape back through the guard that exists to refuse it.

MY-P1 and MY-P2 are denominator holes on the unmodified document. Section 5.5 states "The audio
thread owns every `ChainState`" and "the audio thread reconciles its own state in place, inside the
cycle". VR5 states that "the live transport is an identity the audio thread owns". Neither name is
in the block, so a heap field on either passes.

The same regular expression shape reads the substitution block. My probe MY-R6 removed that block
and kept its heading, and the roster printed
`FAIL: ... the block holds ['f32', 'str', 'u8'] and the script applies ['S1', ... 'S7']`. It exits 2
there only because the identifier compare rejects the garbage. PG26 has no such compare, so the same
defect fails open.

**EVIDENCE.** `architecture.md:4008` (the promise), `:4018` to `:4041` (the block), `:3823` and
`:3638` (`ChainState`), `:4197` to `:4211` (`Transport`), `tools/placement_check.py:343` to `:355`;
my five runs above.

**WHAT SHOULD CHANGE.** Anchor the regular expression to the fence that follows the heading, and
fail closed when the parsed set holds a token that is not an identifier. Give PG26 a floor: a run
whose audio-owned set is below the recorded count is a failure. Add `ChainState` and `Transport` to
the block. Then state at the site that the block is a hand list, and name the rule that keeps it
complete.

---

### WARNING: WR-11. Three queue ends are declared with a type that cannot hold the stated rule

**OBSERVATION.** Section 5.8 declares three boundaries as a `crossbeam` queue:

```
architecture.md:4164  | MIDI thread | `duet-core` | `crossbeam::queue::ArrayQueue<NoteEntry>`, B31 |
architecture.md:4165  | Audio | Disk | `crossbeam::queue::ArrayQueue<RefillRequest>`, B33 |
architecture.md:4169  | MIDI presence listener | MIDI thread | `crossbeam::queue::ArrayQueue<PlatformPort>`, B87 |
```

Section 15.10 and section 15.11 declare the same three ends as an `rtrb` producer:

```
architecture.md:8436      requests: Producer<RefillRequest>,
architecture.md:8520      notes: Producer<NoteEntry>,
architecture.md:8529      ports: Producer<PlatformPort>,
```

The section 1.2 name map binds `Producer` to `rtrb`, and the external-name block writes
`Producer  rtrb::Producer`.

**CLAIM.** Two sections declare one mechanism differently, and the declared type cannot implement
the two overflow rules the document states.

**ARGUMENT.** Section 8.1 step 4 states: "On overflow either queue keeps the newest value and sets a
resync flag." Section 8.3 states: "A full note-entry queue drops the oldest record and increments a
counter." `crossbeam::queue::ArrayQueue` supplies `force_push`, which evicts the oldest element.
`rtrb::Producer::push` returns the value on a full ring and evicts nothing, and the producer end
holds no method that reads or removes an element. Neither stated rule is reachable from the declared
type.

The three bodies are new in revision 12. Revision 11 wrote `/* private */` for `MidiSink`,
`HotplugSink`, and `DiskReader`, so no reader could compare the field with the mechanism table.

PG12 passes either way, because the `duet-midi` and `duet-engine` rows list both `rtrb` and
`crossbeam`. No rule reads a mechanism table against a field type.

**EVIDENCE.** `architecture.md:4164` to `:4170`, `:5160` (step 4), `:5321` (the note-entry rule),
`:8430` to `:8438`, `:8516` to `:8532`, `:118` to `:133` (the name map), `:1201` to `:1203` (the
external-name block).

**WHAT SHOULD CHANGE.** Pick one mechanism per boundary and write it in both places. Keep
`ArrayQueue` where the overflow rule evicts, and keep `rtrb` where the producer only pushes. Then
correct the field type or the table row so that section 15 and section 5.8 name one type.

### WARNING: WR-12. The sanctioned `HotplugSubscription` suppression does not build

**OBSERVATION.** Appendix B.1 lists five `missing_copy_implementations` expectations. One of them
names a `Drop` impl in its own reason:

```
architecture.md:9303  | `duet-midi::hotplug::HotplugSubscription` | "a subscription is an identity
  whose `Drop` unregisters the listener, so a caller must not duplicate it in silence" |
```

Section 15.11 declares the type with the same reason and the doc comment "Dropping it stops the
notification".

**CLAIM.** The expectation is unfulfilled once the `Drop` impl lands, and `-D warnings` makes an
unfulfilled expectation a build error.

**ARGUMENT.** `missing_copy_implementations` fires on a type that **could** implement `Copy`. A type
with a manual `Drop` impl cannot implement `Copy`, so rustc skips it and the lint never fires. VR5's
own site states the mechanism in the other direction: "`unfulfilled_lint_expectations` is a rustc
lint at `warn`, and `-D warnings` then turns an expectation that cannot be fulfilled into a **build
error**".

I proved it with the pinned toolchain in a separate crate:

```
error: this lint expectation is unfulfilled
 --> src/lib.rs:2:5
  |
2 |     missing_copy_implementations,
  = note: `-D unfulfilled-lint-expectations` implied by `-D warnings`
```

PG25 passes because the roster carries no `impl Drop` block. That is substitution 5's stated limit
reached in a new place: the roster reads a declaration and never an impl.

This is the fourth of the four faults the Architect's own report names, answered with the opposite
defect. The report reads: "`HotplugSubscription` is all-`Copy` and must refuse the derive, because
its `Drop` unregisters a listener."

**EVIDENCE.** `architecture.md:2546` to `:2556` (the VR5 mechanism), `:8541` to `:8549` (the
declaration), `:9303` (the B.1 row); my compiler run above.

**WHAT SHOULD CHANGE.** Delete the row. A type with a `Drop` impl needs no expectation, because the
lint cannot fire on it. Then state that rule at the VR5 site, because three more handles in this
plan carry a `Drop` and a future one may be all-`Copy`.

### WARNING: WR-13. Two elements hold an identifier range where the contract needs a value range

**OBSERVATION.** Section 7.1 declares the type:

```
architecture.md:4632  /// The contiguous identifier range one slot's parameters occupy.
architecture.md:4633  pub struct ParamRange { first: ParamId, count: u16 }
```

Section 15.16 gives it to two elements as a scale:

```
architecture.md:9118      scale: ParamRange,          // LevelMeter
architecture.md:9125  pub(crate) struct Knob { parameter: ParamId, value: Finite, range: ParamRange }
```

**CLAIM.** Neither element can compute the value it must paint. A `ParamId` range carries no
decibel bound and no parameter minimum or maximum.

**ARGUMENT.** Design contract 4.4 states the meter scale: "Scale from -60 dB at the bottom to +6 dB
at the top, non-linear. -20 dB lands at 50 percent of the height." A pair of `ParamId` values and a
count of 12 cannot produce that mapping.

Design contract 4.5 states the knob rule: "The ratio is one percent of the range per pixel." The
range there is the parameter's own value range. `ParamRange::count` is a count of parameters in one
slot, which B96 sets at 12 for an equalizer and 3 for a gate.

Both fields are new in revision 12. Both bodies were `/* private */` in revision 11.

PG19, PG20, and PG25 all pass: the type exists, `duet-session` is reachable from `crates/duet`, and
the struct compiles. The defect is the meaning of the field, which no rule reads.

**EVIDENCE.** `architecture.md:4628` to `:4633`, `:9113` to `:9125`, `:809` (B96);
`design-contract.md:752` to `:775` and the section 4.5 block.

**WHAT SHOULD CHANGE.** Declare the value range the two elements need, or pass the two bounds the
contract states. Then correct the `LevelMeter` and `Knob` field types and name the section that
declares the new type.

### WARNING: WR-14. The substitution block states two impl counts and both are false

**OBSERVATION.** Section 1.9 holds the block that PG25 reads as its own contract:

```
architecture.md:1319  S5 bodyless-impl  an `impl` block with one item that has no body is not
  compiled; the roster states 33 blocks and compiles the 11 that are complete
```

Section 1.5 repeats one of the numbers: "the roster compiles 11 complete `impl` blocks"
(`architecture.md:631`). Section 15's preamble states a third number: "This document does spell out
15 `impl` blocks in full" (`architecture.md:7513`).

**CLAIM.** The document holds 39 impl blocks and the roster compiles 15. Two of the three sites are
false, inside the block that the revision added to make the count one number.

**ARGUMENT.** I counted the impl blocks inside every ` ```rust ` fence: 39. The baseline run prints
`ROSTER IMPLS: 15`. Only the section 15 preamble is right.

The WR-3 fix is real but narrow. PG25 compares the identifier **set** of the block against the
script, and nothing else. I proved the limit: I renamed `S5` to `S9` and the guard exited 2, and I
changed no number in any other run and no counter moved. The prose inside a line is unchecked.

The same sentence in section 1.5 carries the corrected reason for four relaxed lints, so a reader
who checks that reason meets a false count first.

**EVIDENCE.** `architecture.md:631`, `:1310` to `:1322`, `:7513`, the baseline `ROSTER IMPLS: 15`;
my count of 39 impl blocks and my run of MY-R4.

**WHAT SHOULD CHANGE.** Write 39 and 15, and state that both come from the run. Then make PG25 read
the two numbers out of the S5 line and compare them with its own counts, because a number the guard
prints and a number the block states must be one number.

### WARNING: WR-15. A stale size inside a declaration doc comment passes all three guards

**OBSERVATION.** Section 5.9 states a size in the `MeterSnapshot` doc comment:

```
architecture.md:4245  /// `live` says how many entries the mixer view reads ... One `MeterReading`
architecture.md:4246  /// is 32 bytes, so the snapshot is about 1.6 kilobytes at B86, and one write
architecture.md:4247  /// is a memory copy of that size, which fits B8 with room to spare.
```

The PG25 roster measures the type at **1976 bytes**, which my run confirms in `sizes.txt`. The new
`latched: [Generation; MAX_STRIPS]` array added 384 bytes and the sentence did not follow.

**CLAIM.** WR-2 is closed for VR5 and for an `#[expect]` reason, and open for a doc comment. A
declaration doc comment is code that a chunk writes.

**ARGUMENT.** PG21 reads a size inside an `#[expect]` reason, inside the VR5 table, and inside an
Appendix B.1 reason. It reads no doc comment. I changed "about 1.6 kilobytes" to "about 9000
kilobytes" and ran all three guards: every counter stayed at zero and each guard exited 0.

PG24's own site claims the opposite: "**Every size this document states is a value this rule printed
and PG25 measured**". That sentence is false while this one sentence stands.

**EVIDENCE.** `architecture.md:4241` to `:4260`, `:2558` to `:2563` (the VR5 rule), `:584` (the PG24
claim); my run of MY-P7 and the measured row `MeterSnapshot 1976 8`.

**WHAT SHOULD CHANGE.** Delete the number and cite the PG24 table row for `MeterSnapshot`. Then
extend PG21 to a doc comment, because every declaration in section 5 and section 15 carries one and
a chunk copies it into the code.

### WARNING: WR-16. PG12's name map reaches one row in silence

**OBSERVATION.** PG12 reads two inputs: the section 1.2 dependency column and the third-party name
map. WR-10's fix gave the dependency table a floor. The name map has none.

**CLAIM.** "Found 0 violations" and "scanned 1 mapping" print the same success.

**ARGUMENT.** I deleted the fenced block under the heading `#### The third-party name map` and kept
the heading. The run printed `NAME MAP: 1` and `DEP ROWS: 16   DEP MISSING: 0`, and it exited 0.
PG12's input fell from 16 mappings to one and the run reported a clean dependency column.

The cause is the regular expression that CR-14 names. The reader skipped the deleted block and
parsed the next fenced block, which holds one usable line.

The section 1.5 site states the opposite: WR-10's closure sentence says that every table the guard
reads is fail-closed on zero rows. This input reaches one row, not zero, so a floor at zero would
not have caught it either.

**EVIDENCE.** `architecture.md:112` to `:133` (the name map), `tools/placement_check.py`, my run of
the KEEPHEAD-namemap probe.

**WHAT SHOULD CHANGE.** Give every data block a recorded row count and fail the run when the parsed
count differs. A count is the only check that separates a damaged block from an absent one.

### WARNING: WR-17. The pool-handoff retry has no bound, and the queue cannot drain while the engine is stopped

**OBSERVATION.** Section 5.5 step 3 states the refusal that closes WR-7:

```
architecture.md:3770  `configure` returns `ConfigError::PoolHandoffBusy` and the generation does not
  move, so the audio thread never looks for a handle that did not arrive.
```

Section 15.10 states the recovery in four words: "The caller retries (R12)"
(`architecture.md:8495`).

**CLAIM.** The retry names no caller, no attempt cap, and no backoff, and the trigger the review
found in revision 11 still holds.

**ARGUMENT.** Section 7.3 states that no audio cycle runs in `EngineState::NoServer`, `OpenTimeout`,
`NoDevice`, `RateUnavailable`, and `Faulted`. Section 5.5 step 4 states that the audio thread pops
the handle at the generation check. With no cycle there is no pop, so a full queue at B105 stays
full for as long as the engine stays stopped.

Three `configure` calls while the engine is stopped therefore fill the queue and the third one
refuses. A retry against that state never succeeds. Section 5.12 gives every other cross-boundary
wait a timeout, an action on expiry, and a B id. This one has none.

The user-visible path is a slot insert or a bus add against a stopped engine, which is the ordinary
state on a machine with no audio server. Section 12.4's refusal table carries no row for
`PoolHandoffBusy`, so the message is the generic `Configuration` code of `UpstreamFailure`.

**EVIDENCE.** `architecture.md:3767` to `:3775`, `:4851` to `:4854` (the stopped states), `:817`
(B105), `:8494` to `:8496`, `:6745` to `:6750` (the refusal table), `:4359` to `:4370` (the timeout
table).

**WHAT SHOULD CHANGE.** State who retries, how many times, and what happens on the last attempt.
The simplest correct answer refuses the topology change when the engine is stopped, with a message
that names the engine state, and it needs no retry at all.

---

## 3. Concerns

1. **The recorded-result table is unchecked prose.** I changed PP5's recorded cell from `exit 1` to
   `exit 0` and every counter stayed at zero. DR5 calls this table the contract of both guards, and
   no rule reads a cell of it. This is how CR-3 recurred twice.
2. **The counter-family sentence omits PG26.** `architecture.md:676` says "The guard prints sixteen
   counter lines, one per claim family" and names fifteen families plus PG14. PG26 prints
   `AUDIO OWNED` and `HEAP IN AUDIO` and appears nowhere in the sentence.
3. **One count in section 1.9 is stale.** `architecture.md:1109` says "chunk M0 ports both guards
   and all thirty-five probes". The count is 39: 28 placement probes and 11 conversion probes.
4. **Two `Used by` cells name a section that never cites the id.** B61 and B86 each name 15.16, and
   section 15.16 cites neither. The column's own site calls it "a fact and not a claim". Thirteen of
   the fifteen stale cells are corrected.
5. **Section 10.2 states that two contract sentences still need an edit.** The Orchestrator applied
   both. Contract 1.11 now reads "Stateless: paints one `MeterReading` and a `held` flag" and
   contract 4.4 now reads "A numeric peak readout, painted by `MeterLayer`". The paragraph at
   `architecture.md:5810` is stale under DR1.
6. **`ModeView::body_split` splits nothing.** B95 is 0.62, the field is persisted, and no section
   and no contract row states what the fraction divides. Contract 1.6 gives the sidebar, the work
   area, and the inspector explicit widths, and it names the Mix vertical split separately.
7. **Substitution S2 states one give-up and has two.** Every field becomes `pub`, so the roster's
   public reachability set is larger than a real crate's. `missing_copy_implementations` reads
   reachability, so the roster can demand a derive that the real crate never needs. The effect is
   conservative, and the line does not say so.
8. **The conversion guard misses a `#[path]` module outside the target directory.** I put a cast in
   `m/extra/hot.rs`, reached by `#[path = "../extra/hot.rs"]`, and the run printed `FILES: 2`,
   `FINDINGS: 0`, exit 0. CG1's site states one limit, the members glob, and this is a second one.
   `clippy --all-targets` is the backstop.
9. **A raw identifier gives two false findings.** `let r#as = 1u32;` reports two casts on one line.
   CG3b prefers a false red to a miss, and the stated classes do not name a raw identifier.
10. **A crate outside the `members` glob is still unscanned.** I confirmed the stated limit K-10
    again: a member at `hidden/` with a cast exits 0 with `MEMBERS: 1`.
11. **The substitution block survives a damaged parse by accident.** My MY-R6 run printed
    `the block holds ['f32', 'str', 'u8']`, which is the primitive trait block. The identifier
    compare rejected it and the guard exited 2. The parse was already wrong.
12. **I could not trace "the ADR 0006 collision".** The Architect's revision-12 report names no ADR
    0006 finding and no collision. I checked ADR 0006 against the specification and found the
    decision list in order, every B id resolved, and every cross-citation matched section 10.5 and
    Appendix A. If a collision exists, the Orchestrator holds a source I do not.

---

## Reactive Assessment

- **Responsive: PARTIAL.** Section 5.12 gives every cross-boundary wait a timeout, a B id, and an
  action on expiry, and the audio thread allocates and frees nothing inside the cycle. The
  pool-handoff retry has no cap and no backoff, and its queue cannot drain in five stated engine
  states (WR-17).
- **Resilient: PARTIAL.** Every failure surface is a typed enum, no crate holds `unsafe`, the clip
  latch works in both directions, and the fault queue carries a drop counter. The one guard that
  holds TH1 reports success on three hostile inputs (CR-14), and `duet-session` cannot enforce two
  of its own budget refusals (CR-12).
- **Elastic: PARTIAL.** Every channel, ring, and queue is bounded, and each bound is a B row with a
  named producer and consumer. Three of those bounds are declared with a type whose interface cannot
  perform the stated overflow action (WR-11).
- **Msg Driven: PARTIAL.** Every thread seam is a push: `triple_buffer` for a latest value, `rtrb`
  and `ArrayQueue` for a stream, `async_channel` for structural traffic. Two views each need the one
  read end of one latest-value publication, so the meter seam and the transport seam carry no
  declared fan-out (CR-11).

---

## Verdict

**NOT READY.** Revision 12 is the strongest revision of the twelve. The decision to write all 44
bodies was correct and it paid at once: the compiler found four faults inside the hour, and it let
me find three more that no earlier review could reach. The clip latch, the audio-thread allocation,
the tempo-map owner, and the toolbar seam all close, and PG25 catches every real gate defect I
planted.

Four Criticals block. **The single biggest risk is that the guard set now looks complete and is
not.** PG26 is the only rule that holds the audio-thread contract, and I made it report success
three times with one line of markdown. CR-12 is a Cargo cycle that arrived through a constant, which
no rule reads. CR-11 is a run-time impossibility that compiles. Each of the three sits in the gap
between "the roster compiles" and "the system runs".

The weakest Reactive property is **Resilient**: the design contains every failure it names, and the
mechanism that proves the audio thread never allocates cannot be trusted to fail when it should.

### The blocking list

1. **CR-11** `MeterLayer` and `MasterMeterLayer` each hold a `MeterReader`, and `ComposeView` and
   `RecordView` each own a `PlayheadLayer` with a `TransportReader`. Each publication supplies one
   `Output`, which is neither `Clone` nor `Copy`.
2. **CR-12** `MixState::validate` must compare against B86 and B90. `MAX_STRIPS` sits in `duet-dsp`
   and `MAX_PARAMS` in `duet-engine`, and `duet-session` reaches neither. The `MAX_PARAMS` edge is a
   Cargo cycle.
3. **CR-13** PP25 shape 2 cannot be planted and shape 1's recorded message is not the message the
   run produces. PP24 and PP26 cite section 15.2 for a declaration in section 5.5.
4. **CR-14** PG26 exits 0 with `AUDIO OWNED: 88` when its block is deleted under a kept heading,
   exits 0 with one name in the block, and exits 0 when `SlotState` leaves the block and the
   revision-11 box returns. `ChainState` and `Transport` are outside its set.
5. **WR-11** Three queue ends are declared as `rtrb::Producer` where section 5.8 declares
   `crossbeam::queue::ArrayQueue`, and the declared type cannot keep the newest or drop the oldest.
6. **WR-12** Appendix B.1's `HotplugSubscription` expectation is a build error once its own `Drop`
   lands. A compiler run proves it.
7. **WR-13** `LevelMeter::scale` and `Knob::range` hold a `ParamRange`, which is a `ParamId` range,
   where design contract 4.4 and 4.5 need a value range.
8. **WR-14** The S5 substitution line states 33 blocks and 11 compiled. The document holds 39 and
   the roster compiles 15.
9. **WR-15** The `MeterSnapshot` doc comment states 1.6 kilobytes and the roster measures 1976
   bytes. All three guards pass on 9000 kilobytes.
10. **WR-16** PG12's name map reaches one row in silence and the run reports a clean dependency
    column.
11. **WR-17** `ConfigError::PoolHandoffBusy` names an unbounded retry against a queue that cannot
    drain while the engine is stopped.

### The Concerns, for the Orchestrator to file

| Id | One-line disposition |
|---|---|
| 1 | Add a rule that reads one recorded cell of section 1.9, or state at the DR5 site that the table is unchecked. |
| 2 | Correct `architecture.md:676` to seventeen families and name PG26. |
| 3 | Correct `architecture.md:1109` from thirty-five probes to 39. |
| 4 | Correct the `Used by` cells of B61 and B86, which name section 15.16. |
| 5 | Delete the stale paragraph at `architecture.md:5810`; the Orchestrator applied both contract edits. |
| 6 | State what `ModeView::body_split` divides, or delete the field and B95. |
| 7 | Add the reachability effect to substitution S2's give-up text. |
| 8 | State the `#[path]` limit beside CG1, or walk every module a target reaches. |
| 9 | Name the raw-identifier class in CG3b's stated false-red list. |
| 10 | Keep the stated K-10 limit; no action beyond the sentence that already carries it. |
| 11 | Anchor every data-block regular expression to the fence that follows its heading. |
| 12 | The Orchestrator states which ADR 0006 collision the brief names, or withdraws the item. |
