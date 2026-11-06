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
