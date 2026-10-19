//! Accessible link activation through the pinned GPUI Components Button.

use gpui::{AnyElement, FontWeight, IntoElement, SharedString, Styled as _, px};
use gpui_component::{
    Disableable as _,
    button::{Button as GpuiButton, ButtonVariant, ButtonVariants as _},
};
use rustui_core::Identity;

use crate::{
    components::theme::{required_style, selector},
    view::{ControlAction, EmptyView, RenderContext, View},
};

/// A platform-opened link with an accessible visible label.
#[derive(Clone)]
pub struct LinkView {
    label: SharedString,
    href: SharedString,
    id: Option<Identity>,
    accessibility_label: Option<SharedString>,
    disabled: bool,
    activation: Option<ControlAction>,
}

impl LinkView {
    pub fn new(label: impl Into<SharedString>, href: impl Into<SharedString>) -> Self {
        Self {
            label: label.into(),
            href: href.into(),
            id: None,
            accessibility_label: None,
            disabled: false,
            activation: None,
        }
    }

    /// Sets a stable element identity. Use distinct IDs for repeated links with the same label and URL.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Overrides the accessible name; the visible label remains unchanged.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Observes activation before the URL is handed to the platform.
    /// This reports a user activation, not completion of browser navigation.
    pub fn on_activate(mut self, action: impl Into<ControlAction>) -> Self {
        self.activation = Some(action.into());
        self
    }

    /// Disables pointer and keyboard activation and removes the link from tab traversal.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "link:{}:{}:{}:{}",
                self.label.len(),
                self.label,
                self.href.len(),
                self.href
            ))
        })
    }
}

/// Creates a platform-opened link.
#[allow(non_snake_case)]
pub fn Link(label: impl Into<SharedString>, href: impl Into<SharedString>) -> LinkView {
    LinkView::new(label, href)
}

impl View for LinkView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let style = required_style(context, "link", "body", &selector(context));
        let href = self.href.clone();
        let activation = self
            .activation
            .as_ref()
            .map(|action| context.bind_control_action(action.clone()));
        let accessibility_label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| self.label.clone());

        let typography = style
            .typography
            .as_ref()
            .expect("link theme needs typography");
        GpuiButton::new(self.identity().to_string())
            .label(self.label.clone())
            .accessibility_label(accessibility_label)
            .with_variant(ButtonVariant::Link)
            .text_color(crate::theme::registry::hsla(
                *style
                    .colors
                    .get("foreground")
                    .expect("link theme needs foreground"),
            ))
            .font_family(typography.family.clone())
            .text_size(px(typography.size))
            .font_weight(FontWeight(typography.weight))
            .disabled(self.disabled)
            .tab_stop(!self.disabled)
            .on_click(move |event, window, cx| {
                if let Some(activation) = &activation {
                    activation(event, window, cx);
                }
                cx.open_url(&href);
            })
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_defaults_enabled_and_uses_visible_label_as_accessible_name() {
        let link = Link("RustUI", "https://example.com");
        assert!(!link.disabled);
        assert_eq!(link.label.as_ref(), "RustUI");
        assert_eq!(link.accessibility_label, None);
    }

    #[test]
    fn explicit_accessible_name_and_identity_are_preserved() {
        let link = Link("Read more", "https://example.com")
            .accessibility_label("Read the RustUI overview")
            .id("overview-link");
        assert_eq!(
            link.accessibility_label.as_deref(),
            Some("Read the RustUI overview")
        );
        assert_eq!(link.identity().as_str(), "overview-link");
    }

    #[test]
    fn fallback_identity_is_stable_and_uses_both_label_and_url() {
        let first = Link("RustUI", "https://example.com").identity();
        let same = Link("RustUI", "https://example.com").identity();
        let different_url = Link("RustUI", "https://example.org").identity();
        let different_label = Link("Rust", "https://example.com").identity();

        assert_eq!(first, same);
        assert_ne!(first, different_url);
        assert_ne!(first, different_label);
    }

    #[test]
    fn disabled_state_is_configurable() {
        let link = Link("RustUI", "https://example.com").disabled(true);
        assert!(link.disabled);
    }
}
