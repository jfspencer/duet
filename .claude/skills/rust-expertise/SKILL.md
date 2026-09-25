---
name: rust-expertise
description: "Use when writing, modifying, debugging, or reviewing any Rust code where the language is being leveraged for its strengths: ownership, fearless concurrency, performance, type-driven design. This is the Rust generalist skill covering ownership and lifetimes, error handling (thiserror/anyhow), async with tokio, multithreading (rayon, crossbeam, atomics), trait design, module organization, testing (cargo test, mockall, proptest, criterion), tooling (clippy/rustfmt/cargo-deny), and performance patterns. Pull in for every crate in this workspace: the GPUI Kit app (`crates/bc_app/duet/lang_rust`, together with the `gpui-kit` skill), the LMDB plan store CLI (`tools/bc_plan_store/plan-db/lang_rust`), and the repo automation (`tools/bc_repo_guard/xtask/lang_rust`)."
allowed-tools: Bash(cargo:*) Bash(rustc:*) Bash(grep:*) Bash(find:*) Read Grep Edit
verified: 2026-09-20
verified-against:
  - .claude/skills/rust-expertise/references/async-tokio.md
  - .claude/skills/rust-expertise/references/concurrency.md
  - .claude/skills/rust-expertise/references/error-handling.md
  - .claude/skills/rust-expertise/references/testing-patterns.md
  - .claude/skills/rust-expertise/references/tooling-config.md
  - .claude/skills/rust-expertise/references/traits-and-generics.md
  - .claude/skills/rust-expertise/references/unsafe-and-drop.md
  - tools/bc_plan_store/plan-db/lang_rust/src/main.rs
  - Cargo.toml
  - clippy.toml
length-exception: Rust generalist reference covering ownership, lifetimes, error handling, async with tokio, multithreading, trait design, testing matrix, tooling, and performance patterns; the index covers the full Rust surface so every crate in the workspace can route into the right section; ceiling raised to 400 lines, with detailed pedagogical examples extracted to references/ subdirectory
review-cadence: quarterly
---

# Rust Expertise

This is the Rust generalist skill, runtime-agnostic where possible, opinionated where the repo has a stake. Pull it in any time you're doing more than the bare minimum of Rust:

- The GPUI Kit desktop app (`crates/bc_app/duet/lang_rust`; pair with the `gpui-kit` skill for the UI mechanics)
- Standalone Rust binaries / CLIs (`tools/bc_plan_store/plan-db/lang_rust`, `tools/bc_repo_guard/xtask/lang_rust`, and any future crate)
- Rust microservices, sidecars, or daemons
- Rust libraries that are not wrappers

This repo is Rust-only, so every code change pulls this in. The workspace lint policy (`[workspace.lints]` in the root `Cargo.toml`, `clippy.toml`, `deny.toml`) is the enforced floor; this skill is the judgement above it.

## When NOT to use

- Choosing or wiring a GPUI component, entity, action, or test window. That is the `gpui-kit` skill; pull both in together for app work.
- Any task that is not actually writing Rust code (TypeScript wiring around a binding, JS-side tests, contract schemas).
- Deciding where a test lives or how a scratch directory is disposed. That is the `test-author` skill.

Use this skill when Rust is being leveraged for its strengths: ownership, fearless concurrency, performance, type-driven design.

## Philosophy

**Ownership is the API.** Most Rust design problems disappear when the type system expresses ownership transfers, borrows, and lifetimes correctly. Fighting the borrow checker usually signals a design problem, not a code problem.

**Types are documentation.** Encode invariants in the type system: `NonZeroU32`, `&str` vs `String`, `Cow<'_, str>`, newtype wrappers (`UserId(u64)`), typestate (`Connection<Open>` vs `Connection<Closed>`). A reader should be able to derive correctness from the signatures.

**Zero-cost abstractions, but pay attention.** Rust gives abstraction without runtime cost, `Iterator::map` compiles to the same code as a hand-written loop, `Box<dyn Trait>` is one indirection, generics monomorphize. But "zero-cost" is a property of the optimizer, not the source. Profile before assuming.

**Fearless concurrency, but structured.** `Send` + `Sync` + the borrow checker prevent data races, but they don't prevent deadlocks, livelocks, or unstructured concurrency. Adopt structured concurrency: every spawned task has a parent that waits for it, cancellation propagates downward through the task tree, and no task outlives the scope that owns it. Use tokio for I/O, rayon for CPU, channels for communication, and structured spawning (`JoinSet`, `std::thread::scope`), never bare `tokio::spawn` followed by "and I hope it finishes."

**Errors are values.** `Result<T, E>` is the only error mechanism. `panic!` is for programmer bugs, not runtime conditions. Every fallible operation returns `Result`; every infallible one says so by not returning `Result`.

**Read the docs locally.** Rust's stdlib and the major crates ship with excellent rustdoc, `cargo doc --open -p tokio` renders the *exact* version pinned in your `Cargo.lock`, which is faster and more accurate than the public website. For any non-trivial API question, prefer local docs; for design questions, read the source, it's usually clearer than the prose.

## Decision Framework

### Memory & ownership

| Decision | Choose | Over | Why |
|---|---|---|---|
| Owned vs borrowed string | `&str` (parameter), `String` (return) | always `String` | Borrows avoid allocation; return owns to escape the function |
| Owned vs borrowed slice | `&[T]` (parameter), `Vec<T>` (return) | always `Vec<T>` | Same reasoning |
| Maybe-borrowed | `Cow<'_, str>` | `String` | When the function sometimes allocates and sometimes doesn't |
| Allocation-free collection | `SmallVec<[T; N]>` (mostly stack), `arrayvec::ArrayVec<T, N>` (always stack) | `Vec<T>` | When N is small and known |

### Reference counting & shared mutability

| Decision | Choose | Over | Why |
|---|---|---|---|
| Single-thread shared | `Rc<T>` | `Arc<T>` | Atomic ops aren't free; use Arc only if `Send` is needed |
| Multi-thread shared | `Arc<T>` | `Rc<T>` | Required for `Send` |
| Mutable shared (sync) | `Arc<Mutex<T>>` (general), `Arc<RwLock<T>>` (read-heavy) | `Arc<RefCell<T>>` | RefCell is `!Sync`; choose lock by access pattern |
| Mutable shared across `.await` | `Arc<tokio::sync::Mutex<T>>` | `Arc<std::sync::Mutex<T>>` | Sync mutex held across `.await` is a bug (clippy: `await_holding_lock`) |

### Locks & one-time init

| Decision | Choose | Over | Why |
|---|---|---|---|
| Lock library (sync) | `parking_lot::Mutex` / `RwLock` | `std::sync::Mutex` | Faster, no poisoning, smaller, for everything except std-only crates |
| One-time init (caller-provided) | `OnceLock<T>` (1.70+) | `once_cell::sync::OnceCell` | std primitive |
| Lazy init (closure-defined) | `LazyLock<T>` (1.80+) | `lazy_static!`, `once_cell::sync::Lazy` | std primitive; the canonical replacement for `lazy_static!` |

### Async & concurrency

| Decision | Choose | Over | Why |
|---|---|---|---|
| Async runtime | GPUI's own executor inside the app (`cx.spawn`, `cx.background_spawn`); `tokio` only in a standalone tool that needs it | `async-std`, `smol` | The app already owns an executor; a second runtime in the same process is a defect |
| CPU parallelism | `rayon` | manual `std::thread` | Work-stealing pool, ergonomic `par_iter` |
| Scoped threads | `std::thread::scope` (1.63+) | `crossbeam::scope` | Stdlib equivalent, no extra dependency |
| Channel (1↔1) | `tokio::sync::oneshot` | `tokio::sync::mpsc` (size 1) | Single-message, allocation-free, request/response |
| Channel (M→1, async) | `tokio::sync::mpsc` (bounded) | unbounded variants | Bounded exposes backpressure |
| Channel (M↔N, sync) | `crossbeam::channel` | `std::sync::mpsc` | Faster, true MPMC, more featured |
| Channel (broadcast) | `tokio::sync::broadcast` | manual fan-out | Built-in lagging detection |
| Channel (latest-value) | `tokio::sync::watch` | mpsc + dedupe | "Subscribe to the latest" semantic |

### Dispatch & generics

| Decision | Choose | Over | Why |
|---|---|---|---|
| Trait dispatch | `impl Trait` (parameter), `Box<dyn Trait>` (storage) | always `dyn` | Static dispatch is faster; `dyn` when storage requires a single type |
| Generic vs associated type | associated type (one impl per type), generic (multiple) | always one or the other | `Iterator::Item` is associated; `From<T>` is generic |
| Async in traits | native `async fn` in traits (1.75+) | `async-trait` crate | Native is allocation-free; reach for `async-trait` only when `dyn Trait` storage is required |

### Errors

| Decision | Choose | Over | Why |
|---|---|---|---|
| Library error type | `thiserror::Error` | hand-rolled enum, `Box<dyn Error>` | Stable taxonomy, derive impls |
| Binary / app error type | `anyhow::Result` | `thiserror` | Adds context, no need for stable taxonomy |

### Ecosystem defaults

| Decision | Choose | Over | Why |
|---|---|---|---|
| Logging / tracing | `tracing` | `log` | Spans, structured fields, async-friendly |
| Serialization | `serde` (with `serde_derive`) | hand-rolled | Universal: JSON, bincode, postcard, etc. |
| String parsing | `winnow` | `nom`, regex (for structured grammars) | `winnow` is the maintained successor to `nom` |
| HTTP client | `reqwest` | `hyper` directly | `reqwest` is the high-level client; `hyper` is for servers |
| HTTP server | `axum` | `actix-web`, `warp`, `rocket` | tokio-native, tower middleware, type-driven extractors |
| CLI parsing | `clap` (derive) | hand-rolled | Industry standard; derive macro is excellent |

## Project Structure

A non-trivial Rust crate organizes around clear module boundaries:

Crate layout: `Cargo.toml` at the root; `src/lib.rs` (public API surface, re-exports), `src/main.rs` (binary entrypoint when applicable), `src/error.rs` (crate-wide thiserror), `src/config.rs` (serde config), `src/{feature}.rs` plus a `src/{feature}/` directory of private submodules per logical area (`mod_module_files` is denied in this workspace, so never `mod.rs`) with a `#[cfg(test)] mod tests` at the bottom of each file. Top-level dirs: `tests/` (integration; each `.rs` is its own binary), `benches/` (criterion), `examples/` (runnable with `cargo run --example`), and `build.rs` for compile-time generation or link directives.

**Module visibility.** Use `pub(crate)` and `pub(super)` aggressively. The default `pub` exposes through to consumers; most internal helpers should not be public. Re-export the public surface via `pub use` from `lib.rs` rather than scattering `pub` across internal modules.

**`lib.rs` vs `main.rs`.** A binary crate that can also be a library should have both, keep `main.rs` thin (parse CLI, call into lib), put logic in `lib.rs`. This makes the logic testable and reusable.

**Edition.** New crates use `edition = "2024"` (stabilized in Rust 1.85, Feb 2025). Key 2024-edition behaviors: `unsafe_op_in_unsafe_fn` lints by default, `if let` temporaries are scoped to the arm, `gen { ... }` blocks are reserved syntax, lifetime capture rules tightened (`use<>` syntax). Older crates can stay on 2021 if migration is costly.

**Workspace members.** For multi-crate projects, use a Cargo workspace. Common pattern: `core` (pure logic), `cli` (binary wrapping core), `bench` (benchmarks against core). Workspace `Cargo.toml` essentials: `[workspace] resolver = "2"`, members list, `[workspace.package]` sharing `edition`, `rust-version`, `license`. Pin shared dependency versions in `[workspace.dependencies]`. Set repo-wide lint floors in `[workspace.lints.rust]` and `[workspace.lints.clippy]`. Member crates inherit with `package.edition.workspace = true`. This eliminates version skew across the workspace.

**Toolchain pinning.** Reproducible builds need a pinned compiler, drop a `rust-toolchain.toml` at the repo root with `[toolchain] channel = "1.85.0"`, `components = ["rustfmt", "clippy", "rust-src"]`, `profile = "minimal"`. `rustup` reads this and installs the right toolchain on first build. CI must use the same file or a matching `--toolchain` flag.

**`Cargo.lock` policy.** Commit `Cargo.lock` for binaries and applications (reproducible deploys). Commit it for libraries too, modern advice (since cargo 1.78+) is to lock libraries as well so contributors and CI build the same versions. The lockfile is ignored when the crate is consumed as a dependency, so this doesn't constrain downstream users.

## Error Handling

Two libraries, two contexts:

- **Libraries** use `thiserror::Error`, define a stable error enum per logical scope, `#[from]` to bridge other error types, `#[non_exhaustive]` to preserve forward compatibility.
- **Binaries / apps** use `anyhow::Result<T>`, built-in `.context(...)` chaining, no stable error taxonomy required.

The `?` operator unwraps `Result<T, E>` to `T`, returning `Err(e.into())` on error. The conversion uses `From<E>` for the function's return error. Pair with `#[from]` on thiserror variants to make the bridging automatic.

Don't use `Box<dyn Error>` in libraries, it's a footgun: consumers can't downcast reliably, can't inspect specific failure modes, can't add context cleanly. Use `thiserror` for libraries; reserve `Box<dyn Error>` (or `anyhow::Error`) for binaries.

Panics are for programmer bugs (array index out of bounds, broken invariants, "this should never happen"). Anything that depends on runtime state, bad input, network failure, missing file, is a `Result`. `unwrap()` and `expect()` are panics. They're acceptable in tests, `LazyLock`/`OnceLock` initializers where the value must succeed at startup and there is no recovery path (fail-fast at process start), and cases where you've just constructed the value and a panic indicates a logic bug. Production code paths reachable from a network or file boundary should never `unwrap()` directly.

Full examples (thiserror enum, anyhow context chains, walking error sources, `#[non_exhaustive]`, `#[track_caller]`) in [references/error-handling.md](references/error-handling.md).

## Async with tokio

Use `#[tokio::main]` for a standalone binary that needs it (`flavor = "multi_thread"` for production, `"current_thread"` for CLI tools / deterministic tests); explicit runtime construction for libraries. Inside the GPUI app, GPUI owns the executor (`cx.spawn`, `cx.background_spawn`); don't construct your own.

Always handle the `JoinHandle<T>` from `tokio::spawn`, either `await` it or use `JoinSet` for managed groups. Orphaned tasks become silent if they panic. Use `tokio::task::JoinSet` for task groups with await-all / cancel-on-first-failure semantics. Use `tokio::select!` for racing futures, cancellation, timeouts, multi-channel watching, but mind cancellation safety: each branch's future may be dropped if another branch wins. Don't use `select!` with non-cancel-safe futures.

Channels:

| Type | Use case |
|---|---|
| `tokio::sync::mpsc` | M producers → 1 consumer. Bounded (`channel`) or unbounded (`unbounded_channel`). Default: bounded with explicit capacity for backpressure. |
| `tokio::sync::oneshot` | Single message, request/response. The receiver is consumed on first recv. |
| `tokio::sync::broadcast` | 1 producer → M consumers, every consumer sees every message (with lag detection). |
| `tokio::sync::watch` | 1 producer → M consumers, every consumer sees the LATEST value (intermediate values dropped). |
| `crossbeam::channel` | Synchronous M↔N. Use in non-async contexts (rayon tasks, std threads). |

Default to bounded channels. Unbounded channels mask backpressure and crash with OOM under load.

**Holding a sync mutex across `.await` is a bug.** The future may move threads, the mutex won't follow, and cancellation can leave the mutex permanently locked. Clippy's `await_holding_lock` catches this. Use `tokio::sync::Mutex` for state held across `.await`; `std::sync::Mutex` / `parking_lot::Mutex` for state held briefly within sync code.

Use `tokio_util::sync::CancellationToken` for cooperative cancellation across an actor / task tree. The standard graceful shutdown pattern: listen for SIGINT/SIGTERM via `tokio::signal::ctrl_c()`, cancel the root token, `JoinSet::shutdown()` to wait, flush, exit.

`Send` bound contagion: async fns spawned onto a multi-threaded runtime must produce `Send` futures. Common offenders: `Rc<T>` (use `Arc<T>`), `RefCell<T>` (use `Mutex<T>`), raw pointers, `MutexGuard` from `std::sync::Mutex` held across `.await`. For single-threaded workloads, use `tokio::task::LocalSet` and `spawn_local`.

Native `async fn` in traits stabilized in Rust 1.75 (Dec 2023). Default to native syntax. For `Send` bounds, prefer `trait-variant` crate, then explicit `Pin<Box<dyn Future + Send>>`, then `async-trait` (only when `dyn Trait` storage is required). Native is NOT object-safe by default.

Full examples (spawn, JoinSet, select!, mutex pattern, CancellationToken, async traits, `trait-variant`) in [references/async-tokio.md](references/async-tokio.md).

## Multithreading

| Workload | Tool |
|---|---|
| I/O-bound concurrency | tokio |
| CPU-bound parallelism | rayon |
| Long-running OS threads | `std::thread` (rare; usually rayon or tokio fits) |
| Embarrassingly parallel data | `rayon::par_iter` |
| Pipelined work stages | crossbeam channels + scoped threads |
| Lock-free counters | atomics |

`rayon::par_iter()` parallelizes any `IntoParallelIterator`; the pool auto-sizes to CPU count. For more control: `rayon::scope`, `rayon::join`, custom thread pools. **rayon under tokio**: wrap rayon work in `tokio::task::spawn_blocking`. Never call rayon directly inside an async task, it starves the runtime. Inside the GPUI app, CPU-heavy work goes through `cx.background_spawn` and notifies the owning entity when it lands (see the `gpui-kit` skill's async reference).

Use `std::thread::scope` (1.63+) for threads that need to borrow from the parent stack, it guarantees all spawned threads join before the scope returns. Prefer over `crossbeam::scope` for new code (no extra dependency, identical semantics).

`crossbeam::channel` is a faster, more featured `std::sync::mpsc` (true MPMC, bounded and unbounded variants). Other crossbeam pieces: `crossbeam::deque` (work-stealing), `crossbeam::epoch` (lock-free memory reclamation), `crossbeam::utils::CachePadded<T>` (cache-line padding for false-sharing avoidance).

Atomic memory orderings: `Relaxed` (atomicity only), `Acquire`/`Release` (paired publish-consume), `AcqRel` (CAS loops), `SeqCst` (total order, slowest, default when a weaker ordering is not justified). The Rustonomicon's chapter on atomics is the canonical reference; read it before designing lock-free data structures.

One-time init in 2026: `std::sync::LazyLock<T>` (1.80+, closure-defined) is the canonical replacement for `lazy_static!` and `once_cell::sync::Lazy`. `std::sync::OnceLock<T>` (1.70+) for caller-driven init when the value isn't known at definition time. Use `once_cell` only on Rust < 1.80; do not introduce new uses of `lazy_static!`.

Full examples (rayon, scoped threads, atomics, LazyLock, OnceLock) in [references/concurrency.md](references/concurrency.md).

## Unsafe and Drop

Most production Rust crates can and should be 100% safe. Reach for `unsafe` only when crossing FFI to C / system APIs, lock-free data structures with atomics, manual memory management (custom allocators, intrusive lists, arena buffers), or performance-critical hot loops where bounds checks dominate (rare; profile first). Without one of those reasons, `unsafe` is not needed.

The discipline:

- **Encapsulate unsafe in safe wrappers.** Smallest possible `unsafe { ... }` block per named invariant.
- **Every `unsafe` block has a `// SAFETY:` comment.** Non-negotiable. Clippy's `undocumented_unsafe_blocks` lint enforces.
- **`#[deny(unsafe_op_in_unsafe_fn)]`** at the crate root (default in 2024 edition). Inside an `unsafe fn`, every unsafe operation must still be wrapped in its own `unsafe { ... }` block with its own SAFETY comment.
- **Public `unsafe fn` requires a `# Safety` doc section** documenting the caller's obligations.
- **`#![forbid(unsafe_code)]`** at the crate root for crates that genuinely need no unsafe.

Tooling: `cargo +nightly miri test` (UB interpreter), AddressSanitizer via `RUSTFLAGS="-Z sanitizer=address" cargo +nightly test` (FFI bugs Miri can't see), `cargo expand` (review macro-emitted unsafe).

Rust's resource management is RAII: a value owns its resources and releases them on scope exit via the `Drop` trait. File handles, sockets, locks, allocations, FFI handles, all rely on `Drop`. Drop discipline:

- **Drop must not panic.** A panic during drop while another panic is unwinding aborts the process. Log via `tracing`, no propagation.
- **Drop order is field declaration order.** For FFI types where one resource must close before another, declare them in the right order.
- **`Drop` is not called if a value is moved.** Ownership transfer moves the responsibility.
- **`Drop` cannot be `async`.** If cleanup needs to await, expose `async fn close(self)` plus a defensive sync-best-effort `Drop`. The proposed `AsyncDrop` is unstable as of 2026.

Drop-related primitives: `std::mem::drop` (explicit early drop), `std::mem::forget` (skip Drop, useful for FFI ownership transfer), `ManuallyDrop<T>` (suppress automatic drop), `Box::leak` (convert to `&'static mut T`).

Full examples (SAFETY comments, unsafe fn doc patterns, Drop impl for FFI type, drop guard / scopeguard) in [references/unsafe-and-drop.md](references/unsafe-and-drop.md).

## Trait Design

Default to `impl Trait` (static dispatch, monomorphized). Reach for `dyn Trait` when storage requires it (collection of mixed implementors) or when you want a stable ABI.

Use an **associated type** when there's exactly one logical choice per implementor (`Iterator::Item`). Use a **generic parameter** when multiple impls make sense (`From<T>`).

**Sealed traits** prevent downstream consumers from implementing your trait while still allowing them to use it. Use a private `Sealed` supertrait in a private module.

**Newtype pattern**: wrap a primitive in a single-field struct to attach domain meaning (`UserId(u64)` vs `GroupId(u64)`). Combine with derives. Add `From<u64>` only if construction from any u64 is safe.

**Typestate pattern**: encode lifecycle states in the type system using marker types and `PhantomData`. Methods that change state live on the appropriate `impl Connection<State>`. Compile time rejects invalid transitions.

For functions with multiple bounds, prefer `where` clauses for readability. Higher-ranked trait bounds (`for<'a>`) when a closure must work for any caller-supplied lifetime.

`#[must_use]` on types whose return value must be observed: `Result`, `Future`, builders. Compiler warns when a `#[must_use]` value is dropped without being used.

A trait is **object-safe** if all its methods take `&self`/`&mut self` (not `self` or generic `Self`), return types don't reference `Self`, and have no generic parameters on methods. When both static and dynamic dispatch are required, define the trait object-safely and provide an `impl Trait` API on top.

Full examples (impl vs dyn, associated vs generic, sealed traits, newtype, typestate, where clauses, HRTB, `#[must_use]`, object safety) in [references/traits-and-generics.md](references/traits-and-generics.md).

## Macros

**Default to functions and generics. Reach for macros only when the type system can't express the pattern.**

`macro_rules!` declarative macros are for code patterns the type system can't capture: variadic argument lists, syntax that captures expressions, repetitive boilerplate where a function would force allocation or lose `&'static str` literal information. Anti-pattern: writing a macro for what a function would do.

Procedural macros have three flavors: derive (`#[derive(MyTrait)]`), attribute (`#[my_attr]`), function-like (`my_macro!(...)`). All run at compile time on the token stream. Pulling in `syn` + `quote` to write a proc macro is a multi-day commitment, justify it. For trait derivation, prefer existing solutions: `derive_more`, `strum`, `thiserror`, `serde_derive`, `educe`.

`cargo expand` shows the post-expansion source, the single most useful tool for understanding what a macro actually emits. Install with `cargo install cargo-expand`.

## Testing

Unit tests: inline `#[cfg(test)] mod tests` at the bottom of source files. `cargo test` runs them. Use `#[tokio::test]` for async tests.

Integration tests: each file under `tests/` is its own binary that links the library through its public API. Use these to exercise the public surface as a consumer would.

Property tests with `proptest`: generate random inputs that satisfy a strategy and assert invariants. Cheaper than fuzzing, more thorough than example-based tests.

Mocking with `mockall`: `#[cfg_attr(test, mockall::automock)]` on the trait, then `MockStorage::new()` with `expect_*` builders.

Benchmarks with `criterion`: `benches/<name>.rs`, `criterion_group!` + `criterion_main!`. CI should snapshot benchmarks and flag regressions > 10%.

Use `cargo nextest run` instead of `cargo test` for any non-trivial test suite (faster, better failure isolation). Coverage via `cargo-llvm-cov nextest --html`. Track coverage trend in CI and fail PRs that drop coverage on touched lines.

Full examples (unit tests, proptest, mockall, criterion bench, nextest, llvm-cov) in [references/testing-patterns.md](references/testing-patterns.md).

## Tooling

Always-on: `clippy` and `rustfmt`. Configure clippy via `[lints.clippy]` in `Cargo.toml` (Rust 1.74+), preferred over `#![warn(clippy::...)]` source attributes. CI runs `cargo clippy --all-targets --all-features -- -D warnings`.

Auxiliary tools:

- `cargo-deny`, license + security audit (RustSec advisory DB, allowlist, banned crates).
- `cargo-audit`, RustSec-focused subset of cargo-deny.
- `cargo-outdated`, dependency freshness check.
- `cargo-machete`, unused-dependency detection.
- `cargo-flamegraph`, profiling.
- `cargo-expand`, macro post-expansion source.
- `bacon`, watch-mode test/check/clippy runner.

Linker speedup: `mold` (Linux) / `lld` (cross-platform) via `.cargo/config.toml`. Cuts incremental link times 5-20× for large binaries.

Compile cache: `sccache` as `rustc-wrapper` in `~/.cargo/config.toml`. Combines with GitHub Actions cache or S3.

Module-level lint configuration prefers `[lints]` in `Cargo.toml` over crate-level attributes (centralized, inheritable from workspace, doesn't clutter source). Crate-level attributes still belong to things that aren't lints: `#![no_std]`, `#![cfg_attr]`-gated items, crate-level documentation.

Public items get `///` doc comments with at least: one-line summary, optional longer explanation, `# Errors` section if returns `Result`, `# Panics` section if can panic, `# Safety` section if `unsafe`, `# Examples` section for non-trivial APIs (compiles as a doctest).

Full configuration examples (`Cargo.toml [lints]`, `rustfmt.toml`, `.cargo/config.toml`, `~/.cargo/config.toml`) in [references/tooling-config.md](references/tooling-config.md).

## Performance

Order of operations: write the simplest correct code, benchmark with criterion, profile with flamegraph or `perf`, identify the bottleneck, optimize that one thing, re-benchmark. Don't pre-optimize. The compiler's optimizer is excellent; "obviously slow" code is often fast enough.

Allocation in hot paths: reuse `Vec<T>` buffers across iterations (`vec.clear()` then `vec.extend(...)`); use `SmallVec<[T; N]>` for "usually small, sometimes large"; use `&str` and `&[T]` in parameters; use `Cow<'_, str>` when the function may or may not allocate.

`Box<dyn Trait>` overhead is one indirection (vtable pointer) + one heap allocation per object. Acceptable for low-frequency calls; avoid for per-element-of-hot-loop dispatch. Static dispatch via `impl Trait` or generics is free.

`Copy` types are bitwise-copied, no destructor, no heap. Prefer `Copy` for small POD types (≤ 16 bytes). Anything with a destructor or heap allocation is `Clone` only.

`#[inline]` policy:

- **No annotation**: fine for most code. The compiler decides.
- **`#[inline]`**: hints "consider inlining across crates." Use on small accessor methods, getters, hot trivial helpers.
- **`#[inline(always)]`**: forces inlining. Almost always wrong. Use only after benchmarks show it's necessary.
- **`#[inline(never)]`**: prevents inlining. Useful for cold paths (error printers, panic handlers) to keep the hot path's code cache clean.

Memory layout: `#[repr(C)]` (C-compatible layout, required for FFI structs), `#[repr(transparent)]` (single-field newtype with same ABI as inner field, required for FFI primitive wrappers), `#[repr(packed)]` (no padding, almost never what you want). Niche optimization: `Option<NonZeroU32>`, `Option<&T>`, `Option<Box<T>>` are all the same size as the wrapped value. Field reordering: Rust freely reorders fields by default for size; use `#[repr(C)]` only when ABI matters.

`vec.iter().enumerate()` and `for i in 0..vec.len()` compile to identical assembly post-LLVM in almost all cases. Prefer the iterator form, it's idiomatic and harder to introduce off-by-one bugs.

Release profile tuning in `[profile.release]`: `lto = "fat"` (whole-program optimization, slower compiles), `codegen-units = 1` (max optimization), `panic = "abort"` (smaller binary; only when panics are not caught), `strip = "symbols"`. `lto = "thin"` is a good middle ground. For long-running servers, also `debug = "line-tables-only"` (backtraces in production without bloat).

Global allocator: for latency-sensitive workloads, swap in `mimalloc` or `jemallocator` via `#[global_allocator]`. Benchmark before and after, gains are workload-dependent (typically 5-30% on allocation-heavy paths).

Write `Arc::clone(&handle)` rather than `handle.clone()` for shared ownership. Both produce identical code, but the explicit form makes ref-count bumps grep-able and signals intent.

## Common Patterns

**Builder pattern** for types with many optional fields. `#[must_use]` on the builder type and the `&mut self -> Self` methods catches "I forgot to call `.build()`" at compile time. The `derive_builder` and `bon` crates generate this from a struct. Typestate builders enforce required fields at the type level, `build()` only exists once all required fields are set.

**Tracing setup**: use `tracing` for logging and `tracing-subscriber` to wire it up. Init shape: `tracing_subscriber::registry().with(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into())).with(fmt::layer().with_target(true)).init()` inside a `fn init_tracing()`. Instrument functions with `#[instrument(skip(...), fields(...))]` and emit events via `info!`, `warn!`, `error!`. `tracing-error::ErrorLayer` adds span context to error chains; `tracing-opentelemetry` exports spans to OpenTelemetry collectors.

**Marker traits**: empty traits that flag a property (`pub trait Sendable: Send + 'static {}` with `impl<T: Send + 'static> Sendable for T {}`). Use to express constraints in trait bounds without inventing a new abstract verb.

**`From` / `Into` / `TryFrom` / `TryInto`**: define `From<T>` for infallible conversions, `TryFrom<T>` for fallible. The `Into` traits are auto-implemented from `From`. They compose with `?` and into iterator adapters.

Full examples (standard builder, typestate builder) in [references/builder-patterns.md](references/builder-patterns.md).

## Anti-Patterns

- **Don't use `unwrap()` in production code paths.** Pattern-match or `?`. `expect("clear message")` is acceptable for invariant violations.
- **Don't use `Box<dyn Error>` in libraries.** Use `thiserror`. Reserve `Box<dyn Error>` / `anyhow::Error` for binaries.
- **Don't hold a sync `Mutex` across `.await`.** Use `tokio::sync::Mutex`, or restructure to release before awaiting. Clippy: `await_holding_lock`.
- **Don't run rayon directly inside a tokio task.** Use `spawn_blocking` to bridge.
- **Don't use `Arc<Mutex<HashMap<K, V>>>` reflexively.** Consider `dashmap`, sharded locks, or actor patterns for write-heavy maps.
- **Don't use unbounded channels by default.** Bounded channels expose backpressure; unbounded ones mask it until OOM.
- **Don't `clone()` in hot loops.** Profile and determine whether borrowing is feasible instead.
- **Don't use `lazy_static!` in new code.** Use `LazyLock` (closure-defined) or `OnceLock` (caller-driven).
- **Don't use `String` parameters when `&str` suffices.** Forces the caller to allocate.
- **Don't return `&Vec<T>`. Return `&[T]`.** Slices compose with arrays, smallvecs, and other slice-like types.
- **Don't `panic!` in async code paths reachable from external input.** Network bytes, file contents, user input, all are runtime conditions, not bugs.
- **Don't `panic!` in `Drop` impls.** Aborts the process if another panic is unwinding. Log and absorb.
- **Don't blanket-derive `Clone` on every type.** Cloning has cost; require it explicitly where used.
- **Don't write `pub` reflexively.** Default to `pub(crate)`; promote to `pub` only when consumers need it.
- **Don't ignore clippy warnings.** Either fix them or annotate `#[allow(clippy::...)]` with a `// reason: ...` comment.
- **Don't write `unsafe { ... }` without a `// SAFETY:` comment.** A SAFETY-less unsafe block is a code review failure even if it works.
- **Don't `mem::transmute` reflexively.** Almost always wrong. Prefer `as` casts, `From`/`Into`, or `bytemuck::cast` for POD types.
- **Don't `unsafe impl Send for T` casually.** Most attempts are wrong. Prove the safety contract in a SAFETY comment first.
- **Don't write a macro when a function works.** Macros have worse error messages and tooling. Reach for them only when the type system can't express the pattern.
- **Don't reach for `async-trait` by default in new code.** Native `async fn` in traits has been stable since Rust 1.75; use `trait-variant` for `Send` bounds.
- **Don't ignore `#[must_use]` warnings.** They flag dropped `Result`s, unconsumed builders, and forgotten futures.
- **Don't pin `edition = "2018"` or `"2015"` in new code.** Use `edition = "2024"` (or `"2021"` only when a reason justifies it).

## Reference Reading

When in doubt, read these:

- [Rust Book](https://doc.rust-lang.org/book/), language fundamentals
- [Rustonomicon](https://doc.rust-lang.org/nomicon/), unsafe Rust, atomics, FFI internals
- [Rust Reference](https://doc.rust-lang.org/reference/), language semantics in detail
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/), naming, trait impls, documentation
- [Rust Performance Book](https://nnethercote.github.io/perf-book/), profiling, allocation, layout
- [Async Book](https://rust-lang.github.io/async-book/), `Future` internals, `Pin`, `Waker`
- [tokio docs](https://docs.rs/tokio), runtime, sync primitives, channels, IO
- [tokio tutorial](https://tokio.rs/tokio/tutorial), structured concurrency, cancellation
- [Rust for Rustaceans](https://nostarch.com/rust-rustaceans) (Jon Gjengset), the canonical "production Rust" book
- [Programming Rust, 2nd ed.](https://www.oreilly.com/library/view/programming-rust-2nd/9781492052586/) (Blandy/Orendorff/Tindall), broader reference
- [cheats.rs](https://cheats.rs/), language cheat sheet
- [Rust Edition Guide](https://doc.rust-lang.org/edition-guide/), what changed in 2018/2021/2024
- `cargo doc --open -p {crate}`, local docs for any installed crate, pinned to your `Cargo.lock` version

For GPUI async, entity, and background-executor bridging specifically, see the `gpui-kit` skill.

In-skill references (extracted to keep this file scannable):

- [references/error-handling.md](references/error-handling.md), `thiserror`, `anyhow`, error chain walking, `#[non_exhaustive]`, `#[track_caller]`
- [references/async-tokio.md](references/async-tokio.md), spawn, JoinSet, select!, mutex pattern, CancellationToken, async traits
- [references/concurrency.md](references/concurrency.md), rayon, scoped threads, atomics, LazyLock, OnceLock
- [references/unsafe-and-drop.md](references/unsafe-and-drop.md), SAFETY comments, Drop impl, drop guards
- [references/traits-and-generics.md](references/traits-and-generics.md), impl vs dyn, associated vs generic, sealed traits, typestate, HRTB
- [references/testing-patterns.md](references/testing-patterns.md), unit tests, proptest, mockall, criterion, nextest, llvm-cov
- [references/builder-patterns.md](references/builder-patterns.md), standard builder, typestate builder
- [references/tooling-config.md](references/tooling-config.md), `Cargo.toml [lints]`, `rustfmt.toml`, `.cargo/config.toml`, sccache
