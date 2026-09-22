# Inner review of Duet v1 architecture, revision 23

## What I read, what I ran, and what I did not cover

`architecture.md` md5 at the start of my run: `eb3890c943a565eaa681b990cfc9d705`.
`architecture.md` md5 at the end of my run: `eb3890c943a565eaa681b990cfc9d705`.
**The two agree.** The frozen tree did not change while I worked.

I read `architecture.md` sections 1.2, 1.3, 1.5 PG40 and PG41, 1.9, 5.4, 5.5, 5.7, 5.8, 5.10, 5.12,
6.4, 6.6, 7.3, 8.2, 8.3, 9.2 to 9.6, 12.4 extracts, 13.2, 13.4, 15.10, 15.11, 15.12, 15.14 and 15.16
in full. Two forks of me held sections 12, 13, 14 and 1.6, and the six ADRs,
`product-requirements.md`, `design-contract.md`, `research/`, Appendix A, Appendix B and Appendix
C.27, against the same frozen document. **I verified every finding below against both sides myself**
before I wrote it. My coverage is complete over every area the brief names.

I ran the section 1.9 command verbatim on the frozen copy. One shared cargo target served the whole
run, and I deleted it when the run ended. Every hostile shape went into a throwaway copy under my
own scratch directory. I ran no git command and I wrote no file inside `/Users/james/Developer/duet`.

### The section 1.9 run, on the frozen copy

```
cd .../scratchpad/frozen-r23-inner/duet-v1
PYTHONDONTWRITEBYTECODE=1 ROSTER_TARGET_DIR=.../r23-work/target \
    python3 tools/run_all_gates.py architecture.md .../r23-work/roster-scratch /Users/james/Developer/duet
```

```
=== placement_check        exit 0  OK      2.4s
=== probe_run              exit 0  OK    516.4s
=== probe_closure          exit 0  OK      2.0s
=== conversion_check       exit 0  OK      0.2s
=== probe_conversion       exit 0  OK      2.3s
=== closure_check r16      exit 0  OK      0.2s
=== closure_check r17      exit 0  OK      0.2s
=== closure_check r18      exit 0  OK      0.2s
=== closure_check r19      exit 0  OK      0.2s
=== closure_check r20      exit 0  OK      0.2s
=== closure_check r21      exit 0  OK      0.2s
=== closure_check r21-inner exit 0  OK      0.2s
=== closure_check r22-inner exit 0  OK      0.2s
=== roster_compile         exit 0  OK    110.9s
=== probe_roster           exit 0  OK     48.6s
GATES RUN: 15     GATES BAD: 0     MODE: FULL
```

**C22I-5 is closed and I measured it.** The twenty-second review found eight red gates on a frozen
copy. This run is green at fifteen gates from a directory outside the repository, which is the one
configuration every external review uses. The carrier counters print as the site states:
`CARRIERS: 23     CARRIER FIELDS: 44     CARRIER BAD: 0`.

---

## CRITICAL 1. The agent gateway seam has no carrier row, no declared end, and no crate that can spell one

**OBSERVATION.** Section 9.5 rule 5 at `architecture.md:8966` reads "`async_channel` and
`futures::channel::oneshot` work on both executors. The tokio task sends a `GatewayRequest`; a
`cx.spawn` task on the GPUI side receives it, calls the gateway, and returns the result on the
oneshot." Rule 6 at `architecture.md:8970` reads "The request channel holds B35." B35 at
`architecture.md:1221` is "256 | The agent request channel | 9.5". `AgentBridge` at
`architecture.md:13848` declares `runtime`, `cancel` and `finished`. `GatewayRequest` at
`architecture.md:12344` is `pub struct GatewayRequest { verb: Verb, client: u64 }`. The
`carrier-table` block at `architecture.md:6836` holds 23 rows and none names `GatewayRequest`, B35
or a oneshot. The section 1.2 row for `duet-agent` lists `rmcp`, `tokio`, `tokio-util`, `thiserror`
and `tracing`. The row for `duet` lists `gpui-kit`, `clap`, `arrayvec`, `tokio`, `tracing` and
`tracing-subscriber`.

**CLAIM.** The one seam between the two runtimes of this process is the exact defect class that TH13
and PG41 exist to refuse, and it survived the revision that wrote them.

**ARGUMENT.** TH13 at `architecture.md:6655` binds "Every cross-thread duty this document states".
"tokio workers (B38)" and "GPUI foreground" are two rows of the section 5.7 thread table, so a
`GatewayRequest` that travels from one to the other is a cross-thread duty by TH13's own words. No
row of the `carrier-table` names it. No declared field holds the write end, the read end, the
oneshot sender or the oneshot receiver, so the reverse half of PG41 finds nothing: that half reads
the declared field SET, and the set is empty here.

The dependency half makes the hole wider rather than smaller. `duet-agent` carries no
`async-channel` entry and no `futures` entry, and `crates/duet` carries neither, so neither crate can
name the type that would hold the end. `duet-core` carries both and declares no `GatewayRequest`
end. This is the C20-5 shape, where section 12.4 named a type the crate could not spell.

B23 rests on the same undeclared value. The Appendix B.5 row at `architecture.md:14698` reads
"`tokio::time::timeout` over the reply channel". The reply channel is the oneshot, and no
declaration holds it.

**EVIDENCE.** `architecture.md:8966`, `:8970`, `:1221`, `:13848` to `:13853`, `:12344`, `:6836` to
`:6859`, `:6655`, `:14698`; the section 1.2 rows for `duet-agent` and `duet`.

**WHAT SHOULD CHANGE.** Add two rows to the `carrier-table`: the B35 request channel and the oneshot
reply. Declare each end on a type, which means a `GatewayRequest` channel field on `AgentBridge` and
a reply-sender field inside `GatewayRequest` or beside it. Add the `async-channel` and `futures`
entries to the crates that will hold them, each with the section 1.2 reason. Re-audit the C22I-1 and
C22I-2 closure rows, because both claim that every structural end is now declared.

---

## CRITICAL 2. `EngineDisk` cannot build the `CaptureInfo` the carrier table says it sends, and it holds no file

**OBSERVATION.** `EngineDisk` at `architecture.md:12885` declares six fields: `plan`, `requests`,
`fills`, `drains`, `events` and `pending_event`. Its own doc at `architecture.md:12873` reads
"Revision 22 gave this thread four duties and declared no value it holds... section 6.4 made it the
reporter of one `CaptureInfo` per armed track, and the thread table made it the owner of every take
file... Each duty needed a field and none existed." `CaptureInfo` at `architecture.md:7595` declares
`track`, `start`, `frames`, `loop_offset` and `underruns`. Section 5.10 gives this thread three file
duties: it opens `media/<hh>/<hash>.wav` through `duet_media::SourceReader` at
`architecture.md:7037`, it appends through `duet_media::TakeWriter` at `architecture.md:7064`, and it
builds peaks through `duet_dsp::peaks::PyramidBuilder` at `architecture.md:7097`. A grep of the whole
document for a field whose type is `SourceReader`, `TakeWriter` or `PyramidBuilder` returns nothing.

**CLAIM.** The type minted to close C22I-2 repeats C22I-2 for three of the four duties its own doc
names. The producer of `CaptureInfo` holds no value from which it can build one.

**ARGUMENT.** A `CaptureInfo` names a start, a frame count, a loop offset and an underrun count for
one track across a whole pass. `EngineDisk` declares no per-track accumulation of any of the four,
and `drains` is an `ArrayVec<Consumer<f32>, MAX_DISK_RINGS>` that carries no track identity. The
thread also cannot open a source, cannot append to a take file and cannot write a peak file, because
no field holds a reader, a writer or a builder. B131 at `architecture.md:1314` states that this
thread "must drain a full capture ring and then finish the container header", and
`TakeWriter::finish` is named at `architecture.md:8993`; no declared field holds the writer whose
header it finishes.

The document applies this exact standard to another thread and states it at the site.
`GraphConfigurator.backend` at `architecture.md:5537` carries the doc line "Revision 16 gave the
thread the duty and gave no declaration the value." The same test gives the same answer here, one
revision later, on the type written to answer it.

PG41 cannot see this. The missing fields hold no channel end, no ring end and no publication end, so
the reverse half skips them, and the forward half reads only the ends a row names. The run prints
`CARRIER BAD: 0` over a producer that cannot produce.

**EVIDENCE.** `architecture.md:12873` to `:12901`, `:7595`, `:7037`, `:7064`, `:7097`, `:1314`,
`:8993`, `:5537`, `:424`.

**WHAT SHOULD CHANGE.** Declare the take writer, the source reader, the peak builder and the
per-track capture accumulation on `EngineDisk`, each keyed by `TrackId`. State which of them is fixed
at arm time and which is per pass. Re-audit the C22I-2 and C22I-4 closure rows.

---

## CRITICAL 3. Chunk C2 declares a field whose type chunk C3 writes one phase later

**OBSERVATION.** Chunk C2 at `architecture.md:11023` runs in phase 5. Its goal reads "the disk thread
with `EngineDisk` as its own root... and the one `CaptureInfo` per armed track that the thread
reports on the B138 channel". Chunk C3 at `architecture.md:11024` runs in phase 6. Its goal reads
"... `EngineLink` and the B138 sender, and `ConfigureCommand::Stop`". `EngineDisk` at
`architecture.md:12890` declares `events: EngineLink`. `EngineLink` is declared at
`architecture.md:12870`. The phase table at `architecture.md:11325` and `:11326` puts C2 in phase 5
and C3 in phase 6. Section 13.4 at `architecture.md:11433` carries the row `C2 before C3`.

**CLAIM.** C2 cannot compile, and its Completion command cannot pass.

**ARGUMENT.** SM6 makes a line an ordered chain, so C2 runs before C3. C2 writes
`crates/duet-engine/src/disk/{reader,writer,ring}.rs`, which is where `EngineDisk` lands, and
`EngineDisk.events` names a type that no chunk at or before phase 5 declares. C2's Completion command
selects `test(capture_done)`, and a `CaptureInfo` report needs the B138 sender that C3 writes. The
order C2 needs is the reverse of the order section 13.4 states, and the two orders cannot both hold.

PG31 reads the link row against the phase table and finds the stated order correct, so no guard sees
this. The defect is that the carrier table gave C2 a new field in C3's crate module, and no link row
moved with it.

**EVIDENCE.** `architecture.md:11023`, `:11024`, `:11325`, `:11326`, `:11433`, `:12870`, `:12885` to
`:12901`.

**WHAT SHOULD CHANGE.** Move `EngineLink` and the B138 sender into C2, or move the `EngineDisk` event
half into C3 and state the split in both goals. Correct the section 13.4 row. Re-audit C22I-1 and
C22I-6.

---

## CRITICAL 4. `EngineProcess::sounding` is indexed by a value whose range is B88 and whose array is B139

**OBSERVATION.** `EngineProcess.sounding` at `architecture.md:13248` is
`[SoundingNotes; MAX_BOUND_PORTS]`. B139 at `architecture.md:1325` gives `MAX_BOUND_PORTS` the value
16 and calls it "the MIDI input ports one engine binds at once". `MidiRecord` at
`architecture.md:8618` is `{ port: PortSlot, at: SampleClock, message: MidiMessage }`. `MidiPortMap`
at `architecture.md:8406` declares `next: u16` with the doc "The next slot this process run mints.
B88 bounds it", and `insert` returns `MidiError::PortSlotsExhausted` past B88. B88 at
`architecture.md:1274` is 1024. Section 8.2 step 1 at `architecture.md:8543` reads "the cycle that
reads it synthesizes a note-off for every set bit of that slot and clears the bitset".

**CLAIM.** The index space of `PortSlot` is 1024 wide and the array is 16 long. No declared field and
no stated rule maps one onto the other, so the silence pass cannot address the bitset.

**ARGUMENT.** `MidiPortMap` mints one `PortSlot` per distinct identity for the life of the process,
and a returning port keeps its old slot, so the value is dense over identities SEEN and not over
ports BOUND. A user who connects seventeen different controllers in one session gives a bound port
the slot 16, which is past the end of the array. `clippy::indexing_slicing` is denied, so the audio
thread must use a checked access, and a checked access on an out-of-range slot silently skips the
silence pass. The failure the step exists to prevent is a sustained note the user cannot release, and
that is what returns.

This is the C22I-4 shape, where a value was asked of a party that could not hold it. Here the value
is an index, and the type that carries it states a different range from the array that consumes it.

**EVIDENCE.** `architecture.md:13248`, `:1325`, `:8618`, `:8406`, `:8430`, `:1274`, `:8543`.

**WHAT SHOULD CHANGE.** Declare a bound-port index that B139 bounds, and state where the map from
`PortSlot` to that index lives. The cheapest answer is a `BoundPort` newtype that `MidiRecord` and
`MidiMessage::PortGone` both carry. Re-audit C22I-7.

---

## CRITICAL 5. `DuetCore::take_flags` carries no track identity, so the core cannot apply a flag set to a take

**OBSERVATION.** `DuetCore.take_flags` at `architecture.md:13523` is
`ArrayVec<TakeFlags, MAX_DISK_RINGS>`. Its doc at `architecture.md:13515` reads "**The CORE assembles
`TakeFlags`, and this is the field that holds them**". Section 6.4 at `architecture.md:7543` reads
"`DuetCore::take_flags` accumulates one `TakeFlags` per armed track while the pass runs... The core
then applies `SessionCommand` for the new take, with the flags it assembled". `TakeFlags` at
`architecture.md:7395` is `pub struct TakeFlags(u8)`. `CaptureInfo` at `architecture.md:7595` carries
`track: TrackId`.

**CLAIM.** The element type holds no key, so nothing maps an entry of `take_flags` to the
`CaptureInfo::track` whose take the flags belong to.

**ARGUMENT.** The five observations reach the core from three sources: an `EngineFault` that names a
`TrackId`, a `HotplugEvent` that names a port, and a `Calibration` that names a device. The core must
merge them per track and then apply the result to the take that `CaptureInfo::track` names. An
`ArrayVec<TakeFlags, _>` gives a position and not a track, and no declared field holds the armed
order that would make the position meaningful.

The plan closed the identical hole one revision ago and stated the reason. The C21-4 row at
`architecture.md:15713` reads "`CaptureInfo` names no track, so no declared message ends one take per
armed track", and the fix was `track: TrackId` on the value. The core-side half of the same mechanism
did not take that lesson.

The bound is wrong as well. `MAX_DISK_RINGS` is B140, a count of rings and therefore of channels.
`take_flags` holds one entry per armed TRACK, which B1 bounds at 32. A ring count cannot bound a
track array, and the B140 `Used by` cell at `architecture.md:1326` names "1.6, 15.10" and omits
15.14, where this third use sits.

**EVIDENCE.** `architecture.md:13515` to `:13523`, `:7543`, `:7395`, `:7595`, `:15713`, `:1326`.

**WHAT SHOULD CHANGE.** Give the element a `TrackId`, or declare the armed-track order the position
indexes. Bound the list by the track budget and not by the ring budget, and add 15.14 to the B140
`Used by` cell. Re-audit C22I-4.

---

## CRITICAL 6. The silence pass of C22I-7 is assigned to no chunk

**OBSERVATION.** I extracted section 13, lines 10685 to 11458, and counted occurrences.
`SoundingNotes` 0. `sounding` 0. `EngineProcess` 0. `B139` 0. The C.27 row C22I-7 at
`architecture.md:15797` reads "**CLOSED** | 15.10 `SoundingNotes` and `EngineProcess::sounding`; 8.3
`MidiMessage::PortGone`; 8.2 step 1, which names both; 1.6 B139; 13.2 chunk G4, which mints the
message". Chunk G4 at `architecture.md:11069` writes `crates/duet-midi/src/{bind,entry}.rs` alone.

**CLAIM.** C22I-7 has two halves and the closure row assigns a chunk to one of them. A fresh
orchestrator cannot dispatch the engine half.

**ARGUMENT.** The MIDI half is the message G4 mints. The engine half is the bitset array, the drain
that reads `PortGone`, the note-off synthesis and the clear, all inside `GraphRunner::run`. No goal of
line C names any of the four, and no Completion command of line C selects a test of them. Chunk C3's
goal names the graph runner and the three publications and does not name the note set. SM3 asks one
command to fail before a chunk and pass after it, and no such command exists for this mechanism.

This is the C22I-6 test applied to a mechanism revision 23 added, and it gives the same answer one
revision later. The same count finds more zeros that are also real: `take_flags` 0, `TakeFlags` 0,
and each of B139 to B145 zero.

**EVIDENCE.** `architecture.md:15797`, `:11069`, `:11024`, `:12942`, `:13248`, `:8543`; my counts over
lines 10685 to 11458.

**WHAT SHOULD CHANGE.** Name the note set, the `PortGone` drain and B139 in one line-C chunk goal, and
add one selected test. WARNING 10 and WARNING 11 name the two other mechanisms with the same gap.

---

## WARNING 1. PG41 never tests that a named end HOLDS a carrier end, and it cannot tell a write end from a read end

**OBSERVATION.** The PG41 site at `architecture.md:1017` reads "the Sender cell and the Receiver cell
must each name one `` `Type.field` `` path that a Rust block of this document declares". TH13 at
`architecture.md:6655` reads "A row names the message, the declared field that holds the write end,
the carrier and its bound, the declared field that holds the read end". `carrier_audit` at
`tools/placement_check.py:4230` reads cells 1 and 3 into one `named` dictionary at
`tools/placement_check.py:4271` and never compares a cell against the declared field set. The
existence half lives in the `carrier-end` membership kind at `tools/placement_check.py:3814`, which
tests that the type is declared and that the field exists, and tests nothing else.

**CLAIM.** A row can name a real field that holds no carrier, and a row can name its two ends in the
wrong order. The rule's own sentence names both shapes and the rule fires on neither.

**ARGUMENT.** I planted two shapes in a throwaway copy of the frozen document and ran
`placement_check.py` on each.

Shape one adds a real non-carrier field beside the real sender:

```
| `TempoMap` | `DuetCore.tempo` and `DuetCore.version` | `triple_buffer` | `EngineProcess.tempo` | ...
h5 exit=0
CARRIERS:        23     CARRIER FIELDS: 44     CARRIER BAD: 0
```

Shape two swaps the Sender cell and the Receiver cell of the same row:

```
| `TempoMap` | `EngineProcess.tempo` | `triple_buffer` | `DuetCore.tempo` | Core -> Audio | ...
h3 exit=0
CARRIERS:        23     CARRIER FIELDS: 44     CARRIER BAD: 0
```

Both runs are green. So the guard proves that two named fields are declared, and it does not prove
that either one holds an end or that the two sit on the correct sides. The two recorded PP41 shapes
test the thread-name half and the reverse half, and neither tests this half.

The other three shapes behave as the table states. A phantom type, a phantom field and a phantom B id
are each red:

```
h4 exit=1  MEMBER: carrier-table: ... `Nowhere` is no declaration of this document
h8 exit=1  MEMBER: carrier-table: ... `DuetCore` declares no field `nosuchfield`
h7 exit=1  CARRIER: TempoMap: the carrier cell names B999999 and no section 1.6 row declares it
```

**EVIDENCE.** `architecture.md:1017` to `:1024`, `:6655`; `tools/placement_check.py:4230` to `:4310`,
`:3814` to `:3831`; `tools/probe_run.py:789` to `:803`; my five runs above.

**WHAT SHOULD CHANGE.** Test each named end against the field set the reverse half already builds, so
a cell that names a field with no carrier head is red. Decide direction from the type expression:
`Sender`, `SyncSender`, `Producer` and `Input` are write ends, and `Receiver`, `CoreReceiver`,
`Consumer` and `Output` are read ends. Add one probe per shape.

---

## WARNING 2. Section 5.12 still carries the quit-save protocol C22I-W15 removed

**OBSERVATION.** The section 5.12 row at `architecture.md:7271` reads "The core waits for
`CoreInput::QuitSaveDone` from the `duet-quit-save` thread | B134 | ... The atomic save writes through
`state/.tmp/` and renames, so every document on disk is the last whole one". The C.27 row C22I-W15 at
`architecture.md:15807` reads "**CLOSED** | 9.5 rule 8 step 6 and 1.6 B134, which both name the
section 4.10 save and its `state/save.commit` marker and neither of which names `state/.tmp/`".

**CLAIM.** The closure row names the two sites it corrected, and a third site still states the
protocol the row calls wrong.

**ARGUMENT.** Section 4.10 at `architecture.md:4502` writes each temporary file in the same directory
as its final name and records the set in `state/save.commit`. The crash matrix at
`architecture.md:4523` completes every rename whose temporary file still exists. A start that empties
`state/.tmp/` would delete the files that recovery needs, which is the whole of C22I-W15. Section 5.12
is the one table a reader opens for the bound of a blocking call, and it gives the answer the document
has retracted. No rule reads this cell.

**EVIDENCE.** `architecture.md:7271`, `:15807`, `:4502`, `:4523`, `:1316`, `:9006`.

**WHAT SHOULD CHANGE.** Rewrite the 5.12 cell to cite section 4.10 and `state/save.commit`. Add
section 5.12 to the C22I-W15 closure row.

---

## WARNING 3. The `CaptureInfo` doc states two answers about its own fields, four lines apart

**OBSERVATION.** The doc comment at `architecture.md:7582` reads "**`track` and `flags` are the two
fields that make this message usable** (critic C21-4)". The same doc at `architecture.md:7589` reads
"**It carries NO `TakeFlags`** (critic C22I-4)." The declaration at `architecture.md:7595` holds
`track`, `start`, `frames`, `loop_offset` and `underruns`.

**CLAIM.** One doc comment gives one fact two answers, and DR1 and DR4 each forbid that.

**ARGUMENT.** The first sentence is the C21-4 closure note and it was true of revision 22. Revision 23
deleted the field and added the second sentence below the first rather than correcting it. A reader
who stops at the first paragraph builds a `CaptureInfo` with a `flags` field, which is the defect
C22I-4 closed. No rule reads the text of a doc comment, so the run is green.

**EVIDENCE.** `architecture.md:7582`, `:7589`, `:7595`, `:15713`, `:15794`.

**WHAT SHOULD CHANGE.** Rewrite the first sentence to name `track` alone.

---

## WARNING 4. `CpalBridge` gives one value two owners, and three sites disagree about how many threads it crosses

**OBSERVATION.** `CpalBridge` at `architecture.md:12919` declares `channel`,
`write: Owned<Producer<f32>>` and `read: Owned<Consumer<f32>>`. The `audio-asserted` row at
`architecture.md:6531` reads "CpalBridge argument; the two cpal callbacks own it and both run on the
audio thread (section 5.4)". The `carrier-table` row at `architecture.md:6858` gives it the Threads
cell "Audio -> Audio". Section 5.4 at `architecture.md:4845` reads "cpal has no duplex stream in
0.18.2... Input and output are two callbacks with two clocks (critic 5.2)." The `external-verdicts`
row for `Producer`, `Consumer` at `architecture.md:2151` records the heap allocation and
`PartialEq`/`Eq` and records no `Send` and no `Sync` fact, although the `Collector` row at `:2153` and
the `Receiver` row at `:2155` each record "`Send` and not `Sync`".

**CLAIM.** One value cannot be owned by two closures, and a carrier whose two ends sit on one thread
needs no ring.

**ARGUMENT.** `cpal::Device::build_input_stream` and `build_output_stream` each take their own `FnMut`
closure, and each closure owns what it captures. One `CpalBridge` therefore cannot sit in both. The
document records no `Send` or `Sync` verdict for an `rtrb` end, so a reader cannot even decide the
question from the one table that decides an external name's traits.

The thread count is the second half. If the two callbacks are one thread, the ring is not needed and
section 5.4's own reason for it is false. If they are two threads, the section 5.7 table holds one row
for both and the `audio-asserted` reason states something the backend does not guarantee. The document
must pick one answer, because TH1 binds a thread and not a callback.

**EVIDENCE.** `architecture.md:12919` to `:12927`, `:6531`, `:6858`, `:4845`, `:2151`, `:2153`,
`:2155`.

**WHAT SHOULD CHANGE.** Declare the two halves as two types, one per callback, and give the carrier row
one field for each. Record the `Send` and `Sync` verdict for `Producer` and `Consumer`. State in
section 5.7 whether the input callback is the audio thread row or a row of its own.

---

## WARNING 5. A one-slot `pending_event` cannot hold the burst B138 states it must

**OBSERVATION.** The `EngineEvent` carrier row at `architecture.md:6840` reads "On full the sender
keeps the event in its own `pending_event`". `GraphConfigurator.pending_event` at
`architecture.md:5592` is `Option<EngineEvent>`, and `EngineDisk.pending_event` at
`architecture.md:12899` is the same. Section 9.5 step 3 at `architecture.md:8993` has the core wait
"on the B138 channel for `EngineEvent::CaptureDone` for every armed track". Section 6.6 at
`architecture.md:7599` reads "The B138 channel carries one `EngineEvent::CaptureDone` per armed
track".

**CLAIM.** One retry slot serves a producer that must send one message per armed track and a producer
that mints five kinds of event. A second refusal in one pass is lost.

**ARGUMENT.** The disk thread reports up to one `CaptureDone` per armed track at the end of a pass.
With a full B138 channel it holds one refused event and drops the rest, so the core's owed set never
empties and B131 reaches expiry. The expiry action then marks every unfinished take with
`TakeFlag::HadOverrun`, which is a false fact about a take that completed.

The engine handoff thread is worse, because its events are not interchangeable.
`DuetCore::pending_configure` at `architecture.md:13472` names `EngineEvent::GraphConfigured` as "the
one re-send trigger". If `pending_event` already holds a fault when `configure` completes, the
`GraphConfigured` has nowhere to wait, and the pending topology change is lost for the rest of the
session with no user message. The stated rule that the handoff thread "drains no further fault until
the slot empties" covers faults, which come from a queue, and covers none of the four event kinds the
thread mints itself.

**EVIDENCE.** `architecture.md:6840`, `:5592`, `:12899`, `:8993`, `:7599`, `:13472`, `:1324`.

**WHAT SHOULD CHANGE.** Give each producer a bounded pending LIST with a budget row and a stated drop
rule, or state that the engine side of B138 blocks. State what a dropped `GraphConfigured` does.

---

## WARNING 6. The terminal send of the engine handoff thread has no retry path

**OBSERVATION.** Section 9.5 step 8 at `architecture.md:9012` reads "The engine handoff thread runs
`Collector::collect` and `Collector::try_cleanup`, sends that event on the B138 channel, drops its
backend, and then ends." `GraphConfigurator.pending_event` at `architecture.md:5595` carries the doc
"The one event `try_send` refused, or `None`", and the retry happens "at the head of its next pass".

**CLAIM.** The send of `HandoffStopped` is the last act of the loop, so a refused send has no next
pass, and B119 reaches expiry on a thread that finished its work correctly.

**ARGUMENT.** The B138 senders use `try_send` and never suspend, which B138 at `architecture.md:1324`
states. Every other refused event waits for the next pass. This one has no next pass, and the row
states no alternative. The cost is the cost C22I-3 removed: a five second wait on every quit where the
channel is momentarily full, with `try_cleanup` already run and the core unaware.

**EVIDENCE.** `architecture.md:9012`, `:5595`, `:1324`, `:1303`.

**WHAT SHOULD CHANGE.** State that the terminal send blocks, or that the thread retries it until the
channel accepts or B119 elapses. Name the rule in the B119 row.

---

## WARNING 7. No declared message and no declared method registers a client, so B141 refuses nothing

**OBSERVATION.** `DuetCore.clients` at `architecture.md:13521` is `ArrayVec<ClientLink, MAX_CLIENTS>`.
B141 at `architecture.md:1327` reads "`MAX_CLIENTS`, the clients one core serves at once...
`CoreError::ClientBudgetExceeded` refuses the ninth, because a registry with no bound is a registry an
agent can grow". `CoreInput` at `architecture.md:12455` holds seven arms: `Call`, `FilesChanged`,
`SetEntryContext`, `Resynchronize`, `Shutdown`, `WorkerStopped` and `QuitSaveDone`. The document
declares no `impl DuetCore`.

**CLAIM.** A bounded registry has a refusal and no declared path that can reach it.

**ARGUMENT.** A `ClientLink` holds the two write ends of one client, and a `CoreClient` holds the
matching read ends. Some party must create both halves and put the `ClientLink` in `clients`. No
`CoreInput` arm carries a registration, no method of `DuetCore` is declared, and no carrier row names a
registration message. So `CoreError::ClientBudgetExceeded` is a refusal no caller can trigger and
`clients` is a list no caller can fill. This is the TH13 class in the core's own type, and PG41 cannot
see it because a registration is not a field with a carrier head.

**EVIDENCE.** `architecture.md:13521`, `:1327`, `:12455` to `:12472`, `:13545`, `:13582`.

**WHAT SHOULD CHANGE.** Declare the registration. A `DuetCore::register_client` that returns a
`CoreClient` is the smallest answer, and its `# Errors` section names `CoreError::ClientBudgetExceeded`.

---

## WARNING 8. No step of the nine-step quit path sends `ConfigureCommand::Reset`, and two declarations order it inside that path

**OBSERVATION.** The `ConfigureCommand::Stop` doc at `architecture.md:5511` reads "**The core sends it
after `EngineEvent::StreamClosed` and after `Reset`**, so the backend is stopped and the layout is
forgotten before the thread that owns both goes away." The `EngineEvent::StreamClosed` doc at
`architecture.md:12973` reads "**The core waits for it before it sends `ConfigureCommand::Reset`**".
Section 9.5 rule 8 step 4 at `architecture.md:8997` sends `Close` and waits B116. Step 8 at
`architecture.md:9012` sends `Stop`. No step of the nine sends `Reset`.

**CLAIM.** Two declarations state where `Reset` sits on the quit path, and the quit path has no such
step.

**ARGUMENT.** The order the two docs require is `Close`, `StreamClosed`, `Reset`, `Stop`. The nine
steps give `Close` and `StreamClosed` at step 4 and `Stop` at step 8, with nothing between. An
implementer who follows section 9.5 sends no `Reset` and leaves the layout live while the thread ends.
An implementer who follows the two doc comments adds a step the bounded path does not budget. Section
9.5 states that every step of the nine has a bound of its own, so an unlisted send inside it has none.

**EVIDENCE.** `architecture.md:5511`, `:12973`, `:8997`, `:9012`, `:9019`.

**WHAT SHOULD CHANGE.** Add the `Reset` send to step 8 in words, or delete the ordering clause from
both doc comments and cite section 5.5 alone.

---

## WARNING 9. Chunk G4 is credited with an enum arm its write scope cannot reach

**OBSERVATION.** Chunk G4 at `architecture.md:11069` states in its goal "and `MidiMessage::PortGone`,
which the MIDI thread mints", and its Writes cell is `crates/duet-midi/src/{bind,entry}.rs` alone.
`MidiMessage` is a `duet-command` type: the section 1.2 row at `architecture.md:420` lists it, and
chunk T4 at `architecture.md:10990` writes `crates/duet-command/src/{...,midi,...}.rs` in phase 3.
`MidiMessage::PortGone` is declared at `architecture.md:8611`.

**CLAIM.** One chunk is told to add an arm to an enum another line owns, in a file it may not open.

**ARGUMENT.** SM4 makes a named file the unit of write scope, and PG40 holds a chunk to the crate its
own line owns. Line G owns `duet-midi`. T4 runs four phases earlier and its goal names the type and
not the arm, so no chunk states the arm as work. PG40 stays green, because the Writes cell names no
`crates/duet-command/` path; the defect is in the Goal cell, which PG40 does not read.

**EVIDENCE.** `architecture.md:11069`, `:420`, `:10990`, `:8611`, `:1000`.

**WHAT SHOULD CHANGE.** Move the arm into T4's goal and leave G4 with the mint. State the
`T4 before G4` link if section 13.4 does not carry it.

---

## WARNING 10. The `take_flags` merge has no chunk

**OBSERVATION.** Section 6.4 at `architecture.md:7543` gives the core a five-source merge:
"`DuetCore::take_flags` accumulates one `TakeFlags` per armed track while the pass runs... The core
then applies `SessionCommand` for the new take, with the flags it assembled, in the same undo
transaction as the regions." A grep of section 13, lines 10685 to 11458, for `take_flags` and
`TakeFlags` returns zero hits. The C.27 row C22I-4 at `architecture.md:15794` cites section 6.4,
`DuetCore::take_flags` and `CaptureInfo`, and cites no chunk.

**CLAIM.** Revision 23 moved a duty from the disk thread to the core and gave the new owner no chunk,
no goal and no test.

**ARGUMENT.** The merge is new core work over three inbound sources and one undo transaction. Chunk I2
at `architecture.md:11085` names the channels and the quit path. Chunk I3 at `architecture.md:11086`
names the snapshot, the job registry, the plan build and the readers. Neither names the flag
accumulation, and C2's Completion command selects `test(capture_done)`, which is the disk-thread half
in `duet-engine`.

**EVIDENCE.** `architecture.md:7543`, `:11085`, `:11086`, `:13523`, `:15794`; my counts over lines
10685 to 11458.

**WHAT SHOULD CHANGE.** Name the merge in one chunk goal and add one selected test.

---

## WARNING 11. Three new budget rows point at four functions in a module that no write scope creates

**OBSERVATION.** B143 at `architecture.md:1329`, B144 at `:1330` and B145 at `:1331` each name a
function of `duet_dsp::meter_law`: `lufs_to_fraction`, `true_peak_to_fraction`, `fader_fraction_to_db`
and `fader_db_to_fraction`, declared at `architecture.md:8101`, `:8110`, `:8122` and `:8126`. Chunk D1
at `architecture.md:11031` is the first chunk of line D. Its write scope creates
`crates/duet-dsp/src/{buffer,source,fft,align,gain,filter,meter,dynamics,slot,voice,peaks,pool}.rs`.
It names `meter.rs` and no `meter_law.rs`, and its goal reads "the three meter laws". A count of
`fader_fraction_to_db`, `fader_db_to_fraction` and `taper` over section 13 gives zero.

**CLAIM.** Four new public functions have no chunk goal, no Completion test of their own, and no module
file that any write scope creates.

**ARGUMENT.** SM2 makes the first chunk of a line create every module file the whole line will ever
need. `meter_law` appears in no Writes cell of section 13. The file half is older than this revision,
because B106 at `architecture.md:1292` already cited `duet_dsp::meter_law::db_to_fraction`. The chunk
half is new: revision 23 added four functions and three budget rows to close C22I-W12 and gave them no
goal. `duet_dsp::meter_law` now holds five public functions and the one chunk that builds the module is
told to build three. The `Fader` element that consumes the taper is chunk K4 at `architecture.md:11062`,
one line and one phase away, so the missing function is found at K4 and not at D1.

**EVIDENCE.** `architecture.md:1292`, `:1329` to `:1331`, `:8088`, `:8101`, `:8110`, `:8122`, `:8126`,
`:11031`, `:11062`, `:15810`.

**WHAT SHOULD CHANGE.** Add `meter_law.rs` to D1's write scope, name the four functions in its goal in
place of a count that ages, and select one test for the fader round trip.

---

## WARNING 12. B145 does not determine the map it owns, and it cites the wrong contract section

**OBSERVATION.** B145 at `architecture.md:1331` reads "unity gain at 0.72 of the travel, minus infinity
below 0.08 | The fader taper of **design contract 4.2**". `fader_fraction_to_db` at
`architecture.md:8122` returns "The gain one point of the fader travel sets, in decibels", and its doc
names two constants. Both constants are fractions: `FADER_UNITY_FRACTION` at `architecture.md:1458` is
0.72 and `FADER_SILENCE_FRACTION` at `:1460` is 0.08. The contract text sits under the heading
`### 4.3 Fader` at `design-contract.md:730` and reads "Taper: -inf dB at the bottom, +6 dB at the top.
Unity gain, 0 dB, sits at 72 percent of the travel."

**CLAIM.** The declared constants give the map two positions and no value, so `fader_fraction_to_db`
cannot be written from them. The citation also names section 4.2 for a shape section 4.3 states.

**ARGUMENT.** B106 at `architecture.md:1292` is the worked example of a complete map: a floor in
decibels, a ceiling in decibels and a mid point in decibels at a stated fraction. B145 gives two
fractions and no decibel value. A chunk K4 implementer who needs the gain at the top of the travel
finds +6 dB in the contract, in no budget row and in no constant, so he writes a literal, which DR3
forbids, or he reaches for `METER_CEILING_DB`, which belongs to B106 and to a different scale. This is
C22I-W12 one level down: the row exists and it still does not determine the shape.

**EVIDENCE.** `architecture.md:1331`, `:1458`, `:1460`, `:8114` to `:8122`, `:1292`;
`design-contract.md:730`, `:739`, `:740`.

**WHAT SHOULD CHANGE.** Add the decibel ceiling and the decibel value at the silence fraction to B145,
as constants beside the two fractions, and state the law between them. Correct the citation to contract
4.3.

---

## WARNING 13. Section 10.2 states two audio publications, and the C22I-W8 closure row names section 10.2 as corrected

**OBSERVATION.** `architecture.md:9382` reads "Four frame drivers exist and **two publications exist**,
and a publication supplies exactly one read end that is neither `Clone` nor `Copy`". The line above
reads "it reads **both** once per frame before any child view renders". The C.27 row C22I-W8 at
`architecture.md:15806` reads "**CLOSED** | 5.7 TH9, 5.8, **10.2**, 15.14 and 15.16, none of which
states a count". Test 12 of section 10.7 at `architecture.md:10169` reads "The test publishes one
`MeterSnapshot` and one `TransportSnapshot`... The oracle is that four views read **two publications**
and no view owns a read end".

**CLAIM.** The closure row is false at one of the five sections it names, and the one test that proves
the frame order never exercises the third publication.

**ARGUMENT.** The `snapshot-table` block holds three audio-written rows and the run prints
`SNAPSHOTS: 3`. Revision 23 corrected TH9, section 5.8, 15.14, 15.16 and ADR 0004 decision 12b, and did
not correct section 10.2. The cost is larger than the number. Test 12 publishes two snapshots, runs one
frame and asserts that four views painted that frame. It publishes no `SlotMeterSnapshot` and it reads
no `CoreHost::slot_measure`, which is the value C22I-8 added. A frame order that read two publications
in step and the third one frame late would pass test 12.

**EVIDENCE.** `architecture.md:9381`, `:9382`, `:10169` to `:10174`, `:15806`, `:6738`; the run line
`SNAPSHOTS: 3`.

**WHAT SHOULD CHANGE.** Delete the count at `:9382` and cite section 5.8. Extend test 12 to publish a
`SlotMeterSnapshot` and to assert that a view reads it through `CoreHost::slot_measure` in the same
frame. Restate the C22I-W8 row.

---

## WARNING 14. The `CoreReceiver` verdict row omits `Clone`, and two declarations rest on it

**OBSERVATION.** The `external-verdicts` row for `Sender` at `architecture.md:2152` carries `Clone` in
its Traits cell. The row for `CoreReceiver` at `architecture.md:2156` carries "none". The `CoreClient`
doc at `architecture.md:13540` reads "`async_channel::Receiver` is `Clone`, so the drain task of section
10.2 takes a clone and the owner keeps one". The `JobWorker` doc at `architecture.md:9106` reads
"`queue` is a clone of the read end of the B36 job queue, which is what `async_channel` allows and
`triple_buffer` does not: many workers take from one queue".

**CLAIM.** The one table that decides an external name's traits gives two answers to one question. The
B36 job-queue design and the section 10.2 drain task each need a trait the table records as absent.

**ARGUMENT.** The Traits column is free text for `Clone`, because PG19 closes over nine traits and
`Clone` is not one of them (`tools/placement_check.py:344`). The `Sender`, `Shared` and `SyncSender`
rows each volunteer `Clone`; the `CoreReceiver` row does not. A reader who takes the table as the
decision, which DR4 tells him to do, concludes that B37 workers cannot share one B36 queue. PG25
compiles the roster, so a WRONG positive claim is caught and an OMITTED one cannot be.

**EVIDENCE.** `architecture.md:2152`, `:2156`, `:9106`, `:13540`; `tools/placement_check.py:344`.

**WHAT SHOULD CHANGE.** Put `Clone` in the `CoreReceiver` Traits cell. State at the table's limit
paragraph that `Clone` is recorded and not proved, so a reader knows which half of a cell the compile
holds.

---

## WARNING 15. The `Input` verdict row states a count of three where seven write ends exist

**OBSERVATION.** The `external-verdicts` row for `Input` at `architecture.md:2150` reads
"`triple_buffer::Input` is the write end of the same value... **Section 5.8 names three of them** and
revision 13 decided none". The `snapshot-table` block at `architecture.md:6738` holds seven rows, the
`carrier-table` holds seven `triple_buffer` rows, and seven `Input` fields are declared:
`DuetCore.tempo`, `DuetCore.params`, `DuetCore.plan`, `GraphConfigurator.topology`,
`EngineProcess.transport_out`, `EngineProcess.meters` and `EngineProcess.slot_meters`.

**CLAIM.** A bare count inside the one external table is refuted by two registered blocks of the same
document.

**ARGUMENT.** The count was true of revision 13, which the same sentence dates. Revision 22 added the
third audio publication and revision 23 declared every write end, and the cell did not move. No rule
reads it: PG26 reads the `Why` column for the words "defers", "grows", "lock" and "heap" alone, and
PG19 reads the Traits column. This is the shape of C22I-W2 and C22I-W8, which revision 23 closed
elsewhere in the same changeset.

**EVIDENCE.** `architecture.md:2150`, `:6738` to `:6746`, `:6849` to `:6871`, `:13502`, `:13506`,
`:13508`, `:5559`, `:13228`, `:13231`, `:13233`.

**WHAT SHOULD CHANGE.** Delete the count and cite the `carrier-table` block, which is the one place the
write ends are now decided.

---

## WARNING 16. B131's derivation does not produce the value B131 states

**OBSERVATION.** B131 at `architecture.md:1314` states the value "3 s" and the derivation "**It is B53
plus one B118**, because the disk thread must drain a full capture ring and then finish the container
header". B53 at `architecture.md:1239` is "3 s per channel". B118 at `architecture.md:1323` is "200 ms".

**CLAIM.** B53 plus B118 is 3.2 s and the row states 3 s.

**ARGUMENT.** This is the class C22I-W14 closed for B134 on the same quit path one revision ago. B134 at
`architecture.md:1316` states its own margin in words, so the document shows that it knows how to write
one. B131 states none, and a reader who checks the arithmetic cannot tell whether 3 s or 3.2 s is the
decision. I checked the corrected B134 arithmetic and it is right: 5 temporary files, 4 directories at
step 2, 2 fixed calls at step 3, 4 directories at step 4 and 1 fixed call at step 5 is 16 calls, and 16
times 200 ms is 3.2 s.

**EVIDENCE.** `architecture.md:1239`, `:1314`, `:1316`, `:1323`.

**WHAT SHOULD CHANGE.** State the margin, or raise the value to the number the derivation gives.

---

## CONCERN 1. `closure_check.py` still documents the `--no-store` flag its code removed

**OBSERVATION.** The module docstring at `tools/closure_check.py:18` reads "usage: closure_check.py
<architecture.md> <review.md> <block id> [repo-root] [--no-store]". Line 70 reads "`--no-store` prints
`STORE: skipped` and is for a machine with no store; `run_all_gates.py` never passes it." `main` at
`tools/closure_check.py:369` accepts four or five arguments and no flag.

**ARGUMENT.** I ran the flag and the guard is fail-closed, which is correct:

```
python3 tools/closure_check.py architecture.md reviews/critic-spec-r22-inner.md closure-r22-inner /Users/james/Developer/duet --no-store
usage: closure_check.py <architecture.md> <review.md> <block id> [repo-root]
exit=2
```

Section 1.9 makes these four prototypes the rules an implementer reads. The docstring of the CL1c
prototype still states the fail-open that C22I-W4 removed.

**WHAT SHOULD CHANGE.** Delete both sentences.

---

## CONCERN 2. Three cross-thread atomics carry a fact and no carrier row names them

**OBSERVATION.** `MidiSink.dropped` at `architecture.md:13285` is `Arc<AtomicU32>` with the doc
"`duet-core` holds the other handle and reads it once per frame". `DuetCore.entry_dropped` at
`architecture.md:13464` is the other handle. `HotplugSink.resync` at `architecture.md:13296` is
`Arc<AtomicBool>` with the doc "The MIDI thread reads it, so both threads hold a handle". The PG41
limit paragraph at `architecture.md:1038` declines "a `basedrop::Shared<T>` moved into a callback
closure and the `Arc<ResetGenerations>` of the audio-exempt block" and names nothing else.

**ARGUMENT.** Each of the three carries a FACT from one thread to another: a drop count that becomes
`CoreEvent::EntryDropped`, and a resync flag that makes the MIDI thread rescan. TH13 binds "every
cross-thread duty", and the stated limit excludes two shapes that are not these. So three carriers sit
outside both the rule and its stated limit, and a reader cannot tell whether that is a decision or an
omission.

**WHAT SHOULD CHANGE.** Name the shared-counter shape in the PG41 limit paragraph and say why a row
would add nothing, or give each one a row.

---

## CONCERN 3. One carrier row gives `HotplugSink` two roles that its own doc gives one owner

**OBSERVATION.** The `PlatformPort` row at `architecture.md:6856` names `HotplugSink.ports` as both the
Sender and the Receiver, with the Threads cell "Platform presence listener -> MIDI". `HotplugSink` at
`architecture.md:13291` carries the doc "Where a raw platform port goes. The presence source owns one,
and it pushes into the B87 queue that the MIDI thread drains".

**ARGUMENT.** The block states that one field may carry both ends "when the carrier is a
`crossbeam_queue::ArrayQueue` behind an `Arc`", which holds here. The type's own doc gives the value one
owner, the presence source. So the row asserts that the MIDI thread also holds a `HotplugSink` and the
doc says it does not. The `FaultQueue` row is clean by comparison, because `FaultQueue` is a shared
value that two declared fields hold.

**WHAT SHOULD CHANGE.** State that the MIDI thread holds its own `HotplugSink`, or name the MIDI
thread's own field as the Receiver.

---

## CONCERN 4. CL1c has no repair path after a second append of one suffix

**OBSERVATION.** `sibling_keys` at `tools/closure_check.py:314` reads every stored key that shares one
suffix, and the store half fails when any stored copy differs from the file. `plan-db` offers no
command that removes a key from the append-only keyspace.

**ARGUMENT.** The rule is correct and it closes C22I-W5. Its cost is that one accidental second append
under a suffix, from a retried or interrupted Critic run, makes that closure gate red for ever with no
way to remove the extra copy. The plan should state that cost at the CL1c site, because the party that
pays it is the next reviewer.

**WHAT SHOULD CHANGE.** State the one-append rule at the CL1c site, and state what a second append
costs.

---

## CONCERN 5. `CoreHost::slot_measure` cites a rule that the cited site does not state

**OBSERVATION.** The doc at `architecture.md:13836` reads "Section 1.3 rule 6 keeps every publication
read end out of `crates/duet`". Rule 6 at `architecture.md:358` reads in full "`crates/duet` depends on
no crate that opens a file. It has no `duet-media` edge and no `duet-project` edge. Every byte it paints
arrives through `duet-core`."

**ARGUMENT.** DR4 makes a citation point at the site that states the rule. Rule 6 is about file access
and crate edges. `CoreHost` at `architecture.md:13768` declares `meters: MeterReader`, and `MeterReader`
at `architecture.md:13601` declares two `triple_buffer::Output` fields, so a read end does reach a
`crates/duet` type through one field. The reason `slot_measure` must exist is real and the citation does
not carry it.

**WHAT SHOULD CHANGE.** Cite section 5.8, which states that `duet-core` owns every read end, or write
the rule at one site and cite that site.

---

## CONCERN 6. Four bare counts survive inside ADR 0004 after N22I-7 was recorded CLOSED

**OBSERVATION.** DR6 at `architecture.md:73` requires every number inside an ADR to be a citation. The
C.27 row for N22I-7 at `architecture.md:15823` reads "**CLOSED** | 1.7 and the four ADR clauses it
governs under DR6". Four bare counts survive in ADR 0004 alone:
`adr/0004-backend-and-threading-contract.md:202` "Specification section 8.2 holds the four steps";
`:227` "by the three named means of specification section 5.7"; `:233` "Specification section 11.6
holds the three rules"; `:281` "Five crates carry the real-time discipline".

**ARGUMENT.** The r22 finding named a CLASS and not four sites. Revision 23 corrected the four sites the
review listed and swept none of the others. The site at `:281` is worse than a count: it cites
"Specification section 1.2 lists them in the `duet-engine` row", and that row lists ten third-party
dependency names and no list of five crates. Revision 23 itself added `async-channel` to that row for
the B138 channel, so the cited evidence moved in this changeset and the count did not.

**WHAT SHOULD CHANGE.** Replace each of the four with a citation. Decide what the `:281` consequence
means and cite a place that states it.

---

## CONCERN 7. Appendix A has no row for `EngineDisk`, the root revision 23 added

**OBSERVATION.** Appendix A at `architecture.md:14427` states its own completeness: "Every row names one
state, its one owner, the one path that mutates it, who observes it, and the source of its identity."
`EngineDisk` at `architecture.md:12885` is declared as "The engine disk thread's own state, and the ROOT
of that thread". No row of Appendix A names the engine disk thread, `EngineDisk`, or any value it holds.

**ARGUMENT.** C22I-W7 closed three WRONG rows in this appendix. The new defect is an absent one. The
table carries a row for the audio thread's state at `:14447` and for the engine handoff thread's state
at `:14448`, so the engine disk thread is the one engine thread with no row, and it now owns every take
file, the B33 consumer and the ring ends B140 bounds. No rule reads Appendix A content.

**WHAT SHOULD CHANGE.** Add one row for `EngineDisk`: the owner, the mutation path, the observers and
the identity.

---

## CONCERN 8. The owed-note list names one site, and PR R-14 is a second

**OBSERVATION.** The How-to-read rule at `architecture.md:17` now binds `product-requirements.md`.
Lines `:21` to `:24` name ONE owed note, at design contract 5.3. `product-requirements.md:477` states
the R-14 criterion "when the user opens it, then the application names the time of each drop", with no
note. B126 at `architecture.md:1310` records that the plan defers that criterion.

**ARGUMENT.** The rule's own words are that "a superseded line that stands with no note is a second
answer a reader can find and follow". A deferral is that shape: a chunk K6 implementer who reads R-14
builds a per-drop time list that no declared value can fill, and the record that says otherwise is B126,
in a file he is not reading. The PARTIAL row at `:15824` is honest about the one note it names; the list
is short.

**WHAT SHOULD CHANGE.** Name PR R-14 beside contract 5.3 in the owed-note paragraph.

---

## CONCERN 9. Section 7.3 gives `SlotMeterSnapshot` an ordinal the table refutes

**OBSERVATION.** Section 7.3 at `architecture.md:7936` reads "It is the seventh row of the section 5.8
high-rate table". The `snapshot-table` block at `architecture.md:6738` holds seven rows and
`SlotMeterSnapshot` is the third. The seventh is `Input<PlaybackPlan>`, a core-to-disk publication.

**ARGUMENT.** C22I-W8 deleted every count of audio publications from six sites and left this ordinal,
which is the same hand number over the same table. DR3 exemption 4 covers a count a guard prints, and no
guard prints a row ordinal.

**WHAT SHOULD CHANGE.** Delete the ordinal and cite the `snapshot-table` block.

---

**Counts: 6 Criticals, 16 Warnings, 9 Concerns.**

## What is genuinely closed, and I verified it

C22I-5 is closed: fifteen gates are green from a frozen copy. C22I-W1, C22I-W3, C22I-W4, C22I-W5,
C22I-W6, C22I-W7, C22I-W9, C22I-W10, C22I-W11, C22I-W13, C22I-W14, C22I-W16, C22I-W17, C22I-W18,
N22I-1, N22I-2, N22I-3, N22I-4, N22I-6, N22I-8 and N22I-9 each hold at every site I read. VR7 states
the public-field rule once and exactly three declarations carry a public field, which is the stated
exemption and no fourth. B90 sums to 1413 with the gate removed. B133 is 768 cells and 12,288 bytes
over one array. Chunk D3 reads eight `SlotKind` arms. Chunk C3 selects five tests. The section 12.4
fault path holds a declared field at every hop. The `--no-store` fail-open is gone from the code and
the symbolic-link shape is refused on a resolved path.

## Reactive Assessment

- **Responsive: PARTIAL.** Every wait of the quit path is now a message the core measures, every step
  has its own bound, and `ConfigureCommand::Stop` gives B119 a producer. Three holes remain on that
  path: the terminal send has no retry (WARNING 6), the one-slot retry cannot hold a per-track burst so
  B131 reaches expiry and marks good takes (WARNING 5), and no step sends `ConfigureCommand::Reset`
  (WARNING 8).
- **Resilient: PARTIAL.** The fault path from the audio thread to the user interface is declared at
  every hop and I verified each one. The disk thread holds no writer and no reader, so the take path it
  owns has no declared failure surface (CRITICAL 2); the held-note silence pass cannot address its own
  array (CRITICAL 4); and a dropped `GraphConfigured` loses a topology change with no user message
  (WARNING 5).
- **Elastic: PASS.** Every channel, ring, queue and array carries a bound, a budget row and a stated
  overflow action, and B140 to B142 close the three that had none. The one unbounded collection of
  revision 22 is bounded and its drop rule is stated.
- **Message Driven: PARTIAL.** The `carrier-table` is the right mechanism and 23 boundaries now hold
  two declared ends. The seam between the two runtimes of this process holds none (CRITICAL 1), and
  PG41 proves that two names are declared rather than that they are the two ends (WARNING 1).

## Verdict: NOT READY FOR PLAN AUTHORING

The engineering judgement of this revision is the best of the series. TH13 and PG41 are the right rule,
written in the right place, with a readable denominator and a probe. The 23 carrier rows found six
missing write ends, four missing structural ends, a thread with no type, a note set, a stop command and
a second fault handle, and every one of those is now declared. C22I-5 is closed and I measured fifteen
green gates on a frozen copy.

The single biggest risk is that the rule was written one level above the defect. TH13 reads the ENDS,
and the class is wider than the ends. `EngineDisk` holds every channel end the table asks for and no
file handle, so the thread that owns every take file cannot open one. The agent gateway holds no ends
at all, so the reverse half of PG41 has nothing to find and the forward half reads only the rows the
Architect wrote. Four of my six Criticals are the sentence the twenty-second review wrote: a stated
duty whose carrier no declaration holds. The carrier is sometimes a channel. It is also a file handle,
an index, a key and a chunk.

The weakest Reactive property is Message Driven, for the reason the run cannot show: the one boundary
between tokio and GPUI is undeclared in a document whose new rule exists to declare exactly that.

## The blocking list, in the order I would fix it

1. **CRITICAL 1.** Declare the agent-gateway seam and add its two carrier rows. Add the crate entries
   that let a crate spell the type.
2. **CRITICAL 2.** Give `EngineDisk` the writer, the reader, the peak builder and the per-track
   accumulation, then re-audit C22I-2 and C22I-4.
3. **CRITICAL 5 and CRITICAL 4.** Key `take_flags` by `TrackId` and give the sounding-note array an
   index the message can carry.
4. **CRITICAL 3.** Fix the C2 and C3 order, because no chunk of line C can run until it holds.
5. **CRITICAL 6, WARNING 10, WARNING 11.** Give every mechanism revision 23 added one chunk goal and
   one selected test. The count over section 13 is the check.
6. **WARNING 1.** Close the two PG41 holes before the next freeze, so the next reviewer measures a rule
   and not a reading.
7. **WARNING 5, 6, 8.** Close the three quit-path holes, which all sit on the path C21-3 and C22I-3
   opened.
8. **WARNING 13.** Extend test 12, because it is the one oracle of the frame order and it does not read
   the publication C22I-8 depends on.
9. **WARNING 2, 3, 4, 7, 9, 12, 14, 15, 16.** Correct every restated fact and every number the artifact
   refutes.
10. **The Concerns.** Fix in the changeset, or file each one in a tracked document under `roadmap/`.
