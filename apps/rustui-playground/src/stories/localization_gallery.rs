//! Public RustUI example for named catalog-message interpolation.

use rustui::{
    localization::{Catalogs, Locale, MessageArguments, interpolate},
    prelude::{LocalizedText, Text, VStack, View},
};

pub fn demo() -> impl View {
    let catalogs = super::locale_selector::catalogs();
    let arguments = MessageArguments::new()
        .with("name", "Ada")
        .with("count", 3_i32);

    VStack::new()
        .child(LocalizedText("lab-i18n-title"))
        .child(Text(
            "Each locale resolves its catalog text before interpolation.",
        ))
        .child(locale_row(&catalogs, "en", "English", &arguments))
        .child(locale_row(&catalogs, "fr", "Français", &arguments))
        .child(locale_row(&catalogs, "ar", "العربية", &arguments))
}

fn locale_row(
    catalogs: &Catalogs,
    locale_id: &str,
    locale_name: &str,
    arguments: &MessageArguments,
) -> impl View {
    let locale = Locale::parse(locale_id).expect("Playground locale should be valid");
    let formatted = interpolate(catalogs.resolve(&locale, "lab-i18n-sample"), arguments)
        .unwrap_or_else(|error| format!("Formatting error: {error}"));
    let malformed = interpolate(catalogs.resolve(&locale, "lab-i18n-malformed"), arguments)
        .map(|formatted| format!("Unexpectedly formatted: {formatted}"))
        .unwrap_or_else(|error| error.to_string());

    VStack::new()
        .child(Text(format!("{locale_name}: {formatted}")))
        .child(Text(format!(
            "{}: {malformed}",
            catalogs.resolve(&locale, "lab-i18n-error-label")
        )))
}
