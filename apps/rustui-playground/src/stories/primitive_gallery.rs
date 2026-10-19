//! Basic controls composed through the public RustUI API.

use rustui::prelude::*;
use std::path::PathBuf;

pub fn demo() -> impl View {
    Component("primitive-gallery", PrimitiveGallery::default)
}

#[component]
struct PrimitiveGallery {
    #[state(25.0)]
    progress: f32,
}

impl PrimitiveGallery {
    fn advance(&mut self) {
        self.progress = if self.progress >= 100.0 {
            0.0
        } else {
            self.progress + 25.0
        };
    }
}

impl View for PrimitiveGallery {
    fn body(&self) -> impl View {
        let image =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../assets/playground-sample.png");
        Container(
            VStack::new()
                .child(Text("Basic components"))
                .child(Label("Search").secondary("⌘K").icon(IconName::Search))
                .child(
                    Icon(IconName::Search)
                        .size(24.0)
                        .id("gallery-search-icon")
                        .accessibility_label("Search icon"),
                )
                .child(
                    Image(image)
                        .size(96.0, 64.0)
                        .fit(ImageFit::Contain)
                        .id("gallery-image")
                        .accessibility_label("Purple and cyan sample image"),
                )
                .child(
                    Progress("gallery-progress", self.progress()).accessibility_label(format!(
                        "Gallery progress {} percent",
                        self.progress()
                    )),
                )
                .child(
                    Progress("gallery-progress-loading", 0.0)
                        .indeterminate(true)
                        .accessibility_label("Background operation in progress"),
                )
                .child(
                    Button("Advance progress", self.action(Self::advance))
                        .id("gallery-progress-advance")
                        .accessibility_label(format!(
                            "Advance progress: {} percent",
                            self.progress()
                        )),
                )
                .child(
                    Spinner()
                        .id("gallery-spinner")
                        .accessibility_label("Gallery loading"),
                )
                .spacing(8.0),
        )
        .width(260.0)
        .padding(8.0)
        .border()
    }
}
