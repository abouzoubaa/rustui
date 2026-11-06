use rustui::component;
use rustui::prelude::{
    ColorPicker, Component, Hsla, Image, ImageFit, Slider, TabItem, TabView, TabViewStyle, Text,
    VStack, View,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UtilityWindowPanel {
    ColorWheel,
    Sliders,
    Swatches,
    Images,
    Crayons,
}

const UTILITY_WINDOW_TABS: [(&str, &str, UtilityWindowPanel); 5] = [
    ("color-wheel", "Color Wheel", UtilityWindowPanel::ColorWheel),
    ("sliders", "Sliders", UtilityWindowPanel::Sliders),
    ("swatches", "Swatches", UtilityWindowPanel::Swatches),
    ("images", "Images", UtilityWindowPanel::Images),
    ("crayons", "Crayons", UtilityWindowPanel::Crayons),
];
const UTILITY_WINDOW_TITLE: &str = "Utility Panel";
const UTILITY_WINDOW_CHANNEL_LABELS: [&str; 3] = ["Red", "Green", "Blue"];
const UTILITY_WINDOW_SAMPLE_IMAGE_LABEL: &str = "Purple and cyan sample image";

pub fn demo() -> impl View {
    Component("utility-window-gallery", UtilityWindowGallery::default)
}

#[component]
struct UtilityWindowGallery {
    #[state(String::from("color-wheel"))]
    selected_tab: String,
    #[state(None)]
    wheel_color: Option<Hsla>,
    #[state(0.5)]
    red: f32,
    #[state(0.5)]
    green: f32,
    #[state(0.5)]
    blue: f32,
    #[state(None)]
    swatch_color: Option<Hsla>,
    #[state(None)]
    crayon_color: Option<Hsla>,
}

impl View for UtilityWindowGallery {
    fn body(&self) -> impl View {
        VStack::new().spacing(8.0).child(Text(UTILITY_WINDOW_TITLE)).child(
            TabView(
                UTILITY_WINDOW_TABS
                    .iter()
                    .map(|(id, label, panel)| match panel {
                        UtilityWindowPanel::ColorWheel => TabItem::new(
                            *id,
                            *label,
                            VStack::new()
                                .spacing(8.0)
                                .child(Text("Color Wheel"))
                                .child(ColorPicker("Color", self.wheel_color_binding())),
                        ),
                        UtilityWindowPanel::Sliders => TabItem::new(
                            *id,
                            *label,
                            VStack::new()
                                .spacing(8.0)
                                .child(Text("Color channels"))
                                .child(
                                    Slider(self.red_binding(), 0.0..=1.0)
                                        .accessibility_label(UTILITY_WINDOW_CHANNEL_LABELS[0]),
                                )
                                .child(
                                    Slider(self.green_binding(), 0.0..=1.0)
                                        .accessibility_label(UTILITY_WINDOW_CHANNEL_LABELS[1]),
                                )
                                .child(
                                    Slider(self.blue_binding(), 0.0..=1.0)
                                        .accessibility_label(UTILITY_WINDOW_CHANNEL_LABELS[2]),
                                ),
                        ),
                        UtilityWindowPanel::Swatches => TabItem::new(
                            *id,
                            *label,
                            VStack::new()
                                .spacing(8.0)
                                .child(Text("Featured colors"))
                                .child(ColorPicker("Featured color", self.swatch_color_binding())),
                        ),
                        UtilityWindowPanel::Images => TabItem::new(
                            *id,
                            *label,
                            VStack::new().spacing(8.0).child(Text("Images")).child(
                                Image("assets/playground-sample.png")
                                    .size(180.0, 120.0)
                                    .fit(ImageFit::Contain)
                                    .id("utility-window-sample-image")
                                    .accessibility_label(UTILITY_WINDOW_SAMPLE_IMAGE_LABEL),
                            ),
                        ),
                        UtilityWindowPanel::Crayons => TabItem::new(
                            *id,
                            *label,
                            VStack::new()
                                .spacing(8.0)
                                .child(Text("Crayon colors"))
                                .child(ColorPicker("Crayon color", self.crayon_color_binding())),
                        ),
                    }),
                self.selected_tab_binding(),
            )
            .id("utility-window-tabs")
            .style(TabViewStyle::Segmented)
            .accessibility_label("Utility palette categories"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        UtilityWindowGallery, UtilityWindowPanel, UTILITY_WINDOW_CHANNEL_LABELS,
        UTILITY_WINDOW_SAMPLE_IMAGE_LABEL, UTILITY_WINDOW_TABS, UTILITY_WINDOW_TITLE,
    };

    #[test]
    fn utility_window_gallery_uses_the_catalog_described_title() {
        assert_eq!(UTILITY_WINDOW_TITLE, "Utility Panel");
    }

    #[test]
    fn utility_window_gallery_matches_the_reference_tab_groups() {
        assert_eq!(
            UTILITY_WINDOW_TABS,
            [
                ("color-wheel", "Color Wheel", UtilityWindowPanel::ColorWheel),
                ("sliders", "Sliders", UtilityWindowPanel::Sliders),
                ("swatches", "Swatches", UtilityWindowPanel::Swatches),
                ("images", "Images", UtilityWindowPanel::Images),
                ("crayons", "Crayons", UtilityWindowPanel::Crayons),
            ]
        );
        assert_eq!(
            UtilityWindowGallery::default().selected_tab(),
            "color-wheel"
        );
    }

    #[test]
    fn utility_window_gallery_names_accessible_controls() {
        assert_eq!(UTILITY_WINDOW_CHANNEL_LABELS, ["Red", "Green", "Blue"]);
        assert_eq!(
            UTILITY_WINDOW_SAMPLE_IMAGE_LABEL,
            "Purple and cyan sample image"
        );
    }
}
