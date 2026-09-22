# Specification review, revision 4: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is the fourth pass. Revisions 1, 2, and 3 each returned NOT READY.

Sources read in full: `roadmap/duet-v1/architecture.md` (4222 lines), the six ADRs,
`product-requirements.md` sections 8 and 11, `design-contract.md` sections 1, 3.3, 4.4, 4.2, 4.6,
5.2, 5.3, `CLAUDE.md`, the root `Cargo.toml`, `scripts/dod.sh`, `deny.toml`, `scripts/bootstrap.sh`,
and `crates/duet/src/`.

I ran the placement script and four adversarial tests against it. I also ran three mechanical tests
against the repository gate in a throwaway copy of the tree. I wrote no repository file and ran no
git command against the working tree.

Revision 4 is the best of the four. It closes every one of the eight blockers in the architecture
text, sixteen of the eighteen Warnings, and all four Concerns. The plan graph is now correct: 53
serial links, no backward link, no same-phase link except the stated `M<phase>` exception.

It does not close. Three families of defect remain.

1. **The vocabulary does not compile.** `Verb` derives `Hash` over eight payload types that carry
   none, and `ExportSpec` carries a raw `f32`. That is blocker 2 of the premise review in its widest
   form, reopened by the revision-4 type moves.
2. **The repeat.** Each fix wrote a rule and did not apply the rule to every site. Seam rule two
   covers nine lines only on paper. The edit-in-place rule left six new stale pairs in the ADRs.
   `SessionCommand::SetViewState` survives beside its own correction and makes a Cargo cycle.
3. **The guards do not guard.** `cargo xtask check-placement` passes when I plant the exact defect
   that was blocker P1. It also passes on an empty candidate set. `cargo xtask check-conversions`
   cannot pass at the end of chunk M0, which is its own Completion command.

---

## 1. Closure check against the revision 3 review

### 1.1 The eight blockers

| # | Finding | State | Section and reason |
|---|---|---|---|
| P1 | `ViewState` and `ProjectView` break the placement rule | **PARTIAL** | 1.5 and 10.2 place every field type correctly. Section 3.4 still holds `SessionCommand::SetViewState { mode: Mode, state: Box<ViewState> }`, and Appendix A still routes through it. See R1. |
| P2 | `TopBar`, `TransportBar`, `ModeSwitcher`, `CoreHost`, `AgentBridge` have no file | **CLOSED** | 13.2 lists fifteen shell files. K1 fills eight; K6 fills seven. Each named entity has a file, a chunk, and a story. |
| P3 | Every manifest chunk fails `cargo machete` | **CLOSED** | 13.0 rule one. I tested it: an unused root pin and a skeleton crate with no `[dependencies]` are both clean. See the evidence in R3. |
| P4 | Chunk K1 fails `cargo clippy` | **CLOSED for K1** | 10.3 and the K1 row give K1 all eleven element stubs and all fifteen shell files. Rule two is not applied to nine other lines. See R2. |
| P5 | The `BufferPool` free list has two writers | **PARTIAL** | 5.5 removes the free list and publishes a whole new pool. The `pool` field sits on `ChainTopology`, which is per track, and the memory table counts one pool per project. See R5. |
| P6 | A deferred verb has no thread | **PARTIAL** | 9.6 gives a `JobRunner`, and 4.4, 4.5, and 5.7 agree. ADR 0003 items 6b and 15 and ADR 0005 item 5 still carry the sentences that P6 attacked. See R4. |
| P7 | The plan ignores `app.rs` and the existing crates | **CLOSED** | Rule zero, the K1 row ("removes `src/app.rs`"), and the 13.1 note that `crates/duet` already exists. |
| P8 | `Track` has no field for `name`, `input`, `monitor`, `armed` | **CLOSED** | 6.1 adds all four, with the persisted set, the runtime set, and two refusals tied to `armed`. |

### 1.2 The eighteen Warnings

| # | State | Section and reason |
|---|---|---|
| P9 `Finite` bit order | CLOSED | ADR 0001 item 3a and 2.6a both say `f64::total_cmp`. The reason is correct. |
| P10 `Finite` lint policy | CLOSED | `Ord` is `total_cmp`. `PartialOrd` returns `Some(self.cmp(other))`. The word "derives" is gone. |
| P11 `AudibleState` path | CLOSED | `ChainTopology::audible`, computed off the audio thread, published by C3. |
| P12 The bus role | CLOSED | `StripKind::Bus(BusRole)`, plus a paragraph on how the mixer view draws each role. |
| P13 Unplaced field types | CLOSED | Every named type is in the 1.5 table. `Timestamp` became `UnixSeconds` in `duet-time`. |
| P14 The 1.2 dependency column | CLOSED | Rule one and B.3 both say the column is third-party only and 1.3 owns every internal edge. |
| P15 `proptest` and `criterion` | PARTIAL | B.3 gives both a licence and a manifest owner. `criterion` belongs to M7 in phase 7, and A4 needs it in phase 5. See R10. |
| P16 Six manifest chunks with no row | CLOSED | 13.1 holds M0 to M8. Rule four moves every policy file to M0 and M5, both dispatched to the Orchestrator. |
| P17 The note-head budget | CLOSED | 8.3 names `Window::on_next_frame` with a source citation. 12.0 plus 16.7 is 28.7. |
| P18 `StartView` opens a file | PARTIAL | `duet-project` owns `RecentList`. `Verb::RecentList` and `Verb::RecentAdd` are not in the verb list, and Appendix A still gives `StartView` the configuration file. See R6. |
| P19 Six stale sentences | PARTIAL | All six named pairs are corrected, and the edit-in-place rule is added. Six new pairs appear. See R4. |
| P20 The writer lock after a crash | CLOSED | 3.8 gives the window start path and the three ordered steps for a lost lock. |
| P21 The disjoint-write-scope claim | CLOSED | 13.3 states the `M<phase>` exception and restricts the claim to line chunks. |
| P22 Completion commands green before | PARTIAL | M1 and M2 are fixed. M4, M7, and M8 still use `cargo check --workspace`, which is green before and after. See R9. |
| P23 The ring undercount | PARTIAL | 5.5 gives a per-state ring table and 104 MB. ADR 0004 item 8e still states 132 MB and 123 MB. See R4. |
| P24 The guard registration point | CLOSED | M0 writes `check_conversions.rs`, edits `tools/xtask/src/main.rs`, and edits `scripts/dod.sh`. All three are in scope. |
| P25 Rung one claims a gate | CLOSED as a statement | 14 states plainly that two lines are absent today and that M0 adds them. The guard then cannot pass at M0. See R7. |
| P26 `PoolExhausted` | CLOSED | 5.5 gives a product reason for 8 and 4, a `GatewayError` mapping, and a `fault_text.rs` message. |

### 1.3 The four Concerns

| # | State | Reason |
|---|---|---|
| P27 Per-platform Completion | CLOSED | Rule three allows a named per-platform command. G2 and G3 use it. |
| P28 tokio rule 4 | PARTIAL | 9.5 rule 4 names three kinds and exempts the `Runtime` and a `Handle`. ADR 0005 item 11 keeps the unqualified ban. |
| P29 `ViewState` gaps | CLOSED | `mix_vertical_split` and `ProjectView::clamped`, which K6 writes. |
| P30 Two ADR numbers | CLOSED | ADR 0004 item 13 says 16 bytes. ADR 0003 item 12 points at the three bounds of 4.9. |

### 1.4 The seventeen PARTIAL rows

Fourteen close. Three do not: "PR X-08 and C-03 context" (R1), "PR C-01 start surface" (R6), and
"N14 stale sentences" (R4). Design contract 3.3 now names `RecordView` as the cache owner and
`duet-core` as the peak source, so that row closes.

**Count: 41 CLOSED, 9 PARTIAL, 0 OPEN.**

---

## 2. New findings in revision 4

### CRITICAL: R0. `Verb` cannot derive `Eq` and `Hash`

**OBSERVATION.** Section 9.1 gives `Verb` the derive set
`Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize`. Section 3.5 asserts
`assert_eq_hash::<Verb>()` and `assert_eq_hash::<EditCommand>()`. I extracted every derive line in
the document and compared it against the payload types that `Verb` names.

**CLAIM.** Eight payload types carry no `Hash`, and two carry neither `Eq` nor `Hash`. The
vocabulary crate does not compile, and the compile-time assertion that proves the property is the
line that fails.

**ARGUMENT.** A derived `Hash` on an enum requires `Hash` on every field of every variant. A
derived `Eq` requires `Eq` on every field. `Verb::ScoreApply(Box<ScoreCommand>)` therefore requires
`ScoreCommand: Hash`, and `ScoreCommand` derives `Debug, Clone, PartialEq, Eq, Serialize,
Deserialize` only. The same rule fails at seven more sites. `ExportSpec` is worse: it derives
`PartialEq` alone, and `Normalization` holds `dbfs: f32`, `lufs: f32`, and `true_peak_dbtp: f32`.
A raw `f32` can never carry `Eq` or `Hash`. `Finite` exists to stop exactly that, and revision 4
moved `ExportSpec` into `duet-command` without applying `Finite` to it. This is blocker 2 of the
premise review and finding N1 of the revision-2 review, one layer wider.

**EVIDENCE.** A mechanical pass over every `#[derive(...)] pub struct|enum` line in
`architecture.md` reports these types as reached by `Verb` and short of the required traits:

| Type | Section | Derives it has | Missing |
|---|---|---|---|
| `ScoreCommand` | 3.4 | `PartialEq, Eq` | `Hash` |
| `Selection` | 3.4 | `PartialEq, Eq` | `Hash` |
| `Tempo` | 2.9 | `PartialEq, Eq` | `Hash` |
| `RecordMode` | 6.2 | `PartialEq, Eq` | `Hash` |
| `AlignChoice` | 6.4 | `PartialEq, Eq` | `Hash` |
| `SendTap` | 7.1 | `PartialEq, Eq` | `Hash` |
| `CompRequest`, `CompRange` | 9.1 | `PartialEq, Eq` | `Hash` |
| `Duration` | 3.3 | `PartialEq, Eq` | `Hash` |
| `ExportSpec` | 7.4 | `PartialEq` | `Eq`, `Hash` |
| `Normalization` | 7.4 | `PartialEq` | `Eq`, `Hash`, and it holds three `f32` fields |

Three more types fail one level down. `Clipboard` derives `PartialEq, Eq, Hash` and holds
`Vec<Note>` and `Vec<Spanner>`. `Note` and `Spanner` derive `Debug, Clone, Serialize, Deserialize`
only. `ViewState` derives `Hash` and holds `selection: Selection`, which has none.

**WHAT SHOULD CHANGE.** Add `Hash` to every type the closure reaches, and add `PartialEq`, `Eq`,
and `Hash` to `Note`, `Spanner`, and `Duration`. Replace every `f32` in `Normalization` with
`Finite`, and give `ExportSpec` the full set. Then state the rule once: a type that `Verb` reaches
carries `PartialEq`, `Eq`, `Hash`, `Serialize`, and `Deserialize`, and it holds no bare float. Add
that rule to the `check-placement` guard, because a table cannot hold it and a human pass missed it
three times.

### CRITICAL: R1. `SessionCommand::SetViewState` makes a Cargo cycle

**OBSERVATION.** Section 10.2 states the fix for P1: "Revision 3 routed it through
`SessionCommand`, which would have put a `duet-command` type inside a `duet-session` command.
`Verb::ViewSetState` and `Verb::ViewGetState` are the only paths." Section 3.4 line 893 still holds
`SetViewState { mode: Mode, state: Box<ViewState> }` inside `SessionCommand`. Appendix A still says
the state is "written through `SessionCommand::SetViewState`" and "saved in `session/session.json`".

**CLAIM.** `duet-session` would name two `duet-command` types. Section 1.3 gives `duet-session` one
edge, to `duet-time`. The edge `duet-session -> duet-command` plus the existing edge
`duet-command -> duet-session` is a Cargo cycle, which Cargo refuses.

**ARGUMENT.** This is the defect class of premise-review blocker 1 and of P1. The document states
the correction in section 10.2 and leaves the corrected text in place in two other sections. A plan
author who reads 3.4 builds a crate that does not resolve. The document's own rule 1 in "How to
read" forbids this shape.

**EVIDENCE.** `architecture.md:893`, `architecture.md:2998`, `architecture.md:3792`. Section 10.2
also says `duet-project` persists the state as `view.json`, while Appendix A says
`session/session.json`. Section 4.1 lists no `view.json`, and section 4.2 does not track it.

**WHAT SHOULD CHANGE.** Delete `SetViewState` from `SessionCommand`. Correct the Appendix A row to
name `Verb::ViewSetState` and `view.json`. Add `view.json` to the 4.1 layout and to the 4.2 tracked
list. Name the chunk that writes it.

### CRITICAL: R2. Seam rule two is written and not applied to nine lines

**OBSERVATION.** Rule two says: "The first chunk of a line creates every module file the whole line
will ever need, **at every depth**, as a stub", and "The rule reaches a second-level directory,
which revision 3's version did not." I checked every line's write scope in 13.2 against every later
chunk in that line.

**CLAIM.** Nine of eleven lines break the rule. The first chunk's write scope names no
second-level file, so each first chunk publishes a `mod` line for a file that does not exist. That
is chunk K1's revision-3 failure repeated nine times.

**ARGUMENT.** Rule two requires the first chunk to write every `mod` line, so that no later chunk
edits a file it does not own. The first chunk's `mod` line names a file inside a directory that only
a later chunk creates. `cargo clippy --workspace --all-targets --locked -- -D warnings` fails at the
end of that phase, and the pre-commit hook blocks the commit. If the first chunk omits the `mod`
line instead, the later chunk's files never compile, and its Completion command finds no test and
fails under `--no-tests=fail`.

**EVIDENCE.**

| Line | First chunk write scope | A later chunk creates | Result |
|---|---|---|---|
| B | `src/{musicxml,smf}.rs`, `src/musicxml/read.rs` | B2: `src/musicxml/write.rs` | `musicxml.rs` names a missing module |
| C | `src/{backend,dummy,transport,disk,chain,mix,cpal,audio}.rs` | C2 `src/disk/`, C3 `src/chain/` and `src/mix/`, C4 `src/cpal/` | Four directories with no stub |
| D | ten depth-one files | D2 `src/dynamics/`, D3 `src/peaks/` | Two directories |
| E | `src/{peaks,pyin,loudness}.rs` | E2 `src/pyin/` | One directory |
| F | thirteen depth-one files, `src/templates/` | F2 `src/store/`, F3 `src/history/`, F5 `src/watch/` | Three directories |
| H | `src/{render,loudness,encode,preset}.rs` | H2 `src/loudness/`, H3 `src/encode/` | Two directories |
| I | four files plus four stubs | I2 `src/channel/` | One directory |
| K | `element.rs`, five siblings, eleven element stubs, fifteen shell files | K2 `src/compose/`, K3 `src/record/`, K4 `src/mix/`, K5 `src/master/` | Four directories |

Only lines G and N satisfy the rule. Lines I and J also omit `src/lib.rs` from the first chunk's
write scope, while 13.3 and 13.4 both say `M<phase>` shares `src/lib.rs` with the first line chunk.

**WHAT SHOULD CHANGE.** List every second-level file in each first chunk's write scope, by name.
Add `src/lib.rs` to the I1 and J1 write scopes. A directory is not a write scope under the rule of
13.0, because the rule names "a directory or a named file" and a stub needs a name.

### CRITICAL: R3. A manifest chunk cannot commit, because `Cargo.lock` has no owner

**OBSERVATION.** Rule one says "**`Cargo.lock` is not a write-scope key**". Every `M<phase>` chunk
creates one or more new member crates. `scripts/dod.sh` runs
`cargo clippy --workspace --all-targets --locked -- -D warnings`, and every `M<phase>` Completion
command carries `--locked`.

**CLAIM.** A new member crate adds a `[[package]]` entry to `Cargo.lock`. With `--locked`, cargo
refuses to update the lock file and exits non-zero. Every manifest chunk therefore fails its own
Completion command and fails the pre-commit hook. P3 closes, and the same class reopens on the
lock file.

**ARGUMENT.** `--locked` is not a preference. It is the flag the gate uses, and the plan's own
commands repeat it. A file that no chunk may write, and that the gate requires to be current, stops
the commit. The rule that resolves a merge conflict on the lock file does not help, because the
chunk cannot stage the file in the first place.

**EVIDENCE.** I copied the tree, added the `duet-time` skeleton exactly as rule one specifies, and
ran the two commands:

```
$ cargo check -p duet-time --locked
error: cannot update the lock file ... because --locked was passed to prevent this

$ cargo clippy --workspace --all-targets --locked -- -D warnings
error: cannot update the lock file ... because --locked was passed to prevent this
```

Without `--locked`, the same skeleton passes clippy and `cargo doc` at `-D warnings`. The skeleton
itself is correct; only the lock ownership is missing.

**WHAT SHOULD CHANGE.** Name `Cargo.lock` in the write scope of every `M<phase>` chunk, beside the
root `Cargo.toml`. Keep the merge-resolution rule, which is sound.

Two claims of rule one are correct, and I verified both. `cargo machete` 0.9.2 reports nothing for
an unused `[workspace.dependencies]` pin, and it reports nothing for a member crate with no
`[dependencies]` section. P3 closes on the mechanism it names.

### CRITICAL: R4. The edit-in-place rule left six new stale pairs

**OBSERVATION.** "How to read this document" adds rule 1: "A decision is edited in place ... It
never adds a lettered successor beside a sentence it contradicts." I read the six ADRs against the
architecture sections that correct them.

**CLAIM.** Six sentences survive beside their own correction, and two of them carry the exact
numbers and phrases that P6 and P23 named. The rule was written and not applied to the documents it
was written for.

**ARGUMENT.** A reader who follows an ADR decision builds the wrong thing. That is the same harm
P19 described. The ADR is the decision record, so a stale ADR decision outranks a corrected
architecture paragraph in a reader's model.

**EVIDENCE.**

| Location | Stale text | Correction |
|---|---|---|
| ADR 0003 item 6b | "`Verb::Gc` runs inside the core, which builds the set in one pass **while it holds the write lock**" | 9.6: the core has no lock, and the walk runs on a job thread |
| ADR 0003 item 15 | "Every `History` method runs under a 5-second budget **on the disk thread**" | 9.6 and ADR 0004 item 17c: a job thread |
| ADR 0004 item 8e | "about **132 MB** at 32 tracks, of which the disk ring buffers are **123 MB**" | ADR 0004 item 12a in the same file: "about 104 MB"; architecture 5.5: 104 MB |
| ADR 0004 item 15 | "`basedrop` ... Its collector runs on **the disk thread**" | 5.7 and 9.6: a job thread. Architecture 5.8 repeats the stale form |
| ADR 0005 item 5 | "A deferred verb runs in a background task and **lands through `cx.update`**" | ADR 0005 item 15 in the same file: a `JobRunner` of two `std::thread` workers |
| ADR 0005 item 11 | "**No tokio type** crosses into GPUI code" | 9.5 rule 4: no runtime-bound type, and the `Runtime` is exempt. This reopens P28 |

ADR 0004 items 8e and 12a contradict each other inside one file, which is precisely the shape rule 1
forbids.

**WHAT SHOULD CHANGE.** Edit all six in place. Correct architecture 5.8 to say a job thread. Then
apply rule 1 as a checklist pass over all seven documents, not as a sentence in one of them.

### CRITICAL: R5. The buffer pool sits on the wrong type, and it breaks two derives

**OBSERVATION.** Section 5.5 gives `ChainTopology` the field `pool: BufferPool`. `ChainTopology`
derives `Debug, Clone, PartialEq, Eq`. `BufferPool` derives `Debug` only. `GraphChain` holds
`chains: Box<[ChainTopology]>` and derives the same four. ADR 0004 item 8e says "The pool is 8 delay
buffers and 4 reverb buffers **per project**". Appendix A says the pool lives "inside `GraphState`".

**CLAIM.** Three statements disagree, and the code as written does not compile.

**ARGUMENT.** `ChainTopology` is one chain, which section 5.5 ties to one `TrackId`. A `BufferPool`
field on that type gives one pool per track. At the PR 11 Q6 budget of 32 tracks that is 32 pools,
or about 288 MB, while the memory table counts 9.0 MB for one project-wide pool. A derived `Clone`
also requires `BufferPool: Clone`, and a derived `PartialEq` requires `BufferPool: PartialEq`.
Neither derive exists, so the two published types do not build. Appendix A puts the pool in
`GraphState`, which is the audio thread's own mutable state, and section 5.5 says the pool is
published and adopted. Both cannot hold.

**EVIDENCE.** `architecture.md:1634` to `1651`, `architecture.md:1705`, `architecture.md:1793`,
`architecture.md:3794`, ADR 0004 item 8e.

**WHAT SHOULD CHANGE.** Move `pool: BufferPool` from `ChainTopology` to `GraphChain`, which is the
graph-wide published value. Give `BufferPool` the derives its container needs, or hold it as an
`Arc<BufferPool>` and state why. Correct the Appendix A row to say the pool arrives with the
topology and that `GraphState` holds only the adopted handle.

The publish-and-adopt path itself is correct, and P5's concurrency half closes. State the memory
peak: two pools are live between the publication and the `basedrop` drain, so the peak is 18 MB and
not 9 MB.

### CRITICAL: R6. Two verbs that the verb list does not contain

**OBSERVATION.** Section 10.2 says `StartView` "calls `Verb::RecentList`, `Verb::RecentAdd`,
`Verb::ProjectCreate`, and `Verb::ProjectOpen`". Section 13.4 repeats it: "`StartView` calls
`RecentList`, which `recent.rs` in F2 provides." The `Verb` enum in section 9.1 holds neither
variant.

**CLAIM.** Two capabilities sit outside the verb list. ADR 0005 decision 1 states that a capability
outside the list falsifies the claim that the list is the whole interface.

**ARGUMENT.** This is the fix for P18 written against the finding text and not against the enum.
`RecentList` is also placed in `duet-project` in the 1.5 table, and `duet-project` sits above
`duet-command`, so a `VerbData` that carries the list would cross an absent edge. Appendix A
compounds it: it still gives `StartView` the mutation path "`on_click`, and the user configuration
file", which is the sentence P18 attacked.

**EVIDENCE.** `architecture.md:2609` to `2691` (the `Verb` body), `architecture.md:3028`,
`architecture.md:3624`, `architecture.md:3790`, `architecture.md:174`.

**WHAT SHOULD CHANGE.** Add `RecentList` and `RecentAdd` to `Verb`, with a `VerbData` variant whose
payload is declared in `duet-command`. Correct the Appendix A row to name the two verbs.

### CRITICAL: R7. Chunk M0 cannot pass its own Completion command

**OBSERVATION.** M0's Completion is `cargo xtask check-conversions && cargo xtask check-placement`.
Section 2.3 gives `check-conversions` two hard rules: it "asserts that a match appears in
`crates/duet-time/src/convert.rs` and nowhere else", and "It fails when it scanned fewer than ten
files". M0 creates only the `duet-time` skeleton, whose `src/lib.rs` holds a `//!` comment and
`#![forbid(unsafe_code)]`.

**CLAIM.** Both rules fail at the end of M0. M0 is the first chunk of phase 0, and its commit runs
the gate that M0 itself just extended. Phase 0 cannot commit.

**ARGUMENT.** The repository holds five `.rs` files under `crates/*/src` and `tools/*/src` today. M0
adds `check_conversions.rs`, `check_placement.rs`, and the `duet-time` skeleton, which gives eight.
Eight is fewer than ten, so the file-count rule fails. `convert.rs` does not exist until T1, so the
existence rule fails as well. The link "M0 before T1" makes T1 unavailable to rescue it.

**EVIDENCE.**

```
$ find crates tools -name '*.rs'
crates/duet/src/app.rs
crates/duet/src/main.rs
tools/plan-db/src/main.rs
tools/plan-db/tests/roundtrip.rs
tools/xtask/src/main.rs
tools/xtask/src/sync_agents.rs
```

Four of those are under `src`. Note that `tests/roundtrip.rs` is outside the two walk patterns that
section 2.3 states.

**WHAT SHOULD CHANGE.** Lower the floor to a count the phase-0 tree meets, or make the floor a
fraction of the discovered crate count. Make the `convert.rs` rule conditional: fail when a match
appears outside `convert.rs`, and pass when no match appears anywhere. Then give M0 a Completion
command that the phase-0 tree can satisfy.

### WARNING: R8. `check-placement` passes on the defect it was built to catch

**OBSERVATION.** Section 1.5 says: "Three revisions of this specification each moved a container and
left its field types behind, so the rule now has a machine that reads it." The prototype is
`roadmap/duet-v1/tools/placement_check.py`. I ran it and then attacked it.

**CLAIM.** The guard reports success it did not earn. It cannot see the exact defect that was
blocker P1, it passes on an empty candidate set, and it ignores its own command-line argument.

**ARGUMENT.** A guard shows what it accepts when you read it. It shows what it rejects only when you
run it with hostile input. Three of my four probes passed.

**EVIDENCE.**

```
$ python3 roadmap/duet-v1/tools/placement_check.py roadmap/duet-v1/architecture.md
CANDIDATE TYPES: 246
PLACED:          246
UNPLACED:        0        EXIT=0
```

Probe A. I restored the exact P1 defect, `sidebar_width: Px` in place of `sidebar_width:
LogicalPx`. Result: `UNPLACED: 0`, exit 0. The `EXTERNAL` allow-list holds `Pixels` and `Px`, so a
GPUI type in a pure crate is invisible to the guard.

Probe B. I renamed every ` ```rust ` fence to ` ```rs `. Result: `CANDIDATE TYPES: 0`, exit 0. There
is no floor on the candidate count. The sibling guard `check-conversions` has exactly that floor;
this one does not.

Probe C. I pointed the script at an empty file. Result: 246 of 246 placed, exit 0. `PATH` is
hardcoded and `sys.argv` is never read, so the argument in the brief has no effect.

Probe D. I planted a new unplaced type. Result: `UNPLACED: 1`, exit 1. The guard does work for that
one case.

Three further holes follow from the code. The candidate set comes from fenced Rust blocks only, so
a type named in prose or in a table is never checked. The variant filter removes any candidate whose
name matches an enum variant anywhere in the document, so a struct named `Track`, `Bus`, or `Master`
is excluded by accident. The `EXTERNAL` list also holds `Marker`, `Input`, `Output`, `Timer`,
`Value`, `Guard`, `Sender`, and `Receiver`, each of which is a plausible Duet domain name.

**WHAT SHOULD CHANGE.** Remove `Px` and `Pixels` from the allow-list, and make a GPUI type in a pure
crate a failure rather than an exemption. Add a floor: fail when the candidate count falls below a
stated number. Read `sys.argv`. Scope the variant filter to the enum that declares the variant. Add
three sentinel tests to the xtask version, as `check-conversions` has, and make each one go red
before the guard exists. Until those tests run, do not describe `check-placement` as a guard.

### WARNING: R9. The placement claim and the edge claim are two different claims

**OBSERVATION.** The brief states that the architect ran "a mechanical placement check (246 of 246
types placed in the section 1.5 ownership table, **zero edge misses against section 1.3**)". I read
the script. It parses one table, `### 1.5 Who owns each type`, and compares a candidate set against
it. It never reads section 1.3.

**CLAIM.** The script computes the placement number. It computes nothing about edges. The
edge claim has no mechanical support, and it is false.

**ARGUMENT.** R1 is an edge miss: `SessionCommand` in `duet-session` names `Mode` and `ViewState`
from `duet-command`, and section 1.3 gives `duet-session` no such edge. A check that reads only the
ownership table cannot see it, because both types are correctly placed in the table.

**EVIDENCE.** `placement_check.py` line 62: `tbl = re.search(r'### 1\.5 Who owns each type\n(.*?)\n###
', s, re.S)`. No other section is parsed. Output is three counters and a list of unplaced names.

Seven types are named in the document and are absent from the 1.5 table. The guard misses each one,
because each appears in prose rather than in a Rust fence.

| Type | Where | Placed? |
|---|---|---|
| `EngineEvent` | 12.4 step 2, Appendix A | No |
| `EngineState` | 5.12, 12.4 step 5 | No |
| `ShutdownError` | 9.5 rule 7 | No |
| `GatewayRequest` | 9.5 rule 5 | No |
| `MeterSlot` | 10.2 | No |
| `MenuPath` | 10.2, Appendix A | No |
| `CommandError` | 12.1 | No |
| `TakeSegment` | ADR 0006 decision 5 | No, and the guard reads no ADR |

Section 12.1 and section 1.5 also disagree in both directions. 12.1 names `CommandError`, which 1.5
omits. 1.5 declares `MediaError`, `CoreError`, `AgentError`, and `DocumentError`, which 12.1 omits.

**WHAT SHOULD CHANGE.** State the placement claim and the edge claim separately, and support each
one. Add an edge check that reads section 1.3 and the 1.5 table together. Extend the candidate set to
backtick-quoted names outside a code fence, or accept that the guard covers fenced code only and say
so at the site. Place the eight types above. Reconcile 12.1 against 1.5.

### WARNING: R10. Three Completion commands are green before their chunk, and one pin is late

**OBSERVATION.** Rule three says a Completion command "fails before the chunk and passes after it".
M4, M7, and M8 all use `cargo check --workspace --locked --all-targets`. M4 and M7 add only root
pins. M8 creates nothing and writes nothing.

**CLAIM.** Three rows repeat P22. A command that is green on the current tree is not a completion
check, and the document says so two paragraphs earlier.

**ARGUMENT.** I verified that an unused root pin changes no build output. `cargo check --workspace`
therefore gives the same result before and after M4 and M7. M8 has an empty write scope, so it is
not a chunk at all; it is a phase header.

**EVIDENCE.** `architecture.md:3373`, `3376`, `3377`. The M8 row reads "Creates the skeleton for:
none | Also writes: none".

A second defect sits in the same area. B.3 gives `criterion` the manifest owner M7, and 13.1 puts
`criterion` in M7's pins. Chunk A4 needs the `[[bench]]` target and runs in phase 5. M7 runs in
phase 7. A4 cannot build its bench.

**WHAT SHOULD CHANGE.** Give M4 and M7 a command that names the new pin, such as a `cargo metadata`
assertion. Delete M8 and fold its phase into the phase table, or give it work. Move the `criterion`
pin to M5.

### WARNING: R11. A deferred `Save` has no place in the queue and no staleness rule

**OBSERVATION.** Section 9.6 lists `Save`, `ProjectOpen`, and `HistoryCheckout` among the deferred
verbs. The queue holds 32 jobs and returns `GatewayError::JobQueueFull` when it is full. Step 1
builds an `Arc<Snapshot>`; step 3 says the core "applies it on the core thread, re-validating
against current state".

**CLAIM.** Three gaps follow. A user `Save` can be refused because an agent filled the queue. The
re-validation rule is stated for `Gc` alone. The snapshot cost is asserted and not derived.

**ARGUMENT.** The queue is a single first-in first-out list with no priority and no reservation. A
script that submits 32 exports blocks the next `Save`, and the user sees a refusal with no recovery
named. For staleness, `Gc` gets a full re-check paragraph. `Save` gets none, so a document edited
during the 2.0 s deferred budget is written from a stale snapshot, and `WriterLedger` then records a
hash for bytes that do not match memory. For cost, step 1 calls the snapshot "a memory copy of small
structures" that "fits the 4.0 ms immediate budget". A 300-bar SATB score at the PR 11 Q6 budget is
not small: every `Note` holds a `SmallVec` of articulations, a `SmallVec` of lyrics, and a
`BTreeMap<String, Value>` extra bag. The copy runs on the GPUI foreground thread, and 5.12 gives it
one frame.

**EVIDENCE.** `architecture.md:2858` to `2873`, `architecture.md:2005` to `2007`,
`architecture.md:688` to `702`, `architecture.md:1070`.

**WHAT SHOULD CHANGE.** Reserve a queue slot for the verbs a user starts, or give `Save` and
`ProjectOpen` a separate path. State the re-validation rule for every deferred verb that writes, not
for `Gc` alone. Measure the snapshot cost, or make the snapshot a shared immutable structure that
costs a pointer copy.

### WARNING: R12. A cancelled job leaves a partial file with no owner

**OBSERVATION.** Section 9.6 says "Cancellation is a polled flag ... There is no forced stop, so no
job is interrupted mid-write." Section 7.4 gives the export two passes, and pass one writes "an f32
intermediate file". `JobState::Cancelled` exists.

**CLAIM.** A cancelled export leaves an intermediate file and a partly written output file. No
section names who deletes them, where they live, or what the user sees.

**ARGUMENT.** "No job is interrupted mid-write" bounds the granularity of the stop. It does not
clean up. The crash matrix of 4.10 covers `state/` and the bundle text files, and `exports/` is
outside it. The store bound table of 4.1 covers four stores and not `exports/`. A user who cancels
an export therefore finds a truncated `.wav` in the export directory that looks like a finished
render. PR MA-03 makes cancel a MUST for the measure job, so the path is required, not optional.

**EVIDENCE.** `architecture.md:2861`, `architecture.md:2400` to `2410`, `architecture.md:1128` to
`1133`, `architecture.md:1114`.

**WHAT SHOULD CHANGE.** State the cleanup contract for a cancelled job and for a failed job. Name
the directory for the intermediate file, give `exports/` a bound and an eviction rule, and state what
the user sees after a cancel.

### WARNING: R13. The ring ceiling has no user-facing answer

**OBSERVATION.** Section 5.5 gives the worst case as 104 MB and says
"`EngineError::RingBudgetExceeded` refuses a configuration whose rings would pass a 128 MB ceiling,
and the user interface names the track count." Rings are allocated "per track when the track starts
to play or arm".

**CLAIM.** `PoolExhausted` got a full treatment under P26. `RingBudgetExceeded` did not. It has no
`GatewayError` mapping, no `fault_text.rs` row, no `EngineFault` variant, and no stated user
recovery.

**ARGUMENT.** The refusal fires at `Play`, because that is when a ring is allocated. A `Play` that
the engine refuses is a hard product wall. The budget is 32 tracks, and the ceiling sits at about 40
playing tracks, so a user who exceeds the stated budget by a quarter loses playback with no named
message. Section 12.4 maps every other engine fault to a message table; this one is absent from it.
The allocation itself is also unstated: 92 MB of rings at `Play` is a long pause on whichever thread
runs it.

**EVIDENCE.** `architecture.md:1749` to `1762`, `architecture.md:1743`, `architecture.md:3296` to
`3309`.

**WHAT SHOULD CHANGE.** Map `RingBudgetExceeded` to a `GatewayError` and to a message, as
`PoolExhausted` is mapped. Name the thread that allocates the rings and the time budget for it. State
what the user does next.

### WARNING: R14. A derived `Deserialize` bypasses the one constructor

**OBSERVATION.** Section 2.6a says of `Finite`: "Two invariants hold, and **the one constructor is
the only way in**." The type then derives `Serialize, Deserialize`. `Unit` and `LogicalPx` have the
same shape.

**CLAIM.** A derived `Deserialize` on a newtype over `f64` writes the inner field directly. It does
not call `new`. A hand-edited `-0.0` in a canonical file therefore produces a `Finite(-0.0)`.

**ARGUMENT.** Invariant 2 canonicalizes a negative zero to a positive zero, and section 2.6a says
that invariant is what makes `to_bits` injective. A `Finite(-0.0)` read from disk compares unequal to
`Finite(0.0)` and hashes differently, while `total_cmp` orders them apart. `Eq` and `Hash` then lie.
PR 11 Q2 and ADR 0002 make a hand edit of the canonical files a product path, so the input is not
hypothetical.

**EVIDENCE.** `architecture.md:479` to `507`, `architecture.md:984`, ADR 0001 decision 3a, ADR 0002
decision 4b.

**WHAT SHOULD CHANGE.** Give `Finite`, `Unit`, and `LogicalPx` a manual `Deserialize` that calls
`new` and returns a serde error on failure. Add the property to the 2.6a proptest: a round trip of
every hand-writable byte sequence either rejects or canonicalizes.

Property 3a of the same test reads `Finite::new(a) < Finite::new(b)`. `new` returns `Option<Finite>`,
so that expression compares two `Option` values, not two `Finite` values. Correct the property.

### WARNING: R15. Two MUST stories have no view chunk

**OBSERVATION.** Section 13.2 names fifteen story rows. M-03 "The vocal tool set" and MA-02 "Master
chain" are both MUST and appear in neither the story table nor the eleven-element table. Design
contract 5.2 requires each of five master stages to paint "the stage's own curve": an equalizer
response curve, a compressor transfer curve with a live dot, and a limiter gain-reduction bar.
Design contract 4.2 row 4 gives every mixer strip four insert-slot buttons, and 1.10 says the Mix
inspector shows "The selected strip's full chain".

**CLAIM.** No element, no file, and no chunk draws an equalizer curve, a compressor transfer curve,
or the slot editor that M-03 needs. K4's goal names `LevelMeter`, `Fader`, `Knob`,
`AutomationLane`, `MixView`, `MeterLayer`, mute, solo, and the bus strips. K5's goal names
`LufsMeter`, `MasterView`, the export dialog, and the measured report.

**ARGUMENT.** This is finding P2 in two new places. The design contract specifies both surfaces in
full. The architecture adopts "the design contract's eleven" elements, and the eleven do not include
a stage curve. A plan built on this text ships M-03 and MA-02 with no user interface.

**EVIDENCE.** `design-contract.md:704` to `728`, `design-contract.md:873` to `896`,
`architecture.md:3041` to `3059`, `architecture.md:3529` to `3546`.

**WHAT SHOULD CHANGE.** Add a twelfth element for a stage curve, or state which existing element
draws it. Add M-03 and MA-02 to the story table with a model chunk, a runtime chunk, and a view
chunk. Confirm the element list with the Designer.

### WARNING: R16. The arm control has three surfaces and one owner

**OBSERVATION.** Section 13.2 says "`TransportBar` holds the arm control of R-05", and K1 writes
`transport_bar.rs`. Design contract 1.7 puts an arm button in every sidebar row. Design contract 4.2
row 8 puts an arm button on every mixer strip.

**CLAIM.** Three surfaces write one field, and the story table names one chunk. K6 writes the
sidebar and K4 writes the mixer strip.

**ARGUMENT.** A control in three places needs one state owner and three readers. `Track::armed` is
that owner, and `Verb::TrackSetArmed` is the write path, so the model is right. The plan is not: the
story row gives R-05 to K1 alone, and two other chunks must draw the same control. A plan author who
follows the table builds one of three.

**EVIDENCE.** `architecture.md:3541`, `architecture.md:3523`, `design-contract.md:143`,
`design-contract.md:721`.

Section 9.1 also keeps `TransportArmRecord { tracks: Box<[TrackId]> }` beside `TrackSetArmed`, while
section 6.1 says "`TransportArmRecord` writes it, one track at a time". A list is not one track, and
two verbs write one field.

**WHAT SHOULD CHANGE.** Name all three view chunks on the R-05 row. State which verb writes
`Track::armed` and delete or redefine the other one.

### CONCERN: R17. The error-enum list and the ownership table disagree

Section 12.1 names nineteen error enums. Four types that section 1.5 declares are absent from it:
`MediaError`, `CoreError`, `AgentError`, and `DocumentError`. One type it names, `CommandError`, is
absent from section 1.5. Reconcile the two lists, and state whether `duet-command` declares one
error enum or two.

### CONCERN: R18. `Track::input` has two ways to say "no input"

Section 6.1 gives `Track` the field `input: Option<InputSelection>` with the comment "`None` means
the track plays back and records nothing". Section 3.4 gives `InputSelection` its own `None`
variant. Two encodings of one state invite a mismatch. Drop the `Option`, or drop the variant.

### CONCERN: R19. The fault report has two single places

Design contract 1.9 says the status bar "is the single place where Duet reports a device problem, so
no mode repeats a device banner except Record". Architecture 12.4 step 4 pushes a notification for
every fault through `WindowExt::push_notification`. A `CaptureShortfall` during a take would then
raise a toast that the contract forbids. The report to the operator does not list this conflict.

### CONCERN: R20. `cargo-nextest` is still unpinned

P22 asked for a pin or for `--no-tests=fail`. Revision 4 took the flag. `scripts/bootstrap.sh` runs
`install_tool cargo-nextest cargo-nextest`, which takes the newest release. The installed version is
0.9.145 and it accepts `--no-tests`, so the risk is small today. An older or newer runner is not
bound by anything. Pin the version in `scripts/bootstrap.sh`.

### CONCERN: R21. `variant_size_differences` is a rustc lint, not a clippy lint

Section 9.1 and ADR 0005 decision 3 both say "`clippy::variant_size_differences`". The root
`Cargo.toml` sets `variant_size_differences = "warn"` under `[workspace.lints.rust]`, and
`.cargo/config.toml` makes it an error. Correct the path in both documents.

---

## 3. Reactive assessment

- **Responsive: PARTIAL.** Nine budgets and eight timeouts are real, and the `JobRunner` takes every
  slow verb off the frame thread. The snapshot cost on that thread is asserted and not measured
  (R11), and the ring allocation at `Play` has no stated budget (R13).
- **Resilient: PARTIAL.** The five-step save, the crash matrix, the four-source live set, the writer
  lock recovery, and the typed errors are correct and complete. Two failures stay uncontained: a
  cancelled export leaves a partial file with no owner (R12), and `RingBudgetExceeded` has no report
  path (R13).
- **Elastic: PARTIAL.** Every channel, ring, queue, store, and cache carries a bound, and the ring
  table is now per state rather than per track. The job queue has no priority, so a user `Save` can
  be refused by agent traffic (R11), and `exports/` carries no bound (R12).
- **Message Driven: PARTIAL.** The vocabulary, the one writer, the versioned state, the note-entry
  drain, and the published buffer pool are correct. `ViewState` reaches `duet-session` through a
  surviving `SessionCommand` variant, which is a cycle and not a message (R1), and the vocabulary
  does not compile (R0).

---

## 4. Plan graph check on section 13

I parsed the phase table and all 53 serial links and compared them mechanically.

1. **The graph is acyclic.** No link runs backwards in phase order.
2. **Every link agrees with its phase.** The one same-phase link is "M0 before T1", which 13.3
   states as the `M<phase>` exception. Every other link crosses a phase boundary.
3. **Every chunk in the phase table has a table row, and every row has a phase.** 64 rows, 0 gaps.
4. **Write scopes are disjoint between line chunks in every phase.** I checked phases 1 to 8 file by
   file. The claim in 13.3 is now correctly qualified.
5. **Seam rule two is not implemented by nine of eleven line write scopes** (R2).
6. **`Cargo.lock` has no owner, and every `M<phase>` chunk needs one** (R3).
7. **Three Completion commands are green before their chunk** (R10). Two run on one platform, and
   each states it at the row, which rule three now allows.
8. **Two MUST stories map to no view chunk** (R15). One MUST story maps to one chunk and needs three
   (R16).
9. **One pin lands two phases after the chunk that needs it** (R10, `criterion`).
10. **No non-manifest chunk writes a policy file.** Rule four holds, and M0 and M5 are both
    dispatched to the Orchestrator.

---

## 5. Consistency across the documents

1. **Closed.** Design contract 3.3 now names `RecordView` as the path-cache owner and `duet-core` as
   the peak source. Two revision-3 rows close together.
2. **Closed.** Design contract 4.4 now names `MeterLayer` as the leaf that owns the single animation
   frame.
3. **Closed.** Design contract 1.6 requires the clamp and the Mix vertical split. `ProjectView::clamped`
   and `ViewState::mix_vertical_split` supply both.
4. **Closed.** Design contract 1.11 and architecture 10.3 name the same eleven elements.
5. **Closed.** ADR 0006 and architecture 10.5 both return `Ticks` from `beat_for_x`, and both say so.
6. **Open.** Six ADR decisions contradict their own corrections (R4).
7. **Open.** Design contract 5.2 and 4.2 specify surfaces that no element and no chunk builds (R15).
8. **Open.** Design contract 1.7 and 4.2 put the arm control where the story table does not (R16).
9. **Open.** Design contract 1.9 and architecture 12.4 give two answers for a device fault (R19).
10. **Open.** Design contract 3.3 gives the path cache a two-part key; ADR 0006 and Appendix A give
    it four parts. The system is what makes the wrapped view correct, so state the four.
11. **Open.** Architecture 3.4 contradicts architecture 10.2 on `SetViewState`, and Appendix A
    contradicts 10.2 on both the command path and the file name (R1).

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The engineering is sound, the plan graph is correct, and the document is closer than any earlier
revision. Every one of the eight blockers closes in the architecture text. The `JobRunner`, the
published buffer pool, the fifteen shell files, the writer-lock recovery, the `total_cmp` order, the
per-state ring table, and the honest statement about the two missing gate lines are all correct
answers to hard questions. I verified the `cargo machete` half of the manifest seam by experiment,
and it holds.

It fails on three families.

The first is new and is the worst. **The vocabulary does not compile.** `Verb` derives `Hash` over
eight payload types that carry none, `ExportSpec` and `Normalization` carry raw `f32` fields, and
`Clipboard` derives `Eq` over `Note` and `Spanner`, which derive neither. The compile-time assertion
that section 3.5 added to prove the property is the line that fails. That is blocker 2 of the premise
review, reopened by the revision-4 type moves and not re-checked after them.

The second is the repeat, now in its fourth revision. Each fix wrote a rule and applied the rule to
the site the finding named. Seam rule two reaches every depth on paper and reaches two lines in the
write scopes. The edit-in-place rule is stated in "How to read" and leaves six new stale pairs in the
ADRs, two of which carry the exact numbers P6 and P23 rejected. `SessionCommand::SetViewState`
survives beside the paragraph that deletes it, and it makes a Cargo cycle.

The third is mechanical and is the one I can prove. **The guards do not guard.** I planted the exact
defect that was blocker P1 and `check-placement` passed. I emptied its candidate set and it passed.
I pointed it at an empty file and it reported 246 of 246. The claim of "zero edge misses against
section 1.3" is not something the script computes, and R1 is an edge miss it cannot see. Chunk M0
cannot pass its own Completion command, because `check-conversions` requires ten source files and a
`convert.rs` that phase 0 does not have. Every `M<phase>` chunk fails the `--locked` gate, because
`Cargo.lock` is in no write scope.

The single biggest risk is unchanged from revision 3, and it is now measurable: the closure appendix
is written against the finding text and not against the resulting types. Appendix F cites a section
for every row, and every citation is accurate. Nine rows are still open, because the fix landed in
the cited section and not in the two or three other places that name the same thing. A machine now
exists to catch that class, and the machine passes on the defect it was built for.

The weakest Reactive property is Message Driven, because the one vocabulary that every component
shares does not build.

### Findings that block plan authoring

1. R0. `Verb` cannot derive `Eq` and `Hash` over ten payload types.
2. R1. `SessionCommand::SetViewState` survives and makes a Cargo cycle.
3. R2. Seam rule two is not applied to nine of eleven line write scopes.
4. R3. Every `M<phase>` chunk fails the `--locked` gate, because `Cargo.lock` has no owner.
5. R4. Six ADR decisions contradict their own corrections.
6. R5. The buffer pool sits on `ChainTopology` and breaks two derives.
7. R6. `Verb::RecentList` and `Verb::RecentAdd` are not in the verb list.
8. R7. Chunk M0 cannot pass its own Completion command.

Findings R8 to R16 are Warnings. Every one must close before FINISH, and none is deferrable.
Findings R17 to R21 are Concerns. Close each one in the specification, or file it in a document
under `roadmap/duet-v1/`.

### The placement script, as run

```
$ python3 roadmap/duet-v1/tools/placement_check.py roadmap/duet-v1/architecture.md
CANDIDATE TYPES: 246
PLACED:          246
UNPLACED:        0
EXIT=0
```

The script runs and reproduces the architect's number. Section 2, finding R8, states what the number
does and does not prove.
