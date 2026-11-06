//! Descriptive hover and long-press text for a RustUI child view.

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div,
};
use gpui_component::tooltip::Tooltip as NativeTooltip;
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, View};

enum TooltipText {
    Literal(SharedString),
    Message {
        identifier: SharedString,
        fallback: SharedString,
    },
}

fn resolved_tooltip_text(
    identifier: &SharedString,
    resolved: SharedString,
    fallback: &SharedString,
) -> SharedString {
    if resolved.trim().is_empty() || resolved.trim() == identifier.trim() {
        fallback.clone()
    } else {
        resolved
    }
}

/// Adds descriptive tooltip text to a child view without changing its accessible name.
pub struct TooltipView<V: View> {
    text: TooltipText,
    content: V,
    id: Option<Identity>,
}

impl<V: View> TooltipView<V> {
    pub fn new(text: impl Into<SharedString>, content: V) -> Self {
        let text = text.into();
        assert!(!text.trim().is_empty(), "Tooltip text must be nonempty");
        Self {
            text: TooltipText::Literal(text),
            content,
            id: None,
        }
    }

    /// Resolves the tooltip text from the active RustUI message catalogs.
    pub fn localized(mut self, identifier: impl Into<SharedString>) -> Self {
        let identifier = identifier.into();
        assert!(
            !identifier.trim().is_empty(),
            "Tooltip localization key must be nonempty"
        );
        let fallback = match &self.text {
            TooltipText::Literal(text) => text.clone(),
            TooltipText::Message { fallback, .. } => fallback.clone(),
        };
        self.text = TooltipText::Message {
            identifier,
            fallback,
        };
        self
    }

    /// Sets the required stable identity on the tooltip wrapper.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Tooltip identity must be nonempty"
        );
        self.id = Some(id);
        self
    }
}

/// Adds descriptive tooltip text to a RustUI child view.
#[allow(non_snake_case)]
pub fn Tooltip<V: View>(text: impl Into<SharedString>, content: V) -> TooltipView<V> {
    TooltipView::new(text, content)
}

impl<V: View> View for TooltipView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let text = match &self.text {
            TooltipText::Literal(text) => text.clone(),
            TooltipText::Message {
                identifier,
                fallback,
            } => resolved_tooltip_text(
                identifier,
                context.localized_message(identifier.as_ref()).into(),
                fallback,
            ),
        };
        let content = self.content.into_element(context);
        let id = self.id.as_ref().unwrap_or_else(|| {
            panic!("Tooltip requires an explicit stable identity; add `.id(\"tooltip-name\")`")
        });
        tooltip_trigger(content, id.to_string(), text).into_any_element()
    }
}

fn tooltip_trigger(content: AnyElement, id: String, description: SharedString) -> impl IntoElement {
    let popup_text = description.clone();
    div()
        .id(id)
        .role(gpui::accesskit::Role::Group)
        .aria_description(description)
        .child(content)
        .self_start()
        .tooltip(move |window, cx| NativeTooltip::new(popup_text.clone()).build(window, cx))
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{Element as _, accesskit::Node};

    #[test]
    #[should_panic(expected = "Tooltip identity must be nonempty")]
    fn blank_identity_is_rejected() {
        let _ = Tooltip("Help", EmptyView).id(" \t ");
    }

    #[test]
    #[should_panic(expected = "Tooltip localization key must be nonempty")]
    fn blank_localization_key_is_rejected() {
        let _ = Tooltip("Help", EmptyView).localized(" \t ");
    }

    #[test]
    fn untranslated_localized_tooltip_keeps_its_original_literal_fallback() {
        let tooltip = Tooltip("Helpful details", EmptyView)
            .localized("tooltip.first")
            .localized("tooltip.help");
        let TooltipText::Message {
            identifier,
            fallback,
        } = tooltip.text
        else {
            panic!("expected a localized tooltip");
        };

        assert_eq!(
            resolved_tooltip_text(&identifier, " tooltip.help ".into(), &fallback),
            "Helpful details"
        );
        assert_eq!(
            resolved_tooltip_text(&identifier, "Détails utiles".into(), &fallback),
            "Détails utiles"
        );
    }

    #[test]
    fn tooltip_text_is_exposed_as_trigger_description() {
        let element = tooltip_trigger(
            div().into_any_element(),
            "help".to_owned(),
            "Helpful details".into(),
        )
        .into_element();

        assert_eq!(element.a11y_role(), Some(gpui::accesskit::Role::Group));
        let mut node = Node::new(gpui::accesskit::Role::Group);
        element.write_a11y_info(&mut node);
        assert_eq!(node.description(), Some("Helpful details"));
    }
}
