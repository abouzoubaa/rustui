//! Parser and lookup for RustUI's initial Fluent-compatible catalog subset.

use std::{collections::BTreeMap, error::Error, fmt, sync::Arc};

use crate::{Locale, MessageId};

#[derive(Clone, Debug)]
pub struct Catalog {
    locale: Locale,
    messages: BTreeMap<MessageId, String>,
    fluent: Option<Arc<crate::fluent::FluentCatalog>>,
}

impl Catalog {
    pub fn parse(locale: Locale, source: &str) -> Result<Self, CatalogError> {
        let mut messages = BTreeMap::new();
        for (index, source_line) in source.lines().enumerate() {
            let line_number = index + 1;
            let line = source_line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((identifier, value)) = line.split_once('=') else {
                return Err(CatalogError::new(
                    locale.clone(),
                    line_number,
                    "expected `message-id = value`",
                ));
            };
            let identifier = MessageId::parse(identifier).map_err(|error| {
                CatalogError::new(locale.clone(), line_number, error.to_string())
            })?;
            let value = value.trim();
            if value.is_empty() {
                return Err(CatalogError::new(
                    locale.clone(),
                    line_number,
                    format!("message `{identifier}` has an empty value"),
                ));
            }
            if messages
                .insert(identifier.clone(), value.to_owned())
                .is_some()
            {
                return Err(CatalogError::new(
                    locale.clone(),
                    line_number,
                    format!("message `{identifier}` is defined more than once"),
                ));
            }
        }

        Ok(Self {
            locale,
            messages,
            fluent: None,
        })
    }

    /// Parses full Project Fluent syntax. Legacy `parse` retains its original
    /// placeholder grammar. Fluent values are evaluated through `Catalogs::format`;
    /// borrowed `get`/`resolve` never expose unevaluated FTL patterns.
    pub fn parse_fluent(locale: Locale, source: &str) -> Result<Self, crate::FluentFormatError> {
        let fluent = Arc::new(crate::fluent::FluentCatalog::parse(&locale, source)?);
        let messages = fluent.message_ids().map(|id| (id, String::new())).collect();
        Ok(Self {
            locale,
            messages,
            fluent: Some(fluent),
        })
    }

    pub fn is_fluent(&self) -> bool {
        self.fluent.is_some()
    }

    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    pub fn get(&self, identifier: &str) -> Option<&str> {
        if self.is_fluent() {
            return None;
        }
        self.messages.get(identifier).map(String::as_str)
    }

    pub(crate) fn fluent_arguments(
        &self,
        identifier: &str,
    ) -> Option<std::collections::BTreeSet<String>> {
        self.fluent.as_ref()?.arguments(identifier)
    }

    fn contains(&self, identifier: &str) -> bool {
        self.fluent.as_ref().map_or_else(
            || self.messages.contains_key(identifier),
            |fluent| fluent.contains(identifier),
        )
    }

    pub fn message_ids(&self) -> impl Iterator<Item = &MessageId> {
        self.messages.keys()
    }
}

#[derive(Clone, Debug)]
pub struct Catalogs {
    fallback: Locale,
    catalogs: BTreeMap<Locale, Catalog>,
}

impl Catalogs {
    pub fn new(fallback: Locale) -> Self {
        Self {
            fallback,
            catalogs: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, catalog: Catalog) -> Result<(), CatalogError> {
        if self.catalogs.contains_key(catalog.locale()) {
            return Err(CatalogError::new(
                catalog.locale().clone(),
                0,
                "a catalog for this locale is already registered",
            ));
        }
        self.catalogs.insert(catalog.locale().clone(), catalog);
        Ok(())
    }

    pub fn fallback(&self) -> &Locale {
        &self.fallback
    }

    pub fn get(&self, locale: &Locale) -> Option<&Catalog> {
        self.catalogs.get(locale)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Locale, &Catalog)> {
        self.catalogs.iter()
    }

    fn selected_catalog(&self, locale: &Locale, identifier: &str) -> Option<&Catalog> {
        let mut candidates = vec![locale.clone()];
        push_extension_free_fallbacks(&mut candidates, locale);
        candidates.push(self.fallback.clone());
        push_extension_free_fallbacks(&mut candidates, &self.fallback);
        candidates.into_iter().find_map(|candidate| {
            self.catalogs
                .get(&candidate)
                .filter(|catalog| catalog.contains(identifier))
        })
    }

    /// Resolves plain UI text. Legacy catalogs preserve borrowed lookup
    /// semantics, including literal placeholder text; Fluent catalogs evaluate
    /// the selected pattern with an empty argument set.
    pub fn format_message(
        &self,
        locale: &Locale,
        identifier: &str,
    ) -> Result<String, crate::InterpolationError> {
        let Some(catalog) = self.selected_catalog(locale, identifier) else {
            return Ok(identifier.to_owned());
        };
        match &catalog.fluent {
            Some(fluent) => fluent
                .format(identifier, &crate::MessageArguments::new())
                .map_err(crate::InterpolationError::Fluent),
            None => Ok(catalog
                .get(identifier)
                .expect("legacy message exists")
                .to_owned()),
        }
    }

    /// Resolves the locale fallback chain and evaluates the selected catalog's
    /// declared syntax. Fluent errors return no partially formatted output.
    /// Legacy interpolation uses the active locale; Fluent NUMBER uses the
    /// resource locale without Unicode extensions.
    pub fn format(
        &self,
        locale: &Locale,
        identifier: &str,
        arguments: &crate::MessageArguments,
    ) -> Result<String, crate::InterpolationError> {
        match self.selected_catalog(locale, identifier) {
            Some(catalog) => match &catalog.fluent {
                Some(fluent) => fluent
                    .format(identifier, arguments)
                    .map_err(crate::InterpolationError::Fluent),
                None => crate::interpolate_with_locale(
                    catalog.get(identifier).expect("legacy message exists"),
                    arguments,
                    locale,
                ),
            },
            None => crate::interpolate_with_locale(identifier, arguments, locale),
        }
    }

    /// Borrowed legacy lookup. A selected Fluent resource returns the identifier
    /// here; use `format_message` or `format` to obtain evaluated Fluent text.
    pub fn resolve<'a>(&'a self, locale: &Locale, identifier: &'a str) -> &'a str {
        self.selected_catalog(locale, identifier)
            .and_then(|catalog| catalog.get(identifier))
            .unwrap_or(identifier)
    }
}

fn push_extension_free_fallbacks(candidates: &mut Vec<Locale>, locale: &Locale) {
    let mut candidate = locale.without_extensions();
    loop {
        if !candidates.contains(&candidate) {
            candidates.push(candidate.clone());
        }
        if candidate == candidate.primary() {
            break;
        }

        let prefix = candidate
            .as_str()
            .rsplit_once('-')
            .map(|(prefix, _)| prefix)
            .expect("non-primary locale has a trailing subtag");
        candidate = Locale::parse(prefix).expect("prefix of a valid locale remains valid");
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CatalogError {
    locale: Locale,
    line: usize,
    detail: String,
}

impl CatalogError {
    fn new(locale: Locale, line: usize, detail: impl Into<String>) -> Self {
        Self {
            locale,
            line,
            detail: detail.into(),
        }
    }
}

impl fmt::Display for CatalogError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.line == 0 {
            write!(formatter, "catalog `{}`: {}", self.locale, self.detail)
        } else {
            write!(
                formatter,
                "catalog `{}` line {}: {}",
                self.locale, self.line, self.detail
            )
        }
    }
}

impl Error for CatalogError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validate_catalogs;

    fn locale(value: &str) -> Locale {
        Locale::parse(value).unwrap()
    }

    fn catalog(locale: &str, source: &str) -> Catalog {
        Catalog::parse(self::locale(locale), source).unwrap()
    }

    #[test]
    fn parses_comments_whitespace_and_message_ids() {
        let catalog = catalog(
            "en-US",
            "# comment\n\n  greeting =  Hello there  \napp.title=RustUI\n",
        );

        assert_eq!(catalog.locale().as_str(), "en-US");
        assert_eq!(catalog.get("greeting"), Some("Hello there"));
        assert_eq!(catalog.get("app.title"), Some("RustUI"));
        assert_eq!(catalog.get("missing"), None);
        assert_eq!(
            catalog
                .message_ids()
                .map(MessageId::as_str)
                .collect::<Vec<_>>(),
            ["app.title", "greeting"]
        );
    }

    #[test]
    fn parse_errors_report_source_line_and_cause() {
        let malformed =
            Catalog::parse(locale("fr"), "# heading\nok = oui\nnot a pair").unwrap_err();
        assert_eq!(
            malformed.to_string(),
            "catalog `fr` line 3: expected `message-id = value`"
        );

        let invalid_id = Catalog::parse(locale("fr"), "ok = oui\n9bad = non").unwrap_err();
        assert_eq!(
            invalid_id.to_string(),
            "catalog `fr` line 2: invalid message identifier `9bad`"
        );

        let empty = Catalog::parse(locale("fr"), "\nbye =   ").unwrap_err();
        assert_eq!(
            empty.to_string(),
            "catalog `fr` line 2: message `bye` has an empty value"
        );

        let duplicate = Catalog::parse(locale("fr"), "hello = bonjour\nhello = salut").unwrap_err();
        assert_eq!(
            duplicate.to_string(),
            "catalog `fr` line 2: message `hello` is defined more than once"
        );
    }

    #[test]
    fn duplicate_locale_registration_is_rejected() {
        let mut catalogs = Catalogs::new(locale("en"));
        catalogs.insert(catalog("fr", "hello = bonjour")).unwrap();
        let error = catalogs.insert(catalog("fr", "hello = salut")).unwrap_err();

        assert_eq!(
            error.to_string(),
            "catalog `fr`: a catalog for this locale is already registered"
        );
        assert_eq!(
            catalogs.get(&locale("fr")).unwrap().get("hello"),
            Some("bonjour")
        );
    }

    #[test]
    fn resolve_uses_full_locale_then_primary_then_fallbacks() {
        let mut catalogs = Catalogs::new(locale("en-GB"));
        catalogs
            .insert(catalog(
                "en",
                "shared = primary\nfallback = primary fallback\n",
            ))
            .unwrap();
        catalogs
            .insert(catalog("en-GB", "shared = requested\n"))
            .unwrap();
        catalogs
            .insert(catalog("fr", "shared = francais\n"))
            .unwrap();

        assert_eq!(catalogs.resolve(&locale("en-GB"), "shared"), "requested");
        assert_eq!(catalogs.resolve(&locale("en-AU"), "shared"), "primary");
        assert_eq!(catalogs.resolve(&locale("fr-CA"), "shared"), "francais");
        assert_eq!(
            catalogs.resolve(&locale("de"), "fallback"),
            "primary fallback"
        );
        assert_eq!(catalogs.resolve(&locale("de"), "absent"), "absent");
    }

    #[test]
    fn resolve_uses_extension_free_locale_before_primary_language() {
        let mut catalogs = Catalogs::new(locale("en"));
        catalogs.insert(catalog("en", "hello = generic")).unwrap();
        catalogs
            .insert(catalog("en-GB", "hello = British"))
            .unwrap();

        assert_eq!(
            catalogs.resolve(&locale("en-GB-u-ca-gregory"), "hello"),
            "British"
        );
    }

    #[test]
    fn resolve_uses_extension_free_fallback_before_fallback_primary() {
        let mut catalogs = Catalogs::new(locale("fr-CA-u-nu-latn"));
        catalogs
            .insert(catalog("fr", "hello = fallback primary"))
            .unwrap();
        catalogs
            .insert(catalog("fr-CA", "hello = fallback base"))
            .unwrap();

        assert_eq!(catalogs.resolve(&locale("de"), "hello"), "fallback base");
    }

    #[test]
    fn resolve_truncates_variants_after_removing_extensions() {
        let mut catalogs = Catalogs::new(locale("en"));
        catalogs.insert(catalog("sl", "hello = Slovenian")).unwrap();
        catalogs
            .insert(catalog("sl-rozaj", "hello = Resian"))
            .unwrap();

        assert_eq!(
            catalogs.resolve(&locale("sl-rozaj-biske-u-nu-latn"), "hello"),
            "Resian"
        );
    }

    #[test]
    fn validation_reports_missing_fallback_and_catalog_differences() {
        let mut no_fallback = Catalogs::new(locale("en"));
        assert!(validate_catalogs(&no_fallback).fallback_catalog_missing);
        assert!(!validate_catalogs(&no_fallback).is_valid());

        no_fallback
            .insert(catalog("en", "hello = Hello\nbye = Goodbye"))
            .unwrap();
        no_fallback
            .insert(catalog("fr", "hello = Bonjour\nextra = En plus"))
            .unwrap();
        let report = validate_catalogs(&no_fallback);

        assert!(!report.fallback_catalog_missing);
        assert_eq!(
            report.missing[&locale("fr")]
                .iter()
                .map(MessageId::as_str)
                .collect::<Vec<_>>(),
            ["bye"]
        );
        assert_eq!(
            report.extra[&locale("fr")]
                .iter()
                .map(MessageId::as_str)
                .collect::<Vec<_>>(),
            ["extra"]
        );
        assert!(!report.is_valid());
    }
}
