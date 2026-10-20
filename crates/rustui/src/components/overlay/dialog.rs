//! Trigger-driven modal dialog backed by GPUI Components.

use std::rc::Rc;

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement as _, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, Window, div,
};
use gpui_component::{
    WindowExt as _,
    button::{Button as NativeButton, ButtonVariant, ButtonVariants as _},
    dialog::{DialogAction, DialogClose, DialogFooter},
};
use rustui_core::Identity;

use crate::{
    components::basic::button::Button,
    runtime::AppAction,
    view::{AnyView, ControlAction, EmptyView, RenderContext, RetentionScope, View},
};

#[derive(Clone)]
enum DialogText {
    Literal(SharedString),
    Message(SharedString),
}

impl DialogText {
    fn resolve(&self, context: &RenderContext<'_>) -> SharedString {
        match self {
            Self::Literal(text) => text.clone(),
            Self::Message(key) => SharedString::from(context.localized_message(key.as_ref())),
        }
    }
}

/// Keeps arbitrary RustUI dialog content owned across native builder rerenders.
struct DialogContent {
    content: Rc<AnyView>,
    accessibility_id: SharedString,
    accessibility_label: SharedString,
}

fn update_dialog_accessibility_label(content: &mut DialogContent, label: &SharedString) {
    content.accessibility_label = label.clone();
}

impl View for DialogContent {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        // GPUI Components does not expose its native Dialog surface element.
        // Keep a semantic dialog node around the RustUI body; this cannot
        // assign a role or contain focus on the upstream Dialog host.
        div()
            .id(self.accessibility_id.clone())
            .role(Role::Dialog)
            .aria_label(self.accessibility_label.clone())
            .child(self.content.into_element(context))
            .into_any_element()
    }
}

/// A trigger that opens one native modal dialog in its window.
///
/// The active GPUI Components dialog owns the content host until native
/// dismissal. The trigger does nothing while another dialog is active in the
/// same window. This first slice has no controlled visibility or owner-driven
/// close operation; native dismissals close the topmost dialog.
pub struct DialogView {
    id: Identity,
    trigger: DialogText,
    title: Option<DialogText>,
    accessibility_label: Option<DialogText>,
    content: Option<Rc<AnyView>>,
    confirm_label: Option<DialogText>,
    cancel_label: Option<DialogText>,
    on_confirm: Option<ControlAction>,
    on_cancel: Option<ControlAction>,
}

impl DialogView {
    pub fn new(id: impl Into<Identity>, trigger: impl Into<SharedString>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Dialog identity must be nonempty"
        );
        Self {
            id,
            trigger: DialogText::Literal(trigger.into()),
            title: None,
            accessibility_label: None,
            content: None,
            confirm_label: None,
            cancel_label: None,
            on_confirm: None,
            on_cancel: None,
        }
    }

    /// Resolves the trigger label from the active RustUI message catalog.
    pub fn localized_trigger(mut self, message_key: impl Into<SharedString>) -> Self {
        self.trigger = DialogText::Message(message_key.into());
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(DialogText::Literal(title.into()));
        self
    }

    /// Resolves the title from the active RustUI message catalog when opened.
    pub fn localized_title(mut self, message_key: impl Into<SharedString>) -> Self {
        self.title = Some(DialogText::Message(message_key.into()));
        self
    }

    /// Names the dialog content for assistive technology.
    ///
    /// By default, the visible title is used when present; otherwise the
    /// trigger label is used.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(DialogText::Literal(label.into()));
        self
    }

    /// Resolves the dialog's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_key: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(DialogText::Message(message_key.into()));
        self
    }

    fn accessibility_label_text(&self) -> &DialogText {
        self.accessibility_label
            .as_ref()
            .or(self.title.as_ref())
            .unwrap_or(&self.trigger)
    }

    /// Adds arbitrary RustUI content to the dialog body.
    pub fn content<V: View>(mut self, content: V) -> Self {
        self.content = Some(Rc::new(AnyView::new(content)));
        self
    }

    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = Some(DialogText::Literal(label.into()));
        self
    }

    /// Resolves the confirm label from the active RustUI message catalog.
    pub fn localized_confirm_label(mut self, message_key: impl Into<SharedString>) -> Self {
        self.confirm_label = Some(DialogText::Message(message_key.into()));
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = Some(DialogText::Literal(label.into()));
        self
    }

    /// Resolves the cancel label from the active RustUI message catalog.
    pub fn localized_cancel_label(mut self, message_key: impl Into<SharedString>) -> Self {
        self.cancel_label = Some(DialogText::Message(message_key.into()));
        self
    }

    /// Routes the native confirm action to this RustUI control action.
    pub fn on_confirm(mut self, action: impl Into<ControlAction>) -> Self {
        self.on_confirm = Some(action.into());
        self
    }

    /// Routes the native cancel action to this RustUI control action.
    pub fn on_cancel(mut self, action: impl Into<ControlAction>) -> Self {
        self.on_cancel = Some(action.into());
        self
    }
}

/// Creates a trigger-driven native modal dialog.
#[allow(non_snake_case)]
pub fn Dialog(id: impl Into<Identity>, trigger: impl Into<SharedString>) -> DialogView {
    DialogView::new(id, trigger)
}

impl View for DialogView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Key(self.id.clone()), |context| {
            let accessibility_label = self.accessibility_label_text().resolve(context);
            let content = self.content.as_ref().map(|content| {
                context
                    .retained_component_with_props(
                        Identity::from(format!("{}:content", self.id)),
                        || DialogContent {
                            content: Rc::clone(content),
                            accessibility_id: format!("{}:dialog", self.id).into(),
                            accessibility_label: accessibility_label.clone(),
                        },
                        &accessibility_label,
                        update_dialog_accessibility_label,
                    )
                    .0
            });
            let trigger = self.trigger.resolve(context);
            let title = self.title.as_ref().map(|title| title.resolve(context));
            let confirm_label = self.confirm_label.as_ref().map_or_else(
                || SharedString::from(context.localized_message("rustui-action-confirm")),
                |label| label.resolve(context),
            );
            let cancel_label = self.cancel_label.as_ref().map_or_else(
                || SharedString::from(context.localized_message("rustui-action-cancel")),
                |label| label.resolve(context),
            );
            let confirm_handler = self
                .on_confirm
                .clone()
                .map(|action| Rc::new(context.bind_control_action(action)));
            let cancel_handler = self
                .on_cancel
                .clone()
                .map(|action| Rc::new(context.bind_control_action(action)));
            let id = self.id.clone();

            let open = AppAction::with_window_action(move |_, window: &mut Window, cx| {
                if window.has_active_dialog(cx) {
                    return;
                }

                let content = content.clone();
                let title = title.clone();
                let confirm_label = confirm_label.clone();
                let cancel_label = cancel_label.clone();
                let confirm_handler = confirm_handler.clone();
                let cancel_handler = cancel_handler.clone();
                let confirm_button_id = format!("{}:confirm", id);

                window.open_dialog(cx, move |mut dialog, _, _| {
                    if let Some(title) = title.clone() {
                        dialog = dialog.title(title);
                    }

                    if let Some(body) = content.clone() {
                        dialog = dialog.content(move |dialog_content, _, _| {
                            dialog_content.child(body.clone())
                        });
                    }

                    let cancel_button =
                        DialogClose::new().trigger(|button| button.label(cancel_label.clone()));
                    let confirm_button = DialogAction::new().child(
                        NativeButton::new(confirm_button_id.clone())
                            .label(confirm_label.clone())
                            .with_variant(ButtonVariant::Primary),
                    );
                    dialog = dialog.footer(
                        DialogFooter::new()
                            .child(cancel_button)
                            .child(confirm_button),
                    );

                    if let Some(handler) = confirm_handler.clone() {
                        dialog = dialog.on_ok(move |event, window, cx| {
                            handler(event, window, cx);
                            true
                        });
                    }
                    if let Some(handler) = cancel_handler.clone() {
                        dialog = dialog.on_cancel(move |event, window, cx| {
                            handler(event, window, cx);
                            true
                        });
                    }
                    dialog
                });
            });

            Button(trigger, open)
                .id(format!("{}:trigger", self.id))
                .into_element(context)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_requires_a_nonempty_stable_identity() {
        assert_eq!(Dialog("settings", "Open").id.as_str(), "settings");
        assert!(std::panic::catch_unwind(|| Dialog(" ", "Open")).is_err());
    }

    #[test]
    fn native_actions_are_optional_and_content_can_be_omitted() {
        let dialog = Dialog("confirm", "Open")
            .on_confirm(AppAction::new(|_| {}))
            .on_cancel(AppAction::new(|_| {}));

        assert!(dialog.content.is_none());
        assert!(dialog.on_confirm.is_some());
        assert!(dialog.on_cancel.is_some());
    }

    #[test]
    fn semantic_dialog_content_has_a_stable_id_and_accessible_name() {
        let content = DialogContent {
            content: Rc::new(AnyView::new(EmptyView)),
            accessibility_id: "confirm:dialog".into(),
            accessibility_label: "Confirm changes".into(),
        };

        assert_eq!(content.accessibility_id.as_ref(), "confirm:dialog");
        assert_eq!(content.accessibility_label.as_ref(), "Confirm changes");
    }

    #[test]
    fn retained_dialog_accessible_name_tracks_catalog_changes() {
        let mut content = DialogContent {
            content: Rc::new(AnyView::new(EmptyView)),
            accessibility_id: "confirm:dialog".into(),
            accessibility_label: "Confirm changes".into(),
        };

        update_dialog_accessibility_label(&mut content, &"Confirmer les changements".into());

        assert_eq!(
            content.accessibility_label.as_ref(),
            "Confirmer les changements"
        );
    }

    #[test]
    fn accessible_name_prefers_explicit_label_then_title_then_trigger() {
        let trigger = Dialog("confirm", "Open").accessibility_label("Explicit name");
        assert!(matches!(
        trigger.accessibility_label_text(),
        DialogText::Literal(label) if label.as_ref() == "Explicit name"
        ));

        let localized = trigger.localized_accessibility_label("dialog.confirm");
        assert!(matches!(
        localized.accessibility_label_text(),
        DialogText::Message(key) if key.as_ref() == "dialog.confirm"
        ));

        let titled = Dialog("titled", "Open").title("Visible title");
        assert!(matches!(
        titled.accessibility_label_text(),
        DialogText::Literal(label) if label.as_ref() == "Visible title"
        ));

        let untitled = Dialog("untitled", "Open");
        assert!(matches!(
        untitled.accessibility_label_text(),
        DialogText::Literal(label) if label.as_ref() == "Open"
        ));
    }
}
