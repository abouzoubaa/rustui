//! Structural semantics examples built through RustUI's public API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("semantics-gallery", SemanticsGallery::default)
}

#[component]
struct SemanticsGallery {
    #[state(0)]
    count: i32,
}

impl SemanticsGallery {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for SemanticsGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(Text("Structural semantics"))
            .child(Text(
                "Groups add names and roles to existing content without changing its layout.",
            ))
            .child(
                Semantics(
                    "semantics-gallery-counter",
                    HStack::new()
                        .child(Text(format!("Count: {}", self.count())))
                        .child(
                            Button("Increment", self.action(Self::increment))
                                .id("semantics-gallery-increment"),
                        )
                        .spacing(8.0),
                )
                .role(SemanticsRole::Group)
                .label("Counter controls")
                .description("A counter and its increment action."),
            )
            .child(
                Semantics(
                    "semantics-gallery-live-status",
                    Text(format!("{} items added", self.count())),
                )
                .role(SemanticsRole::Status)
                .live_region_value(
                    format!("{} items added", self.count()),
                    LivePoliteness::Polite,
                ),
            )
            .child(
                Semantics(
                    "semantics-gallery-localized",
                    Text("Change locale to update this group's accessible name."),
                )
                .role(SemanticsRole::Region)
                .localized_label("rustui-action-close"),
            )
            .child(Semantics(
                "semantics-gallery-passthrough",
                Text("This wrapper has no metadata; the child remains visible."),
            ))
    }
}
