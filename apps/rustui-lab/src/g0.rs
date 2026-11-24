//! Scripted normal input in real Metal windows. No forced redraws or extra runtime.
#[path = "../../latency-ref/native_input.rs"]
mod native_input;
use rustui::{
    gpui::{
        self, App, AppContext as _, Context, Entity, InteractiveElement as _, IntoElement as _,
        ParentElement as _, Styled as _, Window,
    },
    gpui_component::ActiveTheme as _,
    prelude::*,
    runtime::component_host::ComponentHost,
    theme::{AppearanceMode, activate_theme, parse_theme},
    view::{EmptyView, RenderContext},
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::{Duration, Instant, SystemTime},
};

mod cpu;

const WIDTH: f32 = 1200.0;
const HEIGHT: f32 = 800.0;
const PAD: f32 = 32.0;
const INITIAL_LOG_LINES: usize = 1000;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Scene {
    #[default]
    Idle,
    Typing,
    Hover,
    Log,
    Drag,
    Acceptance,
}
impl Scene {
    pub fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Typing => "typing",
            Self::Hover => "hover",
            Self::Log => "log",
            Self::Drag => "drag",
            Self::Acceptance => "acceptance",
        }
    }
    fn parse(value: &str) -> Option<Self> {
        [
            Self::Idle,
            Self::Typing,
            Self::Hover,
            Self::Log,
            Self::Drag,
            Self::Acceptance,
        ]
        .into_iter()
        .find(|scene| scene.name() == value)
    }
    fn hz(self) -> u32 {
        match self {
            Self::Idle => 0,
            Self::Typing => std::env::var("RUSTUI_G0_TYPING_HZ")
                .ok()
                .and_then(|s| s.parse().ok())
                .filter(|hz| [10, 30].contains(hz))
                .unwrap_or(10),
            Self::Log => 10,
            Self::Hover | Self::Drag | Self::Acceptance => 120,
        }
    }
}

fn epoch_ms() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        * 1000.0
}
fn due_ticks(elapsed: Duration, hz: u32) -> usize {
    (elapsed.as_nanos() * u128::from(hz) / 1_000_000_000) as usize
}
fn editor_source() -> String {
    (0..10_000)
        .map(|line| match line % 20 {
            0 => format!("// G0 source block {}", line / 20),
            1 => format!("fn block_{}() {{", line / 20),
            19 => "}".to_owned(),
            _ => format!("    let value_{line} = {line};"),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[component]
struct Benchmark {
    scene: Scene,
    #[state(String::from("G0 idle caret"))]
    caret: String,
    #[state(String::new())]
    source: String,
    #[state(None)]
    selected: Option<usize>,
    rows: Rc<[gpui::SharedString]>,
    native: Option<Entity<NativeSurface>>,
}
impl View for Benchmark {
    fn body(&self) -> impl View {
        let content = match self.scene {
            Scene::Idle => AnyView::new(TextField("G0 blinking caret", self.caret_binding())),
            Scene::Typing => AnyView::new(
                CodeEditor("g0-ten-thousand-lines", self.source_binding())
                    .language("rust")
                    .height(700.0),
            ),
            Scene::Hover => AnyView::new(
                List(self.rows.iter().cloned(), self.selected_binding())
                    .height(700.0)
                    .row_height(30.0),
            ),
            Scene::Acceptance => AnyView::new(rustui_playground::g5_acceptance_view()),
            Scene::Log | Scene::Drag => {
                AnyView::new(EntityView(self.native.as_ref().unwrap().clone()))
            }
        };
        Container(content).width(WIDTH).height(HEIGHT).padding(PAD)
    }
}

struct EntityView<T: gpui::Render>(Entity<T>);
impl<T: gpui::Render> View for EntityView<T> {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, _: &mut RenderContext<'_>) -> gpui::AnyElement {
        self.0.clone().into_any_element()
    }
}
type Owner = Entity<ComponentHost<Benchmark>>;
struct RootBridge(Rc<RefCell<Option<Owner>>>);
impl View for RootBridge {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, _: &mut RenderContext<'_>) -> gpui::AnyElement {
        self.0.borrow().as_ref().unwrap().clone().into_any_element()
    }
}

struct NativeSurface {
    scene: Scene,
    lines: Vec<gpui::SharedString>,
    scroll: gpui::UniformListScrollHandle,
    card: gpui::Point<gpui::Pixels>,
    grab: gpui::Point<gpui::Pixels>,
    dragging: bool,
    moves: usize,
}
impl NativeSurface {
    fn new(scene: Scene) -> Self {
        Self {
            scene,
            lines: if scene == Scene::Log {
                (0..INITIAL_LOG_LINES)
                    .map(|i| format!("{i:06}  G0 log record").into())
                    .collect()
            } else {
                Vec::new()
            },
            scroll: gpui::UniformListScrollHandle::new(),
            card: gpui::point(gpui::px(80.0), gpui::px(80.0)),
            grab: gpui::point(gpui::px(0.0), gpui::px(0.0)),
            dragging: false,
            moves: 0,
        }
    }
    fn append(&mut self, cx: &mut Context<Self>) {
        self.lines
            .push(format!("{:06}  G0 appended log record", self.lines.len()).into());
        self.scroll
            .scroll_to_item_strict(self.lines.len() - 1, gpui::ScrollStrategy::Bottom);
        cx.notify();
    }
    fn down(&mut self, event: &gpui::MouseDownEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.grab = event.position - self.card;
        self.dragging = true;
        cx.notify();
    }
    fn moved(&mut self, event: &gpui::MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.dragging && event.dragging() {
            self.card = event.position - self.grab;
            self.moves += 1;
            cx.notify();
        }
    }
    fn up(&mut self, _: &gpui::MouseUpEvent, _: &mut Window, cx: &mut Context<Self>) {
        self.dragging = false;
        cx.notify();
    }
}
impl gpui::Render for NativeSurface {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let background = cx.theme().background;
        let foreground = cx.theme().foreground;
        if self.scene == Scene::Log {
            let lines = self.lines.clone();
            return gpui::div()
                .size_full()
                .bg(background)
                .text_color(foreground)
                .child(
                    gpui::uniform_list("g0-log-lines", lines.len(), move |range, _, _| {
                        range
                            .map(|i| gpui::div().id(i).h(gpui::px(24.0)).child(lines[i].clone()))
                            .collect::<Vec<_>>()
                    })
                    .size_full()
                    .track_scroll(&self.scroll),
                )
                .into_any_element();
        }
        gpui::div()
            .id("g0-board")
            .size_full()
            .relative()
            .bg(background)
            .text_color(foreground)
            .on_mouse_move(cx.listener(Self::moved))
            .on_mouse_up(gpui::MouseButton::Left, cx.listener(Self::up))
            .child(
                gpui::div()
                    .absolute()
                    .left(gpui::px(360.0))
                    .top(gpui::px(220.0))
                    .w(gpui::px(200.0))
                    .h(gpui::px(120.0))
                    .bg(gpui::rgb(0x789498))
                    .shadow_sm()
                    .child("Stationary card"),
            )
            .child(
                gpui::div()
                    .id("g0-moving-card")
                    .absolute()
                    .left(self.card.x)
                    .top(self.card.y)
                    .w(gpui::px(200.0))
                    .h(gpui::px(120.0))
                    .bg(gpui::rgb(0x4284d6))
                    .shadow_sm()
                    .child("Drag card")
                    .on_mouse_down(gpui::MouseButton::Left, cx.listener(Self::down)),
            )
            .into_any_element()
    }
}

fn mount(scene: Scene, app: &mut App) -> (Owner, Entity<NativeSurface>) {
    let native = app.new(|_| NativeSurface::new(scene));
    let component = Benchmark {
        scene,
        source: if scene == Scene::Typing {
            editor_source()
        } else {
            String::new()
        },
        rows: if scene == Scene::Hover {
            (0..5000)
                .map(|i| format!("G0 list row {i:05}").into())
                .collect()
        } else {
            Rc::from([])
        },
        native: Some(native.clone()),
        ..Benchmark::default()
    };
    (app.new(|cx| ComponentHost::new(component, cx)), native)
}
fn drag_position(tick: usize) -> gpui::Point<gpui::Pixels> {
    let phase = tick % 480;
    let step = if phase <= 240 { phase } else { 480 - phase } as f32;
    gpui::point(
        gpui::px(PAD + 100.0 + step * 3.0),
        gpui::px(PAD + 100.0 + step * 1.5),
    )
}
fn hover_pixels() -> bool {
    static ENABLED: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(|| std::env::var("RUSTUI_G0_HOVER_PIXELS").as_deref() == Ok("1"))
}
fn step(
    scene: Scene,
    tick: usize,
    native: &Entity<NativeSurface>,
    window: &mut Window,
    app: &mut App,
) -> bool {
    if native_input::enabled() && matches!(scene, Scene::Typing | Scene::Hover | Scene::Drag) {
        let phase = tick % 42;
        let row = if hover_pixels() {
            if tick <= 240 { 2 } else { 8 }
        } else if phase < 22 {
            phase
        } else {
            42 - phase
        };
        let point = if scene == Scene::Drag {
            drag_position(tick)
        } else {
            gpui::point(gpui::px(400.), gpui::px(PAD + 15. + row as f32 * 30.))
        };
        return native_input::post(
            if scene == Scene::Typing {
                0
            } else if scene == Scene::Drag {
                2
            } else {
                1
            },
            tick as u64,
            f32::from(point.x) as f64,
            f32::from(point.y) as f64,
        );
    }
    let stamp = if scene != Scene::Idle {
        // Log appends retain at least the newest 20 rows in this fixed viewport;
        // discrete edits/moves only claim the exact state version actually painted.
        let first = if scene == Scene::Log {
            tick.saturating_sub(19)
        } else {
            tick
        };
        window.fast_measure_input(scene.name(), tick as u64, first as u64)
    } else {
        None
    };
    let handled = match scene {
        Scene::Idle => true,
        Scene::Typing => window.dispatch_keystroke(
            gpui::Keystroke::parse(if tick % 2 == 1 { "x" } else { "backspace" }).unwrap(),
            app,
        ),
        Scene::Log => {
            native.update(app, |state, cx| state.append(cx));
            true
        }
        Scene::Hover => {
            let phase = tick % 42;
            let row = if hover_pixels() {
                if tick <= 240 { 2 } else { 8 }
            } else if phase < 22 {
                phase
            } else {
                42 - phase
            };
            window.dispatch_event(
                gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent {
                    position: gpui::point(
                        gpui::px(400.0),
                        gpui::px(PAD + 15.0 + row as f32 * 30.0),
                    ),
                    ..Default::default()
                }),
                app,
            );
            true
        }
        Scene::Acceptance => {
            #[allow(deprecated)]
            let event = gpui::ScrollWheelEvent {
                position: gpui::point(gpui::px(900.), gpui::px(520.)),
                delta: gpui::ScrollDelta::Pixels(gpui::point(
                    gpui::px(0.),
                    gpui::px(if tick % 600 < 300 { -500. } else { 500. }),
                )),
                touch_phase: if tick == 1 {
                    gpui::TouchPhase::Started
                } else {
                    gpui::TouchPhase::Moved
                },
                ..Default::default()
            };
            window.dispatch_event(gpui::PlatformInput::Scroll(event), app);
            true
        }
        Scene::Drag => {
            window.dispatch_event(
                gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent {
                    position: drag_position(tick),
                    pressed_button: Some(gpui::MouseButton::Left),
                    ..Default::default()
                }),
                app,
            );
            true
        }
    };
    if handled {
        window.fast_input_handled(stamp);
    }
    handled
}

fn playback(
    scene: Scene,
    duration: Duration,
    owner: Owner,
    native: Entity<NativeSurface>,
    complete: Rc<Cell<bool>>,
    app: &mut App,
) {
    app.spawn(async move |cx| {
        cx.background_executor().timer(Duration::from_millis(200)).await;
        let Some(handle) = cx.update(|app| app.windows().first().copied()) else { cx.update(|app| app.quit()); return; };
        if matches!(scene, Scene::Idle | Scene::Typing) {
            let deadline = Instant::now() + Duration::from_secs(10);
            loop {
                let focused = handle.update(cx, |_, window, app| { window.focus_next(app); window.focused(app).is_some() }).unwrap_or(false);
                if focused { break; }
                if Instant::now() >= deadline { eprintln!("G0 failed to focus the input"); cx.update(|app| app.quit()); return; }
                cx.background_executor().timer(Duration::from_millis(20)).await;
            }
            cx.background_executor().timer(Duration::from_millis(100)).await;
            let _ = handle.update(cx, |_, window, app| {
                if scene == Scene::Typing { for _ in 0..2 { window.dispatch_keystroke(gpui::Keystroke::parse("right").unwrap(), app); } }
                window.dispatch_keystroke(gpui::Keystroke::parse("x").unwrap(), app);
            });
        }
        if scene == Scene::Log {
            native.update(cx, |state, cx| {
                state.scroll.scroll_to_item_strict(INITIAL_LOG_LINES - 1, gpui::ScrollStrategy::Bottom);
                cx.notify();
            });
        }
        if native_input::enabled() && std::env::var("RUSTUI_G5_PIXEL_CAPTURE").as_deref()==Ok("1") {
            let wait=Instant::now();
            while native_input::pixels_state()!=1 {
                assert!(native_input::pixels_state()>=0 && wait.elapsed()<Duration::from_secs(10), "native ScreenCaptureKit setup failed");
                cx.background_executor().timer(Duration::from_millis(20)).await;
            }
        }
        cx.background_executor().timer(Duration::from_secs(2)).await;
        if scene == Scene::Drag {
            if native_input::enabled() {
                let p = drag_position(0);
                assert!(native_input::post(3, 0, f32::from(p.x) as f64, f32::from(p.y) as f64));
                cx.background_executor().timer(Duration::from_millis(75)).await;
            } else {
            let _ = handle.update(cx, |_, window, app| { window.dispatch_event(gpui::PlatformInput::MouseDown(gpui::MouseDownEvent { button: gpui::MouseButton::Left, position: drag_position(0), click_count: 1, ..Default::default() }), app); });
            }
        }
        let initial_len = owner.read_with(cx, |host, _| host.component().source.len());
        let initial_native_text = (native_input::enabled() && scene == Scene::Typing)
            .then(|| owner.read_with(cx, |host, _| host.component().source.clone()));
        let input_ready = owner.read_with(cx, |host, _| if scene == Scene::Idle { host.component().caret != "G0 idle caret" } else { true });
        if !input_ready { eprintln!("G0 caret did not receive warm-up input"); cx.update(|app| app.quit()); return; }
        let (scale, width, height) = handle.update(cx, |_, window, _| (window.scale_factor(), f32::from(window.viewport_size().width), f32::from(window.viewport_size().height))).unwrap();
        eprintln!("RUSTUI_G0_LOG {{\"event\":\"start\",\"scene\":\"{}\",\"duration_s\":{},\"epoch_ms\":{:.6},\"mach\":{},\"input_hz\":{},\"scale\":{},\"width\":{},\"height\":{}}}", scene.name(), duration.as_secs(), epoch_ms(), gpui::fast_latency_tick(), scene.hz(), scale, width, height);
        let cpu_start = match cpu::Snapshot::take() {
            Ok(snapshot) => snapshot,
            Err(error) => { eprintln!("G0 getrusage start failed: {error}"); cx.update(|app| app.quit()); return; }
        };
        let start = Instant::now();
        let manual = scene == Scene::Typing && native_input::physical();
        if manual { native_input::physical_count(1); }
        let mut ticks = 0;
        let mut handled = true;
        let mut min_len = initial_len;
        let mut max_len = initial_len;
        loop {
            let elapsed = start.elapsed().min(duration);
            let mut due = if manual {
                ticks = native_input::physical_count(-1) as usize;
                ticks
            } else {
                due_ticks(elapsed, scene.hz())
            };
            // Replaying a backlog of pointer events into AppKit makes its
            // queue length part of the workload. Sample motion once per wake.
            if native_input::enabled() && matches!(scene, Scene::Hover | Scene::Drag) {
                due = due.min(ticks + 1);
            }
            while ticks < due {
                ticks += 1;
                handled &= handle.update(cx, |_, window, app| step(scene, ticks, &native, window, app)).unwrap_or(false);
            }
            let length = owner.read_with(cx, |host, _| host.component().source.len());
            min_len = min_len.min(length); max_len = max_len.max(length);
            if elapsed >= duration { break; }
            cx.background_executor().timer(if scene == Scene::Idle { duration } else { Duration::from_nanos(1_000_000_000 / u64::from(scene.hz())) }).await;
        }
        if manual { ticks = native_input::physical_count(0) as usize; }
        let cpu_end = match cpu::Snapshot::take() {
            Ok(snapshot) => snapshot,
            Err(error) => { eprintln!("G0 getrusage end failed: {error}"); cx.update(|app| app.quit()); return; }
        };
        let end_epoch = epoch_ms();
        let end_mach = gpui::fast_latency_tick();
        let (cpu_user_s, cpu_sys_s) = cpu_start.elapsed(cpu_end);
        if scene == Scene::Drag && native_input::enabled() {
            let p = drag_position(ticks);
            assert!(native_input::post(4, 0, f32::from(p.x) as f64, f32::from(p.y) as f64));
        } else if scene == Scene::Drag { let _ = handle.update(cx, |_, window, app| { window.dispatch_event(gpui::PlatformInput::MouseUp(gpui::MouseUpEvent { button: gpui::MouseButton::Left, position: drag_position(ticks), click_count: 1, ..Default::default() }), app); }); }
        cx.background_executor().timer(Duration::from_millis(250)).await;
        let (added, moves, dropped) = native.read_with(cx, |state, _| (state.lines.len().saturating_sub(INITIAL_LOG_LINES), state.moves, !state.dragging));
        let valid = handled && match scene { Scene::Typing => max_len > min_len && initial_native_text.as_ref().is_none_or(|initial| {
            ticks % 2 != 0 || owner.read_with(cx, |host, _| &host.component().source == initial)
        }), Scene::Log => added == ticks, Scene::Drag => dropped && if native_input::enabled() { moves > 0 } else { moves == ticks }, _ => true };
        eprintln!("RUSTUI_G0_LOG {{\"event\":\"end\",\"scene\":\"{}\",\"epoch_ms\":{:.6},\"mach\":{},\"ticks\":{},\"log_lines_added\":{},\"drag_moves\":{},\"valid\":{},\"cpu_user_s\":{:.9},\"cpu_sys_s\":{:.9}}}", scene.name(), end_epoch, end_mach, ticks, added, moves, valid, cpu_user_s, cpu_sys_s);
        if native_input::enabled() && std::env::var("RUSTUI_G5_PIXEL_CAPTURE").as_deref()==Ok("1") {
            native_input::pixels_stop();let wait=Instant::now();
            while native_input::pixels_state()!=2 {assert!(wait.elapsed()<Duration::from_secs(10),"native capture did not stop");cx.background_executor().timer(Duration::from_millis(20)).await;}
        }
        complete.set(valid);
        cx.update(|app| app.quit());
    }).detach();
}

pub fn run_cli(args: &[String]) -> Result<(), String> {
    let scene = args
        .get(1)
        .and_then(|value| Scene::parse(value))
        .ok_or("expected --g0-scene idle|typing|hover|log|drag|acceptance")?;
    let seconds = if args.len() == 2 {
        60
    } else if args.len() == 4 && args[2] == "--duration" {
        args[3]
            .parse::<u64>()
            .map_err(|_| "duration must be an integer")?
    } else {
        return Err("expected [--duration seconds]".into());
    };
    if seconds == 0 || seconds > 3600 {
        return Err("duration must be 1..=3600 seconds".into());
    }
    let bridge = Rc::new(RefCell::new(None));
    let slot = bridge.clone();
    let complete = Rc::new(Cell::new(false));
    let result = complete.clone();
    let theme = parse_theme(include_str!("../../../themes/macos27.json"))
        .map_err(|error| error.to_string())?;
    RustUI::new()
        .assets(rustui::component_assets())
        .window_size(WIDTH, HEIGHT)
        .title(format!("RustUI G0 — {}", scene.name()))
        .on_launch(move |app| {
            activate_theme(&theme, Some(AppearanceMode::Dark), app).expect("bundled theme");
            let (owner, native) = mount(scene, app);
            *slot.borrow_mut() = Some(owner.clone());
            playback(
                scene,
                Duration::from_secs(seconds),
                owner,
                native,
                result,
                app,
            );
        })
        .run(RootBridge(bridge));
    if complete.get() {
        Ok(())
    } else {
        Err("G0 scene did not complete its workload".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn workloads_have_exact_document_and_append_rates() {
        assert_eq!(editor_source().lines().count(), 10_000);
        assert_eq!(due_ticks(Duration::from_secs(60), Scene::Log.hz()), 600);
        assert_eq!(due_ticks(Duration::from_millis(99), 10), 0);
        assert_eq!(due_ticks(Duration::from_millis(100), 10), 1);
        assert_eq!(Scene::parse("unknown"), None);
        assert_eq!(Scene::parse("acceptance"), Some(Scene::Acceptance));
        assert_eq!(Scene::Acceptance.hz(), 120);
    }

    #[gpui::test]
    fn normal_input_edits_fields_and_moves_then_drops_the_card(cx: &mut gpui::TestAppContext) {
        cx.update(|app| {
            rustui::gpui_component::init(app);
            let theme = parse_theme(include_str!("../../../themes/macos27.json")).unwrap();
            activate_theme(&theme, Some(AppearanceMode::Dark), app).unwrap();
        });
        let (owner, _) = cx.update(|app| mount(Scene::Idle, app));
        let inner = owner.clone();
        let handle = cx.open_window(
            gpui::size(gpui::px(WIDTH), gpui::px(HEIGHT)),
            |window, cx| rustui::gpui_component::Root::new(inner, window, cx),
        );
        let mut window = gpui::VisualTestContext::from_window(handle.into(), cx);
        window.update(|window, app| {
            window.draw(app).clear(app);
            window.focus_next(app);
            window.dispatch_keystroke(gpui::Keystroke::parse("x").unwrap(), app);
        });
        window.run_until_parked();
        window.update(|_, app| assert_ne!(owner.read(app).component().caret, "G0 idle caret"));

        let (owner, native) = cx.update(|app| mount(Scene::Typing, app));
        let inner = owner.clone();
        let handle = cx.open_window(
            gpui::size(gpui::px(WIDTH), gpui::px(HEIGHT)),
            |window, cx| rustui::gpui_component::Root::new(inner, window, cx),
        );
        let mut window = gpui::VisualTestContext::from_window(handle.into(), cx);
        window.update(|window, app| {
            window.draw(app).clear(app);
            window.focus_next(app);
        });
        window.run_until_parked();
        window.update(|window, app| window.draw(app).clear(app));
        let before = window.update(|_, app| owner.read(app).component().source.clone());
        window.update(|window, app| assert!(step(Scene::Typing, 1, &native, window, app)));
        window.run_until_parked();
        window.update(|_, app| assert_ne!(owner.read(app).component().source, before));
        window.update(|window, app| assert!(step(Scene::Typing, 2, &native, window, app)));
        window.run_until_parked();
        window.update(|_, app| assert_eq!(owner.read(app).component().source, before));

        let (owner, native) = cx.update(|app| mount(Scene::Drag, app));
        let handle = cx.open_window(
            gpui::size(gpui::px(WIDTH), gpui::px(HEIGHT)),
            |window, cx| rustui::gpui_component::Root::new(owner, window, cx),
        );
        let mut window = gpui::VisualTestContext::from_window(handle.into(), cx);
        window.update(|window, app| {
            window.draw(app).clear(app);
            window.dispatch_event(
                gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                    button: gpui::MouseButton::Left,
                    position: drag_position(0),
                    click_count: 1,
                    ..Default::default()
                }),
                app,
            );
            assert!(native.read(app).dragging);
            step(Scene::Drag, 100, &native, window, app);
            assert_eq!(native.read(app).moves, 1);
            window.dispatch_event(
                gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                    button: gpui::MouseButton::Left,
                    position: drag_position(100),
                    ..Default::default()
                }),
                app,
            );
            assert!(!native.read(app).dragging);
        });
    }
}
