use rustui_i18n::{
    Catalog, Catalogs, CivilDate, CivilDateTime, CivilTime, DateLength, FluentFormatError,
    InterpolationError, Locale, LocaleFormatter, MessageArguments, TimeFields, validate_catalogs,
};
fn locale(tag: &str) -> Locale {
    Locale::parse(tag).unwrap()
}
fn catalogs(locale_tag: &str, source: &str) -> Catalogs {
    let locale = locale(locale_tag);
    let mut catalogs = Catalogs::new(locale.clone());
    catalogs
        .insert(Catalog::parse_fluent(locale, source).unwrap())
        .unwrap();
    catalogs
}
fn visible(value: String) -> String {
    value.replace(['\u{2068}', '\u{2069}'], "")
}
#[test]
fn fluent_terms_references_attributes_selectors_and_default_number_work() {
    let catalogs = catalogs(
        "en-US",
        r#"
-brand = RustUI
welcome = Welcome to { -brand }, { $name }!
button = Open
    .aria-label = Open { -brand }
reference = { button }
count = { $count ->
    [one] One item
   *[other] { NUMBER($count) } items
}
rank = { NUMBER($rank, type: "ordinal") ->
    [one] { $rank }st
    [two] { $rank }nd
    [few] { $rank }rd
   *[other] { $rank }th
}
"#,
    );
    let locale = locale("en-US");
    let arguments = MessageArguments::new()
        .with("name", "Ada")
        .with("count", 1234)
        .with("rank", 22);
    assert_eq!(
        visible(catalogs.format(&locale, "welcome", &arguments).unwrap()),
        "Welcome to RustUI, Ada!"
    );
    assert_eq!(
        visible(
            catalogs
                .format(&locale, "button.aria-label", &arguments)
                .unwrap()
        ),
        "Open RustUI"
    );
    assert_eq!(
        visible(catalogs.format(&locale, "reference", &arguments).unwrap()),
        "Open"
    );
    assert_eq!(
        visible(catalogs.format(&locale, "count", &arguments).unwrap()),
        "1,234 items"
    );
    assert_eq!(
        visible(catalogs.format(&locale, "rank", &arguments).unwrap()),
        "22nd"
    );
    assert_eq!(
        catalogs
            .format(&locale, "count", &MessageArguments::new().with("count", 1))
            .unwrap(),
        "One item"
    );
}
#[test]
fn fluent_keeps_isolation_and_exposes_evaluation_failures() {
    let catalogs = catalogs(
        "en",
        "hello = Hello { $name }!\nbroken = { missing }\ncycle = { cycle }",
    );
    let locale = locale("en");
    assert_eq!(
        catalogs
            .format(
                &locale,
                "hello",
                &MessageArguments::new().with("name", "Ada")
            )
            .unwrap(),
        "Hello \u{2068}Ada\u{2069}!"
    );
    for id in ["hello", "broken", "cycle"] {
        assert!(matches!(
            catalogs.format(&locale, id, &MessageArguments::new()),
            Err(InterpolationError::Fluent(FluentFormatError::Evaluation(_)))
        ));
    }
}
#[test]
fn unsupported_options_and_functions_are_visible_structured_errors() {
    for option in [
        "maximumFractionDigits: 2",
        "style: \"currency\"",
        "useGrouping: \"false\"",
        "minimumIntegerDigits: 3",
        "unknown: 1",
        "type: \"other\"",
    ] {
        assert!(matches!(
            Catalog::parse_fluent(
                locale("en"),
                &format!("value = {{ NUMBER($value, {option}) }}")
            ),
            Err(FluentFormatError::UnsupportedNumberOption(_))
        ));
    }
    assert!(
        matches!(Catalog::parse_fluent(locale("en"), "date = { DATETIME($date) }"), Err(FluentFormatError::UnsupportedFunction(name)) if name == "DATETIME")
    );
    assert!(matches!(
        Catalog::parse_fluent(locale("en"), "bad = {"),
        Err(FluentFormatError::InvalidResource(_))
    ));
    assert!(matches!(
        Catalog::parse_fluent(locale("en"), "x = First\nx = Second"),
        Err(FluentFormatError::InvalidResource(_))
    ));
}
#[test]
fn fluent_preserves_catalog_fallback_and_never_leaks_source_through_borrowed_lookup() {
    let mut catalogs = catalogs("en", "shared = English\nonly-en = English fallback");
    catalogs
        .insert(Catalog::parse_fluent(locale("fr"), "shared = Français").unwrap())
        .unwrap();
    catalogs
        .insert(Catalog::parse_fluent(locale("fr-CA"), "shared = Québec").unwrap())
        .unwrap();
    let args = MessageArguments::new();
    assert_eq!(
        catalogs
            .format(&locale("fr-CA-u-nu-latn"), "shared", &args)
            .unwrap(),
        "Québec"
    );
    assert_eq!(
        catalogs.format(&locale("fr-BE"), "shared", &args).unwrap(),
        "Français"
    );
    assert_eq!(
        catalogs.format(&locale("fr-CA"), "only-en", &args).unwrap(),
        "English fallback"
    );
    assert_eq!(
        catalogs.format(&locale("de"), "missing", &args).unwrap(),
        "missing"
    );
    assert_eq!(catalogs.resolve(&locale("fr"), "shared"), "shared");
    assert_eq!(catalogs.get(&locale("en")).unwrap().get("only-en"), None);
}
#[test]
fn legacy_catalog_and_interpolation_keep_their_original_meaning() {
    let mut catalogs = Catalogs::new(locale("en"));
    let source = "app.title = Legacy {name}\nvalue = {{literal}} {count}";
    catalogs
        .insert(Catalog::parse(locale("en"), source).unwrap())
        .unwrap();
    let args = MessageArguments::new()
        .with("name", "Ada")
        .with("count", 1234);
    assert_eq!(
        catalogs.resolve(&locale("fr"), "app.title"),
        "Legacy {name}"
    );
    assert_eq!(
        catalogs.format(&locale("fr"), "app.title", &args).unwrap(),
        "Legacy Ada"
    );
    assert_eq!(
        catalogs.format(&locale("en"), "value", &args).unwrap(),
        "{literal} 1,234"
    );
    assert!(validate_catalogs(&catalogs).is_valid());
}
#[test]
fn fluent_contract_validation_compares_referenced_arguments_and_attributes() {
    let mut catalogs = catalogs(
        "en",
        "name = Hello { $name }\nwelcome = { name }\nbutton = Open\n    .title = Open { $target }",
    );
    catalogs.insert(Catalog::parse_fluent(locale("fr"), "name = Bonjour { $nom }\nwelcome = { name }\nbutton = Ouvrir\n    .title = Ouvrir { $target }").unwrap()).unwrap();
    let validation = validate_catalogs(&catalogs);
    let mismatch = validation.argument_mismatches[&locale("fr")]
        .iter()
        .map(|id| id.as_str())
        .collect::<Vec<_>>();
    assert_eq!(mismatch, ["name", "welcome"]);
}

#[test]
fn fluent_validation_rejects_missing_and_recursive_terms() {
    for source in [
        "welcome = { -missing }",
        "-first = { -second }\n-second = { -first }\nwelcome = { -first }",
    ] {
        let report = validate_catalogs(&catalogs("en", source));
        assert!(
            report.invalid_patterns[&locale("en")]
                .iter()
                .any(|id| id.as_str() == "welcome")
        );
    }
}

#[test]
fn fluent_validation_keeps_term_variables_local() {
    let mut catalogs = catalogs(
        "en",
        "-brand = { $case }\nwelcome = { -brand(case: \"formal\") } { $name }",
    );
    catalogs
        .insert(
            Catalog::parse_fluent(
                locale("fr"),
                "-brand = { $style }\nwelcome = { -brand(style: \"formel\") } { $name }",
            )
            .unwrap(),
        )
        .unwrap();
    assert!(validate_catalogs(&catalogs).is_valid());
}
#[test]
fn icu_decimal_keeps_exact_digits_grouping_and_unicode_number_preferences() {
    let english = LocaleFormatter::try_new(&locale("en-US")).unwrap();
    assert_eq!(english.format_integer(1_000_007), "1,000,007");
    assert_eq!(
        english.format_decimal("9007199254740993.50").unwrap(),
        "9,007,199,254,740,993.50"
    );
    assert_eq!(
        LocaleFormatter::try_new(&locale("fr-FR"))
            .unwrap()
            .format_decimal("1234.50")
            .unwrap(),
        "1\u{202f}234,50"
    );
    assert_eq!(
        LocaleFormatter::try_new(&locale("en-u-nu-thai"))
            .unwrap()
            .format_decimal("1234.50")
            .unwrap(),
        "\u{0e51},\u{0e52}\u{0e53}\u{0e54}.\u{0e55}\u{0e50}"
    );
    assert!(english.format_decimal("NaN").is_err());
    assert!(english.format_decimal("1,234").is_err());
}
#[test]
fn preformatted_strings_keep_full_preferences_while_fluent_uses_resource_locale() {
    let catalogs = catalogs("en", "amount = { NUMBER($amount) }\nexact = { $formatted }");
    let active_locale = locale("en-u-nu-thai");
    let args = MessageArguments::new().with("amount", 1234).with(
        "formatted",
        LocaleFormatter::try_new(&active_locale)
            .unwrap()
            .format_integer(1234),
    );
    assert_eq!(
        visible(catalogs.format(&active_locale, "amount", &args).unwrap()),
        "1,234"
    );
    assert_eq!(
        visible(catalogs.format(&active_locale, "exact", &args).unwrap()),
        "\u{0e51},\u{0e52}\u{0e53}\u{0e54}"
    );
    assert!(matches!(
        catalogs.format(
            &active_locale,
            "amount",
            &MessageArguments::new().with("amount", i64::MAX)
        ),
        Err(InterpolationError::Fluent(
            FluentFormatError::InvalidArgument { .. }
        ))
    ));
}
#[test]
fn icu_date_time_validates_civil_input_and_applies_calendar_preferences() {
    let date = CivilDate {
        year: 2025,
        month: 1,
        day: 15,
    };
    let time = CivilTime {
        hour: 16,
        minute: 9,
        second: 35,
    };
    let formatter = LocaleFormatter::try_new(&locale("en-GB-u-hc-h23")).unwrap();
    assert_eq!(
        formatter.format_time(time, TimeFields::HourMinute).unwrap(),
        "16:09"
    );
    assert_eq!(
        formatter
            .format_time(time, TimeFields::HourMinuteSecond)
            .unwrap(),
        "16:09:35"
    );
    for length in [DateLength::Short, DateLength::Medium, DateLength::Long] {
        assert!(!formatter.format_date(date, length).unwrap().is_empty());
        assert!(
            formatter
                .format_datetime(CivilDateTime { date, time }, length, TimeFields::HourMinute)
                .unwrap()
                .contains("16:09")
        );
    }
    let buddhist = LocaleFormatter::try_new(&locale("en-u-ca-buddhist"))
        .unwrap()
        .format_date(date, DateLength::Medium)
        .unwrap();
    assert!(buddhist.contains("2568"), "{buddhist}");
    assert!(
        formatter
            .format_date(
                CivilDate {
                    year: 2025,
                    month: 2,
                    day: 30
                },
                DateLength::Medium
            )
            .is_err()
    );
    assert!(
        formatter
            .format_time(
                CivilTime {
                    hour: 25,
                    minute: 0,
                    second: 0
                },
                TimeFields::HourMinute
            )
            .is_err()
    );
    assert!(LocaleFormatter::try_new(&locale("en-u-ca-unknown")).is_err());
}

#[test]
fn fluent_text_selectors_term_parameters_and_russian_categories_use_evaluator() {
    let catalogs = catalogs(
        "ru",
        r#"
-brand = { $case ->
    [object] RustUI-object
   *[other] RustUI
}
label = { $mode ->
    [formal] Formal { -brand(case: "object") }
   *[other] Informal { -brand }
}
items = { $count ->
    [one] one
    [few] few
    [many] many
   *[other] other
}
"#,
    );
    let locale = locale("ru");
    assert_eq!(
        visible(
            catalogs
                .format(
                    &locale,
                    "label",
                    &MessageArguments::new().with("mode", "formal")
                )
                .unwrap()
        ),
        "Formal RustUI-object"
    );
    for (count, expected) in [
        (1, "one"),
        (2, "few"),
        (5, "many"),
        (11, "many"),
        (21, "one"),
    ] {
        assert_eq!(
            catalogs
                .format(
                    &locale,
                    "items",
                    &MessageArguments::new().with("count", count)
                )
                .unwrap(),
            expected
        );
    }
}
