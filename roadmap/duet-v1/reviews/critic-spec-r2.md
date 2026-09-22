# Specification review, revision 2: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.
This is a re-run. Revision 1 returned NOT READY with twelve blocking findings.

Sources read in full: `roadmap/duet-v1/architecture.md` (3267 lines), the six ADRs,
`product-requirements.md` sections 1 to 11, `design-contract.md` sections 1.11, 3.1 to 3.4, 4.4,
and the orchestrator decisions, `CLAUDE.md`, `Cargo.toml`, `clippy.toml`, `deny.toml`,
`scripts/dod.sh`, `.github/workflows/ci.yml`.

Revision 2 is a large advance. Ten of the twelve blockers are closed by a real mechanism, not by a
restatement. The two that remain open are both in section 13. The new findings below are of one
family: a type or a chunk needs an edge that the crate graph does not carry.

---

## 1. Closure check against the revision 1 review

### 1.1 The twelve findings that blocked plan authoring (Appendix D.1)

| Row | Section cited | State | Reason |
|---|---|---|---|
| 1. The `duet-score` and `duet-command` cycle | 1.3 rule 1, 3.4, 13.1 | CLOSED | `ScoreCommand` moves to `duet-score`, `duet-command` wraps it, and T4 follows T2 and T3. |
| 2. The derives on `Verb`, `TransportCommand`, `VerbOutcome` | 2.5, 9.1, ADR 0001 | OPEN | `Verb` derives `Eq` and carries `Finite`, an `f64` newtype. `f64` has no `Eq`. `Span` still derives no `PartialEq`. See finding N1. |
| 3. The `TakeSegment` linear-scale defect | ADR 0006 decisions 5 and 6 | CLOSED | The segment holds a beat span and an `Arc<SystemXMap>`, and no x range and no frame range. |
| 4. One `rtrb` ring with two consumers | 5.8, 8.3, 5.12 | CLOSED | Two destinations, two mechanisms, two budgets. The monitor path leaves the GPUI thread. |
| 5. The missing audio file dependency | 1.2, 1.3, 5.10, 7.5, B.3 | CLOSED | `duet-media` owns every format, and the four crates enter B.3 with licences. |
| 6. The platform dependency in `duet-midi` | 11.2 | CLOSED | The `[target.'cfg(target_os)']` tables sit in the `duet-midi` manifest, and the rule is stated. |
| 7. `duet gc` against an uncommitted take | 4.4 | PARTIAL | The four-source live set covers the uncommitted take. It omits the undo stack. See finding N4. |
| 8. The missing directory `fsync` | 4.10 | CLOSED | Five steps, every `fsync` named, and a crash matrix with five rows. |
| 9. The unspecified `LatencyReport` source | 5.4, 6.4, ADR 0004 | PARTIAL | The loopback calibration is a real source. The split rule underflows a `u32`, and the crate edge is absent. See findings N2 and N3. |
| 10. The published `GraphChain` with mutable state | 5.5, 5.6, ADR 0004 | CLOSED | `ChainTopology` and `ChainState` split, and `GraphRunner::run` takes `&` and `&mut`. |
| 11. The owner of the root `Cargo.toml` and `Cargo.lock` | 13.0 rule one, 13.3 | OPEN | `M<phase>` owns the root files. No chunk owns a member `Cargo.toml`, and every new member changes `Cargo.lock`. See finding N5. |
| 12. The `element.rs` collision and the module seam | 13.0 rule two, 10.3, 13.2 | PARTIAL | K1 writes `element.rs`. Lines A and B still add top-level modules that their first chunk did not create. See finding N7. |

### 1.2 The PARTIAL closures from the premise review (Appendix D.2)

| Row | Section cited | State | Reason |
|---|---|---|---|
| No contract for plain git | 4.5, the pointer table | CLOSED | Five pointers named, each with a form and a reason. No stored pointer is a bare identifier. |
| The playhead and meters as events | 10.2 | CLOSED | One frame driver per work area, and `MeterLayer` is a leaf that paints every strip meter. |
| cpal has two callbacks | 5.4 | CLOSED | The loopback calibration is the stated source, with a default and a median over five passes. |
| tokio shutdown | 9.5 rules 7 to 9 | CLOSED | One idempotent choke point, four callers, four ordered steps, and a stated best-effort contract. |
| Note placement is not linear in time | ADR 0006 decision 5 | CLOSED | The segment holds no scale. Every x and every frame comes from a map lookup. |
| macOS is case insensitive | 4.3 | CLOSED | The test carries `cfg(target_os = "macos")`, and the Linux substitute is a name-shape unit test. |
| XML gives no stable line order | 3.3, 3.6 | CLOSED | `Pitch` and `SpannerKind` derive `Ord`, and a trailing identifier makes each key total. |
| A tempo change does not move x | ADR 0006 decision 5 | CLOSED | No samples-per-pixel value is cached for a system or for a segment. |
| Zoom invalidates every pixel cache | ADR 0006 decision 6 | PARTIAL | The two caches are separated correctly. The 256-path bound has no relation to the visible set. See finding N8. |
| The complexity ceilings will bite | B.1 | CLOSED | The list is a budget, and each reason names an invariant in the function body. |

### 1.3 The Warning rows (Appendix D.3)

| Row | Section cited | State | Reason |
|---|---|---|---|
| `Pitch` and `SpannerKind` derive no `Ord` | 3.3, 3.6 | CLOSED | Both derive `Ord`, and the sort key is total. |
| `Curve` can hold a wrong-domain point | 7.2 | CLOSED | A point holds a raw `i64`, `Curve::new` is the only constructor, and `thin` needs no map. |
| The undo bound of 200 bounds no memory | 4.9 | CLOSED | Three bounds, whole-transaction drop, and a `UndoHorizonMoved` event to the user. |
| `duet-session` defines `pub struct Send` | 7.1 | CLOSED | It is `AuxSend`. A new name collision appeared elsewhere; see finding N9. |
| Verbs for create, open, status, gc, cancel | 9.1 | PARTIAL | Six groups added. Five MUST stories still have no verb. See finding N6. |
| The cpal shortfall has no report path | 5.4, 12.4 | PARTIAL | `EngineFault::CaptureShortfall` exists. `Take` holds no field for `TakeFlag`. See finding N10. |
| The writer lock removes a live lock | 3.8 | CLOSED | Four steps, a liveness check on the identifier and the start time, and a fixed retry rule. |
| The soak test runs in the pre-commit gate | 14, the test table | PARTIAL | The test is `#[ignore]`. The CI command contradicts the same table, and no chunk edits CI. See finding N12. |
| The ` as ` token test proves nothing | 2.3 | PARTIAL | The new guard is better. It has no chunk, it is absent from `scripts/dod.sh`, and it misses an inner attribute. See finding N11. |
| Two reasons are caller obligations | 2.3, B.1 | CLOSED | `Unit` carries the range, and four suppressions are deleted. |
| The resynchronize path shares the channel | 5.8 | CLOSED | A dedicated `bounded(1)` snapshot channel, a one-second window, and a `ClientDegraded` event. |
| Three statements disagree on the frame driver | 10.2 | CLOSED | One driver per area. Design contract 4.4 already carries the matching wording. |
| No timeout on three paths | 5.12 | PARTIAL | Four timeouts are stated. The calibration and the MIDI port open carry none. See finding N13. |
| The root manifest has no owner | 13.0, 13.3 | OPEN | See row 11 above and finding N5. |
| The module seam covers two lines only | 13.0 rule two | PARTIAL | The rule binds every line. Lines A and B do not implement it. See finding N7. |
| No chunk builds the shell surfaces | 13.2, chunk K6 | PARTIAL | K6 covers six surfaces. The start surface (C-01) and the menu bar (X-07) have no chunk. See finding N6. |
| The ADR rejects the path cache | ADR 0006 decision 6 | CLOSED | The tessellated `Path<Pixels>` cache is adopted, and only the bitmap cache is rejected. |
| `smufl` has no survey row | 1.2, B.3, chunk A1 | PARTIAL | The decision is right in three places. Section 10.5 still names the `smufl` crate. See finding N14. |
| Two peak implementations will exist | 1.2, 5.10 | CLOSED | `duet-dsp` owns the format, the header, and the builder, and both readers depend on it. |
| The shutdown depends on a quit action | 9.5 rules 7 to 9 | CLOSED | Four callers reach one idempotent choke point. |

### 1.4 The Concern rows (Appendix D.4)

| Row | Section cited | State | Reason |
|---|---|---|---|
| `duet-command` is the new hub | 1.3 | CLOSED | The recompilation cost is stated and accepted with a reason. |
| `Bbt` derives `Ord` against the text | 2.8 | CLOSED | The derive is deleted. |
| The `extra` bag absorbs a misspelt field | 3.6 | CLOSED | Every unknown key returns as an `ImportWarning`, and the interface prints it. |
| A non-finite `f64` has no JSON form | 3.6, 7.2 | PARTIAL | `Finite` closes the JSON defect and breaks `Verb: Eq`. See finding N1. |
| `duet-dsp` lists two unused crates | B.3 | CLOSED | `rustfft` and `realfft` move to `duet-analysis`, which pYIN uses. |
| The licence table asks for known entries | B.4 | CLOSED | Only `Unlicense` is new, and the Orchestrator owns the edit. |
| `push_notification` has no citation | 12.4 | CLOSED | Two source paths and the re-export path are named. |
| The case-insensitive test on Linux | 4.3 | CLOSED | The test is macOS only, and the substitute is stated. |
| `duet-project` reaches types by re-export | 1.3 rule 3 | OPEN | Rule 3 takes the edge. Section 4.3 still says the types arrive through `duet-command`. See finding N14. |
| Every crate needs `description` | 1.2, 14 | CLOSED | The metadata rule is in section 1.2 and in rung one. |
| Four stores carry no bound | 4.1 | CLOSED | A bound and an eviction rule for each of the four stores. |
| The undo stack sits in the wrong chapter | 4.9 | CLOSED | `DuetCore` owns it, and the section says so. |

### 1.5 The cross-document conflicts (Appendix D.5)

| Row | State | Reason |
|---|---|---|
| PR 11 Q3 against 4.2 | CLOSED | Q3 now reads "Yes, by reference", and it cites architecture 4.2. |
| Design contract 1.11 names eleven elements | CLOSED | Section 10.3 lists the same eleven, with a file and a chunk for each. |
| Design contract 3.3 needs a path cache | CLOSED | ADR 0006 decision 6 adopts it with a key, a bound, and an invalidation rule. |
| Design contract 4.4 drives meters from the mixer view | CLOSED | Contract 4.4 already names `MeterLayer` as the leaf that owns the frame. |
| Design contract 2.6 needs edit commands | CLOSED | `SetVoice`, `SetPart`, `Move`, `Duplicate`, `Remove`, `Paste`, and `Score::copy` exist. |
| Design contract 2.7 group 6 needs marks | CLOSED | `ScoreMark`, `MarkKind`, `AddMark`, and `RemoveMark` exist. |
| Design contract 3.1 names `SystemXMap` | CLOSED | Both documents name `SystemXMap`, `x_for_beat`, and `beat_for_x`, and the contract returns `Ticks`. |
| Design contract 3.4 needs comp mode | CLOSED | `TakeComp` and `CompRequest` cover a range across takes. |
| Design contract 2.7 binds `1` to `6` | CLOSED | `DurationSelector` holds a sticky value and a held override. |
| PR X-08 layout persistence | PARTIAL | `LayoutStore` holds three widths. X-08 also needs the mode, the zoom, the scroll, the selection, and the window geometry. See finding N6. |
| PR A-04 status in one call | CLOSED | The `Status` verb exists and rung two asserts its take count. |
| 5.12 against 8.3 on the MIDI path | CLOSED | Two destinations and two budgets, 12.0 ms and 28.7 ms. |
| 2.8 against itself on `Bbt` | CLOSED | The derive is deleted. |

**Count: 45 CLOSED, 16 PARTIAL, 3 OPEN.**

---

## 2. New findings in revision 2

### 2.1 Ranked table

| # | Tier | Finding | What must change | Section |
|---|---|---|---|---|
| N1 | Critical | `Verb` derives `Eq`. `Verb::MixSetParam { value: Finite }` carries `Finite`, an `f64` newtype from section 3.6. `f64` implements no `Eq`, so `Finite` cannot derive `Eq` and `Verb` does not compile. `Verb::MixWriteCurve` reaches the same `f64` through `Curve`. Section 2.6 also gives `Span` the derives `Debug, Clone, Copy, Serialize, Deserialize` only, while ADR 0001 decision 3 says `Span` derives `PartialEq` and `Eq`. `Verb::TransportSetLoop { span: Option<Span> }` needs them. The blocking finding of revision 1 is therefore still open, through two new types. | Add `PartialEq` and `Eq` to `Span` in section 2.6. Decide one answer for `Finite`: drop `Eq` from `Verb` and use `PartialEq` alone, or hold a fixed-point newtype in every vocabulary variant. State the choice, because ADR 0005 decision 2a rests on the comparison. | 2.6, 3.6, 7.2, 9.1 |
| N2 | Critical | Five types are used by a crate that section 1.3 gives no edge to. (a) `MidiRecord` lives in `duet-midi`; the audio thread in `duet-engine` reads a ring of it and `ComposeView` in `crates/duet` drains a queue of it, and neither crate depends on `duet-midi`. (b) Section 5.4 puts the calibration cross-correlation in `duet-analysis`, and link "E1 before C4" makes `duet-engine` call it; `duet-engine` does not depend on `duet-analysis`. (c) `Calibration` and `DeviceKey` are defined with the cpal backend in `duet-engine`, and they are stored in `session/session.json` and carried by `Verb::TransportCalibrate`; neither `duet-session` nor `duet-command` depends on `duet-engine`. (d) `Finite` carries `ScoreError::NotFinite`, so it lives in `duet-score`; `duet-session` holds it in `CurvePoint` and depends only on `duet-time`. (e) `LaneView` in `crates/duet` builds a peak path from a source file, and `crates/duet` has no `duet-media` edge. This is the crate-cycle defect class of revision 1, in five new places. | Place each type in the lowest crate that every consumer reaches. Move `MidiRecord`, `Calibration`, `DeviceKey`, and `Finite` into `duet-command` or `duet-time`. Add the `duet-engine` to `duet-analysis` edge, or move the correlation into `duet-dsp`. Then re-derive the topological order and re-check the phase table. | 1.3, 5.4, 5.8, 7.2, 8.3, 13.4 |
| N3 | Critical | Section 5.4 splits the measured round trip by `capture = round_trip - block - reported_output_buffer`. `LatencyReport::capture` is a `u32`. `[profile.release]` sets `overflow-checks = true` with `panic = "abort"`. A measured round trip below `block + reported_output_buffer` therefore ends the process. A low-latency interface or an Aggregate Device can produce that measurement. The product's core promise runs through this line. | Compute the split with `saturating_sub`, or hold the split in `i64` and return `BackendError::Calibration` for a negative capture value. State the floor and add a test with a round trip below one block. | 5.4 |
| N4 | Critical | The `gc` live set has four sources and omits the in-session undo stack. PR R-07 states that undo of a record pass removes the take and keeps the audio file, and PR R-09 and X-13 state that a delete keeps the audio file. After that undo or delete the working `media.manifest` names no such hash. `duet gc` then moves the file to `media/dead/`, and `duet gc --purge <hash>` recomputes the same live set, accepts the hash, and deletes the user's audio. A redo then finds nothing. | Add a fifth source: every `SourceHash` that the undo stack and the redo stack name. The core owns both stacks, so `gc` must ask the core, not only the disk. State that `gc` refuses to run when the core is not reachable. | 4.4, 4.9 |
| N5 | Critical | Seam rule one gives `M<phase>` the root `Cargo.toml` and `Cargo.lock`, and forbids every other chunk to touch them. No chunk owns a member `Cargo.toml`. Fifteen new crates need one, and only G1 lists one. A member manifest that adds a dependency changes `Cargo.lock`, so every crate-creating chunk in a phase rewrites the lock file. `scripts/dod.sh` runs `cargo clippy --locked`, which fails on a stale lock. The rule therefore does not close the collision it was written to close. Seam rule one also says `M<phase>` adds "every `members` entry", while the root manifest already globs `crates/*` and `tools/*`. | State one of two designs. Either `M<phase>` writes every member `Cargo.toml` of the phase plus the lock, and each line chunk writes source only. Or each line chunk owns its own member manifest, and the phase serializes one lock refresh at its end. Delete the `members` sentence. | 13.0, 13.2, 13.3 |
| N6 | Critical | Six MUST stories have no model, no verb, and no chunk. M-04 needs a reverb bus at first draw; `SlotKind` holds no `Reverb` and no chunk builds one. M-06 needs solo and mute; `Strip` holds no mute field and no solo field, and the `Verb` list holds neither. R-02 needs track add, rename, and delete; no verb creates a track. R-04 needs input selection and monitor control; no verb sets either. X-07 needs a macOS menu bar and a Linux window menu; no section and no chunk holds one. C-01 needs the start surface and the recent project list; no entity and no chunk holds it. X-08 also needs the mode, the zoom, the scroll, the selection, and the window geometry, and `LayoutState` holds three widths. ADR 0005 decision 1 says a capability outside the verb list falsifies the claim that the list is the whole interface. | Add the missing model fields, the missing verbs, and the missing chunks, or move each story to LATER with the operator's decision recorded. Name the chunk for each MUST story. | 7.1, 9.1, 10.2, 13.2 |
| N7 | Warning | Two chunk pairs share a write scope inside one phase, and one stated rule is false. Phase 5 runs I1, I2, and I3 together, and section 13.4 states "I1 before I2 and I3". Section 13.3 claims every serial link puts its predecessor in an earlier phase. I1 and I3 both write `crates/duet-core/src/snapshot.rs` and `src/job.rs`. Phase 6 runs J1 and J2 together, both write `crates/duet-agent/src/socket.rs` and `src/path.rs`, and no link orders them. Lines A and B also break seam rule two: A2 and A3 create six new top-level modules, and B3 creates `src/smf.rs`, which their first chunk did not write. | Move I2 and I3 to phase 6, or fold them into I1. Move J2 to phase 7, or fold it into J1. Add `spacing`, `system`, `beam`, `spanner`, `lyric`, and `mark` to A1's write scope, and `smf.rs` to B1's. Correct the claim in 13.3. | 13.2, 13.3, 13.4 |
| N8 | Warning | ADR 0006 decision 6 bounds the path cache at 256 paths with no relation to the visible set. The key is `(SourceHash, RegionId, SystemId, ZoomStep)`, so one region needs one path per system. At the PR 11 Q6 budget of 32 tracks, with four systems on screen and four stacked takes per lane that design contract 3.4 draws, the visible working set is about 512 paths. A least-recently-used cache smaller than its working set evicts and rebuilds every frame, which is the tessellation cost the cache exists to remove. The cache also carries a count bound and no memory bound, which is the defect that revision 1 raised against the undo stack. Appendix A gives the same cache a different key, `(TrackId, SourceHash, ZoomStep)`, and puts it on each `LaneView`, where no global bound can hold. | State the bound against the visible working set at the stated budget, add a byte bound, and give the cache one owner and one key. State what the lane draws on a cache miss. | ADR 0006 decision 6, Appendix A, 3.4 of the design contract |
| N9 | Warning | `MeterPoint` names two different types. `duet-time` section 2.9 defines `pub struct MeterPoint { ticks, clock, bbt, meter }`, a time-signature entry. Section 7.1 defines `pub enum MeterPoint { Input, PreFader, PostFader, Output }`, a metering tap, and `ChainTopology` holds a `meter_point: MeterPoint`. `duet-engine` depends on both crates. This is the `pub struct Send` defect of revision 1 in a new place. Section 3.3 also lists `Meter` and `NoteValue` as `duet-score` value objects while section 2.9 defines both in `duet-time`, and the document never says which crate owns them. | Rename the metering tap to `MeterTap`. State that `Meter`, `NoteValue`, and `Tempo` are defined in `duet-time` and re-exported by `duet-score`, or the reverse. One name, one type. | 2.9, 3.3, 7.1 |
| N10 | Warning | `TakeFlag` is an enum with three variants, and `Take` holds `id`, `name`, and `regions` only. Section 5.4 says a take "is committed with a `TakeFlag::Uncalibrated`", and section 6.4 adds `HadShortfall` and `HadDrift`. One pass can produce all three facts at once, and an enum holds one. The flag has no field to live in. | Add `flags: TakeFlags` to `Take`, as a bitset or a small set, and state which layer writes it. State how the flag reaches `session/takes.jsonl`. | 5.4, 6.1, 6.4 |
| N11 | Warning | `cargo xtask check-conversions` is listed as a rung-one gate. `scripts/dod.sh` does not run it, no chunk writes `tools/xtask/`, and no chunk writes `scripts/dod.sh`. The guard therefore has no implementer and no gate. The guard itself has a hole: it searches for the text `#[expect(clippy::as_conversions`, which does not match the inner attribute form `#![expect(clippy::as_conversions`. A crate-level or module-level suppression passes the guard. The file-count floor of ten is correct and is the one part of the design that resists a vacuous pass. | Name the chunk that writes the xtask subcommand and the chunk that edits `scripts/dod.sh`, and state that the Orchestrator owns the gate edit. Make the guard match both attribute forms. Require a test that plants a suppression outside `convert.rs` and asserts a non-zero exit with a named message. | 2.3, 14 |
| N12 | Warning | The rung-one test table says the ten-million-cycle soak runs in CI through `--run-ignored all`, and the same table says the visual regression suite runs in neither the hook nor CI. `cargo nextest run --run-ignored all` runs every ignored test, so it runs the visual suite too. `render_to_image` exists on macOS only, so the Linux runner fails. The command also sits outside `scripts/dod.sh`, which CLAUDE.md makes the only gate surface, and no chunk edits `.github/workflows/ci.yml`. The claim "the hook stays under 60 seconds" carries no measurement for a seventeen-crate workspace. | Give the soak and the large proptests their own nextest filter or their own feature, never `--run-ignored all`. State who edits the CI workflow, and state that a second CI step is a policy change the Orchestrator owns. Delete the 60-second claim or state how it is measured. | 14, rung one |
| N13 | Warning | The timeout table has four rows and misses two cross-boundary waits that revision 2 introduced. The loopback calibration plays a chirp and records five passes with no stated deadline; a device that returns no input leaves the user on a progress surface forever. `MidiTransport::open_input` and `subscribe_hotplug` carry no timeout, and section 8.2 binds a port automatically on arrival, on the MIDI thread. | Add a calibration deadline, a per-pass deadline, and the action on expiry. Add a MIDI port open timeout and state that a timeout leaves the port unbound and raises `MidiEvent::BindFailed`. | 5.4, 5.12, 8.1, 8.2 |
| N14 | Warning | Three sentences of the first draft survived beside their own correction. Section 10.5 says "`FontMetrics` comes from the `smufl` crate", while section 1.2, Appendix B.3, and chunk A1 say the reader is our own deliverable. Section 4.3 says `duet-project` reaches `SourceHash` "through `duet-command`", while section 1.3 rule 3 takes the direct edge and the graph line carries it. Section 13.2 says the report asks the operator to add the pitch overlay to the product requirements, and PR R-15 already holds it as a SHOULD story. A reader who follows the wrong sentence builds the wrong thing. | Delete the three stale sentences. Cite the section that decided each point. | 4.3, 10.5, 13.2 |
| N15 | Warning | Section 5.5 migration rule 3 says a dropped slot "is cleared in place. Nothing is freed, because `SlotState` holds only inline arrays", and rule 4 says a track "costs the memory of eight idle processors, which is a few kilobytes". `SlotKind::Delay` needs a delay line. Two seconds of stereo delay at 48 kHz is 768 KB as an inline array. `ChainState` holds `MAX_SLOTS` pre-fader and `MAX_SLOTS` post-fader states, so a track costs about 12 MB, and 32 tracks cost about 393 MB. M-04 also needs a reverb, which is larger. The stated number is wrong by three orders of magnitude. If the delay line is a heap buffer instead, then rule 3 frees memory on the audio thread, which ADR 0004 decision 11 forbids. | State the delay line length, compute the real per-track cost, and state the memory budget at 32 tracks. If the cost is too high, hold one shared preallocated delay pool that the topology hands out, and state the hand-out rule. | 5.5, ADR 0004 8c |
| N16 | Warning | `SlotKind` and `SlotState` hold a `PitchCorrect` variant. PR 8.2 lists "pitch correction and time alignment of a vocal take" as explicitly out of v1. No chunk builds it, and `clippy::wildcard_enum_match_arm` is denied, so every match on `SlotKind` must carry a dead arm. | Delete the variant. Add it when the product adds the story. | 5.5, PR 8.2 |
| N17 | Warning | Section 4.1 states that `derived/` "is a cache in full. A user may delete the whole directory, and every file in it rebuilds." `derived/` also holds `save.commit`, `duet.lock`, and `agent.sock.path`. A save marker is the only record of an in-flight multi-file save; it rebuilds from nothing. A user who follows the stated advice between step 3 and step 5 of the save destroys the recovery oracle and leaves a mixture of old and new files. Section 4.10 addresses only the `git clean -xdf` case. | Move the marker, the lock, and the socket path out of `derived/`, into an untracked `state/` directory, and state that `state/` is not a cache. Or narrow the section 4.1 claim to the three cache subdirectories by name. | 4.1, 4.10 |
| N18 | Warning | The compile-time plain-data assertion in section 3.5 names `Command`, `DomainEvent`, and `Snapshot`. The vocabulary type is `Verb` (section 9.1), and section 4.9 uses `Command` again for a different thing. `Verb`, `VerbOutcome`, and `VerbData` carry no assertion. The guard therefore proves nothing about the type that crosses the transport. | Assert over `Verb`, `VerbOutcome`, `VerbData`, `DomainEvent`, and `Snapshot`. Use one name for one type across sections 3.5, 4.9, and 9.1. | 3.5, 4.9, 9.1 |
| N19 | Concern | No chunk in section 13 carries a completion command. The brief for a plan author needs one command per chunk that runs on macOS and on Linux. `cargo test -p <crate>` is derivable for a first chunk, and it is not derivable for A2, A3, C3, F4, I3, or K4, which add a module to a crate an earlier chunk already tested. | Add a completion command column to every chunk table. Use a named test target or a named test filter, so that each command fails before the chunk and passes after it. | 13.2 |
| N20 | Concern | The media line is labelled "Line M, media" and its chunks are N1, N2, and N3. The prefix `M` already names the manifest chunks M0 to M8. A reader who sees "M3" cannot tell a phase manifest from a media chunk. | Rename the line to "Line N, media", or rename the manifest chunks. | 13.2 |
| N21 | Concern | Section 5.4 says the default calibration is a deliberate overestimate so that "a take therefore lands slightly early rather than late, and a singer hears a lead rather than a lag". The commit-time offset does not change what the singer hears during the pass; the monitor path does. The stated reason is therefore wrong on its own terms, and the choice of early over late carries no other argument. | State the real reason, or measure it. Name the maximum error the default can produce at a stated block size, so a reviewer can judge the size of the shift. | 5.4, 6.4 |
| N22 | Concern | `duet-command` names `BundleDocument` in section 1.1 and chunk T4, and the serial link "T4 before F1" makes the save path write through it. The document never defines the trait. A trait is also not plain data, which is the invariant section 1.2 gives the crate. | Define `BundleDocument` with its methods and its error type, or delete it and state how `duet-project` serializes a document it does not define. | 1.1, 1.2, 13.1, 13.4 |
| N23 | Concern | `ScoreCommand::Move { selection, by_ticks: i64, to_staff }` carries a bare `i64` where `Ticks` exists. `CurvePoint { at: i64 }` does the same. Both defeat the rule of ADR 0001 that a time value never loses its meaning. `CurvePoint` trades a typed `Position` for an untyped integer to stop a cross-domain point, when an enum over two typed point lists gives the same guard and keeps the type. | Use `Ticks` in `Move`. Hold the curve points as an enum of `Vec<(Ticks, Finite)>` and `Vec<(SuperClock, Finite)>`, so the domain field cannot disagree with the data. | 3.4, 7.2 |
| N24 | Concern | `AgentBridge::Drop` calls `shut_down`, which moves the `Runtime` onto a detached `std::thread` and sends on a channel. A `Drop` that spawns a thread can fail, and a failure in `Drop` has no return path. A panic there during an unwind aborts the process. | State that the `Drop` path absorbs every failure and logs it, and that it never panics. | 9.5 rule 7 |

### 2.2 Reactive assessment

- **Responsive: PARTIAL.** Nine budgets and four timeouts are real and measurable. The calibration
  and the MIDI port open have no deadline (N13). The path cache is smaller than its working set at
  the stated budget, so a scroll pays the tessellation cost it was meant to avoid (N8).
- **Resilient: PARTIAL.** The five-step save, the crash matrix, the fault queue, the typed errors,
  and the `gc` refusals are correct and complete. Three failures stay uncontained: the `u32`
  underflow in the calibration split (N3), the `gc` live set that omits the undo stack (N4), and the
  save marker that the document tells the user to delete (N17).
- **Elastic: PARTIAL.** Every channel, every ring, every queue, and every store now carries a bound.
  Two bounds are not derived from the load they must hold: the 256-path cache (N8) and the
  `ChainState` slot pool at 32 tracks (N15).
- **Message Driven: PARTIAL.** The vocabulary, the one writer, the versioned state, and the
  resynchronize path are correct. The property drops from PASS because `ComposeView` and the audio
  thread reach into `duet-midi` types across a boundary the crate graph does not carry (N2). That is
  a shared-type reach, not a message.

---

## 3. Plan graph check on section 13

1. **The graph is acyclic.** I checked every link in 13.4 against the phase table. No link runs
   backwards in phase order.
2. **Three links contradict the phase table.** "I1 before I2", "I1 before I3", and the write-scope
   overlap in phase 6 between J1 and J2. Section 13.3 claims no such case exists (N7).
3. **Every serial link carries a reason.** The three added links, C3 before H1, F3 before I1, and G4
   before K2, each carry one. The reason for "B1 and B2 before B3" states honestly that write scope
   alone forces it. That is the correct form.
4. **One link contradicts the crate graph.** "G4 before K2" makes `crates/duet` drain a queue of
   `duet-midi` types. Section 1.3 gives `crates/duet` no `duet-midi` edge (N2).
5. **Write scopes are disjoint in every phase except two.** Phase 5 collides on
   `crates/duet-core/src/snapshot.rs` and `src/job.rs`. Phase 6 collides on
   `crates/duet-agent/src/socket.rs` and `src/path.rs` (N7).
6. **The manifest seam does not hold.** Each phase has exactly one `M<phase>` chunk, and phase 9 has
   none, which is correct. G1 writes `crates/duet-midi/Cargo.toml` in phase 3, beside M3. Every
   other crate-creating chunk writes an unowned member manifest and changes `Cargo.lock` (N5).
7. **No chunk carries a completion command** (N19).
8. **Six MUST stories map to no chunk**: M-04 reverb bus, M-06 solo and mute, R-02 track lifecycle,
   R-04 input and monitor, X-07 menus, C-01 start surface. X-08 maps to K6 in part only (N6).
9. **Three repository files that the plan needs have no owner**: `tools/xtask/` for
   `check-conversions`, `scripts/dod.sh` for the gate line, and
   `crates/duet-engine/src/audio/README.md` for the forbidden-call list of section 5.7.
10. **Chunk E2 now has a consumer.** PR R-15 exists as a SHOULD story, and K3 paints the overlay.
    The conditional sentence in 13.2 is stale (N14).

---

## 4. Consistency across the three documents

1. **Closed.** Design contract 3.1, 3.3, 4.4, and 1.11 all match the architecture after the edits.
   PR 11 Q3 matches architecture 4.2.
2. **Open.** PR M-04 needs a reverb bus and a delay bus at first draw. `SlotKind` holds `Delay` and
   no `Reverb`, and no chunk builds either processor.
3. **Open.** PR M-02 and M-06 need mute and solo on every strip. Architecture 7.1 `Strip` holds
   neither, and the `Verb` list holds neither.
4. **Open.** PR 8.2 rules pitch correction out of v1. `SlotKind::PitchCorrect` contradicts it.
5. **Open.** PR X-08 and PR C-03 need the mode, the zoom, the scroll, and the selection to survive a
   restart. `LayoutState` holds three widths.
6. **Open.** PR X-07 needs a platform menu bar. No architecture section names one.
7. **Open.** PR C-01 needs a start surface with a recent project list. The entity tree in 10.2 holds
   no such view.
8. **Open.** PR MA-03 needs a measure action with progress and cancel. The `Verb` list has no
   loudness measure verb, and `ExportAudio` is the only path to a report.
9. **Internal.** Architecture 10.5 contradicts 1.2 and B.3 on `smufl`. Architecture 4.3 contradicts
   1.3 rule 3 on the `duet-project` edge.

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The engineering is sound and the direction is right. Revision 2 closes ten of the twelve blockers
with a real mechanism, closes every premise PARTIAL, and turns Elastic from a list of unbounded
stores into a table with a bound and an eviction rule for each one. The loopback calibration, the
topology and state split, the five-step save, and the frame-driver decision are all correct answers
to hard questions. The document is close.

It fails on the same class of defect as revision 1, in new places. The specification still does not
compile: `Verb` derives `Eq` over an `f64`, and `Span` lost the derives its own ADR gives it. Five
types sit in crates that their consumers cannot reach, and one of those is a serial link the plan
already states. The plan graph breaks two of its own rules inside two phases, and the manifest seam
rule does not close the collision it was written for. Six MUST stories have no model and no chunk.

The single biggest risk is not any one finding. It is that the closure table in Appendix D was
written against the finding text and not against the resulting types. Every OPEN row above is a case
where a section states the right decision and another section, or a type, or a chunk table,
contradicts it. A plan built on this text would stop in chunk T3.

The weakest Reactive property is Message Driven, which drops from PASS to PARTIAL. Two threads and
one view reach a `duet-midi` type across a boundary that carries no edge, which is shared state, not
a message.

### Findings that block plan authoring

1. N1. `Verb: Eq` over `Finite`, and the missing `Span` derives.
2. N2. Five types used across an edge the crate graph does not carry.
3. N3. The `u32` underflow in the calibration split.
4. N4. The `gc` live set that omits the undo stack.
5. N5. The member manifest and `Cargo.lock` ownership.
6. N6. Six MUST stories with no model, no verb, and no chunk.

Findings N7 to N18 are Warnings. Every one must close before FINISH, and none is deferrable.
Findings N19 to N24 are Concerns. Close each one in the specification, or file it in a document
under `roadmap/duet-v1/`.
