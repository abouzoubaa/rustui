//! Root-window creation using GPUI and GPUI Components ownership.

use gpui::{
    AnyWindowHandle, App, AppContext as _, Bounds, Window, WindowBounds, WindowKind, WindowOptions,
    point, px, size,
};
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
pub type AnchoredPopupHandle = AnyWindowHandle;

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
    let window_options = WindowOptions {
        kind: WindowKind::AnchoredPopup(popup),
        window_bounds: Some(WindowBounds::Windowed(Bounds::new(
            point(px(0.0), px(0.0)),
            size(px(width), px(height)),
        ))),
        ..WindowOptions::default()
    };
    open_root_window_handle(root, environment, window_options, cx)
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
