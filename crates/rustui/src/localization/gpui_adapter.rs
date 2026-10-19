//! GPUI Components locale adapter.
//!
//! This module is the RustUI boundary for synchronizing the selected
//! application locale with GPUI Components built-in control translations.

use rustui_i18n::Locale;

pub(crate) fn set_locale(locale: &Locale) {
    gpui_component::set_locale(locale.as_str());
}
