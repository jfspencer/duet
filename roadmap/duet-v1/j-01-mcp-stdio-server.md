---
id: J1
line: J
depends_on: [M8, I1]
write_scope:
  - crates/duet-agent/Cargo.toml
  - crates/duet-agent/src/lib.rs
  - crates/duet-agent/src/stdio.rs
  - crates/duet-agent/src/shutdown.rs
  - crates/duet-agent/src/socket.rs
  - crates/duet-agent/src/path.rs
  - crates/duet-agent/src/lock.rs
  - crates/duet-agent/tests/end_to_end.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-agent -E 'test(stdio) + test(shutdown)' --no-tests=fail passes; cargo clippy -p duet-agent --all-targets -- -D warnings is clean; commit SHA on a branch chunk/j1-mcp-stdio-server"
---

# J1: The Model Context Protocol server over stdio, and the headless end-to-end test

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk builds the transport adapter of architecture section 9.2 over standard input
and standard output, with `rmcp` on tokio. It also writes the shutdown choke point of section 9.5
rule 7 for the headless hosts, and the headless end-to-end test that proves everything phase 8
delivers. It implements architecture sections 9.2, 9.3, 9.5, 12.1 and 15.15, and ADR
`adr/0005-agent-gateway.md`. It carries product stories A-01, A-03, A-04, A-05, C-02, H-01, H-02,
H-03, H-04, H-06, R-05, R-07 and X-08.

**`duet-agent` is a transport adapter. It holds no vocabulary.** Architecture section 9.2 states
that rule, and section 1.2 gives the crate the invariant "A transport translates; it owns no
vocabulary". Every value this crate carries is a `duet-command` type.

This chunk is the FIRST chunk of line J, so SM2 makes it create every module file the line will ever
need. Chunk J2 fills `socket.rs`, `path.rs` and `lock.rs`.

## Files

- `crates/duet-agent/Cargo.toml` — modify. M8 created the skeleton with no `[dependencies]` section.
- `crates/duet-agent/src/lib.rs` — modify. M8 wrote the `//!` crate doc and
  `#![forbid(unsafe_code)]`. Add one `mod` line per file below.
- `crates/duet-agent/src/stdio.rs` — create. The `rmcp` server over standard input and output.
- `crates/duet-agent/src/shutdown.rs` — create. The signal handler and the idempotent choke point.
- `crates/duet-agent/src/socket.rs` — create. Documentation stub; chunk J2 fills it.
- `crates/duet-agent/src/path.rs` — create. Documentation stub; chunk J2 fills it.
- `crates/duet-agent/src/lock.rs` — create. Documentation stub; chunk J2 fills it.
- `crates/duet-agent/tests/end_to_end.rs` — create. SM2 covers `src/` only, so an integration test
  needs no `mod` line and this chunk creates the file in its own write scope.
- `Cargo.lock` — modify (SM5 rule 2).

## Types and signatures

### Declared by this chunk, in `crates/duet-agent/src/lib.rs`

Copied from architecture section 15.15.

```rust
/// Every way the transport adapter refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AgentError { Bind(Box<str>), Accept(Box<str>), Protocol(Box<str>), ReplyTimeout, Gateway(GatewayError) }
```

### Consumed from `duet-command` and `duet-core`

| Type | Crate | Declaring section |
|---|---|---|
| `Verb`, `VerbOutcome`, `VerbData`, `Gateway` | `duet-command` | 9.1 |
| `GatewayRequest { verb, client }`, `GatewayError` | `duet-command` | 15.5 |
| `DuetCore`, `CoreClient`, `CoreLink`, `CoreError` | `duet-core` | 15.14 |

### The transport rules, copied from architecture section 9.2 and 9.3

- **stdio.** `duet mcp` serves MCP over standard input and standard output with `rmcp` 3.4.0 on
  tokio. The `duet` binary never writes to standard output in this mode except through the
  transport.
- **Each MCP tool maps to exactly one `Verb`.** The tool schema is generated from the `serde`
  derive, so the two never drift.
- **Headless mode.** `duet mcp --project <path>` opens a bundle with the dummy backend and never
  calls `gpui_kit::application()`. GPUI is not initialized and no window opens.
- **The worker count is B38, and it is explicit.** `Builder::new_multi_thread().worker_threads(2)`.
- **Backpressure is the bound.** The request channel holds B35. A tokio task that finds it full
  suspends on `send().await`.
- **The reply is best effort.** A verb whose reply does not arrive inside B23 receives no reply at
  all, and the connection closes. The agent contract states that a verb which returned no reply may
  or may not have been applied, and that the agent must call `Status` or `HistoryLog` to find out
  (section 9.5 rule 9).

### The shutdown choke point, copied from architecture section 9.5 rule 7

The choke point is idempotent. It checks an `AtomicBool` and returns at once on a second call. In
the headless hosts the caller is a signal handler thread that `duet mcp` and `duet serve` install
for `SIGINT` and `SIGTERM`. In the window host the caller is `AgentBridge::shut_down`, which chunk
K1 writes in `crates/duet`.

**The `Drop` path absorbs every failure and never panics.** `Drop` has no `?`, no `unwrap`, and no
`expect`. A panic during an unwind would abort the process.

## Steps

1. Read `crates/duet-agent/Cargo.toml` and `crates/duet-agent/src/lib.rs`. Confirm that M8 wrote the
   package fields, `description`, `[lints] workspace = true`, the `//!` crate doc and
   `#![forbid(unsafe_code)]`, and that no `[dependencies]` section exists. Stop and report a
   discrepancy.
2. Add the `[dependencies]` section. Add one `{ workspace = true }` entry per crate this chunk
   names: `duet-command`, `duet-core`, `rmcp`, `tokio`, `tokio-util`, `thiserror`, `tracing`.
   Architecture section 1.2 lists exactly those third-party crates for this crate.
3. Run `cargo build --workspace` and commit the `Cargo.lock` it produces.
4. Create every module file of the Files table with its `//!` module documentation, and add the
   `mod` lines to `src/lib.rs`.
5. Declare `AgentError` in `src/lib.rs` with the five arms of section 15.15. Derive
   `thiserror::Error` and write one `#[error("...")]` message per arm in Simplified Technical
   English.
6. Write the failing test `stdio_maps_one_tool_to_one_verb` in `src/stdio.rs`, inside a
   `#[cfg(test)] mod tests`. It builds the tool list and asserts that every tool name maps to
   exactly one `Verb` arm and that no `Verb` arm has two tools.
7. Run `cargo nextest run -p duet-agent -E 'test(stdio)' --no-tests=fail` and confirm that it fails.
8. Implement the tokio runtime builder in `src/stdio.rs` with
   `Builder::new_multi_thread().worker_threads(2)`, which is B38.
9. Implement the `rmcp` server over standard input and standard output. Generate each tool schema
   from the `serde` derive of `Verb`, so the two never drift.
10. Implement the request path: a tokio task sends a `GatewayRequest` on the B35 channel and awaits
    the reply under a `tokio::time::timeout` of B23. On an elapsed wait the task abandons the wait
    and closes the connection.
11. Implement the headless host in `src/stdio.rs`: open the bundle with the dummy backend, register
    one client with `DuetCore::register_client`, and never call `gpui_kit::application()`.
12. Implement `src/shutdown.rs`: the idempotent choke point over an `AtomicBool`, the `SIGINT` and
    `SIGTERM` handler thread, and the `Drop` path that absorbs every failure through a
    `tracing::warn!`.
13. Write `crates/duet-agent/tests/end_to_end.rs`, wrapped in a `#[cfg(test)] mod tests` because
    `clippy::tests_outside_test_module` is denied. Architecture section 13.2 states its whole
    assertion set, and this test asserts what phase 8 delivers and nothing more:
    `ProjectCreate` with the SATB template, `TrackAdd`, `TrackSetInput`, `TrackSetArmed`, a MIDI
    file import, eight recorded bars from the dummy backend, a commit, a branch, a note edit, a
    second commit, a checkout of the first commit with a byte compare of the score, `Status` with
    the take count, and a `ViewSet` and `ViewGet` round trip.
14. Leave the test without `#[ignore]`. It runs green in every phase from this one, so
    `scripts/dod.sh` runs it at every commit from phase 8 onward.
15. Write the remaining tests of the Tests section.
16. Run the Completion command and confirm that it passes.
17. Run `cargo clippy -p duet-agent --all-targets -- -D warnings` and confirm that it is clean.
18. Commit on a branch named `chunk/j1-mcp-stdio-server`.

## Tests

Every unit test lives in a `#[cfg(test)] mod tests` in the same file. The integration test lives in
`crates/duet-agent/tests/end_to_end.rs`, wrapped in a `#[cfg(test)] mod tests`. Every assert carries
a message.

| Test | File | What it asserts |
|---|---|---|
| `stdio_maps_one_tool_to_one_verb` | `src/stdio.rs` | Every tool name maps to exactly one `Verb` arm, and no `Verb` arm carries two tools (section 9.2). |
| `stdio_writes_nothing_outside_the_transport` | `src/stdio.rs` | The headless host writes no byte to standard output except through the transport (section 9.2). |
| `stdio_uses_two_tokio_workers` | `src/stdio.rs` | The runtime builder names B38 workers and never the default count (section 9.5 rule 3). |
| `stdio_suspends_a_producer_on_a_full_request_channel` | `src/stdio.rs` | A B35 channel that is full suspends `send().await` and returns no refusal, so a runaway agent slows down and never exhausts memory. |
| `stdio_abandons_a_reply_past_the_budget` | `src/stdio.rs` | A reply that does not arrive inside B23 leaves the task with no reply and closes the connection (section 9.5 rule 9). |
| `shutdown_is_idempotent` | `src/shutdown.rs` | A second call answers `AlreadyRun` and performs no work. |
| `shutdown_absorbs_every_failure_in_drop` | `src/shutdown.rs` | The `Drop` path discards a refusal through a warning and returns, and it holds no `?`, no `unwrap` and no `expect`. |
| `shutdown_runs_from_a_signal` | `src/shutdown.rs` | A simulated `SIGTERM` reaches the choke point exactly once. |
| `end_to_end` (module `tests`) | `crates/duet-agent/tests/end_to_end.rs` | The thirteen steps of architecture section 13.2, in order, with a byte compare of the score after the checkout and an equal `ViewState` after the `ViewSet` and `ViewGet` round trip. |

**The headless product path is TWO tests, and the phase numbers are the reason** (critic C16-9). The
export half needs chunk I4 in phase 10, chunk H3 in phase 9 and chunk H4 in phase 10. A single test
would fail every commit of phases 8 and 9, and CLAUDE.md forbids `--no-verify`. Chunk J3 writes
`export_end_to_end` in phase 11.

## Verification

1. `cargo nextest run -p duet-agent -E 'test(stdio) + test(shutdown)' --no-tests=fail` passes and
   selects at least one test per term.
2. `cargo nextest run -p duet-agent --test end_to_end --no-tests=fail` passes.
3. `cargo clippy -p duet-agent --all-targets -- -D warnings` prints no warning.
4. One commit on a branch named `chunk/j1-mcp-stdio-server`. The native git hook runs
   `scripts/dod.sh`. Quote the command output before any claim of success, per the
   `verification-before-completion` skill.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
