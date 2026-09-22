# Engineering Critic — specification review, revision 16

**Document under review.** `/Users/james/Developer/duet/roadmap/duet-v1/architecture.md`, 12167 lines.
**Frozen copy md5:** `dcb7b1c3238a08d37860403731a1e15c`.
Every line number below is a line of that frozen copy. The freeze was the first action of this
review. No file in the repository was written. No git command that writes was run.

**The document moved under this review, for the third revision running.** At the end of the review
the live file `architecture.md` had md5 `c06a6b67af2efc90a22c45edcb4c86ca` and 12175 lines, against
the frozen `dcb7b1c3238a08d37860403731a1e15c` and 12167 lines. The diff is 54 changed lines. Every
finding below is against the frozen copy and every line number is a frozen line number. A finding may
already be closed in the live file; the closure claim must be checked against a run, not against this
report. I checked one thing in the live file, because it is the top Critical: `MeterReading::SILENT`
still omits `cap`. **The Architect must stop editing a document that is under review.** A review of a
moving document costs the reviewer's whole run and gives the author an unreliable answer.

**Scope.** The full document, the six records under `roadmap/duet-v1/adr/`, and the six files under
`roadmap/duet-v1/tools/`. Cross-checked against `design-contract.md`, `product-requirements.md`,
`research/`, `CLAUDE.md`, `Cargo.toml`, `clippy.toml`, `deny.toml`, `scripts/dod.sh`, and
`crates/duet/src/`. Pinned sources checked under
`/Users/james/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/`.

**Verdict: NOT READY.**

---

## What I ran

| Run | Result |
|---|---|
| `placement_check.py` over the frozen document | `exit 0`. Baseline green. |
| `probe_run.py`, the full placement probe set | `PROBES BAD: 0`. 49 rows, every planted shape red. |
| `probe_conversion.py`, the full conversion probe set | `CONVERSION PROBES BAD: 0`. |
| `roster_compile.sh` over the frozen document | **`exit 1`. `ROSTER CLIPPY: FAIL`.** |
| `probe_roster.sh` over the frozen document | **`exit 1`. `FAIL: the baseline roster run is not green, so no shape is meaningful.`** |
| My own probes: MINE-16A, MINE-16B, G1 to G5 | Two guard defeats proven. See Critical 2 and Warning 1. |

Disk before and after every cargo run stayed above 110 GB free. One scratch directory was used.
`ROSTER_TARGET_DIR` was exported to one path and the target was deleted.

---

# CRITICAL

## CRITICAL 1. The third guard is red on this revision, and section 1.9 records a green run that no run produced

**OBSERVATION.** `roster_compile.sh` over the frozen document exits 1:

```
ROSTER CRATES:   17
ROSTER ITEMS:    395     FLOOR: 395
ROSTER IMPL BLOCKS: 49     FLOOR: 49
ROSTER LOCK:     887 packages in the repository lock, 873 after resolution
ROSTER LINT:     cargo clippy --workspace --all-targets -- -D warnings
duet-dsp/src/lib.rs:75:30: error[E0063]: missing field `cap` in initializer of `MeterReading`: missing `cap`
error: could not compile `duet-dsp` (lib) due to 1 previous error
ROSTER CLIPPY:   FAIL
```

The cause is at lines 6223 and 6228 to 6233. Line 6223 declares the field that closes C15-11:

```
pub struct MeterReading { peak: Finite, rms: Finite, true_peak: Finite, hold: Finite, cap: Finite }
```

Lines 6228 to 6233 do not set it:

```
    pub const SILENT: Self = Self {
        peak: Finite::ZERO,
        rms: Finite::ZERO,
        true_peak: Finite::ZERO,
        hold: Finite::ZERO,
    };
```

Section 1.9 lines 1317 and 1318 record the opposite:

```
ROSTER CLIPPY:   clean
ROSTER SIZES:    419 measured
```

**CLAIM.** The document records a measurement that no run of revision 16 produced, and the guard
that would have caught it is the guard the record falsifies.

**ARGUMENT.** The document's own rule DR3 states that every coverage count in the document is a line
of a real run. Critic finding CR-13 of the revision-12 review is the same defect: a recorded PP25
cell that no run produced. The header comment of `probe_roster.sh` names that finding and says "This
script is the run". The script now refuses to run: `probe_roster.sh` prints `FAIL: the baseline
roster run is not green, so no shape is meaningful.` and exits 1. So the whole PG25 probe set is
unexercised on revision 16, and `ROSTER SIZES: 419 measured` is a number no machine produced. Every
size the PG24 table states for a type the roster measures is therefore unverified this revision.

**EVIDENCE.** architecture.md:1317, :1318, :6223, :6228-6233. My run log at
`.../critic16/roster2/out.log`, line 150. `probe_roster.sh` exit 1.

**WHAT SHOULD CHANGE.** Add `cap: Finite::ZERO` to `MeterReading::SILENT`. Then run
`roster_compile.sh` and `probe_roster.sh`, and paste the real lines into section 1.9. Do not accept
this document until `probe_roster.sh` exits 0.

---

## CRITICAL 2. CG1b does not close guard hole 2. I reproduced the vacuous green it was written to stop

**OBSERVATION.** Appendix C.19 line 12157 claims guard hole 2 is closed by "CG1b, the member and
file floor, which exits 2 below either". The floor is a frozen constant in
`tools/conversion_check.py` lines 507 and 508:

```python
MEMBER_FLOOR = int(os.environ.get("CONVERSION_MEMBER_FLOOR", "3"))
FILE_FLOOR = int(os.environ.get("CONVERSION_FILE_FLOOR", "6"))
```

I built a throwaway workspace of five crates under `crates/*`, put a real cast in one, and excluded
that one in the manifest. Production default floors, no environment override:

```
=== exclude one crate of five, production default floors 3/6 ===
MEMBERS: 4   FILES: 8   FINDINGS: 0   MEMBER FLOOR: 3   FILE FLOOR: 6
EXIT=0

=== control: exclude line removed ===
  CAST: crates/duet-dsp/src/lib.rs: line 2
MEMBERS: 5   FILES: 10   FINDINGS: 1   MEMBER FLOOR: 3   FILE FLOOR: 6
EXIT=1
```

The exact shape the row names — `exclude = [...]` with a real cast in the excluded crate — exits 0.

**CLAIM.** CG1b is vacuous for the whole life of this project after chunk M1, and a second,
undocumented environment seam disables it even today.

**ARGUMENT.** A count floor bounds the scan from below by a constant. The document's own control
works only because the repository holds exactly three members today: excluding one leaves two, which
is below three. Section 13 creates fifteen more crates. From chunk M1 onward the workspace holds
four or more members and far more than six files, so one `exclude` line removes any crate and the
remaining count still clears the floor. The guard then reports success over the crates it did see,
which is the definition of the hole. The floor does not track the thing it measures, and nothing in
section 13 raises it.

Second, `CONVERSION_MEMBER_FLOOR` and `CONVERSION_FILE_FLOOR` are read at the production entry
point, not only in probes. I measured it:

```
=== A: default floors ===
FAIL: the scan covers 2 members and 2 files, and the floor is 3 members and 6 files; the guard is fail-closed.
EXIT=2
=== B: same workspace, floors zeroed from the environment ===
MEMBERS: 2   FILES: 2   FINDINGS: 1   MEMBER FLOOR: 0   FILE FLOOR: 0
EXIT=1
```

One environment assignment removes the floor. `grep` over architecture.md finds neither variable
name anywhere in the document. Chunk M0 ports this prototype to
`tools/xtask/src/check_conversions.rs`, and the specification the port follows does not state that
the seam exists. The port will either drop it, which breaks every conversion probe, or reproduce an
undocumented environment seam in the production gate.

Third, the source comment at line 505 of `conversion_check.py` names `CONVERSION_FLOOR`, a variable
the code does not read, and line 534's comment states "The floor is data in the document". It is
not. It is a literal in the script.

**EVIDENCE.** `tools/conversion_check.py`:72-77, :505-508, :534-538. architecture.md:1226, :2167-2172,
:12157. My runs above, reproducible at
`.../critic16/mine/cg1b2`.

**WHAT SHOULD CHANGE.** A count floor cannot express the invariant. The invariant is: every
directory under `crates/` and `tools/` that holds a `Cargo.toml` is a scanned member. Derive the
expected member set from that directory listing and fail closed on any member the scan does not
cover, by name. Remove the environment override from the production path, or gate it behind an
explicit argument that the gate never passes. State the mechanism in section 2.3 so chunk M0 can
port it.

---

## CRITICAL 3. `ConfigError::NothingToHandOff` is a phantom variant, and a `# Errors` section names it

**OBSERVATION.** Appendix C.19 line 12159 claims C15-6 is closed at "15.10, where
`HandoffRetryPending` and `NothingToHandOff` are two arms". Section 15.10 lines 10311 to 10340
declare `ConfigError` with these arms: `ChannelMismatch`, `PoolExhausted`, `RingBudgetExceeded`,
`StripBudgetExceeded`, `ParamBudgetExceeded`, `PoolHandoffBusy`, `HandoffRetryPending`, `Mix`.
There is no `NothingToHandOff`.

The name occurs three times in the document and never in a declaration:
- line 4989, inside the `# Errors` section of `hand_off`: "`ConfigError::NothingToHandOff` when `handoff_retry` is `None`";
- line 8474, a section 12.4 row;
- line 12159, the closure row itself.

**CLAIM.** A public function's `# Errors` contract names an error variant that does not exist. This
is a phantom reference.

**ARGUMENT.** `# Errors` is the contract a caller matches on. A caller that writes the arm the
document tells it to write does not compile. Line 8474 states "Both rows exist so the match stays
exhaustive" — a match cannot carry an arm for a variant that is not declared, so the sentence
asserts something impossible. The finding C15-6 was "one error variant names two opposite
preconditions"; the closure split the prose and did not split the type.

**EVIDENCE.** architecture.md:4989, :8474, :10311-10340, :12159.

**WHAT SHOULD CHANGE.** Either declare `NothingToHandOff` in `ConfigError` or delete both citations.
Do not leave a `# Errors` line naming an undeclared variant.

---

## CRITICAL 4. C15-7 is not closed, and section 15.14 states the opposite of the closure claim

**OBSERVATION.** Line 12160 claims: "5.5 step 5, the third outcome: drop `pending_request`, clear
`attempts`, send `EngineEvent::ConfigureRefused`, and the core clears `pending_configure` and never
re-sends a refusal".

Section 5.5 step 5 is lines 4711 to 4719. It states two outcomes, not three: the success path
(advance the layout, clear `pending_request` and `attempts`, send `GraphConfigured`) and the B95
fault path (send `EngineFault::HandoffBusy`, drop `pending_request`). `EngineEvent::ConfigureRefused`
appears nowhere in section 5.5. Its only two sites in the document are the declaration at line 10287
and line 10542.

Line 10542, in section 15.14, states the opposite of the closure claim:

```
    /// `EngineEvent::ConfigureRefused`, so a refusal loses no user change
```

The surrounding sentence at lines 10540 to 10543 says the core **re-sends** `pending_configure` on
`GraphConfigured` and on `ConfigureRefused`. Line 4725 to 4726 says `pending_configure` "covers one
case only": a full B109 channel.

**CLAIM.** Zero of the five clauses the closure row names are present at the site it names, and the
claim "never re-sends a refusal" is contradicted by the roster it is supposed to be closed in.

**ARGUMENT.** A refusal with no stated outcome and an uncapped re-send is an unbounded retry across
a component boundary. That is a Resilient failure and an Elastic failure at once: the core re-sends
on every `EngineEvent`, and the only cap anywhere near is B95, which bounds the engine-side wait
before a build and not any re-send. The budget refusals `PoolExhausted`, `StripBudgetExceeded`,
`ParamBudgetExceeded` and `RingBudgetExceeded` are raised in step 2 at lines 4689 to 4691, and no
step states what the thread does with `pending_request`, with `attempts`, or what it sends to the
core after one.

**EVIDENCE.** architecture.md:4689-4691, :4711-4719, :4725-4726, :10287, :10540-10543, :12160.

**WHAT SHOULD CHANGE.** Write the third outcome into step 5 with all five clauses, or change the
closure row to say what the document really does. Then give the `pending_configure` re-send one
stated cap and one stated trigger set, and make sections 5.5 and 15.14 agree.

---

## CRITICAL 5. C15-5 is not closed. One retry loop still carries two periods, 8 ms and 150 ms

**OBSERVATION.** Line 12158 claims: "1.6 B95 and the three other sites, all now B108". Three sites
say B108. Four sites still say B7:

| Line | Text |
|---|---|
| 935 | B95's own 1.6 row: "tries again on the next pass of its own loop, one attempt per B7" |
| 5779 | The 5.12 timeout table: "B95 attempts, one per B7" |
| 10324 | `ConfigError::PoolHandoffBusy` doc: "one attempt per B7" |
| 11330 | Appendix A: "one attempt per B7" |

Against lines 4627, 4689 and 4745, which state B108. B7 is 2.67 ms (line 847). B108 is 50 ms
(line 948).

**CLAIM.** The same three-attempt cap is about 8 ms at four sites and about 150 ms at three. The
finding was moved, not closed, and the budget row that names the id is one of the four that was not
changed.

**ARGUMENT.** B95 is a **Responsive** bound: it is the upper bound on how long a topology change
waits before the engine faults. Two values that differ by a factor of nineteen are not one bound. A
reader who takes the B95 row at line 935 as canonical builds the 8 ms behaviour; a reader who takes
`GraphConfigurator::attempts` at line 4627 builds the 150 ms behaviour. The loop parks on a bounded
receive of B108 (line 4745), so the implementation can only be 150 ms, and the budget row is wrong.

A second contradiction sits on the same id. Appendix B.5 line 11553 gives B95 a clock
(`Instant::elapsed`); Appendix C.15 line 12076 says B95 "states an attempt count and no clock".

**EVIDENCE.** architecture.md:847, :935, :948, :4627, :4689, :4745, :5779, :10324, :11330, :11553,
:12076, :12158.

**WHAT SHOULD CHANGE.** Change all four B7 sites to B108, starting with the B95 row at line 935.
Resolve the B.5 and C.15 disagreement about the clock in the same edit.

---

## CRITICAL 6. Three framework paths do not resolve, and one of them is a chunk instruction

**OBSERVATION.** I checked the pinned sources.

| Site | Written | Real path |
|---|---|---|
| architecture.md:8757 (chunk K1 instruction), :10715, :10717, :10788, :11331 | `gpui_kit::Theme` | `gpui_kit::component::Theme`. `gpui-component-0.6.4/src/lib.rs:119` `pub use theme::*;`; `gpui-kit-0.6.4/src/lib.rs` names no `Theme`. |
| architecture.md:7797, :11336, adr/0006:56 | `gpui_kit::VirtualList` | `gpui_kit::component::VirtualList`. `gpui-component-0.6.4/src/lib.rs:122`. |
| architecture.md:7752 | `gpui_kit::component::DropdownMenu` | `gpui_kit::component::menu::DropdownMenu`. `gpui-component-0.6.4/src/lib.rs:56` declares `pub mod menu;` with no glob re-export. |

The document already holds the correct form for one of the three: line 1606 of the machine-read
framework block writes `VirtualList   gpui_kit::component::VirtualList`.

`DropdownMenu` is also a trait, not a widget: `gpui-component-0.6.4/src/menu/dropdown_menu.rs:12`
declares `pub trait DropdownMenu: Styled + Selectable + ...`, and line 34 of the same file carries
`impl DropdownMenu for Button {}`. Line 7752 says `TopBar` "builds both from
`gpui_kit::component::DropdownMenu`". A view does not build a trait.

**CLAIM.** Chunk K1 is told to write code that does not compile, and the wrong paths are invented
API of exactly the class the `gpui-kit` skill names as a documented failure mode.

**ARGUMENT.** Line 8757 hands the engineer "the `cx.observe_global::<gpui_kit::Theme>` subscription".
`gpui_kit::Theme` does not exist at that path. The engineer will hit a compile error, guess, and
either reach the right path or add a second UI dependency. The document is the specification; a
specification that names a path the pinned crate does not export has already failed.

I confirmed that no guard catches this. I planted `gpui_kit::NoSuchThing` in prose at line 7797 and
ran the placement guard:

```
G5 invented framework path in prose EXIT= 0
['FRAMEWORK NAMES: 53    NAME MAP: 20', 'MISCLAIMED:      0     FRAMEWORK MISUSE: 0']
```

`Theme` is not in the name map at all, so the block that would hold it to a path does not hold it.

**EVIDENCE.** architecture.md:1606, :7752, :7797, :8757, :10715, :10717, :10788, :11331, :11336;
adr/0006-wrapped-timeline.md:56. `gpui-kit-0.6.4/src/lib.rs:95,106,108,110,143,145,149`;
`gpui-component-0.6.4/src/lib.rs:56,119,122`; `gpui-component-0.6.4/src/menu/dropdown_menu.rs:12,34`.

**WHAT SHOULD CHANGE.** Correct the nine sites. Add `Theme` to the framework name-map block so the
guard holds it. Extend the framework rule to check a `gpui_kit::` path written in prose and in a
chunk cell, not only one written in a declaration; the current rule leaves every chunk instruction
unguarded.

---

## CRITICAL 7. No declaration in the roster holds an `AudioBackend`, and two steps call methods on one

**OBSERVATION.** Line 4569 to 4570 tells the engine handoff thread to "call `AudioBackend::open`
with it, then call `AudioBackend::start`". The trait method at line 3912 takes `&mut self`. A grep of
the whole document for `dyn AudioBackend` returns nothing, and `GraphConfigurator` (lines 4601 to
4630) declares no backend field. The 5.7 thread row for that thread (line 5386) lists `configure`,
ring allocation, one `Input<GraphChain>`, one B105 producer and the collector, and no backend.

**CLAIM.** C15-8 closed half the hole. The command now carries the `StreamRequest`; nothing carries
the backend.

**ARGUMENT.** The doc comment that records the fix says it itself. Line 4573 to 4574 reads "Revision
15 gave the thread the duty and no input: no field reached an `AudioBackend`, and no command carried
a `StreamRequest`." The second clause is now false. The first clause is still true. An
`&mut self` method needs an owner, and the roster names none.

**EVIDENCE.** architecture.md:3912, :4569-4577, :4601-4630, :5386.

**WHAT SHOULD CHANGE.** Give `GraphConfigurator` a field that owns the backend, place the type in
the section 1.5 table, and state which thread constructs it and which drops it. The drop matters:
closing a `cpal` stream on the handoff thread is a different contract from closing it on the core.

---

## CRITICAL 8. Section 7.3 says the peak cap is not published. `MeterSnapshot` publishes it

**OBSERVATION.** Lines 6205 to 6207 state the C15-11 decision:

> **The audio thread is untouched**: it writes `MeterSnapshot` once per cycle under TH9, and a cap
> derived per frame on the foreground thread needs no new published field and no new fixed-size
> type.

Line 5657 declares `MeterSnapshot`:

```
    readings: [MeterReading; MAX_STRIPS],
```

`MeterReading` now carries `cap` (line 6223). `MeterReading` carries the `**Audio-owned**` marker at
line 6221 and sits in the audio-owned block at line 5304. `MeterSnapshot` carries the same marker at
line 5654.

**CLAIM.** The field is published, the audio thread writes it every cycle, and the stated owner is
the foreground thread. That is a field with two writers and one of them does not know it.

**ARGUMENT.** The audio thread constructs every `MeterReading` it puts in the snapshot. It must
write `cap` because Rust has no partial struct literal. It has no value to write, because line 6202
gives the cap to `MeterReader::poll`. So the audio thread writes a placeholder that the reader
overwrites. That is not "untouched": it is a per-cycle write of a field the writer does not own, and
it enlarges the per-cycle memory copy that line 5650 to 5651 charges to B8. The recorded PG24 size
for `MeterSnapshot` is therefore stale, and the run that would have measured it is the roster
compile of Critical 1, which did not complete.

A third gap sits beside it. `MeterReader` (lines 10562 to 10566) holds `output`, `resets` and
`latest`. B111 requires the cap to be held "at the peak that set it" for 1200 ms. Nothing in
`MeterReader` and nothing in `MeterReading` holds a per-strip timestamp, and no field holds the
frame clock line 6205 says the reader reads. `MeterReader::poll`'s own doc at line 10569 still
describes `poll` without the cap.

**EVIDENCE.** architecture.md:5650-5657, :6202-6207, :6221-6223, :10562-10566, :10569.

**WHAT SHOULD CHANGE.** Choose one. Either take `cap` out of `MeterReading` and put it in a
foreground type the reader owns, which keeps the published snapshot unchanged and makes line 6206
true; or keep it and delete the false sentence, restate the B8 arithmetic, and give `MeterReader` the
per-strip deadline state that B111 needs.

---

## CRITICAL 9. The `end_to_end` gate test cannot pass in the two phases after the phase that writes it

**OBSERVATION.** The `selected-tests` guard block records:

```
| `end_to_end` | J1 | 7 | `crates/duet-agent/tests/end_to_end.rs` | Plain | Rung two, command 1 |
```

`Plain` means it runs in `scripts/dod.sh` at every commit. Lines 9136 to 9146 say the test calls
`MasterMeasure`, asserts progress and completion, and exports a 48 kHz 24-bit WAV with the Streaming
preset. Lines 8741 to 8745 state that I1 returns `GatewayError::NotYetImplemented` for
`ExportAudio`, `MasterMeasure` and `Gc` until I4. I4 is in phase 9. H3, the encode stage, is in
phase 8. H4, the Streaming preset, is in phase 9. J1 is the only writer of the file and no later
chunk names it.

**CLAIM.** Every commit of phases 7 and 8 is blocked by the repository's own gate, and the plan gives
no way out.

**ARGUMENT.** `scripts/dod.sh` is the only gate surface, and the `.githooks/pre-commit` hook runs it.
A `Plain` integration test that asserts an outcome no code delivers for two more phases fails at
every commit in between. CLAUDE.md forbids `--no-verify`. The plan therefore contains an interval in
which no engineer can commit. The same class was raised as critic S4 and the document says at line
8736 that it closed it for I3.

**EVIDENCE.** architecture.md:8741-8745, :8750, :9096, :9117, :9136-9146; the `selected-tests` block.
`/Users/james/Developer/duet/scripts/dod.sh`.

**WHAT SHOULD CHANGE.** Mark the test `#[ignore]` until phase 9 and name the chunk that removes the
attribute, or split the test so the phase-7 half asserts only what phase 7 delivers, and give the
second half its own row and its own chunk.

---

## CRITICAL 10. Chunk I4 cannot commit its own work, and it must edit a crate that is not in its scope

**OBSERVATION.** I4's whole write scope is one file:

```
| I4 | 9 | The `ExportAudio`, `MasterMeasure`, and `Gc` dispatch arms, with the TH8 re-validation | `crates/duet-core/src/gateway.rs` | ... |
```

Line 8994 says the `MasterMeasure` arm calls the measure job and line 8995 says the `ExportAudio`
arm calls the encode stage. Both live in `duet-export`. SM1 (line 8506) requires the chunk that uses
a dependency to add the manifest entry in the same commit, and `cargo machete` forbids an earlier
chunk from adding it unused. SM5 rule 2 (line 8551) then also requires `Cargo.lock`. I4's scope
holds neither `crates/duet-core/Cargo.toml` nor `Cargo.lock`.

Second, line 8744 says "I4 deletes the variant" — `GatewayError::NotYetImplemented`. Line 394 places
`GatewayError` in `duet-command`. The only chunk that writes `duet-command` is T4, in phase 2. I4
must edit a file seven phases outside its own line and outside its own write scope.

**CLAIM.** Chunk I4 as written is not executable.

**ARGUMENT.** A write scope is the contract that lets phases run in parallel. A chunk that must touch
a file outside its scope either breaks the contract or stops. Both faults are mechanical and both are
visible from the row itself.

**EVIDENCE.** architecture.md:394, :8506, :8551, :8730, :8744, :8994-8995.

**WHAT SHOULD CHANGE.** Add `crates/duet-core/Cargo.toml` and `Cargo.lock` to I4's write scope. Move
the deletion of `NotYetImplemented` to a chunk that owns `duet-command`, or keep the variant and say
so.

---

## CRITICAL 11. Chunk C3 creates a file that SM2 forbids it to create

**OBSERVATION.** SM2 (lines 8516 to 8520): "The first chunk of a line creates every module file the
whole line will ever need, at every depth, as a stub, and names each one in its write scope. ...
Every later chunk in the line **modifies** a stub and creates no file."

C1's write scope (line 8665) names `src/chain/{topology,state,graph,migrate}.rs`.
C3's write scope (line 8667) names `chain/{topology,state,graph,migrate,configure}.rs`.

`configure.rs` is in C3 and not in C1.

**CLAIM.** C3 creates a file, which SM2 forbids, and `chain.rs` needs a `mod configure;` line that
C1's stub does not carry.

**ARGUMENT.** SM2 exists so that a later chunk never has to touch a `mod` declaration an earlier
chunk owns. The missing stub reintroduces exactly the coupling SM2 removes.

**EVIDENCE.** architecture.md:8516-8520, :8665, :8667.

**WHAT SHOULD CHANGE.** Add `configure.rs` to C1's stub list.

---

## CRITICAL 12. Three chunk pairs share a phase across a crate edge the document itself states

**OBSERVATION.** Section 13.3 puts these pairs in one phase, and section 13.4 carries no link between
them, although section 1.3 or section 13.1 states the dependency.

| Pair | Phase | The stated edge | Where 1.3 states it |
|---|---|---|---|
| T2 and T3 | 1 | `duet-session` -> `duet-score` | line 277 |
| T4 and X1 | 2 | `duet-interchange` -> `duet-command` | line 278 |
| J1 and K1 | 7 | `duet` -> `duet-agent` | 1.3 edge list; line 8610; the shell table at line 8775 gives K1 `agent_bridge.rs` |

I confirmed by grep that no `T2 before T3`, no `T4 before X1`, and no `J1 before K1` row exists.

**CLAIM.** Under SM1 each of the three consumers must add a `{ workspace = true }` entry for a crate
its partner is building in the same phase. The three pairs cannot run in parallel.

**ARGUMENT.** The phase table is the parallelism plan. A phase asserts that its chunks have no
ordering between them. These three pairs do. The trunk is four lines of one chunk each (line 8626),
so SM6 supplies no intra-line order for T2 and T3 either.

**EVIDENCE.** architecture.md:277, :278, :8610, :8626, :8631-8633, :8657, :8750, :8757, :8775, the
13.4 table at :8949-9001.

**WHAT SHOULD CHANGE.** Add the three links and move the later chunk of each pair to the next phase,
or state why the edge does not bind at build time.

---

## CRITICAL 13. The 12.4 refusal table is keyed on `GatewayError` and lists a variant `GatewayError` does not hold

**OBSERVATION.** Line 8447: "| The refusal table | `GatewayError` | One row per variant, including
`PoolExhausted`, `RingBudgetExceeded`, `StripBudgetExceeded`, `ParamBudgetExceeded`,
`BusRoleReserved`, `PoolHandoffBusy`, ...". Line 8477 repeats the list.

`GatewayError` at lines 10009 to 10027 holds `NoProject`, `ProjectBusy`, `RecordInProgress`,
`HistoryUnresolved`, `SaveRaced`, `JobQueueFull`, `PoolExhausted`, `RingBudgetExceeded`,
`StripBudgetExceeded`, `ParamBudgetExceeded`, `BusRoleReserved`, `NotYetImplemented`, `Score`,
`Session`, `Mix`, `Command`, `Upstream`. There is no `PoolHandoffBusy` and no bridge arm for
`ConfigError`.

**CLAIM.** `PoolHandoffBusy` is a `ConfigError` variant. It cannot appear in a table keyed on
`GatewayError`, and no declared conversion carries it there.

**ARGUMENT.** Line 8476 says one table in `crates/duet/src/shell/fault_text.rs` "fixes the user
message for each variant, and for each `GatewayError` variant". Chunk K1 writes it. K1 will write a
match on `GatewayError` and find no arm to put the `PoolHandoffBusy` message in. The engine-side
failure then has no user message, which is the Responsive property the whole of 12.4 exists to
supply.

**EVIDENCE.** architecture.md:8447, :8473-8477, :10009-10027, :10329.

**WHAT SHOULD CHANGE.** Either add a `Config(ConfigError)` arm to `GatewayError` and say which
conversion produces it, or move the `PoolHandoffBusy` row into the fault table, which is keyed on
`EngineFault` and already carries `HandoffBusy`.

---

## CRITICAL 14. `duet_time::convert::finite_to_f32_saturating` is a phantom function, and the suppression count disagrees with itself

**OBSERVATION.** Line 7464 gives `LogicalPx::to_f32` a body:

> ``/// `duet_time::convert::finite_to_f32_saturating`, which is the one``

Lines 2067 to 2134 are the whole `duet-time::convert` declaration block. It declares `Unit`,
`Rounding`, `muldiv`, `i16_to_f32`, `i24_to_f32`, `i32_to_f32`, `unit_to_i24`, `unit_to_i32`,
`f64_to_f32`, `ticks_to_f64`, `superclock_to_samples` and `samples_to_superclock`.
`finite_to_f32_saturating` is not among them. Section 15.1 (lines 9285 to 9312) does not declare it
either.

The counts disagree with each other:
- Line 2136: "**Six** functions carry a suppression", and it names the six.
- Line 11364: "**Seven** suppressions in `duet-time::convert`."
- Line 11378: the seventh row, for `finite_to_f32_saturating`.
- Line 11449: "Twenty sites are predicted in total: **seven** conversions, six complexity refusals,
  four `missing_copy_implementations` expectations, and three `variant_size_differences`
  expectations." Against the section 2.3 count of six, the sum is nineteen, not twenty.

**CLAIM.** A method body names a function no chunk writes, and Appendix B.1 holds a suppression row
for it.

**ARGUMENT.** Chunk T1 writes `crates/duet-time/src/convert.rs` from section 2.3. Section 2.3
declares twelve items and this is not one. `LogicalPx::to_f32` therefore calls nothing. Appendix B.1
is the register of every accepted suppression in the plan; a row there for a function that does not
exist is a suppression with no site, which is the exact shape the appendix exists to stop.

**EVIDENCE.** architecture.md:2067-2134, :2136-2137, :7464, :9285-9312, :11364, :11378, :11449-11451.

**WHAT SHOULD CHANGE.** Declare the function in section 2.3, add it to the list at line 2136, and
correct the total at line 11449 to the real sum.

---

## CRITICAL 15. Master mode has a linear timeline in the contract and no reachable path cache in the plan

**OBSERVATION.** ADR 0006 decision 9 area, line 55: "**Mix and Master use a linear timeline with one
samples-per-pixel scalar, and `MixView` owns their path cache.**" Line 7975 and line 11318 say the
same. The entity tree at lines 7259 to 7267 makes `MixView` and `MasterView` two children of
`DuetApp` with no edge between them. `MasterView` (lines 11086 to 11097) declares `meters`,
`stages`, `report`, `target`, `export`, `job`, `state`, `focus` and `subscriptions`. It holds no
cache, no `Entity<MixView>`, no scroll handle and no zoom field. `design-contract.md:198` gives
Master a linear timeline above the master chain.

**CLAIM.** Master is told to paint a timeline from a cache it has no handle to.

**ARGUMENT.** A sibling entity is not reachable. GPUI gives a child no path to a sibling except
through the parent, and `MasterView` declares nothing that would carry it. Chunk K5 writes
`src/master/{view,export_dialog,report}.rs` only (line 8761); the cache module `src/record/cache.rs`
belongs to K3 (line 8759). No chunk gives `MasterView` a cache and no chunk gives `DuetApp` a
forwarding path.

**EVIDENCE.** architecture.md:7259-7267, :7975, :8759, :8761, :11086-11097, :11318;
adr/0006-wrapped-timeline.md:55; design-contract.md:198.

**WHAT SHOULD CHANGE.** Either give `MasterView` its own cache field and say who fills it, or move
the shared cache into `DuetApp` and give both views a read handle. Then correct ADR 0006 line 55.

---

## CRITICAL 16. ADR 0004 clause 12a says no second refusal variant exists. Four exist, with the same names and payloads

**OBSERVATION.** `adr/0004-backend-and-threading-contract.md:167-169`:

> **One enum owns that refusal.** `configure` returns `ConfigError`, and `EngineError` wraps it with
> `#[from]`, so a caller still sees the cause and **no second variant exists**.

architecture.md:10314-10318 declares in `ConfigError`: `PoolExhausted { kind: SlotKind }`,
`RingBudgetExceeded { requested_bytes: u64 }`, `StripBudgetExceeded { requested: u32 }`,
`ParamBudgetExceeded { requested: u32 }`.

architecture.md:10016-10019 declares in `GatewayError`: the same four names with the same payloads.

**CLAIM.** The ADR clause is false against the body of the specification it governs.

**ARGUMENT.** The ADR says revision 6 "wrote both names and a reader could not tell which one
`configure` returns". Revision 16 writes both names again. A caller matching a refusal must now know
which of the two enums it is looking at, and the two carry no conversion between them that the
document declares. This is the failure the clause was written to prevent, restored.

**EVIDENCE.** adr/0004-backend-and-threading-contract.md:167-169; architecture.md:10009-10027,
:10311-10340.

**WHAT SHOULD CHANGE.** Declare the conversion from `ConfigError` to `GatewayError` and say which
one a gateway caller sees, or withdraw clause 12a's last sentence.

---

## CRITICAL 17. No chunk changes the live CI file that rung one describes

**OBSERVATION.** Lines 9023 to 9026 say rung one "runs on `ubuntu-26.04` and on `macos-26`" and that
"The Linux job installs `libpipewire-0.3-dev`, `libasound2-dev`, and `pkg-config` before the gate".
The live file `/Users/james/Developer/duet/.github/workflows/ci.yml` line 23 reads
`os: [macos-latest, ubuntu-latest]`, and its package list at lines 37 to 41 holds `libasound2-dev`
and `pkg-config` and not `libpipewire-0.3-dev`.

Chunk M0's write scope (line 8602) covers `.github/workflows/ci.yml` **(the `plan-lint` job)** only.
No chunk changes the runner labels of the `dod` job, and no chunk adds the package.

**CLAIM.** Section 11.5 says cpal links both libraries at build time. The first commit that adds the
cpal dependency fails the Linux gate, and the file that would fix it is outside every write scope.

**EVIDENCE.** architecture.md:8222, :8602, :9023-9026;
`/Users/james/Developer/duet/.github/workflows/ci.yml:23,37-41`.

**WHAT SHOULD CHANGE.** Give one chunk the `dod` job of `ci.yml` in its write scope, name it as a
policy file under SM4, and state the runner labels the plan requires.

---

# WARNING

## WARNING 1. PG26d closes the named shape and not the class. A four-line edit still disarms all three TH1 rules

**OBSERVATION.** Line 12156 claims PG26d holds the `audio-owned` block and the `**Audio-owned**`
marker "to ONE SET in both directions". That claim is true, and I confirmed the two documented
probes are red. I then planted the two-sided version.

I moved `Transport` out of the block and `ChannelConfig` in, and moved the marker line from the
`Transport` declaration to the `ChannelConfig` declaration. Four lines. Result:

```
MINE-16A swap-only EXIT=0
AUDIO OWNED:     37     AUDIO EXEMPT: 1     AUDIO DEFERRED: 33     HEAP IN AUDIO: 0     DROP IMPLS: 3
GROW IN AUDIO:   0     LOCK IN AUDIO: 0     ROOT BAD: 0
```

I then added the payload the rules exist to catch, `probe_heap: Vec<u8>` on `Transport`:

```
=== MINE-16A + payload ===
HEAP IN AUDIO: 0     ROOT BAD: 0
EXIT=0

=== CONTROL: payload alone, no swap ===
EXIT=1
```

The control is red. The swapped run is green with the same payload.

**CLAIM.** The two sources PG26d compares are two hand-written copies of one judgement. Holding them
consistent tests self-consistency, not truth.

**ARGUMENT.** The block and the marker are both assertions by the same author in the same document in
the same changeset. Neither is derived from anything. A guard built on two copies of one claim raises
the cost of a disarm from one line to four and adds no independent oracle. Guard hole 1 named the
one-line shape; the class is "the root set is asserted, never derived".

I checked whether a derived oracle is available. It is not a clean substitute: of the 37 roots, 11
are not reachable from `GraphState` through declared fields, and 10 declared types that are reachable
are not roots. So the set is a genuine hand judgement about what `GraphRunner::run` touches.

**EVIDENCE.** architecture.md:5231-5320, :5595, :10188, :12156. `tools/placement_check.py`:632-670.
My runs at `.../critic16/mine/pg26d`.

**WHAT SHOULD CHANGE.** Every other rule in section 1.9 states its own limit at its own site. PG26d
states none. Write the limit: "PG26d holds two declarations of one judgement consistent. A
coordinated edit of both defeats it, and no third source exists." Then either derive a third source
from the `GraphRunner::run` sketch, or accept the limit in writing so the next reviewer does not read
"ONE SET in both directions" as a proof it is not.

## WARNING 2. The recorded probe output is never compared with the run, and one cell is already stale

**OBSERVATION.** `probe_run.py` decides pass or fail at line 681:

```python
ok = code == expected and len(lines) == len(tokens)
```

It compares the exit code and the number of output lines. It never compares the text. The docstring
at line 8 says it "prints the exit code and the lines the probe's row records", which is true, and it
does not check them.

The PG27 row at line 1221 records, for the `audio-owned` block:

> ``FAIL: the `audio-owned` block of this document: it holds 0 rows and the stated minimum is 33 ...``
> the same line with `it holds 32 rows`

The guard's registered minimum is 37 (`tools/placement_check.py`:404) and the real run prints:

```
PP27:audio-owned:deleted   exit 2  OK   FAIL: ... it holds 0 rows and the stated minimum is 37
PP27:audio-owned:emptied   exit 2  OK   FAIL: ... it holds 36 rows and the stated minimum is 37
```

**CLAIM.** The harness is a weak oracle, and it has already let a recorded measurement go stale.

**ARGUMENT.** The whole point of `probe_run.py` is that "a recorded result is a measurement, so a
measurement needs a machine" (its own docstring). A machine that checks only the exit code and the
line count admits any text. Critic CR-3 and CR-13 were exactly this class.

**EVIDENCE.** `tools/probe_run.py`:8, :681. `tools/placement_check.py`:404. architecture.md:1221.

**WHAT SHOULD CHANGE.** Compare the recorded text with the produced text, and fail on a difference.
Then correct line 1221 to 37 and 36.

## WARNING 3. The token `Path` names two types in the machine-read blocks, and one row's assertion is now false

**OBSERVATION.** Line 347 puts `Path` in the framework-types block. Line 1559 of the name map writes
`Path          std::path::Path`. Line 1534 gives `Path` an external verdict:

> | `Path` | `not Copy` | ... | A borrowed path. It is unsized, so **no declaration stores one**, and the row states the true verdict (critic K-5) |

Line 7949 now stores one:

```
    paths: BTreeMap<PathKey, Arc<Path<Pixels>>>,
```

Lines 1494 to 1496 state that the external block holds a name that "section 1.5 does not place **and
the framework block does not name**". `Path` is in the framework block and has an external row.

**CLAIM.** One token carries two meanings in blocks a machine reads, and a row that a machine reads
asserts a fact the document contradicts.

**ARGUMENT.** The C15-2 fix is right in substance: the cache should hold the tessellated value.
`gpui-pre-0.3.5/src/scene.rs:789` declares `Path<P>` over plain data, so it is `Send`, and
`PathBuilder::build` at `path_builder.rs:244` is a pure tessellation that needs no window. The
design is sound. The naming is not. A roster compile that resolves `Arc<Path<Pixels>>` through the
name map reaches `std::path::Path<Pixels>`.

**EVIDENCE.** architecture.md:347, :1494-1496, :1534, :1559, :7949. `gpui-pre-0.3.5/src/scene.rs:789`;
`gpui-pre-0.3.5/src/path_builder.rs:244`.

**WHAT SHOULD CHANGE.** Rename one of the two. Give the framework entry a distinct token, or spell
the standard-library one `StdPath`, and state in 10.5 that `Path<Pixels>` is `Send` with the cite.

## WARNING 4. Two pinned-source claims in machine-read or load-bearing text are false

**OBSERVATION.** Two, both verified against the pinned source.

1. Line 1525: "| `Producer`, `Consumer` | `not Copy` | none | ... each supplies none of the nine".
   `rtrb-0.4.0/src/lib.rs:291` and `:516` both carry `#[derive(Debug, PartialEq, Eq)]`. `PartialEq`
   and `Eq` are two of the nine.
2. Lines 4765 to 4767: "It calls `Collector::try_cleanup` and reports a refusal as
   `EngineFault::HandoffBusy`, because a refusal means a `Handle` outlived step 3."
   `basedrop-0.1.3/src/collector.rs:328-341` refuses on a live `Handle` **or** a non-zero allocation
   count. The crate's own doc at `collector.rs:304-305` says so.

**CLAIM.** One is a wrong row in a block the document says a machine reads. The other is a wrong
diagnosis on a shutdown path.

**ARGUMENT.** The first is conservative in direction: a row that under-reports traits can only
produce a false red, never a miss. It still matters, because the document presents these rows as
verified facts about pinned sources, and one is not. The second is not conservative: a refusal caused
by a live `Owned` or `Shared` is reported as "a `Handle` outlived step 3", which sends the reader
looking in the wrong place during a shutdown fault.

**EVIDENCE.** architecture.md:1525, :4765-4767. `rtrb-0.4.0/src/lib.rs:291,516`.
`basedrop-0.1.3/src/collector.rs:304-305,328-341`.

**WHAT SHOULD CHANGE.** Correct line 1525 to `PartialEq` `Eq`. Give step 5 the second cause and say
what the thread reports for it.

## WARNING 5. `MixView` holds a path cache and no build task

**OBSERVATION.** `RecordView` (lines 11020 to 11035) carries `cache: PathCache` at 11026 and
`build: Option<Task<()>>` at 11027. `MixView` (lines 11057 to 11069) carries `cache: PathCache` at
11063 and no build-task field. Section 10.5 line 7975 gives Mix and Master a linear-key cache that the
same background task must fill.

**CLAIM.** The Mix cache has no owner for the task that fills it.

**ARGUMENT.** A `Task` stored nowhere is dropped at the end of the statement and the work is
cancelled before it starts. This is a documented GPUI failure mode. Every path Mix paints would miss,
and by the stated miss rule (line 7910) a miss paints nothing, so the Mix waveform never appears.

**EVIDENCE.** architecture.md:7910-7913, :7975, :11026-11027, :11063.

**WHAT SHOULD CHANGE.** Give `MixView` the same `build: Option<Task<()>>` field, or state which
entity owns the Mix build task.

## WARNING 6. Three link reasons cross an edge section 1.3 forbids

**OBSERVATION.** 1.3 rule 6 gives `crates/duet` no `duet-engine`, no `duet-media` and no
`duet-project` edge. Three 13.4 reasons name a type or a call on the far side.

| Line | Reason | The problem |
|---|---|---|
| 8983 | "C3 before K4 \| The meters read the `MeterSnapshot`" | `MeterSnapshot` is `duet-engine` (line 399). `crates/duet` cannot name it. Line 5505 gives the real seam: `duet-core` holds the `Output<MeterSnapshot>` and `MeterReader`, which I3 writes. |
| 8981 | "C1 and C3 before K3 \| The input and monitor controls call the device selection" | Device selection is `duet-engine` work (line 8665). The view must send a verb through `duet-core`. |
| 8988 | "F1 before K6 \| `StartView` offers the templates that F1 writes" | `crates/duet` has no `duet-project` edge. Line 8990 phrases the same class correctly through a verb. |

**CLAIM.** Three reasons describe a call the architecture forbids. A reason is the thing an engineer
reads to decide what to write.

**EVIDENCE.** architecture.md:399, :5505, :8665, :8981, :8983, :8988, :8990; 1.3 rule 6.

**WHAT SHOULD CHANGE.** Restate the three reasons through the `duet-core` seam, as line 8990 already
does.

## WARNING 7. Section 13.4 omits the one seam that carries the meters and the playhead to the view

**OBSERVATION.** C3 writes `TransportSnapshot` and `MeterSnapshot` (line 8667). I3 writes
`MeterReader` and `TransportReader` (line 8729). K3 and K4 read them. Story X-02 (line 8875) routes
the playhead through "C3 `TransportSnapshot`" to "K3 `PlayheadLayer`". No `C3 before I3`, `I3 before
K3` or `I3 before K4` row exists. Line 8949 says the table holds every cross-line link.

**CLAIM.** The phase numbers happen to satisfy the order, so nothing breaks today. The table claims
completeness and is not complete.

**ARGUMENT.** A schedule that is correct by accident is correct until someone moves a chunk. The
table is the artefact a person moves a chunk against.

**EVIDENCE.** architecture.md:8667, :8729, :8875, :8949.

**WHAT SHOULD CHANGE.** Add the three rows.

## WARNING 8. Three of the four SHOULD rows describe work that is not the story, and the prose contradicts the table

**OBSERVATION.**

| Story | Table row | Prose | The story |
|---|---|---|---|
| C-12 | line 8894: "K2 \| The Compose context menu gains a `Transpose` item" | line 8898: "X3 and K1 carry C-12" | MIDI file import in the user interface |
| MA-05 | line 8896: "K5 \| the Master export dialog gains a dither choice" | line 8899: "H3 carries MA-05 and MA-06" | Per-part export |
| R-12 | line 8895: "K3 \| the take list gains a solo toggle per take" | — | Region move and snap |
| C-14 | line 8784 gives it to K6 | line 8898 gives it to K2 | — |

**CLAIM.** Three SHOULD stories have a chunk row that delivers different work, and two of them also
have two conflicting owners. Appendix C.19's N15 line claims "13.2, the four SHOULD stories with
real work". The work is real; it is not the story's work.

**ARGUMENT.** A dither choice is MA-06. A per-take solo toggle is R-10 shaped. Neither the per-track
preview of C-12 nor the drag and snap of R-12 is delivered by any chunk. Three SHOULD stories
(R-10, MA-05, MA-06) also have no view chunk although each states a user-interface behaviour.

**EVIDENCE.** architecture.md:8784, :8894-8899; product-requirements.md, the SHOULD rows.

**WHAT SHOULD CHANGE.** Rewrite the four rows against the story text, resolve the C-14 owner, and
give the three user-interface SHOULD stories a view chunk or move them to LATER.

## WARNING 9. Chunk I3 names a framework type inside `duet-core`, which line 338 forbids

**OBSERVATION.** I3's goal (line 8729) places "the note-entry drain in `Window::on_next_frame`" in
`crates/duet-core/src/midi_entry.rs`. Line 338 states that only `crates/duet` and `duet-agent` may
name a framework type, and the guard block at line 339 lists `Window`. `duet-core` has no `gpui-kit`
edge in 1.3. Line 6798 repeats it.

**CLAIM.** The registration belongs to K1's `CoreHost` frame pump. `duet-core` can own a plain drain
method only.

**ARGUMENT.** This is the framework-boundary rule that the whole crate graph exists to hold. The
placement guard reads declarations, not chunk cells, so nothing catches it.

**EVIDENCE.** architecture.md:338-339, :6798, :8729, :8757.

**WHAT SHOULD CHANGE.** Move the registration to K1 and leave `duet-core` a drain method with no
framework type in its signature.

## WARNING 10. Chunk K5 is told to write a bench file that is not in its write scope

**OBSERVATION.** K5's goal (line 8761) names "the `criterion` bench over one Master frame that
section 10.2 requires". K5's Writes column holds three source paths and no `crates/duet/benches/`,
no `crates/duet/Cargo.toml`, no `Cargo.lock`. Line 8760 gives all three to K4 in phase 10, and SM6
forbids K5 from sharing them.

**CLAIM.** Either the goal is wrong or the chunk cannot do what it is told.

**EVIDENCE.** architecture.md:8760, :8761, :9212-9214; SM6 at :8560.

**WHAT SHOULD CHANGE.** Add the three paths to K5, or move the Master bench into K4 and correct the
goal.

## WARNING 11. Rung two command 4 claims a property no chunk writes a test for

**OBSERVATION.** Lines 9168 to 9170 say the `-p duet-engrave` run "also asserts that two writes of one
score give equal bytes, which is the deterministic-output property of section 3.6". That property
belongs to the `duet-score` canonical writer, which T2 writes. No Writes column names a determinism
test, and `-p duet-engrave` cannot run a `duet-score` test.

**CLAIM.** Rung two claims a machine-checked behaviour that no machine checks.

**ARGUMENT.** Rung two is the completion oracle for version one. A rung that names an unwritten test
is the same class as a recorded run that never ran.

**EVIDENCE.** architecture.md:9168-9170; section 3.6; the 13.2 Writes columns.

**WHAT SHOULD CHANGE.** Name the test, name the chunk that writes it, and put it in the correct
package command.

## WARNING 12. SM4's policy-file list omits the root `Cargo.toml`, which holds `[workspace.lints]`

**OBSERVATION.** SM4 (line 8540) lists the files that go to the Orchestrator: `scripts/dod.sh`,
`scripts/bootstrap.sh`, `deny.toml`, `.cargo/config.toml`, the workflows, and the skill file. The
root `Cargo.toml` is not on the list. M1, M2 and M3 write "root pins" (lines 8603 to 8605) and carry
no other policy file, so SM4 sends them to an engineer, and nothing limits the edit to
`[workspace.dependencies]`.

**CLAIM.** An engineer has an unbounded edit on the file that holds the lint policy.

**ARGUMENT.** CLAUDE.md states that an edit to `[workspace.lints]` made to get past the gate is a
defect that escalates. The plan gives three chunks write access to that file with no stated limit.
The cells also say "root pins", which is a subject and not a path, while line 8481 makes a named file
the unit of write scope.

**EVIDENCE.** architecture.md:8481, :8540, :8603-8605. `/Users/james/Developer/duet/CLAUDE.md`, the
lint-policy rule.

**WHAT SHOULD CHANGE.** Name `Cargo.toml` as a path in each cell and state that the edit is confined
to `[workspace.dependencies]`. Put `[workspace.lints]` on SM4's list.

## WARNING 13. The `C2 before C3` row breaks the table's own scope rule and states a wrong reason

**OBSERVATION.** Line 8951: "A link inside one line is SM6 and is not repeated here. This table holds
the cross-line links only." Line 8977 is an intra-line row: C2 and C3 are both chunks of line C. Its
reason reads "it allocates the ring TYPES that C2 declares". A type is not allocated. Line 8666 says
C2 "consumes the ends that `configure` hands it", and `configure` is C3 work, one phase later.

**CLAIM.** The row that closes C15-1's ordering half is in the wrong table and its reason does not
say the thing a reader needs.

**ARGUMENT.** C15-1 was a real finding and the substance is closed: I confirmed all six named sites
hold the mechanism, and the 1.6 arithmetic uses `ChainLayout::pairs` (B55 and B56 both compute from
it). The ordering row is the one weak part.

**EVIDENCE.** architecture.md:8666, :8951, :8977.

**WHAT SHOULD CHANGE.** State the reason as a compile fact: C3 names the ring types C2 declares, and
the ends C2 consumes are `rtrb::Producer` and `rtrb::Consumer`. Say why the row is in this table
although it is intra-line, or move it to SM6.


## WARNING 14. The conversion guard has twelve rules and the chunk that ports it is told eleven

**OBSERVATION.** Line 1087 lists twelve: "CG1 to CG8, with CG1b, CG2b, CG3b, and CG4b". The section
1.9 probe table at lines 1225 to 1236 holds twelve rows. Section 2.3 line 2156 says "Its contract has
**eleven** rules", and line 2310 says "**Eleven tests in `tools/xtask` cover it, one per rule
(DR5).**"

**CLAIM.** DR5 requires one probe per rule. Chunk M0 ports eleven and the guard has twelve, so one
rule reaches the real gate with no test.

**ARGUMENT.** CG1b is the rule this revision added, and section 2.3 is the section that was not
updated with it. The rule most likely to be dropped in the port is therefore the one Critical 2 is
about. A guard rule that no test covers is a rule nobody will notice is missing.

**EVIDENCE.** architecture.md:1087, :1225-1236, :2156, :2310;
adr/0001-time-kernel-representation.md:65-67, which still says "CG1 to CG8".

**WHAT SHOULD CHANGE.** Correct both counts to twelve and correct ADR 0001 decision 10.

## WARNING 15. `scripts/dod.sh` runs `shellcheck` and the specification never names the step

**OBSERVATION.** `/Users/james/Developer/duet/scripts/dod.sh` lines 88 to 90 run
`shellcheck -x -S warning` over four directories when the tool is installed, under `set -euo
pipefail`. Section 14 line 9021 lists "`bash -n` passes on every shell hook", and line 9028 lists the
steps as "`fmt`, `clippy`, `doc`, `nextest`, `doctest`, `deny`, `machete`, `sync-agents`, `bash -n`,
and `typos`". Neither names `shellcheck`. A grep of the whole document for the word returns nothing.

**CLAIM.** Chunk M0 rewrites `scripts/dod.sh` and `scripts/bootstrap.sh`, and chunk M7 writes
`audio-smoke.yml`, against a specification that does not know one of the gate's steps exists.

**EVIDENCE.** `/Users/james/Developer/duet/scripts/dod.sh:13,28,88-90`; architecture.md:9021,
:9028-9029, :8602, :8607.

**WHAT SHOULD CHANGE.** Add the step to both lists in section 14.

## WARNING 16. Rung one claims a gate step for `[lints] workspace = true` that no gate step performs

**OBSERVATION.** Line 9014: "Every crate is a workspace member and declares `[lints] workspace =
true` and a `description`." `scripts/dod.sh` reads no member manifest for a `[lints]` table.
`description` is covered, because `clippy::cargo` is denied and the group holds
`cargo_common_metadata`. `[lints] workspace = true` is covered by nothing.

**CLAIM.** The plan creates fifteen crates. A crate that omits the line sits outside the whole lint
policy and every gate stays green.

**ARGUMENT.** CLAUDE.md states the rule and says a crate without it "sits silently outside the lint
policy". Rung one asserts the gate checks it. The gate does not. This is a rung that reports a state
it did not measure.

**EVIDENCE.** architecture.md:9014; `/Users/james/Developer/duet/scripts/dod.sh:19-30,46-96`;
`/Users/james/Developer/duet/CLAUDE.md`, the crates section.

**WHAT SHOULD CHANGE.** Add the check to `scripts/dod.sh` and name the chunk that writes it, or
delete the claim from rung one.

## WARNING 17. `SmallVec<[Ticks; 12]>` is one element short of the range B43 tests

**OBSERVATION.** Line 883: "| B43 | divisors 2 to **13**, spans 1 to 7680 ticks | The tuplet sum test
range |". Line 2334: `pub fn split_tuplet(span: Ticks, parts: NonZeroU8) -> SmallVec<[Ticks; 12]>;`

**CLAIM.** A thirteen-part tuplet spills to the heap on every call, inside the range the test covers.

**ARGUMENT.** The inline capacity is the whole reason to use `SmallVec`. A bound one below the tested
maximum guarantees an allocation at the top of the range rather than avoiding one.

**EVIDENCE.** architecture.md:883, :2334; adr/0001-time-kernel-representation.md:61-62.

**WHAT SHOULD CHANGE.** Raise the inline capacity to 13, and state the B id it comes from.

## WARNING 18. Two research contradictions and one pin contradiction that no document records

**OBSERVATION.**

1. `research/crate-survey.md:10` says the `pipewire` stream path has `unsafe` buffer handling and
   records "**Excluded: no unsafe.**" `research/linux-macos-platform.md:19` says the same path is
   safe. `adr/0004-backend-and-threading-contract.md:301` says "**both sources refute**" the unsafe
   claim. One source states it.
2. Line 1636 pins `tokio-util 0.7.18`. `research/crate-survey.md:122` records `0.7.19`.
3. `research/crate-survey.md:117` records that `blake3` "on x86 ... needs a C compiler unless the
   `pure` feature is on". Appendix B.3 line 11481 states no feature, and line 11520 says every pin
   the B.5 table omits takes its default feature set. `adr/0003-history-model.md:74` rejects `git2`
   because "It links a C build."

**CLAIM.** The C15-13 closure rests on a "both sources" claim that one source contradicts, and the
plan rejects one crate for a property it then accepts in another.

**EVIDENCE.** As cited. architecture.md:1636, :11481, :11520;
research/crate-survey.md:10, :117, :122; research/linux-macos-platform.md:19;
adr/0003-history-model.md:74; adr/0004-backend-and-threading-contract.md:301.

**WHAT SHOULD CHANGE.** Correct `crate-survey.md` or correct the ADR, and say which one is now
right. Correct the `tokio-util` pin. Give `blake3` `features = ["pure"]` in B.5 or state why the C
build is accepted here and not for `git2`.

---

# CONCERN

## CONCERN 1. Three of this revision's closures are prose only, and I confirmed a regression at each is invisible

I planted the reverse of three revision-16 fixes and ran the placement guard:

```
G1-drop-cap        (delete MeterReading::cap)                       EXIT=0
G2-drop-monitor    (rename FrameDemand::any_monitoring)             EXIT=0
G4-path-untessellate (PathCache back to Arc<PathPlacement>)         EXIT=0
G3-drop-B112       (unname a budget id a section cites)             EXIT=1  caught by the member rule
```

Appendix C's own standard is that the mechanism is visible in the section's text, so prose is
allowed. I record it because three of the four fixes this revision made can be undone with no guard
reacting, and the document's direction of travel is toward mechanised closure. Name the guard you
would add, or say in the row that the closure is prose.

## CONCERN 2. The environment seam of the conversion guard is undocumented in the specification

`CONVERSION_MEMBER_FLOOR` and `CONVERSION_FILE_FLOOR` appear in `tools/conversion_check.py` and in
`tools/probe_conversion.py` and in no line of architecture.md. Chunk M0 ports the prototype to
`tools/xtask/src/check_conversions.rs` against a specification that does not mention them. State the
seam in section 2.3, or remove it and give the probes a different mechanism. See Critical 2, of which
this is the smaller half.

## CONCERN 3. `Arc<Path<Pixels>>` buys nothing at the paint site

`gpui-pre-0.3.5/src/window.rs:4457` declares `pub fn paint_path(&mut self, mut path: Path<Pixels>,
...)`. The path is taken by value. A cache that holds `Arc<Path<Pixels>>` must clone the whole vertex
vector at every paint, which is one heap allocation per painted path per frame. The tessellation
saving is real and the fix is directionally right. Section 10.5 states no such cost. State it, or say
why the `Arc` is there.

## CONCERN 4. One pinned-source citation reads against the crate's own doc comment

Line 6800 says the `on_next_frame` callback "runs at the start of a frame, before the render pass of
the same frame". The implementation agrees: `gpui-pre-0.3.5/src/window.rs:1765` and `:1769` run the
callbacks before `:1782-1791` draw and present. The crate's own doc comment at `window.rs:2585` says
the opposite. The document cites `:2586` alone, so a reader who checks it finds the contradiction and
not the proof. Cite `window.rs:1765` beside it.

## CONCERN 5. `Cargo.lock` has up to six writers in one phase, and three phases have no manifest chunk to serialize them

Per-phase writer counts, phases 0 to 12: 2, 4, 6, 6, 6, 4, 4, 5, 3, 0, 0, 0, 0. Phases 4, 5, 8 and
later have no manifest chunk. Line 8926 and SM5 rule 4 declare the overlap and give a resolution, so
the risk is accepted rather than hidden. I record the counts so the size is visible.

## CONCERN 6. Five smaller plan-graph defects

- A4's write scope (line 8648) holds a bare `benches/`. Read as written, that is the repository root.
  Every other bench path is written in full.
- M0 and M7 re-pin crates the root `Cargo.toml` already holds (`serde`, `serde_json`, `thiserror`,
  `clap`). Under SM0 an engineer who verifies the write scope first must report the discrepancy and
  stop.
- `gpui-kit` and `tracing-subscriber` appear in the 1.2 row for `duet` (line 170) and in no pin list
  of 13.1. Both exist today, so the plan works; SM1 claims the manifest chunk pins every third-party
  dependency the phase needs.
- Rung one (line 9018) writes `cargo nextest run` where `scripts/dod.sh` runs `cargo nextest run
  --workspace --locked`, and the bullet list omits the `typos` step that line 9032 names.
- Lines 8898 and 8899 follow the SHOULD table with no blank line. Under GitHub Flavored Markdown both
  sentences render as one-cell rows inside that table.

## CONCERN 7. Several load-bearing negative claims rest on crates I cannot open

`cpal`, `gix`, `hound`, `midir`, `symphonia`, `rmcp`, `blake3`, `flacenc`, `bwavfile` and
`notify` 8.2.0 are not vendored in the local registry. The document makes negative claims about four
of them that decide a design: "cpal has no duplex stream in 0.18.2" (line 4031, ADR 0004:13), "gix
exposes no repack and no garbage collection" (line 3777), "hound has no RF64 support" (line 223),
"midir reports no connect and no disconnect" (section 8.1). A negative claim about a crate no
reviewer on this machine can open is the weakest evidence in the document. Record the version and the
file each was checked against, or vendor the crates before the next review.

## CONCERN 8. Budget `Used by` cells count a closure appendix as a use site

B111 and B112 (lines 950, 951) give `Used by` as "7.3, C.19". C.19 is the closure appendix. A budget
whose only two use sites are one section and the record of the finding that created it has one real
use site. The same pattern runs through B95's cell. It weakens PG30, whose whole job is to hold the
`Used by` column and the citations to one set.


## CONCERN 9. Three smaller policy gaps with no owner

- `clippy.toml` is on no chunk's write scope and on no SM4 policy list. `pedantic` is denied, so
  `clippy::doc_markdown` fires on every unbackticked proper noun. The plan's vocabulary adds
  `PipeWire`, `CoreAudio`, `CoreMIDI`, `MusicXML`, `SMuFL`, `Bravura`, `XWayland`, `Wayland`, `APFS`
  and `RF64`. Either each doc line backticks them, which no rule states, or `doc-valid-idents` grows,
  which no chunk may edit.
- The `deny.toml` advisory ignore list is scoped entirely to `gpui-kit` 0.6.4 transitives. The plan
  adds about thirty crates, one of which (`midly`) the survey records as unmaintained since 2023. No
  chunk owns a new advisory ignore, and `cargo deny check` fails the gate on a new RUSTSEC id.
- Appendix B.5 line 11526 says the `clap` pin is "Already in the root manifest", and chunk M7 is
  told to pin it again at line 8607. `tracing` has the same shape at line 8605.

## CONCERN 10. Section 5.12 does not hold the whole timeout table that ADR 0004 names

`adr/0004:219-220` says "**Every cross-boundary wait has a timeout**, and specification section 5.12
holds the table." The 5.12 table (lines 5768 to 5781) names B15 to B23, B27, B95 and B110. Appendix
B.5 (lines 11542 to 11555) adds B24, B25, B26, B85 and B108. Five timeout ids have a mechanism row
and no row in the one table the ADR names. The 5.12 table also carries B110, which bounds an
allocation and not a wait.

## CONCERN 11. The design contract's three meter zones have no B row

`design-contract.md:758-761` gives the level meter three colour zones at -12 dB and -3 dB. B106
(line 946) carries the scale and no zone threshold. Line 829 states that every number in the
specification is a row in 1.6. `MeterLayer` and `LevelMeter` must paint the zones and neither
threshold is declared.

---

# What is genuinely closed, and what passed

I record this so the next revision does not re-open settled work.

**Closed and closed well (5 of the 13 numbered rows).** C15-1, the ring-allocation owner: all six
named sites hold the mechanism, `ChainLayout::pairs` counts every allocated pair, and the 1.6
arithmetic computes B55 and B56 from it. C15-9, the generation skew window: sections 5.6 and 12.4
and test 14 agree on the window, its cost and its end. C15-10, the fourth frame-pump start term:
field, both 10.2 rows, and the test half all exist. C15-12, the split `CoreHost` doc comments: no
undocumented declaration remains in section 15.16. C15-13, the `pipewire` rejection reason: the
citation to Appendix B.2 is correct and the ADR row states the reason the platform research
supports.

**Guard work that holds.** The placement guard is green on the frozen document and all 49 documented
placement probes are red on plant. All conversion probes pass. PG26d does catch the one-for-one swap
and the deleted marker, which is the shape guard hole 1 named. PG30 caught my own planted defect
(G3). The topological sort of the 55-chunk graph is acyclic, every phase width matches its row, and
every chunk sits in exactly one phase.

**Cross-document checks that pass.** Every licence the plan needs is in `deny.toml` except
`Unlicense`, and M0 owns that edit. No pin trips the source or ban policy. Every lint Appendix B.1
names is denied in the live policy. No Rust block in the document uses `unwrap`, `expect`, `panic!`,
`todo!`, `unimplemented!`, `println!`, `#[allow]`, an `as` cast, or slice indexing. Binaries return
`ExitCode` with a locked handle. The 64 MUST stories and the 64 MUST rows are the same set with no
phantom id. The existing `crates/duet/src` is fully accounted for. The B-number arithmetic holds
where it can be checked.

---

# Reactive assessment

| Property | Verdict | Why |
|---|---|---|
| Responsive | **FAIL** | B95, the bound on how long a topology change waits before the engine faults, carries two values that differ by a factor of nineteen (Critical 5). The refusal path of section 5.5 has no stated outcome and the `pending_configure` re-send has no cap (Critical 4). `MixView` has no owner for its path build task, so every Mix waveform would miss forever (Warning 5). |
| Resilient | **FAIL** | `ConfigError::NothingToHandOff` is named in a `# Errors` contract and declared nowhere (Critical 3). `PoolHandoffBusy` has no route to a user message (Critical 13). `GatewayError` duplicates four `ConfigError` arms against ADR 0004 clause 12a (Critical 16). The `try_cleanup` refusal is diagnosed on one of its two causes (Warning 4). The document's own third guard is red and every PG25 probe is unexercised (Critical 1). |
| Elastic | **PARTIAL** | The ring-allocation ownership of C15-1 is genuinely closed at all six sites and the memory arithmetic uses `ChainLayout::pairs`. The uncapped `pending_configure` re-send is the one unbounded producer left (Critical 4). |
| Message Driven | **PARTIAL** | The engine seam is message driven: `ConfigureCommand`, `EngineEvent`, the B105 ring and the triple buffers. Three 13.4 link reasons describe a direct call across a crate edge the graph forbids (Warning 6), chunk I3 puts a GPUI `Window` inside `duet-core` (Warning 9), and `MasterView` is told to read a sibling entity's cache it has no handle to (Critical 15). |

---

# Verdict

**NOT READY.** The engineering is not sound enough to author a plan from. Seventeen Criticals,
eighteen Warnings, eleven Concerns.

The single biggest risk is Critical 1: the document's own third guard exits 1 on this revision while
section 1.9 records `ROSTER CLIPPY: clean`, so `ROSTER SIZES: 419 measured` and every size it
underwrites are unverified, and the entire PG25 probe set did not execute. That is the exact defect
class (critic CR-13) that the roster harness was written to end, recurring in the revision that
claims to have ended it. Close behind it is Critical 2: I reproduced the vacuous green that CG1b was
written to stop, at the production default floors, in a workspace the size section 13 builds. A guard
that reports success it did not earn is worse than no guard.

The revision-15 closure record is not trustworthy as written. Of the thirteen numbered rows in
Appendix C.19, three are not closed at all (C15-5, C15-6, C15-7), two are partly closed with a new
defect introduced by the fix (C15-2, C15-11), one leaves half its hole open (C15-8), and both guard
holes remain reachable — one fully (guard hole 2), one by an adjacent shape the new rule does not
state it cannot see (guard hole 1). Five rows are genuinely closed and closed well. The failure
pattern is uniform: prose was written at the named site, and the type, the budget row, or the
remaining citations were not changed with it. The appendix's own standard is "the mechanism is
visible in that section's text or in its types", and in six rows it is visible in the text and absent
from the types.

The weakest Reactive property is **Resilient**. Four failure surfaces this revision touched reach a
caller that cannot handle them: an undeclared variant in a `# Errors` list, an engine refusal with no
user message, two enums that duplicate four refusal variants with no declared conversion, and a
third guard whose red run nobody saw.

---

# Blocking list

Every Critical blocks plan authoring. Fix in this order.

1. **C1** — Add `cap: Finite::ZERO` to `MeterReading::SILENT` (line 6228). Run `roster_compile.sh`
   and `probe_roster.sh` and paste the real output into section 1.9 lines 1308 to 1320.
2. **C2** — Replace the CG1b count floor with a derived member set. Remove or gate the
   `CONVERSION_MEMBER_FLOOR` and `CONVERSION_FILE_FLOOR` seam. State the mechanism in section 2.3.
3. **C3** — Declare `ConfigError::NothingToHandOff` or delete its two citations.
4. **C14** — Declare `finite_to_f32_saturating` in section 2.3, or delete the B.1 row and give
   `LogicalPx::to_f32` a real body. Correct the six-against-seven and the nineteen-against-twenty.
5. **C4** — Write the third outcome into 5.5 step 5, cap the `pending_configure` re-send, and make
   section 15.14 agree.
6. **C5** — Change lines 935, 5779, 10324 and 11330 from B7 to B108. Resolve the B.5 and C.15 clock
   disagreement.
7. **C6** — Correct the nine framework-path sites. Add `Theme` to the name-map block.
8. **C13 and C16** — Give `GatewayError` one declared route from `ConfigError`, or move the
   `PoolHandoffBusy` row to the fault table. Then make ADR 0004 clause 12a true.
9. **C7** — Give some declaration ownership of the `AudioBackend`.
10. **C8** — Resolve the `MeterReading::cap` publication contradiction, and give `MeterReader` the
    B111 deadline state.
11. **C15** — Give `MasterView` a reachable path cache, and correct ADR 0006 line 55.
12. **C9** — Mark `end_to_end` `#[ignore]` until phase 9, or split it.
13. **C10** — Give I4 its manifest and lock scope, and move the `NotYetImplemented` deletion.
14. **C11** — Add `chain/configure.rs` to C1's stub list.
15. **C12** — Add the `T2 before T3`, `T4 before X1` and `J1 before K1` links and re-phase.
16. **C17** — Give one chunk the `dod` job of `ci.yml`, and name it under SM4.

The eighteen Warnings must be closed before FINISH. A Warning is not deferrable. The eleven Concerns
are either fixed in this changeset or filed as tracked follow-ups under `roadmap/duet-v1/`.

**Re-review condition.** Do not send revision 17 for review until `probe_roster.sh` exits 0 and a
run of the CG1b shape in a five-member workspace exits 2. Both are one command each. A revision that
cannot produce those two exits has not closed the two findings that matter most.
