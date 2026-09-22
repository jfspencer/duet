# Specification review, revision 6: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is the sixth pass. Revisions 1 to 5 each returned NOT READY.

Sources read in full: `roadmap/duet-v1/architecture.md` (5556 lines), the six ADRs,
`research/linux-macos-platform.md`, `product-requirements.md` section 8, `design-contract.md`
sections 1, 3.1, 3.3, 4.4 and the `StageCurve` appendix, `CLAUDE.md`, the root `Cargo.toml`,
`.cargo/config.toml`, `deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`.

I ran the guard, eleven Architect probes, and two probes of my own. I ran nine mechanical passes.
I ran two real compiler probes with `rustc` and `clippy-driver`. Every probe ran in a throwaway
copy made with one read-only `git archive HEAD`. I wrote no repository file. I ran no other git
command.

Revision 6 is the strongest revision so far. Every one of the twenty revision-5 findings has a
mechanism. Every guard hole that revision 5 left open is now closed, and I proved each closure by
planting the defect. The plan graph is correct on every axis I can check by machine.

It does not close. One family blocks it, and it is the same family that blocked revisions 4 and 5.

**A derive that the declared code cannot compile.** `MidiRecord` and `NoteEntry` derive `Copy` over
`MidiPortId`, which revision 6 declares as `Box<str>`. I proved the failure with `rustc`. The new
guard rule 7 checks only the opposite direction, so it reports `COPY MISSING: 0` on the live
document. The same field also puts a heap free on the audio thread, which TH1 forbids.

---

## 1. Closure check

### 1.1 The twenty revision-5 findings

| Id | Finding | State | Section and reason |
|---|---|---|---|
| S1 | Five types fail `missing_copy_implementations` | **CLOSED** | 3.5 VR5 sets a 32-byte rule. 3.3 and 7.4 take the derive. 4.3, 5.9 and 6.1 carry an `#[expect]` with an invariant reason. B.1 lists three sites. Guard rule 7 goes red when I remove `Spanner`'s `Copy`. |
| S2 | `view.json` has no document type | **CLOSED** | 10.2 declares `ModeView`, `ViewState` and `ViewDocument`. `ViewDocument` implements `BundleDocument`. `ViewGet` returns the whole value. ADR 0005 16a and 16b agree. |
| S3 | Four phase-4 pairs write one member manifest | **CLOSED** | SM6 gives each line one chunk per phase. I parsed all 55 rows: no line has two chunks in one phase. |
| S4 | Chunk I3 owns a bench and no manifest | **CLOSED** | I3 writes `crates/duet-core/benches/`, the member manifest and `Cargo.lock`. B2 writes its manifest for `proptest`. M3 pins `criterion`. |
| S5 | The guard passes on the real shape of P1 | **PARTIAL** | Probe 1b now gives `FRAMEWORK MISUSE: 1` and exit 1. A path-qualified spelling still passes. See N8. |
| S6 | The variant filter is not scoped | **CLOSED** | `mask_enum_arm_names` masks an arm only inside its own enum. Probe 8 now gives `UNPLACED: 1` and exit 1. |
| S7 | `EngineState::NoDevice` carries no cause | **CLOSED** | 12.4 splits the state into `NoServer { detail }`, `OpenTimeout { device }`, `NoDevice` and `RateUnavailable`. Three tables key on one enum each. Six message rows read apart. |
| S8 | Two workflows are vacuous green | **PARTIAL** | Every filtered command now carries `--no-tests=fail`; I checked all of them. The macOS store test is an ordinary gate test at F2. `audio-smoke.yml` moved to M7, after C4. B85 bounds the socket wait. `soak.yml` still selects two tests that no chunk writes. See N3. |
| S9 | `MidiPortId` crosses two namespaces | **PARTIAL** | 8.1 splits the trait, declares `MidiPortMap` and `PlatformPort`, and states the collision rule. The identity type it chose breaks two other rules. See N1. |
| S10 | `check-conversions` rule 4 and the multi-line `use` | **CLOSED** | Rule 4 joins the `use` item to its `;` across lines. A fifth test covers the repository form. I swept the tree under the revision-6 rules and found zero hits. |
| S11 | `dod.sh` depends on a roadmap markdown file | **PARTIAL** | The gate line is gone. The guard runs as the `plan-lint` job, fail closed, with a stated lifecycle. The nine guard tests still run inside the gate. See N4. |
| S12 | The MUST table covers 18 rows of about 70 | **CLOSED** | Product requirements section 8 holds 64 MUST rows. The 13.2 table holds 64 rows. The two sets match exactly. All ten SHOULD stories are named. |
| S13 | 10.5 contradicts contract 3.1 | **CLOSED** | Contract 3.1, ADR 0006 decision 2 and architecture 10.5 all state `beat_for_x(x) -> Ticks`. |
| S14 | The edge check skips an absent subject | **CLOSED** | `edges.get(subject, set())` gives an absent subject the empty set. Probe 9 now gives `EDGE CLAIMS BAD: 1` and exit 1. |
| S15 | The `coremidi` and `pipewire` pins have no owner | **CLOSED** | M3 names both root pins. B.3 carries a `coremidi` row with owner M3. |
| S16 | `midir`'s own `alsa` version is unchecked | **CLOSED** | 8.1 reason 3 states the gap. M3 runs `cargo tree -i alsa` and records the result. The fallback holds if the check fails. |
| S17 | `exports/.tmp` has two shapes | **CLOSED** | 4.1, 7.4 and 9.6 all use `exports/.tmp/<job>/` as a directory. `state/.tmp/` is in the layout and in the bound table. |
| S18 | DR3 has an unstated exemption | **CLOSED** | DR3 names three exemptions: a version pin, a platform or format constant, and a type size. ADR 0005 decision 8 writes B80. |
| S19 | `check-conversions` scans `src` only | **CLOSED** | Rule 1 takes its file set from `cargo metadata --no-deps` and walks `src`, `tests`, `benches`, `examples` and `build.rs`. |
| S20 | I3's Completion may be green before I3 | **CLOSED** | The filter is `test(snapshot) + test(job_registry)`. SM3 rule 2 states the rule for every line. |

**Count: 16 CLOSED, 4 PARTIAL, 0 OPEN.**

### 1.2 The three revision-5 probes

| Probe | Shape | State | Evidence |
|---|---|---|---|
| 1b | A framework type in a field of a `duet-command` struct | **CLOSED** | `FRAMEWORK: duet-command::ModeView holds Px`, exit 1 |
| 8 | An unplaced type named after an enum variant | **CLOSED** | `UNPLACED: Reverb`, exit 1 |
| 9 | An edge claim whose subject carries no edge | **CLOSED** | `EDGE MISS: duet-time -> duet-project`, exit 1 |

### 1.3 The guard, as run

Baseline, on the unmodified document:

```
DOCUMENT:        roadmap/duet-v1/architecture.md
FRAMEWORK NAMES: 51        CANDIDATE TYPES: 265      PLACED: 265
UNPLACED:        0         DUPLICATED:      0        MISCLAIMED: 0
FRAMEWORK MISUSE:0         COPY MISSING:    0
EDGES PARSED:    58        EDGE CLAIMS BAD: 0        EXIT=0
```

Thirteen probes, each one in the throwaway copy:

| # | Probe | Source | Result | Caught |
|---|---|---|---|---|
| 1 | A GPUI type in the `duet-command` table row | Architect | `MISCLAIMED: 1`, exit 1 | Yes |
| 1b | A GPUI type in a `duet-command` field | Critic r5 | `FRAMEWORK MISUSE: 1`, exit 1 | Yes |
| 2 | Every ` ```rust ` fence renamed | Architect | `CANDIDATE TYPES: 0`, exit 1 | Yes |
| 3 | An empty file | Architect | `CANDIDATE TYPES: 0`, exit 1 | Yes |
| 4 | No argument | Architect | usage line, exit 2 | Yes |
| 5 | A second declaration of `Knot` | Architect | `DUPLICATED: 1`, exit 1 | Yes |
| 6 | An unplaced field type | Architect | `UNPLACED: 1`, exit 1 | Yes |
| 7 | A reverse edge claim | Architect | `EDGE CLAIMS BAD: 1`, exit 1 | Yes |
| 8 | An unplaced type named after a variant | Critic r5 | `UNPLACED: 1`, exit 1 | Yes |
| 9 | An edge claim whose subject is the root | Critic r5 | `EDGE CLAIMS BAD: 1`, exit 1 | Yes |
| 10 | A missing `Copy` derive on `Spanner` | Architect | `COPY MISSING: 1`, exit 1 | Yes |
| 11 | An unreadable file, and a missing file | Architect | fail-closed line, exit 2 | Yes |
| 12 | A `Copy` derive over a `Box<str>` newtype | **Mine** | exit 0 | **No** |
| 13 | A path-qualified framework type, `gpui_kit::Px` | **Mine** | exit 0 | **No** |

Eleven of eleven Architect probes are caught. Two probes of mine are not.

### 1.4 The nine mechanical passes

1. **A type declared twice: none.** `DUPLICATED: 0` over 265 candidates, and probe 5 goes red.
2. **A number outside 1.6: four, all exempt.** They are `24 bits`, `13 parts`, `96 kHz` and
   `48 kHz`. DR3 exempts a format constant and a sample rate.
3. **A budget number in an ADR: none.** Every hit is a revision reference or an appendix label.
4. **A B id with no row: none.** All 85 ids resolve. B49 and B50 are cited only inside 1.6.
5. **A chunk absent from a table: none.** 55 chunks appear in 13.1 or 13.2 and in 13.3, with the
   same phase in both.
6. **A same-phase link: one, and it is the stated exception.** The link is M0 before T1.
7. **Two chunks of one line in one phase: none.** Every line is a strictly increasing chain.
8. **A MUST story with no chunk: none.** 64 of 64 rows, and every cited chunk exists.
9. **A filtered nextest command with no `--no-tests=fail`: none.**

---

## 2. New findings in revision 6

### CRITICAL: N1. `MidiRecord` and `NoteEntry` derive `Copy` over a `Box<str>` field

**OBSERVATION.** Section 8.1 declares the port identity as a string newtype.

```
architecture.md:3250   pub struct MidiPortId(Box<str>);
architecture.md:3337   #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
architecture.md:3338   pub struct MidiRecord { port: MidiPortId, at: SampleClock, message: MidiMessage }
architecture.md:3343   pub struct NoteEntry { port: MidiPortId, at: SampleClock, note: MidiNote, velocity: Velocity }
```

Section 3.5 rule ID1 states the rule that the same document then breaks: "A newtype over a string
follows the same shape with `Box<str>` in place of `u64` and **without `Copy`**"
(`architecture.md:1351`).

**CLAIM.** Three separate rules break at this one field. The derive does not compile. The stated
size is wrong. The audio thread frees a heap allocation on every cycle, which TH1 forbids.

**ARGUMENT.** `Box<str>` is not `Copy`, so `#[derive(Copy)]` on a struct that holds one is a hard
error. Revision 6 introduced `MidiPortId(Box<str>)` to close S9, and it did not re-check the two
types that already held a `MidiPortId`. That is finding R0 and finding S1 in a third revision: the
declared code does not build.

The size claim follows. Section 8.3 reads "It is `Copy` and 16 bytes", and section 5.8 reads "A
duplicated 16-byte record is cheaper than a second thread". A `Box<str>` is a fat pointer of 16
bytes on its own, so `MidiRecord` is at least 32 bytes. The reason given for the duplicate write
rests on a number that is wrong by a factor of two or more.

The thread rule follows next, and it is the worst of the three. Section 5.8 puts a
`rtrb` ring of `MidiRecord` at B32 between the MIDI thread and the audio thread
(`architecture.md:2565`). A ring of a type that owns a heap allocation means the producer allocates
on every note and the consumer frees on every note. TH1 reads "The audio thread never allocates,
never frees, never locks, never blocks". The sound path of section 8.3 therefore violates the one
rule that section 5.7 calls unverifiable by machine, and B9 is the budget it would break.

A fourth rule is in the same place. VR1 row 1 gives "Every identifier newtype" `Eq` and `Hash` for
use as a map key. `MidiPortId` derives neither. `MidiPortMap::midir_index(&self, port)` has to find
the identity, so the map needs `Ord` or `Hash + Eq` that the declaration withholds.

**EVIDENCE.** I compiled the exact shape with `rustc --edition 2024 --crate-type lib`:

```
error[E0204]: the trait `Copy` cannot be implemented for this type
 --> lib.rs:4:12
  |
3 | #[derive(Debug, Clone, Copy, PartialEq)]
  |                        ---- in this derive macro expansion
4 | pub struct MidiRecord { port: MidiPortId, at: u64 }
  |            ^^^^^^^^^^   ---------------- this field does not implement `Copy`
```

Guard rule 7 does not see it. I planted the same shape as probe 12 and the guard printed
`COPY MISSING: 0` with exit 0. Rule 7 checks a type that should derive `Copy` and does not. It never
checks a type that derives `Copy` and cannot.

**WHAT SHOULD CHANGE.** Make the record carry a fixed-size port handle, not the app identity. A
`u16` or a `u32` port slot that `MidiPortMap` mints satisfies `Copy`, keeps the record at 16 bytes,
and removes every allocation from the ring. Keep `MidiPortId(Box<str>)` for the verb surface and for
`session.json`. Then give `MidiPortId` the `Eq`, `Ord` and `Hash` that VR1 row 1 already promises
it. Last, add guard rule 9: a type that derives `Copy` and holds a field this document proves is not
`Copy` is a failure. Add the probe that goes red without it.

### WARNING: N2. The section 1.2 dependency column omits serde for four crates and smallvec for two

**OBSERVATION.** Section 1.2 states: "a member manifest is built from both" the dependency column
and the section 1.3 edge list. I extracted every declared type, mapped it to its crate through the
1.5 table, and compared its derive list with its crate's dependency column.

**CLAIM.** Four crates declare a `serde` derive and list no `serde`. Two crates use `SmallVec` and
list no `smallvec`. One crate needs `serde_json` under the rule of section 3.6 and lists none.

**ARGUMENT.** SM1 makes the line chunk add the `{ workspace = true }` entry to its member manifest.
The engineer who writes that manifest reads the 1.2 column. A missing row means a missing entry,
which means the derive does not compile at the chunk that adds it. `serde_json` does not supply the
`Serialize` derive, so a `serde_json` row does not stand in for a `serde` row.

**EVIDENCE.**

| Crate | Missing pin | Proof |
|---|---|---|
| `duet-dsp` | `serde` | `MeterLaw` derives `Serialize, Deserialize` (7.3) |
| `duet-engine` | `serde` | `TransportCommand` derives `Serialize, Deserialize` (5.9) |
| `duet-midi` | `serde` | `HotplugEvent` derives `Serialize, Deserialize` (8.1) |
| `duet-project` | `serde` | `ManifestEntry` derives `Serialize, Deserialize` (4.3) |
| `duet-time` | `smallvec` | `split_tuplet` returns `SmallVec<[Ticks; 12]>` (2.4); B.3 names T1 |
| `duet-core` | `smallvec` | `Transaction::sources` is `SmallVec<[SourceHash; 4]>` (4.9) |
| `duet-session` | `serde_json` | 3.6 property 4: "Every record carries `#[serde(flatten)] extra: BTreeMap<String, serde_json::Value>`" |

Appendix B.3 gives `smallvec` the chunks "T1, T3" and omits I1, so the two tables also disagree.

**WHAT SHOULD CHANGE.** Add each missing crate to its 1.2 row. Add I1 to the `smallvec` row of B.3.
Then state whether property 4 of section 3.6 covers `takes.jsonl`, `regions.jsonl` and
`automation.jsonl`, or only the score files, and make the 1.2 column agree.

### WARNING: N3. Two rung-two commands and both `soak.yml` commands select tests no chunk writes

**OBSERVATION.** Section 14 rung two gives five commands. Command 2 is
`cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(soak)' --no-tests=fail`.
Command 3 is `cargo nextest run -p duet-engine --test alignment --no-tests=fail`. `soak.yml` runs
the soak filter and `-p duet-time -E 'test(proptest_large)'`.

**CLAIM.** No chunk row writes a test named `soak`, a test named `proptest_large`, or the file
`crates/duet-engine/tests/alignment.rs`. No chunk of line C owns a `tests/` directory at all.

**ARGUMENT.** `--test alignment` names an integration target, so it needs a file under
`crates/duet-engine/tests/`. I parsed every Writes column. Only T1, T2, T3, T4, A4, B2 and J1 carry
a `tests/` path. Line C carries none, in any of its four chunks. The soak test is the only
mechanical means that section 5.7 offers for TH1, and its goal appears in no chunk. This is finding
S4 in a new place: work that a rung owns and that no write scope covers.

`--no-tests=fail` makes the consequence louder, not quieter. Revision 5's defect was a filter that
matched nothing and passed. Revision 6's is a filter that matches nothing and fails. `soak.yml`
lands at M6 in phase 6 and is red on the first nightly run, and it stays red until somebody writes
two tests the plan never commissioned.

**EVIDENCE.** `architecture.md:4682` to `:4700` for line C. `architecture.md:5041` for `soak.yml`.
`architecture.md:5091` and `:5100` for rung two. `architecture.md:2512` for the soak test.

**WHAT SHOULD CHANGE.** Name the chunk that writes the soak test, and add
`crates/duet-engine/tests/` to a line C write scope. Name the chunk that writes `proptest_large`,
which is T1 on the face of it. Name the chunk that writes `crates/duet-engine/tests/alignment.rs`.
Then check the phase order: `soak.yml` at M6 must land after the chunk that writes the soak test.

### WARNING: N4. The nine guard tests run inside the gate, so `scripts/dod.sh` can still depend on the roadmap document

**OBSERVATION.** Section 1.5 states: "Nine tests in `tools/xtask` hold the guard, one per Critic
probe." Two of the nine plant "renamed code fences" and "an empty file". `scripts/dod.sh` line 62
runs `cargo nextest run --workspace --locked`, and `tools/xtask` is a workspace member.

**CLAIM.** Revision 6 removed the gate line that read the roadmap document and left nine gate tests
that may read the same document. The specification never says the tests use a self-contained
fixture.

**ARGUMENT.** S11's mechanism was the coupling, not the line. A test that copies
`roadmap/duet-v1/architecture.md`, mutates the copy and asserts an exit code is still a gate that
depends on that file. Any Architect revision that adds a type with no 1.5 row then breaks the
commit of every engineer in the repository, which is the exact outcome that moving the guard to
`plan-lint` was meant to prevent. The fix is one sentence, and its absence leaves the choice to the
implementer at nine sites.

**EVIDENCE.** `architecture.md:4288` to `:4294` for the nine tests. `scripts/dod.sh:62`.
`architecture.md:4609` for the `plan-lint` decision.

**WHAT SHOULD CHANGE.** State that each of the nine tests builds its own small fixture document in
a temporary directory and never reads `roadmap/`. Add a tenth test that asserts the guard rejects a
fixture whose 1.5 table is absent, so the fixture shape is itself proved.

### WARNING: N5. The two rules that revision 6 added have no commissioned test

**OBSERVATION.** The guard now carries eight rules. Rule 1 makes it fail closed on a document that
does not open. Rule 7 is the `Copy` audit that closes S1. Section 1.5 commissions nine tests: five
placement defects, two broken parses and two edge claims.

**CLAIM.** Neither rule 1 nor rule 7 is among the nine. The Architect reports eleven probes; the
plan orders nine. The two that the plan drops are exactly the two that the revision added.

**ARGUMENT.** A guard rule with no red-before test is an unverified rule. Chunk M0 ports the
prototype to Rust, and a port is precisely where a rule silently disappears. Rule 7 is the whole
mechanism of the revision-5 Critical. Rule 1 is the fail-closed property that S11 asked for. I
confirmed that the Python prototype implements both. Nothing in the plan makes the Rust port
implement either.

**EVIDENCE.** `architecture.md:4288` lists the nine. `architecture.md:4241` states rule 1.
`architecture.md:4254` states rule 7. Section 14 repeats "The nine guard probes are part of rung
two" at `:5108`.

**WHAT SHOULD CHANGE.** Raise the count to eleven. Add a probe that removes one `Copy` derive and
asserts a non-zero exit. Add a probe that names an unreadable path and asserts exit 2. Then add the
twelfth probe that finding N1 needs.

### WARNING: N6. The audio thread is the dropper on both audio-to-user-interface triple buffers

**OBSERVATION.** Section 5.8 gives the reason for `triple_buffer`: "A `triple_buffer` reader borrows
and never owns, so the writer thread drops every retired value." The same table puts
`triple_buffer::Output<TransportSnapshot>` and `triple_buffer::Output<MeterSnapshot>` on the audio
to user-interface path. On those two paths the audio thread is the writer.

**CLAIM.** The argument that makes `triple_buffer` correct for the three core-to-audio paths makes
it a TH1 hazard on the two audio-to-user-interface paths. Neither snapshot type is declared, so
nothing bounds what the audio thread drops.

**ARGUMENT.** `Input::write` assigns into the back buffer, which drops the value that buffer held.
The drop runs on the writer's thread. On the core-to-audio paths that is correct and it is the
stated gain. On the audio-to-user-interface paths the audio thread runs the drop once per frame. If
`MeterSnapshot` holds a `Vec` or a `Box<[MeterReading]>` over B1's 32 tracks and its buses, the
audio thread calls `free` on every cycle. TH1 forbids it, and section 5.7 states that no mechanical
guard for TH1 can exist in this repository. The types are in the 1.5 table and no Rust block
declares them, so the hazard is invisible to the derive audit as well.

**EVIDENCE.** `architecture.md:2544` and `:2545` for the two rows. `architecture.md:2548` for the
reason. `architecture.md:257` places both types in `duet-engine` and no block declares either.
`architecture.md:2489` for TH1.

**WHAT SHOULD CHANGE.** Declare both types in section 5.6 or 5.9 with a fixed-size payload. An
`ArrayVec<MeterReading, N>` at the B1 strip count allocates nothing and drops nothing. State the
rule once: a value the audio thread publishes owns no heap. Add it to the forbidden-call list in
`crates/duet-engine/src/audio/README.md`.

### WARNING: N7. `check-conversions` rule 3 rejects qualified path syntax, and it is a gate line

**OBSERVATION.** Section 2.3 rule 3 reads: "It fails on any whole-word `as` token outside
`crates/duet-time/src/convert.rs`. The cast itself, not only its suppression, is the thing the rule
forbids." Rule 4 names one excluded form, the `use` item. Chunk M0 adds the guard to
`scripts/dod.sh`.

**CLAIM.** Qualified path syntax, `<T as Trait>::method`, carries a whole-word `as` token and is not
a cast and not a `use` item. The guard fails on legal Rust that the rule never meant to forbid, and
it fails for the whole repository with no escape hatch.

**ARGUMENT.** `<i64 as TryFrom<i128>>::try_from` and `<Self as Default>::default` are ordinary
forms. A trait method that two traits share can be written no other way. The guard is not a lint, so
`#[expect]` does not reach it, and `scripts/dod.sh` is the only gate surface. From M0 onward any
engineer who needs the form is blocked and has no documented answer. The five commissioned tests
prove that the guard rejects `let n = x as u32;` and accepts two `use` shapes. None proves what it
does with qualified path syntax, so the behaviour is unknown until the first crate needs it.

`duet-time` is the crate most likely to need it, and `convert.rs` is the one file the guard skips,
which hides the problem until a second crate meets it.

**EVIDENCE.** `architecture.md:668` to `:690` for the five rules and the five tests.
`architecture.md:4600` for the `dod.sh` line.

**WHAT SHOULD CHANGE.** Add qualified path syntax as the second excluded form, and remove the span
from `<` to `>` before rule 3 runs. Add a sixth test that plants `<i64 as TryFrom<i128>>::try_from`
and asserts a zero exit.

### WARNING: N8. Guard rule 5 misses a path-qualified framework type

**OBSERVATION.** Rule 5 reads: "It fails on a framework type inside a non-application declaration."
`strip_paths` removes every `::Name` segment before the check runs
(`placement_check.py:98` to `:104`).

**CLAIM.** A framework type written with its module path is invisible to rule 5. I proved it.

**ARGUMENT.** `strip_paths` exists so that `basedrop::Owned` and `TimeError::NotFinite` do not
become candidates. It runs on the same text that rule 5 later reads, so `gpui_kit::Px` becomes
`gpui_kit::` and the name `Px` is gone. Rule 5 is the mechanism that closes blocker P1 and finding
S5. A one-word spelling change defeats it. Rule 5's own sentence claims it fails on "any name in the
framework block of section 1.3", and that claim is false for the qualified spelling.

**EVIDENCE.** Probe 13: I replaced `sidebar_width: LogicalPx` with `sidebar_width: gpui_kit::Px` in
`ModeView` and the guard printed `FRAMEWORK MISUSE: 0` with exit 0. The unqualified form of the same
edit gives `FRAMEWORK: duet-command::ModeView holds Px` and exit 1.

**WHAT SHOULD CHANGE.** Run the framework check on the text before `strip_paths`, or match a
framework name with an optional leading path. Add a probe that plants the qualified spelling.

### WARNING: N9. The design contract keeps a two-part path-cache key

**OBSERVATION.** Design contract 3.3 reads: "`RecordView` caches the built `Path<Pixels>` per region
and per zoom step". ADR 0006 decision 6 reads: "The cache key is
`(SourceHash, RegionId, SystemId, ZoomStep)`." Appendix A gives the same four parts.

**CLAIM.** The contract's two-part key is wrong in a wrapped view, and the revision-5 review already
named it. Revision 6 closed every other cross-document item and left this one.

**ARGUMENT.** A region that runs longer than one system appears on several systems, and each
appearance has its own path. A key of region plus zoom step returns one system's path for every
system, so the second lane paints the first lane's shape. Section 10.5 and ADR 0006 both say a long
take is several segments over one region, which is the case the two-part key cannot address. The
Designer and chunk K3 build from the contract, so the wrong key is the one an implementer reads.

**EVIDENCE.** `design-contract.md:563`. `adr/0006-wrapped-timeline.md:56`.
`architecture.md` Appendix A, the `PathCache` row.

**WHAT SHOULD CHANGE.** Edit design contract 3.3 to the four-part key, and cite ADR 0006 decision 6.

### CONCERN: N10. ADR 0005 decision 2a contradicts VR1

Decision 2a reads "**No vocabulary type derives `Eq` or `Hash`.**" VR1 row 5 gives `Mode` `Eq`,
`Ord` and `Hash`, and `Mode` is a `duet-command` type that ADR 0005 decision 16 itself names. ADR
0001 adds "the vocabulary derives `PartialEq` and asks its payloads for nothing else", and
`BTreeMap<Mode, ModeView>` asks `Mode` for `Ord`. Edit 2a in place: the vocabulary asks a payload
for nothing, and VR1 names the five places where a type carries more for its own use.

### CONCERN: N11. `RingBudgetExceeded` names two different enums

Section 1.6 B57 and section 5.5 write `EngineError::RingBudgetExceeded`. The `configure` signature
in the same section and ADR 0004 decision 11 write `ConfigError::RingBudgetExceeded`. Section 12.1
says `EngineError` wraps `ConfigError` with `#[from]`, so both can exist, and a reader cannot tell
which one `configure` returns. Pick one, and make B57 and 5.5 name it.

### CONCERN: N12. Three sentences describe work that is already done

The document header at line 3 reads "Revision 5". Section 10.2 reads "The report to the operator
lists the one contract wording change this implies", and design contract 4.4 already carries the
`MeterLayer` wording. Section 10.3 reads "The report to the operator asks the Designer to confirm
the twelfth element", and the contract already carries the `StageCurve` appendix with the
orchestrator decision. Edit all three.

### CONCERN: N13. Guard rule 7 covers 85 declared types of 265 candidates

The `Copy` audit reads the document's own Rust blocks. 180 names in the 1.5 table are declared in no
block, and ID1 seeds only the `Id`, `Name` and `Text` suffixes. A field whose type is one of the
other 180 makes the containing type unknown, and an unknown type is never reported. The stated limit
is narrower than the real one: the document says only "a field whose type is external and `Copy` is
invisible to it". State the real denominator, and print the unknown count as a fourth counter so the
blind spot is visible in the run.

### CONCERN: N14. The 1.5 "Declared in" column is stale and nothing checks it

`MidiPortId` is declared in 8.1, `MidiRecord` and `NoteEntry` in 8.3, and `JobState` in 9.6. The
`duet-command` row lists 1.8, 3.5, 7.4, 8.4, 9.1, 9.5, 10.2 and 12.4. The guard reads the crate cell
and the type cell and never reads the section cell, so the column is unverified data in a table that
the whole document treats as the register. Correct the four rows, and add the column to the guard.

### CONCERN: N15. Only cpal's pin names a feature set

Section 1.2 states that `symphonia` "decodes MP3, AAC, FLAC, and OGG". `symphonia`'s default feature
set carries neither MP3 nor AAC, and no chunk row and no B.3 row names a feature. `gix`, `tokio` and
`rmcp` are in the same state: section 9.5 needs `rt-multi-thread`, section 9.2 needs `net`, and the
signal handler of rule 7 needs `signal`. cpal shows the right shape,
`default-features = false, features = ["pipewire"]`. Give every heavy pin the same treatment in the
manifest chunk that owns it.

### CONCERN: N16. `view.json` is tracked, and it holds the scroll offset and the selection

Section 4.2 tracks `view.json`. `ModeView` holds `scroll`, `zoom` and `selection`, and Appendix A
writes the state on `on_drag_move` and `on_scroll_wheel`. Every user commit therefore carries view
churn that the user did not intend, and a checkout of an earlier commit moves the viewport and the
selection. Section 3.6 property 2 promises "A save with no change produces no git diff", and a
scroll makes that false. State the decision: either keep the view state untracked beside `state/`,
or say plainly that history records the viewport.

### CONCERN: N17. The `plan-lint` deletion has no chunk and no owner

Section 14 states: "The job is deleted by the chunk that completes the plan, which is the acceptance
run of phase 13." The phase 13 row names no chunk id, appears in no 13.2 table, and carries no write
scope and no Completion command. `.github/workflows/ci.yml` is a policy file under SM4, so its edit
goes to the Orchestrator. Rung three is a human review that the operator signs. Give the deletion a
real chunk with a write scope, or state that the job stays.

### CONCERN: N18. Section 14 gives the `plan-lint` job two different powers

One row reads "the job fails the pull request that carries the edit". Two paragraphs earlier the
same section reads "CLAUDE.md makes `scripts/dod.sh` the only gate surface, so a failure in any
workflow blocks a merge by review." A workflow that blocks by review does not fail a pull request.
Pick one sentence.

---

## 3. Reactive assessment

- **Responsive: PARTIAL.** 85 budgets and eleven timeouts are real and cited by id. The `JobRunner`,
  the reserved slots and the `Arc` snapshot keep every slow verb off the frame thread. B85 now
  bounds the last unbounded wait. One gap stays: the MIDI ring allocates and frees inside the B9
  path (N1).
- **Resilient: PARTIAL.** The five-step save, the crash matrix, the four-source live set, the
  writer-lock recovery, the cleanup contract, the typed error rule and the split `EngineState` are
  correct and complete. Three gaps: the declared code does not compile (N1), two gate guards carry
  an unproved rule each (N5, N7), and the audio thread drops a value of unknown shape (N6).
- **Elastic: PASS.** Every channel, ring, queue, store and cache carries a bound with an id. The job
  queue reserves four slots for the user. I found no unbounded producer.
- **Message Driven: PASS.** One vocabulary, one writer, versioned documents, published snapshots,
  and no shared mutable state across the user-interface seam. `view.json` now has a document type,
  and the MIDI port identity now has a map. The seam defects of S2 and S9 are closed at the design
  level; N1 is an implementation-level defect inside a correct seam.

---

## 4. Plan graph check on section 13

1. **The graph is acyclic and every link runs forward.** I expanded every cross-line link. No link
   runs backwards in phase order.
2. **One same-phase link, and it is the stated exception.** M0 before T1.
3. **Every chunk has a phase and a row.** 55 chunks, 0 gaps, 0 phase mismatches between 13.1, 13.2
   and 13.3.
4. **SM6 holds in every line.** No line has two chunks in one phase. Every line is a strictly
   increasing chain.
5. **Write scopes are disjoint per phase.** No two line chunks in one phase share a file. Two lines
   own two crates, and SM6 keeps one line's chunks apart.
6. **Every chunk has a runnable Completion command, and every filtered one carries the flag.**
7. **The manifest seam holds.** Each member manifest has one writer per phase. M3 pins `criterion`
   before A4 in phase 5. M7 owns `audio-smoke.yml` after C4 in phase 6.
8. **MUST story coverage is complete.** 64 of 64, and every cited chunk exists.
9. **No line chunk writes a policy file.** M0, M6 and M7 carry them all, and all three go to the
   Orchestrator.
10. **Two write-scope gaps stay.** No chunk owns `crates/duet-engine/tests/`, and no chunk owns the
    deletion of the `plan-lint` job (N3, N17).

---

## 5. Consistency across the documents

1. **Closed.** Contract 3.1, ADR 0006 decision 2 and architecture 10.5 agree on `beat_for_x`.
2. **Closed.** Contract 4.4 carries the `MeterLayer` wording that architecture 10.2 needs.
3. **Closed.** The contract carries the `StageCurve` appendix with the file, the chunk, the stories
   and the visual rules.
4. **Closed.** Contract section 8 and architecture `WorkAreaState` give the four surfaces one type
   and one file.
5. **Closed.** ADR 0005 16a and 16b, architecture 10.2 and ADR 0003 decision 1 agree on
   `ViewDocument`.
6. **Closed.** The platform research, architecture 8.1 reason 3 and B.4 agree that `midir`'s `alsa`
   version is unknown and that M3 records it.
7. **Open.** Design contract 3.3 keeps a two-part path-cache key (N9).
8. **Open.** ADR 0005 decision 2a contradicts VR1 row 5 (N10).
9. **Open.** ADR 0004 decision 11 and architecture B57 name two different enums for one refusal
   (N11).

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The engineering is sound, and revision 6 is the best revision of the six. Sixteen of twenty findings
close by mechanism, and I verified each closure by planting the defect rather than by reading the
text. The guard now catches all eleven Architect probes, including the three holes that revision 5
left. The plan graph passes every mechanical test I can write: 55 chunks, thirteen phases, one
same-phase link, no backward link, no shared write scope, and 64 of 64 MUST stories placed. SM6 is a
rule a machine checks by reading one table, and it bought correctness for three extra phases. That
trade is right.

One Critical blocks it, and it is the fourth revision in a row where the same class blocks. A
revision closed a finding by introducing a new type, and it did not re-check the types that already
held the old one. `MidiPortId` became a `Box<str>`, and two structs that derive `Copy` over it now
fail to compile. I proved it with `rustc`. The same field puts a heap free on the audio thread,
which is the one rule the document says no machine can check. The new guard rule 7 reports zero,
because it checks the missing direction and not the impossible one.

The eight Warnings share one shape with the Critical. A rule is written, a machine is written beside
it, and the machine implements a narrower rule than the sentence claims. Rule 5 says it fails on any
framework name and misses the qualified spelling. Rule 3 of `check-conversions` says it forbids the
cast and forbids qualified path syntax as well. The nine guard tests are commissioned for the seven
old rules and for neither new one. Two rung-two commands and both `soak.yml` commands select tests
that no chunk writes. The 1.2 dependency column is treated as the source a manifest is built from,
and it omits seven entries.

The single biggest risk is unchanged from revision 5 and it is narrower now: the specification's
machines are trustworthy exactly where a probe has been run against them, and nowhere else. Every
rule that a probe covers is correct. Every rule that no probe covers is wrong or unverified. The
answer is not more rules. It is one probe per rule, and the count of probes must equal the count of
rules.

The weakest Reactive property is Resilient. The declared code does not build, two gate guards carry
an unproved rule each, and the audio thread drops a value whose shape no section states.

### Findings that block plan authoring

1. N1 (Critical). `MidiRecord` and `NoteEntry` derive `Copy` over a `Box<str>` field.
2. N2 (Warning). The 1.2 dependency column omits serde for four crates and smallvec for two.
3. N3 (Warning). Four commands select tests that no chunk writes.
4. N4 (Warning). The nine guard tests run inside the gate with no stated fixture.
5. N5 (Warning). The two rules revision 6 added have no commissioned test.
6. N6 (Warning). The audio thread is the dropper on both audio-to-user-interface triple buffers.
7. N7 (Warning). `check-conversions` rule 3 rejects qualified path syntax, and it is a gate line.
8. N8 (Warning). Guard rule 5 misses a path-qualified framework type.
9. N9 (Warning). The design contract keeps a two-part path-cache key.

Findings N10 to N18 are Concerns. Close each one in the specification, or file it in a document
under `roadmap/duet-v1/`.
