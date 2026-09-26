---
id: T3
line: trunk
depends_on: [M2, T2, M94]
write_scope:
  - crates/bc_document/duet-session/lang_rust/Cargo.toml
  - crates/bc_document/duet-session/lang_rust/src/lib.rs
  - crates/bc_document/duet-session/lang_rust/src/session.rs
  - crates/bc_document/duet-session/lang_rust/src/track.rs
  - crates/bc_document/duet-session/lang_rust/src/take.rs
  - crates/bc_document/duet-session/lang_rust/src/region.rs
  - crates/bc_document/duet-session/lang_rust/src/location.rs
  - crates/bc_document/duet-session/lang_rust/src/mix.rs
  - crates/bc_document/duet-session/lang_rust/src/curve.rs
  - crates/bc_document/duet-session/lang_rust/src/slot.rs
  - crates/bc_document/duet-session/lang_rust/src/calibration.rs
  - crates/bc_document/duet-session/lang_rust/src/command.rs
  - crates/bc_document/duet-session/lang_rust/src/event.rs
  - crates/bc_document/duet-session/lang_rust/src/error.rs
  - crates/bc_document/duet-session/lang_rust/tests/session.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-session --no-tests=fail passes; cargo clippy -p duet-session --all-targets -- -D warnings is clean; commit SHA on a branch chunk/t3-session"
---

# T3: The session and the mix document

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk builds `duet-session`: tracks, takes as playlists, regions over immutable sources,
locations, calibrations, the mix document with its strips, slots, sends and curves, and
`SessionCommand`. It implements architecture sections 3.4, 4.3, 5.4, 6.1, 6.2, 6.3, 6.4, 6.5, 7.1,
7.2, and 15.4. Section 13.1 states the goal, the write scope, and the Completion command.

Section 13.4 puts `T2 before T3`: `Track` names one `PartId` and one `StaffId`, `BusRole::Part`
names a `PartId`, and T3 adds the `{ workspace = true }` path entry for `duet-score` (SM1, SM8).
Chunk M2 creates the `crates/bc_document/duet-session/lang_rust` skeleton, so the crate root and the member manifest
already exist. Dispatch: **Duet Engineer**.

## Files

| Path | Action |
|---|---|
| `crates/bc_document/duet-session/lang_rust/Cargo.toml` | modify (add `[dependencies]`) |
| `crates/bc_document/duet-session/lang_rust/src/lib.rs` | modify (add the `mod` and `pub use` lines) |
| `crates/bc_document/duet-session/lang_rust/src/session.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/track.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/take.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/region.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/location.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/mix.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/curve.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/slot.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/calibration.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/command.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/event.rs` | create |
| `crates/bc_document/duet-session/lang_rust/src/error.rs` | create |
| `crates/bc_document/duet-session/lang_rust/tests/session.rs` | create |
| `Cargo.lock` | modify (SM5 rule 2) |

## Types and signatures

Every signature below is copied from the architecture section the heading names. `Ticks`,
`Position`, `Delta`, `Span`, `SampleRate`, `FrameCount`, `UnixSeconds`, `SchemaVersion`, `GainDb`,
`Finite`, `TimeDomain`, `TempoMap`, `MAX_STRIPS`, `MAX_PARAMS`, `MAX_SLOTS`, and `MAX_SENDS` come
from `duet-time` (sections 2.1, 2.5, 2.6, 2.6a, 2.9, 1.6). `PartId` and `StaffId` come from
`duet-score` (section 3.2).

### `duet_session` identifiers and names (section 15.4)

```rust
/// A stable identifier for one track.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TrackId(u64);
```

`TakeId`, `RegionId`, `LocationId`, `StripId`, `SendId`, `SlotId`, and `ParamId` each carry the same
shape and the same derive set (section 15.4).

```rust
/// A track name the user reads and edits. It is never empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct TrackName(Box<str>);
```

`TakeName`, `LocationName`, and `StripName` each carry the same shape (section 15.4).

```rust
/// Which layer a region occupies inside its take. Zero is the bottom.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Layer(u8);
```

### `duet_session::track` (section 6.1, section 3.4)

```rust
/// One track. A track holds many takes and plays one.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Track {
    id: TrackId,
    part: PartId,
    /// The staff whose line the audio lane follows.
    staff: StaffId,
    /// PR R-02. The sidebar shows it and `TrackRename` writes it.
    name: TrackName,
    takes: Vec<TakeId>,
    active: TakeId,
    record_mode: RecordMode,
    align: AlignChoice,
    /// PR R-04. `InputSelection::None` means the track plays back and
    /// records nothing. The field is not an `Option`, because two encodings
    /// of one state can disagree.
    input: InputSelection,
    /// PR R-04.
    monitor: MonitorMode,
    /// PR R-05. `Verb::TrackSetArmed` is the one writer, and three surfaces
    /// read it: `TransportBar`, `Sidebar`, and the mixer strip.
    armed: bool,
}

/// One immutable audio source. An edit never touches its bytes.
#[expect(
    missing_copy_implementations,
    reason = "a source is an identity a caller must not duplicate in silence, and the type is \
              the size the PG24 table prints for `Source`"
)]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Source {
    hash: SourceHash,
    channels: NonZeroU8,
    sample_rate: SampleRate,
    frames: u64,
    /// Where the source was captured on the timeline.
    natural_position: Position,
}

/// Where a track takes its input. PR R-04.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InputSelection {
    None,
    Device { device: DeviceKey, channel: ChannelIndex },
    Pair { device: DeviceKey, left: ChannelIndex, right: ChannelIndex },
}

/// When a track passes its input to the monitor bus. PR R-04.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitorMode { Off, WhenArmed, Always }

/// What a new region does to the regions under it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordMode {
    /// Add the new region on top and keep the old one under it.
    Layered,
    /// Partition the take so that the new region cuts a hole.
    Replace,
    /// Mark the new region transparent so that both sound together.
    SoundOnSound,
}

/// Where a new take lands on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AlignChoice {
    /// Put the take where the transport clock said.
    CaptureTime,
    /// Shift the take earlier so that it lines up with what the singer heard.
    ExistingMaterial,
}
```

The `Source` `#[expect]` is one of the four `missing_copy_implementations` sites of Appendix B.1.
The reason text above is the exact text that appendix records. `SourceHash` and `AudioContainer` are
`duet-session` types of section 4.3, and this chunk declares both in `session.rs`.

### `duet_session::take` (section 6.1, section 6.4)

```rust
/// One take. A take is a playlist of regions on one track.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Take {
    id: TakeId,
    name: TakeName,
    regions: Vec<RegionId>,
    /// Whether this take is silent in the playable set.
    muted: bool,
    /// When the pass that produced this take started, as the wall clock read
    /// it.
    recorded_at: UnixSeconds,
    /// Facts about how this take was recorded. A pass can produce more than
    /// one at once, so it is a set, not a single value.
    flags: TakeFlags,
}

/// A set of `TakeFlag` values, held as a bitset over a `u8`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct TakeFlags(u8);

impl TakeFlags {
    pub fn contains(self, flag: TakeFlag) -> bool;
    pub fn with(self, flag: TakeFlag) -> Self;
    pub fn is_empty(self) -> bool;
}

/// A fact about how a take was recorded. It is stored with the take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TakeFlag {
    /// The device had no measured calibration, so the offset used the default.
    Uncalibrated,
    /// The input fell behind, so part of the take is silent.
    HadShortfall,
    /// The capture ring filled, so the file is missing frames the singer
    /// sang.
    HadOverrun,
    /// The backend reported clock drift during the pass.
    HadDrift,
    /// A bound MIDI input left during the pass.
    MidiPortLost,
}

/// One armed track's accumulating flag set, while a pass runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TrackFlags { track: TrackId, flags: TakeFlags }
```

### `duet_session::region` (section 6.1, section 7.2)

```rust
/// One view of a source on the timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Region {
    id: RegionId,
    source: SourceHash,
    position: Position,
    start: Delta,
    length: Delta,
    gain: GainDb,
    /// The two region fades.
    fade_in: Fade,
    fade_out: Fade,
    /// A transparent region sums with the layer under it.
    opaque: bool,
    layer: Layer,
}

/// One region fade: a shape and a length, and no parameter address.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Fade { shape: Interpolation, length: Delta }
```

Section 6.1 states that a fade is not an automatable parameter, so `Fade` carries no `ParamId`.

### `duet_session::location` (section 6.3)

```rust
/// One mark on the timeline. One flagged type covers every kind.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Location { id: LocationId, name: LocationName, span: Span, kind: LocationKind }

/// What a location means.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LocationKind { Mark, Range, Loop, Punch, SessionRange, Xrun }
```

### `duet_session::calibration` (section 5.4)

```rust
/// A stable key for one audio device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeviceKey(u64);

/// A measured device calibration, stored in the bundle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Calibration {
    device: DeviceKey,
    sample_rate: SampleRate,
    block: FrameCount,
    /// The measured time from a written frame to the same frame read back.
    round_trip_frames: u32,
    /// How the value was obtained.
    source: CalibrationSource,
    measured_at: UnixSeconds,
}

/// How a calibration value was obtained.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CalibrationSource {
    /// The user ran the loopback measurement on this device.
    Measured,
    /// No measurement exists. The value is the stated default.
    Default,
}
```

### `duet_session::slot` (section 5.5, section 7.1)

```rust
/// Which processor one slot runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotKind { HighPass, Equalizer, Compressor, Gate, DeEsser, Limiter, Delay, Reverb }

/// Where one slot sits in a chain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SlotPosition { PreFader(u8), PostFader(u8) }

/// One slot as the mix document stores it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SlotConfig { id: SlotId, kind: SlotKind, at: SlotPosition, params: ParamIdRange, bypassed: bool }

/// One box of the master chain, in the fixed order contract 5.2 draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MasterStage { Gain, Equalizer, Compressor, Limiter, Dither }

impl MasterStage {
    /// The five stages in the contract 5.2 order, which is the arm order.
    pub const ORDER: [Self; 5] = [
        Self::Gain, Self::Equalizer, Self::Compressor, Self::Limiter, Self::Dither,
    ];

    /// The `SlotKind` this stage inserts, or `None` when the stage is the
    /// strip trim or the render dither.
    pub const fn slot_kind(self) -> Option<SlotKind>;
}

/// The contiguous identifier range one slot's parameters occupy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamIdRange { first: ParamId, count: u16 }

/// The value range of one automatable parameter: its bounds and the value a
/// reset returns it to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParamRange { min: Finite, max: Finite, default: Finite }

/// The value range of the parameter at `offset` inside a slot of `kind`.
///
/// It returns `None` when `offset` is at or above the count B96 gives that
/// kind. It is the one source of a knob taper and of an inspector bound, so
/// no view carries a range of its own (critic WR-13).
#[must_use]
pub fn param_range(kind: SlotKind, offset: u16) -> Option<ParamRange>;
```

`MasterStage::slot_kind` returns `None` for `Gain` and for `Dither`, and it returns
`Some(SlotKind::Equalizer)`, `Some(SlotKind::Compressor)`, and `Some(SlotKind::Limiter)` for the
other three arms (section 7.1).

### `duet_session::mix` (section 7.1)

```rust
/// One channel strip. A track strip, a bus, and the master share one type.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Strip {
    id: StripId,
    kind: StripKind,
    /// The name the user reads and edits.
    name: StripName,
    trim: ParamId,
    polarity: ParamId,
    /// At most `MAX_SLOTS` entries, in signal order.
    pre_fader: Vec<SlotId>,
    fader: ParamId,
    pan: ParamId,
    /// At most `MAX_SLOTS` entries, in signal order.
    post_fader: Vec<SlotId>,
    /// At most `MAX_SENDS` entries.
    sends: Vec<SendId>,
    /// Where the strip output goes. The master strip outputs to the device.
    output: StripTarget,
    meter_tap: MeterTap,
    /// PR M-02 and M-06 make both a MUST on every strip.
    mute: bool,
    solo: bool,
}

/// What a strip carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StripKind { Track(TrackId), Bus(BusRole) }

/// What a bus is for. PR M-04 makes a reverb bus and a delay bus MUST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BusRole { Part(PartId), Reverb, Delay, Monitor, Master }

/// Where a strip sends its output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StripTarget { Strip(StripId), Device, None }

/// One send to a bus.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuxSend { id: SendId, target: StripId, level: ParamId, tap: SendTap }

/// Where a send takes its signal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SendTap { PreFader, PostFader }

/// Where a strip meter reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeterTap { Input, PreFader, PostFader, Output }

/// What the runtime computes from the mute and solo flags, once per
/// topology change, off the audio thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AudibleState { explicit_mute: bool, implicit_mute: bool }

/// The mix document: strips, sends, buses, and automation curves.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MixState {
    strips: BTreeMap<StripId, Strip>,
    sends: BTreeMap<SendId, AuxSend>,
    slots: BTreeMap<SlotId, SlotConfig>,
    curves: BTreeMap<ParamId, Curve>,
    schema: SchemaVersion,
}

/// The byte blocks of the mix documents, in `paths` order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MixDocument { strips: Vec<u8>, automation: Vec<u8> }

/// Reject a send or an output that would make a cycle.
///
/// # Errors
/// Returns `MixError::RoutingCycle` with the two strip identifiers,
/// `MixError::BusRoleReserved` for a second master or monitor bus,
/// `MixError::StripBudgetExceeded` past `MAX_STRIPS`,
/// `MixError::ParamBudgetExceeded` past `MAX_PARAMS`,
/// `MixError::SlotBudgetExceeded` for a strip whose `pre_fader` or
/// `post_fader` list passes `MAX_SLOTS`, `MixError::SendBudgetExceeded` for a
/// strip whose `sends` list passes `MAX_SENDS`, and
/// `MixError::CurveBudgetExceeded` for a curve whose point list passes B135.
pub fn validate(mix: &MixState) -> Result<(), MixError>;
```

Section 7.1 states the solo-in-place rule that `AudibleState` records, and states that `pre_fader`,
`post_fader`, and `sends` each stay a `Vec` so that a hand-edited `mix/strips.json` parses and then
fails `validate` with a named refusal.

The two `duet-session` constants of section 1.6 live in `session.rs`.

```rust
/// The current schema number of `session/session.json` (B89).
pub const SESSION_SCHEMA: SchemaVersion = SchemaVersion::new(1);
/// The current schema number of `mix/strips.json` (B89).
pub const MIX_SCHEMA: SchemaVersion = SchemaVersion::new(1);
```

### `duet_session::curve` (section 7.2)

```rust
/// The points of one curve. The variant carries the domain, so a curve has
/// no domain field that could disagree with its data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CurvePoints {
    /// Sorted by `Ticks`, with no duplicate position.
    Beats(Vec<(Ticks, Finite)>),
    /// Sorted by `SuperClock`, with no duplicate position.
    Audio(Vec<(SuperClock, Finite)>),
}

/// One automation curve. The same type serves the fader, the pan, a send
/// level, an equalizer gain, and a MIDI controller (Ardour 7).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Curve {
    parameter: ParamId,
    interpolation: Interpolation,
    points: CurvePoints,
}

/// How a curve reads between two points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Interpolation { Discrete, Linear, Exponential }

impl Curve {
    /// Build a curve. The points are sorted and de-duplicated here.
    ///
    /// # Errors
    /// Returns `MixError::EmptyCurve` for an empty point list,
    /// `MixError::DuplicatePoint` for two points at one position, and
    /// `MixError::CurveBudgetExceeded` for a list above B135.
    pub fn new(parameter: ParamId, interpolation: Interpolation, points: CurvePoints)
        -> Result<Self, MixError>;

    /// Which domain this curve is locked to.
    pub fn domain(&self) -> TimeDomain;

    /// The position of one point, in the curve's own domain.
    pub fn position_of(&self, index: usize) -> Option<Position>;

    /// The value at a position. The map converts when the domains differ.
    pub fn value_at(&self, at: Position, map: &TempoMap) -> Finite;

    /// Remove points that a straight line already covers within a tolerance.
    pub fn thin(&mut self, tolerance: Finite);
}
```

### `duet_session::session` (section 15.4)

```rust
/// The session aggregate: tracks, takes, regions, and locations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Session {
    tracks: BTreeMap<TrackId, Track>,
    takes: BTreeMap<TakeId, Take>,
    regions: BTreeMap<RegionId, Region>,
    sources: BTreeMap<SourceHash, Source>,
    locations: BTreeMap<LocationId, Location>,
    calibrations: BTreeMap<DeviceKey, Calibration>,
    schema: SchemaVersion,
}

/// The byte blocks of the session documents, in `paths` order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionDocument { session: Vec<u8>, takes: Vec<u8>, regions: Vec<u8>, locations: Vec<u8> }
```

### `duet_session::command` (section 3.4)

```rust
/// One intent against the session: tracks, takes, and the mix document.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionCommand {
    TrackAdd { part: PartId, staff: StaffId, name: TrackName },
    TrackRename { track: TrackId, name: TrackName },
    TrackReorder { track: TrackId, before: Option<TrackId> },
    TrackRemove { track: TrackId },
    TrackSetInput { track: TrackId, input: InputSelection },
    TrackSetMonitor { track: TrackId, monitor: MonitorMode },
    TrackSetArmed { track: TrackId, armed: bool },
    TakeSetMuted { take: TakeId, muted: bool },
    RegionSetStart { region: RegionId, start: Position },
    StripSetMute { strip: StripId, mute: bool },
    StripSetSolo { strip: StripId, solo: bool },
    StripClearSolo,
    StripInsertSlot { strip: StripId, at: SlotPosition, kind: SlotKind },
    StripRemoveSlot { strip: StripId, slot: SlotId },
    StripAddSend { from: StripId, to: StripId, tap: SendTap },
    BusAdd { name: StripName, role: BusRole },
    /// Change `Strip::name`.
    StripRename { strip: StripId, name: StripName },
}
```

Section 3.4 states the track-safety rule: `TrackRemove` returns `SessionError::TrackArmed` when
`Track::armed` is true, and `SessionError::TrackHasTakes { track, count }` when the track holds a
take. `TrackSetInput` returns `SessionError::TrackArmed` for the same reason. `TrackSetMonitor` and
`TrackRename` are safe while armed and do not refuse. Section 3.4 also states that `armed` is
run-time state and reaches no file.

### `duet_session::event` and `duet_session::error` (section 15.4)

```rust
/// One fact about a session change.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionEvent {
    TrackAdded(TrackId),
    TrackRemoved(TrackId),
    TrackChanged(TrackId),
    TakeAdded(TakeId),
    TakeRemoved(TakeId),
    TakeSelected { track: TrackId, take: TakeId },
    RegionsChanged(TakeId),
    LocationChanged(LocationId),
    StripChanged(StripId),
    StripAdded(StripId),
    StripRemoved(StripId),
    CurveChanged(ParamId),
}

/// Every way the session aggregate refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum SessionError {
    MissingTrack(TrackId),
    MissingTake(TakeId),
    MissingRegion(RegionId),
    MissingLocation(LocationId),
    TrackArmed(TrackId),
    TrackHasTakes { track: TrackId, count: u32 },
    Schema { found: SchemaVersion, expected: SchemaVersion },
    Parse(Box<str>),
    Time(TimeError),
}

/// Every way the mix document refuses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum MixError {
    RoutingCycle { from: StripId, to: StripId },
    BusRoleReserved(BusRole),
    StripBudgetExceeded { requested: u32 },
    ParamBudgetExceeded { requested: u32 },
    MissingStrip(StripId),
    MissingParam(ParamId),
    EmptyCurve(ParamId),
    DuplicatePoint { parameter: ParamId, at: Position },
    SlotBudgetExceeded { strip: StripId, requested: u32 },
    SendBudgetExceeded { strip: StripId, requested: u32 },
    CurveBudgetExceeded { parameter: ParamId, requested: u32 },
}
```

## Steps

1. Read `crates/bc_document/duet-session/lang_rust/Cargo.toml` and `crates/bc_document/duet-session/lang_rust/src/lib.rs`. Confirm that M2
   created both and that the manifest holds no `[dependencies]` section. Confirm that `duet-time`
   exports `MAX_STRIPS`, `MAX_PARAMS`, `MAX_SLOTS`, and `MAX_SENDS`, and that `duet-score` exports
   `PartId` and `StaffId`. Report a discrepancy and stop if any one is false.
2. Add the dependency entries to `crates/bc_document/duet-session/lang_rust/Cargo.toml`. The section 1.2 row for
   `duet-session` names two third-party crates, and section 1.3 gives the two internal edges.

   ```toml
   [dependencies]
   duet-time = { workspace = true }
   duet-score = { workspace = true }
   serde = { workspace = true }
   thiserror = { workspace = true }
   ```

3. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains the four edges.
4. Create `src/error.rs` with `SessionError` and `MixError`, and add the `mod` line. Run
   `cargo check -p duet-session`. Expected result: it succeeds.
5. Create `src/session.rs` with the eight identifier newtypes, the four name newtypes, `Layer`,
   `SourceHash`, `AudioContainer`, `Session`, `SessionDocument`, and the two schema constants. Add
   the `mod` line. Run `cargo check -p duet-session`. Expected result: it succeeds.
6. Create `src/calibration.rs`, `src/location.rs`, `src/region.rs`, `src/take.rs`, and
   `src/track.rs` with the types the section above names. Write the two `#[expect]` attributes that
   Appendix B.1 sanctions, with the exact reason text. Add the five `mod` lines. Run
   `cargo check -p duet-session`. Expected result: it succeeds.
7. Write the failing `MasterStage` tests in a `#[cfg(test)] mod tests` block at the foot of
   `src/slot.rs`. One asserts that `MasterStage::ORDER` holds the five arms in the contract 5.2
   order. One asserts that `slot_kind` returns `None` for `Gain` and for `Dither`, and the matching
   `SlotKind` for the other three. Run
   `cargo nextest run -p duet-session -E 'test(master_stage)' --no-tests=fail`. Expected result: the
   run fails.
8. Create `src/slot.rs` with `SlotKind`, `SlotPosition`, `SlotConfig`, `MasterStage` with `ORDER`
   and `slot_kind`, `ParamIdRange`, `ParamRange`, and `param_range`. `param_range` matches on every
   `SlotKind` arm, because `clippy::wildcard_enum_match_arm` is denied. Add the `mod` line. Run the
   same command. Expected result: the run passes.
9. Write the failing curve tests in a `#[cfg(test)] mod tests` block at the foot of `src/curve.rs`.
   They cover `EmptyCurve`, `DuplicatePoint`, the sort that `Curve::new` performs, and `thin`. Run
   `cargo nextest run -p duet-session -E 'test(curve)' --no-tests=fail`. Expected result: the run
   fails.
10. Create `src/curve.rs` with `CurvePoints`, `Curve`, `Interpolation`, and the five `Curve`
    methods. Add the `mod` line. Run the same command. Expected result: the run passes.
11. Write the failing mix tests in `crates/bc_document/duet-session/lang_rust/tests/session.rs`. They cover the B86 strip
    refusal, the B90 parameter refusal, the `MAX_SLOTS` refusal, the `MAX_SENDS` refusal, the
    routing-cycle refusal, and the reserved-role refusal. Run
    `cargo nextest run -p duet-session --test session --no-tests=fail`. Expected result: the run
    fails.
12. Create `src/mix.rs` with `Strip`, `StripKind`, `BusRole`, `StripTarget`, `AuxSend`, `SendTap`,
    `MeterTap`, `AudibleState`, `MixState`, `MixDocument`, and `validate`. Add the `mod` line. Run
    the same command. Expected result: the run passes.
13. Write the failing track-safety tests in `tests/session.rs`. One asserts that `TrackRemove` on an
    armed track gives `SessionError::TrackArmed`. One asserts that `TrackRemove` on a track with one
    take gives `SessionError::TrackHasTakes`. One asserts that `TrackSetInput` on an armed track
    gives `SessionError::TrackArmed`. One asserts that `TrackRename` and `TrackSetMonitor` succeed
    while armed. Run `cargo nextest run -p duet-session --test session --no-tests=fail`. Expected
    result: the run fails.
14. Create `src/command.rs` and `src/event.rs` with `SessionCommand` and `SessionEvent`, and add the
    two `mod` lines. Complete the refusal paths that the tests name. Run the same command. Expected
    result: the run passes.
15. Run `cargo build --workspace` again, then `git add` the write scope and `git commit`. The native
    hook runs `scripts/dod.sh`.

## Tests

Every assert carries a message. The unit tests live in a `#[cfg(test)] mod tests` block in the same
file. `tests/session.rs` is an integration target and wraps its tests in a `#[cfg(test)] mod tests`
block. Author guidance: the `test-author` skill.

| Test | What it asserts | Where it lives |
|---|---|---|
| `master_stage_order_is_the_arm_order` | `MasterStage::ORDER` holds `Gain`, `Equalizer`, `Compressor`, `Limiter`, `Dither` in that order | `src/slot.rs` |
| `master_stage_slot_kind_maps_three_arms` | `slot_kind` returns `None` for `Gain` and `Dither`, and the matching `SlotKind` for the other three | `src/slot.rs` |
| `param_range_refuses_an_offset_past_the_count` | `param_range` returns `None` at or above the count of each `SlotKind` | `src/slot.rs` |
| `curve_new_refuses_an_empty_list` | `Curve::new` returns `MixError::EmptyCurve` | `src/curve.rs` |
| `curve_new_refuses_a_duplicate_point` | `Curve::new` returns `MixError::DuplicatePoint` | `src/curve.rs` |
| `curve_new_sorts_the_points` | `Curve::new` returns a point list in ascending position order | `src/curve.rs` |
| `curve_thin_keeps_the_end_points` | `thin` keeps the first and the last point at every tolerance | `src/curve.rs` |
| `take_flags_set_operations` | `with` then `contains` reports the flag, and `is_empty` reports the empty set | `src/take.rs` |
| `validate_refuses_past_max_strips` | `validate` returns `MixError::StripBudgetExceeded` above B86 | `tests/session.rs` |
| `validate_refuses_past_max_params` | `validate` returns `MixError::ParamBudgetExceeded` above B90 | `tests/session.rs` |
| `validate_refuses_a_ninth_slot` | `validate` returns `MixError::SlotBudgetExceeded` above `MAX_SLOTS` | `tests/session.rs` |
| `validate_refuses_a_ninth_send` | `validate` returns `MixError::SendBudgetExceeded` above `MAX_SENDS` | `tests/session.rs` |
| `validate_refuses_a_routing_cycle` | `validate` returns `MixError::RoutingCycle` with the two strip identifiers | `tests/session.rs` |
| `bus_add_refuses_a_reserved_role` | `BusAdd` with `BusRole::Master` and with `BusRole::Monitor` each give `MixError::BusRoleReserved` | `tests/session.rs` |
| `track_remove_refuses_an_armed_track` | `TrackRemove` gives `SessionError::TrackArmed` | `tests/session.rs` |
| `track_remove_refuses_a_track_with_takes` | `TrackRemove` gives `SessionError::TrackHasTakes` with the count | `tests/session.rs` |
| `track_set_input_refuses_an_armed_track` | `TrackSetInput` gives `SessionError::TrackArmed` | `tests/session.rs` |
| `track_rename_and_monitor_allow_an_armed_track` | Both commands succeed while `armed` is true | `tests/session.rs` |

## Verification

```
cargo nextest run -p duet-session --no-tests=fail
cargo clippy -p duet-session --all-targets -- -D warnings
```

Expected output: the first command reports every unit test and every test of `tests/session.rs` as
passed, and reports no filter miss. The second command prints no warning. The two `#[expect]` sites
this chunk writes are fulfilled, so `unfulfilled_lint_expectations` raises no error.

Then commit on a branch named `chunk/t3-session`. The native git hook runs `scripts/dod.sh`, and the
commit lands only when every gate passes.

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
