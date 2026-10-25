//! Cross-catalog validation against the configured fallback catalog.

use std::collections::{BTreeMap, BTreeSet};

use crate::{Catalogs, Locale, MessageId, formatting::message_contract};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CatalogValidation {
    pub fallback_catalog_missing: bool,
    pub missing: BTreeMap<Locale, Vec<MessageId>>,
    pub extra: BTreeMap<Locale, Vec<MessageId>>,
    /// Messages whose placeholder names differ from the fallback translation.
    pub argument_mismatches: BTreeMap<Locale, Vec<MessageId>>,
    /// Messages whose plural, selectordinal, or select expression structure differs from fallback.
    pub expression_mismatches: BTreeMap<Locale, Vec<MessageId>>,
    /// Messages that do not follow the supported escaped-brace placeholder syntax.
    pub invalid_patterns: BTreeMap<Locale, Vec<MessageId>>,
}

impl CatalogValidation {
    pub fn is_valid(&self) -> bool {
        !self.fallback_catalog_missing
            && self.missing.is_empty()
            && self.extra.is_empty()
            && self.argument_mismatches.is_empty()
            && self.expression_mismatches.is_empty()
            && self.invalid_patterns.is_empty()
    }
}

pub fn validate_catalogs(catalogs: &Catalogs) -> CatalogValidation {
    let Some(reference) = catalogs.get(catalogs.fallback()) else {
        return CatalogValidation {
            fallback_catalog_missing: true,
            ..CatalogValidation::default()
        };
    };
    let expected = reference.message_ids().cloned().collect::<BTreeSet<_>>();
    let mut validation = CatalogValidation::default();

    for (locale, catalog) in catalogs.iter() {
        let actual = catalog.message_ids().cloned().collect::<BTreeSet<_>>();
        let missing = expected.difference(&actual).cloned().collect::<Vec<_>>();
        let extra = actual.difference(&expected).cloned().collect::<Vec<_>>();
        if !missing.is_empty() {
            validation.missing.insert(locale.clone(), missing);
        }
        if !extra.is_empty() {
            validation.extra.insert(locale.clone(), extra);
        }

        let mut argument_mismatches = Vec::new();
        let mut expression_mismatches = Vec::new();
        let mut invalid_patterns = Vec::new();
        for identifier in expected.intersection(&actual) {
            let expected_contract = reference
                .get(identifier.as_str())
                .and_then(|value| message_contract(value).ok());
            let actual_contract = catalog
                .get(identifier.as_str())
                .and_then(|value| message_contract(value).ok());
            match (expected_contract, actual_contract) {
                (Some(expected), Some(actual)) => {
                    if !expected.arguments_match(&actual) {
                        argument_mismatches.push(identifier.clone());
                    }
                    if !expected.matches(&actual) {
                        expression_mismatches.push(identifier.clone());
                    }
                }
                _ => invalid_patterns.push(identifier.clone()),
            }
        }
        for identifier in actual.difference(&expected) {
            if catalog
                .get(identifier.as_str())
                .and_then(|value| message_contract(value).ok())
                .is_none()
            {
                invalid_patterns.push(identifier.clone());
            }
        }
        if !expression_mismatches.is_empty() {
            validation
                .expression_mismatches
                .insert(locale.clone(), expression_mismatches);
        }
        if !argument_mismatches.is_empty() {
            validation
                .argument_mismatches
                .insert(locale.clone(), argument_mismatches);
        }
        if !invalid_patterns.is_empty() {
            validation
                .invalid_patterns
                .insert(locale.clone(), invalid_patterns);
        }
    }

    validation
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Catalog;

    fn locale(value: &str) -> Locale {
        Locale::parse(value).unwrap()
    }

    fn insert(catalogs: &mut Catalogs, locale_name: &str, source: &str) {
        catalogs
            .insert(Catalog::parse(locale(locale_name), source).unwrap())
            .unwrap();
    }

    #[test]
    fn validation_reports_translations_with_different_argument_names() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(&mut catalogs, "en", "greeting = Hello {name}, {count}!");
        insert(&mut catalogs, "fr", "greeting = Bonjour {nom}!");

        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.argument_mismatches[&locale("fr")],
            [MessageId::parse("greeting").unwrap()]
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn validation_treats_repeated_placeholder_names_as_a_set() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(&mut catalogs, "en", "greeting = Hello {name}, then {name}!");
        insert(&mut catalogs, "fr", "greeting = Bonjour {name}!");

        let report = validate_catalogs(&catalogs);
        assert!(report.argument_mismatches.is_empty());
        assert!(report.is_valid());
    }

    #[test]
    fn validation_ignores_escaped_braces_and_placeholder_order() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(&mut catalogs, "en", "sample = {{literal}} {name} / {count}");
        insert(
            &mut catalogs,
            "fr",
            "sample = {count} et {name} ({{texte}})",
        );

        let report = validate_catalogs(&catalogs);
        assert!(report.argument_mismatches.is_empty());
        assert!(report.is_valid());
    }

    #[test]
    fn validation_reports_malformed_placeholders() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(&mut catalogs, "en", "sample = Hello {name}");
        insert(&mut catalogs, "fr", "sample = Bonjour {nom");

        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.invalid_patterns[&locale("fr")],
            [MessageId::parse("sample").unwrap()]
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn validation_reports_malformed_extra_messages() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(&mut catalogs, "en", "greeting = Hello");
        insert(&mut catalogs, "fr", "greeting = Bonjour\nextra = {name");

        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.extra[&locale("fr")],
            [MessageId::parse("extra").unwrap()]
        );
        assert_eq!(
            report.invalid_patterns[&locale("fr")],
            [MessageId::parse("extra").unwrap()]
        );
        assert!(!report.is_valid());
    }
}

#[cfg(test)]
mod expression_contract_tests {
    use super::*;
    use crate::{Catalog, MessageArguments, interpolate};

    fn locale(value: &str) -> Locale {
        Locale::parse(value).unwrap()
    }
    fn insert(catalogs: &mut Catalogs, tag: &str, source: &str) {
        catalogs
            .insert(Catalog::parse(locale(tag), source).unwrap())
            .unwrap();
    }

    #[test]
    fn validates_expression_kind_select_keys_and_exact_number_parity() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "items = {count, plural, one {one} =7 {seven} other {many}}\nwho = {gender, select, female {she} other {they}}",
        );
        insert(
            &mut catalogs,
            "fr",
            "items = {count, plural, one {un} =7 {sept} other {plusieurs}}\nwho = {gender, select, female {elle} other {iel}}",
        );
        let report = validate_catalogs(&catalogs);
        assert!(report.is_valid());

        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "items = {count, plural, one {one} =7 {seven} other {many}}",
        );
        insert(
            &mut catalogs,
            "fr",
            "items = {count, plural, one {un} =8 {huit} other {plusieurs}}",
        );
        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.expression_mismatches[&locale("fr")],
            [MessageId::parse("items").unwrap()]
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn detects_placeholders_moved_between_expression_branches() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "sample = {count, plural, one {{name}} other {{count}}}",
        );
        insert(
            &mut catalogs,
            "fr",
            "sample = {count, plural, one {{count}} other {{name}}}",
        );

        let report = validate_catalogs(&catalogs);
        assert!(report.argument_mismatches.is_empty());
        assert_eq!(
            report.expression_mismatches[&locale("fr")],
            [MessageId::parse("sample").unwrap()]
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn repeated_select_sites_keep_their_own_branch_keys() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "sample = {gender, select, female {she} other {they}} then {gender, select, male {he} other {they}}",
        );
        insert(
            &mut catalogs,
            "fr",
            "sample = {gender, select, male {il} other {iel}} puis {gender, select, female {elle} other {iel}}",
        );

        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.expression_mismatches[&locale("fr")],
            [MessageId::parse("sample").unwrap()]
        );
        assert_eq!(
            interpolate(
                "{gender, select, male {il} other {iel}} puis {gender, select, female {elle} other {iel}}",
                &MessageArguments::new().with("gender", "female"),
            )
            .unwrap(),
            "iel puis elle"
        );
    }

    #[test]
    fn nested_select_sites_in_shared_plural_categories_keep_their_own_keys() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "sample = {count, plural, one {{gender, select, female {one female} other {one other}}} other {{gender, select, male {many male} other {many other}}}}",
        );
        insert(
            &mut catalogs,
            "fr",
            "sample = {count, plural, one {{gender, select, male {un homme} other {un autre}}} other {{gender, select, female {des femmes} other {d'autres}}}}",
        );

        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.expression_mismatches[&locale("fr")],
            [MessageId::parse("sample").unwrap()]
        );
    }

    #[test]
    fn extra_plural_categories_may_repeat_existing_nested_contracts() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "sample = {count, plural, one {{gender, select, female {one female} other {one other}}} other {{gender, select, male {many male} other {many other}}}}",
        );
        insert(
            &mut catalogs,
            "ar",
            "sample = {count, plural, zero {{gender, select, female {zero female} other {zero other}}} one {{gender, select, female {one female} other {one other}}} two {{gender, select, male {two male} other {two other}}} few {{gender, select, female {few female} other {few other}}} many {{gender, select, male {many male} other {many other}}} other {{gender, select, male {other male} other {other text}}}}",
        );

        let report = validate_catalogs(&catalogs);
        assert!(report.is_valid(), "{report:#?}");
    }

    #[test]
    fn extra_plural_category_cannot_introduce_a_new_nested_contract() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "sample = {count, plural, one {{gender, select, female {one female} other {one other}}} other {{gender, select, male {many male} other {many other}}}}",
        );
        insert(
            &mut catalogs,
            "ar",
            "sample = {count, plural, zero {{gender, select, neutral {zero neutral} other {zero other}}} one {{gender, select, female {one female} other {one other}}} other {{gender, select, male {many male} other {many other}}}}",
        );

        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.expression_mismatches[&locale("ar")],
            [MessageId::parse("sample").unwrap()]
        );
    }

    #[test]
    fn malformed_expression_is_reported_as_invalid_pattern() {
        let mut catalogs = Catalogs::new(locale("en"));
        insert(
            &mut catalogs,
            "en",
            "items = {count, plural, one {one} other {many}}",
        );
        insert(
            &mut catalogs,
            "fr",
            "items = {count, plural, un {un} autre {plusieurs}}",
        );
        let report = validate_catalogs(&catalogs);
        assert_eq!(
            report.invalid_patterns[&locale("fr")],
            [MessageId::parse("items").unwrap()]
        );
        assert!(!report.is_valid());
    }
}
