//! Keyed insertion and removal transitions using GPUI's retained springs.
use crate::view::{RenderContext, RetentionScope};
use gpui::{
    AnimationExt as _, AnyElement, App, FocusHandle, InteractiveElement as _, IntoElement as _,
    ParentElement as _, Styled as _, Subscription, Window, div, px,
};
use rustui_core::Identity;
use rustui_i18n::LayoutDirection;
use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

/// Logical edge from which a child slides.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    Leading,
    Trailing,
    Top,
    Bottom,
}

/// The visual transition for a keyed child.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Transition {
    pub(crate) fade: bool,
    pub(crate) slide: Option<Edge>,
}
impl Transition {
    pub const fn fade() -> Self {
        Self {
            fade: true,
            slide: None,
        }
    }
    pub const fn slide(edge: Edge) -> Self {
        Self {
            fade: false,
            slide: Some(edge),
        }
    }
    pub const fn fade_slide(edge: Edge) -> Self {
        Self {
            fade: true,
            slide: Some(edge),
        }
    }
}

pub(crate) type Renderer = Rc<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;
type Children = [Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>];
struct Entry {
    key: Identity,
    render: Renderer,
    transition: Transition,
    index: usize,
    seen: bool,
    exiting: Rc<Cell<bool>>,
    finished: Rc<Cell<bool>>,
    exit_progress_seen: Rc<Cell<bool>>,
    focus: FocusHandle,
    _focus_subscription: Subscription,
}
struct Inner {
    entries: Vec<Entry>,
    focus: FocusHandle,
}
struct Stored {
    inner: Rc<RefCell<Inner>>,
}
/// The active stack collector; nested stacks replace and restore it.
pub(crate) struct Collector {
    inner: Rc<RefCell<Inner>>,
    index: Cell<usize>,
}

fn skip_focus(
    handle: &FocusHandle,
    container: &FocusHandle,
    backward: bool,
    window: &mut Window,
    app: &mut App,
) {
    for _ in 0..256 {
        if !handle.contains_focused(window, app) {
            return;
        }
        let before = window.focused(app);
        if backward {
            window.focus_prev(app);
        } else {
            window.focus_next(app);
        }
        if window.focused(app) == before {
            break;
        }
    }
    if handle.contains_focused(window, app) {
        container.focus(window, app);
    }
}

fn make_entry(
    key: Identity,
    render: Renderer,
    transition: Transition,
    index: usize,
    inner: &Rc<RefCell<Inner>>,
    window: &mut Window,
    app: &mut App,
) -> Entry {
    let focus = app.focus_handle();
    let exiting = Rc::new(Cell::new(false));
    let is_exiting = exiting.clone();
    let item_focus = focus.clone();
    let container = inner.borrow().focus.clone();
    let subscription = window.on_focus_in(&focus, app, move |window, app| {
        if is_exiting.get() {
            skip_focus(
                &item_focus,
                &container,
                window.modifiers().shift,
                window,
                app,
            );
        }
    });
    Entry {
        key,
        render,
        transition,
        index,
        seen: true,
        exiting,
        finished: Rc::new(Cell::new(false)),
        exit_progress_seen: Rc::new(Cell::new(false)),
        focus,
        _focus_subscription: subscription,
    }
}

fn render_entry(
    key: Identity,
    render: Renderer,
    transition: Transition,
    exiting: Rc<Cell<bool>>,
    finished: Rc<Cell<bool>>,
    exit_progress_seen: Rc<Cell<bool>>,
    focus: FocusHandle,
    context: &mut RenderContext<'_>,
) -> AnyElement {
    let direction = context.layout_direction();
    let child =
        context.with_element_scope(RetentionScope::Key(key.clone()), |context| render(context));
    let is_exiting = exiting.get();
    let mut wrapper = div()
        .id(format!("rustui-transition:{}", key.as_str()))
        .relative()
        .track_focus(&focus)
        .child(child);
    if is_exiting {
        wrapper = wrapper.child(div().absolute().inset_0().occlude());
    }
    let animation = gpui::SpringAnimation::new(gpui::SpringConfig::new(170.0, 26.0, 1.0))
        .to(if is_exiting { 0.0 } else { 1.0 })
        .from(if is_exiting { 1.0 } else { 0.0 })
        .with_epsilon(0.001);
    wrapper
        .with_spring(
            format!("rustui-transition-spring:{}", key.as_str()),
            animation,
            move |mut element, value: f32| {
                if exiting.get() {
                    if value > 0.0 {
                        exit_progress_seen.set(true);
                    } else if exit_progress_seen.get() {
                        finished.set(true);
                    }
                }
                let progress = value.clamp(0.0, 1.0);
                if transition.fade {
                    element = element.opacity(progress);
                }
                let offset = 16.0 * (1.0 - progress);
                match transition.slide {
                    Some(Edge::Leading) => {
                        element = element.left(px(if direction == LayoutDirection::RightToLeft {
                            offset
                        } else {
                            -offset
                        }))
                    }
                    Some(Edge::Trailing) => {
                        element = element.left(px(if direction == LayoutDirection::RightToLeft {
                            -offset
                        } else {
                            offset
                        }))
                    }
                    Some(Edge::Top) => element = element.top(px(-offset)),
                    Some(Edge::Bottom) => element = element.top(px(offset)),
                    None => {}
                }
                element
            },
        )
        .into_any_element()
}

pub(crate) fn render_keyed(
    key: Identity,
    render: Renderer,
    transition: Transition,
    context: &mut RenderContext<'_>,
) -> Option<AnyElement> {
    let collector = context.transition_collector.clone()?;
    if context.app().reduce_motion() {
        return None;
    }
    let index = collector.index.get();
    let exists = {
        let mut inner = collector.inner.borrow_mut();
        if let Some(entry) = inner.entries.iter_mut().find(|entry| entry.key == key) {
            assert!(!entry.seen, "duplicate transitioning key: {}", key.as_str());
            entry.seen = true;
            entry.index = index;
            entry.render = render.clone();
            entry.transition = transition;
            entry.exiting.set(false);
            entry.finished.set(false);
            entry.exit_progress_seen.set(false);
            true
        } else {
            false
        }
    };
    if !exists {
        let (window, app) = context.transition_window_app();
        let entry = make_entry(
            key.clone(),
            render,
            transition,
            index,
            &collector.inner,
            window,
            app,
        );
        collector.inner.borrow_mut().entries.push(entry);
    }
    let entry = {
        let inner = collector.inner.borrow();
        let entry = inner.entries.iter().find(|entry| entry.key == key).unwrap();
        (
            entry.key.clone(),
            entry.render.clone(),
            entry.transition,
            entry.exiting.clone(),
            entry.finished.clone(),
            entry.exit_progress_seen.clone(),
            entry.focus.clone(),
        )
    };
    context
        .transition_window_app()
        .0
        .set_focus_scope_inert(&entry.6, false);
    Some(render_entry(
        entry.0, entry.1, entry.2, entry.3, entry.4, entry.5, entry.6, context,
    ))
}

pub(crate) fn render_children(
    context: &mut RenderContext<'_>,
    children: &Children,
) -> (Vec<AnyElement>, Option<FocusHandle>) {
    let identity = context.next_implicit_retained_identity(Identity::from("transition-stack"));
    let (stored, _) = context.retained_entity(identity, |_, cx| Stored {
        inner: Rc::new(RefCell::new(Inner {
            entries: Vec::new(),
            focus: cx.focus_handle(),
        })),
    });
    let inner = stored.read(context.app()).inner.clone();
    if context.app().reduce_motion() {
        inner.borrow_mut().entries.clear();
    }
    for entry in &mut inner.borrow_mut().entries {
        entry.seen = false;
    }
    let collector = Rc::new(Collector {
        inner: inner.clone(),
        index: Cell::new(0),
    });
    let previous = context.transition_collector.replace(collector.clone());
    let mut rendered = Vec::with_capacity(children.len());
    for (index, child) in children.iter().enumerate() {
        collector.index.set(index);
        rendered.push(child(context));
    }
    context.transition_collector = previous;
    let mut exits = Vec::new();
    {
        let mut state = inner.borrow_mut();
        state
            .entries
            .retain(|entry| !entry.exiting.get() || !entry.finished.get() || entry.seen);
        let container = state.focus.clone();
        for entry in &mut state.entries {
            if !entry.seen {
                if !entry.exiting.replace(true) {
                    entry.finished.set(false);
                    let (window, app) = context.transition_window_app();
                    skip_focus(&entry.focus, &container, false, window, app);
                    window.set_focus_scope_inert(&entry.focus, true);
                }
                exits.push((
                    entry.index,
                    entry.key.clone(),
                    entry.render.clone(),
                    entry.transition,
                    entry.exiting.clone(),
                    entry.finished.clone(),
                    entry.exit_progress_seen.clone(),
                    entry.focus.clone(),
                ));
            }
        }
    }
    exits.sort_by_key(|entry| entry.0);
    for (index, key, render, transition, exiting, finished, exit_progress_seen, focus) in exits {
        let element = render_entry(
            key,
            render,
            transition,
            exiting,
            finished,
            exit_progress_seen,
            focus,
            context,
        );
        rendered.insert(index.min(rendered.len()), element);
    }
    let has_entries = !inner.borrow().entries.is_empty();
    if inner
        .borrow()
        .entries
        .iter()
        .any(|entry| entry.exiting.get())
    {
        let (window, _) = context.transition_window_app();
        schedule_exit_cleanup(window, Rc::downgrade(&inner));
    }
    let focus = has_entries.then(|| inner.borrow().focus.clone());
    (rendered, focus)
}

fn schedule_exit_cleanup(window: &mut Window, weak: Weak<RefCell<Inner>>) {
    window.on_next_frame(move |window, _| {
        let Some(inner) = weak.upgrade() else {
            return;
        };
        let mut inner = inner.borrow_mut();
        let before = inner.entries.len();
        inner
            .entries
            .retain(|entry| !entry.exiting.get() || !entry.finished.get());
        let changed = before != inner.entries.len();
        let still_exiting = inner.entries.iter().any(|entry| entry.exiting.get());
        drop(inner);
        if changed {
            window.refresh();
        }
        if still_exiting {
            schedule_exit_cleanup(window, weak);
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        layout::VStack,
        runtime::ComponentHost,
        view::{EmptyView, Keyed, View},
    };
    use gpui::{AppContext as _, Entity, StatefulInteractiveElement as _};
    use gpui_base::Root;
    use std::{cell::Cell, rc::Rc};

    struct Child {
        clicks: Rc<Cell<usize>>,
        focus: FocusHandle,
    }
    impl View for Child {
        fn body(&self) -> impl View {
            EmptyView
        }
        fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
            let clicks = self.clicks.clone();
            div()
                .id("transition-child")
                .debug_selector(|| "transition-child".into())
                .track_focus(&self.focus)
                .tab_stop(true)
                .w(px(100.))
                .h(px(40.))
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                .into_any_element()
        }
    }
    struct Owner {
        show: bool,
        clicks: Rc<Cell<usize>>,
        focus: FocusHandle,
        transition: bool,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            let mut stack = VStack::new();
            if self.show {
                let child = Keyed(
                    "child",
                    Child {
                        clicks: self.clicks.clone(),
                        focus: self.focus.clone(),
                    },
                );
                stack = if self.transition {
                    stack.child(child.transition(Transition::fade_slide(Edge::Leading)))
                } else {
                    stack.child(child)
                };
            }
            stack
        }
    }
    fn setup(
        cx: &mut gpui::TestAppContext,
    ) -> (
        Entity<ComponentHost<Owner>>,
        &mut gpui::VisualTestContext,
        Rc<Cell<usize>>,
    ) {
        let clicks = Rc::new(Cell::new(0));
        let focus = cx.update(|app| {
            gpui_component::init(app);
            app.focus_handle().tab_stop(true)
        });
        let result = Rc::new(RefCell::new(None));
        let slot = result.clone();
        let count = clicks.clone();
        let (_, window) = cx.add_window_view(move |window, cx| {
            let owner = cx.new(|cx| {
                ComponentHost::new(
                    Owner {
                        show: true,
                        clicks: count,
                        focus,
                        transition: true,
                    },
                    cx,
                )
            });
            *slot.borrow_mut() = Some(owner.clone());
            Root::new(owner, window, cx)
        });
        let owner = result.borrow_mut().take().unwrap();
        (owner, window, clicks)
    }
    fn draw(window: &mut gpui::VisualTestContext) {
        window.update(|window, app| {
            window.refresh();
            window.draw(app).clear(app);
        });
        window.cx.run_until_parked();
    }
    #[gpui::test]
    fn exiting_key_stays_visible_and_click_is_occluded(cx: &mut gpui::TestAppContext) {
        let (owner, window, clicks) = setup(cx);
        draw(window);
        std::thread::sleep(std::time::Duration::from_secs(1));
        window.update(|window, app| {
            window.simulate_next_frame(app);
        });
        draw(window);
        let before = window.debug_bounds("transition-child").unwrap().center();
        window.simulate_click(before, gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1);
        owner.update(window, |owner, cx| {
            owner.component_mut().show = false;
            cx.notify();
        });
        draw(window);
        assert!(window.debug_bounds("transition-child").is_some());
        let exiting = window.debug_bounds("transition-child").unwrap().center();
        window.simulate_click(exiting, gpui::Modifiers::default());
        assert_eq!(clicks.get(), 1);
    }
}

#[cfg(test)]
mod lifecycle_tests {
    use super::*;
    use crate::{
        layout::VStack,
        runtime::ComponentHost,
        view::{EmptyView, Keyed, View},
    };
    use gpui::{AppContext as _, Entity, StatefulInteractiveElement as _};
    use gpui_base::Root;
    use std::{cell::Cell, rc::Rc, time::Duration};

    struct Child {
        focus: FocusHandle,
        clicks: Rc<Cell<usize>>,
    }
    impl View for Child {
        fn body(&self) -> impl View {
            EmptyView
        }
        fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
            let clicks = self.clicks.clone();
            div()
                .id("life-child")
                .debug_selector(|| "life-child".into())
                .track_focus(&self.focus)
                .tab_stop(true)
                .w(px(80.))
                .h(px(30.))
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
                .into_any_element()
        }
    }
    struct Owner {
        show: bool,
        animated: bool,
        focus: FocusHandle,
        clicks: Rc<Cell<usize>>,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            let children = crate::view::ForEach(self.show.then_some(()), |_| {
                let view = Keyed(
                    "item",
                    Child {
                        focus: self.focus.clone(),
                        clicks: self.clicks.clone(),
                    },
                );
                if self.animated {
                    view.transition(Transition::fade_slide(Edge::Leading))
                } else {
                    view
                }
            });
            VStack::new().children(children)
        }
    }
    fn setup(
        cx: &mut gpui::TestAppContext,
        reduced: bool,
        animated: bool,
    ) -> (
        Entity<ComponentHost<Owner>>,
        &mut gpui::VisualTestContext,
        Rc<Cell<usize>>,
        FocusHandle,
    ) {
        let clicks = Rc::new(Cell::new(0));
        let focus = cx.update(|app| {
            gpui_component::init(app);
            app.set_reduce_motion(reduced);
            app.focus_handle().tab_stop(true)
        });
        let slot = Rc::new(RefCell::new(None));
        let target = slot.clone();
        let count = clicks.clone();
        let child_focus = focus.clone();
        let (_, window) = cx.add_window_view(move |window, cx| {
            let owner = cx.new(|cx| {
                ComponentHost::new(
                    Owner {
                        show: true,
                        animated,
                        focus: child_focus,
                        clicks: count,
                    },
                    cx,
                )
            });
            *target.borrow_mut() = Some(owner.clone());
            Root::new(owner, window, cx)
        });
        let owner = slot.borrow_mut().take().unwrap();
        (owner, window, clicks, focus)
    }
    fn draw(window: &mut gpui::VisualTestContext) {
        window.update(|window, app| {
            window.refresh();
            window.draw(app).clear(app);
        });
        window.cx.run_until_parked();
    }
    fn frames(window: &mut gpui::VisualTestContext, duration: Duration) -> usize {
        std::thread::sleep(duration);
        window.cx.executor().advance_clock(duration);
        let count = window.update(|window, app| window.simulate_next_frame(app));
        window.cx.run_until_parked();
        count
    }
    fn set_show(
        owner: &Entity<ComponentHost<Owner>>,
        window: &mut gpui::VisualTestContext,
        show: bool,
    ) {
        owner.update(window, |owner, cx| {
            owner.component_mut().show = show;
            cx.notify();
        });
        draw(window);
    }
    #[gpui::test]
    fn insertion_and_exit_settle_and_remove(cx: &mut gpui::TestAppContext) {
        let (owner, window, _, _) = setup(cx, false, true);
        draw(window);
        let first = window.debug_bounds("life-child").unwrap().origin.x;
        assert!(frames(window, Duration::from_secs(2)) > 0);
        let last = window.debug_bounds("life-child").unwrap().origin.x;
        assert!(
            last > first,
            "slide should move from its leading edge: first={first:?} last={last:?}"
        );
        set_show(&owner, window, false);
        assert!(window.debug_bounds("life-child").is_some());
        frames(window, Duration::from_secs(2));
        frames(window, Duration::from_millis(20));
        draw(window);
        assert!(window.debug_bounds("life-child").is_none());
    }
    #[gpui::test]
    fn reinserting_during_exit_keeps_one_instance(cx: &mut gpui::TestAppContext) {
        let (owner, window, _, _) = setup(cx, false, true);
        draw(window);
        frames(window, Duration::from_secs(2));
        set_show(&owner, window, false);
        frames(window, Duration::from_millis(50));
        set_show(&owner, window, true);
        frames(window, Duration::from_secs(2));
        draw(window);
        window.update(|_, app| {
            let key = Identity::from(format!(
                "rustui-text-{}:implicit-retained:0:transition-stack",
                owner.entity_id().as_u64()
            ));
            let state = owner
                .read(app)
                .retained_entity_for_test::<Stored>(vec![], key)
                .unwrap();
            let shared = state.read(app).inner.borrow();
            assert_eq!(shared.entries.len(), 1);
            assert!(!shared.entries[0].exiting.get());
        });
    }
    #[gpui::test]
    fn reduced_motion_and_no_transition_remove_immediately(cx: &mut gpui::TestAppContext) {
        let (owner, window, _, _) = setup(cx, true, true);
        draw(window);
        set_show(&owner, window, false);
        assert!(window.debug_bounds("life-child").is_none());
        assert_eq!(frames(window, Duration::from_millis(50)), 0);
        owner.update(window, |owner, cx| {
            owner.component_mut().show = true;
            owner.component_mut().animated = false;
            cx.notify();
        });
        draw(window);
        assert!(window.debug_bounds("life-child").is_some());
        set_show(&owner, window, false);
        assert!(window.debug_bounds("life-child").is_none());
    }
}

#[cfg(test)]
mod focus_tests {
    use super::*;
    use crate::{
        layout::VStack,
        runtime::ComponentHost,
        view::{EmptyView, Keyed, View},
    };
    use gpui::{AppContext as _, Entity, StatefulInteractiveElement as _};
    use gpui_base::Root;
    struct FocusNode {
        id: &'static str,
        focus: FocusHandle,
    }
    impl View for FocusNode {
        fn body(&self) -> impl View {
            EmptyView
        }
        fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
            div()
                .id(self.id)
                .track_focus(&self.focus)
                .tab_stop(true)
                .w(px(90.))
                .h(px(32.))
                .into_any_element()
        }
    }
    struct Owner {
        show: bool,
        before: FocusHandle,
        child: FocusHandle,
        after: FocusHandle,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            let mut stack = VStack::new().child(FocusNode {
                id: "before",
                focus: self.before.clone(),
            });
            if self.show {
                stack = stack.child(
                    Keyed(
                        "item",
                        FocusNode {
                            id: "focus-child",
                            focus: self.child.clone(),
                        },
                    )
                    .transition(Transition::fade()),
                );
            }
            stack.child(FocusNode {
                id: "after",
                focus: self.after.clone(),
            })
        }
    }
    fn draw(window: &mut gpui::VisualTestContext) {
        window.update(|window, app| {
            window.refresh();
            window.draw(app).clear(app);
        });
        window.cx.run_until_parked();
    }
    struct ScopeOwner {
        before: FocusHandle,
        scope: FocusHandle,
        child: FocusHandle,
        after: FocusHandle,
    }

    impl View for ScopeOwner {
        fn body(&self) -> impl View {
            EmptyView
        }

        fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
            div()
                .child(div().track_focus(&self.before).tab_stop(true))
                .child(
                    div()
                        .track_focus(&self.scope)
                        .child(div().track_focus(&self.child).tab_stop(true)),
                )
                .child(div().track_focus(&self.after).tab_stop(true))
                .into_any_element()
        }
    }

    #[gpui::test]
    fn inert_scope_skips_tab_and_rejects_direct_focus(cx: &mut gpui::TestAppContext) {
        let (before, scope, child, after) = cx.update(|app| {
            gpui_component::init(app);
            (
                app.focus_handle().tab_stop(true),
                app.focus_handle(),
                app.focus_handle().tab_stop(true),
                app.focus_handle().tab_stop(true),
            )
        });
        let (b, s, c, a) = (before.clone(), scope.clone(), child.clone(), after.clone());
        let (_, window) = cx.add_window_view(move |window, cx| {
            let owner = cx.new(|cx| {
                ComponentHost::new(
                    ScopeOwner {
                        before: b,
                        scope: s,
                        child: c,
                        after: a,
                    },
                    cx,
                )
            });
            Root::new(owner, window, cx)
        });
        draw(window);
        window.update(|window, app| {
            window.set_focus_scope_inert(&scope, true);
            before.focus(window, app);
            child.focus(window, app);
            assert_eq!(window.focused(app), Some(before.clone()));
            window.focus_next(app);
            assert_eq!(window.focused(app), Some(after.clone()));
            window.focus_prev(app);
            assert_eq!(window.focused(app), Some(before.clone()));
            window.set_focus_scope_inert(&scope, false);
            window.focus_next(app);
            assert_eq!(window.focused(app), Some(child.clone()));
        });
    }

    #[gpui::test]
    fn exit_moves_focus_out_and_tab_skips_exiting_child(cx: &mut gpui::TestAppContext) {
        let (before, child, after) = cx.update(|app| {
            gpui_component::init(app);
            (
                app.focus_handle().tab_stop(true),
                app.focus_handle().tab_stop(true),
                app.focus_handle().tab_stop(true),
            )
        });
        let slot = Rc::new(RefCell::new(None));
        let save = slot.clone();
        let b = before.clone();
        let c = child.clone();
        let a = after.clone();
        let (_, window) = cx.add_window_view(move |window, cx| {
            let owner: Entity<ComponentHost<Owner>> = cx.new(|cx| {
                ComponentHost::new(
                    Owner {
                        show: true,
                        before: b,
                        child: c,
                        after: a,
                    },
                    cx,
                )
            });
            *save.borrow_mut() = Some(owner.clone());
            Root::new(owner, window, cx)
        });
        let owner = slot.borrow_mut().take().unwrap();
        draw(window);
        window.update(|window, app| child.focus(window, app));
        owner.update(window, |owner, cx| {
            owner.component_mut().show = false;
            cx.notify();
        });
        draw(window);
        window.update(|window, app| assert_eq!(window.focused(app), Some(after.clone())));
        window.update(|window, app| before.focus(window, app));
        draw(window);
        window.simulate_keystrokes("tab");
        draw(window);
        window.update(|window, app| assert_eq!(window.focused(app), Some(after.clone())));
    }
}
