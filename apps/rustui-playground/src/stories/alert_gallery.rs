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
        let variant_message = match self.variant() {
            AlertVariant::Default => "lab-alert-variant-default",
            AlertVariant::Info => "lab-alert-variant-info",
            AlertVariant::Success => "lab-alert-variant-success",
            AlertVariant::Warning => "lab-alert-variant-warning",
            AlertVariant::Error => "lab-alert-variant-error",
        };
        let visibility_message = if self.visible() {
            "lab-alert-visible"
        } else {
            "lab-alert-hidden"
        };
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-alert-heading"))
            .child(super::LocaleSelector::new())
            .child(
                Alert(
                    "alert-gallery-saved",
                    "Your changes were saved.",
                    self.visible_binding(),
                )
                .localized_message("lab-alert-message")
                .localized_title("lab-alert-title")
                .variant(self.variant()),
            )
            .child(
                HStack::new()
                    .spacing(4.0)
                    .child(LocalizedText(visibility_message))
                    .child(LocalizedText(variant_message)),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Hide alert", self.action(Self::hide_alert))
                            .id("alert-gallery-hide")
                            .localized_label("lab-alert-hide"),
                    )
                    .child(
                        Button("Show alert", self.action(Self::show_alert))
                            .id("alert-gallery-show")
                            .localized_label("lab-alert-show"),
                    ),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button("Default", self.action(Self::show_default))
                            .id("alert-gallery-default")
                            .localized_label("lab-alert-default"),
                    )
                    .child(
                        Button("Info", self.action(Self::show_info))
                            .id("alert-gallery-info")
                            .localized_label("lab-alert-info"),
                    )
                    .child(
                        Button("Success", self.action(Self::show_success))
                            .id("alert-gallery-success")
                            .localized_label("lab-alert-success"),
                    )
                    .child(
                        Button("Warning", self.action(Self::show_warning))
                            .id("alert-gallery-warning")
                            .localized_label("lab-alert-warning"),
                    )
                    .child(
                        Button("Error", self.action(Self::show_error))
                            .id("alert-gallery-error")
                            .localized_label("lab-alert-error"),
                    ),
            )
    }
}
