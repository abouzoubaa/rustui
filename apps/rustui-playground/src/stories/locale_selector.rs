//! Runtime locale and logical-direction selection for Playground.

use rustui::{
    localization::{
        Catalog, Catalogs, LayoutDirection, Locale, set_locale_action as rustui_set_locale_action,
        validate_catalogs,
    },
    prelude::{Button, ButtonStyle, HStack, VerticalAlignment, View},
    runtime::AppAction,
};

fn document_direction(locale: &Locale) -> &'static str {
    if LayoutDirection::for_locale(locale).is_right_to_left() {
        "rtl"
    } else {
        "ltr"
    }
}

fn locale_action(locale: Locale) -> AppAction {
    let action = rustui_set_locale_action(locale.clone());
    #[cfg(target_arch = "wasm32")]
    {
        let language = locale.as_str().to_owned();
        let direction = document_direction(&locale);
        action.then(AppAction::new(move |_| {
            set_document_locale(&language, direction);
        }))
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        action
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function setDocumentLocale(language, direction) { document.documentElement.lang = language; document.documentElement.dir = direction; }"
)]
extern "C" {
    #[wasm_bindgen::prelude::wasm_bindgen(js_name = setDocumentLocale)]
    fn set_document_locale(language: &str, direction: &str);
}

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
                Button("English", locale_action(locale("en")))
                    .id("lab-locale-en")
                    .button_style(ButtonStyle::Bordered),
            )
            .child(
                Button("Français", locale_action(locale("fr")))
                    .id("lab-locale-fr")
                    .button_style(ButtonStyle::Bordered),
            )
            .child(
                Button("العربية", locale_action(locale("ar")))
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
    let validation = validate_catalogs(&catalogs);
    assert!(
        validation.missing.is_empty()
            && validation.extra.is_empty()
            && validation.argument_mismatches.is_empty()
            && validation.expression_mismatches.is_empty()
            && validation
                .invalid_patterns
                .values()
                .flatten()
                .all(|identifier| identifier.as_str() == "lab-i18n-malformed"),
        "Playground catalogs should match except for the intentional malformed-pattern demo: {validation:?}"
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
    fn all_bundled_catalogs_pass_runtime_validation() {
        let _ = catalogs();
    }

    #[test]
    fn table_gallery_messages_resolve_in_each_runtime_locale() {
        use rustui::localization::MessageArguments;
        let catalogs = catalogs();
        for tag in ["en", "fr", "ar"] {
            let locale = locale(tag);
            let catalog = catalogs.get(&locale).unwrap();
            for id in [
                "lab-table-reconcile-title",
                "lab-table-select-0",
                "lab-table-select-2",
                "lab-table-clear",
                "lab-table-invalid",
                "lab-table-reverse",
                "lab-table-remove",
                "lab-table-restore",
                "lab-table-rows",
                "lab-table-reversed",
                "lab-table-forward",
                "lab-table-selected",
                "lab-table-no-selection",
                "lab-table-reconcile-help",
                "lab-table-index-header",
                "lab-table-row-header",
            ] {
                assert!(catalog.get(id).is_some(), "{id} missing in {tag}");
            }
            assert!(
                catalogs
                    .format(
                        &locale,
                        "lab-table-rows",
                        &MessageArguments::new().with("rows", "Alpha, Beta")
                    )
                    .unwrap()
                    .contains("Alpha, Beta")
            );
            assert!(
                !catalogs
                    .format(
                        &locale,
                        "lab-table-selected",
                        &MessageArguments::new().with("index", 2_i64)
                    )
                    .unwrap()
                    .is_empty()
            );
        }
    }

    #[test]
    fn runtime_validation_rejects_unsupported_interpolation_syntax() {
        let language = locale("en");
        let mut catalogs = Catalogs::new(language.clone());
        catalogs
            .insert(Catalog::parse(language.clone(), "value = Value: { $value }").unwrap())
            .unwrap();
        let report = validate_catalogs(&catalogs);
        assert!(
            report.invalid_patterns[&language]
                .iter()
                .any(|id| id.as_str() == "value")
        );
    }

    #[test]
    fn slider_gallery_value_formats_in_every_runtime_locale() {
        use rustui::localization::{MessageArguments, format_integer};
        let catalogs = catalogs();
        for identifier in ["en", "fr", "ar"] {
            let locale = locale(identifier);
            let value = catalogs
                .format(
                    &locale,
                    "lab-slider-gallery-value",
                    &MessageArguments::new().with("value", 35.0_f32),
                )
                .expect("slider value should format");
            assert!(
                value.contains(&format_integer(35, &locale)),
                "{identifier}: {value}"
            );
        }
    }

    #[test]
    fn browser_document_direction_matches_playground_locale() {
        for (identifier, expected) in [("en", "ltr"), ("fr", "ltr"), ("ar", "rtl")] {
            assert_eq!(document_direction(&locale(identifier)), expected);
        }
    }

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

    #[test]
    fn animation_identity_messages_format_values_in_each_locale() {
        use rustui::localization::{MessageArguments, format_integer};

        let catalogs = catalogs();
        for (identifier, mirror_label, visible_label, field_label) in [
            ("en", "Spring mirrors", "A fields visible", "Spring A"),
            ("fr", "Valeurs du ressort", "champs A visibles", "Ressort A"),
            ("ar", "قيم النابض", "حقول A ظاهرة", "النابض A"),
        ] {
            let locale = locale(identifier);
            assert_eq!(
                catalogs.resolve(&locale, "lab-identity-spring-a"),
                field_label
            );
            let mirror = catalogs
                .format(
                    &locale,
                    "lab-identity-spring-mirrors",
                    &MessageArguments::new().with("a", "alpha").with("b", "beta"),
                )
                .expect("identity values should format");
            assert!(mirror.contains(mirror_label), "{identifier}: {mirror}");
            assert!(mirror.contains("alpha") && mirror.contains("beta"));
            let status = catalogs
                .format(
                    &locale,
                    "lab-identity-visible",
                    &MessageArguments::new().with("count", 12_i64),
                )
                .expect("identity status should format");
            assert!(status.contains(visible_label), "{identifier}: {status}");
            assert!(
                status.contains(&format_integer(12, &locale)),
                "{identifier}: {status}"
            );
        }
    }

    #[test]
    fn save_dialog_status_formats_in_each_locale() {
        use rustui::localization::MessageArguments;

        let catalogs = catalogs();
        for (identifier, expected) in [
            ("en", "Saved Notes as PDF."),
            ("fr", "Notes enregistré au format PDF."),
            ("ar", "حُفظ Notes بتنسيق PDF."),
        ] {
            let locale = locale(identifier);
            let status = catalogs
                .format(
                    &locale,
                    "lab-save-dialog-saved-pdf",
                    &MessageArguments::new().with("name", "Notes"),
                )
                .expect("dialog status should format");
            assert_eq!(status, expected);
        }
    }

    #[test]
    fn interpolation_story_messages_exist_in_every_playground_catalog() {
        let catalogs = catalogs();
        for identifier in ["en", "fr", "ar"] {
            let catalog = catalogs
                .get(&locale(identifier))
                .expect("Playground locale should be registered");
            for message in [
                "lab-i18n-title",
                "lab-i18n-description",
                "lab-i18n-sample",
                "lab-i18n-plural",
                "lab-i18n-select",
                "lab-i18n-malformed",
                "lab-i18n-error-label",
                "lab-tab-overview-accessibility",
            ] {
                assert!(
                    catalog.get(message).is_some(),
                    "{message} missing in {identifier}"
                );
            }
        }
    }
}
