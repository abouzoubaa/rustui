//! Owner-held status for GPUI-scheduled asynchronous work.

/// A component's current asynchronous work state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AsyncState<T, E> {
    Idle,
    Loading,
    Success(T),
    Error(E),
}

impl<T, E> Default for AsyncState<T, E> {
    fn default() -> Self {
        Self::Idle
    }
}
