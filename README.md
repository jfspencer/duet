# Duet

A desktop application built in Rust on [GPUI Kit](https://gpui-kit.com), with an autonomous engineering fleet (Claude Code, Codex, OpenCode) wired in from day one.

## Layout

```text
crates/duet/          the app (binary `duet`)
tools/plan-db/        LMDB plan-store CLI for the agent fleet
tools/xtask/          `cargo xtask sync-agents` — generates the Codex/OpenCode agent mirrors
roadmap/              plans (one directory per plan; the plan store is keyed on the path)
scripts/dod.sh        the ONLY Definition of Done gate (fmt, clippy, doc, tests, deny, machete, agents, hooks)
.githooks/            versioned pre-commit + pre-push hooks -> scripts/dod.sh
.claude/              canonical agents, skills, hooks, plan coordination
.codex/  .opencode/   generated adapters over .claude (config + mirrors)
```

## Setup

```sh
scripts/bootstrap.sh     # toolchain, cargo-deny, cargo-machete, cargo-nextest, typos, plan-db, git hooks
cargo run -p duet        # run the executable directly
scripts/dod.sh --plan    # see the gate
```

`rust-toolchain.toml` pins the compiler. `Cargo.toml` carries the strict `[workspace.lints]` policy, `.cargo/config.toml` makes every warning an error, `clippy.toml` holds the API bans, `deny.toml` the dependency policy, `rustfmt.toml` the formatter.

On macOS, build and open the application bundle to use the Duet icon in Finder and the Dock:

```sh
scripts/package-macos.sh
open target/debug/Duet.app
```

Use `scripts/package-macos.sh --release` for a release bundle. The script prints the bundle path and respects Cargo's target directory.
The bundle uses the approved icon in `crates/duet/assets/icons/duet.icns`.
Direct `cargo run` launches the executable without macOS bundle metadata or its application icon.

## Agents

`.claude/agents/engineering/` is the canonical fleet: Hypervisor, Orchestrators, one Rust implementer (Duet Engineer), Architect, Product Manager, Designer, Engineering Critic, Security Reviewer, Task Runner, Memory Agent. After editing any agent:

```sh
cargo xtask sync-agents          # regenerate .codex/agents and .opencode/agents
cargo xtask sync-agents --check  # what the gate runs
```

Durable coordination between agents is an LMDB store at `~/.claude/plan-dbs/<repo>__<plan>/`, accessed only through `.claude/plan-coordination/db.sh`. See `.claude/plan-coordination/README.md`.

## License

MIT. See `LICENSE`.
