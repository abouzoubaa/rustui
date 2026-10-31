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
    struct LifecycleProbe {
        mounts: std::rc::Rc<std::cell::Cell<usize>>,
        unmounts: std::rc::Rc<std::cell::Cell<usize>>,
    }

    impl crate::view::View for LifecycleProbe {
        fn body(&self) -> impl crate::view::View {
            crate::view::EmptyView
        }

        fn on_mount(&mut self, _: &mut gpui::App) {
            self.mounts.set(self.mounts.get() + 1);
        }

        fn on_unmount(&mut self, _: &mut gpui::App) {
            self.unmounts.set(self.unmounts.get() + 1);
        }
    }

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

    #[gpui::test]
    fn closing_root_window_unmounts_the_owner(cx: &mut gpui::TestAppContext) {
        let mounts = std::rc::Rc::new(std::cell::Cell::new(0));
        let unmounts = std::rc::Rc::new(std::cell::Cell::new(0));
        let window = cx.update(|app| {
            gpui_component::init(app);
            super::open_root_window(
                LifecycleProbe {
                    mounts: mounts.clone(),
                    unmounts: unmounts.clone(),
                },
                rustui_core::EnvironmentValues::default(),
                gpui::WindowOptions::default(),
                app,
            )
            .unwrap();
            *app.windows().last().unwrap()
        });
        assert_eq!(mounts.get(), 1);
        cx.update(|app| {
            window
                .update(app, |_, window, _| window.remove_window())
                .unwrap();
        });
        cx.run_until_parked();
        assert_eq!(unmounts.get(), 1);
    }
}
