use gpui::{AppContext as _, Context, IntoElement, Render, TestAppContext, Window, div};
use std::time::Duration;
use gpui_component::{Root, TrackedDialogExt as _, WindowExt as _};

struct Host;
impl Render for Host {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

#[gpui::test]
fn tracked_dialog_can_be_queried_and_closed_by_id(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);
    let window = cx
        .add_window(|window, cx| {
            let content = cx.new(|_| Host);
            Root::new(content, window, cx)
        })
        .into();
    let id = cx
        .update_window(window, |_, window, cx| {
            window.open_dialog_tracked(cx, |dialog, _, _| dialog)
        })
        .unwrap();
    cx.update_window(window, |_, window, cx| {
        assert!(window.has_dialog_id(id, cx));
        assert!(window.close_dialog_by_id(id, cx));
        assert!(window.has_active_dialog(cx) == false);
    })
    .unwrap();
    cx.background_executor.advance_clock(Duration::from_secs(1));
    cx.run_until_parked();
    cx.update_window(window, |_, window, cx| {
        window.draw(cx).clear(cx);
        assert!(!window.has_dialog_id(id, cx));
    })
    .unwrap();
}
