# Specification review: Duet v1 architecture

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before plan authoring.

Sources read in full: `roadmap/duet-v1/architecture.md`, the six ADRs, `product-requirements.md`
sections 8, 10, and 11, `design-contract.md` sections 0 to 4, `research/critic-premise-review.md`,
`research/gpui-kit-audit.md`, `CLAUDE.md`, `Cargo.toml`, `clippy.toml`, `deny.toml`,
`crates/duet/Cargo.toml`, `.cargo/config.toml`.

The specification is a large advance on the hypothesis. The time kernel, the storage decision, the
history model, the backend trait, and the vocabulary crate are correct. The findings below are
places where a closure states a rule but the types, the dependency table, or the plan graph
contradict it. Ten of them stop the work as written.

---

## 1. Closure check against the premise review

### 1.1 Critical rows

| Row | Section cited | State | Reason |
|---|---|---|---|
| Derived `Ord` on a tagged position | 2.5, ADR 0001 | CLOSED | `Position` derives no comparison trait, and `TempoMap::cmp` is the only order. |
| Git repository against content hash | 4.2, 4.3, ADR 0003 | CLOSED | Git holds text and `media.manifest`; audio lives in a content store outside the index. |
| No contract for plain git | 4.6, ADR 0003 | PARTIAL | The `HEAD` watch and the record refusal are mechanisms, but no list names which pointers must be symbolic references. |
| MusicXML cannot hold the model | 3.6, ADR 0002 | CLOSED | The canonical format is the storage format at version one. |
| A MusicXML fork cannot be a dependency | 3.7, ADR 0002 | CLOSED | A narrow in-house reader and writer over `quick-xml`, with a lossless bucket. |
| The channel contract is unstated | 5.8, 9.5 | CLOSED | Bounds, a version counter, and a resynchronize path are all named. |
| The playhead and meters as events | 5.8, 7.3, 10.2 | PARTIAL | Section 10.2 makes `PlayheadLayer` the only view that requests an animation frame, and every meter also samples once per frame. No driver joins the two. |
| `arc-swap` frees on the audio thread | 5.8, ADR 0004 | CLOSED | `triple_buffer` replaces it, and `arc-swap` is a dependency of no crate. |
| cpal has two callbacks | 5.1, 5.4, ADR 0004 | PARTIAL | The one-cycle trait and the one-device policy are right. No rule says how the cpal backend computes `LatencyReport`, which alignment depends on. |
| tokio shutdown | 9.5, ADR 0005 | PARTIAL | The four steps are correct. No rule guarantees that step one runs on every exit path, and no acknowledgement precedes the exit. |
| Note placement is not linear in time | 10.5, ADR 0006 | PARTIAL | `SystemMap` is right. `TakeSegment` carries one x range and one frame range, which restores a single linear scale per segment. |
| The vocabulary has no crate | 1.1, 1.2, 1.3 | CLOSED | `duet-command` exists below the application, the engine, and the transport. |
| `as_conversions` has no sanctioned path | 2.3, B.1 | CLOSED | One conversion module, named sites, round-trip tests. Two sites are avoidable; see finding W10. |

### 1.2 Warning rows

| Row | Section cited | State | Reason |
|---|---|---|---|
| "Audio clock" names no unit | 2.1 | CLOSED | `SuperClock` at 282,240,000 ticks per second, with samples only at the device edge. |
| 1920 ticks and a septuplet | 2.4 | CLOSED | `div_euclid` with remainder absorption, and a test for divisors 2 to 13. |
| gix runs no filters or hooks | 4.7 | CLOSED | A repository-local configuration and `.gitattributes` at creation, plus a cross-check test. |
| gix has no repack | 4.8 | CLOSED | The object count is bounded by design, with stated numbers. |
| macOS is case insensitive | 4.3 | PARTIAL | Lowercase hexadecimal closes the defect. The stated test needs a case-insensitive volume, which Linux CI cannot mount. |
| XML gives no stable line order | 3.6 | PARTIAL | JSON Lines with stable sort keys is the right mechanism. The named key includes `Pitch`, which derives no `Ord`. |
| A command may carry a GPUI type | 3.5 | CLOSED | A compile-time assertion holds the plain-data bound. |
| No latency budget | 5.12 | CLOSED | Eight numbered budgets with a stated measurement for each. |
| The block size may vary | 5.2 | CLOSED | `cycle.frames()` per cycle, and the dummy backend varies the count with a fixed seed. |
| No mechanical no-allocation check | 5.7, B.2 | CLOSED | Three means are named and the specification claims no more. |
| Abort on overflow loses a take | 2.2 | CLOSED | No `Add` and no `Sub` trait on a time newtype, so a bare operator does not compile. |
| A tokio type outside the runtime | 9.5 | CLOSED | Rule 4 makes the boundary a channel. |
| tokio workers compete with audio | 9.5 | CLOSED | `worker_threads(2)`, with the reason recorded. |
| A tempo change does not move x | 10.5, ADR 0006 | PARTIAL | Decision 3 forbids a cached samples-per-pixel value. `TakeSegment` reintroduces one per segment. |
| A take must not split as a region | ADR 0006 | CLOSED | Layout produces derived segments; the region keeps three numbers. |
| Zoom invalidates every pixel cache | ADR 0006 | PARTIAL | The peak pyramid is correct. The ADR also forbids a cached tessellated path, which the audit and the design contract both require. |
| `duet-project` becomes the hub | 1.1, 1.3 | CLOSED | It owns the bundle, the store, the history, and the watch, and defines no document. |
| Session and engine may cycle | 1.3 | CLOSED | The engine depends on the session, and the session names no engine type. |
| Mix and master hold no invariant | 1.1, 1.2 | CLOSED | The mixer folds into the engine; `duet-export` owns the offline graph. |
| The paint element must compute nothing | 10.5 | CLOSED | The input is an absolute placement list, and `paint` adds only the bounds origin. |
| Six untestable interface behaviours | 10.7 | CLOSED | Rung three names all six as required tests. |
| A golden-image suite is macOS only | 10.7 | CLOSED | The visual suite is a developer tool and never part of `scripts/dod.sh`. |
| `integer_division` is denied | 2.2, 12.3 | CLOSED | `div_euclid`, `div_ceil`, `checked_div`, and `NonZero` divisors. |
| The complexity ceilings will bite | B.1 | PARTIAL | The list predicts twelve sites. It reads as a pre-approval, two reasons are caller obligations, and two sites are avoidable. |

---

## 2. New findings

### 2.1 Ranked table

| Tier | Finding | What must change | Section |
|---|---|---|---|
| Critical | `duet-score` and `duet-command` form a dependency cycle. Section 3.4 puts `ScoreCommand`, `ScoreEvent`, and `Applied` in `duet-command`, and gives `Score::apply(&mut self, command: ScoreCommand)` on a `duet-score` type. Section 1.3 also makes `duet-command` depend on `duet-score`. Cargo refuses the cycle, and the trunk order T2 before T4 becomes impossible. | Put `ScoreCommand`, `ScoreEvent`, and `Applied` in `duet-score`. Make `duet-command` re-export them. The vocabulary crate then composes documents and defines none. | 1.3, 3.4, 13.1 |
| Critical | `Verb`, `TransportCommand`, and `VerbOutcome` derive `PartialEq`, and `TransportCommand` also derives `Eq` and `Copy`. Their fields hold `Position`, `Span`, and `Delta`, which implement no `PartialEq` by design. The specification does not compile. | Delete the derives, or hold single-domain values (`Ticks`, `SuperClock`) in every variant the vocabulary carries. State which choice applies, because the agent transport needs a comparison for idempotence. | 2.5, 5.9, 9.1 |
| Critical | `TakeSegment` holds `ticks: (Ticks, Ticks)`, `x: (f32, f32)`, and `source_frames: (u64, u64)`. Two endpoints encode one linear map from x to frames inside the segment. That is the per-system samples-per-pixel defect at a finer grain, and ADR 0006 rejects it by name. | State the segment rule: a segment breaks at every `SystemMap` knot and at every tempo point inside its span. Then the per-segment linear map is exact. Add a test for a tempo change inside one system. | ADR 0006 decisions 3 and 5 |
| Critical | Section 8.3 pushes MIDI records into one `rtrb` ring and states "Two consumers read it, one per mode". `rtrb` is strictly single producer and single consumer, so two consumers cannot exist. In Compose the path also runs MIDI to sound through the `ComposeView` frame drain, which adds up to one frame. Section 5.12 claims the path never crosses the GPUI thread. | Use one ring to the audio thread for the monitor voice in every mode. Use a second ring, or a bounded `ArrayQueue`, for note entry into the view. State which ring the 12.0 ms budget covers. | 5.12, 8.3 |
| Critical | `duet-engine` reads `media/<hh>/<hash>.wav` and writes `media/incoming/<uuid>.wav`, and `duet-analysis` reads the same files for peaks and pYIN. Neither crate lists an audio file dependency. `bwavfile` is a dependency of `duet-export` only. | Add the file reader and writer to the crate table and to Appendix B.3, with a licence row. Name which crate owns WAV and RF64 input and output, and state the RF64 promotion rule for a long take. | 1.2, 5.10, B.3 |
| Critical | The crate table lists `coremidi` and `alsa` as plain dependencies of `duet-midi`. `alsa` does not build on macOS and `coremidi` does not build on Linux, so the workspace builds on neither platform. Section 11.2 rule 4 states the target rule for a consumer manifest only. | State that `coremidi` and `alsa` sit under `[target.'cfg(target_os = "...")'.dependencies]` inside the `duet-midi` manifest. Correct the crate table so it is not read as a plain dependency list. | 1.2, 11.2 |
| Critical | Section 4.4 defines a live object as one that a reachable commit names. A take recorded and not yet committed appears in the working `media.manifest` only. `duet gc` therefore moves every uncommitted take to `media/dead/`, and `duet gc --purge` then deletes it. That is a documented path to data loss. | Add the working-tree manifest and `media/incoming/` to the live set. State that `gc` refuses to run while a record pass runs, and that `--purge` refuses an object younger than a stated age. | 4.4 |
| Critical | The two-phase save writes and `fsync`s the temporary files and the marker, renames each file, then deletes the marker. It never `fsync`s a directory after the renames. A crash after the marker delete can leave the old files, because the rename metadata is not durable. The single-file rule fsyncs its directory; the multi-file rule drops that step. | Insert step 3.5: `fsync` every directory that a rename touched, then delete the marker. Also state that the marker lives in `derived/`, which `.gitignore` covers and `git clean -xdf` deletes. | 4.10 |
| Critical | The cpal backend has no stated source for `LatencyReport`. cpal exposes no latency query, only callback timestamps. Section 6.4 computes the `ExistingMaterial` offset from `capture + playback + monitor`, so the product's core promise rests on a number the backend cannot supply. The input ring adds a second unmeasured offset. | State how the cpal backend derives capture latency, playback latency, and the ring-induced offset. Add a loopback alignment test over the dummy backend with a stated maximum error in frames. | 5.4, 6.4, ADR 0004 |
| Critical | Section 5.6 publishes `GraphChain` through `triple_buffer`, and `Output::read` gives `&GraphChain`. Section 5.5 puts the processors inside the chain (`fader: Gain`, `pre_fader: UserSlots`), and a processor needs `&mut self` to run. A shared reference cannot run them. Where the mutable DSP state lives is unspecified, and a published chain would also hold three copies of it. | Separate the immutable topology from the mutable processor state. Publish the topology, keep the state on the audio thread, and state the rule that migrates state across a topology change. | 5.5, 5.6, 5.8 |
| Warning | `Pitch` and `SpannerKind` derive no `Ord`. Section 3.6 sorts `notes.jsonl` by "staff, voice, onset, pitch" and `spanners.jsonl` by "start note, kind". The deterministic-output property therefore cannot be computed as written. | Derive `Ord` on `Pitch` and `SpannerKind`, or state the exact key function that the writer uses. Add a test that two writes of one score give equal bytes. | 3.3, 3.6 |
| Warning | `Curve` holds a `domain` field and `Vec<CurvePoint>` where `CurvePoint.at` is a `Position`. Nothing stops a beats curve from holding an audio point. `Curve::thin` takes no `&TempoMap`, so it cannot order its own points. | Hold single-domain values in the point list, or make `thin` take the map. State the invariant that every point matches the curve domain, and enforce it in the constructor. | 7.2 |
| Warning | The undo bound of 200 transactions bounds no memory. The inverse of `RemoveMeasures { from, count }` must hold one `InsertNote` per removed note, so one transaction can hold tens of thousands of commands. | State a second bound on total command count or bytes, and state what the application does when it drops an entry. | 4.9 |
| Warning | `duet-session` defines `pub struct Send`. A struct and a trait share the type namespace, so any module that imports it cannot write a `T: Send` bound. | Rename it. `AuxSend` or `SendSlot` states the same thing and breaks nothing. | 7.1 |
| Warning | The `Verb` list has no verb to create a project, open a project, report status, run `gc`, or cancel a job. `VerbOutcome::Started { job }` returns a `JobId` that no verb accepts. PR A-04 makes "Status in one call" a MUST, and rung two creates a bundle from the SATB template. | Add `ProjectCreate`, `ProjectOpen`, `Status`, `Gc`, `JobStatus`, and `JobCancel`. The claim that one flat verb list is the whole interface fails while `duet gc` sits outside it. | 9.1, 9.3, 14 |
| Warning | The cpal input shortfall is "zero filled and counted". No `EngineFault` variant carries that count, so silence enters a vocal take and nothing tells the user. `CaptureOverflow` names the opposite condition. | Add `EngineFault::CaptureShortfall { track, frames }`, route it through the fault queue of section 12.4, and state the user message. | 5.4, 12.4 |
| Warning | The writer lock rule removes a lock whose socket does not answer. A second writer that starts before the first writer binds its socket will take the lock from a live process. No retry bound and no liveness check on the recorded process identifier exist. | Check that the recorded process identifier is alive before removal. Write the lock only after the socket binds, or write the socket path in a second step. State the retry count and the wait. | 3.8, 9.3 |
| Warning | Rung one runs `cargo nextest run`, which includes the ten-million-cycle soak test of rung two. The gate runs on every commit, on macOS and on Linux, through `.githooks/pre-commit`. | Mark the soak test `#[ignore]`, or move it behind a feature, and state its wall-clock cost. State which rung-two commands the pre-commit gate runs and which run in CI only. | 14 |
| Warning | The guard "no source file outside `convert.rs` contains the token ` as `" reads the sources of one crate, so it proves nothing about the workspace. It also fails on the English word "as" in a doc comment, and it passes when its file list is empty. `clippy::as_conversions` already denies the cast everywhere. | Delete the token test. Replace it with a check that no `#[expect(clippy::as_conversions` attribute exists outside `duet-time/src/convert.rs`, over the whole workspace, that fails when it scans zero files. | 2.3, 14 |
| Warning | Two of the twelve predicted reasons state a caller obligation, not an invariant: `f32_to_i24` and `f32_to_i32` say "the caller clamps to -1.0 to 1.0 before the call". The function takes a raw `f32`, so no reader can check the claim at the site. `len_to_f64` is avoidable through `u32::try_from` and `f64::from`. The two `NonZeroI64` constants are avoidable through a `match` with a non-panic fallback arm. | Take a clamped newtype in the two float-to-integer functions. Delete the two avoidable suppressions. State that the list is a budget that an implementer must try to avoid, not an approval. | 2.1, 2.3, B.1 |
| Warning | The event channel holds 1024 and uses `try_send`. On full, the core marks the client stale and the client asks for a full `Snapshot`. A snapshot of a 300-bar, 8-part score travels the same channel. A slow client therefore falls behind again at once. | State a rate limit or a hysteresis on the resynchronize path, and state the snapshot transfer mechanism. A snapshot that shares the event channel has no bound of its own. | 5.8 |
| Warning | Section 10.2 states that `PlayheadLayer` is the only view that requests an animation frame. Appendix A states that each meter view samples the snapshot once per frame. Design contract 4.4 drives every meter from one mixer-view animation frame, which marks the whole mixer dirty. The three statements do not agree. | Name one frame driver for the meters. If the mixer view drives it, state the cost, because the audit says `request_animation_frame` marks the whole view dirty. | 10.2, Appendix A |
| Warning | No timeout covers the command-line client that connects to the socket, the device open path, or a gix read. A hung window process makes every agent verb hang with no bound. The premise review named these three paths. | State a connect timeout and a request timeout for the socket client, and a timeout for the device open. State the action after each timeout. | 3.8, 5.12, 9.2 |
| Warning | The root `Cargo.toml` and `Cargo.lock` are called a serial seam, and no chunk owns them. Phase 1 runs T2, T3, and D1, and all three add `[workspace.dependencies]` entries. Phase 4 and phase 5 each run ten chunks, of which at least six add a dependency. Every one rewrites `Cargo.lock`. | Add one manifest chunk at the start of each phase that owns the root manifest and the lock file. Name it in the phase table. State that no other chunk writes either file. | 13.1, 13.3 |
| Warning | `clippy::mod_module_files` is denied, so a module directory needs a sibling file. Every directory chunk therefore also writes a file in the crate source root, and every line shares the crate root `mod` seam. Only lines G and K state the rule. `crates/duet/src/element.rs` receives a `mod` line from K2, K3, and K4, which run together in phase 6. | State the seam rule for every line, as line G states it. Make K1 write `crates/duet/src/element.rs` with every `mod` line. Add the sibling files to each chunk's write scope. | 13.2, 13.4 |
| Warning | No chunk builds the sidebar tree, the inspector, the status bar, the title bar, the history sheet, the project templates, or the audio settings surface. The design contract makes each one part of the shell, and PR X-08, C-01, C-02, and H-02 are MUST stories. | Add the missing chunks to line K, or add a second application line. State which chunk writes the project templates of PR Q5. | 13.2 |
| Warning | ADR 0006 consequence 3 states that a path is built per segment at draw time rather than blitted from a cached image. The audit says `PathBuilder::build()` tessellates on the central processor on each call, and advises a cached `Path<Pixels>`. Design contract 3.3 requires the cache. The ADR rejects a pixel cache and then rejects the path cache with it. | Separate the two caches. Keep the peak pyramid, reject the pixel cache, and adopt a tessellated `Path<Pixels>` cache per region and per zoom step. State the invalidation rule. | ADR 0006 decision 6, audit gap 4 |
| Warning | The crate table gives `duet-engrave` a `smufl` dependency. Appendix B.3 does not list it, so it gets no survey row and no licence check. Design contract 2.2 states that the metadata reader is our own deliverable in `duet-engrave`. | Decide one answer. If `smufl` enters, add a B.3 row. If we write the reader, delete the dependency and add the work to chunk A1. | 1.2, B.3 |
| Warning | Section 5.10 computes peak bins on the disk thread inside `duet-engine`. Chunk E1 puts the peak pyramid in `duet-analysis`. `duet-engine` depends on no analysis crate, so two implementations of one format will exist. | Move the pyramid format and the builder to `duet-dsp`, which both crates already depend on, or add the edge. State which crate owns the `.peaks` file format and its header. | 1.3, 5.10, 13.2 |
| Warning | The tokio shutdown depends on a quit action. A platform quit, a window close, or a signal skips it, and `Runtime::drop` then blocks the main thread. Step 2 sends `CoreInput::Shutdown` and waits for nothing before the process exits, so an in-flight verb reply can be lost. | Name the one choke point that runs step 1 on every exit path. State whether an in-flight verb reply is guaranteed, and say so in the agent contract if it is not. | 9.5 rule 7 |
| Concern | `duet-command` is the new hub. It depends on `duet-score` and `duet-session`, and nine crates depend on it. A note field change recompiles the engine, the MIDI crate, and the project crate. This is the defect that finding 8.2 named for `duet-project`. | State the cost, or split the vocabulary by context and keep one facade crate. The ADR names the cost; the specification should name it in section 1.3 as well. | 1.3, ADR 0005 |
| Concern | `Bbt` derives `Ord`, and section 2.8 states that it is never a sort key because it is not monotonic across a meter change. The order is in fact monotone in time, so one of the two statements is wrong. The same document argues against a derive that must not be used. | Correct the text, or delete the derive. | 2.8 |
| Concern | `#[serde(flatten)] extra` absorbs a misspelt field from a hand edit and writes it back forever. The agent path in section 3.8 has no validation report, so a wrong edit looks like a correct one. | State that the reader reports unknown keys as an `ImportWarning`, and that the command-line interface prints them. | 3.6, 3.8 |
| Concern | A non-finite `f64` has no JSON form. `serde_json` writes `null`, which fails the round trip. `Curve.value` and the gain fields are `f64`. | Reject a non-finite value at the edge with a typed error, and add a proptest for the round trip. | 3.6, 7.2 |
| Concern | `duet-dsp` lists `rustfft` and `realfft`, and no signal block in chunks D1 to D3 needs a transform. `cargo machete` fails on an unused dependency. | Delete both, or name the block that uses them. | 1.2, 13.2 |
| Concern | Appendix B.4 asks for licence entries that `deny.toml` already allows: MPL-2.0, CC0-1.0, Apache-2.0, and OFL-1.1. Only `Unlicense` is missing. `symphonia` carries a row and appears in no crate's dependency list. | Correct the table against the current `deny.toml`. Delete the `symphonia` row, or add the crate to the dependency table. | B.4 |
| Concern | Section 12.4 calls `window.push_notification`. The audit lists no notification interface, and every other interface in the specification carries an audit citation. | Cite the source path, or name the component the application builds. | 12.4 |
| Concern | Section 4.3 states that a test runs on a case-insensitive volume. macOS creates one with `hdiutil`. Linux needs a loopback image and privileges that CI does not hold. | Mark the test macOS only, and state the Linux substitute. | 4.3, 11.3 |
| Concern | `duet-project` holds `SourceHash` and `AudioContainer` in `ManifestEntry` and depends on `duet-command`, not on `duet-session`. A re-export carries the types. The graph hides a real edge. | State the re-export rule in section 1.3, or take the edge. It stays acyclic either way. | 1.3, 4.3 |
| Concern | Every new crate needs `description` in `[package]` for `clippy::cargo_common_metadata`, which the denied `cargo` group contains. Section 1.2 names only `[lints] workspace = true`. | Add the metadata rule to section 1.2. | 1.2 |
| Concern | `derived/peaks/`, `derived/analysis/`, `media/dead/`, and `media/incoming/` have no size bound and no eviction rule. `media/dead/` never empties without a user command. | State a bound or a policy for each one. Elastic needs a bound on every store, not only on the channels. | 4.1, 4.4 |
| Concern | Section 4.9 puts the undo stack in the `duet-project` chapter. Appendix A and chunk I1 put it in `duet-core`. | Move section 4.9 to the core chapter, or state that the section compares two mechanisms and owns neither. | 4.9, Appendix A |

### 2.2 Reactive assessment

- **Responsive: PARTIAL.** The eight budgets of section 5.12 are real and measurable. Three paths
  still have no timeout: the socket client, the device open, and a gix read. The MIDI-to-sound claim
  is false in Compose mode, where the path crosses the GPUI frame drain.
- **Resilient: PARTIAL.** Typed errors, the fault queue, the fault as view state, and the two-phase
  save are correct. Three failures stay uncontained: the missing directory `fsync`, the garbage
  collector that reaps an uncommitted take, and the capture shortfall that has no report path.
- **Elastic: PARTIAL.** This is the largest advance. Every channel, ring, and queue carries a bound
  and a named backpressure signal. Four stores carry none: the peak cache, the analysis cache,
  `media/dead/`, and `media/incoming/`. The undo bound of 200 bounds no memory.
- **Message Driven: PASS.** One vocabulary, one writer, bounded channels, versioned state, a
  resynchronize path, and latest-value snapshots for high-rate data. The remaining defects are type
  defects, not architecture defects.

---

## 3. Plan graph check on section 13

1. **The trunk is thin and correct.** Four chunks, four pure crates, no input or output, no thread.
2. **The graph is acyclic.** I checked all 40 serial links against the phase table. Every link puts
   its predecessor in an earlier phase. No link runs backwards.
3. **Every serial link carries a reason.** The reason for "B1 and B2 before B3" states honestly that
   write scope alone forces it. That is the correct form.
4. **The lines are write-scope-disjoint at the crate level, and not at the crate root.** Every
   directory chunk also needs a sibling module file, because `clippy::mod_module_files` is denied.
   Every line shares one crate root `mod` seam. Only lines G and K state the rule.
5. **`crates/duet/src/element.rs` collides.** K2, K3, and K4 run together in phase 6, and all three
   add a `mod` line to it. K1 must write the file.
6. **The root `Cargo.toml` and `Cargo.lock` collide with no rule.** Phase 1 runs T2, T3, and D1, and
   each one adds workspace dependencies. Phase 4 and phase 5 each run ten chunks; C2, C3, F2, F3,
   G2, and G3 all add a dependency, so all six rewrite `Cargo.lock`. The stated rule, "one chunk at
   a time touches them", contradicts the phase table. Name an owner chunk per phase.
7. **Three serial links are missing.**
   - C3 before H1. H1 renders the mix graph that C3 builds, and both sit in phase 4.
   - F3 before I1. The gateway dispatches `HistoryCommit` and `HistoryCheckout`, and both sit in
     phase 4.
   - G4 before K2. `ComposeView` drains the note-entry ring that G4 creates.
8. **Verb dispatch cannot complete in phase 4.** I1 must dispatch `ExportAudio`, and H3 and H4 land
   in phase 6 and phase 7. State that I1 delivers dispatch in stages, or move the export arm.
9. **Chunk E2 has no consumer.** pYIN writes `derived/analysis/<hash>.pitch`, and PR 8.2 rules out
   pitch correction and audio-to-notation. No MUST story reads the file. Cut E2, or name the story.
10. **No chunk covers the shell surfaces.** See the finding above on the sidebar, the inspector, the
    status bar, the title bar, the history sheet, and the project templates.

---

## 4. Consistency across the three documents

1. **PR 11 Q3 against architecture 4.2.** The decision reads "Yes. The history holds the audio". The
   architecture keeps audio outside the git index and states that a clone carries no audio. The
   intent matches, the words do not. Amend PR 11 Q3, or the two documents disagree on a MUST.
2. **Design contract 1.11 against architecture 10.3.** The contract names eleven custom elements.
   The architecture names six. `LufsMeter`, `AutomationLane`, `PunchRange`, and `Toolbar` appear in
   no architecture table and in no chunk. `LufsMeter` serves MA-03, which is a MUST.
3. **Design contract 3.3 against ADR 0006 decision 6.** The contract requires a cached
   `Path<Pixels>` per region and per zoom step. The ADR builds the path at draw time.
4. **Design contract 4.4 against architecture 10.2.** The contract drives every meter from one
   mixer-view animation frame. The architecture gives `PlayheadLayer` the only animation frame and
   gives each meter its own child view.
5. **Design contract 2.6 against the `ScoreCommand` list.** The context menu offers `Voice`, `Part`,
   `Cut`, `Copy`, `Paste`, and `Duplicate`. No command moves a note between voices, staves, or
   parts, and none moves a note in time. PR C-16 makes a clef change a MUST, and only `AddStaff`
   sets a clef. Design contract 2.7 group 6 offers `System break`, `Repeat`, and `Rehearsal mark`,
   which the score model does not hold.
6. **Design contract 3.1 against ADR 0006.** The contract names `SystemXMap` with
   `x_for_beat(beat) -> f32` and `beat_for_x(x) -> f32`. The ADR names `SystemMap` with
   `x_of(at: Ticks)` and `ticks_at(x: f32)`. One name and one signature must win.
7. **Design contract 3.4 against the `Verb` list.** Comp mode selects a range across takes. No verb
   creates a composite from a range, and `TakeSplit` plus `TakeSelect` do not cover it. R-09 is a
   MUST.
8. **Design contract 2.7 against architecture 10.6.** The contract binds `1` to `6` to the duration
   group as well as the six held letters. The architecture models the held letters only.
9. **PR X-08 against architecture 10.2.** The story makes layout persistence a MUST, and the design
   contract persists the split per project and per mode. The entity tree holds no such state and no
   chunk writes it.
10. **PR A-04 against the `Verb` list.** "Status in one call" is a MUST, and no `Status` verb exists.
11. **Architecture 5.12 against architecture 8.3.** Section 5.12 states that the MIDI-to-sound path
    never crosses the GPUI thread. Section 8.3 routes the Compose monitor voice through the
    `ComposeView` frame drain.
12. **Architecture 2.8 against itself.** `Bbt` derives `Ord` and the text forbids its use as a sort
    key.

---

## Verdict

**NOT READY FOR PLAN AUTHORING.**

The design is sound in its structure. The premise review's thirteen Critical rows are closed or
close to closed, the bounded contexts are right, and Elastic moved from FAIL to PARTIAL, which is
the largest single gain. The specification fails on its own types, not on its reasoning: three
enums do not compile, one crate pair forms a cycle, two crates cannot build on either platform, and
two crates read files they carry no dependency to read. A plan built on this text would stop in
chunk T2.

The single biggest risk is the cpal alignment reference. Every other Critical has a mechanical fix
of known size. `LatencyReport` has no source in cpal, and the product's core promise, a vocal take
that lines up with its notation, rests on it. The weakest Reactive property is Responsive, because
three cross-boundary paths still have no timeout and one stated budget is false.

### Findings that block plan authoring

1. The `duet-score` and `duet-command` cycle.
2. The derives on `Verb`, `TransportCommand`, and `VerbOutcome` over `Position`, `Span`, `Delta`.
3. The `TakeSegment` linear-scale defect.
4. One `rtrb` ring with two consumers, and the MIDI-to-sound path through the frame drain.
5. The missing audio file dependency in `duet-engine` and `duet-analysis`.
6. The platform dependency declaration in `duet-midi`.
7. `duet gc` against an uncommitted take.
8. The missing directory `fsync` in the two-phase save.
9. The unspecified `LatencyReport` source for cpal.
10. The published `GraphChain` that holds mutable processor state.
11. The owner of the root `Cargo.toml` and `Cargo.lock` per phase.
12. The `crates/duet/src/element.rs` collision, and the crate root `mod` seam rule for every line.

Items 1 to 10 are corrections to the specification. Items 11 and 12 are corrections to section 13.
Every Warning above must close before FINISH; none of them is deferrable.
