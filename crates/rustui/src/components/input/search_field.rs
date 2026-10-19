//! Search input using the retained RustUI text-field path.

use gpui::SharedString;
use rustui_core::Binding;

use super::text_field::{TextFieldKind, TextFieldView};

/// Creates a controlled search field with the native clear affordance.
#[allow(non_snake_case)]
pub fn SearchField(placeholder: impl Into<SharedString>, value: Binding<String>) -> TextFieldView {
    TextFieldView::new(placeholder, value).with_kind(TextFieldKind::Search)
}
