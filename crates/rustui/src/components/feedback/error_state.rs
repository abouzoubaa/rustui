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
    accessibility_label: Option<SharedString>,
}

impl ErrorStateView {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            id: None,
            accessibility_label: None,
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
        self.accessibility_label = Some(label.into());
        self
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
        let accessibility_label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| self.title.clone());
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
        assert!(state.accessibility_label.is_none());
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
            state.accessibility_label.as_deref(),
            Some("Content failed to load")
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
}
