---
name: test-author
description: Use when creating or modifying any Rust test in this workspace, when the user asks to "write a test", "add coverage", "test this", or when a bug fix needs a regression test. Encodes where tests live (same-file `#[cfg(test)] mod tests`, `tests/` integration tests wrapped in a test module, GPUI UI tests with `#[gpui_kit::test]`), the System-Under-Test builder pattern, per-test scratch directories, mandatory assert messages, and what a test may do that committed code may not.
allowed-tools: Bash(cargo:*) Bash(grep:*) Read Write Edit
verified: 2026-09-20
verified-against:
  - Cargo.toml
  - clippy.toml
  - scripts/dod.sh
  - tools/plan-db/tests/roundtrip.rs
  - crates/duet/src/app.rs
  - .claude/skills/gpui-kit/references/coding-guides.md
review-cadence: on-architectural-change
---

# Test Author

## Trigger

A new test, a changed test, or a bug fix that needs a regression test. Every test in this workspace goes through this procedure.

## When NOT to use

- Diagnosing a failing test. Use `systematic-debugging` first, then return here for the regression test.
- Deciding which GPUI component or entity shape to use. That is the `gpui-kit` skill; this skill covers only how to test it.
- Claiming the suite is green. Use `verification-before-completion`.

## The gate

`scripts/dod.sh` runs `cargo nextest run --workspace --locked` (or `cargo test --workspace --locked` when nextest is absent) and then `cargo test --doc --workspace --locked`. Doctests count. The gate runs on every commit through `.githooks/pre-commit`. Do not hand-run the whole gate; run one targeted command while you iterate:

```sh
cargo nextest run -p <crate> <filter>
cargo test -p <crate> --doc
```

## Where a test lives

| Test shape | Location | Why |
|---|---|---|
| Unit test of one function or one entity | `#[cfg(test)] mod tests { ... }` at the bottom of the same file | The test sees private items; `tests_outside_test_module` is denied, so a `#[test]` outside a `#[cfg(test)]` module fails clippy |
| Integration test of a binary or a public API | `<crate>/tests/<name>.rs`, with the WHOLE file body inside `#[cfg(test)] mod tests { ... }` | The same lint applies to integration test files; the wrapper also gives the test module scope for its helpers |
| GPUI UI integration test | `#[gpui_kit::test]` in a `#[cfg(test)] mod tests`, with the `gpui-kit` `test-support` feature enabled as a dev dependency | Renders the real view in a headless window and drives it; see the `gpui-kit` skill's testing reference |
| Contract a code example must satisfy | A doctest on the public item, or a `compile_fail` doctest for a type-level invariant | `cargo test --doc` runs it |

The reference integration test is `tools/plan-db/tests/roundtrip.rs`. The reference unit test is the `tests` module in `crates/duet/src/app.rs`.

## The System-Under-Test pattern

1. **Build the SUT in one function.** A test module has one `fn sut(...) -> ...` (or a small set of named builders) that constructs the thing under test with explicit inputs. A test body reads as: build, act, assert.
2. **No shared mutable state.** No `static mut`, no `OnceLock` that a test mutates, no fixed temp path, no fixed port. Two tests run in parallel by default; shared state makes them flaky.
3. **Per-test scratch directory.** When a test touches the filesystem, it creates its own directory under `std::env::temp_dir()` with a unique suffix (process id plus a nanosecond timestamp), and removes it at the end. `scratch()` in `tools/plan-db/tests/roundtrip.rs` is the pattern.
4. **No process-global mutation.** `std::env::set_var` and `remove_var` are banned in `clippy.toml`. Pass configuration explicitly: a `Command::env(...)` for a spawned binary, a constructor argument for a library.
5. **A spawned binary is the real one.** An integration test of a CLI runs `env!("CARGO_BIN_EXE_<name>")`, never a hand-built path.
6. **Deterministic completion.** Wait on a signal, a channel, or a returned value. Never `sleep` and hope.

## Assertions

- **Every assert carries a message.** `missing_assert_message` is denied. The message states the property, not the values: `assert_eq!(code, 0, "init exits 0")`.
- **One property per assert.** A failing assert names one thing that is wrong.
- **Assert on behavior, not on structure.** Test what a caller can observe. A test that reads a private field to check an intermediate value breaks on every refactor.
- **A regression test fails before the fix.** Write it, run it, watch it go red, then fix the code. A test nobody has watched go red proves nothing.

## What a test may do that committed code may not

`clippy.toml` carves these out for test code only (`allow-*-in-tests = true`): `unwrap()`, `expect()`, `panic!`, and indexing or slicing. `dbg!` stays banned everywhere. The carve-out applies inside a `#[cfg(test)]` module and inside `#[test]` functions. A helper that lives OUTSIDE the test module gets no carve-out, which is one more reason the whole integration test file sits inside `mod tests`.

Every other lint applies: a doc line on every helper (`missing_docs_in_private_items`), no shadowing (`shadow_unrelated`; name each result `init_code`, `put_code`, not `code` three times), no `#[allow]` (`allow_attributes`; use `#[expect(lint, reason = "...")]` if a lint is provably wrong at one site).

## Rules

- **No `#[ignore]` without a reason string.** `#[ignore = "needs a display; tracked in roadmap/<plan>/..."]`. A bare `#[ignore]` is a deleted test that still looks like coverage.
- **No new dev dependency without `cargo deny`.** `cargo deny check` runs in the gate. A property-test or mock crate is a dependency like any other: justify it, and prefer a plain table-driven loop over a list of cases when that is enough.
- **Table-driven cases are a loop.** `for (input, want) in [(0, "Not clicked yet"), (1, "Clicked once")] { assert_eq!(sut(input), want, "label for {input}"); }`. The message names the case.
- **One test file per binary contract.** An integration test for `plan-db` covers the CLI contract; a unit test covers the pure helpers. Do not test a pure function by spawning the binary.
- **GPUI tests test the production view.** Render the real entity, dispatch the real action, assert the outcome through normal Rust assertions. Do not build a test-only copy of the view.
- **Doctests are tests.** A code block in a doc comment compiles and runs under `cargo test --doc` unless it is marked `no_run` or `ignore`; mark it honestly.

## Worked example

An integration test of the `plan-db` CLI, mirroring `tools/plan-db/tests/roundtrip.rs`:

```rust
//! End-to-end check of the CLI against a real store in a scratch directory.

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};
    use std::process::Command;
    use std::time::{SystemTime, UNIX_EPOCH};
    use std::{env, fs};

    /// A scratch directory that is NOT inside a git repository.
    fn scratch() -> PathBuf {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("clock").as_nanos();
        let dir = env::temp_dir().join(format!("plan-db-test-{}-{nanos}", std::process::id()));
        fs::create_dir_all(dir.join("work")).expect("scratch layout");
        dir
    }

    /// Run the binary with the scratch layout and return `(exit code, stdout)`.
    fn db(root: &Path, args: &[&str]) -> (i32, String) {
        let output = Command::new(env!("CARGO_BIN_EXE_plan-db"))
            .args(args)
            .current_dir(root.join("work"))
            .env("PLAN_DB_ROOT", root.join("dbs"))
            .output()
            .expect("spawn plan-db");
        let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
        (output.status.code().unwrap_or(-1), stdout)
    }

    #[test]
    fn put_then_get_round_trips() {
        let root = scratch();
        let (put_code, _) = db(&root, &["put", "roadmap/alpha", "control:signal", "pause"]);
        assert_eq!(put_code, 0, "put exits 0");
        let (_, value) = db(&root, &["get", "roadmap/alpha", "control:signal"]);
        assert_eq!(value.trim(), "pause", "get returns the stored value");
        fs::remove_dir_all(&root).expect("cleanup");
    }
}
```

Run it alone with `cargo nextest run -p plan-db put_then_get_round_trips`, then commit and let the hook run the gate.

## Related skills

- `gpui-kit`, for `#[gpui_kit::test]`, `TestAppContext`, and the UI integration testing reference
- `rust-expertise`, for the wider testing matrix (property tests, benchmarks) when a plain test is not enough
- `failure-mode-author`, when the test is the guard for a defect a tool should have caught
- `verification-before-completion`, before claiming the suite is green
