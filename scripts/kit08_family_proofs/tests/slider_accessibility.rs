use gpui::{
    AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Styled as _,
    TestAppContext, Window,
    accesskit::{Action, ActionData},
    div, px,
};
use gpui_base::slider::{Slider, SliderState};

struct SliderHost {
    state: Entity<SliderState>,
}

impl Render for SliderHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        Slider::new(&self.state)
            .accessibility_label("Volume")
            .size(px(100.))
            .child(div().size(px(20.)))
    }
}

#[gpui::test]
fn slider_accessibility_actions_are_advertised_and_clamp_to_range(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(10.)
                .step(3.)
                .default_value(6.)
        })
    });
    let (view, window) = cx.add_window_view({
        let state = state.clone();
        move |_, _| SliderHost { state }
    });
    window.update(|window, cx| {
        let _ = view.read(cx);
        window.draw(cx).clear(cx);
    });

    let tree = window.update(|window, _| window.debug_a11y_tree_json().unwrap());
    assert!(
        tree.contains("Volume"),
        "slider label missing from tree: {tree}"
    );
    for action in [Action::SetValue, Action::Increment, Action::Decrement] {
        assert!(
            tree.contains(&format!("{action:?}")),
            "{action:?} was not advertised by the slider: {tree}"
        );
    }

    window.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.set_accessibility_value(Some(&ActionData::NumericValue(8.8)), window, cx);
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 9.);

    window.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.step_thumb(false, state.accessibility_step(), window, cx);
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 10.);
    window.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.step_thumb(false, -state.accessibility_step(), window, cx);
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 6.);
}

#[gpui::test]
fn slider_set_value_uses_zero_origin_steps_and_preserves_range_start(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        cx.new(|_| {
            SliderState::new()
                .min(-10.)
                .max(10.)
                .step(6.)
                .default_value((-10., 0.))
        })
    });
    let (view, window) = cx.add_window_view({
        let state = state.clone();
        move |_, _| SliderHost { state }
    });
    window.update(|window, cx| {
        let _ = view.read(cx);
        window.draw(cx).clear(cx);
    });
    window.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.set_accessibility_value(Some(&ActionData::NumericValue(8.8)), window, cx);
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().start()), -10.);
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 6.);
}

#[gpui::test]
fn slider_pointer_snapping_clamps_to_configured_maximum(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        cx.new(|_| {
            SliderState::new()
                .min(2.)
                .max(10.)
                .step(6.)
                .default_value(2.)
        })
    });
    let (view, window) = cx.add_window_view({
        let state = state.clone();
        move |_, _| SliderHost { state }
    });
    window.update(|window, cx| {
        let _ = view.read(cx);
        window.draw(cx).clear(cx);
    });
    window.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.set_bounds(gpui::Bounds::new(
                gpui::point(px(0.), px(0.)),
                gpui::size(px(100.), px(10.)),
            ));
            state.update_value_by_position(
                gpui::Axis::Horizontal,
                gpui::point(px(100.), px(0.)),
                false,
                window,
                cx,
            );
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 10.);
    assert_eq!(window.update(|_, cx| state.read(cx).percentage().end), 1.);
}
