//! Validated, normalized BCP 47 style locale identifiers.

use std::{collections::HashSet, error::Error, fmt, str::FromStr};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Locale(String);

impl Locale {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, LocaleError> {
        let value = value.as_ref().trim().replace('_', "-");
        let mut subtags = value.split('-');
        let language = subtags.next().unwrap_or_default();
        if !(2..=8).contains(&language.len())
            || !language.bytes().all(|byte| byte.is_ascii_alphabetic())
        {
            if language.eq_ignore_ascii_case("x") {
                let private_use = subtags.collect::<Vec<_>>();
                if private_use.is_empty()
                    || private_use.iter().any(|subtag| {
                        subtag.is_empty()
                            || subtag.len() > 8
                            || !subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
                    })
                {
                    return Err(LocaleError::new(value));
                }

                return Ok(Self(format!(
                    "x-{}",
                    private_use
                        .iter()
                        .map(|subtag| subtag.to_ascii_lowercase())
                        .collect::<Vec<_>>()
                        .join("-")
                )));
            }
            return Err(LocaleError::new(value));
        }

        let mut normalized = language.to_ascii_lowercase();
        let mut subtags = subtags.peekable();
        let mut extension_value_required = false;
        let mut in_extension = false;
        let mut private_use = false;
        let mut extension_singletons = HashSet::new();
        let mut in_unicode_extension = false;
        let mut unicode_extension_keyword_seen = false;
        let mut unicode_extension_attributes = HashSet::new();
        let mut unicode_extension_keys = HashSet::new();
        let mut variants = HashSet::new();
        let mut script_seen = false;
        let mut region_seen = false;
        let mut extlangs_open = (2..=3).contains(&language.len());
        let mut extlangs_seen = 0;
        while let Some(subtag) = subtags.next() {
            let extension_subtag = private_use || in_extension;
            if subtag.is_empty()
                || subtag.len() > 8
                || !subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
            {
                return Err(LocaleError::new(value));
            }

            if !extension_subtag {
                let is_extlang = subtag.len() == 3
                    && subtag.bytes().all(|byte| byte.is_ascii_alphabetic());
                if is_extlang {
                    if !extlangs_open || extlangs_seen == 3 {
                        return Err(LocaleError::new(value));
                    }
                    extlangs_seen += 1;
                } else {
                    extlangs_open = false;
                }

                let is_script =
                    subtag.len() == 4 && subtag.bytes().all(|byte| byte.is_ascii_alphabetic());
                let is_region = (subtag.len() == 2
                    && subtag.bytes().all(|byte| byte.is_ascii_alphabetic()))
                    || (subtag.len() == 3 && subtag.bytes().all(|byte| byte.is_ascii_digit()));
                if !(is_extlang
                    || is_script
                    || is_region
                    || is_variant_subtag(subtag)
                    || subtag.len() == 1)
                {
                    return Err(LocaleError::new(value));
                }
                if subtag.len() == 2 && !is_region {
                    return Err(LocaleError::new(value));
                }
                if is_script {
                    if script_seen || region_seen || !variants.is_empty() {
                        return Err(LocaleError::new(value));
                    }
                    script_seen = true;
                } else if is_region {
                    if region_seen || !variants.is_empty() {
                        return Err(LocaleError::new(value));
                    }
                    region_seen = true;
                }
            }
            if !extension_subtag
                && is_variant_subtag(subtag)
                && !variants.insert(subtag.to_ascii_lowercase())
            {
                return Err(LocaleError::new(value));
            }

            if extension_value_required {
                if subtag.len() < 2 {
                    return Err(LocaleError::new(value));
                }
                extension_value_required = false;
            } else if !private_use && subtag.len() == 1 {
                if subtag.eq_ignore_ascii_case("x") {
                    if subtags.peek().is_none() {
                        return Err(LocaleError::new(value));
                    }
                    private_use = true;
                    in_unicode_extension = false;
                } else {
                    if subtags.peek().is_none()
                        || !extension_singletons.insert(subtag.as_bytes()[0].to_ascii_lowercase())
                    {
                        return Err(LocaleError::new(value));
                    }
                    in_extension = true;
                    in_unicode_extension = subtag.eq_ignore_ascii_case("u");
                    unicode_extension_keyword_seen = false;
                    extension_value_required = true;
                }
            }
            if in_unicode_extension {
                if subtag.len() == 2 {
                    if !unicode_extension_keys.insert(subtag.to_ascii_lowercase()) {
                        return Err(LocaleError::new(value));
                    }
                    unicode_extension_keyword_seen = true;
                } else if !unicode_extension_keyword_seen
                    && !unicode_extension_attributes.insert(subtag.to_ascii_lowercase())
                {
                    return Err(LocaleError::new(value));
                }
            }

            normalized.push('-');
            if extension_subtag {
                normalized.push_str(&subtag.to_ascii_lowercase());
            } else if subtag.len() == 4 && subtag.bytes().all(|byte| byte.is_ascii_alphabetic()) {
                let mut characters = subtag.chars();
                normalized.extend(
                    characters
                        .next()
                        .map(|character| character.to_ascii_uppercase()),
                );
                normalized.extend(characters.map(|character| character.to_ascii_lowercase()));
            } else if (subtag.len() == 2 && subtag.bytes().all(|byte| byte.is_ascii_alphabetic()))
                || (subtag.len() == 3 && subtag.bytes().all(|byte| byte.is_ascii_digit()))
            {
                normalized.push_str(&subtag.to_ascii_uppercase());
            } else {
                normalized.push_str(&subtag.to_ascii_lowercase());
            }
        }
        Ok(Self(normalized))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn primary_language(&self) -> &str {
        self.0
            .split('-')
            .next()
            .expect("validated locales are nonempty")
    }

    pub fn primary(&self) -> Self {
        if self.primary_language() == "x" {
            self.clone()
        } else {
            Self(self.primary_language().to_owned())
        }
    }

    /// Returns the language tag without extension or private-use subtags.
    pub fn without_extensions(&self) -> Self {
        if self.primary_language() == "x" {
            return self.clone();
        }
        Self(
            self.0
                .split('-')
                .take_while(|subtag| subtag.len() != 1)
                .collect::<Vec<_>>()
                .join("-"),
        )
    }

    pub fn subtags(&self) -> impl Iterator<Item = &str> {
        self.0.split('-')
    }
}

fn is_variant_subtag(subtag: &str) -> bool {
    (5..=8).contains(&subtag.len()) || (subtag.len() == 4 && subtag.as_bytes()[0].is_ascii_digit())
}

impl fmt::Display for Locale {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl FromStr for Locale {
    type Err = LocaleError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::parse(value)
    }
}

impl TryFrom<&str> for Locale {
    type Error = LocaleError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::parse(value)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LocaleError {
    value: String,
}

impl LocaleError {
    fn new(value: String) -> Self {
        Self { value }
    }
}

impl fmt::Display for LocaleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid locale identifier `{}`", self.value)
    }
}

impl Error for LocaleError {}

#[cfg(test)]
mod tests {
    use super::Locale;

    #[test]
    fn rejects_duplicate_or_out_of_order_script_and_region_subtags() {
        for tag in [
            "en-Latn-Cyrl",
            "en-US-GB",
            "en-419-001",
            "en-US-Latn",
            "en-1901-Latn",
            "en-1901-US",
        ] {
            assert!(
                Locale::parse(tag).is_err(),
                "accepted duplicate or out-of-order subtags: {tag}"
            );
        }

        assert_eq!(Locale::parse("en-Latn-US").unwrap().as_str(), "en-Latn-US");
        assert_eq!(
            Locale::parse("en-Latn-US-1901").unwrap().as_str(),
            "en-Latn-US-1901"
        );
        assert_eq!(
            Locale::parse("en-US-u-rg-gbzzzz").unwrap().as_str(),
            "en-US-u-rg-gbzzzz"
        );
        assert_eq!(Locale::parse("en-x-US-US").unwrap().as_str(), "en-x-us-us");
    }

    #[test]
    fn accepts_four_character_primary_language_subtags() {
        assert_eq!(Locale::parse("abcd").unwrap().as_str(), "abcd");
        assert_eq!(Locale::parse("ABCD-Latn").unwrap().as_str(), "abcd-Latn");
    }


    #[test]
    fn allows_extlangs_only_before_script_region_and_variants() {
        for tag in [
            "en-US-abc",
            "en-Latn-abc",
            "en-1901-abc",
            "abcd-abc",
            "en-abc-def-ghi-jkl",
        ] {
            assert!(
                Locale::parse(tag).is_err(),
                "accepted misplaced or excessive extlang subtags: {tag}"
            );
        }

        for tag in [
            "en-abc",
            "en-abc-def-ghi-Latn-US-1901",
            "zh-cmn-Hans-CN",
            "en-u-abc",
            "en-x-abc-def-ghi-jkl",
        ] {
            assert!(
                Locale::parse(tag).is_ok(),
                "rejected structurally ordered extlang subtags: {tag}"
            );
        }
    }

    #[test]
    fn rejects_invalid_two_character_core_subtags_but_allows_extension_values() {
        for tag in ["en-12", "en-a1"] {
            assert!(
                Locale::parse(tag).is_err(),
                "accepted invalid two-character core subtag: {tag}"
            );
        }

        assert_eq!(Locale::parse("en-u-12").unwrap().as_str(), "en-u-12");
        assert_eq!(Locale::parse("en-x-12").unwrap().as_str(), "en-x-12");
    }

    #[test]
    fn rejects_unclassified_core_subtags_but_allows_extension_values() {
        for tag in ["en-a1b", "en-a123", "en-12a"] {
            assert!(
                Locale::parse(tag).is_err(),
                "accepted unclassified core subtag: {tag}"
            );
        }

        assert_eq!(Locale::parse("en-u-a1b").unwrap().as_str(), "en-u-a1b");
        assert_eq!(Locale::parse("en-9-a1b").unwrap().as_str(), "en-9-a1b");
        assert_eq!(Locale::parse("en-x-a1b").unwrap().as_str(), "en-x-a1b");
    }

    #[test]
    fn parses_extension_and_private_use_subtags() {
        assert_eq!(Locale::parse(" EN_us ").unwrap().as_str(), "en-US");
        assert_eq!(
            Locale::parse("EN-us-u-ca-gregory").unwrap().as_str(),
            "en-US-u-ca-gregory"
        );
        assert_eq!(
            Locale::parse("en-x-Company-variant").unwrap().as_str(),
            "en-x-company-variant"
        );
        assert_eq!(Locale::parse("en-9-abc").unwrap().as_str(), "en-9-abc");
    }

    #[test]
    fn rejects_extensions_without_a_value() {
        assert!(Locale::parse("en-u").is_err());
        assert!(Locale::parse("en-u-x").is_err());
        assert!(Locale::parse("en-x").is_err());
        assert!(Locale::parse("en-u-ca-u-nu").is_err());
    }

    #[test]
    fn covers_rfc_subtag_and_generic_extension_boundaries() {
        for tag in [
            "abcdefgh",
            "en-abcdefgh",
            "en-a-12345678",
            "en-419",
            "en-a-12",
        ] {
            assert!(Locale::parse(tag).is_ok(), "rejected valid boundary tag: {tag}");
        }
        for tag in [
            "abcdefghi",
            "en-abcdefghi",
            "en-a-123456789",
            "en-a-b",
            "en-A-foo-a-bar",
            "",
            "en--US",
            "éé",
            "en-Å",
        ] {
            assert!(Locale::parse(tag).is_err(), "accepted invalid boundary tag: {tag}");
        }
    }

    #[test]
    fn rejects_duplicate_variant_subtags_case_insensitively() {
        assert!(Locale::parse("en-1901-1901").is_err());
        assert!(Locale::parse("sl-rozaj-ROZAJ").is_err());
        assert_eq!(
            Locale::parse("sl-rozaj-biske").unwrap().as_str(),
            "sl-rozaj-biske"
        );
    }

    #[test]
    fn rejects_duplicate_unicode_extension_keys_case_insensitively() {
        assert!(Locale::parse("en-US-u-nu-latn-NU-arab").is_err());
    }

    #[test]
    fn rejects_duplicate_unicode_extension_attributes_case_insensitively() {
        assert!(Locale::parse("en-u-foobar-FOOBAR").is_err());
        assert_eq!(
            Locale::parse("en-u-foo-ca-foo").unwrap().as_str(),
            "en-u-foo-ca-foo"
        );
    }

    #[test]
    fn accepts_private_use_only_tags_and_rejects_empty_or_oversized_payloads() {
        let locale = Locale::parse("X-Company-Tag").unwrap();
        assert_eq!(locale.as_str(), "x-company-tag");
        assert_eq!(locale.primary(), locale);
        assert_eq!(locale.without_extensions(), locale);

        for tag in ["x", "x-", "x--tag", "x-123456789"] {
            assert!(
                Locale::parse(tag).is_err(),
                "accepted invalid private-use-only tag: {tag}"
            );
        }
    }

}
