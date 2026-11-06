//! Accessible error feedback.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px,
};
use rustui_core::Identity;

use crate::{
    components::basic::text::{Text, TextWeight},
    view::{EmptyView, RenderContext, View},
};

/// An alert message for an operation or content error.
#[derive(Clone)]
pub struct ErrorStateView {
    title: SharedString,
    detail: Option<SharedString>,
    id: Option<Identity>,
    accessibility_label: ErrorStateAccessibilityLabel,
}

#[derive(Clone)]
enum ErrorStateAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

impl ErrorStateView {
    pub fn new(title: impl Into<SharedString>) -> Self {
        let title = title.into();
        assert!(
            !title.trim().is_empty(),
            "ErrorState title must provide a non-empty accessible name"
        );
        Self {
            title: title.clone(),
            detail: None,
            id: None,
            accessibility_label: ErrorStateAccessibilityLabel::Literal(title),
        }
    }

    /// Adds supporting error information beneath the title.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets a stable element identity. Use distinct IDs for repeated states in one view tree.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Names the alert for assistive technology; defaults to the title.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "ErrorState accessibility label must not be empty"
        );
        self.accessibility_label = ErrorStateAccessibilityLabel::Literal(label);
        self
    }

    /// Resolves the alert name from the active locale, falling back to the explicit label or title.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "ErrorState accessibility localization key must not be empty"
        );
        let fallback = match &self.accessibility_label {
            ErrorStateAccessibilityLabel::Literal(label)
            | ErrorStateAccessibilityLabel::Message {
                fallback: label, ..
            } => label.clone(),
        };
        self.accessibility_label = ErrorStateAccessibilityLabel::Message { id, fallback };
        self
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        match &self.accessibility_label {
            ErrorStateAccessibilityLabel::Literal(label) => label.clone(),
            ErrorStateAccessibilityLabel::Message { id, fallback } => {
                let localized = localized_message(id.as_ref());
                if localized.trim().is_empty() || localized.trim() == id.trim() {
                    fallback.clone()
                } else {
                    localized.into()
                }
            }
        }
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            let detail = self.detail.as_deref().unwrap_or_default();
            Identity::from(format!(
                "error-state:{}:{}:{}:{}",
                self.title.len(),
                self.title,
                detail.len(),
                detail
            ))
        })
    }
}

/// Creates an error alert. Compose a retry Button alongside it when needed.
#[allow(non_snake_case)]
pub fn ErrorState(title: impl Into<SharedString>) -> ErrorStateView {
    ErrorStateView::new(title)
}

impl View for ErrorStateView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let theme = gpui_component::Theme::global(context.app());
        let danger = theme.colors.danger;
        let muted_foreground = theme.colors.muted_foreground;
        let accessibility_label =
            self.resolved_accessibility_label(|id| context.localized_message(id));
        let mut state = div()
            .id(self.identity().to_string())
            .role(gpui::Role::Alert)
            .aria_label(accessibility_label)
            .flex()
            .flex_col()
            .gap(px(6.0))
            .text_color(danger);

        state = state.child(
            Text(self.title.clone())
                .font_weight(TextWeight::Semibold)
                .into_element(context),
        );
        if let Some(detail) = &self.detail {
            state = state.aria_description(detail.clone());
            state = state.child(
                div()
                    .text_color(muted_foreground)
                    .child(Text(detail.clone()).into_element(context)),
            );
        }
        state.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_state_defaults_to_title_without_detail_or_overrides() {
        let state = ErrorState("Could not load");
        assert_eq!(state.title.as_ref(), "Could not load");
        assert!(state.detail.is_none());
        assert!(state.id.is_none());
        assert!(matches!(
            state.accessibility_label,
            ErrorStateAccessibilityLabel::Literal(ref label) if label == "Could not load"
        ));
    }

    #[test]
    fn error_state_builders_set_detail_identity_and_accessible_name() {
        let state = ErrorState("Could not load")
            .detail("Check your connection and try again.")
            .id("load-error")
            .accessibility_label("Content failed to load");
        assert_eq!(
            state.detail.as_deref(),
            Some("Check your connection and try again.")
        );
        assert_eq!(state.identity().as_str(), "load-error");
        assert_eq!(
            state
                .resolved_accessibility_label(|_| unreachable!())
                .as_ref(),
            "Content failed to load"
        );
    }

    #[test]
    fn localized_accessibility_label_uses_catalog_and_preserves_fallback() {
        let explicit = ErrorState("Could not load")
            .accessibility_label("Content failed to load")
            .localized_accessibility_label("error.load_failed");
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| format!("{id} traduit"))
                .as_ref(),
            "error.load_failed traduit"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| id.to_owned())
                .as_ref(),
            "Content failed to load"
        );
        assert_eq!(
            explicit
                .resolved_accessibility_label(|_| " \t ".to_owned())
                .as_ref(),
            "Content failed to load"
        );

        let default = ErrorState("Could not load").localized_accessibility_label("error.title");
        assert_eq!(
            default
                .resolved_accessibility_label(|_| String::new())
                .as_ref(),
            "Could not load"
        );
    }

    #[test]
    fn padded_localized_key_echo_uses_explicit_or_default_fallback() {
        let explicit = ErrorState("Could not load")
            .accessibility_label("Content failed to load")
            .localized_accessibility_label("error.load_failed");
        assert_eq!(
            explicit
                .resolved_accessibility_label(|_| " \terror.load_failed\n".to_owned())
                .as_ref(),
            "Content failed to load"
        );

        let default = ErrorState("Could not load").localized_accessibility_label("error.title");
        assert_eq!(
            default
                .resolved_accessibility_label(|_| " \terror.title\n".to_owned())
                .as_ref(),
            "Could not load"
        );
    }

    #[test]
    fn fallback_identity_uses_title_and_detail() {
        assert_ne!(
            ErrorState("Could not load").identity(),
            ErrorState("Could not load")
                .detail("Check your connection.")
                .identity()
        );
    }

    #[test]
    #[should_panic(expected = "ErrorState title must provide a non-empty accessible name")]
    fn error_state_rejects_a_blank_title() {
        ErrorState("  ");
    }

    #[test]
    #[should_panic(expected = "ErrorState accessibility label must not be empty")]
    fn error_state_rejects_a_blank_accessible_name() {
        ErrorState("Could not load").accessibility_label("\t ");
    }

    #[test]
    #[should_panic(expected = "ErrorState accessibility localization key must not be empty")]
    fn error_state_rejects_a_blank_localization_key() {
        ErrorState("Could not load").localized_accessibility_label(" \t ");
    }
}
