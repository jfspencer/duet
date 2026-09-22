# External review of Duet v1 architecture, revision 22

## What I read, what I ran, and what I did not cover

`architecture.md` md5 at the start of my run: `b28e979bafda0a9b3f954a1eaec371a0`.
`architecture.md` md5 at the end of my run: `b28e979bafda0a9b3f954a1eaec371a0`.
**The two agree.** The frozen tree did not change while I worked.

I read `architecture.md` sections 1, 2.6a, 5, 6, 7, 9, 12, 13, 14 opening, 15.10, 15.13, 15.14, 15.16 extracts, Appendix A, Appendix B extracts, and Appendix C.23 to C.26 in full. A fork of me read the six ADRs, `product-requirements.md`, `design-contract.md` and `research/` in full and held each claim against `architecture.md`. I read every tool under `tools/` that I attacked.

**A second fork did not report before my deadline.** It held sections 2, 3, 4, 8, 10, 11, 14 and the appendices. My coverage of those eight areas is therefore partial, and I say so rather than imply a sweep I did not finish.

I ran the section 1.9 command verbatim on the frozen copy. One shared cargo target served the whole run. I deleted that target when the run ended. Every probe I planted went into a throwaway copy under my own scratch directory. I ran no git command and I wrote no file inside `/Users/james/Developer/duet`.

### The section 1.9 run, on the frozen copy

```
cd .../scratchpad/frozen-r22/duet-v1
PYTHONDONTWRITEBYTECODE=1 ROSTER_TARGET_DIR=.../critic-r22/target \
    python3 tools/run_all_gates.py architecture.md .../critic-r22/scratch /Users/james/Developer/duet
```

```
=== placement_check        exit 0  OK      2.2s
=== probe_run              exit 0  OK    451.1s
=== probe_closure          exit 1  RED     0.2s
=== conversion_check       exit 0  OK      0.1s
=== probe_conversion       exit 0  OK      2.5s
=== closure_check r16      exit 1  RED     0.1s
=== closure_check r17      exit 1  RED     0.1s
=== closure_check r18      exit 1  RED     0.1s
=== closure_check r19      exit 1  RED     0.1s
=== closure_check r20      exit 1  RED     0.1s
=== closure_check r21      exit 1  RED     0.1s
=== closure_check r21-inner exit 1  RED     0.1s
=== roster_compile         exit 0  OK    114.2s
=== probe_roster           exit 0  OK     48.9s
GATES RUN: 14     GATES BAD: 8     MODE: FULL
EXIT=1
```

The `roster_compile` transcript and the `probe_roster` transcript each reproduce the recorded fence of section 1.9 line for line. `probe_run` reports `PROBES BAD: 0`. `probe_conversion` reports `CONVERSION PROBES BAD: 0`. CRITICAL 5 below explains the eight red gates.

A separate agent gave me the Architect's run of the same command on the LIVE tree at the same md5. I read that transcript. I did not produce it. It prints `GATES RUN: 14     GATES BAD: 0     MODE: FULL` and `STORE: matches` at every closure gate. The two runs differ in one variable. That pair is the evidence for CRITICAL 5.

---

## CRITICAL 1. No declared value carries an `EngineEvent` from the engine to the core

**OBSERVATION.** `EngineEvent` is declared at `architecture.md:12474` with the doc line "One fact the engine reports on the structural channel". Section 5.8 decides every boundary. Its structural table at `architecture.md:6493` holds four rows: client to core over `CoreInput`, core to client over `CoreEvent`, core to client over `Snapshot`, and core to the engine handoff thread over `ConfigureCommand`. It holds no engine-to-core row. `CoreInput` is declared at `architecture.md:12051` with seven arms: `Call`, `FilesChanged`, `SetEntryContext`, `Resynchronize`, `Shutdown`, `WorkerStopped` and `QuitSaveDone`. No arm carries an `EngineEvent`. `DuetCore` at `architecture.md:12931` declares no receiver and no sender. `GraphConfigurator` at `architecture.md:5373` declares `backend`, `collector`, `commands`, `handoffs`, `topology`, `layout`, `pending_request`, `handoff_retry` and `attempts`. It declares no sender of any kind.

**CLAIM.** Six named messages have no declared path from the engine to the core. The design waits on five of them.

**ARGUMENT.** Section 6.6 at `architecture.md:7319` states "`CoreInput` carries one `EngineEvent::CaptureDone` per armed track". The `CoreInput` declaration refutes that sentence. The same hole blocks `EngineEvent::StreamClosed`, which B116 waits for at step 4 of section 9.5 rule 8. It blocks `EngineEvent::HandoffStopped`, which B119 waits for at step 8. It blocks `EngineEvent::GraphConfigured`, which `DuetCore::pending_configure` names as its one re-send trigger. It blocks `EngineEvent::ConfigureRefused`, which section 12.4 gives a whole message table. It blocks `EngineEvent::Fault`, which is step 3 of the fault path at `architecture.md:10042`: "`duet-core` receives it, records it on the engine state, and emits `CoreEvent::EngineFault`". No declared field receives one.

No rule can see this. PG20 reads a field type against the crate graph. PG13 reads the payload type of a publication. PG26 walks audio-owned declarations. None asks whether a stated message has a carrier.

**EVIDENCE.** `architecture.md:12474`, `:12051`, `:12931`, `:5373`, `:6493`, `:7319`, `:8632`, `:8640`, `:8670`, `:10042`.

**WHAT SHOULD CHANGE.** Add the engine-to-core row to the section 5.8 structural table. Declare the channel end on `GraphConfigurator` and the receive end on `DuetCore`, or add a `CoreInput::Engine(EngineEvent)` arm and declare the sender. Then re-audit closure rows C19-7, C19-10, C20-4, C20-5, C21-2, C21-4 and C21I-2, each of which names a mechanism this hole disables.

---

## CRITICAL 2. Six of the seven rows of the `snapshot-table` have no declared write end

**OBSERVATION.** The `snapshot-table` block at `architecture.md:6528` is a registered block with seven rows. Three rows name the audio thread as the source: `TransportSnapshot`, `MeterSnapshot` and `SlotMeterSnapshot`. Four rows name a writer off the audio thread: `Input<TempoMap>`, `Input<GraphChain>`, `Input<ParamSnapshot>` and `Input<PlaybackPlan>`. A grep of the whole document for `Input<` returns exactly one field declaration: `topology: Input<GraphChain>` at `architecture.md:5397`.

**CLAIM.** Six publications have no owner for their write end. The disk thread has no declared type at all, so it also has no read end for `PlaybackPlan`.

**ARGUMENT.** `triple_buffer::triple_buffer()` returns one `Input` and one `Output`. Section 5.8 at `architecture.md:6549` states the rule: "Each publication has exactly one read end, and `duet-core` owns it". The read ends are declared. `MeterReader` holds `output: Output<MeterSnapshot>` and `slots: Output<SlotMeterSnapshot>`. `TransportReader` holds `output: Output<TransportSnapshot>`. `EngineProcess` holds three read ends. No declaration holds the matching write end for six of the seven pairs.

TH9 at `architecture.md:6449` binds "Every type behind a `triple_buffer::Input` whose writer is the audio thread". The field that rule governs does not exist. `EngineProcess` at `architecture.md:12712` claims "Every heap handle here sits inside a `basedrop` wrapper". The three `Input` values it omits are heap handles the audio thread would have to hold, and PG26 would refuse a bare one. The doc comment asserts completeness over a set that is incomplete.

The core side is the same shape. `DuetCore` holds `pending_configure` and section 5.5 states that the core makes "ONE `try_send`" of it. `DuetCore` declares no `SyncSender<ConfigureCommand>` for the B109 channel. It declares no end of the B28 `CoreInput` channel, the B29 `CoreEvent` channel or the B30 `Snapshot` channel.

This is the C20-3 class one revision after the plan closed it. C20-3 read "the audio thread's real root is undeclared, so six heap-owning handles sit outside every audio guard".

**EVIDENCE.** `architecture.md:6528` to `:6537`, `:5397`, `:6449`, `:6549`, `:12724` to `:12746`, `:12931` to `:12965`, `:12980`, `:13033`.

**WHAT SHOULD CHANGE.** Declare each write end on the type that owns it. Put the three audio-written `Input` values on `EngineProcess`, each inside a `basedrop::Owned`. Put `Input<TempoMap>`, `Input<ParamSnapshot>`, `Input<PlaybackPlan>` and the B109 sender on `DuetCore`. Declare the disk thread's own type and give it the `Output<PlaybackPlan>` and the B33 consumer.

---

## CRITICAL 3. No declared command ends the engine handoff thread, so B119 waits for a message with no producer

**OBSERVATION.** `ConfigureCommand` at `architecture.md:5314` holds four arms: `Apply`, `Open`, `Close` and `Reset`. None of the four ends the thread's loop. Step 8 of section 9.5 rule 8 at `architecture.md:8670` reads "Wait B119 for `EngineEvent::HandoffStopped`. The engine handoff thread runs `Collector::collect` and `Collector::try_cleanup`, sends that event, and then ends." The Appendix B.5 row for B119 at `architecture.md:14061` reads "started when the core asks the engine handoff thread to stop".

**CLAIM.** No declared message performs that ask. The wait cannot end in the normal case, only at expiry.

**ARGUMENT.** The five shutdown steps of section 5.5 at `architecture.md:5581` name `ConfigureCommand::Close`, a drop of `GraphState`, a drop of the `basedrop::Handle` clones, a drain, and `try_cleanup`. Not one of them ends the loop. Section 5.5 at `architecture.md:5383` says the thread drops the backend "when its loop ends" and never says what ends it.

The plan shows that it knows the correct shape. Step 5 at `architecture.md:8642` states it for the job workers: "`JobRunner::shut_down` cancels every queued job and drops the queue sender, so each worker sees a closed channel and ends its loop." The engine handoff thread got no equivalent sentence and no equivalent field.

The cost is not theoretical. B119 is a report and not a cancel, so on expiry the core exits with the thread abandoned and `Collector::try_cleanup` unrun. With no producer for `HandoffStopped`, every quit takes the expiry path. The quit then pays B119 of five seconds on every exit.

**EVIDENCE.** `architecture.md:5314`, `:5383`, `:5581`, `:8670`, `:12497`, `:14061`.

**WHAT SHOULD CHANGE.** Add a `ConfigureCommand` arm that ends the loop, or state that the core drops the B109 sender and declare that sender. Correct the B.5 row to name the mechanism. Re-audit the C21-2 closure row.

---

## CRITICAL 4. `CaptureInfo::flags` asks the disk thread for three flags it cannot observe

**OBSERVATION.** `CaptureInfo` at `architecture.md:7315` declares `flags: TakeFlags`. Its doc comment states "`flags` carries the `TakeFlags` the pass observed, which section 6.4 states the disk thread reports". Section 6.4 at `architecture.md:7267` states "At the end of a pass it reports one `CaptureInfo` per armed track to `duet-core` on the structural channel". The observer table at `architecture.md:7274` names five flags and five observers. `Uncalibrated` is observed by "The backend, at arm time". `HadDrift` is observed by "The backend, per B70 window". `MidiPortLost` is observed by "`duet-midi`, on a hot-plug departure". `HadShortfall` and `HadOverrun` are observed by "The audio thread, per cycle".

**CLAIM.** The disk thread cannot know four of the five flags. No declared boundary carries one to it.

**ARGUMENT.** Section 5.8 names every boundary the disk thread has. Audio to disk carries `f32` samples and `RefillRequest`. Core to disk carries `PlaybackPlan`. `DiskWriter` at `architecture.md:12446` holds `track`, `take`, `channels`, `samples` and `written_frames`. No channel and no field carries a backend observation, an audio-thread observation, or a MIDI departure to the disk thread. So `CaptureInfo::flags` can hold at most the flag the disk thread raises itself through its own B118 expiry.

This is the finding the C21-4 row claims to close. That row at `architecture.md:15072` reads "6.6 `CaptureInfo`, which gains `track: TrackId` and `flags: TakeFlags`". The `track` half is sound. The `flags` half names a value the producing thread cannot build.

**EVIDENCE.** `architecture.md:7315`, `:7267`, `:7274` to `:7280`, `:12446`, `:6587` to `:6600`, `:15072`.

**WHAT SHOULD CHANGE.** State which party assembles `TakeFlags`. If the core assembles it, delete `flags` from `CaptureInfo` and state that the core merges the flags from the `EngineFault` records it already holds. If the disk thread assembles it, declare the boundaries that carry the four observations to it.

---

## CRITICAL 5. The section 1.9 command is red at eight gates on a frozen copy, and the CL1c site states the opposite

**OBSERVATION.** `run_all_gates.py` builds each closure gate with three arguments at `tools/run_all_gates.py:196` to `:206`. It never passes its own `<repo-root>`, although it holds `repo` and validates `os.path.join(repo, "Cargo.toml")` at line 161. `closure_check.py` then falls back to its module default at `tools/closure_check.py:100`: `REPO = os.path.dirname(os.path.dirname(PLAN))`. The CL1c site at `architecture.md:921` states the rule in bold: "**The repository root is an explicit ARGUMENT and never an environment variable.** A frozen copy of this plan sits outside the repository, so the guard cannot derive the path to `.claude/plan-coordination/db.sh` from its own location; `run_all_gates.py` passes its own `<repo-root>` argument through, and a guard that guessed would be a guard that guessed wrong."

**CLAIM.** The script does the thing the specification forbids in the same paragraph. The documented command cannot go green on a frozen copy, and a frozen copy is the one configuration every external review uses.

**ARGUMENT.** My run of the documented command exited 1. Eight gates are red. Each one prints the same line. This is the raw stdout:

```
=== closure_check r21      exit 1  RED     0.1s
REVIEW: critic-spec-r21.md   BLOCK: closure-r21   GENERATED: 45   ROWS: 45   STORE: unreachable   CLOSURE BAD: 1
  STORE:     key `36304kid4m4d3-review-r21`: the store launcher /private/tmp/.../scratchpad/.claude/plan-coordination/db.sh does not open; CL1c is fail-closed
```

The Architect's run on the live tree, at the same md5, prints `STORE: matches` and `GATES BAD: 0`. The two runs differ in the location of the plan directory and in nothing else. `probe_closure.py` fails for the same reason: it uses `closure_check.REPO` at `tools/probe_closure.py:72` and takes no repo argument, so its own BASE shape is red and it prints "FAIL: the baseline run is not green, so no shape is meaningful."

The consequence reaches further than one red run. C20-1 and C20-2 cost this plan one revision because a recorded result came from a tree the document no longer described. `run_all_gates.py` is the machine built to stop that class. A machine that only goes green inside the repository cannot verify a frozen document, and the section 1.9 baseline is therefore not reproducible by the command section 1.9 prints.

I confirmed the guard is fail-closed on a bad input, which is correct:

```
python3 tools/closure_check.py architecture.md reviews/critic-spec-r21.md closure-r21 <a directory with no db.sh>
REVIEW: critic-spec-r21.md   BLOCK: closure-r21   GENERATED: 45   ROWS: 45   STORE: unreachable   CLOSURE BAD: 1
exit=1
```

So the defect is the missing argument and not the rule.

**EVIDENCE.** `tools/run_all_gates.py:196` to `:206` and `:161`; `tools/closure_check.py:100`; `tools/probe_closure.py:72`; `architecture.md:921`; my transcript above; the live-tree transcript I read at `.../scratchpad/gates-r22-final.txt`.

**WHAT SHOULD CHANGE.** Pass `repo` as the fourth argument to `closure_check.py` from `run_all_gates.py`. Give `probe_closure.py` a repo argument and pass it. Add a probe that runs the whole harness from a copy outside the repository and asserts exit 0. Re-record the section 1.9 baseline from that run.

---

## CRITICAL 6. Section 13 assigns no chunk to four mechanisms this plan added, and chunk D3 states six enum arms against eight

**OBSERVATION.** I extracted section 13, lines 10281 to 11060, and counted occurrences. `SlotMeterSnapshot` 0. `SlotMeasure` 0. `slot_measure` 0. `PlaybackPlan` 0. `PlaybackSpan` 0. `FaultRun` 0. `quit-save` 0. `QuitSaveDone` 0. `WorkerStopped` 0. `HandoffStopped` 0. `CaptureInfo` 0. `B119` 0. `B131` 0. `B132` 0. `B134` 0. `B133` 0. `B136` 0. Chunk C3's goal at `architecture.md:10620` reads "the triple buffers with `TransportSnapshot` and `MeterSnapshot`". Chunk D3's goal at `architecture.md:10629` reads "`SlotState` over all six kinds". `SlotKind` at `architecture.md:4943` holds eight arms.

**CLAIM.** A fresh orchestrator cannot dispatch the quit path, the per-slot publication, the playback plan or the fault latch. Chunk D3 is told to build six of eight processor states.

**ARGUMENT.** SM2 at `architecture.md:10323` binds the plan: "The first chunk of a line creates every module file the whole line will ever need, at every depth, as a stub... Every later chunk in the line **modifies** a stub and creates no file." Chunk I1 at `architecture.md:10680` creates `src/{gateway,aggregate,undo,job_runner,channel,snapshot,job,midi_entry,reader}.rs` and `src/channel/{input,event,resync}.rs`. No file holds the nine-step quit path, the `duet-quit-save` thread of TH12, or the four bounds B119, B131, B132 and B134. No chunk goal names them. No Completion command exercises them. The same holds for the `PlaybackPlan` build on a job thread, which C21I-1 closed one revision ago.

C3 and I3 hold the write scopes that the per-slot publication would land in, so an engineer could place it. The goal column decides what the engineer builds, and both goals name two publications where the design now has three. The `SlotKind` case is sharper. `clippy::wildcard_enum_match_arm` is denied. A `SlotState` with six arms against an eight-arm `SlotKind` does not compile at the first match site.

**EVIDENCE.** `architecture.md:10620`, `:10629`, `:10680`, `:10323`, `:4943`, `:6457`, `:8655`; my grep counts over lines 10281 to 11060.

**WHAT SHOULD CHANGE.** Name the quit path, the `duet-quit-save` thread and the four bounds in one chunk goal and one write scope. Add the module file to chunk I1. Name `SlotMeterSnapshot` in C3's goal and `slot_measure` in I3's goal. Correct D3's goal to eight kinds. Add a Completion command that selects a test of the quit path.

---

## WARNING 1. B133 states half the true size and describes a shape the declaration does not hold

**OBSERVATION.** B133 at `architecture.md:1256` reads "`MAX_SLOT_METERS`, the length of both `SlotMeterSnapshot` arrays... At two `Finite` arrays it is 6144 bytes, which one `triple_buffer` write copies well inside B8". `SlotMeterSnapshot` at `architecture.md:6731` declares `measures: [SlotMeasure; MAX_SLOT_METERS]`, `live: u16` and `generation: Generation`. That is one array. `SlotMeasure` at `architecture.md:7709` declares `input: Finite` and `reduction: Finite`. `Finite` at `architecture.md:3079` wraps one `f64`. The `primitive-sizes` block at `architecture.md:2452` gives `f64` eight bytes.

**CLAIM.** The array is 12,288 bytes and not 6,144. The row describes two arrays and the type holds one.

**ARGUMENT.** `MAX_SLOT_METERS` is 48 times 2 times 8, or 768. `SlotMeasure` is 16 bytes. 768 times 16 is 12,288. The row states half that figure. The row also names a shape that no declaration holds, which is the class this review's brief names. No rule reads the cell. PG21 refuses a size literal in an `#[expect]` reason or a doc comment, and this is a budget table cell. PG24 computes sizes and compares them to the `recorded-sizes` block, which carries no row for this type.

**EVIDENCE.** `architecture.md:1256`, `:6731`, `:7709`, `:3079`, `:2452`, `:2188`.

**WHAT SHOULD CHANGE.** Correct the figure to 12,288 bytes. Rewrite the cell to describe one array of `SlotMeasure`. Add a `recorded-sizes` row so PG25 measures it.

---

## WARNING 2. Three sites state audio-root counts the run refutes, and the asserted list omits the root this revision added

**OBSERVATION.** The PG26e site at `architecture.md:806` reads "It covers the REACHABLE part of the root set, which is 26 of the 37 roots today. Eleven roots reach the audio thread as a function argument or as a published value". It then lists eleven names. `SlotMeterSnapshot` is not one of them. Line 812 reads "26 roots are derived, 11 are asserted twice". The PG26f site at `architecture.md:6362` reads "PG26e derives 35 of the 40 roots". Line 1742 reads "**26 of the 37 roots are derived and 11 are asserted twice**". My run prints `AUDIO OWNED: 41` and `ASSERTED ROOTS: 12`.

**CLAIM.** Four counts are wrong. The rule's own limit statement names the wrong root set.

**ARGUMENT.** The `audio-owned` block holds 41 rows and the `audio-asserted` block holds 12. So 29 roots are reachable and 12 are asserted. None of 26, 37, 35, 40 or eleven matches. DR3 exemption 4 covers a count the guard prints. These are hand numbers inside a rule's own limit statement, and a limit statement is the honesty mechanism of this document. A reader who asks which roots a coordinated four-line edit can still defeat gets a list of eleven that omits `SlotMeterSnapshot`, which revision 22 added. The same paragraph at line 1732 quotes `AUDIO REACHABLE: 36` where the recorded run in the same section prints 80, and line 1741 calls the leaf block "the ten-row leaf block" where the marker states 45.

**EVIDENCE.** `architecture.md:806` to `:812`, `:1732` to `:1743`, `:6362` to `:6374`, `:6182`, `:6345`; the baseline run at `architecture.md:1705` to `:1707`.

**WHAT SHOULD CHANGE.** Replace each count with the counter the run prints. Add `SlotMeterSnapshot` to the named list at the PG26e site.

---

## WARNING 3. PG40 has no denominator floor and refuses no unmapped chunk id

**OBSERVATION.** `chunk_crate_audit` at `tools/placement_check.py:4030` reads `owner = owners.get(chunk) or owners.get(head)` and then `if owner is None: continue`. The run prints `LINE CHUNKS` with no floor beside it.

**CLAIM.** A chunk row whose line prefix the `line-map` block does not carry is skipped in silence, whatever it writes.

**ARGUMENT.** I planted the recorded shape and one hostile shape. The recorded shape is red, which confirms the rule fires:

```
pg40-recorded    exit=1    CHUNK CRATE:A2: the row writes under `crates/duet-dsp/` and line `A` owns `duet-engrave` (PG40)
                 LINE CHUNKS:     50     CHUNK CRATE BAD: 1
```

The hostile shape adds one chunk row with an unmapped prefix that writes into another line's crate. The run is green:

```
pg40-unmapped    exit=0  (no finding line)
                 LINE CHUNKS:     50     CHUNK CRATE BAD: 0
```

I also renamed one existing chunk id. The denominator falls and PG40 stays silent:

```
pg40-drop exit=1
LINE CHUNKS:     49     CHUNK CRATE BAD: 0
  MEMBER:     phase-table: ... `A4` is no chunk of section 13.2
```

The exit 1 there comes from the `phase-table` membership rule, not from PG40. That coupling covers a rename by accident. It covers an addition not at all. N21-3 forced a floor on PG37 for this exact reason, and the run prints `VALUE FLOOR` beside `VALUE ROWS`. PG40 shipped with neither.

**EVIDENCE.** `tools/placement_check.py:4030` to `:4075`; `tools/placement_check.py:4502`; my three probe runs above.

**WHAT SHOULD CHANGE.** Make an unmapped chunk id a failure rather than a skip. Print a floor beside `LINE CHUNKS` and refuse a count below it.

---

## WARNING 4. CL1c ships a flag that removes the check and exits 0

**OBSERVATION.** `closure_check.py` accepts `--no-store` at line 316 and sets `store_state = "skipped"`. The exit code stays 0. The CL1b site at `architecture.md:890` condemns that shape in its own words: "The cut-off is the review's own stated revision and never a flag a runner passes, because a flag that removes a check is the defect C16-2 already cost this plan one revision."

**CLAIM.** The rule that exists to remove a fail-open ships a fail-open flag with no refusal and no denominator assertion.

**ARGUMENT.** My run of the real block with the real review, over the live repository, prints:

```
REVIEW: critic-spec-r21.md   BLOCK: closure-r21   GENERATED: 45   ROWS: 45   STORE: skipped   CLOSURE BAD: 0
exit=0
```

`run_all_gates.py` keys on the exit code. A run in which every closure block printed `STORE: skipped` is, by exit code, identical to a run in which every one printed `STORE: matches`. CG1b refuses a zero denominator with exit 2 for the same class of reason. CL1c has no such assertion. The `plan-lint` job of chunk M0 is the place this matters. That job runs `cargo xtask check-closure` "against the store the workflow restores". A workflow that cannot restore the store has one obvious repair, and that repair disables the rule.

**EVIDENCE.** `tools/closure_check.py:316` to `:321` and `:373` to `:392`; `architecture.md:890`, `:918`; my run above.

**WHAT SHOULD CHANGE.** Delete `--no-store`, or make a skipped store exit 2. State the count of blocks whose store matched, and refuse a run in which that count is below the block count.

---

## WARNING 5. CL1c lets the party it checks choose which stored copy it compares

**OBSERVATION.** The CL1c site at `architecture.md:903` states the mechanism: "The Critic stores every review with `.claude/plan-coordination/db.sh append`, which mints a `<simpleflake>-<suffix>` key. That keyspace is append-only, the store sits outside the repository, and the Critic is the writer, so the Architect cannot rewrite a stored review. **The closure section records that key beside the digest**". `closure_check.py` reads the key from the document window at line 372 and compares the copy at that key.

**CLAIM.** The rule binds the block to the key the document names, and the Architect owns the document. So the Architect selects the second source.

**ARGUMENT.** The store refuses a rewrite of an existing key. It does not refuse an append. `.claude/plan-coordination/README.md:65` states that `append` mints a new key and prints it, and every agent may call it. A coordinated edit therefore needs no rewrite. It appends the doctored review under a new key and edits one sentence of the closure section. Both acts sit inside the Architect's own scope. Probe PP32 covers a key that holds a DIFFERENT review, which differs and is red. It does not cover a key that holds the doctored review, which matches.

The rule states a narrower limit than the one that binds. It says the seven historical keys are a checksum and not a second party, because the Architect appended them. It does not say that key SELECTION is the Architect's on every future block as well.

All seven registered closure blocks are pre-revision-22. So CL1c supplies a second party for zero of the seven blocks it guards today. The C21-W1 row at `architecture.md:15076` records CLOSED. By the rule's own limit paragraph the row is PARTIAL.

**EVIDENCE.** `architecture.md:903` to `:919`, `:15076`; `tools/closure_check.py:366` to `:392`; `/Users/james/Developer/duet/.claude/plan-coordination/README.md:65`.

**WHAT SHOULD CHANGE.** Bind the key to a value the Critic prints and the Architect cannot choose. Record the key in the review file itself, which the digest already covers. Restate the C21-W1 row as PARTIAL until a block exists whose key the Critic printed.

---

## WARNING 6. A symbolic link defeats the `ROSTER_TARGET_DIR` path test

**OBSERVATION.** Both roster scripts test the path with a shell `case` against the repository string. `tools/roster_compile.sh:62` and `tools/probe_roster.sh:51` each hold the same three arms.

**CLAIM.** The test is textual. An absolute path outside the repository that resolves inside it passes.

**ARGUMENT.** I ran all three shapes against a throwaway repository. The two recorded shapes refuse:

```
E1 exit=2
FAIL: ROSTER_TARGET_DIR must be an absolute path; the guard is fail-closed.
E2 exit=2
FAIL: ROSTER_TARGET_DIR must sit outside <fakerepo>; the guard is fail-closed.
```

The third shape is a symbolic link outside the repository that points at `<fakerepo>/target`. The guard prints no refusal about `ROSTER_TARGET_DIR` at all, and the script goes on to its next step:

```
E3 exit=2
FAIL: the repository manifest holds no `[workspace.lints]` table.
(count of ROSTER_TARGET_DIR mentions in the output: 0)
```

This plan already owns the lesson. CG7 at `architecture.md:1673` records that a symbolic link defeats a path exemption in the conversion guard, and the conversion guard resolves the path. The roster scripts did not take that lesson, and the value reaches an `rm -rf` in one of them.

**EVIDENCE.** `tools/roster_compile.sh:62` to `:74`; `tools/probe_roster.sh:51` to `:66`; `architecture.md:1673`; my three runs above.

**WHAT SHOULD CHANGE.** Resolve both paths before the compare. Add a probe for the symbolic-link shape.

---

## WARNING 7. Appendix A restates three facts the document has changed

**OBSERVATION.** Appendix A states its own purpose at `architecture.md:13786`: "Every row names one state, its one owner, the one path that mutates it, who observes it, and the source of its identity." Three rows carry a superseded fact. Line 13797 reads "`Entity<RecordView>` in Record; `Entity<MixView>` in Mix and Master". Line 13816 reads "`EngineEvent::Fault` from the engine disk thread". Line 13807 names "`TransportSnapshot` and `MeterSnapshot`" as the audio publications.

**CLAIM.** The one appendix a reader opens for an ownership question gives three wrong answers.

**ARGUMENT.** Section 10.5 at `architecture.md:9636` gives `MasterView` its own cache field. ADR 0006 decision 6 names the old sentence as the defect C21-W6 closed. B61 at `architecture.md:1186` states three caches and names all three owners. Appendix A still gives Master no cache of its own. Section 12.4 step 2 at `architecture.md:10029` puts the fault drain on the engine handoff thread, which is the whole of the C19-10 fix, and Appendix A still names the disk thread. Section 5.8 now carries three audio-written publications and the run prints `SNAPSHOTS: 3`.

No rule reads Appendix A content. PG30 reads it only as a citation label. PG38 reads its cell counts.

**EVIDENCE.** `architecture.md:13786`, `:13797`, `:13807`, `:13816`, `:9636`, `:10029`, `:1186`, `:6533`, `:1710`.

**WHAT SHOULD CHANGE.** Correct all three rows. Add Appendix A to the C21-W6 and C19-10 closure rows.

---

## WARNING 8. Two sites state two audio publications where three exist

**OBSERVATION.** Section 5.7 at `architecture.md:6472` reads "On the two audio-to-user-interface paths the audio thread is the writer... Section 5.9 declares both published types". ADR 0004 decision 12b at `adr/0004-backend-and-threading-contract.md:173` carries the same sentence. Section 5.8 holds three such rows and the run prints `SNAPSHOTS: 3`.

**CLAIM.** The rule that TH9 states covers two of three publications by its own words.

**ARGUMENT.** Revision 22 added `SlotMeterSnapshot` for C21-7. PG13 reads the table, so the guard covers all three. The prose did not move. A reader who applies TH9 as written leaves the third publication outside the rule that forbids a heap field on an audio-written value.

**EVIDENCE.** `architecture.md:6472` to `:6476`, `:6533`, `:1710`; `adr/0004-backend-and-threading-contract.md:173`.

**WHAT SHOULD CHANGE.** Delete the count from both sites and cite section 5.9.

---

## WARNING 9. Two ADR clauses state a shutdown contract revision 22 replaced

**OBSERVATION.** ADR 0005 decision 13a at `adr/0005-agent-gateway.md:46` reads "**Shutdown runs in four bounded steps, and the main thread never blocks.** ... B23 the wait for in-flight replies, and B24 the runtime shutdown." Section 9.5 holds nine steps and seven bounds. ADR 0004 decision 17c at `adr/0004-backend-and-threading-contract.md:220` reads "Every other file operation in the product runs on a `JobRunner` thread in `duet-core`". TH12 at `architecture.md:6457` declares a third file-writer thread.

**CLAIM.** The one decision record for the quit path names two of its seven bounds. A second record forbids the thread revision 22 added.

**ARGUMENT.** C19-W23 changed the shutdown to cover the whole process. C21-3 then added B134 and TH12. Neither record moved. An implementer who takes the contract from ADR 0005 loses B131, B116, B132, B134 and B119. An implementer who applies ADR 0004 clause 17c refuses the `duet-quit-save` thread and puts the `fsync` back on the thread that paints, which is the C21-3 defect.

**EVIDENCE.** `adr/0005-agent-gateway.md:46`; `adr/0004-backend-and-threading-contract.md:220`; `architecture.md:8625`, `:8691`, `:6457`, `:6405`.

**WHAT SHOULD CHANGE.** Cut both clauses to the decision plus a citation, which DR6 requires.

---

## WARNING 10. B90 credits PR M-03 with a processor the story does not name

**OBSERVATION.** B90 at `architecture.md:1215` reads "The 32 track strips of B1 each carry a high-pass, an equalizer, a compressor, a de-esser, and a gate, which PR M-03 names one by one". PR M-03 at `product-requirements.md:512` reads "it offers gain, high-pass, de-esser, compressor, and equalizer". No line of `product-requirements.md` names a gate.

**CLAIM.** A default slot with no product source costs 96 parameters of the B90 budget.

**ARGUMENT.** C21-W8 closed the same class at B125 and B126, where a budget row cited a story that carries no such threshold. M-03 names five tools, and the `SlotKind` doc at `architecture.md:4935` correctly sends gain to `ChainState::trim`. That leaves four slot tools. Design contract 4.2 at `design-contract.md:719` draws the mixer strip with four insert rows. A fifth default slot does not fit the strip the contract draws.

**EVIDENCE.** `architecture.md:1215`, `:4935`; `product-requirements.md:512`, `:513`; `design-contract.md:719`.

**WHAT SHOULD CHANGE.** Name the source of the gate, or remove it from the default set and recompute the sum. State how five default slots fit four insert rows.

---

## WARNING 11. `Take` carries no date, and a MUST story asks for one

**OBSERVATION.** PR R-09 at `product-requirements.md:432` reads "when the user opens the list, then each row shows a number, a length, and a date." Design contract 3.4 at `design-contract.md:583` reads "A take without a name shows `Take 3 - 14:22`." `Take` at `architecture.md:7111` declares `id`, `name`, `regions`, `muted` and `flags`.

**CLAIM.** No declared field holds a take time. The length is derivable and the date is not.

**ARGUMENT.** `Source` at `architecture.md:7058` holds no timestamp either. The type this plan needs exists: `UnixSeconds` sits in `duet-time`, and `Calibration`, `RecentEntry` and `CommitSummary` each carry one. This is the C21-W11 and C21-W15 class, which the revision-21 sweep closed for `Take::muted` and `Strip::name` and did not close here.

**EVIDENCE.** `product-requirements.md:432`; `design-contract.md:583`; `architecture.md:7111`, `:7058`, `:2573`.

**WHAT SHOULD CHANGE.** Add `recorded_at: UnixSeconds` to `Take`, or name the declared value the take list reads.

---

## WARNING 12. Three contract shapes have no declared owner and no budget row

**OBSERVATION.** Design contract 5.3 at `design-contract.md:904` gives the LUFS bar a scale from -36 LUFS to 0 LUFS, and line 912 gives it a tolerance band of plus or minus 1 LU. Line 930 gives the true-peak bar a scale from -12 dBTP to +3 dBTP. Design contract 4.2 at `design-contract.md:739` gives the fader a taper with unity gain at 72 percent of the travel and minus infinity below 8 percent. `LufsMeter` at `architecture.md:13703` declares `report` and `target`. `Fader` at `architecture.md:13680` declares `parameter`, `value` and `strip`.

**CLAIM.** B106 claims that no view holds a scale of its own. Three painted shapes have no declared map and no budget row.

**ARGUMENT.** `db_to_fraction` maps -60 dB to +6 dB. It cannot map a -36 LUFS bar or a -12 dBTP bar. `ParamRange` gives bounds alone, so it cannot express a unity point at 72 percent of the travel. B76 is a test tolerance and not the painted band, which its own cell states. A chunk K4 or K5 implementer must invent each shape or write a literal, and DR3 forbids the literal. This is the WR-13 class that the level meter closed and the loudness panel and the fader did not.

**EVIDENCE.** `design-contract.md:739`, `:904`, `:912`, `:930`; `architecture.md:1231`, `:1201`, `:7455`, `:13680`, `:13703`.

**WHAT SHOULD CHANGE.** Add budget rows for each scale, the tolerance band, the unity-gain fraction and the minus-infinity fraction. Declare one `duet-dsp` function per map beside `db_to_fraction`.

---

## CONCERN 1. The CL rule family sits in no index, and PG39 declines it in silence

**OBSERVATION.** DR4 at `architecture.md:54` makes section 1.7 the one index of rule ids. PG32 declares CL0 to CL5 as rules in their own right. Section 1.7 at `architecture.md:1424` holds no CL row. `INDEX_FAMILIES` at `tools/placement_check.py:3980` is `("PG", "PP", "CG", "CP")`. The PG39 limit sentence names DR, PL, VR, TH and SM as the families it declines, and it does not name CL.

**ARGUMENT.** I added a row that names ten invented ids, `CL0 to CL9`. The run is green and the denominator rises:

```
pg39-cl        exit=0  (no finding line)
               INDEX IDS: 171     INDEX BAD: 0
```

So section 1.7 can carry a whole invented family, and the real CL family is indexed nowhere. The four recorded shapes each behave as the table states, so the rule is sound over the four families it reads.

**WHAT SHOULD CHANGE.** Add a CL row to section 1.7 and a CL family to PG39, or name CL in the PG39 limit sentence.

---

## CONCERN 2. `Strip` holds three `Vec` lists where the runtime holds an `ArrayVec`

**OBSERVATION.** `Strip` at `architecture.md:7366` declares `pre_fader: Vec<SlotId>`, `post_fader: Vec<SlotId>` and `sends: Vec<SendId>`. `ChainTopology` declares each as an `ArrayVec` at `MAX_SLOTS` and `MAX_SENDS`.

**ARGUMENT.** C21-6 is genuinely closed at the refusal level: `MixError::SlotBudgetExceeded` and `MixError::SendBudgetExceeded` are declared at `architecture.md:11859` and `:11862`, and the `MixState::validate` contract names both. The type still admits the illegal state, so the invariant rests on every caller. The section 1.2 row for `duet-session` carries no `arrayvec` entry, which is the likely reason, and the document does not state that trade-off.

**WHAT SHOULD CHANGE.** State why the type is not the bound, or add the `arrayvec` entry and make each list an `ArrayVec`.

---

## CONCERN 3. The `# Errors` contract of `configure` omits the arm that carries the new refusals

**OBSERVATION.** The `# Errors` section at `architecture.md:5833` names `ChannelMismatch`, `PoolExhausted`, `RingBudgetExceeded`, `StripBudgetExceeded`, `ParamBudgetExceeded` and `HandoffRetryPending`. Section 5.5 outcome three at `architecture.md:5503` names "`PoolExhausted`, `RingBudgetExceeded`, `StripBudgetExceeded`, `ParamBudgetExceeded`, and `Mix`".

**ARGUMENT.** `ConfigError::Mix(MixError)` is the one arm that can carry `SlotBudgetExceeded`, `SendBudgetExceeded` and `CurveBudgetExceeded` out of `configure`. The contract a caller matches on omits it. C16-3 records the reverse direction of this defect.

**WHAT SHOULD CHANGE.** Add `Mix` to the `# Errors` list.

---

## CONCERN 4. The per-slot publication adds about 24 KB that no budget row names

**OBSERVATION.** `SlotMeterSnapshot` holds 12,288 bytes of array. `MeterReader` at `architecture.md:12991` holds a second `[SlotMeasure; MAX_SLOT_METERS]` in `latest_slots`. B55 and B56 bound engine memory and neither names either array.

**ARGUMENT.** `triple_buffer` holds three copies of the published value, so the engine side is about 36 KB and the foreground side is about 12 KB. The total is small against B55 at 104 MB. The plan's own rule is that every number has a row.

**WHAT SHOULD CHANGE.** Name the publication cost in B55, or add a row.

---

## CONCERN 5. The correction-note rule covers `research/` alone

**OBSERVATION.** The rule at `architecture.md:17` binds `research/` and names no other source of record. `product-requirements.md` and `design-contract.md` are both sources of record in the same list.

**ARGUMENT.** Design contract 5.3 at `design-contract.md:911` offers four loudness targets. PR Q7 at `product-requirements.md:994` and B75 each give two presets plus Custom. A chunk K5 implementer who reads contract 5.3 builds a four-item control. A superseded contract line has no home for a note, so the second answer stands.

**WHAT SHOULD CHANGE.** Extend the dated correction-note rule to both files. Add a note at contract 5.3, or add the third preset under B75.

---

## CONCERN 6. PR R-14 asks for the time of each drop, and no declared field carries one

**OBSERVATION.** PR R-14 at `product-requirements.md:477` reads "when the user opens it, then the application names the time of each drop." `TakeFlags` is a bitset. `CaptureInfo::underruns` is a count. `EngineFault::CaptureOverflow` carries a whole-run frame total.

**ARGUMENT.** R-14 is a SHOULD, so the gap is debt. A frame total is not a list of times, and the K6 status-bar line is one line and not a per-take list.

**WHAT SHOULD CHANGE.** State in the R-14 row which criteria the plan meets and which it defers. File the deferral in a plan document.

---

## CONCERN 7. Four ADR decision clauses carry a bare count and one is stale

**OBSERVATION.** DR6 at `architecture.md:73` requires every number inside an ADR to be a citation. `adr/0001-time-kernel-representation.md:66` says "twelve rules and twelve tests". `adr/0003-history-model.md:31` says "Four sources define the live set". `adr/0004-backend-and-threading-contract.md:196` says "the five steps". `adr/0004-backend-and-threading-contract.md:282` says "A fifth engine thread is a cost this record accepts".

**ARGUMENT.** N21-12 and N21-14 closed this class twice inside ADR 0004 alone. Three counts hold today. The fourth is stale: the section 5.7 table gives the engine and the backend four threads before the quit-save thread, and five after it. No rule reads an ADR count.

**WHAT SHOULD CHANGE.** Replace each count with a citation. Record the gap as accepted debt if no rule is worth writing.

---

## CONCERN 8. Chunk C3 carries the largest goal of the plan and one Completion command

**OBSERVATION.** Chunk C3 at `architecture.md:10620` names `ChainTopology`, `ChainState`, `PoolHandle`, `ChainSlot`, `ChainSource`, `ChainLayout`, `GraphConfigurator`, the engine handoff thread, its collector, its B95 retry, the per-position migration, the drain rule, the generation proof, the mixer runtime, the graph runner, two triple buffers, and the soak test. Its Completion command selects two tests.

**ARGUMENT.** SM3 asks one command to fail before a chunk and pass after it. Two tests over seventeen named mechanisms is a weak oracle for the single largest chunk of the plan. Every Critical of the last three reviews landed inside this chunk's scope.

**WHAT SHOULD CHANGE.** Split C3, or add one selected test per named mechanism to its Completion command.

---

**Counts: 8 Criticals, 18 Warnings, 9 Concerns.**

## Reactive Assessment

- **Responsive: FAIL.** Every bound of the quit path waits on a message that no declared channel carries, and step 8 waits on a message that no declared command produces. A quit therefore reaches expiry on every path (CRITICAL 1, CRITICAL 3).
- **Resilient: FAIL.** The fault path from the audio thread to the user interface is complete up to the engine handoff thread and stops there. That thread holds no fault-queue handle and no event sender, so no fault reaches a user (CRITICAL 1, CRITICAL 2).
- **Elastic: PARTIAL.** Every queue, ring, array and channel carries a bound, a budget row and a refusal, and `Curve` now carries one as well. The ends of six publications and four channels are undeclared, so no owner exists for the backpressure the rows state (CRITICAL 2).
- **Message Driven: PARTIAL.** The design is message driven by intent at every seam. Section 5.8 decides one primitive per boundary and states the producer count first. The declarations hold one write end out of seven and no structural channel end at all, so the foundation is stated and not declared (CRITICAL 1, CRITICAL 2).

## Verdict: NOT READY FOR PLAN AUTHORING

The engineering judgement of this document is high and the closure discipline is real. C21-1, C21-5, C21-6, C21-W14, C21-W17 and C21-W19 are closed at the declaration level, and I verified each one. The guard set answered every hostile shape I planted at PG38, PG39 and the four recorded PG40 and CL1c shapes.

The single biggest risk is unchanged from the twenty-first review, and it has moved one level deeper. That review found closure rows that named a FIELD the declaration did not hold. This revision holds every field those rows named. It does not hold the CHANNEL, the SENDER or the RECEIVER that carries any of those fields between two threads. Section 15 is called the complete type roster, and it declares one write end out of seven publications and no end of any structural channel. Every one of the last four revisions closed a Critical of the form "a duty with no caller that can perform it". Revision 22 closes four more and creates six.

The weakest Reactive property is Resilient. A fault is measured, latched, bounded, named and given a user message, and no declared value carries it out of the engine.

## The blocking list, in the order I would fix it

1. **CRITICAL 1.** Declare the engine-to-core boundary. Nothing else in section 9.5 or section 12.4 can be true until it exists.
2. **CRITICAL 2.** Declare the six missing write ends and the four structural channel ends. Declare the disk thread's own type.
3. **CRITICAL 3.** Give the engine handoff thread a stated end, then re-audit the C21-2 row.
4. **CRITICAL 4.** Decide who assembles `TakeFlags`, then correct `CaptureInfo` or section 6.4.
5. **CRITICAL 5.** Pass `<repo-root>` from `run_all_gates.py`, add a frozen-copy probe, and re-record the section 1.9 baseline from that run. Do this before the next freeze, or the next reviewer measures the same false red.
6. **CRITICAL 6.** Assign the four orphaned mechanisms to chunks and correct chunk D3.
7. **WARNING 1, 2, 7, 8, 9.** Correct every number and every restated fact that the artifact refutes.
8. **WARNING 3, 4, 5, 6.** Close the four guard holes.
9. **WARNING 10 to 12.** Close the three product and contract gaps.
10. **The Concerns.** Fix in the changeset, or file each one in a tracked document under `roadmap/`.

## Addendum: the finished sweep of sections 2, 3, 4, 8, 10, 11, 14 and the appendices

My first hand-back stated that the sweep of those eight areas had not finished and that my counts were a floor. The sweep has now finished. I verified every finding below myself against both sides. `architecture.md` md5 is still `b28e979bafda0a9b3f954a1eaec371a0`. **The count is a total and no longer a floor.** Every earlier finding stands.

---

## CRITICAL 7. The audio thread silences held notes with no field and no message

**OBSERVATION.** Section 8.2 step 1 at `architecture.md:8204` reads "**The engine silences the held notes within one cycle.** The audio thread keeps the set of sounding notes per `PortSlot`... the set is a fixed-size bitset over the 128 note numbers."

**CLAIM.** No declaration holds that set, and no declared message tells the audio thread that a port left. The step cannot run.

**ARGUMENT.** Two halves fail. First the set: `EngineProcess` at `:12724` declares nine fields and none is a note set; `GraphState` at `:5995` declares six and none is a note set; `ChainState` is per strip. A grep for "sounding" and "bitset" returns the two prose sites at `:8205` and `:8208`, the VR1 row at `:3746`, and four unrelated types. Second the message: the MIDI thread reaches the audio thread over one boundary, the B32 `rtrb` ring of `MidiRecord`. `MidiMessage` at `:8246` holds `NoteOn`, `NoteOff`, `ControlChange`, `PitchBend` and `Sustain`. No arm carries a departure. A departure travels to the CORE over B100, and no declared path runs core to audio. This is the class of my CRITICAL 1 and CRITICAL 4. The cost is a stuck note the user cannot release.

**EVIDENCE.** `architecture.md:8204`, `:8205`, `:8208`, `:12724`, `:5995`, `:8246`, `:3746`.

**WHAT SHOULD CHANGE.** Declare the set as a field of `EngineProcess`. Declare the departure message. A `MidiMessage::PortGone { port: PortSlot }` arm on the B32 ring is cheapest, because the ring already orders by `SampleClock`. Name the thread that mints it.

---

## CRITICAL 8. `StageCurve::measure` has no declared path from the reader to the view

**OBSERVATION.** `MeterReader::slot_measure` at `:13024` answers one cell, and its doc at `:13020` calls it "the one producer of `StageCurve::measure`". `StageCurve` at `:13700` declares `cell: u16` and `measure: Option<SlotMeasure>`. `CoreHost` holds `meters: MeterReader` privately and declares four methods: `poll_frame`, `frame_demand`, `meters`, `transport`.

**CLAIM.** `CoreHost` exposes no accessor for a `SlotMeasure`, so no view can fill `StageCurve::measure`.

**ARGUMENT.** I read the four signatures at `:13189` to `:13203`. `meters` returns a `MeterView` and `transport` returns a `TransportView`. Neither carries a slot cell. The view crate cannot hold the reader: section 1.3 rule 6 forbids a publication read end in `crates/duet`. `MixView`, `MasterView` and `Inspector` build `StageCurve` and none declares a field for the value. The meter path shows the missing pattern: `MeterLayer` "reads the meters through `CoreHost::meters`". Taken with my CRITICAL 2, the C21-7 chain is broken at both ends.

**EVIDENCE.** `architecture.md:13024`, `:13020`, `:13700`, `:13189` to `:13203`.

**WHAT SHOULD CHANGE.** Add `pub(crate) fn slot_measure(&self, cell: u16) -> Option<SlotMeasure>` to `CoreHost`. Name it in the `StageCurve` doc, in section 7.3 beside the cell formula, and in the C21-7 row.

---

## WARNING 13. Section 2.11 sends the tempo map to a thread no row and no field serve

**OBSERVATION.** `:3267` reads "It reaches the audio thread and the engine disk thread through `triple_buffer`". The section 5.8 table holds one tempo row, Core to Audio, at `:6534`. One `Output<TempoMap>` exists, at `:12732` inside `EngineProcess`.

**ARGUMENT.** The disk thread has no read end and no row. Section 2.11 cites section 5.8 for a rule it does not hold, which DR4 forbids.

**WHAT SHOULD CHANGE.** Delete "and the engine disk thread": the disk thread reads `PlaybackSpan`, which carries `SampleClock`.

---

## WARNING 14. The B134 derivation is wrong in both halves

**OBSERVATION.** B134 at `:1255` reads "**The value is eight B19 halves and not a guess**: section 4.10 names eight `fsync` calls plus one per temporary file and one per touched directory".

**ARGUMENT.** I read the five steps at `:4510` to `:4515`. The fixed count is three, not eight: step 3 syncs the marker and `state/`, step 5 syncs `state/`. The variable groups are three, not two, and a directory is synced twice, at step 2 and at step 4. Separately B19 is 5 s at `:1144`, so eight halves is 20 s and the row states 5 s. Both halves are false, on the bound that closes C21-3.

**WHAT SHOULD CHANGE.** Rewrite the derivation, and change the value if the arithmetic gives a larger one.

---

## WARNING 15. The quit-path save names two incompatible recovery mechanisms

**OBSERVATION.** B134 and section 9.5 step 6 both say the save "writes through `state/.tmp/` and renames" and "the next start empties `state/.tmp/`", and both cite section 4.10. Section 4.10 at `:4502` writes each temporary file "in the same directory" and records the set in `state/save.commit`.

**ARGUMENT.** The crash matrix at `:4523` reads "The start reads the marker and completes every rename whose temporary file still exists." A start that empties `state/.tmp/` would delete the files recovery needs. `state/.tmp/<job>/` is the section 9.6 job convention.

**WHAT SHOULD CHANGE.** Pick one protocol and state it.

---

## WARNING 16. Two sites name a `FrameDemand` field the declaration does not hold

**OBSERVATION.** `:8892` reads "`any_armed` reads the armed set that `DuetCore` holds" and `:13616` reads "**The START condition is `FrameDemand::any_armed` or `FrameDemand::transport_moves`**". I read `:13114`: `FrameDemand` holds `transport_moves`, `input_live` and `meter_decays`.

**ARGUMENT.** The `input_live` doc states the C15-10 merge. The rename landed in the declaration and in the closure row and not in these two sites. No rule reads a field name in a doc comment.

**WHAT SHOULD CHANGE.** Replace `any_armed` with `input_live` at both sites.

---

## WARNING 17. The `MidiNote` derive reason contradicts the VR1 table that governs it

**OBSERVATION.** `:8231` and `:8205` each read that `MidiNote` derives `Ord` and `Hash` "because the engine holds a set of sounding notes per port". The VR1 row at `:3746` reads "**The sounding-note set of section 8.2 is a fixed-size bitset and asks for neither**".

**ARGUMENT.** PG22 reads a row's existence and never its text, so the run is green. One fact has two answers, which DR1 and DR4 forbid.

**WHAT SHOULD CHANGE.** Delete the reason at both sites and cite the VR1 row.

---

## WARNING 18. `RecentList` is an unbounded collection that an agent verb grows

**OBSERVATION.** `:12874` declares `pub struct RecentList { entries: Vec<RecentEntry> }`. `Verb::RecentAdd` at `:8390` appends to it and `duet-project` persists it.

**ARGUMENT.** `:7566` states the rule this breaks: "**Every other collection of this design carries a bound, a budget row and a refusal**". A script that calls `RecentAdd` in a loop grows the vector and the file without limit.

**WHAT SHOULD CHANGE.** Add a budget row, then refuse past it or drop the oldest entry and say so in the row.

---

### WARNING 8, extended by the finished sweep

My WARNING 8 named section 5.7 at `:6472` and ADR 0004 decision 12b. Four more sites carry the same count of two: `:8984`, `:9775`, `:13140` and `:13821`. `MeterReader` declares two `Output` fields at `:12981` and `:12986` and `TransportReader` declares a third at `:13034`. Six sites state two where three exist.

---

## CONCERN 9. The public-field rule holds in one section and three declarations break it

**OBSERVATION.** `:4653` reads "No public field appears on any type here, per the coding guide rule for public API." `Applied` at `:3575`, `MusicXmlImport` at `:4015` and `SmfImport` at `:8350` each carry public fields.

**ARGUMENT.** No guard reads a field's visibility.

**WHAT SHOULD CHANGE.** State the rule once at the section 3.5 vocabulary rules and remove the fields, or state the exemption and why.

---

## Two classes are clean in that scope

Every cross-boundary wait of Appendix B.5 resolves to a section 5.12 row. Every view that takes a subscription or a task retains it in a declared field.

## The one mechanical gap I would close before any of the eight Criticals

All eight Criticals are one class: a stated duty whose carrier no declaration holds. The class spans the engine, the core, the MIDI path, the view seam and the plan graph, and it is invisible to all 63 rules. This plan has a rule for a type with no place, a place with no type, a derive with no use, a citation with no site, and a chunk with no crate. It has no rule for a message with no channel. A rule that reads every "sends", "receives", "publishes", "reports", "waits for" and "carries" claim against the declared field set would have caught seven of my eight Criticals on its first run.

## The revised blocking list

1. **CRITICAL 1, 2, 7, 8.** Declare every missing carrier.
2. **CRITICAL 3.** Give the engine handoff thread a stated end.
3. **CRITICAL 4.** Decide who assembles `TakeFlags`.
4. **CRITICAL 5.** Pass `<repo-root>` and re-record the section 1.9 baseline from a frozen-copy run.
5. **CRITICAL 6.** Assign the four orphaned mechanisms to chunks and correct chunk D3.
6. Add the message-carrier rule before the next freeze.
7. **WARNING 14 and 15**, which both sit on the path C21-3 closed.
8. The remaining Warnings, then the Concerns.

My coverage is now complete over every section the brief names. I deleted my cargo target and I wrote no file in the repository.
