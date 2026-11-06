//! Progress indicators backed by the pinned GPUI Components progress bar.

use gpui::{AnyElement, IntoElement, SharedString};
use gpui_component::progress::Progress as GpuiProgress;

use crate::view::{EmptyView, RenderContext, View};

/// A linear progress indicator whose value is a percentage from 0 to 100.
#[derive(Clone)]
pub struct ProgressView {
    id: SharedString,
    value: f32,
    indeterminate: bool,
    accessibility_label: ProgressAccessibilityLabel,
}

#[derive(Clone)]
enum ProgressAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

impl ProgressView {
    pub fn new(id: impl Into<SharedString>, value: f32) -> Self {
        assert!(value.is_finite(), "progress value must be finite");
        let id = id.into();
        assert!(!id.trim().is_empty(), "Progress identity must not be blank");
        Self {
            id,
            value: value.clamp(0.0, 100.0),
            indeterminate: false,
            accessibility_label: ProgressAccessibilityLabel::Literal("Progress".into()),
        }
    }

    /// Shows the upstream indeterminate loading animation instead of a value.
    ///
    /// GPUI Components applies the app's reduced-motion setting to this mode.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Names the operation for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Progress accessibility label must not be empty"
        );
        self.accessibility_label = ProgressAccessibilityLabel::Literal(label);
        self
    }

    /// Resolves an accessible name from the active locale, falling back to
    /// the explicit name or the default `Progress` label.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Progress accessibility localization key must not be empty"
        );
        let fallback = match &self.accessibility_label {
            ProgressAccessibilityLabel::Literal(label)
            | ProgressAccessibilityLabel::Message {
                fallback: label, ..
            } => label.clone(),
        };
        self.accessibility_label = ProgressAccessibilityLabel::Message { id, fallback };
        self
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        match &self.accessibility_label {
            ProgressAccessibilityLabel::Literal(label) => label.clone(),
            ProgressAccessibilityLabel::Message { id, fallback } => {
                resolve_message_label(id, localized_message(id.as_ref()), fallback)
            }
        }
    }
}

fn resolve_message_label(
    id: &SharedString,
    localized: String,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.clone()
    } else {
        localized.into()
    }
}

/// Creates a progress bar with a stable element ID; it is determinate by default.
#[allow(non_snake_case)]
pub fn Progress(id: impl Into<SharedString>, value: f32) -> ProgressView {
    ProgressView::new(id, value)
}

impl View for ProgressView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        GpuiProgress::new(self.id.clone())
            .value(self.value)
            .loading(self.indeterminate)
            .accessibility_label(
                self.resolved_accessibility_label(|id| context.localized_message(id)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_is_bounded_to_percentage() {
        assert_eq!(Progress("upload", -10.0).value, 0.0);
        assert_eq!(Progress("upload", 30.0).value, 30.0);
        assert_eq!(Progress("upload", 120.0).value, 100.0);
    }

    #[test]
    #[should_panic(expected = "Progress identity must not be blank")]
    fn rejects_whitespace_only_identity() {
        Progress(" \t ", 30.0);
    }

    #[test]
    fn preserves_nonblank_identity_verbatim() {
        let progress = Progress(" progress ", 30.0);

        assert_eq!(progress.id.as_ref(), " progress ");
    }

    #[test]
    fn indeterminate_mode_is_opt_in_and_does_not_change_the_value() {
        let determinate = Progress("upload", 30.0);
        assert!(!determinate.indeterminate);
        assert_eq!(determinate.value, 30.0);

        let loading = determinate.indeterminate(true);
        assert!(loading.indeterminate);
        assert_eq!(loading.value, 30.0);
        assert_eq!(
            loading
                .resolved_accessibility_label(|_| unreachable!())
                .as_ref(),
            "Progress"
        );
    }

    #[test]
    fn accessibility_label_preserves_nonblank_input() {
        let progress = Progress("upload", 30.0).accessibility_label(" Uploading files ");
        assert_eq!(
            progress
                .resolved_accessibility_label(|_| unreachable!())
                .as_ref(),
            " Uploading files "
        );
    }

    #[test]
    #[should_panic(expected = "Progress accessibility label must not be empty")]
    fn accessibility_label_rejects_blank_input() {
        Progress("upload", 30.0).accessibility_label("\t ");
    }

    #[test]
    fn localized_accessibility_label_uses_catalog_and_fallback() {
        let explicit = Progress("upload", 30.0)
            .accessibility_label("Uploading files")
            .localized_accessibility_label("progress.upload");
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| format!("{id} traduit"))
                .as_ref(),
            "progress.upload traduit"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            "Uploading files"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| format!(" \t{id}\n"))
                .as_ref(),
            "Uploading files"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|_| " \t ".to_owned())
                .as_ref(),
            "Uploading files"
        );

        let default = Progress("upload", 30.0).localized_accessibility_label("progress.label");
        assert_eq!(
            default
                .resolved_accessibility_label(|_| String::new())
                .as_ref(),
            "Progress"
        );
    }

    #[test]
    #[should_panic(expected = "Progress accessibility localization key must not be empty")]
    fn localized_accessibility_label_rejects_blank_message_id() {
        Progress("upload", 30.0).localized_accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "progress value must be finite")]
    fn non_finite_value_is_rejected() {
        Progress("upload", f32::NAN);
    }
}
