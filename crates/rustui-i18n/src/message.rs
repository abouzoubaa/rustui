//! Validated message identifiers for the supported catalog subset.

use std::{borrow::Borrow, error::Error, fmt};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MessageId(String);

impl MessageId {
    pub fn parse(value: impl AsRef<str>) -> Result<Self, MessageIdError> {
        let value = value.as_ref().trim();
        let mut bytes = value.bytes();
        let valid = bytes.next().is_some_and(|byte| byte.is_ascii_alphabetic())
            && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'));
        if valid {
            Ok(Self(value.to_owned()))
        } else {
            Err(MessageIdError {
                value: value.to_owned(),
            })
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for MessageId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Borrow<str> for MessageId {
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MessageIdError {
    value: String,
}

impl fmt::Display for MessageIdError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "invalid message identifier `{}`", self.value)
    }
}

impl Error for MessageIdError {}
