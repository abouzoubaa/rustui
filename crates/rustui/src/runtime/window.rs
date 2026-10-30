//! Root-window creation using GPUI and GPUI Components ownership.

use gpui::{App, AppContext as _, WindowOptions};
use rustui_core::EnvironmentValues;

use super::ComponentHost;
use crate::view::View;

pub(crate) fn open_root_window<V: View>(
    root: V,
    environment: EnvironmentValues,
    options: WindowOptions,
    cx: &mut App,
) -> Result<(), String> {
    cx.open_window(options, |window, cx| {
        let content = cx.new(|cx| ComponentHost::with_environment(root, environment, cx));
        cx.new(|cx| gpui_component::Root::new(content, window, cx))
    })
    .map(|_| ())
    .map_err(|error| format!("{error:#}"))
}

#[cfg(test)]
mod tests {
    #[gpui::test]
    fn opens_a_root_window_in_the_test_platform(cx: &mut gpui::TestAppContext) {
        cx.update(|app| {
            gpui_component::init(app);
            let before = app.windows().len();
            super::open_root_window(
                crate::view::EmptyView,
                rustui_core::EnvironmentValues::default(),
                gpui::WindowOptions::default(),
                app,
            )
            .expect("test platform should open a root window");
            assert_eq!(app.windows().len(), before + 1);
        });
    }
}
