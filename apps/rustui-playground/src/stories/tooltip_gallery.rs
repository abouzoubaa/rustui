//! Tooltip hover and child-action demo using RustUI's public API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("tooltip-gallery", TooltipGallery::default)
}

#[component]
struct TooltipGallery {
    #[state(0)]
    count: u32,
}

impl TooltipGallery {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for TooltipGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(10.0)
            .child(Text("Tooltips"))
            .child(Text("Hover over the text and button. Click the button to check that its action still works."))
            .child(
                Tooltip("This is help for plain text.", Text("Hover over this text"))
                    .id("tooltip-gallery-text"),
            )
            .child(
                Tooltip(
                    "Increment the counter.",
                    Button("Increment", self.action(Self::increment)).id("tooltip-gallery-button"),
                )
                .id("tooltip-gallery-button-help"),
            )
            .child(Text(format!("Button clicks: {}", self.count())))
            .child(
                Tooltip(
                    "This action is unavailable.",
                    Button("Unavailable", self.action(Self::increment))
                        .id("tooltip-gallery-disabled-button")
                        .disabled(true),
                )
                .id("tooltip-gallery-disabled-help"),
            )
    }
}
