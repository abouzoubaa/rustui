//! Owner-held status for GPUI-scheduled asynchronous work.

/// A component's current asynchronous work state.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub enum AsyncState<T, E> {
    #[default]
    Idle,
    Loading,
    Success(T),
    Error(E),
}

#[cfg(test)]
mod tests {
    use super::AsyncState;

    #[test]
    fn default_does_not_require_default_payloads() {
        struct NoDefault;
        let _ = NoDefault;
        let state: AsyncState<NoDefault, NoDefault> = Default::default();
        assert!(matches!(state, AsyncState::Idle));
    }
}
