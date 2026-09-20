# Builder Patterns

## Standard builder

For types with many optional fields:

```rust
#[must_use]
pub struct ClientBuilder {
    timeout: Option<Duration>,
    retries: u32,
}

impl ClientBuilder {
    #[must_use]
    pub fn timeout(mut self, t: Duration) -> Self {
        self.timeout = Some(t);
        self
    }

    pub fn build(self) -> Result<Client, ConfigError> { /* ... */ }
}
```

`#[must_use]` on the builder type and the `&mut self -> Self` methods catches "I forgot to call `.build()`" at compile time.

The `derive_builder` and `bon` crates generate this from a struct. Use for any type with > 3 optional configuration fields.

## Typestate builder

Typestate builders enforce required fields at the type level, `build()` only exists once all required fields are set:

```rust
pub struct NoEndpoint;
pub struct WithEndpoint(String);

pub struct ClientBuilder<E> { endpoint: E, timeout: Option<Duration> }

impl ClientBuilder<NoEndpoint> {
    pub fn endpoint(self, url: String) -> ClientBuilder<WithEndpoint> {
        ClientBuilder { endpoint: WithEndpoint(url), timeout: self.timeout }
    }
}

impl ClientBuilder<WithEndpoint> {
    pub fn build(self) -> Client { /* endpoint guaranteed set */ }
}
```

Now `ClientBuilder::new().build()` doesn't compile, the caller must set the endpoint first.
