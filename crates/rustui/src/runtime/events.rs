//! Application actions that need GPUI context instead of component mutation.

use std::rc::Rc;

use gpui::{App, ClickEvent, Window};

use super::render_diagnostics::{
    RenderDiagnosticsSnapshot, clear_render_diagnostics as clear_diagnostics,
    disable_render_diagnostics as disable_diagnostics,
    enable_render_diagnostics as enable_diagnostics,
    render_diagnostics_snapshot as diagnostics_snapshot,
};

type AppActionHandler = dyn Fn(&mut Window, &mut App);
type ClickActionHandler = dyn Fn(&ClickEvent, &mut Window, &mut App);

#[derive(Clone)]
enum AppActionKind {
    Context(Rc<AppActionHandler>),
    Click(Rc<ClickActionHandler>),
}

#[derive(Clone)]
pub struct AppAction {
    invoke: AppActionKind,
}

impl AppAction {
    pub fn new(action: impl Fn(&mut App) + 'static) -> Self {
        Self {
            invoke: AppActionKind::Context(Rc::new(move |_window, cx| action(cx))),
        }
    }

    /// Sets GPUI's reduced-motion preference when this action runs.
    pub fn reduced_motion(reduced: bool) -> Self {
        Self::new(move |cx| cx.set_reduce_motion(reduced))
    }

    /// Refreshes all GPUI windows when this action runs.
    pub fn refresh_windows() -> Self {
        Self::new(|cx| cx.refresh_windows())
    }

    /// Starts a new render-diagnostics session with empty counters.
    pub fn enable_render_diagnostics() -> Self {
        Self::new(enable_diagnostics)
    }

    /// Clears render-diagnostics counters without ending the session.
    pub fn clear_render_diagnostics() -> Self {
        Self::new(clear_diagnostics)
    }

    /// Stops render-diagnostics recording and discards its counters.
    pub fn disable_render_diagnostics() -> Self {
        Self::new(disable_diagnostics)
    }

    /// Reads the current render-diagnostics snapshot into the supplied callback.
    pub fn read_render_diagnostics_snapshot(
        on_snapshot: impl Fn(RenderDiagnosticsSnapshot) + 'static,
    ) -> Self {
        Self::new(move |cx| on_snapshot(diagnostics_snapshot(cx)))
    }

    /// Reads whether GPUI currently considers the action's window accessibility-active.
    pub fn read_accessibility_active(on_status: impl Fn(bool) + 'static) -> Self {
        Self {
            invoke: AppActionKind::Context(Rc::new(move |window, _cx| {
                on_status(window.is_a11y_active());
            })),
        }
    }

    pub(crate) fn with_window_action(
        action: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            invoke: AppActionKind::Click(Rc::new(action)),
        }
    }

    /// Runs two application actions in order with the same native event context.
    pub fn then(self, next: Self) -> Self {
        if self.keyboard_compatible() && next.keyboard_compatible() {
            Self {
                invoke: AppActionKind::Context(Rc::new(move |window, cx| {
                    self.dispatch_keyboard(window, cx);
                    next.dispatch_keyboard(window, cx);
                })),
            }
        } else {
            Self {
                invoke: AppActionKind::Click(Rc::new(move |event, window, cx| {
                    self.dispatch(event, window, cx);
                    next.dispatch(event, window, cx);
                })),
            }
        }
    }

    pub(crate) fn keyboard_compatible(&self) -> bool {
        matches!(&self.invoke, AppActionKind::Context(_))
    }

    pub(crate) fn dispatch_keyboard(&self, window: &mut Window, cx: &mut App) {
        match &self.invoke {
            AppActionKind::Context(invoke) => invoke(window, cx),
            AppActionKind::Click(_) => {
                panic!("click-only application action used as keyboard action")
            }
        }
    }

    pub(crate) fn dispatch(&self, event: &ClickEvent, window: &mut Window, cx: &mut App) {
        match &self.invoke {
            AppActionKind::Context(invoke) => invoke(window, cx),
            AppActionKind::Click(invoke) => invoke(event, window, cx),
        }
    }
}
