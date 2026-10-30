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
