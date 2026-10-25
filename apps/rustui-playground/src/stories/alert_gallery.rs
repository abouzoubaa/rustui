//! Inline alert interaction through the public RustUI API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("alert-gallery", AlertGallery::default)
}

#[component]
struct AlertGallery {
    #[state(true)]
    visible: bool,
    #[state(AlertVariant::Success)]
    variant: AlertVariant,
}

impl AlertGallery {
    fn show_alert(&mut self) {
        self.visible = true;
    }

    fn hide_alert(&mut self) {
        self.visible = false;
    }

    fn show_default(&mut self) {
        self.variant = AlertVariant::Default;
        self.visible = true;
    }

    fn show_info(&mut self) {
        self.variant = AlertVariant::Info;
        self.visible = true;
    }

    fn show_success(&mut self) {
        self.variant = AlertVariant::Success;
        self.visible = true;
    }

    fn show_warning(&mut self) {
        self.variant = AlertVariant::Warning;
        self.visible = true;
    }

    fn show_error(&mut self) {
        self.variant = AlertVariant::Error;
        self.visible = true;
    }
}

impl View for AlertGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(Text("Inline alert").font_weight(TextWeight::Semibold))
            .child(super::LocaleSelector::new())
            .child(
                Alert(
                    "alert-gallery-saved",
                    "Your changes were saved.",
                    self.visible_binding(),
                )
                .title("Saved")
                .variant(self.variant()),
            )
            .child(Text(format!(
                "Visible: {} · variant: {:?}",
                self.visible(),
                self.variant()
            )))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Hide alert", self.action(Self::hide_alert))
                            .id("alert-gallery-hide"),
                    )
                    .child(
                        Button("Show alert", self.action(Self::show_alert))
                            .id("alert-gallery-show"),
                    ),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Default", self.action(Self::show_default))
                            .id("alert-gallery-default"),
                    )
                    .child(Button("Info", self.action(Self::show_info)).id("alert-gallery-info"))
                    .child(
                        Button("Success", self.action(Self::show_success))
                            .id("alert-gallery-success"),
                    )
                    .child(
                        Button("Warning", self.action(Self::show_warning))
                            .id("alert-gallery-warning"),
                    )
                    .child(
                        Button("Error", self.action(Self::show_error)).id("alert-gallery-error"),
                    ),
            )
    }
}
