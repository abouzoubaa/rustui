use crate::{
    components::overlay::popover::Popover,
    runtime::{Binding, ComponentHost},
    view::{EmptyView, RenderContext, View},
};
use gpui::{
    AnyElement, AppContext as _, Entity, FocusHandle, InteractiveElement as _, IntoElement as _,
    ParentElement as _, StatefulInteractiveElement as _, Styled as _, canvas, div, point, px,
};
use gpui_base::{GlobalState, Root};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

struct Probe {
    open: bool,
    mounted: bool,
    generation: u8,
    origin: (f32, f32),
    lifetime: Option<Rc<()>>,
    renders: Rc<Cell<usize>>,
    writes: usize,
}

impl Default for Probe {
    fn default() -> Self {
        Self {
            open: false,
            mounted: true,
            generation: 0,
            origin: (80.0, 80.0),
            lifetime: Some(Rc::new(())),
            renders: Rc::new(Cell::new(0)),
            writes: 0,
        }
    }
}

impl View for Probe {
    fn body(&self) -> impl View {
        let open = Binding::new(
            self.open,
            |owner: &Self| owner.open,
            |owner, value| {
                owner.open = value;
                owner.writes += 1;
            },
        );
        ProbeScene {
            open,
            mounted: self.mounted,
            generation: self.generation,
            origin: self.origin,
            lifetime: self.lifetime.clone(),
            renders: self.renders.clone(),
        }
    }
}

struct ProbeScene {
    open: Binding<bool>,
    mounted: bool,
    generation: u8,
    origin: (f32, f32),
    lifetime: Option<Rc<()>>,
    renders: Rc<Cell<usize>>,
}

impl View for ProbeScene {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut scene = div().size_full();
        if self.mounted {
            let popup = Popover(
                "lifecycle-popover",
                "Options",
                self.open.clone(),
                ContentProbe {
                    generation: self.generation,
                    lifetime: self.lifetime.as_ref().unwrap().clone(),
                    renders: self.renders.clone(),
                },
            );
            scene = scene.child(
                div()
                    .absolute()
                    .left(px(self.origin.0))
                    .top(px(self.origin.1))
                    .debug_selector(|| "popover-test-owner".into())
                    .child(popup.into_element(context)),
            );
        }
        scene.into_any_element()
    }
}

struct ContentProbe {
    generation: u8,
    lifetime: Rc<()>,
    renders: Rc<Cell<usize>>,
}

impl View for ContentProbe {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        let selector = if self.generation == 0 {
            "popover-test-content-0"
        } else {
            "popover-test-content-1"
        };
        let lifetime = self.lifetime.clone();
        let renders = self.renders.clone();
        div()
            .id("popover-test-content")
            .size(px(60.0))
            .debug_selector(move || selector.into())
            .child(
                canvas(
                    move |_, _, _| renders.set(renders.get() + 1),
                    |_, (), _, _| {},
                )
                .size_full(),
            )
            .on_click(move |_, _, _| {
                let _ = &lifetime;
            })
            .into_any_element()
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
        window.draw(app).clear(app);
    });
    window.cx.run_until_parked();
}

fn update(
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

fn open(owner: &Entity<ComponentHost<Probe>>, window: &mut gpui::VisualTestContext) -> FocusHandle {
    draw(window);
    let owner_focus = window.update(|window, app| {
        window.focus_next(app);
        window.focused(app).expect("popover owner is focusable")
    });
    window
        .debug_bounds("popover-test-owner")
        .expect("popover owner is rendered");
    // Enter can open the popup only when focus_next selected its owner control.
    window.simulate_keystrokes("enter");
    draw(window);
    window.update(|_, app| {
        assert!(owner.read(app).component().open);
        assert!(GlobalState::is_in_deferred_context(app));
    });
    assert!(window.debug_bounds("popover-test-content-0").is_some());
    owner_focus
}

fn assert_closed(
    owner: &Entity<ComponentHost<Probe>>,
    window: &mut gpui::VisualTestContext,
    owner_focus: &FocusHandle,
) {
    draw(window);
    window.update(|window, app| {
        assert!(!owner.read(app).component().open);
        assert_eq!(window.focused(app).as_ref(), Some(owner_focus));
        assert!(!GlobalState::is_in_deferred_context(app));
    });
    assert!(window.debug_bounds("popover-test-content-0").is_none());
    assert!(window.debug_bounds("popover-test-content-1").is_none());
}

#[gpui::test]
fn popover_opens_and_follows_its_owner(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    let _ = open(&owner, window);
    let before_owner = window.debug_bounds("popover-test-owner").unwrap();
    let before_popup = window.debug_bounds("popover-test-content-0").unwrap();
    assert!(before_popup.top() >= before_owner.bottom());
    assert!(before_popup.top() <= before_owner.bottom() + px(48.0));
    assert!(before_popup.left() >= before_owner.left() - px(24.0));
    assert!(before_popup.left() <= before_owner.right() + px(24.0));

    update(&owner, window, |owner| owner.origin = (120.0, 110.0));
    let after_owner = window.debug_bounds("popover-test-owner").unwrap();
    let after_popup = window.debug_bounds("popover-test-content-0").unwrap();
    assert_eq!(
        after_popup.origin.x - before_popup.origin.x,
        after_owner.origin.x - before_owner.origin.x
    );
    assert_eq!(
        after_popup.origin.y - before_popup.origin.y,
        after_owner.origin.y - before_owner.origin.y
    );
}

#[gpui::test]
fn popover_escape_closes_and_restores_owner_focus(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    let owner_focus = open(&owner, window);
    window.simulate_keystrokes("escape");
    assert_closed(&owner, window, &owner_focus);
}

#[gpui::test]
fn popover_outside_click_closes_and_restores_owner_focus(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    let owner_focus = open(&owner, window);
    window.simulate_click(point(px(5.0), px(5.0)), Default::default());
    assert_closed(&owner, window, &owner_focus);
}

#[gpui::test]
fn popover_reopens_with_fresh_content_and_no_stale_popup(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    let owner_focus = open(&owner, window);
    window.simulate_keystrokes("escape");
    assert_closed(&owner, window, &owner_focus);
    update(&owner, window, |owner| owner.generation = 1);

    let bounds = window.debug_bounds("popover-test-owner").unwrap();
    window.simulate_click(bounds.center(), Default::default());
    draw(window);
    window.update(|_, app| {
        assert!(owner.read(app).component().open);
        assert_eq!(owner.read(app).component().writes, 3);
        assert!(GlobalState::is_in_deferred_context(app));
    });
    assert!(window.debug_bounds("popover-test-content-0").is_none());
    assert!(window.debug_bounds("popover-test-content-1").is_some());
    let renders = window.update(|_, app| owner.read(app).component().renders.clone());
    renders.set(0);
    window.update(|window, app| {
        window.refresh();
        window.draw(app).clear(app);
    });
    assert_eq!(
        renders.get(),
        1,
        "reopen rendered more than one popup surface"
    );
    window.simulate_keystrokes("escape");
    assert_closed(&owner, window, &owner_focus);
}

#[gpui::test]
fn popover_owner_removal_drops_open_popup_context(cx: &mut gpui::TestAppContext) {
    setup(cx);
    let (owner, window) = window(cx);
    let _ = open(&owner, window);
    let lifetime = window
        .update(|_, app| Rc::downgrade(owner.read(app).component().lifetime.as_ref().unwrap()));
    update(&owner, window, |owner| {
        owner.mounted = false;
        owner.lifetime = None;
    });
    draw(window);
    assert!(window.debug_bounds("popover-test-owner").is_none());
    assert!(window.debug_bounds("popover-test-content-0").is_none());
    window.update(|_, app| assert!(!GlobalState::is_in_deferred_context(app)));
    assert!(lifetime.upgrade().is_none(), "popup content was retained");
}
