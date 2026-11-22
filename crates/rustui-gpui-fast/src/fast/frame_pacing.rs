// RustUI G5-2: opt-in frame pacing on GPUI's existing window/executor.
use crate::{PlatformInput, WindowId};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::OnceLock,
    time::{Duration, Instant},
};

pub fn fast_frame_ahead_enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| fast_view_link_requested())
}
pub fn fast_view_link_requested() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("GPUI_FAST_VIEW_DISPLAY_LINK").as_deref() == Ok("1"))
}
pub fn fast_force_legacy_link() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("GPUI_FAST_FORCE_LEGACY_LINK").as_deref() == Ok("1"))
}
struct InputState {
    at: Option<Instant>,
    motion: bool,
    view: bool,
}
thread_local! {static WINDOWS: RefCell<HashMap<WindowId,InputState>>=RefCell::new(HashMap::new());}
pub(crate) fn install(id: WindowId, view: bool) {
    if fast_frame_ahead_enabled() && (view || fast_force_legacy_link()) {
        WINDOWS.with(|w| {
            w.borrow_mut().insert(
                id,
                InputState {
                    at: None,
                    motion: false,
                    view,
                },
            );
        });
    }
}
pub(crate) fn input(id: WindowId, input: &PlatformInput) {
    if !fast_frame_ahead_enabled() {
        return;
    }
    let motion = matches!(input, PlatformInput::MouseMove(_));
    if !motion
        && !matches!(
            input,
            PlatformInput::KeyDown(_)
                | PlatformInput::MouseDown(_)
                | PlatformInput::MouseUp(_)
                | PlatformInput::Scroll(_)
        )
    {
        return;
    }
    WINDOWS.with(|w| {
        if let Some(s) = w.borrow_mut().get_mut(&id) {
            s.at = Some(Instant::now());
            s.motion = motion;
        }
    });
}
pub fn fast_frame_input_active(id: WindowId) -> bool {
    WINDOWS.with(|w| {
        w.borrow()
            .get(&id)
            .is_some_and(|s| s.at.is_some_and(|at| at.elapsed() < Duration::from_secs(1)))
    })
}
pub fn fast_frame_continuous(id: WindowId) -> bool {
    WINDOWS.with(|w| {
        w.borrow().get(&id).is_some_and(|s| {
            s.motion && s.at.is_some_and(|at| at.elapsed() < Duration::from_secs(1))
        })
    })
}
pub fn fast_frame_uses_view(id: WindowId) -> bool {
    WINDOWS.with(|w| w.borrow().get(&id).is_some_and(|s| s.view))
}
pub(crate) fn close(id: WindowId) {
    if crate::fast_immediate_enabled() {}
    WINDOWS.with(|w| {
        w.borrow_mut().remove(&id);
    });
}
