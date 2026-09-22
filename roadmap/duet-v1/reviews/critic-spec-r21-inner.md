# Engineering Critic — specification review of revision 21

Scope: the frozen copy at
`/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/frozen-r21/duet-v1/`.

`architecture.md` md5 at the start of my run: `e42bc8b6640a5831d0342f5868e11fc7`.
`architecture.md` md5 at the end of my run: `e42bc8b6640a5831d0342f5868e11fc7`.
**The two agree.** I wrote no file in the frozen tree and no file in the repository. Every plant went
into a throwaway copy under my own scratch directory.

I did the engineering pass over sections 5, 6, 7, 9, 10 and 12 first, then the guard pass. The three
Criticals below all come from the engineering pass. The guard tooling is in good order: eleven of
the twelve gates hold against hostile input, and I record where each one held. One new rule, PG37,
is weak, and I measured how weak.

---

## CRITICAL 1. No declared path carries a take to the audio thread, so playback has no source

**OBSERVATION.** Section 5.10 states that the disk thread "fills each ring from
`media/<hh>/<hash>.wav` through `duet_media::SourceReader`". The value that reaches the disk thread
is `RefillRequest`, declared at line 11751:

```
pub struct RefillRequest { track: TrackId, channel: ChannelIndex, from: SampleClock, frames: FrameCount }
```

It names a track and a device sample position. It names no `SourceHash`, no `RegionId` and no take.
`DiskReader` (line 11773) holds `track`, `channels`, two ring ends and `position`, and nothing else.
`EngineProcess` (line 12018), which section 5.6 declares the real root of the audio thread, holds a
`GraphState`, three `triple_buffer::Output` values, a MIDI consumer, the refill producer, the fault
queue, a `Transport` and a `ChainRunner`. `ChainTopology` and `ChainState` (section 5.5) hold slots,
sends, a fader, a meter and two optional disk paths. Not one of these declarations holds a take, a
region, a source or a playlist. `duet_media::SourceReader` (line 11657) reads one file and holds one
`SourceHash`, so it answers "read this file", never "which file".

**CLAIM.** The specification declares no mechanism that turns `(TrackId, SampleClock)` into a source
file, a byte offset and a region envelope. The playback half of the product has no runtime.

**ARGUMENT.** Section 6.1 makes a take a playlist of regions over immutable sources, and gives each
`Region` a `source`, a `position`, a `start`, a `length`, a `gain`, a `fade_in`, a `fade_out`, an
`opaque` flag and a `layer`. Section 6.2 gives three record modes that decide how a new region
covers an old one. Every one of those facts must reach the sample path, because every one of them
changes the samples the user hears. Section 5.8 states that its tables are the one place each
boundary is decided, and no row of any of those tables carries a region, a take or a source across
any boundary. Section 5.7 names twelve threads and gives the disk thread the duty "ring refill and
drain, take files, peak writes", and no thread row owns the resolution. `SlotKind` holds eight arms
and the enum doc states that the set is closed and covers every tool the product stories name; none
of the eight applies a region gain or a fade. So three things are true at once: the disk thread must
open a file the request cannot name, the audio thread must apply an envelope no declaration carries,
and the topology change path — which is the only path that reaches `ChainState` — carries a
`ConfigureRequest` whose fields are `chains`, `order`, `stream` and `generation`. A chunk that
implements this specification exactly builds a mixer that runs over rings that nothing fills.

The same gap removes the answer to a locate. `TransportState::Locating { target }` exists at line
6393. Nothing states which thread invalidates each ring and refills it from the new position, and
`RefillRequest::from` is a `SampleClock`, so the disk thread cannot tell a locate from an ordinary
refill.

**EVIDENCE.** `architecture.md` lines 6493 to 6503 (section 5.10 playback), 11751
(`RefillRequest`), 11773 to 11781 (`DiskReader`), 12018 to 12040 (`EngineProcess`), 4778 to 4815
(`ChainState`), 6296 to 6380 (every section 5.8 boundary table), 6762 to 6773 (`Region`), 6393
(`TransportState`). Search over the whole document:

```
$ grep -n 'playable set\|active take\|SourceHash' architecture.md
```
returned no line in section 5 or section 15.10 that binds a track to a source.

**WHAT SHOULD CHANGE.** Declare the playback resolution as a first-class boundary of section 5.8.
Name the type that carries a track's ordered, resolved region list to the disk thread, name the
thread that builds it, name the bound on its size, and state what a locate does to every ring. Then
state where region gain, fade and layer are applied: either a declared stage of the chain, or a
declared step of the disk thread before the ring. Until that exists, sections 6.1 and 6.2 describe a
model with no consumer.

---

## CRITICAL 2. Two steps of the quit path have no bound row and no stated action on expiry

**OBSERVATION.** Section 9.5 rule 8 gives the quit path nine steps. Step 2 waits up to B23 for every
in-flight verb reply. Step 3 waits up to B23 for `EngineEvent::CaptureDone` for every armed track
and commits each take. Step 5 joins every job worker inside B23. The section then states:

> **Every step of the nine has a bound and a stated outcome**, which is what makes the quit path a
> Responsive path and not a hope: B23 covers steps 2, 3 and 5 ...  **No step waits without one.**

The B23 row in section 1.6 reads `250 ms | The shutdown wait for in-flight verb replies`. The section
5.12 cross-boundary table holds exactly one B23 row, `The shutdown wait for in-flight verb replies`,
whose expiry action is `The reply is dropped and the connection closes`. Appendix B.5 holds exactly
one B23 row, `The shutdown wait, on tokio | tokio::time::timeout over the reply channel`.

**CLAIM.** Steps 3 and 5 are two cross-boundary waits with no row in either table, no stated action
on expiry, and a mechanism that cannot serve them. The stated value is also too small for step 3 by
an order of magnitude.

**ARGUMENT.** Section 5.12 states "This table is every cross-boundary wait", and Appendix B.5 states
"Every row here has a row there and every row there has a row here". Both claims are false for steps
3 and 5. The mechanism cell names `tokio::time::timeout` over the reply channel; step 3 waits on
`EngineEvent::CaptureDone` on the structural channel and step 5 joins `std::thread` workers, so
neither runs on tokio and neither reads a reply channel.

The value is wrong for step 3 by the document's own numbers. B53 makes the capture ring 3 s per
channel, B14 makes the disk cycle 20 ms, and B118 bounds one `TakeWriter` append and its flush at
200 ms. One slow append therefore consumes 80 percent of the whole 250 ms budget before any drain
progresses. Step 3 exists because revision 19 "abandoned a take whose container header `TakeWriter`
had never finished"; with no stated expiry action, an implementer either waits without a bound,
which contradicts the completeness claim, or proceeds and loses exactly the container header that
step 3 was added to write. That is data loss of a vocal take on a documented path.

Step 5 has the same shape. The section states that "a worker inside an export render answers the
cancel at its next progress point". No bound constrains the distance between two progress points, so
a 250 ms join is a bound on the wait and not on the work, and the section states nothing about what
follows expiry.

This is the third revision in a row with the same defect class. Appendix C records N20-1 ("section
5.12's completeness claim excludes a wait that Appendix B.5 carries") and C19-7 and C19-10 ("three
waits with no row at all"). Section 5.12 names that history itself and then repeats it.

**EVIDENCE.** `architecture.md` lines 8158 to 8175 (the nine steps), 8188 to 8191 (the completeness
claim), 1054 (the B23 row), 6706 (the one 5.12 B23 row), 13314 (the one B.5 B23 row), 1084 (B53),
1044 (B14), 1155 (B118), 6690 to 6730 (the whole 5.12 table). Every B23 use in the document:

```
$ grep -n 'B23' architecture.md
1054, 6706, 8160, 8163, 8169, 8180, 8190, 13297, 13314
```
Lines 8163 and 8169 are steps 3 and 5. No row in either timeout table answers them.

**WHAT SHOULD CHANGE.** Give step 3 and step 5 each a budget id of their own, sized against B53,
B14 and B118 for the take drain and against the export progress interval for the join. Give each one
a row in the section 5.12 table and a row in Appendix B.5, and state the action on expiry in the
words B116 and B119 already use. State what happens to a take whose header is unwritten at expiry,
because that is the outcome the user meets.

---

## CRITICAL 3. The third buffer-pool class is closed at four sites and open at three

**OBSERVATION.** Revision 21 adds a limiter class to `BufferPool` to close finding N20-3. The
declaration at line 4869 now holds three arrays: `delays`, `reverbs` and `limiters`. Three sites
still state two classes.

1. Line 1083, the budget row: `| B52 | 9.0 MB | One BufferPool, which is B47 times B49 plus B48
   times B50 |`. The limiter term, B120 times B121, is absent.
2. Line 5436, section 5.5: "`BufferPool` holds B47 delay buffers of B49 bytes and B48 reverb buffers
   of B50 bytes. All four are fixed numbers, so one pool is B52 bytes whatever the topology is."
3. Line 5439 and line 5258, the refusal rule: "`configure` returns `ConfigError::PoolExhausted
   { kind }` when the new topology asks for more buffers than B47 or B48 allows", and "It refuses
   first and allocates second: `PoolExhausted` past B47 or B48".

The `# Errors` contract of `GraphConfigurator::configure` at line 5615 repeats the same omission:
"`ConfigError::PoolExhausted` past B47 or B48".

**CLAIM.** B52 states a pool size the declaration refutes, and the documented refusal rule cannot
refuse a topology that asks for more limiter slots than B120 holds.

**ARGUMENT.** The refusal is the part that matters. Section 5.5 states at line 5455 that
"`SlotKind::Limiter` is the `kind` a `PoolExhausted` refusal carries for it, so the enum already
names all three classes and `configure` computes three sums rather than two". Two sentences in the
same section, plus the error contract of the function itself, name two sums. A chunk reads the
`# Errors` section of the function it writes, so the two-class form is the one that reaches the
code, and a project with ten limiters then takes a pool slot the free word does not own or silently
shares one. `PoolSlot` is a single `u16` index across all three classes, so a wrong class sum is not
a clean refusal; it is an aliased buffer on the audio thread.

B52 is a smaller defect and a real one. The formula gives 8 × 384 KB plus 4 × 1.5 MB, which is
exactly 9.0 MB. The declared pool is that plus 9 × 8 KB. B55 and B56 both derive from B52, so the
steady total and the migration peak that B57 enforces are both computed from a pool the design no
longer builds.

This is the failure shape that the revision-20 closure block itself names four lines above the row
that commits it: "Two rows of C.23 were not honestly CLOSED when revision 20 wrote them. C19-7 and
C19-10 each had a fix that was complete at three sites and open at one." The N20-3 row cites
`BufferPool::limiters`, B120, B121, B122, `LimiterState` and the `LIMITER_SLOTS` constant, and cites
neither B52 nor the refusal rule. No guard covers it: I measured that PG37 declines B52 because its
value is a byte count, which is the limit PG37 states for itself.

**EVIDENCE.** `architecture.md` lines 1083 (B52), 1149 (B120), 1150 (B121), 4869 to 4895
(`BufferPool`), 5258, 5436 to 5441, 5455 to 5457, 5615 (the `# Errors` contract), 14228 (the N20-3
closure row), 14201 (the preamble that names the defect class). Proof that no guard answers it:

```
$ cd <plant copy>/duet-v1 && python3 - <<'EOF'   # B52 9.0 MB -> 90.0 MB
$ PYTHONDONTWRITEBYTECODE=1 python3 tools/placement_check.py architecture.md
planted B52: | B52 | 9.0 MB | -> | B52 | 90.0 MB |
  EXIT=0
REGISTER BAD:    0     FLOOR SLACK: 0     VALUE ROWS: 3     VALUE BAD: 0
```

**WHAT SHOULD CHANGE.** Rewrite B52 as B47 times B49 plus B48 times B50 plus B120 times B121, and
restate B55 and B56 from the corrected value. Rewrite the three refusal sentences and the `# Errors`
contract of `configure` to name three sums. Add B52 and the refusal rule to the N20-3 closure row.

---

## WARNING 1. PG37's oracle accepts a wrong value when another integer in the same block matches

**OBSERVATION.** PG37 is new in revision 21. Its implementation collects every integer literal from
every Rust or text block that mentions the budget id anywhere, and it passes when the stated value
appears anywhere in that set (`tools/placement_check.py`, `budget_value_audit`, lines 3111 to 3140).
I planted four wrong values for B120, whose true value is 9 slots:

```
planted B120 = 4 slots (true value is 9)     EXIT=0   VALUE BAD: 0
planted B120 = 8 slots (true value is 9)     EXIT=0   VALUE BAD: 0
planted B120 = 48 slots (true value is 9)    EXIT=0   VALUE BAD: 0
planted B120 = 2048 slots (true value is 9)  EXIT=0   VALUE BAD: 0
```

Only a value outside the set goes red, and its own message names the set:

```
planted B120: | B120 | 9 slots | -> | B120 | 5 slots |
  EXIT=1
  VALUE:      B120: the row states 5 and the declaration that cites it writes 4, 8, 9, 48, 256, 2048
```

**CLAIM.** PG37 is a membership test against a bag of unrelated constants, not a comparison against
the declaration that names the id, so it passes five of the six values in that bag.

**ARGUMENT.** B120 is cited only inside the section 1.6 constants block, which declares
`DELAY_SLOTS = 8`, `REVERB_SLOTS = 4`, `LIMITER_SLOTS = 9`, `MAX_STRIPS = 48`, `PEAK_STAGING_BINS =
256` and `MAX_PARAMS = 2048`. The rule reads all six and asks only whether the stated value is one of
them. The rule row in section 1.9 claims that PG37 refuses "a budget VALUE that states a capacity the
declaration citing it refutes". For B120 the declaration that cites it is one line,
`pub const LIMITER_SLOTS: usize = 9;`, and the rule never reads that line in isolation. A stale value
that happens to match a neighbouring constant is exactly the drift a stale budget produces, so the
rule declines the most likely instance of its own class.

**EVIDENCE.** `tools/placement_check.py` lines 3111 to 3140. The five runs quoted above, each one a
separate copy of the frozen tree with one edit. The recorded probe PP37 (B113 from 13 to 12) does go
red, which I confirmed:

```
planted PP37: B113 13 -> 12 elements
  EXIT=1
  VALUE:      B113: the row states 12 and the declaration that cites it writes 13
```
PP37 passes because B113's citing block holds one integer. It proves the rule fires; it does not
prove the oracle is sound.

**WHAT SHOULD CHANGE.** Bind each budget id to the one declaration line that names it in its own doc
comment, and compare against that line alone. State the new probe as a value that matches another
constant in the same block, which is the shape that passes today.

---

## WARNING 2. PG37 decides three budget rows out of 127, and it declines B86

**OBSERVATION.** Every green run prints `VALUE ROWS: 3` beside `BUDGET ROWS: 127`. I planted a wrong
value in three further budget rows, each in its own copy:

```
planted B47: | B47 | 8 | -> | B47 | 7 |             EXIT=0   VALUE ROWS: 3   VALUE BAD: 0
planted B34: | B34 | 64 | -> | B34 | 32 |           EXIT=0   VALUE ROWS: 3   VALUE BAD: 0
planted B86: | B86 | 48 strips | -> | 64 strips |   EXIT=0   VALUE ROWS: 3   VALUE BAD: 0
```

**CLAIM.** The rule's scope test is a word match on the value cell, so it declines three capacities
that sit squarely inside the class its own sentence names, B86 among them.

**ARGUMENT.** The rule states its limit as "the budget rows whose value is a count of elements,
slots, bins, or records". B47 is `BufferPool::DELAY_SLOTS`, which is a count of slots; its cell reads
the bare word `8`, so the word test rejects it. B34 is a queue capacity in records; its cell reads
`64`. B86 is `MAX_STRIPS`, which sizes every `MeterSnapshot` array, the `ResetGenerations` array and
`MAX_STRIPS` itself; its cell reads `48 strips`, and "strips" is not one of the four accepted words.
B86 is the most load-bearing capacity in the document and the rule skips it for a cosmetic reason.
The run prints the denominator honestly, which is correct practice, but a reader who sees
`VALUE ROWS: 3` cannot tell that B47 and B86 were skipped on their spelling rather than on their
kind.

**EVIDENCE.** The three runs above. `tools/placement_check.py` lines 3103 to 3133, and the `VALUE
ROWS: 3` line of the full green run at `gates-full.txt` line 34.

**WHAT SHOULD CHANGE.** Decide the scope from the declaration that cites the id, not from the wording
of the value cell. A budget whose citing declaration holds a fixed capacity is in scope whatever the
cell calls it. Print the ids the rule skipped, so the denominator is readable.

---

## WARNING 3. A region fade needs a `ParamId` that the B90 budget does not count

**OBSERVATION.** `Region` holds `fade_in: Curve` and `fade_out: Curve` (lines 6768 and 6769). `Curve`
holds `parameter: ParamId` (line 7141). `ParamSnapshot` is `[Finite; MAX_PARAMS]` at B90, "addressed
by `ParamId`" (line 11808). The B90 row derives its 1509 from strip fixed parameters, send levels and
slot parameters, and counts no region fade. The two refusals, `MixError::ParamBudgetExceeded` from
`MixState::validate` and from `SessionCommand::StripInsertSlot`, both act on mix documents, and a
region fade is a session edit.

**CLAIM.** Either every region fade consumes a cell of the 2048 the design bounds, in which case a
comped vocal project exhausts the space and no declared refusal catches it, or a fade curve's
`ParamId` addresses nothing, in which case the type forces a value with no meaning.

**ARGUMENT.** `Curve::new` takes a `ParamId` and there is no second constructor, so a `Region` cannot
be built without one. At B1 the product allows 300 bars across 32 tracks; a comp of that length holds
hundreds of regions, and two fades each. B90 leaves 539 free cells above its stated 1509. The
document nowhere states that a fade curve is exempt from the `ParamSnapshot` address space, and
nowhere states a sentinel. Both readings are consistent with the text, which is the defect class
section 5.10 already named for the capture ring: a sentence that three designs satisfy.

**EVIDENCE.** `architecture.md` lines 6762 to 6773 (`Region`), 7136 to 7144 (`Curve`), 7156
(`Curve::new`), 11808 to 11811 (`ParamSnapshot`), 1126 (the B90 row and its arithmetic), 5686 to 5705
(the parameter refusal, and its two callers).

**WHAT SHOULD CHANGE.** State whether a region fade curve takes a `ParamId` out of the B90 space. If
it does, add the term to the B90 arithmetic and name the refusal that a region edit hits. If it does
not, split the type: a fade envelope is not an automatable parameter and should not carry a
parameter address.

---

## WARNING 4. The fault table is declared with one row per variant and holds three of sixteen

**OBSERVATION.** Section 12.4 declares four tables and states that
`crates/duet/src/shell/fault_text.rs` "fixes the user message for each variant of each of the four
keys. The wording is reviewed once and not written at each site." The first row of the table of
tables reads `| The fault table | EngineFault | One row per variant |`. `EngineFault` holds sixteen
variants. Section 12.4 prints the engine-state table, the refusal table and the CONFIGURE table, and
prints no fault table at all. Three fault messages exist in the document, and all three are written
in other sections: `CaptureShortfall` and `CalibrationTimeout` in section 5.4, and `HandoffBusy` in
section 5.5.

**CLAIM.** The one table that answers every device and engine fault on the status bar is declared
and not written, and the three rows that do exist are written at the sites the section forbids.

**ARGUMENT.** Section 12.4 states the rule that a fault reaches the user as a message and that the
status bar is the single surface. Thirteen of the sixteen variants therefore have a stated surface
and no stated text. Chunk K1 writes `fault_text.rs`, and the section states that chunk K1 "reads this
file top to bottom", so the chunk meets a declared table with no rows. The three that are written
elsewhere prove the harm of the split, which Warning 5 measures.

**EVIDENCE.** `architecture.md` lines 9630 to 9637 (the table of tables), 9651 to 9653 (the
"reviewed once" rule), 9538 to 9612 (the `EngineFault` declaration, sixteen variants), 4671 and 4652
(the two section 5.4 messages), 5310 to 5312 (the section 5.5 message). No guard reads any of these
tables: the marker list shows no `GUARD BLOCK` between lines 9400 and 9681.

**WHAT SHOULD CHANGE.** Write the fault table with one row per variant, in section 12.4, and delete
the three messages that other sections carry. Register the table as a guard block with its own floor,
so a variant added later without a row is red.

---

## WARNING 5. The refusal table names five rows and holds four, and two refusals carry two messages

**OBSERVATION.** Section 12.4 states: "The refusal table carries a row for `PoolExhausted`,
`RingBudgetExceeded`, `StripBudgetExceeded`, `ParamBudgetExceeded`, and `BusRoleReserved`". The table
that follows holds four rows, and `PoolExhausted` and `RingBudgetExceeded` are not among them.
Section 5.5 writes both missing messages itself, in wording that differs from the CONFIGURE table's
wording for the same two variants:

| Variant | Section 5.5 text | Section 12.4 CONFIGURE text |
|---|---|---|
| `PoolExhausted` | `This project already uses all <n> <kind> slots. Remove one before you add another.` | `This project already uses every <kind> buffer the audio engine holds. Remove one before you add another.` |
| `RingBudgetExceeded` | `This project needs <n> MB of audio buffers, which is over the <m> MB limit. Hide a part, or play fewer tracks.` | `This change needs more audio memory than Duet reserves. Remove a track or a bus, then try again.` |

**CLAIM.** One user-facing string has two homes and the two disagree, which is the exact outcome the
document's own DR3 and DR4 exist to prevent.

**ARGUMENT.** The two wordings are not variants of one sentence. One names a count and a kind; the
other names no number. One tells the user to hide a part; the other tells the user to remove a track
or a bus. A chunk that writes `fault_text.rs` from section 5.5 and a reviewer who reads section 12.4
will not agree on what the product says. The sentence that names five rows against a four-row table is
the second half of the same defect: a reader who trusts the sentence looks for two rows that are not
there and finds the section 5.5 copies instead.

**EVIDENCE.** `architecture.md` lines 9651 to 9654 (the five-row sentence), 9658 to 9663 (the
four-row table), 5459 to 5461 and 5533 to 5534 (the two section 5.5 strings), 9668 to 9673 (the
CONFIGURE table rows).

**WHAT SHOULD CHANGE.** Put every refusal message in section 12.4 and delete both section 5.5
copies. Correct the sentence to name the rows the table holds, or add the two rows it names.

---

## WARNING 6. `PlaybackStarved` has no latch, and the document states the reason it needs one

**OBSERVATION.** Section 5.6 latches `EngineFault::ChainSlotVacant` and gives the reason:

> an unlatched report is about 375 faults each second at B7, which fills the B34 queue, then fills
> the B29 event channel, and then passes B39 and degrades the client.

`EngineFault::PlaybackStarved { track, frames }` carries no latch and no rate limit. Section 5.10
states that a starved ring "returns `CycleOutcome::PlaybackStarved`", and section 5.4 states that the
cpal error callback "pushes `EngineFault::PlaybackStarved` into the B34 `FaultQueue`, exactly as a
starved ring does". `CaptureShortfall { track, frames }` and `CaptureOverflow { track, frames }` are
the same shape.

**CLAIM.** The three per-track, per-cycle faults are the ones a real device failure raises, and they
carry none of the containment the document applied to the fault that cannot happen in normal use.

**ARGUMENT.** `ChainSlotVacant` is a malformed-plan fault; the document itself calls it the outcome
of a condition that lasts "until the next topology change". `PlaybackStarved` is the ordinary outcome
of a loaded machine or a slow volume, and it is per track. At B1 with 32 playing tracks, one starved
cycle raises 32 faults, and 375 cycles a second raises up to 12,000. B34 holds 64, so the queue and
its drop counter do absorb the burst, which is correct backpressure. The cost lands one boundary
later: the engine handoff thread sends one `EngineEvent::Fault` per drained record, at up to 64 per
B108 drain, which is about 1,300 events a second into the B29 channel of every client, against a
frame rate of 60. The document performed this arithmetic for one variant and did not perform it for
the three that a user actually meets.

**EVIDENCE.** `architecture.md` lines 5875 to 5885 (the latch and its arithmetic), 5538 to 5555
(`EngineFault` variants), 4670 to 4675 (section 5.4 xrun row), 6493 to 6503 (section 5.10), 6326 to
6335 (the B34 row of section 5.8), 1071 (B29), 1081 (B39).

**WHAT SHOULD CHANGE.** Coalesce the per-track faults at the producer or at the drain. Either latch
per track and per condition, as `ChainSlotVacant` is latched, or state a rate at which the drain
collapses repeats into one record with a count, in the shape `FaultsDropped` already has.

---

## CONCERN 1. The document header states revision 20

Line 3 reads `Author: Software Architect. Date: 2026-09-21. Status: proposed. Revision 20.` The
preamble at line 18 reads "Revision 21 wrote four such notes". A reader who opens the file meets the
wrong number first. This is the shape of finding C19-W12, which the document records as closed.
Correct line 3.

## CONCERN 2. Eleven thread rules, two places that say ten

Section 5.7 line 6184 reads "**The ten thread rules.**" and the rule index at line 1302 reads
`| TH1 to TH10 | The thread rules | 5.7 |`. Eleven rules exist: TH1 to TH11. TH11 governs the cpal
error callback, which is the second producer of the fault queue and the whole reason that queue is a
`crossbeam_queue::ArrayQueue`. It was added in this revision to close C20-W5, and neither the count
nor the index range moved with it. The list also prints TH11 before TH10. Correct the count, the
index range and the order.

## CONCERN 3. Appendix C states a review count that has aged

The preamble at line 108 states "The review count is the number of closure blocks Appendix C
registers, so it is a fact of this document and not a sentence that ages." Appendix C line 13346 then
states "Fourteen reviews produced findings against this design: the premise review, and the
specification reviews of revisions 1 to 13." Appendix C holds sections C.1 to C.24 and registers five
closure blocks. Three counts, none of which agree. Delete the sentence, or derive it.

## CONCERN 4. The "one set" claim between the two timeout tables has an unstated exception

Section 5.12 states that "every row of the Appendix B.5 timeout table is a row here, and the two
tables are one set", and then states "**B110 is the one row that bounds an allocation and not a
wait.**" Appendix B.5 states the rule without the exception: "Every row here has a row there and
every row there has a row here." B110 has a row in section 5.12 and no row in B.5. State the
exception at both sites, or move B110 out of the set.

---

## Guard pass: what I planted and what each guard answered

Every command below ran from the frozen `duet-v1` directory or from a throwaway copy of it, with
`PYTHONDONTWRITEBYTECODE=1` set, against one shared cargo target that `roster_compile.sh` created
and deleted.

**The one full run.**

```
$ cd <frozen>/duet-v1
$ PYTHONDONTWRITEBYTECODE=1 python3 tools/run_all_gates.py architecture.md <scratch> /Users/james/Developer/duet
=== placement_check        exit 0  OK      1.1s
=== probe_run              exit 0  OK    242.2s
=== probe_closure          exit 0  OK      0.9s
=== conversion_check       exit 0  OK      0.1s
=== probe_conversion       exit 0  OK      2.2s
=== closure_check r16      exit 0  OK      0.1s
=== closure_check r17      exit 0  OK      0.1s
=== closure_check r18      exit 0  OK      0.1s
=== closure_check r19      exit 0  OK      0.1s
=== closure_check r20      exit 0  OK      0.1s
=== roster_compile         exit 0  OK    110.5s
=== probe_roster           exit 0  OK    143.3s
GATES RUN: 12     GATES BAD: 0     MODE: FULL
```

The closure counts match the reviews this revision closes:
`REVIEW: critic-spec-r19.md   BLOCK: closure-r19   GENERATED: 45   ROWS: 45   CLOSURE BAD: 0` and
`REVIEW: critic-spec-r20.md   BLOCK: closure-r20   GENERATED: 23   ROWS: 23   CLOSURE BAD: 0`.
Forty-five is 11 plus 25 plus 9, and 23 is 5 plus 10 plus 8, so both blocks hold one row per finding.

**Plant 1, a deleted row in a registered block. The guard held.**

```
planted: ChainRunner removed from audio-owned
EXIT=2
FAIL: the `audio-owned` block of this document: it holds 39 rows and the stated minimum is 40; the guard is fail-closed (DR7).
```

**Plant 2, the documented floor tool against that deletion. The tool is fail-closed.** This was my
main attack on the floor scheme, because `sync_floors.py` writes every floor from the current count
and could turn any deletion green. It refuses:

```
$ PYTHONDONTWRITEBYTECODE=1 python3 tools/sync_floors.py architecture.md
FAIL: the `audio-owned` block does not read: it holds 39 rows and the stated minimum is 40
SYNC EXIT=2
```
`counted()` reads the block through `placement_check.read_block`, which still enforces the marker, so
the tool cannot lower a floor. The design is correct and I record it as a pass.

**Plant 3, the deletion with the document marker lowered by hand. The register compare held.**

```
planted: row deleted AND document marker lowered to 39
EXIT=2
FAIL: the `audio-owned` block of this document: the marker states rows>=39 and the register states 40; the guard is fail-closed (DR7).
```

**Plant 4, the deletion with the marker and the `placement_check.py` register both lowered. The
closure rule held independently of the floor.** This is the strongest result of the guard pass:

```
EXIT=1
  ROOT:       ChainRunner: its declaration carries the marker and the block omits it
  CLOSURE:    ChainRunner: EngineProcess reaches it through a declared field and it is neither an audio-owned root nor an audio-reachable leaf
```
PG26e derives the root set from the declaration bodies below `EngineProcess`, so a floor edit alone
cannot disarm the audio rules. The three-source design does what section 5.7 claims for it.

**Plant 5, PG37 on its own recorded shape. Red, as recorded.**

```
planted PP37: B113 13 -> 12 elements
EXIT=1
  VALUE:      B113: the row states 12 and the declaration that cites it writes 13
```

**Plants 6 to 13, PG37 scope and oracle. Green on eight wrong values.** Reported as Warning 1 and
Warning 2. One violation per run, each in its own copy of the tree, so no upstream gate masked a
downstream hole.

I also confirmed the harnesses go red on their own planted shapes rather than only passing clean.
`probe_run.py` prints 77 recorded fragments against a floor of 45 with `PROBES BAD: 0`;
`probe_closure.py` prints twelve shapes, eleven at exit 1 and one at exit 2, with
`CLOSURE PROBES BAD: 0`; `probe_conversion.py` prints `CP8c exit 2 OK   a member file the guard
cannot open   FAIL: m/src/locked.rs: the file does not open; the guard is fail-closed.`, which is the
fail-closed-on-unreadable-input behaviour rather than a fail-open.

---

## Reactive Assessment

- **Responsive — FAIL.** Two steps of the quit path wait with no bound row and no stated action on
  expiry, and the value they share is too small for the work by an order of magnitude (Critical 2).
  The rest of the document is strong here: twenty-one cross-boundary waits carry a bound, and
  revision 21 correctly converts B18, B116 and B119 from a cancel a blocking call cannot perform into
  a report the measuring party can make.
- **Resilient — FAIL.** The playback path has no declared source resolution, so the failure mode is
  not a degraded one, it is an absent one (Critical 1). The `PoolExhausted` refusal cannot refuse the
  third pool class (Critical 3). Faults are typed, contained and latched in one place and unlatched in
  the three that a real device failure raises (Warning 6).
- **Elastic — PARTIAL.** Every queue is bounded and every overflow action is named, which is the
  strongest part of the document. The producer count now decides the primitive, which is correct. The
  gap is the fault storm of Warning 6: bounded at the queue, unbounded at the event channel behind it.
- **Message Driven — PASS.** Each boundary names one primitive, one producer and one consumer, and
  section 5.8 is the one place that decides. No lock crosses the audio seam, the publications are
  `triple_buffer` rather than `arc-swap` with a stated reason, and `basedrop` moves every free off the
  audio thread. The one seam that is missing is the one Critical 1 names.

## Verdict

The guard tooling is now sound, and revision 21 should stop building it. Eleven of the twelve gates
answered every hostile input I put to them, including the floor attack I expected to succeed; only
PG37, the rule this revision added, is weak, and its weakness is an oracle and a denominator rather
than a fail-open. The engineering is a different matter. The specification describes a mixer in
extraordinary detail and does not describe how a recorded take reaches it: no declared type, no
declared boundary and no declared thread turns a track and a position into a source file and a region
envelope, so sections 6.1 and 6.2 have no runtime at all. That is the single biggest risk, and it is
larger than every guard finding of the last three reviews combined. Beside it sit two smaller
Criticals of the same family — a quit path whose completeness claim is false at the two steps that
lose a take, and a third buffer-pool class closed at four sites and left open at the refusal that
protects it. Each of the three is a sentence that claims completeness over a set that the
declarations do not cover, which is the defect class Appendix C has now recorded four revisions
running. The weakest Reactive property is Resilient: the design contains failures well wherever it
has declared the path, and Critical 1 is a path it has not declared.

**Counts: 3 Criticals, 6 Warnings, 4 Concerns.**
