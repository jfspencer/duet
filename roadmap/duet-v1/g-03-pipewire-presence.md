---
id: G3
line: G
depends_on: [G2]
write_scope:
  - crates/duet-midi/src/platform_linux.rs
  - crates/duet-midi/Cargo.toml
  - Cargo.lock
parallelism: independent
completion: "Linux, on ubuntu-26.04: cargo nextest run -p duet-midi -E 'test(pipewire_presence)' --no-tests=fail passes; commit SHA on a branch chunk/g3-pipewire-presence"
---

# G3: The Linux PipeWire registry presence source

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds `PipeWireRegistryPresence`, the Linux implementation of `MidiPresence` over the `pipewire` registry listener of architecture section 8.1, filtered to MIDI nodes and ports. It is push and never a poll. `roadmap/duet-v1/research/linux-macos-platform.md` supplies every verified platform fact. It carries MUST story C-10 (MIDI keyboard entry with no setup) on Linux.

**This chunk is platform specific, so it names a per-platform command and the runner it runs on** (SM3 rule 3). `pipewire_presence` carries `#[cfg(target_os = "linux")]`, so it exists on one platform alone and the `ubuntu-26.04` runner is the one that proves it.

## Files

- `crates/duet-midi/src/platform_linux.rs` — modify. Chunk G1 created the stub.
- `crates/duet-midi/Cargo.toml` — modify. Add the `pipewire` entry under the Linux target table.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).

## Types and signatures

### `duet_midi::platform_linux` (architecture 8.1, 11.1, 11.2)

```rust
/// The Linux presence source. It is the `pipewire` registry listener,
/// filtered to MIDI nodes and ports, so it is push and it never polls
/// (section 8.1).
///
/// **It mints no identity.** The listener classifies a port and pushes one
/// `PlatformPort` into the B87 queue, and nothing else: it never touches
/// `MidiPortMap`, never opens a port, and never parses a byte (section 5.7,
/// the platform presence listener row).
#[derive(Debug)]
pub struct PipeWireRegistryPresence { /* the fields the implementation needs */ }

impl MidiPresence for PipeWireRegistryPresence {
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

The platform handle this source produces is the `PipeWire` arm of `PlatformPort`, which chunk G1 declared:

```rust
// in duet-midi, chunk G1
pub enum PlatformPort {
    PipeWire { global: u32, name: Box<str> },
    CoreMidi { unique: i32, name: Box<str> },
    Midir { index: usize, name: Box<str> },
}
```

PipeWire supplies a platform unique identifier, the registry global, so the identity is unique on its own and the collision rule of section 8.1 never runs on this path.

### Why the PipeWire registry and not the ALSA sequencer (architecture 8.1)

Three reasons decide the hot-plug source, and the order matters because the first one is conditional.

1. **It needs no `unsafe`.** `ListenerLocalBuilder::global`, `global_remove`, and `register` are safe Rust, so node and port hot-plug needs no unsafe in the caller. `unsafe` is denied workspace-wide, so this reason is sufficient on its own.
2. **One session bus serves audio and MIDI.** cpal's PipeWire host already connects to the server, so the MIDI listener adds no second connection and no second failure mode.
3. **It is expected to add no duplicate crate version.** cpal 0.18.2 pins `alsa` 0.11 on Linux and that pin is not optional. The PipeWire feature of cpal already brings `pipewire` 0.10.1, so the MIDI listener shares that exact version. Chunk G1 ran `cargo tree -i alsa` and recorded the result; if `midir` brings a second `alsa` major, this reason is void, reasons 1 and 2 still decide, and `deny.toml` reports the duplicate as a warning rather than a failure.

`alsa` is not a direct Duet dependency on either reading.

### The member manifest (architecture 11.2)

```toml
[target.'cfg(target_os = "linux")'.dependencies]
pipewire = { workspace = true }
```

The root `[workspace.dependencies]` pins `pipewire` at 0.10.1, which chunk M4 wrote and which is the version cpal 0.18.2 brings through its `pipewire` feature, so the tree holds one version. A pin is not a dependency, so the pin is harmless on macOS. **The same `cfg` guards the module and its dependency**, which is rule 1 of section 11.2, and chunk G1 wrote the guarded `mod` line in the crate root.

### The fallback this module states (architecture 11.2 rule 3)

When `subscribe_hotplug` fails to open, or when it passes B22, the platform presence source is abandoned and `MidirPresence` takes over with its documented poll at B22. The module states that in its own `//!` documentation.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `MidiPresence`, `PlatformPort`, `HotplugSink`, `HotplugSubscription`, `MidiError` | `duet-midi` | chunk G1 |
| `MidiPortId`, `PortSlot` | `duet-command` | `duet_command` |

**`pipewire` needs no `unsafe` in this caller.** Appendix B.2 states that section 8.1 is the one caller of the `pipewire` crate and that it uses the registry listener, which is safe Rust. Only the raw `pw_buffer` path and `pw::deinit()` are `unsafe`, and neither is needed. When a needed entry point is `unsafe`, report the discrepancy to the Orchestrator and stop; do not write an `unsafe` block.

## Steps

1. Read `crates/duet-midi/src/platform_linux.rs` and `crates/duet-midi/Cargo.toml`. Confirm that chunk G1 left the module a stub with a `//!` line, that the crate root holds the guarded `mod` line, and that chunk G2 has landed. Report a discrepancy and stop.
2. Add to `crates/duet-midi/Cargo.toml` the Linux target table with `pipewire = { workspace = true }`. Run `cargo build --workspace` on Linux and on macOS, and keep `Cargo.lock` for the same commit. The macOS build must still succeed, because the pin is not a dependency there.
3. Run `cargo tree -d` on Linux and confirm that the tree holds one `pipewire` version. Report the result to the Orchestrator.
4. Write the failing test `pipewire_presence_reports_an_arrival_from_the_registry` in `src/platform_linux.rs`, inside a `#[cfg(test)] mod tests` guarded by `#[cfg(target_os = "linux")]`. Run `cargo nextest run -p duet-midi -E 'test(pipewire_presence)' --no-tests=fail` on Linux and confirm that it fails to compile.
5. Write `PipeWireRegistryPresence` in `src/platform_linux.rs`. `ports` walks the registry and returns one `PlatformPort::PipeWire { global, name }` per MIDI node and per MIDI port. It mints no identity.
6. Write `subscribe_hotplug`. It registers the registry listener through `ListenerLocalBuilder::global` and `global_remove`, and it returns a `HotplugSubscription` keyed by the token. The listener classifies the global and pushes one `PlatformPort` into the `HotplugSink`, and it does nothing else. Run the test and confirm that it passes.
7. Write the failing test `pipewire_presence_filters_out_a_non_midi_node`. Run it and confirm that it fails, then implement the MIDI filter and confirm that it passes.
8. Write the failing test `pipewire_presence_departure_reaches_the_sink`. Run it and confirm that it fails, then implement the `global_remove` path and confirm that it passes.
9. Write the failing test `pipewire_presence_drop_stops_the_notification`. Run it and confirm that it fails, then implement the `Drop` behaviour of the subscription and confirm that it passes.
10. Bound the subscribe at B22. On expiry return `MidiError::OpenTimeout`, and state in the module documentation that the caller then falls back to `MidirPresence`.
11. Write the platform-conditional test of section 11.6: one test that opens the platform transport, asserts that `ports()` returns without an error, and closes it. An empty port list is a pass. It needs no daemon.
12. Confirm that the module holds no `cfg(target_os)` inside a function body (section 11.2 rule 5), that it calls no deprecated interface, and that the crate still carries `#![forbid(unsafe_code)]`.
13. Run `cargo clippy -p duet-midi --all-targets -- -D warnings` on Linux, and `cargo check -p duet-midi` on macOS to prove that the guarded module does not break the macOS build.
14. Commit on the branch `chunk/g3-pipewire-presence`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of `src/platform_linux.rs`, and the whole module is guarded by `#[cfg(target_os = "linux")]`. No test needs a running daemon; a `HotplugSink` the test owns records what the listener pushed, and a registry double supplies synthetic globals.

| Test | File | Platform | What it asserts |
|---|---|---|---|
| `pipewire_presence_reports_an_arrival_from_the_registry` | `src/platform_linux.rs` | Linux | A synthetic MIDI global reaches the `HotplugSink` as one `PlatformPort::PipeWire` with the global and the name the registry gave. Message: "an arrival reaches the sink as one PipeWire platform port". |
| `pipewire_presence_filters_out_a_non_midi_node` | `src/platform_linux.rs` | Linux | An audio-only node produces no push into the sink. Message: "the listener filters out a node that is not MIDI". |
| `pipewire_presence_departure_reaches_the_sink` | `src/platform_linux.rs` | Linux | A `global_remove` for a MIDI global reaches the `HotplugSink` for the same global. Message: "a departure reaches the sink for the same global". |
| `pipewire_presence_mints_no_identity` | `src/platform_linux.rs` | Linux | Nothing the source produces is a `MidiPortInfo`, and the source holds no reference to a `MidiPortMap`. Message: "the presence source mints no identity". |
| `pipewire_presence_drop_stops_the_notification` | `src/platform_linux.rs` | Linux | After the `HotplugSubscription` is dropped, a later synthetic global reaches the sink no more. Message: "dropping the subscription stops the notification". |
| `pipewire_presence_subscribe_expiry_returns_open_timeout` | `src/platform_linux.rs` | Linux | A registration that does not answer inside B22 returns `MidiError::OpenTimeout`. Message: "a subscription past B22 returns OpenTimeout". |
| `pipewire_presence_ports_answers_an_empty_list` | `src/platform_linux.rs` | Linux | `ports()` returns without an error on a runner with no daemon, and an empty list is a pass. Message: "ports answers without an error and an empty list is a pass". |

## Verification

1. **Linux, on `ubuntu-26.04`:** `cargo nextest run -p duet-midi -E 'test(pipewire_presence)' --no-tests=fail` passes. It fails before this chunk, because no test of that name exists.
2. `cargo nextest run -p duet-midi --no-tests=fail` passes on Linux.
3. `cargo check -p duet-midi` succeeds on `macos-26`, which proves that the Linux-only module and its target dependency do not break the macOS build.
4. `cargo clippy -p duet-midi --all-targets -- -D warnings` prints nothing on Linux.
5. `cargo tree -d` on Linux shows one `pipewire` version, and the result is reported to the Orchestrator.
6. `cargo deny check` passes, and any duplicate `alsa` major is a warning and not a failure.
7. One commit on the branch `chunk/g3-pipewire-presence` passes the native git hook. The commit carries `crates/duet-midi/Cargo.toml` and `Cargo.lock` together.

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
