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
    accessibility_label: Option<SharedString>,
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
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Disclosure label must not be blank"
        );
        Self {
            id,
            label,
            accessibility_label: None,
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

    fn resolved_accessibility_label(&self) -> SharedString {
        self.accessibility_label
            .clone()
            .unwrap_or_else(|| self.label.clone())
    }

    /// Overrides the visible label used as the trigger's accessible name.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Disclosure accessibility label must not be blank"
        );
        self.accessibility_label = Some(label);
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
        let accessibility_label = self.resolved_accessibility_label();
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
                accessibility_label,
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
    use super::{Disclosure, DisclosureView, toggle_open_if_enabled};
    use crate::view::EmptyView;
    use rustui_core::Binding;

    fn disclosure() -> DisclosureView<EmptyView> {
        Disclosure(
            "details",
            "Details",
            Binding::new(
                false,
                |value: &bool| *value,
                |value: &mut bool, next| *value = next,
            ),
            EmptyView,
        )
    }

    #[test]
    fn visible_label_is_the_default_accessible_name() {
        assert_eq!(disclosure().resolved_accessibility_label(), "Details");
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text() {
        assert_eq!(
            disclosure()
                .accessibility_label(" Additional details ")
                .accessibility_label
                .as_deref(),
            Some(" Additional details ")
        );
    }

    #[test]
    #[should_panic(expected = "Disclosure accessibility label must not be blank")]
    fn blank_accessible_name_is_rejected() {
        disclosure().accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "Disclosure label must not be blank")]
    fn blank_visible_label_is_rejected() {
        Disclosure(
            "details",
            " \t ",
            Binding::new(
                false,
                |value: &bool| *value,
                |value: &mut bool, next| *value = next,
            ),
            EmptyView,
        );
    }

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
