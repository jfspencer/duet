# Testing Patterns

## Unit tests

Inline `#[cfg(test)]` modules at the bottom of source files (or in a `tests.rs` sibling):

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn analyzes_simple_frame() {
        let frame = Frame::black(64, 64);
        let result = analyze(&frame);
        assert!(result.is_ok());
    }
}
```

`cargo test` runs them. Use `#[tokio::test]` for async tests.

## Integration tests

Each file under `tests/` is its own binary that links the library through its public API. Use these to exercise the public surface as a consumer would.

## Property tests with `proptest`

Use proptest to generate random inputs that satisfy a strategy and assert invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_is_identity(input in any::<Vec<u8>>()) {
        let encoded = encode(&input);
        let decoded = decode(&encoded).unwrap();
        prop_assert_eq!(input, decoded);
    }
}
```

Cheaper than fuzzing, more thorough than example-based tests. Prefer for any function with a non-trivial invariant.

## Mocking with `mockall`

For trait-bound dependencies that are hard to construct in tests:

```rust
#[cfg_attr(test, mockall::automock)]
trait Storage {
    async fn get(&self, key: &str) -> Result<Vec<u8>>;
    async fn put(&self, key: &str, value: Vec<u8>) -> Result<()>;
}
```

In tests:

```rust
let mut mock = MockStorage::new();
mock.expect_get()
    .with(eq("key1"))
    .times(1)
    .returning(|_| Box::pin(async { Ok(vec![1, 2, 3]) }));
```

## Benchmarks with `criterion`

For performance-sensitive code, use criterion (cargo's built-in `cargo bench` is unstable + less ergonomic):

```rust
// benches/analyze.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_analyze(c: &mut Criterion) {
    let frame = Frame::test_frame();
    c.bench_function("analyze", |b| {
        b.iter(|| analyze(black_box(&frame)))
    });
}

criterion_group!(benches, bench_analyze);
criterion_main!(benches);
```

`cargo bench` produces HTML reports under `target/criterion/`. CI should snapshot benchmarks and flag regressions > 10%.

## Test runner

Use `cargo nextest run` instead of `cargo test` for any non-trivial test suite, faster, better failure isolation, deterministic output. Configure in `.config/nextest.toml`.

For stdout debugging during a failing test, pass `--no-capture`:

```bash
cargo nextest run --no-capture my_failing_test
```

(`cargo test` uses `-- --nocapture`; `nextest` uses `--no-capture` directly.)

## Coverage with `cargo-llvm-cov`

`cargo-llvm-cov` is the modern coverage tool, uses LLVM source-based coverage, integrates with `nextest`, produces HTML and lcov outputs:

```bash
cargo install cargo-llvm-cov
cargo llvm-cov nextest --html       # local report at target/llvm-cov/html
cargo llvm-cov nextest --lcov --output-path lcov.info  # for Codecov / Coveralls
```

CI should track coverage trend and fail PRs that drop coverage on touched lines. Don't optimize raw coverage % at the expense of test quality, branch coverage is more meaningful than line coverage.
