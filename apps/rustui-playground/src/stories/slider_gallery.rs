//! Focused, controlled Slider examples using the public RustUI API.
use rustui::localization::MessageArguments;
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("slider-gallery", SliderGallery::default)
}

#[component]
struct SliderGallery {
    #[state(35.0_f32)]
    value: f32,
    #[state(false)]
    disabled: bool,
}

impl SliderGallery {
    fn reset(&mut self) {
        self.value = 35.0;
    }

    fn toggle_disabled(&mut self) {
        self.disabled = !self.disabled;
    }
}

impl View for SliderGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(12.0)
            .child(LocalizedText("lab-slider-gallery-title"))
            .child(LocalizedText("lab-slider-gallery-help"))
            .child(
                LocalizedText("lab-slider-gallery-value").with_arguments(
                    MessageArguments::new().with("value", self.value().to_string()),
                ),
            )
            .child(
                Slider(self.value_binding(), 0.0..=100.0)
                    .id("slider-gallery-horizontal")
                    .step(5.0)
                    .disabled(self.disabled())
                    .localized_accessibility_label("lab-slider-gallery-horizontal"),
            )
            .child(LocalizedText("lab-slider-gallery-reverse"))
            .child(
                Slider(self.value_binding(), 0.0..=100.0)
                    .id("slider-gallery-reverse")
                    .step(5.0)
                    .reverse(true)
                    .disabled(self.disabled())
                    .localized_accessibility_label("lab-slider-gallery-reverse"),
            )
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Reset", self.action(Self::reset))
                            .id("slider-gallery-reset")
                            .localized_label("lab-slider-gallery-reset"),
                    )
                    .child(
                        Button("Toggle disabled", self.action(Self::toggle_disabled))
                            .id("slider-gallery-disabled")
                            .localized_label("lab-slider-gallery-toggle-disabled"),
                    ),
            )
    }
}
