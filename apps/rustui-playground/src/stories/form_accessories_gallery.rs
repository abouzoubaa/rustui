//! Public-API composition for the macOS 27 Form Accessories reference.

use rustui::prelude::*;
use std::sync::OnceLock;

const FORM_ITEMS: [&str; 11] = [
    "Button",
    "Checkboxes",
    "Combo Box",
    "Detail Disclosure",
    "Detail",
    "Detail Info",
    "Detail Menu",
    "Pop-Up Button",
    "Radio Button",
    "Stepper",
    "Switch",
];

const FORM_ITEM_KEYS: [&str; 11] = [
    "lab-form-button",
    "lab-form-checkboxes",
    "lab-form-combo",
    "lab-form-disclosure",
    "lab-form-detail",
    "lab-form-detail-info",
    "lab-form-detail-menu",
    "lab-form-popup",
    "lab-form-radio",
    "lab-form-stepper",
    "lab-form-switch",
];

fn form_item_names() -> &'static Vec<Vec<String>> {
    static NAMES: OnceLock<Vec<Vec<String>>> = OnceLock::new();
    NAMES.get_or_init(|| {
        let catalogs = super::locale_selector::catalogs();
        FORM_ITEM_KEYS
            .iter()
            .map(|key| {
                ["en", "fr", "ar"]
                    .iter()
                    .map(|language| {
                        let locale = rustui::localization::Locale::parse(language)
                            .expect("supported locale");
                        catalogs
                            .get(&locale)
                            .and_then(|catalog| catalog.get(key))
                            .unwrap_or(key)
                            .to_lowercase()
                    })
                    .collect()
            })
            .collect()
    })
}

fn visible_form_items(query: &str) -> Vec<&'static str> {
    let query = query.trim().to_lowercase();
    FORM_ITEMS
        .iter()
        .copied()
        .zip(form_item_names())
        .filter(|(_, names)| names.iter().any(|name| name.contains(&query)))
        .map(|(item, _)| item)
        .collect()
}

fn reference_row<V: View>(label: &'static str, control: V) -> impl View {
    HStack::new()
        .spacing(12.0)
        .child(LocalizedText(
            FORM_ITEM_KEYS[FORM_ITEMS
                .iter()
                .position(|item| *item == label)
                .expect("known form item")],
        ))
        .child(control)
}

pub fn demo() -> impl View {
    Component("form-accessories-gallery", FormAccessoriesGallery::default)
}

#[component]
struct FormAccessoriesGallery {
    #[state(String::new())]
    query: String,
    #[state(true)]
    checked_checkbox: bool,
    #[state(false)]
    unchecked_checkbox: bool,
    #[state(String::from("Label"))]
    combo_selection: String,
    #[state(false)]
    detail_disclosure_open: bool,
    #[state(String::from("first"))]
    radio_selection: String,
    #[state(67)]
    stepper_value: i32,
    #[state(true)]
    switch_checked: bool,
    #[state(String::from("None"))]
    last_action: String,
}

impl FormAccessoriesGallery {
    fn activate_button(&mut self) {
        self.last_action = String::from("Button");
    }

    fn activate_detail(&mut self) {
        self.last_action = String::from("Detail");
    }

    fn activate_detail_info(&mut self) {
        self.last_action = String::from("Detail Info");
    }

    fn last_action_feedback_key(&self) -> &'static str {
        match self.last_action().as_str() {
            "Button" => "lab-form-action-button",
            "Detail" => "lab-form-action-detail",
            "Detail Info" => "lab-form-action-detail-info",
            _ => "lab-form-action-none",
        }
    }

    fn radio_binding(&self, choice: &'static str) -> Binding<bool> {
        Binding::new_with(
            self.radio_selection() == choice,
            String::from(choice),
            Self::read_radio_selection,
            Self::write_radio_selection,
        )
    }

    fn read_radio_selection(owner: &Self, choice: &String) -> bool {
        owner.radio_selection() == *choice
    }

    fn write_radio_selection(owner: &mut Self, choice: &String, selected: bool) -> bool {
        selected && owner.set_radio_selection(choice.clone())
    }

    fn reference_rows(&self) -> Vec<(&'static str, AnyView)> {
        vec![
            (
                FORM_ITEMS[0],
                AnyView::new(reference_row(
                    FORM_ITEMS[0],
                    Button("Label", self.action(Self::activate_button))
                        .localized_label("lab-form-control-label")
                        .id("form-accessories-button"),
                )),
            ),
            (
                FORM_ITEMS[1],
                AnyView::new(reference_row(
                    FORM_ITEMS[1],
                    HStack::new()
                        .spacing(8.0)
                        .child(
                            Checkbox("Label", self.checked_checkbox_binding())
                                .localized_label("lab-form-control-label")
                                .id("form-accessories-checkbox-checked"),
                        )
                        .child(
                            Checkbox("Label", self.unchecked_checkbox_binding())
                                .localized_label("lab-form-control-label")
                                .id("form-accessories-checkbox-unchecked"),
                        ),
                )),
            ),
            (
                FORM_ITEMS[2],
                AnyView::new(reference_row(
                    FORM_ITEMS[2],
                    ComboBox("Label", self.combo_selection_binding(), ["Label"])
                        .id("form-accessories-combo-box"),
                )),
            ),
            (
                FORM_ITEMS[3],
                AnyView::new(reference_row(
                    FORM_ITEMS[3],
                    Disclosure(
                        "form-accessories-detail-disclosure",
                        "Label",
                        self.detail_disclosure_open_binding(),
                        rustui::view::EmptyView,
                    )
                    .localized_label("lab-form-control-label"),
                )),
            ),
            (
                FORM_ITEMS[4],
                AnyView::new(reference_row(
                    FORM_ITEMS[4],
                    Button("Label", self.action(Self::activate_detail))
                        .localized_label("lab-form-control-label")
                        .id("form-accessories-detail"),
                )),
            ),
            (
                FORM_ITEMS[5],
                AnyView::new(reference_row(
                    FORM_ITEMS[5],
                    HStack::new()
                        .spacing(8.0)
                        .child(
                            Button("Label", self.action(Self::activate_detail_info))
                                .localized_label("lab-form-control-label")
                                .id("form-accessories-detail-info"),
                        )
                        .child(Icon(IconName::Info)),
                )),
            ),
            (
                FORM_ITEMS[6],
                AnyView::new(reference_row(
                    FORM_ITEMS[6],
                    HStack::new()
                        .spacing(8.0)
                        .child(
                            Menu("Label", [])
                                .localized_label("lab-form-control-label")
                                .id("form-accessories-detail-menu"),
                        )
                        .child(Icon(IconName::Ellipsis)),
                )),
            ),
            (
                FORM_ITEMS[7],
                AnyView::new(reference_row(
                    FORM_ITEMS[7],
                    Picker("Label", self.combo_selection_binding(), ["Label"])
                        .id("form-accessories-pop-up-button"),
                )),
            ),
            (
                FORM_ITEMS[8],
                AnyView::new(reference_row(
                    FORM_ITEMS[8],
                    HStack::new()
                        .spacing(8.0)
                        .child(
                            Radio("Label", self.radio_binding("first"))
                                .localized_label("lab-form-control-label")
                                .id("form-accessories-radio-first"),
                        )
                        .child(
                            Radio("Label", self.radio_binding("second"))
                                .localized_label("lab-form-control-label")
                                .id("form-accessories-radio-second"),
                        ),
                )),
            ),
            (
                FORM_ITEMS[9],
                AnyView::new(reference_row(
                    FORM_ITEMS[9],
                    Stepper("Label", self.stepper_value_binding())
                        .localized_label("lab-form-control-label")
                        .id("form-accessories-stepper"),
                )),
            ),
            (
                FORM_ITEMS[10],
                AnyView::new(reference_row(
                    FORM_ITEMS[10],
                    Toggle("Label", self.switch_checked_binding())
                        .localized_label("lab-form-control-label")
                        .id("form-accessories-switch"),
                )),
            ),
        ]
    }
}

impl View for FormAccessoriesGallery {
    fn body(&self) -> impl View {
        let query = self.query_binding();
        let action_feedback_key = self.last_action_feedback_key();
        let visible = visible_form_items(&self.query());
        let mut content = VStack::new()
            .spacing(10.0)
            .child(LocalizedText("lab-form-heading"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Icon(IconName::Search))
                    .child(LocalizedText("lab-form-filter"))
                    .child(
                        TextField("Filter form items", query)
                            .id("form-accessories-filter")
                            .localized_placeholder("lab-form-filter-placeholder")
                            .localized_accessibility_label("lab-form-filter"),
                    ),
            );
        for (label, row) in self.reference_rows() {
            if visible.contains(&label) {
                content = content.child(row);
            }
        }
        content.child(
            Semantics(
                "form-accessories-action-status",
                LocalizedText(action_feedback_key),
            )
            .localized_live_region_value(action_feedback_key, LivePoliteness::Polite),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{FORM_ITEMS, FormAccessoriesGallery, visible_form_items};

    #[test]
    fn filter_matches_translated_names_and_english() {
        assert_eq!(visible_form_items("interrupteur"), vec!["Switch"]);
        assert_eq!(visible_form_items("مربعات"), vec!["Checkboxes"]);
        assert_eq!(visible_form_items("CHECKBOX"), vec!["Checkboxes"]);
        assert!(visible_form_items("no matching item").is_empty());
    }

    #[test]
    fn form_accessories_reference_composition_matches_svg_controls_and_initial_states() {
        let mut gallery = FormAccessoriesGallery::default();
        let rendered_items = gallery
            .reference_rows()
            .into_iter()
            .map(|(label, _)| label)
            .collect::<Vec<_>>();

        assert_eq!(rendered_items, FORM_ITEMS);
        assert!(gallery.checked_checkbox());
        assert!(!gallery.unchecked_checkbox());
        assert_eq!(gallery.combo_selection(), "Label");
        assert!(!gallery.detail_disclosure_open());
        assert!(gallery.radio_binding("first").value());
        assert!(!gallery.radio_binding("second").value());
        assert_eq!(gallery.stepper_value(), 67);
        assert!(gallery.switch_checked());

        let first = gallery.radio_binding("first");
        let second = gallery.radio_binding("second");
        assert!(second.write_to(&mut gallery, true));
        assert_eq!(first.read_from(&gallery), Some(false));
        assert_eq!(second.read_from(&gallery), Some(true));
        assert!(!first.write_to(&mut gallery, false));
        assert_eq!(second.read_from(&gallery), Some(true));
    }

    #[test]
    fn form_accessories_filter_matches_reference_labels_and_blank_restores_all() {
        assert_eq!(visible_form_items("DISC"), ["Detail Disclosure"]);
        assert_eq!(visible_form_items("  "), FORM_ITEMS);
    }

    #[test]
    fn form_accessory_buttons_report_the_activated_reference_action() {
        let mut gallery = FormAccessoriesGallery::default();
        assert_eq!(gallery.last_action(), "None");
        assert_eq!(gallery.last_action_feedback_key(), "lab-form-action-none");

        gallery.activate_button();
        assert_eq!(gallery.last_action(), "Button");
        assert_eq!(gallery.last_action_feedback_key(), "lab-form-action-button");
        gallery.activate_detail();
        assert_eq!(gallery.last_action(), "Detail");
        assert_eq!(gallery.last_action_feedback_key(), "lab-form-action-detail");
        gallery.activate_detail_info();
        assert_eq!(gallery.last_action(), "Detail Info");
        assert_eq!(
            gallery.last_action_feedback_key(),
            "lab-form-action-detail-info"
        );
    }
}
