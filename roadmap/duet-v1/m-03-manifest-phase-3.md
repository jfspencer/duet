---
id: M3
line: M
depends_on: [M2, T3, A1, D2, E1]
write_scope:
  - Cargo.toml
  - Cargo.lock
  - crates/duet-command/Cargo.toml
  - crates/duet-command/src/lib.rs
  - crates/duet-media/Cargo.toml
  - crates/duet-media/src/lib.rs
parallelism: serial-only: SM1 runs the manifest chunk alone before every line chunk of its phase.
completion: "cargo check -p duet-command -p duet-media --locked exits 0; commit SHA on a branch chunk/m3-manifest-phase-3"
---

# M3: The phase-3 manifest

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk opens phase 3. It pins `hound` and `arrayvec`, it creates the `duet-command` and
`duet-media` skeletons, and it adds the root path entry of each skeleton. It implements architecture
section 13.1 (the M3 row and ownership decision 6), section 13.0 rules SM1 and SM5, section 7.5, and
Appendix B.5. Section 13.3 puts M2, T3, A1, D2, and E1 in phase 2, so this chunk starts after all
five land.

**Phase 4 has a manifest chunk**, so SM1 rule 1 asks this chunk for the phase-3 set alone. Two names
are missing from the root manifest. Section 7.5 states the first use: `hound` is the WAV writer of
`duet-media`, and it is the default container for a render under the B67 limit. Section 5.10 states
the second: `PyramidBuilder::staging` is an `ArrayVec<PeakBin, PEAK_STAGING_BINS>` at B123, and chunk
D3 writes that builder in this same phase. **`arrayvec` moved to this chunk from M4** (architecture
section 13.1, ownership decision 6, and Appendix B.3): a pin that lands in phase 4 cannot serve a
phase-3 commit, and the chunk author of line D reported that D3 could add no entry at all.

The chunk does exactly four things (SM1): it pins, it creates two skeletons with no
`[dependencies]` section, it adds one root path entry per skeleton, and it settles `Cargo.lock`.
Dispatch: **Duet Engineer**. This chunk carries no policy file, so SM4 does not route it to the
Orchestrator.

## Files

| Path | Action |
|---|---|
| `Cargo.toml` | modify (`[workspace.dependencies]` only) |
| `Cargo.lock` | modify |
| `crates/duet-command/Cargo.toml` | create |
| `crates/duet-command/src/lib.rs` | create |
| `crates/duet-media/Cargo.toml` | create |
| `crates/duet-media/src/lib.rs` | create |

## Types and signatures

This chunk declares no Rust type. It writes two crate roots and two member manifests.

### The two third-party pins (`research/crate-survey.md`, sections 5.10 and 7.5, Appendix B.3 and B.5)

```toml
arrayvec = "0.7.8"
hound = "3.5.1"
```

Appendix B.5 holds no `hound` row, so that pin takes its default feature set. The `arrayvec` row of
Appendix B.5 names the default set too, and it states why: no `ArrayVec` field of this design is
serialized, so the serde feature would be an unused dependency. This chunk records both choices in
its verification step.

### The two internal path entries (SM1 rule 4)

```toml
duet-command = { path = "crates/duet-command" }
duet-media = { path = "crates/duet-media" }
```

A member crate reaches an internal crate with `duet-<crate> = { workspace = true }`, and that entry
resolves only when the root table already carries the path entry. SM4 keeps the root manifest out of
every line chunk's reach, so the chunk that creates the skeleton is the one chunk that can add it.
Revision 23 gave the internal entries to no chunk at all, and five chunk authors each reported the
same hole.

### The `duet-command` skeleton (SM1 rule 2)

```toml
[package]
name = "duet-command"
description = "The Duet transport vocabulary: every value that crosses the client seam, as plain data."
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
//! The Duet transport vocabulary.
//!
//! Every value that crosses the transport is plain data. The crate holds the
//! verb list, the outcome and event types, the view state, the MIDI records,
//! and the `BundleDocument` trait. Sections 1.8, 3.5, and 9.1 of
//! `roadmap/duet-v1/architecture.md` state the design.

#![forbid(unsafe_code)]
```

### The `duet-media` skeleton (SM1 rule 2)

```toml
[package]
name = "duet-media"
description = "The Duet media crate: the immutable audio source reader and the take writer."
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
//! The Duet media crate.
//!
//! It reads an immutable audio source and it writes a take. Every container
//! this product writes and every container it decodes passes through it.
//! Sections 7.5 and 15.9 of `roadmap/duet-v1/architecture.md` state the
//! design.

#![forbid(unsafe_code)]
```

Chunk T4 adds the `duet-command` member entries and chunk N1 adds the `duet-media` member entries,
each in the same commit as the code that uses them (SM1). Chunk D3 adds
`arrayvec = { workspace = true }` to `crates/duet-dsp/Cargo.toml` in this same phase, under the same
rule.

## Steps

1. Confirm that M2, T3, A1, D2, and E1 landed: `crates/duet-session`, `crates/duet-engrave`, and
   `crates/duet-analysis` each hold source beyond the skeleton, and
   `cargo nextest run -p duet-session --no-tests=fail` passes. Confirm that `crates/duet-command`
   and `crates/duet-media` do not exist. Report a discrepancy and stop if any one is false.
2. Add the `arrayvec` pin and the `hound` pin to `[workspace.dependencies]` of the root
   `Cargo.toml`, in alphabetical order with the entries the table already holds. Add the two
   internal path entries `duet-command` and `duet-media` to the same table (SM1 rule 4). Edit no
   other table (SM4).
3. Create the two member manifests and the two crate roots with the four blocks the section above
   gives.
4. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains one `[[package]]` entry for each new crate. The `hound` pin adds no edge yet, because no
   member declares it.
5. Run `cargo xtask check-manifests`. Expected result: exit 0 over every member.
6. Confirm the feature set of both pins. Run `cargo info hound@3.5.1` and
   `cargo info arrayvec@0.7.8`, and paste each output into the commit body (Appendix B.5). Confirm
   that `arrayvec` publishes a `serde` feature and that this pin enables none of it, so the pin
   takes the default set. **Do NOT run `cargo tree -e features -i hound` or
   `cargo tree -e features -i arrayvec` in this chunk.** Step 4 states the reason for `hound`, and
   `arrayvec` is in the same state: no member declares either one at this commit, so `cargo tree`
   exits 101 and prints `error: package ID specification` with the crate name. **Chunk N1 records
   the `hound` tree and chunk D3 records the `arrayvec` tree**, because each one adds the
   `{ workspace = true }` entry (Appendix B.3, Appendix B.5).
7. Run the Completion command: `cargo check -p duet-command -p duet-media --locked`. Expected
   result: exit 0. The command fails before this chunk, because `cargo` reports two unknown
   packages.
8. `git add` the write scope and `git commit`. The native hook runs `scripts/dod.sh`.

## Tests

This chunk writes no test. Its Completion command is the check (SM3):
`cargo check -p duet-command -p duet-media --locked` fails before the chunk and passes after it.
`cargo xtask check-manifests`, which chunk M0 added to `scripts/dod.sh`, is the mechanical guard
over both new manifests, and the native hook runs it on the commit.

## Verification

```
cargo check -p duet-command -p duet-media --locked
cargo xtask check-manifests
cargo info hound@3.5.1
cargo info arrayvec@0.7.8
cargo deny check
```

Expected output: the first command exits 0 and prints two `Checking` lines. The second command exits
0. The third command prints the published `hound` feature list and the fourth prints the published
`arrayvec` feature list, and the commit body carries both. **No `cargo tree` command belongs in this
chunk**, for the reason step 6 states. The fifth command reports no licence failure and no new
advisory; `hound` carries the Apache-2.0 licence and `arrayvec` carries MIT or Apache-2.0, and
`deny.toml` already allows all three.

Then commit on a branch named `chunk/m3-manifest-phase-3`. The native git hook runs
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
