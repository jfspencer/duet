# Specification review, revision 3: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is the third pass. Revision 1 and revision 2 both returned NOT READY.

Sources read in full: `roadmap/duet-v1/architecture.md` (3856 lines), the six ADRs,
`product-requirements.md` sections 8, 9, 10, and 11, `design-contract.md` sections 1, 3.1 to 3.6,
4.4 to 4.6, `CLAUDE.md`, the root `Cargo.toml`, `scripts/dod.sh`, `crates/duet/src/`, and
`research/crate-survey.md`.

Revision 3 is the strongest of the three. Every OPEN row closes. Nineteen of the twenty-four new
findings close with a real mechanism. Appendix E is honest about what it changed.

It does not close. The defects below are of two families. The first family is the repeat: a fix
placed a new type, a new field, or a new chunk, and did not apply the document's own placement rule
to it. The second family is new: three mechanical gates in `scripts/dod.sh` reject the plan's own
chunk shapes.

---

## 1. Closure check against the revision 2 review

### 1.1 The three OPEN rows

| Row | Section | State | Reason |
|---|---|---|---|
| The `Verb` derives over `Finite` and `Span` | 2.6, 2.6a, 3.5 | CLOSED | `Span` derives the full set. `Finite` gives `Eq` and `Hash` over `to_bits`. The `Ord` half is wrong; see P9. |
| The member manifest and the lock owner | 13.0 rule one | CLOSED | `M<phase>` owns every member manifest and the lock. The mechanism fails `cargo machete`; see P3. |
| `duet-project` reaches types by re-export | 1.3 rule 3, 4.3 | CLOSED | 4.3 now says the types live in `duet-session` and the edge is direct. |

### 1.2 The sixteen PARTIAL rows

| Row | Section | State | Reason |
|---|---|---|---|
| `gc` against an uncommitted take | 4.4, 4.9 | CLOSED | Four sources, read through the gateway. `Transaction::sources` carries them. |
| The `LatencyReport` source | 5.4 | CLOSED | `split` uses `checked_sub`, returns `Result`, and a test feeds one frame. |
| The module seam on lines A and B | 13.2 | CLOSED | A1 and B1 write scopes now list every module file. Line K does not; see P4. |
| Zoom invalidates every pixel cache | ADR 0006 item 6 | CLOSED | 1024 paths or 64 MB, one owner, one key, derived from 512 visible paths. |
| Verbs for the MUST stories | 9.1, 13.2 | PARTIAL | Six verb groups exist. `Track` holds no field for four of them; see P8. |
| The capture shortfall has no field | 6.1, 6.4 | CLOSED | `TakeFlags` is a bitset, and a table names the writer of each flag. |
| The soak test in the gate | 14 rung one | CLOSED | `--run-ignored ignored-only` with a filter, in a nightly workflow. |
| The ` as ` guard | 14 rung two | PARTIAL | Four rules and three tests are right. No chunk registers the subcommand; see P24. |
| No timeout on the calibration and the MIDI open | 5.12 | CLOSED | Eight rows, each with an action on expiry. |
| The shell surfaces | 13.2 chunk K6 | PARTIAL | `StartView` and `ViewStateStore` land. Three shell views still have no chunk; see P2. |
| `smufl` named in 10.5 | 10.5, B.3 | CLOSED | The sentence is replaced, and three places agree. |
| PR X-08 layout persistence | 10.2 | PARTIAL | `ViewState` exists and breaks the placement rule three times; see P1. |
| The non-finite `f64` against `Verb: Eq` | 2.6a | CLOSED | `Finite` gives a total, injective equality key. |
| A stale conditional on the pitch overlay | 13.2 | CLOSED | The table cites PR R-15 as an existing SHOULD story. |
| The frame driver, the resynchronize path, the writer lock | 10.2, 5.8, 3.8 | CLOSED | Unchanged from revision 2 and still correct. |
| The complexity budget | B.1 | CLOSED | Unchanged. The budget rule stands. |

### 1.3 The twenty-four new findings

| # | State | Reason |
|---|---|---|
| N1 | CLOSED | `Span` and `Finite` carry the derives the vocabulary needs. See P9 and P10 for the order. |
| N2 | PARTIAL | The five container types are placed. Their field types are not, and `ViewState` adds three new breaks. See P1 and P13. |
| N3 | CLOSED | `checked_sub`, a typed error, and a one-frame test. |
| N4 | CLOSED | Four sources, the gateway path, and three stated refusals. |
| N5 | CLOSED | Seam rule one names one owner. See P3 for its cost. |
| N6 | PARTIAL | Six stories gain a model, a verb, and a chunk. Four model fields and three views are absent. See P2 and P8. |
| N7 | CLOSED | I2 moves to phase 6, I3 and J2 to phase 7. I checked every row of the phase table. |
| N8 | CLOSED | One owner, one key, a count bound, and a byte bound. |
| N9 | CLOSED | `MeterTap` renames the tap, and 1.5 gives every shared type an owner. |
| N10 | CLOSED | `Take::flags` holds a `TakeFlags` set. |
| N11 | PARTIAL | The guard matches both attribute forms. It has no registration point and no gate line. See P24 and P25. |
| N12 | CLOSED | The nightly workflow uses a filter. The hook runs no ignored test. |
| N13 | CLOSED | The calibration, the pass, and the MIDI open each carry a deadline. |
| N14 | PARTIAL | The three named sentences are gone. Six new stale sentences sit in the ADRs. See P19. |
| N15 | CLOSED | The buffer pool and a memory table replace the wrong number. The pool creates P5, P23, and P26. |
| N16 | CLOSED | `SlotKind::PitchCorrect` is deleted. |
| N17 | CLOSED | `state/` holds the marker, the lock, and the socket path. ADR 0005 still says `derived/`; see P19. |
| N18 | CLOSED | Two assertions over six types, and `EditCommand` is one name for one thing. |
| N19 | PARTIAL | The line tables carry a Completion column. Six manifest chunks have no row. See P16 and P22. |
| N20 | CLOSED | The media line is `N`. |
| N21 | CLOSED | The reason is a one-signed error with a measured bound of 8 ms. |
| N22 | CLOSED | Section 1.6 defines `BundleDocument` with three methods and an error type. |
| N23 | CLOSED | `Move { by: Ticks }` and `CurvePoints` as an enum over two domains. |
| N24 | CLOSED | The `Drop` path logs, absorbs every failure, and has no `?`. |

### 1.4 The nine cross-document rows

| Row | State | Reason |
|---|---|---|
| The rows closed in revision 2 | CLOSED | Still correct after the edits. |
| PR M-04 reverb and delay buses | PARTIAL | `BusRole` exists. `StripKind::Bus` carries no role, so the model loses it. See P12. |
| PR M-02 and M-06 mute and solo | PARTIAL | The model and the verbs exist. No published type carries `AudibleState`. See P11. |
| PR 8.2 against `PitchCorrect` | CLOSED | The variant is deleted. |
| PR X-08 and C-03 context | PARTIAL | The state exists in the wrong crate. See P1. |
| PR X-07 menus | CLOSED | `MenuHost`, one list of pairs, and chunk K1. |
| PR C-01 start surface | PARTIAL | `StartView` exists and opens a file, which rule 6 forbids. See P18. |
| PR MA-03 measured report | CLOSED | `MasterMeasure`, `Started { job }`, H2, I4, K5. |
| 10.5 and 4.3 internal conflicts | CLOSED | Both sentences are corrected. |

### 1.5 The ten plan-graph rows

| Row | State | Reason |
|---|---|---|
| The graph is acyclic | CLOSED | I re-checked every link in 13.4 against the phase table. No link runs backwards. |
| Three links contradict the phase table | CLOSED | All three are gone. |
| Every serial link carries a reason | CLOSED | Each new link names a type or a file. |
| "G4 before K2" against the crate graph | CLOSED | It is now "G4 before I3", and the core owns the drain. |
| Write scopes disjoint in every phase | PARTIAL | `M<phase>` and the first line chunk both write `src/lib.rs`. See P21. |
| The manifest seam | CLOSED | One owner per phase. See P3 for the gate failure. |
| No chunk carries a completion command | PARTIAL | Six manifest chunks still carry none. See P16. |
| Six MUST stories map to no chunk | PARTIAL | Those six land. Five other MUST stories lose their view. See P2. |
| Three repository files have no owner | PARTIAL | The engine README lands in C1. `tools/xtask/src/main.rs` and the gate line do not. See P24 and P25. |
| Chunk E2 has a consumer | CLOSED | PR R-15 is cited. |

**Count: 45 CLOSED, 17 PARTIAL, 0 OPEN.**

---

## 2. New findings in revision 3

### 2.1 Ranked table

| # | Tier | Finding | What must change | Section |
|---|---|---|---|---|
| P1 | Critical | The N6 fix breaks the placement rule of section 1.3 in three new places, all in one type. Section 10.2 puts `ViewState` and `ProjectView` in `duet-session`. `ViewState` holds `selection: Selection`, and `Selection` is a `duet-score` type from section 3.4. The graph gives `duet-session` one edge, to `duet-time`. `ViewState` also holds `sidebar_width: Px` and `inspector_width: Px`. `Px` is a GPUI type, so `duet-session` would name GPUI, which section 1.2 forbids for a pure crate. `SessionCommand::SetViewState` then carries that GPUI type across the transport, which the assertion of section 3.5 exists to stop. `ProjectView` holds `mode: Mode`, and section 10.1 defines `Mode` in `crates/duet`, the top of the graph. `Verb::ViewGetState { mode: Mode }` repeats it. `ZoomStep`, `ScrollOffset`, `TrackFilter`, `WindowGeometry`, and `Permille` are named and never placed. This is the exact defect class that blocked revision 1 and revision 2. | Place every field type in the lowest crate that each consumer reaches. Define `Mode` in `duet-session` and re-export it from `crates/duet`. Hold widths as an integer newtype, not `Px`. Decide whether `Selection` moves to `duet-time` or whether `ViewState` holds a serializable selection of its own. Then add the five unplaced types to the section 1.5 table. | 1.3, 1.5, 3.5, 9.1, 10.2 |
| P2 | Critical | Three entities of the section 10.2 tree have no file and no chunk: `TopBar`, `TransportBar`, and `ModeSwitcher`. `CoreHost` and `AgentBridge` have none either. K1's write scope is `main.rs`, `element.rs`, five sibling module files, `shell/mod_files.rs`, and `shell/menu.rs`. K6 writes seven named shell files, and none of them is a top bar, a transport bar, or a mode switcher. Five MUST stories therefore lose their view: C-15 the top bar of notation options, C-17 tempo, R-05 arm a track, R-06 count-in, and C-22 playback. R-01 and M-01 need the mode switcher. Design contract 1.2, 1.4, 1.5, and 1.8 specify all three surfaces in full. This is finding N6 in three new places. | Name the file and the chunk for `TopBar`, `TransportBar`, `ModeSwitcher`, `CoreHost`, and `AgentBridge`. Add each MUST story of section 8 to the story table of 13.2, not only the six that revision 2 missed. State which chunk holds the per-track arm state. | 10.2, 13.2, PR 8 |
| P3 | Critical | Seam rule one makes every `M<phase>` chunk fail the Definition of Done. The rule says the manifest chunk writes a member `Cargo.toml` with "the dependency list of section 1.2" and a `src/lib.rs` that holds only a `//!` comment and `#![forbid(unsafe_code)]`. `scripts/dod.sh` runs `cargo machete`, which fails on an unused dependency. M3 would add `cpal`, `rtrb`, `triple_buffer`, `basedrop`, `crossbeam`, `arrayvec`, `gix`, `blake3`, `notify`, `quick-xml`, `midir`, `hound`, `bwavfile`, `flacenc`, and `symphonia` beside five empty library files. Every one is unused at that commit. The pre-commit hook blocks the commit, so no manifest chunk can land as specified. | State that a manifest chunk adds only the dependencies its phase's line chunks consume, or that the manifest chunk and the first line chunk of each crate are one chunk. A phase that cannot commit is not a phase. | 13.0 rule one, 13.1, 13.3, `scripts/dod.sh` |
| P4 | Critical | Chunk K1 cannot compile and therefore cannot commit. Section 10.3 says "Chunk K1 writes that file with every `mod` line", and K1's write scope holds `crates/duet/src/element.rs` and no file under `crates/duet/src/element/`. The eleven element files are written by K2 to K6, which run one phase later. At the end of phase 6 `element.rs` names eleven modules that do not exist. `cargo clippy --workspace --all-targets --locked -- -D warnings` fails, and the hook blocks the commit. Seam rule two covers top-level module files only, so it does not reach a second-level directory. | Give K1 the eleven stub files in its write scope, or move the `mod` lines to the chunk that writes each file and state how K2 to K6 avoid the collision. State the same answer for `src/shell/`, where K6 writes seven files that `shell.rs` must name. | 10.3, 13.0 rule two, 13.2 |
| P5 | Critical | The `BufferPool` has two writers and no stated synchronization. Section 5.5 says `configure` runs off the audio thread and "assigns a `PoolSlot` to every `Delay` and `Reverb` slot in the new topology". Migration rule 3 says a dropped slot "returns its `PoolSlot`" and that "a pool buffer is only released back to the pool's free list", and that migration runs on the audio thread inside the cycle. Appendix A says the pool lives "inside `GraphState`", owned by the audio thread, with "Nothing; no other thread holds a reference". Two threads therefore mutate one free list, and the only mechanisms that would make that safe are a lock or an atomic, which ADR 0004 decision 11 forbids on the audio thread. The fix for N15 created a new real-time defect. | State one owner of the free list. The straightforward answer is that `configure` computes the whole assignment off the audio thread and publishes it inside `GraphChain`, so the audio thread reads a handle and never edits a free list. Then say what a release means. | 5.5, 5.6, Appendix A, ADR 0004 8e |
| P6 | Critical | A deferred verb has no thread. Section 5.7 says "In the application, the core runs on the GPUI foreground thread inside one entity". Section 4.4 says `Verb::Gc` "runs inside the core, which builds the set from all four sources in one pass while it holds the write lock". That set includes "every commit reachable from any reference", which is a full git object walk, plus a walk of `media/`. Section 5.12 gives `Gc` a 300 s client timeout, so the design expects it to be slow. A multi-second walk on the GPUI foreground thread stalls every window. Section 9.1 says a deferred verb "goes to a background task" and that "the core is still mutated on one thread only", and the two sentences cannot both hold for a verb that needs the core's live state. Section 4.5 adds a third thread: it puts the 5 s gix budget "on the disk thread", and section 5.7 gives the disk thread to `duet-engine`, which `duet-project` does not depend on. `Save`, `HistoryCheckout`, and `ProjectOpen` have the same problem. | Name the execution model for a deferred verb that reads or writes core state. State which thread owns `duet-project`'s file work, and stop calling it the disk thread. State what "the write lock" is, because the threading contract of 5.7 has no lock. | 4.4, 4.5, 5.7, 5.12, 9.1 |
| P7 | Critical | The plan does not match the repository. `crates/duet` exists now and holds `src/main.rs` and `src/app.rs`. `tools/xtask` exists. `crates/duet/Cargo.toml` exists with `gpui-kit`. M0 says "Skeletons for `duet-time` and `tools/xtask`", and M6 must create a skeleton for `crates/duet` under seam rule one, which also says the skeleton holds a `src/lib.rs`. `crates/duet` is a binary and has no library target. K1 writes `main.rs` and leaves `src/app.rs` with no reference, so `cargo clippy -D warnings` fails on the dead module or on a stale `Root` view. No section and no chunk mentions `app.rs`. | State, per existing crate, whether the plan extends it or replaces it. Put `crates/duet/src/app.rs` in a write scope, or state that K1 deletes it. Correct the M0 and M6 rows. | 13.0 rule one, 13.1, 13.2, the repository |
| P8 | Critical | Four new verbs write to model fields that do not exist. Section 6.1 gives `Track` the fields `id`, `part`, `staff`, `takes`, `active`, `record_mode`, and `align`. `SessionCommand::TrackAdd { name: TrackName }` and `TrackRename { name }` have no `name` field to write. `TrackSetInput { input: InputSelection }` and `TrackSetMonitor { monitor: MonitorMode }` have no field either. R-05 makes a per-track arm a MUST, and neither `Track` nor `Transport` holds an armed set; `TransportArmRecord { tracks }` names a list that no type stores. This is finding N10 repeated in four places: a command exists and its state has nowhere to live. | Add `name`, `input`, `monitor`, and `armed` to `Track`. State which of them reach `session/session.json` and which are runtime state. Re-check every new `SessionCommand` variant against the struct it edits. | 6.1, 3.4, 9.1 |
| P9 | Warning | ADR 0001 decision 3a says `Finite` implements "`PartialEq`, `Eq`, `Hash`, `PartialOrd`, and `Ord` **over the bits**". An order over `f64::to_bits` is not the numeric order for a negative value. The sign bit makes every negative value compare above every positive value, and among negative values the order reverses. Under that rule -1.0 sorts above 1.0, and -2.0 sorts above -1.0. Section 2.6a shows a different rule: `PartialOrd` "delegates to `f64`". Two normative documents give two answers, and the ADR's answer is wrong. Section 2.6a also says the canonical writer "sorts a curve by value when it needs a total key", and a gain curve is routinely negative, so the wrong order reaches the file on disk. | Correct ADR 0001 decision 3a. State that `Ord` is `f64::total_cmp`, which is total, numerically correct for every non-NaN value, and needs no panic primitive. Add a proptest property that the order agrees with `<` for every pair of finite inputs. | ADR 0001 item 3a, 2.6a |
| P10 | Warning | The `Finite` comparison code as written does not pass the lint policy. Section 2.6a gives a hand-written `PartialOrd` and a separate hand-written `Ord`. `clippy::non_canonical_partial_ord_impl` fires when a type implements both and `partial_cmp` does not return `Some(self.cmp(other))`. The obvious `Ord` body is `self.0.partial_cmp(&other.0).unwrap()`, and `clippy::unwrap_used` is denied. Section 2.6a also says `Finite` "derives" `PartialOrd` and `Ord`, three lines under a manual `impl`. A derive over an `f64` field does not compile. | Write `Ord` as `self.0.total_cmp(&other.0)` and `PartialOrd` as `Some(self.cmp(other))`. Delete the word "derives" for those two traits. | 2.6a |
| P11 | Warning | `AudibleState` has no path to the audio thread. Section 7.1 says it is "computed with the topology, never in the cycle, so the audio thread reads one boolean per strip". `ChainTopology` in section 5.5 holds `track`, `channels`, `polarity`, `pre_fader`, `post_fader`, `sends`, `meter_tap`, and `output`. It holds no mute field, no solo field, and no `AudibleState`. `ParamSnapshot` carries values addressed by `ParamId`, and a mute is a boolean, not a parameter value. M-06 is a MUST and its runtime value never arrives. | Add `audible: AudibleState` to `ChainTopology`, or state the snapshot that carries it. Name the chunk that publishes it. | 5.5, 7.1 |
| P12 | Warning | The M-04 model loses the bus role. `SessionCommand::BusAdd { name, role: BusRole }` and `Verb::MixAddBus` both carry a `BusRole`. Section 7.1 gives `Strip` a `kind: StripKind`, and `StripKind` is `Track(TrackId)`, `Bus`, or `Master`. After the command applies, a reverb bus and a delay bus are the same value. K4's goal names "the reverb and delay bus strips", and the view has nothing to read. The `AudibleState` rule 2 also needs to know which bus a soloed strip feeds. | Give `StripKind::Bus` a `BusRole` payload, or give `Strip` a `role` field. State how the mixer view draws each role. | 7.1, 3.4, 9.1 |
| P13 | Warning | The N2 fix moved five container types and left their field types unplaced. `MidiRecord { port: MidiPortId, at: SampleClock, message: MidiMessage }` lives in `duet-command`, so `MidiPortId`, `SampleClock`, `MidiMessage`, `MidiNote`, and `Velocity` must live in `duet-command` or lower. Section 8.1 uses `MidiPortId` and `MidiPortInfo` in the `duet-midi` trait, and section 5.1 uses `SampleClock` in the `duet-engine` backend trait. Neither type appears in the section 1.5 ownership table. `Calibration` lives in `duet-session` and holds `block: FrameCount` and `measured_at: Timestamp`. `FrameCount` is an engine type, and `duet-session` has no engine edge. `Timestamp` has no crate and no dependency; the `duet-session` dependency list is `duet-time`, `serde`, `smallvec`, and `thiserror`, with no clock crate. `InputSelection` holds `ChannelIndex`, which section 5.1 uses in `Cycle::input`. | Add every field type of every moved container to the section 1.5 table. Pin a clock crate for `Timestamp`, or hold the time as an integer the caller supplies. | 1.3, 1.5, 5.1, 5.4, 8.1, 8.3 |
| P14 | Warning | Seam rule one builds every member manifest from "the dependency list of section 1.2", and that column is wrong for four crates. The `duet-export` row lists `rubato`, `thiserror`, and `tracing`, while the graph in 1.3 gives it six internal edges. The `duet-core` row lists four third-party crates and none of its twelve internal edges. `duet-agent` and `duet` do the same. A manifest chunk that follows the rule literally produces four crates that do not compile. | State the convention for the column, and list every internal crate in every row, or point seam rule one at section 1.3 instead. | 1.2, 1.3, 13.0 rule one |
| P15 | Warning | The test dependencies have no owner and no pin. T1, T2, and 2.6a require proptests. A4 requires a `criterion` bench, and section 14 runs `cargo bench -p duet-engrave`. `proptest` and `criterion` appear in no section 1.2 row, in no Appendix B.3 row, in no survey row, and in the root `[workspace.dependencies]`. A criterion bench also needs a `[[bench]]` target with `harness = false` in the member manifest, and seam rule one forbids A4 to touch a manifest. | Add `proptest` and `criterion` to Appendix B.3 with a licence and a manifest owner. State that the `M` chunk writes every `[[bench]]` and `[dev-dependencies]` entry its phase needs. | 1.2, B.3, 13.0 rule one, 13.2 |
| P16 | Warning | Six manifest chunks have no table row, and two manifest chunks write policy files that the Orchestrator owns. Section 13.1 gives M0, M1, and M2 a goal, a write scope, and a Completion command. M3 to M8 appear only in the phase table of 13.3, with no crate list, no write scope, and no Completion command. Section 13.1 also gives M0 `scripts/dod.sh` in its write scope, and 13.3 gives M5 `.github/workflows/soak.yml`. Section 14 says "The Orchestrator owns the workflow file and the `scripts/dod.sh` edit". A chunk write scope and an ownership statement cannot both hold. | Add a row for M3 to M8 with the crate list and the Completion command. Remove both policy files from every chunk write scope, and state that the Orchestrator lands them in a separate change before the phase runs. | 13.1, 13.3, 14 |
| P17 | Warning | The stated MIDI budget does not match its own arithmetic. Section 5.12 gives 28.7 ms for a MIDI note to a visible note head, and it explains the number as "the 12.0 ms sound path plus one frame of drain and one frame of paint". Section 8.3 repeats the same composition. One frame at 60 Hz is 16.7 ms, so 12.0 plus two frames is 45.4 ms, not 28.7 ms. The core drains the queue in a task, and GPUI gives no order between a spawned task and the render pass, so a note that arrives after render waits a whole frame. | Publish the honest worst case, or state the mechanism that puts the drain and the paint in one frame. A `cx.observe` on the core entity is the push form that would justify one frame. | 5.12, 8.3 |
| P18 | Warning | `StartView` opens a file, and two rules forbid it. Section 1.3 rule 6 says "`crates/duet` depends on no crate that opens a file", and section 10.1 says the crate "holds presentation only. It computes no domain value". Section 10.2 says the recent project list "lives in the user's configuration directory", and Appendix A gives `StartView` the mutation path "`on_click`, and the user configuration file". The peak path was fixed the correct way in section 1.3: the core reads and the view paints. The recent list is not. | Give the recent list a verb and let the core own the file, or state the exception and its reason at the site. Name the crate that reads the user configuration directory. | 1.3 rule 6, 10.1, 10.2, Appendix A |
| P19 | Warning | Six sentences survive beside their own correction, which is finding N14 in new places. ADR 0003 decision 13 still writes the save marker to `derived/save.commit` and `fsync`s `derived/`, while decision 13a in the same file moves it to `state/`. ADR 0005 decision 8 still writes `derived/agent.sock.path`, and decision 9 still writes `derived/duet.lock`. ADR 0004 decision 6a still puts the calibration cross-correlation "in `duet-analysis`", while architecture 1.2 correction 3 moves it to `duet-dsp` and 13.4 changes the link for that reason. ADR 0006's consequence list still says "the 256-path bound keeps the cache memory known", while decision 6 in the same file sets 1024 paths or 64 MB. Architecture 10.5 and ADR 0006 both say the design contract writes `beat_for_x(x) -> f32`, and design contract 3.1 already returns `Ticks`. A reader who follows the wrong sentence builds the wrong thing. | Delete all six. Add a rule that an ADR decision is edited in place, never annotated with a lettered successor that contradicts it. | ADR 0003 item 13, ADR 0004 item 6a, ADR 0005 items 8 and 9, ADR 0006 consequences, 10.5 |
| P20 | Warning | The writer lock has no recovery path for the window process. Section 3.8 step 1 says the window creates `state/duet.lock` with `create_new`, which fails when the file exists. Steps 2 to 4 describe "a second writer", and step 4 ends with the command-line answer: exit with `GatewayError::ProjectBusy`. After a crash the next window start meets an existing lock and a dead process, and no step names that path. Section 4.1 states the second cost: a deletion of `state/` while the application runs "breaks the writer lock". It does not say what the running application does then, so a second process can take the lock and two writers edit one bundle. | State the window start path against a stale lock. State what the running application does when its own lock file disappears, and make the answer stop the writes. | 3.8, 4.1, 4.10 |
| P21 | Warning | Section 13.3 claims "No two chunks in one phase share a write scope", and every phase breaks it. Seam rule one gives `M<phase>` the `src/lib.rs` of each new crate. Seam rule two gives the first line chunk the same file, and ten chunk rows list `src/lib.rs` in their write scope: A1, B1, C1, D1, E1, F1, G1, H1, I1, J1, and N1. The link "M<phase> before every chunk in that phase" serializes them, so the hazard is not a parallel write. The claim is still false as written, and a plan author who trusts it will dispatch the two together. | Correct the claim. State that `M<phase>` runs alone and that its `lib.rs` is the only file it shares. | 13.0, 13.3 |
| P22 | Warning | Two Completion commands do not fail before their chunk. Section 13 says each command "fails before the chunk and passes after it". M1 and M2 both use `cargo check --workspace --locked`, which is green on the current tree and stays green. A command that passes before the work is not a completion check. Every `cargo nextest run -p <crate>` command in an empty crate also depends on the exit code that `cargo nextest` gives for zero matched tests, which changed between releases and which `scripts/bootstrap.sh` does not pin. | Give M1 and M2 a command that names the new crates, such as a `cargo metadata` assertion or a `cargo check -p` list. Pin the `cargo-nextest` version, or add `--no-tests=fail` to every filtered command. | 13.0, 13.1, 13.2 |
| P23 | Warning | The engine memory table undercounts the ring buffers. The row reads "Ring buffers, 5 s per channel at 96 kHz, 1.9 MB, 32 tracks, 2 channels each", for 123 MB. Section 5.8 lists one `rtrb` ring per playback channel and one per capture channel, plus one per input channel between the two cpal callbacks. A stereo track that plays and records therefore needs four rings, not two, and the cpal bridge adds more. The stated total of about 132 MB is the floor, not the number. | State the ring count per track in each state: idle, playing, and armed. Recompute the total at the PR 11 Q6 budget. | 5.5, 5.8, 5.10 |
| P24 | Warning | The conversion guard has no registration point. Section 14 says chunk M0 writes `tools/xtask/src/check_conversions.rs`, and M0's write scope lists that file and `scripts/dod.sh`. `tools/xtask/src/main.rs` is in no write scope, so nothing adds the `mod` line and nothing adds the subcommand arm. `cargo xtask check-conversions` therefore does not exist, and it is the Completion command of M0. Section 2.3 also calls the guard "an integration test in `tools/xtask`", while section 14 calls it a subcommand. Those are two different things. | Add `tools/xtask/src/main.rs` to the M0 write scope. Choose one form, a subcommand or a test, and use one name for it in both sections. | 2.3, 13.1, 14 |
| P25 | Warning | Rung one claims a gate that `scripts/dod.sh` does not run. The rung-one list says "`cargo xtask sync-agents --check` and `cargo xtask check-conversions` are green". I read the file. It runs `fmt`, `clippy`, `doc`, `nextest`, `doctest`, `deny`, `machete`, `sync-agents`, `bash -n`, and `typos`. It does not run `check-conversions`, and no chunk may edit it under the rule of P16. A guard that is not in the gate is documentation, not a guard. | State that the Orchestrator adds the line before phase 0 ends, and make the sequence explicit. Until the line lands, do not describe the guard as a gate. | 14 rung one, `scripts/dod.sh` |
| P26 | Warning | The buffer pool holds 8 delay buffers and 4 reverb buffers for a whole project, and no surface reports exhaustion. The budget of PR 11 Q6 is 32 tracks, and `MAX_SLOTS` is 8 pre-fader and 8 post-fader per strip. A user reaches the limit at a ninth delay anywhere in the project. `configure` returns `ConfigError::PoolExhausted { kind }`, and no `VerbOutcome`, no `GatewayError` variant, no notification text, and no design-contract state covers it. | State the reason for 8 and 4 against the product, not against memory alone. Map `PoolExhausted` to a gateway error and to a user message in `fault_text.rs`. | 5.5, 12.4, ADR 0004 8e |
| P27 | Concern | The Completion column rule says every command runs "on macOS and on Linux". G2's command carries "(macOS runner)" and G3's carries "(Linux runner)". The exception is stated at the row, which is the right form, and the blanket sentence above it is now false. | Reword the rule to allow a named per-platform command. | 13.0, 13.2 |
| P28 | Concern | `duet` depends on `tokio`, and section 10.2 gives `Entity<AgentBridge>` a `tokio::runtime::Runtime`. Rule 4 of section 9.5 says "No tokio type crosses into GPUI code". The rule means no timer, no socket, and no future, and it does not say so. | Reword rule 4 to name the types it bans and to exempt the runtime handle its own rule 2 requires. | 9.5, 10.2 |
| P29 | Concern | `ViewState` does not cover two things design contract 1.6 requires. The contract adds a `v_resizable` inside the Mix work area with its own split, and `ViewState` holds one `body_split`. The contract also says "On restore, Duet clamps each value against the current window width before it applies the value", and no architecture section states the clamp. | Add the Mix vertical split to `ViewState`. State the clamp rule and the chunk that applies it. | 10.2, design contract 1.6 |
| P30 | Concern | ADR 0004 decision 13 calls the MIDI record "a duplicated 8-byte record". Section 8.3 and section 5.8 both say 16 bytes. ADR 0003 decision 12 says undo is "bounded at 200 transactions", and section 4.9 has three bounds. Neither number changes a decision, and both make a reader check which document is current. | Correct both numbers in the ADRs. | ADR 0003 item 12, ADR 0004 item 13 |

### 2.2 Reactive assessment

- **Responsive: PARTIAL.** Nine budgets and eight timeouts are real. `Gc` and `Save` have no stated
  thread, and the core sits on the GPUI foreground thread (P6). The note-head budget contradicts its
  own arithmetic (P17).
- **Resilient: PARTIAL.** The five-step save, the crash matrix, the four-source live set, the typed
  errors, and the calibration split are correct and complete. Two failures stay uncontained: the
  writer lock after a crash (P20) and `PoolExhausted` with no report path (P26).
- **Elastic: PARTIAL.** Every channel, ring, queue, store, and cache carries a bound, and the path
  cache bound is now derived from the visible set. The ring-buffer memory number is a floor, not a
  total (P23), and the delay pool is a hard project limit with no story (P26).
- **Message Driven: PARTIAL.** The vocabulary, the one writer, the versioned state, and the note-entry
  drain are correct, and the `duet-midi` reach of revision 2 is gone. The property does not reach PASS
  because `ViewState` carries a GPUI type across the transport (P1), and because the buffer pool is
  shared state between two threads rather than a published value (P5).

---

## 3. Plan graph check on section 13

1. **The graph is acyclic.** I checked every link in 13.4 against the phase table of 13.3. No link
   runs backwards in phase order.
2. **Every link agrees with its phase.** The three contradictions of revision 2 are gone. I verified
   the four new links: "D1 before C4", "G4 before I3", "I1 before I2, I3, and I4", and "J1 before J2".
3. **Write scopes are disjoint between line chunks in every phase.** I checked phases 1 to 8 file by
   file. The only sharing is `M<phase>` against the first line chunk of each crate, over `src/lib.rs`
   (P21). That link is serial, so it is a false claim rather than a collision.
4. **A later chunk in a line must edit a parent module file it does not own.** C2 writes `src/disk/`
   and needs a `mod` line in `src/disk.rs`, which C1 owns. The same holds for C3, D2, D3, E2, F2, F3,
   H2, H3, and I2. Every case is serial across phases, so the risk is an unowned edit, not a race.
   Seam rule two does not cover it.
5. **Line K breaks at phase 6.** K1 publishes eleven `mod` lines with no files (P4).
6. **Six manifest chunks have no row** (P16). Three repository files still have no owner:
   `tools/xtask/src/main.rs`, the `scripts/dod.sh` gate line, and `crates/duet/src/app.rs` (P7, P24).
7. **Not every MUST story maps to a chunk.** C-15, C-17, R-05, R-06, and C-22 have a verb and no view
   chunk. R-01 and M-01 need a mode switcher that no chunk writes (P2).
8. **Two Completion commands do not fail before their chunk** (P22). Two run on one platform (P27).
   No Completion command names a test module that no chunk writes; I checked all thirty-six.
9. **No non-manifest chunk writes a `Cargo.toml`.** G1 states the rule for itself. A4 needs a
   `[[bench]]` target that no chunk may write (P15).
10. **`M<phase>` depends on the prior phase only through the phase order.** That is sound, because a
    skeleton manifest needs no source from an earlier phase.

---

## 4. Consistency across the three documents

1. **Closed.** Design contract 3.1 now returns `Ticks` from `beat_for_x`, and 4.4 names `MeterLayer`
   as the leaf that owns the frame. Both match the architecture.
2. **Closed.** The eleven custom elements of design contract 1.11 match section 10.3, name for name.
3. **Open.** Design contract 3.3 says "`LaneView` caches the built `Path<Pixels>` per region and per
   zoom step". ADR 0006 decision 6 and Appendix A give the cache to `RecordView` and `MixView`, and
   reject the per-lane form by name. The report to the operator does not list this wording change.
4. **Open.** Design contract 3.3 says the peak data comes "from `duet-analysis`". Section 1.3 puts
   `Pyramid` in `duet-dsp`, and `duet-core` hands it to the view.
5. **Open.** Design contract 1.2, 1.4, 1.5, and 1.8 specify the top bar, the mode switcher, and the
   transport bar in full. No architecture chunk builds any of the three (P2).
6. **Open.** Design contract 1.6 needs a clamp on restore and a Mix vertical split. `ViewState` holds
   neither (P29).
7. **Open.** ADR 0005 decisions 8 and 9 keep the socket path and the lock in `derived/`. Architecture
   4.1 moves both to `state/` (P19).
8. **Open.** ADR 0004 decision 6a keeps the calibration correlation in `duet-analysis`. Architecture
   1.2 correction 3 moves it to `duet-dsp`, and link "D1 before C4" depends on the move (P19).
9. **Open.** ADR 0001 decision 3a orders `Finite` over the bits. Architecture 2.6a delegates to `f64`.
   The ADR is numerically wrong (P9).
10. **Open.** ADR 0003 decision 13 keeps the save marker in `derived/`, and decision 13a in the same
    file moves it to `state/` (P19).
11. **Closed.** Every MUST story that revision 2 left without a verb now has one. PR 8.2 and
    `SlotKind` agree. PR 11 Q3 and architecture 4.2 agree.

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The engineering is sound and the document is close. Revision 3 closes every OPEN row, twelve of the
sixteen PARTIAL rows, and nineteen of the twenty-four new findings, each with a real mechanism. The
buffer pool, the four-source live set, the `state/` directory, the `checked_sub` split, the
`TakeFlags` set, and the corrected serial links are all correct answers to hard questions. The
Completion column and the seam rules are the right shape.

It fails on two families. The first is the repeat. Each fix placed a new type, a new field, or a new
chunk, and did not apply the document's own placement rule to it. `ViewState` reaches three crates it
cannot see, `Track` has no field for four new verbs, three shell views have no chunk, and the buffer
pool has two writers. Every one of those is the defect class that blocked revision 1 and revision 2,
one layer further in.

The second family is new and is worse, because it is mechanical. Three of the plan's own chunk shapes
are rejected by `scripts/dod.sh`, which runs on every commit. A manifest chunk fails `cargo machete`.
Chunk K1 fails `cargo clippy`. The conversion guard has no registration point and is not in the gate.
A plan built on this text stops at its first manifest chunk.

The single biggest risk is that Appendix E is written against the finding text and not against the
resulting types, for the third revision in a row. The weakest Reactive property is Responsive,
because the deferred verb that walks the git object graph has no thread but the one that draws
frames.

### Findings that block plan authoring

1. P1. `ViewState` and `ProjectView` break the placement rule three times.
2. P2. `TopBar`, `TransportBar`, and `ModeSwitcher` have no file and no chunk.
3. P3. Every manifest chunk fails `cargo machete`.
4. P4. Chunk K1 fails `cargo clippy` and cannot commit.
5. P5. The `BufferPool` free list has two writers.
6. P6. A deferred verb has no thread, and `Gc` runs on the GPUI foreground thread.
7. P7. The plan ignores `crates/duet/src/app.rs` and the two crates that already exist.
8. P8. `Track` has no field for `name`, `input`, `monitor`, or the arm state.

Findings P9 to P26 are Warnings. Every one must close before FINISH, and none is deferrable.
Findings P27 to P30 are Concerns. Close each one in the specification, or file it in a document under
`roadmap/duet-v1/`.
