//! Caller-supplied font fallback families for GPUI text shaping.

use gpui::FontFallbacks;

/// An explicit ordered list of fallback font family names.
///
/// GPUI owns font resolution and shaping; this policy only supplies the list.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FontFallbackPolicy(FontFallbacks);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontFallbackPolicyError {
    Empty,
    BlankFamily { index: usize },
}

impl FontFallbackPolicy {
    /// Creates a fallback policy, rejecting an empty list or whitespace-only names.
    pub fn new(
        families: impl IntoIterator<Item = impl Into<String>>,
    ) -> Result<Self, FontFallbackPolicyError> {
        let families = families.into_iter().map(Into::into).collect::<Vec<_>>();
        if families.is_empty() {
            return Err(FontFallbackPolicyError::Empty);
        }
        if let Some(index) = families.iter().position(|family| family.trim().is_empty()) {
            return Err(FontFallbackPolicyError::BlankFamily { index });
        }
        Ok(Self(FontFallbacks::from_fonts(families)))
    }

    /// Returns the configured family names in resolution order.
    pub fn families(&self) -> &[String] {
        self.0.fallback_list()
    }

    pub(crate) fn as_gpui(&self) -> &FontFallbacks {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preserves_fallback_order() {
        let policy = FontFallbackPolicy::new(["Noto Sans", "Noto Sans CJK SC"]).unwrap();
        assert_eq!(policy.families(), ["Noto Sans", "Noto Sans CJK SC"]);
    }

    #[test]
    fn rejects_empty_or_blank_families() {
        assert_eq!(
            FontFallbackPolicy::new(Vec::<String>::new()).unwrap_err(),
            FontFallbackPolicyError::Empty
        );
        assert_eq!(
            FontFallbackPolicy::new(["Noto Sans", "  "]).unwrap_err(),
            FontFallbackPolicyError::BlankFamily { index: 1 }
        );
    }
}
