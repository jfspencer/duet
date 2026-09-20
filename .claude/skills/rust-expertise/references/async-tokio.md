# Async with tokio Patterns

Use `#[tokio::main]` for binaries; explicit runtime construction for libraries. Inside the GPUI app, GPUI owns the executor (`cx.spawn`, `cx.background_spawn`); don't construct your own.

## Spawning

`tokio::spawn` returns a `JoinHandle<T>`. Always handle the join handle, either `await` it or use `JoinSet` for managed groups. A spawned task that nobody waits on becomes orphaned; if it panics, you find out via tracing logs only.

```rust
let handle = tokio::spawn(async move {
    work().await
});

let result = handle.await
    .context("join task")?
    .context("inner work")?;
```

## Structured concurrency with `JoinSet`

Use `tokio::task::JoinSet` for groups of tasks where you want to await all and collect results, or cancel the rest on first failure.

```rust
use tokio::task::JoinSet;

let mut set = JoinSet::new();
for item in items {
    set.spawn(process(item));
}

let mut results = Vec::with_capacity(set.len());
while let Some(res) = set.join_next().await {
    results.push(res??);
}
```

`JoinSet::abort_all()` cancels every outstanding task. Combine with `select!` for "first-success-wins" patterns.

## `select!` and cancellation safety

`tokio::select!` races multiple futures. Use for cancellation, timeouts, watching multiple channels.

```rust
tokio::select! {
    res = work() => res?,
    _ = cancel_token.cancelled() => return Err(Cancelled.into()),
    _ = tokio::time::sleep(Duration::from_secs(10)) => return Err(Timeout.into()),
}
```

Watch out for cancellation safety. Each branch's future may be dropped if another branch wins. Don't use `select!` with non-cancel-safe futures (e.g. one that's halfway through writing to a buffer). The tokio docs flag which APIs are cancel-safe.

## Mutex: sync vs async

`std::sync::Mutex` / `parking_lot::Mutex` for state held briefly within sync code. `tokio::sync::Mutex` for state held across `.await` points.

Holding a sync mutex across `.await` is a bug. The future may be moved to a different thread, the mutex won't follow, and cancellation can leave the mutex permanently locked. Clippy's `await_holding_lock` catches this.

If you need to do async work while holding state, restructure to scope the lock tightly:

```rust
// BAD: lock held across await (parking_lot::Mutex shown)
let mut guard = mutex.lock();
guard.update(fetch().await?);  // future may move threads; lock can leak

// GOOD: do async work, then briefly lock to commit
let new_value = fetch().await?;
mutex.lock().update(new_value);
```

If using `std::sync::Mutex`, `.lock()` returns `Result<MutexGuard, PoisonError>`, propagate with `?` or unwrap (poisoning means another thread panicked while holding the lock). `parking_lot::Mutex::lock()` returns the guard directly with no poisoning.

## Cancellation tokens

Use `tokio_util::sync::CancellationToken` for cooperative cancellation across an actor / task tree.

```rust
let token = CancellationToken::new();
let child = token.child_token();

tokio::spawn(async move {
    tokio::select! {
        _ = child.cancelled() => return,
        res = work() => { /* ... */ }
    }
});

// Later:
token.cancel(); // propagates to all children
```

## Native `async fn` in traits

Stabilized in Rust 1.75 (Dec 2023). Default to native syntax in new code, reach for `async-trait` only when a specific limitation forces it. Native async-fn-in-traits returns an opaque `impl Future` that does NOT automatically implement `Send`.

Three options for `Send` bounds:

1. **`trait-variant` crate (preferred)** generates a `Send`-bounded variant of the trait automatically, no allocation.

   ```rust
   #[trait_variant::make(StorageSend: Send)]
   trait Storage {
       async fn get(&self, key: &str) -> Result<Vec<u8>>;
   }
   ```

2. **Return a boxed future explicitly** preserves object safety with an unambiguous bound: return type is `Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>>`.

3. **`async-trait` crate** is a proc-macro that desugars to `Pin<Box<dyn Future>>` returns. Adds one allocation per call but produces object-safe traits with `Send` bounds out of the box. Right choice when you need `dyn Trait` storage.

Native `async fn` in traits is NOT object-safe by default. `Box<dyn Storage>` won't compile if `Storage` contains native `async fn`s. Use option 2 or 3 above for trait-object collections.
