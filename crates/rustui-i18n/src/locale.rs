//! Validated, normalized BCP 47 style locale identifiers.

use std::{error::Error, fmt, str::FromStr};

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
            return Err(LocaleError::new(value));
        }

        let mut normalized = language.to_ascii_lowercase();
        for subtag in subtags {
            if subtag.is_empty()
                || subtag.len() > 8
                || !subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
            {
                return Err(LocaleError::new(value));
            }

            normalized.push('-');
            if subtag.len() == 4 && subtag.bytes().all(|byte| byte.is_ascii_alphabetic()) {
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
        Self(self.primary_language().to_owned())
    }

    pub fn subtags(&self) -> impl Iterator<Item = &str> {
        self.0.split('-')
    }
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
