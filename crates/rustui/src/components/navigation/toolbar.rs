//! Ordered RustUI actions in the pinned GPUI Components toolbar.

use std::collections::HashSet;

use gpui::{AnyElement, IntoElement, SharedString, Styled as _, px};
use gpui_component::toolbar::Toolbar as GpuiToolbar;
use rustui_core::Identity;

use crate::{
    components::basic::button::{ButtonStyle, themed_button},
    theme::ThemeProfile,
    theme::registry::active_theme,
    view::{ControlAction, EmptyView, RenderContext, View},
};

fn shell_metrics(profile: &ThemeProfile) -> (f32, f32) {
    (
        profile.dimensions.toolbar_height as f32,
        profile.spacing.space_1 as f32,
    )
}

enum ToolbarLabel {
    Literal(SharedString),
    Message(SharedString),
}

/// One action with a stable identity inside a toolbar.
pub struct ToolbarItem {
    id: Identity,
    label: ToolbarLabel,
    action: ControlAction,
    disabled: bool,
}

impl ToolbarItem {
    pub fn action(
        id: impl Into<Identity>,
        label: impl Into<SharedString>,
        action: impl Into<ControlAction>,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Toolbar item ID must be nonempty"
        );
        Self {
            id,
            label: ToolbarLabel::Literal(label.into()),
            action: action.into(),
            disabled: false,
        }
    }

    pub fn localized(mut self, message_id: impl Into<SharedString>) -> Self {
        self.label = ToolbarLabel::Message(message_id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// An ordered action toolbar with a caller-owned stable identity.
pub struct ToolbarView {
    id: Identity,
    items: Vec<ToolbarItem>,
    disabled: bool,
}

impl ToolbarView {
    pub fn new(id: impl Into<Identity>, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Toolbar ID must be nonempty"
        );
        let items: Vec<_> = items.into_iter().collect();
        let mut ids = HashSet::with_capacity(items.len());
        for item in &items {
            assert!(
                ids.insert(item.id.clone()),
                "duplicate Toolbar item ID: {}",
                item.id
            );
        }
        Self {
            id,
            items,
            disabled: false,
        }
    }

    /// Disables both toolbar keyboard navigation and its RustUI actions.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[allow(non_snake_case)]
pub fn Toolbar(
    id: impl Into<Identity>,
    items: impl IntoIterator<Item = ToolbarItem>,
) -> ToolbarView {
    ToolbarView::new(id, items)
}

impl View for ToolbarView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut toolbar = GpuiToolbar::new(self.id.to_string()).disabled(self.disabled);
        if let Some(theme) = active_theme(context.app()) {
            // The toolbar record has no body artwork; its profile owns shell geometry.
            let (height, spacing) = shell_metrics(&theme.profile);
            toolbar = toolbar.h(px(height)).gap(px(spacing));
        }
        for item in &self.items {
            let label = match &item.label {
                ToolbarLabel::Literal(label) => label.clone(),
                ToolbarLabel::Message(message_id) => {
                    SharedString::from(context.localized_message(message_id.as_ref()))
                }
            };
            let disabled = self.disabled || item.disabled;
            let mut button = themed_button(
                context,
                SharedString::from(format!("{}:{}", self.id, item.id)),
                label.clone(),
                label,
                ButtonStyle::Automatic,
                disabled,
                0,
                true,
            );
            if !disabled {
                button = button.on_click(context.bind_control_action(item.action.clone()));
            }
            toolbar = toolbar.child(button);
        }
        toolbar.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::AppAction;
    use crate::theme::parse_theme;

    #[test]
    fn shell_geometry_follows_macos_profile() {
        let document = parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        let record = document.component_theme("toolbar").unwrap();
        assert!(record.styles.is_empty());
        assert_eq!(
            record.metadata["metrics"]["height"],
            "dimensions.toolbar_height"
        );
        assert_eq!(shell_metrics(&document.platform_snapshot), (48.0, 4.0));
    }

    fn item(id: &str) -> ToolbarItem {
        ToolbarItem::action(id, "Action", AppAction::new(|_| {}))
    }

    #[test]
    fn identities_must_be_stable_and_unique() {
        assert!(std::panic::catch_unwind(|| item(" ")).is_err());
        assert!(std::panic::catch_unwind(|| Toolbar(" ", [item("one")])).is_err());
        assert!(std::panic::catch_unwind(|| Toolbar("bar", [item("one"), item("one")])).is_err());
    }

    #[test]
    fn preserves_action_order_and_disabled_state() {
        let bar = Toolbar("bar", [item("first"), item("second").disabled(true)]).disabled(true);
        assert_eq!(
            bar.items
                .iter()
                .map(|item| item.id.as_str())
                .collect::<Vec<_>>(),
            ["first", "second"]
        );
        assert!(bar.items[1].disabled);
        assert!(bar.disabled);
    }
}
