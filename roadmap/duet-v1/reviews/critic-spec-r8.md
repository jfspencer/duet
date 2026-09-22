# Specification review, revision 8: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is the eighth pass. Revisions 1 to 7 each returned NOT READY.

Sources read in full: `roadmap/duet-v1/architecture.md` (6482 lines), the six ADRs,
`research/linux-macos-platform.md`, `product-requirements.md` section 8, `design-contract.md`
sections 1, 3.1, 3.3, 4.4 and the `StageCurve` appendix, `CLAUDE.md`, the root `Cargo.toml`,
`clippy.toml`, `.cargo/config.toml`, `rust-toolchain.toml`, `deny.toml`, `scripts/dod.sh`, and
`crates/duet/src/`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/` directory. I ran both guards on the real document and on the real tree. I ran all
twenty-seven recorded probes. I added nine probes of my own to the placement guard and six to the
conversion guard. I ran twelve mechanical passes over the document and the plan graph. I wrote no
repository file. I ran no other git command.

**Revision 8 is the best revision of the eight, and the "state what you do not cover" rule works.**
All twenty-seven recorded probes reproduced the exact exit code and the exact message. Both new
guard rules, PG10b and PG17, turn red on the defect they name. The two prototypes are real files.
The justified-unknown table matches the guard output name for name, with no stale row and no gap.
The plan graph passes every mechanical test. The budget table is arithmetically correct in every
derived row.

It does not close. One Critical and eleven Warnings block it.

---

## 1. Closure check on Q1 to Q22

| Id | Finding | State | Section and reason |
|---|---|---|---|
| Q1 | `Calibration` derives `Copy` over an undeclared `DeviceKey` | **CLOSED** | 5.4 declares `DeviceKey(u64)` as a 64-bit digest, with the sentence "ID1 does not reach this name". PG10b exists and PP10b goes red. |
| Q2 | PG14 does not count an undecidable field under a `Copy` derive | **PARTIAL** | `COPY UNDECIDED` and `UNKNOWN` are separate counters and both print. PG10b makes the first a failure. The counter still drops a fixed set of field types. See R2. |
| Q3 | A real cast passes between a less-than and a greater-than | **PARTIAL** | CG3b exists and CP3b goes red. The fix catches the parenthesized shape only. See R3. |
| Q4 | The default calibration lands a take late at the top of B82 | **CLOSED** | B72 is 384 frames, B73 is "116 to 516 frames, always late", and 500 minus 384 and 900 minus 384 are exactly those numbers. 5.4 states the one-signed reason. |
| Q5 | No rule bounds the strip count against B86 | **PARTIAL** | `ConfigError::StripBudgetExceeded` exists and 12.4 holds the message. B86 is too small for the product budget, and the read-time half names an error `duet-session` may not use. See R5 and R11. |
| Q6 | The MIDI presence path crosses two threads with no mechanism | **CLOSED** | 8.1 makes the MIDI thread the one owner of `MidiPortMap`, `MidiPortInfo` travels inside `HotplugEvent`, and B87 bounds the queue with a drop rule. |
| Q7 | A lost MIDI port during a record pass has no behaviour | **CLOSED** | 8.2 gives four ordered steps with an owner each: the note-off sweep, `MidiEvent::InputLost`, `TakeFlag::MidiPortLost`, and the rebind. |
| Q8 | `Session` and `MixState` carry no version | **CLOSED** | 3.6 property 1 gives both a `schema: SchemaVersion` first field, B89 holds both constants, and `SchemaVersion` derives `Ord`. |
| Q9 | The `tokio` feature list omits `time` | **CLOSED** | B.5 carries `time`, and a new table maps B15, B16, B17, B23, B24, B18, B19, B20, B21, and B22 to a mechanism and a feature. |
| Q10 | `StripKind::Master` and `BusRole::Master` are two encodings | **CLOSED** | 7.1 declares `StripKind { Track, Bus }` with no `Master` arm, and `BusAdd` refuses `Master` and `Monitor` with `MixError::BusRoleReserved`. |
| Q11 | The over mark has no type, no verb, and no chunk | **PARTIAL** | `OverMark` exists in 7.3, `MeterSnapshot` carries the array, `Verb::MeterReset` exists, and 13.2 names I1 and K4. The reset mechanism cannot serve the verb. See R6. |
| Q12 | The 1.2 dependency column claims are false | **CLOSED** | 1.2 now says 70 pairs and 13 proved. I measured 70 and 13. `duet-core` carries no `serde` entry. |
| Q13 | PG11 reads eight verbs and not `depends on` | **CLOSED** | PG11 reads nine verbs, `depends` included, and states its own limit. PP11 goes red. |
| Q14 | PG7 reads field types and variant payloads only | **CLOSED** | PG7 states the limit at its site and names the compiler as the backstop. My MINE-P9 confirms a method signature passes, as the text says. |
| Q15 | A dependency cell's prose can stand in for its list | **CLOSED** | PG12 reads code spans only. I confirmed the `duet-midi` cell yields five names, not the twelve words it holds. |
| Q16 | CG2 does not remove a nested raw string | **CLOSED** | `strip_raw_strings` matches the opening hash count. CP2 passes with `r##"say "as" here and x as u32"##`. |
| Q17 | The exemption and a bad workspace have no rule | **CLOSED** | CG7 canonicalizes both paths and CG8 exits 2 on a refused workspace. CP7 gives `MEMBERS: 2 FILES: 3 FINDINGS: 0` through a symbolic link. |
| Q18 | `PortSlot` retirement has no exhaustion rule | **CLOSED** | B88 bounds the distinct identities, `insert` returns `MidiError::PortSlotsExhausted`, and a re-plug mints no slot. |
| Q19 | Section 8.1 says a file stores `MidiPortId` | **CLOSED** | The clause is gone. |
| Q20 | Two sections name two owners for the `view.json` write | **CLOSED** | 10.2 states that `duet-project` writes every bundle file and `duet-core` writes nothing itself. |
| Q21 | `BundleDocument` carries no tracked marker | **CLOSED** | 1.8 adds `fn tracked() -> bool` with the four answers. |
| Q22 | A refused `view.json` has no degraded path | **PARTIAL** | 10.2 states the degraded path and `CoreEvent::ViewStateReset`. The named constructor does not exist. See R7. |

**Count: 17 CLOSED, 5 PARTIAL, 0 OPEN.**

### 1.1 Every UNKNOWN name against the 1.9 justified list

I extracted the guard's own unknown set and compared it with the table.

- 58 undecidable field instances, over 52 distinct field types.
- The 1.9 table names exactly 52 types.
- Table names that are not in fact unknown: **none**.
- Unknown types that the table omits: **none**.

The mechanical claim holds exactly. Four of the reasons are not true; see C1.

### 1.2 Every B id cited by a Completion command or a workflow

89 budget rows. Every `B<n>` token in the document resolves to a row. No Completion command and no
workflow command cites a budget id that has no row.

### 1.3 The baseline run, as I measured it

```
DOCUMENT:        roadmap/duet-v1/architecture.md
FRAMEWORK NAMES: 51    NAME MAP: 12
CANDIDATE TYPES: 272   DECLARED: 169
PLACED:          272
UNPLACED:        0     DUPLICATED:   0
MISCLAIMED:      0     FRAMEWORK MISUSE: 0
COPY MISSING:    0     COPY IMPOSSIBLE:  0
COPY UNDECIDED:  0     UNKNOWN: 58     JUSTIFIED: 52     UNJUSTIFIED: 0
EDGES PARSED:    58    EDGE CLAIMS BAD: 0
DEP ROWS:        16    DEP MISSING: 0
SNAPSHOTS:       2     SNAPSHOT BAD: 0
REGISTER BAD:    0
TESTS SELECTED:  5     TEST ROWS BAD: 0
EXIT=0
```

The conversion guard on the repository tree gives `MEMBERS: 3   FILES: 6   FINDINGS: 0`, exit 0. The
tree holds exactly six `.rs` files outside `target/`, so the denominator is complete. Both runs match
section 1.9 exactly.

### 1.4 The twenty-seven recorded probes

Every probe reproduced the recorded exit code and the recorded message.

| Probe | Result | Probe | Result |
|---|---|---|---|
| PP1 | exit 2, `usage: placement_check.py <architecture.md>` | PP10b | exit 1, `NOT DECIDED: duet-session::InputSelection derives Copy over DeviceKey` |
| PP2 | exit 2, `FAIL: cannot open ...; the guard is fail-closed.` | PP11 | exit 1, `EDGE MISS: duet-time -> duet-project` |
| PP3 | exit 1, `CANDIDATE TYPES: 0   DECLARED: 0` | PP12 | exit 1, `DEP MISS: duet-score uses serde through Accidental` |
| PP4 | exit 1, `UNPLACED: ProbeUnplaced` | PP13 | exit 1, `SNAPSHOT: MeterSnapshot: the declaration derives no Copy` |
| PP5 | exit 1, `DUPLICATED: Knot` | PP14 | exit 1, `COPY UNDECIDED: 1`, `UNJUSTIFIED: 0` |
| PP6 | exit 1, `MISCLAIMED: Pixels claimed by duet-command` | PP15 | exit 1, `REGISTER: duet-command declares MidiMessage in 8.3` |
| PP7 | exit 1, `FRAMEWORK: duet-command::ModeView holds Px` | PP16 | exit 1, `TEST MISS: soak: no row in the selected-test table` |
| PP8 | exit 1, `UNPLACED: Reverb` | PP17 | exit 1, `UNJUSTIFIED: 1`, `COPY UNDECIDED: 0` |
| PP9 | exit 1, `COPY: duet-score::Spanner needs Copy or an #[expect]` | | |
| CP1 | exit 1, one `CAST` in `benches/` | CP5 | exit 0, `FINDINGS: 0` |
| CP2 | exit 0, `FINDINGS: 0` | CP6 | exit 1, `FINDINGS: 2`, one per form |
| CP3 | exit 1, one `CAST` in `src/lib.rs` | CP7 | exit 0, `MEMBERS: 2   FILES: 3   FINDINGS: 0` |
| CP3b | exit 1, `FINDINGS: 1` | CP8 | exit 2, the fail-closed line |
| CP4 | exit 0, `FINDINGS: 0` | | |

The stated extra check for CG5 also holds. A file with `<i64 as TryFrom<i128>>::try_from(v)`,
`xs.len() as u32`, and `n as usize` gives `FINDINGS: 2`, one per real cast.

### 1.5 My own fifteen probes

| Probe | Shape | Result | Caught |
|---|---|---|---|
| MINE-P1 | A `Copy` derive over an `AtomicU32` field | exit 0, `COPY UNDECIDED: 0` | **No (R2)** |
| MINE-P1-control | The same shape over a `String` field | exit 1, `NOT COPY` | Yes |
| MINE-P2 | A `Copy` derive over an `I24` field | exit 0, `UNKNOWN` unchanged | **No (R2)** |
| MINE-P3 | A `Copy` derive over `CommitId` | exit 0 | **No (R1)** |
| MINE-P4 | A negated edge claim, "`duet-time` never writes `duet-project`" | exit 1, false positive | No (C3) |
| MINE-P5 | A `Copy` derive over `Option<Score>` | exit 1, `COPY UNDECIDED: 1` | Yes |
| MINE-P6 | A 1.9 row that names a type no field uses | exit 0, `JUSTIFIED: 53` | No (C2) |
| MINE-P7 | The 1.9 table header renamed | exit 1, `UNJUSTIFIED: 58` | Yes, fail-closed |
| MINE-P8 | Two declarations of `Tuplet` inside one Rust block | exit 1, `DUPLICATED: Tuplet` | Yes |
| MINE-P9 | A framework type in a `duet-session` method signature | exit 0 | No, and PG7 says so |
| MINE-C1 | `a<b && c as u32 > d`, a cast with no parenthesis in the span | exit 0 | **No (R3)** |
| MINE-C2 | A `use` token with no terminating `;`, in a `macro_rules!` pattern | exit 0 | **No (R4)** |
| MINE-C3 | A char literal that holds a quotation mark, then a cast | exit 1 | Yes |
| MINE-C4 | A second `convert.rs` in another crate | exit 1 | Yes |
| MINE-C5 | An apostrophe in a comment, then a cast | exit 1 | Yes |
| MINE-C6 | `m < n as u32 && n > m` | exit 1 | Yes |

### 1.6 The twelve mechanical passes

1. **A type declared twice: none.** `DUPLICATED: 0` over 272 candidates. PP5 and MINE-P8 both go red.
2. **A budget number outside 1.6: none.** Eighteen hits remain, and every one is a type size in
   bytes or a platform constant, which DR3 exempts.
3. **A B id with no row: none.** 89 rows, and every citation resolves.
4. **A rule id that no text mentions: none.** PG1 to PG17 with PG10b, PP1 to PP17 with PP10b, CG1 to
   CG8 with CG3b, CP1 to CP8 with CP3b, VR1 to VR6, ID1, TH1 to TH9, SM0 to SM7, DR1 to DR5, and PL1
   to PL3 all appear.
5. **A chunk absent from a table: none.** 55 chunks in 13.3, 55 rows in 13.1 and 13.2, no phase
   mismatch between the two.
6. **A same-phase link: one, and it is the stated exception.** M0 before T1.
7. **A backward link: none.** I expanded every cross-line link of 13.4 against the 13.3 phases.
8. **Two chunks of one line in one phase: none.**
9. **A MUST story with no row: none.** Product requirements section 8 holds exactly 64 MUST rows.
   The 13.2 table holds exactly 64. The two sets are identical.
10. **A filtered nextest command with no `--no-tests=fail`: none.** I checked every command in the
    document.
11. **A selected test with no row or no chunk: none.** All five names resolve, and each chunk lands
    at or before the phase of the command that selects it.
12. **A write-scope overlap inside one phase: none.** The only shared names are `src/lib.rs`,
    `tests/`, and `Cargo.lock`, and the first two are relative to two different crate directories.

### 1.7 The VR1 derive audit

I compared every declaration that derives `Eq`, `Hash`, `PartialOrd`, or `Ord` against the VR1 table.
Three declarations derive a trait that VR1 names no use for: `duet-dsp::OverMark` (`Eq`, new in this
revision), `duet-time::Rounding` (`Eq`), and `duet-time::Unit` (`PartialOrd`, which VR1 does not
govern). See C5.

---

## 2. New findings in revision 8

### CRITICAL: R1. `CommitId` is a git object identifier, and the only shape the document gives it is ID1's `u64`

**OBSERVATION.** Section 1.5 places `CommitId` in `duet-command`. No Rust block declares it. It
appears in six signatures.

```
architecture.md:2201   fn commit(&mut self, message: &str, files: &[TrackedFile]) -> Result<CommitId, HistoryError>;
architecture.md:2215   fn read_tree(&self, commit: CommitId) -> Result<TreeSnapshot, HistoryError>;
architecture.md:4244   HistoryCheckout { commit: CommitId },
```

ID1 reads: "A type in section 1.5 whose name ends in `Id`, `Name`, or `Text` and that no Rust block
declares follows ID1, and **no chunk may invent a second shape**" (`architecture.md:1770`). The
integer form of ID1 is `pub struct ThingId(u64)`.

**CLAIM.** `CommitId` names a git commit object. A `u64` cannot hold one. The chunk that writes the
type has one stated shape and that shape is wrong, and the guard actively certifies it.

**ARGUMENT.** `History::commit` returns the identifier that `gix` produced. ADR 0003 decision 2 makes
the store content addressed and the history a real git repository. A git object identifier is a
SHA-1 digest of 20 bytes, or a SHA-256 digest of 32 bytes; `gix` returns `gix::ObjectId`. Section 4.5
states that a commit identifier inside a `Verb` call "is a transient argument" and that the verb fails
with `HistoryError::MissingCommit` when the identifier is gone, so the value must round trip to a real
git reference. A 64-bit digest of the object identifier would need a process-local map that no section
declares, and a truncation would collide.

The document handles the same problem correctly twice next door. `DeviceKey` carries the sentence
"ID1 does not reach this name, so this block declares it" (`architecture.md:5.4`), and `SourceHash` is
declared as 32 bytes and named in VR1 row 1 beside the identifier class. `CommitId` got neither
treatment, and the reason is that ID1 **does** reach it and gives the wrong answer.

The guard makes the wrong answer authoritative. PG10b reads: "a name a `Copy` derive touches is
declared in a Rust block, or it ends in `Id`, `Name`, or `Text` and ID1 decides it. **Nothing else
passes**" (`architecture.md:392`). `copy_verdicts` sets `is_copy["CommitId"] = True` from the suffix
alone (`placement_check.py:386`). So PG10b never asks about a `CommitId` field, whatever it holds.

This is the fifth revision in a row where the blocking finding is a type whose stated shape cannot be
written. Revisions 4 to 6 failed on `Box<str>` under a `Copy` derive. Revision 7 failed on an
undeclared `DeviceKey`. Revision 8 fails on a declared rule that states the wrong shape.

**EVIDENCE.** `architecture.md:307` (the 1.5 row), `:1748` to `:1772` (ID1, both forms),
`:2201`, `:2205`, `:2209`, `:2215`, `:4244`, `:4245`, `:2229` to `:2237` (the pointer table),
`adr/0003-history-model.md:26`, `:50`. MINE-P3: I added `commit: CommitId` to `Calibration`, which
derives `Copy`. The guard printed `COPY UNDECIDED: 0` and exited 0.

**WHAT SHOULD CHANGE.** Declare `CommitId` in section 4.5 with its byte width and its derive list,
and add the sentence that 5.4 already carries for `DeviceKey`. State which git hash function the
repository uses, because the width follows from it. Then make ID1's `Id` suffix state that it decides
the integer form **only** for a name the document does not declare and that is not a digest, and give
PG10b a probe that plants a digest-shaped identifier. Audit the other twenty-seven ID1-governed names
in the same pass; `DeviceId` in `duet-engine` is the next candidate, because cpal gives a device a
name and not a number.

### WARNING: R2. PG10b and PG14 drop a fixed set of field types, and `AtomicU32` is one of them

**OBSERVATION.** PG10b reads: "A type that derives `Copy` and holds a field **whose Copy-ness this
document does not decide** is a failure" (`architecture.md:388`). PG14 reads: "**Every** undecidable
field of every declaration is counted and printed with its type and its field"
(`architecture.md:414`).

`field_names` drops a token that sits in `EXTERNAL` and in neither `COPY_EXTERNAL` nor
`NOT_COPY_EXTERNAL` (`placement_check.py:398` to `:405`). `AtomicU32`, `AtomicU64`, `AtomicBool`,
`Value`, `Path`, `Error`, `Formatter`, and `Owned` are all in that gap. None of the eight is `Copy`.
Separately, `body_type_names` drops a token that `str.isupper()` accepts, so `I24` never reaches the
audit either (`placement_check.py:242` to `:252`).

**CLAIM.** Both sentences are false for a named, closed set of types, and one of those types is the
one this design uses on the audio path.

**ARGUMENT.** Section 7.3 puts the over-mark reset on "an `AtomicU32` counter that `duet-core`
increments and the audio thread reads once per cycle". Section 5.8 carries two more atomic counters,
the fault-drop counter and the meter-reset counter. The chunk that writes the counter will hold it in
a struct. The moment a `Copy`-deriving declaration carries that field, PG10b reports nothing, PG14
counts nothing, and PG17 has nothing to check against the 1.9 table. `rustc` then refuses the derive,
which is the exact failure PG10 and PG10b exist to catch before the chunk is written.

The defect class is the one revision 8 set out to end. DR5's new paragraph says a rule "ends with what
the rule declines to decide, and a counter prints how often it declines". PG10b declines silently for
these types, and the counter that should expose it is the counter with the same hole.

**EVIDENCE.** MINE-P1: I declared `MeterState { reset: AtomicU32 }` with `#[derive(Debug, Clone,
Copy, PartialEq)]`. The guard printed `COPY UNDECIDED: 0`, `UNJUSTIFIED: 0`, and exited 0.
MINE-P1-control: the same declaration over a `String` field gave
`NOT COPY: duet-dsp::MeterState derives Copy over String` and exit 1, so the probe is sound and the
gap is in the type set. MINE-P2: I added `sample: I24` to `MidiRecord`, which derives `Copy`. The
guard exited 0 and `UNKNOWN` did not move.

**WHAT SHOULD CHANGE.** Move `AtomicU32`, `AtomicU64`, `AtomicBool`, `Value`, `Path`, `Error`,
`Formatter`, and `Owned` into the not-`Copy` set. Drop the all-upper filter for a token that the 1.5
table places, so `I24` is decided by its own declaration. Then state at PG14's site the one class that
is still dropped, which is a trait name in a type position, and give PG10b a second probe that plants
an atomic field.

### WARNING: R3. CG3b removes a span that holds a real cast when no bracket sits inside it

**OBSERVATION.** CG3b reads: "An exclusion never hides a cast... A removable span is a balanced `use`
item, or a balanced qualified path whose `as` sits at the span's own bracket depth with a type-shaped
token on each side. **A comparison pair on one line is neither**" (`architecture.md:1010` to `:1015`).

`qualified_path_span` returns `None` on any `;{}()[]` inside the span
(`conversion_check.py:144`). It then tests the text before the `as` with
`before.split("::")[-1].split()[-1]` and the text after it with `after.split("<")[0]`
(`conversion_check.py:162` to `:164`). The two sides are not symmetric: the left side keeps only the
last whitespace token, and the right side keeps the whole remainder.

**CLAIM.** CP3b passes only because its span holds a parenthesis. A comparison pair with no bracket
inside it is still removed, and the cast inside it still disappears. This is the revision-7 Q3 defect,
narrowed and still live.

**ARGUMENT.** CP3b plants `n < xs.len() as u64 && a > b`. The `()` of `xs.len()` trips the bail-out at
line 144, so the span survives and CG3 finds the cast. Remove the call and the bail-out never fires.
In `a<b && c as u32 > d` the walk from `<` reaches the comparison `>` at depth zero, the single `as`
sits at depth zero, `before` reduces to `c`, and `after` is `u32`. Both pass `TYPE_SHAPED`, the span is
blanked, and CG3 reads no `as`.

**The gate as a whole does not fail open, and I say so plainly.** The root `Cargo.toml` sets
`as_conversions = "deny"` in `[workspace.lints.clippy]`, and `scripts/dod.sh` runs
`cargo clippy --workspace --all-targets --locked -- -D warnings`. Clippy refuses the same cast. The
defect is that CG3b's sentence claims a property the machine does not hold, and CP3b is the probe that
was written to prove it.

**EVIDENCE.** MINE-C1, in a throwaway cargo workspace:
`pub fn f(a: u8, b: u8, c: u8, d: u32) -> bool { a<b && c as u32 > d }` gives
`MEMBERS: 1   FILES: 1   FINDINGS: 0` and exit 0. MINE-C6, `m < n as u32 && n > m`, gives
`FINDINGS: 1`, so the outcome turns on where the `as` sits inside the span.

**WHAT SHOULD CHANGE.** Make the two sides symmetric: require the text before the `as` and the text
after it to each reduce to one type-shaped token with no residue. Require the opening `<` to follow a
type-shaped token or to start the line. Then replace CP3b with a span that holds no bracket, because
the present probe passes for the wrong reason.

### WARNING: R4. CG4 blanks the rest of a file after a `use` token with no terminating semicolon

**OBSERVATION.** `drop_use_items` finds each `\buse\b` token and blanks from it to the next `;`. When
no `;` follows, it blanks to the last character of the file (`conversion_check.py:118` to `:120`).

**CLAIM.** One `use` token that no semicolon closes disables CG3 for the whole file, silently. The
conversion guard has no rule for its own bad input, and CG8 covers the workspace only.

**ARGUMENT.** A `macro_rules!` arm may hold a bare `use` in its pattern. A file that a developer is
part way through editing holds one too. In either case the guard prints `FINDINGS: 0` and exit 0 for a
file that holds any number of casts. That is a fail-open on malformed input, and the adversarial rule
the document applies to CG8 applies here with the same force: a check that reports success it did not
earn is worse than no check.

CG8's own sentence is the model. "A workspace that `cargo metadata` refuses is a failure, not a
pass... It is **fail-closed**" (`architecture.md:1048`). CG4 has no such sentence, and the code takes
the opposite branch.

**EVIDENCE.** MINE-C2, in a throwaway cargo workspace:

```rust
macro_rules! m { (use $i:ident) => {} }
pub fn f(x: u64) -> u32 { let n = x as u32; n }
```

gives `MEMBERS: 1   FILES: 1   FINDINGS: 0` and exit 0. The same file with the macro removed gives
`FINDINGS: 1`.

**WHAT SHOULD CHANGE.** Make a `use` item with no terminating `;` a failure that names the file and
exits 2, in the shape of CG8. Add the "does not cover" sentence to CG4 and give it a probe that plants
the unterminated form.

### WARNING: R5. B86 refuses a project that three MUST stories build automatically at the product budget B1

**OBSERVATION.** B86 reads: "41 strips. `MAX_STRIPS`: 32 track strips at B1, 8 buses of any role, and
the master" (`architecture.md:558`). Section 5.5 states: "B86 budgets the buses as one pool of eight,
of any role. A project that uses all eight part buses has no room for a reverb bus, and the refusal
names the limit so the user removes one" (`architecture.md:2921`).

B1 is 8 parts and 32 tracks. Three MUST acceptance criteria create buses with no user action.

- PR M-02: "Given four parts with many tracks, when Mix mode draws, then **each part has one bus
  strip**" (`product-requirements.md:504`).
- PR M-04: "Given a project, when Mix mode first draws, then **one reverb bus and one delay bus
  already exist**" (`product-requirements.md:525`).
- Section 7.1: "Bundle creation builds exactly one of each" master and monitor bus.

**CLAIM.** At B1 the required bus count is eleven and the budget is eight. The project refuses to
build itself at the product budget this design sizes every other number against.

**ARGUMENT.** Count the strips a B1 project holds before the user adds anything: 32 track strips, 8
part buses, 1 reverb bus, 1 delay bus, 1 monitor bus, and 1 master. That is 44. `configure` returns
`ConfigError::StripBudgetExceeded` past 41, and `MixState::validate` runs the same check at read time,
so the bundle is refused at creation and again at open.

The 5.5 paragraph frames the shortfall as a user trade: "the refusal names the limit so the user
removes one". That framing is wrong for this case, because the user added none of the eleven. PR M-04
also allows two to four reverb buses, which section 5.5 itself quotes, and that pushes the floor to
47.

The number is the only defect. The refusal, the mapping to `GatewayError`, and the 12.4 message are
all correct, and they are what makes the shortfall visible instead of silent.

**EVIDENCE.** `architecture.md:558` (B86), `:603` (`MAX_STRIPS`), `:2914` to `:2923`, `:3173` to
`:3175` (both `MeterSnapshot` arrays), `:3580` to `:3583`, `:473` (B1),
`product-requirements.md:502` to `:527`.

**WHAT SHOULD CHANGE.** Raise B86 to at least 47 and rewrite its arithmetic to name every automatic
strip: 32 tracks, 8 part buses, 4 reverb buses, 1 delay bus, 1 monitor bus, and 1 master. State the
resulting `MeterSnapshot` size, because the array is 32 bytes per reading and the 5.9 comment states
"about 1.4 kilobytes" against 41. Then rewrite the 5.5 paragraph, because the present one describes a
choice the user never makes.

### WARNING: R6. The over-mark reset cannot serve its own verb, and it has no path while no audio cycle runs

**OBSERVATION.** Section 9.1 declares the verb with a per-strip payload.

```
architecture.md:4230   /// Clear the held over mark. `None` clears every strip. PR M-05.
architecture.md:4231   MeterReset { strip: Option<StripId> },
```

Section 7.3 declares the mechanism. "The reset travels the other way as an `AtomicU32` counter that
`duet-core` increments and the audio thread reads once per cycle with a relaxed load... **A change of
the counter clears every `OverMark` in the same cycle**" (`architecture.md:3686` to `:3691`).

**CLAIM.** One counter carries no strip identity, so `MeterReset { strip: Some(id) }` has no
mechanism. The reset also has no path in the five engine states in which no cycle runs.

**ARGUMENT.** Design contract 4.4 states the user action: "The state holds until the user **clicks the
meter** or presses the `Clear clip` command" (`design-contract.md:765`). A click on one strip's meter
is a per-strip reset, and the verb models it with `Option<StripId>`. The counter clears all 41 marks
whatever the payload. Two encodings of one intent disagree, which is the rule section 3.4 states for
`Track::input` and section 7.1 restates for `StripKind`.

The second half is a lost reset. `EngineState` carries `NoServer`, `OpenTimeout`, `NoDevice`,
`RateUnavailable`, and `Faulted`, and in each one no audio cycle runs. The last `MeterSnapshot` the
audio thread published keeps its latched marks, the mixer keeps painting `CLIP`, and every increment
of the counter is read by nobody. The user clicks and nothing happens. Section 7.3 states no answer,
and 12.4 lists no message for it.

Contract 4.4 adds a third value to the same reset: "A numeric peak readout... shows the highest peak
since **the last reset**". `MeterReading::hold` carries that value and section 7.3 says the counter
clears `OverMark` only.

**EVIDENCE.** `architecture.md:4230`, `:3170` to `:3175`, `:3673` to `:3691`, `:3092` (the 5.8 row),
`design-contract.md:762` to `:770`, architecture 12.4 `EngineState` table.

**WHAT SHOULD CHANGE.** Replace the single counter with a per-strip generation, or state plainly that
the reset is global and change the verb to `MeterReset` with no payload and the contract sentence with
it. State what `hold` does on a reset. Then state what happens when no cycle runs: either the core
clears the last published snapshot itself, or the mixer hides the latch in a non-running engine state.

### WARNING: R7. `ViewState::default` does not exist, and no section states the default window geometry

**OBSERVATION.** Section 10.2 states the degraded path that closes Q22: "The core logs the warning,
**opens the project with `ViewState::default`**, and emits `CoreEvent::ViewStateReset { reason }`"
(`architecture.md:4658`).

The declarations carry no `Default`.

```
architecture.md:4585   #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]  pub enum Mode
architecture.md:4607   #[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]  pub struct WindowGeometry
architecture.md:4626   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]  pub struct ViewState
architecture.md:1166   #[derive(Debug, Clone, Copy, Serialize, Deserialize)]  pub struct Finite
```

**CLAIM.** The named constructor cannot be built from the declared derives, and the value it would
produce is wrong.

**ARGUMENT.** `ViewState` holds `mode: Mode`, `per_mode: BTreeMap<Mode, ModeView>`, and
`window: WindowGeometry`. A derived `Default` needs `Mode: Default` and `WindowGeometry: Default`.
`WindowGeometry` holds four `LogicalPx`, which wraps `Finite`, and `Finite` derives no `Default` and
has only the constant `ZERO`. So the chain stops at `Finite`.

Even if each derive were added, the value is wrong. A `WindowGeometry` of four zeros is a window of
zero width at the screen origin. Section 10.2 gives `ProjectView::clamped` for the opposite case, a
stored width larger than the window, and nothing for this one. `crates/duet/src/main.rs` already holds
the real default, `point(px(120.0), px(120.0))` and `size(px(960.0), px(640.0))`, and those four
numbers appear in no section and carry no B row, which DR3 requires.

VR1's blanket sentence fixes the derive set for this crate: "Every other type in `duet-command`,
`duet-score`, and `duet-session` derives `Debug`, `Clone`, `PartialEq`, `Serialize`, and
`Deserialize`, and adds `Copy` when every field is `Copy`". An implementer who adds `Default` to five
types is outside that sentence with no rule that allows it.

**EVIDENCE.** `architecture.md:4657` to `:4662`, `:4583` to `:4638`, `:1166`,
`crates/duet/src/main.rs:34` to `:45`.

**WHAT SHOULD CHANGE.** Replace `ViewState::default` with a named constructor, for example
`ViewState::first_open(window: WindowGeometry)`, and state which `Mode` it opens in. Add a B row for
the default window size and position, and cite it from 10.2 and from the chunk that writes
`main_window_options`. Then state whether `Finite` gains a `Default`, because the answer binds every
type above it.

### WARNING: R8. The threading contract omits three threads that the design starts

**OBSERVATION.** Section 5.7 is the threading contract. Its table holds seven rows: GPUI foreground,
Core, Audio, Engine disk, Job threads, GPUI background, and tokio workers
(`architecture.md:5.7`).

The document names three more threads.

- The **MIDI thread**, in twelve places. It owns `MidiPortMap` (`:3906`), it writes both B32 and B31
  (`:3087`, `:3088`), it runs `open_input` under B22 (`:3979`), and it is the producer on the B9 path
  (`:3259`).
- The **platform presence listener**: "The platform listener runs on its own thread" (`:3930`).
- The **file-watch thread**: "`duet-project` starts `notify` with `notify-debouncer-full` on the
  thread that `notify` creates" (`:4368`).

**CLAIM.** Three threads have no owner row and no "Never does" row, and one of them is the producer on
the tightest latency budget in the document.

**ARGUMENT.** The table's purpose is stated by its own column heading: it is the one place that says
what each thread may never do. TH1 binds the audio thread, TH4 the engine disk thread, TH6 and TH7 the
tokio workers. No rule binds the MIDI thread. Section 8.3 then relies on an unstated rule: "a
`Box<str>` inside either one... would allocate on the MIDI thread and free on the audio thread, which
TH1 forbids on the B9 path" (`:4066`). TH1 says nothing about the MIDI thread. The sentence is right
about the outcome and wrong about which rule produces it.

The same gap reaches Elastic. B87 bounds the hot-plug queue and B31 and B32 bound the two note paths,
so the MIDI thread's outputs are bounded. Nothing states that the MIDI thread may not block, may not
run a file read, and may not call into `duet-core`. A blocking call on that thread costs B9 directly.

**EVIDENCE.** `architecture.md:5.7` (the seven-row table), `:275`, `:3087`, `:3088`, `:3259`,
`:3906`, `:3930`, `:3979`, `:4066`, `:4368`, `:6148` (the B22 row, which names "The MIDI thread").

**WHAT SHOULD CHANGE.** Add three rows to the 5.7 table with an owner and a "Never does" cell each.
Give the MIDI thread a thread rule in the TH family that names the B9 path, and cite it from 8.3
instead of TH1.

### WARNING: R9. Four chunks modify `top_bar.rs`, and none of the four owns it

**OBSERVATION.** Section 13.2 states: "`TopBar` shows a per-mode group set, so **K2, K3, K4, and K5
each add their own group to the file K1 created**" (`architecture.md:5567`).

The four Writes columns name no shell file.

- K2: `crates/duet/src/element/staff_system.rs`, `src/compose/{view,caret,menu,duration}.rs`.
- K3: `crates/duet/src/element/{waveform_lane,playhead_layer,punch_range}.rs`, `src/record/{...}.rs`.
- K4: `crates/duet/src/element/{level_meter,fader,knob,automation_lane,stage_curve}.rs`, `src/mix/{...}.rs`.
- K5: `crates/duet/src/element/lufs_meter.rs`, `src/master/{view,export_dialog,report}.rs`.

**CLAIM.** Four chunks must edit `crates/duet/src/shell/top_bar.rs`, which sits in no write scope but
K1's. SM0 makes each of them stop.

**ARGUMENT.** SM0 reads: "verify the current state of the files in the write scope; report a
discrepancy and stop, instead of proceeding". A chunk that must change a file outside its scope either
stops, which blocks the plan, or writes outside its scope, which breaks the rule that makes the phase
table a proof of disjoint write scopes. Section 13.3 claim 1 rests on that proof.

The stories are MUST stories. PR C-15 to C-18 give the top bar the notation controls, and section
13.2 assigns each to K2 through K5 through the `TopBar` row. So the work is real and the file is
named.

**EVIDENCE.** `architecture.md:5567`, the four chunk rows in 13.2, the `top_bar.rs` row of the shell
file table (`:5570`), and SM0 in 13.0. I confirmed the write scopes with a mechanical pass over every
chunk row: `src/shell/` appears in K1's scope and in K6's seven named files, and in no other chunk.

**WHAT SHOULD CHANGE.** Add `crates/duet/src/shell/top_bar.rs` to the write scope of K2, K3, K4, and
K5. SM6 already puts the five chunks in five phases, so the file keeps one writer per phase and claim
1 of 13.3 still holds.

### WARNING: R10. The section 1.5 denominator is stale in all three of its numbers

**OBSERVATION.** Section 1.5 reads: "**The real denominator is stated, not left to a reader.** The
table places **354** types; the document's Rust blocks declare **146** of them; **208** are named in
the table alone" (`architecture.md:445`).

I measured the same three numbers with the document's own parser.

| Claim | Stated | Measured |
|---|---|---|
| Types the 1.5 table places | 354 | **355** |
| Of those, declared by a Rust block | 146 | **169** |
| Named in the table alone | 208 | **186** |

The section 1.9 baseline prints `DECLARED: 169`, so the document contradicts itself by 23.

**CLAIM.** The one paragraph that states the guard's coverage carries revision 7's numbers. It also
breaks DR3.

**ARGUMENT.** The paragraph exists to answer the adversarial question the last three reviews asked:
how large is the set the guard cannot decide. A reader who takes 146 of 354 computes 41 percent
coverage. The truth is 169 of 355, which is 48 percent. The direction of the error flatters the guard.

DR3 says a number is stated exactly once, in section 1.6, and it exempts four forms. One of the four
is "a recorded count from a guard run, **which section 1.9 prints**". These three counts are recorded
counts that section 1.5 prints, so the exemption does not reach them and no other exemption does.

The same paragraph in Appendix C claims "the twenty new declarations" for row Q1. The measured
difference between revision 7 and revision 8 is 23.

**EVIDENCE.** `architecture.md:443` to `:455`, `:731` (the 1.9 baseline), `:6442` (the Q1 row). My
measurement uses `ownership_table` and `declarations` from the document's own prototype.

**WHAT SHOULD CHANGE.** Rewrite the three numbers to 355, 169, and 186, and correct "twenty" to the
real count. Then move them into section 1.9 beside the baseline run, so DR3's fourth exemption covers
them and one guard run refreshes all three.

### WARNING: R11. The strip-budget refusal has two homes, and one of them cannot name the error

**OBSERVATION.** Section 5.5 states the refusal twice.

```
architecture.md:2904   /// `ConfigError::StripBudgetExceeded` past B86.
architecture.md:2918   `MixState::validate` runs the same check, so a hand-edited
architecture.md:2919   `mix/strips.json` is refused at read time and not at the first cycle.
```

`configure` is a `duet-engine` function and `ConfigError` is a `duet-engine` type
(`architecture.md:312`). `MixState::validate` is a `duet-session` function and returns `MixError`
(`architecture.md:3588` to `:3593`). The document names four `MixError` variants: `RoutingCycle`,
`BusRoleReserved`, `EmptyCurve`, and `DuplicatePoint`. None of the four is a strip budget.

**CLAIM.** `MixState::validate` cannot return the refusal that section 5.5 assigns to it. PL1 forbids
the type, and `MixError` carries no variant for it.

**ARGUMENT.** Section 1.3 lists every internal edge. `duet-session` depends on `duet-time` alone.
`duet-engine` depends on `duet-session` and not the reverse, and rule 4 of section 1.3 states it:
"`duet-session` names no engine type." So `validate` cannot name `ConfigError::StripBudgetExceeded`,
and the chunk that writes T3 has no error to return.

The read-time half is the important half. Section 5.5's own reason for it is that a hand-edited
`mix/strips.json` must be refused "at read time and not at the first cycle". `configure` runs at the
first cycle. With no `MixError` variant, the read-time refusal does not exist, and a hand-edited file
with 60 strips reaches the audio thread, where 19 strips write no meter for the whole session. That
is the exact defect critic Q5 recorded, half closed.

ADR 0004 carries a third spelling. Decision 12a1 reads "`BusAdd` now returns
`ConfigError::StripBudgetExceeded` past B86". `SessionCommand::BusAdd` is a `duet-session` command,
so that sentence names an impossible return type as well.

**EVIDENCE.** `architecture.md:2898` to `:2923`, `:3581`, `:3588` to `:3593`, `:3629` to `:3630`,
`:202` to `:204` (the edge list), `:249` (rule 4), `:312` (the `ConfigError` row),
`adr/0004-backend-and-threading-contract.md:143` to `:147`.

**WHAT SHOULD CHANGE.** Add `MixError::StripBudgetExceeded { requested }` and give it a 12.4 row
beside the `ConfigError` one. State which of the two each caller returns: `BusAdd` and `validate`
return the `MixError` form, and `configure` returns the `ConfigError` form. Then correct ADR 0004
decision 12a1.

### WARNING: R12. Three ADR decisions state a rule the architecture replaced, and one of them risks two writers

**OBSERVATION.** Three decision records still carry revision-7 text that the architecture rewrote.

1. **ADR 0004 decision 6b**: "**The default when no calibration exists is B72**, which is a
   deliberate overestimate for a consumer interface. B73 bounds the error, and it is **always early,
   never late**" (`adr/0004:70`). The architecture states the opposite sign. B72 "guesses nothing"
   (`architecture.md:544`) and B73 is "always **late**" (`:545`).
2. **ADR 0004 decision 12a1**: "`BusAdd` now returns `ConfigError::StripBudgetExceeded`"
   (`adr/0004:146`). See R11.
3. **ADR 0005 decision 9**: "A second writer that finds the lock tries the socket, forwards its verbs
   when the socket answers, and **takes the lock when it does not**" (`adr/0005:41`).

**CLAIM.** Item 3 is a data-safety divergence. Items 1 and 2 send a chunk the wrong sign and the
wrong type. DR1 makes a revision edit a decision in place, and these three were not edited.

**ARGUMENT.** Architecture 3.8 gives the writer-lock protocol four steps. Step 3 reads liveness
before it takes the lock: "A live process with a dead socket is a start in progress, so the second
writer waits." Step 4 retries under B25 and exits with `GatewayError::ProjectBusy`. The lock also
holds "the start time of that process", which "defends against a reused process identifier". ADR 0005
decision 9 has no liveness check, no retry, and no start time. An implementer who follows it lets a
second process take the lock while the first process is still binding its socket. Two writers then
hold one bundle, and section 3.8 exists to make that impossible. Chunk J2 writes "the client lock
helper", so the risk reaches a real chunk.

Item 1 reaches chunk C4, which writes the calibration split and the alignment test. The alignment
test asserts the region position for the B72 default case. An implementer who reads the ADR applies
the offset in the wrong direction, and the test that would catch it takes its expectation from the
same wrong sentence.

The ADRs are not commentary. Section "How to read this document" states "Decision records live in
`roadmap/duet-v1/adr/`. Six decisions carry one", and section 13.2 sends each chunk to its ADR.

**EVIDENCE.** `adr/0004-backend-and-threading-contract.md:70` to `:73`, `:146`;
`adr/0005-agent-gateway.md:41`; `architecture.md:544`, `:545`, `:2656` to `:2660`, `:1937` to
`:1950`.

**WHAT SHOULD CHANGE.** Rewrite the three decisions in place. Then run one pass over all six ADRs
against the current sections, because four more stale statements are in the Concerns below and the
class is systematic.

---

## 3. Concerns

### CONCERN: C1. Four rows of the 1.9 justified-unknown table name the wrong crate

The class label is part of the reason DR5 now requires. Four of the 52 names sit in a class whose
crate they do not belong to. `ScoreSelector` is in the class "A boxed `Verb` payload in
`duet-command`" and section 1.5 places it in `duet-score`. `Layer`, `SlotConfig`, and `SlotPosition`
are in the class "A notation value object in `duet-score`" and section 1.5 places all three in
`duet-session`. The load-bearing half of each reason, "None sits under a `Copy` derive", is true and
PG10b enforces it.

### CONCERN: C2. PG17 checks one direction only, and its sentence does not say so

PG17 fails on an undecidable field type the table omits. It never fails on a table row that names a
type no field uses. MINE-P6: I added `NoSuchTypeAtAll` to the `SharedString` row; `JUSTIFIED` went
from 52 to 53 and the guard exited 0. PG15 states the same one-directional limit at its own site and
PG17 does not, which is the DR5 sentence the revision added.

### CONCERN: C3. A negated edge claim gives a false failure

`edge_claims` tests for `no`, `not`, or `never` only in the text **after** the verb
(`placement_check.py:359`). MINE-P4: I planted "`duet-time` never writes `duet-project`" and the guard
reported `EDGE MISS: duet-time -> duet-project` and exited 1. The document carries no such sentence
today, so the hole is latent, and the failure is loud rather than silent.

### CONCERN: C4. The CP1 and CP3 recorded messages do not reproduce in a macOS temporary directory

`main` computes the displayed path with `os.path.relpath(path, root)` while `cargo metadata` returns a
resolved path (`conversion_check.py:200`). CG7 canonicalizes the exemption and leaves the display
alone. Under `/var/folders/...` on macOS, CP1 prints
`CAST: ../../../../../../private/var/.../m/benches/bench.rs` instead of the recorded
`CAST: m/benches/bench.rs`. The verdict is correct in both cases. Chunk M0 ports each probe to a test
that builds a temporary workspace, and CI runs it on `macos-26`, so a test that asserts the recorded
text is red there and green on Linux.

### CONCERN: C5. VR1 has three live violations and one false reason

VR1 reads: "`Eq`, `Hash`, and `Ord` appear only where this rule names the use." `duet-dsp::OverMark`,
new in this revision, derives `Eq` and the table names no use. `duet-time::Rounding` derives `Eq` and
the table names no use. `duet-time::Unit` derives `PartialOrd`, which the rule's own sentence does not
govern. Separately, the row "`MidiNote`, `Velocity` | The set of sounding notes per port" names a use
that section 8.2 step 1 replaced: "the set is a fixed-size bitset over the 128 note numbers". A bitset
indexes by value and needs no `Ord` and no `Hash`, and it holds no velocity at all. `MidiPortInfo` sits
in row 2, "The two halves of `MidiPortMap`", and it is neither half.

### CONCERN: C6. The document header says Revision 7

Line 3 reads "Status: proposed. Revision 7." Line 342 reads "Revision 7 carried sixteen. Revision 8
adds PG10b". DR1 makes a revision rewrite the sentence it changes.

### CONCERN: C7. Two crates are in the dependency graph, in no crate table row, and in no chunk

Section 1.3 says "This list is the whole internal graph" and carries `duet-backend-coreaudio` and
`duet-backend-alsa`. The 1.2 crate table holds 16 rows and names neither. The 1.5 ownership table
places no type in either. No chunk of section 13 creates either skeleton. Section 5.4 treats both as
later work: "The native backends of section 11 remove the two-callback shape altogether, and this
measurement is the evidence that decides when to build them."

### CONCERN: C8. `roadmap/duet-v1/acceptance.md` has no author and no chunk

Rung three reads the checklist from that path. The file does not exist. Phase 13 of section 13.3 reads
"The acceptance run of section 14" and carries no chunk id, no write scope, and no Completion command.
The seven judgements are fully stated in section 14, so the content exists and the file does not.

### CONCERN: C9. The `Clear clip` command has no action

Design contract 4.4 names two user paths: a click on the meter, and the `Clear clip` command. Section
10.1 declares four actions, all mode actions, and section 1.5 places exactly those four action types
in `crates/duet`. Section 10.2's menu rule reads "A verb with no menu item is allowed; a menu item
with no verb is not", so a command needs an action. `Verb::MeterReset` covers the click and not the
command. The string `Clear clip` appears in no other document.

### CONCERN: C10. `EngineFault::ClockDrift` has two shapes in one document

Section 5.4 writes `EngineFault::ClockDrift(report)` (`architecture.md:2701`). Section 12.4 declares
`ClockDrift { parts_per_million: i32 }` (`:5216`). `EngineFault` derives `Copy` and `Serialize`, and
`DriftReport` derives neither `Serialize` nor `Deserialize`, so the tuple form would not build. The
declaration is authoritative and the prose is wrong.

### CONCERN: C11. The 12.4 refusal table lists two `EngineState` variants as `GatewayError` variants

Section 12.4 states "the table carries a row for `PoolExhausted`, `RingBudgetExceeded`,
`StripBudgetExceeded`, `BusRoleReserved`, `NoServer`, and `RateUnavailable`" for the table it keys on
`GatewayError` (`architecture.md:5270` to `:5273`). `NoServer` and `RateUnavailable` are `EngineState`
variants (`:5228`, `:5235`) with their own table three lines above. The same section records the same
defect as closed: "Revision 5 named a row for `NoServer`, which belonged to no enum it listed (critic
S7)" (`:5250`).

### CONCERN: C12. Four more ADR statements are stale

Each one is a smaller instance of R12. ADR 0002 decision 3c groups the view document with the session
and mix documents and says all three "refuse a version they cannot read"; architecture 10.2 and ADR
0005 give `view.json` a degraded open. ADR 0002 decision 3b says a refused version returns
`CommandError::Schema`; `duet_score::read` returns `ScoreError::Schema` (`architecture.md:1873`). ADR
0005 decision 2a names "the six places" of VR1 and the table holds twelve rows, and it names "the two
halves of `MidiPortMap`" where VR1 row 2 names three types. ADR 0001 cites VR1 row 1 for `PortSlot`
and `MidiPortId`, which sit in row 2. ADR 0005 decision 2a also states VR5 with no size bound, and
the 32-byte clause decides three of the five rows of the VR5 table.

### CONCERN: C13. Appendix B.3 gives `quick-xml` a second manifest owner

Chunk M2 pins `quick-xml` in phase 2 (`architecture.md:5407`). Appendix B.3 names M3 as its owner
(`:6082`). Chunk B1, the only consumer, runs in phase 2, so M2 is the correct owner and B.3 is wrong.

### CONCERN: C14. The `StageCurve` appendix names a token the token table does not hold

Design contract line 1495 reads "the live dot uses `duet.meter.peak`". The meter tokens are
`duet.meter.low`, `duet.meter.mid`, `duet.meter.high`, `duet.meter.clip`, `duet.meter.peak_cap`, and
`duet.meter.scale` (`design-contract.md:1147` to `:1152`). The rest of the appendix agrees with
architecture 10.3, 10.7, and chunk K4.

### CONCERN: C15. Section 10.5 names no type, owner, or chunk for virtualization

ADR 0006 decision 8 reads "**Virtualization is a `VirtualList` over systems**". `VirtualList` appears
once in the architecture, inside the framework-name block that the guard reads
(`architecture.md:264`). Section 10.7 asserts the behaviour, "the correct systems render at a given
scroll offset" (`:4962`), and no section names the element, its owner, or the chunk that writes it.
ADR 0006 decision 8 also allows "a pixel bitmap cache" in Mix and Master and says "the specification
states both"; the architecture states one draw path.

### CONCERN: C16. ADR 0006 keys the Mix and Master path cache by a scalar

ADR 0006 decision 8 reads "They keep the same `Path<Pixels>` cache, keyed by the zoom scalar rather
than by a system" (`adr/0006:64`). Architecture Appendix A gives one key to all four modes:
"`Entity<MixView>` in Mix and Master ... `(SourceHash, RegionId, SystemId, ZoomStep)`"
(`architecture.md:5989`). Section 10.5 then states "three documents now say so" about the four-part
key (`:4886`).

---

## 4. Reactive assessment

- **Responsive: PASS.** 89 budgets and eleven timeouts are real, cited by id, and arithmetically
  correct in every derived row. B72 and B73 now state a one-signed error that matches B82 exactly. The
  timeout table of Appendix B.5 names a mechanism and a crate feature for every bound. The `JobRunner`,
  the reserved slots, and the `Arc` snapshot keep every slow verb off the frame thread.
- **Resilient: PARTIAL.** The five-step save, the crash matrix, the four-source live set, the typed
  error rule, the split `EngineState`, the four-step MIDI port-loss path, and the schema refusal with
  a real version are correct and complete. Four gaps: an identifier type has an impossible stated
  shape (R1), the derive guard drops a named type set (R2), the degraded view path names a
  constructor that cannot exist (R7), and the threading contract omits the thread that feeds B9 (R8).
- **Elastic: PARTIAL.** Every channel, ring, queue, store, and cache in section 5.8 carries a bound
  with an id, and B87 closes the hot-plug path with a drop rule. One bound is wrong in the other
  direction: B86 is smaller than the strip set that three MUST stories build with no user action
  (R5).
- **Message Driven: PARTIAL.** One vocabulary, one writer, versioned documents, published snapshots,
  and no shared mutable state across the user-interface seam. `MidiPortMap` now has one owner and
  every consumer reads a minted `MidiPortInfo`. One path is wrong: the over-mark reset carries no
  strip identity and has no delivery while no audio cycle runs (R6).

---

## 5. Plan graph check on section 13

1. **The graph is acyclic and every link runs forward.** I expanded every cross-line link of 13.4
   against the 13.3 phases by machine. No link runs backward.
2. **One same-phase link, and it is the stated exception.** M0 before T1.
3. **Every chunk has a phase and a row.** 55 chunks, 0 gaps, 0 phase mismatches between 13.1, 13.2,
   and 13.3.
4. **SM6 holds in every line.** No two chunks of one line share a phase.
5. **Write scopes are disjoint per phase**, with one exception that is a plan defect and not a rule
   defect: four chunks must write a file outside their scope (R9).
6. **Every chunk has a runnable Completion command, and every filtered one carries
   `--no-tests=fail`.**
7. **SM3 rule 2 holds.** I checked each filter against every term an earlier chunk of the same line
   produces.
8. **The manifest seam holds.** Each member manifest has one writer per phase. M3 pins `criterion`
   before A4 in phase 5. M7 owns `audio-smoke.yml` after C4 in phase 6.
9. **MUST story coverage is exact.** 64 MUST rows in the product requirements, 64 rows in the 13.2
   table, and the two sets are identical.
10. **No line chunk writes a policy file.** M0, M6, and M7 carry them all, and all three go to the
    Orchestrator.
11. **SM0 matches the repository.** `crates/duet/src/` holds `main.rs` and `app.rs`, which is what K1
    expects. `.github/workflows/ci.yml` exists, so M0 edits it. `scripts/dod.sh` runs the ten steps
    rung one names and does not run `check-conversions`, which is what M0 adds.

---

## 6. Consistency across the documents

1. **Closed.** Contract 3.1, ADR 0006 decision 2, and architecture 10.5 agree on
   `beat_for_x(x) -> Ticks`.
2. **Closed.** Contract 3.3, ADR 0006 decision 6, architecture 10.5, and Appendix A carry the same
   four-part path-cache key, `(SourceHash, RegionId, SystemId, ZoomStep)`.
3. **Closed.** ADR 0003 and architecture 4.5 agree on the tracked set, the content store, the four
   live-set sources, the five save steps, the crash matrix, and the symbolic pointers. Neither states
   the shape of `CommitId` (R1).
4. **Closed.** ADR 0001 and architecture 2 agree on B40, B41, the two kernel newtypes, the absent
   order on `Position` and `Delta`, and the six named methods.
5. **Closed.** The platform research, section 5.4, and section 11 agree on cpal 0.18.2 with
   `default-features = false, features = ["pipewire"]`, on `HostId::PipeWire` and `HostId::CoreAudio`,
   on no ALSA fallback, on `ubuntu-26.04` and `macos-26`, and on the two development libraries.
6. **Closed.** Product requirements section 8 holds 64 MUST rows and the 13.2 table holds 64. The two
   sets are identical. Every one of the ten SHOULD stories the paragraph below the table names is a
   SHOULD in the product requirements.
7. **Closed.** `deny.toml` carries every licence that B.4 calls present, and it does not carry
   `Unlicense`, which B.4 asks M0 to add for `midly`.
8. **Closed.** The root `Cargo.toml` sets `panic = "abort"` and `overflow-checks = true` in
   `[profile.release]`, which is what section 2.2 states, and `as_conversions = "deny"`, which is what
   section 2.3 names as the clippy backstop.
9. **Open.** ADR 0004 states the opposite sign of the uncalibrated take error and names an impossible
   return type; ADR 0005 states a writer-lock protocol that allows two writers (R12).
10. **Open.** Contract 4.4 needs a per-meter reset and a `Clear clip` command; the architecture gives
    a global counter and no action (R6, C9).
11. **Open.** ADR 0006 keys the Mix and Master path cache by a scalar and names `VirtualList` and a
    bitmap cache that the architecture does not carry (C15, C16). ADR 0002, ADR 0005, and ADR 0001
    carry four more stale statements (C12).

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The engineering is sound and revision 8 is the best of the eight. The rule the last review asked for
works: every guard rule now states the class it declines to decide, `COPY UNDECIDED` and `UNJUSTIFIED`
print beside `UNKNOWN`, and PG10b and PG17 both turn red on a real defect. I ran all twenty-seven
probes and every recorded exit code and every recorded message reproduced. Seventeen of the
twenty-two revision-7 findings close by mechanism, and I proved each closure with a planted defect
rather than with the text. The justified-unknown table matches the guard output name for name, with no
stale row and no gap. The plan graph passes twelve mechanical tests: 55 chunks, thirteen phases, one
same-phase link, no backward link, 64 of 64 MUST rows, 89 budget rows with no orphan citation, and no
filter that matches an earlier chunk's test.

One Critical blocks it, and it is the same class for the sixth revision in a row: a type whose stated
shape cannot be written. This time the shape comes from a rule rather than from a declaration.
`CommitId` names a git object, and ID1 tells the chunk to write a `u64`. The guard then reads the same
suffix and certifies the answer, so PG10b, the rule this revision added to close that class, is the
rule that hides it.

The single biggest risk has moved one level up, and it has two halves that share one shape. The first
half: **a rule that decides a shape is as dangerous as a rule that declines to decide one, and only
the second kind has a counter.** ID1 decides 28 names by suffix, and nothing tests any of those 28
against the domain the name comes from. The second half: **the six decision records went eight
revisions with no guard, and they now hold nine statements the architecture replaced.** Both guards
read one file, and that file is the only one revision 8 kept true. The next step is one row per
ID1-governed name that states its inner type, and one pass over the six ADRs with a rule that says an
ADR statement is a citation of a section, never a second copy of it.

Eleven Warnings sit under the Critical, and they fall into four shapes. Three are a machine narrower
than its own sentence: PG10b and PG14 drop a named type set, CG3b removes a span it promises to keep,
and CG4 fails open on a file it cannot parse. Five are a design fact with no mechanism: B86 refuses a
project three MUST stories build, the over-mark reset carries no strip identity, `ViewState::default`
cannot be built, the threading contract omits the thread that feeds B9, and the read-time strip
refusal has no error type it may name. Two are plan defects: four chunks write a file they do not
own, and the coverage paragraph carries the previous revision's numbers. One is document drift: three
ADR decisions state a replaced rule, and one of them allows two writers on one bundle.

The weakest Reactive property is Resilient. An identifier type has an impossible shape, the guard
that would catch it certifies it instead, the degraded path that protects a project on open names a
constructor that does not exist, and the decision record for the writer lock removes the liveness
check that stops two processes from writing one bundle.

### Findings that block plan authoring

1. R1 (Critical). `CommitId` is a git object identifier and ID1 gives it a `u64`.
2. R2 (Warning). PG10b and PG14 drop `AtomicU32` and seven other field types.
3. R3 (Warning). CG3b removes a span that holds a real cast when no bracket sits inside it.
4. R4 (Warning). CG4 blanks the rest of a file after a `use` with no semicolon.
5. R5 (Warning). B86 refuses a project that three MUST stories build at B1.
6. R6 (Warning). The over-mark reset cannot serve its own verb and has no path while the engine is stopped.
7. R7 (Warning). `ViewState::default` does not exist and the default window geometry is unstated.
8. R8 (Warning). The threading contract omits the MIDI thread, the presence listener, and the watcher.
9. R9 (Warning). K2, K3, K4, and K5 modify `top_bar.rs` and none of them owns it.
10. R10 (Warning). The section 1.5 denominator is stale in all three numbers.
11. R11 (Warning). `MixState::validate` cannot name the strip-budget refusal that 5.5 assigns to it.
12. R12 (Warning). Three ADR decisions state a replaced rule, and one allows two writers.

### Concerns, each with a disposition for the orchestrator

| Id | Concern | Disposition |
|---|---|---|
| C1 | Four rows of the 1.9 table name the wrong crate | Move `ScoreSelector` to its own class or correct the label; split the notation class into `duet-score` and `duet-session` |
| C2 | PG17 checks one direction only and does not say so | Add the one-directional limit sentence to PG17, in PG15's shape |
| C3 | A negated edge claim gives a false failure | Test for `no`, `not`, and `never` in the text before the verb as well as after it |
| C4 | CP1 and CP3 do not reproduce in a macOS temporary directory | Canonicalize `root` before `relpath`, as CG7 already does for the exemption |
| C5 | VR1 has three unnamed derives and one false reason | Remove `Eq` from `OverMark` and `Rounding`, or add a row; rewrite the `MidiNote` row to name the bitset and drop `Velocity` |
| C6 | The header says Revision 7 | Set line 3 to Revision 8 |
| C7 | Two backend crates are in the graph, in no table, and in no chunk | State that both are later work and move them out of the 1.3 list, or add the two 1.2 rows |
| C8 | `acceptance.md` has no chunk | Add the file to M7's write scope, or state that the operator writes it |
| C9 | The `Clear clip` command has no action | Add the action to `gpui_kit::actions!` and a 1.5 row, or delete the clause from contract 4.4 |
| C10 | `EngineFault::ClockDrift` has two shapes | Rewrite line 2701 to the declared struct form |
| C11 | The 12.4 refusal table lists two `EngineState` variants | Move `NoServer` and `RateUnavailable` to the engine-state sentence |
| C12 | Four more ADR statements are stale | Fix them in the same pass as R12; add a rule that an ADR cites a section rather than copies it |
| C13 | Appendix B.3 gives `quick-xml` a second owner | Change the B.3 owner cell to M2 |
| C14 | The `StageCurve` appendix names `duet.meter.peak` | Change it to `duet.meter.peak_cap` |
| C15 | Virtualization has no type, owner, or chunk | Name the element and its chunk in 10.5, or delete the `VirtualList` and bitmap claims from ADR 0006 |
| C16 | ADR 0006 keys the Mix path cache by a scalar | Delete the second key; Appendix A and 10.5 already state one key for four modes |
