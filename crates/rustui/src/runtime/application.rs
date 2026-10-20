//! Thin RustUI application bootstrap over GPUI's native application lifecycle.

use gpui::{
    App, KeyBinding, NoAction, SharedString, WindowBackgroundAppearance, WindowBounds,
    WindowOptions, px, size,
};
#[cfg(target_os = "macos")]
use gpui::{Menu, MenuItem};
use rustui_core::EnvironmentValues;

use super::window::open_root_window;
use crate::view::View;

type LaunchHook = Box<dyn FnOnce(&mut App)>;
type AssetSetup = Box<dyn FnOnce(gpui::Application) -> gpui::Application>;

gpui::actions!(rustui_runtime, [Quit]);

fn tree_confirm_key_binding() -> KeyBinding {
    KeyBinding::new(
        "enter",
        gpui_base::actions::Confirm { secondary: false },
        Some("Tree"),
    )
}

/// Configures and runs one root RustUI window.
#[must_use]
pub struct RustUI {
    window_options: WindowOptions,
    window_size: Option<(f32, f32)>,
    on_launch: Option<LaunchHook>,
    asset_setup: Option<AssetSetup>,
    environment: EnvironmentValues,
}

impl RustUI {
    pub fn new() -> Self {
        Self::default()
    }

    /// Replaces the GPUI options used for the root window.
    pub fn window_options(mut self, options: WindowOptions) -> Self {
        self.window_options = options;
        self
    }

    /// Opens the root window at a centered size in logical pixels.
    pub fn window_size(mut self, width: f32, height: f32) -> Self {
        assert!(
            width.is_finite() && width > 0.0 && height.is_finite() && height > 0.0,
            "window dimensions must be finite and positive"
        );
        self.window_size = Some((width, height));
        self
    }

    /// Sets the native root window title with a transparent titlebar.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        let titlebar = self
            .window_options
            .titlebar
            .get_or_insert_with(Default::default);
        titlebar.title = Some(title.into());
        titlebar.appears_transparent = true;
        self
    }

    /// Uses the platform's blurred window background where supported.
    pub fn blurred_window(mut self) -> Self {
        self.window_options.window_background = WindowBackgroundAppearance::Blurred;
        self
    }

    /// Runs application setup after GPUI Components is initialized and before
    /// the root window is created.
    pub fn on_launch(mut self, setup: impl FnOnce(&mut App) + 'static) -> Self {
        self.on_launch = Some(Box::new(setup));
        self
    }

    /// Registers the application's asset source with GPUI before launch.
    ///
    /// Names passed to `Image` and upstream icon paths resolve through this source.
    /// The last call replaces the previous source.
    pub fn assets(mut self, source: impl gpui::AssetSource) -> Self {
        self.asset_setup = Some(Box::new(move |app| app.with_assets(source)));
        self
    }

    /// Adds a typed value to the root component's render environment.
    pub fn environment<T: 'static>(mut self, value: T) -> Self {
        self.environment.set(value);
        self
    }

    /// Starts GPUI with standard Quit defaults before application launch setup,
    /// then mounts the root component and activates the application.
    pub fn run<V: View>(mut self, root: V) {
        let on_launch = self.on_launch.take();
        let mut window_options = self.window_options;
        #[cfg(target_os = "macos")]
        let menu_title = window_options
            .titlebar
            .as_ref()
            .and_then(|titlebar| titlebar.title.clone())
            .unwrap_or_else(|| "RustUI".into());
        let window_size = self.window_size;
        let environment = self.environment;

        let app = gpui_platform::application();
        let app = match self.asset_setup.take() {
            Some(setup) => setup(app),
            None => app,
        };
        app.run(move |cx| {
            gpui_component::init(cx);
            cx.bind_keys([tree_confirm_key_binding()]);
            // Pinned Popover binds Space to Confirm. Suppress that binding only
            // while an Input inside a Popover has focus, leaving text insertion
            // to the input and Button activation and Escape to their own paths.
            cx.bind_keys([KeyBinding::new(
                "space",
                NoAction {},
                Some("Popover > Input"),
            )]);
            cx.on_action(|_: &Quit, cx| cx.quit());
            #[cfg(target_os = "macos")]
            {
                cx.bind_keys([KeyBinding::new("cmd-q", Quit, None)]);
                cx.set_menus([Menu::new(menu_title).items([MenuItem::action("Quit", Quit)])]);
            }
            #[cfg(not(target_os = "macos"))]
            cx.bind_keys([KeyBinding::new("ctrl-q", Quit, None)]);
            if let Some(setup) = on_launch {
                setup(cx);
            }
            if let Some((width, height)) = window_size {
                window_options.window_bounds =
                    Some(WindowBounds::centered(size(px(width), px(height)), cx));
            }
            open_root_window(root, environment, window_options, cx);
            cx.activate(true);
        });
    }
}

impl Default for RustUI {
    fn default() -> Self {
        Self {
            window_options: WindowOptions::default(),
            window_size: None,
            on_launch: None,
            asset_setup: None,
            environment: EnvironmentValues::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use gpui::{KeyBinding, Keystroke};

    use super::tree_confirm_key_binding;

    #[test]
    fn tree_confirm_binding_matches_enter_only() {
        let binding: KeyBinding = tree_confirm_key_binding();
        let enter = [Keystroke::parse("enter").unwrap()];
        let space = [Keystroke::parse("space").unwrap()];

        assert!(
            binding
                .action()
                .partial_eq(&gpui_base::actions::Confirm { secondary: false })
        );
        assert_eq!(binding.match_keystrokes(&enter), Some(false));
        assert_eq!(binding.match_keystrokes(&space), None);
    }
}
