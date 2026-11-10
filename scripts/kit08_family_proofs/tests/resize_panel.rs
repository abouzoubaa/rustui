use std::{cell::Cell, rc::Rc};

use gpui::{
    div, point, px, AppContext as _, Context, IntoElement, Modifiers, MouseButton, Render,
    InteractiveElement, ParentElement, Styled, TestAppContext, Window,
};
use gpui_base::{h_resizable, resizable_panel, ResizableState};

struct ResizeHost {
    state: gpui::Entity<ResizableState>,
    resized: Rc<Cell<usize>>,
}

impl Render for ResizeHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let resized = self.resized.clone();
        div().w(px(400.)).h(px(100.)).child(
            h_resizable("proof-resize")
                .with_state(&self.state)
                .on_resize(move |_, _, _| resized.set(resized.get() + 1))
                .child(
                    resizable_panel()
                        .size(px(150.))
                        .child(div().size_full().debug_selector(|| "first-panel".into())),
                )
                .child(
                    resizable_panel()
                        .size(px(250.))
                        .child(div().size_full().debug_selector(|| "second-panel".into())),
                ),
        )
    }
}

fn mount(
    cx: &mut TestAppContext,
) -> (
    &mut gpui::VisualTestContext,
    gpui::Entity<ResizableState>,
    Rc<Cell<usize>>,
) {
    let state = cx.update(|cx| cx.new(|_| ResizableState::default()));
    let resized = Rc::new(Cell::new(0));
    let (_, cx) = cx.add_window_view({
        let state = state.clone();
        let resized = resized.clone();
        move |_, _| ResizeHost { state, resized }
    });
    cx.update(|window, cx| window.draw(cx).clear(cx));
    cx.update(|window, cx| window.draw(cx).clear(cx));
    (cx, state, resized)
}

#[gpui::test]
fn resize_handle_applies_the_first_pointer_position_immediately(cx: &mut TestAppContext) {
    let (cx, state, _) = mount(cx);
    let boundary = cx.debug_bounds("second-panel").unwrap().left();
    cx.simulate_mouse_down(
        point(boundary - px(2.), px(50.)),
        MouseButton::Left,
        Modifiers::default(),
    );
    cx.simulate_mouse_move(
        point(boundary + px(10.), px(50.)),
        Some(MouseButton::Left),
        Modifiers::default(),
    );
    state.read_with(cx, |state, _| {
        assert_eq!(state.sizes()[0], px(160.));
    });
}

#[gpui::test]
fn resize_drag_finishes_when_left_button_is_no_longer_pressed(cx: &mut TestAppContext) {
    let (cx, state, resized) = mount(cx);
    let boundary = cx.debug_bounds("second-panel").unwrap().left();
    cx.simulate_mouse_down(
        point(boundary - px(2.), px(50.)),
        MouseButton::Left,
        Modifiers::default(),
    );
    cx.simulate_mouse_move(
        point(boundary + px(10.), px(50.)),
        Some(MouseButton::Left),
        Modifiers::default(),
    );
    cx.simulate_mouse_move(
        point(boundary + px(20.), px(50.)),
        None,
        Modifiers::default(),
    );
    assert_eq!(resized.get(), 1);
    cx.simulate_mouse_move(
        point(boundary + px(30.), px(50.)),
        None,
        Modifiers::default(),
    );
    assert_eq!(resized.get(), 1);
    assert_eq!(state.read_with(cx, |state, _| state.sizes()[0]), px(170.));
}
