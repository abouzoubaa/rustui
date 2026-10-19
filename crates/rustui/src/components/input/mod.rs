//! input component family.
//!
//! Scaffold only. The modules below do not yet implement their planned APIs.

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

// Select keeps a displayed value snapshot and a separate popup cursor. Replacing
// its items can leave the cursor at the old index even when the value matches.
fn select_needs_reconciliation(
    options_changed: bool,
    retained_value: Option<&str>,
    owner_value: &str,
) -> bool {
    options_changed || retained_value != Some(owner_value)
}

#[cfg(test)]
mod tests {
    use super::select_needs_reconciliation;

    #[test]
    fn changed_options_resync_cursor_even_when_selected_value_is_unchanged() {
        assert!(select_needs_reconciliation(true, Some("Skia"), "Skia"));
        assert!(select_needs_reconciliation(false, Some("GPUI"), "Skia"));
        assert!(!select_needs_reconciliation(false, Some("Skia"), "Skia"));
    }
}
