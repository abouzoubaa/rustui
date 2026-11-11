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

fn first_nonblank_trigger_label(labels: impl IntoIterator<Item = SharedString>) -> SharedString {
    labels
        .into_iter()
        .find(|label| !label.trim().is_empty())
        .expect("Popover trigger accessible name must not be blank")
}

fn resolved_localized_trigger_label(
    key: &SharedString,
    localized: SharedString,
) -> Option<SharedString> {
    (localized.trim() != key.trim() && !localized.trim().is_empty()).then_some(localized)
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
        let trigger_label = trigger_label.into();
        assert!(
            !trigger_label.trim().is_empty(),
            "Popover trigger requires a non-empty visible label"
        );
        Self {
            id,
            trigger_label,
            localized_label: None,
            is_open,
            content,
        }
    }

    /// Resolve the trigger label from the active RustUI message catalog.
    pub fn localized_label(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Popover trigger requires a non-empty localization key"
        );
        self.localized_label = Some(message_key);
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
            let localized_label = self.localized_label.as_ref().and_then(|key| {
                resolved_localized_trigger_label(
                    key,
                    SharedString::from(context.localized_message(key.as_ref())),
                )
            });
            let label = first_nonblank_trigger_label([
                localized_label.unwrap_or_default(),
                self.trigger_label.clone(),
            ]);
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
    use super::{
        first_nonblank_trigger_label, resolved_localized_trigger_label, set_open_if_changed,
    };
    use crate::components::basic::text::Text;

    fn read(_: &PopoverOwner) -> bool {
        false
    }

    fn write(_: &mut PopoverOwner, _: bool) {}

    struct PopoverOwner;

    #[test]
    #[should_panic(expected = "Popover trigger requires a non-empty visible label")]
    fn blank_visible_trigger_labels_are_rejected() {
        super::Popover(
            "details",
            " \t",
            rustui_core::Binding::new(false, read, write),
            Text("Body"),
        );
    }

    #[test]
    #[should_panic(expected = "Popover trigger requires a non-empty localization key")]
    fn blank_localization_keys_are_rejected() {
        super::Popover(
            "details",
            "Details",
            rustui_core::Binding::new(false, read, write),
            Text("Body"),
        )
        .localized_label(" \t");
    }

    #[test]
    fn blank_localized_trigger_label_falls_back_to_visible_label() {
        assert_eq!(
            first_nonblank_trigger_label([" \t".into(), "Details".into()]),
            "Details"
        );
    }

    #[test]
    fn unresolved_localized_trigger_label_falls_back_to_visible_label() {
        let key: gpui::SharedString = "popover.details".into();
        assert_eq!(
            resolved_localized_trigger_label(&key, key.clone()).unwrap_or_default(),
            ""
        );
        assert_eq!(
            first_nonblank_trigger_label([
                resolved_localized_trigger_label(&key, key.clone()).unwrap_or_default(),
                "Details".into(),
            ]),
            "Details"
        );
    }

    #[test]
    fn padded_localized_trigger_key_echo_falls_back_to_visible_label() {
        let key: gpui::SharedString = "popover.details".into();
        let echoed_key: gpui::SharedString = " \tpopover.details \t".into();
        assert_eq!(
            resolved_localized_trigger_label(&key, echoed_key).unwrap_or_default(),
            ""
        );
        assert_eq!(
            first_nonblank_trigger_label([
                resolved_localized_trigger_label(&key, " \tpopover.details \t".into())
                    .unwrap_or_default(),
                "Details".into(),
            ]),
            "Details"
        );
    }

    #[test]
    fn resolved_localized_trigger_label_is_preserved() {
        let key: gpui::SharedString = "popover.details".into();
        assert_eq!(
            resolved_localized_trigger_label(&key, "Détails".into()),
            Some("Détails".into())
        );
    }

    #[test]
    #[should_panic(expected = "Popover trigger accessible name must not be blank")]
    fn all_blank_trigger_labels_are_rejected() {
        first_nonblank_trigger_label([" ".into(), "\n".into()]);
    }

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
