This crate provides a type called `Transitionable<T>` which gets you transition any `T` from one state to the next by value, even when you only have acess to a mutable reference `&mut T`.

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

- [`takable`](https://crates.io/crates/takeable) is similar but does not optimize when `panic = "abort"`.
- [`replace_with`](https://crates.io/crates/replace_with) can be used when you can not wrap your `T` in another type but has different performance characteristics and the optimized version is behind an `unsafe` function.
