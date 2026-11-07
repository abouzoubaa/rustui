//! Test-window coverage for the public Menu popup path.

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, Entity, FocusHandle, InteractiveElement as _, IntoElement as _, ParentElement as _,
    div, point, px,
};

use crate::{
    prelude::{HorizontalAlignment, Menu, MenuItem, MenuView, VStack},
    runtime::{AppAction, ComponentHost},
    view::{EmptyView, RenderContext, View},
};

struct MeasuredMenu {
    menu: MenuView,
    owner_focus: FocusHandle,
}

impl View for MeasuredMenu {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        div()
            .debug_selector(|| "menu-lifecycle-owner".into())
            .track_focus(&self.owner_focus)
            .child(self.menu.into_element(context))
            .into_any_element()
    }
}

struct Owner {
    mounted: bool,
    generation: usize,
    calls: Rc<RefCell<Vec<usize>>>,
    token: Option<Rc<()>>,
    focus: FocusHandle,
}

impl View for Owner {
    fn body(&self) -> impl View {
        let mut view = VStack::new().alignment(HorizontalAlignment::Start);
        if self.mounted {
            let calls = self.calls.clone();
            let generation = self.generation;
            let token = self.token.clone();
            let action = AppAction::new(move |_| {
                let _ = &token;
                calls.borrow_mut().push(generation);
            });
            view = view.child(MeasuredMenu {
                menu: Menu("Actions", [MenuItem::action("run", "Run", action)])
                    .id("lifecycle-menu"),
                owner_focus: self.focus.clone(),
            });
        }
        view
    }
}

fn setup(
    cx: &mut gpui::TestAppContext,
    calls: Rc<RefCell<Vec<usize>>>,
    token: Rc<()>,
) -> (
    Entity<ComponentHost<Owner>>,
    &mut gpui::VisualTestContext,
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
    let focus = cx.update(|app| app.focus_handle());
    let owner_focus = focus.clone();
    let (owner, window) = cx.add_window_view(move |_, cx| {
        ComponentHost::new(
            Owner {
                mounted: true,
                generation: 0,
                calls,
                token: Some(token),
                focus: owner_focus,
            },
            cx,
        )
    });
    (owner, window, focus)
}

fn draw(window: &mut gpui::VisualTestContext) {
    window.update(|window, app| {
        window.refresh();
        window.draw(app).clear(app);
    });
    window.cx.run_until_parked();
}

fn click_owner(window: &mut gpui::VisualTestContext) {
    let bounds = window
        .debug_bounds("menu-lifecycle-owner")
        .expect("menu owner must be rendered");
    window.simulate_click(bounds.center(), Default::default());
    draw(window);
}

fn first_action_row_point(window: &mut gpui::VisualTestContext) -> gpui::Point<gpui::Pixels> {
    let owner = window
        .debug_bounds("menu-lifecycle-owner")
        .expect("menu owner must be rendered");
    point(
        owner.origin.x + px(16.0),
        owner.origin.y + owner.size.height + px(16.0),
    )
}

fn focus_owner(window: &mut gpui::VisualTestContext, focus: &FocusHandle) {
    window.update(|window, app| focus.focus(window, app));
    draw(window);
    assert!(window.update(|window, _| focus.is_focused(window)));
}

fn choose_first_action(window: &mut gpui::VisualTestContext) {
    window.simulate_keystrokes("down");
    window.simulate_keystrokes("enter");
    draw(window);
}

#[gpui::test]
fn menu_open_dispatches_its_action(cx: &mut gpui::TestAppContext) {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (_, window, focus) = setup(cx, calls.clone(), Rc::new(()));
    draw(window);
    focus_owner(window, &focus);
    click_owner(window);
    let row = first_action_row_point(window);
    window.simulate_click(row, Default::default());
    draw(window);
    assert_eq!(&*calls.borrow(), &[0]);
}

#[gpui::test]
fn menu_escape_and_outside_click_restore_owner_focus(cx: &mut gpui::TestAppContext) {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (_, window, owner_focus) = setup(cx, calls.clone(), Rc::new(()));
    draw(window);
    focus_owner(window, &owner_focus);

    click_owner(window);
    assert!(!window.update(|window, _| owner_focus.is_focused(window)));
    window.simulate_keystrokes("escape");
    draw(window);
    assert!(window.update(|window, _| owner_focus.is_focused(window)));
    let row = first_action_row_point(window);
    window.simulate_click(row, Default::default());
    draw(window);
    assert!(calls.borrow().is_empty());

    focus_owner(window, &owner_focus);
    click_owner(window);
    assert!(!window.update(|window, _| owner_focus.is_focused(window)));
    window.simulate_click(point(px(300.0), px(300.0)), Default::default());
    draw(window);
    assert!(window.update(|window, _| owner_focus.is_focused(window)));
    let row = first_action_row_point(window);
    window.simulate_click(row, Default::default());
    draw(window);
    assert!(calls.borrow().is_empty());
}

#[gpui::test]
fn menu_reopen_dispatches_the_current_action(cx: &mut gpui::TestAppContext) {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let (owner, window, focus) = setup(cx, calls.clone(), Rc::new(()));
    draw(window);
    focus_owner(window, &focus);
    click_owner(window);
    choose_first_action(window);
    assert_eq!(&*calls.borrow(), &[0]);

    owner.update(window, |host, cx| {
        host.component_mut().generation = 1;
        cx.notify();
    });
    draw(window);
    click_owner(window);
    choose_first_action(window);
    assert_eq!(&*calls.borrow(), &[0, 1]);
}

#[gpui::test]
fn menu_owner_removal_releases_the_open_action(cx: &mut gpui::TestAppContext) {
    let token = Rc::new(());
    let weak = Rc::downgrade(&token);
    let (owner, window, owner_focus) = setup(cx, Rc::new(RefCell::new(Vec::new())), token);
    draw(window);
    focus_owner(window, &owner_focus);
    click_owner(window);
    assert!(!window.update(|window, _| owner_focus.is_focused(window)));

    owner.update(window, |host, cx| {
        let owner = host.component_mut();
        owner.mounted = false;
        owner.token = None;
        cx.notify();
    });
    draw(window);
    draw(window);
    window.simulate_keystrokes("escape");
    draw(window);
    assert!(weak.upgrade().is_none(), "removed menu retained its action");
}
