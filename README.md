[actions-badge]: https://github.com/mickvangelderen/transitionable/workflows/main/badge.svg
[actions-url]: https://github.com/mickvangelderen/transitionable/actions/workflows/main.yaml?query=branch%3Amain
[mit-badge]: https://img.shields.io/badge/license-MIT-blue.svg
[mit-url]: https://github.com/mickvangelderen/transitionable/blob/master/LICENSE

[![Build Status][actions-badge]][actions-url]
[![MIT License][mit-badge]][mit-url]
![Crates.io Version](https://img.shields.io/crates/v/transitionable)

[docs-url]: https://docs.rs/transitionable/latest/transitionable/

[**Documentation**][docs-url]

# Transitionable

Allows transitioning a `T` from one state to the next by value, even when you only have access to a mutable reference `&mut T`.

```rust
use transitionable::Transitionable;

// Imagine this is your `T` type.
enum State { A, B }

impl State {
    // Imagine you would like to call this function which requires `Self` by value.
    fn swap(self) -> Self {
        match self {
            Self::A => Self::B,
            Self::B => Self::A,
        }
    }
}

// Imagine some other code forces you to work with `&mut`.
let t = &mut Transitionable::new(State::A);

// Despite our `Transitionable` being behind an `&mut`, we can consume the contained `State` by value and produce a new `State`.
Transitionable::transition(t, State::swap);

// Transitionable acts like a smart pointer; we can dereference it and verify that the value has indeed transitioned to a new state.
assert!(matches!(**t, State::B));
```

These crates offer similar functionality:

- [`takeable`](https://crates.io/crates/takeable) is similar but does not optimize when `panic = "abort"`.
- [`replace_with`](https://crates.io/crates/replace_with) can be used when you can not wrap your `T` in another type but has different performance characteristics and the optimized version is behind an `unsafe` function.

View the [documentation][docs-url] for more details.
