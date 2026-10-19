//! Controlled Popover over the pinned GPUI Components presentation.

use std::{cell::Cell, rc::Rc};

use gpui::{AnyElement, IntoElement, ParentElement as _, SharedString, Styled as _, div};
use gpui_component::popover::Popover as GpuiPopover;
use rustui_core::{Binding, Identity};

use crate::{
    components::basic::button::{ButtonStyle, themed_button},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

fn set_open_if_changed(current: &mut bool, requested: bool) -> bool {
    if *current == requested {
        return false;
    }
    *current = requested;
    true
}

/// A labeled trigger and arbitrary RustUI content with controlled open state.
pub struct PopoverView<V> {
    id: Identity,
    trigger_label: SharedString,
    localized_label: Option<SharedString>,
    is_open: Binding<bool>,
    content: V,
}

impl<V: View> PopoverView<V> {
    pub fn new(
        id: impl Into<Identity>,
        trigger_label: impl Into<SharedString>,
        is_open: Binding<bool>,
        content: V,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Popover identity must be nonempty"
        );
        Self {
            id,
            trigger_label: trigger_label.into(),
            localized_label: None,
            is_open,
            content,
        }
    }

    /// Resolve the trigger label from the active RustUI message catalog.
    pub fn localized_label(mut self, message_key: impl Into<SharedString>) -> Self {
        self.localized_label = Some(message_key.into());
        self
    }
}

#[allow(non_snake_case)]
pub fn Popover<V: View>(
    id: impl Into<Identity>,
    trigger_label: impl Into<SharedString>,
    is_open: Binding<bool>,
    content: V,
) -> PopoverView<V> {
    PopoverView::new(id, trigger_label, is_open, content)
}

impl<V: View> View for PopoverView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Popover(self.id.clone()), |context| {
            let open = self.is_open.value();
            let label = self.localized_label.as_ref().map_or_else(
                || self.trigger_label.clone(),
                |key| SharedString::from(context.localized_message(key.as_ref())),
            );
            let id = self.id.to_string();
            let trigger = themed_button(
                context,
                SharedString::from(format!("{id}:trigger")),
                label.clone(),
                label,
                ButtonStyle::Automatic,
                false,
                0,
                true,
            );
            let requested = Rc::new(Cell::new(open));
            let requested_for_update = requested.clone();
            let update = context.binding_updater(self.is_open.clone(), move |current| {
                set_open_if_changed(current, requested_for_update.get())
            });
            let mut popover = GpuiPopover::new(id)
                .trigger(trigger)
                .open(open)
                .on_open_change(move |next, _window, cx| {
                    requested.set(*next);
                    update(cx);
                });
            if open {
                popover = popover.child(self.content.into_element(context));
            }
            div().flex().child(popover).into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::set_open_if_changed;

    #[test]
    fn open_binding_changes_only_on_transition() {
        let mut open = false;
        assert!(!set_open_if_changed(&mut open, false));
        assert!(set_open_if_changed(&mut open, true));
        assert!(open);
        assert!(!set_open_if_changed(&mut open, true));
        assert!(set_open_if_changed(&mut open, false));
        assert!(!open);
    }
}
