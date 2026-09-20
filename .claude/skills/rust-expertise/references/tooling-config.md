# Tooling Configuration

## `clippy`

Always-on. Configure in `Cargo.toml` under `[lints.clippy]` (Rust 1.74+) or via `[workspace.lints.clippy]` for workspaces, preferred over the older `#![warn(clippy::...)]` attributes in source:

```toml
[lints.clippy]
all = "warn"
pedantic = "warn"  # advisory; review case-by-case
nursery = "warn"
undocumented_unsafe_blocks = "deny"
await_holding_lock = "deny"
```

CI runs `cargo clippy --all-targets --all-features -- -D warnings`. No exceptions: either fix the lint or annotate `#[allow(clippy::...)]` with a `// reason: ...` comment.

## `rustfmt`

Always-on. `cargo fmt --check` is part of CI. Configure via `rustfmt.toml`:

```toml
edition = "2024"
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
```

## Other tools

- `cargo-deny`, license + security audit. Configure `deny.toml` with license allowlist (MIT, Apache-2.0, etc.), RustSec advisory database checks, banned crates / minimum versions, source allowlist. CI runs `cargo deny check`.
- `cargo-audit`, subset of cargo-deny focused on RustSec advisories. Use alongside cargo-deny in a separate CI job that runs daily on `main`.
- `cargo-outdated`, detect dependencies with newer versions. Run periodically; bump deliberately.
- `cargo-machete`, detect unused dependencies in `Cargo.toml`. Useful for hygiene PRs and CI gating in monorepos.
- `cargo-flamegraph`, profiling: `cargo flamegraph --bin myapp` produces an SVG flame graph. Pair with criterion benchmarks to identify hot paths.
- `cargo-expand`, show post-expansion source for any module: `cargo expand path::to::module`. Indispensable for debugging derive macros, declarative macros, and `tracing::instrument` expansions.
- `bacon`, modern background runner that watches files and re-runs `cargo check` / `clippy` / `test` on every save. Successor to `cargo-watch` with better terminal UX.

## Linker speedup: `mold` / `lld`

Default linker is slow on large workspaces. Switch to `mold` (Linux) or `lld` (cross-platform) via `.cargo/config.toml`:

```toml
[target.x86_64-unknown-linux-gnu]
linker = "clang"
rustflags = ["-C", "link-arg=-fuse-ld=mold"]
```

Cuts incremental link times by 5-20× for large binaries.

## Compile cache: `sccache`

Caches compilation artifacts across CI runs and local rebuilds:

```toml
# ~/.cargo/config.toml
[build]
rustc-wrapper = "sccache"
```

Combine with GitHub Actions cache or S3 backend for shared CI cache.

## Module-level lint configuration

In modern Rust (1.74+), prefer `[lints]` in `Cargo.toml` over crate-level attributes, it's centralized, inheritable from the workspace, and avoids cluttering source files:

```toml
# Cargo.toml
[lints.rust]
missing_docs = "warn"
rust_2018_idioms = "warn"
missing_debug_implementations = "warn"
unsafe_op_in_unsafe_fn = "deny"

[lints.clippy]
all = "warn"
pedantic = "warn"
undocumented_unsafe_blocks = "deny"
await_holding_lock = "deny"
```

Crate-level attributes still have a place: keep them for things that aren't lints (`#![no_std]`, `#![cfg_attr]`-gated items) and for crate-level documentation.

```rust
//! Crate-level docs at the top of lib.rs.

#![cfg_attr(not(test), forbid(unsafe_code))]  // strongest: ban unsafe outside tests
```
