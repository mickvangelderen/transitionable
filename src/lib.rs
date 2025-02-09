//! This crate provides `Transitionable`, a type that can be used in places where you have an `&mut
//! T` but need a `T`.

#![cfg_attr(not(any(test, feature = "_test")), no_std)]
#![warn(missing_docs)]

mod poison;

use poison::PoisonError;

impl core::fmt::Display for PoisonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        "poisoned transitionable: lost value due to panic in previous transition".fmt(f)
    }
}

/// This type can be used in places where you have an `&mut T` but need a `T`. It is similar to
/// `Option` but with a more limited API that allows an optimization when the panic strategy is
/// abort.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Transitionable<T>(Inner<T>);

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
enum Inner<T> {
    Ok(T),
    #[cfg(not(panic = "abort"))]
    Poisoned,
}

// NOTE: Following C-SMART-PTR https://rust-lang.github.io/api-guidelines/predictability.html#smart-pointers-do-not-add-inherent-methods-c-smart-ptr

impl<T> Transitionable<T> {
    /// Construct a new `Transitionable`.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self(Inner::Ok(value))
    }

    /// Deconstructs the `Transitionable` back into its held value.
    ///
    /// # Panics
    ///
    /// Panics if the `Transitionable` is poisoned.
    #[inline]
    pub fn into_inner(transitionable: Self) -> T {
        Self::try_into_inner(transitionable).unwrap()
    }

    #[inline]
    fn try_into_inner(transitionable: Self) -> Result<T, PoisonError> {
        match transitionable.0 {
            Inner::Ok(value) => Ok(value),
            #[cfg(not(panic = "abort"))]
            Inner::Poisoned => Err(PoisonError::new()),
        }
    }

    /// Transition the held value by from one state to the next through the provided function.
    /// # Example
    ///
    /// ```rust
    /// # use transitionable::Transitionable;
    /// enum State { A, B }
    /// let t = &mut Transitionable::new(State::A);
    /// Transitionable::transition(t, |_: State| State::B);
    /// assert!(matches!(**t, State::B));
    /// ```
    /// # Panics
    ///
    /// Panics if the `Transitionable` is poisoned.
    #[inline]
    pub fn transition<F: FnOnce(T) -> T>(transitionable: &mut Self, f: F) -> &mut Self {
        Self::try_transition(transitionable, f).unwrap()
    }

    #[inline]
    fn try_transition<F: FnOnce(T) -> T>(
        transitionable: &mut Self,
        f: F,
    ) -> Result<&mut Self, PoisonError> {
        #[cfg(not(panic = "abort"))]
        {
            let value = match core::mem::replace(&mut transitionable.0, Inner::Poisoned {}) {
                Inner::Ok(value) => Ok(value),
                Inner::Poisoned => return Err(PoisonError::new()),
            }?;
            transitionable.0 = Inner::Ok(f(value));
        }
        #[cfg(panic = "abort")]
        {
            // SAFETY: We are guaranteed to overwrite the temporarily duplicated value since the
            // panic strategy is abort.
            unsafe {
                let Inner::Ok(value) = core::ptr::read(&transitionable.0);
                core::ptr::write(&mut transitionable.0, Inner::Ok(f(value)));
            }
        }
        Ok(transitionable)
    }

    /// A `Transitionable` becomes poisoned when a panic occurs inside the function passed to
    /// `Transitionable::transition`. To use a `Transitionable` in an application that may catch and recover
    /// from panics, you can use this function to determine whether a `Transitionable` is poisoned.
    /// If a `Transitionable` is poisoned, you will have to replace it by creating a new
    /// `Transitionable`.
    #[inline]
    pub const fn is_poisoned(transitionable: &Self) -> bool {
        #[cfg(not(panic = "abort"))]
        {
            matches!(transitionable.0, Inner::Poisoned)
        }
        #[cfg(panic = "abort")]
        {
            _ = transitionable;
            false
        }
    }

    #[inline]
    fn get(transitionable: &Self) -> &T {
        Self::try_get(transitionable).unwrap()
    }

    #[inline]
    fn try_get(transitionable: &Self) -> Result<&T, PoisonError> {
        match &transitionable.0 {
            Inner::Ok(value) => Ok(value),
            #[cfg(not(panic = "abort"))]
            Inner::Poisoned => Err(PoisonError::new()),
        }
    }

    #[inline]
    fn get_mut(transitionable: &mut Self) -> &mut T {
        Self::try_get_mut(transitionable).unwrap()
    }

    #[inline]
    fn try_get_mut(transitionable: &mut Self) -> Result<&mut T, PoisonError> {
        match &mut transitionable.0 {
            Inner::Ok(value) => Ok(value),
            #[cfg(not(panic = "abort"))]
            Inner::Poisoned => Err(PoisonError::new()),
        }
    }
}

impl<T> From<T> for Transitionable<T> {
    fn from(value: T) -> Self {
        Transitionable::new(value)
    }
}

impl<T> core::ops::Deref for Transitionable<T> {
    type Target = T;

    /// # Panics
    ///
    /// Panics if the `Transitionable` is poisoned.
    fn deref(&self) -> &Self::Target {
        Self::get(self)
    }
}

impl<T> core::ops::DerefMut for Transitionable<T> {
    /// # Panics
    ///
    /// Panics if the `Transitionable` is poisoned.
    fn deref_mut(&mut self) -> &mut Self::Target {
        Self::get_mut(self)
    }
}

// NOTE: Tests can not be run with `panic = "abort"`. To work around this we make the tests public
// and run them from a custom binary compiled with `panic = "abort"`.
#[cfg(any(test, feature = "_test"))]
#[doc(hidden)]
pub mod tests {
    use super::*;

    #[cfg(not(panic = "abort"))]
    fn poisoned() -> Transitionable<()> {
        let mut t = Transitionable::new(());
        assert!(std::panic::catch_unwind(core::panic::AssertUnwindSafe(|| {
            Transitionable::transition(&mut t, |_| panic!("oops"));
        }))
        .is_err());
        t
    }

    #[cfg_attr(test, test)]
    pub fn transition_works() {
        let mut t = Transitionable::new(());
        Transitionable::transition(&mut t, |x: ()| x);
    }

    #[cfg_attr(test, test)]
    pub fn is_poisoned_works() {
        {
            let t = Transitionable::new(());
            assert!(!Transitionable::is_poisoned(&t));
        }
        #[cfg(not(panic = "abort"))]
        {
            let t = poisoned();
            assert!(Transitionable::is_poisoned(&t));
        }
    }

    #[cfg_attr(test, test)]
    pub fn deref_works() {
        let t = Transitionable::new(());
        let _: &() = &t;
    }

    #[cfg(not(panic = "abort"))]
    #[test]
    #[should_panic]
    pub fn when_poisoned_panic_on_deref() {
        let t = poisoned();
        let _: &() = &t;
    }

    #[cfg_attr(test, test)]
    pub fn deref_mut_works() {
        let mut t = Transitionable::new(());
        let _: &mut () = &mut t;
    }

    #[cfg(not(panic = "abort"))]
    #[test]
    #[should_panic]
    pub fn when_poisoned_panic_on_deref_mut() {
        let mut t = poisoned();
        let _: &mut () = &mut t;
    }
}
