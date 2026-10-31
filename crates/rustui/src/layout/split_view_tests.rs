use super::*;
use crate::{localization::activate_localization, runtime::ComponentHost};
use rustui_i18n::{Catalogs, Locale};
use std::{cell::Cell, rc::Rc};

struct SplitProbe {
    axis: Axis,
    unhandled: Rc<Cell<usize>>,
    three_panels: bool,
}

impl View for SplitProbe {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let split = match self.axis {
            Axis::Horizontal => SplitView::horizontal("event-path"),
            Axis::Vertical => SplitView::vertical("event-path"),
        }
        .panels(
            ["first", "second", "third"]
                .into_iter()
                .take(if self.three_panels { 3 } else { 2 })
                .map(|id| {
                    crate::prelude::ResizablePanel::new(id, EmptyView)
                        .initial_size(200.0)
                        .size_range(100.0..300.0)
                }),
        );
        let unhandled = self.unhandled.clone();
        div()
            .w(px(400.0))
            .h(px(400.0))
            .on_key_down(move |_, _, _| unhandled.set(unhandled.get() + 1))
            .child(split.into_element(context))
            .into_any_element()
    }
}

fn divider_size(native: &Entity<ResizableState>, cx: &mut gpui::VisualTestContext) -> f32 {
    cx.update(|window, app| {
        let _ = window.draw(app);
        native.read(app).sizes()[0].as_f32()
    })
}

#[gpui::test]
fn rendered_split_divider_drag_resizes_the_first_panel(cx: &mut gpui::TestAppContext) {
    cx.update(gpui_component::init);
    for (axis, three_panels) in [
        (Axis::Horizontal, false),
        (Axis::Vertical, false),
        (Axis::Horizontal, true),
        (Axis::Vertical, true),
    ] {
        let (owner, window) = cx.add_window_view(|_, app| {
            ComponentHost::new(
                SplitProbe {
                    axis,
                    unhandled: Rc::new(Cell::new(0)),
                    three_panels,
                },
                app,
            )
        });
        let native = window.update(|window, app| {
            let _ = window.draw(app);
            owner
                .read(app)
                .retained_entity_for_test::<RetainedSplit>(
                    vec![RetentionScope::SplitView(Identity::from("event-path"))],
                    Identity::from("split-state"),
                )
                .expect("the production render retained split state")
                .read(app)
                .native
                .clone()
        });
        let before = divider_size(&native, window);
        let point = |position| match axis {
            Axis::Horizontal => gpui::point(px(position), px(50.0)),
            Axis::Vertical => gpui::point(px(50.0), px(position)),
        };
        let modifiers = gpui::Modifiers::default();
        window.simulate_mouse_down(point(before - 2.0), gpui::MouseButton::Left, modifiers);
        window.simulate_mouse_move(
            point(before + 40.0),
            Some(gpui::MouseButton::Left),
            modifiers,
        );
        let after_first = divider_size(&native, window);
        assert!(
            after_first > before + 10.0,
            "{axis:?} divider did not move on the first event: {before} -> {after_first}"
        );
        window.simulate_mouse_move(
            point(before + 70.0),
            Some(gpui::MouseButton::Left),
            modifiers,
        );
        window.simulate_mouse_up(point(before + 70.0), gpui::MouseButton::Left, modifiers);
        let after_second = divider_size(&native, window);
        assert!(
            after_second > after_first + 10.0,
            "{axis:?} divider did not continue moving: {after_first} -> {after_second}"
        );
        // Deliver a complete drag before a draw can refresh the listeners.
        window.update(|window, app| {
            window.dispatch_event(
                gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                    position: point(after_second - 2.0),
                    button: gpui::MouseButton::Left,
                    modifiers,
                    click_count: 1,
                    first_mouse: false,
                }),
                app,
            );
            window.dispatch_event(
                gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent {
                    position: point(after_second - 40.0),
                    pressed_button: Some(gpui::MouseButton::Left),
                    modifiers,
                }),
                app,
            );
            window.dispatch_event(
                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                    position: point(after_second - 40.0),
                    button: gpui::MouseButton::Left,
                    modifiers,
                    click_count: 1,
                }),
                app,
            );
        });
        let after_release = divider_size(&native, window);
        assert!(
            after_release < after_second - 10.0,
            "{axis:?} second drag did not move before release: {after_second} -> {after_release}"
        );
        window.simulate_mouse_move(point(after_release + 60.0), None, modifiers);
        let after_idle_move = divider_size(&native, window);
        assert_eq!(
            after_release, after_idle_move,
            "{axis:?} divider moved after mouse release"
        );
        // If the OS misses mouse-up (for example after a focus change),
        // a move without the left button must end the resize.
        window.simulate_mouse_down(
            point(after_idle_move - 2.0),
            gpui::MouseButton::Left,
            modifiers,
        );
        window.simulate_mouse_move(
            point(after_idle_move - 30.0),
            Some(gpui::MouseButton::Left),
            modifiers,
        );
        let during_drag = divider_size(&native, window);
        assert!(during_drag < after_idle_move - 10.0);
        window.simulate_mouse_move(point(during_drag + 60.0), None, modifiers);
        let after_lost_release = divider_size(&native, window);
        assert_eq!(
            during_drag, after_lost_release,
            "{axis:?} divider moved without the mouse button"
        );
        window.simulate_mouse_move(
            point(during_drag + 80.0),
            Some(gpui::MouseButton::Left),
            modifiers,
        );
        assert_eq!(
            after_lost_release,
            divider_size(&native, window),
            "{axis:?} lost release did not end the drag"
        );
        window.simulate_mouse_up(
            point(during_drag + 80.0),
            gpui::MouseButton::Left,
            modifiers,
        );
    }
}

#[gpui::test]
fn focused_divider_routes_physical_arrows_and_clamps_through_the_rendered_control(
    cx: &mut gpui::TestAppContext,
) {
    cx.update(gpui_component::init);
    for (axis, locale, decrease, increase, unrelated, panel) in [
        (Axis::Horizontal, "en", "left", "right", "up", "first"),
        (Axis::Horizontal, "ar", "left", "right", "up", "second"),
        (Axis::Vertical, "ar", "up", "down", "left", "first"),
    ] {
        cx.update(|app| {
            activate_localization(
                Catalogs::new(Locale::parse("en").unwrap()),
                Locale::parse(locale).unwrap(),
                app,
            );
        });
        let unhandled = Rc::new(Cell::new(0));
        let (owner, window) = cx.add_window_view(|_, app| {
            ComponentHost::new(
                SplitProbe {
                    axis,
                    unhandled: unhandled.clone(),
                    three_panels: false,
                },
                app,
            )
        });
        let native = window.update(|window, app| {
            let _ = window.draw(app);
            let retained = owner
                .read(app)
                .retained_entity_for_test::<RetainedSplit>(
                    vec![RetentionScope::SplitView(Identity::from("event-path"))],
                    Identity::from("split-state"),
                )
                .expect("the production render retained the split state");
            assert_eq!(retained.read(app).ids[0].as_str(), panel);
            retained.read(app).native.clone()
        });
        let initial = divider_size(&native, window);
        assert_eq!(initial, 200.0);
        // Empty panels contain no controls; the rendered divider is the only tab stop.
        window.update(|window, app| {
            window.focus_next(app);
            assert!(window.focused(app).is_some());
        });

        window.simulate_keystrokes(decrease);
        assert_eq!(divider_size(&native, window), initial - 10.0);
        window.simulate_keystrokes(increase);
        assert_eq!(divider_size(&native, window), initial);
        assert_eq!(unhandled.get(), 0, "handled arrow must stop propagation");

        window.simulate_keystrokes(unrelated);
        window.simulate_keystrokes(&format!("shift-{decrease}"));
        assert_eq!(divider_size(&native, window), initial);
        assert_eq!(unhandled.get(), 2, "ignored keys must reach the ancestor");

        for _ in 0..12 {
            window.simulate_keystrokes(decrease);
        }
        assert_eq!(divider_size(&native, window), 100.0);
        for _ in 0..24 {
            window.simulate_keystrokes(increase);
        }
        assert_eq!(divider_size(&native, window), 300.0);
    }
}
