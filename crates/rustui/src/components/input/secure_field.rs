//! Password input using the retained RustUI text-field path.

use gpui::SharedString;
use rustui_core::Binding;

use super::text_field::{TextFieldKind, TextFieldView};

/// Creates a controlled password field with masked text and a reveal control.
#[allow(non_snake_case)]
pub fn SecureField(placeholder: impl Into<SharedString>, value: Binding<String>) -> TextFieldView {
    TextFieldView::new(placeholder, value).with_kind(TextFieldKind::Secure)
}
