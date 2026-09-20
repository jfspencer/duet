# Unsafe, Drop, and RAII

Most production Rust crates can and should be 100% safe. Reach for `unsafe` only when crossing FFI to C / system APIs, lock-free data structures with atomics, manual memory management, or rare performance-critical hot loops where bounds checks dominate (profile first).

## Unsafe discipline

Encapsulate unsafe in safe wrappers. The smallest possible `unsafe { ... }` block that justifies a single, named invariant. Public API stays safe; internal code carries the proof obligation.

Every `unsafe` block has a `// SAFETY:` comment. Non-negotiable. The comment explains why the operation is sound, what invariants the surrounding code guarantees, what the caller must uphold, why the pointer is valid. No SAFETY comment = the code is wrong even if it works. Clippy's `undocumented_unsafe_blocks` lint can enforce this.

```rust
// SAFETY: `ptr` was returned by `Box::into_raw` above and has not been freed
// or aliased; this restores ownership exactly once.
let boxed = unsafe { Box::from_raw(ptr) };
```

`#[deny(unsafe_op_in_unsafe_fn)]` at the crate root (default in 2024 edition). Inside an `unsafe fn`, every unsafe operation must still be wrapped in its own `unsafe { ... }` block with its own SAFETY comment. The function being marked unsafe does not licence the body.

Public `unsafe fn` requires a `# Safety` doc section documenting the caller's obligations:

```rust
/// Reads a `T` from the given pointer.
///
/// # Safety
///
/// `ptr` must be non-null, properly aligned for `T`, and point to an
/// initialized `T` that is not concurrently mutated. The caller must
/// ensure no other `&mut T` aliases this location for the duration of
/// the read.
pub unsafe fn read_volatile<T>(ptr: *const T) -> T { /* ... */ }
```

`#![forbid(unsafe_code)]` at the crate root for crates that genuinely need no unsafe, stronger than `#![deny]` (cannot be locally `#[allow]`'d).

## Unsafe tooling

- `cargo +nightly miri test` runs your test suite under an interpreter that catches undefined behavior (use-after-free, uninitialized reads, out-of-bounds, data races on atomics). Mandatory for any crate with non-trivial unsafe.
- AddressSanitizer via `RUSTFLAGS="-Z sanitizer=address" cargo +nightly test` catches unsafe FFI bugs that Miri can't see (e.g. into a C library).
- `cargo expand` shows what macros expand into when reviewing unsafe-emitting macros.

## Unsafe anti-patterns

- No SAFETY comment. Reviewer should reject the PR.
- Wide `unsafe { ... }` block covering a dozen lines of mostly-safe code. Tighten to the smallest scope.
- Public `unsafe fn` without `# Safety` section. The contract is undocumented; consumers cannot use it correctly.
- `mem::transmute` reflexively. Almost always wrong. Prefer `as` casts, `From`/`Into`, `bytemuck::cast` for POD types, or `slice::from_raw_parts` with proper invariants.
- Holding a raw pointer across an `.await`. Easy way to alias a moved value; the borrow checker won't catch it.
- `unsafe impl Send for T` without justification. Most uses are wrong; the few correct ones need a SAFETY comment explaining why aliasing is sound.

## Drop and RAII

Rust's resource management is RAII: a value owns its resources and releases them when it goes out of scope via the `Drop` trait. File handles, sockets, locks, allocations, FFI handles, all rely on `Drop`. Get this right and resource leaks become structurally impossible.

```rust
pub struct DnsServiceRef {
    ref_: *mut sys::DNSServiceRef,
}

impl Drop for DnsServiceRef {
    fn drop(&mut self) {
        // SAFETY: `self.ref_` was obtained from DNSServiceCreateConnection,
        // is non-null (enforced by constructor), and is dropped exactly once
        // because we have unique ownership.
        unsafe { sys::DNSServiceRefDeallocate(self.ref_); }
    }
}
```

## Drop discipline

- Drop must not panic. A panic during drop while another panic is unwinding aborts the process. Drop impls perform cleanup; they must absorb errors (log via `tracing`, no propagation).
- Drop order is field declaration order. For FFI types where one resource must close before another (e.g. a child handle before its parent connection), declare them in the right order, the compiler will not reorder for you.
- `Drop` is not called if a value is moved. Ownership transfers move the responsibility; the original binding's drop runs only if it still holds the value at scope exit.
- `Drop` cannot be `async`. If cleanup needs to await, the type needs an explicit `async fn close(self)` method that consumers must call before drop, plus a defensive `Drop` impl that does the synchronous best-effort cleanup (and warns via `tracing` that the consumer forgot to close). The proposed `AsyncDrop` is unstable as of 2026.

## Drop-related primitives

- `std::mem::drop(value)`, explicit early drop. Useful to release a lock before an `.await`, or to make ownership transitions visible.
- `std::mem::forget(value)`, leaks the value, skipping `Drop`. Use when ownership transfers across an FFI boundary that will free it.
- `ManuallyDrop<T>`, type-level wrapper that suppresses automatic drop. Use when one field of a struct must outlive the struct itself (e.g. building a self-referential structure).
- `Box::leak(boxed)`, converts `Box<T>` to `&'static mut T`, skipping the destructor. Right tool for "I want this allocation to live forever," wrong tool for "I'm fighting the borrow checker."

## Drop guards

Run cleanup on scope exit, including panic unwind:

```rust
struct DeferCleanup<F: FnOnce()>(Option<F>);
impl<F: FnOnce()> Drop for DeferCleanup<F> {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() { f(); }
    }
}

let _guard = DeferCleanup(Some(|| close_handle(handle)));
// cleanup runs whether the function returns normally or panics
```

The `scopeguard` crate provides a polished version (`defer!`, `defer_on_unwind!`, `defer_on_success!`).
