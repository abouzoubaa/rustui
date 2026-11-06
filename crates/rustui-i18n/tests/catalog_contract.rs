use std::collections::BTreeSet;

use rustui_i18n::{
    Catalog, Catalogs, Locale, MessageArguments, interpolate_with_locale, validate_catalogs,
};

const CATALOGS: &[&str] = &[
    include_str!("../../../locales/en/rustui.ftl"),
    include_str!("../../../locales/fr/rustui.ftl"),
    include_str!("../../../locales/ar/rustui.ftl"),
];

fn message_ids(catalog: &str) -> BTreeSet<&str> {
    catalog
        .lines()
        .filter_map(|line| line.split_once(" = ").map(|(id, _)| id))
        .collect()
}

#[test]
fn framework_catalogs_define_the_same_message_ids() {
    let expected = message_ids(CATALOGS[0]);
    assert!(!expected.is_empty());

    for catalog in &CATALOGS[1..] {
        assert_eq!(message_ids(catalog), expected);
    }
}

#[test]
fn framework_catalogs_match_the_supported_interpolation_contract() {
    let fallback = Locale::parse("en").unwrap();
    let mut catalogs = Catalogs::new(fallback);
    for (locale, source) in [
        ("en", CATALOGS[0]),
        ("fr", CATALOGS[1]),
        ("ar", CATALOGS[2]),
    ] {
        catalogs
            .insert(Catalog::parse(Locale::parse(locale).unwrap(), source).unwrap())
            .unwrap();
    }

    let validation = validate_catalogs(&catalogs);
    assert!(
        validation.is_valid(),
        "framework locale catalogs violate the supported message contract: {validation:#?}"
    );
}

#[test]
fn narrow_navigation_open_labels_are_localized_and_name_the_destination() {
    let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
    let cases = [
        (
            "en",
            CATALOGS[0],
            "Open selected destination",
            "Open Settings",
        ),
        (
            "fr",
            CATALOGS[1],
            "Ouvrir la destination sélectionnée",
            "Ouvrir Settings",
        ),
        ("ar", CATALOGS[2], "افتح الوجهة المحددة", "افتح Settings"),
    ];

    for (locale, source, expected_action, expected_accessibility) in cases {
        let locale = Locale::parse(locale).unwrap();
        catalogs
            .insert(Catalog::parse(locale.clone(), source).unwrap())
            .unwrap();
        assert_eq!(
            catalogs.resolve(&locale, "rustui-navigation-open-selected-destination"),
            expected_action
        );
        assert_eq!(
            interpolate_with_locale(
                catalogs.resolve(&locale, "rustui-navigation-open-destination"),
                &MessageArguments::new().with("destination", "Settings"),
                &locale,
            )
            .unwrap(),
            expected_accessibility
        );
    }
}

#[test]
fn navigation_tree_accessibility_name_is_localized() {
    let cases = [
        ("en", CATALOGS[0], "Navigation"),
        ("fr", CATALOGS[1], "Navigation"),
        ("ar", CATALOGS[2], "التنقل"),
    ];

    let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
    for (locale, source, expected) in cases {
        let locale = Locale::parse(locale).unwrap();
        catalogs
            .insert(Catalog::parse(locale.clone(), source).unwrap())
            .unwrap();
        assert_eq!(
            catalogs.resolve(&locale, "rustui-navigation-tree"),
            expected
        );
    }
}

#[test]
fn semantics_gallery_region_accessibility_name_is_localized() {
    let cases = [
        ("en", CATALOGS[0], "Semantics example region"),
        ("fr", CATALOGS[1], "Région d’exemple sémantique"),
        ("ar", CATALOGS[2], "منطقة مثال الدلالات"),
    ];

    let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
    for (locale, source, expected) in cases {
        let locale = Locale::parse(locale).unwrap();
        catalogs
            .insert(Catalog::parse(locale.clone(), source).unwrap())
            .unwrap();
        assert_eq!(
            catalogs.resolve(&locale, "rustui-semantics-example-region"),
            expected
        );
    }
}

#[test]
fn tab_group_accessibility_name_is_localized_in_supported_catalogs() {
    let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
    for (locale, source) in [
        ("en", CATALOGS[0]),
        ("fr", CATALOGS[1]),
        ("ar", CATALOGS[2]),
    ] {
        catalogs
            .insert(Catalog::parse(Locale::parse(locale).unwrap(), source).unwrap())
            .unwrap();
    }

    for (locale, expected) in [("en", "Tabs"), ("fr", "Onglets"), ("ar", "علامات التبويب")]
    {
        assert_eq!(
            catalogs.resolve(&Locale::parse(locale).unwrap(), "rustui-navigation-tabs"),
            expected
        );
    }
}

#[test]
fn split_view_divider_accessibility_name_is_localized_in_supported_catalogs() {
    let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
    for (locale, source) in [
        ("en", CATALOGS[0]),
        ("fr", CATALOGS[1]),
        ("ar", CATALOGS[2]),
    ] {
        catalogs
            .insert(Catalog::parse(Locale::parse(locale).unwrap(), source).unwrap())
            .unwrap();
    }

    for (locale, expected) in [
        ("en", "Resize divider"),
        ("fr", "Redimensionner le séparateur"),
        ("ar", "تغيير حجم الفاصل"),
    ] {
        assert_eq!(
            catalogs.resolve(
                &Locale::parse(locale).unwrap(),
                "rustui-splitter-resize-divider"
            ),
            expected
        );
    }
}

#[test]
fn text_field_clear_button_accessibility_name_is_localized_in_supported_catalogs() {
    let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
    for (locale, source) in [
        ("en", CATALOGS[0]),
        ("fr", CATALOGS[1]),
        ("ar", CATALOGS[2]),
    ] {
        catalogs
            .insert(Catalog::parse(Locale::parse(locale).unwrap(), source).unwrap())
            .unwrap();
    }

    for (locale, expected) in [
        ("en", "Clear text"),
        ("fr", "Effacer le texte"),
        ("ar", "مسح النص"),
    ] {
        assert_eq!(
            catalogs.resolve(
                &Locale::parse(locale).unwrap(),
                "rustui-text-field-clear-button"
            ),
            expected
        );
    }
}

#[test]
fn link_accessible_name_is_localized_in_supported_catalogs() {
    let mut catalogs = Catalogs::new(Locale::parse("en").unwrap());
    for (locale, source) in [
        ("en", CATALOGS[0]),
        ("fr", CATALOGS[1]),
        ("ar", CATALOGS[2]),
    ] {
        catalogs
            .insert(Catalog::parse(Locale::parse(locale).unwrap(), source).unwrap())
            .unwrap();
    }

    for (locale, expected) in [
        ("en", "Open link fixture"),
        ("fr", "Ouvrir le lien de démonstration"),
        ("ar", "افتح رابط المثال"),
    ] {
        assert_eq!(
            catalogs.resolve(&Locale::parse(locale).unwrap(), "rustui-link-open-fixture"),
            expected
        );
    }
}
