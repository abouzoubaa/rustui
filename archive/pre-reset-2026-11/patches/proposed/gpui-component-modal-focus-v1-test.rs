use gpui::{AppContext as _, IntoElement as _, Render, TestAppContext, div};
use gpui_component::{Root, WindowExt as _};

struct Empty;
impl Render for Empty {
    fn render(&mut self, _: &mut gpui::Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        div().into_any_element()
    }
}

#[gpui::test]
fn recovers_topmost_modal_focus(cx: &mut TestAppContext) {
    cx.update(gpui_component::init);
    let (_, window) = cx.add_window_view(|window, cx| {
        let child = cx.new(|_| Empty);
        Root::new(child, window, cx)
    });
    window.update(|window, app| {
        assert!(!window.focus_active_modal(app));
        window.open_sheet(app, |sheet, _, _| sheet.title("Sheet"));
        let sheet_focus = window.focused(app).unwrap();
        window.open_dialog(app, |dialog, _, _| dialog.title("First dialog"));
        let first_dialog_focus = window.focused(app).unwrap();
        window.open_dialog(app, |dialog, _, _| dialog.title("Second dialog"));
        let second_dialog_focus = window.focused(app).unwrap();
        for expected in [second_dialog_focus, first_dialog_focus, sheet_focus] {
            let _ = window.draw(app);
            window.blur(app);
            assert!(window.focus_active_modal(app));
            assert_eq!(window.focused(app), Some(expected));
            if window.has_active_dialog(app) { window.close_dialog(app); }
            else { window.close_sheet(app); }
        }
        window.blur(app);
        assert!(!window.focus_active_modal(app));
        assert!(window.focused(app).is_none());
    });
}
