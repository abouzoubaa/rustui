//! Basic controls composed through the public RustUI API.

use rustui::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TextStyleExample {
    pub(super) name: &'static str,
    pub(super) size: f32,
    pub(super) weight: TextWeight,
}

pub(super) const TEXT_STYLE_EXAMPLES: [TextStyleExample; 8] = [
    TextStyleExample {
        name: "Large Title",
        size: 26.0,
        weight: TextWeight::Bold,
    },
    TextStyleExample {
        name: "Title1",
        size: 22.0,
        weight: TextWeight::Bold,
    },
    TextStyleExample {
        name: "Body",
        size: 13.0,
        weight: TextWeight::Regular,
    },
    TextStyleExample {
        name: "Large Title Regular",
        size: 26.0,
        weight: TextWeight::Regular,
    },
    TextStyleExample {
        name: "Title1 Regular",
        size: 22.0,
        weight: TextWeight::Regular,
    },
    TextStyleExample {
        name: "Title2",
        size: 17.0,
        weight: TextWeight::Regular,
    },
    TextStyleExample {
        name: "Title3",
        size: 15.0,
        weight: TextWeight::Regular,
    },
    TextStyleExample {
        name: "Headline",
        size: 13.0,
        weight: TextWeight::Bold,
    },
];

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
        Container(
            VStack::new()
                .child(Text("Basic components"))
                .children(TEXT_STYLE_EXAMPLES.iter().map(|example| {
                    Text(example.name)
                        .font_size(example.size)
                        .font_weight(example.weight)
                }))
                .child(Label("Search").secondary("⌘K").icon(IconName::Search))
                .child(
                    Icon(IconName::Search)
                        .size(24.0)
                        .id("gallery-search-icon")
                        .accessibility_label("Search icon"),
                )
                .child(
                    Image("assets/playground-sample.png")
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

#[cfg(test)]
mod tests {
    use super::{TEXT_STYLE_EXAMPLES, TextStyleExample};
    use rustui::prelude::TextWeight;

    #[test]
    fn text_gallery_matches_reference_size_and_weight_samples() {
        assert_eq!(
            TEXT_STYLE_EXAMPLES,
            [
                TextStyleExample {
                    name: "Large Title",
                    size: 26.0,
                    weight: TextWeight::Bold,
                },
                TextStyleExample {
                    name: "Title1",
                    size: 22.0,
                    weight: TextWeight::Bold,
                },
                TextStyleExample {
                    name: "Body",
                    size: 13.0,
                    weight: TextWeight::Regular,
                },
                TextStyleExample {
                    name: "Large Title Regular",
                    size: 26.0,
                    weight: TextWeight::Regular,
                },
                TextStyleExample {
                    name: "Title1 Regular",
                    size: 22.0,
                    weight: TextWeight::Regular,
                },
                TextStyleExample {
                    name: "Title2",
                    size: 17.0,
                    weight: TextWeight::Regular,
                },
                TextStyleExample {
                    name: "Title3",
                    size: 15.0,
                    weight: TextWeight::Regular,
                },
                TextStyleExample {
                    name: "Headline",
                    size: 13.0,
                    weight: TextWeight::Bold,
                },
            ]
        );
    }
}
