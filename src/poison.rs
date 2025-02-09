// NOTE: Design copied from `std::sync::poison`. The `_never` field prevents us from writing code
// that creates an instance of poison error when the panic strategy is abort.

/// The error that is returned when attempting to use a `crate::Transitionable` that has been
/// poisoned.
pub(crate) struct PoisonError {
    _private: (),

    #[cfg(panic = "abort")]
    _never: ::core::convert::Infallible,
}

impl PoisonError {
    #[cfg(not(panic = "abort"))]
    #[cold]
    pub(crate) const fn new() -> Self {
        Self { _private: () }
    }
}

impl core::error::Error for PoisonError {}

impl core::fmt::Debug for PoisonError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("PoisonError").finish_non_exhaustive()
    }
}
