//! Controlled disclosure with a native button and Collapsible content.

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, div,
};
use gpui_component::collapsible::Collapsible as GpuiCollapsible;
use rustui_core::{Binding, Identity};

use crate::{
    components::{
        basic::button::{ButtonStyle, themed_button},
        theme::{required_style, selector, styled_element},
    },
    view::{EmptyView, RenderContext, RetentionScope, View},
};

fn toggle_open_if_enabled(open: &mut bool, disabled: bool) -> bool {
    if disabled {
        return false;
    }
    *open = !*open;
    true
}

/// A labeled, controlled disclosure whose hidden content is unmounted.
pub struct DisclosureView<V> {
    id: Identity,
    label: SharedString,
    is_open: Binding<bool>,
    content: V,
    disabled: bool,
}

impl<V: View> DisclosureView<V> {
    pub fn new(
        id: impl Into<Identity>,
        label: impl Into<SharedString>,
        is_open: Binding<bool>,
        content: V,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Disclosure identity must be nonempty"
        );
        Self {
            id,
            label: label.into(),
            is_open,
            content,
            disabled: false,
        }
    }

    /// Prevents user activation while preserving externally controlled state.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[allow(non_snake_case)]
pub fn Disclosure<V: View>(
    id: impl Into<Identity>,
    label: impl Into<SharedString>,
    is_open: Binding<bool>,
    content: V,
) -> DisclosureView<V> {
    DisclosureView::new(id, label, is_open, content)
}

impl<V: View> View for DisclosureView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let style = required_style(
            context,
            "disclosure_control",
            "disclosure",
            &selector(context),
        );
        let open = self.is_open.value();
        let label = self.label.clone();
        let visible_label =
            SharedString::from(format!("{} {}", if open { "▾" } else { "▸" }, label));
        let disabled = self.disabled;
        let update = context.binding_updater(self.is_open.clone(), move |current| {
            toggle_open_if_enabled(current, disabled)
        });
        let trigger = styled_element(
            themed_button(
                context,
                SharedString::from(format!("{}:trigger", self.id)),
                visible_label,
                label,
                ButtonStyle::Plain,
                disabled,
                0,
                true,
            ),
            &style,
        )
        .on_click(move |_, _, cx| {
            if !disabled {
                update(cx);
            }
        });
        let mut disclosure = GpuiCollapsible::new().open(open).child(
            div()
                .id(format!("{}:disclosure", self.id))
                .role(Role::Group)
                .aria_expanded(open)
                .flex()
                .child(trigger),
        );
        if open {
            let content = context.with_element_scope(RetentionScope::Key(self.id.clone()), |cx| {
                self.content.into_element(cx)
            });
            disclosure = disclosure.content(content);
        }
        disclosure.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::toggle_open_if_enabled;

    #[test]
    fn disabled_disclosure_does_not_mutate_open_state() {
        let mut open = false;
        assert!(!toggle_open_if_enabled(&mut open, true));
        assert!(!open);
        assert!(toggle_open_if_enabled(&mut open, false));
        assert!(open);
        assert!(toggle_open_if_enabled(&mut open, false));
        assert!(!open);
    }
}
