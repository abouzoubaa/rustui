//! Search input using the retained RustUI text-field path.

use gpui::SharedString;
use rustui_core::Binding;

use super::text_field::{TextFieldKind, TextFieldView};

fn default_accessibility_label(placeholder: &str) -> Option<&'static str> {
    placeholder.trim().is_empty().then_some("Search")
}

/// Creates a controlled search field with the native clear affordance.
#[allow(non_snake_case)]
pub fn SearchField(placeholder: impl Into<SharedString>, value: Binding<String>) -> TextFieldView {
    let placeholder = placeholder.into();
    let field = TextFieldView::new(placeholder.clone(), value).with_kind(TextFieldKind::Search);
    match default_accessibility_label(placeholder.as_ref()) {
        Some(label) => field.accessibility_label(label),
        None => field,
    }
}

#[cfg(test)]
mod tests {
    use super::default_accessibility_label;

    #[test]
    fn blank_placeholder_still_identifies_search_field() {
        assert_eq!(default_accessibility_label("  \t"), Some("Search"));
        assert_eq!(default_accessibility_label("Search files"), None);
    }
}
