//! Presentation demand for the current UI-thread frame request.
use std::{cell::Cell, marker::PhantomData, rc::Rc};

thread_local! {
    static REQUIRED: Cell<bool> = const { Cell::new(true) };
}

// Keep the guard on its originating thread, including during nested callbacks.
pub(crate) struct RequestHint {
    previous: bool,
    _ui_thread: PhantomData<Rc<()>>,
}

pub(crate) fn for_request(require_presentation: bool, high_rate_input: bool) -> RequestHint {
    let previous =
        REQUIRED.with(|required| required.replace(require_presentation || high_rate_input));
    RequestHint {
        previous,
        _ui_thread: PhantomData,
    }
}

impl Drop for RequestHint {
    fn drop(&mut self) {
        REQUIRED.with(|required| required.set(self.previous));
    }
}

/// Whether the current frame must be presented even when its pixels are unchanged.
/// System presentation requests and sustained fast input require presentation.
/// Outside a frame request the conservative answer is `true`.
pub fn fast_present_required() -> bool {
    REQUIRED.with(Cell::get)
}

/// Run a test callback with the same scoped hint used by frame requests.
#[cfg(any(test, feature = "test-support"))]
pub fn with_fast_present_required<R>(required: bool, run: impl FnOnce() -> R) -> R {
    let _request = for_request(required, false);
    run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_required_outside_frame_requests() {
        assert!(fast_present_required());
    }

    #[test]
    fn system_request_or_fast_input_requires_presentation() {
        for (system, input, expected) in [
            (false, false, false),
            (false, true, true),
            (true, false, true),
            (true, true, true),
        ] {
            let request = for_request(system, input);
            assert_eq!(fast_present_required(), expected);
            drop(request);
            assert!(fast_present_required());
        }
    }

    #[test]
    fn nested_requests_restore_the_enclosing_hint() {
        let outer = for_request(false, false);
        assert!(!fast_present_required());
        {
            let _inner = for_request(true, false);
            assert!(fast_present_required());
        }
        assert!(!fast_present_required());
        drop(outer);
        assert!(fast_present_required());
    }
}
