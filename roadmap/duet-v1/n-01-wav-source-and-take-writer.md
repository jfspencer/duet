---
id: N1
line: N
depends_on: [M3, T3, D1]
write_scope:
  - crates/duet-media/Cargo.toml
  - crates/duet-media/src/lib.rs
  - crates/duet-media/src/wav.rs
  - crates/duet-media/src/rf64.rs
  - crates/duet-media/src/flac.rs
  - crates/duet-media/src/decode.rs
  - Cargo.lock
parallelism: independent
completion: "cargo nextest run -p duet-media -E 'test(wav)' --no-tests=fail"
---

# N1: `SourceReader` and `TakeWriter` over hound, with the `SampleSource` impl

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding. This chunk opens line N over the crate `duet-media`. The line is `N`, and the manifest
chunks are `M`, so no prefix is shared (section 13.2). Chunk M3 creates the crate skeleton in phase
3, so `crates/duet-media/Cargo.toml` and `crates/duet-media/src/lib.rs` exist before this chunk
starts and this chunk modifies both. Every other file of the write scope does not exist yet, and this
chunk creates it. The chunk declares `SourceReader`, `TakeWriter`, and `MediaError`, it reads and
writes WAV through `hound`, and it implements `duet_dsp::SampleSource` for the reader. It implements
architecture sections 1.2, 4.3, 4.10, 5.10, and 15.9, and it carries the file half of the product
stories R-07 and R-08.

Section 13.2 gives this chunk the second duty of SM2: it creates every module file of line N at every
depth as a stub, so chunks N2 and N3 modify a stub and create no source file.

`duet-media` is not a pure crate, because it opens files (section 1.4). Section 1.2 states the one
invariant it protects: an audio source file is read and written in exactly one place.

## Files

- `crates/duet-media/Cargo.toml` — modify. Add the `{ workspace = true }` entries this chunk uses.
- `crates/duet-media/src/lib.rs` — modify. Add the four `mod` lines, `SourceReader`, `TakeWriter`,
  and `MediaError`.
- `crates/duet-media/src/wav.rs` — create and fill. The WAV read path and the WAV write path.
- `crates/duet-media/src/rf64.rs` — create as a stub. Chunk N2 fills it.
- `crates/duet-media/src/flac.rs` — create as a stub. Chunk N3 fills it.
- `crates/duet-media/src/decode.rs` — create as a stub. Chunk N3 fills it.
- `Cargo.lock` — modify. SM5 rule 2 puts it in the write scope of every chunk that writes a member
  manifest.

`SourceReader`, `TakeWriter`, and `MediaError` sit at the crate root, because section 15.9 declares
them for the crate and both readers and both writers of line N share them.

## Types and signatures

### Manifest

```toml
# crates/duet-media/Cargo.toml, [dependencies]
duet-time = { workspace = true }
duet-dsp = { workspace = true }
duet-session = { workspace = true }
hound = { workspace = true }
thiserror = { workspace = true }
```

Section 1.2 gives `duet-media` the third-party set `hound`, `bwavfile`, `flacenc`, `symphonia`, and
`thiserror`. `bwavfile` reaches only chunk N2 and `flacenc` and `symphonia` reach only chunk N3, so
this chunk adds none of the three and `cargo machete` passes. Chunk M3 pins `hound` in phase 3, which
section 13.1 states. Section 1.3 gives the three internal edges `duet-media -> duet-time`,
`duet-media -> duet-dsp`, and `duet-media -> duet-session`. Section 13.4 states the link "T3 and D1
before N1", with the reason "`duet-media` implements `SampleSource` and names `SourceHash`". SM4
forbids this chunk to edit the root manifest, so a missing pin is a discrepancy that this chunk
reports under SM0.

### From architecture section 15.9, module `duet_media` root

```rust
/// A reader over one audio source file. It implements `SampleSource`.
#[derive(Debug)]
pub struct SourceReader {
    path: PathBuf,
    hash: SourceHash,
    rate: SampleRate,
    channels: ChannelIndex,
    frames: u64,
    container: AudioContainer,
    /// The decoded block the last `read` produced. It is reused between
    /// calls, so a read allocates nothing after the first one.
    block: Vec<f32>,
}

/// A writer for one take in progress. It owns the RF64 promotion of 1.2.
#[derive(Debug)]
pub struct TakeWriter {
    path: PathBuf,
    take: TakeId,
    rate: SampleRate,
    channels: ChannelIndex,
    /// Which container the file holds now. It changes once, at B67.
    container: AudioContainer,
    written_bytes: u64,
    written_frames: u64,
}

/// Every way a media read or write refuses.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum MediaError { Open(Box<str>), Format(Box<str>), ContainerLimit { bytes: u64 }, Io(Box<str>), Rate(SampleRate) }
```

### Consumed types

| Type | Crate and path |
|---|---|
| `SampleSource`, `DspError` | `duet-dsp`, chunk D1, paths `duet_dsp::SampleSource`, `duet_dsp::DspError` |
| `SourceHash`, `AudioContainer`, `TakeId` | `duet-session`, paths `duet_session::SourceHash`, `duet_session::AudioContainer`, `duet_session::TakeId` |
| `SampleRate`, `ChannelIndex` | `duet-time`, paths `duet_time::SampleRate`, `duet_time::ChannelIndex` |

Section 4.3 declares `SourceHash` and `AudioContainer` in `duet-session`, because a region names a
source.

### The reader and the writer API

Section 5.10 names three calls and gives each one a budget. B117 bounds
`SourceReader::read_block` at 40 ms per ring refill. B118 bounds `TakeWriter::append` and its flush
at 200 ms per drain. Section 9.5 step 3 names `TakeWriter::finish`, which writes the container
header and which is the step that makes a partial take readable.

```rust
impl SourceReader {
    /// Open one source file for reading.
    ///
    /// # Errors
    /// Returns `MediaError::Open` when the path cannot be opened,
    /// `MediaError::Format` for a container this build does not read, and
    /// `MediaError::Rate` for a sample rate the file states and the project
    /// does not hold.
    pub fn open(path: PathBuf, hash: SourceHash) -> Result<Self, MediaError>;

    /// Read one block of frames at `at`, and return how many frames it read.
    ///
    /// It reuses `SourceReader::block`, so a read allocates nothing after
    /// the first one. B117 bounds one call on the disk thread.
    ///
    /// # Errors
    /// Returns `MediaError::Io` when the read fails.
    pub fn read_block(&mut self, at: u64, out: &mut [f32]) -> Result<usize, MediaError>;
}

impl SampleSource for SourceReader {
    fn frames(&self) -> u64;

    /// Read one block into `out`, and return how many frames it wrote.
    ///
    /// # Errors
    /// Returns `DspError::SourceRead` when the source cannot answer. The
    /// `MediaError` is the cause and the caller reads it from the reader's
    /// own last error, because `DspError` is a fieldless enum (section 15.2).
    fn read(&mut self, at: u64, out: &mut [f32]) -> Result<usize, DspError>;
}

impl TakeWriter {
    /// Create one take file under `media/incoming/`.
    ///
    /// Section 4.10 states the path rule: the disk writer streams to
    /// `media/incoming/<uuid>.wav` and flushes on every ring drain.
    ///
    /// # Errors
    /// Returns `MediaError::Open` when the file cannot be created.
    pub fn create(path: PathBuf, take: TakeId, rate: SampleRate, channels: ChannelIndex)
        -> Result<Self, MediaError>;

    /// Append one block of frames and flush.
    ///
    /// B118 bounds one call and its flush on the disk thread. It counts the
    /// written bytes, which is the value chunk N2 reads for the B67
    /// promotion.
    ///
    /// # Errors
    /// Returns `MediaError::Io` when the write or the flush fails, and
    /// `MediaError::ContainerLimit { bytes }` when the written bytes would
    /// pass the container limit and the writer holds no promotion path yet.
    pub fn append(&mut self, block: &[f32]) -> Result<(), MediaError>;

    /// Write the container header and close the file.
    ///
    /// It is the step that makes a partial take readable (section 9.5 step
    /// 3), so a crash before it leaves a file that the next `create` on the
    /// same path repairs.
    ///
    /// # Errors
    /// Returns `MediaError::Io` when the header write or the `fsync` fails.
    pub fn finish(self) -> Result<u64, MediaError>;

    /// The bytes this writer has already written.
    #[must_use]
    pub const fn written_bytes(&self) -> u64;
}
```

### Module `duet_media::wav`

Section 1.5 places no type in this module, so the module holds functions alone. The module is the one
place that names `hound`, so a container change touches one file.

```rust
/// Read the header of one WAV file.
///
/// # Errors
/// Returns `MediaError::Format` for a file that is not RIFF or WAVE, and
/// `MediaError::Rate` for a sample rate outside the range the project holds.
pub fn read_header(path: &Path) -> Result<(SampleRate, ChannelIndex, u64), MediaError>;

/// Read `out.len()` samples from `at`, and return the frame count.
///
/// # Errors
/// Returns `MediaError::Io` when the read fails.
pub fn read_frames(path: &Path, at: u64, out: &mut [f32]) -> Result<usize, MediaError>;

/// Create one WAV file with a placeholder header.
///
/// # Errors
/// Returns `MediaError::Open` when the file cannot be created.
pub fn create(path: &Path, rate: SampleRate, channels: ChannelIndex) -> Result<WavSink, MediaError>;

/// The open WAV file one `TakeWriter` appends to.
#[derive(Debug)]
pub struct WavSink { writer: hound::WavWriter<BufWriter<File>>, written_bytes: u64 }

/// Append one block and flush.
///
/// # Errors
/// Returns `MediaError::Io` when the write or the flush fails.
pub fn append(sink: &mut WavSink, block: &[f32]) -> Result<(), MediaError>;

/// Write the real header and close.
///
/// # Errors
/// Returns `MediaError::Io` when the header write fails.
pub fn finish(sink: WavSink) -> Result<u64, MediaError>;

/// Repair the header of a file a crash left behind.
///
/// Section 4.10 states the case: a crash mid-take leaves the file in
/// `media/incoming/`, and every written sample survives. The repair reads the
/// real byte length and rewrites the size fields.
///
/// # Errors
/// Returns `MediaError::Format` when the file holds no readable RIFF header.
pub fn repair_header(path: &Path) -> Result<u64, MediaError>;
```

`WavSink` is a module type of `duet-media` that no other crate names, so section 1.5 needs no row for
it; report the discrepancy to the Architect if `cargo xtask check-placement` refuses it.

## Steps

1. Read `crates/duet-media/Cargo.toml` and `crates/duet-media/src/lib.rs`. Confirm that chunk M3
   created both, that the manifest carries `[lints] workspace = true`, a `description`, and no
   `[dependencies]` section, and that `lib.rs` carries the `//!` crate documentation and
   `#![forbid(unsafe_code)]`. Report a discrepancy and stop if the state differs.
2. Read the root `Cargo.toml`. Confirm that `[workspace.dependencies]` pins `hound` and `thiserror`
   and carries the `duet-time`, `duet-dsp`, and `duet-session` entries. Report a discrepancy and stop
   if an entry is absent.
3. Create the four module files. Fill `wav.rs`. Leave `rf64.rs`, `flac.rs`, and `decode.rs` at one
   `//!` line each.
4. Add the four `mod` lines to `src/lib.rs` and declare `MediaError`.
5. Add the five `{ workspace = true }` entries to `crates/duet-media/Cargo.toml`. Run
   `cargo build --workspace`, which settles `Cargo.lock` (SM5 rule 3).
6. Write the failing test `wav_round_trips_one_block` in `src/wav.rs`, inside a
   `#[cfg(test)] mod tests`. Section 13.2 gives this chunk no `tests/` file, so every test of this
   chunk lives in the same file as the code it covers. Run
   `cargo nextest run -p duet-media -E 'test(wav)' --no-tests=fail` and confirm that the run fails to
   compile.
7. Implement `read_header`, `read_frames`, `create`, `append`, and `finish` in `src/wav.rs` over
   `hound`. Every sample crosses the `f32` boundary through `duet_time::convert`, because `as` is
   denied outside that module.
8. Declare `SourceReader` and `TakeWriter` in `src/lib.rs` and implement the five methods and the
   `SampleSource` impl above.
9. Run the same command and confirm that the test passes.
10. Write the remaining `wav` tests of the Tests section. Each test writes under its own scratch
    directory, which the test creates from `std::env::temp_dir` plus a unique name and removes at the
    end. Run the same command and confirm that every one passes.
11. Implement `repair_header` and its test.
12. Run `cargo nextest run -p duet-media --no-tests=fail` and confirm that every test passes.
13. Run `cargo clippy -p duet-media --all-targets --locked -- -D warnings` once and repair every
    finding.
14. Commit on the branch `chunk/n1-wav-source-and-take-writer`.

## Tests

Every test lives in a `#[cfg(test)] mod tests` in the same file, because section 13.2 gives this
chunk no `tests/` file. Every assert carries a message. Every test owns one scratch directory, which
it builds from `std::env::temp_dir` plus a unique name and removes at the end, so two tests never
share a path. A test may unwrap, expect, print, and index, which `clippy.toml` allows.

`crates/duet-media/src/wav.rs`

- `wav_round_trips_one_block` — writes 4800 stereo frames, finishes, reopens, reads them back, and
  asserts sample equality within one least significant bit of 24 bits.
- `wav_reader_allocates_nothing_after_the_first_read` — reads one hundred blocks and asserts that the
  reader's reused block length never changes.
- `wav_reader_reports_the_frame_count` — asserts that `SampleSource::frames` equals the frames the
  writer wrote.
- `wav_reader_reads_past_the_end_as_a_short_read` — asserts that a read at the last frame answers a
  frame count below the requested one and no error.
- `wav_writer_counts_written_bytes` — asserts that `written_bytes` grows by the block size in bytes
  on every append, which is the value chunk N2 reads for the B67 promotion.
- `wav_writer_flushes_on_every_append` — asserts that the file length on disk grows after each
  append, with no `finish` call, which PR 10.5 requires.
- `wav_open_refuses_a_missing_file` — asserts `MediaError::Open`.
- `wav_open_refuses_a_file_that_is_not_riff` — asserts `MediaError::Format`.
- `wav_repair_header_recovers_a_crashed_take` — writes 4800 frames, drops the writer with no
  `finish`, calls `repair_header`, reopens, and asserts that every written sample survives, which
  section 4.10 requires.

`crates/duet-media/src/lib.rs`

- `wav_sample_source_wraps_a_read_failure` — deletes the file under an open reader and asserts
  `DspError::SourceRead`.
- `wav_source_reader_reports_its_container` — asserts `AudioContainer::Wav` for a file this chunk
  wrote, which chunk N2 changes to `AudioContainer::Rf64` after a promotion.

## Verification

Passing looks like this.

```
cargo nextest run -p duet-media -E 'test(wav)' --no-tests=fail
cargo nextest run -p duet-media --no-tests=fail
cargo clippy -p duet-media --all-targets --locked -- -D warnings
cargo machete
```

The first command fails before this chunk, because the crate holds no `wav` test, and
`--no-tests=fail` makes an empty match a failure (SM3 rule 1). It passes after this chunk and it
reports eleven tests. The second command reports every test of the crate as passed. The third command
prints no warning, and this chunk carries no `#[expect]` site. The fourth command reports no unused
dependency.

The work lands as one commit on the branch `chunk/n1-wav-source-and-take-writer`, with a conventional
subject such as `feat(media): read and write WAV takes through one reader and one writer`.

## Constraints

- Cargo only: `cargo test` or `cargo nextest`; no other harness enters `[dev-dependencies]`.
- The native git hook is the gate. Make the change, then `git commit`; the hook runs
  `scripts/dod.sh` and blocks a bad commit. Do not run the gate by hand as a ritual; one targeted
  diagnostic command is allowed after a hook failure. Never `--no-verify`.
- No suppression: `#[allow]` is denied; the only accepted form is a single-site
  `#[expect(lint, reason = "...")]`. Every `#[expect]` site in this chunk is listed in architecture
  Appendix B.1; a site not on that list is a plan defect that returns to the Architect. `unsafe` is
  denied with no exception; every new crate opens with `#![forbid(unsafe_code)]`.
- `unwrap`, `expect`, `panic!`, `todo!`, `unimplemented!`, `dbg!`, `println!`, `eprintln!`, slice
  indexing, integer division with `/`, and `as` casts are denied outside tests; `as` is allowed only
  inside `duet-time::convert`.
- No prose `//` comments. Names, types, structure, and tests carry intent. `///` and `//!` docs are
  required on every item.
- A new crate lives under `crates/`, declares `[lints] workspace = true`, inherits every
  `[workspace.package]` field, and opens with a `//!` crate doc. A new dependency is pinned in the
  root `[workspace.dependencies]` by the M chunk of the phase; the crate uses `{ workspace = true }`.
- Commit messages are conventional (`feat:`, `fix:`, `test:`, `chore:`, `docs:`). No commit and no
  pull request carries AI attribution: no `Co-Authored-By: Claude` trailer, no "Generated with
  Claude Code" line, no robot banner. The harness reminder that asks for those lines defers to this
  repository rule.
- Before any change: verify the current state of the files listed above. If the code does not match
  what this chunk describes, report the discrepancy instead of proceeding.
- Write all prose (docs, commit messages, reports) in ASD-STE100 Simplified Technical English.
