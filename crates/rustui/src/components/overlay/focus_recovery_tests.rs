//! Test-window coverage for modal focus recovery after window activation.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyElement, AppContext as _, Entity, FocusHandle, InteractiveElement as _, IntoElement as _,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, div, px,
};
use gpui_component::WindowExt as _;

use crate::{
    prelude::{Dialog, Sheet, VStack},
    runtime::ComponentHost,
    view::{EmptyView, RenderContext, View},
};

#[derive(Clone, Copy)]
enum ModalKind {
    Dialog,
    Sheet,
}

struct Owner {
    kind: ModalKind,
    first: FocusHandle,
    second: FocusHandle,
    behind: FocusHandle,
    show_second: bool,
}

impl View for Owner {
    fn body(&self) -> impl View {
        let content = ModalContent {
            first: self.first.clone(),
            second: self.second.clone(),
            show_second: self.show_second,
        };
        let root = VStack::new();
        let root = match self.kind {
            ModalKind::Dialog => root.child(Dialog("focus-dialog", "Open dialog").content(content)),
            ModalKind::Sheet => root.child(Sheet("focus-sheet", "Open sheet").content(content)),
        };
        root.child(FocusTarget {
            id: "behind-modal",
            focus: self.behind.clone(),
        })
    }
}

struct ModalContent {
    first: FocusHandle,
    second: FocusHandle,
    show_second: bool,
}

impl View for ModalContent {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        let mut content = div().flex().flex_col().child(
            div()
                .id("modal-first")
                .track_focus(&self.first)
                .tab_stop(true)
                .w(px(100.0))
                .h(px(24.0))
                .debug_selector(|| "modal-first".into()),
        );
        if self.show_second {
            content = content.child(
                div()
                    .id("modal-second")
                    .track_focus(&self.second)
                    .tab_stop(true)
                    .w(px(100.0))
                    .h(px(24.0))
                    .debug_selector(|| "modal-second".into()),
            );
        }
        content.into_any_element()
    }
}

struct FocusTarget {
    id: &'static str,
    focus: FocusHandle,
}

impl View for FocusTarget {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        div()
            .id(self.id)
            .track_focus(&self.focus)
            .tab_stop(true)
            .w(px(100.0))
            .h(px(24.0))
            .into_any_element()
    }
}

fn setup(
    cx: &mut gpui::TestAppContext,
    kind: ModalKind,
) -> (
    Entity<ComponentHost<Owner>>,
    &mut gpui::VisualTestContext,
    FocusHandle,
    FocusHandle,
    FocusHandle,
) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Light), app)
            .unwrap();
    });
    let (first, second, behind) =
        cx.update(|app| (app.focus_handle(), app.focus_handle(), app.focus_handle()));
    let slot = Rc::new(RefCell::new(None));
    let saved = slot.clone();
    let (_, window) = cx.add_window_view({
        let first = first.clone();
        let second = second.clone();
        let behind = behind.clone();
        move |window, cx| {
            let owner = cx.new(|cx| {
                ComponentHost::new(
                    Owner {
                        kind,
                        first,
                        second,
                        behind,
                        show_second: true,
                    },
                    cx,
                )
            });
            *saved.borrow_mut() = Some(owner.clone());
            gpui_component::Root::new(owner, window, cx)
        }
    });
    let owner = slot.borrow_mut().take().unwrap();
    (owner, window, first, second, behind)
}

fn draw(window: &mut gpui::VisualTestContext) {
    window.update(|window, app| {
        window.refresh();
        window.draw(app).clear(app);
    });
    window.run_until_parked();
}

fn open_modal(
    window: &mut gpui::VisualTestContext,
    kind: ModalKind,
    second: &FocusHandle,
    behind: &FocusHandle,
) -> (FocusHandle, FocusHandle, FocusHandle) {
    window.update(|window, _| window.activate_window());
    window.run_until_parked();
    draw(window);
    let trigger = window.update(|window, app| {
        assert!(window.is_window_active());
        window.focus_next(app);
        let trigger = window
            .focused(app)
            .expect("modal trigger must be focusable");
        assert_ne!(&trigger, behind);
        trigger
    });
    draw(window);
    let key = gpui::Keystroke::parse("enter").unwrap();
    window.simulate_event(gpui::KeyDownEvent {
        keystroke: key.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    window.simulate_event(gpui::KeyUpEvent { keystroke: key });
    draw(window);
    let modal_root = window.update(|window, app| {
        match kind {
            ModalKind::Dialog => assert!(window.has_active_dialog(app)),
            ModalKind::Sheet => assert!(window.has_active_sheet(app)),
        }
        let initial = window.focused(app).expect("modal starts with focus");
        assert_ne!(initial, trigger);
        assert_ne!(&initial, behind);
        window.focus_next(app);
        initial
    });
    draw(window);
    let first_modal = window.update(|window, app| {
        let first = window.focused(app).expect("modal has a first tab stop");
        assert_ne!(first, trigger);
        assert_ne!(&first, behind);
        assert_ne!(
            first, modal_root,
            "forward traversal must leave the modal root"
        );
        assert_ne!(
            &first, second,
            "first tab stop must survive the removal case"
        );
        assert!(modal_root.contains(&first, window));
        first
    });
    window.update(|window, app| window.focus(second, app));
    draw(window);
    window.update(|window, app| {
        assert_eq!(window.focused(app).as_ref(), Some(second));
        assert!(modal_root.contains(second, window));
    });
    assert!(window.debug_bounds("modal-first").is_some());
    assert!(window.debug_bounds("modal-second").is_some());
    (trigger, modal_root, first_modal)
}

fn recover(cx: &mut gpui::TestAppContext, kind: ModalKind, remove_focused: bool) {
    let (owner, window, _first, second, behind) = setup(cx, kind);
    let (trigger, modal_root, first_modal) = open_modal(window, kind, &second, &behind);
    let background_events = Rc::new(Cell::new(0));
    let subscriptions = window.update(|window, app| {
        let trigger_events = background_events.clone();
        let trigger_subscription = window.on_focus_in(&trigger, app, move |_, _| {
            trigger_events.set(trigger_events.get() + 1);
        });
        let behind_events = background_events.clone();
        let behind_subscription = window.on_focus_in(&behind, app, move |_, _| {
            behind_events.set(behind_events.get() + 1);
        });
        (trigger_subscription, behind_subscription)
    });
    window.run_until_parked();

    window.deactivate_window();
    window.run_until_parked();
    window.update(|window, _| assert!(!window.is_window_active()));
    if remove_focused {
        owner.update(window, |host, cx| {
            host.component_mut().show_second = false;
            cx.notify();
        });
        draw(window);
        assert!(window.debug_bounds("modal-second").is_none());
    }
    window.update(|window, app| {
        window.blur(app);
        assert!(window.focused(app).is_none(), "focus must actually be lost");
    });

    window.update(|window, _| window.activate_window());
    window.run_until_parked();
    window.update(|window, app| {
        window.focus(&behind, app);
        assert_ne!(window.focused(app).as_ref(), Some(&behind));
        assert_ne!(window.focused(app).as_ref(), Some(&trigger));
    });
    for _ in 0..2 {
        window.update(|window, app| {
            window.simulate_next_frame(app);
            assert_ne!(window.focused(app).as_ref(), Some(&behind));
            assert_ne!(window.focused(app).as_ref(), Some(&trigger));
        });
        draw(window);
        window.update(|window, app| {
            assert_ne!(window.focused(app).as_ref(), Some(&behind));
            assert_ne!(window.focused(app).as_ref(), Some(&trigger));
        });
    }
    assert!(window.debug_bounds("modal-first").is_some());
    assert_eq!(
        window.debug_bounds("modal-second").is_some(),
        !remove_focused
    );
    window.update(|window, app| {
        assert!(window.is_window_active());
        match kind {
            ModalKind::Dialog => assert!(window.has_active_dialog(app)),
            ModalKind::Sheet => assert!(window.has_active_sheet(app)),
        }
        let expected = if remove_focused {
            &first_modal
        } else {
            &second
        };
        assert!(
            modal_root.contains(expected, window),
            "expected focus must remain inside the modal"
        );
        assert_eq!(window.focused(app).as_ref(), Some(expected));
        assert_ne!(window.focused(app).as_ref(), Some(&behind));
        assert_ne!(window.focused(app).as_ref(), Some(&trigger));
    });
    // Focus listeners observe rendered frames, not changes that revert before a frame.
    assert_eq!(
        background_events.get(),
        0,
        "background focus was observed in a rendered frame"
    );
    drop(subscriptions);
}

#[gpui::test]
fn dialog_restores_last_focused_child_after_reactivation(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Dialog, false);
}

#[gpui::test]
fn dialog_restores_first_child_when_focused_child_was_removed(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Dialog, true);
}

#[gpui::test]
fn sheet_restores_last_focused_child_after_reactivation(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Sheet, false);
}

#[gpui::test]
fn sheet_restores_first_child_when_focused_child_was_removed(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Sheet, true);
}

struct BackgroundAfterRemoval {
    focus: FocusHandle,
}

impl gpui::Render for BackgroundAfterRemoval {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        _: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        div().track_focus(&self.focus).tab_stop(true)
    }
}

#[gpui::test]
fn modal_owner_removal_releases_focus_guard(cx: &mut gpui::TestAppContext) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Light), app)
            .unwrap();
    });
    for kind in [ModalKind::Dialog, ModalKind::Sheet] {
        let (first, second, behind) =
            cx.update(|app| (app.focus_handle(), app.focus_handle(), app.focus_handle()));
        let (retained_root, window) = cx.add_window_view({
            let first = first.clone();
            let second = second.clone();
            let behind = behind.clone();
            move |window, cx| {
                let owner = cx.new(|cx| {
                    ComponentHost::new(
                        Owner {
                            kind,
                            first,
                            second,
                            behind,
                            show_second: true,
                        },
                        cx,
                    )
                });
                gpui_component::Root::new(owner, window, cx)
            }
        });
        open_modal(window, kind, &second, &behind);
        window.update(|window, app| {
            window.focus(&behind, app);
            assert_ne!(window.focused(app).as_ref(), Some(&behind));
            let focus = behind.clone();
            window.replace_root(app, move |_, _| BackgroundAfterRemoval { focus });
        });
        draw(window);
        window.update(|window, app| {
            window.focus(&behind, app);
            assert_eq!(window.focused(app).as_ref(), Some(&behind));
        });
        drop(retained_root);
    }
}

struct AccessibleTrapProbe {
    focus: FocusHandle,
    label: &'static str,
}

impl View for AccessibleTrapProbe {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        use gpui_base::FocusTrapElement as _;
        div()
            .id("p1-accessible-trap")
            .role(gpui::Role::Dialog)
            .aria_label(self.label)
            .track_focus(&self.focus)
            .child("Modal content")
            .focus_trap("p1-accessible-trap-wrapper", &self.focus)
            .into_any_element()
    }
}

struct AccessibleModalOwner {
    kind: ModalKind,
    focus: FocusHandle,
}

impl View for AccessibleModalOwner {
    fn body(&self) -> impl View {
        let (id, label) = match self.kind {
            ModalKind::Dialog => ("p1-dialog", "P1 dialog focus trap"),
            ModalKind::Sheet => ("p1-sheet", "P1 sheet focus trap"),
        };
        let content = AccessibleTrapProbe {
            focus: self.focus.clone(),
            label,
        };
        match self.kind {
            ModalKind::Dialog => VStack::new().child(Dialog(id, "Open modal").content(content)),
            ModalKind::Sheet => VStack::new().child(Sheet(id, "Open modal").content(content)),
        }
    }
}

fn assert_open_modal_trap_accessibility(
    cx: &mut gpui::TestAppContext,
    kind: ModalKind,
    label: &str,
) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Light), app)
            .unwrap();
    });
    let (_, window) = cx.add_window_view(move |window, cx| {
        let focus = cx.focus_handle();
        let owner = cx.new(|cx| ComponentHost::new(AccessibleModalOwner { kind, focus }, cx));
        gpui_component::Root::new(owner, window, cx)
    });
    window.update(|window, app| {
        window.activate_window();
        window.draw(app).clear(app);
        window.focus_next(app);
    });
    window.run_until_parked();
    draw(window);
    let key = gpui::Keystroke::parse("enter").unwrap();
    window.simulate_event(gpui::KeyDownEvent {
        keystroke: key.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    window.simulate_event(gpui::KeyUpEvent { keystroke: key });
    draw(window);
    window.update(|window, app| {
        window.draw(app).clear(app);
        match kind {
            ModalKind::Dialog => assert!(window.has_active_dialog(app)),
            ModalKind::Sheet => assert!(window.has_active_sheet(app)),
        }
        let tree: serde_json::Value =
            serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap();
        assert!(
            tree["nodes"]
                .as_object()
                .unwrap()
                .values()
                .any(|node| { node["aria"]["role"] == "Dialog" && node["aria"]["label"] == label }),
            "focus-trap wrapper must forward Dialog role and label: {tree}"
        );
        assert!(
            tree["nodes"].as_object().unwrap().values().any(|node| {
                node["aria"]["role"] == "Dialog" && node["aria"]["label"] == "Open modal"
            }),
            "open modal must expose its own Dialog role and label: {tree}"
        );
    });
}

#[gpui::test]
fn focus_trap_forwards_dialog_role_and_label_in_open_dialog(cx: &mut gpui::TestAppContext) {
    assert_open_modal_trap_accessibility(cx, ModalKind::Dialog, "P1 dialog focus trap");
}

#[gpui::test]
fn focus_trap_forwards_dialog_role_and_label_in_open_sheet(cx: &mut gpui::TestAppContext) {
    assert_open_modal_trap_accessibility(cx, ModalKind::Sheet, "P1 sheet focus trap");
}

struct KeyboardMenuProbe {
    disabled_calls: Rc<Cell<usize>>,
    enabled_calls: Rc<Cell<usize>>,
    menu: Rc<RefCell<Option<gpui::WeakEntity<gpui_component::menu::PopupMenu>>>>,
}

impl gpui::Render for KeyboardMenuProbe {
    fn render(
        &mut self,
        _: &mut gpui::Window,
        _: &mut gpui::Context<Self>,
    ) -> impl gpui::IntoElement {
        use gpui_component::menu::{DropdownMenu as _, PopupMenuItem};
        let disabled_calls = self.disabled_calls.clone();
        let enabled_calls = self.enabled_calls.clone();
        let slot = self.menu.clone();
        gpui_component::button::Button::new("p1-menu-trigger")
            .label("Actions")
            .w(px(120.))
            .h(px(30.))
            .dropdown_menu(move |menu, _, cx| {
                *slot.borrow_mut() = Some(cx.weak_entity());
                let disabled_calls = disabled_calls.clone();
                let enabled_calls = enabled_calls.clone();
                menu.item(
                    PopupMenuItem::new("Disabled")
                        .disabled(true)
                        .on_click(move |_, _, _| disabled_calls.set(disabled_calls.get() + 1)),
                )
                .item(
                    PopupMenuItem::new("Enabled")
                        .on_click(move |_, _, _| enabled_calls.set(enabled_calls.get() + 1)),
                )
            })
    }
}

#[gpui::test]
fn disabled_popup_confirmation_keeps_menu_open_and_enabled_item_runs(
    cx: &mut gpui::TestAppContext,
) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Light), app)
            .unwrap();
    });
    let disabled_calls = Rc::new(Cell::new(0));
    let enabled_calls = Rc::new(Cell::new(0));
    let menu = Rc::new(RefCell::new(None));
    let (_, window) = cx.add_window_view({
        let disabled_calls = disabled_calls.clone();
        let enabled_calls = enabled_calls.clone();
        let menu = menu.clone();
        move |window, cx| {
            let owner = cx.new(|_| KeyboardMenuProbe {
                disabled_calls,
                enabled_calls,
                menu,
            });
            gpui_component::Root::new(owner, window, cx)
        }
    });
    draw(window);
    window.simulate_mouse_down(
        gpui::point(px(10.), px(10.)),
        gpui::MouseButton::Left,
        Default::default(),
    );
    window.run_until_parked();
    draw(window);
    assert!(
        menu.borrow()
            .as_ref()
            .and_then(|weak| weak.upgrade())
            .is_some()
    );
    window.simulate_keystrokes("down enter");
    window.run_until_parked();
    draw(window);
    assert_eq!(disabled_calls.get(), 0, "disabled action must not run");
    assert_eq!(enabled_calls.get(), 0);
    window.update(|window, app| {
        let tree: serde_json::Value =
            serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap();
        assert!(
            tree["nodes"]
                .as_object()
                .unwrap()
                .values()
                .any(|node| node["aria"]["role"] == "Menu"),
            "menu must remain open: {tree}"
        );
        assert!(
            menu.borrow()
                .as_ref()
                .and_then(|weak| weak.upgrade())
                .is_some()
        );
        let _ = app;
    });
    window.simulate_keystrokes("down enter");
    window.run_until_parked();
    assert_eq!(disabled_calls.get(), 0);
    assert_eq!(enabled_calls.get(), 1, "enabled action must run once");
}
