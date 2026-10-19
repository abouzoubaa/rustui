//! Accessible empty-content feedback.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, Styled, div, px,
};
use rustui_core::Identity;

use crate::{
    components::basic::text::{Text, TextWeight},
    view::{EmptyView, RenderContext, View},
};

/// A centered message for an empty content area.
#[derive(Clone)]
pub struct EmptyStateView {
    title: SharedString,
    detail: Option<SharedString>,
    id: Option<Identity>,
    accessibility_label: Option<SharedString>,
}

impl EmptyStateView {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            id: None,
            accessibility_label: None,
        }
    }

    /// Adds supporting text beneath the title.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets a stable element identity. Use distinct IDs for repeated states in one view tree.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Names the status for assistive technology; defaults to the title.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            let detail = self.detail.as_deref().unwrap_or_default();
            Identity::from(format!(
                "empty-state:{}:{}:{}:{}",
                self.title.len(),
                self.title,
                detail.len(),
                detail
            ))
        })
    }
}

/// Creates an empty-content status message.
#[allow(non_snake_case)]
pub fn EmptyState(title: impl Into<SharedString>) -> EmptyStateView {
    EmptyStateView::new(title)
}

impl View for EmptyStateView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let theme = gpui_component::Theme::global(context.app());
        let foreground = theme.colors.foreground;
        let muted_foreground = theme.colors.muted_foreground;
        let accessibility_label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| self.title.clone());
        let mut state = div()
            .id(self.identity().to_string())
            .role(gpui::Role::Status)
            .aria_label(accessibility_label)
            .flex()
            .flex_col()
            .size_full()
            .min_w_0()
            .min_h_0()
            .items_center()
            .justify_center()
            .gap(px(6.0))
            .text_color(foreground);

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
    fn empty_state_defaults_to_title_without_detail_or_overrides() {
        let state = EmptyState("No results");
        assert_eq!(state.title.as_ref(), "No results");
        assert!(state.detail.is_none());
        assert!(state.id.is_none());
        assert!(state.accessibility_label.is_none());
    }

    #[test]
    fn empty_state_builders_set_detail_identity_and_accessible_name() {
        let state = EmptyState("No results")
            .detail("Try changing your search.")
            .id("search-empty")
            .accessibility_label("Search results are empty");
        assert_eq!(state.detail.as_deref(), Some("Try changing your search."));
        assert_eq!(state.identity().as_str(), "search-empty");
        assert_eq!(
            state.accessibility_label.as_deref(),
            Some("Search results are empty")
        );
    }

    #[test]
    fn fallback_identity_uses_title_and_detail() {
        assert_ne!(
            EmptyState("No results").identity(),
            EmptyState("No results")
                .detail("Try another search.")
                .identity()
        );
    }
}
