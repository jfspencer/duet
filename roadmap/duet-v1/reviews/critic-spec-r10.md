# Specification review, revision 10: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-21. Mode: specification review, before plan authoring.
This is the tenth pass. Revisions 1 to 9 each returned NOT READY.

Sources read in full, in the plan: `roadmap/duet-v1/architecture.md` (8499 lines), the six ADRs,
`research/linux-macos-platform.md`, `product-requirements.md` section 8, and `design-contract.md`
sections 1, 1.6, 3.1, 3.3, 4.4 with the `StageCurve` appendix. Sources read in full, in the
repository: `CLAUDE.md`, the root `Cargo.toml`, `clippy.toml`, `rust-toolchain.toml`,
`.cargo/config.toml`, `deny.toml`, `scripts/dod.sh`, and `crates/duet/src/`.

I made one throwaway copy with one read-only `git archive HEAD`, plus a copy of the untracked
`roadmap/duet-v1/`. I ran both guards on the real inputs. I ran all 35 recorded probes. I added
11 probes to the placement guard and 19 probes to the conversion guard. I ran six compiler runs
against the pinned toolchain, rustc 1.98.1 and clippy 0.1.98, under the real `[workspace.lints]`
table. I ran eight mechanical passes. I wrote no repository file. I ran no other git command.

**Revision 10 keeps every promise it makes about the guards.** It also closes every finding that
the guard family can see. All 35 recorded probes reproduce the recorded exit code and the recorded
message. The baseline run matches section 1.9 line for line: `DERIVE CLOSURE: 0`, `REACH BAD: 0`,
`B.1 BAD: 0`, `VR1 BAD: 0`, `UNDECLARED: 0`. PG19 and PG20 each go red on the shape the
revision-9 review planted by hand. The five new 1.3 edges make no cycle, and the topological order
holds. The plan graph passes every mechanical test.

It does not close. **The guard family reads this document and no other file, and the lint policy
is a second file.** Four Criticals block it. Three of the four come from a compiler run and not from an argument. 81
declarations trip a denied nursery lint. One enum does not build at all. Three of the six stated
type sizes are wrong, by a factor of up to 3.6.

---

## 1. Closure check on S1 to S4, W1 to W16, and C1 to C8

| Id | Finding | State | Section and reason |
|---|---|---|---|
| S1 | Ten `Default` derives over `Finite` and `FrameCount` | **CLOSED** | 2.6a gives `Finite` a `Default` derive with a stated reason; 2.1 gives `FrameCount` one; PG19 exists and PP19 prints `DERIVE CLOSURE: 30`. My compiler run builds `Finite` as declared. |
| S2 | Sixteen field references cross an absent edge; five are Cargo cycles | **CLOSED** | 1.3 adds five edges and moves four types; `InverseCost`, `TransportCommand`, `LoudnessReport`, and `UpstreamFailure` sit below their consumers; PG20 exists and `REACH BAD: 0`. The order stays topological. |
| S3 | Three `missing_copy_implementations` sites in a binary crate | **CLOSED** | 15.16 carries none and states why; VR5 gives the reachability rule; B.1 lists nothing from `crates/duet`; PG21 goes red in both directions. |
| S4 | Three `Serialize` derives reach a type that derives none | **CLOSED** | `CanonicalDocument` and `LoudnessReport` gain serde; `GatewayError` gains serde and the `Upstream` mirror; four error enums gain serde. |
| W1 | B90 is false at B86 and carries no refusal | **CLOSED** | B90 is 2048 and the row carries the whole arithmetic. I recomputed it: 192 plus 384 plus 704 plus 128 plus 12 plus 6 plus 25 is 1451. B96 exists, and both refusals exist. |
| W2 | The per-strip reset has no storage and no stopped-engine path | **PARTIAL** | `ResetGenerations` and `ChainState::reset_seen` close the storage half. The stopped-engine half is wrong: the stale snapshot restores the mark one frame later. See W-1. |
| W3 | `from_finite_const` cannot be a `const fn` | **CLOSED** | The body asserts. `f64::is_finite` is `const` on 1.98 and my compiler run builds the function. The nine `duet-command` constants sit in 1.6. |
| W4 | `first_open` leaves `per_mode` empty | **CLOSED** | `ModeView::first_open(mode)` is declared, the field table gives all eight values, and `per_mode` holds one entry per `Mode`. |
| W5 | `ModeToolbar` has no declaration and no chunk owns the toolbar | **PARTIAL** | 15.16 declares it, 1.5 places it, `Toolbar` moves to K1, and K1 creates four stubs. Nothing wires `TopBar` to the four stubs. See W-2. |
| W6 | The presence listener must mint a value only the MIDI thread may mint | **CLOSED** | 5.7, 5.8, 8.1, B87, and B100 agree: the listener pushes a `PlatformPort`, the MIDI thread mints. ADR 0004 13c disagrees. See W-8. |
| W7 | B.1 lists three sites and the document holds eight | **CLOSED** | B.1 lists five, the document holds five, and PG21 proves both directions. |
| W8 | Property 4 needs an `extra` bag that `Spanner` cannot hold | **CLOSED** | All eight records carry the bag; `Spanner`, `Measure`, and `Voice` drop `Copy`; the tempo-map exclusion is stated; ADR 0002 decision 2 defers to 3.6. |
| W9 | `ObjectId` collides, and `duet.toml` is not the format authority | **PARTIAL** | `CommitDigest` and `History::object_format` close both halves. 4.7 then writes a git extension that plain `git` refuses. See C-3. |
| W10 | The conversion guard drops a cast between two strings | **PARTIAL** | CG2 as one lexer pass and CG2b close the plain string and the line-comment shape. A raw byte string still hides a real cast. See W-3. |
| W11 | DR6 is stated and not applied | **PARTIAL** | ADR 0005 15 and 16c, ADR 0004 12b, and ADR 0003 13 are all rewritten as citations. ADR 0004 13c and ADR 0001 10 are stale now. See W-8. |
| W12 | The first-open window is below the contract minimum | **PARTIAL** | B91 and B92 are 1024 by 700, which the contract calls the minimum and not the default. The narrow breakpoint then refuses B94 and B97. See W-5. |
| W13 | `VirtualList` has no paragraph, no owner, and no chunk | **CLOSED** | 10.3 holds the section with the owner, the item, the width model, the identity, and the two chunks. Appendix A holds a row. ADR 0006 decision 9 cites 10.3. |
| W14 | `ZoomScalar` is undeclared and unplaced | **CLOSED** | 10.2 declares it with `ZoomStep::scalar`, 1.5 places it, VR1 names the use, and 10.5 holds both keys in one table. |
| W15 | `Snapshot` carries no session and no mix state | **CLOSED** | `Snapshot` carries `SessionDocument` and `MixDocument`; 15.4 declares both; B101 gives the size. |
| W16 | VR1's table is stale and eight derives have no use | **CLOSED** | VR1 names every type, six types drop a derive, PG22 exists, and `VR1 BAD: 0`. |
| C1 | Chunk ids B1 to B3 collide with budget ids | **CLOSED** | The interchange line is X. I checked 13.2, 13.3, 13.4, B.3, and B.4: no B id names a chunk. |
| C2 | The guard carries an unstated drop list | **PARTIAL** | The 70-name drop list is a fenced block the guard reads, and the run prints `DROP LIST: 70`. A second hidden table of 17 primitives remains. See C-4. |
| C3 | The conversion guard takes an injectable root | **CLOSED** | CG1 states that the subcommand takes no path, and it bounds the prototype argument to a probe. |
| C4 | 12.1 says three enums and 15.10 declares five | **CLOSED** | 12.1 names all five. |
| C5 | `ParamSnapshot` derives `Copy` with no VR5 answer | **CLOSED** | VR5 carries a large-`Copy` table with two rows and a reason each. I measured `ParamSnapshot` at 16392 bytes, which matches the stated 16 kilobytes. |
| C6 | `Rest` names `Value` and `Note` names `serde_json::Value` | **CLOSED** | `Rest` uses the qualified spelling. |
| C7 | Phase 13 claims a width of one | **CLOSED** | Phase 13 has a width of zero, with the reason. |
| C8 | `mix_vertical_split` and `inspector_width` have no value | **CLOSED** | B97, B98, and B99 exist, and the 10.2 field table lists all eight fields. |

**Count: 21 CLOSED, 7 PARTIAL, 0 OPEN.** Every PARTIAL is a half that a new finding names.

### 1.1 The baseline run, as I measured it

```
DOCUMENT:        roadmap/duet-v1/architecture.md
FRAMEWORK NAMES: 51    NAME MAP: 12
DROP LIST:       70
CANDIDATE TYPES: 369   DECLARED: 368
TABLE NAMES:     368     UNDECLARED: 0
EXTERNAL ROWS:   40     EXTERNAL MISSING: 0
PLACED:          369
UNPLACED:        0     DUPLICATED:   0
MISCLAIMED:      0     FRAMEWORK MISUSE: 0
COPY MISSING:    0     COPY IMPOSSIBLE:  0
COPY UNDECIDED:  0     UNKNOWN: 1     JUSTIFIED: 1     UNJUSTIFIED: 0
DERIVE CLOSURE:  0     DERIVE UNDECIDED: 0
REACH BAD:       0
EXPECTATIONS:    5     B.1 ROWS: 5     B.1 BAD: 0
VR1 ROWS:        67     VR1 BAD: 0
EDGES PARSED:    59    EDGE CLAIMS BAD: 0
DEP ROWS:        16    DEP MISSING: 0
SNAPSHOTS:       2     SNAPSHOT BAD: 0
REGISTER BAD:    0
TESTS SELECTED:  5     TEST ROWS BAD: 0
EXIT=0
```

Every line matches section 1.9. The conversion guard on the repository tree gives
`MEMBERS: 3   FILES: 6   FINDINGS: 0`, exit 0. The tree holds exactly six `.rs` files outside
`target/`, so the denominator is whole.

### 1.2 The 35 recorded probes

Every probe reproduced the recorded exit code and the recorded message.

| Probe | Result | Probe | Result |
|---|---|---|---|
| PP1 | exit 2, `usage: placement_check.py <architecture.md>` | PP13 | exit 1, `SNAPSHOT: MeterSnapshot: the declaration derives no Copy` |
| PP2 | exit 2, the fail-closed line | PP14 | exit 1, `COPY UNDECIDED: 1` |
| PP3 | exit 1, `CANDIDATE TYPES: 0   DECLARED: 0` | PP15 | exit 1, `REGISTER: duet-command declares MidiMessage in 8.3` |
| PP4 | exit 1, `UNPLACED: ProbeUnplaced` | PP16 | exit 1, `TEST MISS: soak` |
| PP4b | exit 1, `UNDECLARED: 1`, `VoiceId` | PP17 | exit 1, `UNJUSTIFIED: 1` |
| PP5 | exit 1, `DUPLICATED: Knot` | PP18 | exit 1, `EXTERNAL: FontMetrics names BTreeMap` |
| PP6 | exit 1, `MISCLAIMED: Pixels claimed by duet-command` | PP19 | exit 1, `DERIVE CLOSURE: 30`, `BiquadState derives Default over b0` |
| PP7 | exit 1, `FRAMEWORK: duet-command::ModeView holds Px` | PP20 | exit 1, `REACH: duet-engrave::FontMetrics holds StripId` |
| PP8 | exit 1, `UNPLACED: Reverb` | PP21 | exit 1, `B.1: ConfigError carries an expectation` |
| PP9 | exit 1, `COPY: duet-command::ExportSpec needs Copy` | PP22 | exit 1, `VR1: duet-score::Revision derives Eq` |
| PP10 | exit 1, both shapes | CP1 | exit 1, `CAST: m/benches/bench.rs` |
| PP10b | exit 1, `NOT DECIDED: InputSelection derives Copy over DeviceKey` | CP2 | exit 0, `FINDINGS: 0` |
| PP11 | exit 1, `EDGE MISS: duet-time -> duet-project` | CP2b | exit 1, `FINDINGS: 2`, one per shape |
| PP12 | exit 1, `DEP MISS: duet-score uses serde through Accidental` | CP3 | exit 1, `CAST: m/src/lib.rs` |
| CP3b | exit 1, `FINDINGS: 1` | CP4 | exit 0, `FINDINGS: 0` |
| CP4b | exit 2, ``a `use` at line 11 has no `;` `` | CP5 | exit 0, `FINDINGS: 0` |
| CP6 | exit 1, `FINDINGS: 2`, one per form | CP7 | exit 0, `MEMBERS: 2   FILES: 3`, direct and through a link |
| CP8 | exit 2, the fail-closed line | - | - |

### 1.3 My own 30 probes

| Probe | Shape | Result | Caught |
|---|---|---|---|
| MINE-P1 | A tuple struct in `duet-engrave` over `StripId` | exit 1 | Yes, `REACH BAD: 1` |
| MINE-P2 | A variant payload in `duet-engrave` over `StripId` | exit 1 | Yes, `REACH` |
| MINE-P3 | A generic parameter `T` under `Copy` and `Default` | exit 1 | Loud, by `EXTERNAL` on `T` |
| MINE-P4 | A `&'a str` field under a `Copy` derive | exit 1 | **False red**; `&T` is always `Copy` (C-6) |
| MINE-P5 | `Box<dyn AudioProcess>` in a `duet-command` declaration | exit 1 | **No**; `REACH BAD: 0` (W-4) |
| MINE-P6 | A `where` clause | exit 1 | Loud, by `EXTERNAL` on `T` |
| MINE-P7 | A `Default` derive over `[Finite; 64]` | exit 0 | **No**; the stated exclusion (C-7) |
| MINE-P8 | A `Default` derive over `Box<[Finite]>` | exit 1 | **False red**; the compiler accepts it (C-5) |
| MINE-P9 | A `PartialEq` derive with no `Eq` over an `Eq` field | exit 0 | **No** (C-1) |
| MINE-P10 | A false byte count inside an expectation reason | exit 0 | **No**; PG21 reads a name (C-4) |
| MINE-P11 | A `Default` derive over a `char` field | exit 0 | Correct; `char` does implement `Default` |
| MINE-C1 | A raw identifier `r#type` before a cast | exit 1 | Yes |
| MINE-C2 | `br##"say "as" here"##` before a cast | exit 1 | **False red** on the literal (W-3) |
| MINE-C3 | A string that ends with an escaped backslash | exit 1 | Yes |
| MINE-C4 | A character literal `'\''` before a cast | exit 1 | Yes |
| MINE-C5 | Lifetimes, an `impl` block, and a `where` clause | exit 1 | Yes, no false red |
| MINE-C6 | `<Vec<u8> as Default>::default()` beside a cast | exit 1 | **False red** on the path (C-8) |
| MINE-C7 | A cast of a const generic, `N as u32` | exit 1 | Yes |
| MINE-C8 | A cast in `build.rs` | exit 1 | Yes; the denominator holds |
| MINE-C9 | A crate under `crates/` outside the members list | exit 0 | **Not scanned** (C-10) |
| MINE-C10 | The prototype root argument | exit 0 | Stated by CG1; the subcommand drops it |
| MINE-C11 | A plain byte string `b"..as.."` | exit 0 | Correct |
| MINE-C12 | A raw byte string `br#"..as.."#` | exit 0 | Correct by luck; see MINE-C17 |
| MINE-C13 | A C string `c"..as.."` | exit 0 | Correct |
| MINE-C14 | A raw string that holds `//` | exit 1 | Yes |
| MINE-C15 | A file that is not UTF-8 beside a real cast | exit 1 | **Traceback**, not the stated exit 2 (C-9) |
| MINE-C16 | A cast inside a `macro_rules!` body | exit 1 | Yes |
| CONTROL | `r#"a " b"#` before a real cast | exit 1 | Yes, `FINDINGS: 1` |
| MINE-C17 | `br#"a " b"#` before the same real cast | exit 0 | **No**; the cast vanishes (W-3) |
| MINE-C18 | `cr#"a " b"#` before the same real cast | exit 0 | **No**; the cast vanishes (W-3) |

### 1.4 The eight mechanical passes

1. **Budgets.** 101 rows, B1 to B101, no gap, no B id without a row. Only B62 is cited nowhere
   but ADR 0006, which the previous review already recorded.
2. **Rule ids.** No gap. The families are DR1 to DR6, PL1 to PL3, VR1 to VR6, TH1 to TH10, and SM0 to
SM7. The guard families are PG1 to PG22 with PG4b and PG10b, and CG1 to CG8 with CG2b, CG3b, and
CG4b. PP and CP mirror them.
3. **Chunk coverage.** 55 chunks, every one in 13.1 or 13.2 and in 13.3, with no phase mismatch.
4. **SM6.** No line runs two chunks in one phase. The trunk is four lines of one chunk each,
   which 13.1 states.
5. **The plan graph.** I expanded every cross-line link of 13.4 against the 13.3 phases. No link
   runs backward. One same-phase link, M0 before T1, which 13.4 states.
6. **Write scopes.** The only shared names per phase are `Cargo.lock`, `src/lib.rs`, and `tests/`.
   SM5 rule 4 resolves the first, and the other two are relative to two crate directories.
7. **MUST stories.** 64 MUST rows in the product requirements and 64 rows in 13.2. The two sets
   are identical.
8. **DR3.** Every unit-bearing literal outside section 1.6 is a type size in bytes or a platform
   sample rate. Both are stated exemptions. Three of the six byte counts are wrong; see C-4.

---

## 2. New findings in revision 10

### CRITICAL: C-1. 81 declarations trip `clippy::derive_partial_eq_without_eq`, which `nursery = "deny"` makes a build error

**OBSERVATION.** The root `Cargo.toml` sets `nursery = { level = "deny", priority = -1 }` and
relaxes exactly two nursery lints, `multiple_crate_versions` and `redundant_pub_crate`.
`clippy::derive_partial_eq_without_eq` is a nursery lint. It fires on a type that derives
`PartialEq` and could derive `Eq`.

VR1 reads: "`Eq`, `Hash`, and `Ord` appear only where this rule names the use"
(`architecture.md:2015`). Revision 10 **removed** `Eq` from six more types to satisfy PG22:
`ElementRef`, `RepeatSide`, `NodeIndex`, `BackendName`, `TransactionName`, and `Velocity`
(`architecture.md:2052`).

I compiled four of the declared shapes against the real lint table on the pinned toolchain.

```
error: you are deriving `PartialEq` and can implement `Eq`
 --> crates/probe/src/lib.rs:7:30
  = note: `-D clippy::derive-partial-eq-without-eq` implied by `-D clippy::nursery`
```

The four shapes were `RepeatSide` (a plain enum), `GainDb` (a newtype over a hand-`Eq` type),
`DeviceLabel` (a `Box<str>` newtype), and `ImportWarning` (three `Box<str>` fields). All four
failed.

**CLAIM.** 81 of the 368 declarations do not build. VR1 and the workspace lint policy are
mutually exclusive as written, and revision 10 widened the collision.

**ARGUMENT.** I ran the guard's own parser over every declaration. I took each type that derives
`PartialEq`, derives no `Eq`, and carries no hand `impl Eq`. I then asked the guard's own
`node_traits` whether every field supplies `Eq`. **81 types answer yes**, so the lint fires on
every one of them.

The list crosses every crate and every line of the plan.

- `duet-time` (T1): `Tuplet`, `I24`, `GainDb`.
- `duet-score` (T2): `Selection`, `TieState`, `ElementRef`, `CanonicalDocument`, `ScoreSelector`.
- `duet-session` (T3): `BusRole`, `InputSelection`, `AudibleState`, `Source`, `ParamRange`,
  `CurvePoints`, `SessionEvent`, `SlotPosition`, `StripTarget`, `TakeFlags`, `SessionDocument`,
  `MixDocument`.
- `duet-command` (T4): 18 types.
- `duet-dsp` (D1 and D3): every state type.
- `duet-engine` (C1 and C3): nine types.
- `duet-midi` (G1): `HotplugEvent`, `PlatformPort`, `MidiError`, `MidiEvent`.
- `duet-project` (F1): `HeadState`, `TrackedFile`, `HistoryError`.
- `duet-core` (I1): `TransactionName`.
- `crates/duet` (K1 and K4): `AppEvent`, `Fader`, `Toolbar`, `ModeToolbar`, `PunchRange`,
  `DurationKeySet`.

Two answers exist and the document states neither.

The first answer restores `Eq` everywhere the compiler allows it. That deletes the reason for
VR1's own table, because the table exists to keep `Eq` rare. It also deletes the reason for PG22,
which revision 10 added to hold that table. The design then needs a different rule for `Hash` and `Ord` alone. ADR 0001's rejected-
alternatives row about a demand that points upward needs a rewrite too.

The second answer adds `derive_partial_eq_without_eq = "allow"` to `[workspace.lints]`. CLAUDE.md
states the verdict on that: "An edit to `[workspace.lints]`, `clippy.toml`, or the `deny.toml`
ignore list made to get past the gate is a defect that escalates, never a resolution."

No guard rule sees this. PG19 reads a derive against a field and passes, because the derives are
internally consistent. PG22 reads the derives the table names and passes. MINE-P9 confirms it: I
planted `#[derive(Debug, Clone, Copy, PartialEq)] pub struct ProbePartialEq { value: Ticks }` and
the guard exited 0 with every counter at zero.

**EVIDENCE.** `Cargo.toml` `[workspace.lints.clippy]` lines for `nursery`,
`multiple_crate_versions`, and `redundant_pub_crate`; `architecture.md:2015` (VR1),
`:2052` (the six drops); the compiler run above; my scan of all 368 declarations through
`placement_check.py`'s own `node_traits`.

**WHAT SHOULD CHANGE.** Decide which answer the design takes, and write it once. If VR1 keeps its present shape, the policy needs an Orchestrator decision with its own pull
request. B.1 then needs a row that states the removal condition. If the policy stands, restore `Eq` on every type the compiler allows. Narrow VR1 to `Hash` and
`Ord`, rewrite PG22 to read those two, and give the new rule a probe. Then add a guard rule that
reads a `PartialEq` derive against the field set. That rule stops the next revision from reopening
the class.

### CRITICAL: C-2. `SlotState` does not build, because `variant_size_differences` refuses it

**OBSERVATION.** Section 5.5 declares `SlotState` as an enum over six state types. Section 15.2
declares each of the six. I built all seven declarations verbatim and measured them.

```
  EqualizerState = 224
 CompressorState = 48
       GateState = 24
    DeEsserState = 72
      DelayState = 32
     ReverbState = 32
       SlotState = 232
```

The compiler run under the real lint table gives:

```
error: enum variant is more than three times larger (224 bytes) than the next largest
   --> crates/probe/src/lib.rs:115:5
```

**CLAIM.** `SlotState` is a build error. Chunk D3 writes it and stops.

**ARGUMENT.** `variant_size_differences` is a rustc lint set to `warn` in
`[workspace.lints.rust]`, and `.cargo/config.toml` adds `-D warnings`. That is the exact
mechanism VR5 relies on for `missing_copy_implementations`, and ADR 0005 decision 3 names the
same lint for `Verb`. `EqualizerState` is `[BiquadState; 4]`, and `BiquadState` is seven `Finite`
fields, so the arm is 224 bytes. The next largest arm is `DeEsserState` at 72 bytes. The rule
fires when the largest arm passes three times the second, and 224 is above 216.

ADR 0005 decision 3 states the answer the design already uses for the same lint: box the large
payload. `Verb` boxes every large payload for this reason. `SlotState` boxes nothing, and
`ArrayVec<SlotState, MAX_SLOTS>` then costs 1856 bytes per slot list.

The same measurement breaks a second claim. B51 reads "512 B | One `SlotState` with no buffer".
The real value is 232 bytes. B55 and B56 therefore rest on a number that is more than twice the
truth, in the safe direction.

No guard rule reads a size. PG9, PG10, PG10b, and PG21 all read a name or a derive.

**EVIDENCE.** `architecture.md:3159` to `:3174` (`SlotState`), `:6699` to `:6755` (the six state
types), `:644` (B51), `adr/0005-agent-gateway.md` decision 3; the two runs above.

**WHAT SHOULD CHANGE.** Box the equalizer arm, or hold the four biquad sections behind a pool
handle as the delay and the reverb already do. State the new measured size at B51 and re-derive
B55 and B56. Then state at VR5's site that `variant_size_differences` binds every enum in the
workspace, not only `Verb`.

### CRITICAL: C-3. Bundle creation writes a version-1-only git extension at repository format version 0

**OBSERVATION.** Section 4.7 gives the repository-local configuration that bundle creation writes.

```
architecture.md:2672   # repository-local config
architecture.md:2673   core.autocrlf=false
architecture.md:2674   core.safecrlf=false
architecture.md:2675   extensions.objectFormat=sha1
```

The block sets no `core.repositoryFormatVersion`, so the repository stays at version 0.

**CLAIM.** `extensions.objectFormat` is a version-1 extension. At version 0 git refuses to open
the repository at all, and section 4.6 makes a user who runs plain `git` a supported path.

**ARGUMENT.** Git's own `Documentation/technical/repository-version.txt` states the rule: a
version-0 repository carries no extension, and `objectFormat` is listed under the version-1
extensions. Git's `verify_repository_format` reports "repo version is 0, but v1-only extensions
found" and returns a failure, so every `git` command inside the bundle stops. `git init
--object-format=sha1` writes no such key; only `--object-format=sha256` writes the key and raises
the version to 1.

Three consequences follow. Every bundle this build creates is unreadable by plain `git`, which
breaks section 4.6 and story A-03. The test that section 4.7 names, which "writes a bundle with
the real `git` binary, reads it with gix, and asserts the bytes match", fails. Chunk F1 writes
the block and stops.

The second half is a gap rather than a defect. `History::object_format` reads
`extensions.objectFormat`. In a correct SHA-1 repository that key is **absent**. The trait documentation names one failure
only: "Returns `HistoryError::Read` when the configuration cannot be read." An absent key is not
an unreadable configuration. No sentence says that the absent case answers `HashKind::Sha1`. `ProjectOpen` then refuses every bundle this design creates correctly.

I did not run `git`, because the brief forbids any other git command. The Architect reproduces it
with two commands in a temporary directory: `git init` and then `git config
extensions.objectFormat sha1`, followed by `git status`.

**EVIDENCE.** `architecture.md:2671` to `:2680` (4.7), `:2591` to `:2603` (`object_format`),
`:2626` to `:2628` (the open refusal), `:2642` to `:2654` (4.6);
`adr/0003-history-model.md` decision 10.

**WHAT SHOULD CHANGE.** Delete the `extensions.objectFormat` line, because SHA-1 is the default
and a version-0 repository states it by omission. Then state in 4.5 that
`History::object_format` returns `HashKind::Sha1` when the key is absent, and add that sentence
to the method's own documentation. If the design wants the key written, it must also write
`core.repositoryFormatVersion=1`, and 4.7 must say so.

### CRITICAL: C-4. Three of the six stated type sizes are wrong, and one of them makes an approved suppression rest on a false invariant

**OBSERVATION.** DR3's fourth exemption reads: "A **type size in bytes** is a compiler fact that
VR5 reads" (`architecture.md:41`). VR5's table and Appendix B.1 state six such facts. I built
every one of the six declarations verbatim and measured them.

| Type | Stated | Measured | Verdict |
|---|---|---|---|
| `Source` | 64 bytes | 64 | correct |
| `ManifestEntry` | 56 bytes | 56 | correct |
| `ExportSpec` | 32 bytes | 32 | correct |
| `Transport` | 120 bytes, "two `Option<Span>` are 40 each" | **104**, and each `Option<Span>` is **32** | wrong |
| `SlotState` | 64 bytes, "a compressor holds six `Finite` values" | **232** | wrong |
| `ConfigError` | 40 bytes, "the `Mix` arm carries a `MixError`" | **24** | wrong, and it inverts the rule |

**CLAIM.** Three stated compiler facts are not facts. `ConfigError` at 24 bytes is **below** the 32-byte bound. VR5 therefore demands the `Copy` derive,
and the `#[expect]` reason is false.

**ARGUMENT.** VR5 names three refusals, and the reason text must name which one applies. The
reason for `ConfigError` is the third: "it is already larger than the size bound". At 24 bytes it
is not. `MixError` is 24 bytes and derives `Copy`. Every arm of `ConfigError` is therefore `Copy`, and the
whole enum fits in one machine word plus a tag. VR5's own sentence therefore requires the
derive, and B.1 carries a row for a suppression the rule does not sanction.

That is the "WRONG suppression" class in full. An `#[expect]` whose reason states a false invariant is worse than no suppression. B.1 is the list
the Orchestrator adjudicates before dispatch, and the reason is what the Orchestrator reads.

`Transport` keeps a valid reason, the second refusal, and states a false number beside it.
`SlotState` states a number that is wrong by a factor of 3.6, and C-2 shows the type does not
build at its real size.

No rule reads a size. MINE-P10 confirms it: I changed the `ConfigError` reason text from 40 bytes
to 9000 bytes and the guard exited 0 with `B.1 BAD: 0`. PG21 compares names.

**EVIDENCE.** `architecture.md:41` (DR3), `:2113` to `:2121` (the VR5 answer table), `:2487` to
`:2491`, `:3160` to `:3163`, `:3586` to `:3590`, `:3758` to `:3762`, `:7537` to `:7541` (the five
reason texts), `:7965` to `:7971` (B.1); my six measured sizes above.

**WHAT SHOULD CHANGE.** Give `ConfigError` the `Copy` derive, delete its B.1 row, and set the
total at B.1 from seventeen to sixteen. Correct the `Transport` number to 104 and delete the
`Option<Span>` clause, because a `Span` is 32 bytes and the niche in `Position` costs nothing.
Correct `SlotState` after C-2 changes its shape. Then give each of the five sites a verification step in its own chunk. The step is a `size_of`
assertion in a test, in the shape section 8.3 already gives `MidiRecord` and `NoteEntry`.

---

### WARNING: W-1. The stale meter snapshot restores every cleared over mark one frame later

**OBSERVATION.** Section 7.3 states the stopped-engine path. "In `EngineState::NoServer`,
`OpenTimeout`, `NoDevice`, `RateUnavailable`, and `Faulted` no audio cycle runs, so nothing reads
the generation and **the last published `MeterSnapshot` keeps its marks**"
(`architecture.md:4174`).

The paragraph below it states the algorithm. "`MeterLayer` keeps its own `shown: HeldMarks`:
**each frame it adds the marks the `MeterSnapshot` reports**, and on the event it removes the
strips the core named. The view therefore paints no `CLIP` for a cleared strip whether or not a
cycle runs" (`architecture.md:4179`).

**CLAIM.** The two sentences contradict each other. While the engine is stopped, the snapshot
still reports the mark, so the next frame adds back every bit the event removed. The `CLIP` never
clears, which is the one case the second path exists for.

**ARGUMENT.** Follow one strip. The audio thread set `over[i]` and published the snapshot. The
engine then stopped. The triple buffer holds that value forever, because its writer is the audio
thread. `Verb::MeterReset` reaches the core, the core increments the generation, and it emits
`CoreEvent::HeldMarksCleared { strips }`. `MeterLayer` clears bit `i` of `shown`. On the next
frame `MeterLayer` reads the same stale snapshot and sets bit `i` again. Design contract 4.4
requires the opposite: "The state holds until the user clicks the meter or presses the `Clear
clip` command."

`MeterLayer` cannot detect the stale value. `TransportSnapshot` carries a `generation` field and
`MeterSnapshot` carries none: its three fields are `readings`, `over`, and `live`. So the view has
no way to tell a fresh mark from a mark the engine left behind before it stopped.

The same defect has a smaller form while the engine runs. The audio thread applies the generation
at its next cycle, which is inside B7, and the view paints at B2. One frame can fall between the
event and the cycle, so the mark can flash back once. The stated claim, "neither path loses a
reset", is true for the audio thread and false for the view.

Responsive and Message Driven both suffer. The user presses a command and the interface undoes
its own answer without a fault or a message.

**EVIDENCE.** `architecture.md:4153` to `:4188` (7.3), `:3630` to `:3642` (`MeterSnapshot`),
`:7151` to `:7158` (`HeldMarks`), `:7249` to `:7251` (`HeldMarksCleared`), `:7909` to `:7910`
(Appendix A), `design-contract.md:764` to `:767`.

**WHAT SHOULD CHANGE.** Give `MeterSnapshot` a `generation: Generation` field, in the shape
`TransportSnapshot` already has. Make `MeterLayer` add a mark only from a snapshot whose
generation moved since the last frame. State that rule in 7.3 and in Appendix A. Then add one
required test to 10.7 rung three. It clears a clip while `EngineState` is not `Running`, runs ten
frames, and asserts that no `CLIP` returns.

### WARNING: W-2. Nothing wires `TopBar` to the four per-mode toolbar files, so each stub is unreachable

**OBSERVATION.** Section 13.2 states the seam. "K1 writes `top_bar.rs`, the `Toolbar` element that
carries the overflow rule of contract 1.5, and the four per-mode stubs. K2, K3, K4, and K5 each
fill exactly one stub and **none of them edits `top_bar.rs`**, which sits in K1's write scope and
in no other" (`architecture.md:6247`).

SM2 defines a stub. "A stub holds the `//!` module documentation and nothing else, so it compiles
under the full lint set" (`architecture.md:5976`).

**CLAIM.** `TopBar` cannot call a stub that holds only documentation, and no chunk may edit
`top_bar.rs` after K1. The four toolbars are therefore dead code, and `dead_code` is a rustc
lint that `-D warnings` makes an error.

**ARGUMENT.** `TopBar` "composes exactly one `ModeToolbar`" per mode (`architecture.md:7802`), and
"each mode's groups live in their own file under `src/shell/`". For K1's `TopBar` to compose the
Compose groups it must name an item inside `toolbar_compose.rs`. At K1 that file holds no item, so
the call does not compile and K1 fails its own Completion command.

If K1 instead writes no call, then K2 fills `toolbar_compose.rs` with an item that nothing uses.
`shell.rs` carries the `mod` line, so the item is reachable from the module tree and unused from
the call graph. `dead_code` fires, and K2 fails its own Completion command.

The third path is that K1 creates each stub with a function signature, which SM2 forbids by its
own sentence.

This is finding R9 and finding W5 in a third form. Revision 8 sent four chunks to one file that no
chunk gave them. Revision 9 declared the type in one doc comment. Revision 10 gives each chunk its
own file and names no owner for the wiring between them.

**EVIDENCE.** `architecture.md:5975` to `:5979` (SM2), `:6212` to `:6217` (K1 to K5),
`:6219` to `:6250` (the shell-file table and the seam paragraph), `:7802` to `:7804` (`TopBar`),
`:7871` to `:7879` (`ModeToolbar`).

**WHAT SHOULD CHANGE.** State one of two rules.

1. K1 writes a `match Mode` in `top_bar.rs`. It creates each stub with a `groups` function that
   returns an empty list, and SM2 gains a stated exception for a seam function.
2. `top_bar.rs` enters the write scope of K2 to K5 as well. Section 13.3 then accepts that five
   chunks share one file across five phases.

Name the choice in 13.2 and in SM2.

### WARNING: W-3. A raw byte string hides a real cast, and the one-pass lexer claims it cannot

**OBSERVATION.** CG2b reads: "A comment token inside a literal opens no comment, and a quotation
mark inside a comment opens no string. **This is the half CG2 states and a three-pass scrub cannot
hold** ... The one pass removes the class by construction" (`architecture.md:1238`).

CG2 reads: "**A raw string ends at the quotation mark that carries its own opening hash count**, so
`r##"say "as" here"##` is one literal and not two" (`architecture.md:1228`).

I ran a control and two probes in a throwaway workspace.

```
CONTROL   let s = r#"a " b"#;   then  let n = x as u32;   ->  FINDINGS: 1, exit 1
MINE-C17  let s = br#"a " b"#;  then  let n = x as u32;   ->  FINDINGS: 0, exit 0
MINE-C18  let s = cr#"a " b"#;  then  let n = x as u32;   ->  FINDINGS: 0, exit 0
```

**CLAIM.** The lexer recognises `r"` and `r#"` and does not recognise `br#"` or `cr#"`. The `r`
sits inside an identifier, so the pass opens an ordinary string at the first quotation mark. A
real cast then falls inside a false string and vanishes. That is a vacuous green: the guard reports
success it did not earn.

**ARGUMENT.** The control proves the pass handles the shape without the prefix. The two probes
differ from the control by one byte each. Both are ordinary Rust: a raw byte string and a raw C
string are stable forms, and edition 2024 is what this workspace pins.

MINE-C2 shows the false-red direction of the same defect: `br##"say "as" here"##` reports a cast
on the literal's own line. CG3b states the guard prefers a false red to a miss. MINE-C17 and
MINE-C18 are the miss, so the preference does not hold.

**The gate as a whole does not fail open, and I say so plainly.** The root `Cargo.toml` sets
`as_conversions = "deny"` and `scripts/dod.sh` runs clippy with `-D warnings`. The defect is that
CG2 and CG2b each claim a property the machine does not hold, and that CP2 and CP2b cover neither
prefixed form. Revision 9 failed on this class in a different shape. Revision 10 closed two instances of it and
left a third in the same function.

**EVIDENCE.** `architecture.md:1228` to `:1243` (CG2 and CG2b), `:881` to `:882` (CP2 and CP2b),
`conversion_check.py` raw-string handling; the control and the two probes above.

**WHAT SHOULD CHANGE.** Let the lexer read a string prefix. A `b`, a `c`, or a `br`, `cr`, or `rb`
run that ends at a `"` or at a `#` opens the matching literal form. State the prefix set at CG2's
site. Then replace CP2b with a probe that plants `br#"a " b"#` and `cr#"a " b"#` before a real
cast, and assert that the cast is still found.

### WARNING: W-4. PG20 is blind to a trait-object field, so the cycle it was built to catch passes

**OBSERVATION.** PG20 reads: "Every field type of every declaration sits in a crate that the
section 1.3 graph reaches from the declaring crate" (`architecture.md:521`). It exists to close
S2, which was blocker V1 reopened.

I planted `pub struct ProbeDyn { handler: Box<dyn AudioProcess> }` in `duet-command` and placed it
in the 1.5 table. `AudioProcess` is a `duet-engine` trait, and `duet-engine` depends on
`duet-command`, so the reference is a Cargo cycle.

```
REACH BAD:       0
  UNJUSTIFIED:ProbeDyn.AudioProcess is in no section 1.9 row
```

I then asked the guard's own parser:

```
parse_type("Box<dyn AudioProcess>")  ->  ('name', 'Box', [None])
leaf_names(...)                      ->  {'Box'}
```

**CLAIM.** `parse_type` returns `None` for every `dyn Trait` argument, so `leaf_names` never sees
the trait name and PG20 never asks about it. The rule that closes the design's oldest blocker has
a hole in the exact shape that blocker takes.

**ARGUMENT.** The guard is loud today, but for the wrong reason and through the wrong rule. PG17
complains that the trait name has no justified-unknown row. An author who reads that message adds
a row to the 1.9 justified-unknown table, which is the documented answer to a PG17 failure. The
Cargo cycle then passes in silence, with a `UNKNOWN` count of two and a green exit.

The document already holds one `dyn Trait`: `AudioBackend::open` takes
`process: Box<dyn AudioProcess>`. That is a method signature, which PG7 and PG19 both exclude by
their own stated limits. A field is not excluded, and PG20 states no limit at all.

VR2 forbids a trait object in a vocabulary type, and no rule reads one either. The compile-time
assertion of 3.5 would catch a `Box<dyn AudioProcess>` inside `Verb`, because a trait object is
not `Serialize`. It would not catch one inside a type the assertion does not list.

**EVIDENCE.** `architecture.md:521` to `:527` (PG20), `:877` (PP20), `:2072` (VR2),
`:2821` (`AudioBackend::open`); `placement_check.py` `parse_type`; the probe run above.

**WHAT SHOULD CHANGE.** Make `parse_type` strip a leading `dyn` and a leading `impl`, and return
the trait path as a name. Add the limit sentence to PG20's site: name every form the parser
declines. Then add a probe that plants a `dyn Trait` field across an absent edge and assert
`REACH BAD: 1`.

### WARNING: W-5. The first-open geometry is the contract's minimum, and the contract's narrow breakpoint then refuses the stored widths

**OBSERVATION.** B91 is 1024 logical pixels and B92 is 700, and each row cites contract 1.1. B94
is 240 and B97 is 300.

Design contract 1.1 states two different numbers. "Minimum window size: 1024 x 700 px ... **Default
window size: 1440 x 900 px**" (`design-contract.md:46` to `:48`).

The same section gives the breakpoints. "**narrow**, 1024 px to 1199 px: sidebar collapses to a 48
px icon rail, the inspector becomes a `Sheet`" (`design-contract.md:56`).

**CLAIM.** Duet opens at the contract's minimum, not at the contract's default. At that width the contract puts the shell in the narrow breakpoint. The sidebar is then 48 pixels
and the inspector is a sheet. B94 and B97 therefore name two values the first frame cannot use.

**ARGUMENT.** Revision 9 opened below the minimum and revision 10 raised the value to the minimum.
A minimum is the floor a window may reach; a default is the size it opens at. The contract states
both, and the architecture reads only the first.

The arithmetic makes the second half concrete. Contract 1.6 gives the work area a minimum of 560
pixels. In Mix the inspector is visible by default, so the first-open width is 1024 minus 240
minus 300, which is 484. That is below the stated minimum, so `ModeView::first_open(Mix)` names a
layout the contract refuses. In Compose the inspector is hidden, so 1024 minus 240 is 784 and the layout fits. The sidebar is
still 240 where the breakpoint says 48.

`ProjectView::clamped` is the stated repair, and 10.2 binds it to one case: "`ViewStateStore` calls
`clamped` on restore, so a project saved on a large display opens usable on a small one". A first
open is not a restore, and no sentence says `clamped` runs on the value `first_open` builds. A
clamp is also not a breakpoint: contract 1.1 replaces the sidebar with a rail rather than
narrowing it.

**EVIDENCE.** `architecture.md:684` to `:691` (B91, B92, B94, B97, B98), `:5243` to `:5263` (the
first-open field table), `:5279` to `:5289` (`clamped`), `design-contract.md:44` to `:58`,
`:113` to `:130`.

**WHAT SHOULD CHANGE.** Set B91 and B92 to the contract's default, 1440 by 900. State in 10.2 that `ViewStateStore` applies the breakpoint rules of contract 1.1 before any stored
width. That rule binds a first open as well as a restore. Then add one required test to 10.7 rung
three. It opens at each of the three breakpoints and asserts the sidebar form.

### WARNING: W-6. `LogicalPx` has no accessor, and every pixel value crosses a denied cast with no named site

**OBSERVATION.** Section 10.2 declares the type and its one method.

```
architecture.md:5111   pub struct LogicalPx(Finite);
architecture.md:5116   pub const fn new(value: Finite) -> Self;
```

`Finite` wraps an `f64`. `gpui_kit::Pixels` is backed by an `f32`. Section 1.6 states that chunk
K1 substitutes `FIRST_OPEN_WIDTH`, `FIRST_OPEN_HEIGHT`, and `FIRST_OPEN_ORIGIN` for the three
literals in `main_window_options`.

**CLAIM.** `crates/duet` cannot read a value out of a `LogicalPx`, and the narrowing it needs is
a cast that `clippy::as_conversions` denies outside one module.

**ARGUMENT.** Three call sites need the value. `main_window_options` builds a `Bounds<Pixels>`
from three constants. `ProjectView::clamped` compares a stored width with `Bounds<Pixels>`.
`ViewStateStore` applies a sidebar width and an inspector width to a resizable panel.

Every one of them needs `f64` to `f32`. The one sanctioned path is
`duet_time::convert::f64_to_f32`, which returns `Result<f32, TimeError>` by rule 2 of section 2.3.
`main_window_options` returns a `WindowOptions`, not a `Result`, and `unwrap_used` and
`expect_used` are both denied. So K1 must write a fallback value, and DR3 forbids a literal in any
Rust block outside section 1.6.

The type also has no getter at all. `Finite::get` exists; `LogicalPx` exposes nothing, and 5.1
states that no public field appears on any type. So the declaration of record cannot be consumed
as written.

Both halves are cheap to fix and neither is stated. B.1 predicts seventeen suppression sites and
names no conversion outside `duet-time::convert`, so an implementer who reaches for one is outside
the budget.

**EVIDENCE.** `architecture.md:5109` to `:5117` (`LogicalPx`), `:771` to `:774` (the K1
substitution), `:1114` to `:1116` (the two conversion rules), `:1164` (`f64_to_f32`),
`:2878` (the public-field rule), `:5279` to `:5289` (`ProjectView`).

**WHAT SHOULD CHANGE.** Declare `LogicalPx::get(self) -> Finite`. Declare one total conversion the
application may call, for example `LogicalPx::to_f32(self) -> f32`. Put its body in `duet-
time::convert`, and state in its documentation why the value is always in range. Name the
conversion site in 10.2. If the site needs a suppression, add a row for it to B.1.

### WARNING: W-7. `LevelMeter` and `MeterLayer` both paint the strip meter and both hold the clip state

**OBSERVATION.** Section 10.3 gives `LevelMeter` a job: "Peak, RMS, and clip in one bar"
(`architecture.md:5363`). Section 15.16 declares it with live per-frame fields.

```
architecture.md:7852   pub(crate) struct LevelMeter { reading: MeterReading, over: OverMark, strip: StripId }
```

Section 10.2 gives `MeterLayer` the same job: "Every strip meter, in one canvas"
(`architecture.md:5080`). Design contract 4.4 agrees with 10.2: "Every strip meter paints in one
canvas owned by a `MeterLayer` leaf view that spans the strip row."

**CLAIM.** Two elements paint one thing, and the held over mark lives in two places. Chunk K4
writes both.

**ARGUMENT.** `MeterLayer` is the one frame driver of Mix, so it reads a fresh `MeterSnapshot`
every frame. `LevelMeter` is a `RenderOnce` element inside `StripView`, so it is rebuilt only when
the strip view renders. A `reading` field on a `RenderOnce` element is therefore stale chrome that
updates at an unrelated rate.

The clip state is worse, because it is stateful. Appendix A names `Entity<MeterLayer>` as the owner
of "the over marks the user interface paints", as a `HeldMarks` bitset. `LevelMeter` carries its
own `over: OverMark`. Two owners for one piece of state is the defect Appendix A's own table exists to remove. W-1 shows
that the value is already hard to keep correct with one owner.

**EVIDENCE.** `architecture.md:5074` to `:5090` (the frame drivers), `:5363` (the element row),
`:7850` to `:7852` (`LevelMeter`), `:7909` to `:7910` (Appendix A),
`design-contract.md:773` to `:776`.

**WHAT SHOULD CHANGE.** Decide which element paints the live bar and give the other one the static
chrome. If `MeterLayer` paints every bar, remove `reading` and `over` from `LevelMeter`. Change its 10.3
row to name the scale ticks, the numeric readout, and the frame. Then state in Appendix A
that `MeterLayer` is the one owner of both the reading and the mark.

### WARNING: W-8. ADR 0004 decision 13c contradicts section 8.1 on which thread drains the B87 queue

**OBSERVATION.** Section 8.1 states the path in five numbered steps. "**The listener thread pushes
a `PlatformPort` into a bounded `crossbeam::queue::ArrayQueue<PlatformPort>` at B87** ... **The MIDI
thread drains that queue and calls `insert` or `retire`** ... It then pushes one `HotplugEvent`
into a second bounded queue at B100, core bound. **`duet-core` drains the B100 queue once per
frame**" (`architecture.md:4440` to `:4449`).

ADR 0004 decision 13c states: "The listener thread pushes into a bounded `ArrayQueue` at B87;
**the core drains it on the frame**; on overflow the queue keeps the newest event and sets a
resync flag" (`adr/0004-backend-and-threading-contract.md:172`).

**CLAIM.** The record puts `duet-core` on the B87 queue, which is the two-owner defect W6 named.
It also calls the B87 payload an event, and B87 carries a `PlatformPort`.

**ARGUMENT.** DR6 exists for exactly this: "A decision record cites a section; it never carries a
second copy of one." Decision 13c carries a second copy, and revision 10 rewrote four other
decisions to citations and left this one. An implementer of chunk G1 who reads ADR 0004 builds a
two-queue path with the wrong consumer on the first queue.

A second DR6 breach sits in ADR 0001 decision 10. It restates the conversion-guard contract and
says "A `use` rename is the one excluded form". CG5 adds qualified path syntax as a second
excluded form, so the restatement is a revision behind.

A third statement disagrees about a condition rather than a mechanism. ADR 0004 decision 7 starts
the native backends "when a user needs a two-device record pass". Section 1.3 and section 5.4 both
start them on the drift measurement at B70, and ADR 0004 decision 6 agrees with the sections.

**EVIDENCE.** `architecture.md:4440` to `:4457` (8.1), `:3558` to `:3559` (the two 5.8 rows),
`:680` and `:693` (B87 and B100); `adr/0004-backend-and-threading-contract.md:172`, `:86`, `:61`;
`adr/0001-time-kernel-representation.md:73` to `:79`.

**WHAT SHOULD CHANGE.** Rewrite ADR 0004 decision 13c to cite section 8.1 and state the decision
only. Rewrite ADR 0001 decision 10 to cite section 2.3. Make ADR 0004 decision 7 cite the B70
condition that decision 6 already states. Then run one DR6 pass over all six records and delete
every remaining restatement, because DR6 exists to make that pass unnecessary.

### WARNING: W-9. `from_finite_const` adds a panic primitive to `duet-time`, and section 12.3 names four house forms that do not include it

**OBSERVATION.** Section 12.3 reads: "`unwrap_used`, `expect_used`, `panic`, `todo`,
`unimplemented`, `unreachable`, and `indexing_slicing` are denied, and **four house forms replace
them**" (`architecture.md:5806`). The four are `.get` with a `let ... else`, a checked division,
`duet-time::convert`, and `ExitCode`.

Section 2.6a declares a fifth. "This `const fn` **asserts** instead ... the function needs no panic
primitive on any path a binary takes (section 12.3)" (`architecture.md:1435` to `:1439`).

I compiled the function verbatim under the real lint table.

```
error: docs for function which may panic missing `# Panics` section
   --> crates/probe/src/lib.rs:30:5
30 |     pub const fn from_finite_const(value: f64) -> Self {
   = note: `-D clippy::missing-panics-doc` implied by `-D clippy::pedantic`
```

**CLAIM.** `assert!` is a panic path. The build refuses the function until its documentation
carries a `# Panics` section, and that section states the panic the design says does not exist.

**ARGUMENT.** The claim "on any path a binary takes" holds only while every caller is a `const`
item. Nothing in the type system or the lint policy enforces that. A `const fn` is an ordinary
function at run time, and `[profile.release]` sets `panic = "abort"`, so one run-time call with a
non-finite value ends the process.

The lint proves the point without any argument from me. `clippy::missing_panics_doc` is in
`pedantic`, which the workspace denies, and it fires on the declared body. The fix is one documentation section. The document then carries a `# Panics` section in the crate
whose ADR says "An overflow cannot end the process, because no bare arithmetic exists on a time
value".

`clippy::panic` does not fire, because the root macro is `assert` and not `panic`. I checked that
by compiling. So the lint policy admits the form and the design's own rule does not name it.

**EVIDENCE.** `architecture.md:1433` to `:1446` (`from_finite_const`), `:5804` to `:5812` (12.3);
`Cargo.toml` `[workspace.lints.clippy]` `pedantic`; the compiler run above.

**WHAT SHOULD CHANGE.** Add the assertion to section 12.3 as a fifth house form. State the one
condition that makes it sound: a `const` item is the only caller. Give `from_finite_const` a `#
Panics` section in section 2.6a. Then state in B.1 whether the site needs a row. B.1 predicts
seventeen sites and this one is outside them.

---

## 3. Concerns

### CONCERN: K-1. Section 15 declares no documentation on any field and on almost any variant

`missing_docs` is a rustc lint at `deny` and `clippy::missing_docs_in_private_items` is denied. I
compiled `ImportWarning` verbatim and got three errors, one per undocumented field. Nearly every
struct and enum in section 15 carries the same shape. Section 15's preamble calls each block "the
declaration of record", and CLAUDE.md states the documentation rule elsewhere, so an implementer
will add the lines. The plan should say so once, in the section 15 preamble.

### CONCERN: K-2. `clippy::missing_const_for_fn` refuses the declared accessors

I compiled `Finite::get` verbatim and got `this could be a `const fn``, implied by
`-D clippy::nursery`. Section 2 declares about forty accessors as plain functions. Each one needs
`const` or a suppression. The fix is mechanical and the budget names none of it.

### CONCERN: K-3. `ResetGenerations::new` trips `clippy::new_without_default`

Section 5.6 states that the type "derives no `Default`" and "carries a named constructor instead".
I compiled it and got "you should consider adding a `Default` implementation for
`ResetGenerations`". A hand-written `impl Default` that calls `new` satisfies the lint and breaks
no stated rule. Say so at the declaration.

### CONCERN: K-4. A second hidden table of 17 primitives survives inside the guard

Revision 10 closed critic C2. It moved the 70-name drop list into a fenced block the guard reads.
`placement_check.py` still holds `PRIMITIVES`, a table of seventeen primitive names and three
trait sets, which no section states and no probe covers. PG4's site now claims "the guard carries
no hidden set", and that claim is false. I checked the entries I could check and found them
correct, `char: Default` included. Move the table into a fenced block beside the drop list.

### CONCERN: K-5. Three rows of the 1.9 `Traits` column give a wrong or incomplete verdict

`Ordering` and `Path` both sit in one row, with the verdict `not Copy` and no traits. The row
reads "Each one is a trait name or a borrowed type in a signature". `core::cmp::Ordering` is `Copy`, `Eq`,
`Hash`, and `Ord`. `Path` is `PartialEq`, `Eq`, `Hash`, and `Ord`. The `Box` row makes `Default`
conditional on the payload, and I compiled `Box<str>` and `Box<[u32]>` under a `Default` derive:
both build. The `Cow` row omits `Default`, which `Cow` does implement. MINE-P8 shows the
consequence: the guard reds a `Default` derive over `Box<[Finite]>` that the compiler accepts.
No declaration reaches any of these today, so every one is latent.

### CONCERN: K-6. PG19 reds a reference field that is always `Copy`

MINE-P4 planted `pub struct ProbeRef<'a> { text: &'a str }` under a `Copy` derive. The guard
printed `CLOSURE: duet-command::ProbeRef derives Copy over text: &'a str has no Copy`. Every
shared reference is `Copy`. The rule is loud rather than silent, so it is safe, and PG19's site
states no limit for a reference.

### CONCERN: K-7. PG19 passes a `Default` derive over an array longer than 32

MINE-P7 planted `[Finite; 64]` under a `Default` derive and the guard exited 0. PG19 states the
exclusion and names the compiler as the backstop. The stated reason, "`ResetGenerations` is the
one long array here", is true of the current text only, and nothing holds it true.

### CONCERN: K-8. CG5 reds a qualified path whose left side is generic

MINE-C6 planted `<Vec<u8> as Default>::default()` beside a real cast on one line. The guard
reported two findings. CG3b states that the guard prefers a false red, so the behaviour is inside
the policy. CG5's sentence names `<Self as Default>::default` and gives no answer for a generic
self type, and `<ArrayVec<SlotSpec, MAX_SLOTS> as Default>::default()` is a shape this design will
write.

### CONCERN: K-9. A file that is not UTF-8 raises a traceback, not the stated fail-closed exit

MINE-C15 put one byte pair that is not UTF-8 in a member file, beside a real cast. The guard raised an
uncaught `UnicodeDecodeError` and exited 1. CG2 and CG8 both state a fail-closed form: one named
line and exit 2. Exit 1 is the findings code, so a caller that reads only the exit status cannot
tell the two apart. CG8's own site names this class: "Revision 7 raised an uncaught exception,
which was fail-closed by accident and stated by no rule."

### CONCERN: K-10. A crate outside the workspace members list is silently unscanned

MINE-C9 put a crate with a real cast under `crates/`. It left the crate out of `members`. The guard
printed `MEMBERS: 1   FILES: 1   FINDINGS: 0` and exited 0. CG1 draws the file set from `cargo metadata`. Section 2.3 states: "A file set drawn from `cargo
metadata` is empty only when the workspace is empty". That is true for the empty case and not for the partial case. The
repository globs `crates/*` and `tools/*`, and CLAUDE.md states the rule, so the risk is bounded.
Clippy `--workspace` shares the same denominator.

### CONCERN: K-11. `ModeToolbar` is a twelfth element and section 10.3's list names eleven

Section 10.3 states "the list is the contract's eleven plus `StageCurve`". It holds twelve rows,
and one of them is not a GPUI element. Section 15.16 then declares `ModeToolbar` with `IntoElement`,
under `src/shell/` rather than `src/element/`. The element list is therefore not the whole element
list, and nothing checks it.

### CONCERN: K-12. `ModeToolbar::overflow_from` is a fixed cut point and contract 1.5 needs a width-dependent one

The field's documentation reads "the index of the first group the overflow rule of contract 1.5
moves into the menu, which is fixed per mode". Contract 1.5 states that the bar "removes groups
from the trailing end" until the widths fit. The order is fixed per mode; the count follows the
available width. One `u8` cannot express a count that changes with the window.

### CONCERN: K-13. ADR 0006 gives the Compose path cache to `RecordView`

ADR 0006 decision 6 reads "`RecordView` in Compose and Record". Section 10.2's entity tree gives
Compose to `ComposeView`. Appendix A repeats the ADR. Compose paints no waveform, so the cache is
probably unneeded there; the sentence should say so rather than name a view from another mode.

### CONCERN: K-14. `?` does not compose from an upper crate's error to `GatewayError`

Section 12.1 states that "`#[from]` bridges a lower crate's error so that `?` composes". The four
upper crates convert to `UpstreamFailure`, and no `From<EngineError> for GatewayError` impl can
exist: both types are foreign to `duet-core`, so the orphan rule refuses it. Every call site in
`duet-core` therefore writes `map_err`. The `From<Local> for UpstreamFailure` impls are sound; I
checked the orphan rule for all four. Only the `?` sentence is too broad.

---

## 4. Reactive assessment

- **Responsive: PASS.** 101 budgets and eleven timeouts are real, cited by id, and correct in every
  derived row. I recomputed B90 and B86 and both hold. The `JobRunner`, the reserved slots, the `Arc` snapshot, and one frame driver per area keep every
slow verb off the frame path. The one gap is a user-visible answer that the view undoes (W-1).
- **Resilient: FAIL.** 81 declarations do not build under the workspace lint policy, and one enum
does not build at all. The bundle-creation path writes a git configuration that plain `git`
refuses. The save path, the crash matrix, and the four-source live set are correct and complete. So are the
writer lock, the typed error rule, and the MIDI port-loss path. The failure is that the
  guard family reads one markdown file and the lint policy is a second file that no rule reads.
- **Elastic: PASS.** Every channel, ring, queue, store, and cache carries a bound with an id. B86
  and B90 each carry a refusal at two callers, and the arithmetic of both is exact.
- **Message Driven: PARTIAL.** One vocabulary, one writer, versioned documents, published
  snapshots, and no shared mutable state across the user-interface seam. Three paths are wrong. The cleared over mark returns from a stale snapshot (W-1). `LevelMeter` and
`MeterLayer` hold one state twice (W-7). ADR 0004 puts the core on the wrong MIDI queue (W-8).

---

## 5. Plan graph check on section 13

1. **The graph is acyclic and every link runs forward.** I expanded every cross-line link of 13.4
   against the 13.3 phases by machine. No link runs backward.
2. **One same-phase link, and it is the stated exception.** M0 before T1.
3. **Every chunk has a phase and a row.** 55 chunks, 0 gaps, 0 phase mismatches, and every phase
   width equals its chunk count.
4. **SM6 holds in every line.** The trunk is four lines of one chunk each, which 13.1 states.
5. **Write scopes are disjoint per phase.** The only shared names are `Cargo.lock`, `src/lib.rs`,
   and `tests/`, and the last two are relative to two crate directories.
6. **Every chunk has a runnable Completion command, and every filtered one carries
   `--no-tests=fail`.**
7. **SM7 holds.** PG16 passes, and all five selected tests resolve to a chunk in an earlier phase.
8. **The manifest seam holds.** M3 pins `criterion` before A4 in phase 5, and M7 owns
   `audio-smoke.yml` after C4 in phase 6.
9. **MUST story coverage is exact.** 64 of 64, and the two sets are identical.
10. **No line chunk writes a policy file.** M0, M6, and M7 carry them all, and all three go to the
    Orchestrator.
11. **SM0 matches the repository.** `crates/duet/src/` holds `main.rs` and `app.rs`, which is what
    K1 expects. `scripts/dod.sh` runs the ten steps rung one names and does not run
    `check-conversions`, which is what M0 adds.
12. **One ordering defect.** K2 to K5 each fill a file that nothing calls (W-2).

---

## 6. Consistency across the documents

1. **Closed.** Contract 3.1, ADR 0006 decision 2, and architecture 10.5 agree on
   `beat_for_x(x) -> Ticks` and `x_for_beat(beat) -> f32`.
2. **Closed.** Contract 3.3, architecture 10.5, Appendix A, and ADR 0006 all carry two keys.
`ZoomScalar` is a declared and placed type.
3. **Closed.** ADR 0003 and architecture 4.5 agree on the tracked set, the content store, and the
four live-set sources. They also agree on the five save steps, the crash matrix, and the symbolic
pointers.
4. **Closed.** ADR 0001 and architecture 2 agree on B40, B41, and the two kernel newtypes. They also
agree on the absent order, the six named methods, and the `Finite` derive.
5. **Closed.** The platform research, section 5.4, and section 11 agree on cpal 0.18.2 with `default-
features = false, features = ["pipewire"]`. They agree on the two host identifiers, on no ALSA
fallback, and on the two runners.
6. **Closed.** Product requirements section 8 holds 64 MUST rows and 13.2 holds 64. The sets are
   identical.
7. **Closed.** `deny.toml` carries every licence B.4 calls present and does not carry `Unlicense`,
   which B.4 asks M0 to add for `midly`.
8. **Closed.** The root `Cargo.toml` sets `panic = "abort"`, `overflow-checks = true`, and
   `as_conversions = "deny"`, which is what sections 2.2 and 2.3 state.
9. **Open.** Contract 4.4 gives one canvas one owner and architecture 10.3 gives a second element
   the same job (W-7).
10. **Open.** Contract 1.1 gives a default window of 1440 by 900 and B91 and B92 give the minimum
    (W-5).
11. **Open.** ADR 0004 13c and ADR 0001 10 each state a rule the architecture replaced (W-8).
12. **Closed.** The `StageCurve` appendix names `duet.meter.peak_cap`, which contract 7.2 holds,
    and chunk K4 owns the file.

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

Revision 10 is the best revision of the ten on the axis it set out to fix. It is the first one
whose guard family reads a declaration rather than a name. PG19 closes the derive families, PG20
reads every field type against the 1.3 graph, PG21 holds Appendix B.1 in both directions, and PG22
holds VR1. All 35 recorded probes reproduce, the baseline prints `DERIVE CLOSURE: 0` and
`REACH BAD: 0`, and 21 of the 28 revision-9 findings close by mechanism. The five new 1.3 edges make
no cycle. B90's arithmetic is exact. The plan graph passes nine tests.

Four Criticals block it, and they share one cause. **The guard reads `architecture.md` and nothing else. The file that refuses this design is
`Cargo.toml`.** 81 declarations trip a nursery lint that VR1's own rule creates. `SlotState` trips
a rustc lint that ADR 0005 already names for `Verb`. Three of the six byte counts that DR3 exempts as "compiler facts" are not facts. One of them rests
an approved suppression on a false invariant. Section 4.7 writes a git configuration key that makes every bundle unreadable by plain
`git`. I proved three of the four with the compiler on the pinned toolchain. I took the fourth
from git's own documented rule.

The single biggest risk is C-1, and it is not the count. **VR1 and the lint policy cannot both stand.** VR1 exists to stop a trait demand from pointing
upward. `derive_partial_eq_without_eq` exists to make `Eq` universal where it is possible. Revision 10 answered the previous review by removing `Eq` from six more types. That moved the
design further from the policy while every guard counter stayed at zero. The design must choose, once, and write the choice in one place. Every other
Critical is a number or a line that one edit corrects.

Nine Warnings sit under the Criticals, in three shapes. Three are a half-closed finding. The
cleared over mark returns from a stale snapshot. The four toolbar stubs have no caller. ADR 0004
still puts the core on the wrong MIDI queue.

Three are a seam the design declares and never names. A `dyn Trait` field escapes PG20.
`LogicalPx` has no accessor and no conversion site. Two elements paint one meter.

Three are a rule the document states and then breaks. A raw byte string hides a cast the lexer
claims it cannot. The first-open window is the minimum and not the default. `assert!` is a panic
form that section 12.3 does not list.

The weakest Reactive property is **Resilient**, for the third revision running, and for a new
reason. The failure is no longer a declaration this document contradicts. It is a declaration this document states clearly and the toolchain refuses. The repository's own
CLAUDE.md calls a policy edit a defect that escalates. The next revision needs one rule that reads the declarations
against `[workspace.lints]`, and a probe for it, in the shape PG19 and PG20 already have.
