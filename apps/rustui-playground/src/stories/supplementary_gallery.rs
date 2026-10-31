//! Accessible supplementary components and observable link activation.
use rustui::localization::MessageArguments;
use rustui::prelude::*;

fn fixture_url(name: &str) -> String {
    #[cfg(target_family = "wasm")]
    {
        format!("./assets/{name}")
    }
    #[cfg(not(target_family = "wasm"))]
    {
        crate::file_url_for_resource(&format!("assets/{name}"))
    }
}

pub fn demo() -> impl View {
    Component("supplementary-gallery", SupplementaryGallery::default)
}

#[component]
struct SupplementaryGallery {
    #[state(0)]
    inbox_activations: i32,
    #[state(0)]
    pointer_taps: i32,
    #[state(0)]
    link_activations: i32,
}

impl SupplementaryGallery {
    fn increment_inbox(&mut self) {
        self.inbox_activations += 1;
    }
    fn tap_cursor_target(&mut self) {
        self.pointer_taps += 1;
    }
    fn activate_link(&mut self) {
        self.link_activations += 1;
    }
}

impl View for SupplementaryGallery {
    fn body(&self) -> impl View {
        HStack::new().child(
            Container(
                VStack::new()
                    .child(LocalizedText("lab-supplementary-title"))
                    .child(Avatar("Ada Lovelace").size(32.0).id("gallery-ada-avatar"))
                    .child(
                        Badge(
                            Button("Inbox", self.action(Self::increment_inbox))
                                .id("gallery-inbox")
                                .localized_label("lab-supplementary-inbox")
                                .accessibility_label(format!(
                                    "Inbox, activated {} times",
                                    self.inbox_activations()
                                )),
                        )
                        .count(7)
                        .id("gallery-inbox-badge")
                        .accessibility_label("Inbox, 7 unread")
                        .localized_accessibility_label("lab-supplementary-inbox-unread"),
                    )
                    .child(
                        LocalizedText("lab-supplementary-inbox-activations").with_arguments(
                            MessageArguments::new().with("count", self.inbox_activations()),
                        ),
                    )
                    .child(LocalizedText("lab-supplementary-cursor-help"))
                    .child(PointerCursor(TapRegion(
                        "gallery-pointer-cursor",
                        Container(
                            VStack::new()
                                .child(LocalizedText("lab-supplementary-cursor-target"))
                                .child(LocalizedText("lab-supplementary-cursor-child")),
                        )
                        .width(260.0)
                        .padding(8.0)
                        .border(),
                        self.action(Self::tap_cursor_target),
                    )))
                    .child(
                        LocalizedText("lab-supplementary-pointer-taps").with_arguments(
                            MessageArguments::new().with("count", self.pointer_taps()),
                        ),
                    )
                    .child(
                        Button(
                            "Keyboard alternative: tap cursor target",
                            self.action(Self::tap_cursor_target),
                        )
                        .id("gallery-pointer-cursor-keyboard")
                        .localized_label("lab-supplementary-cursor-keyboard"),
                    )
                    .child(
                        Link("Open link fixture", fixture_url("link-fixture.html"))
                            .id("gallery-enabled-link")
                            .localized_label("lab-supplementary-open-link")
                            .accessibility_label(format!(
                                "Open link fixture, activated {} times",
                                self.link_activations()
                            ))
                            .localized_accessibility_label("rustui-link-open-fixture")
                            .on_activate(self.action(Self::activate_link)),
                    )
                    .child(
                        Link(
                            "Disabled link fixture",
                            fixture_url("link-disabled-fixture.html"),
                        )
                        .id("gallery-disabled-link")
                        .localized_label("lab-supplementary-disabled-link")
                        .disabled(true)
                        .accessibility_label("Disabled link fixture")
                        .localized_accessibility_label("lab-supplementary-disabled-link")
                        .on_activate(self.action(Self::activate_link)),
                    )
                    .child(
                        LocalizedText("lab-supplementary-link-activations").with_arguments(
                            MessageArguments::new().with("count", self.link_activations()),
                        ),
                    )
                    .spacing(8.0),
            )
            .width(300.0)
            .padding(8.0)
            .border(),
        )
    }
}
