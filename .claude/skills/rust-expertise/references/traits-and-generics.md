# Trait Design and Generics

## `impl Trait` vs `dyn Trait`

```rust
// impl: static dispatch, monomorphizes per type, can't store mixed types
fn process(items: impl Iterator<Item = u32>) { /* ... */ }

// dyn: dynamic dispatch, one indirection, allows heterogeneous collections
fn process(items: Box<dyn Iterator<Item = u32>>) { /* ... */ }

// Storage of mixed types requires dyn
struct App {
    handlers: Vec<Box<dyn Handler>>,
}
```

Default to `impl Trait`. Reach for `dyn` when storage requires it (collection of mixed implementors) or when you want a stable ABI (avoid template explosion).

## Associated types vs generics

```rust
// Associated type: one Item per Iterator implementor
trait Iterator {
    type Item;
    fn next(&mut self) -> Option<Self::Item>;
}

// Generic: multiple From<T> impls per receiver type
trait From<T> {
    fn from(t: T) -> Self;
}
```

Use an associated type when there's exactly one logical choice per implementor. Use a generic parameter when multiple impls make sense.

## Sealed traits

Prevent downstream consumers from implementing your trait while still allowing them to use it:

```rust
mod sealed { pub trait Sealed {} }

pub trait MyTrait: sealed::Sealed {
    fn method(&self);
}

// Each implementor must also implement Sealed, but Sealed is private
impl sealed::Sealed for ConcreteType {}
impl MyTrait for ConcreteType { /* ... */ }
```

Use for traits where you want to add methods in future versions without breaking downstream implementations.

## Newtype pattern

Wrap a primitive in a single-field struct to attach domain meaning:

```rust
pub struct UserId(u64);
pub struct GroupId(u64);

// Compiler now rejects passing a UserId where a GroupId is expected.
fn lookup_group(id: GroupId) -> Option<Group> { /* ... */ }
```

Combine with `derive(Debug, Clone, Copy, PartialEq, Eq, Hash)` to get all the basic operations. Add `From<u64>` only if it's safe to construct from any u64; otherwise expose a fallible constructor.

## Typestate pattern

Encode lifecycle states in the type system using marker types and `PhantomData`:

Shape: `struct Connection<S> { socket: TcpStream, _state: PhantomData<S> }` plus marker structs `Closed`, `Open`. Methods that change state live on the appropriate `impl Connection<State>`, e.g. `impl Connection<Closed> { fn open(...) -> Result<Connection<Open>> }` and `impl Connection<Open> { fn send(...); fn close(self) -> Connection<Closed> }`.

The compiler now rejects calling `send()` on a `Connection<Closed>`. State transitions are enforced at compile time.

## Where clauses and trait bounds

For functions with multiple bounds, prefer `where` clauses for readability, they keep the signature scannable:

```rust
// Cluttered
fn process<T: Clone + Send + Sync + 'static, U: AsRef<str>>(items: Vec<T>, key: U) { /* ... */ }

// Readable
fn process<T, U>(items: Vec<T>, key: U)
where
    T: Clone + Send + Sync + 'static,
    U: AsRef<str>,
{ /* ... */ }
```

## Higher-ranked trait bounds (HRTB)

When a closure must work for any lifetime the caller supplies, use `for<'a>`:

```rust
fn apply<F>(f: F)
where
    F: for<'a> Fn(&'a str) -> &'a str,
{
    let s = String::from("hello");
    let r = f(&s);
    println!("{r}");
}
```

Without HRTB the closure would have to be tied to a single, fixed `'a` chosen by the caller, almost always not what you want when the closure receives a borrowed value with a scope determined inside the function.

## `#[must_use]`

Mark types whose return value must be observed, `Result`, `Future`, builders, anything where ignoring the value is a bug:

```rust
#[must_use = "this `Result` may be an `Err`; use `?` or pattern-match to handle"]
pub enum Result<T, E> { Ok(T), Err(E) }

#[must_use]
pub struct ClientBuilder { /* ... */ }

impl ClientBuilder {
    #[must_use]
    pub fn timeout(mut self, t: Duration) -> Self { /* ... */ self }
}
```

The compiler warns when a `#[must_use]` value is dropped without being used.

## Object safety

A trait is object-safe (can be `Box<dyn Trait>` / `&dyn Trait`) if all its methods:
- Take `&self` or `&mut self` (not `self` or generic `Self`)
- Return types that don't reference `Self` (no `-> Self`)
- Don't have generic parameters on methods

If you need both static and dynamic dispatch, define the trait object-safely and provide an `impl Trait` API on top.
