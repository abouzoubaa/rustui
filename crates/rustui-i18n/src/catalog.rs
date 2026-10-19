//! Parser and lookup for RustUI's initial Fluent-compatible catalog subset.

use std::{collections::BTreeMap, error::Error, fmt};

use crate::{Locale, MessageId};

#[derive(Clone, Debug)]
pub struct Catalog {
    locale: Locale,
    messages: BTreeMap<MessageId, String>,
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

        Ok(Self { locale, messages })
    }

    pub fn locale(&self) -> &Locale {
        &self.locale
    }

    pub fn get(&self, identifier: &str) -> Option<&str> {
        self.messages.get(identifier).map(String::as_str)
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

    pub fn resolve<'a>(&'a self, locale: &Locale, identifier: &'a str) -> &'a str {
        let requested_primary = locale.primary();
        let fallback_primary = self.fallback.primary();
        for candidate in [
            Some(locale),
            (requested_primary != *locale).then_some(&requested_primary),
            Some(&self.fallback),
            (fallback_primary != self.fallback).then_some(&fallback_primary),
        ]
        .into_iter()
        .flatten()
        {
            if let Some(value) = self
                .catalogs
                .get(candidate)
                .and_then(|catalog| catalog.get(identifier))
            {
                return value;
            }
        }
        identifier
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
            catalog.message_ids().map(MessageId::as_str).collect::<Vec<_>>(),
            ["app.title", "greeting"]
        );
    }

    #[test]
    fn parse_errors_report_source_line_and_cause() {
        let malformed = Catalog::parse(locale("fr"), "# heading\nok = oui\nnot a pair").unwrap_err();
        assert_eq!(malformed.to_string(), "catalog `fr` line 3: expected `message-id = value`");

        let invalid_id = Catalog::parse(locale("fr"), "ok = oui\n9bad = non").unwrap_err();
        assert_eq!(invalid_id.to_string(), "catalog `fr` line 2: invalid message identifier `9bad`");

        let empty = Catalog::parse(locale("fr"), "\nbye =   ").unwrap_err();
        assert_eq!(empty.to_string(), "catalog `fr` line 2: message `bye` has an empty value");

        let duplicate = Catalog::parse(locale("fr"), "hello = bonjour\nhello = salut").unwrap_err();
        assert_eq!(duplicate.to_string(), "catalog `fr` line 2: message `hello` is defined more than once");
    }

    #[test]
    fn duplicate_locale_registration_is_rejected() {
        let mut catalogs = Catalogs::new(locale("en"));
        catalogs.insert(catalog("fr", "hello = bonjour")).unwrap();
        let error = catalogs.insert(catalog("fr", "hello = salut")).unwrap_err();

        assert_eq!(error.to_string(), "catalog `fr`: a catalog for this locale is already registered");
        assert_eq!(catalogs.get(&locale("fr")).unwrap().get("hello"), Some("bonjour"));
    }

    #[test]
    fn resolve_uses_full_locale_then_primary_then_fallbacks() {
        let mut catalogs = Catalogs::new(locale("en-GB"));
        catalogs.insert(catalog("en", "shared = primary\nfallback = primary fallback\n")).unwrap();
        catalogs.insert(catalog("en-GB", "shared = requested\n")).unwrap();
        catalogs.insert(catalog("fr", "shared = francais\n")).unwrap();

        assert_eq!(catalogs.resolve(&locale("en-GB"), "shared"), "requested");
        assert_eq!(catalogs.resolve(&locale("en-AU"), "shared"), "primary");
        assert_eq!(catalogs.resolve(&locale("fr-CA"), "shared"), "francais");
        assert_eq!(catalogs.resolve(&locale("de"), "fallback"), "primary fallback");
        assert_eq!(catalogs.resolve(&locale("de"), "absent"), "absent");
    }

    #[test]
    fn validation_reports_missing_fallback_and_catalog_differences() {
        let mut no_fallback = Catalogs::new(locale("en"));
        assert!(validate_catalogs(&no_fallback).fallback_catalog_missing);
        assert!(!validate_catalogs(&no_fallback).is_valid());

        no_fallback.insert(catalog("en", "hello = Hello\nbye = Goodbye")).unwrap();
        no_fallback.insert(catalog("fr", "hello = Bonjour\nextra = En plus")).unwrap();
        let report = validate_catalogs(&no_fallback);

        assert!(!report.fallback_catalog_missing);
        assert_eq!(report.missing[&locale("fr")].iter().map(MessageId::as_str).collect::<Vec<_>>(), ["bye"]);
        assert_eq!(report.extra[&locale("fr")].iter().map(MessageId::as_str).collect::<Vec<_>>(), ["extra"]);
        assert!(!report.is_valid());
    }
}
