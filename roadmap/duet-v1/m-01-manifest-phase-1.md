---
id: M1
line: M
depends_on: [M0, T1]
write_scope:
  - Cargo.toml
  - Cargo.lock
  - crates/duet-score/Cargo.toml
  - crates/duet-score/src/lib.rs
  - crates/duet-dsp/Cargo.toml
  - crates/duet-dsp/src/lib.rs
parallelism: serial-only: SM1 runs the manifest chunk alone before every line chunk of its phase.
completion: "cargo check -p duet-score -p duet-dsp --locked exits 0; commit SHA on a branch chunk/m1-manifest-phase-1"
---

# M1: The phase-1 manifest

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk opens phase 1. It pins the two signal-processing crates that phase 1 and phase 2 need,
and it creates the `duet-score` and `duet-dsp` skeletons. It implements architecture section 13.1
(the M1 row), section 13.0 rules SM1 and SM5, and Appendix B.5. Section 13.3 puts M0 and T1 in
phase 0, so this chunk starts after both land.

**SM1 rule 1 covers the phases after this one as well.** Phase 2 has a manifest chunk, so this
chunk pins the phase-1 set alone. `rustfft` and `realfft` are the two names the root manifest lacks.

The chunk does exactly four things (SM1): it pins, it creates a skeleton with no `[dependencies]`
section, it adds one root path entry per skeleton (SM1 rule 4), and it settles `Cargo.lock`. It writes no crate source beyond the two `//!` roots.
Dispatch: **Duet Engineer**. This chunk carries no policy file, so SM4 does not route it to the
Orchestrator.

## Files

| Path | Action |
|---|---|
| `Cargo.toml` | modify (`[workspace.dependencies]` only) |
| `Cargo.lock` | modify |
| `crates/duet-score/Cargo.toml` | create |
| `crates/duet-score/src/lib.rs` | create |
| `crates/duet-dsp/Cargo.toml` | create |
| `crates/duet-dsp/src/lib.rs` | create |

## Types and signatures

This chunk declares no Rust type. It writes two crate roots and two member manifests.

### The pins (Appendix B.3, `research/crate-survey.md`, Appendix B.5)

```toml
rustfft = "6.4.1"
realfft = "3.5.0"
```

Appendix B.5 states the rule for a pin its table omits: **the pin takes its default feature set, and
this chunk records that choice in its own verification step.** Neither `rustfft` nor `realfft` has a
B.5 row, so both take the default set.

### The two internal path entries (SM1 rule 4)

```toml
duet-dsp = { path = "crates/duet-dsp" }
duet-score = { path = "crates/duet-score" }
```

A member crate reaches an internal crate with `duet-<crate> = { workspace = true }`, and that entry
resolves only when the root table already carries the path entry. SM4 keeps the root manifest out of
every line chunk's reach, so the chunk that creates the skeleton is the one chunk that can add it.
Revision 23 gave the internal entries to no chunk at all, and five chunk authors each reported the
same hole.

### The `duet-score` skeleton (SM1 rule 2)

```toml
[package]
name = "duet-score"
description = "The Duet score aggregate: entities, commands, events, and the canonical document."
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
//! The Duet score aggregate.
//!
//! It holds the score root, its entities and value objects, the command and
//! event vocabulary, and the canonical document. It names no file format
//! parser. Section 3 of `roadmap/duet-v1/architecture.md` states the design.

#![forbid(unsafe_code)]
```

### The `duet-dsp` skeleton (SM1 rule 2)

```toml
[package]
name = "duet-dsp"
description = "The Duet signal kernels: the peak pyramid, the meters, and the allocation-free processor states."
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
//! The Duet signal kernels.
//!
//! A signal block allocates nothing and locks nothing, and one peak file
//! format serves every reader. Sections 5.5, 5.10, and 7.3 of
//! `roadmap/duet-v1/architecture.md` state the design.

#![forbid(unsafe_code)]
```

Neither skeleton carries a `[dependencies]` section. Chunk T2 adds the `duet-score` entries and
chunk D1 adds the `duet-dsp` entries, each in the same commit as the code that uses them (SM1).

## Steps

1. Confirm that M0 landed: `crates/duet-time` exists, the root `[workspace.dependencies]` table
   holds `smallvec`, `proptest`, and `md-5`, and `scripts/dod.sh` holds the `check-conversions`
   line. Confirm that T1 landed: `cargo nextest run -p duet-time --no-tests=fail` passes. Confirm
   that `crates/duet-score` and `crates/duet-dsp` do not exist. Report a discrepancy and stop if any
   one is false.
2. Add the two pins and the two internal path entries `duet-dsp` and `duet-score` (SM1 rule 4) to
   `[workspace.dependencies]` of the root `Cargo.toml`, in alphabetical order with the entries the
   table already holds. Edit no other table (SM4).
3. Create `crates/duet-score/Cargo.toml` and `crates/duet-score/src/lib.rs` with the two blocks the
   section above gives.
4. Create `crates/duet-dsp/Cargo.toml` and `crates/duet-dsp/src/lib.rs` with the two blocks the
   section above gives.
5. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains one `[[package]]` entry for each new crate. The two pins add no edge yet, because no member
   declares either one.
6. Run `cargo xtask check-manifests`. Expected result: exit 0. Both new manifests hold
   `[lints] workspace = true` and a non-empty `description`.
7. Confirm the feature set of both pins. Run `cargo info rustfft@6.4.1` and
   `cargo info realfft@3.5.0`, and paste each output into the commit body. Both pins take the
   default feature set, and Appendix B.5 requires the record for every pin its table omits.
   **Do NOT run `cargo tree -e features -i rustfft` or `cargo tree -e features -i realfft` in this
   chunk.** Step 5 states the reason: the two pins add no edge yet, because no member declares
   either one, so `cargo tree` exits 101 and prints `error: package ID specification` with the crate
   name. **Chunk D1 records the resolved tree of both crates**, because D1 adds both
   `{ workspace = true }` entries to `crates/duet-dsp/Cargo.toml`. Appendix B.5 states the rule.
8. Run the Completion command: `cargo check -p duet-score -p duet-dsp --locked`. Expected result:
   exit 0. The command fails before this chunk, because neither package exists.
9. `git add` the write scope and `git commit`. The native hook runs `scripts/dod.sh`.

## Tests

This chunk writes no test. Its Completion command is the check (SM3): `cargo check -p duet-score -p
duet-dsp --locked` fails before the chunk, because `cargo` reports two unknown packages, and it
passes after the chunk. `cargo xtask check-manifests`, which chunk M0 added to `scripts/dod.sh`, is
the mechanical guard over both new manifests, and the native hook runs it on the commit.

## Verification

```
cargo check -p duet-score -p duet-dsp --locked
cargo xtask check-manifests
cargo info rustfft@6.4.1
cargo info realfft@3.5.0
```

Expected output: the first command exits 0 and prints two `Checking` lines. The second command exits
0. The last two commands print the published feature list of each pin, and the commit body carries
both. **No `cargo tree` command belongs in this chunk**, for the reason step 7 states.

Then commit on a branch named `chunk/m1-manifest-phase-1`. The native git hook runs
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
