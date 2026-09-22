# Shared brief: Duet v1 plan genesis (2026-09-20)

## Product
Duet is a Rust desktop application on GPUI Kit (`gpui-kit` 0.6.4) for music composition, recording, mixing, and mastering of vocal performance. macOS and Linux. MIT licence. One project moves through four modes:
1. Compose: treble and bass staves. A MIDI keyboard connects with no setup and enters notes. A click on the staff adds a quarter note; a click with `w` held adds a whole note, `h` a half note, `e` an eighth note. A context menu edits a note. A top bar holds every notation option. The UI is smooth and animated.
2. Record: the interface shifts; one audio lane appears under each line of the score, so the user sees notes and audio together. The user picks the part (Soprano, Alto, Tenor, Bass); many tracks per part. Overwrite record, cut, splice. The user shows one track or many.
3. Mix: the notation collapses away. A full vocal-oriented mix tool set.
4. Master: modern mastering tools finish the project.
Cross-cutting: one unified timeline for notes (beats) and audio (samples); smart project structure and file handling; AI-first: a terminal AI agent (Claude Code, Codex) reads and edits the composition, imports MIDI files, and commits to a git-like history of the composition with checkout of earlier commits; MIDI devices are plug and play; Reactive Manifesto v2 properties (responsive, resilient, elastic, message driven) and strict domain-driven design.

## Operator decisions (final, do not reopen)
- Plugins (CLAP, VST3, AU, LV2) are OUT of scope. `unsafe` stays denied workspace-wide. Nothing in this plan may require unsafe code.
- MPL-2.0 and Apache-2.0 crates are accepted (NOTICE text ships with the app).
- tokio is the async runtime for network and agent I/O, beside GPUI's own executor.
- Pitch detection is in-house pYIN over rustfft.
- MusicXML is the interchange format and the first storage format; a canonical format of our own replaces it as storage if the application needs it. The storage format must be durable and extensible. A fork or in-house MusicXML code is acceptable.
- Ardour (`/Users/james/Developer/ardour`, GPL v2) is a concept reference only. Never copy code.

## Research (read these files first)
- `roadmap/duet-v1/research/gpui-kit-audit.md`: what GPUI Kit can and cannot draw, with source paths.
- `roadmap/duet-v1/research/ardour-concepts.md`: Ardour concept map with "adopt" and "reject" lists.
- `roadmap/duet-v1/research/crate-survey.md`: crate candidates with versions and licences.

## Repository state
`crates/duet` is a skeleton (one window, one root view). `tools/plan-db` and `tools/xtask` exist. Read `CLAUDE.md` for the lint policy (no unwrap, no panic, no indexing, no `as`, no prose `//` comments, every item documented), the test rules, and the Definition of Done. A new crate goes under `crates/` with `[lints] workspace = true`.

## Working hypothesis (the orchestrator's; challenge it with reasons)
Bounded context -> crate: `duet-time` (position with a domain tag beats|audio clock, integer beats at 1920 ticks, tempo map behind arc-swap), `duet-score` (score aggregate, commands, events, canonical form), `duet-engrave` (pure layout to glyph and path placements), `duet-session` (tracks, takes as playlists, regions over immutable sources, record modes layered|replace|sound-on-sound, punch, part assignment), `duet-engine` (transport, processor chain, graph, disk reader and writer, ring buffers, backend trait with a dummy backend first, cpal next, native CoreAudio and ALSA modules later), `duet-midi` (device I/O with hot-plug per platform, MIDI file import), `duet-mix`, `duet-master` (LUFS and true-peak, two-pass export), `duet-project` (bundle layout, serde state, git history through gix, file watch), `duet-analysis` (peaks, pYIN, loudness), `duet-agent` (MCP server over stdio and Unix socket, a flat verb list, headless mode), `crates/duet` (GPUI views and custom elements). The core runs headless; the GPUI app and the agent are two clients. UI and core exchange commands and events over channels; the audio thread and the core exchange data only through lock-free ring buffers and swapped snapshots. Compose and Record use a wrapped timeline: the engraver breaks the score into systems and the audio lane under each staff follows the same line breaks. Mix and Master use a linear timeline.

## Writing standard
Write ALL prose in ASD-STE100 Simplified Technical English: active voice, one instruction per sentence, no -ing verb forms, no idiom, no em dashes, expand an abbreviation at first use. This binds your final report too.

## Report protocol
Write your FULL result to a file under the scratchpad directory `/private/tmp/claude-501/-Users-james-Developer-duet/cb581a6d-1ce5-4cb9-8b4e-948690e49339/scratchpad/`, then store it with:
    .claude/plan-coordination/db.sh append roadmap/duet-v1 <SUFFIX> "$(cat <your-file>)"
Run that command from `/Users/james/Developer/duet`. Then return ONLY: a five-line summary, the printed store key, and an importance flag CRITICAL | HIGH | MEDIUM | LOW. Do not run any git command. Do not edit any file outside the write scope your brief names.

## Operator decision added 2026-09-20 (later in the session)
- Linux audio targets PipeWire, at its latest version, as the audio server. ALSA APIs are allowed, because PipeWire sits on top of ALSA and provides the ALSA client and sequencer interfaces. No PulseAudio or JACK client code exists in this app. No deprecated API is used on Linux or on macOS. No legacy support of any kind.
- Ubuntu 26.04 is the minimum Linux release. CI runs on it.
- macOS 26 is the minimum macOS release. No support for an earlier macOS. CI runs on it.
