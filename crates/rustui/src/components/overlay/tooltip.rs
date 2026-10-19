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
    Message(SharedString),
}

/// Adds descriptive tooltip text to a child view without changing its accessible name.
pub struct TooltipView<V: View> {
    text: TooltipText,
    content: V,
    id: Option<Identity>,
}

impl<V: View> TooltipView<V> {
    pub fn new(text: impl Into<SharedString>, content: V) -> Self {
        Self {
            text: TooltipText::Literal(text.into()),
            content,
            id: None,
        }
    }

    /// Resolves the tooltip text from the active RustUI message catalogs.
    pub fn localized(mut self, identifier: impl Into<SharedString>) -> Self {
        self.text = TooltipText::Message(identifier.into());
        self
    }

    /// Sets the required stable identity on the tooltip wrapper.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
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
            TooltipText::Message(identifier) => {
                SharedString::from(context.localized_message(identifier.as_ref()))
            }
        };
        let content = self.content.into_element(context);
        let id = self.id.as_ref().unwrap_or_else(|| {
            panic!("Tooltip requires an explicit stable identity; add `.id(\"tooltip-name\")`")
        });
        div()
            .id(id.to_string())
            .child(content)
            .self_start()
            .tooltip(move |window, cx| NativeTooltip::new(text.clone()).build(window, cx))
            .into_any_element()
    }
}
