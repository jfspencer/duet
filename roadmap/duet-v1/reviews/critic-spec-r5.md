# Specification review, revision 5: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is the fifth pass. Revisions 1 to 4 each returned NOT READY.

Sources read in full: `roadmap/duet-v1/architecture.md` (5090 lines), the six ADRs,
`research/linux-macos-platform.md`, `research/crate-survey.md`, `product-requirements.md` sections 8
and 11, `design-contract.md` sections 0 to 5 and the `StageCurve` appendix, `CLAUDE.md`, the root
`Cargo.toml`, `.cargo/config.toml`, `clippy.toml`, `deny.toml`, `scripts/dod.sh`, and
`crates/duet/src/`.

I ran the placement guard, nine hostile probes against it, one rustc lint probe, and five mechanical
passes over the document. Every probe ran in a throwaway copy. I wrote no repository file. I ran one
read-only `git archive HEAD`.

Revision 5 is a large step. The consolidation works. Twenty of the twenty-two revision-4 findings
close, the derive audit is clean, the plan graph is acyclic, and the guard now catches six of the
seven shapes the Architect claims.

It does not close. Three families remain.

1. **One lint the specification never names.** `missing_copy_implementations` is a workspace lint at
   `warn`, and `-D warnings` makes it an error. Five declared public types trip it. I proved this
   with rustc.
2. **The guard still passes on blocker P1.** Its own rule 5 describes a table row that nobody would
   write. The real shape, a GPUI type in a field of a pure-crate type, passes. Its rule 6 is written
   and not implemented.
3. **The plan graph's disjoint-scope claim is false again.** Four pairs of phase-4 chunks write one
   member manifest each. One chunk owns a bench and owns neither the manifest nor the directory.

---

## 1. Closure check against the revision 4 review

### 1.1 The eight blocking findings

| Id | Finding | State | Section and reason |
|---|---|---|---|
| R0 | `Verb` cannot derive `Eq` and `Hash` | **CLOSED** | 3.5 VR1 points the requirement down. I extracted all 122 derive lines. No type outside the VR1 allow list carries `Eq`, `Hash`, or `Ord`. |
| R1 | `SessionCommand::SetViewState` makes a Cargo cycle | **CLOSED** | 3.4 deletes the variant. `view.json` is in the 4.1 layout, the 4.2 tracked list, the 9.4 watch set, and ADR 0003 decision 1. |
| R2 | Seam rule two reaches nine lines only | **CLOSED** | I checked all eleven lines file by file. Every second-level file a later chunk modifies is in the first chunk's write scope. I1 and J1 now list `src/lib.rs`. |
| R3 | `Cargo.lock` has no owner | **CLOSED for the lock** | SM5 names it in every `M` chunk and in every chunk that writes a member manifest. I verified all 56 rows. See S4 for the inverse defect. |
| R4 | Six ADR decisions contradict their own corrections | **CLOSED** | I read all six pairs. Each one is edited in place. ADR 0004 items 8e and 12a now both cite B ids. |
| R5 | The buffer pool sits on the wrong type | **CLOSED** | 5.5 and 5.6 put the pool on `GraphState`. `BufferPool` derives `Debug` alone, and no published type holds one. B55 plus B52 equals B56. |
| R6 | `RecentList` and `RecentAdd` are not verbs | **CLOSED** | 9.1 holds both. Appendix A names both on the `StartView` row. |
| R7 | Chunk M0 cannot pass its own Completion command | **PARTIAL** | 2.3 removes the floor and the `convert.rs` existence rule. Rule 4 meets a form the repository already holds, and no stated test covers it. See S10. |

### 1.2 The nine warnings

| Id | State | Section and reason |
|---|---|---|
| R8 `check-placement` passes on blocker P1 | **PARTIAL** | Probe 1b passes. Probe 8 passes. See S5 and S6. |
| R9 Two claims, two counters | **PARTIAL** | The guard prints both counters and parses section 1.3. It skips every subject the edge list omits. See S14. |
| R10 Three Completion commands are green before their chunk | **CLOSED** | M4 asserts the pin. M7 and M8 are gone. `criterion` moved to M5. |
| R11 A deferred `Save` has no queue place | **CLOSED** | B36 reserves 4 of 32 slots. TH8 covers five writing verbs. The snapshot is an `Arc::clone`. |
| R12 A cancelled job leaves a partial file | **CLOSED** | 9.6 gives five cleanup rules and the user message. 4.1 bounds `exports/.tmp/`. See S17 for one shape mismatch. |
| R13 The ring ceiling has no user answer | **CLOSED** | B57 maps to `GatewayError::RingBudgetExceeded` and to `fault_text.rs`. `Play` allocates nothing. |
| R14 A derived `Deserialize` bypasses the constructor | **CLOSED** | `#[serde(try_from = "f64")]` on `Finite`. Property 4 now compares two `Finite` values. |
| R15 Two MUST stories have no view chunk | **CLOSED** | `StageCurve` is the twelfth element. The design contract carries a matching appendix. See S12 for the wider gap. |
| R16 The arm control has three surfaces | **CLOSED** | The R-05 row names K1, K6, and K4. `TransportArmRecord` carries no track list. |

### 1.3 The five concerns

| Id | State | Reason |
|---|---|---|
| R17 The error lists disagree | **CLOSED** | 12.1 states the rule and lists nothing. `DocumentError` is deleted. |
| R18 `Track::input` has two encodings | **CLOSED** | `input: InputSelection`, with `InputSelection::None`. |
| R19 The fault report has two single places | **CLOSED** | 12.4 gives a fault-class table with three rows. |
| R20 `cargo-nextest` is unpinned | **CLOSED** | Section 14 pins 0.9.145 in `scripts/bootstrap.sh`, and M0 owns the file. |
| R21 `variant_size_differences` is a rustc lint | **CLOSED** | 9.1 and ADR 0005 decision 3 both say rustc. I confirmed the level in `[workspace.lints.rust]`. |

**Count: 19 CLOSED, 3 PARTIAL, 0 OPEN.**

### 1.4 The four document rules

| Id | State | Reason |
|---|---|---|
| DR1 A decision is edited in place | **PARTIAL** | The six ADR pairs close. Architecture 10.5 still describes a design contract sentence that the contract no longer holds. See S13. |
| DR2 A type is defined exactly once | **CLOSED** | The guard reports `DUPLICATED: 0`. I planted a second `Knot` declaration and the guard failed with exit 1. |
| DR3 A number is stated exactly once | **PARTIAL** | Five number-and-unit pairs sit outside 1.6, and each is structural, not a budget. Version numbers carry no id and no stated exemption. See S18. |
| DR4 A rule is stated exactly once | **CLOSED** | 1.7 indexes DR, PL, VR, TH, and SM. Every id in the index exists at the cited section. |

### 1.5 The three mechanical passes

1. **Two declarations of one type: none.** The guard reports `DUPLICATED: 0` over 254 candidates.
2. **A number outside section 1.6: five, all benign.** They are `13 parts` (a divisor), `32 bytes`
   and `16 bytes` (two type sizes), and two sample rates inside a column header and a test
   description. One numeric literal sits in a Rust block outside 1.6: `SmallVec<[Ticks; 12]>`.
3. **A number in an ADR: none that is a budget.** Every hit is a revision reference, a version pin,
   an item index, or the literal `0` inside `unwrap_or(0)`. One exception: ADR 0005 decision 8
   writes the socket mode `0600` rather than B80.

---

## 2. New findings in revision 5

### CRITICAL: S1. Five declared types fail `missing_copy_implementations`

**OBSERVATION.** The root `Cargo.toml` sets `missing_copy_implementations = "warn"` in
`[workspace.lints.rust]`. `.cargo/config.toml` sets `rustflags = ["-D", "warnings"]`. The lint is
therefore an error on every build and in `scripts/dod.sh`.

**CLAIM.** Five public types that this specification declares have all-`Copy` fields and derive no
`Copy`. Each one is a hard error. Appendix B.1 budgets twelve suppressions and names none of them.

**ARGUMENT.** `missing_copy_implementations` fires on a public type whose every field is `Copy` and
which does not implement `Copy`. The lint is not in clippy, so no clippy configuration relaxes it.
An implementer who meets it has two answers. The first is to add `Copy`, which is correct. The
second is an `#[expect]`, which is a policy defect outside the B.1 budget. The specification names
neither, so the choice falls to the implementer at five sites. This is the defect class of R0
against a different lint: the declared code does not build under the repository's own policy.

**EVIDENCE.** I built the exact `ManifestEntry` declaration of section 4.3 with
`rustc --edition 2024 --crate-type lib` and `#![deny(missing_copy_implementations)]`:

```
error: type could implement `Copy`; consider adding `impl Copy`
  --> lib.rs:16:1
16 | / pub struct ManifestEntry {
```

The same rule reaches four more declarations.

| Type | Section | Fields, all `Copy` |
|---|---|---|
| `Source` | 6.1 | `SourceHash`, `NonZeroU8`, `SampleRate`, `u64`, `Position` |
| `ManifestEntry` | 4.3 | `SourceHash`, `NonZeroU8`, `SampleRate`, `u64`, `u64`, `AudioContainer` |
| `Spanner` | 3.3 | `SpannerId`, `SpannerKind`, `NoteId`, `NoteId` |
| `ExportSpec` | 7.4 | `Container`, `SampleFormat`, `SampleRate`, `DitherKind`, `Normalization` |
| `Transport` | 5.9 | `TransportState`, `RecordState`, `SuperClock`, two `FrameCount`, two `Option<Span>` |

**WHAT SHOULD CHANGE.** Add `Copy` to all five. Then state the rule once, beside VR1: a type whose
every field is `Copy` derives `Copy`, because the workspace lint makes the omission an error. Add
the same test to the derive pass that produced this list.

### WARNING: S2. `view.json` has no document type that holds what it must persist

**OBSERVATION.** Section 10.2 says "`ViewState` implements `BundleDocument`", and `duet-project`
persists `view.json`. Section 4.1 calls `view.json` the "per-mode view state". Appendix A's row
reads "Mode, splits, zoom, scroll, selection, window geometry ... `duet-project` saves it in
`view.json`".

**CLAIM.** `ViewState` holds one mode's state. It holds no `Mode` field and no `WindowGeometry`. No
declared type aggregates the four modes and the window, so `view.json` cannot hold the three facts
that Appendix A assigns to it.

**ARGUMENT.** `ProjectView` holds `mode`, `per_mode: BTreeMap<Mode, ViewState>`, and `window`. It is
an app type in `crates/duet`, and `duet-project` carries no edge to the application. `duet-project`
therefore cannot serialize it. `ViewStateRequest { mode, state, window }` carries the three facts
across the transport, and it is a request, not a document. X-08 and C-03 are MUST stories, and their
persistence type does not exist. `Verb::ViewGet { mode }` compounds it: it returns one mode's state,
so no verb reads the window geometry back.

**EVIDENCE.** `architecture.md:1461`, `:3671` to `:3706`, `:3692`, `:4771`. The section 1.5 row for
`duet-command` holds `Mode`, `ViewState`, `WindowGeometry`, and `ViewStateRequest`, and no
aggregate.

**WHAT SHOULD CHANGE.** Declare one document type in `duet-command`, for example
`ProjectViewState { mode, per_mode, window }`. Implement `BundleDocument` for it. State whether
`ViewGet` returns one mode or the whole document. Place the new type in the 1.5 table.

### WARNING: S3. Four pairs of phase-4 chunks write one member manifest

**OBSERVATION.** Section 13.3 states: "No two *line* chunks in one phase share a write scope, except
`Cargo.lock`". SM1 states: "A member manifest has one owner ... Two lines never write one member
manifest." I parsed all 56 chunk rows and compared their write scopes per phase.

**CLAIM.** Four pairs in phase 4 each write one member `Cargo.toml`. The exception clause names
`Cargo.lock` alone, so the claim in 13.3 is false and SM5's deterministic merge rule does not cover
the conflict.

**ARGUMENT.** SM1 requires the chunk that uses a dependency to add the `{ workspace = true }` entry
in the same commit. C2 needs `rtrb`, and C3 needs `arrayvec`, `triple_buffer`, and `basedrop`. Both
therefore edit `crates/duet-engine/Cargo.toml`. Phase 4 runs eleven chunks together, so two agents
hold one file. SM5 rule 4 resolves a `Cargo.lock` conflict with `--ours` and says nothing about a
manifest. A manifest conflict loses one side's dependency entries, and `cargo machete` or the build
then fails for a reason neither agent caused. Section 13.4 makes the error visible: the "C2 and C3
in parallel" row reasons about `src/disk/` and `src/chain/` and never mentions the manifest, and the
"G2 and G3 in parallel" row says M3 wrote the target tables while both rows still write the file.

**EVIDENCE.**

| Phase | Shared file | Chunks |
|---|---|---|
| 4 | `crates/duet-engine/Cargo.toml` | C2, C3 |
| 4 | `crates/duet-media/Cargo.toml` | N2, N3 |
| 4 | `crates/duet-midi/Cargo.toml` | G2, G3 |
| 4 | `crates/duet-project/Cargo.toml` | F2, F3 |

`architecture.md:4350` to `:4394`, `:4506`, `:4540`, `:4557`.

**WHAT SHOULD CHANGE.** Give each member manifest one owner per phase. Two answers work. Move every
dependency a phase needs into the first chunk of the line, which is one phase earlier, and drop the
manifest from the later chunks. Or split phase 4 so that no two chunks of one line run together.
State which answer you take, and correct the 13.3 claim and the two 13.4 reasons.

### WARNING: S4. Chunk I3 owns the `criterion` bench and owns neither the manifest nor `benches/`

**OBSERVATION.** Appendix B.3 gives `criterion` the chunks "A4, I3". Section 13.0 states: "A
`[[bench]]` target and a `[dev-dependencies]` entry belong to the member manifest, so the chunk that
adds the bench adds the target." Section 5.12 and section 9.6 both name a `criterion` bench in
`duet-core` that measures the `Arc::make_mut` clone against B5.

**CLAIM.** I3's write scope is `crates/duet-core/src/{snapshot,job,midi_entry}.rs` and nothing else.
It holds no `Cargo.toml`, no `Cargo.lock`, and no `benches/` directory. I3 cannot write the bench it
owns.

**ARGUMENT.** This is R3's defect class inverted. R3 named a file the gate needs and no chunk owns.
S4 names work a chunk owns and no write scope covers. The consequence is measurable: B5's second
half is the one clone on the core thread, section 5.12 lists it as a measured path, and no chunk can
measure it. I1 writes the manifest in phase 5 and cannot add a dev-dependency for code that lands in
phase 7, because `cargo machete` reports an unused dev-dependency.

A second row has the same shape. B2's goal is "the MusicXML writer and the round-trip property
test", and its write scope is `crates/duet-interchange/src/musicxml/write.rs` and `tests/`. If that
test uses `proptest`, B2 cannot add the dev-dependency. B.3 lists T1 as the only chunk that needs
`proptest`, so state which reading is correct.

**EVIDENCE.** `architecture.md:4412`, `:4342`, `:4291`, `:4852`, `:2560`.

**WHAT SHOULD CHANGE.** Add `crates/duet-core/benches/`, `crates/duet-core/Cargo.toml`, and
`Cargo.lock` to I3's write scope. Check phase 7 for a manifest overlap after the change. State
whether B2 needs `proptest`, and give it the manifest if it does.

### WARNING: S5. `check-placement` still passes on the real shape of blocker P1

**OBSERVATION.** Section 1.5 rule 5 reads: "`Pixels`, `Px`, `Bounds`, `Entity`, `Window`, `App`,
`Context`, `Runtime`, and `CancellationToken` are recognised names, and a row other than
`crates/duet` or `duet-agent` that claims one is a failure. That is the exact shape of P1". I read
the guard and then ran it with hostile input.

**CLAIM.** The rule describes a section 1.5 table row. Blocker P1 was not a table row. It was a GPUI
type in a field of a `duet-command` struct. The guard cannot see that, and I proved it.

**ARGUMENT.** `main` builds `misclaimed` from `FRAMEWORK` names that appear in the ownership table
with a non-application crate. No author writes `Px` into the 1.5 table, so that check fires on a
defect nobody produces. The real defect is a `FRAMEWORK` name in the `used` set. `unplaced` excludes
every `FRAMEWORK` name, and the guard reads no crate context for a Rust block, so the name is
invisible. Rule 5's own sentence claims the opposite.

**EVIDENCE.** Probe 1b restored the exact P1 defect, `sidebar_width: Px` in place of
`sidebar_width: LogicalPx`:

```
CANDIDATE TYPES: 255
PLACED:          255
UNPLACED:        0        DUPLICATED: 0     MISCLAIMED: 0     EXIT=0
```

Probe 1 put `Px` into the `duet-command` table row and the guard failed with `MISCLAIMED: 1`.
`placement_check.py:264` to `:270`.

**WHAT SHOULD CHANGE.** Bind each Rust block to a crate. The simplest source is the nearest section
heading, which the 1.5 table already maps to a crate. Then fail on a `FRAMEWORK` name used inside a
block that belongs to a crate outside `APP_ROWS`. Add a test that plants `sidebar_width: Px` and
goes red. Until that test runs, do not describe rule 5 as the P1 shape.

### WARNING: S6. The variant filter is not scoped, so one name collision hides an unplaced type

**OBSERVATION.** Section 1.5 rule 6 reads: "The variant filter is scoped to the enum that declares
the variant. Revision 4 removed every candidate whose name matched a variant anywhere in the
document". `enum_variants` returns a dictionary keyed by the declaring enum.

**CLAIM.** `main` discards the key and unions every arm into one global set. The scope is claimed
and not implemented. An unplaced type whose name matches any enum variant anywhere is invisible.

**ARGUMENT.** The line is
`variants |= {arm for arm in arms if arm not in declared_counts}`. It drops `enum_name`. The filter
is then `name not in variants` over one flat set, which is exactly revision 4's behaviour. The only
change is a guard on `declared_counts`, and that guard reads the counts as they stand at that point
in the loop. A type declared later in the document than the colliding variant is therefore removed.
The document holds many one-word variants: `Reverb`, `Delay`, `Gate`, `Equalizer`, `Compressor`,
`Measured`, `Default`, `Repeat`, `Wav`, `Flac`, `Peak`, `Rms`.

**EVIDENCE.** Probe 8 added `pub struct Reverb { tail: Finite }` in a block after section 5.5 and
placed it nowhere:

```
CANDIDATE TYPES: 254   PLACED: 254   UNPLACED: 0   EXIT=0
```

Probe 6 added `MeterHold`, which collides with nothing, and the guard failed with `UNPLACED: 1`.
`placement_check.py:251` to `:252`.

**WHAT SHOULD CHANGE.** Keep the enum name. Remove a candidate only when it appears in a type
position inside the declaring enum's own body. Add a test that plants an unplaced type named after
an existing variant and goes red.

### WARNING: S7. `EngineState::NoDevice` carries no cause, so three failures give one message

**OBSERVATION.** Section 5.4 says a missing PipeWire server returns `BackendError::NoServer` and
"The engine stays in `EngineState::NoDevice`". It then fixes one message: "Duet found no PipeWire
audio server." Section 5.12 says an open past B18 returns `BackendError::OpenTimeout` and "The
engine stays in `EngineState::NoDevice`". `EngineState::NoDevice` is a unit variant.

**CLAIM.** Two distinct backend failures, and the ordinary no-device case, collapse into one
payload-free state. The user interface reads the state, so it cannot show the cause. The
no-fallback decision exists to give the user a clear failure, and this erases it.

**ARGUMENT.** ADR 0004 decision 4c rejects the ALSA host because "A fallback would trade a clear
failure for a dropout the user cannot diagnose". The same reasoning applies here. A user whose
device open timed out reads that PipeWire is missing, installs PipeWire, and nothing changes. A
macOS user, for whom `NoServer` "cannot happen", reads a PipeWire message. Section 12.4 makes it
worse: it says the `fault_text.rs` table fixes a message "for each `EngineFault` variant, and for
each `GatewayError` variant", and then requires a row for `NoServer`, which belongs to neither enum.

**EVIDENCE.** `architecture.md:1966`, `:1975`, `:2583`, `:4202` to `:4207`, `:4210` to `:4213`.

**WHAT SHOULD CHANGE.** Give `NoDevice` a cause: `NoDevice { reason: NoDeviceReason }`, with at
least `NoServer`, `OpenTimeout`, and `NoneFound`. State which enum `fault_text.rs` keys on, and make
every row in its stated list a variant of that enum.

### WARNING: S8. The two workflows are vacuous green, and `soak.yml` cannot select a test it claims

**OBSERVATION.** SM3 states: "Every filtered `cargo nextest` command carries `--no-tests=fail`, so a
filter that matches nothing is a failure." Section 14 gives three workflow commands and one rung-two
command. None carries the flag. Section 14's test table says the macOS case-insensitive volume test
runs in `soak.yml`.

**CLAIM.** Two defects. The workflow commands report success when their filter matches nothing. One
test the table assigns to `soak.yml` is outside every command that `soak.yml` runs.

**ARGUMENT.** A filtered run with no match is the exact vacuous green that SM3 exists to stop, and
the specification breaks its own rule at four sites. The ordering makes the risk live, not
theoretical. M5 writes `audio-smoke.yml` in phase 5, and C4 writes the cpal backend in the same
phase, after M5. `audio-smoke.yml` runs "every run", so it runs green with no matching test from M5
until C4. For the second defect, `soak.yml` runs `-p duet-engine -E 'test(soak)'` and
`-p duet-time -E 'test(proptest_large)'`. The case-insensitive test lives in `duet-project`, per
section 4.3. Neither command reaches that package, so a test that guards a documented data-loss path
on APFS runs nowhere.

**EVIDENCE.** `architecture.md:4264`, `:4640` to `:4641`, `:4652`, `:4680`, `:4093`, `:1541` to
`:1544`, `:4500`.

The `audio-smoke.yml` comment also says "start a user pipewire process, wait for its socket" and
states no bound. A wait with no timeout is an unbounded wait, which section 5.12 forbids everywhere
else.

**WHAT SHOULD CHANGE.** Add `--no-tests=fail` to every filtered command in section 14 and in both
workflows. Move `audio-smoke.yml` to a chunk that runs after C4, or state that the job is expected
to find no test until C4 lands and give it a skip condition. Add a third `soak.yml` command that
selects the `duet-project` test, or move that test row to another workflow. Bound the socket wait.

### WARNING: S9. `MidiPortId` crosses two port namespaces with no stated map

**OBSERVATION.** Section 8.1 declares one trait, `MidiTransport`, with `ports`, `open_input`,
`open_output`, and `subscribe_hotplug`. It names three implementations. It then states: "The MIDI
byte stream stays on `midir` on both platforms."

**CLAIM.** Two statements disagree about what a platform transport does, and the identifier that
joins the two layers has no stated mapping. C-10, plug and play with no setup, is a MUST story that
rests on that mapping.

**ARGUMENT.** If the byte stream is always `midir`, then `CoreMidiTransport::open_input` and
`PipeWireRegistryTransport::open_input` either open a second byte path, which contradicts the rule,
or delegate to `midir`, which the document never says. Section 11.1 gives a third reading: it calls
`MidirTransport` the "Fallback", not the byte path. A PipeWire registry global identifier, a
CoreMIDI endpoint unique identifier, and a `midir` port index are three namespaces. Section 8.2
binds a port on `PortAdded`, so a `MidiPortId` that the registry produced must open through `midir`.
That translation is the whole Linux plug-and-play path and it is unspecified. An implementer will
invent it, and the two ports will disagree whenever the port list reorders.

**EVIDENCE.** `architecture.md:3049` to `:3104`, `:3111`, `:3979`.

**WHAT SHOULD CHANGE.** Split the trait. Name one trait for hot-plug notification and one for byte
input and output, or state that a platform transport implements `subscribe_hotplug` and `ports`
alone and delegates the rest. State how a platform port identity resolves to a `midir` port, and
what happens when it does not resolve.

### WARNING: S10. `check-conversions` rule 4 meets a form the repository already holds

**OBSERVATION.** Section 2.3 rule 4 reads: "A `use` item is the one excluded form ... The guard
skips a statement whose first token is `use`". The three stated tests plant an outer attribute, an
inner attribute, and `let n = x as u32;`. A fourth test plants `use std::fmt::Write as _;`.

**CLAIM.** The repository holds one `as` token on a line that does not start with `use`, inside a
multi-line `use` item. No stated test covers that form, so M0's Completion command can fail on the
tree M0 leaves.

**ARGUMENT.** `crates/duet/src/main.rs` opens `use gpui_kit::{` on line 13 and carries
`AppContext as _` on line 14. Rule 4 says "statement", which is correct, and the four tests only
prove the single-line form. A line-oriented port passes every stated test and fails the repository.
M0's Completion is `cargo xtask check-conversions && cargo xtask check-placement ...`, and the
pre-commit hook runs the same guard, so M0 cannot commit. That is R7 in a new place.

**EVIDENCE.** I swept the current tree for whole-word `as` tokens under `crates/*/src` and
`tools/*/src`. Every hit is a doc comment, which rule 2 strips, or a single-line `use`, except one:

```
crates/duet/src/main.rs:14:    App, AppContext as _, AsyncApp, Bounds, TitlebarOptions, ...
```

**WHAT SHOULD CHANGE.** Add a fifth test that plants a multi-line `use` with a rename on a
continuation line and asserts a zero exit. State that the guard joins lines to a `;` before it
applies rule 4.

### WARNING: S11. `scripts/dod.sh` gains a permanent dependency on a roadmap markdown file

**OBSERVATION.** M0 adds `cargo xtask check-placement roadmap/duet-v1/architecture.md` to
`scripts/dod.sh`. Section 14 lists it under rung one. `scripts/dod.sh` is the only gate surface, and
the `.githooks/pre-commit` hook runs it on every commit.

**CLAIM.** From M0 onward, every commit in this repository depends on one roadmap markdown file. The
specification states no lifecycle for that file, and the guard's behaviour on a missing file is
unstated.

**ARGUMENT.** Three consequences follow, and each has a live trigger. First, the plan-authoring step
that this review precedes will edit `roadmap/duet-v1/`. Any edit that adds a type without a 1.5 row
blocks every engineer on the repository, not only the author. Second, the prototype opens the path
with no guard, so a moved or archived document raises `FileNotFoundError` and the gate fails for a
reason unrelated to the commit. The Rust port's behaviour on a missing file is unspecified, so
fail-open and fail-closed are both still available. Third, the guard reads the document alone. It
never compares the document with the code, so its value falls to zero once the crates exist while
its cost stays on every commit forever.

**EVIDENCE.** `architecture.md:4298`, `:4609` to `:4610`, `:4621`. `placement_check.py:59` to `:62`.
`scripts/dod.sh` lines 19 to 31 hold the current gate list, which carries neither guard today.

**WHAT SHOULD CHANGE.** State the document's lifecycle in section 14: which commit removes the gate
line, and what replaces it. State the guard's behaviour on a missing file, and choose fail-closed.
State who may edit `architecture.md` after M0, because the edit blocks the whole fleet.

### WARNING: S12. The MUST story table claims completeness and covers 18 rows of about 70

**OBSERVATION.** Section 13.2 carries the heading "Where each MUST story lands". The table holds 18
rows. Product requirements section 8 holds about 70 MUST stories.

**CLAIM.** The heading claims completeness and the table is a subset. Five MUST stories appear
nowhere in the architecture document.

**ARGUMENT.** Most absent rows are covered implicitly by a chunk goal, and a reader can find them.
Five are not. X-09 background work, X-10 load states, X-11 empty states, X-13 destructive actions,
and R-03 show one track or all tracks have no chunk and no element. Design contract section 8
specifies a load state, an empty state, an error state, and a no-device state for each of the four
modes, and no chunk goal names one. That is finding P2 and finding R15 in a sixth place: a
specified surface with no file and no chunk. A plan author who reads the table builds from it.

**EVIDENCE.** I counted each MUST identifier in `architecture.md`. X-09, X-10, X-11, X-01 to X-06,
C-08, C-10, C-13, C-16, C-18 to C-21, M-05, MA-01, MA-04, H-02 to H-04, and H-06 each return zero
hits. `design-contract.md:1182` to the end of section 8. `architecture.md:4464` to `:4489`.

**WHAT SHOULD CHANGE.** Rename the heading to what the table is, for example "MUST stories that
needed a placement decision". Then add a row for X-09, X-10, X-11, X-13, and R-03, or state in one
sentence which chunk carries the load, empty, and error surfaces of each mode.

### CONCERN: S13. Architecture 10.5 describes a design contract sentence that the contract dropped

Section 10.5 reads: "The contract writes `beat_for_x(x) -> f32`; this specification returns
`Ticks`". Design contract 3.1 now reads: "The map answers `x_for_beat(beat) -> f32` and
`beat_for_x(x) -> Ticks`." ADR 0006 decision 2 says "Both documents use the contract's names. Both
return `Ticks` from the inverse." Three documents give two answers, and the architecture holds the
stale one. This is DR1's own defect class inside the revision that added DR1. Edit 10.5 to say that
the contract and the specification agree, and delete the operator report item.

### CONCERN: S14. The edge check skips every subject that the section 1.3 list omits

`edge_claims` reads `known = edges.get(subject)` and returns early when the result is `None`. The
1.3 list has no line whose left side is `duet-time`, because `duet-time` is the root and carries no
edge. Every prose claim with `duet-time` as the subject is therefore unchecked. I planted
"`duet-time` reads the `duet-project` manifest on every conversion" and the guard reported
`EDGE CLAIMS BAD: 0` with exit 0. A leaf crate is where a false edge claim matters most. Treat an
absent key as an empty edge set, and add a test that plants this exact sentence.

### CONCERN: S15. The `coremidi` and `pipewire` root pins have no named owner

Section 11.2 states: "`[workspace.dependencies]` in the root manifest pins both versions." M3's
"Also writes" column names thirteen pins "and both `[target.'cfg(target_os)']` tables of
`duet-midi`". Those two tables live in the member manifest, not the root. No chunk row names the two
root pins. Appendix B.3 gives `pipewire` the owner M3 and omits `coremidi` entirely, because the
survey lists `coremidi` 0.9.2. Add both pins to M3's column by name.

### CONCERN: S16. `midir`'s own `alsa` version is never checked

Section 8.1 reason 1 rejects the ALSA sequencer because an `alsa` 0.12.1 pin "would put two `alsa`
versions in the tree" beside cpal's 0.11. `midir` is the byte path on Linux, and `midir` reaches
ALSA through its own `alsa` dependency. The platform research file records cpal's pin and `midir`'s
missing hot-plug callback, and it never records `midir`'s `alsa` version. If `midir` 0.11 pins a
different `alsa` major, the duplicate arrives through the dependency the design keeps. `deny.toml`
sets `multiple-versions = "warn"`, so the gate does not fail, and reason 1's premise does. Record
`midir`'s transitive `alsa` version in the research file before M3 is dispatched.

### CONCERN: S17. `exports/.tmp` has two shapes, and `state/.tmp/` is absent from section 4.1

Section 7.4 names the intermediate file `exports/.tmp/<job>.f32`, which is a file. Section 9.6 rule
1 names `exports/.tmp/<job>/`, which is a directory, and adds `state/.tmp/<job>/`. Section 4.1's
layout shows `exports/.tmp/` with the comment "one intermediate file per running export job" and
shows no `state/.tmp/`. The store-bound table has a row for `exports/.tmp/` and none for
`state/.tmp/`. Pick one shape, add `state/.tmp/` to the layout, and give it a bound row.

### CONCERN: S18. DR3 has an unstated exemption for a version number

DR3 reads: "A number is stated exactly once. Section 1.6 is the one table of budgets and bounds ...
Every other place writes the id." The document writes `cpal 0.18.2`, `notify` 8.2.0,
`notify-debouncer-full` 0.7.0, `rmcp` 3.4.0, `gpui-kit` 0.6.4, `pipewire` 0.10.1, and
`cargo-nextest` 0.9.145 outside 1.6, with no id. Each is correct, because a version pin is not a
budget. State the exemption in DR3. ADR 0005 decision 8 writes the socket mode `0600` rather than
B80, which is a real DR3 miss; edit it.

### CONCERN: S19. `check-conversions` scans `src` only, and the rule text says "anywhere"

Section 2.3 rule 1 says the guard "walks every `.rs` file under that member's `src` directory". Rule
3 says it "fails on any whole-word `as` token outside `crates/duet-time/src/convert.rs`". Those two
disagree: a `tests/` file, a `benches/` file, and an `examples/` file hold casts that the guard never
reads. `tools/plan-db/tests/roundtrip.rs` is one such file today. Clippy covers those targets through
`--all-targets`, so the risk is small. State the scope limit in rule 3, as section 1.5 states its own
limit for fenced blocks.

### CONCERN: S20. Chunk I3's Completion command may be green before I3

I3's Completion is `cargo nextest run -p duet-core -E 'test(snapshot) + test(job)' --no-tests=fail`.
`+` is a union in a nextest filterset. I1 builds the `JobRunner` and writes
`crates/duet-core/src/job_runner.rs`, so a unit test there carries `job_runner` in its full name and
matches `test(job)`. The union is then green after I1 and before I3, which SM3 forbids. Change the
filter to `test(snapshot) + test(job_registry)`, or name the I3 tests so that no I1 test matches.

---

## 3. Reactive assessment

- **Responsive: PARTIAL.** Twenty-three budgets and ten timeouts are real and cited by id. The
  `JobRunner`, the reserved queue slots, and the `Arc` snapshot take every slow verb off the frame
  thread. Two gaps: B5's `Arc::make_mut` half has no chunk that can write its bench (S4), and the
  `audio-smoke` socket wait carries no bound (S8).
- **Resilient: PARTIAL.** The five-step save, the crash matrix, the four-source live set, the writer
  lock recovery, the job cleanup contract, and the typed error rule are correct and complete. Three
  failures stay undiagnosable or unproved: `EngineState::NoDevice` erases the cause of three
  different failures (S7), the placement guard passes on the defect it names (S5, S6), and two
  workflows report success they did not earn (S8).
- **Elastic: PASS.** Every channel, ring, queue, store, and cache carries a bound with an id. The
  job queue reserves four slots for a user-started verb. Backpressure is explicit on the agent
  channel, the note-entry queue, and the fault queue. I found no unbounded producer.
- **Message Driven: PARTIAL.** One vocabulary, one writer, versioned documents, published snapshots,
  and no shared mutable state across the user-interface seam. The Cargo cycle is gone. Two seams are
  undefined: `view.json` has no document type (S2), and `MidiPortId` crosses two namespaces with no
  map (S9).

---

## 4. Plan graph check on section 13

I parsed the phase table, all 56 chunk rows, and all 71 link rows, and compared them mechanically.

1. **The graph is acyclic.** I expanded 134 chunk-to-chunk edges. None runs backwards in phase order.
2. **Every link agrees with its phase.** The one same-phase link is "M0 before T1", which 13.3 states
   as the exception.
3. **Every chunk has a phase and a row.** 56 rows, 0 gaps.
4. **Write scopes are not disjoint in phase 4.** Four pairs share one member manifest each (S3).
5. **Every chunk has a runnable Completion command.** All 56 rows carry one `cargo` command. One may
   be green before its chunk (S20). M4's command greps the root manifest, so it depends on the pin's
   exact formatting and on the working directory; prefer `cargo metadata` with a `jq` assertion.
6. **Seam rule two is applied in every line.** I checked all eleven. Every second-level file a later
   chunk modifies is a named stub in the first chunk's write scope. R2 closes.
7. **The lock rule is applied in every line.** Every chunk that writes a member manifest also writes
   `Cargo.lock`. The inverse fails once: I3 writes a bench and neither file (S4).
8. **MUST story coverage is partial.** Five MUST stories map to no chunk anywhere (S12).
9. **No line chunk writes a policy file.** SM4 holds. M0 and M5 carry every policy file and both go
   to the Orchestrator.
10. **The manifest seam holds.** No `M` chunk has an empty write scope, and phases 7 and 8 have no
    manifest chunk, which SM1 now allows.

---

## 5. Consistency across the documents

1. **Closed.** `view.json` reaches section 4.1, section 4.2, section 9.4, and ADR 0003 decision 1.
2. **Closed.** The design contract carries a `StageCurve` appendix that names the file, the chunk,
   the two stories, and the visual rules. Architecture 10.3 and design contract 1.11 agree on eleven
   GPUI elements plus `SystemXMap`.
3. **Closed.** Design contract 1.9 and architecture 12.4 give one answer for a device fault.
4. **Closed.** Design contract 1.6 and architecture 10.2 agree on the clamp and the Mix split.
5. **Closed.** All six ADR pairs that R4 named are edited in place.
6. **Closed.** ADR 0005 decision 3 and architecture 9.1 both call `variant_size_differences` a rustc
   lint.
7. **Open.** Architecture 10.5 contradicts design contract 3.1 and ADR 0006 decision 2 on
   `beat_for_x` (S13).
8. **Open.** Design contract 3.3 still gives the path cache a two-part key, "per region and per zoom
   step". ADR 0006 decision 6 and Appendix A give four parts. The system is what makes the wrapped
   view correct, so the contract must state all four.
9. **Open.** Design contract section 8 specifies a load, empty, error, and no-device state for each
   mode. No architecture chunk names one (S12).
10. **Open.** The research file records no `alsa` version for `midir`, and architecture 8.1 reason 1
    depends on one (S16).
11. **Open.** Appendix B.3 gives `pipewire` a manifest owner and gives `coremidi` none (S15).

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The engineering is sound and the document is far better than revision 4. The consolidation rules
work: one declaration per type, one number per fact, and one statement per rule removed the defect
class that killed four revisions. Twenty of twenty-two findings close, and each closure is real
rather than cited. I verified the derive closure, the seam rule, the lock rule, the ADR edits, and
the plan graph by machine, and all five hold.

Three families block it.

The first is a lint the specification never names. `missing_copy_implementations` is a workspace
lint at `warn`, and `-D warnings` makes it an error. Five declared public types trip it. I proved it
with rustc against the exact `ManifestEntry` declaration. That is finding R0 against a different
lint: the declared code does not build under this repository's own policy, and Appendix B.1 budgets
no answer.

The second is the guard, for the second revision running. Rule 5 names blocker P1 and describes a
table row that nobody writes. I restored the exact P1 defect and the guard passed. Rule 6 claims the
variant filter is scoped to the declaring enum, and one line in `main` unions every arm into one
flat set. I planted an unplaced type named after an existing variant and the guard passed. Six of
seven claimed probes are caught; the two shapes the guard exists for are not.

The third is the plan graph. The disjoint-scope claim is false again, in a fourth revision. Four
pairs of phase-4 chunks write one member manifest each, and SM5's merge rule covers `Cargo.lock`
alone. One chunk, I3, owns a bench that section 5.12 lists as a measured path, and its write scope
holds neither the manifest nor the directory.

The single biggest risk is now clear and it is not the document. It is that a mechanism is written
as a rule, and a machine is written beside it, and the machine implements a different rule. Rule 5
and rule 6 of the placement guard both say what the code does not do. SM3 says every filtered
nextest command carries `--no-tests=fail`, and four commands in section 14 do not. A guard that
reports success it did not earn is worse than no guard, because it launders an unverified state as
verified.

The weakest Reactive property is Resilient. Three failures are undiagnosable or unproved: the engine
state that erases its own cause, the guard that passes on its own blocker, and two workflows that
cannot fail.

### Findings that block plan authoring

1. S1 (Critical). Five declared types fail `missing_copy_implementations`.
2. S2 (Warning). `view.json` has no document type that holds what it must persist.
3. S3 (Warning). Four pairs of phase-4 chunks write one member manifest.
4. S4 (Warning). Chunk I3 owns a bench and owns neither the manifest nor `benches/`.
5. S5 (Warning). `check-placement` passes on the real shape of blocker P1.
6. S6 (Warning). The variant filter is not scoped, so a name collision hides an unplaced type.
7. S7 (Warning). `EngineState::NoDevice` carries no cause.
8. S8 (Warning). Two workflows are vacuous green, and one claimed test is unreachable.
9. S9 (Warning). `MidiPortId` crosses two namespaces with no map.
10. S10 (Warning). `check-conversions` rule 4 meets a form the repository already holds.
11. S11 (Warning). `dod.sh` gains a permanent dependency on a roadmap markdown file.
12. S12 (Warning). The MUST story table claims completeness and covers 18 rows of about 70.

Findings S13 to S20 are Concerns. Close each one in the specification, or file it in a document
under `roadmap/duet-v1/`.

---

## Appendix: the guard, as run

Baseline, on the unmodified document:

```
$ python3 roadmap/duet-v1/tools/placement_check.py roadmap/duet-v1/architecture.md
DOCUMENT:        roadmap/duet-v1/architecture.md
CANDIDATE TYPES: 254
PLACED:          254
UNPLACED:        0
DUPLICATED:      0
MISCLAIMED:      0
EDGES PARSED:    58
EDGE CLAIMS BAD: 0                                    EXIT=0
```

Nine probes, all in a throwaway copy made with `git archive HEAD`:

| # | Probe | Source | Result | Caught |
|---|---|---|---|---|
| 1 | A GPUI type claimed by the `duet-command` table row | Architect | `MISCLAIMED: 1`, exit 1 | Yes |
| 1b | A GPUI type in a field of a `duet-command` struct, the real P1 | Mine | exit 0 | **No** |
| 2 | Every ` ```rust ` fence renamed to ` ```rs ` | Architect | `CANDIDATE TYPES: 0`, exit 1 | Yes |
| 3 | An empty file | Architect | `CANDIDATE TYPES: 0`, exit 1 | Yes |
| 4 | No argument | Architect | usage line, exit 2 | Yes |
| 5 | A second declaration of `Knot` | Architect | `DUPLICATED: 1`, exit 1 | Yes |
| 6 | An unplaced field type, `MeterHold` | Architect | `UNPLACED: 1`, exit 1 | Yes |
| 7 | A planted reverse-edge sentence, `duet-session` to `duet-command` | Architect | `EDGE CLAIMS BAD: 1`, exit 1 | Yes |
| 8 | An unplaced type named after an enum variant, `Reverb` | Mine | exit 0 | **No** |
| 9 | An edge claim whose subject is absent from the 1.3 left column | Mine | exit 0 | **No** |

Seven of seven Architect probes are caught as they are literally stated. Probe 1b shows that probe
1's literal form is not the shape of blocker P1. Probes 8 and 9 are two further holes.

One rustc probe, which produced finding S1:

```
$ rustc --edition 2024 --crate-type lib lib.rs
error: type could implement `Copy`; consider adding `impl Copy`
  --> lib.rs:16:1
16 | / pub struct ManifestEntry {
```
