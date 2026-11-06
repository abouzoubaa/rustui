//! Native spring target changes through the public RustUI API.

use rustui::{component, localization::MessageArguments, prelude::*};

pub fn demo() -> impl View {
    Component("spring-gallery", SpringGallery::default)
}

#[component]
struct SpringGallery {
    #[state(true)]
    bright: bool,
    #[state(0)]
    clicks: u32,
    #[state(0)]
    from_generation: u32,
}

impl SpringGallery {
    fn toggle_target(&mut self) {
        self.bright = !self.bright;
    }

    fn count(&mut self) {
        self.clicks += 1;
    }

    fn remount_from_zero(&mut self) {
        self.from_generation += 1;
    }
}

impl View for SpringGallery {
    fn body(&self) -> impl View {
        let target = if self.bright() { 1.0 } else { 0.15 };
        let from_generation = self.from_generation();

        VStack::new()
            .spacing(10.0)
            .child(LocalizedText("lab-spring-title"))
            .child(LocalizedText("lab-spring-description"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Toggle target", self.action(Self::toggle_target))
                            .id("spring-target")
                            .localized_label("lab-spring-toggle"),
                    )
                    .child(
                        Button("Reduce motion", AppAction::reduced_motion(true))
                            .id("spring-reduce-motion")
                            .localized_label("lab-motion-reduce"),
                    )
                    .child(
                        Button("Allow motion", AppAction::reduced_motion(false))
                            .id("spring-allow-motion")
                            .localized_label("lab-motion-allow"),
                    ),
            )
            .child(
                LocalizedText("lab-spring-status").with_arguments(
                    MessageArguments::new()
                        .with("target", target)
                        .with("clicks", i64::from(self.clicks())),
                ),
            )
            .child(
                HStack::new().child(
                    SpringOpacity::new(
                        "spring-sample",
                        Container(
                            Button("Count activity", self.action(Self::count))
                                .id("spring-activity")
                                .localized_label("lab-spring-count"),
                        )
                        .padding(16.0)
                        .border()
                        .background(Surface::Panel),
                        target,
                    )
                    .expect("gallery target is in range")
                    .config(SpringConfig::new(170.0, 8.0, 1.0).expect("gallery physics are valid")),
                ),
            )
            .child(LocalizedText("lab-spring-help"))
            .child(
                LocalizedText("lab-spring-initial-status").with_arguments(
                    MessageArguments::new()
                        .with("generation", i64::from(from_generation))
                        .with("target", target),
                ),
            )
            .child(
                HStack::new().child(
                    Button("Remount from zero", self.action(Self::remount_from_zero))
                        .id("spring-from-remount")
                        .localized_label("lab-spring-remount"),
                ),
            )
            .child(
                HStack::new().child(Keyed(
                    format!("spring-from-generation-{from_generation}"),
                    SpringOpacity::new(
                        "spring-from-sample",
                        Container(LocalizedText("lab-spring-initial-sample"))
                            .padding(16.0)
                            .border()
                            .background(Surface::Panel),
                        target,
                    )
                    .expect("gallery target is in range")
                    .from(0.0)
                    .expect("gallery initial opacity is in range"),
                )),
            )
    }
}
