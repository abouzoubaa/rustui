use rustui::component;
use rustui::prelude::{
    ColorPicker, Component, Hsla, Image, ImageFit, LocalizedText, Slider, TabItem, TabView,
    TabViewStyle, VStack, View,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UtilityWindowPanel {
    ColorWheel,
    Sliders,
    Swatches,
    Images,
    Crayons,
}

impl UtilityWindowPanel {
    fn label_message_id(self) -> &'static str {
        match self {
            Self::ColorWheel => "lab-utility-tab-color-wheel",
            Self::Sliders => "lab-utility-tab-sliders",
            Self::Swatches => "lab-utility-tab-swatches",
            Self::Images => "lab-utility-tab-images",
            Self::Crayons => "lab-utility-tab-crayons",
        }
    }
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
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-utility-title"))
            .child(
                TabView(
                    UTILITY_WINDOW_TABS.iter().map(|(id, label, panel)| {
                        let item = match panel {
                            UtilityWindowPanel::ColorWheel => TabItem::new(
                                *id,
                                *label,
                                VStack::new()
                                    .spacing(8.0)
                                    .child(LocalizedText("lab-utility-color-wheel"))
                                    .child(
                                        ColorPicker("Color", self.wheel_color_binding())
                                            .localized_label("lab-utility-picker-color"),
                                    ),
                            ),
                            UtilityWindowPanel::Sliders => TabItem::new(
                                *id,
                                *label,
                                VStack::new()
                                    .spacing(8.0)
                                    .child(LocalizedText("lab-utility-color-channels"))
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
                                    .child(LocalizedText("lab-utility-featured-colors"))
                                    .child(
                                        ColorPicker("Featured color", self.swatch_color_binding())
                                            .localized_label("lab-utility-picker-featured"),
                                    ),
                            ),
                            UtilityWindowPanel::Images => TabItem::new(
                                *id,
                                *label,
                                VStack::new()
                                    .spacing(8.0)
                                    .child(LocalizedText("lab-utility-images"))
                                    .child(
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
                                    .child(LocalizedText("lab-utility-crayon-colors"))
                                    .child(
                                        ColorPicker("Crayon color", self.crayon_color_binding())
                                            .localized_label("lab-utility-picker-crayon"),
                                    ),
                            ),
                        };
                        item.localized_label(panel.label_message_id())
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
        UTILITY_WINDOW_CHANNEL_LABELS, UTILITY_WINDOW_SAMPLE_IMAGE_LABEL, UTILITY_WINDOW_TABS,
        UTILITY_WINDOW_TITLE, UtilityWindowGallery, UtilityWindowPanel,
    };
    use rustui::localization::Locale;

    #[test]
    fn utility_window_messages_resolve_in_supported_locales() {
        let catalogs = crate::stories::locale_selector::catalogs();
        for tag in ["en", "fr", "ar"] {
            let locale = Locale::parse(tag).unwrap();
            for key in [
                "lab-utility-title",
                "lab-utility-color-wheel",
                "lab-utility-color-channels",
                "lab-utility-featured-colors",
                "lab-utility-images",
                "lab-utility-crayon-colors",
                "lab-utility-picker-color",
                "lab-utility-picker-featured",
                "lab-utility-picker-crayon",
            ] {
                assert_ne!(catalogs.resolve(&locale, key), key, "{tag}: {key}");
            }
            for (_, _, panel) in UTILITY_WINDOW_TABS {
                let key = panel.label_message_id();
                assert_ne!(catalogs.resolve(&locale, key), key, "{tag}: {key}");
            }
        }
    }

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
