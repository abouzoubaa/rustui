use gpui::{
    Context, FocusHandle, InteractiveElement, Render, Styled, TestAppContext, Window, div, px,
};
use gpui_component::toolbar::Toolbar;

struct ToolbarHarness {
    items: [FocusHandle; 3],
}

impl Render for ToolbarHarness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
        let [first, second, third] = &self.items;
        Toolbar::new("rtl-toolbar")
            .right_to_left(true)
            .content(div().id("first").size(px(20.)).track_focus(first))
            .content(div().id("second").size(px(20.)).track_focus(second))
            .content(div().id("third").size(px(20.)).track_focus(third))
    }
}

#[gpui::test]
fn rtl_toolbar_arrows_follow_visual_order(cx: &mut TestAppContext) {
    let (state, cx) = cx.add_window_view(|window, cx| {
        let items = [
            cx.focus_handle().tab_stop(true),
            cx.focus_handle().tab_stop(true),
            cx.focus_handle().tab_stop(true),
        ];
        items[0].focus(window, cx);
        ToolbarHarness { items }
    });
    let items = state.read_with(cx, |harness, _| harness.items.clone());
    cx.update(|window, cx| {
        window.draw(cx).clear(cx);
    });

    cx.simulate_keystrokes("left");
    cx.update(|window, _| assert!(items[1].is_focused(window)));
    cx.simulate_keystrokes("left");
    cx.update(|window, _| assert!(items[2].is_focused(window)));
    cx.simulate_keystrokes("right");
    cx.update(|window, _| assert!(items[1].is_focused(window)));
}
