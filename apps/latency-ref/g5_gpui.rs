#[path = "native_input.rs"]
mod native_input;
// Local latency reference only: one ASCII input on GPUI Fast, no RustUI or Kit.
use gpui::{
    AppContext as _, Context, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Window,
};
use std::{
    cell::RefCell,
    rc::Rc,
    time::{Duration, Instant, SystemTime},
};
#[path = "../rustui-lab/src/g0/cpu.rs"]
mod cpu;
struct Input {
    focus: FocusHandle,
    text: String,
    version: u64,
    renders: Rc<RefCell<Vec<(u64, u64)>>>,
    pixels: bool,
    scene: &'static str,
    measuring: bool,
    position: gpui::Point<gpui::Pixels>,
    dragging: bool,
    tear_bar: bool,
    overlay: bool,
    _surface: Option<gpui::WindowCompositionSurface>,
}
impl Render for Input {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.renders
            .borrow_mut()
            .push((self.version, gpui::fast_latency_tick()));
        let marker = if self.pixels && !native_input::enabled() {
            Some(
                gpui::div()
                    .absolute()
                    .left(gpui::px(1000.))
                    .top(gpui::px(10.))
                    .w(gpui::px(104.))
                    .h(gpui::px(32.))
                    .flex()
                    .children((0..26).map(|column| {
                        let color = if column == 0 || column == 25 {
                            0xff0000
                        } else {
                            let bit = if column <= 8 {
                                (0xa5 >> (8 - column)) & 1
                            } else {
                                (self.version >> (24 - column)) & 1
                            };
                            if bit == 1 { 0xffffff } else { 0x000000 }
                        };
                        gpui::div()
                            .w(gpui::px(4.))
                            .h(gpui::px(32.))
                            .bg(gpui::rgb(color))
                    })),
            )
        } else {
            None
        };
        gpui::div()
            .size_full()
            .relative()
            .p_8()
            .bg(gpui::rgb(0x202020))
            .text_color(gpui::rgb(0xffffff))
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, window, cx| {
                if !this.measuring || this.scene != "typing" {
                    return;
                }
                match event.keystroke.key.as_str() {
                    "x" => this.text.push('x'),
                    "backspace" => {
                        this.text.pop();
                    }
                    _ => return,
                }
                this.version += 1;
                gpui::fast_latency_config(
                    window.window_handle().window_id(),
                    "g53_bare_text_applied",
                    [this.text.len() as f64, this.version as f64, 0., 0.],
                );
                cx.notify();
                cx.stop_propagation();
            }))
            .on_mouse_down(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.dragging = true;
                }),
            )
            .on_mouse_up(
                gpui::MouseButton::Left,
                cx.listener(|this, _, _, _| {
                    this.dragging = false;
                }),
            )
            .on_mouse_move(cx.listener(|this, event: &gpui::MouseMoveEvent, _, cx| {
                if !this.measuring
                    || this.scene == "typing"
                    || (this.scene == "drag" && !this.dragging)
                    || (this.tear_bar && this.position == event.position)
                {
                    return;
                }
                this.position = event.position;
                this.version += 1;
                cx.notify();
                cx.stop_propagation();
            }))
            .child(SharedString::from(format!("{}▏", self.text)))
            .children(self.tear_bar.then(|| {
                gpui::div()
                    .absolute()
                    .left(gpui::px(100. + ((self.version * 17) % 900) as f32))
                    .top_0()
                    .h_full()
                    .w(gpui::px(12.))
                    .bg(gpui::rgb(0x00ffff))
            }))
            .children((self.scene == "hover" && !self.tear_bar).then(|| {
                gpui::div()
                    .absolute()
                    .left(gpui::px(100.))
                    .top(self.position.y)
                    .w(gpui::px(500.))
                    .h(gpui::px(24.))
                    .bg(gpui::rgb(0x4284d6))
                    .child("Highlighted row")
            }))
            .children((self.scene == "drag" && !self.overlay).then(|| {
                gpui::div()
                    .absolute()
                    .left(self.position.x)
                    .top(self.position.y)
                    .w(gpui::px(180.))
                    .h(gpui::px(90.))
                    .bg(gpui::rgb(0x4284d6))
                    .child("Dragged card")
            }))
            .children(self.overlay.then(|| {
                gpui::deferred(
                    gpui::div()
                        .absolute()
                        .left(self.position.x)
                        .top(self.position.y)
                        .w(gpui::px(180.))
                        .h(gpui::px(90.))
                        .bg(gpui::rgb(0x4284d6))
                        .child(SharedString::from(format!("Overlay {}", self.version))),
                )
            }))
            .children(marker)
    }
}
fn epoch() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        * 1000.
}
unsafe extern "C" {
    fn g5_pixels_start();
    fn g5_pixels_state() -> i32;
    fn g5_pixels_stop();
    fn g5_refresh_start();
    fn g5_refresh_stop();
}
fn main() {
    let seconds = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "60".into())
        .parse::<u64>()
        .unwrap();
    let scene = match std::env::var("RUSTUI_G5_SCENE").as_deref() {
        Ok("hover") => "hover",
        Ok("drag") => "drag",
        _ => "typing",
    };
    let hz = std::env::var("RUSTUI_G5_INPUT_HZ")
        .ok()
        .map_or(10, |v| v.parse::<u64>().unwrap());
    let hz = if scene == "typing" { hz } else { 120 };
    let sparse = std::env::var("RUSTUI_L1_SPARSE").as_deref() == Ok("1");
    let reported_hz = if sparse { 0.5 } else { hz as f64 };
    assert!([10, 30, 120].contains(&hz));
    let tear_bar = std::env::var("RUSTUI_G5_TEAR_BAR").as_deref() == Ok("1");
    let pixels = std::env::var("RUSTUI_G5_PIXEL_CAPTURE").as_deref() == Ok("1");
    let async_wake = std::env::var("RUSTUI_G5_ASYNC_WAKE").as_deref() == Ok("1");
    let fullscreen = std::env::var("RUSTUI_G5_FULLSCREEN").as_deref() == Ok("1");
    let warmup = std::env::var("RUSTUI_G5_WARMUP_S")
        .ok()
        .map_or(2, |v| v.parse::<u64>().unwrap());
    gpui_platform::application().run(move|cx|{
  let renders=Rc::new(RefCell::new(Vec::new()));let saved=renders.clone();
  let options=gpui::WindowOptions{window_bounds:Some(gpui::WindowBounds::centered(gpui::size(gpui::px(1200.),gpui::px(800.)),cx)),..Default::default()};
  let typed=cx.open_window(options,|window,cx|{let overlay=std::env::var("RUSTUI_G5_OVERLAY").as_deref()==Ok("1");let surface=overlay.then(||window.enable_window_composition().unwrap().create_gpui_surface(None).unwrap());let view=cx.new(|cx|Input{overlay,_surface:surface,focus:cx.focus_handle(),text:String::new(),version:0,renders,pixels,scene,measuring:false,position:gpui::point(gpui::px(100.),gpui::px(150.)),dragging:false,tear_bar});let focus=view.read(cx).focus.clone();window.focus(&focus,cx);if fullscreen{window.toggle_fullscreen();}view}).unwrap();
  let handle=cx.windows().first().copied().unwrap();cx.activate(true);
  cx.spawn(async move|cx|{
   if pixels {
    unsafe{g5_pixels_start();}
    let waiting=Instant::now();
    loop {let state=unsafe{g5_pixels_state()};if state==1{break;}assert!(state>=0 && waiting.elapsed()<Duration::from_secs(10),"ScreenCaptureKit startup failed");cx.background_executor().timer(Duration::from_millis(10)).await;}
   }
   cx.background_executor().timer(Duration::from_secs(warmup)).await;
   unsafe{g5_refresh_start();}
   typed.update(cx,|input,_,cx|{input.measuring=true;cx.notify();}).unwrap();
   if scene=="drag" {handle.update(cx,|_,window,app|{window.dispatch_event(gpui::PlatformInput::MouseDown(gpui::MouseDownEvent{button:gpui::MouseButton::Left,position:gpui::point(gpui::px(100.),gpui::px(150.)),click_count:1,..Default::default()}),app);}).unwrap();}
   let cpu_start=cpu::Snapshot::take().unwrap();let start=Instant::now();
   eprintln!("RUSTUI_G0_LOG {{\"event\":\"start\",\"scene\":\"bare\",\"epoch_ms\":{},\"mach\":{},\"duration_s\":{seconds},\"input_hz\":{reported_hz},\"fullscreen\":{fullscreen}}}",epoch(),gpui::fast_latency_tick());
   let mut version=0u64;let mut valid=true;
   if native_input::physical() {
    native_input::physical_count(1);
    cx.background_executor().timer(Duration::from_secs(seconds)).await;
    version=native_input::physical_count(0);
   } else if async_wake {
    valid &= handle.update(cx,|_,window,app|{let stamp=window.fast_measure_input("bare",1,1);let ok=window.dispatch_keystroke(gpui::Keystroke::parse("x").unwrap(),app);window.fast_input_handled(stamp);ok}).unwrap_or(false);
    cx.background_executor().timer(Duration::from_secs(1)).await;
    version=2;
    typed.update(cx,|input,window,cx|{let stamp=window.fast_measure_input("bare",2,2);input.text.push('y');input.version=2;cx.notify();window.fast_input_handled(stamp);}).unwrap();
    if let Some(wait)=Duration::from_secs(seconds).checked_sub(start.elapsed()){cx.background_executor().timer(wait).await;}
   } else {
   while start.elapsed()<Duration::from_secs(seconds){
    version+=1;
    valid &=handle.update(cx,|_,window,cx|{
     if native_input::enabled() {
       return native_input::post(if scene=="typing"{0}else if scene=="drag"{2}else{1},version,100.+(version%650)as f64,150.+(version%20)as f64*24.);
     }
     let stamp=window.fast_measure_input("bare",version,version);
     let handled=if scene=="typing" {window.dispatch_keystroke(gpui::Keystroke::parse(if version%2==1{"x"}else{"backspace"}).unwrap(),cx)}else {
       window.dispatch_event(gpui::PlatformInput::MouseMove(gpui::MouseMoveEvent{position:gpui::point(gpui::px(100.+(version%650) as f32),gpui::px(150.+(version%20) as f32*24.)),pressed_button:(scene=="drag").then_some(gpui::MouseButton::Left),..Default::default()}),cx);true
     };
     if handled{window.fast_input_handled(stamp)}handled
    }).unwrap_or(false);
    let deadline=if sparse{Duration::from_secs(version*2)}else{Duration::from_secs_f64(version as f64/hz as f64)};
    if let Some(wait)=deadline.checked_sub(start.elapsed()){cx.background_executor().timer(wait).await;}
   }
   }
   let cpu_end=cpu::Snapshot::take().unwrap();let end=epoch();let end_mach=gpui::fast_latency_tick();let (user,sys)=cpu_start.elapsed(cpu_end);
   if scene=="drag" {handle.update(cx,|_,window,app|{window.dispatch_event(gpui::PlatformInput::MouseUp(gpui::MouseUpEvent{button:gpui::MouseButton::Left,position:gpui::point(gpui::px(100.),gpui::px(150.)),click_count:1,..Default::default()}),app);}).unwrap();}
   cx.background_executor().timer(Duration::from_millis(500)).await;
   unsafe{g5_refresh_stop();}
   if pixels {unsafe{g5_pixels_stop();}let waiting=Instant::now();while unsafe{g5_pixels_state()}!=2{assert!(waiting.elapsed()<Duration::from_secs(10));cx.background_executor().timer(Duration::from_millis(10)).await;}}
   let final_state=typed.update(cx,|input,_,_|(input.text.clone(),input.version,input.dragging)).unwrap();
   valid &= (scene!="drag" || !final_state.2) && final_state.1==version && final_state.0==if async_wake{"xy"}else if scene=="typing" && version%2==1{"x"}else{""};
   eprintln!("RUSTUI_G0_LOG {{\"event\":\"end\",\"scene\":\"bare\",\"epoch_ms\":{end},\"mach\":{end_mach},\"ticks\":{version},\"valid\":{valid},\"cpu_user_s\":{user:.9},\"cpu_sys_s\":{sys:.9},\"final_version\":{},\"final_text\":{:?}}}",final_state.1,final_state.0);
   for (v,t) in saved.borrow().iter(){eprintln!("G5_VIEW {{\"event\":\"rendered\",\"version\":{v},\"mach\":{t}}}");}
   cx.update(|cx|cx.quit());
  }).detach();
 });
}
