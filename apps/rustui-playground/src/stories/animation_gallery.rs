//! GPUI-scheduled mount animation, exposed through public RustUI views.

use std::time::Duration;

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("animation-gallery", AnimationGallery::default)
}

#[component]
struct AnimationGallery {
    #[state(0)]
    generation: u32,
}

impl AnimationGallery {
    fn restart(&mut self) {
        self.generation += 1;
    }
}

impl View for AnimationGallery {
    fn body(&self) -> impl View {
        let shadow_color = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.35,
        };

        VStack::new()
            .spacing(10.0)
            .child(Text("Opacity animation"))
            .child(Text("Restart changes the view identity. GPUI owns the three-second fade and reduced-motion setting."))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Button("Restart fade", self.action(Self::restart)).id("animation-restart"))
                    .child(Button("Reduce motion", AppAction::reduced_motion(true)).id("animation-reduce-motion"))
                    .child(Button("Allow motion", AppAction::reduced_motion(false)).id("animation-allow-motion")),
            )
            .child(Text(format!("Animation generation: {}", self.generation())))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Container(Text("Static opacity: 1.0"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel)
                            .opacity(1.0),
                    )
                    .child(
                        Container(Text("Static opacity: 0.5"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel)
                            .opacity(0.5),
                    ),
            )
            .child(Text("Container shadow"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Container(Text("Without shadow"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel),
                    )
                    .child(
                        Container(Text("Offset 0, 4; blur 8; spread -1"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel)
                            .shadow(0.0, 4.0, 8.0, -1.0, shadow_color),
                    ),
            )
            .child(Keyed(
                format!("fade-generation-{}", self.generation()),
                AnimatedOpacity::new(
                    "sample-fade",
                    Container(Text("Animated content reaches full opacity."))
                        .padding(12.0)
                        .border()
                        .background(Surface::Panel),
                    0.0,
                    1.0,
                    Duration::from_secs(3),
                )
                .expect("fixed demo animation parameters are valid"),
            ))
            .child(Text("One-shot mount animation; changing a target on the same identity does not restart it. Reduced motion uses the endpoint immediately."))
    }
}
