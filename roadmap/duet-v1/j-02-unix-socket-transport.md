---
id: J2
line: J
depends_on: [J1]
write_scope:
  - crates/bc_gateway/duet-agent/lang_rust/src/socket.rs
  - crates/bc_gateway/duet-agent/lang_rust/src/path.rs
  - crates/bc_gateway/duet-agent/lang_rust/src/lock.rs
parallelism: independent
completion: "cargo nextest run -p duet-agent -E 'test(socket_transport)' --no-tests=fail passes; cargo clippy -p duet-agent --all-targets -- -D warnings is clean; commit SHA on a branch chunk/j2-unix-socket-transport"
---

# J2: The Unix socket transport, the path resolver, the client lock helper, and the client timeouts

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk gives the agent its second transport. `duet serve` accepts on a Unix stream
socket, so an agent edits a project while the user holds the window open. It implements architecture
sections 9.2, 9.3, 3.8, 5.12 and 15.15. It carries product story A-02, which is a MUST.

This chunk modifies three files and creates none. SM2 gave every file to chunk J1.

## Files

- `crates/bc_gateway/duet-agent/lang_rust/src/socket.rs` — modify. The accept loop over `tokio::net::UnixStream`.
- `crates/bc_gateway/duet-agent/lang_rust/src/path.rs` — modify. The platform socket path resolver.
- `crates/bc_gateway/duet-agent/lang_rust/src/lock.rs` — modify. The client lock helper and the two client timeouts.

No manifest edit is expected. Chunk J1 added `tokio` with the feature set `rmcp` and the socket both
need. Add an entry only if a step below names a crate the manifest lacks, and then add `Cargo.lock`
to the write scope and report the change.

## Types and signatures

### Consumed, declared by chunk J1 in `duet-agent`

```rust
/// Every way the transport adapter refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AgentError { Bind(Box<str>), Accept(Box<str>), Protocol(Box<str>), ReplyTimeout, Gateway(GatewayError) }
```

`Bind` and `Accept` are the two arms this chunk first uses. `ReplyTimeout` is the B16 and B17
refusal.

### The socket path, copied from architecture section 9.2

| Platform | Path |
|---|---|
| Linux | `$XDG_RUNTIME_DIR/duet/<project-hash>.sock` |
| macOS | `$TMPDIR/duet/<project-hash>.sock` |

`<project-hash>` is the lowercase hexadecimal BLAKE3 hash of the canonical bundle path. **The
application writes the resolved path into `state/agent.sock.path`**, so this crate reads that file
and computes no hash itself; `blake3` is a `duet-project` dependency and section 1.2 gives this
crate none. The socket path is outside the bundle, because a socket must not reach the git index.
The socket mode is B80.

### The two client timeouts, copied from architecture section 9.2 and 5.12

| Bound | What it bounds |
|---|---|
| B15 | The command-line client connect attempt |
| B16 | The command-line client wait for a verb reply |
| B17 | The same wait for `ExportAudio` and `Gc` |

A hung window process cannot hang an agent verb without bound.

### The transport, copied from architecture section 9.2

`rmcp`'s `AsyncRwTransport` works over any type that implements `AsyncRead` and `AsyncWrite`, so
`tokio::net::UnixStream` fits with no extra layer.

### The writer lock, copied from architecture section 3.8 and 9.3

`duet <verb>` with no server takes the writer lock, applies the verb to the files, and saves. It
exits with `ExitCode::SUCCESS`, or with one line on standard error and a failure code. The retry
policy is B25: 5 attempts, 200 ms apart, 1 s total.

## Steps

1. Read the three files of the write scope. Confirm that chunk J1 created each one with `//!`
   documentation and no item. Stop and report a discrepancy.
2. Write the failing test `socket_transport_accepts_one_client` in `src/socket.rs`, inside a
   `#[cfg(test)] mod tests`. It binds a socket in a per-test scratch directory, connects one client,
   and asserts that one `Verb::Status` round trip returns a `VerbData::Status`.
3. Run `cargo nextest run -p duet-agent -E 'test(socket_transport)' --no-tests=fail` and confirm
   that it fails.
4. Implement the path resolver in `src/path.rs`. Read `state/agent.sock.path` from the bundle when
   the file exists. When it does not, build the path from the platform table above, with the hash
   the application wrote. Return `AgentError::Bind` when the runtime directory is absent.
5. Create the parent directory with the mode B80 and remove a stale socket file before the bind.
6. Implement the accept loop in `src/socket.rs`. Bind a `tokio::net::UnixListener`, accept each
   `UnixStream`, and serve `rmcp` over it with `AsyncRwTransport`. Refuse with
   `AgentError::Accept` and continue the loop, so one bad client never ends the server.
7. Register one client per connection through `DuetCore::register_client`, and call
   `release_client` when the connection closes. The B141 refusal reaches the client as
   `CoreError::ClientBudgetExceeded`.
8. Implement the client timeouts in `src/lock.rs`. Bound the connect attempt with B15. Bound a verb
   reply with B16, and bound a reply for `Verb::ExportAudio` and `Verb::Gc` with B17. An elapsed
   wait answers `AgentError::ReplyTimeout`.
9. Implement the writer lock helper in `src/lock.rs` for the no-server path of section 9.3. Retry at
   B25 and answer `GatewayError::ProjectBusy { pid }` when the lock stays held.
10. Write the remaining tests of the Tests section.
11. Run the Completion command and confirm that it passes.
12. Run `cargo clippy -p duet-agent --all-targets -- -D warnings` and confirm that it is clean.
13. Commit on a branch named `chunk/j2-unix-socket-transport`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, and each one uses its own scratch
directory, per the `test-author` skill. Every assert carries a message.

| Test | File | What it asserts |
|---|---|---|
| `socket_transport_accepts_one_client` | `src/socket.rs` | One client connects and one `Verb::Status` round trip returns a `VerbData::Status`. |
| `socket_transport_serves_two_clients_at_once` | `src/socket.rs` | Two connections each hold their own `CoreClient`, and a verb from one never reaches the other. |
| `socket_transport_survives_a_refused_connection` | `src/socket.rs` | A client that closes mid-message produces `AgentError::Accept` or `AgentError::Protocol`, and the accept loop continues. |
| `socket_transport_releases_a_client_on_close` | `src/socket.rs` | A closed connection calls `release_client`, so the B141 registry frees the slot. |
| `socket_transport_times_out_a_reply` | `src/lock.rs` | A reply that does not arrive inside B16 answers `AgentError::ReplyTimeout`, and an `ExportAudio` reply uses B17 instead. |
| `socket_transport_resolves_the_path_from_the_bundle_file` | `src/path.rs` | The resolver reads `state/agent.sock.path` when the file exists, and it computes no hash of its own. |
| `socket_transport_builds_the_platform_path` | `src/path.rs` | On Linux the path sits under `$XDG_RUNTIME_DIR/duet/`, and on macOS under `$TMPDIR/duet/`. Each half is `#[cfg(target_os = ...)]`. |
| `socket_transport_removes_a_stale_socket_file` | `src/path.rs` | A leftover socket file is removed before the bind, and the bind then succeeds. |
| `socket_transport_retries_the_writer_lock` | `src/lock.rs` | A held lock produces 5 attempts, 200 ms apart, and then `GatewayError::ProjectBusy { pid }` (B25). |

## Verification

1. `cargo nextest run -p duet-agent -E 'test(socket_transport)' --no-tests=fail` passes.
2. `cargo nextest run -p duet-agent --no-tests=fail` passes, so chunk J1's `end_to_end` test still
   passes.
3. `cargo clippy -p duet-agent --all-targets -- -D warnings` prints no warning on macOS and on
   Linux. Both platform halves compile, because each one carries `#[cfg(target_os = ...)]`.
4. One commit on a branch named `chunk/j2-unix-socket-transport`. The native git hook runs
   `scripts/dod.sh`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are required on every item.
- A new crate lives at `crates/bc_<context>/<crate>/lang_rust/` in the context that architecture section 1.2 names (ADR 0011), declares `[lints] workspace = true`, inherits every `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- After chunk M94 lands, every `.rs` file a commit writes carries one front-matter block (`cargo xtask check-ddd --write`), and the gate refuses a changed `.rs` file with none.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
