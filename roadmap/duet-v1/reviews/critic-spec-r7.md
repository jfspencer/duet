# Specification review, revision 7: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is the seventh pass. Revisions 1 to 6 each returned NOT READY.

Sources read in full: `roadmap/duet-v1/architecture.md` (5987 lines), the six ADRs,
`research/linux-macos-platform.md`, `research/crate-survey.md`, `product-requirements.md` section 8,
`design-contract.md` sections 0, 1, 3.1, 3.3, 4.4 and the `StageCurve` appendix, `CLAUDE.md`, the
root `Cargo.toml`, `.cargo/config.toml`, `deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/` directory. I ran both guards on the real document and the real tree. I ran all
twenty-two recorded probes. I added six probes of my own to the placement guard and six to the
conversion guard. I ran eleven mechanical passes over the document and the plan graph. I wrote no
repository file. I ran no other git command.

**Revision 7 is the strongest revision so far, and DR5 works.** Every one of the twenty-two recorded
probes reproduced exactly, including the exit code and the message text. The two revision-6 holes
that I found by hand are closed and proved. The plan graph passes every mechanical test. The
budget table is arithmetically correct in all but one row.

It does not close. One Critical and ten Warnings block it. The Critical is the same class that
blocked revisions 4, 5, and 6: a `Copy` derive over a field whose shape the document never states.
DR5 did not catch it, because the rule that exists to catch it is blind here, and the rule that
exists to make that blindness visible does not count this direction.

---

## 1. Closure check

### 1.1 The eighteen revision-6 findings

| Id | Finding | State | Section and reason |
|---|---|---|---|
| N1 | `MidiRecord` and `NoteEntry` derive `Copy` over `Box<str>` | **CLOSED** | 8.1 splits `PortSlot(u16)` from `MidiPortId(Box<str>)`. 8.3 declares both records over `PortSlot` and states the 16-byte arithmetic. PG10 exists. PP10 goes red with the exact message. |
| N2 | The 1.2 dependency column omits serde and smallvec | **PARTIAL** | PG12 exists and PP12 goes red. The column is derived for 13 of 76 pairs. The 1.2 sentence "Three rows carry a crate that no declaration proves" is false. See Q12. |
| N3 | Four commands select tests that no chunk writes | **CLOSED** | SM7, the section 14 selected-test table, and PG16. C3 writes `tests/soak.rs`, C4 writes `tests/alignment.rs`, T1 writes `tests/proptest_large.rs`. 13.4 carries all three links. |
| N4 | The guard tests run inside the gate | **CLOSED** | 1.5 and 2.3 both state the inline-fixture rule. No guard test opens a file under `roadmap/`. |
| N5 | Two rules have no commissioned test | **CLOSED** | DR5 and the 1.9 one-to-one table. Sixteen rules and sixteen probes; six rules and six probes. I ran all twenty-two. |
| N6 | The audio thread is the dropper on two triple buffers | **CLOSED** | TH9, B86, 5.9 declares both types `Copy` with fixed-size fields, PG13, PP13 red. |
| N7 | `check-conversions` rejects qualified path syntax | **CLOSED** | CG5 and CP5. The stated extra check gives two findings, one per real cast. A new hole opens beside it. See Q3. |
| N8 | PG7 misses a path-qualified framework type | **CLOSED** | PG7 runs before `strip_paths` and matches the last segment. PP7 red on `gpui_kit::Px`. |
| N9 | The design contract keeps a two-part path-cache key | **CLOSED** | Contract 3.3, ADR 0006 decision 6, architecture 10.5, and Appendix A all carry the four parts. |
| N10 | ADR 0005 decision 2a contradicts VR1 | **CLOSED** | Decision 2a is rewritten in place and names `Mode` and `MidiPortMap`. |
| N11 | `RingBudgetExceeded` names two enums | **CLOSED** | `ConfigError::RingBudgetExceeded` in 5.5, B57, and ADR 0004 decisions 11 and the per-state table. |
| N12 | Three sentences describe work already done | **CLOSED** | The header, 10.2, and 10.3 each state a present fact. |
| N13 | Guard rule 7 covers 85 of 265 candidates | **PARTIAL** | The denominator is stated and correct: I measured 354 placed, 146 declared, 208 in the table alone. PG14 counts one direction only. See Q2. |
| N14 | The `Declared in` column is stale | **CLOSED** | PG15 and PP15 red. |
| N15 | Only `cpal` names a feature set | **PARTIAL** | B.5 exists and states the default-set rule. The `tokio` row omits `time`, and three timeouts need it. See Q9. |
| N16 | `view.json` is tracked and holds the scroll offset | **CLOSED** | `state/view.json` in 4.1, 4.2, 9.4, 10.2, and Appendix A. `.gitignore` covers `state/`. |
| N17 | The `plan-lint` deletion has no chunk | **CLOSED** | "The job stays. This plan does not delete it." |
| N18 | Section 14 gives `plan-lint` two powers | **CLOSED** | One power row. Rung one and 11.6 both say "blocks a merge by review". |

**Count: 15 CLOSED, 3 PARTIAL, 0 OPEN.**

### 1.2 The two uncaught revision-6 probes

| Probe | Shape | State | Evidence |
|---|---|---|---|
| 12 | A `Copy` derive over a `Box<str>` newtype | **CLOSED** | PP10: `NOT COPY: duet-command::MidiRecord derives Copy over MidiPortId`, exit 1 |
| 13 | A path-qualified framework type | **CLOSED** | PP7: `FRAMEWORK: duet-command::ModeView holds Px`, exit 1 |

### 1.3 The baseline run, as I measured it

```
DOCUMENT:        roadmap/duet-v1/architecture.md
FRAMEWORK NAMES: 51    NAME MAP: 12
CANDIDATE TYPES: 268   DECLARED: 146
PLACED:          268
UNPLACED:        0     DUPLICATED:   0
MISCLAIMED:      0     FRAMEWORK MISUSE: 0
COPY MISSING:    0     COPY IMPOSSIBLE:  0
UNKNOWN:         23
EDGES PARSED:    58    EDGE CLAIMS BAD: 0
DEP ROWS:        16    DEP MISSING: 0
SNAPSHOTS:       2     SNAPSHOT BAD: 0
REGISTER BAD:    0
TESTS SELECTED:  5     TEST ROWS BAD: 0
EXIT=0
```

The conversion guard on the real tree gives `FILES: 6   FINDINGS: 0`, exit 0. Both runs match
section 1.9 exactly.

### 1.4 The twenty-two recorded probes

Every probe reproduced the recorded exit code and the recorded message.

| Probe | Result | Probe | Result |
|---|---|---|---|
| PP1 | exit 2, usage line | PP9 | exit 1, `COPY: duet-score::Spanner needs Copy or an #[expect]` |
| PP2 | exit 2, fail-closed line | PP10 | exit 1, `NOT COPY: duet-command::MidiRecord derives Copy over MidiPortId` |
| PP3 | exit 1, `CANDIDATE TYPES: 0   DECLARED: 0` | PP11 | exit 1, `EDGE MISS: duet-time -> duet-project` |
| PP4 | exit 1, `UNPLACED: ProbeUnplaced` | PP12 | exit 1, `DEP MISS: duet-score uses serde through Clipboard` |
| PP5 | exit 1, `DUPLICATED: Knot` | PP13 | exit 1, `SNAPSHOT: MeterSnapshot: the declaration derives no Copy` |
| PP6 | exit 1, `MISCLAIMED: Pixels claimed by duet-command` | PP14 | exit 0, `UNKNOWN: 23` becomes `UNKNOWN: 24` |
| PP7 | exit 1, `FRAMEWORK: duet-command::ModeView holds Px` | PP15 | exit 1, `REGISTER: duet-command declares MidiRecord in 8.3` |
| PP8 | exit 1, `UNPLACED: Reverb` | PP16 | exit 1, `TEST MISS: soak: no row in the selected-test table` |
| CP1 | exit 1, one `CAST` in `benches/` | CP4 | exit 0, `FINDINGS: 0` |
| CP2 | exit 0, `FINDINGS: 0` | CP5 | exit 0, `FINDINGS: 0` |
| CP3 | exit 1, one `CAST` in `src/lib.rs` | CP6 | exit 1, `FINDINGS: 2`, one per form |

The stated extra check for CG5 also holds: one qualified path plus two real casts gives
`FINDINGS: 2`.

### 1.5 My own twelve probes

| Probe | Shape | Result | Caught |
|---|---|---|---|
| MINE-P1 | An edge claim spelled `depends on` | exit 0 | No (Q13) |
| MINE-P2 | A `Copy` derive over the undeclared type `MidiPortInfo` | exit 0, `UNKNOWN: 23` | **No (Q2)** |
| MINE-P3 | A type alias to `gpui_kit::Px` inside `duet-command` | exit 0 | No (Q14) |
| MINE-P4 | `gpui_kit::Px` in a `duet-session` method signature | exit 0 | No (Q14) |
| MINE-P5 | A bare `Px` in a `duet-session` method signature | exit 0, candidates 269 | No (Q14) |
| MINE-P6 | A duplicate `Knot` under an Appendix heading | exit 1, `DUPLICATED: Knot` | Yes |
| MINE-P7 | Prose tokens stand in for the dependency cell | exit 0 | No (Q15) |
| MINE-P8 | `serde` removed from the `duet-midi` row | exit 1, `DEP MISS` | Yes |
| MINE-C1 | A cast between `<` and `>` on one line | exit 0 | **No (Q3)** |
| MINE-C2 | `n < xs.len() as u64 && a > b` | exit 0 | **No (Q3)** |
| MINE-C3 | A nested raw string `r##"say "as" here"##` | exit 1 (false failure) | No (Q16) |
| MINE-C4 | The exempt file under a workspace root reached through a symlink | exit 1 (false failure) | No (Q17) |
| MINE-C5 | A workspace that `cargo metadata` refuses | exit 1, uncaught traceback | Fail-closed (Q17) |
| MINE-C6 | The exempt file under a canonical root | exit 0 | Correct |

### 1.6 The eleven mechanical passes

1. **A type declared twice: none.** `DUPLICATED: 0` over 268 candidates, and PP5 and MINE-P6 both
   go red.
2. **A budget number outside 1.6: none.** Every hit is a type size or a format constant, and DR3
   exempts both. Revision 6 had four hits; revision 7 has none that DR3 does not name.
3. **A B id with no row: none.** All 86 ids resolve. B42, B49, B50, and B62 are cited inside 1.6 or
   inside an ADR only.
4. **A rule id that no text mentions: none.** PG1 to PG16, PP1 to PP16, CG1 to CG6, CP1 to CP6, VR1
   to VR6, TH1 to TH9, SM0 to SM7, DR1 to DR5, and PL1 to PL3 all appear.
5. **A chunk absent from a table: none.** 55 chunks appear in 13.1 or 13.2 and in 13.3, with the
   same phase in both.
6. **A same-phase link: one, and it is the stated exception.** The link is M0 before T1.
7. **A backward link: none.** I expanded every cross-line link in 13.4.
8. **Two chunks of one line in one phase: none.** T2 and T3 share phase 1, and 13.1 states that the
   trunk is four lines of one chunk each.
9. **A MUST story with no row: none.** Product requirements section 8 holds 64 MUST rows, and the
   13.2 table holds the same 64. The two sets match exactly.
10. **A filtered nextest command with no `--no-tests=fail`: none.** I checked every command in the
    document.
11. **A selected test with no row or no chunk: none.** All five names resolve, and each chunk lands
    at or before the phase of the command that selects it.

### 1.7 The budget arithmetic

I recomputed every derived row. B7, B9, B10, B49, B50, B52, B54, B55, B56, B66, and B72 all hold.
B73 holds against the lower end of B82 and fails against the upper end. See Q4.

---

## 2. New findings in revision 7

### CRITICAL: Q1. `Calibration` derives `Copy` over `DeviceKey`, and no section states the shape of `DeviceKey`

**OBSERVATION.** Section 5.4 declares the calibration record.

```
architecture.md:2345   #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
architecture.md:2346   pub struct Calibration {
architecture.md:2347       device: DeviceKey,
```

No Rust block in the document declares `DeviceKey`. The name appears five times: in the PL2
placement table, in the 1.5 register, in `InputSelection::Device`, in `InputSelection::Pair`, in
`Calibration`, and as `Verb::TransportCalibrate { device: Box<DeviceKey> }`.

**CLAIM.** The `Copy` derive on `Calibration` may not compile, and no rule lets a chunk decide. This
is finding W1 of revision 6 in a second place.

**ARGUMENT.** ID1 assigns a shape to a name in section 1.5 that ends in `Id`, `Name`, or `Text`.
`DeviceKey` ends in none of the three, so ID1 does not reach it, and no template gives it a field
type or a derive list. Three facts point at a string. A device key names a hardware device, section
5.4 stores the calibration "keyed by device, sample rate, and block size", and `Verb` boxes the
value. A small `Copy` value needs no box; section 9.1 boxes a payload so that
`variant_size_differences` stays satisfied. If `DeviceKey` wraps a `Box<str>`, then
`#[derive(Copy)]` on `Calibration` is a hard error, exactly as `rustc` refused `MidiRecord` in
revision 6.

A second rule breaks at the same field. A map key needs `Ord` or `Hash` and `Eq`. VR1 names six
groups, and `DeviceKey` is in none of them. Row 1 covers "Every identifier newtype, and
`SourceHash`", and nothing states that `DeviceKey` is an identifier newtype under ID1.

Chunk T3 must declare `Calibration` and `DeviceKey` in one commit. The chunk has no shape to write,
and B.1 states that a suppression outside its list returns to the Architect.

**EVIDENCE.** `architecture.md:226`, `:294`, `:1432`, `:1433`, `:2347`, `:3777`. The guard reports
nothing, because its verdict for `DeviceKey` is unknown rather than false. I proved the blindness
with MINE-P2 below.

**WHAT SHOULD CHANGE.** Declare `DeviceKey` in section 5.4 or 6.1 with its field type and its derive
list. If the key holds a device name, remove `Copy` from `Calibration` and add the row to B.1 with a
VR5 reason. Then add a rule: every type that a `Copy` derive reaches is declared, or ID1 decides it.
Give that rule a probe.

### WARNING: Q2. PG14 does not count an undecidable field under a `Copy` derive

**OBSERVATION.** PG14 reads: "A declaration whose Copy-ness the guard cannot decide is counted and
printed. The `UNKNOWN` counter is part of the run, so the blind spot of PG9 and PG10 is a number a
reader sees rather than a limitation a reader must remember" (`architecture.md:386`).

`copy_audit` builds the `unknown` list inside the branch that runs when the declaration does **not**
derive `Copy` (`placement_check.py:411` to `:422`). A declaration that derives `Copy` returns early,
and the loop reports a field only when the verdict is exactly `False`.

**CLAIM.** The counter covers the PG9 direction and not the PG10 direction. PG10 is the rule that
closes the Critical of revision 6, and its blind spot is invisible.

**ARGUMENT.** PG10 fails on a `Copy` derive over a field the document **proves** is not `Copy`. A
field whose type no block declares, and which ID1 does not decide, gives no proof in either
direction. PG14 exists so that a reader sees how often that happens. It does not count this case, so
the printed `UNKNOWN: 23` understates the blind spot. I measured the real number: **25 declarations
that derive `Copy` hold at least one field with an undecidable verdict**, and the counter reports
none of them.

The rule is therefore narrower than its own sentence. DR5 exists to end that class, and this is the
revision that introduced DR5.

**EVIDENCE.** MINE-P2: I replaced `port: PortSlot` with `port: MidiPortInfo` in `MidiRecord`.
`MidiPortInfo` is in the 1.5 register and no block declares it. The guard printed `UNKNOWN: 23` and
exited 0. The twenty-five live instances include `duet-session::Calibration` over `DeviceKey`,
`duet-engine::TransportSnapshot` over `FrameCount` and `Generation`, and
`duet-command::MidiRecord` over `MidiMessage` and `SampleClock`.

**WHAT SHOULD CHANGE.** Count an undecidable field under a `Copy` derive in the same `UNKNOWN`
counter, and print the type and the field. Change PP14 so that it plants the defect on a
`Copy`-deriving type, because the present probe plants it on `UndoStack`, which derives no `Copy`.

### WARNING: Q3. A real cast passes `check-conversions` between a less-than and a greater-than

**OBSERVATION.** CG5 removes the span from a `<` to its matching `>` whenever that span carries a
whole-word `as` token. `drop_qualified_paths` blanks the whole span before CG3 runs
(`conversion_check.py:55` to `:85`).

**CLAIM.** A comparison pair on one line is such a span. CG3 therefore accepts a cast that sits
between the two operators, and CG3 is the rule that section 2.3 calls the point of the guard.

**ARGUMENT.** The scanner starts at any `<` whose previous character is not `-`, `=`, or `<`. It
walks forward and stops at the first `>` at depth zero, or at a `;`, a `{`, or a `}`. In
`n < xs.len() as u64 && a > b` the first `>` at depth zero is the comparison operator. The span
carries `as`, so the guard blanks the whole span, and the cast disappears before CG3 reads the text.

Section 2.3 states the rule as "The cast itself, not only its suppression, is the thing the rule
forbids". CP3 proves only the simplest shape, `let n = x as u32;`, on its own line.

**The gate as a whole does not fail open, and I say so plainly.** The root `Cargo.toml` sets
`as_conversions = "deny"` in `[workspace.lints.clippy]` (line 109), and `scripts/dod.sh` runs
`cargo clippy --workspace --all-targets --locked -- -D warnings`. Clippy rejects the same cast. The
unique value of the guard over clippy is CG6, which bans a suppression outside one file. CG3
duplicates a lint that already holds. The defect is a false claim about CG3, not an open gate.

**EVIDENCE.** MINE-C1 and MINE-C2, each in a throwaway cargo workspace:
`pub fn f(n: u64, xs: &[u8], a: u64, b: u64) -> bool { n < xs.len() as u64 && a > b }` gives
`FILES: 1   FINDINGS: 0` and exit 0.

**WHAT SHOULD CHANGE.** Require the span to start at a `<` that follows a type-shaped token or a
`<` that opens a qualified path, and require the `as` to sit at the span's own depth with a
type-shaped token on each side. Add a probe that plants the comparison shape and asserts one
finding. Then state that clippy is the backstop, so a reader knows the true coverage.

### WARNING: Q4. The default calibration lands a take late at the top of B82

**OBSERVATION.** B72 is 864 frames. B82 is "500 to 900 frames", the measured round trip of a
consumer interface at 128 frames and 48 kHz. Section 5.4 reads: "A measured consumer interface
returns the round trip B82, so the default is never low. B73 gives the maximum error, and every
uncalibrated take lands early, never late."

**CLAIM.** 864 is inside B82 and below its upper bound. At 900 frames the default is low by 36
frames, and the take lands late. The claim "never late" is false against the document's own range.

**ARGUMENT.** `AlignChoice::ExistingMaterial` shifts a take earlier by the offset. The offset is
`LatencyReport::capture` plus `LatencyReport::playback` plus the monitor chain latency, and an
uncalibrated device uses B72. If the true round trip is larger than the applied offset, the shift is
too small and the take sits late. B71, the alignment tolerance, is 2 frames at 48 kHz. An error of
36 frames is 0.75 ms and is eighteen times the tolerance.

B73 states one side of the error only: "up to 364 frames, or 7.6 ms, always early". 364 is
864 minus 500. The other side, 900 minus 864, has no row and no sentence.

**EVIDENCE.** `architecture.md:502` (B72), `:503` (B73), `:512` (B82), `:501` (B71), `:2406` to
`:2409`.

**WHAT SHOULD CHANGE.** Raise B72 above the top of B82, or state both signs of the error and give
B73 a second value. The "one-signed error" sentence is the reason the default is acceptable, so the
reason must match the range.

### WARNING: Q5. No rule bounds the strip count against B86, and the audio thread has no answer

**OBSERVATION.** `MeterSnapshot` holds `readings: [MeterReading; MAX_STRIPS]`, and `MAX_STRIPS` is
B86, which is 48. The B86 row budgets 32 track strips, 8 part buses, one reverb bus, one delay bus,
one monitor bus, the master, and four spare.

`SessionCommand::BusAdd { name, role }` and `Verb::MixAddBus` accept a bus with no count limit. B1
bounds parts, tracks, and bars, and no row bounds strips or buses.

**CLAIM.** The mix document is an unbounded producer against a fixed-size consumer buffer, and no
refusal exists.

**ARGUMENT.** Section 5.5 states that "A vocal project uses send-based reverb on two to four buses,
which PR M-04 states". B86's own arithmetic budgets one reverb bus. Four reverb buses take the count
to 47, and one more user bus passes 48.

`ConfigError` carries `ChannelMismatch`, `PoolExhausted`, and `RingBudgetExceeded`. None of them
covers a strip count, and the 12.4 refusal table carries no row. `clippy::indexing_slicing` is
denied, so the audio thread writes through `get_mut` under house form 4 of section 5.11. The 49th
strip's meter reads zero for the whole session, and no fault reaches the user.

Every other bound in this design carries a refusal and a message. `PoolExhausted` and
`RingBudgetExceeded` both name a user action. B86 names none.

**EVIDENCE.** `architecture.md:516` (B86), `:552`, `:2891`, `:1422`, `:3813`, `:2580` to `:2586`.

**WHAT SHOULD CHANGE.** Add `ConfigError::StripLimitExceeded { requested }`, map it to a
`GatewayError` variant, and give `fault_text.rs` a row. Refuse `BusAdd` off the audio thread, as
`PoolExhausted` does. Then state the worst-case strip count that B86 must cover, with B47 and B48 in
the arithmetic.

### WARNING: Q6. The MIDI presence path crosses two thread boundaries with no mechanism and no bound

**OBSERVATION.** Section 5.8 is titled "Which mechanism crosses each boundary" and holds three
tables. `HotplugSink`, `HotplugSubscription`, `MidiSink`, `HotplugEvent`, and `MidiEvent` appear in
no row of any of the three. No Rust block declares any of the five.

Section 8.1 states "The map lives on the MIDI thread". Section 8.3 states "`duet-core` turns a
`PortSlot` back into a name through `MidiPortMap::identity` when a user-facing message needs one".

**CLAIM.** Two threads reach one `MidiPortMap`, and no section names the mechanism. The hot-plug
event path carries no bound, and every other cross-boundary path carries one.

**ARGUMENT.** `MidiPortMap::identity` takes `&self`, and `insert` and `remove` take `&mut self`. The
MIDI thread calls `insert` and `remove` on every hot-plug event. The core thread calls `identity`.
Rust refuses the shared reference across threads without a mechanism, so the design as written does
not compile, or an implementer adds a lock that no section sanctions.

Elastic needs a bound on every producer. B28 to B35 bound the core input channel, the core event
channel, the snapshot channel, the note-entry queue, the MIDI ring, the refill queue, the fault
queue, and the agent request channel. A device that connects and disconnects repeatedly produces
`HotplugEvent` values with no stated limit and no stated drop rule. The note-entry queue states its
drop rule and its counter; the hot-plug path states neither.

**EVIDENCE.** `architecture.md:2754` to `:2823` (the three 5.8 tables), `:301`, `:3477`, `:3489`,
`:3582`, `:3655`.

**WHAT SHOULD CHANGE.** Add a row to the structural table of 5.8 for the presence path, with a B id
and a drop rule. State who owns `MidiPortMap` and how the core reads it. A snapshot published on a
hot-plug event, or a name carried inside the event, removes the shared reference.

### WARNING: Q7. A MIDI port that leaves during a record pass has no specified behaviour

**OBSERVATION.** Section 8.1 states that `MidiPortMap::remove` retires the slot. Section 8.2 covers
arrival, the automatic bind, and a failed bind. No section covers a departure while
`RecordState::Recording` holds.

**CLAIM.** The MIDI input path has no equivalent of `EngineFault::DeviceLost`, and a truncated MIDI
take carries no evidence of the cause.

**ARGUMENT.** The audio path is explicit. `DeviceLost` moves the engine to `EngineState::Faulted`,
12.4 gives it a modal surface, and `CaptureShortfall` marks the region and names the frame count.
Section 5.4 states the reason: "The first draft counted it and told nobody, so silence entered a
vocal take in silence."

The MIDI path names two events, `MidiEvent::InputBound` and `MidiEvent::BindFailed`. Neither covers
a bound port that leaves. `TakeFlag` carries `Uncalibrated`, `HadShortfall`, and `HadDrift`, and
none records a lost input. Nothing states whether the application binds the next candidate, whether
the monitor voice stops, or whether the open take is marked.

PR C-10 makes plug and play with no setup a MUST. A device that unplugs is the ordinary case for a
plug-and-play design, and the design covers arrival only.

**EVIDENCE.** `architecture.md:3499`, `:3568` to `:3571`, `:3611` to `:3624`, `:3142` to `:3150`,
`:4770` to `:4790`.

**WHAT SHOULD CHANGE.** Add a `MidiEvent` variant for a bound port that leaves. State the three
answers: what happens to the open take, what happens to the monitor voice, and whether the
application re-binds. Add a `TakeFlag` value, or state that the take needs none and why.

### WARNING: Q8. `Session` and `MixState` carry no version, so property 4's refusal has no input

**OBSERVATION.** Section 3.6 property 1 gives a version field to `score/meta.json` alone. Property 4
reads: "**The session, mix, and view documents carry no `extra` bag**, and a version they cannot
read is refused with `CommandError::Schema`."

`ViewDocument` carries `schema: SchemaVersion` at B84. No declaration gives `Session` or `MixState` a
version field. `duet.toml` carries a "bundle identity and schema number", which no type declares,
which no B row names, and which no reader reads.

**CLAIM.** For two of the three documents the stated refusal reads a version that does not exist.

**ARGUMENT.** The reason for the split is sound, and I accept it. A foreign tool hand-edits the score
files, an unknown notation field costs nothing, and an unknown routing field would produce audio the
user did not ask for. The mechanism does not follow the reason. A refusal needs a version to compare
against, and `session/session.json` and `mix/strips.json` carry none.

A second gap sits beside it. `SchemaVersion(u32)` derives `Debug`, `Clone`, `Copy`, `PartialEq`,
`Serialize`, and `Deserialize`. It derives no order, and VR1 does not name it. Property 1 says "A
reader migrates an older document through a chain of steps", and property 4 refuses a newer one.
Neither decision can be written without an order on the version.

**EVIDENCE.** `architecture.md:1599` to `:1616`, `:4173` to `:4175`, `:1776`, `:472`, `:514`.

**WHAT SHOULD CHANGE.** Give `Session` and `MixState` a `SchemaVersion` field with its own B row, or
state that `duet.toml` holds the one version for both and declare the type that reads it. Add
`PartialOrd` and `Ord` to `SchemaVersion` and add a VR1 row that names the use.

### WARNING: Q9. The B.5 `tokio` feature list omits `time`, and three declared timeouts need it

**OBSERVATION.** Appendix B.5 gives `tokio` the line
`default-features = false, features = ["rt-multi-thread", "net", "signal", "io-util", "sync", "macros"]`.

B15, B16, and B17 bound the command-line client. Section 9.2 states "**The client carries two
timeouts**, B15 to connect and B16 for a reply, or B17 for `ExportAudio` and `Gc`." The client
speaks the Model Context Protocol over `tokio::net::UnixStream`.

**CLAIM.** The `time` feature is absent, and no sentence says how the client bounds those three
waits without it.

**ARGUMENT.** `tokio::time::timeout` needs the `time` feature. Section 9.5 rule 4 bans a
runtime-bound tokio type from crossing into GPUI code; it does not ban `tokio::time` inside
`duet-agent` or inside the client, so the ban is not the reason for the omission.

B.5's own rule saves an omitted pin: "every pin that this table omits takes its default feature set".
The rule does not save this row, because the row is present and its list is explicit and closed by
`default-features = false`.

Section 5.12 states "A boundary with no timeout is an unbounded wait" and lists all three. An
explicit feature list that removes the mechanism for three listed timeouts is a contradiction inside
one document.

**EVIDENCE.** `architecture.md:5696`, `:3933`, `:2986` to `:2987`, `:445` to `:447`, `:3973`.

**WHAT SHOULD CHANGE.** Add `time` to the row, or name the mechanism the client uses instead and
cite it from 9.2. M7's verification step then records the resolved list, as the row already asks.

### WARNING: Q10. `StripKind::Master` and `BusRole::Master` are two encodings of one strip

**OBSERVATION.** Section 7.1 declares both enums.

```
architecture.md:3231   pub enum StripKind { Track(TrackId), Bus(BusRole), Master }
architecture.md:1442   pub enum BusRole { Part(PartId), Reverb, Delay, Monitor, Master }
```

**CLAIM.** The master strip has two encodings, and `SessionCommand::BusAdd` lets a user build a
second master and a second monitor.

**ARGUMENT.** `BusAdd { name: StripName, role: BusRole }` takes any `BusRole`, so
`BusRole::Master` and `BusRole::Monitor` are both reachable from a verb. Section 7.1 states that the
master is "a bus with a true-peak meter" and that the mixer view pins it to the trailing edge.
`MixState::validate` checks routing cycles only, so nothing refuses the second master.

The same document rejects this shape elsewhere and states the rule. Section 3.4 reads: "`None` is the
one encoding of 'no input'. `Track::input` is therefore not an `Option`, and two encodings of one
state cannot disagree (critic R18)."

Section 7.1's own rule then reads apart from the type. "How the mixer view draws each role" says
"`StripKind::Bus(BusRole)` decides the strip header", which no `StripKind::Master` value reaches.

**EVIDENCE.** `architecture.md:1422`, `:1442`, `:3228` to `:3231`, `:3276` to `:3281`, `:3289`,
`:1425` to `:1434`.

**WHAT SHOULD CHANGE.** Remove `Master` from one of the two enums. If `BusRole` keeps it, make
`BusAdd` refuse `Master` and `Monitor` with a named `MixError` variant, and say why one strip of
each exists from creation.

### WARNING: Q11. The over mark of PR M-05 has no type, no verb, and no chunk

**OBSERVATION.** PR M-05 is a MUST. Its second acceptance criterion reads: "Given a peak above the
ceiling, when it occurs, then the meter holds a clear over mark until the user clears it"
(`product-requirements.md:533`).

Design contract 4.4 specifies the state: the top 6 px in `duet.meter.clip`, the word `CLIP` beside
the meter, and "The state holds until the user clicks the meter or presses the `Clear clip`
command."

Section 7.3 declares `MeterReading { peak: Finite, rms: Finite, true_peak: Finite, hold: Finite }`.
No field holds the latch. `Verb` carries no clear verb. Section 10.1 declares four actions, all of
them mode actions.

**CLAIM.** A MUST acceptance criterion has no model, no path, and no chunk. The 13.2 row for M-05
names T3, D1, C3, and K4, and none of the four holds the latch or the reset.

**ARGUMENT.** The latch is state that survives between frames and that a user clears. The audio
thread computes the peak, so the latch lives with the meter state or with the view. If it lives on
the audio thread, the reset must cross the user-interface boundary, and 5.8 carries one path in that
direction: `triple_buffer::Input<ParamSnapshot>`, addressed by `ParamId`. Nothing states that the
clip reset uses it.

ADR 0005 decision 1 states that "a capability outside the list falsifies the claim that the list is
the whole interface". `Clear clip` is a capability that the contract names and the verb list omits.
Revision 5 failed on the same shape when three capabilities had no verb, and section 9.1 closed that
gap with three verb groups.

**EVIDENCE.** `product-requirements.md:533`, `design-contract.md:752` to `:776`,
`architecture.md:3368` to `:3370`, `:4110`, `:5201`.

**WHAT SHOULD CHANGE.** Add the latch to `MeterReading` or to a named view type. Add the reset verb
or state that the reset is a view action and amend ADR 0005 decision 1. Then name the chunk in the
13.2 row for M-05.

### CONCERN: Q12. The 1.2 dependency column is derived for 13 of 76 pairs, and one sentence is false

Section 1.2 reads "**The column is derived, not written**" and "**Three rows carry a crate that no
declaration proves, and each one is a decision the guard cannot see.**" I measured both claims.
PG12 proves 13 of the 76 crate-and-dependency pairs, and 63 pairs are unproved across every row, not
three. `duet-media` carries five unproved entries, `duet-agent` five, and `duet` five. The limit
paragraph beside PG12 is accurate, and the two sentences above it are not. `duet-core` lists `serde`
that no declaration proves and that no stated use needs; `cargo machete` would reject that entry at
the chunk that adds it.

### CONCERN: Q13. PG11 reads eight verbs and not `depends on`

PG11's sentence lists the eight verbs, so the rule is honest. The claim family is broader. The
document writes "depends on" for an edge in section 1.2 and in section 1.3, which are the sections
where the edge decisions live. MINE-P1 planted "`duet-time` depends on `duet-project`" and the guard
exited 0. I checked every live `depends on` claim, and all of them hold, so the hole is latent.

### CONCERN: Q14. PG7 reads field types and variant payloads only

Section 1.3 states "Only `crates/duet` and `duet-agent` may name one" framework type. PG7 enforces a
narrower rule: a field type or a variant payload type of a placed declaration. MINE-P3 planted
`pub type SidebarWidth = gpui_kit::Px;` in `duet-command`, MINE-P4 planted
`fn width(self) -> gpui_kit::Px` on a `duet-session` type, and MINE-P5 planted the bare `Px` in the
same signature. All three passed. The compiler is the backstop, because neither crate carries an
edge to `gpui-kit`.

### CONCERN: Q15. A dependency cell's prose can stand in for its dependency list

`dependency_table` reads the cell with the word pattern `[A-Za-z_][A-Za-z0-9_-]*`, so prose becomes
dependency names. The `duet-midi` row already yields the tokens `and`, `two`, `target`,
`dependencies`, and `section`. MINE-P7 removed the real list and left prose that contains the words
`serde` and `rtrb`; the guard exited 0. MINE-P8 confirms the direct case goes red.

### CONCERN: Q16. CG2 does not remove a nested raw string

CG2's sentence reads "It removes every line comment, every block comment, and every string literal".
The pattern `r#*"(?:.|\n)*?"#*` ends a raw string at the first quotation mark, so
`r##"say "as" here"##` leaves the word `as` in the scanned text. MINE-C3 gives one false `CAST`
finding. The failure is loud, not silent.

### CONCERN: Q17. The conversion guard has no rule for its own exemption and none for a bad workspace

Two gaps sit beside each other. First, `main` compares `os.path.relpath(path, root)` with the
literal `crates/duet-time/src/convert.rs`. `cargo metadata` returns a canonical path, so a workspace
root reached through a symbolic link never matches. MINE-C4 on a macOS temporary directory reported
the sanctioned suppression and the sanctioned cast inside `convert.rs`; MINE-C6 on a canonical root
is correct. Second, a workspace that `cargo metadata` refuses raises an uncaught Python exception.
The behaviour is fail-closed, which is right, and no CG rule states it and no probe covers it. The
placement guard states the same property as PG2 and proves it as PP2.

### CONCERN: Q18. `PortSlot` retirement has no exhaustion rule

Section 8.1 states that a slot "is retired and never reused while this process runs". `PortSlot`
wraps a `u16`. A long session with repeated connect and disconnect events exhausts the range, and no
sentence says what `insert` returns then.

### CONCERN: Q19. Section 8.1 says a file stores `MidiPortId`, and no file does

Section 8.1 reads "`MidiPortId` is the app-stable identity a user reads, a file stores, and a verb
carries". No declared document field holds a `MidiPortId`. `Track::input` holds an `InputSelection`
over `DeviceKey`, and `Calibration` keys on `DeviceKey`. The sentence names no file.

### CONCERN: Q20. Two sections name two owners for the `view.json` write

Section 10.2 reads "`duet-core` writes the file on the save path". Appendix A reads "`duet-project`
saves a `ViewDocument` as `state/view.json`". One write needs one owner.

### CONCERN: Q21. `BundleDocument` carries no marker for a tracked document

`duet-command` implements the trait for `Score`, `Session`, `MixState`, and `ViewState`. Three enter
a commit and one does not. The trait exposes `paths`, `to_bytes`, `from_bytes`, and `warnings`, and
no method separates the two classes. An implementer must derive the split from the path list of
section 4.2.

### CONCERN: Q22. A `state/view.json` that this build cannot read has no degraded path

Section 4.1 states that a deletion of `state/view.json` "costs the viewport and the selection, which
the user restores with one scroll". Property 4 of section 3.6 refuses an unreadable version with
`CommandError::Schema`. No sentence says that `ProjectOpen` continues with a default view state
after that refusal, so a project opened once by a newer build may refuse to open at all.

---

## 3. Reactive assessment

- **Responsive: PARTIAL.** 86 budgets and eleven timeouts are real, cited by id, and arithmetically
  correct in all but one row. The `JobRunner`, the reserved job slots, and the `Arc` snapshot keep
  every slow verb off the frame thread. Two gaps stay: the default calibration lands a take late at
  the top of B82 (Q4), and the declared `tokio` feature set removes the mechanism for three declared
  timeouts (Q9).
- **Resilient: PARTIAL.** The five-step save, the crash matrix, the four-source live set, the
  writer-lock recovery, the cleanup contract, the typed error rule, and the split `EngineState` are
  correct and complete. Four gaps: a `Copy` derive rests on an undeclared type (Q1), the guard
  counter that should expose that class does not (Q2), two documents refuse a version they do not
  carry (Q8), and a lost MIDI port during a record pass has no answer (Q7).
- **Elastic: PARTIAL.** Every channel, ring, queue, store, and cache that section 5.8 lists carries a
  bound with an id. Two producers stay unbounded: the hot-plug event path has no B row (Q6), and the
  bus count has no limit and no refusal against the fixed `MeterSnapshot` array (Q5).
- **Message Driven: PARTIAL.** One vocabulary, one writer, versioned documents, published snapshots,
  and no shared mutable state across the user-interface seam. `PortSlot` closes the seam defect of
  revision 6 correctly. One seam is open: two threads reach one `MidiPortMap`, and section 5.8, which
  claims to hold every boundary, carries no row for the MIDI presence path (Q6).

---

## 4. Plan graph check on section 13

1. **The graph is acyclic and every link runs forward.** I expanded every cross-line link in 13.4.
   No link runs backwards in phase order.
2. **One same-phase link, and it is the stated exception.** M0 before T1.
3. **Every chunk has a phase and a row.** 55 chunks, 0 gaps, 0 phase mismatches between 13.1, 13.2,
   and 13.3.
4. **SM6 holds in every line.** T2 and T3 share phase 1, and 13.1 states that the trunk is four
   lines of one chunk each. Every other line is a strictly increasing chain.
5. **Write scopes are disjoint per phase.** No two line chunks in one phase share a file.
6. **Every chunk has a runnable Completion command, and every filtered one carries the flag.**
7. **SM3 rule 2 holds.** I checked each filter against every term that an earlier chunk of the same
   line produces. No filter matches an earlier test.
8. **The manifest seam holds.** Each member manifest has one writer per phase. M3 pins `criterion`
   before A4 in phase 5. M7 owns `audio-smoke.yml` after C4 in phase 6.
9. **MUST story coverage is complete at row level.** 64 of 64, and every cited chunk exists. One
   acceptance criterion inside M-05 has no chunk (Q11).
10. **No line chunk writes a policy file.** M0, M6, and M7 carry them all, and all three go to the
    Orchestrator.
11. **SM0 matches the repository.** `crates/duet/src/` holds `main.rs` and `app.rs`, which is what
    K1 expects. `scripts/dod.sh` runs the ten steps that rung one names, and it does not run
    `check-conversions`, which is what M0 adds.

---

## 5. Consistency across the documents

1. **Closed.** Contract 3.1, ADR 0006 decision 2, and architecture 10.5 agree on
   `beat_for_x(x) -> Ticks`.
2. **Closed.** Contract 3.3, ADR 0006 decision 6, architecture 10.5, and Appendix A carry the same
   four-part path-cache key.
3. **Closed.** Contract 4.4 and architecture 10.2 agree that one `MeterLayer` leaf owns the frame.
4. **Closed.** The contract carries the `StageCurve` appendix with the file, the chunk, the stories,
   and the visual rules. The eleven GPUI elements and the twelfth row agree across 10.3, 10.7, and
   K1.
5. **Closed.** ADR 0005 decision 2a agrees with VR1 on `Mode` and on `MidiPortMap`.
6. **Closed.** ADR 0004 decisions 11 and the per-state table agree with 5.5 and B57 on
   `ConfigError::RingBudgetExceeded`.
7. **Closed.** The platform research, section 5.4, section 8.1, section 11, and the crate survey
   agree on cpal 0.18.2, the PipeWire host, the runner names, and the `alsa` question that M3
   records.
8. **Closed.** `deny.toml` carries every licence that B.4 calls present, and it does not carry
   `Unlicense`, which B.4 asks M0 to add. The root manifest already pins `serde` and `clap` with
   `derive`, as B.5 states.
9. **Open.** Contract 4.4 and PR M-05 need an over mark that no type, verb, or chunk holds (Q11).

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The engineering is sound, and revision 7 is the best revision of the seven. DR5 is the right rule
and it works: I ran all twenty-two probes and every recorded exit code and every recorded message
reproduced exactly. Fifteen of the eighteen revision-6 findings close by mechanism, and I proved each
closure by the planted defect rather than by the text. The plan graph passes every mechanical test I
can write: 55 chunks, thirteen phases, one same-phase link, no backward link, no shared write scope,
64 of 64 MUST rows, and no filter that matches an earlier chunk's test. DR3 now holds with no
unexempt hit. The budget table is arithmetically correct in every derived row but one.

One Critical blocks it, and it is the fifth revision in a row where the same class blocks. A `Copy`
derive rests on a type the document never declares. Revision 6 failed on `MidiPortId(Box<str>)`
inside `MidiRecord`; revision 7 fixes that one and leaves `DeviceKey` inside `Calibration`. The
guard that exists to catch it, PG10, needs proof that a field is not `Copy`, and an undeclared type
gives no proof. PG14 exists so that a reader sees how often that happens, and it counts one direction
only. I measured the real number: twenty-five `Copy`-deriving declarations hold an undecidable field,
and `UNKNOWN: 23` reports none of them.

The ten Warnings share two shapes. Five are a rule that a machine implements more narrowly than the
sentence beside it: PG14, CG3, PG7, PG11, and PG12. Five are a design fact with no mechanism: the
strip count against B86, the MIDI presence path across two threads, a lost MIDI port during a record
pass, a schema refusal with no version to read, and a MUST acceptance criterion with no type.

The single biggest risk has narrowed again, and it is now one sentence. **A probe proves that a rule
fires; it does not prove that the rule covers the class its sentence names.** DR5 made the counts
equal and it closed the gap that revision 6 left. The next step is not more rules. It is one line per
rule that states the class the rule does **not** cover, and a counter that prints how often the rule
declines to decide.

The weakest Reactive property is Resilient. A declared derive rests on an undeclared type, two
documents refuse a version they do not carry, and a lost MIDI input during a record pass has no
answer at all.

### Findings that block plan authoring

1. Q1 (Critical). `Calibration` derives `Copy` over `DeviceKey`, which no section declares.
2. Q2 (Warning). PG14 does not count an undecidable field under a `Copy` derive.
3. Q3 (Warning). A real cast passes `check-conversions` between a less-than and a greater-than.
4. Q4 (Warning). The default calibration lands a take late at the top of B82.
5. Q5 (Warning). No rule bounds the strip count against B86, and no refusal exists.
6. Q6 (Warning). The MIDI presence path crosses two threads with no mechanism and no bound.
7. Q7 (Warning). A MIDI port that leaves during a record pass has no specified behaviour.
8. Q8 (Warning). `Session` and `MixState` carry no version, so property 4's refusal has no input.
9. Q9 (Warning). The B.5 `tokio` feature list omits `time`, and three timeouts need it.
10. Q10 (Warning). `StripKind::Master` and `BusRole::Master` are two encodings of one strip.
11. Q11 (Warning). The over mark of PR M-05 has no type, no verb, and no chunk.

### Concerns, each with a disposition for the orchestrator

| Id | Concern | Disposition |
|---|---|---|
| Q12 | The 1.2 column is derived for 13 of 76 pairs, and one sentence is false | Correct the two sentences in 1.2; drop `serde` from the `duet-core` row or name its use |
| Q13 | PG11 reads eight verbs and not `depends on` | Add `depends` to the verb list, or state the limit at PG11's site |
| Q14 | PG7 reads field types and variant payloads only | State the limit at PG7's site and name the compiler as the backstop |
| Q15 | A dependency cell's prose can stand in for its list | Read the cell as a comma-separated list of code spans |
| Q16 | CG2 does not remove a nested raw string | Match the opening hash count when the guard ends a raw string |
| Q17 | The exemption has no rule and a bad workspace has no rule | Canonicalize both paths before the comparison; add a CG rule and a probe for each |
| Q18 | `PortSlot` retirement has no exhaustion rule | State what `insert` returns when the range is full |
| Q19 | Section 8.1 says a file stores `MidiPortId` | Delete the clause, or name the field that holds one |
| Q20 | Two sections name two owners for the `view.json` write | Pick one owner and cite it from the other section |
| Q21 | `BundleDocument` carries no tracked marker | Add a `tracked()` method, or cite section 4.2 from the trait |
| Q22 | A refused `view.json` has no degraded path | State that `ProjectOpen` continues with the default view state |
