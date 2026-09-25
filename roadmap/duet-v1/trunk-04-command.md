---
id: T4
line: trunk
depends_on: [M3, T2, T3]
write_scope:
  - crates/bc_document/duet-command/lang_rust/Cargo.toml
  - crates/bc_document/duet-command/lang_rust/src/lib.rs
  - crates/bc_document/duet-command/lang_rust/src/verb.rs
  - crates/bc_document/duet-command/lang_rust/src/outcome.rs
  - crates/bc_document/duet-command/lang_rust/src/request.rs
  - crates/bc_document/duet-command/lang_rust/src/event.rs
  - crates/bc_document/duet-command/lang_rust/src/snapshot.rs
  - crates/bc_document/duet-command/lang_rust/src/view.rs
  - crates/bc_document/duet-command/lang_rust/src/document.rs
  - crates/bc_document/duet-command/lang_rust/src/fault.rs
  - crates/bc_document/duet-command/lang_rust/src/midi.rs
  - crates/bc_document/duet-command/lang_rust/src/recent.rs
  - crates/bc_document/duet-command/lang_rust/src/export.rs
  - crates/bc_document/duet-command/lang_rust/src/error.rs
  - crates/bc_document/duet-command/lang_rust/tests/vocabulary.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-command --no-tests=fail passes; cargo clippy -p duet-command --all-targets -- -D warnings is clean; commit SHA on a branch chunk/t4-command"
---

# T4: The transport vocabulary

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk builds `duet-command`, the crate whose invariant is that every value that crosses the
transport is plain data. It delivers `Verb`, `VerbOutcome`, `VerbData`, `GatewayError`, the request
records, `EditCommand`, `Mode`, `ModeView`, `ViewState`, `ViewDocument`, the MIDI record types with
their `size_of` test, `DitherKind`, `FaultCode`, the snapshots, the `BundleDocument` trait with its
four impls, and the plain-data assertion. It implements architecture sections 1.8, 3.5, 7.4, 8.1,
8.3, 8.4, 9.1, 9.5, 9.6, 10.2, 12.4, and 15.5, and ADR `adr/0005-agent-gateway.md`. Section 13.1
states the goal, the write scope, and the Completion command.

Section 13.4 puts `T2 and T3 before T4`, because `Verb` wraps `ScoreCommand` and `SessionCommand`.
Chunk M3 creates the `crates/bc_document/duet-command/lang_rust` skeleton, so the crate root and the member manifest
already exist. Dispatch: **Duet Engineer**.

## Files

| Path | Action |
|---|---|
| `crates/bc_document/duet-command/lang_rust/Cargo.toml` | modify (add `[dependencies]`) |
| `crates/bc_document/duet-command/lang_rust/src/lib.rs` | modify (add the `mod` lines, the `pub use` lines, and the plain-data assertion) |
| `crates/bc_document/duet-command/lang_rust/src/verb.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/outcome.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/request.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/event.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/snapshot.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/view.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/document.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/fault.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/midi.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/recent.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/export.rs` | create |
| `crates/bc_document/duet-command/lang_rust/src/error.rs` | create |
| `crates/bc_document/duet-command/lang_rust/tests/vocabulary.rs` | create |
| `Cargo.lock` | modify (SM5 rule 2) |

## Types and signatures

Every signature below is copied from the architecture section the heading names. `Position`,
`Span`, `Delta`, `SuperClock`, `SampleClock`, `SampleRate`, `FrameCount`, `BarCount`,
`UnixSeconds`, `SchemaVersion`, `Finite`, `Tempo`, `Meter`, and `NoteValue` come from `duet-time`.
`ScoreCommand`, `ScoreEvent`, `Selection`, `ScoreSelector`, `Clipboard`, `CanonicalDocument`,
`Revision`, `ElementRef`, `KeySignature`, `PartId`, and `Score` come from `duet-score`.
`SessionCommand`, `SessionEvent`, `Session`, `MixState`, `SessionDocument`, `MixDocument`,
`SessionError`, `MixError`, `SourceHash`, `DeviceKey`, `TrackId`, `TakeId`, `RegionId`, `StripId`,
`SlotId`, `SendId`, `ParamId`, `TrackName`, `TakeName`, `StripName`, `InputSelection`,
`MonitorMode`, `RecordMode`, `AlignChoice`, `BusRole`, `SendTap`, `SlotKind`, `SlotPosition`,
`Interpolation`, and `CurvePoints` come from `duet-session`.

### `duet_command::error` (section 15.5)

```rust
/// Every way the vocabulary crate refuses a document.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum CommandError {
    Serialize(Box<str>),
    Parse(Box<str>),
    Schema { found: SchemaVersion, expected: SchemaVersion },
}

/// Every way the gateway refuses a verb.
#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
pub enum GatewayError {
    NoProject,
    ProjectBusy { pid: u32 },
    RecordInProgress,
    HistoryUnresolved,
    SaveRaced,
    JobQueueFull { user_half: bool },
    PoolExhausted { kind: SlotKind },
    RingBudgetExceeded { requested_bytes: u64 },
    StripBudgetExceeded { requested: u32 },
    ParamBudgetExceeded { requested: u32 },
    BusRoleReserved(BusRole),
    NotYetImplemented,
    Score(ScoreError),
    Session(SessionError),
    Mix(MixError),
    Command(CommandError),
    Upstream(UpstreamFailure),
}
```

No arm of `GatewayError` names a crate above `duet-command`. `Upstream` carries the plain-data
mirror instead (section 15.5).

### `duet_command::fault` (section 12.4, section 15.5)

```rust
/// Which crate above `duet-command` refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureSurface { Engine, Project, Interchange, Export }

/// A stable machine-readable code for one upstream failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FailureCode {
    NoServer, DeviceOpen, DeviceRate, Configuration, Transport,
    Io, Parse, Schema, NotFound, Limit, Format, Cancelled, Timeout,
}

/// One failure from a crate above `duet-command`, mirrored as plain data.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpstreamFailure { surface: FailureSurface, code: FailureCode, detail: Box<str> }

/// Which fault one cycle raised. The audio thread writes a code and never a
/// message, so it formats nothing and allocates nothing (TH1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultCode { Starved, Overflow, Shortfall, Misconfigured, DeviceGone }

/// A fault the audio thread reports. Every variant is `Copy`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineFault {
    PlaybackStarved { track: TrackId, frames: u32 },
    CaptureOverflow { track: TrackId, frames: u32 },
    CaptureShortfall { track: TrackId, frames: u32 },
    BlockSizeAdjusted { requested: u32, actual: u32 },
    CaptureStalled { track: TrackId },
    AllocationSlow { milliseconds: u32 },
    HandoffBusy { generation: u64 },
    HandoffGenerationSkew { published: u64, adopted: u64 },
    ChainSlotVacant { index: u16 },
    CalibrationInvalid,
    CalibrationTimeout,
    DeviceLost,
    ClockDrift { parts_per_million: i32 },
    ChainMisconfigured { strip: StripId },
    FaultsDropped { count: u32 },
    DeviceStalled,
}

/// What the engine can be, as the user interface shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineState {
    NoServer { detail: ServerDetail },
    OpenTimeout { device: DeviceLabel },
    NoDevice,
    RateUnavailable { project: SampleRate, device: SampleRate },
    Running,
    Faulted { fault: EngineFault },
}

/// Why no audio server answered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerDetail { SocketMissing, LibraryMissing, ConnectionRefused }

/// A device name the user reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceLabel(Box<str>);
```

Section 12.4 states that `HandoffGenerationSkew` carries two plain `u64` values, because
`Generation` is a `duet-engine` type and this enum lives in `duet-command`.

### `duet_command::midi` (section 8.1, section 8.3)

```rust
/// One MIDI port, as the audio path and the note path address it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PortSlot(u16);

/// A stable identity for one MIDI port, in the form
/// `<platform>:<unique>:<name>`.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MidiPortId(Box<str>);

/// One port as every other thread sees it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiPortInfo { slot: PortSlot, id: MidiPortId, is_source: bool }

/// A MIDI note number, 0 to 127.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct MidiNote(u8);

/// A MIDI velocity, 0 to 127.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Velocity(u8);

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

/// Which of the bound input ports one record came from, at B139.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundPort(u16);

/// One parsed MIDI record. Every field is `Copy` and fixed size, so a
/// duplicate write is cheaper than a second thread.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MidiRecord { port: BoundPort, at: SampleClock, message: MidiMessage }

/// One note the user played, ready for the score. The core turns it into a
/// `ScoreCommand`.
/// **Audio-owned** (section 5.7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NoteEntry { port: PortSlot, at: SampleClock, note: MidiNote, velocity: Velocity }
```

Section 8.3 states that neither record names `MidiPortId`, and that a `size_of` test in this crate
asserts both sizes.

### `duet_command::view` (section 10.2, section 1.6)

```rust
/// The four product modes. It carries `Ord` because `ViewState` keys a
/// `BTreeMap` by it (VR1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Mode { Compose, Record, Mix, Master }

/// A length in logical pixels. It is a number, not a GPUI type, so it
/// crosses the transport and lands in a text file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LogicalPx(Finite);

/// One zoom step of the wrapped timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ZoomStep(u8);

impl ZoomStep {
    /// Build a step at compile time, for the constants of section 1.6.
    pub const fn new(step: u8) -> Self;

    /// Build a step from a value this process did not write.
    #[must_use]
    pub fn from_step(step: u8) -> Option<Self>;

    /// The staff space this step sets, in logical pixels (B137).
    #[must_use]
    pub fn staff_space(self) -> Finite;

    /// The samples-per-pixel scale this step gives a linear timeline.
    #[must_use]
    pub fn scalar(self) -> ZoomScalar;
}

/// The samples-per-pixel scale of a linear timeline.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct ZoomScalar(NonZeroU32);

/// A scroll position, in logical pixels from the top of the content.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScrollOffset(LogicalPx);

impl ScrollOffset {
    /// Build an offset at compile time, for the constants of section 1.6.
    pub const fn new(value: LogicalPx) -> Self;
}

/// Which tracks a mode shows. PR R-03 makes the choice a MUST.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackFilter { All, Part(PartId), Tracks(Vec<TrackId>) }

/// The window position and size, in logical pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct WindowGeometry { x: LogicalPx, y: LogicalPx, width: LogicalPx, height: LogicalPx }

/// What one mode's work area remembers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModeView {
    sidebar_width: LogicalPx,
    inspector_width: LogicalPx,
    /// The Mix work area holds a second, vertical split (contract 1.6).
    mix_vertical_split: Finite,
    zoom: ZoomStep,
    scroll: ScrollOffset,
    selection: Selection,
    shown_tracks: TrackFilter,
}

/// Everything about the project's presentation that survives a restart.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewState {
    /// The mode the project reopens in.
    mode: Mode,
    /// One entry per mode the user has visited.
    per_mode: BTreeMap<Mode, ModeView>,
    window: WindowGeometry,
}
```

`LogicalPx` holds one `Finite`, so its derived `Deserialize` reaches `Finite::try_from` and inherits
both invariants (section 2.6a). `LogicalPx::new` is a `const fn` for the section 1.6 constants, and
`LogicalPx::to_f32` calls `duet_time::convert::finite_to_f32_saturating` (section 2.3).

The `duet-command` block of section 1.6 lives in `view.rs`.

```rust
/// The current schema number of `view.json` (B84).
pub const VIEW_SCHEMA: SchemaVersion = SchemaVersion::new(1);
/// The first-open window width (B91).
pub const FIRST_OPEN_WIDTH: LogicalPx = LogicalPx::new(Finite::from_finite_const(1_440.0));
/// The first-open window height (B92).
pub const FIRST_OPEN_HEIGHT: LogicalPx = LogicalPx::new(Finite::from_finite_const(900.0));
/// The minimum window width (B102).
pub const MIN_WINDOW_WIDTH: LogicalPx = LogicalPx::new(Finite::from_finite_const(1_024.0));
/// The minimum window height (B103).
pub const MIN_WINDOW_HEIGHT: LogicalPx = LogicalPx::new(Finite::from_finite_const(700.0));
/// The width below which the window refuses to shrink further (B104).
pub const HARD_MIN_WINDOW_WIDTH: LogicalPx = LogicalPx::new(Finite::from_finite_const(900.0));
/// The first-open window origin, on both axes (B93).
pub const FIRST_OPEN_ORIGIN: LogicalPx = LogicalPx::new(Finite::from_finite_const(120.0));
/// The first-open sidebar width (B94).
pub const FIRST_OPEN_SIDEBAR: LogicalPx = LogicalPx::new(Finite::from_finite_const(240.0));
/// The first-open inspector width (B97).
pub const FIRST_OPEN_INSPECTOR: LogicalPx = LogicalPx::new(Finite::from_finite_const(300.0));
/// The first-open Mix vertical split (B98).
pub const FIRST_OPEN_MIX_SPLIT: Finite = Finite::from_finite_const(0.45);
/// The staff space each `ZoomStep` sets, in logical pixels (B137).
pub const ZOOM_STAFF_SPACE: [Finite; 6] = [
    Finite::from_finite_const(6.0),
    Finite::from_finite_const(7.0),
    Finite::from_finite_const(8.0),
    Finite::from_finite_const(10.0),
    Finite::from_finite_const(12.0),
    Finite::from_finite_const(16.0),
];
/// The first-open zoom step (B99).
pub const FIRST_OPEN_ZOOM: ZoomStep = ZoomStep::new(2);
/// The first-open scroll offset: the top of the content.
pub const FIRST_OPEN_SCROLL: ScrollOffset = ScrollOffset::new(LogicalPx::new(Finite::ZERO));
```

`ZoomStep::new` clamps to the last index of `ZOOM_STAFF_SPACE`, and `ZoomStep::from_step` returns
`None` at or above that length (section 10.2).

### `duet_command::document` (section 1.8, section 10.2)

```rust
/// A document the bundle stores as one or more canonical text files.
pub trait BundleDocument: Sized {
    /// The paths this document owns, relative to the bundle root, in a
    /// fixed order. The order is the order of `to_bytes` and `from_bytes`.
    fn paths() -> &'static [&'static str];

    /// Serialize into one byte block per path, in `paths` order.
    ///
    /// # Errors
    /// Returns `CommandError::Serialize` when a value has no canonical form,
    /// which today means only a non-finite float that escaped `Finite`.
    fn to_bytes(&self) -> Result<Vec<Vec<u8>>, CommandError>;

    /// Parse from one byte block per path, in `paths` order.
    ///
    /// # Errors
    /// Returns `CommandError::Parse` for malformed text, and
    /// `CommandError::Schema` for a version this build cannot migrate.
    fn from_bytes(blocks: &[Vec<u8>]) -> Result<Self, CommandError>;

    /// Unknown keys the reader kept in an `extra` bag (section 3.6).
    fn warnings(&self) -> &[ImportWarning];

    /// Whether git tracks this document's paths.
    fn tracked() -> bool;
}

/// One unknown key a reader kept, with the path that carried it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportWarning { path: Box<str>, key: Box<str>, detail: Box<str> }

/// The `view.json` document. It is the one type that `BundleDocument`
/// carries for the view, and `duet-project` serializes it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewDocument { schema: SchemaVersion, view: ViewState }
```

`duet-command` implements `BundleDocument` for `Score`, `Session`, `MixState`, and `ViewState`,
which the orphan rule allows because the trait is local (section 1.8). `Score`, `Session`, and
`MixState` return `true` from `tracked`. `ViewState` returns `false`, because `state/view.json` holds
the viewport and a scroll must not churn a commit (section 4.2).

### `duet_command::event` (section 9.5, section 15.5, section 3.5)

```rust
/// A job identifier. It is unique for one process run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct JobId(u64);

/// A document version counter. Every core document carries one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Version(u64);

/// What a job reports while it runs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running { done: u32, total: u32 },
    Done,
    Cancelled,
    Failed(GatewayError),
}

/// One undoable edit. The undo stack holds these, never a `Verb`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum EditCommand {
    Score(ScoreCommand),
    Session(SessionCommand),
}

/// One fact a client reads about a domain change.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DomainEvent { Score(ScoreEvent), Session(SessionEvent), Transport(TransportCommand), Job { job: JobId, state: JobState } }

/// What a client sends the core.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreInput {
    Call(Box<GatewayRequest>),
    FilesChanged(Vec<PathBuf>),
    SetEntryContext { caret: Position, duration: NoteValue },
    Resynchronize { client: u64 },
    Shutdown,
    /// One `JobRunner` worker saw its queue close and its loop has ended.
    WorkerStopped { worker: u16 },
    /// The `duet-quit-save` thread finished the atomic save of section 4.10
    /// for every dirty document and released the writer lock.
    QuitSaveDone { outcome: Result<(), Box<CommandError>> },
}

/// What the core sends a client.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CoreEvent {
    Changed { version: Version, events: Vec<DomainEvent> },
    JobProgress { job: JobId, state: JobState },
    EngineFault(EngineFault),
    EngineStateChanged(EngineState),
    ScoreChanged(Revision),
    PeaksReady(SourceHash),
    MissingSource { hash: SourceHash },
    DocumentReloaded { version: Version },
    ExternalEditRejected { path: PathBuf, error: Box<CommandError> },
    ExternalEditConflict { paths: Vec<PathBuf> },
    CheckoutRejected { commit: CommitId, error: Box<CommandError> },
    ViewStateReset { reason: Box<str> },
    UndoHorizonMoved { dropped: u32, oldest_name: Box<str> },
    EntryDropped { count: u32 },
    ClientDegraded { client: u64 },
    LockLost,
}
```

### `duet_command::snapshot` (section 15.5, section 9.1)

```rust
/// What the user interface reads about the transport, as plain data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportView {
    position: SuperClock,
    rolling: bool,
    recording: bool,
    count_in_remaining: FrameCount,
    preroll_remaining: FrameCount,
}

impl TransportView {
    /// The transport stopped at the start.
    #[must_use]
    pub const fn stopped() -> Self;
}

/// One transport intent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportCommand {
    Play, Stop, Locate(SuperClock), ArmRecord, DisarmRecord,
    SetLoop(Option<Span>), SetPunch(Option<Span>), SetCountIn(BarCount),
}

/// What a loudness measurement produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LoudnessReport { integrated_lufs: Finite, short_term_lufs: Finite, momentary_lufs: Finite, true_peak_dbtp: Finite, range_lu: Finite }

/// A whole-state copy for a client that resynchronized.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    version: Version,
    score: Box<CanonicalDocument>,
    session: Box<SessionDocument>,
    mix: Box<MixDocument>,
    view: Box<ViewState>,
    engine: EngineState,
}
```

### `duet_command::export` (section 7.4)

```rust
/// One declarative record that fully determines a render (Ardour 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportSpec {
    container: Container,
    sample_format: SampleFormat,
    sample_rate: SampleRate,
    dither: DitherKind,
    normalization: Normalization,
}

/// The output container.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Container { Wav, Rf64, Flac }

/// The output sample format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SampleFormat { Int16, Int24, Float32 }

/// Which dither a render applies before it narrows to an integer format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DitherKind { None, Rectangular, Triangular, Shaped }

/// How the render sets the level. Every level is a `Finite`, so a hand
/// edited file cannot carry a NaN into the limiter (VR4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Normalization {
    None,
    Peak { dbfs: Finite },
    Loudness { lufs: Finite, true_peak_dbtp: Finite },
}
```

### `duet_command::recent` (section 9.1)

```rust
/// One entry of the recent-project list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecentEntry { path: PathBuf, name: String, opened_at: UnixSeconds }
```

### `duet_command::request` (section 8.4, section 9.1, section 15.5)

```rust
/// A git object digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommitDigest { kind: HashKind, bytes: [u8; 32] }

/// Which digest a repository's object format uses.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum HashKind { Sha1, Sha256 }

/// A commit identifier. It wraps a git object digest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct CommitId(CommitDigest);

/// A git branch name, without its `refs/heads/` prefix. It is never empty.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BranchName(Box<str>);

/// One line of the history list the user reads.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitSummary { id: CommitId, message: Box<str>, author: Box<str>, at: UnixSeconds, parents: Vec<CommitId> }

/// What a client asks the gateway for, across the transport seam.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GatewayRequest { verb: Verb, client: u64 }

/// Create one project from a template.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CreateRequest {
    path: PathBuf,
    name: Box<str>,
    template: Box<str>,
    sample_rate: SampleRate,
    /// The first key of the score.
    key: KeySignature,
    /// The first time signature of the score.
    meter: Meter,
    /// The first tempo of the score.
    tempo: Tempo,
}

/// Import one file into the open project.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImportRequest { path: PathBuf, options: SmfImportOptions }

/// Export the score to one interchange file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportRequest { path: PathBuf, overwrite: bool }

/// Render the mix to one audio file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AudioExportRequest { path: PathBuf, spec: ExportSpec, overwrite: bool, part: Option<PartId> }

/// Rename one named element.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenameRequest { target: ElementRef, track: Option<TrackId>, take: Option<TakeId>, name: Box<str> }

/// Add one track to the session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackAddRequest { part: PartId, staff: StaffId, name: TrackName, channels: NonZeroU8 }

/// Set one track's input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct InputRequest { track: TrackId, input: InputSelection }

/// Add one bus to the mix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusAddRequest { name: StripName, role: BusRole }

/// Commit the tracked files.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitRequest { message: Box<str>, tag: Option<BranchName> }

/// Branch from one commit.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchRequest { name: BranchName, from: CommitId, checkout: bool }

/// Measure the master output without writing a file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeasureRequest { span: Option<Span>, normalization: Normalization }

/// Write one automation curve.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CurveWrite { parameter: ParamId, interpolation: Interpolation, points: CurvePoints }

/// Build a composite take from ranges across other takes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompRequest {
    track: TrackId,
    /// The new take receives one region per range, in order.
    ranges: Vec<CompRange>,
    name: TakeName,
}

/// One range taken from one source take.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CompRange { from_take: TakeId, start: Position, end: Position }

/// Which Standard MIDI File track feeds which part.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PartMapping { entries: Vec<(u16, PartId)>, create_missing: bool }

/// What an import does with the file's tempo map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TempoImport { Replace, Keep, Append }

/// How to map a Standard MIDI File onto the score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmfImportOptions {
    /// Snap every onset and duration to this value. `None` keeps the raw ticks.
    quantize: Option<NoteValue>,
    /// Which track goes to which part.
    part_map: PartMapping,
    /// Take the tempo map from the file, or keep the current one.
    tempo: TempoImport,
}
```

### `duet_command::verb` (section 9.1)

```rust
/// One intent a client sends the gateway.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Verb {
    ProjectCreate(Box<CreateRequest>),
    ProjectOpen { path: Box<PathBuf> },
    ProjectClose,
    ProjectConvertRate { rate: SampleRate },
    Status,
    Gc { purge: Box<[SourceHash]> },
    RecentList,
    RecentAdd { path: Box<PathBuf> },
    JobStatus { job: JobId },
    JobCancel { job: JobId },
    TransportPlay,
    TransportStop,
    TransportLocate { position: Position },
    TransportSetLoop { span: Option<Span> },
    TransportSetPunch { span: Option<Span> },
    TransportSetCountIn { bars: BarCount },
    TransportArmRecord,
    TransportDisarmRecord,
    TransportSetTempo { at: Position, tempo: Tempo },
    TransportCalibrate { device: DeviceKey },
    ScoreApply(Box<ScoreCommand>),
    ScoreCopy(Box<Selection>),
    ScoreQuery(Box<ScoreSelector>),
    ScoreImportMidi(Box<ImportRequest>),
    ScoreImportMusicXml(Box<ImportRequest>),
    ScoreExportMusicXml(Box<ExportRequest>),
    TakeList { track: TrackId },
    TakeSelect { track: TrackId, take: TakeId },
    TakeRename(Box<RenameRequest>),
    TakeSplit { take: TakeId, at: Position },
    TakeTrim { region: RegionId, start: Delta, length: Delta },
    TakeComp(Box<CompRequest>),
    TakeSetRecordMode { track: TrackId, mode: RecordMode },
    TakeSetAlign { track: TrackId, align: AlignChoice },
    TakeDelete { take: TakeId },
    TrackAdd(Box<TrackAddRequest>),
    TrackRename(Box<RenameRequest>),
    TrackReorder { track: TrackId, before: Option<TrackId> },
    TrackRemove { track: TrackId },
    TrackSetInput(Box<InputRequest>),
    TrackSetMonitor { track: TrackId, monitor: MonitorMode },
    TrackSetArmed { track: TrackId, armed: bool },
    MixSetParam { parameter: ParamId, value: Finite },
    MixSetMute { strip: StripId, mute: bool },
    MixSetSolo { strip: StripId, solo: bool },
    MixClearSolo,
    MixAddSend { from: StripId, to: StripId, tap: SendTap },
    MixAddBus(Box<BusAddRequest>),
    MixInsertSlot { strip: StripId, at: SlotPosition, kind: SlotKind },
    MixRemoveSlot { strip: StripId, slot: SlotId },
    MixWriteCurve(Box<CurveWrite>),
    /// Clear the held over mark. `None` clears every strip. PR M-05.
    MeterReset { strip: Option<StripId> },
    MasterMeasure(Box<MeasureRequest>),
    ViewSet(Box<ViewState>),
    ViewGet,
    HistoryCommit(Box<CommitRequest>),
    HistoryLog { limit: u32 },
    HistoryBranch(Box<BranchRequest>),
    HistoryCheckout { commit: CommitId },
    HistoryDiff { from: CommitId, to: CommitId },
    Undo,
    Redo,
    Save,
    ExportAudio(Box<AudioExportRequest>),
    Resynchronize,
}

/// How much a verb costs on the calling thread.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerbCost { Immediate, Deferred }

impl Verb {
    /// Whether the verb fits B5 on the calling thread.
    pub fn cost(&self) -> VerbCost;

    /// Whether a user started this verb, which reserves a job slot (B36).
    pub fn is_user_started(&self) -> bool;
}

/// The one interface every client calls.
pub trait Gateway {
    /// Apply one verb.
    ///
    /// # Errors
    /// Returns `GatewayError` when the verb is refused or the state is wrong.
    fn call(&mut self, verb: Verb) -> Result<VerbOutcome, GatewayError>;
}
```

### `duet_command::outcome` (section 9.1, section 15.5)

```rust
/// What a verb produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbOutcome {
    /// The verb changed state. The events carry the new version.
    Changed { version: Version, events: Vec<DomainEvent> },
    /// The verb answered a question.
    Data(Box<VerbData>),
    /// The verb started long work. Progress arrives as events, and
    /// `JobStatus` and `JobCancel` accept the identifier.
    Started { job: JobId },
}

/// What a query verb answered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum VerbData {
    Status { version: Version, takes: u32, engine: EngineState },
    Takes(Vec<TakeId>),
    Commits(Vec<CommitSummary>),
    Recents(Vec<RecentEntry>),
    View(Box<ViewState>),
    Job { job: JobId, state: JobState },
    Loudness(Box<LoudnessReport>),
    Clipboard(Box<Clipboard>),
}
```

### The plain-data assertion (section 3.5)

This block lives in `src/lib.rs`.

```rust
/// Fail the build if a vocabulary type stops being plain data.
const fn assert_plain_data<T: Serialize + DeserializeOwned + Send + 'static>() {}

const _: () = {
    assert_plain_data::<Verb>();
    assert_plain_data::<VerbOutcome>();
    assert_plain_data::<VerbData>();
    assert_plain_data::<DomainEvent>();
    assert_plain_data::<Snapshot>();
    assert_plain_data::<EditCommand>();
    assert_plain_data::<ViewState>();
    assert_plain_data::<ExportSpec>();
};
```

## Steps

1. Read `crates/bc_document/duet-command/lang_rust/Cargo.toml` and `crates/bc_document/duet-command/lang_rust/src/lib.rs`. Confirm that M3
   created both and that the manifest holds no `[dependencies]` section. Confirm that `duet-score`
   and `duet-session` export every type this chunk names. Report a discrepancy and stop if any one
   is false.
2. Add the dependency entries to `crates/bc_document/duet-command/lang_rust/Cargo.toml`. The section 1.2 row for
   `duet-command` names three third-party crates, and section 1.3 gives the three internal edges.

   ```toml
   [dependencies]
   duet-time = { workspace = true }
   duet-score = { workspace = true }
   duet-session = { workspace = true }
   serde = { workspace = true }
   serde_json = { workspace = true }
   thiserror = { workspace = true }
   ```

3. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains the six edges.
4. Create `src/error.rs` with `CommandError` and `GatewayError`, and `src/fault.rs` with
   `FailureSurface`, `FailureCode`, `UpstreamFailure`, `FaultCode`, `EngineFault`, `EngineState`,
   `ServerDetail`, and `DeviceLabel`. Add both `mod` lines. Run `cargo check -p duet-command`.
   Expected result: it succeeds.
5. Write the failing `size_of` test in a `#[cfg(test)] mod tests` block at the foot of `src/midi.rs`.
   It asserts the size of `MidiRecord` and of `NoteEntry` against the two rows of the section 1.9
   recorded-size block. Run `cargo nextest run -p duet-command -E 'test(record_size)' --no-tests=fail`.
   Expected result: the run fails, because `midi.rs` holds no declaration.
6. Create `src/midi.rs` with `PortSlot`, `MidiPortId`, `MidiPortInfo`, `MidiNote`, `Velocity`,
   `MidiMessage`, `BoundPort`, `MidiRecord`, and `NoteEntry`. Add the `mod` line. Run the same
   command. Expected result: the run passes. If a size differs from the recorded block, report the
   discrepancy and stop (SM0).
7. Create `src/view.rs` with `Mode`, `LogicalPx`, `ZoomStep`, `ZoomScalar`, `ScrollOffset`,
   `TrackFilter`, `WindowGeometry`, `ModeView`, `ViewState`, and the section 1.6 constant block. Add
   the `mod` line. Run `cargo check -p duet-command`. Expected result: it succeeds.
8. Create `src/export.rs`, `src/recent.rs`, and `src/snapshot.rs` with the types the section above
   names. Add the three `mod` lines. Run `cargo check -p duet-command`. Expected result: it
   succeeds.
9. Create `src/request.rs` with the digest types, the branch and commit types, `GatewayRequest`, the
   thirteen request records, `PartMapping`, `TempoImport`, and `SmfImportOptions`. Add the `mod`
   line. Run `cargo check -p duet-command`. Expected result: it succeeds.
10. Create `src/event.rs` with `JobId`, `Version`, `JobState`, `EditCommand`, `DomainEvent`,
    `CoreInput`, and `CoreEvent`. Add the `mod` line. Run `cargo check -p duet-command`. Expected
    result: it succeeds.
11. Write the failing `Verb::cost` test in a `#[cfg(test)] mod tests` block at the foot of
    `src/verb.rs`. It asserts `VerbCost::Deferred` for `ProjectOpen`, `ExportAudio`,
    `MasterMeasure`, `HistoryCheckout`, `Gc`, and `TransportCalibrate`, and `VerbCost::Immediate`
    for `Status`, `TransportPlay`, and `ViewGet`. Run
    `cargo nextest run -p duet-command -E 'test(verb_cost)' --no-tests=fail`. Expected result: the
    run fails.
12. Create `src/verb.rs` with `Verb`, `VerbCost`, the `Verb` impl, and the `Gateway` trait, then
    `src/outcome.rs` with `VerbOutcome` and `VerbData`. `cost` and `is_user_started` each name every
    arm, because `clippy::wildcard_enum_match_arm` is denied. Add both `mod` lines. Run the same
    command. Expected result: the run passes.
13. Write the failing `BundleDocument` tests in `crates/bc_document/duet-command/lang_rust/tests/vocabulary.rs`. They
    assert that `Score`, `Session`, and `MixState` return `true` from `tracked`, that `ViewState`
    returns `false`, that each `paths` list has the same length as its `to_bytes` output, and that
    `from_bytes` of `to_bytes` returns an equal value. Run
    `cargo nextest run -p duet-command --test vocabulary --no-tests=fail`. Expected result: the run
    fails.
14. Create `src/document.rs` with the `BundleDocument` trait, `ImportWarning`, `ViewDocument`, and
    the four impls. Add the `mod` line. Run the same command. Expected result: the run passes.
15. Add the plain-data assertion block to `src/lib.rs`. Run `cargo check -p duet-command`. Expected
    result: it succeeds, which proves that every one of the eight named types is `Serialize`,
    `DeserializeOwned`, `Send`, and `'static`.
16. Write the `ViewState` round-trip test and the `ZoomStep` refusal test in `tests/vocabulary.rs`.
    Run `cargo nextest run -p duet-command --no-tests=fail`. Expected result: every test passes.
17. Run `cargo build --workspace` again, then `git add` the write scope and `git commit`. The native
    hook runs `scripts/dod.sh`.

## Tests

Every assert carries a message. The unit tests live in a `#[cfg(test)] mod tests` block in the same
file. `tests/vocabulary.rs` is an integration target and wraps its tests in a
`#[cfg(test)] mod tests` block. Author guidance: the `test-author` skill.

| Test | What it asserts | Where it lives |
|---|---|---|
| `record_size_of_midi_record` | `size_of::<MidiRecord>()` equals the section 1.9 recorded-size row | `src/midi.rs` |
| `record_size_of_note_entry` | `size_of::<NoteEntry>()` equals the section 1.9 recorded-size row | `src/midi.rs` |
| `verb_cost_deferred_arms` | Six named verbs report `VerbCost::Deferred` | `src/verb.rs` |
| `verb_cost_immediate_arms` | Three named verbs report `VerbCost::Immediate` | `src/verb.rs` |
| `verb_is_user_started` | A user-started verb reserves a job slot and a core-started verb does not | `src/verb.rs` |
| `zoom_step_from_step_refuses_out_of_range` | `ZoomStep::from_step` returns `None` at or above the length of `ZOOM_STAFF_SPACE` | `src/view.rs` |
| `zoom_step_staff_space_matches_the_table` | `staff_space` returns the `ZOOM_STAFF_SPACE` entry of its own index | `src/view.rs` |
| `transport_view_stopped_is_at_zero` | `TransportView::stopped` reports position zero, not rolling, and not recording | `src/snapshot.rs` |
| `bundle_document_tracked_flags` | `Score`, `Session`, and `MixState` return `true`; `ViewState` returns `false` | `tests/vocabulary.rs` |
| `bundle_document_paths_match_blocks` | Each `paths` list has the same length as its `to_bytes` output | `tests/vocabulary.rs` |
| `bundle_document_round_trip` | `from_bytes` of `to_bytes` returns an equal value for all four documents | `tests/vocabulary.rs` |
| `view_state_json_round_trip` | A `ViewDocument` survives a `serde_json` round trip with an equal value | `tests/vocabulary.rs` |
| `view_document_refuses_a_newer_schema` | A `ViewDocument` above `VIEW_SCHEMA` gives `CommandError::Schema` | `tests/vocabulary.rs` |
| `gateway_error_json_round_trip` | Every `GatewayError` arm survives a `serde_json` round trip | `tests/vocabulary.rs` |

## Verification

```
cargo nextest run -p duet-command --no-tests=fail
cargo clippy -p duet-command --all-targets -- -D warnings
cargo doc -p duet-command --no-deps --document-private-items
```

Expected output: the first command reports every unit test and every test of `tests/vocabulary.rs`
as passed, and reports no filter miss. The second command prints no warning. The third command
prints no warning, which proves that every item carries its `///` or `//!` documentation.

Then commit on a branch named `chunk/t4-command`. The native git hook runs `scripts/dod.sh`, and the
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
