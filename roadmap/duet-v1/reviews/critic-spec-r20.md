# Engineering Critic, specification review, revision 20

Scope: the frozen `architecture.md` at 13570 lines, the six ADRs, every file under `tools/`,
`design-contract.md`, `product-requirements.md`, and `research/`. Cross-checked against
`/Users/james/Developer/duet`: `CLAUDE.md`, `Cargo.toml`, `.cargo/config.toml`, `clippy.toml`,
`deny.toml`, `scripts/dod.sh`, `.github/workflows/ci.yml`, and `crates/duet/src/`.

md5 of `architecture.md` at the start of the run: `bc39f6132246c215bbd02d5d475964f2`.
md5 at the end of the run: `bc39f6132246c215bbd02d5d475964f2`. The two agree.

I ran every check the Architect reported as green. Two of them are RED on this tree. I also planted
defects against the guards. One planted deletion reached a green run.

## CRITICAL 1. The probe harness does not run, so no guard rule is verified in revision 20

**OBSERVATION.** `python3 tools/probe_run.py architecture.md` exits 1 with a Python traceback and
runs zero probes:

```
File ".../tools/probe_run.py", line 912, in block_probes
  shapes.append((f"PP27:{block_id}:stray", add_row(block_id, STRAY_ROWS[block_id])))
KeyError: 'closure-r19'
```

Exit code, measured twice: `1`. The Architect reported `exit 0`, `PROBES BAD: 0  RECORDED
FRAGMENTS: 73  FLOOR: 43`. That output cannot come from this tree.

**CLAIM.** Every one of the 58 rows of the section 1.9 probe table is an unverified assertion in
revision 20. The recorded exit codes and the recorded messages are text, not evidence.

**ARGUMENT.** Reading a guard shows what it accepts. Only a run with hostile input shows what it
rejects. `probe_run.py` is the only mechanism in this plan that plants a defect per rule and proves
the rule refuses it. Section 14 rung two states the contract: "section 1.9 pairs every rule with its
one probe and **records the result of the run against this revision**". No run against this revision
exists, because the harness aborts before the first probe starts. The root cause is
`placement_check.DATA_BLOCKS`, which gained a `closure-r19` entry at line 459, against a
`STRAY_ROWS` table in `probe_run.py` that holds rows for r16, r17 and r18 only. `block_probes()`
iterates `DATA_BLOCKS`, so the new block kills the harness. The declared gap about the lost
nineteenth review explains why `reviews/critic-spec-r19.md` is absent. It does not explain why a
closure block was registered for a review file that can never exist, and it does not excuse a green
claim over a red run.

**EVIDENCE.**
`/private/tmp/claude-501/.../frozen-r20/duet-v1/tools/probe_run.py:912`;
`/private/tmp/claude-501/.../frozen-r20/duet-v1/tools/placement_check.py:459`;
`ls reviews/` holds `critic-spec-r2.md` to `critic-spec-r18.md` and no r19 file;
`python3 tools/closure_check.py architecture.md reviews/critic-spec-r19.md closure-r19` exits 2 with
`FAIL: cannot open reviews/critic-spec-r19.md; the guard is fail-closed.`

**WHAT SHOULD CHANGE.** Add the `closure-r19` row to `STRAY_ROWS`, or remove the `closure-r19`
registration until a review file backs it. Then run `probe_run.py` and record the real transcript.
Until a run produces `PROBES BAD: 0` against this document, the section 1.9 table states no proven
property. The rule that PG29 scans the four prototypes for their own rule ids does not help here:
PG29 checks that the ids are present, and a harness that aborts emits no id at all.

## CRITICAL 2. The section 1.9 roster transcript is three revisions stale, and its two guards are red

**OBSERVATION.** Section 1.9 records the PG25 baseline transcript at lines 1554 to 1565:
`ROSTER ITEMS: 395  FLOOR: 395`, `ROSTER IMPL BLOCKS: 50  FLOOR: 50`, `ROSTER SIZES: 419 measured`.
A real `roster_compile.sh` run against this document prints `ROSTER ITEMS: 398  FLOOR: 398`,
`ROSTER IMPL BLOCKS: 52  FLOOR: 52`, `ROSTER SIZES: 422 measured`. I counted the `impl-sites` guard
block myself: it holds 52 rows. `probe_roster_text.py` exits 1 and `probe_roster.sh` exits 1, both
on the text compare:

```
TEXT:  1.9 transcript: the baseline roster run does not print it: ROSTER ITEMS: 395 FLOOR: 395
TEXT:  1.9 transcript: the baseline roster run does not print it: ROSTER IMPL BLOCKS: 50 FLOOR: 50
TEXT:  1.9 transcript: the baseline roster run does not print it: ROSTER SIZES: 419 measured
TEXT:  PP25: the cell records 8 output lines and the floor is 14; the harness is fail-closed.
```

The recorded line numbers are stale too: the cell records `duet-engine/src/lib.rs:166`, `:178` and
`duet-dsp/src/lib.rs:97`; the run produces `:167`, `:179` and `:99`.

**CLAIM.** The document's recorded guard output is not the output of any run against the document.
Section 14's sentence "section 1.9 ... records the result of the run against this revision" is false
for PG25 as well as for the probe table.

**ARGUMENT.** Chunk M0 is instructed to port "the recorded TEXT of every cell as well as its exit
code", because a Critic run once rewrote a recorded message and kept a green guard (critic C-2). A
ported test that asserts `ROSTER ITEMS: 395` will fail on the first run, because the document itself
now places 398 names. The stale cell therefore blocks the chunk that depends on it, and it does so
at the exact place the plan chose to defend against a forged transcript.

**EVIDENCE.** `architecture.md:1554-1565`; the `impl-sites` block at `architecture.md:2041` with a
declared minimum of 52 and 52 rows; the run output above.

**WHAT SHOULD CHANGE.** Re-run `roster_compile.sh` and `probe_roster.sh`, paste the real transcript
into the section 1.9 PG25 cell, and re-run `probe_roster_text.py` until it exits 0. The cell must
also hold at least 14 output lines, which its own floor states and which it does not meet.

## CRITICAL 3. The audio thread's real root is undeclared, so six heap-owning handles sit outside every audio guard

**OBSERVATION.** `AudioBackend::open` takes a `Box<dyn AudioProcess>` (section 5.1). `AudioProcess`
is a trait. No type in the document implements it. The `impl-sites` guard block holds 52 rows and no
`AudioProcess` row. The section 1.5 placement table for `duet-engine` names `AudioProcess` and names
no implementor. Section 5.6 then states where the audio thread keeps its read ends: "The three
`triple_buffer::Output` values the audio thread reads sit in the backend callback's own state".
`GraphState` holds five fields, and none of them is a fault queue end, a `RefillRequest` producer,
or a `MidiRecord` consumer, although section 5.8 puts the audio thread on one side of all three.

**CLAIM.** The audio-thread guard set is rooted at `GraphState`, and `GraphState` is a field of the
real root. Six heap-owning handles live above it, in a type no declaration names, and PG26, PG26b,
PG26c, PG26e and PG26f cannot read one of them.

**ARGUMENT.** PG26's own rule text states the property it claims to enforce: "A bare `rtrb`,
`triple_buffer`, `crossbeam_queue`, `async_channel`, or `std::thread` handle inside an audio-owned
declaration is a failure, because every one of those owns a heap allocation that a drop on this
thread would free (critic CR-16)." That rule is the reason `GraphState::handoffs` is a
`HandoffReader(Owned<Consumer<GraphHandoff>>)` and not a bare `Consumer`. The same rule applies word
for word to the fault producer, the refill producer, the MIDI consumer, and the three
`triple_buffer::Output` values. Not one of the six is declared anywhere, so the rule reaches none of
them. The document then claims the opposite: "PG26 refuses a bare `rtrb` end inside any audio-owned
declaration, **so no reader has to trace which thread drops which value**". A reader must trace
exactly that for the six handles this design leaves undeclared. The closure rule PG26e is described
as CLOSED: "every reachable name is a root or is here, and nothing falls between them". The closure
is closed below `GraphState` and open above it.

**EVIDENCE.** `architecture.md:4317` (`pub trait AudioProcess`); `architecture.md:2041-2093` (the 52
impl sites, with no `AudioProcess` row); `architecture.md:401` (the `duet-engine` placement row);
`architecture.md:5641-5665` (the five fields of `GraphState`); `architecture.md:5646` ("the backend
callback's own state"); `architecture.md:6096-6098` (the three ring rows whose audio end has no
declared home).

**WHAT SHOULD CHANGE.** Declare the concrete `AudioProcess` implementor, place it in section 1.5 and
section 15.10, add it to the `audio-owned` block, and give every handle it holds the
`basedrop::Owned` wrapper the rule already demands. Raise the `audio-owned` floor by the number of
new roots. Add an `impl-sites` row for the trait impl. Without this the whole audio-thread guard
proves a property about a subtree and states it about a thread.

## CRITICAL 4. B116 gives a blocking call a cancel that no declared mechanism can perform

**OBSERVATION.** Three places state a cancel. Budget row B116: "On expiry the thread abandons the
stop, drops the last `GraphState` anyway, and answers `EngineEvent::StreamClosed` with the refusal,
**so a device that hangs cannot hold the shutdown open for ever**." Section 5.12: "The stop is
abandoned, the last `GraphState` is dropped anyway". Appendix B.5 then states the mechanism:
"`std::time::Instant` around `AudioBackend::stop`, read on the thread that owns the backend."

**CLAIM.** An `Instant` read around a synchronous call cannot abandon that call. B116 is a timeout
that cannot expire, and the property it advertises is exactly the one it does not have.

**ARGUMENT.** `AudioBackend::stop(&mut self)` is a blocking method on the engine handoff thread, and
section 5.5 names that thread as its one caller. If `stop` returns, the elapsed time is known and
the shutdown already completed, so no action is left to take. If `stop` hangs, the next statement
never runs, the `Instant` is never read, and the thread stays inside the call. The document knows
this rule and applies it correctly one row lower, for B117 and B118: "**The bound is a REPORT and
not a cancel**, because a blocking file call on macOS and on Linux cannot be interrupted without a
second thread". A device teardown is the same class of call. The same appendix opens with the rule
that catches this: "A feature list that omits the mechanism for a declared timeout is a
contradiction inside one document, and revision 7 carried one (critic Q9)." Revision 20 carries one
again, and it carries it in the row that closes C19-7.

Two further defects follow from the same row. First, the prescribed expiry action is unsound even if
it could fire. `open` moved the process body into the callback state, so the handoff thread does not
own `GraphState` and cannot drop it without dropping the stream, which is the call that hung. A drop
that proceeds while `stop` has not returned releases values that the audio callback may still read;
section 5.5 states the correct rule for the success path and then breaks it on the expiry path: "it
can do that because `stop` has returned, which is `cpal`'s own guarantee that the callback will not
run again." Second, B18 has the same shape. Appendix B.5 gives B18 the mechanism "`std::time::Instant`
plus a bounded `recv_timeout`" on "the engine thread". Section 5.5 calls `AudioBackend::open`
directly through `self.backend` on the engine handoff thread, so no channel receive exists on that
path, and the thread table of section 5.7 has no row named "the engine thread".

**EVIDENCE.** `architecture.md:1136` (B116); `architecture.md:6396` (the 5.12 stop row);
`architecture.md:12747` (the B.5 B116 mechanism); `architecture.md:12748` (the B117 and B118 row
that states the correct rule); `architecture.md:12745` (the B18 mechanism);
`architecture.md:5225-5233` (the five shutdown steps); `architecture.md:5926-5940` (the thread
table).

**WHAT SHOULD CHANGE.** Choose one. Either state B116 as a report, in the words B117 and B118
already use, and accept that a hung device holds the shutdown, or declare the second thread and the
join that a real cancel needs, and state what happens to the stream handle when the join expires.
Do not leave a sentence that promises a bound the mechanism cannot deliver. Give B18 the same
treatment and name a thread the thread table holds.

## CRITICAL 5. The engine fault queue carries two contradictory declarations, and C19-10 is not closed at section 5.8

**OBSERVATION.** Section 5.8 is the section that decides which mechanism crosses each boundary. Its
row for the fault queue reads: `| Audio | Disk | rtrb ring of EngineFault, B34, plus an AtomicU32
drop counter | One |`. Section 12.4 step 1 reads: "The audio thread pushes a `Copy` record into
`ArrayQueue<EngineFault>` at B34." Section 12.4 step 2 reads: "**The engine HANDOFF thread drains the
queue**". Section 5.10 agrees with 12.4: "**The disk thread drains no fault queue** (critic C19-10)."
The B14 budget row agrees too.

**CLAIM.** The C19-10 fix was applied in section 5.10, section 12.4 and the B14 row, and it was not
applied at section 5.8. The two declarations disagree on the consumer and on the primitive, and the
12.4 form names a crate that `duet-engine` does not depend on.

**ARGUMENT.** Three consequences follow, and each one is checkable.

First, the consumer. Section 5.8's row still names the disk thread, which is the thread the
revision-20 fix removed from this path. Section 5.8 carries the rule that decides these rows: "One
primitive per boundary, and the declaration agrees with the row (critic WR-11) ... No boundary names
two." This boundary names two.

Second, the crate. The section 1.2 crate table gives `duet-engine` the third-party set `serde`,
`cpal`, `rtrb`, `triple_buffer`, `basedrop`, `arrayvec`, `thiserror`, `tracing`. It holds no
`crossbeam-queue`. Appendix B.3 states the whole reason for that pin: "The three bounded queues that
must evict: the note-entry queue at B31, the presence queue at B87, and the hot-plug queue at B100".
The fault queue is not one of the three. The closure row for WR-11 states the rule outright: "1.2,
where `duet-core` carries `crossbeam-queue` and `duet-engine` carries none". So
`ArrayQueue<EngineFault>` is a type `duet-engine` cannot name, and the 12.4 declaration does not
compile in the crate that owns the fault path.

Third, the audio end has no declared home, which CRITICAL 3 states. The one path by which every
engine fault reaches the user is therefore declared twice, in two mechanisms, with no owner on the
producer side.

The Reactive cost is direct. Section 12.4 argues, correctly, that "The fault path must be the
fastest path in the engine, not the slowest." The argument is sound and the declaration that carries
it is not settled.

**EVIDENCE.** `architecture.md:6098` (the 5.8 row); `architecture.md:9092-9094` (12.4 steps 1 and 2);
`architecture.md:6265` (5.10); `architecture.md:1034` (B14); `architecture.md:165` (the `duet-engine`
crate row); `architecture.md:12675` (B.3 `crossbeam-queue`); `architecture.md:13251` (the WR-11
closure row); `architecture.md:6080-6086` (the one-primitive rule).

**WHAT SHOULD CHANGE.** Decide the mechanism once, at section 5.8, and make every other site follow.
If the answer is `ArrayQueue`, add the `crossbeam-queue` edge to the `duet-engine` row of section
1.2, add the fault queue to the B.3 reason cell, and correct the WR-11 closure row. If the answer is
`rtrb`, correct section 12.4 and state which thread holds the single consumer. Then declare the
producer end on the audio side.

## WARNING 1. `pending_configure` has one drain trigger and no invalidation

**OBSERVATION.** "**The trigger set is `EngineEvent::GraphConfigured` alone.** On that event, and only
on that event, the core makes ONE `try_send` of whatever `pending_configure` holds". The field is
cleared on a successful send and on `EngineEvent::ConfigureRefused`. No rule clears it on
`EngineFault::HandoffBusy`, on `EngineEvent::StreamClosed`, or on a stream re-open.

**CLAIM.** Two reachable states leave a topology change queued with no event that can ever drain it,
and one of them later replays a request that names a dead stream.

**ARGUMENT.** Past B95 the engine handoff thread "reports `EngineFault::HandoffBusy`, drops
`pending_request` into `basedrop`, and **stops building until a `ConfigureCommand::Reset` arrives**".
The engine then moves to `EngineState::Faulted`. In that state no `GraphConfigured` event is
produced, so a `pending_configure` set during the window cannot be sent. The user must close and
re-open the device, which the fault message says. A restart sends `Close`, `Reset` and `Open`, and
none of the three clears the field. The first `GraphConfigured` after the restart then sends the
stale `ConfigureRequest`. That request carries a `generation` and a `stream: StreamInfo`, and
`StreamInfo` is "The stream the rings are sized for", so `configure` would size rings for the device
that is gone. The second state is narrower: a full B109 channel holding only `Close` and `Reset`
commands produces no `GraphConfigured`, so the request waits with no trigger.

**EVIDENCE.** `architecture.md:5183-5190` (the trigger and the two clears); `architecture.md:5149-5152`
(the B95 stop rule); `architecture.md:4983-4990` (`ConfigureRequest::stream`);
`architecture.md:11705` (`pending_configure: Option<Box<ConfigureRequest>>`).

**WHAT SHOULD CHANGE.** Clear `pending_configure` on `EngineEvent::StreamClosed` and on
`EngineFault::HandoffBusy`, and state the rule beside the two clears that already exist. A queued
request that names a closed stream is worse than a lost one.

## WARNING 2. The section 5.11 chunk form does not compile, and its stated reason names the wrong type

**OBSERVATION.** Form 2 of the house idiom reads `output.chunks_exact_mut(channels.output().get())`,
and its reason reads "**The argument is a `ChannelCount`, which wraps a `NonZeroU16`**, so the one
panic `chunks_exact_mut` carries cannot be reached." `ChannelCount::get` is declared as
`pub const fn get(self) -> NonZeroU16`, with the doc comment "The count as a non-zero number, which
is what a chunk size needs."

**CLAIM.** `slice::chunks_exact_mut` takes a `usize`. The expression passes a `NonZeroU16`, so it
does not build, and the sentence that closes C19-8 names a type the standard library does not accept
there.

**ARGUMENT.** The argument at the call site is a `usize` on every correct spelling of this form. The
`ChannelCount` newtype is still the right design, because `usize::from(count.get().get())` is
infallible and never zero, so the panic does stay unreachable. The defect is that the document
declares one exact expression as the house form for every audio module, that expression is a type
error, and the chunk that writes it has no conversion site named. Section 12.3 form 3 requires
`duet-time::convert` for "every numeric narrowing", and this is a widening, so the implementer has no
stated rule either. This matters more than an ordinary typo, because section 5.11 says "Every audio
module uses these forms and no other".

**EVIDENCE.** `architecture.md:6319-6323` (form 2); `architecture.md:2294-2302` (`ChannelCount` and
`get`); `architecture.md:13566` (the C19-8 closure row, which cites "a `ChannelCount` chunk size").

**WHAT SHOULD CHANGE.** Write the form as it compiles, name the widening explicitly, and correct the
`ChannelCount::get` doc comment so it does not state a false fact about the standard library.

## WARNING 3. Form 1 of the audio idiom breaks the rule the section states for it

**OBSERVATION.** Form 1 is "A per-sample loop: `for (dst, src) in output.iter_mut().zip(input.iter())`.
**`zip` stops at the shorter side**, so a length disagreement truncates and never panics." The rule
that binds the six forms reads: "on the audio path a length is either **proved by a type**, as form 2
proves it, or **answered by an `Option`**, as forms 3, 4 and 5 answer it."

**CLAIM.** Form 1 does neither. It proves nothing and answers nothing. It converts a length
disagreement into a silent partial write, and the section's own binding rule excludes it.

**ARGUMENT.** Forms 3, 4 and 5 each return `CycleOutcome::Faulted(FaultCode::Misconfigured)` on a bad
length, so the failure reaches section 12.4's fault path and the user learns of it. Form 1 leaves the
tail of the output buffer holding whatever the previous cycle wrote. On a playback buffer that is a
repeated block, which a listener hears as a click or a stutter, and no fault is raised and no counter
moves. This is the same defect class that section 5.4 names and fixes for the capture shortfall: "The
first draft counted it and told nobody, so silence entered a vocal take in silence." Revision 20
closed C19-8 by proving the six forms are panic free. It did not check that every form still answers
a bad length, and one does not.

**EVIDENCE.** `architecture.md:6310-6312` (form 1); `architecture.md:6345-6348` (the binding rule);
`architecture.md:4560-4566` (the capture shortfall precedent).

**WHAT SHOULD CHANGE.** Either add a length check before form 1 that returns the same
`CycleOutcome::Faulted`, or state plainly that form 1 is legal only where a type already proves the
two lengths are equal, and name that proof.

## WARNING 4. The peak pyramid grows without a bound and without a budget row

**OBSERVATION.** `PyramidBuilder` holds `bins: Vec<PeakBin>`, documented as "Every closed bin, level
by level, in the pyramid's own order". `Pyramid`, the read type, holds the same field. B59 bounds
`derived/peaks/` on disk at "2 percent of the referenced audio bytes". No budget row bounds either
`Vec` in memory.

**CLAIM.** A long record pass grows a resident allocation on the disk thread without limit, and a
long take's pyramid loads whole into memory on the draw path. Neither cost appears in any budget.

**ARGUMENT.** B68 gives level `k` a bin per `2^(k+6)` frames, so level 0 is one bin per 64 frames. A
`PeakBin` is two `i16` and one `u16`, which is six bytes. One hour of stereo at 96 kHz is 345.6
million frames per channel, which is 5.4 million level-0 bins per channel, or about 65 MB for level 0
alone. The higher levels roughly double that, so about 130 MB per recorded hour, held in RAM and
never released during the pass. The whole engine's steady worst case, B55, is 104 MB. A field that
can pass the whole engine budget by itself, on the thread that is also the slowest, is an Elastic
gap: the memory grows with the length of the take and nothing pushes back. The design is careful
about exactly this kind of growth everywhere else, which is why the omission stands out.

**EVIDENCE.** `architecture.md:6294-6302` (`PyramidBuilder`); `architecture.md:10397` (`Pyramid`);
`architecture.md:1088` (B68); `architecture.md:1079` (B59); `architecture.md:1075` (B55).

**WHAT SHOULD CHANGE.** State that `PyramidBuilder` drains `bins` into the file on every append and
retains only the open bin per level, and give the read path a level selector so the draw path loads
the level the zoom needs. Add a budget row for whichever residency remains.

## WARNING 5. The cpal error callback is an undeclared thread that performs a declared push

**OBSERVATION.** The PipeWire host table states: "An xrun arrives as `ErrorKind::Xrun` | The error
callback pushes `EngineFault::PlaybackStarved` into the fault queue, exactly as a starved ring does."
The thread table of section 5.7 lists eleven threads and none of them is a cpal error callback.

**CLAIM.** A second producer pushes into a queue that section 5.8 declares single producer, from a
thread no rule of this design covers.

**ARGUMENT.** Section 5.8 states the rule: "**Every `rtrb` ring is single producer and single
consumer**". Under the 5.8 declaration the second push does not compile, because one `Producer`
exists and it is not `Sync`. Under the 12.4 declaration it compiles and is correct, because
`ArrayQueue` is multi producer. Either way the thread itself is undeclared, so no rule states what it
may allocate, whether TH1 binds it, or which thread drops the handle it holds. CRITICAL 5 must be
settled before this one can be, and this one must then be written down.

**EVIDENCE.** `architecture.md:4431` (the xrun row); `architecture.md:6091` (the single-producer rule);
`architecture.md:5926-5940` (the thread table).

**WHAT SHOULD CHANGE.** Add a thread-table row for the cpal error callback, state what it may do,
and make the fault queue's producer count match it.

## WARNING 6. Section 14 rung one and section 11.5 state four facts about the live repository that are false

**OBSERVATION.** I read the repository and compared it with the present-tense claims.

1. "This rung runs on `ubuntu-26.04` and on `macos-26`" and "**The gate runs on `ubuntu-26.04` and on
   `macos-26`**". `/Users/james/Developer/duet/.github/workflows/ci.yml:23` reads
   `os: [macos-latest, ubuntu-latest]`.
2. "The Linux job installs `libpipewire-0.3-dev`, `libasound2-dev`, and `pkg-config` before the gate."
   `ci.yml:37-41` installs `libasound2-dev` and `pkg-config` and no `libpipewire-0.3-dev`.
3. Section 11.5 says the Linux packages are "Installed by `scripts/bootstrap.sh`, and the CI job
   before the gate." `/Users/james/Developer/duet/scripts/bootstrap.sh` installs four cargo tools and
   no system package of any kind.
4. "`typos` is clean" is listed as a rung-one pass condition. `/Users/james/Developer/duet/scripts/dod.sh:93-96`
   runs `typos || printf ...`, so a typos finding cannot fail the gate.

**CLAIM.** Rung one is the measurable completion outcome, and four of its statements describe a
repository that does not exist yet, in the present tense, at the point of use.

**ARGUMENT.** Chunk M0 does own the CI file, so claims 1 to 3 are planned work. The defect is that a
reader of section 14 cannot tell a fact from a plan, and section 14 is the section whose whole job is
to state what "done" means. Claim 4 is different: it asserts a gate condition that the script cannot
enforce at all, and the appendix records the matching finding P25, "Rung one claims a gate `dod.sh`
does not run", as CLOSED.

**EVIDENCE.** `/Users/james/Developer/duet/.github/workflows/ci.yml:23,37-41`;
`/Users/james/Developer/duet/scripts/bootstrap.sh:15-31`;
`/Users/james/Developer/duet/scripts/dod.sh:93-96`; `architecture.md:10066-10069`, `:10058`,
`:8977-8980`, `:8984`, `:12937`.

**WHAT SHOULD CHANGE.** Write every rung-one line that names an unbuilt state in the future tense and
name the chunk beside it. Either make `typos` fail the gate or drop it from the pass conditions.

## WARNING 7. `cargo_common_metadata` cannot fire, so the description half of rung one is unguarded

**OBSERVATION.** Rung one states: "the `description` half alone is covered, by `clippy::cargo` denying
`cargo_common_metadata`." The workspace sets `publish = false` at `Cargo.toml:22`, and every member
inherits it.

**CLAIM.** `cargo_common_metadata` skips a package whose `publish` field is set, so the lint is not
firing in this repository and it covers nothing.

**ARGUMENT.** The evidence is in the tree, not only in the lint's documentation. The same lint also
demands `readme`, `keywords` and `categories`. No member manifest sets any of the three, and the
clippy gate is green on `main`. A lint that would deny three missing fields and denies none is a lint
that is not running. So the plan's one stated mechanism for "every crate declares a description" does
not exist, and fifteen new crates would land with no check.

**EVIDENCE.** `/Users/james/Developer/duet/Cargo.toml:22` (`publish = false`), `:82`
(`cargo = "deny"`); the three member manifests, none of which sets `readme`, `keywords` or
`categories`; `architecture.md:10049-10050`.

**WHAT SHOULD CHANGE.** Do not rest the check on this lint. Give chunk M0 an explicit manifest check
for `description` beside the `[lints] workspace = true` check it already owns, and correct the
sentence.

## WARNING 8. Two pins the plan requires carry no version, and the pins guard cannot express the feature negation

**OBSERVATION.** Appendix B.3 states the rule: each third-party crate "needs a survey row, a version
pin in `[workspace.dependencies]`, and a `cargo deny check` pass". `proptest` and `criterion` each
have a B.3 row, a survey entry and an owning chunk, and no version anywhere in the document. The
`pins` guard block holds exactly 12 rows and neither name. Separately, the block's line format cannot
carry `default-features = false`: it writes `tokio 1.49.0 rt-multi-thread net time signal io-util
sync macros`, while B.5 requires `default-features = false, features = [...]` for `tokio`, `cpal`,
`gix`, `rmcp` and `criterion`.

**CLAIM.** Two required pins are invisible to the guard that counts pins, and the guard that records
pins records a feature set that differs from the one B.5 mandates for five crates.

**ARGUMENT.** Finding P15, "`proptest` and `criterion` have no owner", is recorded as closed against
B.3 and 13.1. B.3 gave them an owner. It did not give them a version, which is the other half of the
rule B.3 itself states one paragraph earlier. The feature gap is the more expensive half: the roster
compile builds from the pins block, so it would resolve `tokio` with its default feature set, and the
resolution it proves is not the resolution the plan wants. The new PG36 rule checks pin OWNERS against
the 13.1 manifest table. It does not check that a pin exists or that its feature set matches B.5.

**EVIDENCE.** `architecture.md:12663` (the B.3 rule); `architecture.md:12669,12678` (the two rows with
no version); `architecture.md:1898-1912` (the pins block); `architecture.md:12721` (the B.5 `tokio`
row); `architecture.md:1424` (PG36); `architecture.md:12927` (P15 recorded closed).

**WHAT SHOULD CHANGE.** Pin `proptest` and `criterion` with a version, add both to the pins block and
raise its floor, and extend the block's line format so a `default-features = false` pin is expressible
and checkable.

## WARNING 9. A recorded closure cites an Appendix B.5 row that does not exist

**OBSERVATION.** The closure row for C16-W18 reads: "**CLOSED** | ... ; Appendix B.5, `blake3` with
`features = ["pure"]` and the reason it is not the `git2` case." Appendix B.5 holds eleven rows: cpal,
serde, clap, symphonia, gix, tokio, rmcp, criterion, smallvec, crossbeam-queue, arrayvec. There is no
`blake3` row. `blake3` appears in the document at four places and none of them is in B.5.

**CLAIM.** A finding is recorded as closed against a phantom reference, and the closure appendix is
the document's own audit trail.

**ARGUMENT.** The consequence is small and real. The survey records that `blake3` needs a C compiler
on x86 unless the `pure` feature is on. The Linux CI job does install `clang`, so the build would
succeed, but the decision the appendix claims to hold is not held anywhere, and a chunk that writes
the pin has no instruction. The larger cost is to the appendix. If one row can say CLOSED against a
target that is not there, then every row's weight drops, and Appendix C is the instrument this plan
uses to prove that fourteen reviews were answered.

**EVIDENCE.** `architecture.md:13422` (the C16-W18 row); `architecture.md:12714-12726` (Appendix B.5,
eleven rows, no `blake3`); `research/crate-survey.md` (the `pure` feature note).

**WHAT SHOULD CHANGE.** Add the `blake3` row to B.5 with the feature and the reason, or reopen
C16-W18. Then consider a guard: a closure cell that names an appendix row is a reference a script can
resolve, exactly as PG36 resolves a pin owner.

## WARNING 10. The `audio-owned` floor is one below its row count, and the block's stated property is therefore false

**OBSERVATION.** The block marker reads `<!-- GUARD BLOCK id=audio-owned rows>=37 -->` and the block
holds 38 rows. The baseline run prints `AUDIO OWNED: 38`. The document states the property the floor
is supposed to give: "A run whose parsed set is below the recorded minimum is a failure, so **a
deletion cannot pass as a clean run**."

**CLAIM.** One deletion does pass as a clean run. I planted it and measured the exit code.

**ARGUMENT.** I copied the frozen tree to a scratch directory, removed the `ChainState` row from the
`audio-owned` block, removed its `**Audio-owned**` marker, and added a `ChainState` row to the
`audio-reachable-leaf` block. `python3 tools/placement_check.py architecture.md` exited **0** with
`AUDIO OWNED: 37  ... ROOT BAD: 0` and `AUDIO REACHABLE: 37  REACHABLE LEAVES: 11  CLOSURE BAD: 0`.
The stated property does not hold at the current floor.

I then tested whether the disarm was effective, because a floor breach that disarms nothing is a
smaller finding than one that disarms something. I planted `scratch: Vec<f32>` and `guard: Mutex<u32>`
on `ChainState` in the same patched copy. The run exited **1** with `GROW IN AUDIO: 6` and
`LOCK IN AUDIO: 6`, because `GraphState`, `ChainSlot`, `ChainSetHandle`, `GraphHandoff` and
`HandoffReader` still reach `ChainState` through their own fields and the walk descends. The guard
held. So this is a Warning and not a Critical: the floor is wrong, the document's claim about the
floor is wrong, and the defence in depth saved it this time.

The cause is visible: revision 20 added `LimiterState` to the block, which took 37 rows to 38, and it
did not raise the floor. The same drift will repeat on the next addition, and the next one may be a
root that nothing else reaches.

**EVIDENCE.** `architecture.md:5766` (the marker) and the 38 rows under it; my planted runs, exit 0
then exit 1, in a throwaway copy at
`/private/tmp/claude-501/.../scratchpad/attack/duet-v1`. The frozen tree was not edited; its md5 is
unchanged.

**WHAT SHOULD CHANGE.** Raise the floor to 38 in the same changeset that added `LimiterState`. Better:
derive the floor from the row count at generation time rather than writing it by hand, so the two
cannot drift. Seven other blocks carry the same slack, which CONCERN 5 records.

## CONCERN 1. Section 5.12's completeness claim excludes a wait that Appendix B.5 carries

**OBSERVATION.** Section 5.12 states "**This table is every cross-boundary wait**". Appendix B.5's
timeout table holds a row for B108, "The bounded `recv_timeout` that parks the loop". Section 5.12
mentions B108 only as the period of B95 and gives it no row of its own.

**CLAIM.** The two tables disagree about what a cross-boundary wait is, and this is the third
revision in which they have disagreed.

**ARGUMENT.** The document records the same defect twice already: "Revision 17 left B24, B25, B26, and
B85 with a mechanism row in Appendix B.5 and no row here, so the one table the ADR cites was not the
whole table (critic N16-10, N17-2)." B108 is a timed park on a channel the core writes, so it is a
wait on another party under any reading the other rows use. The cost is low, because B108 has a
stated period and a stated action. The pattern is the finding.

**EVIDENCE.** `architecture.md:6405` (the claim); `architecture.md:12746` (the B.5 B108 row);
`architecture.md:6394` (the only B108 mention in 5.12).

**WHAT SHOULD CHANGE.** Give B108 a row in section 5.12, or state the exclusion rule that keeps it out
and apply that rule to the whole B.5 timeout table. A guard could hold this: the two tables are one
set, as PG36 now holds B.3 and B.5 against 13.1.

## CONCERN 2. "ADR 0004 clause 24" names a clause the ADR does not have

**OBSERVATION.** Section 5.12 reads "**This table is every cross-boundary wait, and ADR 0004 clause 24
names it**". `adr/0004-backend-and-threading-contract.md` numbers its decision clauses 1 to 22, with
lettered sub-clauses. There is no clause 23, 24, 25 or 26 in the file. The clause that actually names
the table is 17a: "**Every cross-boundary wait has a timeout**, and specification section 5.12
holds ...".

**CLAIM.** The citation is a phantom reference, in the one sentence that asserts the table's
completeness.

**ARGUMENT.** A reader who follows the citation to check the claim finds nothing. That is the whole
purpose of a citation in a document that carries this many of them.

**EVIDENCE.** `architecture.md:6405`; `adr/0004-backend-and-threading-contract.md:225` (clause 17a);
the clause enumeration, which ends at 22.

**WHAT SHOULD CHANGE.** Correct the citation to 17a. Consider a guard that resolves every "ADR NNNN
clause X" reference against the ADR file, which is the same shape as the existing cross-table rules.

## CONCERN 3. `LimiterState` takes a pool slot and no pool class holds it

**OBSERVATION.** `LimiterState { buffer: Option<PoolSlot>, write_head: u32, envelope, ceiling, release }`.
`BufferPool` holds two classes: `delays: Box<[PoolBuffer]>` at B47 slots of B49 bytes, and
`reverbs: Box<[PoolBuffer]>` at B48 slots of B50 bytes. `free: u64` covers B47 plus B48 bits. Nothing
states which class a `Limiter` slot draws from.

**CLAIM.** The look-ahead limiter that revision 20 added has a buffer handle and no stated home, and
the B47 and B48 arithmetic does not count it.

**ARGUMENT.** B47 is justified as a product decision: "an insert delay on a handful of lead tracks",
eight slots. B48 is four reverb slots for "two to four buses". Neither justification mentions a
limiter. PR MA-02 makes the master limiter a MUST, so at least one limiter exists in every project,
and contract 5.2 draws it. A limiter drawing a delay slot costs B49, which is 384 KB for a look-ahead
of a few milliseconds. That works and it is wasteful, and more to the point no chunk can write
`configure`'s slot assignment without the answer. `ConfigError::PoolExhausted { kind }` keys on the
class, so `kind` must be able to name what a limiter asked for.

**EVIDENCE.** `architecture.md:10440-10444` (`LimiterState`); `architecture.md:4771-4790`
(`BufferPool`); `architecture.md:1067-1070` (B47 to B50); `architecture.md:5296-5300` (the B47 and B48
justification).

**WHAT SHOULD CHANGE.** State the pool class a `Limiter` slot draws from, add its count to the B47 or
B48 arithmetic, and name it in the `PoolExhausted` kind set. A third class sized for a look-ahead is
the cheaper answer.

## CONCERN 4. Six of the twenty pre-authorised suppressions state no verifiable invariant

**OBSERVATION.** Appendix B.1 promises "Each reason names an invariant that a reader checks in the
function body." The six complexity reasons each take the form "a split would hide the order between
the rules", "a split would copy the running spacing state", "a split would duplicate the generation
check", and so on. Two conversion reasons are silent on the endpoint that decides them:
`unit_to_i24` and `unit_to_i32` both read "the `Unit` type carries the range -1.0 to 1.0, so the
scaled value fits with no clamp", and neither names the scale factor. One reason, `ticks_to_f64`,
says "at or above the stated magnitude" and states no magnitude.

**CLAIM.** These reasons defend a design choice rather than name a property a reviewer can check, and
`CLAUDE.md` asks for "a reason a reviewer can verify".

**ARGUMENT.** "A split would hide a missing transition" is a prediction about a refactor nobody
performed. A reader can agree or disagree with it and cannot check it. The two `unit_to_*` rows turn
on one unstated number: `1.0 * 2^23` does not fit `I24` and `1.0 * (2^23 - 1)` does, and the reason
says only "the scaled value". Three rows in the same table are good and show what the standard looks
like: `i24_to_f32` names the mantissa width, `i32_to_f32` names the round-trip proptest, and
`f64_to_f32` points at the guard clause above the cast.

Separately, the workspace policy comment at `Cargo.toml:6` reads "A crate may relax ONE lint at ONE
site". A single `as` cast trips `clippy::as_conversions` and at least one pedantic cast lint, so the
real attribute will name two or three lints. B.1 never addresses this, and thirteen of its twenty
rows name no lint at all.

**EVIDENCE.** `architecture.md:12556-12557` (the promise); `:12564-12570` (the seven conversions);
`:12579-12584` (the six complexity rows); `/Users/james/Developer/duet/Cargo.toml:6,105,109`.

**WHAT SHOULD CHANGE.** Rewrite the six complexity reasons to name a checkable property, such as the
transition count the match must cover. Put the scale factor in the two `unit_to_*` rows and the
magnitude in `ticks_to_f64`. Add a lint column to both tables and state how a multi-lint site is
spelled.

## CONCERN 5. Twelve guard blocks carry a floor below their row count

**OBSERVATION.** I measured every `GUARD BLOCK` marker against the rows under it. Twelve blocks carry
slack: `name-map` 4, `framework-types` 1, `drop-list` 1, `budget-table` 8, `probe-table` 14,
`rule-blocks` 9, `block-members` 8, `external-verdicts` 5, `external-paths` 11, `recorded-sizes` 1,
`vr1-table` 1, `audio-owned` 1. The rest are exact.

**CLAIM.** The "a deletion cannot pass as a clean run" property is partial across the register, and
the largest gaps sit on the probe table and the rule-block table, which are the two that describe the
guard set itself.

**ARGUMENT.** `probe-table` has 58 rows against a floor of 44, so fourteen probe rows can be deleted
with a green run. `rule-blocks` has 30 rows against a floor of 21. Those are the two tables a future
revision would edit to shrink the guard, and they carry the most slack. The floors were plainly
written by hand and left behind by additions, which is the same drift WARNING 10 measured.

**EVIDENCE.** My measurement of all 44 markers, taken from the frozen document; `architecture.md:1380`
(`probe-table rows>=44`), `:1642` (`rule-blocks rows>=21`).

**WHAT SHOULD CHANGE.** Derive each floor from the row count when the block is written, or raise every
floor to its current count and add a rule that a floor below the count is itself a failure. A
hand-written floor that lags is a guard that grants a free deletion per addition.

## CONCERN 6. Two prototypes in `tools/` do not run as a reader would run them

**OBSERVATION.** `python3 tools/conversion_check.py` from the frozen tree exits 2 with
`FAIL: cargo metadata --no-deps refused ...; the guard is fail-closed.` It exits 0 only from the
repository root. `python3 tools/probe_fragments.py` exits 0 and prints nothing, because it is a
library module with no command-line shape.

**CLAIM.** One of the two is correct and honest, and one is a run that a reader can mistake for a
green check.

**ARGUMENT.** `conversion_check.py` is right to fail closed, and its behaviour should be recorded
beside the command so a reader knows the working directory is the input. `probe_fragments.py` is the
problem: an exit code of 0 with no output is indistinguishable from a passing check, and a status line
that says "exit 0" for it asserts nothing. This is the zero-denominator shape that the document's own
CG1b rule exists to catch, at the level of the tool set rather than the scan.

**EVIDENCE.** The two runs above; `architecture.md:1430` (CG1b, the zero-denominator rule).

**WHAT SHOULD CHANGE.** Record the required working directory beside every tool command in section 1.9
and section 14. Give `probe_fragments.py` a main that refuses a direct run, or state in section 1.9
that it is a library and not a check.

## CONCERN 7. The frozen tree is not read-only, and a check run mutates it

**OBSERVATION.** Running the Python tools from the frozen directory rewrites
`tools/__pycache__/*.pyc`. Three files changed size during this review:
`placement_check.cpython-314.pyc` from 219360 to 219530 bytes, `probe_fragments.cpython-314.pyc` from
3410 to 3485, and `review_ids.cpython-314.pyc` from 9311 to 9386.

**CLAIM.** The frozen copy carried compiled bytecode that did not match its own sources, and any
verification run alters the tree it is verifying.

**ARGUMENT.** The md5 of `architecture.md` is unchanged and the review's integrity holds. The process
defect is real: a "frozen" artefact whose contents change when a reviewer checks it cannot be
compared byte for byte across reviewers, and stale bytecode in the tree is evidence that the tools
were last run against a different source.

**EVIDENCE.** The byte-count change above; `ls -la tools/__pycache__/` before and after.

**WHAT SHOULD CHANGE.** Do not include `__pycache__` in a frozen copy. Set `PYTHONDONTWRITEBYTECODE=1`
in the documented run command, or state the invocation from a read-only mount.

## CONCERN 8. `CoreHost::poll_frame` runs twice on an active frame

**OBSERVATION.** Section 10.2 step 1 states that `DuetApp::render` calls `CoreHost::poll_frame` as its
first statement, and then states that "`CoreHost::poll_frame` re-arms itself with
`Context::on_next_frame` ... whenever `frame_demand` reports any active driver". Facts 2 and 3 of the
same section prove that `DuetApp::render` runs on every animation frame.

**CLAIM.** On every frame with an active driver, both callers fire, so `poll_frame` runs twice.

**ARGUMENT.** The harm is small. `MeterReader::poll` takes the milliseconds since its last call, so
the B111 hold and the B112 fall are conserved across the two calls. The cost is one extra read of two
triple buffers per frame. The finding is that the design states two independent guarantees for one
call and never says they overlap, so a reader cannot tell whether the second call is intended or is a
defect.

**EVIDENCE.** `architecture.md:8365-8380` (the frame order, step 1 and the re-arm note);
`architecture.md:8280-8300` (facts 2 and 3).

**WHAT SHOULD CHANGE.** State the overlap in one sentence, and say which caller is the guarantee and
which is the backstop.

## Reactive Assessment

- **Responsive: PARTIAL.** The cross-boundary timeout table is close to complete and each row carries
  an action. B116 declares a cancel that its mechanism cannot perform (CRITICAL 4), and B18 names a
  mechanism and a thread that the design does not hold. Every other bound stands.
- **Resilient: PARTIAL.** Failure containment is strong: a vacant chain slot is a named fault, the
  generation skew costs one cycle, a refusal is arithmetic before allocation, and no path panics. The
  fault queue itself, which is the one route every failure takes to the user, is declared twice in
  two mechanisms with no producer end on the audio side (CRITICAL 5, CRITICAL 3).
- **Elastic: PARTIAL.** Every channel and ring is bounded with a stated overflow action, the B105
  ring is capacity one with a wait and a cap, and the re-send has one trigger and one cap. Two
  collections grow without a bound: the peak pyramid in memory (WARNING 4), and `pending_configure`
  holds a request that no event can drain in two reachable states (WARNING 1).
- **Msg Driven: PASS.** Every boundary is a message with a named primitive, an overflow rule and a
  bound. The audio thread computes nothing it can be told, `AudibleState` is published rather than
  derived, and the meter reset crosses as a per-strip generation rather than as shared mutable state.
  The one polling exception, `MidirPresence`, is documented at the site with its reason.

## Verdict

The engineering is good, and it is better than the guard record around it. Sections 5, 6, 7 and 12
hold a coherent real-time design: one owner per value, one writer per publication, no allocation and
no free and no lock on the audio thread, a migration that moves rather than copies, and a refusal
computed before a byte is allocated. The revision-20 fixes are mostly sound work: `ConfigureCommand::Close`
gives the stop a caller that can perform it, `TakeFlag::HadOverrun` separates two failures a singer
hears apart, and `MasterStage` types five boxes that three stories had asked for in prose.

Three things stop this from being ready. First, the guard record is not evidence: `probe_run.py` does
not run at all, so not one of the 58 probe rows is verified for this revision, and the PG25 transcript
is three revisions stale with two guards red on it. The Architect reported both as green. Second, the
audio-thread guard is rooted one level below the real root, so six heap-owning handles that the rule
was written to catch are invisible to it. Third, the C19-7 and C19-10 fixes are each incomplete at one
site: B116 promises a cancel that an `Instant` cannot deliver, and section 5.8 still routes the fault
queue to the thread that revision 20 removed from that path, in a primitive that contradicts section
12.4 and names a crate `duet-engine` does not depend on.

The single biggest risk is the first one. A plan of this size is only as good as the mechanism that
proves its own rules, and that mechanism has not executed against this document. The weakest Reactive
property is Resilient, because the fault path is the one path whose declaration is not settled, and a
failure that cannot be reported is a failure that is not contained.

**Counts: 5 Criticals, 10 Warnings, 8 Concerns.**

**Verdict: NOT READY.**
