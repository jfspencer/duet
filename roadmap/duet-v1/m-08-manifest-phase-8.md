---
id: M8
line: M
depends_on: [M7, C4, F4, G4, H1, I1]
write_scope:
  - Cargo.toml
  - Cargo.lock
  - .github/workflows/audio-smoke.yml
  - crates/duet-agent/Cargo.toml
  - crates/duet-agent/src/lib.rs
parallelism: serial-only: SM1 runs the manifest chunk alone before every line chunk of its phase, and SM4 makes the workflow file an Orchestrator adjudication.
completion: "cargo check -p duet-agent --locked exits 0; commit SHA on a branch chunk/m8-manifest-phase-8"
---

# M8: The phase-8 manifest and the audio smoke workflow

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk opens phase 8. It pins `rmcp`, `tokio`, and `tokio-util`, it confirms the `clap` pin, it
creates the `duet-agent` skeleton, it adds the root path entry of that skeleton, and it writes
`.github/workflows/audio-smoke.yml`. **It does exactly four things under SM1**: it pins, it creates
the skeleton with no `[dependencies]` section, it adds one root path entry per skeleton (SM1 rule
4), and it settles `Cargo.lock`; `audio-smoke.yml` is the repository file SM1 rule 3 gives it. It
implements architecture section 13.1 (the M8 row and ownership decision 3), section 13.0 rules SM1,
SM3, SM4,
SM5, and SM7, section 9.2, section 9.5, section 11.5, section 11.6, section 14 rung one, Appendix
B.3, and Appendix B.5. Section 13.3 puts M7, C4, F4, G4, H1, and I1 in phase 7, so this chunk starts
after all six land.

**The Orchestrator executes this chunk directly.** `.github/workflows/audio-smoke.yml` is a policy
file under SM4, and CLAUDE.md makes a gate-adjacent edit an adjudication. No engineer may edit it.

**`audio-smoke.yml` belongs to this chunk because C4 lands in phase 7** (section 13.1 decision 3).
The workflow runs `pipewire_smoke`, which chunk C4 writes in `crates/duet-engine/src/cpal/stream.rs`.
A workflow that runs the filter before the cpal backend exists is a filtered run with no match, and
SM3 rule 1 forbids it. M8 is the first manifest chunk that runs after C4.

## Files

| Path | Action |
|---|---|
| `Cargo.toml` | modify (`[workspace.dependencies]` only) |
| `Cargo.lock` | modify |
| `.github/workflows/audio-smoke.yml` | create |
| `crates/duet-agent/Cargo.toml` | create |
| `crates/duet-agent/src/lib.rs` | create |

## Types and signatures

This chunk declares no Rust type. It writes one crate root, one member manifest, and one workflow
file.

### The pins (Appendix B.3, `research/crate-survey.md`, Appendix B.5)

```toml
tokio-util = "0.7.19"
```

Three pins carry more than a version.

| Pin | Version | Feature requirement (Appendix B.5) | State |
|---|---|---|---|
| `rmcp` | 3.4.0 | The server role and a transport over any `AsyncRead` and `AsyncWrite` pair. `default-features = false` plus the server feature and the stream transport feature | **This chunk resolves the exact names against 3.4.0 and records them** |
| `tokio` | **not in the survey** | A multi-thread runtime (section 9.5 rule 3), a Unix stream socket (section 9.2), a signal handler (section 9.5 rule 7), and a deadline for B15, B16, B17, and B23. `default-features = false, features = ["rt-multi-thread", "net", "time", "signal", "io-util", "sync", "macros"]` | **This chunk resolves the version from the `rmcp` 3.4.0 requirement, pins that exact version, confirms each feature name against it, and records `cargo tree -i tokio`** |
| `clap` | 4.6.7 | The derive macro, `features = ["derive"]` | **Already in the root manifest.** This chunk CONFIRMS the pin and the feature rather than adding the entry (section 13.1 decision 4) |

`tokio-util` supplies `CancellationToken` for the shutdown order, and chunk J1 uses it (Appendix
B.3). Appendix B.5 holds no `tokio-util` feature row, so the pin takes its default feature set and
this chunk records that choice.

**`tokio` has no survey row and no Version cell.** Appendix B.3 gives the rule for that shape: the
owning chunk reads the pinned dependent's own manifest, writes the resolved value, and records
`cargo tree -i tokio`. **A chunk that cannot obtain the seven capabilities with the resolved version
reports the discrepancy instead of proceeding** (SM0).

### The internal path entry (SM1 rule 4)

```toml
duet-agent = { path = "crates/duet-agent" }
```

A member crate reaches an internal crate with `duet-<crate> = { workspace = true }`, and that entry
resolves only when the root table already carries the path entry. SM4 keeps the root manifest out of
every line chunk's reach, so the chunk that creates the skeleton is the one chunk that can add it.

### The `duet-agent` skeleton (SM1 rule 2)

```toml
[package]
name = "duet-agent"
description = "The Duet agent gateway: the Model Context Protocol server over stdio and a Unix socket."
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
//! The Duet agent gateway.
//!
//! It exposes one flat verb list to a terminal agent over the Model Context
//! Protocol, on stdio and on a Unix stream socket. It owns the tokio runtime
//! that the application embeds. Section 9 of
//! `roadmap/duet-v1/architecture.md` states the design.

#![forbid(unsafe_code)]
```

Chunk J1 adds the `duet-agent` entries in the same commit as the code that uses them (SM1).

### `.github/workflows/audio-smoke.yml` (section 11.6, section 14)

The workflow runs on `ubuntu-26.04` only, on every run. A macOS runner offers no virtual audio
device (section 11.6). The job has four steps, in this order.

1. `apt-get install pipewire libpipewire-0.3-dev libasound2-dev pkg-config`.
2. Start a user PipeWire process.
3. **Wait for its socket under B85, which is 30 seconds, and fail the job on expiry.** The wait is a
   shell loop with a deadline (Appendix B.5). A wait with no bound is what section 5.12 forbids
   everywhere else.
4. `cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(pipewire_smoke)' --no-tests=fail`.

The test opens one PipeWire stream through the real backend, runs one cycle, asserts
`CycleOutcome::Ran`, and closes the stream. It carries `#[ignore]`, so `scripts/dod.sh` never runs
it and a developer with no daemon is never blocked (section 11.6).

**This workflow is not a gate.** CLAUDE.md makes `scripts/dod.sh` the only gate surface, so a
failure here blocks a merge by review and never by a hook (section 11.6, section 14).

## Steps

1. Confirm that M7, C4, F4, G4, H1, and I1 landed. Confirm that
   `crates/duet-engine/src/cpal/stream.rs` holds a `pipewire_smoke` test and that
   `cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(pipewire_smoke)' --no-tests=fail`
   matches it. **Report a discrepancy and stop if the filter matches nothing**, because SM3 rule 1
   forbids a filtered run with no match.
2. Confirm that `crates/duet-agent` does not exist, and that the root `[workspace.dependencies]`
   table already holds `clap` with `features = ["derive"]`. Report a discrepancy and stop if either
   is false.
3. Read the `rmcp` 3.4.0 manifest. Resolve the server feature name and the stream transport feature
   name, and resolve the `tokio` version that `rmcp` 3.4.0 requires. Record both readings.
4. Add the three pins and the one internal path entry
   `duet-agent = { path = "crates/duet-agent" }` (SM1 rule 4) to `[workspace.dependencies]` of the
   root `Cargo.toml`, with the resolved feature lists and the resolved `tokio` version, in
   alphabetical order with the entries the table already holds. Confirm that the `clap` entry stays
   as it is. Edit no other table (SM4).
5. Create `crates/duet-agent/Cargo.toml` and `crates/duet-agent/src/lib.rs` with the two blocks the
   section above gives.
6. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains the `duet-agent` package entry and the transitive tree of the three new pins.
7. Run `cargo xtask check-manifests`. Expected result: exit 0 over every member.
8. Write `.github/workflows/audio-smoke.yml` with the four steps the section above states. Set the
   trigger to `pull_request` and to `push` on `main`, which is the "every run" of section 14. Write
   the socket wait as a loop whose total deadline is 30 seconds and whose expiry exits non-zero.
9. Run `cargo deny check`. Expected result: it reports no licence failure. `rmcp` carries
   Apache-2.0, `tokio` carries MIT, and `tokio-util` carries MIT, and `deny.toml` already allows all
   three.
10. Record every feature resolution. Run `cargo tree -e features -i rmcp`, `cargo tree -i tokio`,
    and `cargo tree -e features -i tokio-util`, and paste each output into the commit body
    (Appendix B.5).
11. Run the Completion command: `cargo check -p duet-agent --locked`. Expected result: exit 0. The
    command fails before this chunk, because `cargo` reports an unknown package.
12. `git add` the write scope and `git commit`. The native hook runs `scripts/dod.sh`.

## Tests

This chunk writes no test. It SELECTS one test that chunk C4 wrote, and SM7 binds the selection:
section 14 holds one row for `pipewire_smoke` (chunk C4, phase 7,
`crates/duet-engine/src/cpal/stream.rs`, `#[ignore]`), and C4 lands one phase before this chunk.
Step 1 runs the filter once and proves the match.

Its Completion command is the check (SM3): `cargo check -p duet-agent --locked` fails before the
chunk and passes after it. `cargo xtask check-manifests`, which `scripts/dod.sh` runs, is the
mechanical guard over the new manifest.

## Verification

```
cargo check -p duet-agent --locked
cargo xtask check-manifests
cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(pipewire_smoke)' --no-tests=fail
cargo deny check
cargo tree -e features -i rmcp
cargo tree -i tokio
cargo tree -e features -i tokio-util
```

Expected output: the first command exits 0 and prints one `Checking` line. The second command exits
0. The third command reports one test as passed on a machine with a PipeWire daemon, and reports no
filter miss; on a machine with no daemon the test reports its own refusal, and the CI job is the one
place that proves the path. The fourth command reports no licence failure. Each `cargo tree` command
prints its resolved set, and the commit body carries every one.

Then commit on a branch named `chunk/m8-manifest-phase-8`. The native git hook runs
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
