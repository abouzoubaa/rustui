//! Indeterminate loading spinner backed by pinned GPUI Components.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, Role, SharedString,
    StatefulInteractiveElement, div,
};
use gpui_component::spinner::Spinner as GpuiSpinner;
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, View};

/// An indeterminate loading indicator.
#[derive(Clone)]
pub struct SpinnerView {
    id: Option<Identity>,
    accessibility_label: SpinnerAccessibilityLabel,
}

#[derive(Clone)]
enum SpinnerAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

impl Default for SpinnerView {
    fn default() -> Self {
        Self {
            id: None,
            accessibility_label: SpinnerAccessibilityLabel::Literal("Loading".into()),
        }
    }
}

impl SpinnerView {
    /// Sets a stable ID when multiple spinners share one view tree.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Spinner identity must not be blank"
        );
        self.id = Some(id);
        self
    }

    /// Names the operation for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Spinner accessibility label must not be blank"
        );
        self.accessibility_label = SpinnerAccessibilityLabel::Literal(label);
        self
    }

    /// Resolves an accessible name from the active locale, falling back to
    /// the explicit name or the default `Loading` label.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Spinner accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            SpinnerAccessibilityLabel::Literal(label)
            | SpinnerAccessibilityLabel::Message {
                fallback: label, ..
            } => label.clone(),
        };
        self.accessibility_label = SpinnerAccessibilityLabel::Message { id, fallback };
        self
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        match &self.accessibility_label {
            SpinnerAccessibilityLabel::Literal(label) => label.clone(),
            SpinnerAccessibilityLabel::Message { id, fallback } => {
                let localized = localized_message(id.as_ref());
                if localized.trim().is_empty() || localized.trim() == id.trim() {
                    fallback.clone()
                } else {
                    localized.into()
                }
            }
        }
    }
}

/// Creates an indeterminate loading indicator.
#[allow(non_snake_case)]
pub fn Spinner() -> SpinnerView {
    SpinnerView::default()
}

impl View for SpinnerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        div()
            .id(SharedString::from(
                self.id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "rustui-spinner".to_owned()),
            ))
            .role(Role::ProgressIndicator)
            .aria_label(self.resolved_accessibility_label(|id| context.localized_message(id)))
            .child(GpuiSpinner::new())
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_has_an_accessible_name() {
        assert_eq!(
            Spinner()
                .resolved_accessibility_label(|_| unreachable!())
                .as_ref(),
            "Loading"
        );
        assert_eq!(
            Spinner()
                .accessibility_label(" Download progress ")
                .resolved_accessibility_label(|_| unreachable!())
                .as_ref(),
            " Download progress "
        );
    }

    #[test]
    fn localized_accessibility_label_uses_catalog_and_fallback() {
        let explicit = Spinner()
            .accessibility_label("Downloading files")
            .localized_accessibility_label("spinner.download");
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| format!("{id} traduit"))
                .as_ref(),
            "spinner.download traduit"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            "Downloading files"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| format!(" \t{id}\n"))
                .as_ref(),
            "Downloading files"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|_| " \t ".to_owned())
                .as_ref(),
            "Downloading files"
        );

        let default = Spinner().localized_accessibility_label("spinner.loading");
        assert_eq!(
            default
                .resolved_accessibility_label(|_| String::new())
                .as_ref(),
            "Loading"
        );
    }

    #[test]
    #[should_panic(expected = "Spinner accessibility localization key must not be blank")]
    fn localized_accessibility_label_rejects_blank_message_id() {
        Spinner().localized_accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "Spinner accessibility label must not be blank")]
    fn spinner_rejects_a_blank_accessibility_name() {
        Spinner().accessibility_label(" \t ");
    }

    #[test]
    fn spinner_accepts_an_explicit_id() {
        assert_eq!(Spinner().id, None);
        assert_eq!(
            Spinner().id("upload-spinner").id,
            Some(Identity::from("upload-spinner"))
        );
    }

    #[test]
    #[should_panic(expected = "Spinner identity must not be blank")]
    fn spinner_rejects_a_blank_identity() {
        Spinner().id(" \t ");
    }
}
