# Premise review: Duet v1 working hypothesis

Reviewer: Engineering Critic. Date: 2026-09-20. Mode: specification review, before the Software Architect writes the specification.

Sources read: the shared brief, `roadmap/duet-v1/research/gpui-kit-audit.md`, `roadmap/duet-v1/research/ardour-concepts.md`, `roadmap/duet-v1/research/crate-survey.md`, `CLAUDE.md`, `Cargo.toml`, `clippy.toml`, `deny.toml`. Skills used: `rust-expertise`, `gpui-kit`, `simplified-technical-english`.

The hypothesis is strong. The bounded contexts are right, the Ardour concept map is the correct set to adopt, and the headless core is the correct seam. The findings below are the places where the hypothesis is silent, self-contradictory, or in conflict with the lint policy. Silence at this stage becomes a defect at scale.

---

## Claim 1: one position type with a domain tag, integer beats at 1920 ticks

The kernel is correct in shape. Four gaps must close in the specification.

### CRITICAL 1.1: a derived `Ord` on a tagged position gives a wrong order

**OBSERVATION.** The hypothesis states "position with a domain tag beats|audio clock". A Rust enum that carries two domains gets `Ord` by derive in almost every first draft.

**CLAIM.** A derived `Ord` compares the discriminant first. Every beat position then sorts before every audio position, whatever the real time.

**ARGUMENT.** A cross-domain compare is a tempo-map query, not a field compare. The Ardour map records this: `timepos_t` compares in one instruction only when the domains match, and calls the map when they differ. A derive hides the map call and returns a plausible, wrong answer. The compiler cannot catch it, and no test catches it until two domains meet in one sorted list.

**EVIDENCE.** `/Users/james/Developer/ardour/libs/temporal/temporal/timeline.h`, quoted in `ardour-concepts.md` section 1: "compares two values in one instruction when the domains match, and falls back to a map call only when they differ."

**WHAT MUST CHANGE.** `Position` implements neither `Ord`, `PartialOrd`, `Eq`, nor `Hash`. Order comes from one function that takes the tempo map: `TempoMap::cmp(a, b)`. Each single-domain newtype (`Ticks`, `Superclock`) keeps its own derives. Record this as a type-level invariant, not as a convention.

### WARNING 1.2: "audio clock" is not a unit

**OBSERVATION.** The brief says "beats or audio clock". It does not name the audio unit or its rate.

**CLAIM.** If the audio unit is the sample, then a change of sample rate moves every existing take on the timeline.

**ARGUMENT.** A sample count means nothing without a rate. A project opened at 48 kHz after a 44.1 kHz record pass shifts by 8.8 percent. Ardour solved this with the superclock, a rate that divides every common sample rate exactly.

**EVIDENCE.** `ardour-concepts.md` section 1, `superclock.h`: "so that a sample-rate change does not move existing material". 282240000 divides 44100, 48000, 88200, 96000, 176400 and 192000 with no remainder.

**WHAT MUST CHANGE.** Name the unit and the rate in the specification. Use a superclock newtype over `i64`. Convert to samples at the device edge only, with integer multiply and divide, never with a float.

### WARNING 1.3: 1920 ticks is not exact for every tuplet

**OBSERVATION.** The hypothesis states that integer beats at 1920 ticks are "exact enough".

**CLAIM.** 1920 equals 2^7 times 3 times 5. A division into 7, 11 or 13 parts is not exact. One beat in 7 parts gives 274.28 ticks.

**ARGUMENT.** A septuplet is common in vocal music. Without a stated rule, each part rounds down and the tuplet ends 2 ticks short of the next beat. The error accumulates over a bar and shows as a visible and audible drift.

**EVIDENCE.** `ardour-concepts.md` section 1 and "Concepts to adopt" item 4.

**WHAT MUST CHANGE.** Accept 1920, and state the rule. A tuplet divides the span with `div_euclid`, and the last part absorbs the remainder. The invariant is that the parts sum to the span exactly. Test it for divisors 2 to 13.

### CONCERN 1.4: do not pack the tag into the value

**OBSERVATION.** Ardour packs the domain flag into two spare bits of a 64-bit word.

**CLAIM.** The packed form is a C++ space optimisation. In this repository it costs more than it saves.

**ARGUMENT.** A packed form needs masks and `as` casts. `clippy::as_conversions` is denied. Each mask then needs an `#[expect]` with an invariant. A plain enum of two newtypes is 16 bytes, safe, and exhaustive in a `match`. A position is not a per-sample value, so 16 bytes costs nothing measurable.

**EVIDENCE.** `Cargo.toml`, `[workspace.lints.clippy] as_conversions = "deny"`.

**WHAT MUST CHANGE.** Use the enum. Measure before any change to a packed form.

---

## Claim 2: a real git repository inside the bundle, driven by gix, audio by content hash

This is the weakest part of the hypothesis. The model holds a contradiction that the specification must resolve.

### CRITICAL 2.1: "a real git repository" and "audio by content hash" are two different designs

**OBSERVATION.** The hypothesis states both at once. It does not say whether the audio bytes are git objects.

**CLAIM.** Both answers break something, so the design is not yet decided.

**ARGUMENT.** Take the first answer: the audio bytes are git blobs. One five-minute stereo take at 48 kHz and 24 bits is about 86 MB. Audio does not compress well with zlib. Fifty takes give several gigabytes, and every checkout of an old commit rewrites gigabytes in the worktree. Take the second answer: the audio lives in a content-addressed store outside git, and git holds only the manifest. Then a clone of the repository carries no audio, `git clean -xdf` deletes the takes, and the store needs its own garbage collector by reachability from every commit. The two answers produce two different bundle layouts, two different backup stories, and two different failure modes. The hypothesis picks neither.

**EVIDENCE.** `crate-survey.md`, gix row. `ardour-concepts.md` "Concepts to adopt" item 14: media beside a plain-text state file, derived data separate and deletable.

**WHAT MUST CHANGE.** Choose the second answer and write it down. Git tracks text only: the score, the session state, and the manifest that names each audio object by hash. The audio objects live under `media/` inside the bundle and outside the git index. State the garbage-collection rule: an object is live when any reachable commit names it. State what a clone gives, and state that the bundle, not the repository, is the unit of backup.

### CRITICAL 2.2: no contract for the user who runs plain git

**OBSERVATION.** The brief asks what happens when a user runs plain `git` inside the bundle. The hypothesis does not answer.

**CLAIM.** Without a contract, `git checkout`, `git rebase` and `git commit --amend` each corrupt the running application state.

**ARGUMENT.** The application holds the project in memory. A checkout replaces the files under it. A `notify` event then arrives in the middle of a record pass. An amend or a rebase rewrites commit identifiers that the project state stored as pointers, so history entries name objects that no longer exist. This is a data-loss path, not an inconvenience.

**EVIDENCE.** `crate-survey.md` lists `notify` and `notify-debouncer-full` for file watch. `ardour-concepts.md` section 8 shows Ardour's answer for a different problem: an atomic save plus a `.pending` file.

**WHAT MUST CHANGE.** Define the contract in three rules. First, the application watches `HEAD` and the worktree. Second, on an external `HEAD` change the application stops the transport, discards no unsaved work, and asks the user to reload or to save to a new branch. Third, the application stores a symbolic reference, not a bare commit identifier, wherever a pointer must survive a rewrite. State plainly that a rewrite of history is unsupported while the project is open.

### WARNING 2.3: gix does not run filters, hooks, or LFS

**OBSERVATION.** `gix` is a pure-Rust implementation. It does not run clean and smudge filters, it does not run hooks, and it has no Large File Storage support.

**CLAIM.** A user's global git configuration changes what plain `git` writes, and gix then reads a file that differs from the one it wrote.

**ARGUMENT.** A global `core.autocrlf`, a global attributes file, or a global LFS filter applies to every repository the user owns. The bundle is a repository. Repository-local configuration and a repository-local `.gitattributes` win over the global ones, so the defect is preventable, but only if the application sets them when it creates the bundle.

**EVIDENCE.** `crate-survey.md`, gix row: "pure Rust, no git binary".

**WHAT MUST CHANGE.** At bundle creation, write a repository-local `.gitattributes` that marks every path as binary or as text with no conversion, and set `core.autocrlf=false` and `core.safecrlf=false` in the local configuration. Test the read path against a repository that a real `git` binary wrote.

### WARNING 2.4: gix has no repack or garbage-collection path

**OBSERVATION.** The survey names `write_blob`, `write_object`, `edit_tree`, `commit`, `find_tree` and `find_blob`. It names no maintenance call.

**CLAIM.** Loose objects accumulate with every commit, and nothing packs them.

**ARGUMENT.** A composition session produces many small commits. A directory with hundreds of thousands of loose object files slows every read on any file system, and it is slow to copy or to back up. A shell-out to the `git` binary would fix it, but that adds an undeclared runtime dependency that `deny.toml` and the bundle contract do not cover.

**EVIDENCE.** `crate-survey.md`, gix row.

**WHAT MUST CHANGE.** Bound the object count by design: text only in git, one commit per user action, and no derived data in the repository. State the expected object count for a one-year project. If a repack proves necessary, make it an explicit, optional, user-invoked step with a stated dependency.

### WARNING 2.5: macOS is case-insensitive by default

**OBSERVATION.** Content-hash names are usually hexadecimal or base32 text.

**CLAIM.** On an APFS volume with the default settings, two names that differ only in case are the same file.

**ARGUMENT.** A mixed-case encoding such as base64 or base58 collides. A checkout then silently merges two distinct objects.

**WHAT MUST CHANGE.** Use lowercase hexadecimal for every object name. Test on a case-insensitive volume.

### CONCERN 2.6: gix is 0.x with frequent breaking releases

**OBSERVATION.** The survey records this risk.

**WHAT MUST CHANGE.** Pin gix in `[workspace.dependencies]`. Keep every gix type inside `duet-project` behind one trait. No gix type appears in any public signature of any other crate.

---

## Claim 3: MusicXML as the storage format against a canonical format of our own

The operator allows both. The evidence says the canonical format must be the storage format from version one.

### CRITICAL 3.1: MusicXML cannot hold most of Duet's model

**OBSERVATION.** MusicXML describes an engraved score. Duet's project holds takes, playlists, regions over immutable sources, record modes, part assignments, mixer state, automation curves, pitch tracks, and a history.

**CLAIM.** MusicXML can represent none of the second list. So "MusicXML is the storage format" can only mean "MusicXML holds the score and a second format holds everything else".

**ARGUMENT.** A storage format is lossless for the model it stores; that is its definition. A hybrid is already a canonical format plus an interchange format, so the canonical format is not a later option. It is needed on day one. To postpone it is to build the same serialization work twice and to ship a round trip that drops user data in between.

**EVIDENCE.** `ardour-concepts.md` sections 3, 4, 7 and 8 list the objects. `crate-survey.md` shows no crate that maps any of them to MusicXML.

**WHAT MUST CHANGE.** Decide now. The canonical format is the storage format from version one. MusicXML is an import and export format only. This removes the hybrid, removes the lossy round trip, and removes the need to extend MusicXML with private elements.

### CRITICAL 3.2: a MusicXML fork cannot be a dependency, and a vendored fork inherits the full lint policy

**OBSERVATION.** The operator accepts "a fork or in-house MusicXML code". The `musicxml` crate had no release for 22 months.

**CLAIM.** The repository policy forbids the cheap form of a fork, and the expensive form costs more than the plan assumes.

**ARGUMENT.** `deny.toml` sets `unknown-git = "deny"` and `CLAUDE.md` states "No git dependencies". So a fork cannot be a git dependency. A vendored fork must sit under `crates/` or `tools/`, because those are the workspace member globs, and `CLAUDE.md` requires every new crate to declare `[lints] workspace = true`. A full MusicXML 4 implementation then must satisfy `missing_docs`, `missing_docs_in_private_items`, `indexing_slicing`, `as_conversions`, `unwrap_used` and `too-many-lines` on every item it contains. That is a large, unbudgeted body of work on code that no Duet engineer wrote.

**EVIDENCE.** `deny.toml` `[sources]`; `Cargo.toml` `members = ["crates/*", "tools/*"]`; `CLAUDE.md`, "Crates" and "Dependencies".

**WHAT MUST CHANGE.** Price the option before the choice. If MusicXML support is import and export only, write a narrow in-house reader and writer for the subset Duet models. A narrow module under `duet-score` is far smaller than a vendored MusicXML 4 implementation, and it is honest about what it supports.

### WARNING 3.3: the agent edit path and the git diff both need a deterministic serialization

**OBSERVATION.** The product requires a terminal agent to read and edit the composition, and git to record the result.

**CLAIM.** A generic XML serializer gives neither a stable line order nor a small diff.

**ARGUMENT.** An agent edits text. A format with one element per line, stable identifiers, sorted keys and a fixed field order gives a one-line diff for a one-note change. XML with nested attributes gives a large, ambiguous context, and attribute order is not guaranteed across serializers. The quality of the git history in claim 2 depends entirely on this property.

**WHAT MUST CHANGE.** State four properties of the canonical format: a version field, deterministic output for equal input, one record per line, and a defined place to keep unknown fields from a newer version. The last property is what "extensible" means in practice.

---

## Claim 4: the core runs headless, the GPUI app is one client

The seam is correct and I support it. The hypothesis does not yet state the contract that makes it work.

### CRITICAL 4.1: the channel contract is unstated

**OBSERVATION.** The hypothesis says "UI and core exchange commands and events over channels". It does not say bounded or unbounded, lossy or lossless, or what the user interface does after an overflow.

**CLAIM.** Without that contract the design has no backpressure and no recovery path. This is the Elastic property, and it is absent.

**ARGUMENT.** An unbounded channel grows until memory ends. A bounded channel that drops an event turns the user interface into a silently stale replica of the core, and nothing detects the divergence. Both failures are invisible in a demonstration and certain under load, for example a record pass with meters at 60 Hz on sixteen tracks.

**EVIDENCE.** The brief, "working hypothesis". The `rust-expertise` skill states the rule: "Default to bounded channels. Unbounded channels mask backpressure and crash with OOM under load."

**WHAT MUST CHANGE.** Split the traffic by kind. Structural changes travel on a bounded, lossless, ordered channel; an overflow is an error that forces a resynchronization, never a silent drop. High-rate values travel as latest-value state, where a lost intermediate value is correct behaviour. Give every core state a version counter, so a client can detect a gap and ask for a full snapshot.

### CRITICAL 4.2: the playhead and the meters must not travel as events

**OBSERVATION.** The product needs a 60 Hz playhead and live meters.

**CLAIM.** If each frame arrives as an event, the cost scales with the frame rate and with the view count, not with the change.

**ARGUMENT.** An event that reaches a view calls `cx.notify`, which marks the whole view dirty. The audit already states that `request_animation_frame` marks the whole view dirty. A full score view re-renders every frame, and the frame budget of 16.7 ms goes to layout work that did not change.

**EVIDENCE.** `gpui-kit-audit.md` section 3 and "Highest-risk gaps": "`request_animation_frame` marks the whole view dirty. Keep the playhead in a small child view or its own canvas."

**WHAT MUST CHANGE.** The transport position and the meter levels live in a shared snapshot that the engine publishes without locks. The user interface samples that snapshot once per frame from a small child view that owns its own animation frame request. No per-frame value crosses the command and event channel.

### WARNING 4.3: the command and event types must be plain data

**OBSERVATION.** The same verb list must serve the GPUI app and the agent.

**CLAIM.** If a command carries a GPUI type, the seam leaks and the agent cannot use it.

**ARGUMENT.** An `Entity<T>` is not `Send` and is not serializable. One such field in one command variant makes the whole vocabulary unusable over the Model Context Protocol (MCP) transport, and the two clients then drift apart.

**EVIDENCE.** `ardour-concepts.md` "Concepts to adopt" item 9: a flat, device-free verb list as the primary application programming interface.

**WHAT MUST CHANGE.** Commands and events derive `serde::Serialize` and `serde::Deserialize` and contain only plain data and identifiers. Add a compile-time test that asserts the bound.

### WARNING 4.4: name the latency budget and the paths that may not cross the seam

**OBSERVATION.** The hypothesis does not name a time budget anywhere.

**CLAIM.** A seam without a budget cannot be verified.

**WHAT MUST CHANGE.** State three numbers: 16.7 ms for a frame at 60 Hz; the maximum delay from a user command to an observable core state change; the maximum delay from a MIDI note to sound. State that the third path never passes through the user interface or through any channel that the user interface owns.

---

## Claim 5: the audio thread contract

The contract is right. Two of its parts are wrong as stated.

### CRITICAL 5.1: `arc-swap` frees memory on the audio thread

**OBSERVATION.** The hypothesis puts the tempo map behind `arc-swap`. The survey says "`store` drops the old value, so store only from the UI side".

**CLAIM.** That rule is not sufficient. The load side also drops.

**ARGUMENT.** The audio thread loads a snapshot and holds a reference. If the writer swaps and releases its own reference first, the audio thread holds the last one. When the audio thread drops it, the audio thread runs the destructor and calls `free`. An allocator call in the audio callback breaks the no-allocation contract, and the fault appears as a rare audio dropout that no test reproduces.

**EVIDENCE.** `crate-survey.md`, arc-swap and basedrop rows. `ardour-concepts.md` section 5: Ardour never frees on the real-time thread; the butler thread deletes dead pools.

**WHAT MUST CHANGE.** Choose one of two answers and write it down. First answer: use `triple_buffer` for every snapshot the audio thread reads, because the reader never owns and never frees; the licence is MPL-2.0, which the operator accepts. Second answer: keep `arc-swap` and add `basedrop`, with the collector on a non-audio thread. Do not use `arc-swap` alone.

### CRITICAL 5.2: cpal has no single process cycle

**OBSERVATION.** The survey states the defect plainly: "No duplex stream: input and output are two callbacks".

**CLAIM.** Two callbacks give two clocks, two threads, and no defined phase between capture and playback. The whole Ardour model that the hypothesis adopts assumes one callback that drives the session.

**ARGUMENT.** Without one cycle there is no cycle start sample, so latency compensation has no reference point. Capture alignment, which decides where a take lands on the timeline, becomes a guess. Two device clocks also drift, so a long take slowly separates from the score. For an application whose core feature is a vocal take aligned to notation, this is a product-level failure, not an implementation detail.

**EVIDENCE.** `crate-survey.md`, cpal row. `ardour-concepts.md` section 5: "the backend owns the thread and calls `AudioEngine::process_callback(nframes)`; nothing else pulls". Section 4: `AlignStyle` and `ExistingMaterial` need the input latency.

**WHAT MUST CHANGE.** Define the backend trait around one `process(nframes)` call, as Ardour does, and make the dummy backend the first implementation. Then state how each real backend satisfies it: on macOS an aggregate device gives one callback; on Linux ALSA gives a linked capture and playback pair. If cpal is used with two callbacks, then the specification must name the drift compensator, name the resampler (`rubato` is in the survey), and state the measured alignment error. Do not leave this to the implementation.

### WARNING 5.3: the block size is not fixed

**OBSERVATION.** The survey states that the device can refuse `BufferSize::Fixed`.

**WHAT MUST CHANGE.** The engine accepts a variable block size on every cycle. No internal buffer takes its size from an assumed constant. The dummy backend varies the block size on purpose, so that the test suite finds the assumption.

### WARNING 5.4: the no-allocation contract cannot be verified without unsafe

**OBSERVATION.** The operator forbids unsafe anywhere in the plan. `assert_no_alloc` is unmaintained and carries the rare BSD-1-Clause licence.

**CLAIM.** An in-house allocator guard needs `unsafe impl GlobalAlloc`, which the policy denies. So no mechanical check of the contract exists.

**ARGUMENT.** `GlobalAlloc` is an unsafe trait. Any implementation needs `unsafe impl`, and `unsafe_code` is denied at the workspace level. A new crate cannot escape the policy, because `CLAUDE.md` requires `[lints] workspace = true` on every member. The contract therefore holds by review only, and the specification must say so rather than imply a guard that cannot exist.

**EVIDENCE.** `Cargo.toml` `[workspace.lints.rust] unsafe_code = "deny"`; `CLAUDE.md`, "Crates"; `crate-survey.md`, assert_no_alloc row.

**WHAT MUST CHANGE.** Record the decision in the specification. Verification of the audio path is by three means: a written list of forbidden calls in the audio module, a long soak test over the dummy backend that counts underruns, and an external profiler run as a manual step. Name the forbidden list explicitly: no allocation, no free, no lock, no system call, no format, no logging, no panic path.

### WARNING 5.5: `panic = "abort"` plus overflow checks make arithmetic a kill switch

**OBSERVATION.** `[profile.release]` sets `panic = "abort"` and `overflow-checks = true`.

**CLAIM.** An integer overflow anywhere in the audio or time path ends the process, and the user loses the take.

**ARGUMENT.** Overflow checks in release are a good choice for correctness, and the combination with `abort` is correct for a lint policy that treats a panic as a defect. But a real-time media application must not lose a record pass to one arithmetic fault. The answer is to make the fault impossible, not to make it loud.

**EVIDENCE.** `Cargo.toml` `[profile.release]`.

**WHAT MUST CHANGE.** `duet-time` and the audio path use `checked_*` or `saturating_*` arithmetic by construction, inside newtype methods. A value that cannot be represented returns a typed error at the edge, before it reaches the audio thread.

### CONCERN 5.6: which crates satisfy the contract with no unsafe in Duet's own code

For the record, these do: `rtrb` or `ringbuf` for the single-producer single-consumer paths, `triple_buffer` for snapshots, `crossbeam::ArrayQueue` for a bounded multi-producer queue, core atomics for counters, and `basedrop` when an owned value must die off the audio thread. Pick one ring buffer crate, not two. `fundsp` allocates when it builds a graph, so build off the audio thread and hand the graph over.

---

## Claim 6: tokio beside the GPUI executor

Two runtimes in one process are workable. Each hazard needs a named rule.

### CRITICAL 6.1: shutdown order

**OBSERVATION.** The hypothesis states that tokio runs beside the GPUI executor. It states no shutdown order.

**CLAIM.** The default path either blocks the user interface or panics.

**ARGUMENT.** `Runtime::drop` blocks until the worker threads stop. A drop on the GPUI main thread freezes the window. A drop from inside an async context panics. GPUI may also quit without a drop at all, which leaves an MCP client with a half-open connection.

**WHAT MUST CHANGE.** One owner holds the runtime. A quit action cancels a `CancellationToken`, waits with `Runtime::shutdown_timeout` on a thread that is not the main thread, and then lets the application exit. Write the order as four numbered steps in the specification.

### WARNING 6.2: a tokio type outside the runtime panics, and the lint policy does not protect you

**OBSERVATION.** A tokio timer or socket used with no runtime in context panics with "there is no reactor running".

**CLAIM.** That panic comes from tokio, not from Duet code, so `clippy::panic` never sees it.

**WHAT MUST CHANGE.** No tokio type crosses into GPUI code. The boundary is a channel. The runtime handle is held in one place, and every spawn goes through it.

### WARNING 6.3: the result path must return through the foreground executor

**OBSERVATION.** A tokio task runs on a tokio worker thread. A GPUI entity is not `Send`.

**WHAT MUST CHANGE.** A tokio task sends plain data to a channel. A GPUI task created with `cx.spawn` reads that channel and publishes the value with `entity.update`. Never hold an entity handle in a tokio task. The Coding Guides section "Async work and side effects" is the reference.

### WARNING 6.4: worker threads compete with the audio thread

**OBSERVATION.** The audit reports `Priority::RealtimeAudio` and a dedicated operating-system thread. tokio starts one worker per core by default.

**CLAIM.** Default worker counts oversubscribe the machine and add jitter to the audio callback.

**EVIDENCE.** `gpui-kit-audit.md` section 10.

**WHAT MUST CHANGE.** Build the runtime with an explicit, small worker count. Two is enough for the MCP server and the file watch. Record the number and the reason.

---

## Claim 7: the wrapped timeline

This is the product's most distinctive surface, and the hypothesis is silent on its central problem.

### CRITICAL 7.1: note placement is not linear in time

**OBSERVATION.** The hypothesis says the audio lane "follows the same line breaks" as the score. It does not say how a sample maps to an x position.

**CLAIM.** An audio lane under a staff cannot be both a uniform time axis and aligned with the notes above it. The specification must choose.

**ARGUMENT.** A score engraver places a note by its duration class and by optical rules. A half note is not twice the width of a quarter note. So equal time spans get unequal widths. If the lane uses a uniform samples-per-pixel rate, the waveform and the notes disagree everywhere except at the system start, which defeats the feature. If the lane follows the engraver, the waveform stretches and compresses across the system, which is correct for alignment and unusual to look at. There is no third option while the notes stay where the engraver put them.

**EVIDENCE.** `ardour-concepts.md` section 9 gives the linear rule that a conventional editor uses: "x = 0 is `region->start()` samples into the source, and x = N is `N * samples_per_pixel` further on". That rule cannot hold here. Section 11 confirms that Ardour has no score display, so nothing can be adopted for this part.

**WHAT MUST CHANGE.** Choose alignment with the notes. Specify one map per system: a monotone, strictly increasing, piecewise-linear function from beats to x, with a knot at every engraved column. The engraver produces it as an output beside the glyph placements. The inverse gives hit testing. Every other part of this claim follows from that map.

### WARNING 7.2: a tempo change mid-system does not move x

**OBSERVATION.** The brief asks what happens when the tempo changes inside a system.

**CLAIM.** The x axis does not change at all, because note placement follows duration, not elapsed time. Only the sample axis changes.

**ARGUMENT.** The map from beats to x is fixed by the engraver. The map from beats to samples changes at the tempo point. So samples-per-pixel changes at that point and is different on each side. Every waveform segment therefore carries its own scale, and a single scalar zoom value is wrong.

**WHAT MUST CHANGE.** Compose the two maps explicitly: sample to superclock to beats through the tempo map, then beats to x through the layout map. Never cache a single samples-per-pixel value for a system.

### WARNING 7.3: a take longer than one system is several painted segments and one region

**OBSERVATION.** A take crosses a line break.

**CLAIM.** The model must not split the region. The layout produces paint segments.

**ARGUMENT.** A model split would make a line-break change edit user data. The region invariant from Ardour is POSITION, START and LENGTH over an immutable source; a layout pass has no right to touch it.

**EVIDENCE.** `ardour-concepts.md` section 3 and "Concepts to adopt" item 5.

**WHAT MUST CHANGE.** State the invariant: layout produces a list of segments per region per pass, and the list is derived data. A take that starts before the system start clips at the boundary.

### WARNING 7.4: zoom in Compose and Record invalidates every cached image

**OBSERVATION.** The brief asks what happens when the user zooms.

**CLAIM.** Zoom in a wrapped view is staff size, not samples per pixel. A change of staff size relayouts the score, moves every line break, and invalidates every layout map and every cached waveform image.

**ARGUMENT.** Ardour's waveform cache works because zoom is one scalar and a scroll is a blit. Neither holds here. A pixel cache in Compose and Record would be rebuilt on every zoom step and would never pay for itself.

**EVIDENCE.** `ardour-concepts.md` section 9, `WaveViewCache`; `gpui-kit-audit.md` "Highest-risk gaps", item about a per-frame `RenderImage` and the atlas.

**WHAT MUST CHANGE.** In Compose and Record, cache a power-of-two peak pyramid per audio source, not pixels. Build the path per segment from the pyramid. Keep the pixel cache for the Mix and Master linear timeline, where zoom is one scalar. State that these are two different draw paths.

---

## Claim 8: the crate decomposition

Twelve crates plus the application. Most boundaries are real. Four need a change.

### CRITICAL 8.1: the command vocabulary has no crate, so the layering inverts

**OBSERVATION.** `duet-agent` holds "a flat verb list" and "headless mode". `crates/duet` holds the views.

**CLAIM.** The GPUI app needs the same verb list. As drawn, it must depend on `duet-agent` to get it. That is backwards: the application would depend on the protocol adapter.

**ARGUMENT.** The verb list is the core's public interface. The MCP server is one transport over it. If the vocabulary lives in the transport crate, then every user interface change touches the agent crate, and the core cannot be tested without the protocol.

**EVIDENCE.** `ardour-concepts.md` "Concepts to adopt" item 9 and section 11: the verb list is device-free and every surface is a translator over it.

**WHAT MUST CHANGE.** Add one crate that holds the commands, the events, and the error taxonomy. `duet-engine`, `duet-agent` and `crates/duet` all depend on it. `duet-agent` becomes a thin transport adapter with no vocabulary of its own.

### WARNING 8.2: `duet-project` becomes the hub crate

**OBSERVATION.** `duet-project` holds the bundle layout, serde state, git history and file watch.

**CLAIM.** "Serde state" for the whole project makes `duet-project` depend on the score, the session, the mixer and the analysis. Every change then touches it.

**ARGUMENT.** A hub crate is a god object with a manifest. It defeats the bounded contexts that the rest of the plan sets up, and it serializes the build.

**WHAT MUST CHANGE.** Each aggregate owns its own serde model and its own version. `duet-project` owns the bundle layout, the content store, the history, and the file watch only. It composes documents; it does not define them.

### WARNING 8.3: `duet-session` and `duet-engine` will form a cycle unless the direction is stated

**OBSERVATION.** The engine reads playlists and regions to feed the disk reader. The session needs transport state.

**CLAIM.** Two natural dependencies in two directions give a cycle, which Cargo refuses. The refusal arrives at the worst time, in the middle of the engine chunk.

**WHAT MUST CHANGE.** State the rule now: `duet-engine` depends on `duet-session`; `duet-session` names no engine type. Transport state returns as an event and as a snapshot, through the vocabulary crate.

### WARNING 8.4: `duet-mix` and `duet-master` do not justify separate crates

**OBSERVATION.** The processor chain, the graph and the transport are in `duet-engine`. A mixer is a processor chain. A mastering chain is an offline graph plus loudness measurement.

**CLAIM.** `duet-mix` would hold almost nothing but re-exports of engine types.

**ARGUMENT.** A crate must earn its boundary with an invariant it alone protects. A crate whose types all come from its dependency protects nothing and adds a compile unit, a doc surface and an error enum.

**EVIDENCE.** `ardour-concepts.md` section 2: `Amp`, `PeakMeter`, `Delivery` and `PannerShell` are all processors in one chain. Section 10: the export graph is a separate, offline, typed graph.

**WHAT MUST CHANGE.** Fold the mixer into `duet-engine` as a module. Make one `duet-export` crate for the offline graph, the two-pass loudness normalization and the encoders. Drop `duet-master`.

### CONCERN 8.5: a shared DSP crate prevents a back edge

**OBSERVATION.** `duet-engine` and `duet-analysis` both need windows, buffers, resampling and sample-format conversion.

**WHAT MUST CHANGE.** Add `duet-dsp` below both. It also holds the one conversion module that finding 10.2 requires.

### CONCERN 8.6: state the price of each crate

Each crate costs a crate-level document, a typed error enum, an `# Errors` section on every public function that returns `Result`, and a `cargo doc -D warnings` surface. Twelve crates means twelve of each. Justify every crate with one sentence that names the invariant it protects. Delete any crate that fails that test.

---

## Claim 9: the test strategy

"Pure layout crates plus a thin paint element" is necessary and not sufficient.

### WARNING 9.1: make the paint element compute nothing

**OBSERVATION.** The audit states that no API returns painted paths or painted glyphs.

**CLAIM.** The mitigation works only if the element holds no logic at all.

**ARGUMENT.** If the element computes even one position, that computation is untestable forever. If the element takes a placement list and paints it in order, then the placement list is the complete assertion surface, and the element is reviewed once.

**EVIDENCE.** `gpui-kit-audit.md` section 12 and "Highest-risk gaps" item 1.

**WHAT MUST CHANGE.** Specify the element's input as a list of placements with absolute positions. Forbid any arithmetic in `paint` beyond the offset of the element bounds.

### CONCERN 9.2: draw staff lines and bar lines as quads, not paths

**OBSERVATION.** `Window::painted_quads()` returns the quads of the last frame. No equivalent exists for paths.

**CLAIM.** Staff lines, stems, beams and bar lines are axis-aligned rectangles. As quads they become assertable, and the batch does not split.

**ARGUMENT.** The audit warns that the batch splits when the draw order alternates between paths, quads and glyphs. A layout that draws rectangles as quads, then glyphs, has fewer kinds to alternate and gains a test hook at the same time.

**EVIDENCE.** `gpui-kit-audit.md` sections 1 and 12.

**WHAT MUST CHANGE.** Specify quads for every axis-aligned rectangle. Reserve paths for slurs, ties and hairpins.

### WARNING 9.3: what the user interface tests must still cover

The pure crates cannot reach any of these. Name them in the specification as required UI integration tests.

1. The held-key state machine for `w`, `h` and `e`. The audit states that no API reports held letter keys, so the view tracks key down and key up itself.
2. The lost key-up case. The window loses focus while `w` is held. The view must not stay in whole-note mode.
3. Hit testing. A click at a position selects the correct pitch and the correct time, through the inverse layout map.
4. Virtualization. The correct systems render at a given scroll offset.
5. Element identity. A stable, domain-derived `ElementId` per note and per region, never a list index.
6. Subscription retention. A stored `Subscription`, not a dropped one.

**EVIDENCE.** `gpui-kit-audit.md` sections 4, 5 and 12; the Coding Guides sections "Stable identity" and "Events, actions, and focus".

### WARNING 9.4: a golden-image suite cannot be a gate

**OBSERVATION.** `render_to_image` and `capture_screenshot` exist on macOS only, and those tests are `#[ignore]` by default.

**CLAIM.** The Definition of Done runs on macOS and Linux, so a macOS-only visual check cannot be part of it.

**EVIDENCE.** `gpui-kit-audit.md` section 12; `CLAUDE.md`, "Definition of Done".

**WHAT MUST CHANGE.** State that the visual regression suite is a developer tool, run on demand. Do not plan the gate around it.

### CONCERN 9.5: the frame budget needs a bench, not an assertion

There is no hook that asserts a frame cost. Use a `criterion` bench over the pure layout crate and over the peak path build, with a stated budget per system.

---

## Claim 10: the lint policy against real-time DSP

Two of the three named lints are not an obstacle. One needs a decision now. A third lint that nobody named is a bigger problem than both.

### The honest pass for `indexing_slicing`

`indexing_slicing` is not an obstacle for a sample loop, and no suppression is needed. Every form below is lint-clean and removes the bounds check that indexing would add.

1. A per-sample loop: `for (dst, src) in output.iter_mut().zip(input.iter())`.
2. Interleaved frames: `output.chunks_exact_mut(channel_count)`.
3. A two-span ring read: `split_at_mut` plus `copy_from_slice`.
4. A fallible single element: `slice.get(i)` with `let Some(x) = ... else { ... }`.
5. A fixed window: a slice pattern, `[a, b, c, rest @ ..]`.

State these five forms in the specification as the house idiom for the audio path.

### CRITICAL 10.1: `as_conversions` has no honest answer for the sample and pixel conversions

**OBSERVATION.** `as_conversions` is denied. `From<i16> for f32` exists. No `From` exists for `i32` to `f32`, for `u32` to `f32`, or for `usize` to `f64`.

**CLAIM.** Duet needs all three. A 24-bit or 32-bit sample converts to `f32` on every decode. A slice length converts to `f64` on every layout calculation. So the policy blocks an operation that the product requires, and the specification must resolve it before any chunk.

**ARGUMENT.** The lint policy is correct in intent: a scattered `as` hides truncation. But a denied lint with no sanctioned path produces one of two bad outcomes. Either an engineer edits `Cargo.toml` during implementation, which `CLAUDE.md` calls a defect that escalates, or `#[expect]` attributes spread across every DSP and layout file with reasons that restate the code.

**EVIDENCE.** `Cargo.toml` `[workspace.lints.clippy] as_conversions = "deny"`; `CLAUDE.md`, "Lint policy": an edit to the table made to pass the gate "is a defect that escalates, never a resolution".

**WHAT MUST CHANGE.** Decide in the specification, and name the decision as a lint-policy collision. The recommended answer is one conversion module in `duet-dsp` with a small, fixed set of functions: `i24_to_f32`, `i32_to_f32`, `f32_to_i24`, `len_to_f64`, and their inverses. Each carries one `#[expect(clippy::as_conversions, reason = "...")]` whose reason names the exact rounding and range invariant, plus a proptest for the round trip. No `as` appears anywhere else in the workspace. A vetted conversion crate is an alternative, and it needs a licence check against `deny.toml` before it enters the plan.

### WARNING 10.2: `integer_division` is denied and nobody named it

**OBSERVATION.** `Cargo.toml` denies `clippy::integer_division`. The tempo map, the superclock conversion, the tick arithmetic, the frame arithmetic and the peak binning all divide integers.

**CLAIM.** This lint will fire more often than `indexing_slicing` and `as_conversions` together, and the plan does not mention it.

**ARGUMENT.** The lint fires on the `/` operator, not on the methods. `checked_div`, `div_euclid` and `div_ceil` are methods and pass. They are also better: they make the zero-divisor case and the rounding direction explicit, which is exactly what a tempo map needs.

**EVIDENCE.** `Cargo.toml`, restriction group.

**WHAT MUST CHANGE.** State the rule: the audio and time paths divide with `div_euclid`, `div_ceil` or `checked_div`, never with `/`. Make the sample rate a `NonZeroU32` and the ticks-per-beat a `NonZeroU16`, so a zero divisor is impossible by type.

### WARNING 10.3: the complexity ceilings will bite the engraver and the DSP kernels

**OBSERVATION.** `clippy.toml` sets `too-many-lines-threshold = 120` and `cognitive-complexity-threshold = 20`.

**CLAIM.** A beaming algorithm, a note-placement pass and a pitch-tracker decode step exceed both.

**ARGUMENT.** The correct answer is usually to split the function. Sometimes it is not: a state machine with twenty arms is clearer as one function than as five. An `#[expect]` at such a site needs an invariant as its reason, and the specification must predict the sites so that the reviewer is not surprised.

**WHAT MUST CHANGE.** Name the expected suppression sites in the specification, with the reason text. A suppression discovered during implementation is a plan defect.

---

## Reactive Assessment

- **Responsive: PARTIAL.** The seam is correct, but no time budget exists, the per-frame data path is unspecified, and no timeout rule covers the agent, the file watch or the device open path.
- **Resilient: PARTIAL.** The `arc-swap` free on the audio thread, the abort-on-overflow release profile, and the unwritten external-git contract are three uncontained failures. Typed errors per crate are planned, which is right.
- **Elastic: FAIL.** No bound is stated anywhere: not on the event channel, not on the git object store, not on the audio content store, and not on the peak cache. Backpressure is absent from the hypothesis.
- **Message Driven: PARTIAL.** The command and event seam is the correct foundation. Two clients write one state with no stated single-writer rule, and the git repository is a second shared mutable substrate that a third party can edit.

---

## Ranked findings

| Tier | Finding | What must change |
|---|---|---|
| Critical | A derived `Ord` on a tagged position gives a wrong order across domains | Implement no comparison traits; order through a tempo-map function |
| Critical | "A real git repository" and "audio by content hash" are two different designs | Git holds text and a manifest; audio lives in a content store outside the index |
| Critical | No contract for a user who runs plain `git` in the bundle | Watch `HEAD`, stop the transport, refuse a rewrite while the project is open |
| Critical | MusicXML cannot hold takes, mixer state, automation or history | Make the canonical format the storage format at version one |
| Critical | A MusicXML fork cannot be a git dependency, and a vendored fork inherits the full lint policy | Price the work, or write a narrow in-house reader and writer |
| Critical | The command and event channel contract is unstated | Split structural and high-rate traffic; version every state; resynchronize on overflow |
| Critical | A 60 Hz playhead and meters as events re-render the whole view each frame | Publish a lock-free snapshot; sample it in a small child view |
| Critical | `arc-swap` lets the audio thread free memory | Use `triple_buffer`, or add `basedrop` with an off-thread collector |
| Critical | cpal has two callbacks, so there is no process cycle and no alignment reference | Define one `process(nframes)` backend trait; state the duplex or drift answer |
| Critical | tokio shutdown blocks the user interface or panics | One owner, a cancellation token, `shutdown_timeout` off the main thread |
| Critical | Note placement is not linear in time, so the audio lane cannot be a uniform axis | Specify a monotone piecewise-linear map from beats to x, per system |
| Critical | The command vocabulary has no crate, so the app would depend on the agent | Add one vocabulary crate below the app, the engine and the agent |
| Critical | `as_conversions` blocks sample and pixel conversion with no sanctioned path | One conversion module with documented `#[expect]` sites and round-trip tests |
| Warning | "Audio clock" names no unit or rate | Use a superclock newtype; convert to samples at the device edge only |
| Warning | 1920 ticks is not exact for 7, 11 or 13 part tuplets | State the rounding rule and the remainder-absorption invariant |
| Warning | gix runs no filters, hooks or Large File Storage | Write repository-local attributes and configuration at bundle creation |
| Warning | gix has no repack or garbage collection | Bound the object count by design; keep derived data out of the repository |
| Warning | macOS is case-insensitive by default | Use lowercase hexadecimal object names; test on a case-insensitive volume |
| Warning | A generic XML serializer gives no stable line order | Specify deterministic, line-stable output with an unknown-field bag |
| Warning | Commands may carry GPUI types and break the agent client | Plain data plus serde; add a compile-time bound test |
| Warning | No latency budget exists | State the frame, command and MIDI-to-sound budgets in numbers |
| Warning | The block size may vary | Accept a variable block size; vary it in the dummy backend |
| Warning | The no-allocation contract has no mechanical check without unsafe | Record review, soak test and profiler as the verification means |
| Warning | `panic = "abort"` plus overflow checks turn one overflow into data loss | Use checked or saturating arithmetic in `duet-time` and the audio path |
| Warning | A tokio type outside the runtime panics past the lint policy | No tokio type crosses into GPUI code; the boundary is a channel |
| Warning | tokio workers compete with the real-time thread | Build the runtime with an explicit small worker count |
| Warning | A tempo change mid-system changes the sample scale, not x | Compose the tempo map and the layout map; no single scalar per system |
| Warning | A take longer than a system must not split the region | Layout produces derived paint segments per pass |
| Warning | Zoom relayouts the score and invalidates every pixel cache | Cache a peak pyramid in the wrapped view; keep pixels for the linear view |
| Warning | `duet-project` becomes the hub crate | It owns the bundle, the store, the history and the watch only |
| Warning | `duet-session` and `duet-engine` will form a cycle | Engine depends on session; state returns as events and snapshots |
| Warning | `duet-mix` and `duet-master` hold no invariant of their own | Fold the mixer into the engine; make one `duet-export` crate |
| Warning | The paint element must compute nothing | Its input is an absolute placement list; no arithmetic in `paint` |
| Warning | Six user interface behaviours no layout crate can reach | Name them as required UI integration tests |
| Warning | A golden-image suite is macOS only | Keep it out of the Definition of Done |
| Warning | `integer_division` is denied and unmentioned | Divide with `div_euclid`, `div_ceil` or `checked_div`; use `NonZero` divisors |
| Warning | The complexity ceilings will bite the engraver and the DSP kernels | Predict the suppression sites and their invariant reasons in the specification |
| Concern | A packed domain tag costs masks and casts | Use an enum of two newtypes; measure before a change |
| Concern | gix is 0.x and breaks often | Pin it; keep every gix type inside `duet-project` behind one trait |
| Concern | Two crates need the same DSP primitives | Add `duet-dsp` below the engine and the analysis crate |
| Concern | Twelve crates cost twelve doc surfaces and error enums | Justify each crate with the invariant it protects |
| Concern | Paths are not assertable, quads are | Draw every axis-aligned rectangle as a quad |
| Concern | The frame budget has no assertion hook | Use a criterion bench over the pure layout crate |
| Concern | The ring buffer choice is open | Pick one of `rtrb` or `ringbuf`, not both |

---

## Verdict

The engineering is sound in its bones. The bounded contexts, the headless core, the immutable-source edit model and the adopted Ardour concepts are the right foundation, and I found no fallacy in the reasoning that produced them. The single biggest risk is the history model: "a real git repository plus audio by content hash" is two designs stated as one, and every bundle, backup and undo decision hangs from it. The weakest Reactive property is Elastic, and it fails outright: nothing in the hypothesis states a bound or a backpressure rule for any channel, store or cache.

---

## The three findings the Architect must resolve before any chunk is authored

1. **The storage format.** Decide that the canonical format is the storage format at version one, and that MusicXML is import and export only. State its four properties: a version field, deterministic output, one record per line, and a defined place for unknown fields. Everything downstream depends on this: the git diff quality, the agent edit path, the bundle layout and the `duet-score` public interface. (Findings 3.1, 3.2, 3.3.)

2. **The escape-hatch policy for `as_conversions` and `integer_division`.** Name one conversion module, its function list, its `#[expect]` reasons and its round-trip tests. Name the division idiom and the `NonZero` divisor types. This blocks the first chunk, because `duet-time` cannot be written without it, and a suppression discovered during implementation is a plan defect by the repository rule. (Findings 10.1, 10.2, 10.3, and 1.4.)

3. **The history and media model.** State what is a git object, what is a content-addressed object, how a checkout of an earlier commit restores audio, how the content store is garbage collected, and what the application does when an external `git` command moves `HEAD`. (Findings 2.1, 2.2, 2.3, 2.4.)

Two more must be settled before the engine chunk and the Record chunk, and not later: the single process cycle against cpal's two callbacks (finding 5.2), and the map from beats to x under the wrapped timeline (finding 7.1).
