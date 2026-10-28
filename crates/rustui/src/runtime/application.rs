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

#[cfg(target_family = "wasm")]
fn web_application() -> gpui::Application {
    // Keep loaded fonts preferred, then use GPUI's bounded browser fallback for
    // eligible horizontal CJK glyphs absent from the embedded font database.
    let platform = std::rc::Rc::new(gpui_web::WebPlatform::new_with_backend_and_font_fallback(
        true,
        gpui_web::WebBackendPreference::Auto,
        gpui_web::CanvasFontFallback::EmojiAndCjk,
    ));
    let http_client = std::sync::Arc::new(platform.fetch_http_client());
    gpui::Application::with_platform(platform).with_http_client(http_client)
}

#[cfg(any(test, target_family = "wasm"))]
fn web_font_assets() -> Vec<std::borrow::Cow<'static, [u8]>> {
    vec![
        std::borrow::Cow::Borrowed(
            include_bytes!("../../assets/fonts/InterVariable.ttf").as_slice(),
        ),
        std::borrow::Cow::Borrowed(
            include_bytes!("../../assets/fonts/NotoSans-Regular.ttf").as_slice(),
        ),
        std::borrow::Cow::Borrowed(
            include_bytes!("../../assets/fonts/NotoSansArabic.ttf").as_slice(),
        ),
        std::borrow::Cow::Borrowed(
            include_bytes!("../../assets/fonts/NotoSansHebrew.ttf").as_slice(),
        ),
        std::borrow::Cow::Borrowed(
            include_bytes!("../../assets/fonts/FiraMono-Medium.ttf").as_slice(),
        ),
    ]
}

struct EmbeddedAssetSource<S> {
    source: S,
    path: SharedString,
    bytes: &'static [u8],
}

// The pinned package's default-icons.txt selects these component assets.
// Use its public embedding macro on both targets so Web needs no CDN fetch.
// Original SVG sources and license metadata remain in the pinned vendor package.
gpui_kit_assets::icon_assets!(
    ComponentAssets,
    [
        ALargeSmall,
        ArrowDown,
        ArrowLeft,
        ArrowRight,
        ArrowUp,
        Asterisk,
        Ban,
        BatteryCharging,
        BatteryFull,
        BatteryLow,
        BatteryMedium,
        BatteryWarning,
        Battery,
        Bell,
        BookOpen,
        Bot,
        Building2,
        Calendar,
        CaseSensitive,
        ChartPie,
        Check,
        ChevronDown,
        ChevronLeft,
        ChevronRight,
        ChevronUp,
        ChevronsUpDown,
        CircleAlert,
        CircleCheck,
        CircleUser,
        CircleX,
        Close,
        Copy,
        Cpu,
        Dash,
        Delete,
        EllipsisVertical,
        Ellipsis,
        ExternalLink,
        EyeOff,
        Eye,
        FileText,
        File,
        FolderClosed,
        FolderOpen,
        Folder,
        Frame,
        GalleryVerticalEnd,
        Github,
        Globe,
        HardDrive,
        HeartOff,
        Heart,
        Inbox,
        Info,
        Inspector,
        LayoutDashboard,
        LoaderCircle,
        Loader,
        Map,
        Maximize,
        MemoryStick,
        Menu,
        Mic,
        Minimize,
        Minus,
        Moon,
        Network,
        Palette,
        PanelBottomOpen,
        PanelBottom,
        PanelLeftClose,
        PanelLeftOpen,
        PanelLeft,
        PanelRightClose,
        PanelRightOpen,
        PanelRight,
        Pause,
        Play,
        Plus,
        Redo2,
        Redo,
        RefreshCw,
        Replace,
        ResizeCorner,
        RotateCw,
        Search,
        Settings2,
        Settings,
        SortAscending,
        SortDescending,
        SquareTerminal,
        Square,
        StarFill,
        StarOff,
        Star,
        Sun,
        ThumbsDown,
        ThumbsUp,
        TriangleAlert,
        Undo2,
        Undo,
        User,
        WindowClose,
        WindowMaximize,
        WindowMinimize,
        WindowRestore
    ]
);

/// Returns embedded component icons through RustUI's application API.
pub fn component_assets() -> impl gpui::AssetSource {
    ComponentAssets
}

/// Adds one compile-time asset while preserving an application's asset source.
pub fn with_embedded_asset(
    source: impl gpui::AssetSource,
    path: impl Into<SharedString>,
    bytes: &'static [u8],
) -> impl gpui::AssetSource {
    EmbeddedAssetSource {
        source,
        path: path.into(),
        bytes,
    }
}

impl<S: gpui::AssetSource> gpui::AssetSource for EmbeddedAssetSource<S> {
    fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
        if self.path.as_ref() == path {
            Ok(Some(std::borrow::Cow::Borrowed(self.bytes)))
        } else {
            self.source.load(path)
        }
    }

    fn list(&self, path: &str) -> gpui::Result<Vec<SharedString>> {
        let mut assets = self.source.list(path)?;
        let directory = path.trim_end_matches('/');
        let remaining = if directory.is_empty() {
            Some(self.path.as_ref())
        } else {
            self.path
                .as_ref()
                .strip_prefix(directory)
                .and_then(|suffix| suffix.strip_prefix('/'))
        };

        if let Some(name) = remaining
            .and_then(|suffix| suffix.split('/').next())
            .filter(|name| !name.is_empty())
        {
            if !assets.iter().any(|asset| asset.as_ref() == name) {
                assets.push(name.into());
            }
        }
        Ok(assets)
    }
}

gpui::actions!(rustui_runtime, [Quit]);

#[cfg(any(test, target_family = "wasm"))]
fn web_input_key_bindings() -> [KeyBinding; 16] {
    [
        KeyBinding::new("cmd-z", gpui_base::input::Undo, Some("Input")),
        KeyBinding::new("cmd-shift-z", gpui_base::input::Redo, Some("Input")),
        KeyBinding::new("cmd-left", gpui_base::input::MoveHome, Some("Input")),
        KeyBinding::new("cmd-right", gpui_base::input::MoveEnd, Some("Input")),
        KeyBinding::new("cmd-up", gpui_base::input::MoveToStart, Some("Input")),
        KeyBinding::new("cmd-down", gpui_base::input::MoveToEnd, Some("Input")),
        KeyBinding::new(
            "shift-cmd-left",
            gpui_base::input::SelectToStartOfLine,
            Some("Input"),
        ),
        KeyBinding::new(
            "shift-cmd-right",
            gpui_base::input::SelectToEndOfLine,
            Some("Input"),
        ),
        KeyBinding::new(
            "cmd-shift-up",
            gpui_base::input::SelectToStart,
            Some("Input"),
        ),
        KeyBinding::new(
            "cmd-shift-down",
            gpui_base::input::SelectToEnd,
            Some("Input"),
        ),
        KeyBinding::new(
            "alt-left",
            gpui_base::input::MoveToPreviousWord,
            Some("Input"),
        ),
        KeyBinding::new("alt-right", gpui_base::input::MoveToNextWord, Some("Input")),
        KeyBinding::new(
            "cmd-backspace",
            gpui_base::input::DeleteToBeginningOfLine,
            Some("Input"),
        ),
        KeyBinding::new(
            "cmd-delete",
            gpui_base::input::DeleteToEndOfLine,
            Some("Input"),
        ),
        KeyBinding::new(
            "alt-backspace",
            gpui_base::input::DeleteToPreviousWordStart,
            Some("Input"),
        ),
        KeyBinding::new(
            "alt-delete",
            gpui_base::input::DeleteToNextWordEnd,
            Some("Input"),
        ),
    ]
}

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

        #[cfg(target_family = "wasm")]
        gpui_platform::web_init();
        #[cfg(target_family = "wasm")]
        let app = web_application();
        #[cfg(not(target_family = "wasm"))]
        let app = gpui_platform::application();
        let app = match self.asset_setup.take() {
            Some(setup) => setup(app),
            None => app,
        };
        app.run(move |cx| {
            #[cfg(target_family = "wasm")]
            cx.text_system()
                .add_fonts(web_font_assets())
                .expect("bundled Web fonts must register");
            gpui_component::init(cx);
            // The Web target is not macOS at compile time, but browsers on
            // macOS still use Command for editing shortcuts.
            #[cfg(target_family = "wasm")]
            cx.bind_keys(web_input_key_bindings());
            crate::components::overlay::drawer::init(cx);
            // Set the Web default before caller launch/theme customization.
            #[cfg(target_family = "wasm")]
            {
                gpui_component::Theme::global_mut(cx).mono_font_family = "Fira Mono".into();
            }
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
    use gpui::{AssetSource as _, KeyBinding, Keystroke};

    use super::{component_assets, tree_confirm_key_binding, web_font_assets, with_embedded_asset};

    #[test]
    fn component_icons_are_embedded_for_every_pinned_default_path() {
        let source = component_assets();
        let paths = include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../vendor/gpui-kit-assets-0.7.3/default-icons.txt"
        ));
        for path in paths.lines().filter(|line| !line.is_empty()) {
            let bytes = source.load(path).unwrap().expect("default component icon");
            assert!(
                std::str::from_utf8(&bytes).unwrap().contains("<svg"),
                "{path}"
            );
        }
        assert!(source.load("icons/not-a-component.svg").unwrap().is_none());
    }

    struct TestAssetSource;

    impl gpui::AssetSource for TestAssetSource {
        fn load(&self, path: &str) -> gpui::Result<Option<std::borrow::Cow<'static, [u8]>>> {
            Ok((path == "assets/base.png")
                .then_some(std::borrow::Cow::Borrowed(b"base asset".as_slice())))
        }

        fn list(&self, path: &str) -> gpui::Result<Vec<gpui::SharedString>> {
            Ok(if path == "assets" {
                vec!["base.png".into()]
            } else {
                Vec::new()
            })
        }
    }

    #[test]
    fn web_font_registration_includes_variable_weights_and_hebrew() {
        let fonts = web_font_assets();

        assert_eq!(fonts.len(), 5);
        assert!(fonts.iter().all(|font| font.starts_with(b"\0\x01\0\0")));
        assert!(fonts.iter().any(|font| {
            font.as_ref() == include_bytes!("../../assets/fonts/InterVariable.ttf")
        }));
        assert!(fonts.iter().any(|font| {
            font.as_ref() == include_bytes!("../../assets/fonts/NotoSans-Regular.ttf")
        }));
        assert!(fonts.iter().any(|font| {
            font.as_ref() == include_bytes!("../../assets/fonts/NotoSansArabic.ttf")
        }));
        assert!(fonts.iter().any(|font| {
            font.as_ref() == include_bytes!("../../assets/fonts/NotoSansHebrew.ttf")
        }));
        assert!(fonts.iter().any(|font| {
            font.as_ref() == include_bytes!("../../assets/fonts/FiraMono-Medium.ttf")
        }));
    }

    #[test]
    fn embedded_asset_resolves_png_bytes_and_path() {
        let png = b"\x89PNG\r\n\x1a\nplayground sample";
        let source = with_embedded_asset(TestAssetSource, "assets/playground-sample.png", png);

        let loaded = source
            .load("assets/playground-sample.png")
            .unwrap()
            .unwrap();
        assert_eq!(&loaded[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(
            source.load("assets/base.png").unwrap().unwrap().as_ref(),
            b"base asset"
        );
        assert_eq!(source.load("missing.png").unwrap(), None);
        assert_eq!(
            source
                .list("assets")
                .unwrap()
                .iter()
                .map(|asset| asset.as_ref())
                .collect::<Vec<_>>(),
            ["base.png", "playground-sample.png"]
        );
    }

    #[test]
    fn web_input_shortcuts_match_expected_actions() {
        use super::web_input_key_bindings;
        use gpui_base::input::{
            DeleteToBeginningOfLine, MoveHome, MoveToPreviousWord, Redo, SelectToEndOfLine, Undo,
        };

        let bindings = web_input_key_bindings();
        let cases: [(&str, &dyn gpui::Action); 6] = [
            ("cmd-z", &Undo),
            ("cmd-shift-z", &Redo),
            ("cmd-left", &MoveHome),
            ("alt-left", &MoveToPreviousWord),
            ("shift-cmd-right", &SelectToEndOfLine),
            ("cmd-backspace", &DeleteToBeginningOfLine),
        ];
        for (key, expected) in cases {
            let strokes = [Keystroke::parse(key).unwrap()];
            let matching: Vec<_> = bindings
                .iter()
                .filter(|binding| binding.match_keystrokes(&strokes) == Some(false))
                .collect();
            assert_eq!(matching.len(), 1, "{key}");
            assert!(matching[0].action().partial_eq(expected), "{key}");
            let predicate = matching[0].predicate().expect("input-scoped shortcut");
            assert!(predicate.eval(&[gpui::KeyContext::parse("Input").unwrap()]));
            assert!(!predicate.eval(&[gpui::KeyContext::parse("Other").unwrap()]));
        }
    }

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
