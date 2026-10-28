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
    title_key: Option<SharedString>,
    detail_key: Option<SharedString>,
    id: Option<Identity>,
    accessibility_label: Option<SharedString>,
    localized_accessibility_label: Option<(SharedString, SharedString)>,
}

impl EmptyStateView {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            detail: None,
            title_key: None,
            detail_key: None,
            id: None,
            accessibility_label: None,
            localized_accessibility_label: None,
        }
    }

    /// Adds supporting text beneath the title.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Resolves the visible title in the active locale, falling back to the literal title.
    pub fn localized_title(mut self, key: impl Into<SharedString>) -> Self {
        let key = key.into();
        assert!(!key.trim().is_empty(), "localization key must not be empty");
        self.title_key = Some(key);
        self
    }

    /// Resolves the supporting detail in the active locale when detail is present.
    pub fn localized_detail(mut self, key: impl Into<SharedString>) -> Self {
        let key = key.into();
        assert!(!key.trim().is_empty(), "localization key must not be empty");
        self.detail_key = Some(key);
        self
    }

    /// Sets a stable element identity. Use distinct IDs for repeated states in one view tree.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Names the status for assistive technology; defaults to the title.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "EmptyState accessibility label must not be empty"
        );
        self.accessibility_label = Some(label);
        self.localized_accessibility_label = None;
        self
    }

    /// Resolves the status name from the active locale, falling back to the explicit label or title.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "EmptyState accessibility localization key must not be empty"
        );
        let fallback = self
            .localized_accessibility_label
            .as_ref()
            .map(|(_, fallback)| fallback.clone())
            .or_else(|| self.accessibility_label.clone())
            .unwrap_or_else(|| self.title.clone());
        self.localized_accessibility_label = Some((id, fallback));
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

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        let label = match &self.localized_accessibility_label {
            Some((id, fallback)) => {
                let localized = localized_message(id.as_ref());
                if localized.trim().is_empty() || localized.trim() == id.trim() {
                    fallback.clone()
                } else {
                    localized.into()
                }
            }
            None => self
                .accessibility_label
                .clone()
                .unwrap_or_else(|| self.title.clone()),
        };
        assert!(
            !label.trim().is_empty(),
            "EmptyState must provide a non-empty accessible name"
        );
        label
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
        let title = super::resolve_text(&self.title, self.title_key.as_ref(), |key| {
            context.localized_message(key)
        });
        let accessibility_label =
            if self.accessibility_label.is_none() && self.localized_accessibility_label.is_none() {
                title.clone()
            } else {
                self.resolved_accessibility_label(|id| context.localized_message(id))
            };
        assert!(
            !accessibility_label.trim().is_empty(),
            "EmptyState must provide a non-empty accessible name"
        );
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
            Text(title)
                .font_weight(TextWeight::Semibold)
                .into_element(context),
        );
        if let Some(detail) = &self.detail {
            let detail = super::resolve_text(detail, self.detail_key.as_ref(), |key| {
                context.localized_message(key)
            });
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
    fn empty_state_preserves_nonblank_accessible_name() {
        let state = EmptyState("No results").accessibility_label("  Search results are empty  ");
        assert_eq!(
            state.accessibility_label.as_deref(),
            Some("  Search results are empty  ")
        );
    }

    #[test]
    #[should_panic(expected = "EmptyState accessibility label must not be empty")]
    fn empty_state_rejects_a_blank_accessible_name() {
        EmptyState("No results").accessibility_label("\t ");
    }

    #[test]
    fn blank_visible_title_is_allowed_with_nonblank_accessible_name_override() {
        let state = EmptyState("").accessibility_label("No search results");

        assert_eq!(state.title.as_ref(), "");
        assert_eq!(
            state.resolved_accessibility_label(|_| unreachable!()),
            "No search results"
        );
    }

    #[test]
    #[should_panic(expected = "EmptyState must provide a non-empty accessible name")]
    fn blank_visible_title_requires_accessible_name_override() {
        EmptyState("").resolved_accessibility_label(|_| unreachable!());
    }

    #[test]
    fn localized_accessibility_name_uses_catalog_and_preserves_fallback() {
        let explicit = EmptyState("No results")
            .accessibility_label("Search results are empty")
            .localized_accessibility_label("empty_state.no_results");
        assert_eq!(
            explicit
                .resolved_accessibility_label(|id| format!("{id} traduit"))
                .as_ref(),
            "empty_state.no_results traduit"
        );
        for unresolved in [
            "empty_state.no_results".to_owned(),
            " empty_state.no_results \t".to_owned(),
            " \t ".to_owned(),
        ] {
            assert_eq!(
                explicit
                    .resolved_accessibility_label(|_| unresolved.clone())
                    .as_ref(),
                "Search results are empty"
            );
        }

        let default = EmptyState("No results").localized_accessibility_label("empty_state.title");
        assert_eq!(
            default
                .resolved_accessibility_label(|_| String::new())
                .as_ref(),
            "No results"
        );
    }

    #[test]
    #[should_panic(expected = "EmptyState accessibility localization key must not be empty")]
    fn localized_accessibility_name_rejects_a_blank_key() {
        EmptyState("No results").localized_accessibility_label(" \t\n ");
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
