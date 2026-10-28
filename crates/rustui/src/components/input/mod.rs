//! Public input and selection controls with values connected through RustUI bindings.

pub mod checkbox;
pub mod color_picker;
pub mod combo_box;
pub mod date_picker;
pub mod picker;
pub mod radio;
pub mod search_field;
pub mod secure_field;
pub mod slider;
pub mod stepper;
pub mod text_editor;
pub mod text_field;
pub mod toggle;
pub mod token_field;

/// A select option keeps its stable binding value separate from its display title.
#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectOption {
    value: String,
    title: gpui::SharedString,
}

impl gpui_component::searchable_list::SearchableListItem for SelectOption {
    type Value = String;

    fn title(&self) -> gpui::SharedString {
        self.title.clone()
    }

    fn value(&self) -> &String {
        &self.value
    }
}

fn select_options(
    values: &[String],
    keys: &std::collections::HashMap<String, gpui::SharedString>,
    mut resolve: impl FnMut(&str) -> String,
) -> Vec<SelectOption> {
    values
        .iter()
        .map(|value| {
            let title = keys
                .get(value)
                .map(|key| {
                    let text = resolve(key);
                    if text.trim().is_empty() || text.trim() == key.trim() {
                        value.clone()
                    } else {
                        text
                    }
                })
                .unwrap_or_else(|| value.clone());
            SelectOption {
                value: value.clone(),
                title: title.into(),
            }
        })
        .collect()
}

pub(super) fn mark_accessibility_node_disabled(node: &mut gpui::accesskit::Node) {
    node.set_disabled();
}

// Select keeps a displayed value snapshot and a separate popup cursor. Replacing
// its items can leave the cursor at the old index even when the value matches.
fn select_needs_reconciliation(
    options_changed: bool,
    previous_owner_value: &str,
    current_owner_value: &str,
) -> bool {
    options_changed || previous_owner_value != current_owner_value
}

#[cfg(test)]
mod tests {
    #[test]
    fn localized_select_titles_keep_stable_distinct_values_and_literal_fallbacks() {
        use super::select_options;
        use gpui_component::searchable_list::SearchableListItem;
        let values = vec!["first".to_owned(), "second".to_owned(), "third".to_owned()];
        let keys = [
            ("first".to_owned(), "first.key".into()),
            ("second".to_owned(), "second.key".into()),
        ]
        .into_iter()
        .collect();
        let translated = select_options(&values, &keys, |_| "Même titre".into());
        assert_eq!(translated[0].title(), "Même titre");
        assert_eq!(translated[1].title(), "Même titre");
        assert_eq!(translated[0].value(), "first");
        assert_eq!(translated[1].value(), "second");
        assert_eq!(translated[2].title(), "third");
        let missing = select_options(&values, &keys, |key| key.to_owned());
        assert_eq!(missing[0].title(), "first");
        let blank = select_options(&values, &keys, |_| " ".into());
        assert_eq!(blank[1].title(), "second");
    }

    #[test]
    fn localized_options_reject_unknown_values_and_blank_keys() {
        let binding = || {
            Binding::new(
                String::new(),
                |value: &String| value.clone(),
                |value: &mut String, next| *value = next,
            )
        };
        assert!(
            catch_unwind(AssertUnwindSafe(
                || Picker("Mode", binding(), ["one"]).localized_option("missing", "option.key")
            ))
            .is_err()
        );
        assert!(
            catch_unwind(AssertUnwindSafe(
                || ComboBox("Mode", binding(), ["one"]).localized_option("one", " ")
            ))
            .is_err()
        );
    }
    use super::select_needs_reconciliation;
    use crate::components::input::{
        checkbox::Checkbox, combo_box::ComboBox, picker::Picker, radio::Radio, slider::Slider,
        stepper::Stepper, toggle::Toggle,
    };
    use rustui_core::Binding;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    fn binding<T: Clone + 'static>(value: T) -> Binding<T> {
        Binding::new(
            value,
            |value: &T| value.clone(),
            |value, next| *value = next,
        )
    }

    fn assert_blank_identity_rejected(build: impl FnOnce()) {
        assert!(catch_unwind(AssertUnwindSafe(build)).is_err());
    }

    #[test]
    fn picker_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Picker("Mode", binding(String::new()), ["one"]).id("");
        });
    }

    #[test]
    fn combo_box_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            ComboBox("Mode", binding(String::new()), ["one"]).id(" \t ");
        });
    }

    #[test]
    fn toggle_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Toggle("Enabled", binding(false)).id("");
        });
    }

    #[test]
    fn checkbox_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Checkbox("Enabled", binding(false)).id(" \t ");
        });
    }

    #[test]
    fn radio_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Radio("Enabled", binding(false)).id("");
        });
    }

    #[test]
    fn slider_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Slider(binding(0.5_f32), 0.0..=1.0).id(" \t ");
        });
    }

    #[test]
    fn stepper_rejects_blank_explicit_identity() {
        assert_blank_identity_rejected(|| {
            Stepper("Count", binding(1_i32)).id("");
        });
    }

    #[test]
    fn changed_options_resync_cursor_even_when_selected_value_is_unchanged() {
        assert!(select_needs_reconciliation(true, "Skia", "Skia"));
        assert!(select_needs_reconciliation(false, "GPUI", "Skia"));
        assert!(!select_needs_reconciliation(false, "Skia", "Skia"));
    }

    #[test]
    fn disabled_input_accessibility_node_is_marked_disabled() {
        let mut node = gpui::accesskit::Node::new(gpui::accesskit::Role::CheckBox);
        super::mark_accessibility_node_disabled(&mut node);
        assert!(node.is_disabled());
    }
}
#[cfg(test)]
mod focus_tests;
