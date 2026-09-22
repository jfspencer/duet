# Platform facts and decisions: Linux and macOS (verified 2026-09-20)

## Operator decisions
- Linux: PipeWire is the audio server. ALSA APIs are allowed because PipeWire provides them. No PulseAudio or JACK client code. No deprecated API. No legacy support. Ubuntu 26.04 is the minimum release.
- macOS: macOS 26 is the minimum release. No earlier macOS.

## Verified facts (sources: crates.io tarballs, GitHub, docs.rs, Launchpad, Ubuntu release notes)

### cpal 0.18.2
- Cargo features: `default = []`, `pipewire = ["dep:pipewire"]` (pipewire 0.10 with `v0_3_53`), `jack` and `pulseaudio` optional. The `alsa` 0.11 dependency on Linux is NOT optional: `libasound` is always linked. README: "ALSA is needed even when using JACK, PipeWire, or PulseAudio." PipeWire host needs `libpipewire-0.3-dev`; minimum PipeWire 0.3.53.
- Consumer line: `cpal = { version = "0.18.2", default-features = false, features = ["pipewire"] }`. Links libpipewire and libasound only.
- `default_host()` on Linux tries PipeWire (socket present), then PulseAudio, then ALSA. The app must select the PipeWire host by id explicitly and never call `default_host()` on Linux.
- PipeWire host: `BufferSize::Fixed(n)` is validated against the server quantum range and set as `node.latency`, else `UnsupportedConfig`; `Default` uses the current quantum. One sample rate, the graph rate. Xruns arrive as `ErrorKind::Xrun` (0.18.2). Latency from `pw_time.delay` as `capture` and `playback` timestamps.
- No duplex API in 0.18.2 (input and output are two streams). Duplex API interface merged to `develop` 2026-07-05 (PR 1229); PipeWire duplex PR 1272 is a draft.
- Fixed after 0.18.2 (not in the release): socket detection without env vars (issues 1365, PRs 1367 and 1371), `node.rate` property (PR 1341).
- ALSA host over the pipewire-alsa plugin: cpal source documents unreliable period sizes (issues 1029, 1036) and continuous xruns (issue 1250). Not the primary path.

### pipewire 0.10.1 (pipewire-rs)
- docs.rs build fails; canonical docs at pipewire.pages.freedesktop.org. Registry listener (`ListenerLocalBuilder::global`, `global_remove`, `register`) is safe Rust: node and port hot-plug needs no `unsafe` in the caller. Stream `process` with `dequeue_buffer`, `datas_mut`, `Data::data` is safe; only the raw `pw_buffer` path is `unsafe` and not needed. `pw::deinit()` is `unsafe` and optional.

### alsa 0.12.1
- `Seq`, `Addr::system_announce()`, `subscribe_port`, `event_input`, `EventType::{ClientStart, ClientExit, PortStart, PortExit}` are safe and call no deprecated alsa-lib function. alsa-lib `seq.h` carries zero deprecated markers. cpal pins `alsa = "0.11"`, so an app pin of 0.12.1 puts two `alsa` versions in the tree; check `cargo deny` `multiple-versions` before the pin.

### GitHub Actions runners
- `ubuntu-26.04` and `ubuntu-26.04-arm` exist (preview since 2026-06-11). Image 20260907: Ubuntu 26.04.1, kernel 7.0. No PipeWire daemon or libs preinstalled; a job installs `pipewire libpipewire-0.3-dev libasound2-dev` and starts a user `pipewire` process.
- `macos-26` (arm64, also `macos-latest`) and `macos-26-intel` (x64) exist; macOS 26.6, Xcode 26.6 default.

### Ubuntu 26.04
- PipeWire 1.6.2-1ubuntu1.2 in the archive (satisfies the 0.3.53 floor).
- The desktop session runs only on Wayland; X11 apps run through XWayland.

## Orchestrator decisions derived from the facts
1. Audio backend: cpal 0.18.2 with `default-features = false, features = ["pipewire"]`. On Linux the backend selects the PipeWire host by id. If no PipeWire socket exists, the engine reports a fault that names PipeWire; it never falls back to the ALSA host. On macOS the CoreAudio host.
2. Linux MIDI hot-plug: the ALSA sequencer announce port through `alsa` 0.12.1, OR the PipeWire registry listener; the Architect picks the one that avoids a duplicate `alsa` version, and states the `cargo deny` result. The MIDI byte stream stays on midir.
3. CI: the gate runs on `ubuntu-26.04` and `macos-26` with the dummy backend, no daemon needed. A separate `audio-smoke` job on `ubuntu-26.04` installs PipeWire, starts a user daemon, and opens one PipeWire stream through the real backend.
4. Display: Linux targets Wayland; XWayland is not a target. If gpui-kit lets the consumer disable the `x11` feature, disable it; otherwise keep the default and state that X11 is untested.
5. macOS: `LSMinimumSystemVersion` 26.0 in the bundle and `MACOSX_DEPLOYMENT_TARGET=26.0` in the build configuration.
