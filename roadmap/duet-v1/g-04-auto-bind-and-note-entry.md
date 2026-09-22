---
id: G4
line: G
depends_on: [G3, M7]
write_scope:
  - crates/duet-midi/src/bind.rs
  - crates/duet-midi/src/entry.rs
parallelism: independent
completion: "cargo nextest run -p duet-midi -E 'test(auto_bind) + test(note_entry)' --no-tests=fail passes; commit SHA on a branch chunk/g4-auto-bind-and-note-entry"
---

# G4: The auto-bind policy, `InputLost`, the audio ring, and the note-entry queue

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of proceeding. This chunk builds plug and play of architecture section 8.2 and note entry of section 8.3: the auto-bind policy, `MidiEvent::BindFailed`, `MidiEvent::InputLost` with the four steps of section 8.2, the audio ring at B32, the note-entry queue at B31, the B100 write end on `MidiPortMap`, and the MINT of one `MidiMessage::PortGone` per departure, so the audio thread can silence a departed port's held notes. It carries MUST story C-10 (MIDI keyboard entry with no setup) and the MIDI half of R-13.

**The `MidiMessage::PortGone` arm itself is T4's work.** `MidiMessage` is a `duet-command` type, line G owns `duet-midi`, and link `T4 before G1` already orders the two. This chunk mints the value and never declares the arm.

**This chunk writes no member manifest**, so it names neither `crates/duet-midi/Cargo.toml` nor `Cargo.lock` in its write scope, and its Completion command carries no `--locked` flag (SM3 rule 4, SM5).

## Files

- `crates/duet-midi/src/bind.rs` — modify. Chunk G1 created the stub.
- `crates/duet-midi/src/entry.rs` — modify.

## Types and signatures

### `duet_midi::bind` (architecture 8.2)

On `PortAdded` the crate classifies the port. When the port is a music source and no input is bound, the application binds it and emits `MidiEvent::InputBound`. **No dialog appears.** When more than one candidate arrives, the application binds the first and offers a change in the top bar.

**The bind is bounded.** `MidiStream::open_input` runs under B22, on the MIDI thread. On expiry, or on an unresolved identity, the port stays unbound and the crate raises `MidiEvent::BindFailed { port, reason }`. The status bar shows one line with a retry action, and the automatic bind does not try that port again until it leaves and returns.

#### A bound port that leaves, in a record pass and outside one

**The take continues. It is never cancelled and never truncated.** Four steps run in order, and each one has an owner.

1. **The engine silences the held notes within one cycle.** `EngineProcess::sounding` holds one `SoundingNotes` bitset per bound port, at B139, and **the MIDI thread mints one `MidiMessage::PortGone` on the B32 ring when a port leaves**. The audio thread drains that ring every cycle and already orders it by `SampleClock`, so the departure lands in the cycle it belongs to. That cycle synthesizes a note-off for every set bit of that slot and clears the bitset. This chunk owns the MINT; chunk C3 owns the drain and the synthesis.
2. **The crate emits `MidiEvent::InputLost { port }`.** The status bar shows one line that names the port, with no modal surface.
3. **The core sets `TakeFlag::MidiPortLost` on the open take**, through the ordinary `SessionCommand` path of section 6.4. Chunk I2 owns that step.
4. **The bind policy rebinds on return.** `MidiPortMap` keeps the identity and its slot, so the same device returns with the same `PortSlot`, and the automatic bind runs again with no dialog. A record pass that loses and regains a controller therefore continues in one take.

Outside a record pass the same four steps run, and step 3 does nothing because no take is open.

The bind takes a `BoundPort` from `MidiPortMap::bind`, and an unbind releases it:

```rust
// in duet-midi, chunk G1
impl MidiPortMap {
    /// Take the first free `BoundPort` for one slot.
    ///
    /// # Errors
    /// Returns `MidiError::BoundPortsExhausted` when B139 ports are bound.
    fn bind(&mut self, slot: PortSlot) -> Result<BoundPort, MidiError>;

    /// Release one `BoundPort`, so a later bind may reuse the index.
    fn unbind(&mut self, port: BoundPort);
}
```

### `duet_midi::entry` (architecture 8.3, 5.8)

The MIDI thread parses bytes into fixed-size records. It then writes each record to **two independent destinations**, because `rtrb` is strictly single producer and single consumer.

| Destination | Mechanism | Consumer | Purpose |
|---|---|---|---|
| The sound path | `rtrb` ring of `MidiRecord`, B32 | The audio thread, every cycle, in every mode | The monitor voice and, when armed, a MIDI take |
| The note path | `crossbeam_queue::ArrayQueue<NoteEntry>`, B31 | `duet-core`, once per frame | A `ScoreCommand::InsertNote` and then a `CoreEvent` |

Both record types live in `duet-command`:

```rust
// in duet-command
/// One parsed MIDI message: a tag over at most two data values.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MidiMessage {
    NoteOn { note: MidiNote, velocity: Velocity },
    NoteOff { note: MidiNote, velocity: Velocity },
    ControlChange { controller: u8, value: u8 },
    PitchBend { value: i16 },
    Sustain { down: bool },
    /// The port this record names has left (section 8.2 step 1).
    PortGone,
}

/// One parsed MIDI record. Every field is `Copy` and fixed size, so a
/// duplicate write is cheaper than a second thread.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiRecord { port: BoundPort, at: SampleClock, message: MidiMessage }

/// One note the user played, ready for the score.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteEntry { port: PortSlot, at: SampleClock, note: MidiNote, velocity: Velocity }
```

**Neither record names `MidiPortId`.** A `Box<str>` inside either one would break three rules at once: the `Copy` derive would not compile, the size claim would be wrong, and the ring at B32 would allocate on the MIDI thread and free on the audio thread. TH10 forbids the first and TH1 forbids the second. A consumer that needs the name reads the `MidiPortInfo` the hot-plug event carried.

A full note-entry queue drops the OLDEST record with `ArrayQueue::force_push` and increments the shared counter. A full audio ring drops the NEWEST record, because `rtrb::Producer::push` evicts nothing, and increments the same counter. The counter reaches the user interface as `CoreEvent::EntryDropped { count }` on the next drain.

### Types this chunk consumes

| Type | Crate | Path |
|---|---|---|
| `MidiPortMap`, `MidiSink`, `MidiEvent`, `MidiError`, `MidiStream`, `MidiPresence`, `HotplugSink` | `duet-midi` | chunk G1 |
| `MidiMessage`, `MidiRecord`, `NoteEntry`, `MidiNote`, `Velocity`, `PortSlot`, `BoundPort`, `MidiPortInfo`, `HotplugEvent` | `duet-command` | `duet_command` |
| `SampleClock` | `duet-time` | `duet_time` |

Link `G4 before I3` of section 13.4 states the consumer: `duet-core` drains the note-entry queue that this chunk creates.

## Steps

1. Read both files of the write scope. Confirm that chunk G1 left each one a stub, and that chunks G2 and G3 have landed. Confirm that chunk M7 has landed. Report a discrepancy and stop.
2. Write the failing test `auto_bind_binds_the_first_music_source` in `src/bind.rs`. Run `cargo nextest run -p duet-midi -E 'test(auto_bind)' --no-tests=fail` and confirm that it fails to compile.
3. Write the auto-bind policy in `src/bind.rs`. On `PortAdded` it classifies the port; when the port is a music source and no input is bound it calls `MidiPortMap::bind`, opens the input through `MidiStream::open_input` under B22, and emits `MidiEvent::InputBound`. When more than one candidate arrives it binds the first and reports the rest as candidates. Run the test and confirm that it passes.
4. Write the failing test `auto_bind_reports_bind_failed_past_b22`. Run it and confirm that it fails, then implement the expiry path: the port stays unbound, `MidiEvent::BindFailed { port, reason }` is raised, and the automatic bind does not try that port again until it leaves and returns. Confirm that it passes.
5. Write the failing test `auto_bind_mints_port_gone_on_a_departure`. Run it and confirm that it fails.
6. Implement step 1 of section 8.2: on a departure the MIDI thread mints one `MidiRecord` whose `message` is `MidiMessage::PortGone` and whose `port` is the `BoundPort` the map holds, and it pushes that record on the B32 ring. It then calls `MidiPortMap::unbind`. Run the test and confirm that it passes.
7. Write the failing test `auto_bind_emits_input_lost_and_keeps_the_take`. Run it and confirm that it fails, then implement step 2 and confirm that it passes. The crate emits `MidiEvent::InputLost` and it cancels nothing.
8. Write the failing test `auto_bind_rebinds_the_same_slot_on_return`. Run it and confirm that it fails, then implement step 4 and confirm that it passes.
9. Write the failing test `note_entry_writes_both_destinations` in `src/entry.rs`. Run `cargo nextest run -p duet-midi -E 'test(note_entry)' --no-tests=fail` and confirm that it fails.
10. Write the byte parse in `src/entry.rs`. It parses bytes into a `MidiRecord` and a `NoteEntry`, both fixed size and `Copy`, and it writes each one to its own destination. **It allocates nothing on the B9 path, it never blocks, it never reads a file, and it never calls into `duet-core`** (TH10). Run the test and confirm that it passes.
11. Write the failing test `note_entry_full_queue_drops_the_oldest_and_counts`. Run it and confirm that it fails, then implement `force_push` on the note path, the plain `push` on the audio path, and the shared drop counter. Confirm that it passes.
12. Write the failing test `note_entry_carries_no_port_identity`. Confirm by construction that neither record names a `MidiPortId` and that both are `Copy`.
13. Implement the B100 write end use: `MidiPortMap::insert` and `MidiPortMap::retire` each mint one `HotplugEvent` and push it, which chunk G1 wrote; this chunk calls them from the bind path and never mints an event itself.
14. Run `cargo clippy -p duet-midi --all-targets -- -D warnings` on macOS and on Linux. Fix every finding in the code.
15. Commit on the branch `chunk/g4-auto-bind-and-note-entry`. The native git hook runs `scripts/dod.sh`.

## Tests

All tests of this chunk are unit tests in a `#[cfg(test)] mod tests` at the bottom of the file that holds the code under test. A `MidiPresence` double emits synthetic hot-plug events and a `MidiStream` double records what the bind opened, which is rule 2 of section 11.6. No test needs a device and no test sleeps.

| Test | File | What it asserts |
|---|---|---|
| `auto_bind_binds_the_first_music_source` | `src/bind.rs` | With two music sources arriving in order, the first is bound and `MidiEvent::InputBound` names it; the second is reported as a candidate and is not opened. Message: "the policy binds the first music source and offers the rest". |
| `auto_bind_opens_no_dialog` | `src/bind.rs` | A bind produces one `MidiEvent` and no modal request of any kind. Message: "plug and play opens no dialog". |
| `auto_bind_reports_bind_failed_past_b22` | `src/bind.rs` | An `open_input` that does not answer inside B22 leaves the port unbound and raises `MidiEvent::BindFailed { port, reason }`. Message: "an open past B22 leaves the port unbound and reports BindFailed". |
| `auto_bind_does_not_retry_until_the_port_returns` | `src/bind.rs` | After a `BindFailed`, the policy tries that port again only after it leaves and returns. Message: "the policy retries a failed port only after it returns". |
| `auto_bind_mints_port_gone_on_a_departure` | `src/bind.rs` | A departure pushes exactly one `MidiRecord` with `MidiMessage::PortGone` and the departing `BoundPort` on the B32 ring, and then calls `unbind`. Message: "a departure mints one PortGone record and releases the bound port". |
| `auto_bind_emits_input_lost_and_keeps_the_take` | `src/bind.rs` | A departure emits `MidiEvent::InputLost` and cancels no take and truncates no take. Message: "a departure never cancels the take". |
| `auto_bind_rebinds_the_same_slot_on_return` | `src/bind.rs` | The same device that returns binds again with the same `PortSlot` and with no dialog. Message: "a returned device binds again with the same slot". |
| `auto_bind_refuses_past_b139` | `src/bind.rs` | The B139 plus one bind reports `MidiError::BoundPortsExhausted` through `MidiEvent::BindFailed`. Message: "the policy reports a refused bind past B139". |
| `note_entry_writes_both_destinations` | `src/entry.rs` | One note-on produces one `MidiRecord` on the B32 ring and one `NoteEntry` on the B31 queue, with the same note, velocity and `SampleClock`. Message: "one note reaches both destinations". |
| `note_entry_full_queue_drops_the_oldest_and_counts` | `src/entry.rs` | With a full B31 queue the oldest record leaves and the counter rises by one; with a full B32 ring the newest record is dropped and the same counter rises. Message: "a full note queue drops the oldest and a full ring drops the newest". |
| `note_entry_carries_no_port_identity` | `src/entry.rs` | `MidiRecord` and `NoteEntry` are both `Copy` and neither names a `MidiPortId`. Message: "neither record carries a port identity". |
| `note_entry_parses_every_message_arm` | `src/entry.rs` | A table-driven loop over the five parsed arms of `MidiMessage` asserts the parsed value of each one. The message names the case: `"message {case}"`. |

## Verification

1. `cargo nextest run -p duet-midi -E 'test(auto_bind) + test(note_entry)' --no-tests=fail` passes on macOS and on Linux. It fails before this chunk, because no test of either name exists.
2. `cargo nextest run -p duet-midi --no-tests=fail` passes on both platforms.
3. `cargo clippy -p duet-midi --all-targets -- -D warnings` prints nothing on both platforms.
4. `git status` inside `crates/duet-midi` shows no change to `Cargo.toml` and no change to `Cargo.lock`, because this chunk adds no dependency.
5. One commit on the branch `chunk/g4-auto-bind-and-note-entry` passes the native git hook.

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
