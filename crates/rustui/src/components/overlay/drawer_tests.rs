use super::*;
use crate::{
    prelude::{TextField, VStack},
    runtime::{Action, ComponentHost},
};
use gpui::AppContext as _;
use std::cell::RefCell;

struct Probe {
    visible: bool,
    mounted: bool,
    text: String,
    alternative: String,
    replace_binding: bool,
    error: Option<DrawerError>,
    writes: usize,
    edge: DrawerEdge,
    size: f32,
}
impl Default for Probe {
    fn default() -> Self {
        Self {
            visible: false,
            mounted: true,
            text: String::new(),
            alternative: String::new(),
            replace_binding: false,
            error: None,
            writes: 0,
            edge: DrawerEdge::Right,
            size: 350.0,
        }
    }
}
impl View for Probe {
    fn body(&self) -> impl View {
        let open = Binding::new(
            self.visible,
            |owner: &Self| owner.visible,
            |owner, value| {
                owner.visible = value;
                owner.writes += 1;
            },
        );
        let value = if self.replace_binding {
            Binding::new(
                self.alternative.clone(),
                |owner: &Self| owner.alternative.clone(),
                |owner, value| owner.alternative = value,
            )
        } else {
            Binding::new(
                self.text.clone(),
                |owner: &Self| owner.text.clone(),
                |owner, value| owner.text = value,
            )
        };
        let error = Binding::new(
            self.error,
            |owner: &Self| owner.error,
            |owner, value| owner.error = value,
        );
        let mut view =
            VStack::new().child(Button("Before", Action::new(|_: &mut Self| {})).id("before"));
        if self.mounted {
            view = view.child(
                Drawer("probe-drawer", open)
                    .edge(self.edge)
                    .size(self.size)
                    .accessibility_label("Probe dialog")
                    .localized_close_label("rustui-action-close")
                    .presentation_error(error)
                    .content(TextField("Notes", value).id("probe-notes")),
            );
        }
        view
    }
}
fn setup(cx: &mut gpui::TestAppContext) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
            .unwrap();
    });
}
fn window(
    cx: &mut gpui::TestAppContext,
) -> (Entity<ComponentHost<Probe>>, &mut gpui::VisualTestContext) {
    let owner = Rc::new(RefCell::new(None));
    let slot = owner.clone();
    let (_, window) = cx.add_window_view(|window, cx| {
        let entity = cx.new(|cx| ComponentHost::new(Probe::default(), cx));
        *slot.borrow_mut() = Some(entity.clone());
        Root::new(entity, window, cx)
    });
    let entity = owner.borrow_mut().take().unwrap();
    (entity, window)
}
fn draw(window: &mut gpui::VisualTestContext) {
    window.update(|window, app| {
        window.refresh();
        let _ = window.draw(app);
    });
    window.cx.run_until_parked();
}
fn set(
    owner: &Entity<ComponentHost<Probe>>,
    window: &mut gpui::VisualTestContext,
    change: impl FnOnce(&mut Probe),
) {
    owner.update(window, |host, cx| {
        change(host.component_mut());
        cx.notify();
    });
    draw(window);
}

#[gpui::test]
fn drawer_keeps_keyboard_focus_inside_modal(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    draw(window);
    set(&owner, window, |owner| owner.visible = true);
    for key in ["tab", "shift-tab"] {
        let mut visited = Vec::new();
        for _ in 0..5 {
            window.simulate_keystrokes(key);
            draw(window);
            let focus = window.update(|window, app| {
                let overlay = host(window, app).unwrap();
                let overlay = overlay.read(app);
                let active = overlay.active.as_ref().expect("drawer remains open");
                assert!(
                    active.focus.contains_focused(window, app),
                    "focus escaped the open drawer"
                );
                window.focused(app).expect("drawer has focused control")
            });
            if !visited.contains(&focus) {
                visited.push(focus);
            }
        }
        assert!(
            visited.len() >= 2,
            "{key} did not traverse the drawer controls"
        );
    }
}

#[gpui::test]
fn real_root_escape_restores_focus_and_does_not_repeat_visibility_updates(
    cx: &mut gpui::TestAppContext,
) {
    setup(cx);
    let (owner, window) = window(cx);
    draw(window);
    let previous = window.update(|window, app| {
        window.focus_next(app);
        window.focused(app).unwrap()
    });
    set(&owner, window, |owner| owner.visible = true);
    window.update(|window, app| {
        let host = host(window, app).unwrap();
        let active = host.read(app).active.as_ref().unwrap();
        assert_eq!(active.props.name.as_ref(), "Probe dialog");
        assert!(!active.props.close_label.trim().is_empty());
        assert!(active.focus.contains_focused(window, app));
        assert!(!can_open_drawer(window, app));
    });
    window.simulate_keystrokes("escape");
    draw(window);
    window.update(|window, app| {
        assert!(!owner.read(app).component().visible);
        assert_eq!(owner.read(app).component().writes, 1);
        assert!(host(window, app).unwrap().read(app).active.is_none());
        assert!(previous.is_focused(window));
    });
    draw(window);
    window.update(|_, app| assert_eq!(owner.read(app).component().writes, 1));
}

#[gpui::test]
fn hosted_input_uses_latest_original_owner_binding_and_keeps_presentation_identity(
    cx: &mut gpui::TestAppContext,
) {
    setup(cx);
    let (owner, window) = window(cx);
    draw(window);
    set(&owner, window, |owner| owner.visible = true);
    let token = window.update(|window, app| {
        host(window, app)
            .unwrap()
            .read(app)
            .active
            .as_ref()
            .unwrap()
            .token
            .clone()
    });
    window.simulate_keystrokes("tab");
    window.simulate_keystrokes("tab");
    window.simulate_keystrokes("x");
    draw(window);
    window.update(|_, app| assert_eq!(owner.read(app).component().text, "x"));
    set(&owner, window, |owner| {
        owner.replace_binding = true;
        owner.edge = DrawerEdge::Bottom;
        owner.size = 240.0;
    });
    window.update(|window, app| {
        let host = host(window, app).unwrap();
        let active = host.read(app).active.as_ref().unwrap();
        assert_eq!(active.token, token);
        assert_eq!(active.props.edge, DrawerEdge::Bottom);
        assert_eq!(active.props.size, 240.0);
    });
    window.simulate_keystrokes("y");
    draw(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().text, "x");
        assert_eq!(owner.read(app).component().alternative, "y");
    });
}

#[gpui::test]
fn unmount_and_stale_tokens_cannot_close_a_successor(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    draw(window);
    set(&owner, window, |owner| owner.visible = true);
    let old = window.update(|window, app| {
        host(window, app)
            .unwrap()
            .read(app)
            .active
            .as_ref()
            .unwrap()
            .token
            .clone()
    });
    set(&owner, window, |owner| owner.mounted = false);
    draw(window);
    window.update(|window, app| {
        assert!(host(window, app).unwrap().read(app).active.is_none());
        assert!(owner.read(app).component().visible);
    });
    set(&owner, window, |owner| owner.mounted = true);
    let new = window.update(|window, app| {
        host(window, app)
            .unwrap()
            .read(app)
            .active
            .as_ref()
            .unwrap()
            .token
            .clone()
    });
    assert_ne!(old, new);
    window.update(|window, app| {
        dismiss(&old, true, window, app);
        assert_eq!(
            host(window, app)
                .unwrap()
                .read(app)
                .active
                .as_ref()
                .unwrap()
                .token,
            new
        );
    });
    set(&owner, window, |owner| owner.visible = false);
    window.update(|window, app| assert!(host(window, app).unwrap().read(app).active.is_none()));
}

#[gpui::test]
fn native_modal_conflicts_are_rejected_without_closing_the_native_slot(
    cx: &mut gpui::TestAppContext,
) {
    setup(cx);
    let (owner, window) = window(cx);
    draw(window);
    for sheet in [true, false] {
        window.update(|window, app| {
            if sheet {
                window.open_sheet(app, |sheet, _, _| sheet.title("Native sheet"));
            } else {
                window.open_dialog(app, |dialog, _, _| dialog.title("Native dialog"));
            }
        });
        draw(window);
        set(&owner, window, |owner| owner.visible = true);
        draw(window);
        window.update(|window, app| {
            assert!(!owner.read(app).component().visible);
            assert_eq!(
                owner.read(app).component().error,
                Some(DrawerError::ModalConflict)
            );
            assert!(host(window, app).unwrap().read(app).active.is_none());
            if sheet {
                assert!(window.has_active_sheet(app));
                window.close_sheet(app);
            } else {
                assert!(window.has_active_dialog(app));
                window.close_dialog(app);
            }
        });
        draw(window);
    }
}

#[test]
fn finite_positive_sizes_are_required() {
    for size in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY, 0.0, -1.0] {
        let binding = Binding::new(
            false,
            |owner: &Probe| owner.visible,
            |owner, value| owner.visible = value,
        );
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                || Drawer("size", binding).size(size)
            ))
            .is_err()
        );
    }
}

#[gpui::test]
fn backdrop_and_named_close_control_use_one_guarded_dismissal_path(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    draw(window);
    set(&owner, window, |owner| owner.visible = true);
    window.simulate_click(gpui::point(px(20.0), px(20.0)), Default::default());
    draw(window);
    window.update(|_, app| {
        assert!(!owner.read(app).component().visible);
        assert_eq!(owner.read(app).component().writes, 1);
    });
    set(&owner, window, |owner| owner.visible = true);
    window.update(|window, app| window.focus_next(app));
    let keystroke = gpui::Keystroke::parse("space").unwrap();
    window.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    window.simulate_event(gpui::KeyUpEvent { keystroke });
    draw(window);
    window.update(|_, app| {
        assert!(!owner.read(app).component().visible);
        assert_eq!(owner.read(app).component().writes, 2);
    });
}

struct MountMarker {
    mounts: Rc<std::cell::Cell<usize>>,
}
impl View for MountMarker {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn on_mount(&mut self, _: &mut App) {
        self.mounts.set(self.mounts.get() + 1);
    }
}

struct CompatibilityProbe {
    mounts: [Rc<std::cell::Cell<usize>>; 3],
}
impl View for CompatibilityProbe {
    fn body(&self) -> impl View {
        let a = self.mounts[0].clone();
        let b = self.mounts[1].clone();
        let d = self.mounts[2].clone();
        VStack::new()
            .child(
                crate::prelude::Sheet("compat-sheet-a", "Sheet A")
                    .title("Sheet A")
                    .content(crate::prelude::Component("compat-body-a", move || {
                        MountMarker { mounts: a.clone() }
                    })),
            )
            .child(
                crate::prelude::Sheet("compat-sheet-b", "Sheet B")
                    .title("Sheet B")
                    .content(crate::prelude::Component("compat-body-b", move || {
                        MountMarker { mounts: b.clone() }
                    })),
            )
            .child(
                crate::prelude::Dialog("compat-dialog", "Dialog")
                    .title("Dialog")
                    .content(crate::prelude::Component("compat-body-dialog", move || {
                        MountMarker { mounts: d.clone() }
                    })),
            )
    }
}

fn activate_trigger(window: &mut gpui::VisualTestContext, trigger: &FocusHandle) {
    // Deliberate programmatic trigger activation exercises the production button
    // action even when another modal correctly occludes the background control.
    window.update(|window, app| window.focus(trigger, app));
    let key = gpui::Keystroke::parse("space").unwrap();
    window.simulate_event(gpui::KeyDownEvent {
        keystroke: key.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    window.simulate_event(gpui::KeyUpEvent { keystroke: key });
    draw(window);
}

#[gpui::test]
fn native_sheet_dialog_combination_and_sheet_replacement_remain_compatible(
    cx: &mut gpui::TestAppContext,
) {
    setup(cx);
    let mounts: [Rc<std::cell::Cell<usize>>; 3] =
        std::array::from_fn(|_| Rc::new(std::cell::Cell::new(0)));
    let counters = mounts.clone();
    let (_, window) = cx.add_window_view(move |window, cx| {
        let owner = cx.new(|cx| ComponentHost::new(CompatibilityProbe { mounts: counters }, cx));
        Root::new(owner, window, cx)
    });
    draw(window);
    assert!(mounts.iter().all(|count| count.get() == 0));
    let triggers = window.update(|window, app| {
        (0..3)
            .map(|_| {
                window.focus_next(app);
                window.focused(app).expect("three rendered modal triggers")
            })
            .collect::<Vec<_>>()
    });
    assert_ne!(triggers[0], triggers[1]);
    assert_ne!(triggers[1], triggers[2]);

    activate_trigger(window, &triggers[0]);
    assert_eq!(
        mounts[0].get(),
        1,
        "Sheet A's actual hosted content mounted"
    );
    window.update(|window, app| {
        assert!(window.has_active_sheet(app));
        assert!(!window.has_active_dialog(app));
    });

    activate_trigger(window, &triggers[2]);
    assert_eq!(mounts[2].get(), 1, "Dialog must still open above a Sheet");
    window.update(|window, app| {
        assert!(window.has_active_sheet(app));
        assert!(window.has_active_dialog(app));
    });

    activate_trigger(window, &triggers[1]);
    assert_eq!(
        mounts[1].get(),
        1,
        "Sheet B must replace A even while Dialog remains active"
    );
    window.update(|window, app| {
        assert!(window.has_active_sheet(app));
        assert!(window.has_active_dialog(app));
    });

    activate_trigger(window, &triggers[2]);
    window.update(|window, app| {
        window.close_dialog(app);
        assert!(
            !window.has_active_dialog(app),
            "the original active-Dialog guard prevented a second Dialog"
        );
        assert!(
            window.has_active_sheet(app),
            "closing Dialog must preserve the native Sheet"
        );
        window.close_sheet(app);
    });
    draw(window);
}
