# ADR-0011: The DDD path grammar, eight bounded contexts, and front matter

## Status

Accepted. The operator decided the adoption on 2026-09-25, while the program was paused after phase 1.
The move of the landed code and the rewrite of every plan path land in one plan act. Chunk M94 lands
the guard.

## Context

The ultravisor repository publishes a path grammar for a Domain-Driven Design tree
(`packages/ddd-path`), a front-matter format (`packages/ddd-meta`), and a migrator
(`packages/ddd-migrate`). The grammar puts exactly three facts in a path: the bounded context
(`bc_<name>`), the language root (`lang_<language>`), and the test tree. It puts the DDD layer, the
tactical pattern, the subdomain, and the tags in front matter, because a change of classification
must be a one-line diff and never a file move. It puts every fact that code can compute (exports,
imports, dependencies) in neither place.

Duet had sixteen crates at `crates/<crate>/`, with no context in the path. The architecture named the
crates "bounded contexts", but a crate is a build unit, and several crates share one model and one
language. `duet-time` is a shared kernel, `duet-command` is a published language, and `duet-core` is
an application service; none of them is a context.

## Decision

1. **Layout.** A crate lives at `crates/bc_<context>/<package>/lang_rust/`, and a tool at
   `tools/bc_<context>/<package>/lang_rust/`. `Cargo.toml`, `src/`, `tests/`, and `benches/` sit in
   `lang_rust/`. The workspace `members` globs are `crates/*/*/lang_rust` and `tools/*/*/lang_rust`.
2. **The unit segment is the Cargo package name, verbatim.** It is present in a one-crate context too,
   so a context gains a crate with no file move. A package name therefore never holds `_`: the grammar
   reads `x_y` above the shell as a structural kind and refuses an unknown one.
3. **A unit file that is not Rust sits beside `lang_rust/`.** The grammar refuses a file at the
   `lang_rust/` root that no shell declares (`file_outside_shell`), so the Bravura font, the macOS
   `Info.plist`, a per-crate `CLAUDE.md`, and a future `build.rs` sit in the unit directory. Cargo
   reaches a build script there with `build = "../build.rs"`.
4. **Eight product contexts, acyclic** (architecture section 1.2, "The bounded context map"):
   `time`, `document`, `notation`, `audio`, `midi`, `project`, `gateway`, `app`, plus the tool contexts
   `plan_store` and `repo_guard`. Crate names, `use` paths, and every `cargo -p` command do not change.
5. **`.ddd/grammar.toml`** declares the Rust shell: `src` is the source root, `benches` is an extra
   source root, and `tests` and `proptest-regressions` are the test root with the kind `integration`.
6. **Front matter and its gate arrive together, in chunk M94.** The guard ports the grammar subset
   and the `block` and `hash` carriers to Rust, pins the port to upstream with two committed parity
   fixtures, and requires a block on each graded `.rs` file that a commit changes. The subdomain is
   declared once per context in `.ddd/context-map.toml` and never in a file.
7. **In `architecture.md`, `crates/duet` stays the NAME of the application crate row.** Four guard
   sites read that token as a name. Every path token that ends in `/` moved to the new layout.

## Alternatives rejected

- **One context per crate** (`crates/bc_engine/lang_rust/`). It is flatter, but it labels a shared
  kernel, a published language, and an application service as contexts, and the containment tree then
  says nothing about the domain.
- **The first grouping of this act, with `bc_gateway` holding `duet-command`, `duet-core`, and
  `duet-agent`.** The Engineering Critic and the Software Architect both showed four two-way context
  edges: nine crates depend on `duet-command`, and `duet-core` depends on almost every crate. A context
  pair with edges both ways has no upstream, and the `bc_` segment is the most costly segment to change.
  `duet-command` moved to `bc_document` with the two aggregates whose commands it wraps.
- **Front matter on every file in the move itself.** With no checker, hand-written blocks are unchecked
  declarations: a wrong layer value is invisible, and the first run of the guard would be noise.
- **The move as a plan chunk.** It rewrites the write scope of every chunk, so it shares a write-scope
  path with every chunk of any phase, which rule 4 of `check-plan-graph` refuses by construction. It
  landed as a plan act with the fleet paused, the way the phase-1 escalation repairs landed.
- **Running `ddd-migrate apply`.** Its `ratify` step is the operator's act and records the reviewer. The
  move here is a plain `git mv` of six units, and the upstream `classify` checked every result path.

## Consequences

- Every path of every chunk moved. PG40 now fails closed on a path that does not parse as
  `crates/bc_<context>/<crate>/` and on a crate whose context disagrees with the map.
  `check-plan-graph` refuses a `crates/` or `tools/` write-scope path outside the layout.
- The changed-path filters of `scripts/dod.sh` and `.github/workflows/ci.yml` name the new paths, and
  two probes fail when either filter names no live path.
- The fleet stays paused until the pull request merges. An in-flight branch (chunk T2) rebases onto
  the move, and afterwards `git ls-files crates/duet-score` must print nothing.
- Until M94 lands, the `members` globs, CG1b, PG40, and the new write-scope check are the only
  mechanical cover of the layout.
