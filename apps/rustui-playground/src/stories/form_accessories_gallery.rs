//! Public-API composition for the macOS 27 Form Accessories reference.

use rustui::prelude::*;

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

fn visible_form_items(query: &str) -> Vec<&'static str> {
    let query = query.trim().to_lowercase();
    FORM_ITEMS
        .iter()
        .copied()
        .filter(|item| item.to_lowercase().contains(&query))
        .collect()
}

fn reference_row<V: View>(label: &'static str, control: V) -> impl View {
    HStack::new()
        .spacing(12.0)
        .child(Text(label))
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

    fn last_action_feedback(&self) -> String {
        format!("Last activated: {}", self.last_action())
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
                                .id("form-accessories-checkbox-checked"),
                        )
                        .child(
                            Checkbox("Label", self.unchecked_checkbox_binding())
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
                    ),
                )),
            ),
            (
                FORM_ITEMS[4],
                AnyView::new(reference_row(
                    FORM_ITEMS[4],
                    Button("Label", self.action(Self::activate_detail))
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
                        .child(Menu("Label", []).id("form-accessories-detail-menu"))
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
                                .id("form-accessories-radio-first"),
                        )
                        .child(
                            Radio("Label", self.radio_binding("second"))
                                .id("form-accessories-radio-second"),
                        ),
                )),
            ),
            (
                FORM_ITEMS[9],
                AnyView::new(reference_row(
                    FORM_ITEMS[9],
                    Stepper("Label", self.stepper_value_binding()).id("form-accessories-stepper"),
                )),
            ),
            (
                FORM_ITEMS[10],
                AnyView::new(reference_row(
                    FORM_ITEMS[10],
                    Toggle("Label", self.switch_checked_binding()).id("form-accessories-switch"),
                )),
            ),
        ]
    }
}

impl View for FormAccessoriesGallery {
    fn body(&self) -> impl View {
        let query = self.query_binding();
        let visible = visible_form_items(&self.query());
        let mut content = VStack::new()
            .spacing(10.0)
            .child(Text("Form Accessories"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Label("Filter").icon(IconName::Search))
                    .child(TextField("Filter form items", query).id("form-accessories-filter")),
            );
        for (label, row) in self.reference_rows() {
            if visible.contains(&label) {
                content = content.child(row);
            }
        }
        content.child(Text(self.last_action_feedback()))
    }
}

#[cfg(test)]
mod tests {
    use super::{FORM_ITEMS, FormAccessoriesGallery, visible_form_items};

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

        gallery.activate_button();
        assert_eq!(gallery.last_action(), "Button");
        assert_eq!(gallery.last_action_feedback(), "Last activated: Button");
        gallery.activate_detail();
        assert_eq!(gallery.last_action(), "Detail");
        assert_eq!(gallery.last_action_feedback(), "Last activated: Detail");
        gallery.activate_detail_info();
        assert_eq!(gallery.last_action(), "Detail Info");
        assert_eq!(
            gallery.last_action_feedback(),
            "Last activated: Detail Info"
        );
    }
}
