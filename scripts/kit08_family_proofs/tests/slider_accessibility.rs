use gpui::{
    accesskit::{Action, ActionData},
    div, px, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render,
    Styled as _, TestAppContext, Window,
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
            state.handle_accessibility_action(
                Action::SetValue,
                Some(&ActionData::NumericValue(8.8)),
                window,
                cx,
            );
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 9.);

    window.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.handle_accessibility_action(Action::Increment, None, window, cx);
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 10.);
    window.update(|window, cx| {
        state.update(cx, |state, cx| {
            state.handle_accessibility_action(Action::Decrement, None, window, cx);
        });
    });
    assert_eq!(window.update(|_, cx| state.read(cx).value().end()), 6.);
}

#[gpui::test]
fn slider_pointer_snapping_clamps_to_configured_maximum(cx: &mut TestAppContext) {
    let state = cx.update(|cx| {
        cx.new(|_| {
            SliderState::new()
                .min(0.)
                .max(10.)
                .step(6.)
                .default_value(0.)
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
}
