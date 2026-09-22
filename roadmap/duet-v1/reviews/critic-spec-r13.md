# Specification review, revision 13: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-21. Mode: specification review, before plan authoring.
This is the thirteenth pass. Revisions 1 to 12 each returned NOT READY.

## What I read and what I ran

Read in full: `roadmap/duet-v1/architecture.md` (10485 lines), the six records under
`roadmap/duet-v1/adr/`, all six files under `roadmap/duet-v1/tools/`,
`research/linux-macos-platform.md`, `research/gpui-kit-audit.md`, `product-requirements.md`
section 8, `design-contract.md` sections 1, 1.6, 1.11, 3.1, 3.3, 4.4 and the `StageCurve` section,
`CLAUDE.md`, `Cargo.toml`, `.cargo/config.toml`, `clippy.toml`, `deny.toml`, `scripts/dod.sh`, and
`crates/duet/src/`. I also read the Architect's revision-13 report from the plan store.

I read the pinned framework source for the two claims that rest on it:
`~/.cargo/registry/src/index.crates.io-*/gpui-pre-0.3.5/src/window.rs` and `.../src/view.rs`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/`. Every `cargo` and `python3` run happened inside that copy or inside the
scratchpad. I wrote no repository file and I ran no other git command.

**All three guards reproduce the recorded baseline, line for line.** The placement guard prints the
29-line block of section 1.9 exactly. The conversion guard prints `MEMBERS: 3   FILES: 6
FINDINGS: 0`. The roster compile exits 0 in 112 seconds with `ROSTER ITEMS: 382`,
`ROSTER IMPL BLOCKS: 39`, `ROSTER IMPLS: 15`, `ROSTER SIZES: 406 measured`, and
`RECORDED ROWS: 37 checked     HEAD ROWS: 12     SIZE BAD: 0`.

**All three probe harnesses reproduce every recorded result.** `probe_run.py` prints
`PROBES BAD: 0` over 44 probe shapes plus 116 PP27 shapes. `probe_conversion.py` prints
`CONVERSION PROBES BAD: 0`. `probe_roster.sh` prints `ROSTER PROBES BAD: 0`, and both PP25 shapes
and all three planted gate defects emit the exact compiler lines the document records.

**I added 16 probes of my own and 4 roster runs of my own.** Ten against the placement guard, four
more against it with a paired control, thirteen against the conversion guard, and four against the
roster compile. Three of the roster runs are gate-defect plants of my own at a site the Architect
did not use. Section 4 lists every result.

**Revision 13 is the largest advance of the thirteen.** DR7 and PG27 remove the fail-open class
completely. The three harnesses remove the false-record class completely. Three of the four
Criticals and six of the seven Warnings close.

It does not close. **Two Criticals block it**, and both sit in the same place: the audio thread and
the thread that drives it. One is a wait the revision added. One is a hole the exemption block
hides, and my own probe reproduces it.

---

## 1. Closure check

### 1.1 The four Criticals

| Id | Finding | State | Section and reason |
|---|---|---|---|
| CR-11 | Two views each need the one read end of one publication | **CLOSED** | 10.2 gives the frame order in three steps. `CoreHost` holds both readers. 15.16 gives each frame driver an `Entity<CoreHost>`. 5.8, 15.14, and Appendix A all agree. Test 12 asserts it. |
| CR-12 | `duet-session` must enforce two budgets it cannot reach | **CLOSED** | The four limits sit in `duet-time::limits`. The shared-limit block names every enforcer. My run of PP28 reds with `LIMIT BAD: 1`. |
| CR-13 | PP25's recorded result is false in both shapes | **CLOSED** | `probe_roster.sh` is the run. My run of PP25-eq and PP25-drop emits both recorded compiler lines exactly. |
| CR-14 | PG26 fails open, and its denominator is a hand list with two holes | **PARTIAL** | The fail-open half is closed. The denominator half is not. See WR-18 and CR-16. |

**CR-14, stated precisely.** DR7, PG27, and the block register close every shape the revision-12
review planted. I re-ran all 116 PP27 runs and every one exits 2 with a named block and a named
reason. `ChainState` and `Transport` entered the set, and `AUDIO OWNED` is 28.

The denominator half stays open in two directions. The exemption block still removes a whole
subtree from the rule, and my probe MINE-3 proves it. The candidate drop list still removes a type
from four rules, and my paired probe MINE-15 and MINE-16 proves it.

### 1.2 The seven Warnings

| Id | Finding | State | Section and reason |
|---|---|---|---|
| WR-11 | Three queue ends carry a type that cannot hold the stated rule | **CLOSED** | 5.8 names one primitive per boundary. `MidiSink.notes` and `HotplugSink.ports` are each an `Arc<ArrayQueue<..>>`. 1.2 moves `crossbeam-queue` to `duet-core` and `duet-midi`. The roster compiles. |
| WR-12 | The `HotplugSubscription` expectation does not build | **CLOSED** | The B.1 row is gone. The Drop block names three types. PP21e reds. My PP25-drop run emits `type could implement Copy`. |
| WR-13 | Two elements hold an identifier range where the contract needs a value range | **CLOSED** | `ParamIdRange` and `ParamRange` are two types. `LevelMeter` holds no scale. B106 and `duet_dsp::meter_law::db_to_fraction` carry the scale. |
| WR-14 | The substitution block states two impl counts and both are false | **CLOSED** | The S5 line states no number. PG25 refuses a digit in that block. My run prints 39 and 15. |
| WR-15 | A stale size inside a doc comment passes all three guards | **CLOSED** | PG21 reads every `///` and `//!` line. PP21d reds at line 4499. The recorded-size block gained `MidiRecord` and `NoteEntry`. |
| WR-16 | PG12's name map reaches one row in silence | **CLOSED** | `name-map` is a registered block at `rows>=16`. All four PP27 shapes exit 2 in my run. |
| WR-17 | The pool-handoff retry has no bound | **PARTIAL** | B95 bounds the attempt count, and the stopped-engine case refuses at once. The wait itself now runs on the user-interface thread. See CR-15. |

### 1.3 The twelve Concerns

Ten close. Two are partial, and each one is partial for a reason the document states at its own
site.

| Id | State | Evidence |
|---|---|---|
| 1 | **PARTIAL** | PG29 checks the exit code and the rule-to-probe pairing. My PP29 runs red. The recorded text stays unchecked, and my MINE-5 run exits 0 on a rewritten message. The PG29 site states that limit. |
| 2 | CLOSED | Lines 707 to 717 name PG26, PG27, PG28, and PG29 beside every other family. |
| 3 | CLOSED | The probe count is gone. The harness table replaces it. `PROBE ROWS: 44`. |
| 4 | **PARTIAL** | B61 and B86 no longer name 15.16. The class returns in 14 other cells. See concern C-1. |
| 5 | CLOSED | The stale paragraph at 5810 is gone. |
| 6 | CLOSED | `ModeView` holds seven fields. `body_split` and `FIRST_OPEN_BODY_SPLIT` are gone. B95 is reassigned. |
| 7 | CLOSED | The S2 line states the reachability effect and names it conservative. |
| 8 | CLOSED | CG1 states the `#[path]` limit and names clippy as the backstop. My MINE-C7 run reds an in-tree `#[path]` module. |
| 9 | CLOSED | CG3b names the raw identifier. My MINE-C13 run reproduces the two false reds on one line. |
| 10 | CLOSED | No action was asked. The K-10 sentence stands. |
| 11 | CLOSED | DR7 and `read_block` anchor every block on its marker. 116 shapes exit 2. |
| 12 | CLOSED | The Orchestrator withdrew the item. |

### 1.4 The mechanical passes

I re-ran every pass of the revision-12 review and added three.

1. **Budget ids.** 106 rows, B1 to B106. No gap. No duplicate. Every citation resolves. No orphan.
   The forward half of the `Used by` claim is clean: every section a cell names does cite the id.
   The reverse half fails in 14 cells. See concern C-1.
2. **Rule ids.** Every family is complete: DR1 to DR7, PL1 to PL3, VR1 to VR6, TH1 to TH10, SM0 to
   SM7, 32 PG rules against 32 PP probes, and 11 CG rules against 11 CP probes. DR5 holds, and
   PG29 now proves it on every run.
3. **Chunk coverage.** 55 chunks. Every chunk sits in exactly one phase. All 14 width cells agree
   with their row.
4. **The plan graph.** 13.4 expands to 49 ordered pairs. No link runs backward. Every same-phase
   pair is the `M<phase>` shape that SM1 permits.
5. **Write scopes.** `Cargo.lock` is the one shared path, across 75 same-phase pairs. SM5 rule 4
   governs it. No other path collides in any phase.
6. **MUST stories.** 64 MUST rows in the requirements and 64 rows in 13.2. The symmetric
   difference is empty.
7. **ADR citations.** Every section number, every B id, and every type name resolves. No ADR holds
   a code fence, a field list, or a derive list. One sentence states a number. See concern C-5.
8. **Design contract.** Every number in contract 1.1, 1.6, and 4.4 matches its B row. Contract 3.1
   and 3.3 match the declarations. Two types the contract names have no home. See WR-20.
9. **Platform research.** No contradiction against section 11. Minimum releases, the audio server,
   the runner names, the build prerequisites, and the `pipewire` version all agree.
10. **Guard blocks.** 29 blocks. Every actual row count equals its stated minimum. The `rows>=`
    form is tight downward and open upward. See WR-18.

---

## 2. New findings in revision 13

### CRITICAL: CR-15. The pool-handoff retry waits on the GPUI foreground thread

**OBSERVATION.** B95 is the bound that closes WR-17:

```
architecture.md:874  | B95 | 3 attempts, 50 ms apart | The pool-handoff retry the core runs on the
                       core thread; after the third refusal `configure` returns
                       `ConfigError::PoolHandoffBusy` to its caller (section 5.5).
```

Section 5.5 step 3 states who runs it:

```
architecture.md:4028  **The retry is bounded, and it is the core's own** (critic WR-17). `DuetCore`
architecture.md:4029  retries the push B95 times on the core thread, and it returns
                      `ConfigError::PoolHandoffBusy` to its caller after the last attempt.
```

Section 5.7 states where the core thread is:

```
architecture.md:4369  In the application, the core runs on the GPUI foreground thread inside one
                      entity.
```

Section 9.1 declares the one entry point, and it is synchronous:

```rust
architecture.md:4914  fn call(&mut self, verb: Verb) -> Result<VerbOutcome, GatewayError>;
```

**CLAIM.** A topology change can block the GPUI foreground thread for 100 ms. That is 25 times B5
and 6 times B2. Every window on the executor stops for that time.

**ARGUMENT.** Three attempts 50 ms apart hold two waits, so the worst case is 100 ms.
`Gateway::call` returns a `Result` and takes no async context, so the wait inside it cannot be an
executor yield. It must be a blocking sleep on the calling thread. In the application that thread
is the GPUI foreground thread, which paints every window.

B5 gives an immediate verb 4.0 ms from the listener to changed core state. `MixAddBus` and
`MixInsertSlot` edit the mix document and return a value, so neither can be a deferred verb that
answers `VerbOutcome::Started { job }`. A user who adds a reverb bus therefore pays a wait that is
25 times its own budget.

**The wait is also undeclared.** Appendix B.5 holds the table of every timeout and states its own
completeness rule:

```
architecture.md:9955  Each row of section 5.12 resolves to a mechanism here.
```

Section 5.12 carries a B95 row. Appendix B.5 carries no B95 row. The one cross-boundary wait that
revision 13 added is the one wait whose mechanism no section names.

**Two sentences about one bound disagree.** Section 5.12 writes "B95 attempts, not a clock"
(`architecture.md:4706`). The B95 row writes "3 attempts, 50 ms apart". DR1 asks for one sentence
per decision, and these two contradict each other on whether a clock is involved.

**No guard sees this.** PG25 compiles a declaration and never a body. No rule reads a thread
affinity. TH1 binds the audio thread alone, and TH3 bans a git walk, a file scan, an import parse,
and a render on the foreground thread; a sleep is none of the four.

**EVIDENCE.** `architecture.md:874` (B95), `:4028` to `:4034` (the retry), `:4369` (the core
thread), `:4706` (the 5.12 row), `:4914` (`Gateway::call`), `:784` (B5), `:781` (B2), `:8924` to
`:8930` (`ConfigError::PoolHandoffBusy`), `:9952` to `:9970` (the B.5 table with no B95 row).

**WHAT SHOULD CHANGE.** Delete the wait. B105 is 2 and an audio cycle drains the ring every B7,
which is 2.67 ms, so a wait buys almost nothing and costs 100 ms in the worst case. Refuse at once
with `ConfigError::PoolHandoffBusy`, which section 12.4 already gives a message. If the design
keeps a wait, move the retry off the foreground thread and make the verb deferred. Then add the
B95 row to Appendix B.5 and delete "not a clock" from section 5.12.

### CRITICAL: CR-16. No thread owns the resize of `GraphState::chains`, and the exemption block hides it

**OBSERVATION.** The audio thread owns one array of chain state, and each element owns three ring
ends:

```rust
architecture.md:4234  pub struct GraphState {
architecture.md:4235      chains: Box<[ChainState]>,
...
architecture.md:3892  pub struct ChainState {
architecture.md:3894      writer: DiskWriter,
architecture.md:3895      reader: DiskReader,
...
architecture.md:8860  pub struct DiskReader {
architecture.md:8864      samples: Consumer<f32>,
architecture.md:8866      requests: Producer<RefillRequest>,
architecture.md:8872  pub struct DiskWriter {
architecture.md:8877      samples: Producer<f32>,
```

Section 1.9 states that each ring end owns a heap allocation:

```
architecture.md:1327  | `Producer`, `Consumer` | `not Copy` | none | The two ends of an `rtrb` ring.
                        Each one owns a **heap** allocation that `configure` made
```

Section 5.5 states that a ring dies when a track leaves:

```
architecture.md:4057  **Rings are allocated when a track enters the playable set, and released when
                      it leaves.** The playable set changes when a track is added, removed, hidden,
                      shown, or armed.
```

**CLAIM.** The document never states who changes the length of `GraphState::chains`, and it never
states who drops a `ChainState`. The only owner it names is the audio thread, and a drop there is
a free that TH1 forbids.

**ARGUMENT.** The four migration rules of section 5.5 all act inside one chain. Rule 1 keeps a slot
state, rule 2 resets one, rule 3 clears one, and rule 4 takes a default. None of the four adds a
chain or removes one.

Section 5.6 states the whole cycle-top contract: "A topology change runs the in-place migration of
section 5.5. A pool change pops the handoff queue and swaps the pool" (`architecture.md:4256`).
The B105 ring carries a `PoolHandle` and nothing else. No mechanism hands a new `Box<[ChainState]>`
to the audio thread.

`GraphChain::chains` is a `Box<[ChainTopology]>` that the core publishes, and its length changes
with the playable set. `GraphState::chains` must then hold the matching count. If the published
count is larger, the runner has no state for the new index. If it is smaller, the surplus
`ChainState` must die.

Appendix A closes the last door:

```
architecture.md:9746  | The one `BufferPool` and every `ChainState` | The audio thread, inside
                        `GraphState` | ... | Nothing; no other thread holds a reference |
```

No other thread holds a reference, so only the audio thread can perform the swap. That swap drops
the old `Box<[ChainState]>`, which drops every `DiskReader` and every `DiskWriter`, which frees
every `rtrb` allocation. TH1 forbids a free on that thread, and the pool path shows the design
knows it: `GraphState::pool` goes through `basedrop` for exactly this reason.

**PG26 cannot see it, and I proved that.** The exemption block names `ChainState.writer` and
`ChainState.reader`, so the walk stops before it reaches either ring end. My probe MINE-3 added
`probe_heap: Vec<u8>` to `DiskReader` and ran the guard:

```
$ python3 placement_check.py <copy>/architecture.md
AUDIO OWNED:     28     AUDIO EXEMPT: 5     HEAP IN AUDIO: 0     DROP IMPLS: 3
EXIT=0
```

`DiskReader` and `DiskWriter` are not in the audio-owned block, so the exemption hides every field
below them. The block's own prose claims the opposite: "The audio thread pops, pushes, reads, and
writes those; it never grows one, shrinks one, or drops one" (`architecture.md:4350`). That is a
claim about a path the document never writes.

This is CR-2 one level up. Revision 11 put one allocation and one free inside the cycle by a boxed
enum arm. Revision 13 leaves a free inside the cycle by an array whose owner nobody names, and the
exemption that answers CR-14 is the mechanism that hides it.

**EVIDENCE.** `architecture.md:3892` to `:3907`, `:4057`, `:4083` to `:4103` (the four migration
rules), `:4234` to `:4244`, `:4256`, `:4338` to `:4353` (the exemption block and its prose),
`:4374` (TH1), `:8854` to `:8879`, `:1327`, `:9746`; my run of MINE-3 above.

**WHAT SHOULD CHANGE.** State the mechanism that changes the chain set, in the shape the pool
already has. The smallest correct answer builds the new `Box<[ChainState]>` off the audio thread,
hands it over a bounded ring beside B105, and wraps the retired one in `basedrop` so the collector
frees it on a job thread. Then add `DiskReader` and `DiskWriter` to the audio-owned block and state
at the exemption site that an exemption answers the allocation half and never the free half.

### WARNING: WR-18. Every registered block is closed downward and open upward, and three are allow lists

**OBSERVATION.** All 29 registered blocks sit exactly at their stated minimum. I measured every
one. The delta is zero in 29 cases out of 29.

A `rows>=` test refuses a deletion and accepts an addition. Three of the blocks grant coverage by
row, so an added row removes coverage.

**CLAIM.** DR7 closes the deletion class completely and leaves the addition class open. One added
word in a data block takes a type out of four rules, and one added line takes a subtree out of
PG26.

**ARGUMENT.** I ran three paired probes, each one with its own control.

| Probe | Shape | Result |
|---|---|---|
| MINE-15 | A declared `struct ProbeUnplaced` with no section 1.5 row | `UNPLACED: 1`, exit 1 |
| MINE-16 | The same struct, plus `ProbeUnplaced` added to the candidate drop list | `UNPLACED: 0`, `DROP LIST: 77`, exit 0 |
| MINE-3 | `probe_heap: Vec<u8>` inside `DiskReader`, under the exempt `ChainState.reader` | `HEAP IN AUDIO: 0`, exit 0 |
| MINE-8 | A new external-verdict row that names the placed type `ChainState` | `EXTERNAL ROWS: 46`, exit 0 |
| MINE-1 | `MeterSnapshot` added to the candidate drop list | `CANDIDATE TYPES: 392`, exit 0 |

MINE-16 is the sharp one. The same document, with one word added to a data block, changes from a
red run to a green run. PG4, PG4b, PG5, and PG8 all lose the name. The only trace is `DROP LIST`,
which rises from 76 to 77, and `CANDIDATE TYPES`, which falls from 393 to 392. Both counters live
in section 1.9 as prose, and no rule compares either one with a recorded value.

MINE-8 contradicts the external table's own site: "A name that section 1.5 places is never read
here" (`architecture.md:1338`). Nothing holds that sentence.

The PG4 site claims the opposite property for the drop list: "the guard carries no hidden set"
(`architecture.md:464`). Making the set data was correct. Data with no integrity rule is a second
hidden set with a public spelling.

Two of my probes did **not** find a hole, and I state both. MINE-12 added an exemption line for
`ChainState.pre_fader` and planted `Vec<u8>` inside `EqualizerState`; the guard still exits 1,
because `EqualizerState` is a root in its own right. MINE-14 added `MeterSnapshot` to the drop
list and removed its `Copy` derive; PG13 still reds, because it reads the snapshot table and not
the candidate set. The rule that "each name is a root in its own right" works where it applies.

**EVIDENCE.** The 29 measured row counts against the 29 stated minima; `architecture.md:464`,
`:475` to `:484` (the drop list), `:1338`, `:4338` to `:4345` (the exemption block); my runs of
MINE-1, MINE-3, MINE-8, MINE-12, MINE-14, MINE-15, and MINE-16.

**WHAT SHOULD CHANGE.** Change `rows>=` to an exact count for every block, so an addition and a
deletion both trip PG27. Then add one rule per allow list. A drop-list name is not a section 1.5
name, and the document already states that `Duration` is the one overlap. An external-verdict name
is not a section 1.5 name, and the site already states it. An exemption line names a `Type.field`
whose type the audio-owned block also carries, so an exemption can never remove a subtree from the
set.

### WARNING: WR-19. The frame-driver decision rests on a framework behaviour the pinned source contradicts

**OBSERVATION.** Section 10.2 states the saving that the frame-driver decision buys:

```
architecture.md:6192  `MeterLayer` satisfies contract 4.4 ... **without** the cost the contract
architecture.md:6193  implies. The audit says `request_animation_frame` marks the whole view dirty
architecture.md:6194  (audit 3), so a mixer-view frame would relayout every strip sixty times a
                      second. `MeterLayer` is a sibling leaf that spans the strip row ...
```

**CLAIM.** The saving does not exist on the pinned framework. A frame that `MeterLayer` requests
re-renders `DuetApp` and every one of its children, including `MixView` and its 48 `StripView`
entities.

**ARGUMENT.** I read the pinned source. Three facts decide it.

1. `Window::request_animation_frame` notifies the current view:
   `gpui-pre-0.3.5/src/window.rs:2606`, `let entity = self.current_view(); self.on_next_frame(move |_, cx| cx.notify(entity));`
2. A notify marks every ancestor dirty, not only the view:
   `gpui-pre-0.3.5/src/window.rs:2140`, `fn mark_view_dirty(&mut self, view_id: EntityId)` walks
   `view_path_reversed(view_id)` and inserts each ancestor.
3. A view re-renders unless it opts into a cache. `ViewElement::request_layout`
   (`gpui-pre-0.3.5/src/view.rs:314`) matches on `self.cached_style`; with `None` it calls
   `self.view.render(window, cx)` every time. Only `Entity::cached(style)`
   (`gpui-pre-0.3.5/src/view.rs:232`) sets that field. The architecture names `cached` nowhere.

So the whole element tree re-renders on every meter frame. The design pays the cost it says it
avoids, and it pays it in Compose and Record too, where `PlayheadLayer` drives the same loop over
`ComposeView` and every `SystemView`.

**The same mechanism is why the CR-11 fix works.** `DuetApp::render` runs every frame exactly
because the notify climbs to it. Section 10.2 declines to say so: "The order is stated here and not
inferred from the framework" (`architecture.md:6211`). The order inside `render` is indeed the
author's own. The premise that `render` runs at all on a meter frame is an inference, and it is the
premise the whole frame order rests on.

**Nothing measures it.** Section 5.12 benches B3 and B4 on `duet-engrave` alone. No bench covers
the Mix strip row. Test 7 counts animation-frame requests and test 12 asserts the frame order;
neither one asserts a cost.

**EVIDENCE.** `architecture.md:6180` to `:6201`, `:6203` to `:6232`, `:783` (B4), `:4673` to
`:4677` (what measures B3 and B4), `:6810` (test 7), `:6828` (test 12);
`gpui-pre-0.3.5/src/window.rs:2586`, `:2606`, `:2140`; `gpui-pre-0.3.5/src/view.rs:232`, `:271`,
`:314`, `:386`.

**WHAT SHOULD CHANGE.** State the re-render rule from the source, with the three line numbers, and
state that the frame order depends on it. Then choose. Either give the heavy subtrees
`Entity::cached` and state which ones and at what style, or delete the cost claim and put B4 on a
bench over the Mix strip row at B1.

### WARNING: WR-20. The design contract mandates a token layer that the type roster does not declare

**OBSERVATION.** Design contract section 7.1 states the whole colour mechanism:

```
design-contract.md:1113  - Duet publishes a `DuetTokens` struct as a GPUI `Global`.
design-contract.md:1114  - Duet resolves `DuetTokens` once at start and again on every change of
design-contract.md:1115    `cx.theme().mode`.
design-contract.md:1116  - A view reads `cx.global::<DuetTokens>().staff_ink`. No view holds an
                           `Hsla` literal.
```

`architecture.md` names `DuetTokens` zero times. Section 1.5 places no token type. Section 15.16
declares none. No chunk's `Writes` cell names one.

**CLAIM.** Section 15 calls itself the complete type roster and omits the type that every painted
colour in the product passes through.

**ARGUMENT.** PG4 and PG4b hold the section 1.5 table and the Rust blocks to one set, in both
directions. No rule reads the design contract, so a type the contract mandates and the
architecture omits is outside every guard. The guard set cannot find this class at all.

Two chunks need the type. K1 writes the shell and K4 writes `LevelMeter`, `Fader`, `Knob`,
`AutomationLane`, and `StageCurve`. Each one paints, and design contract 7.1 forbids a colour
literal in a view. Neither chunk has a declared type to read a token from. An implementer who
declares one makes PG4 red until the document gains a row.

The same shape appears once more, smaller. Design contract line 413 names a `HeldDuration` field.
The architecture declares `DurationSelector` and `DurationKeySet` at section 10.6 instead. PL3
gives one name to one type, and two documents now name one thing twice.

**EVIDENCE.** `design-contract.md:1105` to `:1120`, `:1176`, `:413`; `architecture.md:389` (the
`crates/duet` row), `:7916` (the section 15 preamble), `:9210` to `:9726` (section 15.16),
`:7454` (chunk K4), `:7458` (the K1 shell files); a search of `architecture.md` for `DuetTokens`
and `HeldDuration`, which returns zero.

**WHAT SHOULD CHANGE.** Declare the token type in `crates/duet`, give it a section 1.5 row and a
section 15.16 block, and name the chunk that writes it and the chunk that resolves it on a theme
change. Then rename `HeldDuration` to the architecture's own name in the contract, or the other
way, so PL3 holds across both documents.

---

## 3. Concerns

**C-1. Fourteen `Used by` cells omit a section that cites the id.** The column's own site calls it
"a fact and not a claim" (`architecture.md:773`). The forward half is clean. The reverse half fails
for B8 at 10.2, B31 at B.3, B51 at 1.9, B61 at C.13, B86 at Appendix A and C.13, B87 at B.3, B90 at
15.10, B95 at C.10 and C.12, B100 at 15.14 and B.3, B105 at 5.12, and B106 at C.13. Revision 11
carried 15 such cells and revision 12 carried 2. A prose fix has now failed three times, so the
rule is the only durable answer.

**C-2. The recorded text of a probe cell stays unchecked.** My MINE-5 run rewrote a recorded
message to text no run emits and kept the exit code; every counter stayed at zero. The PG29 site
states the limit, and the three harnesses are the answer. Chunk M0 ports each cell to a test.

**C-3. The roster compile has no denominator floor.** My MYR-denominator run deleted the `RenderJob`
declaration and the roster exited 0 with `ROSTER ITEMS: 381`. The placement guard is the backstop:
the same copy gives `UNDECLARED: 1` and exits 1. Both run in the `plan-lint` job, so the pair
covers it. A recorded item count would make each guard non-vacuous on its own.

**C-4. PG28 cannot fire while every limit owner is the root crate.** `duet-time` is the root of the
section 1.3 graph, so every crate reaches it and the reachability test always passes. My MINE-6 run
shows the rule works when a row names a wrong owner, and PP28 shows it works when the owner moves.
The rule has future value and proves nothing about the current text. The enforcer list is a hand
list, which the site states.

**C-5. The roster does not model the `limits` module.** The generated `duet-time/src/lib.rs` holds
`pub const MAX_STRIPS: usize = 48;` at the crate root, and `duet-dsp` imports
`use duet_time::MAX_STRIPS;`. The document writes `duet-time::limits` in B45, B46, B86, B90, and
the constant block. PG25 therefore proves the crate and never the module path.

**C-6. The frame-order call and the declared signature do not agree.** Section 10.2 step 1 writes
`self.core.update(cx, CoreHost::poll_frame)`. Section 15.16 declares
`pub(crate) fn poll_frame(&mut self);`. `Entity::update` takes
`impl FnOnce(&mut T, &mut Context<T>) -> R` (`gpui-pre-0.3.5/src/app/entity_map.rs:476`), so the
closure needs two arguments and the method has one. Chunk K1 writes this call and test 12 asserts
it.

**C-7. `TransportReader` has no value for its slot before the first poll.** `MeterView` carries
`silent()` for that reason (`architecture.md:5171`). `TransportView` carries no named constructor
and can derive no `Default`, because `SuperClock` has none. `TransportSnapshot` has the same shape,
and `triple_buffer` needs an initial value at construction.

**C-8. Section 14 invokes `check-roster` with one argument.** Section 1.9 states three: "it takes
the document, the scratch directory, and the repository root as three arguments; it refuses a
scratch directory inside the repository" (`architecture.md:1093`). The `plan-lint` row writes
`cargo xtask check-roster roadmap/duet-v1/architecture.md` (`architecture.md:7756`). The runner
checkout is the repository, so a defaulted scratch directory would be refused.

**C-9. Line 441 states twenty-eight placement rules and the real count is 32.** The same paragraph
counts twenty-four in revision 10 plus three in revision 11 plus PG26, and stops before PG27, PG28,
and PG29. `PROBE ROWS: 44` is the measured number.

**C-10. Two revision counts do not match their own lists.** Line 403 states "Five placements
changed in revision 11" above two items. Line 410 states "Eleven placements changed in revision 10"
above ten items.

**C-11. One sentence of claim 2 in section 13.3 is false.** Line 7615 states "SM1 keeps both in two
phases, so neither is a collision". Every manifest chunk shares its phase with the line chunks that
edit the skeletons it creates: M0 with T1, M1 with T2, T3, and D1, M2 with five, M3 with three, M6
with two, and M7 with J1. The real mechanism is the serial order inside one phase, which SM1
states.

**C-12. ADR 0004 restates a number.** Line 209 reads "macOS 26.0 is the minimum, and specification
section 11.4 names the two places that state it". DR6 asks for the citation alone.

**C-13. A marker outside its own heading is invisible.** My MINE-9 run added a second
`<!-- GUARD BLOCK id=audio-owned rows>=28 -->` line under section 5.12 and the guard exited 0.
`read_block` searches inside the heading's region, so the marker set is not unique across the
document.

**C-14. A marker with an unregistered id is invisible.** My MINE-10 run added a new `####` block
with `id=probe-block` and the guard exited 0. PG27 holds the register and the rule-to-block map to
one set, and it reads no marker the register does not name.

**C-15. B101 sits after B106 in section 1.6.** Every other row is in order. The order is cosmetic
and the id space has no gap.

---

## 4. Every probe I ran, and its result

### 4.1 The three guards on the unmodified document

| Guard | Result |
|---|---|
| `placement_check.py architecture.md` | Exit 0. Every one of the 29 counter lines matches section 1.9, character for character. |
| `conversion_check.py`, from the repository root | Exit 0. `MEMBERS: 3   FILES: 6   FINDINGS: 0`. |
| `roster_compile.sh architecture.md <scratch> <repo>` | Exit 0 in 112 seconds. `ROSTER CRATES: 17`, `ROSTER ITEMS: 382`, `ROSTER IMPL BLOCKS: 39`, `ROSTER IMPLS: 15`, `ROSTER DROPS: 3`, `ROSTER CONSTS: 6`, `ROSTER SUBS: 8`, `ROSTER CLIPPY: clean`, `ROSTER SIZES: 406 measured`, `RECORDED ROWS: 37 checked     HEAD ROWS: 12     SIZE BAD: 0`. |

### 4.2 The three harnesses

| Harness | Result |
|---|---|
| `probe_run.py ../architecture.md` | `PROBES BAD: 0`. All 44 rows and all 116 PP27 shapes reproduce their recorded exit code and their recorded line. |
| `probe_conversion.py` | `CONVERSION PROBES BAD: 0`. All 22 shapes reproduce. |
| `probe_roster.sh` | `ROSTER PROBES BAD: 0`. PP25-eq, PP25-drop, GATE-allow, GATE-unwrap, and GATE-as each emit the exact recorded compiler line. |

**No harness passes a probe the guard did not run.** I checked each one. `probe_run.py` writes the
mutated document to a temporary directory and calls `placement_check.py` as a subprocess, then
compares the real exit code. `probe_conversion.py` builds a throwaway cargo workspace per shape and
runs the guard inside it. `probe_roster.sh` calls `roster_compile.sh` and greps the real output. A
plant that does not apply fails loudly: the plant asserts `text.count(old) == 1` and the harness
prints `PLANT FAILED`. I confirmed that path by breaking one anchor on purpose.

### 4.3 My own placement probes

| Probe | Shape | Result | Verdict |
|---|---|---|---|
| MINE-1 | `MeterSnapshot` added to the candidate drop list | exit 0 | Hole |
| MINE-3 | `probe_heap: Vec<u8>` inside `DiskReader` | exit 0, `HEAP IN AUDIO: 0` | **Hole, CR-16** |
| MINE-4 | `Vec<u8>` inside `PoolSlot`, an audio-owned name | exit 1, `HEAP IN AUDIO: 9` | Caught |
| MINE-5 | A recorded probe message rewritten, exit code kept | exit 0 | Stated limit |
| MINE-6 | `MAX_SLOTS` replaced by a name no constant declares | exit 1, `LIMIT BAD: 1` | Caught |
| MINE-7 | `ModeView` added to the framework block | exit 1, `FRAMEWORK MISUSE: 1` | Caught |
| MINE-8 | An external-verdict row that names the placed type `ChainState` | exit 0, `EXTERNAL ROWS: 46` | Hole |
| MINE-9 | A duplicate `audio-owned` marker under section 5.12 | exit 0 | Hole |
| MINE-10 | A new `####` block with an unregistered marker id | exit 0 | Hole |
| MINE-11 | Control: `Vec<u8>` inside `EqualizerState` | exit 1, `HEAP IN AUDIO: 4` | Caught |
| MINE-12 | The same, plus a `ChainState.pre_fader` exemption line | exit 1, `HEAP IN AUDIO: 3` | Caught |
| MINE-13 | Control: the `Copy` derive removed from `MeterSnapshot` | exit 1, `SNAPSHOT BAD: 1` | Caught |
| MINE-14 | The same, plus `MeterSnapshot` on the drop list | exit 1, `SNAPSHOT BAD: 1` | Caught |
| MINE-15 | Control: a declared `struct ProbeUnplaced` with no 1.5 row | exit 1, `UNPLACED: 1` | Caught |
| MINE-16 | The same, plus `ProbeUnplaced` on the drop list | exit 0, `UNPLACED: 0` | **Hole** |

Each probe planted one violation in its own run. MINE-11 and MINE-12, MINE-13 and MINE-14, and
MINE-15 and MINE-16 are three pairs, and each pair holds a control that reds.

### 4.4 My own conversion probes

| Probe | Shape | Result |
|---|---|---|
| MINE-C1 | A cast in `tests/common/helper.rs`, a nested test module | exit 1, `FILES: 4` |
| MINE-C2 | A cast in `benches/sub/inner.rs` | exit 1, `FILES: 3` |
| MINE-C3 | A cast split across a line break | exit 1 |
| MINE-C4 | An inferred cast, `x as _` | exit 1 |
| MINE-C5 | A cast in `build.rs` | exit 1, `FILES: 2` |
| MINE-C6 | `#[expect(::clippy::as_conversions, ...)]` | exit 0 |
| MINE-C7 | A cast in `src/deep/x.rs` reached by `#[path]` | exit 1 |
| MINE-C8 | A cast in `examples/sub/helper.rs` | exit 1 |
| MINE-C9 | Control: a cast in `src/deep/mod.rs` | exit 1 |
| MINE-C10 | `#[expect(clippy :: as_conversions, ...)]`, spaces around `::` | exit 1 |
| MINE-C11 | A stray `src/stray.rs` under a member whose lib path is `code/lib.rs` | exit 0 |
| MINE-C12 | A cast after a nested raw string with a different hash count | exit 1 |
| MINE-C13 | The raw identifier `r#as` | exit 1, two findings on one line |

**MINE-C6 is not a hole.** I ran the form through the pinned toolchain and rustc refuses it:
`error[E0710]: unknown tool name `{{root}}` found in scoped lint`. The same run refuses a
reason-first attribute with `error[E0452]`. The spaced form of MINE-C10 does compile, and the guard
catches it. **MINE-C11 is not a hole either**: a file outside every target is dead source that
`cargo` never compiles.

The conversion guard is the strongest of the three. I found no miss in thirteen probes.

### 4.5 My own gate-defect plants, at a site the Architect did not use

Every plant sits inside the spelled-out body of `impl core::fmt::Debug for PoolHandle` in section
5.5. Each one ran alone.

| Plant | Result |
|---|---|
| `panic!` inside an `if` | exit 1; ``duet-engine/src/lib.rs:103:5: error: used `panic!()` or assertion in a function that returns `Result` `` and ``104:36: error: `panic` should not be present in production code`` |
| `println!` | exit 1; ``duet-engine/src/lib.rs:104:9: error: use of `println!` in `Debug` impl`` and ``104:9: error: use of `println!` `` |
| Slice indexing | exit 1; `duet-engine/src/lib.rs:105:22: error: indexing may panic` |
| Denominator: the `RenderJob` declaration deleted | exit 0, `ROSTER ITEMS: 381`; the placement guard on the same copy gives `UNDECLARED: RenderJob is placed by 1.5 and no Rust block declares it` and exits 1 |

PG25 catches every real gate defect I planted. The denominator run is concern C-3.

---

## Reactive Assessment

- **Responsive: FAIL.** Every other cross-boundary wait has a bound, an action on expiry, and a B
  id. The one wait revision 13 added blocks the GPUI foreground thread for up to 100 ms, which is
  25 times B5 (CR-15). The frame-driver decision also claims a per-frame saving the pinned
  framework does not supply (WR-19).
- **Resilient: PARTIAL.** Every failure surface is a typed enum. No crate holds `unsafe`. The
  fail-open class is gone: 116 hostile block shapes all exit 2 and none prints a counter. The
  audio thread still has one free with no stated owner, and the exemption block hides it (CR-16).
- **Elastic: PASS.** Every channel, ring, and queue is bounded. Each bound is a B row with a named
  producer, a named consumer, and an overflow action the declared type can perform. WR-11 closes
  the last mismatch, and the roster compiles every declared end.
- **Msg Driven: PASS.** Each publication has one read end, one owner, and a stated frame order.
  Each queue has one primitive and one eviction rule. No view holds an `Output`. No `Global` and no
  lock crosses a view seam.

---

## Verdict

**NOT READY.** Revision 13 is a large and honest advance. The three probe harnesses remove the
false-record class that three revisions failed on, and I reproduced every recorded number on this
machine. DR7 and PG27 remove the fail-open class: I ran all 116 hostile block shapes and every one
exits 2 with a named reason. Three Criticals, six Warnings, and ten Concerns close for real.

Two Criticals block, and both sit on the audio path. **The single biggest risk is that the guard
set now looks complete and the two holes it leaves are exactly where the design is weakest.** CR-16
is a free on the audio thread that the CR-14 exemption block hides, and my own probe put a
`Vec<u8>` behind that exemption and got a green run. CR-15 is a 100 ms sleep on the thread that
paints every window, added by the WR-17 fix, with no mechanism row in the table that claims to hold
every wait.

The weakest Reactive property is **Responsive**. The design bounds every wait it names and then
puts the newest one on the one thread that cannot afford it.

### The blocking list

1. **CR-15** The B95 retry runs on the core thread, which in the application is the GPUI foreground
   thread. `Gateway::call` is synchronous, so the two 50 ms gaps must be blocking sleeps. B5 is
   4.0 ms. Appendix B.5 carries no B95 row, and section 5.12 says "not a clock" while B95 says
   "50 ms apart".
2. **CR-16** No mechanism changes the length of `GraphState::chains`, and no thread but the audio
   thread can drop a `ChainState`. Each one owns three `rtrb` heap allocations. The exemption block
   hides the whole path, and my MINE-3 run exits 0 with `HEAP IN AUDIO: 0`.
3. **WR-18** Every registered block is tight downward and open upward. My MINE-16 run turns a red
   document green with one word added to the candidate drop list. MINE-3 and MINE-8 do the same
   through the exemption block and the external-verdict table.
4. **WR-19** Section 10.2 claims a frame driver avoids a whole-tree relayout. The pinned source
   marks every ancestor dirty and re-renders every uncached view. The same behaviour is why the
   frame order works, and section 10.2 says it infers nothing from the framework.
5. **WR-20** Design contract 7.1 mandates a `DuetTokens` global. `architecture.md` names it zero
   times, section 1.5 places no token type, and no chunk writes one. `HeldDuration` is the same
   shape at a smaller scale.

### The Concerns, for the Orchestrator to file

| Id | One-line disposition |
|---|---|
| C-1 | Correct the 14 `Used by` cells, and add a rule that reads every B citation against the column. |
| C-2 | Keep the stated PG29 limit; chunk M0 ports each recorded cell to a test. |
| C-3 | Give the roster compile a recorded item count, so it fails on a shrunken denominator. |
| C-4 | State at the PG28 site that the rule is vacuous while every owner is the root crate. |
| C-5 | Emit the `limits` module in the roster, or write `duet-time` and drop the module path. |
| C-6 | Correct `self.core.update(cx, CoreHost::poll_frame)` to a two-argument closure. |
| C-7 | Give `TransportView` and `TransportSnapshot` a named first value, as `MeterView::silent` has. |
| C-8 | Write the three `check-roster` arguments in the section 14 `plan-lint` row. |
| C-9 | Correct line 441 from twenty-eight rules to 32, or cite the run. |
| C-10 | Correct the counts at line 403 and line 410 against their own lists. |
| C-11 | Replace the "two phases" sentence at line 7615 with the SM1 serial order inside one phase. |
| C-12 | Replace the macOS version at ADR 0004 line 209 with the section citation alone. |
| C-13 | Make PG27 refuse a registered marker id that appears outside its own heading. |
| C-14 | Make PG27 refuse a `GUARD BLOCK` marker whose id the register does not hold. |
| C-15 | Move the B101 row above B102, or state that the row order is free. |

### Commentary, which is not a finding

`PlayheadLayer::shown` and `MeterLayer::shown` are described as the value that stops a repaint.
`render` must return an element on every call, so the field saves a comparison and no paint. State
what it saves, or delete it.

The three prototypes are the best engineering in this plan. `read_block` is 60 lines and it closes
a whole failure class. The paired-control discipline in `probe_run.py` is correct, and I recommend
that chunk M0 keep the pairing when it ports each probe to `tools/xtask/tests/probes.rs`.
