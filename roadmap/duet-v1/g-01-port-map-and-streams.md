---
id: G1
line: G
depends_on: [T4, M4]
write_scope:
  - crates/duet-midi/Cargo.toml
  - crates/duet-midi/src/lib.rs
  - crates/duet-midi/src/presence.rs
  - crates/duet-midi/src/stream.rs
  - crates/duet-midi/src/portmap.rs
  - crates/duet-midi/src/bind.rs
  - crates/duet-midi/src/entry.rs
  - crates/duet-midi/src/platform_macos.rs
  - crates/duet-midi/src/platform_linux.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-midi -E 'test(port_map) + test(midir_presence)' --no-tests=fail passes; commit SHA on a branch chunk/g1-port-map-and-streams"
---

# G1: The presence and stream traits, `MidiPortMap`, and the midir fallback

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds the first layer of `duet-midi`: the `MidiPresence` and `MidiStream` traits of architecture section 8.1, `MidiPortMap` with the B88 refusal, `PlatformPort`, `MidiSink`, `HotplugSink`, `HotplugSubscription`, `MidiInputHandle`, `MidiOutputHandle`, `MidiEvent`, `MidiError`, the B87 hot-plug queue, the `MidirPresence` fallback with its documented poll, the `MidirStream` byte path, and the open timeouts of section 5.12. It also creates every module file of line G as a stub (SM2). It carries MUST story C-10 (MIDI keyboard entry with no setup) and the port half of R-13. The crate skeleton already exists, because chunk M4 created it.

**The MIDI thread is the one owner of `MidiPortMap`, and no other thread holds a reference** (TH10, section 5.7).

## Files

- `crates/duet-midi/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses, including the two target tables of section 11.2.
- `Cargo.lock` — modify. Commit it in the same commit as the manifest (SM5).
- `crates/duet-midi/src/lib.rs` — modify. Add every `mod` line of the line.
- `crates/duet-midi/src/presence.rs` — create.
- `crates/duet-midi/src/stream.rs` — create.
- `crates/duet-midi/src/portmap.rs` — create.
- `crates/duet-midi/src/bind.rs` — create as a stub.
- `crates/duet-midi/src/entry.rs` — create as a stub.
- `crates/duet-midi/src/platform_macos.rs` — create as a stub.
- `crates/duet-midi/src/platform_linux.rs` — create as a stub.

A stub holds the `//!` module documentation and nothing else (SM2).

## Types and signatures

### `duet_midi::presence` and `duet_midi::stream` (architecture 8.1)

```rust
/// Port discovery and hot-plug notification. One implementation per platform.
pub trait MidiPresence: Send {
    /// Every port the platform can see now, as a raw platform handle.
    ///
    /// **It mints no identity.** `MidiPortMap::insert` is the one place that
    /// turns a platform handle into a `MidiPortInfo`, and the MIDI thread is
    /// the one owner of the map (TH10).
    ///
    /// # Errors
    /// Returns `MidiError::Enumerate` when the platform refuses the query.
    fn ports(&self) -> Result<Vec<PlatformPort>, MidiError>;

    /// Subscribe to port arrival and port departure. The sink carries a raw
    /// `PlatformPort`, for the same reason.
    ///
    /// # Errors
    /// Returns `MidiError::Hotplug` when the platform gives no notification,
    /// and `MidiError::OpenTimeout` past B22.
    fn subscribe_hotplug(&mut self, sink: HotplugSink)
        -> Result<HotplugSubscription, MidiError>;
}

/// The byte stream. `midir` is the one implementation on both platforms.
pub trait MidiStream: Send {
    /// Open an input and push every parsed message into the sink.
    ///
    /// # Errors
    /// Returns `MidiError::Unresolved` when the map holds no `midir` index
    /// for this identity, `MidiError::Open` when the port is gone, and
    /// `MidiError::OpenTimeout` past B22.
    fn open_input(&mut self, port: &MidiPortId, sink: MidiSink)
        -> Result<MidiInputHandle, MidiError>;

    /// # Errors
    /// The same three errors as `open_input`.
    fn open_output(&mut self, port: &MidiPortId) -> Result<MidiOutputHandle, MidiError>;
}

/// One port as every other thread sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiPortInfo { slot: PortSlot, id: MidiPortId, is_source: bool }

/// A device came or went. Both variants carry the whole `MidiPortInfo`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HotplugEvent { PortAdded(MidiPortInfo), PortRemoved(MidiPortInfo) }
```

`MidiPortInfo` and `HotplugEvent` are `duet-command` types under section 1.5, and this chunk consumes them.

The four implementations of section 8.1:

| Implementation | Trait | Platform | Mechanism | Chunk |
|---|---|---|---|---|
| `CoreMidiPresence` | `MidiPresence` | macOS | The `coremidi` client notify callback. Push, no poll. | G2 |
| `PipeWireRegistryPresence` | `MidiPresence` | Linux | The `pipewire` registry listener, filtered to MIDI nodes and ports. Push, no poll. | G3 |
| `MidirPresence` | `MidiPresence` | Both | A port-list comparison at B22. The documented exception, used only when the platform presence source fails to open. | G1 |
| `MidirStream` | `MidiStream` | Both | The one byte path. It opens by `midir` port index. | G1 |

`midir` reports no connect and no disconnect, so the fallback polls. `MidirPresence::subscribe_hotplug` carries the reason in its own documentation comment, as the event-architecture rule requires.

### `duet_midi::portmap` (architecture 8.1, 15.11)

```rust
/// One platform port handle. Three namespaces meet here.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlatformPort {
    PipeWire { global: u32, name: Box<str> },
    CoreMidi { unique: i32, name: Box<str> },
    Midir { index: usize, name: Box<str> },
}

/// The one translation between the four namespaces: a `PortSlot`, a
/// `MidiPortId`, a platform handle, and the `midir` index that opens the
/// port now.
///
/// **It derives no `Default`.** A default map would be a map with no write
/// end of the B100 queue, and the one owner of port identity cannot exist
/// without the carrier that reports a change of it.
#[derive(Debug)]
pub struct MidiPortMap {
    by_slot: BTreeMap<PortSlot, MidiPortInfo>,
    by_id: BTreeMap<MidiPortId, PortSlot>,
    platform: BTreeMap<PortSlot, PlatformPort>,
    /// The next slot this process run mints. B88 bounds it.
    next: u16,
    /// Which `PortSlot` each `BoundPort` names, at B139.
    bound: [Option<PortSlot>; MAX_BOUND_PORTS],
    /// The write end of the B100 hot-plug queue to the core.
    hotplug: Arc<ArrayQueue<HotplugEvent>>,
}

impl MidiPortMap {
    /// Record a platform port and return the value every other thread reads.
    ///
    /// # Errors
    /// Returns `MidiError::PortSlotsExhausted` when B88 distinct identities
    /// have been seen in one process run.
    fn insert(&mut self, port: PlatformPort) -> Result<MidiPortInfo, MidiError>;

    /// The `midir` index that opens this identity now.
    #[must_use]
    fn midir_index(&self, port: &MidiPortId) -> Option<usize>;

    /// Retire a port that left. The record stays, so a later return of the
    /// same identity reuses its slot, and a record still in a ring is never
    /// misread.
    fn retire(&mut self, port: &MidiPortId);

    /// Take the first free `BoundPort` for one slot.
    ///
    /// # Errors
    /// Returns `MidiError::BoundPortsExhausted` when B139 ports are bound.
    fn bind(&mut self, slot: PortSlot) -> Result<BoundPort, MidiError>;

    /// Release one `BoundPort`, so a later bind may reuse the index.
    fn unbind(&mut self, port: BoundPort);
}
```

**A slot is never exhausted by re-plugging.** The counter advances only for an identity the map has never seen, so B88 bounds the number of DISTINCT devices in one process run. Past B88, `insert` returns `MidiError::PortSlotsExhausted`, the port stays unbound, and the status bar shows the `BindFailed` line.

**The collision rule, when two ports share a name.** A platform unique identifier makes the identity unique on its own, and PipeWire and CoreMIDI both supply one. `midir` supplies none. When the map must mint an identity from a name alone, and that name is already present, it appends an ordinal in first-seen order, and the ordinal is stable while the port stays present.

### `duet_midi` crate root (architecture 15.11)

```rust
/// Where a parsed MIDI message goes. The byte path owns one.
#[derive(Debug)]
pub struct MidiSink {
    port: PortSlot,
    /// The note-entry queue to the core, bounded by B31. It must drop the
    /// OLDEST record on a full queue, which `ArrayQueue::force_push` does and
    /// an `rtrb` producer cannot.
    notes: Arc<ArrayQueue<NoteEntry>>,
    /// The record ring to the audio thread, bounded by B32. `rtrb` evicts
    /// nothing, so a full ring drops the newest record.
    records: Producer<MidiRecord>,
    /// Every record either path dropped.
    dropped: Arc<AtomicU32>,
}

/// Where a raw platform port goes. The presence source owns one, and it
/// pushes into the B87 queue that the MIDI thread drains (section 8.1).
#[derive(Debug)]
pub struct HotplugSink {
    /// It must keep the NEWEST port on a full queue, which `force_push`
    /// gives and an `rtrb` producer does not.
    ports: Arc<ArrayQueue<PlatformPort>>,
    /// Set when a push evicted, so the MIDI thread rescans the whole set.
    resync: Arc<AtomicBool>,
}

/// A live hot-plug subscription. Dropping it stops the notification.
#[derive(Debug)]
pub struct HotplugSubscription {
    token: u64,
}

/// A live MIDI input. Dropping it closes the port.
#[derive(Debug)]
pub struct MidiInputHandle {
    port: PortSlot,
    id: MidiPortId,
}

/// A live MIDI output. Dropping it closes the port, through the `Drop` impl
/// the section 1.9 Drop block names.
#[derive(Debug)]
pub struct MidiOutputHandle {
    port: PortSlot,
    id: MidiPortId,
}

/// One fact the MIDI crate reports to the core.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiEvent {
    InputBound(MidiPortInfo),
    BindFailed { port: MidiPortInfo, reason: Box<str> },
    /// A bound input left. Section 8.2 states the four steps.
    InputLost(MidiPortInfo),
    PortsChanged { present: u16 },
}

/// Every way the MIDI crate refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MidiError {
    Enumerate(Box<str>),
    Hotplug(Box<str>),
    Open(Box<str>),
    OpenTimeout(MidiPortId),
    Unresolved { port: MidiPortId },
    PortSlotsExhausted,
}
```

`MidiError` gains a `BoundPortsExhausted` arm, which `MidiPortMap::bind` returns past B139. `HotplugSubscription`, `MidiInputHandle` and `MidiOutputHandle` each write a `Drop` impl, so none of them carries a `missing_copy_implementations` expectation: a type with a hand-written `Drop` impl cannot implement `Copy`, and an unfulfilled expectation is a build error under `-D warnings`.

### The member manifest (architecture 11.2)

```toml
[dependencies]
duet-time = { workspace = true }
duet-command = { workspace = true }
midir = { workspace = true }
rtrb = { workspace = true }
crossbeam-queue = { workspace = true }
thiserror = { workspace = true }
tracing = { workspace = true }

[target.'cfg(target_os = "macos")'.dependencies]
coremidi = { workspace = true }

[target.'cfg(target_os = "linux")'.dependencies]
pipewire = { workspace = true }
```

The root `[workspace.dependencies]` pins both platform crates, which chunk M4 wrote. A pin is not a dependency, so the pin is harmless on either platform.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `PortSlot`, `BoundPort`, `MidiPortId`, `MidiPortInfo`, `HotplugEvent`, `MidiMessage`, `MidiNote`, `Velocity`, `MidiRecord`, `NoteEntry` | `duet-command` | `duet_command` |
| `SampleClock` | `duet-time` | `duet_time` |
| `MAX_BOUND_PORTS` (B139) | `duet-time` | `duet_time` |

Link `T4 before G1` of section 13.4 states the reason: `MidiRecord`, `NoteEntry` and `MidiPortId` are `duet-command` types.

## Steps

1. Read `crates/duet-midi/Cargo.toml` and `crates/duet-midi/src/lib.rs`. Confirm the M4 skeleton: the `*.workspace = true` package fields, a `description`, `[lints] workspace = true`, no `[dependencies]` section, and a `src/lib.rs` that holds the `//!` crate documentation and `#![forbid(unsafe_code)]`. Report a discrepancy and stop.
2. Create every module file of the write scope as a stub. Add one `mod` line per stub to `src/lib.rs`. The `platform_macos` and `platform_linux` `mod` lines sit behind `#[cfg(target_os = "macos")]` and `#[cfg(target_os = "linux")]` in the crate root, which is the one place a `cfg(target_os)` branch appears in this crate (section 11.2 rule 1). **No `cfg(target_os)` appears inside a function body** (rule 5).
3. Write `crates/duet-midi/Cargo.toml` as the block above gives it. Run `cargo build --workspace` on macOS and on Linux, and keep `Cargo.lock` for the same commit.
4. Run `cargo tree -i alsa` on Linux and record the result. cpal 0.18.2 pins `alsa` 0.11 and that pin is not optional; the PipeWire feature of cpal already brings `pipewire` 0.10.1, so the MIDI listener shares that exact version. **If `midir` brings a second `alsa` major**, reasons 1 and 2 of section 8.1 still decide the choice, and `deny.toml` reports the duplicate as a warning rather than a failure. Report the result to the Orchestrator either way.
5. Write the failing test `port_map_mints_one_slot_per_distinct_identity` in `src/portmap.rs`. Run `cargo nextest run -p duet-midi -E 'test(port_map)' --no-tests=fail` and confirm that it fails to compile.
6. Write `PlatformPort` and `MidiPortMap` in `src/portmap.rs`, with the five methods above. `insert` is the one place that turns a platform handle into a `MidiPortInfo`, and it pushes one `HotplugEvent` into the B100 queue. `retire` keeps the record, so a later return of the same identity reuses its slot. Run the test and confirm that it passes.
7. Write the failing test `port_map_refuses_past_b88`. Run it and confirm that it fails, then implement the `MidiError::PortSlotsExhausted` refusal and confirm that it passes.
8. Write the failing test `port_map_bind_refuses_past_b139`. Run it and confirm that it fails, then implement `bind` and `unbind` over the `bound` array and the `MidiError::BoundPortsExhausted` refusal, and confirm that it passes.
9. Write the failing test `port_map_appends_an_ordinal_for_a_duplicate_name`. Run it and confirm that it fails, then implement the collision rule and confirm that it passes.
10. Write `MidiSink`, `HotplugSink`, `HotplugSubscription`, `MidiInputHandle`, `MidiOutputHandle`, `MidiEvent` and `MidiError` in `src/lib.rs`, with a `Drop` impl on each of the three handle types.
11. Write the `MidiPresence` and `MidiStream` traits in `src/presence.rs` and `src/stream.rs`.
12. Write the failing test `midir_presence_reports_an_arrival_by_poll` in `src/presence.rs`. Run `cargo nextest run -p duet-midi -E 'test(midir_presence)' --no-tests=fail` and confirm that it fails.
13. Write `MidirPresence` in `src/presence.rs`: a port-list comparison at B22, with the reason for the poll in the doc comment of `subscribe_hotplug`. It is the documented exception to the push rule, and it is used only when the platform presence source fails to open. Run the test and confirm that it passes.
14. Write `MidirStream` in `src/stream.rs`: the one byte path on both platforms. `open_input` calls `MidiPortMap::midir_index`, and a `None` returns `MidiError::Unresolved { port }`. The `midir` port list is re-read on every hot-plug event, so an index that moved is corrected before the next open.
15. Bound both opens at B22. `MidiStream::open_input` past B22 returns `MidiError::OpenTimeout`, and `MidiPresence::subscribe_hotplug` past B22 abandons the platform presence source so `MidirPresence` takes over.
16. Implement the B87 queue path of section 8.1 steps 1, 2, 4 and 5: the listener pushes a `PlatformPort` with `force_push`, which keeps the newest and sets the resync flag; the MIDI thread drains the queue, calls `insert` or `retire`, and pushes one `HotplugEvent` into the B100 queue; a set resync flag makes the MIDI thread issue one `MidiPresence::ports` enumeration on its next drain, under B22.
17. Run `cargo clippy -p duet-midi --all-targets -- -D warnings` on macOS and on Linux. Fix every finding in the code.
18. Commit on the branch `chunk/g1-port-map-and-streams`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. A `MidiPresence` double emits synthetic hot-plug events and a `MidiStream` double records what the bind opened, which is rule 2 of section 11.6. No test needs a device.

| Test | File | What it asserts |
|---|---|---|
| `port_map_mints_one_slot_per_distinct_identity` | `src/portmap.rs` | One thousand connect and disconnect cycles of one device mint one `PortSlot`. Message: "a re-plug mints no new slot". |
| `port_map_reuses_the_slot_of_a_returned_port` | `src/portmap.rs` | After `retire` and a second `insert` of the same identity, the `PortSlot` is unchanged. Message: "a returned port keeps its slot". |
| `port_map_refuses_past_b88` | `src/portmap.rs` | The B88 plus one distinct identity returns `MidiError::PortSlotsExhausted` and the port stays unbound. Message: "the map refuses a distinct identity past B88". |
| `port_map_bind_refuses_past_b139` | `src/portmap.rs` | The B139 plus one bind returns `MidiError::BoundPortsExhausted`, and an `unbind` frees an index for a later bind. Message: "the map binds at most B139 ports". |
| `port_map_appends_an_ordinal_for_a_duplicate_name` | `src/portmap.rs` | Two `PlatformPort::Midir` values with one name mint two identities whose ordinals differ, in first-seen order. Message: "a duplicate name takes an ordinal in first-seen order". |
| `port_map_insert_pushes_one_hotplug_event` | `src/portmap.rs` | One `insert` pushes one `HotplugEvent::PortAdded` into the B100 queue, and one `retire` pushes one `HotplugEvent::PortRemoved`. Message: "the map mints one hot-plug event per change". |
| `midir_presence_reports_an_arrival_by_poll` | `src/presence.rs` | A port that appears between two polls reaches the `HotplugSink` on the next poll. Message: "the midir fallback reports an arrival on its next poll". |
| `midir_presence_keeps_the_newest_on_a_full_queue` | `src/presence.rs` | With a full B87 queue, `force_push` keeps the newest port and sets the resync flag. Message: "a full presence queue keeps the newest port and sets resync". |
| `midir_presence_subscribe_expiry_hands_over` | `src/presence.rs` | A `subscribe_hotplug` past B22 returns `MidiError::OpenTimeout`, and the caller falls back to `MidirPresence`. Message: "a subscription past B22 hands over to the poll fallback". |
| `midir_stream_open_input_resolves_the_index` | `src/stream.rs` | `open_input` opens the `midir` index the map answers, and an unknown identity returns `MidiError::Unresolved { port }`. Message: "the byte path opens the index the map resolves". |
| `midir_stream_open_expiry_leaves_the_port_unbound` | `src/stream.rs` | An open past B22 returns `MidiError::OpenTimeout` and the port stays unbound. Message: "an open past B22 leaves the port unbound". |

## Verification

1. `cargo nextest run -p duet-midi -E 'test(port_map) + test(midir_presence)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of either name exists.
2. `cargo nextest run -p duet-midi --no-tests=fail` passes on both platforms.
3. `cargo clippy -p duet-midi --all-targets -- -D warnings` prints nothing on both platforms.
4. `cargo build --workspace` succeeds on macOS and on Linux with no feature flag (section 11.2 rule 6).
5. `cargo deny check` passes, and the `cargo tree -i alsa` result of step 4 is reported to the Orchestrator.
6. One commit on the branch `chunk/g1-port-map-and-streams` passes the native git hook. The commit carries `crates/duet-midi/Cargo.toml` and `Cargo.lock` together.

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
