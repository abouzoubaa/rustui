//! Root-window creation using GPUI and GPUI Components ownership.

use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext as _, Bounds, IntoElement, ParentElement as _,
    Pixels, Subscription, Window, WindowBounds, WindowKind, WindowOptions, point, px, size,
};
use gpui_component::ElementExt as _;
use rustui_core::EnvironmentValues;

use super::ComponentHost;
use crate::view::View;

pub(crate) fn open_root_window<V: View>(
    root: V,
    environment: EnvironmentValues,
    options: WindowOptions,
    cx: &mut App,
) -> Result<(), String> {
    open_root_window_handle(root, environment, options, cx).map(|_| ())
}

/// Geometry for a native anchored popup window.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AnchoredPopupOptions {
    /// Anchor rectangle in logical pixels relative to the parent window.
    pub anchor_rect: (f32, f32, f32, f32),
    /// Popup width and height in logical pixels.
    pub size: (f32, f32),
}

/// Handle used by RustUI applications to inspect or close an anchored popup.
#[cfg(not(target_family = "wasm"))]
#[derive(Clone)]
pub struct AnchoredPopupHandle {
    window: AnyWindowHandle,
    anchor_rect: std::rc::Rc<std::cell::Cell<Bounds<Pixels>>>,
    anchor_prepaint_generation: std::rc::Rc<std::cell::Cell<u64>>,
    anchor_detach_watch_started: std::rc::Rc<std::cell::Cell<bool>>,
    _parent_observers: std::rc::Rc<PopupParentObservers>,
}

#[cfg(not(target_family = "wasm"))]
struct PopupParentObservers {
    _bounds: Subscription,
    _display: Subscription,
}

#[cfg(not(target_family = "wasm"))]
impl AnchoredPopupHandle {
    /// Updates the popup's root window, matching GPUI's window handle API.
    pub fn update<C, R>(
        &self,
        cx: &mut C,
        update: impl FnOnce(gpui::AnyView, &mut Window, &mut App) -> R,
    ) -> gpui::Result<R>
    where
        C: gpui::AppContext,
    {
        self.window.update(cx, update)
    }

    /// Returns the most recently measured anchor rectangle in parent-window coordinates.
    pub fn anchor_rect(&self) -> Bounds<Pixels> {
        self.anchor_rect.get()
    }

    /// Returns the GPUI window handle owned by this popup.
    pub fn window_handle(&self) -> AnyWindowHandle {
        self.window
    }

    /// Repositions this popup relative to a new anchor rectangle.
    pub fn set_anchor_rect<C>(&self, rect: Bounds<Pixels>, cx: &mut C) -> Result<(), String>
    where
        C: gpui::AppContext,
    {
        self.window
            .update(cx, |_, window, _| {
                window.set_anchored_popup_anchor_rect(rect)
            })
            .map_err(|error| format!("{error:#}"))?
            .then(|| self.anchor_rect.set(rect))
            .ok_or_else(|| String::from("window does not support anchored popup repositioning"))
    }

    /// Measures this element during prepaint and follows it as layout or scrolling changes.
    /// The popup closes if the anchor is fully clipped out of its active clip or visible window.
    pub fn track_anchor<E>(self, element: E) -> AnyElement
    where
        E: gpui::ParentElement + IntoElement,
    {
        let popup = self.clone();
        let anchor_rect = self.anchor_rect.clone();
        let prepaint_generation = self.anchor_prepaint_generation.clone();
        let detach_watch_started = self.anchor_detach_watch_started.clone();
        element
            .on_prepaint(move |anchor_bounds, window, cx| {
                prepaint_generation.set(prepaint_generation.get().wrapping_add(1));
                let visible = anchor_bounds
                    .intersect(&window.content_mask().bounds)
                    .intersect(&window.fully_visible_bounds());
                if visible.size.width <= px(0.0) || visible.size.height <= px(0.0) {
                    detach_watch_started.set(false);
                    let _ = popup.update(cx, |_, popup_window, _| popup_window.remove_window());
                } else {
                    if anchor_rect.get() != anchor_bounds {
                        let _ = popup.set_anchor_rect(anchor_bounds, cx);
                    }
                    if !detach_watch_started.replace(true) {
                        schedule_anchor_detach_watch(
                            popup.clone(),
                            prepaint_generation.clone(),
                            detach_watch_started.clone(),
                            window,
                        );
                    }
                }
            })
            .into_any_element()
    }
}

#[cfg(not(target_family = "wasm"))]
struct PopupEscapeRoot<V: View> {
    child: V,
    parent: AnyWindowHandle,
    owner_focus: Option<gpui::WeakFocusHandle>,
    popup_focus: gpui::FocusHandle,
    focus_initialized: std::rc::Rc<std::cell::Cell<bool>>,
}

#[cfg(not(target_family = "wasm"))]
impl<V: View> View for PopupEscapeRoot<V> {
    fn body(&self) -> impl View {
        self.child.body()
    }

    fn on_mount(&mut self, cx: &mut App) {
        self.child.on_mount(cx);
    }

    fn on_unmount(&mut self, cx: &mut App) {
        self.child.on_unmount(cx);
    }

    fn into_element(&self, context: &mut crate::view::RenderContext<'_>) -> AnyElement {
        use gpui::{InteractiveElement as _, ParentElement as _, Styled as _};

        let child = self.child.into_element(context);
        let parent = self.parent;
        let owner_focus = self.owner_focus.clone();
        if !self.focus_initialized.replace(true) {
            let (window, app) = context.transition_window_app();
            self.popup_focus.focus(window, app);
        }
        gpui::div()
            .size_full()
            .track_focus(&self.popup_focus)
            .on_key_down(move |event, popup_window, cx| {
                if event.keystroke.key == "escape" {
                    popup_window.remove_window();
                    if let Some(focus) = owner_focus.as_ref().and_then(|focus| focus.upgrade()) {
                        let _ = parent.update(cx, |_, owner_window, app| {
                            focus.focus(owner_window, app);
                        });
                    }
                    cx.stop_propagation();
                }
            })
            .child(child)
            .into_any_element()
    }
}

#[cfg(not(target_family = "wasm"))]
fn schedule_anchor_detach_watch(
    popup: AnchoredPopupHandle,
    prepaint_generation: std::rc::Rc<std::cell::Cell<u64>>,
    detach_watch_started: std::rc::Rc<std::cell::Cell<bool>>,
    window: &mut Window,
) {
    // One callback runs after the current frame; it arms the check for the following frame.
    // That lets the trigger's prepaint run in that frame before we decide it was detached.
    window.on_next_frame(move |window, _| {
        let expected_generation = prepaint_generation.get();
        window.on_next_frame(move |window, cx| {
            if prepaint_generation.get() == expected_generation {
                let _ = popup.update(cx, |_, popup_window, _| popup_window.remove_window());
                detach_watch_started.set(false);
            } else {
                schedule_anchor_detach_watch(
                    popup,
                    prepaint_generation,
                    detach_watch_started,
                    window,
                );
            }
        });
    });
}

/// Opens a parent-anchored native popup.
///
/// The popup opens below the anchor and adjusts to remain on screen. Its parent
/// is the supplied window, so it follows that window and closes with it. On
/// platforms without native anchored-window support, this returns the platform
/// error; use RustUI's in-window `Popover` component for portable content. The
/// returned handle can be used to close the popup explicitly.
#[cfg(not(target_family = "wasm"))]
pub fn open_anchored_popup<V: View>(
    root: V,
    environment: EnvironmentValues,
    options: AnchoredPopupOptions,
    parent: &mut Window,
    cx: &mut App,
) -> Result<AnchoredPopupHandle, String> {
    let parent_handle = parent.window_handle();
    let owner_focus = parent.focused(cx).map(|focus| focus.downgrade());
    let root = PopupEscapeRoot {
        child: root,
        parent: parent_handle,
        owner_focus,
        popup_focus: cx.focus_handle(),
        focus_initialized: std::rc::Rc::new(std::cell::Cell::new(false)),
    };
    let (anchor_x, anchor_y, anchor_width, anchor_height) = options.anchor_rect;
    let (width, height) = options.size;
    if ![
        anchor_x,
        anchor_y,
        anchor_width,
        anchor_height,
        width,
        height,
    ]
    .into_iter()
    .all(f32::is_finite)
        || anchor_width < 0.0
        || anchor_height < 0.0
        || width <= 0.0
        || height <= 0.0
    {
        return Err(String::from(
            "popup values must be finite; anchor size must be non-negative and popup size positive",
        ));
    }

    let popup = gpui::popup::PopupOptions {
        parent: parent.window_handle().into(),
        anchor_rect: Bounds::new(
            point(px(anchor_x), px(anchor_y)),
            size(px(anchor_width), px(anchor_height)),
        ),
        anchor: gpui::popup::PopupAnchor::BottomLeft,
        gravity: gpui::popup::PopupGravity::BottomRight,
        constraint_adjustment: gpui::popup::PopupConstraintAdjustment::FLIP_X
            | gpui::popup::PopupConstraintAdjustment::FLIP_Y
            | gpui::popup::PopupConstraintAdjustment::SLIDE_X
            | gpui::popup::PopupConstraintAdjustment::SLIDE_Y
            | gpui::popup::PopupConstraintAdjustment::RESIZE_X
            | gpui::popup::PopupConstraintAdjustment::RESIZE_Y,
        offset: point(px(0.0), px(0.0)),
        grab: true,
    };
    let anchor_rect = std::rc::Rc::new(std::cell::Cell::new(popup.anchor_rect));
    let anchor_prepaint_generation = std::rc::Rc::new(std::cell::Cell::new(0));
    let anchor_detach_watch_started = std::rc::Rc::new(std::cell::Cell::new(false));
    let window_options = WindowOptions {
        kind: WindowKind::AnchoredPopup(popup),
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(0.0), px(0.0)),
            size(px(width), px(height)),
        ))),
        ..WindowOptions::default()
    };
    let window = open_root_window_handle(root, environment, window_options, cx)?;
    let bounds_window = window;
    let bounds_anchor = anchor_rect.clone();
    let bounds_subscription = parent.observe_window_bounds(move |_, cx| {
        let rect = bounds_anchor.get();
        let _ = bounds_window.update(cx, |_, popup, _| popup.set_anchored_popup_anchor_rect(rect));
    });
    let display_window = window;
    let display_anchor = anchor_rect.clone();
    let display_subscription = parent.observe_window_display(move |_, cx| {
        let rect = display_anchor.get();
        let _ = display_window.update(cx, |_, popup, _| popup.set_anchored_popup_anchor_rect(rect));
    });
    Ok(AnchoredPopupHandle {
        window,
        anchor_rect,
        anchor_prepaint_generation,
        anchor_detach_watch_started,
        _parent_observers: std::rc::Rc::new(PopupParentObservers {
            _bounds: bounds_subscription,
            _display: display_subscription,
        }),
    })
}

fn open_root_window_handle<V: View>(
    root: V,
    environment: EnvironmentValues,
    options: WindowOptions,
    cx: &mut App,
) -> Result<AnyWindowHandle, String> {
    cx.open_window(options, |window, cx| {
        let content = cx.new(|cx| ComponentHost::with_environment(root, environment, cx));
        let root = cx.new(|cx| gpui_component::Root::new(content, window, cx));
        #[cfg(not(target_family = "wasm"))]
        {
            let subscription = install_initial_focus_traversal(window.window_handle(), cx);
            cx.observe_release(&root, move |_, _| drop(subscription))
                .detach();
        }
        root
    })
    .map(Into::into)
    .map_err(|error| format!("{error:#}"))
}

#[cfg(not(target_family = "wasm"))]
fn install_initial_focus_traversal(
    handle: gpui::AnyWindowHandle,
    cx: &mut App,
) -> gpui::Subscription {
    use gpui_component::WindowExt as _;

    // With no focused element GPUI dispatches keys at its root node, outside
    // the Components Root key context. Seed traversal only for this window.
    cx.intercept_keystrokes(move |event, window, cx| {
        let key = &event.keystroke;
        let modifiers = key.modifiers;
        if window.window_handle() != handle
            || window.focused(cx).is_some()
            || key.key != "tab"
            || modifiers.control
            || modifiers.alt
            || modifiers.platform
            || modifiers.function
        {
            return;
        }
        // Components does not expose the active sheet/dialog focus handle.
        // Do not move focus behind either modal when its focus was lost.
        if window.has_active_sheet(cx) || window.has_active_dialog(cx) {
            return;
        }
        if crate::components::overlay::drawer::restore_active_focus(window, cx) {
            cx.stop_propagation();
            return;
        }
        if modifiers.shift {
            window.focus_prev(cx);
        } else {
            window.focus_next(cx);
        }
        if window.focused(cx).is_some() {
            cx.stop_propagation();
        }
    })
}

#[cfg(test)]
mod tests {
    #[cfg(not(target_family = "wasm"))]
    #[derive(Clone)]
    struct AnchorTrackingProbe {
        popup: std::rc::Rc<std::cell::RefCell<Option<super::AnchoredPopupHandle>>>,
        parent: std::rc::Rc<std::cell::Cell<Option<gpui::AnyWindowHandle>>>,
        x: std::rc::Rc<std::cell::Cell<f32>>,
        y: std::rc::Rc<std::cell::Cell<f32>>,
        measured: std::rc::Rc<std::cell::Cell<Option<gpui::Bounds<gpui::Pixels>>>>,
        scroll: gpui::ScrollHandle,
        show_anchor: bool,
    }

    #[cfg(not(target_family = "wasm"))]
    impl gpui::Render for AnchorTrackingProbe {
        fn render(
            &mut self,
            window: &mut gpui::Window,
            _: &mut gpui::Context<Self>,
        ) -> impl gpui::IntoElement {
            use gpui::{
                InteractiveElement as _, IntoElement as _, ParentElement as _,
                StatefulInteractiveElement as _, Styled as _,
            };
            use gpui_component::ElementExt as _;

            self.parent.set(Some(window.window_handle()));
            let content = if self.show_anchor {
                let measured = self.measured.clone();
                let anchor = gpui::div()
                    .debug_selector(|| "NP4_TRIGGER".into())
                    .ml(gpui::px(self.x.get()))
                    .mt(gpui::px(self.y.get()))
                    .w(gpui::px(24.0))
                    .h(gpui::px(24.0))
                    .on_prepaint(move |bounds, _, _| measured.set(Some(bounds)))
                    .child("popup anchor");
                if let Some(popup) = self.popup.borrow().clone() {
                    popup.track_anchor(anchor)
                } else {
                    anchor.into_any_element()
                }
            } else {
                gpui::div().into_any_element()
            };
            let content = gpui::div()
                .w(gpui::px(160.0))
                .h(gpui::px(360.0))
                .child(content);
            gpui::div()
                .id("np4-anchor-list")
                .w(gpui::px(160.0))
                .h(gpui::px(80.0))
                .overflow_y_scroll()
                .track_scroll(&self.scroll)
                .child(content)
        }
    }

    #[cfg(not(target_family = "wasm"))]
    fn open_tracked_popup(
        cx: &mut gpui::TestAppContext,
        popup_cell: std::rc::Rc<std::cell::RefCell<Option<super::AnchoredPopupHandle>>>,
        parent_cell: std::rc::Rc<std::cell::Cell<Option<gpui::AnyWindowHandle>>>,
        x: std::rc::Rc<std::cell::Cell<f32>>,
        y: std::rc::Rc<std::cell::Cell<f32>>,
        measured: std::rc::Rc<std::cell::Cell<Option<gpui::Bounds<gpui::Pixels>>>>,
        scroll: gpui::ScrollHandle,
    ) -> (
        gpui::Entity<AnchorTrackingProbe>,
        gpui::AnyWindowHandle,
        super::AnchoredPopupHandle,
    ) {
        use gpui::AppContext as _;

        let (owner, test_window) = cx.add_window_view(|_, _| AnchorTrackingProbe {
            popup: popup_cell.clone(),
            parent: parent_cell.clone(),
            x,
            y,
            measured: measured.clone(),
            scroll,
            show_anchor: true,
        });
        test_window.update(|window, app| {
            let _ = window.draw(app);
        });
        let parent = parent_cell.get().expect("probe records its parent window");
        let popup = test_window
            .update_window(parent, |_, window, app| {
                super::open_anchored_popup(
                    crate::view::EmptyView,
                    rustui_core::EnvironmentValues::default(),
                    super::AnchoredPopupOptions {
                        anchor_rect: (10.0, 20.0, 24.0, 24.0),
                        size: (120.0, 80.0),
                    },
                    window,
                    app,
                )
                .unwrap()
            })
            .unwrap();
        popup_cell.borrow_mut().replace(popup.clone());
        test_window.run_until_parked();
        test_window.update(|window, app| {
            window.refresh();
            let _ = window.draw(app);
        });
        assert_eq!(popup.anchor_rect(), measured.get().unwrap());
        assert_eq!(
            test_window.debug_bounds("NP4_TRIGGER"),
            measured.get(),
            "tracker measurement must equal the trigger's laid-out bounds"
        );
        drop(test_window);
        (owner, parent, popup)
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn moving_an_anchor_repositions_its_popup(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;
        use std::{cell::Cell, rc::Rc};

        cx.update(gpui_component::init);
        let popup_cell = Rc::new(std::cell::RefCell::new(None));
        let parent_cell = Rc::new(Cell::new(None));
        let x = Rc::new(Cell::new(10.0));
        let y = Rc::new(Cell::new(20.0));
        let measured = Rc::new(Cell::new(None));
        let scroll = gpui::ScrollHandle::default();
        let (owner, parent, popup) = open_tracked_popup(
            cx,
            popup_cell,
            parent_cell,
            x.clone(),
            y,
            measured.clone(),
            scroll,
        );

        x.set(60.0);
        owner.update(cx, |_, cx| cx.notify());
        parent
            .update(cx, |_, window, app| {
                window.refresh();
                let _ = window.draw(app);
            })
            .unwrap();
        cx.run_until_parked();
        let expected = measured.get().unwrap();
        assert_eq!(popup.anchor_rect(), expected);
        assert_eq!(
            cx.anchored_popup_anchor_rect(popup.window_handle()),
            Some(expected)
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn resizing_the_parent_repositions_its_popup(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;
        use std::{cell::Cell, rc::Rc};

        cx.update(gpui_component::init);
        let popup_cell = Rc::new(std::cell::RefCell::new(None));
        let parent_cell = Rc::new(Cell::new(None));
        let measured = Rc::new(Cell::new(None));
        let scroll = gpui::ScrollHandle::default();
        let (_owner, parent, popup) = open_tracked_popup(
            cx,
            popup_cell,
            parent_cell,
            Rc::new(Cell::new(10.0)),
            Rc::new(Cell::new(20.0)),
            measured.clone(),
            scroll,
        );

        cx.simulate_window_resize(parent, gpui::size(gpui::px(720.0), gpui::px(540.0)));
        cx.run_until_parked();
        parent
            .update(cx, |_, window, app| {
                window.refresh();
                let _ = window.draw(app);
            })
            .unwrap();
        cx.run_until_parked();
        assert_eq!(
            cx.anchored_popup_anchor_rect(popup.window_handle()),
            Some(measured.get().unwrap())
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn scrolling_an_anchor_out_of_view_closes_its_popup(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;
        use std::{cell::Cell, rc::Rc};

        cx.update(gpui_component::init);
        let popup_cell = Rc::new(std::cell::RefCell::new(None));
        let parent_cell = Rc::new(Cell::new(None));
        let x = Rc::new(Cell::new(10.0));
        let y = Rc::new(Cell::new(20.0));
        let measured = Rc::new(Cell::new(None));
        let scroll = gpui::ScrollHandle::default();
        let (owner, parent, popup) = open_tracked_popup(
            cx,
            popup_cell,
            parent_cell,
            x,
            y.clone(),
            measured,
            scroll.clone(),
        );

        scroll.set_offset(gpui::point(gpui::px(0.0), gpui::px(-100.0)));
        owner.update(cx, |_, cx| cx.notify());
        parent
            .update(cx, |_, window, app| {
                window.refresh();
                let _ = window.draw(app);
            })
            .unwrap();
        cx.run_until_parked();
        assert!(popup.update(cx, |_, _, _| ()).is_err());
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn removing_a_visible_anchor_closes_its_popup(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;
        use std::{cell::Cell, rc::Rc};

        cx.update(gpui_component::init);
        let popup_cell = Rc::new(std::cell::RefCell::new(None));
        let parent_cell = Rc::new(Cell::new(None));
        let (owner, parent, popup) = open_tracked_popup(
            cx,
            popup_cell,
            parent_cell,
            Rc::new(Cell::new(10.0)),
            Rc::new(Cell::new(20.0)),
            Rc::new(Cell::new(None)),
            gpui::ScrollHandle::default(),
        );

        parent
            .update(cx, |_, window, app| {
                window.simulate_next_frame(app);
            })
            .unwrap();
        owner.update(cx, |probe, cx| {
            probe.show_anchor = false;
            cx.notify();
        });
        parent
            .update(cx, |_, window, app| {
                window.refresh();
                let _ = window.draw(app);
                window.simulate_next_frame(app);
            })
            .unwrap();

        assert!(
            popup.update(cx, |_, _, _| ()).is_err(),
            "removing a still-visible trigger must close its anchored popup"
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn root_tab_focuses_and_activates_button(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;

        struct Owner(std::rc::Rc<std::cell::Cell<usize>>);
        impl crate::view::View for Owner {
            fn body(&self) -> impl crate::view::View {
                crate::prelude::Button(
                    "Increment",
                    crate::runtime::Action::new(|owner: &mut Owner| {
                        owner.0.set(owner.0.get() + 1);
                    }),
                )
            }
        }

        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let count = std::rc::Rc::new(std::cell::Cell::new(0));
        let (_, window) = cx.add_window_view(|window, app| {
            let content =
                app.new(|app| crate::runtime::ComponentHost::new(Owner(count.clone()), app));
            gpui_component::Root::new(content, window, app)
        });
        let _subscription = window.update(|window, app| {
            let _ = window.draw(app);
            super::install_initial_focus_traversal(window.window_handle(), app)
        });
        window.simulate_keystrokes("ctrl-tab");
        window.update(|window, app| assert!(window.focused(app).is_none()));
        window.simulate_keystrokes("tab");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(window.focused(app).is_some(), "Tab should establish focus");
        });
        let keystroke = gpui::Keystroke::parse("space").unwrap();
        window.simulate_event(gpui::KeyDownEvent {
            keystroke: keystroke.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        window.simulate_event(gpui::KeyUpEvent { keystroke });
        assert_eq!(count.get(), 1);
        window.update(|window, app| window.blur(app));
        window.simulate_keystrokes("shift-tab");
        window.update(|window, app| assert!(window.focused(app).is_some()));
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn escape_closes_anchored_popup_and_restores_owner_focus(cx: &mut gpui::TestAppContext) {
        use gpui::{
            InteractiveElement as _, ParentElement as _, StatefulInteractiveElement as _,
            Styled as _,
        };

        struct FocusOwner {
            focus: gpui::FocusHandle,
        }
        impl gpui::Render for FocusOwner {
            fn render(
                &mut self,
                _: &mut gpui::Window,
                _: &mut gpui::Context<Self>,
            ) -> impl gpui::IntoElement {
                gpui::div()
                    .track_focus(&self.focus)
                    .tab_stop(true)
                    .child("owner focus target")
            }
        }

        cx.update(gpui_component::init);
        let (owner, parent_window) = cx.add_window_view(|_, cx| FocusOwner {
            focus: cx.focus_handle(),
        });
        let (parent, owner_focus) = parent_window.update(|window, app| {
            let _ = window.draw(app);
            let focus = owner.read(app).focus.clone();
            window.focus(&focus, app);
            (window.window_handle(), focus)
        });
        let popup = parent
            .update(cx, |_, window, app| {
                super::open_anchored_popup(
                    crate::view::EmptyView,
                    rustui_core::EnvironmentValues::default(),
                    super::AnchoredPopupOptions {
                        anchor_rect: (10.0, 20.0, 24.0, 24.0),
                        size: (120.0, 80.0),
                    },
                    window,
                    app,
                )
                .unwrap()
            })
            .unwrap();

        cx.simulate_keystrokes(popup.window_handle(), "escape");
        cx.run_until_parked();

        assert!(popup.update(cx, |_, _, _| ()).is_err());
        assert!(
            parent
                .update(cx, |_, window, app| window.focused(app).as_ref()
                    == Some(&owner_focus))
                .unwrap()
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn popup_child_can_consume_escape_for_text_editing(cx: &mut gpui::TestAppContext) {
        struct EscapeConsumer {
            handled: std::rc::Rc<std::cell::Cell<bool>>,
            focus: gpui::FocusHandle,
        }
        impl crate::view::View for EscapeConsumer {
            fn body(&self) -> impl crate::view::View {
                crate::view::EmptyView
            }

            fn into_element(&self, _: &mut crate::view::RenderContext<'_>) -> gpui::AnyElement {
                use gpui::{InteractiveElement as _, IntoElement as _, Styled as _};

                let handled = self.handled.clone();
                gpui::div()
                    .size_full()
                    .track_focus(&self.focus)
                    .tab_stop(true)
                    .on_key_down(move |event, _, cx| {
                        if event.keystroke.key == "escape" {
                            handled.set(true);
                            cx.stop_propagation();
                        }
                    })
                    .into_any_element()
            }
        }

        struct Parent;
        impl gpui::Render for Parent {
            fn render(
                &mut self,
                _: &mut gpui::Window,
                _: &mut gpui::Context<Self>,
            ) -> impl gpui::IntoElement {
                gpui::div()
            }
        }

        cx.update(gpui_component::init);
        let child_focus = cx.update(|app| app.focus_handle());
        let handled = std::rc::Rc::new(std::cell::Cell::new(false));
        let (_, parent_window) = cx.add_window_view(|_, _| Parent);
        let parent = parent_window.update(|window, _| window.window_handle());
        let popup = parent
            .update(cx, |_, window, app| {
                super::open_anchored_popup(
                    EscapeConsumer {
                        handled: handled.clone(),
                        focus: child_focus.clone(),
                    },
                    rustui_core::EnvironmentValues::default(),
                    super::AnchoredPopupOptions {
                        anchor_rect: (10.0, 20.0, 24.0, 24.0),
                        size: (120.0, 80.0),
                    },
                    window,
                    app,
                )
                .unwrap()
            })
            .unwrap();

        popup
            .update(cx, |_, window, app| window.focus(&child_focus, app))
            .unwrap();
        cx.simulate_keystrokes(popup.window_handle(), "escape");
        cx.run_until_parked();

        assert!(handled.get(), "popup content should receive Escape first");
        assert!(popup.update(cx, |_, _, _| ()).is_ok());
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn initial_traversal_is_scoped_to_its_root_window(cx: &mut gpui::TestAppContext) {
        use gpui::AppContext as _;

        struct Owner;
        impl crate::view::View for Owner {
            fn body(&self) -> impl crate::view::View {
                crate::prelude::Button("Test", crate::runtime::Action::new(|_: &mut Owner| {}))
            }
        }

        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        let (owned, other) = cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
            super::open_root_window(
                Owner,
                rustui_core::EnvironmentValues::default(),
                gpui::WindowOptions::default(),
                app,
            )
            .unwrap();
            let owned = *app.windows().last().unwrap();
            let other = app
                .open_window(gpui::WindowOptions::default(), |window, app| {
                    let content = app.new(|app| super::ComponentHost::new(Owner, app));
                    app.new(|app| gpui_component::Root::new(content, window, app))
                })
                .unwrap();
            (owned, other.into())
        });
        for handle in [owned, other] {
            handle
                .update(cx, |_, window, app| {
                    let _ = window.draw(app);
                })
                .unwrap();
        }
        cx.simulate_keystrokes(other, "tab");
        other
            .update(cx, |_, window, app| assert!(window.focused(app).is_none()))
            .unwrap();
        cx.simulate_keystrokes(owned, "tab");
        owned
            .update(cx, |_, window, app| assert!(window.focused(app).is_some()))
            .unwrap();
        owned
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
        cx.run_until_parked();
        cx.simulate_keystrokes(other, "tab");
        other
            .update(cx, |_, window, app| assert!(window.focused(app).is_none()))
            .unwrap();
        other
            .update(cx, |_, window, _| window.remove_window())
            .unwrap();
    }

    struct LifecycleProbe {
        mounts: std::rc::Rc<std::cell::Cell<usize>>,
        unmounts: std::rc::Rc<std::cell::Cell<usize>>,
    }

    impl crate::view::View for LifecycleProbe {
        fn body(&self) -> impl crate::view::View {
            crate::view::EmptyView
        }

        fn on_mount(&mut self, _: &mut gpui::App) {
            self.mounts.set(self.mounts.get() + 1);
        }

        fn on_unmount(&mut self, _: &mut gpui::App) {
            self.unmounts.set(self.unmounts.get() + 1);
        }
    }

    #[gpui::test]
    fn opens_a_root_window_in_the_test_platform(cx: &mut gpui::TestAppContext) {
        cx.update(|app| {
            gpui_component::init(app);
            let before = app.windows().len();
            super::open_root_window(
                crate::view::EmptyView,
                rustui_core::EnvironmentValues::default(),
                gpui::WindowOptions::default(),
                app,
            )
            .expect("test platform should open a root window");
            assert_eq!(app.windows().len(), before + 1);
        });
    }

    #[gpui::test]
    fn closing_root_window_unmounts_the_owner(cx: &mut gpui::TestAppContext) {
        let mounts = std::rc::Rc::new(std::cell::Cell::new(0));
        let unmounts = std::rc::Rc::new(std::cell::Cell::new(0));
        let window = cx.update(|app| {
            gpui_component::init(app);
            super::open_root_window(
                LifecycleProbe {
                    mounts: mounts.clone(),
                    unmounts: unmounts.clone(),
                },
                rustui_core::EnvironmentValues::default(),
                gpui::WindowOptions::default(),
                app,
            )
            .unwrap();
            *app.windows().last().unwrap()
        });
        assert_eq!(mounts.get(), 1);
        cx.update(|app| {
            window
                .update(app, |_, window, _| window.remove_window())
                .unwrap();
        });
        cx.run_until_parked();
        assert_eq!(unmounts.get(), 1);
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn closing_anchored_popup_unmounts_its_root(cx: &mut gpui::TestAppContext) {
        let mounts = std::rc::Rc::new(std::cell::Cell::new(0));
        let unmounts = std::rc::Rc::new(std::cell::Cell::new(0));
        let parent = cx.update(|app| {
            gpui_component::init(app);
            super::open_root_window(
                crate::view::EmptyView,
                rustui_core::EnvironmentValues::default(),
                gpui::WindowOptions::default(),
                app,
            )
            .unwrap();
            *app.windows()
                .first()
                .expect("parent root window should exist")
        });
        let popup = cx.update(|app| {
            let previous_count = app.windows().len();
            let popup = parent
                .update(app, |_, window, app| {
                    super::open_anchored_popup(
                        LifecycleProbe {
                            mounts: mounts.clone(),
                            unmounts: unmounts.clone(),
                        },
                        rustui_core::EnvironmentValues::default(),
                        super::AnchoredPopupOptions {
                            anchor_rect: (10.0, 20.0, 1.0, 1.0),
                            size: (200.0, 120.0),
                        },
                        window,
                        app,
                    )
                    .unwrap()
                })
                .unwrap();
            assert_eq!(app.windows().len(), previous_count + 1);
            popup
        });

        cx.run_until_parked();
        assert_eq!(mounts.get(), 1);
        cx.update(|app| {
            popup
                .update(app, |_, window, _| window.remove_window())
                .unwrap();
        });
        cx.run_until_parked();
        assert_eq!(unmounts.get(), 1);
        assert!(
            popup.update(cx, |_, _, _| ()).is_err(),
            "closed popup handles should become stale"
        );
    }

    #[cfg(not(target_family = "wasm"))]
    #[gpui::test]
    fn popup_parent_can_open_another_popup_after_explicit_close(cx: &mut gpui::TestAppContext) {
        use std::{cell::Cell, rc::Rc};

        let parent = cx.update(|app| {
            gpui_component::init(app);
            super::open_root_window(
                crate::view::EmptyView,
                rustui_core::EnvironmentValues::default(),
                gpui::WindowOptions::default(),
                app,
            )
            .unwrap();
            *app.windows()
                .first()
                .expect("parent root window should exist")
        });

        let first_mounts = Rc::new(Cell::new(0));
        let first_unmounts = Rc::new(Cell::new(0));
        let first_popup = cx.update(|app| {
            let previous_count = app.windows().len();
            let popup = parent
                .update(app, |_, window, app| {
                    super::open_anchored_popup(
                        LifecycleProbe {
                            mounts: first_mounts.clone(),
                            unmounts: first_unmounts.clone(),
                        },
                        rustui_core::EnvironmentValues::default(),
                        super::AnchoredPopupOptions {
                            anchor_rect: (10.0, 20.0, 1.0, 1.0),
                            size: (200.0, 120.0),
                        },
                        window,
                        app,
                    )
                    .unwrap()
                })
                .unwrap();
            assert_eq!(app.windows().len(), previous_count + 1);
            popup
        });
        cx.run_until_parked();
        assert_eq!(first_mounts.get(), 1);
        cx.update(|app| {
            first_popup
                .update(app, |_, window, _| window.remove_window())
                .unwrap();
        });
        cx.run_until_parked();
        assert_eq!(first_unmounts.get(), 1);
        cx.update(|app| {
            assert_eq!(
                app.windows().len(),
                1,
                "first popup should leave only its parent window"
            );
        });

        let second_mounts = Rc::new(Cell::new(0));
        let second_unmounts = Rc::new(Cell::new(0));
        let second_popup = cx.update(|app| {
            let previous_count = app.windows().len();
            let popup = parent
                .update(app, |_, window, app| {
                    super::open_anchored_popup(
                        LifecycleProbe {
                            mounts: second_mounts.clone(),
                            unmounts: second_unmounts.clone(),
                        },
                        rustui_core::EnvironmentValues::default(),
                        super::AnchoredPopupOptions {
                            anchor_rect: (30.0, 40.0, 1.0, 1.0),
                            size: (180.0, 100.0),
                        },
                        window,
                        app,
                    )
                    .unwrap()
                })
                .unwrap();
            assert_eq!(app.windows().len(), previous_count + 1);
            popup
        });
        cx.run_until_parked();
        assert_eq!(second_mounts.get(), 1);
        cx.update(|app| {
            second_popup
                .update(app, |_, window, _| window.remove_window())
                .unwrap();
        });
        cx.run_until_parked();
        assert_eq!(second_unmounts.get(), 1);
        cx.update(|app| {
            assert_eq!(
                app.windows().len(),
                1,
                "second popup should leave only its parent window"
            );
        });
    }
}
