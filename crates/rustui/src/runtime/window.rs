//! Root-window creation using GPUI and GPUI Components ownership.

use gpui::{App, AppContext as _, WindowOptions};
use rustui_core::EnvironmentValues;

use super::ComponentHost;
use crate::view::View;

pub(crate) fn open_root_window<V: View>(
    root: V,
    environment: EnvironmentValues,
    options: WindowOptions,
    cx: &mut App,
) -> Result<(), String> {
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
    .map(|_| ())
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
}
