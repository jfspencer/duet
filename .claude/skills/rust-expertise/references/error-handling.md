# Error Handling Patterns

Two libraries, two contexts: `thiserror` for libraries (stable error taxonomy), `anyhow` for binaries (context chaining).

## Libraries: `thiserror`

Define an error enum per logical scope. Derive `Error`, `Debug`. Implement `Display` via `#[error("...")]`. Use `#[from]` to bridge other error types.

```rust
#[derive(thiserror::Error, Debug)]
pub enum AnalyzeError {
    #[error("invalid frame layout: {0}")]
    InvalidFrame(String),

    #[error("decoder error")]
    Decoder(#[from] DecoderError),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, AnalyzeError>;
```

Library errors must be stable: removing a variant or changing its fields is a breaking change. Add new variants behind a `#[non_exhaustive]` enum to preserve compatibility:

```rust
#[derive(thiserror::Error, Debug)]
#[non_exhaustive]
pub enum AnalyzeError { /* ... */ }
```

## Binaries / Apps: `anyhow`

`anyhow::Result<T>` is `Result<T, anyhow::Error>` with built-in context chaining. Use it in main.rs and any code path where you don't need a stable error taxonomy.

```rust
use anyhow::{Context, Result};

fn load_config(path: &Path) -> Result<Config> {
    let bytes = fs::read(path)
        .with_context(|| format!("reading config from {}", path.display()))?;
    let config: Config = toml::from_slice(&bytes)
        .with_context(|| "parsing config as TOML")?;
    Ok(config)
}
```

`.context(...)` adds a stack of human-readable messages without losing the underlying error. Print with `eprintln!("{:#}", err)` for a single-line condensed chain (anyhow's alternate-format), or `eprintln!("{:?}", err)` for the multi-line chain with backtraces (when `RUST_BACKTRACE=1`).

## Walking thiserror error chains

For thiserror chains there's no `{:#}` shortcut, walk `std::error::Error::source()`:

```rust
let mut current: Option<&dyn std::error::Error> = Some(&err);
while let Some(e) = current {
    eprintln!("- {e}");
    current = e.source();
}
```

The `tracing-error` crate's `SpanTrace` integrates this with the `tracing` span hierarchy for production logging.

## `#[non_exhaustive]` more broadly

`#[non_exhaustive]` is not just for errors. Any public enum or struct where you reserve the right to add variants/fields without a major version bump should carry it. Consumers must use `_ => ...` arms for non-exhaustive enums and constructors for non-exhaustive structs.

```rust
#[non_exhaustive]
pub enum LogLevel { Trace, Debug, Info, Warn, Error }

#[non_exhaustive]
pub struct ConnectOpts {
    pub timeout: Duration,
    pub retries: u32,
}
```

## `#[track_caller]` on panic helpers

Use `#[track_caller]` on small helper functions that may panic, the panic message will point at the caller's location, not the helper:

```rust
#[track_caller]
pub fn assert_initialized<T>(opt: &Option<T>) -> &T {
    opt.as_ref().expect("value not yet initialized")
}
```
