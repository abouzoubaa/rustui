//! Stable caller-owned identity for repeated views and controls.

use std::{fmt, sync::Arc};

/// A stable semantic identifier supplied by the caller.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Identity(Arc<str>);

impl Identity {
    pub fn new(value: impl Into<Arc<str>>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Identity {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&str> for Identity {
    fn from(value: &str) -> Self {
        Self::new(Arc::<str>::from(value))
    }
}

impl From<String> for Identity {
    fn from(value: String) -> Self {
        Self::new(Arc::<str>::from(value))
    }
}

impl From<usize> for Identity {
    fn from(value: usize) -> Self {
        Self::from(value.to_string())
    }
}

impl From<u64> for Identity {
    fn from(value: u64) -> Self {
        Self::from(value.to_string())
    }
}
