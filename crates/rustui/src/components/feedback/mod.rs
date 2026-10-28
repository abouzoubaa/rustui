//! Feedback adapters backed by GPUI Components and GPUI primitives.
//!
//! Progress, Spinner, Skeleton, EmptyState, and ErrorState are implemented.
//! Toast is implemented separately in [`crate::components::overlay::toast`].

pub mod empty_state;
pub mod error_state;
pub mod progress;
pub mod skeleton;
pub mod spinner;

// Use literal text when a translation is missing, invalid, or blank.
pub(super) fn resolve_text(
    fallback: &gpui::SharedString,
    key: Option<&gpui::SharedString>,
    resolve: impl FnOnce(&str) -> String,
) -> gpui::SharedString {
    match key {
        Some(key) => {
            let text = resolve(key.as_ref());
            if text.trim().is_empty() || text.trim() == key.trim() {
                fallback.clone()
            } else {
                text.into()
            }
        }
        None => fallback.clone(),
    }
}

#[cfg(test)]
mod localization_tests {
    use super::resolve_text;
    use gpui::SharedString;

    #[test]
    fn localized_feedback_text_uses_translation_and_safe_fallbacks() {
        let fallback: SharedString = "Original".into();
        let key: SharedString = "feedback.title".into();
        assert_eq!(
            resolve_text(&fallback, Some(&key), |_| "Traduit".into()).as_ref(),
            "Traduit"
        );
        assert_eq!(
            resolve_text(&fallback, Some(&key), |_| "feedback.title".into()).as_ref(),
            "Original"
        );
        assert_eq!(
            resolve_text(&fallback, Some(&key), |_| " \t".into()).as_ref(),
            "Original"
        );
        assert_eq!(
            resolve_text(&fallback, None, |_| panic!("no lookup needed")).as_ref(),
            "Original"
        );
    }
}
