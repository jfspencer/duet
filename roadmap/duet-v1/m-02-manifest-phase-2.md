---
id: M2
line: M
depends_on: [M1, T2, D1]
write_scope:
  - Cargo.toml
  - Cargo.lock
  - crates/bc_document/duet-session/lang_rust/Cargo.toml
  - crates/bc_document/duet-session/lang_rust/src/lib.rs
  - crates/bc_notation/duet-engrave/lang_rust/Cargo.toml
  - crates/bc_notation/duet-engrave/lang_rust/src/lib.rs
  - crates/bc_audio/duet-analysis/lang_rust/Cargo.toml
  - crates/bc_audio/duet-analysis/lang_rust/src/lib.rs
parallelism: serial-only: SM1 runs the manifest chunk alone before every line chunk of its phase.
completion: "cargo check -p duet-session -p duet-engrave -p duet-analysis --locked exits 0; commit SHA on a branch chunk/m2-manifest-phase-2"
---

# M2: The phase-2 manifest

Verify the current state of the files in the write scope; report a discrepancy and stop, instead of
proceeding.

This chunk opens phase 2. It creates the `duet-session`, `duet-engrave`, and `duet-analysis`
skeletons. It implements architecture section 13.1 (the M2 row), section 13.0 rules SM1 and SM5.
Section 13.3 puts M1, T2, and D1 in phase 1, so this chunk starts after all three land.

**This phase adds no pin.** The M2 cell of section 13.1 states it plainly: the root
`[workspace.dependencies]` table needs no new THIRD-PARTY entry, because every crate that phase 2
uses is already pinned. **It does need three internal path entries** (SM1 rule 4), so the expected
diff on the root manifest is three lines and not an empty one. The root `Cargo.toml` stays in the
write scope, because section 13.0 makes a named file the unit of write scope and SM4 states the
table split.

The chunk does exactly four things (SM1): it pins, which is no third-party name this time, it
creates three skeletons with no `[dependencies]` section, it adds one root path entry per skeleton
(SM1 rule 4), and it settles `Cargo.lock`. Dispatch: **Duet
Engineer**. This chunk carries no policy file, so SM4 does not route it to the Orchestrator.

## Files

| Path | Action |
|---|---|
| `Cargo.toml` | modify (`[workspace.dependencies]` only; the expected diff is the three internal path entries) |
| `Cargo.lock` | modify |
| `crates/bc_document/duet-session/lang_rust/Cargo.toml` | create |
| `crates/bc_document/duet-session/lang_rust/src/lib.rs` | create |
| `crates/bc_notation/duet-engrave/lang_rust/Cargo.toml` | create |
| `crates/bc_notation/duet-engrave/lang_rust/src/lib.rs` | create |
| `crates/bc_audio/duet-analysis/lang_rust/Cargo.toml` | create |
| `crates/bc_audio/duet-analysis/lang_rust/src/lib.rs` | create |

## Types and signatures

This chunk declares no Rust type. It writes three crate roots and three member manifests. Each
manifest carries the eight `*.workspace = true` package fields, a `description`, and
`[lints] workspace = true`, and no `[dependencies]` section (SM1 rule 2).

### The three internal path entries (SM1 rule 4)

```toml
duet-analysis = { path = "crates/bc_audio/duet-analysis/lang_rust" }
duet-engrave = { path = "crates/bc_notation/duet-engrave/lang_rust" }
duet-session = { path = "crates/bc_document/duet-session/lang_rust" }
```

A member crate reaches an internal crate with `duet-<crate> = { workspace = true }`, and that entry
resolves only when the root table already carries the path entry. SM4 keeps the root manifest out of
every line chunk's reach, so the chunk that creates the skeleton is the one chunk that can add it.
Revision 23 gave the internal entries to no chunk at all, and five chunk authors each reported the
same hole.

### The `duet-session` skeleton

```toml
[package]
name = "duet-session"
description = "The Duet session and mix documents: tracks, takes, regions, strips, slots, and automation curves."
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
//! The Duet session and mix documents.
//!
//! A region names an immutable source and never rewrites audio. Sections 6
//! and 7 of `roadmap/duet-v1/architecture.md` state the design.

#![forbid(unsafe_code)]
```

### The `duet-engrave` skeleton

```toml
[package]
name = "duet-engrave"
description = "The Duet engraver: pure layout from a score to glyph, quad, and path placements."
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
//! The Duet engraver.
//!
//! It turns a score into glyph, quad, and path placements. It is pure: it
//! opens no file, it spawns no task, and it names no framework type.
//! Section 10.5 of `roadmap/duet-v1/architecture.md` states the design.

#![forbid(unsafe_code)]
```

### The `duet-analysis` skeleton

```toml
[package]
name = "duet-analysis"
description = "The Duet analysis crate: pYIN pitch tracking and loudness measurement."
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
//! The Duet analysis crate.
//!
//! It holds the in-house pYIN pitch tracker and the loudness reader.
//! Section 15.8 of `roadmap/duet-v1/architecture.md` states the type roster.

#![forbid(unsafe_code)]
```

Chunk T3 adds the `duet-session` entries, chunk A1 adds the `duet-engrave` entries, and chunk E1
adds the `duet-analysis` entries, each in the same commit as the code that uses them (SM1).

## Steps

1. Confirm that M1, T2, and D1 landed: `crates/bc_document/duet-score/lang_rust` and `crates/bc_audio/duet-dsp/lang_rust` each hold source
   beyond the skeleton, and `cargo nextest run -p duet-score --no-tests=fail` passes. Confirm that
   `crates/bc_document/duet-session/lang_rust`, `crates/bc_notation/duet-engrave/lang_rust`, and `crates/bc_audio/duet-analysis/lang_rust` do not exist. Report a
   discrepancy and stop if any one is false.
2. Read the root `Cargo.toml` and confirm that every third-party crate phase 2 needs is already
   pinned. Add no third-party entry. **If a phase-2 chunk needs a pin this table lacks, report the
   discrepancy to the Architect and stop**, because SM1 rule 1 makes the manifest chunk the one
   owner of a pin and no line chunk may add one. Add the three internal path entries
   `duet-analysis`, `duet-engrave`, and `duet-session` to the same table (SM1 rule 4), in
   alphabetical order with the entries it already holds.
3. Create the three member manifests and the three crate roots with the six blocks the section above
   gives.
4. Run `cargo build --workspace` (SM5 rule 3). Expected result: the build succeeds and `Cargo.lock`
   gains one `[[package]]` entry for each new crate.
5. Run `cargo xtask check-manifests`. Expected result: exit 0 over every member, the three new ones
   included.
6. Run the Completion command:
   `cargo check -p duet-session -p duet-engrave -p duet-analysis --locked`. Expected result: exit 0.
   The command fails before this chunk, because `cargo` reports three unknown packages.
7. `git add` the write scope and `git commit`. The native hook runs `scripts/dod.sh`.

## Tests

This chunk writes no test. Its Completion command is the check (SM3):
`cargo check -p duet-session -p duet-engrave -p duet-analysis --locked` fails before the chunk and
passes after it. `cargo xtask check-manifests`, which chunk M0 added to `scripts/dod.sh`, is the
mechanical guard over the three new manifests, and the native hook runs it on the commit.

## Verification

```
cargo check -p duet-session -p duet-engrave -p duet-analysis --locked
cargo xtask check-manifests
git diff --stat Cargo.toml
```

Expected output: the first command exits 0 and prints three `Checking` lines. The second command
exits 0. The third command prints nothing, which proves that this phase added no pin and that the
root manifest is unchanged.

Then commit on a branch named `chunk/m2-manifest-phase-2`. The native git hook runs
`scripts/dod.sh`, and the commit lands only when every gate passes.

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
