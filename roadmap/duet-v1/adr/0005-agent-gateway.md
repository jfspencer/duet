# ADR-0005: The agent gateway and two runtimes in one process

## Status

Proposed.

## Context

The product is agent first. A terminal agent reads and edits the composition, imports MIDI files, and commits to the history. The GPUI application must not be the owner of state. The research warns that a graphical front end which owns state can never be separated later (research section 12).

Ardour separates "what the application can do" from "who asks". `BasicUI` is a flat, device-free verb list, and every control surface is a translator over it (research section 11, adopt item 9).

Five constraints apply.

1. The command vocabulary needs a crate below the application, the engine, and the transport. Otherwise the application depends on the protocol adapter (critic 8.1).
2. A command must carry plain data. A GPUI `Entity<T>` is not `Send` and is not serializable, so one such field makes the whole vocabulary unusable over a transport (critic 4.3).
3. tokio and GPUI's executor run in one process. `Runtime::drop` blocks, and a drop inside an async context panics. A tokio type used with no runtime panics from inside tokio, past `clippy::panic` (critic 6.1, 6.2).
4. tokio's default worker count is one per core, which oversubscribes the machine and adds jitter to the audio callback (critic 6.4).
5. A derive requirement that points upward breaks the vocabulary. Revision 4 made `Verb` demand `Hash` from every payload it reaches, and each payload then had to carry a trait it had no use for (critic R0).

PR 11 Q2 decided the shape. The `duet` command-line interface is the one interface an agent learns. It works on files with no window, and over a local socket with one. `duet mcp` serves the same verbs over the Model Context Protocol.

## Decision

Every number this record relies on is a row of specification section 1.6, cited by its id. Every
rule is cited by its id. This record carries decisions and consequences only.

1. **One flat `Verb` enum in `duet-command`.** It covers project lifecycle, jobs, transport, score edit, take management, mix, history, and session. Every client speaks it and no other vocabulary exists. `duet gc`, `duet status`, and project creation are verbs, not commands outside the list. A capability outside the list falsifies the claim that the list is the whole interface.
1a. **`duet-command` depends on `duet-score` and `duet-session` and wraps their command types.** `ScoreCommand` and `SessionCommand` live with their aggregates. Nothing below `duet-command` names `Verb`. The first draft put the command types in the vocabulary crate and the aggregate below it, which is a Cargo cycle.
1b. **The list covers every MUST story, and every capability of the product is on it.** Specification section 9.1 holds the verb list and names the story each verb serves. Revision 7 left the over mark of M-05, which design contract 4.4 specifies, with no verb, no type, and no chunk.
2. **Every type in `duet-command` is plain data with `serde`.** A compile-time assertion in the crate fails the build if a command, an event, or a snapshot stops being plain data.
2a. **The vocabulary asks a payload for nothing the payload does not already supply.** VR1 states which trait follows the fields and which one points upward, and the VR1 table is the one list of the uses it keeps. `Verb` therefore asks for neither of the two that point upward, and the one use the agent has, a comparison after a lost reply, needs neither. Specification section 3.5 holds VR1 and specification section 9.1 declares `Verb`.
2b. **One compile-time assertion holds VR2**, and specification section 3.5 holds the block and the type list it runs over. A second rule joins it: **the `Copy` derive of a vocabulary type is not a free choice**, and VR5 states the three answers, the crate class the rule binds, and the two types that must be `Copy` past the size bound. There is no `assert_eq_hash`, because VR1 removed the property it asserted.
2c. **One name for one type.** `Verb` is what crosses the transport. `EditCommand` is what the undo stack holds. Revision 2 used the word `Command` for both.
3. **A large `Verb` payload is boxed, so that `variant_size_differences` stays satisfied.** The lint is a rustc lint and not a clippy lint, and neither document may name it as one. **The lint binds every enum in the workspace, not `Verb` alone, and a box is not always the answer.** Audio-owned state holds no heap, so the chain state keeps every arm inline and carries a single-site expectation instead; PG26 refuses the box and specification section 5.7 holds the block that rule reads. Specification Appendix B.1 lists every site that keeps an expectation, with the reason text the code must carry. PG24 computes every arm size and PG25 compiles the whole roster under the real lint table, so no shape in this plan rests on an argument.
4. **There is no string-keyed escape hatch.** Ardour offers `access_action(path)`, and Duet rejects it. `clippy::wildcard_enum_match_arm` turns a new verb into a compile error at every place that must handle it.
5. **`Verb::cost` splits the work.** An immediate verb runs inline and fits B5. A deferred verb runs on a `JobRunner` thread (decision 15), and the core applies its result on the core thread. The core is mutated on one thread only (TH2), and no deferred verb runs on the thread that draws frames.
6. **One binary.** `duet` with no subcommand opens the window. `duet mcp` serves the Model Context Protocol over stdio. `duet serve` accepts on a Unix socket. `duet <verb>` applies one verb to files or over the socket. In every headless path `gpui_kit::application()` is never called.
7. **`duet-agent` is a transport adapter.** It holds no vocabulary. Each tool maps to exactly one `Verb`, and the tool schema is generated from the `serde` derive so that the two cannot drift.
8. **The socket path lives outside the bundle**, in the platform runtime directory and at the file mode B80. The application writes the resolved path into the bundle. Specification section 9.2 holds the path on each platform.
9. **One writer at a time, and a live process never loses its lock.** A second writer that finds the lock tries the socket under B15 and forwards its verbs when the socket answers. **When the socket does not answer it checks liveness before it does anything else.** A live process with a dead socket is a start in progress, so the second writer waits and retries under B25 and then exits with `GatewayError::ProjectBusy`. It takes the lock only when the recorded process is dead, and a recorded start time is what defends against a reused process identifier. Specification section 3.8 holds the steps; the first draft of this decision had no liveness check, no retry, and no start time, which let two writers hold one bundle.
10. **tokio has one owner, `Entity<AgentBridge>`, and exactly B38 worker threads.**
11. **No runtime-bound tokio type crosses into GPUI code** (TH7). **The `Runtime` value and a `Handle` are exempt**, because decision 10 requires a GPUI entity to hold the runtime. Specification section 5.7 states TH7 and specification section 9.5 names every banned kind and the two channels that carry data across the boundary. No tokio task holds an entity handle (TH6).
12. **The request channel is bounded at B35.** A tokio task that finds it full suspends on its send, so a runaway agent slows down and never exhausts memory.
13. **One choke point runs the shutdown on every exit path.** `AgentBridge::shut_down` is reached from the `Quit` action, from the application quit callback, from the `Drop` impl of `AgentBridge`, and from a signal handler thread. It is idempotent. The first draft tied shutdown to a quit action, and a platform quit, a window close, or a signal skipped it.
13a. **Shutdown covers the whole process, every step carries a bound of its own, and the main thread never blocks.** The `Runtime` moves onto a detached thread, so no drop runs inside an async context. Specification section 9.5 rule 8 holds the steps and the bound of each one; this record states no second copy of them (DR6). Revision 22 named two of the seven bounds here, so an implementer who took the contract from this record lost the waits that stop the transport, close the stream, join no worker, and save the project (critic C22I-W9).
13b. **An in-flight reply is best effort, and the agent contract says so.** A verb whose reply misses the B23 window receives none, and the connection closes. The contract is explicit: a verb that returned no reply may or may not have been applied, and the agent calls `Status` or `HistoryLog` to find out. Every mutating verb that names an identifier is idempotent, which is what makes that recovery work.
13c. **The socket client carries two timeouts**: B15 to connect and B16 for a reply, or B17 for the two long verbs. A hung window process cannot hang an agent verb without bound.
14. **The headless host uses the same seam.** The core runs on a dedicated thread and reads the same channel, so one code path serves both hosts.
15. **A deferred verb runs on a `JobRunner` in `duet-core`, and tokio is not the job runner.** tokio serves the transport only. **The core holds each document behind an `Arc`, so a worker's snapshot is a pointer copy** and a worker never touches core state. TH3 keeps every git walk, file scan, import parse, and render off the GPUI foreground thread. Specification section 9.6 holds the worker count, the queue bound, the reservation, the progress path, the cancellation flag, and the cost of a write; this record states no second copy of them (DR6). **TH3 is stated once, in specification section 5.7**, and this record cites it rather than restating it; revision 15 paraphrased it here without the timed wait and without the B110 allocation (concern N15-11).

15a. **A cancelled job and a failed job clean up after themselves.** Every job that writes owns a
    temporary directory, and the worker deletes that directory and any partly written output before
    it reports a cancellation or a failure. A finished render is renamed into place as the last
    step, so a user never finds a truncated file that looks finished. The next start empties every
    job temporary directory, because no job survives a restart. Specification section 9.6 holds the
    job states.

15b. **Every deferred verb that writes re-validates on the core thread before it commits** (TH8).
    Specification section 9.6 holds the table of re-checks. Revision 4 stated the rule for `Gc`
    alone, so a document edited during a save could have been written from a stale snapshot.

16. **Every type in `Verb` is declared in `duet-command` or lower** (VR3). That rule moved several types down, and `crates/duet` maps a `ViewState` onto GPUI through its own view type. A GPUI type never crosses the transport (VR2). Specification section 1.3 holds every placement and the edge each one uses.

16a. **`Verb::ViewSet` and `Verb::ViewGet` are the only paths to the view state, and git does not track the document that holds it.** `SessionCommand` carries no view-state variant: the view types are `duet-command` types, so such a variant would give `duet-session` an edge back to `duet-command` beside the existing edge, which Cargo refuses.

16b. **The view state is one value, and the document that holds it is declared.** Both verbs carry the whole value, so a restart restores the window as well as the splits. Specification section 10.2 declares both types and states what each field holds. Revision 5 gave the view state one mode's fields and left the aggregate in an application type that `duet-project` cannot reach.

16c. **A view document this build cannot read never blocks a project.** Every other document
refuses the open, because losing a score is not recoverable and losing a viewport is. Specification
section 10.2 names the constructor the open falls back to, the event it emits, and the write that
replaces the file; this record cites that section rather than repeating it (DR6).

16d. **No `GatewayError` arm names a crate above `duet-command`.** Four arms wrapped an upper
crate's error enum, and every one of those crates depends on `duet-command`, so each arm was the
Cargo cycle this record's own rejected-alternatives table names. One arm carries a plain-data mirror
instead, and each upper crate converts its own error into it at its own boundary, which the orphan
rule allows because the source type is local there. **`GatewayError` therefore derives serde**,
which the failed job state needs, which the event and the verb result both carry, and which the tool
schema is generated from. Specification section 15.5 declares the mirror.

## Consequences

Easier:

- The agent and the user interface can never drift, because they call one list and a new verb is a compile error in both.
- The headless host is not a second product. It is the same binary with no window.
- A test drives the whole product through verbs with no window and no hardware, which is what rung two of the completion ladder uses.
- Backpressure is explicit. A slow core slows the agent; it never grows a queue.
- The tokio hazards are each closed by a named rule, so none of them is left to an implementer.

Harder:

- The vocabulary crate is a real coupling point. A change to a command type recompiles the application, the engine, and the transport. That is the intended cost of one vocabulary.
- A boxed variant makes its construction site longer.
- `Verb::cost` must be kept honest. A verb that is marked immediate and is not will miss the frame budget, so the timing test of specification section 5.12 covers it.
- Two runtimes in one process is a standing review item. The rules of specification section 9.5 are the review checklist.
- The agent must handle a lost reply. The contract states it plainly rather than promising a guarantee the shutdown path cannot give.

## Alternatives rejected

| Alternative | Reason |
|---|---|
| A REST or JSON-RPC server of our own | MCP is the protocol the terminal agents already speak, and `rmcp` gives stdio and a stream transport with no extra layer. |
| The GPUI application as the only host, with the agent driving the user interface | It makes the graphical front end the owner of state. The research warns that this cannot be undone later. |
| The verb list inside `duet-agent` | The application would depend on the protocol adapter, so every user interface change would touch the agent crate and the core could not be tested without the protocol. |
| A verb list generated from the user interface action list | It couples the agent interface to view code, which is the inversion this decision exists to prevent. |
| A string-keyed `access_action(path)` escape hatch | It turns a missing handler into a run-time failure. A typed enum with `wildcard_enum_match_arm` denied turns it into a compile error. |
| Touching GPUI entities from a tokio task with a marker impl | An entity is not `Send` by design. The fix is a channel, not a marker impl. |
| Derive `Hash` on `Verb`, as revision 4 did with `Eq` and `Hash` together | The enum then demands the trait from every payload it reaches, so a new payload type breaks the whole vocabulary. `Eq` is no longer a choice under VR1, and the VR1 table names no use for `Hash` here. |
| One job queue with no reservation | A script that fills the queue refuses the user's next `Save`, and the user has no recovery the product named. |
| Running `rmcp` on GPUI's executor | `rmcp` requires tokio. Reimplementing the transport is a large body of work with no product gain. |
| One unbounded channel between tokio and GPUI | No backpressure. A runaway agent would exhaust memory before anything detected it. |
| tokio's default worker count | One worker per core oversubscribes the machine and adds jitter to the audio callback. |
| The command types in `duet-command`, with the aggregate below it | Cargo refuses the cycle. The aggregate must own its own command type, and the vocabulary wraps it. |
| A shutdown tied to the quit action alone | A platform quit, a window close, or a signal skips it, and `Runtime::drop` then blocks the main thread. |
| A guarantee that every in-flight reply arrives | It would mean an unbounded wait on the exit path. A bounded window plus a stated contract is honest. |
| Keep `gc`, `status`, and project creation outside the verb list | The claim that one flat list is the whole interface would be false, and the agent would need a second vocabulary for them. |
| Two binaries, one graphical and one headless | Two front ends drift. PR 11 Q2 decided one interface, two transports. |
