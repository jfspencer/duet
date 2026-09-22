---
id: M4
line: M
depends_on: [M3, T4, A2, D3, E2, N1]
write_scope:
  - Cargo.toml
  - Cargo.lock
  - crates/duet-engine/Cargo.toml
  - crates/duet-engine/src/lib.rs
  - crates/duet-project/Cargo.toml
  - crates/duet-project/src/lib.rs
  - crates/duet-midi/Cargo.toml
  - crates/duet-midi/src/lib.rs
  - crates/duet-interchange/Cargo.toml
  - crates/duet-interchange/src/lib.rs
parallelism: serial-only: SM1 runs the manifest chunk alone before every line chunk of its phase.
completion: "cargo check -p duet-engine -p duet-project -p duet-midi -p duet-interchange --locked exits 0; commit SHA on a branch chunk/m4-manifest-phase-4"
---

# M4: The phase-4 manifest

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk opens phase 4, the widest phase of the plan. It pins twenty-one crates, it creates the
`duet-engine`, `duet-project`, `duet-midi`, and `duet-interchange` skeletons, and it adds the root
path entry of each skeleton. It implements architecture section 13.1 (the M4 row and ownership
decisions 1, 2, 6 and 7), section 13.0 rules SM1 and SM5, section 11.2, section 11.5, Appendix B.3,
Appendix B.4, and Appendix B.5. Section 13.3 puts
M3, T4, A2, D3, E2, and N1 in phase 3, so this chunk starts after all six land.

**SM1 rule 1 makes this chunk pin for phases 5 and 6 as well.** Phases 5 and 6 have no manifest
chunk, so every crate those phases add is pinned here. `criterion` is the stated case: chunk A4 adds
the `[[bench]]` target in phase 5 (section 13.1 decision 2).

**`arrayvec` left this chunk for M3** (section 13.1 decision 6, Appendix B.3 and B.5). Chunk D3
writes `PyramidBuilder::staging`, which is an `ArrayVec<PeakBin, PEAK_STAGING_BINS>` at B123, in
phase 3, so a pin that landed here would land one phase late. The chunk author of line D reported
it. This chunk therefore CONFIRMS the `arrayvec` pin that M3 wrote and adds no entry for it; chunks
C3 and I1 use the same crate in phases 6 and 7 and read the same pin.

**`async-channel` arrived at this chunk from M7** (section 13.1 ownership decision 7, Appendix B.3
and B.5). B138 is the engine-to-core event channel and it carries an `async_channel::Sender`; chunk
C2 writes `EngineLink` and that channel in phase 5, which is inside the reach of SM1 rule 1 above.
A pin that landed with M7, in phase 7, would land two phases after the first commit that needs it.
Chunk I1 adds the `duet-core` entry for the same crate in phase 7 and reads this same pin.

**`coremidi` and `pipewire` are root pins with no `cfg`** (section 11.2, section 13.1 decision 1).
The root manifest pins both plainly. Chunk G2 declares `coremidi` under a macOS `cfg` table in
`crates/duet-midi/Cargo.toml`, and chunk G3 declares `pipewire` under a Linux `cfg` table in the
same file.

The chunk does exactly four things (SM1): it pins, it creates four skeletons with no
`[dependencies]` section, it adds one root path entry per skeleton, and it settles `Cargo.lock`. Dispatch: **Duet Engineer**. This chunk
carries no policy file, so SM4 does not route it to the Orchestrator.

## Files

| Path | Action |
|---|---|
| `Cargo.toml` | modify (`[workspace.dependencies]` only) |
| `Cargo.lock` | modify |
| `crates/duet-engine/Cargo.toml` | create |
| `crates/duet-engine/src/lib.rs` | create |
| `crates/duet-project/Cargo.toml` | create |
| `crates/duet-project/src/lib.rs` | create |
| `crates/duet-midi/Cargo.toml` | create |
| `crates/duet-midi/src/lib.rs` | create |
| `crates/duet-interchange/Cargo.toml` | create |
| `crates/duet-interchange/src/lib.rs` | create |

## Types and signatures

This chunk declares no Rust type. It writes four crate roots and four member manifests.

### The pins (`research/crate-survey.md`, Appendix B.3, Appendix B.5)

Twenty of the twenty-one names below are new. `tracing` is already in the root manifest, so this
chunk CONFIRMS the pin rather than adding the entry (section 13.1 decision 4). `arrayvec` is in no
list below: chunk M3 pinned it in phase 3, and this chunk confirms that entry and adds nothing.

```toml
quick-xml = "0.42.0"
midly = "0.5.3"
ebur128 = "0.1.10"
bwavfile = "2.0.1"
flacenc = "0.5.1"
rtrb = "0.4.0"
triple_buffer = "9.0.0"
basedrop = "0.1.3"
crossbeam-queue = "0.3.14"
async-channel = "2.5.0"
midir = "0.11.0"
coremidi = "0.9.2"
pipewire = "0.10.1"
notify = "8.2.0"
notify-debouncer-full = "0.7.0"
cpal = { version = "0.18.2", default-features = false, features = ["pipewire"] }
blake3 = { version = "1.8.7", default-features = false, features = ["pure", "std"] }
```

Four pins carry a feature list that Appendix B.5 marks "resolves". This chunk reads each pinned
version's own manifest, writes the resolved list, and records `cargo tree -e features -i <crate>`.

| Pin | Version | Feature requirement (Appendix B.5) |
|---|---|---|
| `symphonia` | 0.6.1 | MP3 and AAC decoding, which the default set omits, beside FLAC and OGG. The chunk resolves the exact decoder feature names and the ISO base media container feature, plus the default set |
| `gix` | 0.87.1 | Object write, tree build, commit, reference edit, and reflog read, with no network transport and no clean or smudge filter. `default-features = false` plus the object and reference features that list needs |
| `criterion` | 0.7.0 | The bench harness with no plotting back end, so the tree stays small and `cargo deny` stays quiet. `default-features = false` plus the cargo bench support feature |

`crossbeam-queue` and `async-channel` each take the default set, which Appendix B.5 verifies
against section 5.8, section 5.5, and section 15.14. `arrayvec` takes the default set too, and chunk
M3 owns that pin. Every other pin above takes its default set, which Appendix B.5's stated rule
covers, and this chunk records that choice.

### The four internal path entries (SM1 rule 4)

```toml
duet-engine = { path = "crates/duet-engine" }
duet-interchange = { path = "crates/duet-interchange" }
duet-midi = { path = "crates/duet-midi" }
duet-project = { path = "crates/duet-project" }
```

A member crate reaches an internal crate with `duet-<crate> = { workspace = true }`, and that entry
resolves only when the root table already carries the path entry. SM4 keeps the root manifest out of
every line chunk's reach, so the chunk that creates the skeleton is the one chunk that can add it.

### The four skeletons (SM1 rule 2)

Each member manifest carries the eight `*.workspace = true` package fields, a `description`, and
`[lints] workspace = true`, and no `[dependencies]` section.

```toml
[package]
name = "duet-engine"
description = "The Duet audio engine: the backend trait, the transport, the graph runner, and the disk threads."
version.workspace = true
edition.workspace = true
rust-version.workspace = true
license.workspace = true
repository.workspace = true
authors.workspace = true
publish.workspace = true

[lints]
workspace = true
```

```rust
//! The Duet audio engine.
//!
//! It holds the backend trait, the transport state machine, the typed
//! processor chain, the graph runner, and the disk reader and writer. The
//! audio thread allocates nothing and locks nothing. Section 5 of
//! `roadmap/duet-v1/architecture.md` states the design.

#![forbid(unsafe_code)]
```

The three other skeletons take the same shape with these values.

| Crate | `description` | Crate documentation first line | Architecture section |
|---|---|---|---|
| `duet-project` | "The Duet project bundle: the directory layout, the content store, the git history, and the recent list." | "The Duet project bundle." | 4 |
| `duet-midi` | "The Duet MIDI crate: device presence, hot plug, and the byte stream." | "The Duet MIDI crate." | 8 |
| `duet-interchange` | "The Duet interchange crate: the MusicXML reader and writer and the Standard MIDI File import." | "The Duet interchange crate." | 3.7 and 8.4 |

Chunk C1 adds the `duet-engine` entries, chunk F1 adds the `duet-project` entries, chunk G1 adds the
`duet-midi` entries, and chunk X1 adds the `duet-interchange` entries, each in the same commit as
the code that uses them (SM1).

## Steps

1. Confirm that M3, T4, A2, D3, E2, and N1 landed: `crates/duet-command` and `crates/duet-media`
   each hold source beyond the skeleton, and `cargo nextest run -p duet-command --no-tests=fail`
   passes. Confirm that the four new crate directories do not exist. Confirm that
   `.github/workflows/ci.yml` names `ubuntu-26.04` and installs `libpipewire-0.3-dev`, which chunk
   M0 wrote; **the first `cpal` build fails the Linux gate without it** (section 13.1). Report a
   discrepancy and stop if any one is false.
2. Add the seventeen plain pins to `[workspace.dependencies]` of the root `Cargo.toml`, in
   alphabetical order with the entries the table already holds. Confirm that the `tracing` entry and
   the `arrayvec` entry both stay as they are; chunk M3 wrote `arrayvec` in phase 3. Add the four
   internal path entries `duet-engine`, `duet-interchange`, `duet-midi`, and `duet-project` to the
   same table (SM1 rule 4). Edit no other table (SM4).
3. Resolve and add the three "resolves" pins. For each one, read the pinned version's own manifest,
   write the feature list Appendix B.5 asks for, and keep `default-features = false` where that
   table states it. **A chunk that cannot obtain the stated capability with the pinned version
   reports the discrepancy instead of proceeding** (SM0).
4. Create the four member manifests and the four crate roots with the shapes the section above
   gives.
5. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains one `[[package]]` entry for each new crate and the transitive tree of every new pin. No
   member declares a pin yet, so the lock gains no member edge.
6. Run `cargo xtask check-manifests`. Expected result: exit 0 over every member.
7. Run `cargo deny check`. Expected result: it reports no licence failure. `midly` carries the
   `Unlicense`, which chunk M0 added to the `deny.toml` allow list (Appendix B.4). **`midly` is
   unmaintained since 2023**, so the advisory check may report a RUSTSEC identifier. An ignore entry
   is an SM4 adjudication and never this chunk's edit: report the identifier to the Orchestrator and
   stop if the check fails.
8. Record every feature resolution. Run `cargo tree -e features -i <crate>` for `cpal`, `blake3`,
   `symphonia`, `gix`, `criterion`, `crossbeam-queue`, and `async-channel`, and paste each output
   into the commit body (Appendix B.5). `arrayvec` is chunk M3's pin, and chunk M3 recorded it.
9. Record the transitive ALSA version. Run `cargo tree -i alsa` and paste the output into the commit
   body. Appendix B.4 asks for it, because `midir` brings `alsa` and the survey does not record the
   version.
10. Confirm that `pipewire` 0.10.1 adds no second version. `cpal` 0.18.2 already brings that version
    through its `pipewire` feature, so `cargo deny check` reports no new `multiple-versions` warning
    (Appendix B.3). Record the `cargo tree -i pipewire` output.
11. Run the Completion command:
    `cargo check -p duet-engine -p duet-project -p duet-midi -p duet-interchange --locked`. Expected
    result: exit 0. The command fails before this chunk, because `cargo` reports four unknown
    packages.
12. `git add` the write scope and `git commit`. The native hook runs `scripts/dod.sh`.

## Tests

This chunk writes no test. Its Completion command is the check (SM3):
`cargo check -p duet-engine -p duet-project -p duet-midi -p duet-interchange --locked` fails before
the chunk and passes after it. `cargo xtask check-manifests` and `cargo deny check`, which
`scripts/dod.sh` runs, are the mechanical guards over the four new manifests and the twenty new
pins, and the native hook runs both on the commit.

## Verification

```
cargo check -p duet-engine -p duet-project -p duet-midi -p duet-interchange --locked
cargo xtask check-manifests
cargo deny check
cargo tree -e features -i cpal
cargo tree -e features -i blake3
cargo tree -e features -i symphonia
cargo tree -e features -i gix
cargo tree -e features -i criterion
cargo tree -e features -i async-channel
cargo tree -i alsa
cargo tree -i pipewire
```

Expected output: the first command exits 0 and prints four `Checking` lines. The second command
exits 0. The third command reports no licence failure and no advisory failure. Each `cargo tree`
command prints its resolved set, and the commit body carries every one.

The build must succeed on macOS and on Linux. The Linux run needs `libpipewire-0.3-dev`,
`libasound2-dev`, and `pkg-config`, which chunk M0 added to `scripts/bootstrap.sh` and to the Linux
CI job (section 11.5).

Then commit on a branch named `chunk/m4-manifest-phase-4`. The native git hook runs
`scripts/dod.sh`, and the commit lands only when every gate passes.

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
