# Engineering Critic, specification review, revision 19

Recovered by the Orchestrator on 2026-09-21 from the Critic's hand-back message; the Critic stored no key. The text below is the hand-back verbatim.

**Document under review.** The frozen copy at
`/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/frozen-r19/duet-v1/`.
`architecture.md` is 13188 lines, md5 `c3238aa447e29e4890b69d78897d8732`. The md5 is unchanged after every run.

Review digests, which match the three the document states:
`reviews/critic-spec-r16.md` `371adf1d9ebb71f560ffd7379b6a96be`;
`reviews/critic-spec-r17.md` `b75f780272154e9901cec876834817f0`;
`reviews/critic-spec-r18.md` `bef6dfcdd30c1ec89fe276e24ef19754`.

**I wrote no repository file. I ran no git command.** Every plant went into a throwaway copy under the session scratchpad. One shared cargo target served every run.

**Verdict: NOT READY.**

---

## What I ran

Every guard and every harness, against the frozen copy.

| Run | Result |
|---|---|
| `placement_check.py architecture.md` | exit 0. PROBE ROWS 56, PAIR BAD 0, TAIL BAD 0, SUPPRESSION BAD 0 |
| `closure_check.py` on closure-r16, r17, r18 | exit 0 each. GENERATED equals ROWS at 46, 26 and 20 |
| `review_ids.py` on r16, r17, r18 | 17/18/11, 7/10/9, 6/8/6. Every count matches the appendix |
| `probe_run.py` | exit 0. PROBES BAD 0, RECORDED FRAGMENTS 69, FLOOR 41 |
| `probe_conversion.py` | exit 0. 28 shapes red, RECORDED FRAGMENTS 28, **no floor printed** |
| `probe_closure.py` | exit 0. 8 shapes red, RECORDED FRAGMENTS 8, FLOOR 8 |
| `roster_compile.sh` plus `probe_roster_text.py` | exit 0. 17 crates, 395 items, 50 impl blocks, TRANSCRIPT BAD 0 |

I then planted 12 probes against the closure guard, 7 against the placement guard, and 5 against the conversion guard, plus 3 gate defects of my own. I report every result below.

---

## Part 1. Closure verdicts

### The revision-17 review, 26 findings

The document records all 26 as CLOSED. I verified each row at the section or the tool it names.

**24 are CLOSED.** C17-1, C17-2, C17-3, C17-4, C17-5, C17-6, C17-W1, C17-W2, C17-W3, C17-W4, C17-W6, C17-W7, C17-W8, C17-W9, C17-W10, N17-1, N17-2, N17-3, N17-4, N17-5, N17-6, N17-7, N17-8 and N17-9 each name a mechanism that exists and does what the row claims.

**Two are PARTIAL.**

| Id | My verdict | Section | Reason |
|---|---|---|---|
| C17-7 | PARTIAL | 1.9 line 1529, `probe_conversion.py` | The "Six planted gate defects" table is compared by no harness, and one harness prints no floor. |
| C17-W5 | PARTIAL | Appendix B.5, ADR 0004 | The closure names a `blake3` `features = ["pure"]` row that Appendix B.5 does not hold. |

### The revision-18 review, 20 findings

The document records 19 as CLOSED and N18-2 as OPEN.

**11 are CLOSED.** C18-1, C18-3, C18-4, C18-W2, C18-W3, C18-W8, N18-1, N18-3, N18-4, N18-5 and N18-6 hold at the tool or the section named. I ran each plant and each turns the run red.

**Seven are PARTIAL and one more is OPEN.**

| Id | My verdict | Section | Reason |
|---|---|---|---|
| C18-2 | PARTIAL | `probe_roster_text.py` | The transcript compare runs one direction only, so a deleted line stays green. |
| C18-5 | CLOSED | `probe_closure.py` | Verified. A planted false line prints `FRAGMENTS BAD: 1`. |
| C18-6 | PARTIAL | 1.5 PG32, 13.1 M0 | M0 must port PP32, and no chunk creates the guard that PP32 probes. |
| C18-W1 | PARTIAL | `closure_check.py` `audit` | A CLOSED row that names a tool and no section stays green. Five real rows are that shape. |
| C18-W4 | PARTIAL | `probe_roster_text.py` | The floor subtracts ten. It is 4 against 8 recorded lines. |
| C18-W5 | **OPEN** | `probe_roster_text.py` `holders` | The loop scans every `.out` file. The pool is renamed, not replaced. |
| C18-W6 | PARTIAL | `review_ids.py` `FENCE` | A tilde fence and an unclosed fence both still change the id list. |
| C18-W7 | PARTIAL | C.21 C17-7 row | The rewritten cell adds a new false claim about every harness. |
| N18-2 | OPEN, correct | 1.5 PG31b | I recomputed both sets. They are the same 24 pairs. The row is accurate. |

### What `closure_check.py` answered, and where it disagrees with me

The tool answered `CLOSURE BAD: 0` for all three blocks. It agrees with me on the SHAPE of every row and on every count. It disagrees with me on eight rows of C.22, because it never reads what a row claims.

**That is not a defect in the tool.** PG32 states its own limit at architecture.md:874: "It never decides that the closure the row claims is true." The tool is honest. The defect is that the appendix records CLOSED for eight rows that are not closed, and the document presents the appendix as the closure record.

---

## Part 2. Findings

## CRITICAL 1. Appendix C.22 records CLOSED for eight rows that are not closed

**OBSERVATION.** C.22 marks C18-2, C18-6, C18-W1, C18-W4, C18-W5, C18-W6 and C18-W7 as CLOSED. My verification makes seven PARTIAL and C18-W5 OPEN.

**CLAIM.** The block repeats the C17-1 defect at the row level instead of the count level.

**ARGUMENT.** C17-1 was a wrong denominator, and PG32 now holds the denominator. A row whose text is false costs the same as a missing row: the next reader believes the finding is answered. Seven of the eight r18 Warnings are in this state, so the rate is not one slip.

**EVIDENCE.** C18-W5 is decisive. The row claims the harness "attributes each `.out` file to its shape". The code reads `holders = [name for name, text in own.items() if not missing(piece, flatten(text))]`, which scans every file. I put all eight recorded lines into one wrong shape file and the run was green at exit 0.

**WHAT SHOULD CHANGE.** Rewrite the eight rows with their true state. Then fix each mechanism.

---

## CRITICAL 2. PG32's denominator still rests on one party, and three review shapes change it silently

**OBSERVATION.** `review_ids.py` reads the review with two regular expressions over a file format that no document constrains.

**CLAIM.** CL5 raises the cost of a wrong denominator. It does not create a second source.

**ARGUMENT.** CL5 proves that the block matches the file whose md5 the document states. The Architect writes the document and chooses the file. Both remain under one hand. Three shapes change the denominator and keep every run green.

**EVIDENCE.** I ran each shape.

1. One stray opening fence before the last Concern. `review_ids.py` prints `CONCERNS: 5 TOTAL: 19` for a 20-finding review. I then removed the N18-6 row, retyped the count sentence, retyped the marker, retyped the digest, and edited one line of `DATA_BLOCKS`. `placement_check.py` exit 0 and `closure_check.py` exit 0 over 19 of 20 findings. Six edits, all inside one write scope.
2. A `~~~` fence that quotes `## CRITICAL 7`. `FENCE` strips backtick fences only, so the parse prints `CRITICALS: 7` for a 6-Critical review. That is C18-W6 with one different character.
3. Headings at `### CRITICAL n`, or `## CRITICAL: Title`, or `## C1.`. Each prints `CRITICALS: 0` and exits 0, because C18-W6's fix now permits an empty family. `.claude/agents/engineering/engineering-critic.md:325` prescribes `CRITICAL: [Title]`, which is one of the three.

**WHAT SHOULD CHANGE.** Take the second source from the review's own text. `reviews/critic-spec-r18.md:744` states its own parse and its own count sentence. Make `review_ids.py` read that sentence and refuse a parse that differs. That one change turns all three shapes red. State the required heading format in PG32, and repeat it in the Critic's brief.

---

## CRITICAL 3. PG29 holds the probe table against a hand-typed tuple, and that tuple already names a rule the file does not implement

**OBSERVATION.** `RULE_IDS` at `placement_check.py:261` is a literal. Its comment reads "Every rule id the three prototypes implement".

**CLAIM.** PG29 claims the probe table and the implemented rule set are one set. It compares the table with a constant.

**ARGUMENT.** This is the exact method the eighteenth review named as the single cause of all six Criticals: a rule that takes one input on trust. PG29 was not one of the six sites revision 19 repaired.

**EVIDENCE.** `PG32` appears in `placement_check.py` once, inside `RULE_IDS`. The file implements no PG32 rule; `closure_check.py` does. So the invariant is already broken on the frozen copy. I then added a `PG99` row to the probe table and `PG99` to `RULE_IDS`, with no implementation anywhere. The run printed `PROBE ROWS: 57 PROBE BAD: 0` and exited 0.

**WHAT SHOULD CHANGE.** Derive the rule set from the code, not from a tuple. Scan each prototype for its own rule ids, and include `closure_check.py` and `review_ids.py` in the scan.

---

## CRITICAL 4. PG32's stated reason for not becoming a gate is false, and chunk M0 contradicts itself

**OBSERVATION.** architecture.md:868 reads: "`roadmap/duet-v1/reviews/` holds no file today and no chunk creates one, so it is routed to the Orchestrator and it is not decided here."

**CLAIM.** The premise is false, the decision is already made, and M0's write scope cannot be built.

**ARGUMENT.** That directory holds 19 review files, in the frozen copy and in `/Users/james/Developer/duet`. The Orchestrator decided that the closure check becomes a `plan-lint` gate through chunk M0. The document does not carry that decision. M0's scope at 13.1 names `tools/xtask/src/{check_conversions,check_placement,check_roster}.rs` and omits `check_closure.rs`, yet the same cell says `tools/xtask/tests/probes.rs` "ports every section 1.9 probe". PP32 is a section 1.9 probe. So M0 must port a probe of a guard no chunk writes.

**EVIDENCE.** `ls roadmap/duet-v1/reviews/` returns 19 files. architecture.md:9210 holds M0's cell. architecture.md:1348 holds the PP32 row.

**WHAT SHOULD CHANGE.** Delete the false sentence. Add `tools/xtask/src/check_closure.rs` and the three `cargo xtask check-closure` lines to M0's scope. State that `reviews/` is a tracked plan record.

---

## CRITICAL 5. Appendix B.5 gives six pins a manifest owner that section 13.1 refutes

**OBSERVATION.** B.5 names M3 for `cpal`, `gix`, `crossbeam-queue`, `arrayvec` and `criterion`, and M2 for `symphonia`.

**CLAIM.** An engineer who reads B.5 adds each pin in the wrong chunk and the wrong phase.

**ARGUMENT.** Section 13.1 puts all six in M4's list. M3's own cell adds `hound` alone. M2's cell reads "no new pin; the phase adds none". Decision 2 of 13.1 states plainly that "`criterion` is pinned by M4, not M7", and B.5 says M3. A `cpal` pin added in phase 3 makes the Linux gate fail before M0's package list is relevant.

**EVIDENCE.** architecture.md:12378 to :12391 against architecture.md:9212 to :9214. No rule reads the B.5 Owner column against 13.1.

**WHAT SHOULD CHANGE.** Correct all six Owner cells. Add a rule that holds the B.5 Owner column and the 13.1 pin lists to one set.

---

## CRITICAL 6. The per-phase `Cargo.lock` writer counts are wrong at seven of sixteen phases, and N16-5 is CLOSED on them

**OBSERVATION.** architecture.md:9657 states "phases 0 to 15: 2, 3, 2, 4, 6, 5, 3, 4, 4, 3, 2, 1, 1, 1, 0, 0".

**CLAIM.** The counts are a measurement of the document's own tables, and seven of them are wrong.

**ARGUMENT.** I parsed every chunk row and counted the Writes cells that name `Cargo.lock`. A second reviewer did the same independently and reached the same sequence. DR3 permits a recorded count only when section 1.9 prints it. No guard prints these.

**EVIDENCE.** My count is 2, 3, 4, 3, 7, 6, 4, 4, 4, 3, 1, 0, 1, 1, 0, 0. Phases 2, 3, 4, 5, 6, 10 and 11 differ. Phase 4 holds seven lock writers (C1, E3, F1, G1, M4, N2, X1) against a stated six. The N16-11 row at architecture.md:13089 records N16-5 as CLOSED on these numbers.

**WHAT SHOULD CHANGE.** Recompute the sequence from the tables. Mark N16-5 as OPEN until it is right. Add a rule that derives the sequence, or delete it under DR3.

---

## CRITICAL 7. No declared type closes the audio stream, and three sections depend on the close

**OBSERVATION.** `ConfigureCommand` holds exactly three arms: `Apply`, `Open` and `Reset`.

**CLAIM.** Three sections and one closure row state an action that no declared type can express.

**ARGUMENT.** `GraphConfigurator::backend` is "the one owner of the audio backend" and it lives on the engine handoff thread. The thread contract forbids the core to touch a device. `AudioBackend::stop` exists at architecture.md:4195, and no message carries the request to its owner. This is critic C15-8 at the close path: revision 15 gave the thread the duty of `open` and gave it no input, and revision 19 repeats it for `stop`.

**EVIDENCE.** architecture.md:5019 "The core then closes the stream and marks the engine `Faulted`". architecture.md:5078 "(1) The core closes the stream, so the audio thread stops". architecture.md:12973, a CLOSED row, rests on "the core closes the stream first on every path". The enum is at architecture.md:4840.

**WHAT SHOULD CHANGE.** Add a `ConfigureCommand::Close` arm. Name the engine handoff thread as the one caller of `AudioBackend::stop`. Add a row for that call to the section 5.12 table.

---

## CRITICAL 8. The section 5.11 audio idiom carries three panic paths under `panic = "abort"`

**OBSERVATION.** Section 5.11 prescribes five forms and says "These five forms are lint clean and remove a bounds check".

**CLAIM.** Form 2 and form 3 panic, and TH1 states that the audio thread has no panic path.

**ARGUMENT.** `chunks_exact_mut(channel_count)` panics when the count is zero. `split_at_mut` panics past the length. `copy_from_slice` panics on a length mismatch. `clippy::indexing_slicing` reads none of the three, so the section is lint clean and the claim about panics is still false. The release profile aborts, so one bad length ends the process and loses the take in progress.

**EVIDENCE.** architecture.md:6119 to :6123. TH1 at architecture.md:5785. Section 12.3 at architecture.md:8786.

**WHAT SHOULD CHANGE.** Use `split_at_mut_checked` and `get_mut` with a stated length test. Return `CycleOutcome::Faulted` on a refusal. Carry the channel count as a non-zero type.

---

## CRITICAL 9. The capture ring states no overflow rule, and a capture overrun marks no take

**OBSERVATION.** The 5.8 row "Audio | Disk | `rtrb` ring of `f32` | One per capture channel" names no action on a full ring.

**CLAIM.** The specification leaves the behaviour of a recording overrun to the implementer.

**ARGUMENT.** Section 5.8 states that "The rule that decides is the overflow action". Every other row of that table names one. The shortfall case is answered end to end with a zero fill, a fault, a user message, and `TakeFlag::HadShortfall`. The overrun case has a fault arm and no take flag, so a slow disk loses vocal audio and the take keeps no durable record of it.

**EVIDENCE.** architecture.md:5933 is the row. `TakeFlag` declares `Uncalibrated`, `HadShortfall`, `HadDrift` and `MidiPortLost`, and no overrun arm.

**WHAT SHOULD CHANGE.** State what the audio thread does with the refused frames. Add a `TakeFlag` arm and add it to the section 6.4 table.

---

## CRITICAL 10. The engine disk thread has no time bound and owns the only fault report path

**OBSERVATION.** Section 5.12 says "This table is every cross-boundary wait". No row bounds a media read or a take-file append.

**CLAIM.** A slow volume stops playback refill and the only fault drain at the same moment, and the user sees nothing.

**ARGUMENT.** The disk thread reads `media/<hh>/<hash>.wav` and appends take files. Section 12.4 step 2 puts the fault drain on that same thread and states that the cycle "runs at least every B14". No mechanism enforces B14. So `PlaybackStarved` and `CaptureShortfall` sit in a queue that nobody drains, and silence is the whole user-visible outcome.

**EVIDENCE.** architecture.md:6073 to :6078, architecture.md:6151 to :6184, architecture.md:8814.

**WHAT SHOULD CHANGE.** Give every media read and write a row in section 5.12. Move the fault drain to a thread that performs no file work, or add a core watchdog on B14.

---

## CRITICAL 11. `SlotKind` cannot type three stages that two MUST stories and the design contract name

**OBSERVATION.** `pub enum SlotKind { Equalizer, Compressor, Gate, DeEsser, Delay, Reverb }`.

**CLAIM.** Chunks K4 and K5 cannot build the elements the plan assigns to them.

**ARGUMENT.** PR MA-02 is a MUST and requires a limiter. PR M-03 is a MUST and requires gain and a high-pass. The design contract fixes five master stages: `Gain`, `EQ`, `Compressor`, `Limiter`, `Dither`. `StageCurve` carries `kind: SlotKind`, and section 10.3 says the element paints "a limiter gain-reduction bar". No `MasterChain`, `MasterStage` or `StageKind` type exists. The enum's own doc comment states that the set is closed on purpose, so the gap is a decision and not an omission.

**EVIDENCE.** architecture.md:4527 declares the enum. architecture.md:12077 declares `StageCurve`. architecture.md:8187 states the limiter bar. `product-requirements.md:573` and `:512`. `design-contract.md:881`.

**WHAT SHOULD CHANGE.** Add the missing arms, or declare a separate master-stage type. The change lands in chunk T3 at phase 2, eleven phases before K5 needs it.

---

## Warnings

**W1. CL4 does not enforce what PG32 states.** The rule says a CLOSED row names "a section that a heading of this document holds". The code fires only `if cited and unknown`, so a cell with no section-shaped token passes. I replaced a Section cell with "fixed in the tool" and the run was green. Five real CLOSED rows of C.22 are already that shape: C18-W4, C18-W5, C18-W6, C18-W8 and N18-6.

**W2. CL3 and CL5 search the whole document.** PG32 says the digest sits "beside the block". Both checks are substring tests over 13188 lines. I moved the count sentence and the digest sentence into section 1.1 and both runs stayed green. A reader of C.22 then sees neither.

**W3. CL2 compares sets, so a duplicate row is green.** I duplicated the C18-W1 row with the state flipped to OPEN. The run printed `GENERATED: 20 ROWS: 21 CLOSURE BAD: 0`. The two numbers are printed side by side and never compared. PG32 says "one closure row", and the guard does not enforce "one".

**W4. `review_ids.py` strips backtick fences only.** See CRITICAL 2, shape 2. `~~~` is standard CommonMark and a review of a review quotes headings.

**W5. `probe_roster_text.py` has a tuned floor and a one-way transcript compare.** `floor = max(0, len(shapes) - 10)`, which is 4 against 8 recorded lines. The comment above it says one line per shape and never mentions the ten. The transcript loop asks only whether each recorded line is in the run, so a deleted line stays green with no floor.

**W6. `probe_conversion.py` keeps a second fragment oracle and prints no floor.** It does not import `probe_fragments`, and its own `OUTPUT_HEAD` differs by one character class. N18-6's row names three importers and omits the fourth harness. The C17-7 row claims "Every harness prints a compared denominator and a floor", and this one prints no floor.

**W7. The section 1.9 harness table and its counts are wrong in five places.** The table names three harnesses and five exist; `probe_closure.py` and `probe_roster_text.py` are absent, so a reader who follows 1.9 never runs PP32. Line 1271 says `probe_conversion.py` "reads no document", and it refuses to start without one. Line 1272 says four PP25 shapes and six gate defects; the script plants five and nine. Line 1295 says eleven shapes; the script plants fourteen. Line 1280 says "Three prototypes carry the rules", and PG32 lives in two more.

**W8. The "Six planted gate defects" table is compared by no harness.** It is not a registered block and no harness reads it. It records `duet-engine/src/lib.rs:177:22` for the `GraphConfigurator` unwrap plant; the real compile prints `178:22`, which the PP25 cell at line 1339 also records. One recorded result that no run produced is live today. That is the CR-13 and C16-W2 class.

**W9. A budget VALUE is read by no rule.** I changed the B113 row from "13 elements" to "12 elements" and left the declaration at `SmallVec<[Ticks; 13]>`. Every guard was green and both CLOSED rows for C16-W17 and C17-W4 stayed in place. PG30 reads the citation set and never the value.

**W10. CG1b misses a `members` entry with no slash.** `sibling_patterns` uses `range(1, len(parts))`, which yields no pattern at depth zero. I built a workspace with `members = ["alpha"]`, `exclude = ["beta"]` and a real cast in `beta/src/lib.rs`. The guard printed `MEMBERS: 1 FILES: 1 FINDINGS: 0` and exited 0. This is the fourth instance of the class after C15-4, C16-2, C18-4 and C18-W8. It is latent, because this plan's manifest uses `crates/*` and `tools/*`, and the fix is one line.

**W11. Section 1.9 still describes the refuted PG31 behaviour.** architecture.md:802 reads "It then derives each chunk's line from its own id". architecture.md:9098 says that behaviour was the C17-W9 defect and that "The map is data now".

**W12. `fault_text.rs` holds three tables and FOUR tables, three lines apart.** architecture.md:8916 and :8919. Chunk K1 writes the file and reads the wrong count first. DR1 forbids this shape.

**W13. `SlotState` is inline and boxed in the same document.** architecture.md:3463 says every arm is inline because audio-owned state holds no heap. architecture.md:3475 says it "boxes its equalizer arm". The declaration is inline and `SlotState` is in the `audio-owned` block, so PG26 would refuse an implementer who followed the prose. The prose is still wrong.

**W14. The `basedrop` collector runs on two named threads.** TH5 at architecture.md:5804 names the engine handoff thread and adds "No deferred verb runs on that thread". architecture.md:7669 says the collector "also runs on a job thread (TH5)". An export render holds a job thread for minutes and would delay every free past B108.

**W15. The engine fault boundary names two primitives.** architecture.md:5937 declares an `rtrb` ring of `EngineFault` with an `AtomicU32` drop counter. architecture.md:8812 declares `ArrayQueue<EngineFault>`. The two overflow rules are opposite: `rtrb` drops the newest and `force_push` drops the oldest. The counter is called "the backpressure signal" and no reader is named, so `EngineFault::FaultsDropped` is unreachable.

**W16. The phase table restates the defect C16-17 closed.** architecture.md:9606 gives M0 "the `ci.yml` `plan-lint` job". architecture.md:9221 gives M0 the whole file, with the runner labels and the package list.

**W17. The held duration key resolves two ways.** The PRD MUST criterion is "the key that went down last". architecture.md:8496 says "the shortest held letter wins". `HeldDuration(u8)` is a bit set and cannot record order.

**W18. Two MUST thresholds have no section 1.6 row.** The 25-cent pitch threshold and the one-gigabyte free-space threshold appear nowhere in `architecture.md`. DR3 says section 1.6 is the one table of budgets.

**W19. Appendix B.1's own count sentences are read by nothing.** I changed "Seven suppressions" to "Three" and "Six complexity suppressions" to "Nine". Both runs exited 0 with no finding line. C18-W3 fixed the section 2.3 count word and left these two, three lines from the tables the same rule reads.

**W20. `CreateRequest` omits three fields the MUST create form collects.** The struct holds `path`, `name`, `template` and `sample_rate`. The PRD requires the form to ask for a key, a time signature and a tempo.

**W21. Two closure rows cite probe ids the specification records nowhere.** The C18-4 row cites `CP1b-dot` and the C18-W8 row cites `CP1b-deep`. Neither id appears in `architecture.md`. The CG1b probe row at line 1356 still reads "Three shapes"; the harness runs seven.

**W22. Four `research/` claims are superseded and still live.** `crate-survey.md:110` makes MusicXML the first storage format, and ADR 0002 decision 1 makes the canonical format the storage format. `crate-survey.md:63` prefers an in-house allocator guard, and ADR 0004 says it needs `unsafe impl GlobalAlloc`. `ardour-concepts.md:232` recommends `arc-swap`, and ADR 0004 bans it. `ardour-concepts.md:241` recommends a global image cache, and ADR 0006 rejects it. The document names `research/` as a source of record, and two revisions closed this with a prose sentence.

**W23. The shutdown contract covers the agent bridge alone.** No step stops the transport, closes the stream, finishes an open take, or drains the job queue. `JobRunner` has no stop step and never joins its workers. A quit during a record pass abandons a take whose container header the writer never finished.

**W24. `JobRegistry` holds two maps that never lose an entry.** `states` and `cancels` are keyed by a rising `JobId`, and no step of section 9.6 removes a finished entry. B36 bounds the queue and bounds neither map.

**W25. A vacant chain slot reports every cycle with no latch.** The runner reports `EngineFault::ChainSlotVacant` once per cycle and the condition lasts until the next topology change. At B7 that is about 375 faults each second into the B34 queue, which then fills the B29 event channel and passes B39.

---

## Concerns

| Id | Concern | Disposition |
|---|---|---|
| N19-1 | PG31b rejects nothing today. The found set and the allow list are the same 24 pairs. | Accepted debt, correctly recorded. I rebuilt the pair set and found no wrong row. File the machine SM8 allows in the plan document. |
| N19-2 | `RULE_IDS`, `CLOSURE_SECTIONS`, and the `DATA_BLOCKS` minima are literals that track the document. | File. Each needs an edit every revision, and each is a second copy of a fact. |
| N19-3 | `ROSTER_SIZES` is an environment seam at the placement guard's entry point, named in no section. | File. It changes diagnostic output only and gates no rule. |
| N19-4 | A chunk that writes into a crate it does not own is read by no rule. I moved a K5 path into `crates/duet-engine` and every run was green. | File. SM4 and SM5 both rest on the Writes columns. |
| N19-5 | `probe_roster_text.py` has a dead `import re`. | File with N18-6, which named the same class in `closure_check.py`. |
| N19-6 | PG33's third set is every `pub fn` the document declares, not every function that carries the suppression. | File. A B.1 row that shares a name with an unrelated function stays green. |
| N19-7 | The tools produce `__pycache__` under `roadmap/`, and `.gitignore` has no Python rule. No chunk owns `.gitignore`. | File. Add the rule to M0's scope. |
| N19-8 | Three of the last three reviews are almost wholly about the guard tooling, and my engineering pass found four Criticals in sections 5, 9 and 12. | File. The review loop has turned inward while the audio design went unread. |
| N19-9 | The cross-document pass returned about thirty further contradictions in the ADRs, the PRD and the design contract that I did not verify one by one. | File. I verified a sample of twelve and every one held. Treat the rest as a work list, not as findings. |

---

## Reactive Assessment

- **Responsive: FAIL.** No row bounds a media read or a take-file append, and section 5.12 claims it holds every cross-boundary wait.
- **Resilient: FAIL.** The house audio idiom carries three panic paths under `panic = "abort"`, and no declared type closes the stream.
- **Elastic: PARTIAL.** Every ring and channel carries a bound. `JobRegistry` grows for the life of the process and one vacant slot floods the event channel.
- **Message Driven: PARTIAL.** Most boundaries carry a bounded message with a stated overflow rule. The stream close has no message, the capture ring has no overflow rule, and `JobRegistry` is shared state across the worker seam.

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

Revision 19 built a real machine. PG32, PG33, PG34, PG26f, PG31b and the anchored CG1b all exist, all run, and all turn red on the plants they name. I confirmed every one. The eleven blockers are not a claim that the work was wasted.

The single biggest risk is the method. The eighteenth review said each new rule refuses the last review's exact instance and takes one input on trust, and named that as the cause of all six of its Criticals. Revision 19 applied the remedy at six sites and left the method in place everywhere else. PG29 still holds the probe table against a typed tuple. The B.5 Owner column, every budget value, the per-phase lock counts, and the Appendix B.1 count sentences are read by nothing, and three of those four are wrong right now. The closure appendix itself records CLOSED for eight rows my verification makes PARTIAL or OPEN, and PG32 cannot see that because PG32 checks shape.

The weakest Reactive property is **Resilient**. Section 5.11 prescribes three panic paths for the audio thread, the release profile aborts, and one bad length loses the take in progress. That is the opposite of what TH1 promises.

**Blocking list, in the order I would fix it.**

1. CRITICAL 7, 8, 9, 10 and 11. The audio and Master defects. They are the work, and they have gone unreviewed while the guards absorbed three revisions.
2. CRITICAL 5 and 6. Two plan tables that would misroute chunk work on the first day.
3. CRITICAL 1 and 4. Correct the eight closure rows and the false `reviews/` premise, and record the Orchestrator's gate decision in M0.
4. CRITICAL 2 and 3. Give `review_ids.py` the review's own stated count, and derive PG29's rule set from the code.
5. Warnings 1 to 25. None is deferrable. W1, W2, W3 and W4 are all in the guard that this revision made the centrepiece.

The nine Concerns above are filed with the disposition each row states.

This review holds 11 Criticals, 25 Warnings, and 9 Concerns.
