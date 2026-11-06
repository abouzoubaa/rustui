//! Public RustUI example for named catalog-message interpolation.

use rustui::{
    localization::{
        Catalog, Catalogs, CivilDate, DateLength, Locale, LocaleFormatter, MessageArguments,
    },
    prelude::{LocalizedText, Text, VStack, View},
};

fn runtime_example(tag: &str) -> Result<String, Box<dyn std::error::Error>> {
    let locale = Locale::parse(tag)?;
    let formatter = LocaleFormatter::try_new(&locale)?;
    let mut catalogs = Catalogs::new(locale.clone());
    catalogs.insert(Catalog::parse_fluent(
        locale.clone(),
        "-brand = RustUI\nsample = { -brand } · { $name } · { NUMBER($count) }",
    )?)?;
    let message = catalogs.format(
        &locale,
        "sample",
        &MessageArguments::new()
            .with("name", "Ada")
            .with("count", 3_i32),
    )?;
    let decimal = formatter.format_decimal("1234.50")?;
    let date = formatter.format_date(
        CivilDate {
            year: 2026,
            month: 10,
            day: 25,
        },
        DateLength::Long,
    )?;
    Ok(format!("{tag}: {message} · {decimal} · {date}"))
}

pub fn demo() -> impl View {
    let arguments = MessageArguments::new()
        .with("name", "Ada")
        .with("count", 3_i32);
    let plural_arguments = MessageArguments::new().with("count", 3_i32);
    let decimal_plural_arguments = MessageArguments::new().with("count", 1.5_f64);
    let ordinal_arguments = MessageArguments::new().with("count", 3_i32);
    let select_arguments = MessageArguments::new().with("gender", "female");
    let missing_arguments = MessageArguments::new().with("name", "Ada");

    let mut content = VStack::new()
        .child(LocalizedText("lab-i18n-title"))
        .child(LocalizedText("lab-i18n-description"))
        .child(LocalizedText("lab-i18n-sample").with_arguments(arguments))
        .child(LocalizedText("lab-i18n-plural").with_arguments(plural_arguments))
        .child(LocalizedText("lab-i18n-decimal-plural").with_arguments(decimal_plural_arguments))
        .child(LocalizedText("lab-i18n-ordinal").with_arguments(ordinal_arguments))
        .child(LocalizedText("lab-i18n-select").with_arguments(select_arguments))
        .child(LocalizedText("lab-i18n-error-label"))
        .child(LocalizedText("lab-i18n-malformed").with_arguments(missing_arguments))
        .child(LocalizedText("lab-i18n-runtime-examples"));
    for tag in ["en-US", "fr-FR", "ar-EG"] {
        content = content.child(Text(
            runtime_example(tag).unwrap_or_else(|error| error.to_string()),
        ));
    }
    content
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustui::localization::{interpolate_with_locale, validate_catalogs};

    #[test]
    fn runtime_examples_use_fluent_and_icu_for_each_displayed_locale() {
        for (tag, decimal) in [
            ("en-US", "1,234.50"),
            ("fr-FR", "1\u{202f}234,50"),
            ("ar-EG", "١٬٢٣٤٫٥٠"),
        ] {
            let sample = runtime_example(tag).unwrap();
            assert!(sample.contains("RustUI"), "{sample}");
            assert!(sample.contains("Ada"), "{sample}");
            assert!(sample.contains(decimal), "{sample}");
            assert!(sample.starts_with(tag));
        }
    }

    #[test]
    fn decimal_plural_gallery_message_is_complete_and_uses_locale_rules() {
        let catalogs = super::super::locale_selector::catalogs();
        let validation = validate_catalogs(&catalogs);
        assert!(validation.missing.is_empty());
        assert!(validation.extra.is_empty());
        assert!(validation.argument_mismatches.is_empty());
        assert!(validation.expression_mismatches.is_empty());

        let arguments = MessageArguments::new().with("count", 1.5_f64);
        for (identifier, expected) in [("en", "1.5 notes"), ("fr", "1,5 note"), ("ar", "١٫٥ ملاحظة")]
        {
            let locale = Locale::parse(identifier).expect("Playground locale should be valid");
            let catalog = catalogs
                .get(&locale)
                .expect("Playground locale should be registered");
            let message = catalog
                .get("lab-i18n-decimal-plural")
                .expect("decimal plural message should exist in every locale");
            assert_eq!(
                interpolate_with_locale(message, &arguments, &locale),
                Ok(expected.to_owned()),
                "unexpected decimal plural result for {identifier}"
            );
        }
    }

    #[test]
    fn ordinal_gallery_message_is_complete_and_uses_locale_rules() {
        let catalogs = super::super::locale_selector::catalogs();
        let validation = validate_catalogs(&catalogs);
        assert!(validation.missing.is_empty());
        assert!(validation.extra.is_empty());
        assert!(validation.argument_mismatches.is_empty());
        assert!(validation.expression_mismatches.is_empty());

        let arguments = MessageArguments::new().with("count", 3_i32);
        for (identifier, expected) in [("en", "3rd"), ("fr", "3e"), ("ar", "الترتيب ٣")] {
            let locale = Locale::parse(identifier).expect("Playground locale should be valid");
            let catalog = catalogs
                .get(&locale)
                .expect("Playground locale should be registered");
            let message = catalog
                .get("lab-i18n-ordinal")
                .expect("ordinal message should exist in every locale");
            assert_eq!(
                interpolate_with_locale(message, &arguments, &locale),
                Ok(expected.to_owned()),
                "unexpected ordinal result for {identifier}"
            );
        }
    }
}
