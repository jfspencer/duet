# Specification review, revision 14: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-21. Mode: specification review, before plan authoring.
This is the fourteenth pass. Revisions 1 to 13 each returned NOT READY.

## What I read and what I ran

Read in full: `roadmap/duet-v1/architecture.md` (10,976 lines), the six records under
`roadmap/duet-v1/adr/`, all six files under `roadmap/duet-v1/tools/`,
`research/linux-macos-platform.md`, `product-requirements.md` section 8, `design-contract.md`
sections 1, 1.6, 1.11, 3.1, 3.3, 4.4, 7, and the `StageCurve` section, `CLAUDE.md`, `Cargo.toml`,
`.cargo/config.toml`, `clippy.toml`, `deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`. I read
the thirteenth review and the Architect's revision-14 report from the plan store.

I read the pinned framework source for every claim section 10.2 rests on:
`gpui-pre-0.3.5/src/window.rs`, `src/view.rs`, and `src/app/entity_map.rs`. I also read
`gpui-component-0.6.4/src/theme/`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/`. Every `cargo` and `python3` run happened inside that copy or inside the
scratchpad. I wrote no repository file and I ran no other git command.

**All three guards reproduce the recorded baseline, line for line.** `placement_check.py` prints the
31-line block of section 1.9 exactly, with `MEMBER BAD: 0`, `USED BY BAD: 0`,
`AUDIO OWNED: 33`, `AUDIO EXEMPT: 1`, `AUDIO DEFERRED: 12`, and `HEAP IN AUDIO: 0`.
`conversion_check.py` prints `MEMBERS: 3   FILES: 6   FINDINGS: 0`. `roster_compile.sh` exits 0 with
`ROSTER ITEMS: 385     FLOOR: 385`, `ROSTER IMPL BLOCKS: 46`, `ROSTER IMPLS: 19`,
`ROSTER SIZES: 409 measured`, and `RECORDED ROWS: 37 checked     HEAD ROWS: 13     SIZE BAD: 0`.

**All three harnesses reproduce every recorded result.** `probe_run.py` prints `PROBES BAD: 0` over
45 probe rows and 155 PP27 shapes. `probe_conversion.py` prints `CONVERSION PROBES BAD: 0`.
`probe_roster.sh` prints `ROSTER PROBES BAD: 0`, and all three PP25 shapes and all three recorded
gate defects emit the exact compiler lines the document records.

**I added 26 probes of my own.** Thirteen against the placement guard, nine against the conversion
guard, and four against the roster compile. I also planted three gate defects of my own, at three
sites revision 14 wrote and no earlier revision used. Section 4 lists every result.

**Revision 14 closes every finding of revision 13.** CR-15, CR-16, WR-18, WR-19, WR-20, and
fourteen of the fifteen Concerns are CLOSED. My own probes reproduce the closure of MINE-1, MINE-3,
MINE-8, MINE-9, MINE-15, and MINE-16.

It does not close. **Five Criticals block it.** Four of the five come from the CR-16 answer itself:
migration rule 0 states a mechanism that the ownership model makes impossible, and the memory,
the thread, and the queue that carry it are each wrong. The fifth is a frame loop that revision 14
gave a stop condition and no start condition.

---

## 1. Closure check

### 1.1 The two Criticals

| Id | Finding | State | Section and reason |
|---|---|---|---|
| CR-15 | The pool-handoff retry waits on the GPUI foreground thread | **CLOSED** | B95 states an attempt count and no clock. 5.5 step 3 makes `configure` refuse at once. `CoreHost::handoff_retry` parks each wait on `cx.background_executor().timer`. TH3 names a timed wait. B.5 carries a B95 row. One stale doc comment survives; see WR-21. |
| CR-16 | No thread owns the resize of `GraphState::chains`, and the exemption block hides it | **PARTIAL** | The guard half is closed: my MINE-3 shape now reds, `AUDIO EXEMPT` is 1, and every ring end sits inside `basedrop::Owned`. The design half is not. Rule 0 states a mechanism that Appendix A forbids, and its memory, its thread, and its queue are each wrong. See CR-17, CR-18, CR-19, and CR-20. |

### 1.2 The three Warnings

| Id | Finding | State | Section and reason |
|---|---|---|---|
| WR-18 | Every registered block is closed downward and open upward | **CLOSED** | DR7 gains a membership half. `block-members` names one kind per block. My re-run of MINE-1, MINE-8, MINE-15, and MINE-16 reds all four, and MINE-9 and MINE-10 each exit 2. `MEMBER ROWS: 648`, `MEMBER BAD: 0`. |
| WR-19 | The frame-driver decision rests on a framework behaviour the pinned source contradicts | **PARTIAL** | The three facts are true. I read each one. The `Entity::cached` mitigation holds for `SystemView`, `LaneView`, and `StripView`, which are entities. It cannot hold for `StageCurve`, which derives `IntoElement`. See WR-26. |
| WR-20 | The design contract mandates a token layer that the roster does not declare | **PARTIAL** | `DuetTokens` declares 27 roles and three methods, which answers all 29 rows of contract 7.2 exactly. `HeldDuration` takes the contract's name. No entity holds the theme-change subscription that contract 7.1 rule 2 needs. See WR-28. |

### 1.3 The fifteen Concerns

Fourteen close. One is partial.

| Id | State | Evidence |
|---|---|---|
| C-1 | **CLOSED** | PG30 exists and it is not vacuous. My MINE-A9 and MINE-A10 red in both directions. `BUDGET ROWS: 106     USED BY BAD: 0`. The budget table is a registered block. |
| C-2 | CLOSED | Line 8211 states that chunk M0 ports the recorded text of every cell. |
| C-3 | CLOSED | `FLOOR: 385` sits beside `ROSTER ITEMS: 385`. PP25-floor reds in my run. The impl counts carry no floor; see concern N-3. |
| C-4 | CLOSED | The PG28 site states the vacuity. The `limit-row` kind gives every row a referent. |
| C-5 | CLOSED | A search for `duet-time::limits` returns zero. |
| C-6 | CLOSED | Step 1 of the frame order writes a two-argument closure and cites `entity_map.rs:476`. I read that line and the signature agrees. |
| C-7 | CLOSED | `TransportSnapshot::stopped` and `TransportView::stopped` are both declared. |
| C-8 | CLOSED | Both section 14 rows and the local command block write three arguments. |
| C-9 | CLOSED | The paragraph states no rule count and cites `PROBE ROWS`. |
| C-10 | CLOSED | "Three" above three items, "Two" above two, "Ten" above ten. |
| C-11 | CLOSED | Claim 2 names the SM1 serial order inside one phase and lists both file classes. |
| C-12 | **PARTIAL** | Clause 22 states that no version number appears. Clauses 19 and 20 of the same record write `ubuntu-26.04` and `macos-26`, and line 19 writes "Ubuntu 26.04, macOS 26". See concern N-4. |
| C-13 | CLOSED | My re-run of MINE-9 exits 2 with a duplicate-marker message. |
| C-14 | CLOSED | My re-run of MINE-10 exits 2 with an unregistered-marker message. |
| C-15 | CLOSED | B101 sits above B102. |

### 1.4 The mechanical passes

I re-ran every pass of the revision-13 review.

1. **Budget ids.** 106 rows, B1 to B106. No gap, no duplicate, no row out of order, no orphan.
   Both halves of the `Used by` claim are clean, and PG30 now measures them.
2. **Rule ids.** DR1 to DR7, PL1 to PL3, VR1 to VR6, TH1 to TH10, SM0 to SM7, 33 PG rules against
   33 PP probes, and 11 CG rules against 11 CP probes. Every pair is one to one. Section 1.7 states
   two stale ranges; see concern N-1.
3. **Chunk coverage.** 55 chunks. Every chunk sits in exactly one phase. All 14 width cells agree.
4. **The plan graph.** 77 ordered pairs, 76 of them unique. No link runs backward. Every same-phase
   pair carries the `M<phase>` form that SM1 permits. One pair is stated twice; see concern N-10.
5. **Write scopes.** `Cargo.lock` is the one shared path across 77 same-phase pairs, and SM5 rule 4
   governs it. The 16 skeleton-file pairs fall under the SM1 serial order, which 13.3 now states.
6. **MUST stories.** 64 rows in the requirements and 64 rows in 13.2. The symmetric difference is
   empty. The ten SHOULD ids agree as well.
7. **ADR citations.** Every section, every B id, and every type name resolves. No ADR holds a code
   fence, a field list, or a derive list. One record restates four platform facts; see N-4.
8. **Design contract.** All 29 token roles of contract 7.2 map onto `DuetTokens`. Three names the
   contract uses resolve to nothing in the architecture; see concern N-9.
9. **Platform research.** No contradiction against section 11. One version is written two ways
   inside the architecture; see concern N-8.
10. **Guard blocks.** 31 blocks. Every actual row count equals its stated minimum, and every row
    now names a referent.

---

## 2. New findings in revision 14

### CRITICAL: CR-17. Migration rule 0 needs state that only the audio thread holds

**OBSERVATION.** Rule 0 states how a new chain set is built:

```
architecture.md:4299  `configure` therefore builds a whole new `Box<[ChainState]>` off the audio
                      thread, with every kept chain's state copied across, with every ring of every
                      new chain already allocated, and with the `PoolSlot` handles already assigned.
```

Appendix A states who holds the old set:

```
architecture.md:10224  | The one `BufferPool` and every `ChainState` ... | The audio thread while a
                         cycle runs ... | Nothing; no other thread holds a reference while a cycle
                         runs |
```

Rule 1 states what the copy must preserve:

```
architecture.md:4310  1. A slot whose `SlotId` and `SlotKind` are unchanged keeps its state. A
                         filter does not reset when the user adds an unrelated send.
```

**CLAIM.** `configure` cannot copy a kept chain's state, because the only copy of that state lives
inside a value that the audio thread owns and that no other thread can reach.

**ARGUMENT.** A `ChainState` holds `pre_fader`, `post_fader`, `meter`, `trim`, and `fader`. Those
are live filter histories, meter holds, and gain ramps. The audio thread writes them once per cycle
and no publication carries them back. `GraphState::chains` is a `ChainSetHandle`, which wraps
`Owned<Box<[ChainState]>>`. The core holds no handle on it, and section 5.8 lists no path that
returns one.

A `ChainState` is also not copyable in the Rust sense. It holds a `DiskWriter` and a `DiskReader`,
each of which owns an `rtrb` end inside a `basedrop::Owned`. Those types derive no `Clone` and no
`Copy`, and section 1.9 records `Owned` as `not Copy` with no trait.

So one of two sentences is false. Either the new set carries default state, and rule 1 is false for
every chain on every topology change, or the core reaches the audio thread's state, and Appendix A
is false. Revision 14 states both.

**The `GraphHandoff` declaration makes the scope of the defect the whole system.**

```
architecture.md:4179  Every `configure` builds a whole new pool and a whole new chain-state set, so
                      one message carries both and one generation gates both.
```

Rule 0 says the set changes when a strip enters or leaves. The `GraphHandoff` comment says every
`configure` rebuilds it. Under the second sentence the in-place rules 1 to 4 have nothing to act
on, because a whole new set arrives on every change. Section 5.6 keeps two generations and two
paths, which only the first sentence supports.

**The user-visible result.** A user who adds one aux send to one strip resets the filter history,
the compressor envelope, and the meter hold of all 48 strips. Rule 1 exists to prevent exactly
that, and section 5.5 names it as the reason the in-place migration exists.

**EVIDENCE.** `architecture.md:4030` to `:4054` (`ChainState`), `:4169` (`ChainSetHandle`),
`:4179` to `:4188` (`GraphHandoff`), `:4297` to `:4327` (rule 0 and rules 1 to 4), `:4460` to
`:4473` (`GraphState`), `:4485` to `:4491` (the cycle top), `:9232` to `:9258` (`DiskReader` and
`DiskWriter`), `:10224` (Appendix A), `:1463` (the `Producer` and `Consumer` row).

**WHAT SHOULD CHANGE.** Choose one owner for the kept state and state it once. The smallest correct
answer keeps the audio thread as the owner of every `ChainState` and gives the handoff a plan
rather than a whole set: a `Box<[ChainSlot]>` in which each element is either "keep index k of the
old set" or "adopt this new `ChainState`". The audio thread then moves kept elements inside the
array it already owns, adopts the new elements, and drops the retired ones into `basedrop`. That
keeps rule 1 true, removes the copy that cannot happen, and also answers CR-18.

### CRITICAL: CR-18. Rule 0 doubles the ring set, and B55, B56, and B57 model only the pool

**OBSERVATION.** B56 states the transient peak and names one cause:

```
architecture.md:892  | B56 | 113 MB | The transient engine peak while two pools are live | 5.5 |
architecture.md:4279 The steady total is B55, and B56 is the transient peak while the old pool
                     waits for the collector.
```

B55 is 104 MB, B52 is one pool at 9.0 MB, and B54 is one stereo ring pair at 2.3 MB. B57 is the
ceiling that `ConfigError::RingBudgetExceeded` enforces, at 128 MB.

**CLAIM.** Rule 0 makes every ring live twice during a handoff. The ring set is about 94 MB at B1,
so the true transient peak is about 207 MB. That is 1.8 times B56 and 1.6 times B57.

**ARGUMENT.** The arithmetic is the document's own. The ring table gives one pair per playing
track, one more pair per armed track, and one pair for the device bridge. B1 gives 32 tracks and
B83 gives 8 armed tracks, so the count is 41 pairs. At B54 that is 94.3 MB, and 94.3 plus B52 is
103.3, which is B55.

Rule 0 states that the new set arrives "with every ring of every new chain already allocated". The
old set does not die at that moment; it drops into `basedrop`, and TH5 releases it on a job thread.
So both ring sets and both pools are live at the same time: 2 times 94.3 plus 2 times 9.0, which is
206.6 MB.

B56 is stated as B55 plus one pool. Its own sentence names only the pool. Revision 13 was correct
at 113 MB, because revision 13 reallocated no ring on a handoff. Revision 14 changed the mechanism
and left the number.

**B57 then refuses the change it is meant to allow.** `configure` "refuses a configuration whose
rings and pool would pass B57, off the audio thread, before any allocation"
(`architecture.md:4285`). At B1 the transient set passes B57 by 79 MB, so either the refusal fires
on every topology change at the product budget, or the refusal does not measure the transient,
which makes B57 a bound on nothing.

**No guard sees this.** PG24 measures a type size, not a live set. PG28 reads reachability, not
bytes. No bench and no test in section 14 measures engine memory.

**EVIDENCE.** `architecture.md:885` to `:893` (B49 to B57), `:919` (B83),
`:4269` to `:4288` (the ring table, the memory number, and the B57 refusal), `:4297` to `:4304`
(rule 0), `:4237` to `:4242` (step 4), `:4635` (TH5).

**WHAT SHOULD CHANGE.** Take the CR-17 correction, which allocates a ring for a new chain only.
Then restate B56 from the new arithmetic and state which rings the B57 test counts. If the design
keeps whole-set reallocation, raise B57 above the real peak and say so at the site.

### CRITICAL: CR-19. The whole ring allocation runs on the thread that paints

**OBSERVATION.** Section 5.5 states where `configure` runs:

```
architecture.md:4196  1. `configure` runs off the audio thread.
```

Section 5.7 states where the core runs:

```
architecture.md:4621  In the application, the core runs on the GPUI foreground thread inside one
                      entity.
```

Section 9.1 declares the one entry point, and it is synchronous:

```rust
architecture.md:6185  fn call(&mut self, verb: Verb) -> Result<VerbOutcome, GatewayError>;
```

B27 gives the allocation a budget, and Appendix B.5 states how it is held:

```
architecture.md:863   | B27 | 250 ms | Ring allocation for a change of the playable set |
architecture.md:10447 | B27 | Measured, not enforced | `Instant::elapsed` at the allocation site |
```

**CLAIM.** An immediate verb allocates about 94 MB on the GPUI foreground thread, with a stated
budget of 250 ms and no enforcement. That is 62 times B5 and 15 times B2.

**ARGUMENT.** "Off the audio thread" names one thread that `configure` must not use. It does not
name the thread it does use. The only caller is the core, and in the application the core thread is
the foreground thread. No section routes a topology change to a job thread, and section 9.6 gives a
job thread to a deferred verb alone.

`TrackSetArmed` and `MixAddBus` both change the playable set, and both must answer the user at
once. `MixAddBus` returns a refusal and a new `StripId`, so it cannot answer
`VerbOutcome::Started { job }`. A user who arms a track therefore pays the whole ring allocation on
the thread that paints every window.

**TH3 does not cover it.** TH3 bans a git walk, a file scan, an import parse, a render, and a timed
wait on that thread. Revision 14 added the timed wait to close CR-15. An allocation is none of the
five, and B27 says the bound is measured and not enforced.

**This is CR-15 at a larger scale.** CR-15 was two waits of 50 ms. This is one allocation with a
stated budget of 250 ms, and revision 14 multiplies its size by making every `configure` rebuild
the whole set.

**EVIDENCE.** `architecture.md:863` (B27), `:841` (B5), `:838` (B2), `:4196`, `:4262` to `:4267`,
`:4621`, `:4629` to `:4632` (TH3), `:6185`, `:6303` to `:6353` (section 9.6),
`:7428` (`EngineFault::AllocationSlow`), `:10447` (B.5).

**WHAT SHOULD CHANGE.** Name the thread that runs `configure`, and make it a job thread. Then state
how the result reaches the audio thread and how a verb that waits for it answers inside B5. If the
allocation stays on the core thread, add it to the TH3 ban list and state the contradiction that
follows.

### CRITICAL: CR-20. The B105 queue is first in, first out, and the generation that gates it is latest value

**OBSERVATION.** The core pushes and then publishes:

```
architecture.md:4203  ... pushes that one value into the bounded handoff ring at B105, core to
                      audio. It then publishes the new `GraphChain`, whose `handoff_generation`
                      counter has increased.
```

The audio thread pops exactly one:

```
architecture.md:4237  4. At the generation check the audio thread pops the queued `GraphHandoff`
                         and replaces both `GraphState::pool` and `GraphState::chains` ...
architecture.md:4486  A handoff change pops the B105 queue, adopts the new chain-state set and the
                      new pool together ...
```

B105 is 2, and `GraphChain` travels by `triple_buffer`, which keeps the latest value alone.

**CLAIM.** Two `configure` calls between two audio cycles leave one handoff in the ring for ever,
and the audio thread then runs a chain set that does not match the published topology.

**ARGUMENT.** The core thread is the foreground thread and B7 is 2.67 ms. Two topology verbs inside
one frame are ordinary: a template that adds a bus and then a send, or a retry that lands beside a
fresh call. Both pushes succeed, because B105 is 2.

The counter goes from N to N+1 to N+2. `triple_buffer` keeps the latest value, so the audio thread
reads N+2 at its next cycle. It pops one element, and an `rtrb` ring is first in, first out, so it
gets the N+1 handoff. It then records N+2 as its own `handoff_generation`.

The result is a silent mismatch. `GraphState::chains` holds the N+1 set while `GraphChain::chains`
holds the N+2 topology. The two arrays can differ in length, so the runner indexes a chain that
does not exist. `clippy::indexing_slicing` is denied, so section 5.11 form 4 returns
`CycleOutcome::Faulted`; the user hears a dropout and the fault names no cause. The stale handoff
stays in the ring and is adopted at the next topology change, which is a second wrong state.

**Nothing in the document states a drain rule.** Every sentence writes "pops" in the singular. The
overflow rule that WR-7 added covers a full ring. It does not cover two successful pushes.

**EVIDENCE.** `architecture.md:941` (B105), `:4201` to `:4208`, `:4237` to `:4242`,
`:4417` to `:4426` (`GraphChain`), `:4485` to `:4491`, `:4702` to `:4709` (the
`triple_buffer` table), `:4752` (the ring row), `:4930` (section 5.11 form 4).

**WHAT SHOULD CHANGE.** State the drain rule at the cycle top: pop until the ring is empty, adopt
the last value, and let every earlier value drop into `basedrop`. Then state that the handoff
carries its own generation, so the audio thread can prove that the value it adopted is the value
the published `GraphChain` names.

### CRITICAL: CR-21. The frame loop has a stop condition and no start condition

**OBSERVATION.** Revision 14 gives each frame driver a stop condition:

```
architecture.md:10038  /// **It saves the next frame, not this paint.** ... The driver compares the
architecture.md:10039  /// slot with this field and calls `Window::request_animation_frame` only
                       /// when the two differ or the transport rolls, so a still transport ends the
                       /// 60 Hz loop instead of running it forever.
architecture.md:10059  /// exactly as `PlayheadLayer::shown` does: a silent meter with no held mark
                       /// ends the loop.
```

The slot is filled in one place only:

```
architecture.md:6528  1. **`DuetApp::render` calls `self.core.update(cx, |host, _cx|
                         host.poll_frame())` as its first statement** ...
```

**CLAIM.** After the loop stops, nothing reads the publication again until an unrelated notify
arrives. A user who arms a track and sings with the transport stopped sees a meter that does not
move.

**ARGUMENT.** The chain is circular. `MeterReader::poll` runs inside `CoreHost::poll_frame`.
`poll_frame` runs inside `DuetApp::render`. `render` runs when the window is dirty. The window is
dirty because a frame driver asked for an animation frame. The driver asks only when the slot
differs from `shown`, and the slot changes only when `poll` runs.

So the first frame with a silent meter ends the loop. The audio thread keeps writing the
`triple_buffer`, and no reader reads it. Section 5.8 forbids the alternative by design: "No meter
value travels as an event" (`architecture.md:5552`), and section 10.2 calls a subscription per
frame an event storm.

Input monitoring with the transport stopped is a product function. Design contract 4.4 gives the
meter a peak cap, a fall rate, and a `CLIP` indicator, and a user sets a record level before the
transport rolls. The same hole applies to `MasterMeterLayer`.

The transport half is safer by accident. `Verb::TransportPlay` produces a `CoreEvent`, the drain
task notifies, and `render` runs. The meter half has no such event.

**Revision 13 raised the `shown` field as commentary and not as a finding.** The correct answer to
that commentary was to state what the field saves. Revision 14 instead made the field a gate on the
frame request, which is a behaviour change with a liveness cost.

**EVIDENCE.** `architecture.md:6456`, `:6521` to `:6542` (the frame order), `:5505` to `:5513`
(`MeterReader::poll`), `:5552`, `:10032` to `:10062` (the three drivers and their `shown` fields),
`:7137` (test 7, which counts requests and not liveness), `design-contract.md:752` to `:774`.

**WHAT SHOULD CHANGE.** State the start condition beside the stop condition. The smallest correct
answer keeps the loop alive while any strip is armed or the transport rolls, and stops it only when
both are false. State which value the driver reads to decide that, and state which event restarts
the loop when the value changes.

### WARNING: WR-21. The `PoolHandoffBusy` doc comment still states the revision-13 mechanism

**OBSERVATION.** Section 5.5 states the new rule, and section 15.10 states the old one:

```
architecture.md:4210  **`configure` performs one push and it never waits** (critic CR-15). On a full
                      ring it returns `ConfigError::PoolHandoffBusy` at once.
architecture.md:9309      /// The B105 handoff ring is full after the B95 retry bound, so the audio
architecture.md:9310      /// thread has not adopted the pool the last `configure` built. The core
architecture.md:9311      /// runs that retry on the core thread and it runs none at all while the
architecture.md:9312      /// engine is stopped ... (section 5.5, critic WR-17).
```

**CLAIM.** Two sites state opposite mechanisms for the one wait that CR-15 was about, and the stale
site is the one an implementer copies into the code.

**ARGUMENT.** DR1 says a revision rewrites the sentence it changes. The 15.10 comment is the
revision-13 sentence, word for word, and it names WR-17 without naming CR-15. In the application
the core thread is the GPUI foreground thread, so the stale comment states the exact defect CR-15
raised.

This comment is not decoration. Section 15 is the complete type roster, the roster compile builds
it, and chunk C3 writes `ConfigError` from it. Substitution S1 removes a whole-line comment before
the compiler reads it, so PG25 cannot see the contradiction. No guard reads two sentences against
each other.

**EVIDENCE.** `architecture.md:4210` to `:4236`, `:4973` (the timeout row), `:9307` to `:9315`,
ADR 0004 clause 11a.

**WHAT SHOULD CHANGE.** Rewrite the 15.10 comment to the section 5.5 mechanism, and cite CR-15.

### WARNING: WR-22. The deferring escape answers the free half and opens the allocation half

**OBSERVATION.** PG26 states one escape and one reason:

```
architecture.md:4522  **One escape exists, and it is a type and not a sentence.** A field whose
                      outermost head is an external name whose section 1.9 `Why` cell carries the
                      word "defers" is legal, and the walk stops there.
```

The reason each wrapper carries is the free alone:

```
architecture.md:1458  `basedrop::Owned` owns its value, and it **defers** the free of that value to
                      the collector thread (TH5).
```

**CLAIM.** A growable heap container inside a deferring wrapper passes PG26, and TH1 forbids the
allocation that container performs on the audio thread. My probe proves it.

**ARGUMENT.** I added one field to `DiskReader` and ran the guard.

```
MINE-A1   probe_grow: Owned<Vec<u8>> added to DiskReader
          exit 0
          AUDIO OWNED: 33   AUDIO EXEMPT: 1   AUDIO DEFERRED: 14   HEAP IN AUDIO: 0

MINE-A2   control, probe_grow: Vec<u8> in the same place
          exit 1
```

The escape stops the walk, so nothing below the wrapper is read. `basedrop::Owned` implements
`DerefMut`, which section 15.10 already relies on for `Consumer::pop` and `Producer::push`. A
`push` on a wrapped `Vec` therefore calls the allocator on the audio thread, and the guard counts
it as deferred.

This is the exact inverse of CR-16. Revision 13 exempted a field on the ground that `configure`
allocated it, which answered the allocation and not the free. Revision 14 accepts a field on the
ground that `basedrop` frees it, which answers the free and not the allocation. Section 5.7 states
the first limit at its own site and states nothing about the second.

No text in the document uses a growable type inside a wrapper today, so the run is honest. The rule
is what is open.

**EVIDENCE.** `architecture.md:656` to `:674` (PG26), `:1458` to `:1459`, `:4522` to `:4529`,
`:4582` to `:4591` (the exemption site), `:9215` to `:9231`; my run of MINE-A1 and MINE-A2.

**WHAT SHOULD CHANGE.** State at the PG26 site that a deferring wrapper answers the free half and
never the allocation half. Then make the walk continue through the wrapper for the allocation test
and stop only for the free test, or name the closed set of payloads a wrapper may carry.

### WARNING: WR-23. PG26 reads the heap half of TH1 and never the lock half

**OBSERVATION.** TH1 bans four things on the audio thread:

```
architecture.md:4626  - **TH1.** The audio thread never allocates, never frees, never locks, never
                        blocks ...
```

PG26 refuses a fixed list of heap names plus "any external name whose section 1.9 row carries the
word 'heap'". The section 1.9 row for a lock carries no such word:

```
architecture.md:1455  | `Mutex`, `RwLock` | `not Copy` | `Default` `Serialize` `Deserialize` | A
                        lock is not duplicable, and it supplies no comparison |
```

**CLAIM.** A lock inside an audio-owned declaration passes every guard. My probes prove it.

```
MINE-A4   probe_lock: Mutex<u32> added to ChainState        exit 0
MINE-A6   probe_rw:   RwLock<u32> added to ChainState       exit 0
MINE-A5   control, probe_box: Box<u32> in the same place    exit 1
```

**ARGUMENT.** PG26 is the one rule that reads a thread contract. Its site says it holds TH1, and
section 5.7 states that TH1 has no other mechanical guard. A lock is the second thing TH1 bans and
the one that a reviewer is least likely to see, because a `Mutex` field reads as ordinary Rust.

`Mutex` and `RwLock` are both on the candidate drop list and both map to a `std` path in the
external-path block, so the roster compiles either one. The compiler is no backstop here.

`Shared<T>` from `basedrop` is a second lock-free path the design already names, so the correct
shape exists. The guard simply does not refuse the wrong one.

**EVIDENCE.** `architecture.md:656` to `:674`, `:1455`, `:497` to `:503` (the drop list),
`:1501` to `:1502` (the external paths), `:4626`, `:4661` to `:4670` (the three means of
verification); my runs of MINE-A4, MINE-A5, and MINE-A6.

**WHAT SHOULD CHANGE.** Add `Mutex`, `RwLock`, and every other lock name to the PG26 refusal list,
or add the word that marks a lock to the `Why` cell of that row and read it as PG26 reads "heap".
State at the site which half of TH1 the rule holds and which half it declines.

### WARNING: WR-24. The second sanctioned escape cannot be used

**OBSERVATION.** PG26 names two escapes:

```
architecture.md:4524  `basedrop::Owned` and `basedrop::Shared` are the two names that carry it
                      today ...
```

The candidate drop list carries one of the two:

```
architecture.md:499   ... AtomicI32 NonZeroI64 NonZeroU8 NonZeroU16 NonZeroU32 NonZeroU64 Value Owned
```

**CLAIM.** Any declaration that names `Shared<T>` reds PG4, so the second escape is unreachable.

**ARGUMENT.** I planted a wrapped field with the second head.

```
MINE-A3   probe_shared: Shared<Vec<u8>> added to ChainState
          exit 1
          CANDIDATE TYPES: 399   UNPLACED: 1   AUDIO DEFERRED: 13
```

`AUDIO DEFERRED` rose, so PG26 accepted the escape. `UNPLACED` rose as well, because `Shared` is
not on the drop list and section 1.5 places no such type, so PG4 treats it as a declared type with
no row. The two rules disagree about one name.

Section 5.8 already names `basedrop::Shared<T>` as the mechanism for "any other owned value the
audio thread must release" (`architecture.md:4757`). A chunk that writes that field stops the guard
until an author edits a block that no finding sent them to.

**EVIDENCE.** `architecture.md:493` to `:504`, `:1459`, `:4524`, `:4757`; my run of MINE-A3.

**WHAT SHOULD CHANGE.** Add `Shared` to the candidate drop list, beside `Owned`. The `drop-name`
membership kind then holds it, because the external table and the external-path block both map it.

### WARNING: WR-25. The collector has no period, so the transient peak has no bounded life

**OBSERVATION.** TH5 states the cadence in words and cites no id:

```
architecture.md:4635  - **TH5.** The `basedrop` collector runs on a job thread and drains once per
                        job cycle.
```

B56 states that the peak lasts while the old value waits:

```
architecture.md:4279  ... B56 is the transient peak while the old pool waits for the collector.
```

**CLAIM.** No number in section 1.6 gives a job cycle a period, so B56 bounds an amount and not a
duration, and three sets can be live at once.

**ARGUMENT.** Section 1.6 holds every number this design chooses. A search for "job cycle" returns
one line, and that line is TH5. B37 gives two job threads and B36 gives the queue depth. Neither is
a period.

The consequence is not theoretical. If a second topology change lands before the collector drains,
the retired set of the first change, the retired set of the second, and the live set are all
resident. Under CR-18's arithmetic that is about 300 MB. A job thread that runs a git walk under
B19, which is 5 s, delays every drain behind it.

TH5 also puts the collector on the same two threads that run every deferred verb. An export render
holds a job thread for minutes, so one of the two drains is blocked for that time.

**EVIDENCE.** `architecture.md:855` (B19), `:872` to `:873` (B36, B37), `:892` (B56), `:4279`,
`:4614` (the job-thread row), `:4635`, `:4771`.

**WHAT SHOULD CHANGE.** Give the collector drain its own B row with a period, and state that the
drain runs on a thread that no long verb can hold. Then restate B56 as a peak with a bounded life.

### WARNING: WR-26. The Master mitigation cannot be built, because a stage curve is not an entity

**OBSERVATION.** Section 10.2 names the cached siblings of each frame driver:

```
architecture.md:6501  | `MasterMeterLayer` | `MasterView`, `DuetApp`, `Root` | Every stage curve of
                        `MasterView` |
```

The declaration says what a stage curve is:

```rust
architecture.md:10121  #[derive(Debug, IntoElement)]
                       pub(crate) struct StageCurve { slot: SlotId, kind: SlotKind, params:
                       Arc<[Finite]>, live: Option<Finite> }
```

The pinned source says what a cache needs:

```rust
gpui-pre-0.3.5/src/view.rs:232  pub fn cached(self, style: StyleRefinement) -> ViewElement<Entity<T>>
gpui-pre-0.3.5/src/view.rs:271  pub(crate) fn cached(mut self, style: StyleRefinement) -> Self
```

**CLAIM.** `Entity::cached` takes an `Entity<T>`. A `StageCurve` is a `RenderOnce` element with no
entity and no `EntityId`, so the one mitigation section 10.2 names cannot apply to it.

**ARGUMENT.** I read the source. `ViewElement::cached` is crate private "because only an
entity-backed view has `Context::notify` to bust the cache" (`view.rs:266` to `:270`). The reuse
test at `view.rs:386` also reads `window.dirty_views.contains(&entity_id)`, and
`ViewElement::id` returns `None` when the view has no entity id, so a cached element state cannot
even be keyed.

The three other rows are correct. `SystemView`, `LaneView`, and `StripView` each derive `Debug`
alone and each appears as an `Entity` in Appendix A, so each one can be cached.

So Master mode pays what the section says Mix avoids: `MasterMeterLayer` runs at 60 Hz,
`MasterView` is its ancestor, and all five stage curves re-render on every frame. Chunk K4 benches
one Mix frame and no chunk benches a Master frame, so the cost is neither removed nor measured.

**EVIDENCE.** `architecture.md:6486` to `:6501`, `:6512` to `:6515` (the bench),
`:9604` (why every element derives `IntoElement`), `:10009` (`MasterView`), `:10066`
(`MasterMeterLayer`), `:10121`; `gpui-pre-0.3.5/src/view.rs:223` to `:275`, `:362` to `:401`.

**WHAT SHOULD CHANGE.** Either make each stage curve an entity, or delete the Master row and state
what Master mode pays per frame. Then give K5 a bench over one Master frame, as K4 has for Mix.

### WARNING: WR-27. The retry has one cancel rule and two cancel needs

**OBSERVATION.** The one cancel rule is a replacement:

```
architecture.md:4225  Dropping the task cancels the retry, so a second configuration replaces the
                      first and no attempt outlives its caller.
```

The task body is stated too:

```
architecture.md:4219  ... The task awaits `cx.background_executor().timer` for one B7 period ... and
                      it then re-enters `DuetCore` inside `cx.update` and calls `configure` again.
```

**CLAIM.** Two paths leave a pending retry alive, and on both of them the retry calls `configure`
against state that no longer matches the topology it captured.

**ARGUMENT.** Path one is a project close. `CoreHost` lives for the life of the application, not the
life of a project. `AppEvent::ProjectClosed` exists (`architecture.md:9689`). No section says that
a close clears `handoff_retry`. The pending task therefore fires after the close and re-enters
`DuetCore` with a stale `ChainTopology`.

Path two is a `configure` that succeeds. The rule covers a second refusal, which replaces the task.
It does not cover a second call that the ring accepts, because that call refuses nothing and drops
no task. The older retry then fires and pushes a second, older `GraphHandoff`. That is the second
producer CR-20 needs to make the queue and the generation disagree.

The `handoff_retry` field is an `Option<Task<()>>` and carries no topology, so the document does not
say what the task re-configures. Section 15.16 declares three methods on `CoreHost`, and none of
them starts or clears the retry.

**EVIDENCE.** `architecture.md:4217` to `:4230`, `:9644` to `:9676` (`CoreHost`), `:9689`
(`AppEvent`), `:10229` (Appendix A).

**WHAT SHOULD CHANGE.** State every event that clears `handoff_retry`: a project close, a
successful `configure`, and an engine stop. State what the task holds so a reader knows which
topology it retries, and declare the method that starts it.

### WARNING: WR-28. No entity holds the subscription that keeps the tokens current

**OBSERVATION.** Contract 7.1 rule 2 is a MUST:

```
design-contract.md:1114  - Duet resolves `DuetTokens` once at start and again on every change of
design-contract.md:1115    `cx.theme().mode`.
```

The architecture states the same duty and names no mechanism:

```
architecture.md:9712  ... chunk K1 writes `impl gpui_kit::Global for DuetTokens {}`, resolves the
                      value once at start, and resolves it again on every change of
                      `cx.theme().mode`.
```

**CLAIM.** A change of the theme is observed through a subscription, and no declaration holds one.
Appendix A rule 3 makes that a defect by the document's own rule.

**ARGUMENT.** `gpui_kit::Theme` is a global. A view learns that a global changed through
`observe_global`, which returns a `Subscription`. Appendix A rule 3 states: "Every `Subscription` is
stored on the subscriber entity. A dropped subscription is a defect, and test 6 of section 10.7
catches it" (`architecture.md:4244`).

`DuetApp` holds `subscriptions: Vec<Subscription>` and no sentence assigns the theme observer to it.
`DuetTokens` is a plain struct with `resolve`, `part`, `wave_fill`, and `wave_rms`, so it holds
nothing. No other declaration names a `Subscription` field.

The failure is silent and it breaks the whole token layer. Every painted colour reads
`cx.global::<DuetTokens>()`. If the observer is dropped, the values stay at the start theme, so
dark mode paints light colours, and contract 7.3 rule 6, which re-checks every `own` token for
contrast, never runs again.

**EVIDENCE.** `architecture.md:4244` (Appendix A rule 3), `:9612` to `:9634` (`DuetApp`),
`:9707` to `:9783` (`DuetTokens`), `:7137` to `:7145` (test 6);
`design-contract.md:1113` to `:1119`, `:1175` to `:1179`.

**WHAT SHOULD CHANGE.** Name the entity that observes the theme, give it the `Subscription` field,
and state in the 13.2 cell of chunk K1 that it writes the observer. Then add the theme change to
test 6 or to a test of its own.

### WARNING: WR-29. The declared `configure` cannot do what five sections give it

**OBSERVATION.** The one declaration is per chain:

```rust
architecture.md:4342  pub fn configure(topology: &ChainTopology) -> Result<ChannelConfig, ConfigError>;
```

The prose gives it the whole graph:

```
architecture.md:4196  1. `configure` runs off the audio thread. It builds a **whole new
                         `BufferPool`**, sized for the new topology ...
architecture.md:4299  `configure` therefore builds a whole new `Box<[ChainState]>` off the audio
                      thread ...
architecture.md:4203  ... pushes that one value into the bounded handoff ring at B105 ...
```

**CLAIM.** A function that takes one chain's topology and returns one chain's channel counts cannot
build a whole pool, build a whole chain set, push into a ring, or return `PoolHandoffBusy`.

**ARGUMENT.** `ChainTopology` describes one track chain: one `TrackId`, two slot lists, one send
list, one meter tap, and one output. The pool is sized from the whole graph, because B47 and B48
bound the project. The chain set is the whole array. The B105 ring carries one message for the whole
graph. None of the four can be derived from one chain.

`ConfigError` also carries `StripBudgetExceeded`, `ParamBudgetExceeded`, and `PoolHandoffBusy`, and
all three are project-wide refusals that one chain's topology cannot decide.

PG25 compiles the signature and never a body, so the roster is green. No guard reads a signature
against the duty the prose gives it. An implementer who writes chunk C3 from this text gets a
function that cannot hold the mechanism that answers CR-15, CR-16, and WR-7.

**EVIDENCE.** `architecture.md:3998` to `:4013` (`ChainTopology`), `:4196` to `:4242`,
`:4297` to `:4304`, `:4333` to `:4343`, `:9294` to `:9320` (`ConfigError`), `:717` to `:721`
(what PG25 does not cover).

**WHAT SHOULD CHANGE.** Declare the function the prose describes. It takes the whole `GraphChain`
or the whole session mix state, and it returns the outcome of a whole configuration. Keep the
per-chain function if a caller needs it, and give it a second name.

---

## 3. Concerns

**N-1. Section 1.7 is one rule and one probe short.** Line 1075 states "PG1 to PG29" and line 1076
states "PP1 to PP29". PG30 is stated at line 792 and PP30 has a row at line 1200. Section 1.7 is
the index DR4 names, and no guard reads it.

**N-2. The first pinned-source citation is off by one.** Section 10.2 fact 1 writes
"`src/window.rs:2606` reads `let entity = self.current_view();` and `:2607` schedules
`cx.notify(entity)`". In the pinned crate, `:2606` is the function signature, `:2607` is the
`let`, and `:2608` is the `on_next_frame` call. Facts 2 and 3 and the `entity_map.rs:476` citation
are exact.

**N-3. The two impl counts carry no floor.** C-3 gave `ROSTER ITEMS` a floor and left
`ROSTER IMPL BLOCKS` and `ROSTER IMPLS` open. My run deleted `impl Global for DuetTokens {}` and
the roster exited 0 with the counts at 45 and 18 in place of 46 and 19. The placement guard reds on
a deleted declaration and reads no impl block, so no pair covers this one.

**N-4. ADR 0004 contradicts its own C-12 fix.** Clause 22 states "no version number appears here".
Clause 19 writes `ubuntu-26.04` and `macos-26`, clause 20 writes `ubuntu-26.04`, and the background
at line 19 writes "Ubuntu 26.04, macOS 26". Section 11 owns all four.

**N-5. The cached-sibling work is assigned to one chunk of three.** Section 10.2 gives
`ComposeView`, `RecordView`, and `MixView` the same `Entity::cached` rule. The 13.2 cell of K4 names
"the cached sibling strips of section 10.2". The cells of K2 and K3 name nothing.

**N-6. The exemption block is still an allow list, and no shape exploits it today.** I added
`ChainState.trim  outlives-cycle` to the block and the guard exited 0, because `trim` is a real
field of an audio-owned type and the `field-path` kind asks no more. The walk still stops at an
exempt field. No hole follows today, because every type an audio-owned root reaches is itself a
root in the block.

**N-7. `PoolHandoffBusy` names a message and no user action.** Section 12.4 gives the user a message
that names the engine state. The retry runs three attempts over about 8 ms and then writes
`CoreHost::fault`. The document does not say what the user does next, and the topology change is
lost.

**N-8. One version is written two ways.** Line 5841 writes "pipewire 0.10". Lines 7219 and 10384
write "0.10.1". DR3 exempts a version pin as a fact, and two values for one fact is still a
disagreement.

**N-9. Three contract names resolve to nothing in the architecture.** `DropdownMenu`
(`design-contract.md:56`), `ResizableState` (`:116`), and the `StageCurve` sizes 96 px and 160 px
(`:1495`) appear nowhere in `architecture.md`. The contract also writes the token ids in dotted form
and the architecture in snake case.

**N-10. The plan graph states one pair twice.** Section 13.4's generic row expands to `M0 before
T1`, and the explicit row at line 7972 states the same pair. No other explicit row repeats the
generic row.

---

## 4. Every probe I ran, and its result

### 4.1 The three guards on the unmodified document

| Guard | Result |
|---|---|
| `placement_check.py architecture.md` | Exit 0. Every one of the 31 counter lines matches section 1.9, character for character. |
| `conversion_check.py`, from the repository root | Exit 0. `MEMBERS: 3   FILES: 6   FINDINGS: 0`. |
| `roster_compile.sh architecture.md <scratch> <repo>` | Exit 0. `ROSTER ITEMS: 385     FLOOR: 385`, `ROSTER IMPL BLOCKS: 46`, `ROSTER IMPLS: 19`, `ROSTER SUBS: 8`, `ROSTER CLIPPY: clean`, `ROSTER SIZES: 409 measured`, `RECORDED ROWS: 37 checked     HEAD ROWS: 13     SIZE BAD: 0`. |

### 4.2 The three harnesses

| Harness | Result |
|---|---|
| `probe_run.py` | `PROBES BAD: 0`. All 45 rows and all 155 PP27 shapes reproduce their recorded exit code and their recorded line. |
| `probe_conversion.py` | `CONVERSION PROBES BAD: 0`. All 21 shapes reproduce. |
| `probe_roster.sh` | `ROSTER PROBES BAD: 0`. PP25-eq, PP25-drop, PP25-floor, GATE-allow, GATE-unwrap, and GATE-as each emit the exact recorded compiler line. |

### 4.3 My own placement probes

| Probe | Shape | Result | Verdict |
|---|---|---|---|
| MINE-A1 | `probe_grow: Owned<Vec<u8>>` in `DiskReader` | exit 0, `AUDIO DEFERRED: 14`, `HEAP IN AUDIO: 0` | **Hole, WR-22** |
| MINE-A2 | Control: `probe_grow: Vec<u8>` in the same place | exit 1 | Caught |
| MINE-A3 | `probe_shared: Shared<Vec<u8>>` in `ChainState` | exit 1, `UNPLACED: 1`, `AUDIO DEFERRED: 13` | **Rule clash, WR-24** |
| MINE-A4 | `probe_lock: Mutex<u32>` in `ChainState` | exit 0 | **Hole, WR-23** |
| MINE-A5 | Control: `probe_box: Box<u32>` in the same place | exit 1 | Caught |
| MINE-A6 | `probe_rw: RwLock<u32>` in `ChainState` | exit 0 | **Hole, WR-23** |
| MINE-A7 | A new external row with no "heap" word, plus a field that names it | exit 1, `UNPLACED: 1` | Caught, by PG4 |
| MINE-A8 | The same with the word "heap" in the `Why` cell | exit 1 | Caught |
| MINE-A9 | A `B44` citation under a `####` heading of section 5.5 | exit 1, `USED BY: B44: section 5.5 cites it and the `Used by` cell omits it` | Caught |
| MINE-A10 | A `B99` citation in section 5.11 | exit 1, `USED BY: B99: section 5.11 cites it ...` | Caught |
| MINE-A11 | `Copy` removed from `ParamSnapshot` | exit 1 | Caught |
| MINE-A12 | `Score` added to the audio-owned block | exit 1 | Caught |
| MINE-A13 | `ChainState.trim  outlives-cycle` added to the exemption block | exit 0 | Stated limit, N-6 |

MINE-A2, MINE-A5, MINE-A8, and MINE-A12 are paired controls. Each pair plants one violation per
run.

**PG30 is sound in both directions.** MINE-A9 shows that a `####` sub-heading resolves to its
parent section number, so the label model has no gap below a heading.

### 4.4 My own conversion probes

| Probe | Shape | Result |
|---|---|---|
| MINE-C20 | A cast inside a `macro_rules!` body | exit 1, `CAST: m/src/lib.rs: line 1` |
| MINE-C21 | ` as ` inside a `#[doc = "..."]` attribute | exit 0, `FINDINGS: 0` |
| MINE-C22 | A cast in a `[[test]]` target at `checks/it.rs` | exit 1, `FILES: 2` |
| MINE-C23 | A cast in a file that opens with a byte order mark | exit 1 |
| MINE-C24 | A cast with a block comment between the token and the type | exit 1 |
| MINE-C25 | A cast in a local path dependency that is not a workspace member | exit 1, `MEMBERS: 2` |
| MINE-C26 | The sanctioned exemption reached through a `./src/./lib.rs` library path | exit 0 |
| MINE-C27 | Denominator: one member, one empty source file | exit 0, `FILES: 1` |
| MINE-C28 | Denominator: a member whose only target path does not exist | exit 2, fail closed |

**The conversion guard is the strongest of the three.** I found no miss in nine probes, and I have
now found none in twenty-two across two reviews. MINE-C25 is the sharp one: a local path dependency
outside the member list is still scanned.

### 4.5 My own roster probes and my three gate defects

Each plant sits in text that revision 14 wrote, and each one ran alone.

| Plant | Site | Result |
|---|---|---|
| `#[allow(clippy::struct_field_names)]` on a declaration | `DuetTokens`, section 15.16 | **exit 1**; ``duet/src/lib.rs:157:3: error: #[allow] attribute found: help: replace it with: `expect` `` |
| `.unwrap()` inside a spelled-out body | `impl core::fmt::Debug for ChainSetHandle`, section 5.5 | **exit 1**; ``duet-engine/src/lib.rs:111:27: error: used `unwrap()` on `Some` value`` |
| A bare `as` cast inside a spelled-out body | `impl Default for ResetGenerations`, section 5.6 | **exit 1**; ``duet-engine/src/lib.rs:133:41: error: casting `i64` to `u16` may truncate the value`` |
| MYR-size: a recorded size row the compiler contradicts | `Owned<BufferPool>` changed to 16 bytes | **exit 1** |
| MYR-global: `impl Global for DuetTokens {}` deleted | section 15.16 | **exit 0**; `ROSTER IMPL BLOCKS: 45`, `ROSTER IMPLS: 18`. Concern N-3 |
| MYR-copy: `Copy` removed from `ParamSnapshot` | section 15.10 | exit 0 on the roster; the placement guard reds on the same copy, so the pair covers it |
| MYR-pin: the `basedrop` row deleted from the pin block | section 1.9 | **exit 2**; fail closed with the DR7 message |

PG25 catches all three of my gate defects. No substitution hides any of them.

---

## Reactive Assessment

- **Responsive: FAIL.** The CR-15 wait is gone and the frame cost is now stated from the source. Two
  new paths are worse. A whole ring allocation runs on the painting thread inside an immediate
  verb, with a measured and unenforced budget of 250 ms against B5 at 4.0 ms (CR-19). The meter
  frame loop stops and no stated condition restarts it, so a live input level never repaints
  (CR-21).
- **Resilient: FAIL.** Every failure surface is a typed enum and no crate holds `unsafe`. The
  fail-open class and the allow-list class are both closed. The audio path is not: rule 0 needs
  state that only the audio thread holds (CR-17), and the queue that carries the handoff can
  desynchronise from the generation that gates it and leave the runner on the wrong chain set
  (CR-20). PG26 reads one half of TH1 and accepts a lock (WR-23) and a growable container behind a
  wrapper (WR-22).
- **Elastic: FAIL.** Every channel, ring, and queue is still bounded with a named overflow action.
  The memory model is not. Rule 0 makes every ring live twice, so the transient peak is about 1.8
  times B56 and 1.6 times B57 (CR-18), and the collector that ends the peak has no stated period
  (WR-25).
- **Msg Driven: PASS.** Each publication has one read end, one owner, and a frame order whose
  premise I verified in the pinned source. No view holds an `Output`, no `Global` and no lock
  crosses a view seam, and the retry is a scheduled message rather than a block.

---

## Verdict

**NOT READY.** Revision 14 is honest work and it closes every finding of revision 13 that a guard
can hold. I reproduced all three baselines and all three harnesses on this machine, and my own
re-runs of five recorded holes are now red. PG30 is the first rule in this plan that measures a
whole column rather than reads it, and it works in both directions.

The design half of CR-16 did not close. **The single biggest risk is that migration rule 0 reads as
a mechanism and is not one.** The core cannot copy a kept `ChainState`, because Appendix A gives
the audio thread sole ownership of it. Four Criticals follow from that one sentence: the state it
claims to copy, the memory it doubles, the thread it allocates on, and the queue it pushes into.
The fifth Critical is separate: a frame loop that revision 14 taught to stop and never taught to
start.

The weakest Reactive property is **Responsive**, and three of the four are now failing. The plan
answers each review at the site the review named, and each answer moves the defect one level out.
Revision 11 put an allocation in a cycle. Revision 13 put a sleep on the painting thread. Revision
14 puts a 94 MB allocation there and gives the audio thread a state copy that cannot be made.

### The blocking list

1. **CR-17** Rule 0 copies kept chain state that only the audio thread holds, and Appendix A says no
   other thread holds a reference. Rule 0 and rule 1 cannot both be true.
2. **CR-18** Rule 0 makes both ring sets live at once, so the transient peak is about 207 MB. B56 is
   113 MB and names only the pool, and B57 is 128 MB.
3. **CR-19** `configure` allocates about 94 MB on the GPUI foreground thread inside an immediate
   verb. B27 is 250 ms, measured and not enforced. B5 is 4.0 ms and TH3 does not name an
   allocation.
4. **CR-20** The B105 ring is first in, first out and `handoff_generation` is a latest value. Two
   pushes in one audio cycle leave a stale handoff in the ring and the runner on the wrong chain
   set. No drain rule exists.
5. **CR-21** `PlayheadLayer::shown` and `MeterLayer::shown` stop the animation-frame request, and
   `poll_frame` runs only inside `DuetApp::render`. An armed input meter freezes while the transport
   is stopped.
6. **WR-21** The `ConfigError::PoolHandoffBusy` doc comment still states that the core runs the
   retry on the core thread.
7. **WR-22** A growable heap container inside a `basedrop::Owned` passes PG26. My MINE-A1 run exits
   0 with `AUDIO DEFERRED: 14`.
8. **WR-23** A `Mutex` and an `RwLock` inside an audio-owned declaration pass PG26. TH1 forbids a
   lock and no other guard reads one.
9. **WR-24** `basedrop::Shared` is a sanctioned escape that PG4 refuses, because the candidate drop
   list omits it.
10. **WR-25** TH5 gives the collector no period, so B56 bounds an amount and not a duration.
11. **WR-26** `StageCurve` derives `IntoElement`, so the `Entity::cached` mitigation cannot apply to
    the Master row of section 10.2.
12. **WR-27** `handoff_retry` is cleared by a second refusal alone. A project close and a successful
    `configure` each leave it pending.
13. **WR-28** No declaration holds the `Subscription` that re-resolves `DuetTokens` on a theme
    change, which Appendix A rule 3 makes a defect.
14. **WR-29** The declared `configure` takes one `&ChainTopology` and the prose gives it the whole
    graph, the pool, the chain set, and the B105 push.

### The Concerns, for the Orchestrator to file

| Id | One-line disposition |
|---|---|
| N-1 | Correct section 1.7 to PG1 to PG30 and PP1 to PP30. |
| N-2 | Correct the section 10.2 fact-1 line numbers to `:2607` and `:2608`. |
| N-3 | Give `ROSTER IMPL BLOCKS` and `ROSTER IMPLS` a floor, as `ROSTER ITEMS` has. |
| N-4 | Delete the runner labels from ADR 0004 clauses 19 and 20 and from the background line. |
| N-5 | Name the cached siblings in the 13.2 cells of K2 and K3, as K4 already does. |
| N-6 | State at the exemption site that a `field-path` row proves a referent and not a reason. |
| N-7 | State what the user does after `PoolHandoffBusy`, and whether the topology change is lost. |
| N-8 | Write one `pipewire` version at line 5841, line 7219, and line 10384. |
| N-9 | Reconcile `DropdownMenu`, `ResizableState`, and the `StageCurve` sizes with the contract. |
| N-10 | Delete the duplicate `M0 before T1` row from section 13.4, or state that the generic row is inclusive. |

### Commentary, which is not a finding

The `block-members` block is the best mechanism this plan has added since `read_block`. It turns
every allow list into data with a stated universe, and it closed four of my recorded holes in one
edit. I recommend that chunk M0 keep the kind vocabulary exactly as section 1.9 writes it.

PG30 is the second. A prose fix of the `Used by` column failed in three revisions, and one rule
closed it in one. The same shape answers the two counts in concern N-3.
