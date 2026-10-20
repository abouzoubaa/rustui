//! Runtime locale and logical-direction selection for Playground.

use rustui::{
    localization::{Catalog, Catalogs, Locale, set_locale_action, validate_catalogs},
    prelude::{Button, ButtonStyle, HStack, VerticalAlignment, View},
};

pub struct LocaleSelector;

impl LocaleSelector {
    pub fn new() -> Self {
        Self
    }
}

impl View for LocaleSelector {
    fn body(&self) -> impl View {
        HStack::new()
            .child(
                Button("English", set_locale_action(locale("en")))
                    .id("lab-locale-en")
                    .button_style(ButtonStyle::Bordered),
            )
            .child(
                Button("Français", set_locale_action(locale("fr")))
                    .id("lab-locale-fr")
                    .button_style(ButtonStyle::Bordered),
            )
            .child(
                Button("العربية", set_locale_action(locale("ar")))
                    .id("lab-locale-ar")
                    .button_style(ButtonStyle::Bordered),
            )
            .spacing(8.0)
            .alignment(VerticalAlignment::Center)
    }
}

pub fn initial_locale() -> Locale {
    locale("en")
}

pub fn catalogs() -> Catalogs {
    let fallback = initial_locale();
    let mut catalogs = Catalogs::new(fallback);
    for (locale, framework, lab) in [
        (
            locale("en"),
            include_str!("../../../../locales/en/rustui.ftl"),
            include_str!("../../locales/en/lab.ftl"),
        ),
        (
            locale("fr"),
            include_str!("../../../../locales/fr/rustui.ftl"),
            include_str!("../../locales/fr/lab.ftl"),
        ),
        (
            locale("ar"),
            include_str!("../../../../locales/ar/rustui.ftl"),
            include_str!("../../locales/ar/lab.ftl"),
        ),
    ] {
        let source = format!("{framework}\n{lab}");
        let catalog = Catalog::parse(locale, &source).expect("Playground catalogs should be valid");
        catalogs
            .insert(catalog)
            .expect("Playground locales should be registered once");
    }
    assert!(
        validate_catalogs(&catalogs).is_valid(),
        "Playground catalogs should define the same message IDs"
    );
    catalogs
}

fn locale(identifier: &str) -> Locale {
    Locale::parse(identifier).expect("Playground locale constants should be valid")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_story_messages_exist_in_every_playground_catalog() {
        let catalogs = catalogs();
        for identifier in ["en", "fr", "ar"] {
            let catalog = catalogs
                .get(&locale(identifier))
                .expect("Playground locale should be registered");
            for message in ["lab-sheet-open", "lab-sheet-title", "lab-sheet-name"] {
                assert!(
                    catalog.get(message).is_some(),
                    "{message} missing in {identifier}"
                );
            }
        }
    }
}
