//! GPUI-scheduled mount animation, exposed through public RustUI views.

use std::time::Duration;

use rustui::animation::timing::TimingFunction;
use rustui::localization::MessageArguments;
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
            .child(LocalizedText("lab-animation-title"))
            .child(LocalizedText("lab-animation-description"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Restart fade", self.action(Self::restart))
                            .id("animation-restart")
                            .localized_label("lab-animation-restart"),
                    )
                    .child(
                        Button("Reduce motion", AppAction::reduced_motion(true))
                            .id("animation-reduce-motion")
                            .localized_label("lab-motion-reduce"),
                    )
                    .child(
                        Button("Allow motion", AppAction::reduced_motion(false))
                            .id("animation-allow-motion")
                            .localized_label("lab-motion-allow"),
                    ),
            )
            .child(LocalizedText("lab-animation-generation").with_arguments(
                MessageArguments::new().with("generation", i64::from(self.generation())),
            ))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Container(LocalizedText("lab-animation-static-full"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel)
                            .opacity(1.0),
                    )
                    .child(
                        Container(LocalizedText("lab-animation-static-half"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel)
                            .opacity(0.5),
                    ),
            )
            .child(LocalizedText("lab-animation-shadow"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Container(LocalizedText("lab-animation-no-shadow"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel),
                    )
                    .child(
                        Container(LocalizedText("lab-animation-shadow-values"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel)
                            .shadow(0.0, 4.0, 8.0, -1.0, shadow_color),
                    ),
            )
            .child(
                HStack::new().child(Keyed(
                    format!("fade-generation-{}", self.generation()),
                    AnimatedOpacity::new(
                        "sample-fade",
                        Container(LocalizedText("lab-animation-content"))
                            .padding(12.0)
                            .border()
                            .background(Surface::Panel),
                        0.0,
                        1.0,
                        Duration::from_secs(3),
                    )
                    .expect("fixed demo animation parameters are valid"),
                )),
            )
            .child(LocalizedText("lab-animation-help"))
            .child(Keyed(
                format!("curve-generation-{}", self.generation()),
                HStack::new().spacing(8.0).children(
                    [
                        TimingFunction::Linear,
                        TimingFunction::EaseIn,
                        TimingFunction::EaseOut,
                        TimingFunction::EaseInOut,
                    ]
                    .into_iter()
                    .map(|curve| {
                        AnimatedOpacity::new(
                            format!("curve-{curve:?}"),
                            LocalizedText(match curve {
                                TimingFunction::Linear => "lab-animation-linear",
                                TimingFunction::EaseIn => "lab-animation-ease-in",
                                TimingFunction::EaseOut => "lab-animation-ease-out",
                                TimingFunction::EaseInOut => "lab-animation-ease-in-out",
                            }),
                            0.0,
                            1.0,
                            Duration::from_secs(3),
                        )
                        .expect("fixed demo animation parameters are valid")
                        .timing_function(curve)
                    }),
                ),
            ))
    }
}
