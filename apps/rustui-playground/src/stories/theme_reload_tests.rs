//! R26: dispatch the real Playground watcher with deterministic GPUI timers.
use super::*;
use rustui::{
    gpui::{
        self, AppContext as _, Context, Entity, IntoElement, ParentElement as _, Render, Window,
        div,
    },
    gpui_component::Root,
    runtime::ComponentHost,
    theme::{activate_theme, parse_theme},
    view::EmptyView,
};
use std::{
    cell::Cell,
    fs,
    sync::atomic::{AtomicU64, Ordering},
};

const FIRST: u32 = 0x124578;
const SECOND: u32 = 0xA13567;
const THIRD: u32 = 0x267B41;

struct TempTheme(PathBuf);
impl TempTheme {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let directory = std::env::temp_dir().join(format!(
            "rustui-r26-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::SeqCst)
        ));
        fs::create_dir(&directory).unwrap();
        let file = Self(directory.join("theme.json"));
        file.color(FIRST);
        file
    }
    fn color(&self, color: u32) {
        let mut document: serde_json::Value = serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        document["platform_snapshot"]["appearance"]["light"]["window_background"] =
            format!("#{color:06X}").into();
        fs::write(&self.0, serde_json::to_string(&document).unwrap()).unwrap();
    }
    fn invalid(&self) {
        fs::write(&self.0, "not JSON").unwrap();
    }
}
impl Drop for TempTheme {
    fn drop(&mut self) {
        fs::remove_dir_all(self.0.parent().unwrap()).unwrap();
    }
}

struct Swatch(Rc<Cell<usize>>);
impl View for Swatch {
    fn body(&self) -> impl View {
        self.0.set(self.0.get() + 1);
        // Exercise the ordinary RustUI renderer and its theme invalidation,
        // rather than drawing a test-only color from the asserted global.
        Container(EmptyView)
            .width(41.0)
            .height(29.0)
            .background(Surface::Background)
    }
}
struct Harness {
    watcher: Option<Entity<ComponentHost<FoundationAdditions>>>,
    swatch: Entity<ComponentHost<Swatch>>,
}
impl Render for Harness {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(self.swatch.clone())
            .children(self.watcher.iter().cloned())
    }
}

fn drain(window: &mut gpui::VisualTestContext) {
    // TestAppContext drains runnable work without automatically moving time
    // to the next timer (VisualTestContext's convenience method can do that).
    window.cx.run_until_parked();
}
fn draw(window: &mut gpui::VisualTestContext) {
    drain(window);
    window.update(|window, app| window.draw(app).clear(app));
    drain(window);
}
fn advance(window: &mut gpui::VisualTestContext, duration: Duration) {
    window.cx.executor().advance_clock(duration);
    draw(window);
}
fn active(window: &mut gpui::VisualTestContext) -> (u64, String) {
    window.update(|_, app| {
        let theme = active_theme(app).unwrap();
        assert_eq!(theme.mode, AppearanceMode::Light);
        (
            theme.revision(),
            theme
                .profile
                .colors(Some(theme.mode))
                .window_background
                .clone(),
        )
    })
}
fn assert_painted(window: &mut gpui::VisualTestContext, color: u32) {
    window.update(|window, _| {
        let expected: gpui::Hsla = gpui::rgb(color).into();
        let scale = window.scale_factor();
        assert!(
            window.painted_quads().iter().any(|quad| {
                (quad.bounds.size.width.0 - 41.0 * scale).abs() < 0.01
                    && (quad.bounds.size.height.0 - 29.0 * scale).abs() < 0.01
                    && quad.background.as_solid() == Some(expected)
            }),
            "mounted RustUI swatch must paint #{color:06X}"
        );
    });
}
fn has_error_text(window: &mut gpui::VisualTestContext) -> bool {
    window.update(|window, _| {
        window
            .debug_a11y_tree_json()
            .unwrap()
            .contains("Reload error:")
    })
}

fn click_auto(window: &mut gpui::VisualTestContext, enabled: bool) {
    let prefix = if enabled {
        "Disable automatic theme file reload."
    } else {
        "Enable automatic theme file reload."
    };
    window.update(|window, app| {
        let tree: serde_json::Value =
            serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap();
        let node = tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .find(|node| {
                node["aria"]["label"]
                    .as_str()
                    .is_some_and(|label| label.starts_with(prefix))
            })
            .expect("rendered Playground automatic-reload button");
        let bounds = &node["bounds"];
        let scale = tree["frame"]["scale_factor"].as_f64().unwrap();
        let position = gpui::point(
            gpui::px(
                ((bounds["x"].as_f64().unwrap() + bounds["width"].as_f64().unwrap() / 2.0) / scale)
                    as f32,
            ),
            gpui::px(
                ((bounds["y"].as_f64().unwrap() + bounds["height"].as_f64().unwrap() / 2.0) / scale)
                    as f32,
            ),
        );
        // Dispatch raw events so the convenience mouse helpers cannot advance
        // the executor to future timers while automatic polling is enabled.
        window.dispatch_event(
            gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                position,
                button: gpui::MouseButton::Left,
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
                first_mouse: false,
            }),
            app,
        );
        window.dispatch_event(
            gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                position,
                button: gpui::MouseButton::Left,
                modifiers: gpui::Modifiers::default(),
                click_count: 1,
            }),
            app,
        );
    });
    draw(window);
}

#[derive(Clone, Copy)]
enum Case {
    AutoAndStop,
    InvalidRecoveryAndUnmount,
}
fn run(cx: &mut gpui::TestAppContext, case: Case) {
    let file = TempTheme::new();
    let initial = parse_theme(&fs::read_to_string(&file.0).unwrap()).unwrap();
    cx.update(|app| {
        rustui::gpui_component::init(app);
        activate_theme(&initial, Some(AppearanceMode::Light), app).unwrap();
    });
    let renders = Rc::new(Cell::new(0));
    let saved = Rc::new(RefCell::new(None));
    let (_, window) = cx.add_window_view(|window, app| {
        let swatch = app.new(|cx| ComponentHost::new(Swatch(renders.clone()), cx));
        let watcher = app.new(|cx| {
            let mut watcher = FoundationAdditions::new();
            watcher.path = file.0.clone();
            {
                let mut state = watcher.reload.borrow_mut();
                state.theme_file = Some(ThemeFile::open(&file.0).unwrap());
                state.had_error = false;
            }
            ComponentHost::new(watcher, cx)
        });
        let harness = app.new(|_| Harness {
            watcher: Some(watcher),
            swatch,
        });
        *saved.borrow_mut() = Some(harness.clone());
        Root::new(harness, window, app)
    });
    let harness = saved.borrow_mut().take().unwrap();
    draw(window);
    assert_painted(window, FIRST);
    let (owner, state) = window.update(|_, app| {
        let watcher = harness.read(app).watcher.as_ref().unwrap();
        let component = watcher.read(app).component();
        (watcher.downgrade(), Rc::downgrade(&component.reload))
    });
    let baseline = active(window);
    let before_render = renders.get();
    click_auto(window, false);
    drain(window);
    assert!(state.upgrade().unwrap().borrow().enabled);
    file.color(SECOND);
    // Prove time only moves when requested, including the interval boundary.
    advance(window, RELOAD_INTERVAL - Duration::from_millis(1));
    assert_eq!(active(window), baseline, "poll must not fire early");
    assert_painted(window, FIRST);
    advance(window, Duration::from_millis(2));
    let second = active(window);
    assert_eq!(second, (baseline.0 + 1, format!("#{SECOND:06X}")));
    assert!(
        renders.get() > before_render,
        "mounted swatch must re-render"
    );
    assert_painted(window, SECOND);

    match case {
        Case::AutoAndStop => {
            click_auto(window, true);
            draw(window);
            {
                let state = state.upgrade().unwrap();
                let state = state.borrow();
                assert!(!state.enabled);
                assert!(state.task.is_none(), "stop must drop the polling task");
            }
            file.color(THIRD);
            advance(window, RELOAD_INTERVAL * 4);
            assert_eq!(
                active(window),
                second,
                "stopped watcher cannot activate later edits"
            );
            assert_painted(window, SECOND);
        }
        Case::InvalidRecoveryAndUnmount => {
            file.invalid();
            advance(window, RELOAD_INTERVAL + Duration::from_millis(1));
            assert_eq!(
                active(window),
                second,
                "invalid JSON preserves last good ActiveTheme"
            );
            assert_painted(window, SECOND);
            {
                let state = state.upgrade().unwrap();
                let state = state.borrow();
                assert!(state.had_error);
                assert!(state.result.starts_with("Reload error:"));
                assert_eq!(
                    state.localized_result.as_ref().unwrap().0,
                    "lab-foundation-reload-error"
                );
                assert_eq!(
                    state
                        .theme_file
                        .as_ref()
                        .unwrap()
                        .theme()
                        .platform_snapshot
                        .appearance
                        .light
                        .window_background,
                    format!("#{SECOND:06X}")
                );
            }
            assert!(
                has_error_text(window),
                "reload error must reach rendered controls"
            );
            file.color(THIRD);
            advance(window, RELOAD_INTERVAL + Duration::from_millis(1));
            let recovered = active(window);
            assert_eq!(recovered, (second.0 + 1, format!("#{THIRD:06X}")));
            assert_painted(window, THIRD);
            {
                let state = state.upgrade().unwrap();
                let state = state.borrow();
                assert!(!state.had_error);
                assert!(state.result.starts_with("Applied Light"));
                assert_eq!(
                    state.localized_result.as_ref().unwrap().0,
                    "lab-foundation-applied-light"
                );
            }
            assert!(
                !has_error_text(window),
                "recovery must clear rendered error"
            );
            assert!(state.upgrade().unwrap().borrow().enabled);
            window.update(|_, app| {
                harness.update(app, |harness, cx| {
                    harness.watcher.take();
                    cx.notify();
                })
            });
            draw(window);
            assert!(
                owner.upgrade().is_none(),
                "polling task/action must not retain owner entity"
            );
            assert!(
                state.upgrade().is_none(),
                "unmount must release polling state"
            );
            file.color(FIRST);
            advance(window, RELOAD_INTERVAL * 4);
            assert_eq!(
                active(window),
                recovered,
                "no activation after owner removal"
            );
            assert_painted(window, THIRD);
            assert!(owner.upgrade().is_none());
            assert!(state.upgrade().is_none());
        }
    }
}
#[gpui::test]
fn auto_reload_updates_mounted_swatch_and_stops(cx: &mut gpui::TestAppContext) {
    run(cx, Case::AutoAndStop);
}
#[gpui::test]
fn invalid_reload_recovers_and_unmount_releases_owner(cx: &mut gpui::TestAppContext) {
    run(cx, Case::InvalidRecoveryAndUnmount);
}
