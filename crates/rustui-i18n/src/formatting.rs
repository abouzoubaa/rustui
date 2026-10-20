//! Named message interpolation without locale-sensitive number formatting.
//!
//! Placeholders use `{name}` syntax. Double braces escape a brace (`{{` and
//! `}}`). Every placeholder must have a supplied argument; extra arguments are
//! ignored. Plural selection, dates, and locale-sensitive numbers remain out
//! of scope.

use std::{collections::BTreeMap, error::Error, fmt};

/// A value that can be inserted into a catalog message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageArgument {
    Text(String),
    Integer(i64),
}

impl From<String> for MessageArgument {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for MessageArgument {
    fn from(value: &str) -> Self {
        Self::Text(value.to_owned())
    }
}

impl From<i64> for MessageArgument {
    fn from(value: i64) -> Self {
        Self::Integer(value)
    }
}

impl From<i32> for MessageArgument {
    fn from(value: i32) -> Self {
        Self::Integer(i64::from(value))
    }
}

/// Named arguments supplied to [`interpolate`].
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct MessageArguments(BTreeMap<String, MessageArgument>);

impl MessageArguments {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<MessageArgument>) {
        self.0.insert(name.into(), value.into());
    }

    pub fn with(mut self, name: impl Into<String>, value: impl Into<MessageArgument>) -> Self {
        self.insert(name, value);
        self
    }
}

/// Failure to parse a placeholder or find its required argument.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum InterpolationError {
    InvalidPlaceholder(String),
    MissingArgument(String),
    UnmatchedOpeningBrace,
    UnmatchedClosingBrace,
}

impl fmt::Display for InterpolationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPlaceholder(name) => write!(formatter, "invalid placeholder `{{{name}}}`"),
            Self::MissingArgument(name) => write!(formatter, "missing message argument `{name}`"),
            Self::UnmatchedOpeningBrace => {
                formatter.write_str("unmatched opening brace in message")
            }
            Self::UnmatchedClosingBrace => {
                formatter.write_str("unmatched closing brace in message")
            }
        }
    }
}

impl Error for InterpolationError {}

/// Inserts named text or integer arguments into a message template.
///
/// Integer arguments use ordinary decimal formatting, independent of locale.
/// Literal braces are written as `{{` and `}}`.
pub fn interpolate(
    template: &str,
    arguments: &MessageArguments,
) -> Result<String, InterpolationError> {
    let mut chars = template.chars().peekable();
    let mut output = String::with_capacity(template.len());

    while let Some(character) = chars.next() {
        match character {
            '{' if chars.peek() == Some(&'{') => {
                chars.next();
                output.push('{');
            }
            '{' => {
                let mut name = String::new();
                loop {
                    match chars.next() {
                        Some('}') => break,
                        Some('{') | None => return Err(InterpolationError::UnmatchedOpeningBrace),
                        Some(character) => name.push(character),
                    }
                }
                if !valid_argument_name(&name) {
                    return Err(InterpolationError::InvalidPlaceholder(name));
                }
                let value = arguments
                    .0
                    .get(&name)
                    .ok_or_else(|| InterpolationError::MissingArgument(name.clone()))?;
                match value {
                    MessageArgument::Text(value) => output.push_str(value),
                    MessageArgument::Integer(value) => output.push_str(&value.to_string()),
                }
            }
            '}' if chars.peek() == Some(&'}') => {
                chars.next();
                output.push('}');
            }
            '}' => return Err(InterpolationError::UnmatchedClosingBrace),
            character => output.push(character),
        }
    }

    Ok(output)
}

fn valid_argument_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b'-'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolates_named_text_and_integer_arguments() {
        let arguments = MessageArguments::new()
            .with("name", "Ada")
            .with("count", 3_i32);

        assert_eq!(
            interpolate("Hello, {name}: {count} items", &arguments).unwrap(),
            "Hello, Ada: 3 items"
        );
    }

    #[test]
    fn supports_escaped_braces_and_repeated_placeholders() {
        let arguments = MessageArguments::new().with("name", "RustUI");

        assert_eq!(
            interpolate("{{{name}}} / {name}", &arguments).unwrap(),
            "{RustUI} / RustUI"
        );
    }

    #[test]
    fn rejects_missing_malformed_and_unmatched_placeholders() {
        let arguments = MessageArguments::new();
        assert_eq!(
            interpolate("Hi, {name}", &arguments),
            Err(InterpolationError::MissingArgument("name".into()))
        );
        assert!(matches!(
            interpolate("{bad name}", &arguments),
            Err(InterpolationError::InvalidPlaceholder(_))
        ));
        assert_eq!(
            interpolate("{name", &arguments),
            Err(InterpolationError::UnmatchedOpeningBrace)
        );
        assert_eq!(
            interpolate("name}", &arguments),
            Err(InterpolationError::UnmatchedClosingBrace)
        );
    }
}
