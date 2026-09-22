---
id: G2
line: G
depends_on: [G1]
write_scope:
  - crates/duet-midi/src/platform_macos.rs
  - crates/duet-midi/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "macOS, on macos-26: cargo nextest run -p duet-midi -E 'test(coremidi_presence)' --no-tests=fail passes; commit SHA on a branch chunk/g2-coremidi-presence"
---

# G2: The macOS CoreMIDI presence source

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds `CoreMidiPresence`, the macOS implementation of `MidiPresence` over the `coremidi` client notify callback of architecture section 8.1. It is push and never a poll. It carries MUST story C-10 (MIDI keyboard entry with no setup) on macOS.

**This chunk is platform specific, so it names a per-platform command and the runner it runs on** (SM3 rule 3). `coremidi_presence` carries `#[cfg(target_os = "macos")]`, so it exists on one platform alone and the `macos-26` runner is the one that proves it.

## Files

- `crates/duet-midi/src/platform_macos.rs` — modify. Chunk G1 created the stub.
- `crates/duet-midi/Cargo.toml` — modify. Add the `coremidi` entry under the macOS target table.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_midi::platform_macos` (architecture 8.1, 11.1, 11.2)

```rust
/// The macOS presence source. It is the `coremidi` client notify callback,
/// so it is push and it never polls (section 8.1).
///
/// **It mints no identity.** The callback classifies a port and pushes one
/// `PlatformPort` into the B87 queue, and nothing else: it never touches
/// `MidiPortMap`, never opens a port, and never parses a byte (section 5.7,
/// the platform presence listener row).
#[derive(Debug)]
pub struct CoreMidiPresence { /* the fields the implementation needs */ }

impl MidiPresence for CoreMidiPresence {
    /// # Errors
    /// Returns `MidiError::Enumerate` when the platform refuses the query.
    fn ports(&self) -> Result<Vec<PlatformPort>, MidiError>;

    /// # Errors
    /// Returns `MidiError::Hotplug` when the platform gives no notification,
    /// and `MidiError::OpenTimeout` past B22.
    fn subscribe_hotplug(&mut self, sink: HotplugSink)
        -> Result<HotplugSubscription, MidiError>;
}
```

The platform handle this source produces is the `CoreMidi` arm of `PlatformPort`, which chunk G1 declared:

```rust
// in duet-midi, chunk G1
pub enum PlatformPort {
    PipeWire { global: u32, name: Box<str> },
    CoreMidi { unique: i32, name: Box<str> },
    Midir { index: usize, name: Box<str> },
}
```

CoreMIDI supplies a platform unique identifier, so the identity is unique on its own and the collision rule of section 8.1 never runs on this path.

`HotplugSubscription` keys its registration by `token`, so `Drop` needs the token and nothing else. Dropping the subscription stops the notification.

### The member manifest (architecture 11.2)

```toml
[target.'cfg(target_os = "macos")'.dependencies]
coremidi = { workspace = true }
```

The root `[workspace.dependencies]` pins `coremidi`, which chunk M4 wrote. A pin is not a dependency, so the pin is harmless on Linux. **The same `cfg` guards the module and its dependency**, which is rule 1 of section 11.2, and chunk G1 wrote the guarded `mod` line in the crate root.

### The fallback this module states (architecture 11.2 rule 3)

When `subscribe_hotplug` fails to open, or when it passes B22, the platform presence source is abandoned and `MidirPresence` takes over with its documented poll at B22. The module states that in its own `//!` documentation.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `MidiPresence`, `PlatformPort`, `HotplugSink`, `HotplugSubscription`, `MidiError` | `duet-midi` | chunk G1 |
| `MidiPortId`, `PortSlot` | `duet-command` | `duet_command` |

**`coremidi` needs no `unsafe` in this caller.** Appendix B.2 states that no crate in this plan contains `unsafe`, and `#![forbid(unsafe_code)]` is already in the crate root. When a needed `coremidi` entry point is `unsafe`, report the discrepancy to the Orchestrator and stop; do not write an `unsafe` block.

## Steps

1. Read `crates/duet-midi/src/platform_macos.rs` and `crates/duet-midi/Cargo.toml`. Confirm that chunk G1 left the module a stub with a `//!` line, and that the crate root holds the guarded `mod` line. Report a discrepancy and stop.
2. Add to `crates/duet-midi/Cargo.toml` the macOS target table with `coremidi = { workspace = true }`. Run `cargo build --workspace` on macOS and on Linux, and keep `Cargo.lock` for the same commit. The Linux build must still succeed, because the pin is not a dependency there.
3. Write the failing test `coremidi_presence_reports_an_arrival_through_the_callback` in `src/platform_macos.rs`, inside a `#[cfg(test)] mod tests` guarded by `#[cfg(target_os = "macos")]`. Run `cargo nextest run -p duet-midi -E 'test(coremidi_presence)' --no-tests=fail` on macOS and confirm that it fails to compile.
4. Write `CoreMidiPresence` in `src/platform_macos.rs`. `ports` enumerates every visible source and destination and returns one `PlatformPort::CoreMidi { unique, name }` per port. It mints no identity.
5. Write `subscribe_hotplug`. It registers the `coremidi` client notify callback and returns a `HotplugSubscription` keyed by the token. The callback classifies the port and pushes one `PlatformPort` into the `HotplugSink`, and it does nothing else. Run the test and confirm that it passes.
6. Write the failing test `coremidi_presence_departure_reaches_the_sink`. Run it and confirm that it fails, then implement the departure path and confirm that it passes.
7. Write the failing test `coremidi_presence_drop_stops_the_notification`. Run it and confirm that it fails, then implement the `Drop` behaviour of the subscription and confirm that it passes.
8. Bound the subscribe at B22. On expiry return `MidiError::OpenTimeout`, and state in the module documentation that the caller then falls back to `MidirPresence`.
9. Write the platform-conditional test of section 11.6: one test that opens the platform transport, asserts that `ports()` returns without an error, and closes it. An empty port list is a pass. It needs no device.
10. Confirm that the module holds no `cfg(target_os)` inside a function body (section 11.2 rule 5) and that the crate still carries `#![forbid(unsafe_code)]`.
11. Run `cargo clippy -p duet-midi --all-targets -- -D warnings` on macOS, and `cargo check -p duet-midi` on Linux to prove that the guarded module does not break the Linux build.
12. Commit on the branch `chunk/g2-coremidi-presence`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of `src/platform_macos.rs`, and the whole module is guarded by `#[cfg(target_os = "macos")]`. No test needs a device; a `HotplugSink` the test owns records what the callback pushed.

| Test | File | Platform | What it asserts |
|---|---|---|---|
| `coremidi_presence_reports_an_arrival_through_the_callback` | `src/platform_macos.rs` | macOS | A synthetic arrival reaches the `HotplugSink` as one `PlatformPort::CoreMidi` with the unique identifier and the name the platform gave. Message: "an arrival reaches the sink as one CoreMidi platform port". |
| `coremidi_presence_departure_reaches_the_sink` | `src/platform_macos.rs` | macOS | A synthetic departure reaches the `HotplugSink` as one `PlatformPort::CoreMidi` for the same unique identifier. Message: "a departure reaches the sink for the same unique identifier". |
| `coremidi_presence_mints_no_identity` | `src/platform_macos.rs` | macOS | Nothing the source produces is a `MidiPortInfo`, and the source holds no reference to a `MidiPortMap`. Message: "the presence source mints no identity". |
| `coremidi_presence_drop_stops_the_notification` | `src/platform_macos.rs` | macOS | After the `HotplugSubscription` is dropped, a later synthetic arrival reaches the sink no more. Message: "dropping the subscription stops the notification". |
| `coremidi_presence_subscribe_expiry_returns_open_timeout` | `src/platform_macos.rs` | macOS | A registration that does not answer inside B22 returns `MidiError::OpenTimeout`. Message: "a subscription past B22 returns OpenTimeout". |
| `coremidi_presence_ports_answers_an_empty_list` | `src/platform_macos.rs` | macOS | `ports()` returns without an error on a runner with no device, and an empty list is a pass. Message: "ports answers without an error and an empty list is a pass". |

## Verification

1. **macOS, on `macos-26`:** `cargo nextest run -p duet-midi -E 'test(coremidi_presence)' --no-tests=fail` passes. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-midi --no-tests=fail` passes on macOS.
3. `cargo check -p duet-midi` succeeds on `ubuntu-26.04`, which proves that the macOS-only module and its target dependency do not break the Linux build.
4. `cargo clippy -p duet-midi --all-targets -- -D warnings` prints nothing on macOS.
5. `cargo machete` reports no unused dependency of `duet-midi`.
6. One commit on the branch `chunk/g2-coremidi-presence` passes the native git hook. The commit carries `crates/duet-midi/Cargo.toml` and `Cargo.lock` together.

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
