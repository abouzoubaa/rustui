use gpui::{
    Context, IntoElement, Render, Role, TestAppContext, Window, div, px,
    prelude::*,
};

struct Host;
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().size(px(100.)).child(
            div()
                .id("disabled-row")
                .role(Role::TreeItem)
                .aria_disabled(true)
                .child("Disabled row"),
        )
    }
}

#[gpui::test]
fn aria_disabled_sets_disabled_on_the_accessibility_tree_node(cx: &mut TestAppContext) {
    let (view, window) = cx.add_window_view(|_, _| Host);
    window.update(|window, cx| {
        let _ = view.read(cx);
        window.draw(cx).clear(cx);
    });
    let tree = window.update(|window, _| window.debug_a11y_tree_json().unwrap());
    assert!(
        tree.contains("\"disabled\": true"),
        "accessibility tree did not report the disabled row: {tree}"
    );
}
