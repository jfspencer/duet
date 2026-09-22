---
id: M7
line: M
depends_on: [C3, F3, G3, X3, T1]
write_scope:
  - Cargo.toml
  - Cargo.lock
  - .github/workflows/soak.yml
  - crates/duet-export/Cargo.toml
  - crates/duet-export/src/lib.rs
  - crates/duet-core/Cargo.toml
  - crates/duet-core/src/lib.rs
parallelism: serial-only: SM1 runs the manifest chunk alone before every line chunk of its phase, and SM4 makes the workflow file an Orchestrator adjudication.
completion: "cargo check -p duet-export -p duet-core --locked exits 0; commit SHA on a branch chunk/m7-manifest-phase-7"
---

# M7: The phase-7 manifest and the soak workflow

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk opens phase 7. It pins `futures` and `rubato`, it creates the `duet-export` and
`duet-core` skeletons, it adds the root path entry of each skeleton, and it writes
`.github/workflows/soak.yml`. **It does exactly four things under SM1**: it pins, it creates the two
skeletons with no `[dependencies]` section, it adds one root path entry per skeleton (SM1 rule 4),
and it settles `Cargo.lock`; `soak.yml` is the repository file SM1 rule 3 gives it. It implements
architecture section 13.1 (the M7 row and ownership decision 7), section 13.0 rules SM1, SM3, SM4,
SM5, and SM7, and section 14 rung one.

**`async-channel` left this chunk for M4** (section 13.1 ownership decision 7, Appendix B.3 and
B.5). B138 is the engine-to-core event channel and it carries an `async_channel::Sender`; chunk C2
writes `EngineLink` and that channel in phase 5, and phases 5 and 6 have no manifest chunk, so SM1
rule 1 gives the pin to M4. This chunk therefore CONFIRMS the `async-channel` pin that M4 wrote and
adds no entry for it. **`futures` stays here**, because the oneshot reply of that crate reaches
chunk I1 in this phase and no earlier chunk names it. Section 13.3 puts C3, F3, G3, and X3 in phase 6, so this chunk starts after all four
land.

**The Orchestrator executes this chunk directly.** `.github/workflows/soak.yml` is a policy file
under SM4, and CLAUDE.md makes a gate-adjacent edit an adjudication. No engineer may edit it.

**Two links of section 13.4 reach back past phase 6, and SM7 is the reason.** `soak.yml` runs
`proptest_large`, which chunk T1 writes in phase 0, and it runs `soak`, which chunk C3 writes in
phase 6. A filtered run with no match is a failure and not a pass (SM3 rule 1), so the workflow lands
only after both tests exist.

**SM1 rule 1 makes this chunk pin for phase 8 too, where the next manifest chunk is M8.** Phase 7
and phase 8 each have a manifest chunk, so this chunk pins the phase-7 set alone.

## Files

| Path | Action |
|---|---|
| `Cargo.toml` | modify (`[workspace.dependencies]` only) |
| `Cargo.lock` | modify |
| `.github/workflows/soak.yml` | create |
| `crates/duet-export/Cargo.toml` | create |
| `crates/duet-export/src/lib.rs` | create |
| `crates/duet-core/Cargo.toml` | create |
| `crates/duet-core/src/lib.rs` | create |

## Types and signatures

This chunk declares no Rust type. It writes two crate roots, two member manifests, and one workflow
file.

### The pins (Appendix B.3, `research/crate-survey.md`, Appendix B.5)

```toml
futures = "0.3.34"
rubato = "5.0.0"
```

Appendix B.3 states the purpose of the first: `futures` carries the executor-agnostic oneshot reply,
and chunk I1 adds the member entry in this phase. Appendix B.5 holds no row for either name, so each
one takes its default feature set, and this chunk records that choice. `async-channel` is in no list
here: chunk M4 pinned it in phase 4, and this chunk confirms that entry and adds nothing.

### The two internal path entries (SM1 rule 4)

```toml
duet-core = { path = "crates/duet-core" }
duet-export = { path = "crates/duet-export" }
```

A member crate reaches an internal crate with `duet-<crate> = { workspace = true }`, and that entry
resolves only when the root table already carries the path entry. SM4 keeps the root manifest out of
every line chunk's reach, so the chunk that creates the skeleton is the one chunk that can add it.

### The `duet-export` skeleton (SM1 rule 2)

```toml
[package]
name = "duet-export"
description = "The Duet export pipeline: the two-pass render, the normalization stage, and the encode stage."
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
//! The Duet export pipeline.
//!
//! It renders the mix offline in freewheel, measures the result, and encodes
//! one file per request. Section 7.4 of `roadmap/duet-v1/architecture.md`
//! states the design.

#![forbid(unsafe_code)]
```

### The `duet-core` skeleton (SM1 rule 2)

```toml
[package]
name = "duet-core"
description = "The Duet core: the gateway, the document set, the job runner, and the snapshot publication."
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
//! The Duet core.
//!
//! It owns the document set, it answers every gateway verb, it runs the
//! deferred jobs, and it publishes the snapshots that a client reads. It is
//! headless: it opens no window and it names no framework type. Sections 9
//! and 15.14 of `roadmap/duet-v1/architecture.md` state the design.

#![forbid(unsafe_code)]
```

Chunk H1 adds the `duet-export` entries and chunk I1 adds the `duet-core` entries, each in the same
commit as the code that uses them (SM1).

### `.github/workflows/soak.yml` (section 14)

The workflow runs nightly on `ubuntu-26.04` and on `macos-26`. It runs two commands, in this order,
and each one carries `--no-tests=fail` (SM3 rule 1).

```
cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(soak)' --no-tests=fail
cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large)' --no-tests=fail
```

The job installs the same Linux packages as the `dod` job of `ci.yml`, which chunk M0 wrote:
`libpipewire-0.3-dev`, `libasound2-dev`, and `pkg-config`, beside the GPUI package list. It installs
`cargo-nextest` at 0.9.145, the version chunk M0 pinned in `scripts/bootstrap.sh`.

**This workflow is not a gate.** CLAUDE.md makes `scripts/dod.sh` the only gate surface, so a
failure here blocks a merge by review and never by a hook (section 14).

## Steps

1. Confirm that C3, F3, G3, and X3 landed. Confirm that `crates/duet-engine/tests/soak.rs` exists
   and that `cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(soak)' --no-tests=fail`
   passes. Confirm that `crates/duet-time/tests/proptest_large.rs` exists and that
   `cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large)' --no-tests=fail`
   passes. **Report a discrepancy and stop if either filter matches nothing**, because SM3 rule 1
   forbids a filtered run with no match.
2. Confirm that `crates/duet-export` and `crates/duet-core` do not exist. Report a discrepancy and
   stop if either one does.
3. Add the two pins and the two internal path entries `duet-core` and `duet-export` (SM1 rule 4) to
   `[workspace.dependencies]` of the root `Cargo.toml`, in alphabetical order with the entries the
   table already holds. Confirm that the `async-channel` entry stays as it is; chunk M4 wrote it in
   phase 4. Edit no other table (SM4).
4. Create the two member manifests and the two crate roots with the four blocks the section above
   gives.
5. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains one `[[package]]` entry for each new crate and the transitive tree of the three new pins.
6. Run `cargo xtask check-manifests`. Expected result: exit 0 over every member.
7. Write `.github/workflows/soak.yml` with the two jobs the section above states. Set the trigger to
   a nightly `schedule` entry and a `workflow_dispatch` entry, so an operator can start it by hand.
   Set the matrix to `os: [macos-26, ubuntu-26.04]`.
8. Run the two soak commands by hand on the developer machine, once each. Expected result: both
   pass. The run proves that the filter of each workflow command matches at least one test (SM7).
9. Run `cargo deny check`. Expected result: it reports no licence failure. `futures` carries MIT or
   Apache-2.0 and `rubato` carries MIT, and `deny.toml` already allows both.
10. Record the feature resolution of the two pins. Run `cargo tree -e features -i futures` and
    `cargo tree -e features -i rubato`, and paste each output into the commit body (Appendix B.5).
    `async-channel` is chunk M4's pin, and chunk M4 recorded it.
11. Run the Completion command: `cargo check -p duet-export -p duet-core --locked`. Expected result:
    exit 0. The command fails before this chunk, because `cargo` reports two unknown packages.
12. `git add` the write scope and `git commit`. The native hook runs `scripts/dod.sh`.

## Tests

This chunk writes no test. It SELECTS two tests that earlier chunks wrote, and SM7 binds the
selection: section 14 holds one row for `soak` (chunk C3, phase 6) and one row for `proptest_large`
(chunk T1, phase 0), and both chunks land before phase 7. Step 8 runs each filter once and proves
the match.

Its Completion command is the check (SM3): `cargo check -p duet-export -p duet-core --locked` fails
before the chunk and passes after it. `cargo xtask check-manifests`, which `scripts/dod.sh` runs, is
the mechanical guard over both new manifests.

## Verification

```
cargo check -p duet-export -p duet-core --locked
cargo xtask check-manifests
cargo nextest run -p duet-engine --run-ignored ignored-only -E 'test(soak)' --no-tests=fail
cargo nextest run -p duet-time --run-ignored ignored-only -E 'test(proptest_large)' --no-tests=fail
cargo deny check
bash -n .github/workflows/soak.yml || true
```

Expected output: the first command exits 0 and prints two `Checking` lines. The second command exits
0. The third and the fourth commands each report at least one test as passed and report no filter
miss. The fifth command reports no licence failure. The workflow file is YAML, so read it with a
YAML parser rather than with `bash -n`; the repository gate runs `bash -n` on shell files alone.

Then commit on a branch named `chunk/m7-manifest-phase-7`. The native git hook runs
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
