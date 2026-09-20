# Multithreading and One-Time Init

Workload-to-tool mapping: I/O-bound → tokio, CPU-bound → rayon, embarrassingly parallel data → `rayon::par_iter`, pipelined work stages → crossbeam channels + scoped threads, lock-free counters → atomics.

## rayon

`rayon::par_iter()` parallelizes anything that implements `IntoParallelIterator`. The pool auto-sizes to CPU count.

```rust
use rayon::prelude::*;

let metrics: Vec<Metric> = frames
    .par_iter()
    .map(|frame| analyze(frame))
    .collect();
```

For more control: `rayon::scope` (scoped task spawning), `rayon::join` (binary fork-join), custom thread pools (`rayon::ThreadPoolBuilder`).

rayon under tokio: wrap rayon work in `tokio::task::spawn_blocking`. Never call rayon directly inside an async task, it starves the tokio runtime. Inside the GPUI app, use `cx.background_spawn` for CPU-heavy work and notify the owning entity when it lands (see the `gpui-kit` skill).

## Scoped threads

Use `std::thread::scope` (1.63+) for threads that need to borrow from the parent stack, it guarantees all spawned threads join before the scope returns, so the borrow checker accepts non-`'static` references:

```rust
std::thread::scope(|s| {
    s.spawn(|| {
        // can borrow from outer scope; no Arc/clone needed
        do_work(&data);
    });
    s.spawn(|| analyze(&data));
}); // all spawned threads guaranteed joined here
```

Prefer `std::thread::scope` over `crossbeam::scope` for new code (no extra dependency, identical semantics). Reach for `crossbeam::scope` only when you also want crossbeam's other utilities and are already pulling in the crate.

## crossbeam pieces

`crossbeam::channel` is a faster, more featured `std::sync::mpsc` (true MPMC, bounded and unbounded variants). Use in synchronous contexts where multiple producers AND multiple consumers are needed; for M↔1 async, prefer `tokio::sync::mpsc`.

- `crossbeam::deque`, work-stealing deques (rayon uses these internally)
- `crossbeam::epoch`, epoch-based memory reclamation for lock-free data structures
- `crossbeam::utils::CachePadded<T>`, pad a value to a cache line to avoid false sharing

## Atomics and memory orderings

```rust
use std::sync::atomic::{AtomicU64, Ordering};

let counter = Arc::new(AtomicU64::new(0));
counter.fetch_add(1, Ordering::Relaxed);
```

Pick orderings by intent, not by guess:
- `Relaxed`, atomicity only, no ordering. Use for counters that don't synchronize other state.
- `Acquire` (loads) / `Release` (stores), paired ordering. Use when one thread publishes data via the atomic and another consumes it.
- `AcqRel`, for read-modify-write on synchronization primitives (CAS loops on a lock).
- `SeqCst`, total order across all `SeqCst` atomics. Slowest. Default if you can't justify weaker ordering.

The Rustonomicon's chapter on atomics is the canonical reference. Read it before designing lock-free data structures.

## One-time init: `LazyLock`, `OnceLock`

For one-time initialization in 2026, use the stdlib primitives:

- `std::sync::LazyLock<T>` (1.80+), closure-defined lazy static. The canonical replacement for `lazy_static!` and `once_cell::sync::Lazy`.
- `std::sync::OnceLock<T>` (1.70+), caller-driven one-time init. Use when the value isn't known at definition time.

```rust
use std::sync::LazyLock;

// Lazy: the closure runs on first access.
static GLOBAL_CFG: LazyLock<Config> = LazyLock::new(|| {
    Config::load().expect("config required at startup")  // fail-fast: no recovery path
});

// OnceLock: caller chooses when to initialize.
use std::sync::OnceLock;
static RUNTIME: OnceLock<tokio::runtime::Runtime> = OnceLock::new();

pub fn install_runtime(rt: tokio::runtime::Runtime) {
    RUNTIME.set(rt).expect("runtime installed twice");
}
```

The `expect("config required at startup")` panic is acceptable here because it's a fail-fast at process start with no recovery path, the program literally cannot run without config. Anything reachable from runtime input must propagate via `Result`.

Use `once_cell::sync::Lazy` / `OnceCell` only on Rust < 1.80; do not introduce new uses of `lazy_static!`.
