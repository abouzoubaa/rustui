// PX-4 reference only: a single ASCII text input on GPUI Fast, without RustUI or Kit.
use gpui::{
    AppContext as _, Context, FocusHandle, InteractiveElement as _, IntoElement,
    ParentElement as _, Render, SharedString, Styled as _, Window,
};
use std::time::{Duration, Instant, SystemTime};
struct Input {
    focus: FocusHandle,
    text: String,
}
impl Render for Input {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        gpui::div()
            .size_full()
            .p_8()
            .bg(gpui::rgb(0x202020))
            .text_color(gpui::rgb(0xffffff))
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                match event.keystroke.key.as_str() {
                    "x" => this.text.push('x'),
                    "backspace" => {
                        this.text.pop();
                    }
                    _ => return,
                }
                cx.notify();
                cx.stop_propagation();
            }))
            .child(SharedString::from(format!("{}▏", self.text)))
    }
}
fn epoch() -> f64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
        * 1000.
}
fn main() {
    let seconds = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "60".into())
        .parse::<u64>()
        .unwrap();
    gpui_platform::application().run(move|cx|{
  let options=gpui::WindowOptions{window_bounds:Some(gpui::WindowBounds::centered(gpui::size(gpui::px(1200.),gpui::px(800.)),cx)),..Default::default()};
  let _handle=cx.open_window(options,|window,cx|{let view=cx.new(|cx|Input{focus:cx.focus_handle(),text:String::new()});let focus=view.read(cx).focus.clone();window.focus(&focus,cx);view}).unwrap();
  let handle = cx.windows().first().copied().expect("reference window");
  cx.activate(true);
  cx.spawn(async move|cx|{
   cx.background_executor().timer(Duration::from_secs(2)).await;
   eprintln!("RUSTUI_G0_LOG {{\"event\":\"start\",\"scene\":\"bare\",\"epoch_ms\":{},\"mach\":{},\"duration_s\":{seconds}}}",epoch(),gpui::fast_latency_tick());
   let start=Instant::now();let mut version=0u64;let mut valid=true;
   while start.elapsed()<Duration::from_secs(seconds){
    version+=1;
    valid &= handle.update(cx,|_,window,cx|{
     let stamp=window.fast_measure_input("bare",version,version);
     let handled=window.dispatch_keystroke(gpui::Keystroke::parse(if version%2==1{"x"}else{"backspace"}).unwrap(),cx);
     if handled{window.fast_input_handled(stamp)}
     handled
    }).unwrap_or(false);
    cx.background_executor().timer(Duration::from_millis(100)).await;
   }
   let end=epoch();let end_mach=gpui::fast_latency_tick();cx.background_executor().timer(Duration::from_millis(250)).await;
   eprintln!("RUSTUI_G0_LOG {{\"event\":\"end\",\"scene\":\"bare\",\"epoch_ms\":{end},\"mach\":{end_mach},\"ticks\":{version},\"valid\":{valid}}}");
   cx.update(|cx|cx.quit());
  }).detach();
 });
}
