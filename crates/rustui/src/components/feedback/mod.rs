//! Feedback adapters backed by GPUI Components and GPUI primitives.
//!
//! Progress, Spinner, Skeleton, EmptyState, and ErrorState are implemented.
//! Toast is implemented separately in [`crate::components::overlay::toast`].

pub mod empty_state;
pub mod error_state;
pub mod progress;
pub mod skeleton;
pub mod spinner;

#[cfg(test)]
mod localization_tests {
    use crate::localization::resolve_text;
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
