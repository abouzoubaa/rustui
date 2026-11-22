use gpui::WindowId;
thread_local! {static DEFERRED_SOURCE:std::cell::RefCell<std::collections::HashSet<WindowId>>=Default::default();}
pub(crate) fn defer_source(window: WindowId) {
    DEFERRED_SOURCE.with(|s| s.borrow_mut().insert(window));
}
pub(crate) fn resume_source(window: WindowId, drained: bool) -> bool {
    if !DEFERRED_SOURCE.with(|s| s.borrow().contains(&window)) {
        return false;
    }
    if !drained {
        return false;
    }
    DEFERRED_SOURCE.with(|s| s.borrow_mut().remove(&window))
}

pub(crate) fn forget_source(window: WindowId) {
    DEFERRED_SOURCE.with(|s| s.borrow_mut().remove(&window));
}
