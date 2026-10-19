//! Native spring target changes through the public RustUI API.

use rustui::{component, prelude::*};

pub fn demo() -> impl View {
    Component("spring-gallery", SpringGallery::default)
}

#[component]
struct SpringGallery {
    #[state(true)]
    bright: bool,
    #[state(0)]
    clicks: u32,
}

impl SpringGallery {
    fn toggle_target(&mut self) {
        self.bright = !self.bright;
    }
    fn count(&mut self) {
        self.clicks += 1;
    }
}

impl View for SpringGallery {
    fn body(&self) -> impl View {
        let target = if self.bright() { 1.0 } else { 0.15 };
        VStack::new().spacing(10.0)
            .child(Text("State-driven spring opacity"))
            .child(Text("Toggle during motion to retarget the same spring. First mount starts at its target."))
            .child(HStack::new().spacing(8.0)
                .child(Button("Toggle target", self.action(Self::toggle_target)).id("spring-target"))
                .child(Button("Reduce motion", AppAction::reduced_motion(true)).id("spring-reduce-motion"))
                .child(Button("Allow motion", AppAction::reduced_motion(false)).id("spring-allow-motion")))
            .child(Text(format!("Target opacity: {target:.2}; activity clicks: {}", self.clicks())))
            .child(SpringOpacity::new("spring-sample",
                Container(Button("Count activity", self.action(Self::count)).id("spring-activity"))
                    .padding(16.0).border().background(Surface::Panel), target)
                .expect("gallery target is in range")
                .config(SpringConfig::new(170.0, 8.0, 1.0).expect("gallery physics are valid")))
            .child(Text("GPUI owns timing and retained velocity. Spring overshoot is clamped for display; opacity does not disable hit testing."))
    }
}
