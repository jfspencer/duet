# Rust crate survey for Duet

Survey date: 2026-09-20. Versions and dates come from the crates.io API. "Maintained" means a release or a repository push after 2025-09-20. Plugin hosts are out of scope for this plan (operator decision, 2026-09-20: no plugins, no unsafe), so the plugin rows are kept only as a record.

## Audio device I/O

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| cpal | 0.18.2 | 2026-08-16 | Apache-2.0 | Yes | No duplex stream: input and output are two callbacks; the device can refuse `BufferSize::Fixed`. |
| pipewire | 0.10.1 | 2026-08-19 | MIT | Yes | Registry listener and stream `process` with `dequeue_buffer` are safe Rust for the caller (verified 2026-09-20, see `linux-macos-platform.md`); only the raw `pw_buffer` path and `deinit` are `unsafe`. The server can override the quantum. Duet reaches PipeWire through cpal's PipeWire host. |
| coreaudio-rs | 0.14.2 | 2026-04-29 | MIT/Apache-2.0 | Yes | macOS only; more control than cpal. |
| alsa | 0.12.1 | 2026-07-31 | Apache-2.0/MIT | Yes | Linux only; the app owns the thread, the period size, and xrun recovery. |
| jack | 0.13.5 | 2026-02-01 | MIT | Yes | The JACK server owns buffer size and sample rate. |
| libpulse-binding | 2.30.1 | 2025-04-19 | MIT OR Apache-2.0 | No | Not a real-time API; do not use for the engine. |

cpal gives a real-time callback with `StreamConfig { sample_rate, buffer_size: BufferSize::Fixed(n) }` on CoreAudio, ALSA, JACK, and PipeWire.

## MIDI device I/O and hot-plug

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| midir | 0.11.0 | 2026-04-18 | MIT | Yes | No portable connect or disconnect callback (issues #35, #78, #86, #191, #201 open). |
| coremidi | 0.9.2 | 2026-08-02 | MIT | Yes | macOS only; gives the CoreMIDI client notify callback. |
| coremidi-hotplug-notification | 0.1.4 | 2026-06-20 | MIT | Yes | Small helper; one maintainer. |
| alsa (seq) | 0.12.1 | 2026-07-31 | Apache-2.0/MIT | Yes | Linux only; subscribe to `System:Announce` for port add and remove. |

midir does not report connect or disconnect. The app polls the port list, or adds coremidi on macOS and the ALSA sequencer announce port on Linux.

## MIDI file, MusicXML, theory, notation

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| midly | 0.5.3 | 2023-01-01 | Unlicense | No | Stable and complete SMF read and write; no maintainer response to expect. |
| musicxml | 1.1.2 | 2024-11-05 | MIT | No | Full MusicXML 4 read and write; idle for 22 months. Operator accepts a fork or in-house code. |
| mxml | 0.0.2 | 2020-06-18 | MIT OR Apache-2.0 | No | Abandoned stub. |
| rust-music-theory | 0.4.0 | 2026-07-12 | MIT | Yes | Scales, chords, intervals only. |
| notation | 1.1.0 | 2025-08-17 | non-standard | No | cargo-deny rejects the licence. |
| smufl | 0.2.1 | 2023-03-25 | MIT | Yes (push 2026-02-01) | Reads SMuFL font metadata only; no layout engine. |
| verovio | 0.3.6 | 2026-07-08 | LGPL-3.0-or-later | Yes | C++ engraver binding; heavy build, dynamic link. Not chosen. |

## Plugin hosts (out of scope, record only)

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| clack-host | 0.2.0 | 2026-09-12 | MIT OR Apache-2.0 | Yes | Pre-1.0; the only safe-Rust host with a GUI extension. |
| vst3 (coupler-rs) | 0.3.0 | 2025-12-07 | MIT OR Apache-2.0 | Yes | Raw COM bindings; a host layer is all `unsafe`. |
| objc2-audio-toolbox | 0.3.2 | 2025-10-04 | Zlib OR Apache-2.0 OR MIT | Yes | Raw bindings; no safe host wrapper. |
| livi | 0.7.5 | 2024-11-12 | MIT | No | LV2 audio host, no UI, work in progress. |

The VST 3 SDK is MIT since SDK 3.8.0 (October 2025).

## Real-time safety

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| rtrb | 0.4.0 | 2026-08-17 | MIT OR Apache-2.0 | Yes | SPSC only; fresh major version. |
| ringbuf | 0.5.2 | 2026-09-13 | MIT OR Apache-2.0 | Yes | Larger API. Pick one of rtrb or ringbuf. |
| basedrop | 0.1.3 | 2025-10-29 | MIT/Apache-2.0 | Yes | The collector must run on a non-audio thread. |
| crossbeam | 0.8.5 | 2026-09-05 | MIT OR Apache-2.0 | Yes | Channels allocate; only `ArrayQueue` and atomics belong on the audio thread. |
| arc-swap | 1.9.2 | 2026-06-28 | MIT OR Apache-2.0 | Yes | `load` is lock-free; `store` drops the old value, so store only from the UI side. **Superseded 2026-09-21: `arc-swap` is a dependency of NO Duet crate.** ADR 0004 decision 12 and architecture section 5.8 replace it with `triple_buffer` on every path the audio thread reads, because an `arc-swap` reader can hold the last reference to a retired value and run its destructor inside the audio callback, and a `triple_buffer` reader borrows and never owns. The advice in this cell is therefore about a crate this plan does not use. |
| triple_buffer | 9.0.0 | 2026-02-22 | MPL-2.0 | Yes | MPL-2.0 accepted by the operator. |
| atomic_float | 1.1.0 | 2024-08-31 | Apache-2.0 OR MIT OR Unlicense | No | `AtomicU32` with `f32::to_bits` replaces it. |
| assert_no_alloc | 1.1.2 | 2021-08-03 | BSD-1-Clause | No | Test-only; rare licence. Prefer an in-house allocator guard. |

> **Correction, 2026-09-21 (specification revision 21, critic C19-W22).** SUPERSEDED. An in-house allocator guard needs `unsafe impl GlobalAlloc`, `unsafe_code` is denied workspace-wide, and no crate of this plan may relax it, so no allocator guard of any kind can exist in this repository. ADR 0004 and `architecture.md` section 5.7 state the three means that replace it: a written forbidden-call list, a soak test, and an external profiler run.


## DSP, decode, analysis

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| symphonia | 0.6.1 | 2026-08-13 | MPL-2.0 | Yes | Decode only. MPL accepted. |
| hound | 3.5.1 | 2023-09-25 | Apache-2.0 | Yes (push 2026-08-09) | No RF64; a WAV over 4 GiB fails. |
| bwavfile | 2.0.1 | 2023-06-03 | MIT | Yes (push 2026-05-03) | RF64 and BW64 read and write; one maintainer. |
| rubato | 5.0.0 | 2026-08-10 | MIT OR Apache-2.0 | Yes | New major version. |
| realfft | 3.5.0 | 2025-06-12 | MIT | Yes | None of note. |
| rustfft | 6.4.1 | 2025-09-18 | MIT OR Apache-2.0 | Yes | None of note. |
| dasp | 0.11.0 | 2020-05-29 | MIT OR Apache-2.0 | Commit only | No release in six years. |
| fundsp | 0.23.0 | 2026-01-07 | MIT OR Apache-2.0 | Yes | Graph build allocates; build on the UI thread. |
| ebur128 | 0.1.10 | 2024-10-26 | MIT | Yes (push 2025-12-13) | Port of libebur128; stable API. |
| flacenc | 0.5.1 | 2025-12-18 | Apache-2.0 | Yes | Pure-Rust FLAC encoder; 0.x API. |
| pyin, pitch-detection, pitch_detector | various | 2022 to 2024 | MIT | No | Idle. aubio is GPL-3.0 and excluded. |

No maintained pitch-detection crate exists. Operator decision: in-house pYIN over `rustfft`.

## Versioned project storage

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| gix | 0.87.1 | 2026-08-24 | MIT OR Apache-2.0 | Yes | 0.x with frequent breaking releases; pure Rust, no git binary. `Repository` has `write_blob`, `write_object`, `edit_tree`, `commit`, `find_tree`, `find_blob`. |
| git2 | 0.21.0 | 2026-05-18 | MIT OR Apache-2.0 | Yes | Links libgit2 through a C build. |

## Local agent protocol

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| rmcp | 3.4.0 | 2026-09-15 | Apache-2.0 | Yes | Needs tokio (operator accepts). Transports: stdio, streamable HTTP, `AsyncRwTransport` over any `AsyncRead + AsyncWrite`, so a `tokio::net::UnixStream` works. |
| jsonrpsee | 0.26.0 | 2026-05-27 | MIT | Yes | No stdio transport. |

## File watch

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| notify | 8.2.0 stable; 9.0.0-rc.5 | 2025-08-03; 2026-08-30 | CC0-1.0 | Yes | 9.0 is a release candidate. |
| notify-debouncer-full | 0.7.0 stable; 0.8.0-rc.2 | 2026-01-23; 2026-05-02 | MIT OR Apache-2.0 | Yes | 0.8 tracks notify 9; the pair must match. |

## Operator decisions recorded on 2026-09-20

1. Plugins are out of scope. `unsafe` stays denied workspace-wide.
2. MPL-2.0 and Apache-2.0 crates are accepted; the distribution carries the NOTICE text.
3. tokio is the async runtime beside the GPUI executor.
4. Pitch detection is in-house pYIN.
5. MusicXML is the interchange format and the first storage format. A canonical format of our own replaces it as storage if the application needs it. The storage format must be durable and extensible. A fork or in-house MusicXML code is acceptable.

> **Correction, 2026-09-21 (specification revision 21, critic C19-W22).** SUPERSEDED as a statement about the plan. ADR 0002 decision 1 makes the CANONICAL format the storage format from the first release, and MusicXML stays the interchange format alone (`architecture.md` section 3.6, section 3.7). The operator decision this line records is unchanged; the architecture exercised the option it granted.


## Dependencies the architecture adds (surveyed 2026-09-20)

| Crate | Version | Released | Licence | Maintained | Risk |
|---|---|---|---|---|---|
| quick-xml | 0.42.0 | 2026-08-22 | MIT | Yes | `#![forbid(unsafe_code)]` inside. Enable `serialize` for serde. RUSTSEC-2026-0194 and -0195 are patched at 0.41.0 and later. |
| blake3 | 1.8.7 | 2026-08-20 | CC0-1.0 OR Apache-2.0 OR Apache-2.0 WITH LLVM-exception | Yes | No caller unsafe. On x86 the build needs a C compiler unless the `pure` feature is on. |
| async-channel | 2.5.0 | 2025-07-06 | Apache-2.0 OR MIT | Yes (push 2026-07-13) | No caller unsafe. |
| futures | 0.3.34 | 2026-08-11 | MIT OR Apache-2.0 | Yes | No caller unsafe. |
| smallvec | 1.16.1 | 2026-09-11 | MIT OR Apache-2.0 | Yes | Pin the 1.x line; 2.0.0-beta.1 is a pre-release. Unsafe functions exist but the safe API covers normal use. All advisories patched. |
| arrayvec | 0.7.8 | 2026-07-02 | MIT OR Apache-2.0 | Yes | Unsafe functions exist but the safe API covers normal use. |
| tokio-util | 0.7.19 | 2026-07-21 | MIT | Yes | No default features; pin the feature list (`rt`, `io`, `codec` as needed). |
| Bravura font (not a crate) | bravura-1.482 | 2026-08-24 | OFL-1.1 | Yes | Ship `LICENSE.txt` next to the font file. Never rename a modified font "Bravura". |

No crate above needs a `deny.toml` licence entry beyond MIT, Apache-2.0, MPL-2.0, CC0-1.0, and Unlicense. The font licence goes in the NOTICE file, not in `deny.toml`.
6. Linux audio targets PipeWire (latest) as the server. ALSA APIs are allowed because PipeWire provides them; no PulseAudio or JACK client code; no deprecated API on any platform; no legacy support. Ubuntu 26.04 is the minimum Linux release.
7. macOS 26 is the minimum macOS release; no support for an earlier macOS.
