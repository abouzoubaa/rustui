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
#[ignore = "needs patches/proposed/t13-modal-focus-recovery-v1.patch"]
fn dialog_restores_last_focused_child_after_reactivation(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Dialog, false);
}

#[gpui::test]
#[ignore = "needs patches/proposed/t13-modal-focus-recovery-v1.patch"]
fn dialog_restores_first_child_when_focused_child_was_removed(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Dialog, true);
}

#[gpui::test]
#[ignore = "needs patches/proposed/t13-modal-focus-recovery-v1.patch"]
fn sheet_restores_last_focused_child_after_reactivation(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Sheet, false);
}

#[gpui::test]
#[ignore = "needs patches/proposed/t13-modal-focus-recovery-v1.patch"]
fn sheet_restores_first_child_when_focused_child_was_removed(cx: &mut gpui::TestAppContext) {
    recover(cx, ModalKind::Sheet, true);
}
