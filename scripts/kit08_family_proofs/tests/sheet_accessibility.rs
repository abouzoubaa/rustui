use gpui::{AppContext as _, Context, IntoElement, Render, TestAppContext, Window, div};
use gpui_component::{Root, WindowExt as _};

struct Host;
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[gpui::test]
fn sheet_close_accessibility_label_is_exposed_in_the_test_window_tree(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);
    let window = cx
        .add_window(|window, cx| {
            let content = cx.new(|_| Host);
            Root::new(content, window, cx)
        })
        .into();
    cx.update_window(window, |_, window, cx| {
        window.open_sheet(cx, |sheet, _, _| {
            sheet.title("Details").close_accessibility_label("Dismiss details")
        });
        window.draw(cx).clear(cx);
    })
    .unwrap();
    let tree = cx
        .update_window(window, |_, window, _| window.debug_a11y_tree_json().unwrap())
        .unwrap();
    assert!(tree.contains("Dismiss details"), "accessibility tree: {tree}");
}
