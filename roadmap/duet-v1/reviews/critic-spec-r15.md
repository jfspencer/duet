# Specification review, revision 15: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-21. Mode: specification review, before plan authoring.
This is the fifteenth pass. Revisions 1 to 14 each returned NOT READY.

## What I read and what I ran

I read `roadmap/duet-v1/architecture.md` (11,958 lines), the six records under `roadmap/duet-v1/adr/`,
all six files under `roadmap/duet-v1/tools/`, `research/linux-macos-platform.md`,
`product-requirements.md` section 8, `design-contract.md` sections 1, 1.6, 1.11, 3.1, 3.3, 4.4, 7 and
the `StageCurve` appendix, `CLAUDE.md`, `Cargo.toml`, `.cargo/config.toml`, `clippy.toml`,
`deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`. I read the fourteenth review and the
Architect's revision-15 report from the plan store.

I read the pinned source for every claim this revision rests on: `basedrop-0.1.3/src/owned.rs`,
`src/shared.rs`, `src/collector.rs`, `rtrb-0.4.0/src/lib.rs`, `gpui-pre-0.3.5/src/window.rs`,
`src/view.rs`, `src/app/context.rs`, `src/app/entity_map.rs`, and
`gpui-component-0.6.4/src/theme/mod.rs`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/`. Every `cargo` run happened inside the shared scratch directory the scripts use.
I made no second workspace. Free space was 113 GB at the start and 113 GB at the end. I wrote no
repository file and I ran no other git command.

**All three guards reproduce the recorded baseline, line for line.** `placement_check.py` prints the
33-line block of section 1.9 exactly, with `MEMBER BAD: 0`, `USED BY BAD: 0`, `AUDIO OWNED: 37`,
`AUDIO EXEMPT: 1`, `AUDIO DEFERRED: 33`, `HEAP IN AUDIO: 0`, `GROW IN AUDIO: 0`, and
`LOCK IN AUDIO: 0`. `conversion_check.py` prints `MEMBERS: 3   FILES: 6   FINDINGS: 0`.
`roster_compile.sh` exits 0 with `ROSTER ITEMS: 395     FLOOR: 395`,
`ROSTER IMPL BLOCKS: 49     FLOOR: 49`, `ROSTER IMPLS: 21`, `ROSTER SIZES: 419 measured`, and
`RECORDED ROWS: 37 checked     HEAD ROWS: 13     SIZE BAD: 0`.

**All three harnesses reproduce every recorded result.** `probe_run.py` prints `PROBES BAD: 0` over
230 probe runs. `probe_conversion.py` prints `CONVERSION PROBES BAD: 0`. `probe_roster.sh` prints
`ROSTER PROBES BAD: 0`, and all eleven shapes emit the exact lines the document records.

**I added 24 probes of my own**: 15 against the placement guard, 9 against the conversion guard, and
4 against the roster compile. **I planted three gate defects of my own**, at three sites revision 15
wrote and the Architect did not use. Section 4 lists every result.

**Revision 15 closes the design half of CR-16 and every Critical of revision 14.** The migration is
now a plan and a per-position move. No thread copies a `ChainState`. The pool is built once. The
allocation left the thread that paints. The frame pump has a start condition, and I verified its
premise in the pinned source.

It does not close. **Two Criticals and eleven Warnings block it.** One Critical is a duty with two
owners in one table. The other is a cache that does not cache the thing its own decision record
adopted it for.

---

## 1. Closure check

### 1.1 The five Criticals of revision 14

| Id | Finding | State | Section and reason |
|---|---|---|---|
| CR-17 | Migration rule 0 needs state that only the audio thread holds | **CLOSED** | 5.5 rule 0 is a plan of `Adopt` and `Keep(ChainIndex)`, `ChainLayout` names an order and a count, and the audio thread writes `old[k].state.take()`. No copy is required and no `Clone` is needed. |
| CR-18 | Rule 0 doubles the ring set | **CLOSED** | The pool is built once per open stream. B56 is a formula, and I checked the arithmetic: 9.0 plus 2.3 times 49 is 121.7, which is B56 at 122 and sits below B57 at 128. |
| CR-19 | The whole ring allocation runs on the thread that paints | **PARTIAL** | `configure` and the allocation moved to the engine handoff thread, and TH3 gained B110. The remedy asked for one named thread. The document names two. See C15-1. |
| CR-20 | The B105 queue is first in, first out | **CLOSED** | 5.6 states a drain to empty, an apply in full before the next pop, and a generation carried by the handoff. B105 is capacity one. |
| CR-21 | The frame loop has a stop condition and no start condition | **PARTIAL** | `FrameDemand` carries three start terms, `CoreHost::pumping` re-arms `Context::on_next_frame`, and test 13 asserts both halves. `MonitorMode::Always` is a fourth start condition the table omits. See C15-10. |

### 1.2 The nine Warnings of revision 14

| Id | Finding | State | Section and reason |
|---|---|---|---|
| WR-21 | The `PoolHandoffBusy` doc comment states the old mechanism | **CLOSED** | 15.10 now names the engine handoff thread and cites CR-15. The period it states belongs to C15-5. |
| WR-22 | A growable container passes behind a deferring wrapper | **CLOSED** | PG26b exists. My MINE-5 and MINE-7 both red. The walk continues through the wrapper. |
| WR-23 | PG26 reads one half of TH1 and accepts a lock | **CLOSED** | PG26c exists. My MINE-3, MINE-4, and MINE-6 all red, inside a wrapper and below the exempt field alike. |
| WR-24 | `basedrop::Shared` is an escape that PG4 refuses | **CLOSED** | `Shared` sits on the candidate drop list beside `Owned`. The PP26b shape confirms it. |
| WR-25 | The collector has no period | **CLOSED** | TH5 names the engine handoff thread and B108. No deferred verb runs there. |
| WR-26 | The Master mitigation cannot be built | **CLOSED** | 10.2 states that Master caches nothing and names three things it pays instead. K5 owns the bench. I verified `view.rs:232` and `:266` to `:271`. |
| WR-27 | The retry has one cancel rule and two cancel needs | **PARTIAL** | `Reset`, the clear on a successful push, and the stream-close order are stated. The refusal path is not, and one variant carries two meanings. See C15-6 and C15-7. |
| WR-28 | No entity holds the theme subscription | **CLOSED** | `CoreHost::theme: Subscription`. I verified `context.rs:177` and `theme/mod.rs:193`. |
| WR-29 | The declared `configure` cannot do what five sections give it | **CLOSED** | Two functions exist: `GraphConfigurator::configure` over a whole `ConfigureRequest`, and `configure_chain` over one chain. |

### 1.3 The ten Concerns of revision 14

| Id | State | Evidence |
|---|---|---|
| N-1 | **PARTIAL** | Section 1.7 now names PG26b and PG26c. It states 1.5 as the site, and 1.5 declares neither. See N15-2. |
| N-2 | CLOSED | I read the pinned file. `window.rs:2606` is the signature, `:2607` is the `let`, and `:2608` is the `on_next_frame` call. The three line numbers are exact. |
| N-3 | CLOSED | `impl-sites` gives `ROSTER IMPL BLOCKS` a floor of 49. My MINE-R2 exits 2 and `PP25-impl` reds. The reverse direction stays open; see N15-5. |
| N-4 | **PARTIAL** | No runner label appears in any ADR. Four version numbers appear in ADR 0004, whose clause 22 states that none does. See N15-10. |
| N-5 | CLOSED | The K2 cell names "the cached sibling systems" and the K3 cell names "the cached sibling lanes". |
| N-6 | CLOSED | Section 5.7 states that a `field-path` row proves a referent and not a reason. |
| N-7 | CLOSED | The paragraph "What the user does after a refusal" states that nothing is lost. |
| N-8 | CLOSED | `pipewire` reads 0.10.1 at all four sites. A new split appeared on the macOS version; see N15-16. |
| N-9 | CLOSED | Section 10.3 gives `DropdownMenu`, `ResizableState`, the two `StageCurve` sizes, and the dotted-against-snake-case rule a home. |
| N-10 | CLOSED | I counted 48 ordered pairs and 48 unique pairs in section 13.4. No duplicate remains. |

### 1.4 The twelve rows of Appendix C.17

Each row names a section that now holds its mechanism. I checked every one.

| Row | State |
|---|---|
| The collector on a job thread | CLOSED. 5.8 and TH5 both name the engine handoff thread. |
| `ChainState` forces a ring on every bus | CLOSED. `reader` and `writer` are each an `Option`, and `track` is an `Option<TrackId>`. |
| The B56 first term advances too early | CLOSED for the stated mechanism. The word "seen" overstates it; see N15-6. |
| The exemption runs before the heap test | CLOSED. My MINE-6 and MINE-7 both red below the exempt field. |
| `configure` names two inputs | CLOSED. The doc comment names four. The signature cannot express the gate; see C15-8. |
| `hand_off` declares no deferred value | CLOSED. It returns `Result<Option<Generation>, ConfigError>` and takes no argument. |
| `Reset` forgets the layout while the array is live | CLOSED. Step 5 states the stream-close order on every path. |
| The collector leaks at shutdown | CLOSED. The five-step order ends with `Collector::try_cleanup`. I verified the leak note at `collector.rs:160`. |
| Step 1 writes a document on the painting thread | CLOSED. Step 1 states an in-memory change through `Arc::make_mut`. |
| The array is keyed by `TrackId` and a bus by `StripId` | CLOSED. `StripId` is the one identity at all three sites and in Appendix A. |
| The index-soundness sentence is false | CLOSED. 5.6 gives the apply-in-full rule as the reason. |
| The four Concerns | CLOSED. |

### 1.5 The thirteen rows of Appendix C.18

| Row | State |
|---|---|
| A lock and a growable container below the exempt field | CLOSED. My MINE-6 and MINE-7 prove both. |
| The slot test made the retry unreachable and opened a resend loop | **PARTIAL**. The wait path is closed. The refusal path is not; see C15-7. |
| The peak formula omits the retired array | CLOSED for the mechanism. See N15-6. |
| `configure` states two input lists | CLOSED. |
| The pool has no declared path to the audio thread | **PARTIAL**. The path names `AudioBackend::start`, which takes no argument; see C15-8. |
| The collector has no teardown rule | CLOSED. |
| TH3 forbids an allocation B101 requires | CLOSED. TH3 bounds an immediate verb and names the job thread for a snapshot. |
| Four sentences contradict the declarations | CLOSED. |
| The three name sets sit inside the guard | CLOSED. `heap-names`, `grow-names`, and `lock-names` are registered blocks. |
| The recorded PG26 results name a type no block declares | CLOSED. Every name resolves. |
| `probe_run.py` exits 0 on an unknown id | CLOSED. |
| ADR 0004 clause 8e.3 carries a refuted claim | CLOSED. |
| No rule states what a stream stop and a restart do | CLOSED for the state. The command that starts a stream is still absent; see C15-8. |
| The six Concerns | CLOSED. |

### 1.6 The mechanical passes

1. **Budget ids.** 110 rows, B1 to B110. No gap, no duplicate, no row out of order.
2. **Rule ids.** DR1 to DR7, PL1 to PL3, VR1 to VR6, TH1 to TH10, SM0 to SM7, 33 PG declarations
   against 35 PP rows, and 11 CG rules against 11 CP rows. Two PP rows name a rule that section 1.5
   does not declare; see N15-2. TH9 follows TH10 and CG4b follows CG5; see N15-7.
3. **Chunk coverage.** 55 chunks over sections 13.1 and 13.2. Every chunk sits in exactly one phase.
   All 14 width cells agree, and their sum is the 45 line chunks.
4. **The plan graph.** 48 ordered pairs, all unique. No link runs backward. No pair names an
   undefined chunk.
5. **Write scopes.** `Cargo.lock` is the one shared path across nine phases, and SM5 rule 4 governs
   it. No source file, no manifest, and no policy file is shared inside one phase.
6. **MUST stories.** 64 rows in the requirements and 64 rows in 13.2. The symmetric difference is
   empty. The ten SHOULD ids agree as well, and four of them carry no work; see N15-17.
7. **ADR citations.** Every section number, every B id, and every type name resolves, with one
   exception: `Path<Pixels>`, which is the subject of C15-2. No ADR holds a code fence.
8. **Design contract.** All 29 token roles of contract 7.2 map onto `DuetTokens`. Two contract
   behaviours have no owner; see C15-11 and N15-18.
9. **Platform research.** One contradiction; see C15-13.
10. **Guard blocks.** 35 registered blocks. 35 unique ids. No block sits below its stated minimum.
    Twenty-two sit exactly at it.

---

## 2. New findings in revision 15

### CRITICAL: C15-1. Ring allocation has two owners, and the memory bound counts only one

**OBSERVATION.** One table gives the duty to two threads, two rows apart.

```
architecture.md:5316  | Engine disk | `duet-engine` | Ring refill and drain, ring allocation, take
                        files, peak writes | ...
architecture.md:5318  | Engine handoff | `duet-engine` | `GraphConfigurator`: `configure`, ring
                        allocation, the one `triple_buffer::Input<GraphChain>`, ...
```

TH4 states the first owner as a rule:

```
architecture.md:5344  ... it moves samples between a ring and a take file, it allocates and releases
                      rings, and it writes peaks.
```

Section 5.5 states the second owner as the mechanism:

```
architecture.md:4760  ... That is a topology change, which runs on the engine handoff thread under
                      B27. **A ring is allocated into a `ChainState` that `configure` builds there,
                      and it is released by the collector on that same thread** (TH5).
```

ADR 0004 clause 12a states a third sentence: "the allocation runs on the engine disk thread under
B27".

**CLAIM.** Two threads own one allocation. The memory bound B57 rests on a count that one of them
keeps, so the bound measures nothing if the other one allocates.

**ARGUMENT.** B56 is a formula over `ChainLayout::pairs`, and the definition is exact: "it counts
every ring pair this configurator has allocated and has not yet seen released"
(`architecture.md:4797`). The configurator is the engine handoff thread. A ring the engine disk
thread allocates is outside that count by construction.

So one of two sentences is false. Either the disk thread allocates no ring, and TH4, the disk row,
and ADR clause 12a are each wrong, or it does, and `ChainLayout::pairs` is incomplete and
`configure` refuses against a number smaller than the truth. Revision 15 states both.

**The plan carries the split into two chunks in two phases.** Chunk C2 in phase 4 writes "ring
allocation on a playable-set change" into `crates/duet-engine/src/disk/{reader,writer,ring}.rs`
(`architecture.md:8515`). Chunk C3 in phase 5 writes `GraphConfigurator` and `configure` into
`crates/duet-engine/src/chain/configure.rs` (`architecture.md:8516`). Two engineers therefore build
the same duty in two files, one phase apart, and neither brief names the other.

**This is the CR-19 defect at a smaller radius.** CR-19 asked the document to name the thread that
allocates. The answer named one thread in section 5.5 and left the two older sentences in place,
which DR1 forbids: "A revision rewrites the sentence it changes."

**EVIDENCE.** `architecture.md:5316`, `:5318`, `:5343` to `:5344` (TH4), `:4758` to `:4766`,
`:4797` to `:4809` (the B56 formula), `:8515` (chunk C2), `:8516` (chunk C3), `:865` (B27);
`adr/0004-backend-and-threading-contract.md`, clause 12a.

**WHAT SHOULD CHANGE.** Delete "ring allocation" from the engine disk row and from TH4, and delete
the clause 12a sentence. Then take "ring allocation on a playable-set change" out of the C2 goal, so
one chunk builds it.

### CRITICAL: C15-2. The path cache holds an untessellated point list

**OBSERVATION.** ADR 0006 decision 6 adopts one thing and states one reason:

```
adr/0006-wrapped-timeline.md:43  - **A tessellated `Path<Pixels>` cache is adopted.** The audit
                                 records that `PathBuilder::build()` tessellates on the central
                                 processor on each call and advises a cached path.
```

Design contract 3.3 asks for the same thing. B61 is "The tessellated path cache bound". The
declaration holds something else:

```rust
architecture.md:7808  /// The tessellated path cache. One owner per mode, bounded by B61.
architecture.md:7813  pub(crate) struct PathCache {
                          paths: BTreeMap<PathKey, Arc<PathPlacement>>,
                          order: VecDeque<PathKey>,
                          bytes: u64,
                      }
```

`PathPlacement` is a `duet-engrave` type:

```rust
architecture.md:9889  pub struct PathPlacement { points: Vec<(f32, f32)>, width: f32, filled: bool }
```

**CLAIM.** A `PathPlacement` is a point list, not a tessellated path. Caching it removes none of the
cost the ADR adopted the cache for, and the value it caches is already in memory.

**ARGUMENT.** The whole reason for the cache is one sentence of the ADR and one of the contract:
`PathBuilder::build()` tessellates on each call, so a scroll must reuse the tessellated path.
`PathCache` holds no tessellated path. The architecture never writes `Path<Pixels>` and names
`PathBuilder` once, inside the framework-name list at `architecture.md:347`. No section states where
`PathBuilder::build()` is called or what holds its result.

**The cached value also saves no memory read.** `SystemPlacement::paths` is a `Vec<PathPlacement>`
that the engrave result already carries (`architecture.md:7736`), and `ComposeView` owns those
results. So a second, keyed copy of the same point list is the whole gain.

**The consequence is the one ADR 0006 named.** On every frame that paints a waveform, the element
tessellates again. Design contract 3.3 states the cost, B4 bounds a whole visible frame at 8.0 ms,
and chunk K3 writes the element and the cache from this text. No bench measures it: K4 benches a Mix
frame and K5 benches a Master frame, and neither one paints a wrapped lane.

**No guard sees it.** PG4 reads a declared name, PG20 reads an edge, and PG24 reads a size. No rule
reads a declaration against the decision record that commissioned it.

**EVIDENCE.** `architecture.md:7808` to `:7826`, `:9888` to `:9889`, `:7736`, `:347`, `:899` (B61),
`:11134` (Appendix A), `:8608` (chunk K3); `adr/0006-wrapped-timeline.md:41` to `:53` and `:90`;
`design-contract.md:563`.

**WHAT SHOULD CHANGE.** Declare the cache over the tessellated value, add `Path` to the framework
block so `crates/duet` may name it, and state which call site builds one. If the design prefers to
cache the point list, rewrite ADR 0006 decision 6 and contract 3.3, and state what pays the
tessellation cost per frame.

### WARNING: C15-3. One swapped row in the `audio-owned` block disarms all three TH1 rules

**OBSERVATION.** The block sits at exactly its stated minimum, and its membership kind asks one
thing:

```
architecture.md:1458  audio-owned         declared-name
architecture.md:5213  <!-- GUARD BLOCK id=audio-owned rows>=37 -->
```

`declared-name` "asks for a Rust block" (`architecture.md:1421`). The site states the completeness
argument:

```
architecture.md:5208  ... A run whose parsed set is below the recorded minimum is a failure, so a
                      deletion cannot pass as a clean run.
```

**CLAIM.** A substitution is not a deletion. One row exchanged for another declared name takes a
root out of PG26, PG26b, and PG26c, and every counter stays at the baseline.

**ARGUMENT.** I replaced `Transport` with `ChannelConfig` in the block and changed nothing else.

```
MINE-8b  one-for-one root swap, no other change
         exit 0   BLOCKS: 35   MEMBER ROWS: 750   MEMBER BAD: 0
                  AUDIO OWNED: 37   LOCK IN AUDIO: 0

MINE-9b  the same swap plus `probe_lock: Mutex<u32>` on `Transport`
         exit 0   BLOCKS: 35   MEMBER ROWS: 750   MEMBER BAD: 0
                  AUDIO OWNED: 37   LOCK IN AUDIO: 0

MINE-10b control: the same `Mutex` with the block untouched
         exit 1   LOCK IN AUDIO: 1
```

`Transport` is a root that no other root reaches. `GraphState` reaches `ChainSetHandle`, `ChainSlot`,
`ChainState`, `PoolHandle`, and `HandoffReader`. It does not reach `Transport`, `MeterSnapshot`,
`ParamSnapshot`, `MidiRecord`, `EngineFault`, or `RefillRequest`. Each of those is an independent
root, and each one loses all three rules when its row leaves.

This is the WR-18 class that `block-members` was built to close: "one added word took a type out of
four rules". The kind chosen for this block answers whether a row names a type; it cannot answer
whether that type is one the audio thread owns, and the site's own completeness argument names
deletion alone.

**The two name blocks are stronger, and they show the answer.** I replaced `Mutex` with a token that
names nothing in `lock-names`, and MINE-15 still reds, because PG26c also reads the word "lock" in
the section 1.9 `Why` cell. Two sources beat one. The root set has one source.

**EVIDENCE.** `architecture.md:1421`, `:1458`, `:5204` to `:5252`; my runs of MINE-8b, MINE-9b,
MINE-10b, MINE-1, and MINE-15.

**WHAT SHOULD CHANGE.** Give the block a second source, as `ROSTER ITEMS` and `ROSTER IMPL BLOCKS`
each have. Name the roots in section 5.6 beside `GraphRunner::run` and let PG26 hold the two lists to
one set, or state at the site that a substitution passes and that review holds it.

### WARNING: C15-4. The conversion guard passes on a scanned set that one manifest line shrinks

**OBSERVATION.** The guard prints a count and never a floor:

```
MEMBERS: 3   FILES: 6   FINDINGS: 0
```

CG1's own site states the limit and names no denominator:

```
architecture.md:2144  **CG1's denominator is the members list, and that is the limit.**
```

**CLAIM.** "Found 0 casts" and "scanned 0 files" print the same success. One `exclude` line removes a
whole crate from the scan, and the guard reports success with a bare `as` cast on disk.

**ARGUMENT.** I added `exclude = ["tools/xtask"]` to the workspace manifest of a throwaway copy and
left a real cast in `tools/xtask/src/main.rs`.

```
control, glob intact   exit 1   CAST: tools/xtask/src/main.rs: line 82
                                MEMBERS: 3   FILES: 6   FINDINGS: 1
after the exclude      exit 0   MEMBERS: 2   FILES: 4   FINDINGS: 0
```

Two more shapes give the same class. A member whose `src` directory is a symbolic link scans zero
files for that member and exits 0 with `FILES: 0`. A workspace with no member exits 0 with
`MEMBERS: 0`.

**The document already answered this class once and did not carry it across.** PG25 gained `FLOOR`
because a deleted declaration shrank a count and the run stayed green (critic C-3), and concern N-3
gave the impl count a second floor. The conversion guard holds the whole `as`-cast policy of the
workspace, CG6 refuses a suppression outside one file, and it has neither floor.

**CG8 does not cover it.** CG8 is fail-closed on a manifest that does not parse, on a file that is
not UTF-8, and on a file that does not open. An empty scan is none of the three.

**EVIDENCE.** `architecture.md:2144` to `:2154` (CG1), `:2272` to `:2288` (CG8), `:1220` and
`:1230` (the CP1 and CP8 rows); my runs of the exclude shape, MINE-C3, and MINE-C8.

**WHAT SHOULD CHANGE.** Give CG1 a floor. Record the member count and the file count of this
repository, and exit 2 when a run falls below either one. Add a probe that plants the shrunk set.

### WARNING: C15-5. B95 carries two periods, and the row that owns the id states the wrong one

**OBSERVATION.** Four sites give the retry a period of B7, which is 2.67 ms.

```
architecture.md:933   | B95 | 3 attempts | ... tries again on the next pass of its own loop, one
                      attempt per B7 ...
architecture.md:5702  | The engine handoff thread pushes a handoff into the B105 ring | B95
                      attempts, one per B7 | ...
architecture.md:10150 /// and tries again on the next pass of its own loop, one attempt per B7.
architecture.md:11146 ... the next pass of that thread's own loop tries again, one attempt per B7.
```

Three sites give it a period of B108, which is 50 ms.

```
architecture.md:4573  /// Consecutive passes on which the B105 slot was not free, against B95.
                      /// The period between passes is B108 ...
architecture.md:4635  ... The period between passes is B108, so the cap is a stated time as well as
                      a stated count.
architecture.md:11946 ... `attempts` caps the WAIT at B95 passes of B108 ...
```

**CLAIM.** The loop parks on one clock, and the row that owns the id names another. The stated bound
is wrong by a factor of nineteen.

**ARGUMENT.** The engine handoff thread parks on `Receiver::recv_timeout` at B108
(`architecture.md:4691` and `:5402`). One pass of that loop is therefore B108 and never B7. B.5 says
the same: the B95 row reads "`Instant::elapsed` across the bounded receive of that thread's own
loop" (`architecture.md:11366`), and the bounded receive is the B108 one.

So B95 is either 3 times 2.67 ms, which is 8 ms, or 3 times 50 ms, which is 150 ms. A user waits one
of the two before the engine reports `EngineFault::HandoffBusy` and stops taking changes. DR3 makes
the section 1.6 row the home of the number, and that row carries the value the mechanism refutes.

**EVIDENCE.** `architecture.md:933`, `:4573` to `:4575`, `:4631` to `:4635`, `:4691`, `:5402`,
`:5702`, `:10150`, `:11146`, `:11366`, `:11946`.

**WHAT SHOULD CHANGE.** Write B108 at all four B7 sites, in the B95 row first.

### WARNING: C15-6. One error variant names two opposite preconditions

**OBSERVATION.** `configure` and `hand_off` each declare the same variant, for opposite states.

```
architecture.md:4902  /// `ConfigError::HandoffRetryPending` when `handoff_retry` is already
                      /// `Some` (critic WR-27).
architecture.md:4927  /// Returns ... `ConfigError::HandoffRetryPending` when
                      /// `handoff_retry` is `None`, which means no caller built anything.
```

Two other sites state a mechanism that section 5.5 deletes.

```
architecture.md:10157  /// A second `ConfigureCommand::Apply` arrived while `handoff_retry` was
                       /// already `Some`. One attempt is in flight at a time, so the core keeps
                       /// the request in `DuetCore::pending_configure` and re-sends it ...
architecture.md:8324   | `HandoffRetryPending` | ... The core never returns it to a client: it keeps
                       the request in `DuetCore::pending_configure` and re-sends it ...
```

Section 5.5 says the opposite:

```
architecture.md:4671  **`DuetCore::pending_configure` covers one case only**: a full B109 channel
                      ... It never answers a `HandoffRetryPending`, because the loop produces none.
```

**CLAIM.** A caller that matches on `HandoffRetryPending` cannot tell whether a configuration is
pending or whether none exists. Two sites then state the answer that section 5.5 removed.

**ARGUMENT.** The two preconditions are mutually exclusive. `handoff_retry` is `Some` or it is
`None`. One variant that means both carries no information, and the name states only the first. A
typed error whose arm cannot be acted on is an untyped error with extra ceremony.

The second half is WR-21 at two new sites. Section 15 is the roster that chunk C3 writes from, and
section 12.4 is the table that chunk K1 writes from. Both now carry the pre-revision-15 rule, and DR1
says a revision rewrites the sentence it changes.

**EVIDENCE.** `architecture.md:4671` to `:4673`, `:4896` to `:4904`, `:4925` to `:4929`,
`:8324`, `:10157` to `:10161`.

**WHAT SHOULD CHANGE.** Split the variant, or take one precondition off one function. Then rewrite
the 15.10 comment and the 12.4 row to the section 5.5 rule.

### WARNING: C15-7. A `configure` refusal has no stated outcome, and two re-send rules have no cap

**OBSERVATION.** Step 5 names two outcomes and no third.

```
architecture.md:4657  5. **The engine handoff thread reports the outcome.** On a successful push it
                      advances its `ChainLayout`, clears `pending_request` and `attempts`, and sends
                      `EngineEvent::GraphConfigured { generation }`.
                      Past B95 waiting passes it sends `EngineEvent::Fault` ...
```

Step 2 states five refusals that `configure` can return: `PoolExhausted`, `StripBudgetExceeded`,
`ParamBudgetExceeded`, `RingBudgetExceeded`, and `ChannelMismatch`. The core's field states a
re-send:

```
architecture.md:10365  /// The topology change the engine handoff thread has not taken yet, or
                       /// `None`. **Latest wins**, and the core re-sends it on
                       /// `EngineEvent::GraphConfigured` and on
                       /// `EngineEvent::ConfigureRefused`, so a refusal loses no user change
```

**CLAIM.** A refused configuration is re-sent with no cap, and no rule says what clears
`pending_request` on the handoff thread. `EngineEvent::ConfigureRefused` appears twice in the whole
document: in its own declaration and in that comment.

**ARGUMENT.** Trace one `RingBudgetExceeded`. Step 1 validates on the core thread against B57, but
`ChainLayout::pairs` lives on the handoff thread and carries the retired array the collector has not
drained. The two checks therefore disagree exactly when the refusal is real.

The handoff thread refuses. No step clears `pending_request`, so the next pass builds the same
request. `attempts` does not rise, because the slot is free, so B95 never fires. The core also holds
the request in `pending_configure` and re-sends it on `ConfigureRefused`. Both loops run at one pass
per B108, and neither carries a bound.

This is the shape C.18 closed for the wait path: "an unbounded refuse-and-resend loop between the
core and the engine handoff thread". The wait path is now a wait. The refusal path is not covered.

Section 12.4 gives every refusal a user message, and no step sends one. `EngineEvent::Fault` carries
an `EngineFault`, and no `EngineFault` arm carries a `ConfigError`.

**EVIDENCE.** `architecture.md:4635` to `:4642`, `:4657` to `:4665`, `:4671` to `:4675`,
`:8228` to `:8258` (`EngineFault`), `:10111` to `:10113`, `:10365` to `:10370`, `:8319` to `:8325`.

**WHAT SHOULD CHANGE.** Add a sixth step, or a third outcome to step 5: on a refusal the thread
drops `pending_request`, sends `EngineEvent::ConfigureRefused(error)`, and clears `attempts`. Then
state that the core clears `pending_configure` on a refusal rather than re-sending it, and name the
one case that does re-send.

### WARNING: C15-8. The pool reaches the audio thread through a method that takes no argument

**OBSERVATION.** Section 5.5 states the one path:

```
architecture.md:4724  **How the pool reaches the audio thread, once** ... It hands that one value to
                      the backend at `AudioBackend::start`, which moves it into the callback state.
```

The trait says which method takes a value:

```rust
architecture.md:3883  fn open(&mut self, request: &StreamRequest, process: Box<dyn AudioProcess>)
                          -> Result<StreamInfo, BackendError>;
architecture.md:3888  fn start(&mut self) -> Result<(), BackendError>;
```

**CLAIM.** `start` takes no argument, so it cannot move a `GraphState` anywhere. The stated path
names the wrong method, and the thread it names holds no way to reach the backend at all.

**ARGUMENT.** `open` is the method that takes the `Box<dyn AudioProcess>`. So the sentence is false
as written.

The larger half is the input. `GraphConfigurator` declares seven fields: `collector`, `commands`,
`handoffs`, `topology`, `layout`, `pending_request`, and `attempts`. None of them reaches an
`AudioBackend`, a `StreamRequest`, or a `StreamInfo`. `ConfigureCommand` has two arms, `Apply` and
`Reset`, and neither one opens or closes a stream.

So the thread that the document says builds the first `GraphState` and calls the backend has no
declared way to learn that a stream opened, and no handle to call. The restart paragraph makes the
gap concrete: "A restart therefore builds a new pool, a new first chain array, and a new B105 pair"
(`architecture.md:4700`). `handoffs` is a `Producer<GraphHandoff>` and not an `Option`, so a restart
must replace it, and no command carries a replacement.

This is the answer C.18 recorded for "The buffer pool has no declared path to the audio thread". The
path is named and it is not declared.

**EVIDENCE.** `architecture.md:3881` to `:3893`, `:4538` to `:4576` (`GraphConfigurator`),
`:4520` to `:4536` (`ConfigureCommand`), `:4696` to `:4702`, `:4724` to `:4729`, `:11949`.

**WHAT SHOULD CHANGE.** Write `AudioBackend::open`. Then give `GraphConfigurator` the input it
needs: a third `ConfigureCommand` arm that carries the `StreamRequest`, or a field that holds the
backend, and state which one owns the stream.

### WARNING: C15-9. Push before publish makes the skew fault reachable, and the declaration says it is not

**OBSERVATION.** Step 3 fixes the order and gives the reason:

```
architecture.md:4643  3. **The engine handoff thread pushes the `GraphHandoff` at B105 and then
                      publishes the new `GraphChain`.** The push happens first, so the audio thread
                      never sees a raised `handoff_generation` with no handoff behind it.
```

The fault declaration states that the opposite window does not exist:

```
architecture.md:8247  /// This design makes the state unreachable, and the runner refuses to assume
                      /// it (section 5.6, critic CR-20).
```

**CLAIM.** Push before publish removes one window and opens the other. The audio thread can adopt a
handoff whose topology is not yet published, so the skew state is reachable.

**ARGUMENT.** The two statements are two instructions on the handoff thread. The audio callback runs
on another core, every B7. Between the push and the publish it can resolve the topology at
generation N, then drain the ring and adopt handoff N plus 1.

Section 5.6 states the callback order: "The callback resolves the topology before it calls
`GraphRunner::run` and hands it in as a shared reference" (`architecture.md:5095`). The resolve
therefore precedes the drain inside one cycle, which is exactly the window.

Step 2 of the drain rule then compares the adopted generation with the published one, finds a
difference, and reports `EngineFault::HandoffGenerationSkew`. The runner "runs the array it actually
holds" (`architecture.md:5123`) against a topology of the previous length. A strip with no state
gives `ChainSlotVacant` and one silent cycle; a state with no topology entry is never run.

The cost is one cycle and one fault, so the failure is contained. The claim that the state is
unreachable is still false, and it is the claim a reader uses to decide that the fault path needs no
test. No test in section 14 publishes a topology out of step with a handoff.

**EVIDENCE.** `architecture.md:4643` to `:4649`, `:5094` to `:5098`, `:5104` to `:5131`,
`:8244` to `:8253`.

**WHAT SHOULD CHANGE.** Delete "This design makes the state unreachable" and state the window that
the push-before-publish order creates. Then add a test that opens it, or publish and push under one
sequence the audio thread can read atomically.

### WARNING: C15-10. The frame pump has three start terms and the monitor path needs a fourth

**OBSERVATION.** `FrameDemand` carries three fields, and the meter rows read two of them:

```
architecture.md:7199  | `MeterLayer` | `any_armed` or `transport_moves` | both false and
                      `meter_decays` false | ...
architecture.md:10506  /// At least one strip is armed, so an input level can change with the
                       /// transport stopped.
```

The session model carries a second way for an input level to change:

```rust
architecture.md:2978  pub enum MonitorMode { Off, WhenArmed, Always }
```

**CLAIM.** A user who monitors an input with `MonitorMode::Always`, with no strip armed and the
transport stopped, sees a meter that stops moving. That is the CR-21 symptom on a second path.

**ARGUMENT.** The audio stream runs whenever the engine is `Running`. The transport state does not
stop the callback. `MonitorMode::Always` passes the track input to the monitor bus whatever the arm
state is, and `MeterTap::Input` is a declared tap (`architecture.md:5990`), so a strip meter shows a
live input level.

At the first frame of silence all three terms are false: no strip is armed, the transport does not
move, and no bar and no held mark differ from silence. `poll_frame` re-arms nothing, no driver
requests a frame, and the loop ends. The user then sings and the meter does not move.

Section 10.2 states the product reason for the `any_armed` term: "the peak cap, the fall, and the
`CLIP` indicator all exist so a user can set a record level before the transport rolls". A user who
sets a level through the monitor bus has the same need and no term.

PR R-04 makes input selection and monitoring a MUST, and chunk K3 writes "the input and monitor
controls". Test 13 arms a track and disarms it; it does not set a monitor mode.

**EVIDENCE.** `architecture.md:2977` to `:2978`, `:5792` (`Track::monitor`), `:5990` (`MeterTap`),
`:6018` (the monitor bus), `:7196` to `:7210`, `:7962` to `:7970` (test 13), `:10502` to `:10513`.

**WHAT SHOULD CHANGE.** Add a fourth term, `any_monitoring`, to `FrameDemand`, and add it to the
start condition of both meter rows. Then extend test 13 with a monitor mode and no arm.

### WARNING: C15-11. The contract's peak cap has no field, no bound, and no owner

**OBSERVATION.** Design contract 4.4 gives the meter two peak behaviours.

```
design-contract.md:762  - **Peak** paints as a 2 px horizontal cap line across the full width, in
                        `duet.meter.peak_cap`. It holds for 1200 ms, then falls at 20 dB per second.
design-contract.md:768  - A numeric peak readout ... It shows the highest peak since the last reset
```

The architecture answers the second one:

```
architecture.md:6225  ... clears that strip's `MeterReading::hold`, which is the "highest peak since
                      the last reset" that design contract 4.4 names
```

**CLAIM.** The hold time and the fall rate of the cap line appear nowhere in the architecture. No
field carries the cap, no B row carries either number, and no section names the owner.

**ARGUMENT.** I searched the whole document for "1200" and for "20 dB per second". Both return
nothing. `MeterReading` holds `peak`, `rms`, `true_peak`, and `hold`, and `hold` is bound to the
readout. B106 carries the meter scale and no timing.

So the decaying cap is a MUST of the design contract with no thread, no type, and no budget. DR3
makes section 1.6 the one home of every number this design chooses, and two numbers of a MUST
behaviour are not there. The fourteenth review named the fall rate in the CR-21 argument, and
revision 15 answered the start condition and not the behaviour.

The owner matters. The cap decays per frame, so either the audio thread computes it and publishes it,
which TH9 constrains, or `MeterReader::poll` computes it on the foreground thread from the frame
clock. The two choices differ in what `MeterSnapshot` must carry.

**EVIDENCE.** `design-contract.md:752` to `:775`; `architecture.md:6118` to `:6140` (`MeterReading`),
`:6220` to `:6230`, `:944` (B106); a whole-document search for both numbers.

**WHAT SHOULD CHANGE.** Give the hold time and the fall rate a B row each. Then name the owner and
the field, and say whether `MeterSnapshot` carries the cap or `MeterReader` derives it.

### WARNING: C15-12. The roster's one undocumented declaration is `CoreHost`

**OBSERVATION.** Two doc-comment blocks run together, and the declaration between them is gone.

```
architecture.md:10485  /// The entity that owns `DuetCore`, drains its event channel, and holds the
architecture.md:10494  /// ends where two exist (critic CR-11).
architecture.md:10495  /// Why a frame driver runs this frame.
architecture.md:10502  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
architecture.md:10503  pub(crate) struct FrameDemand {
...
architecture.md:10515  #[derive(Debug)]
architecture.md:10516  pub(crate) struct CoreHost {
```

**CLAIM.** `CoreHost` carries no doc comment, and `FrameDemand` carries two contracts. I scanned
every declaration in the roster: `CoreHost` is the only one of the 395 with no doc comment.

**ARGUMENT.** `clippy::missing_docs_in_private_items` is denied and `.cargo/config.toml` makes every
warning an error, so the declaration as written does not build. Chunk K1 writes `CoreHost` from this
roster.

**No guard sees it, and the reason is stated in the document.** The PG25 profile relaxes exactly the
two lints that would catch it: "`missing_docs` and `clippy::missing_docs_in_private_items`, because
the roster carries declarations and not documentation" (`architecture.md:689`). Substitution S1 then
removes every whole-line comment before the compiler reads the roster, so the merge is invisible on
both paths.

The second half is the contract. `FrameDemand` now opens with a paragraph about `DuetCore`, the event
drain, and two read ends. A plan author who writes the K1 brief from this roster copies that text
onto the wrong type.

**EVIDENCE.** `architecture.md:10484` to `:10516`, `:688` to `:697` (the relaxed profile),
`:1695` (substitution S1); root `Cargo.toml`, `missing_docs_in_private_items = "deny"`; my scan of
every declaration in the roster.

**WHAT SHOULD CHANGE.** Put the blank line back and split the two doc comments.

### WARNING: C15-13. ADR 0004 rejects the `pipewire` crate on a reason its own sources refute

**OBSERVATION.** The rejected-alternatives table of ADR 0004 states one reason first:

```
adr/0004-backend-and-threading-contract.md:294  | The `pipewire` crate as the audio backend, in
                                                place of cpal | Its stream buffer path is `unsafe`,
                                                and `unsafe` is denied. ...
```

The research file says the opposite:

```
research/linux-macos-platform.md:19  Stream `process` with `dequeue_buffer`, `datas_mut`,
                                     `Data::data` is safe; only the raw `pw_buffer` path is `unsafe`
                                     and not needed.
```

Appendix B.2 of the architecture agrees with the research:

```
architecture.md:11275  The `pipewire` registry listener and the `pipewire` stream API that section
                       8.1 and section 5.4 use are safe Rust in the caller. Only the raw `pw_buffer`
                       path and `pw::deinit()` are `unsafe`, and neither is needed
```

**CLAIM.** A locked decision rests on a stated fact that the source of record and the architecture's
own appendix both refute.

**ARGUMENT.** DR6 makes an ADR a citation and not a second copy of a fact. This row states a fact,
and the fact is false. The operator ruled that `unsafe` stays denied and that nothing in the plan may
require it, so "its path is unsafe" is the strongest possible reason and it is the wrong one.

The decision may still stand: the same cell gives a second reason, that cpal's PipeWire host wraps the
same server behind a safe API. A reader who checks the first reason finds the contradiction and then
doubts the second.

**EVIDENCE.** `adr/0004-backend-and-threading-contract.md:294`;
`research/linux-macos-platform.md:19`; `architecture.md:11272` to `:11278`.

**WHAT SHOULD CHANGE.** Delete the unsafe claim from that cell and keep the reason the research
supports. Correct the Appendix B.2 sentence in the same edit: section 5.4 uses cpal's PipeWire host
and calls no `pipewire` stream method; section 8.1 is the one caller.

---

## 3. Concerns

**N15-1. Section 1.9 states three counts that the run below it contradicts.** Line 1274 writes
"`AUDIO DEFERRED` rose from 12 to 23" and the run prints 33. Line 1276 writes "`AUDIO OWNED` rose
from 33 to 36" and the run prints 37. Line 1277 writes "`BLOCKS` rose from 31 to 32" and the run
prints 35. DR3 exemption 4 makes a recorded count a measurement, and these three are hand numbers
beside the measurement that refutes them.

**N15-2. Section 1.7 sends a reader to 1.5 for two rules that 5.7 states.** Line 1081 names PG26b and
PG26c with `1.5` in the `Stated in` cell. Section 1.5 declares neither; the two rules are stated at
lines 5183 and 5193. Appendix C rows at lines 11907 and 11908 cite "1.5 PG26b" and "1.5 PG26c".

**N15-3. Appendix C.16 names an arm that does not exist.** Line 11901 writes "a migration is a plan
of `Keep` and `Live` arms". `ChainSource` declares `Adopt` and `Keep`.

**N15-4. `ConfigError::PoolHandoffBusy` is unreachable and still carries a user message.** Step 3
states that "the push cannot meet a full ring". Section 12.4 line 8322 gives it the message "Start
audio, then try again", which describes the revision-13 retry.

**N15-5. `impl-sites` is a floor and not a set.** My MINE-R3 added an impl block that the block does
not list, and the roster exited 0 with `ROSTER IMPL BLOCKS: 50` against `FLOOR: 49`. The ownership
table and the declarations are one set in both directions; this pair is one-directional.

**N15-6. "not yet seen released" names an observation the loop cannot make.** `Collector::collect`
returns nothing, so the loop learns nothing about which node it freed. The count is bookkeeping, and
the two write points depend on the audio thread queueing its drop before the next drain.

**N15-7. Two rule lists run out of order.** TH10 is stated at line 5363 and TH9 at line 5367. CG5 is
stated before CG4b.

**N15-8. Two shapes escape the conversion guard.** A `#[path = "../outside.rs"]` module above the
crate root is not scanned. A member whose `src` is a symbolic link to a directory scans zero files.

**N15-9. The `not-declared` kind accepts a token that names nothing.** I replaced `Mutex` in
`lock-names` with a token that names nothing and the block passed. The site states the limit; the
probe records it.

**N15-10. ADR 0004 clause 22 states an absolute that four lines of the same record break.** The
clause says "no version number and no runner label appears anywhere in this record". Lines 13, 41,
110, and 235 write 0.18.2 twice, 0.1.3, and 0.6.4. No runner label appears.

**N15-11. Two ADRs restate TH3 and neither matches it.** ADR 0004 clause 11a drops "inside an
IMMEDIATE verb". ADR 0005 decision 15 omits the timed wait and the B110 allocation, which are the
CR-15 and CR-19 answers.

**N15-12. ADR 0006 states a wrong field list.** Line 86 writes that a segment holds "a beat span and
the map, and nothing else". `TakeSegment` carries four fields, `region` and `system` included.

**N15-13. The `Used by` column claims more than PG30 measures.** The column's site says it names
every section outside 1.6 that writes the id. Three rows name an ADR and forty do not, and PG30's
stated limit is that it reads this document alone.

**N15-14. Two rows claim the same contract minimum.** B102 is "The minimum window width, from
contract 1.1" at 1024, and B104 is "The width below which contract 1.1 refuses to shrink" at 900.
The contract states both in three lines.

**N15-15. Appendix C row C14 records a finding the contract closed.** Line 11724 says the
`StageCurve` appendix names `duet.meter.peak`. It now names `duet.meter.peak_cap`.

**N15-16. One version is written two ways.** Lines 7983 and 9061 write "macOS 26". Line 8058 and the
two table cells write "26.0".

**N15-17. Four SHOULD stories exist only in one sentence.** Line 8737 states "Each one has a chunk".
MA-05, R-12, X-14, and C-12 have no type, no verb, no file, and no goal-list entry outside the
assignment line. X-14 is the sharpest: design contract section 9 holds six accessibility
requirements and the K1 goal names none.

**N15-18. Eleven framework names of the design contract have no home.** Section 10.3 gives three
names a home and states that class. `ToggleGroup`, `NumberInput`, `Spinner`, `SidebarHeader`,
`SidebarGroup`, `resizable_panel`, `v_resizable`, `flex_1()`, `min_w_0()`, `canvas()`, and
`Window::paint_glyph` are in the same class. `v_resizable` matters most, because contract 1.6 gives
Mix one and the architecture answers only the persisted fraction.

---

## 4. Every probe I ran, and its result

### 4.1 The three guards on the unmodified document

| Guard | Result |
|---|---|
| `placement_check.py architecture.md` | Exit 0. Every one of the 33 counter lines matches section 1.9, character for character. |
| `conversion_check.py`, from the repository root | Exit 0. `MEMBERS: 3   FILES: 6   FINDINGS: 0`. |
| `roster_compile.sh architecture.md <scratch> <repo>` | Exit 0. `ROSTER ITEMS: 395     FLOOR: 395`, `ROSTER IMPL BLOCKS: 49     FLOOR: 49`, `ROSTER IMPLS: 21`, `ROSTER CLIPPY: clean`, `ROSTER SIZES: 419 measured`, `SIZE BAD: 0`. |

### 4.2 The three harnesses

| Harness | Result |
|---|---|
| `probe_run.py` | `PROBES BAD: 0` over 230 runs. Every row reproduces its recorded exit code and its recorded line. |
| `probe_conversion.py` | `CONVERSION PROBES BAD: 0`. Every shape reproduces. |
| `probe_roster.sh` | `ROSTER PROBES BAD: 0`. All eleven shapes emit the exact recorded compiler line. |

### 4.3 My own placement probes

| Probe | Shape | Result | Verdict |
|---|---|---|---|
| MINE-1 | `Mutex` in `lock-names` replaced by a token that names nothing | exit 0 | Stated limit, N15-9 |
| MINE-2 | The same replacement plus `probe_lock: Mutex<u32>` in `ChainState` | exit 1, `LOCK IN AUDIO: 7` | Caught, by the external `Why` word |
| MINE-3 | Control: `probe_lock: Mutex<u32>` in `ChainState` | exit 1, `LOCK IN AUDIO: 7` | Caught |
| MINE-4 | `probe_lw: Owned<Mutex<u32>>` in `ChainState` | exit 1, `LOCK IN AUDIO: 7` | Caught inside a wrapper |
| MINE-5 | `probe_g: Owned<SmallVec<[u8; 4]>>` in `ChainSlot` | exit 1, `GROW IN AUDIO: 6` | Caught |
| MINE-6 | `probe_lock: Mutex<u32>` in `ResetGenerations`, below the exempt field | exit 1, `LOCK IN AUDIO: 1` | Caught below an exemption |
| MINE-7 | `probe_grow: Vec<u8>` in the same place | exit 1, `GROW IN AUDIO: 1` | Caught below an exemption |
| MINE-8 | `Transport` swapped for `ChainLayout` in `audio-owned` | exit 1, `HEAP IN AUDIO: 1` | Caught, by accident: the new root holds a `Box` |
| MINE-8b | `Transport` swapped for `ChannelConfig`, a clean root | exit 0, every counter at the baseline | **Hole, C15-3** |
| MINE-9b | The same swap plus `probe_lock: Mutex<u32>` in `Transport` | exit 0, `LOCK IN AUDIO: 0` | **Hole, C15-3** |
| MINE-10b | Control: the same `Mutex` with the block untouched | exit 1, `LOCK IN AUDIO: 1` | Caught |
| MINE-11 | The exempt row head changed from `Arc` to `Vec` | exit 1, `GROW IN AUDIO: 1` | Caught |
| MINE-12 | `probe_bare: Producer<f32>` in `DiskWriter`, with no wrapper | exit 1, `HEAP IN AUDIO: 3` | Caught |
| MINE-13 | A `B108` citation planted in section 9.6 | exit 1, `USED BY BAD: 1` | Caught |
| MINE-14 | `9.9` added to the `Used by` cell of B107 | exit 1, `USED BY BAD: 1` | Caught |
| MINE-15 | The `lock-names` row swapped AND the word "lock" removed from the external row, plus a `Mutex` | exit 1, `LOCK IN AUDIO: 1` | Caught, by the `lock-names` block |

MINE-3, MINE-10b, and MINE-12 are the paired controls. MINE-13 and MINE-14 prove PG30 in both
directions on this revision's own text. MINE-2 and MINE-15 together show that the lock rule reads two
sources, so one edited source does not disarm it.

### 4.4 My own conversion probes

| Probe | Shape | Result |
|---|---|---|
| MINE-C1 | A cast split across a line break | exit 1, `CAST: m/src/lib.rs: line 2` |
| MINE-C2 | A cast in `build.rs`, outside `src/` | exit 1, `FILES: 2` |
| MINE-C3 | A member whose `src` is a link to a directory holding a cast | **exit 0**, `FILES: 0` |
| MINE-C4 | A cast-shaped string inside a `cfg_attr` | exit 0, `FINDINGS: 0` |
| MINE-C5 | A cast reached by `#[path = "../outside.rs"]` | exit 0, `FINDINGS: 0` |
| MINE-C6 | A cast inside a `macro_rules!` body beside a `use` pattern | exit 1, `FINDINGS: 1` |
| MINE-C7 | A `[lib] path` outside `src/`, holding a cast | exit 1, `FINDINGS: 1` |
| MINE-C8 | Denominator: a workspace with no member | **exit 0**, `MEMBERS: 0` |
| MINE-C9 | The repository copy with `tools/xtask` excluded, and a real cast in it | **exit 0**, `MEMBERS: 2   FILES: 4` |

MINE-C9 ran against a control: with the glob intact the same cast reds with
`CAST: tools/xtask/src/main.rs: line 82`. MINE-C3, MINE-C8, and MINE-C9 are one class, C15-4.

### 4.5 My own roster probes and my three gate defects

Each plant sits in text revision 15 wrote, and each one ran alone.

| Plant | Site | Result |
|---|---|---|
| MINE-R1: a recorded size the compiler contradicts | `ArrayVec<SlotState,MAX_SLOTS>` set to 1000 | **exit 1**; `SIZE BAD: 1` |
| MINE-R2: one row deleted from `impl-sites` | the `HandoffReader` row | **exit 2**; ``FAIL: the `impl-sites` block ... it holds 48 rows and the stated minimum is 49`` |
| MINE-R3: an impl block `impl-sites` does not list | `impl ChainIndex` added | exit 0; `ROSTER IMPL BLOCKS: 50     FLOOR: 49`. Concern N15-5 |
| MINE-R4: a revision-15 declaration renamed | `ChainIndex` | **exit 1**; `FAIL: the roster holds 394 items and section 1.5 places 395; the first name with no declaration is ChainIndex.` |
| GATE-A: `#[allow(clippy::struct_field_names)]` | `ChainSlot`, section 5.5 | **exit 1**; ``duet-engine/src/lib.rs:120:3: error: #[allow] attribute found: help: replace it with: `expect` `` |
| GATE-B: `.unwrap()` in a spelled-out body | `impl core::fmt::Debug for HandoffReader`, section 5.5 | **exit 1**; ``duet-engine/src/lib.rs:134:26: error: used `unwrap()` on `Some` value`` |
| GATE-C: a bare `as` cast in a spelled-out body | `impl core::fmt::Debug for PoolHandle`, section 5.5 | **exit 1**; the build stops at the same site |

My first attempt at GATE-B and GATE-C tripped `let_underscore_untyped` and `unnecessary_cast` before
the intended lint, which is the masking trap. I re-planted both with a typed binding and a used
value, and `unwrap_used` then answered GATE-B by name. PG25 catches all three of my gate defects at
three sites the Architect did not use.

### 4.6 The pinned-source citations

I checked every citation this revision added. Every one is exact:
`rtrb-0.4.0/src/lib.rs:364` (`slots`), `basedrop-0.1.3/src/collector.rs:238` (`collect`), `:97`
(`queue_drop`), `src/owned.rs:34`, `src/shared.rs:41`, `gpui-pre-0.3.5/src/window.rs:2606` to
`:2608`, `:2586`, `src/view.rs:232` and `:266` to `:271`, `src/app/context.rs:177` and `:293`,
`src/app/entity_map.rs:476`, and `gpui-component-0.6.4/src/theme/mod.rs:193`. The `Owned` and
`Shared` trait lists are correct: `Owned` declares `Deref`, `DerefMut`, `Clone`, and `Drop`, and
neither declares `Debug`. The `collector.rs:160` citation points three lines above the sentence it
quotes.

**No unsafe code is required on this path, and I checked it.** `basedrop` supplies
`unsafe impl<T: Send> Send for Owned<T>` at `owned.rs:21`, and `rtrb` supplies
`unsafe impl<T: Send> Send for Producer<T>` at `lib.rs:311`. `ChainSetHandle` is therefore `Send`
with no impl of our own, and the operator's rule holds.

I also tested the one line section 10.2 mandates:
`self.core.update(cx, |host, cx| host.poll_frame(window, cx))`. A closure parameter that shadows
`cx` trips `clippy::shadow_reuse`, which this workspace does not deny, and not
`clippy::shadow_unrelated`, which it does. The line is lint clean.

---

## Reactive Assessment

- **Responsive: PARTIAL.** The allocation left the thread that paints, TH3 carries B110, and the
  frame pump has a start condition whose premise I verified in the pinned source. Two paths remain.
  The path cache removes none of the tessellation cost that ADR 0006 adopted it for (C15-2), and an
  input meter freezes while a user monitors without arming (C15-10).
- **Resilient: PARTIAL.** Every failure surface is a typed enum and no crate holds `unsafe`. The
  migration needs no copy, the audio thread frees nothing, and `Option::take` is sound because a
  plan is applied in full. One variant carries two opposite preconditions (C15-6), a refusal has no
  stated outcome (C15-7), the pool reaches the audio thread through a method that takes no argument
  (C15-8), and one root can leave the TH1 guard set with every counter unchanged (C15-3).
- **Elastic: FAIL.** The memory model is a formula and the arithmetic holds. The duty it counts has
  two owners, so the count is incomplete by construction (C15-1). A refused configuration re-sends
  with no cap on either side of the seam (C15-7).
- **Msg Driven: PASS.** Each publication has one read end and one owner. One thread owns `configure`,
  the ring, the publication, and the collector. No lock and no `Global` crosses a view seam, and
  every wait sits on a thread that paints nothing.

---

## Verdict

**NOT READY.** Revision 15 is the strongest revision this plan has produced. It closes every
Critical of revision 14, it closes the design half of CR-16, and the method that produced it, two
inner Critic passes before the freeze, is the method that should have produced revision 14. I
reproduced all three guards and all three harnesses on this machine, and every pinned-source
citation it added is exact.

**The single biggest risk is a duty with two owners.** Ring allocation belongs to the engine handoff
thread in section 5.5 and to the engine disk thread in TH4, in the disk row of the same table, and in
ADR 0004. The B56 formula counts the pairs one of them allocates. Two chunks in two phases build it.
That is CR-19 solved in one sentence and left standing in three.

The second is a cache that caches the wrong value. `PathCache` holds a point list where a locked ADR
decision, a design-contract MUST, and a budget row all name a tessellated path.

The weakest Reactive property is **Elastic**. The memory bound rests on a count that one of two
owners keeps, and one refusal path resends without a cap.

I also record what improved. The `ChainSlot` plan is the right shape: a move is a move, the type
says so, and no `Clone` is needed. PG26b and PG26c are real rules with real probes, and my six
lock-and-grow probes all red, inside a wrapper and below an exemption alike. The union of a block
and a `Why` word gives the three name sets two sources, which is why one edited block did not disarm
them. That pattern is the answer to C15-3.

### The blocking list

1. **C15-1** Ring allocation has two owners: the engine disk row and TH4 against section 5.5 and the
   engine handoff row. `ChainLayout::pairs` counts only one of them, and B57 rests on that count.
   Chunks C2 and C3 both build it.
2. **C15-2** `PathCache` holds `Arc<PathPlacement>`, a point list. ADR 0006 decision 6, design
   contract 3.3, and B61 all name a tessellated `Path<Pixels>`, and the reason they give is the
   `PathBuilder::build()` cost that the declared cache does not remove.
3. **C15-3** A one-for-one swap in the `audio-owned` block takes a root out of PG26, PG26b, and
   PG26c. My MINE-9b exits 0 with a `Mutex` on `Transport` and every counter at the baseline.
4. **C15-4** The conversion guard has no denominator floor. My run exits 0 with `MEMBERS: 2` and a
   real cast on disk, after one `exclude` line.
5. **C15-5** B95 carries two periods. Four sites write B7 and three write B108, and the row that
   owns the id writes B7.
6. **C15-6** `ConfigError::HandoffRetryPending` names `handoff_retry` as `Some` at one site and as
   `None` at another. Sections 12.4 and 15.10 both state the mechanism section 5.5 deletes.
7. **C15-7** The five steps state no outcome for a `configure` refusal, nothing clears
   `pending_request`, and the core re-sends on `ConfigureRefused` with no cap.
8. **C15-8** The pool path names `AudioBackend::start`, which takes no argument. `GraphConfigurator`
   holds no backend, and `ConfigureCommand` has no arm that opens or closes a stream.
9. **C15-9** Push before publish makes `HandoffGenerationSkew` reachable, and the declaration says
   the design makes it unreachable.
10. **C15-10** `FrameDemand` carries three start terms. `MonitorMode::Always` is a fourth, and PR
    R-04 makes it a MUST.
11. **C15-11** The 1200 ms hold and the 20 dB per second fall of design contract 4.4 appear nowhere
    in the architecture.
12. **C15-12** `CoreHost` is the one declaration of 395 with no doc comment, and the PG25 profile
    relaxes both lints that would catch it.
13. **C15-13** ADR 0004 rejects the `pipewire` crate because its stream path is unsafe. The research
    file and Appendix B.2 both say that path is safe.

### The Concerns, for the Orchestrator to file

| Id | One-line disposition |
|---|---|
| N15-1 | Rewrite the three counts at lines 1274 to 1278 from the recorded run, or delete the sentence. |
| N15-2 | Declare PG26b and PG26c in section 1.5, or change the 1.7 cell and the two C.16 rows to 5.7. |
| N15-3 | Write `Adopt` for `Live` in the C.16 CR-17 row. |
| N15-4 | State at the `PoolHandoffBusy` site that the loop produces none, and rewrite the 12.4 message. |
| N15-5 | Hold `impl-sites` and the parsed impl blocks to one set, as PG25 holds the substitution block. |
| N15-6 | Replace "not yet seen released" with the bookkeeping rule the loop can run. |
| N15-7 | Move the TH9 paragraph above TH10, and the CG4b paragraph above CG5. |
| N15-8 | State at the CG1 site that a `#[path]` above the crate root and a linked `src` are outside it. |
| N15-9 | State at the three name blocks that `not-declared` proves a non-collision and not a referent. |
| N15-10 | Delete the four version numbers from ADR 0004, or narrow clause 22 to the runner label. |
| N15-11 | Write TH3 in ADR 0004 clause 11a and ADR 0005 decision 15 exactly as section 5.7 states it. |
| N15-12 | Delete the field list from ADR 0006 line 86 and cite the declaration. |
| N15-13 | Narrow the `Used by` sentence to this document, or add the ADR to all 43 rows. |
| N15-14 | Give B102 and B104 one sentence each that names which floor it is. |
| N15-15 | Close the Appendix C row for C14 against `duet.meter.peak_cap`. |
| N15-16 | Write `macOS 26.0` at lines 7983 and 9061. |
| N15-17 | Name the work for MA-05, R-12, X-14, and C-12 in the chunk goals, or drop the claim at 8737. |
| N15-18 | Add the eleven framework names to the section 10.3 paragraph that already holds three. |

### Commentary, which is not a finding

The `ChainSlot` design is the best mechanism this plan has produced. It turns a migration into data,
it needs no `Clone`, it needs no suppression, and the compiler proves the ownership. The Architect
chose a struct over an enum because the roster compile told him the enum tripped a lint. That is the
guard set doing the job it was built for, before a line of code exists.

The two-source pattern is the second. `heap-names`, `grow-names`, and `lock-names` are each read
beside a word in the external table, and my two disarm probes both failed because of it. Every allow
list in this plan should take that shape, and the `audio-owned` block is the one that still does not.
