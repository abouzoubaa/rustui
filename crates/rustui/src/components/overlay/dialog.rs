//! Trigger-driven modal dialog backed by GPUI Components.

use std::rc::Rc;

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement as _, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _, Window, div, px,
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
    view::{AnyView, ControlAction, EmptyView, RenderContext, RenderSeed, RetentionScope, View},
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

fn first_nonblank_accessibility_label(
    labels: impl IntoIterator<Item = SharedString>,
    identity: SharedString,
) -> SharedString {
    labels
        .into_iter()
        .find(|label| !label.trim().is_empty())
        .unwrap_or(identity)
}

fn first_nonblank_trigger_label(
    key: Option<&SharedString>,
    localized: Option<SharedString>,
    fallback: SharedString,
) -> SharedString {
    localized
        .filter(|label| {
            !label.trim().is_empty() && key.is_none_or(|key| label.trim() != key.trim())
        })
        .unwrap_or(fallback)
}

fn resolved_localized_accessibility_candidate(
    key: &SharedString,
    label: SharedString,
) -> Option<SharedString> {
    (label.trim() != key.trim() && !label.trim().is_empty()).then_some(label)
}

/// Keeps arbitrary RustUI dialog content owned across native builder rerenders.
struct DialogContent {
    content: Rc<AnyView>,
    seed: RenderSeed,
    accessibility_id: SharedString,
    accessibility_label: SharedString,
}

#[derive(Clone)]
struct DialogContentProps {
    content: Rc<AnyView>,
    seed: RenderSeed,
    label: SharedString,
}

impl PartialEq for DialogContentProps {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.content, &other.content) && self.label == other.label
    }
}

fn update_dialog_content(content: &mut DialogContent, props: &DialogContentProps) {
    content.content = props.content.clone();
    content.seed = props.seed.clone();
    content.accessibility_label = props.label.clone();
}

fn dialog_semantic_element(
    accessibility_id: SharedString,
    accessibility_label: SharedString,
    content: AnyElement,
) -> impl gpui::Element + gpui::IntoElement {
    div()
        .id(accessibility_id)
        .role(Role::Dialog)
        .aria_label(accessibility_label)
        .child(content)
}

impl View for DialogContent {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        // GPUI Components does not expose its native Dialog surface element.
        // Keep a semantic dialog node around the RustUI body; this cannot
        // assign a role or contain focus on the upstream Dialog host.
        let body =
            context.with_seed_owner(&self.seed, |context| self.content.into_element(context));
        dialog_semantic_element(
            self.accessibility_id.clone(),
            self.accessibility_label.clone(),
            body,
        )
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
    localized_trigger: Option<SharedString>,
    title: Option<DialogText>,
    accessibility_label: Option<DialogText>,
    accessibility_fallback: Option<SharedString>,
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
        let trigger = trigger.into();
        assert!(
            !trigger.trim().is_empty(),
            "Dialog trigger requires a non-empty visible label"
        );
        Self {
            id,
            trigger: DialogText::Literal(trigger),
            localized_trigger: None,
            title: None,
            accessibility_label: None,
            accessibility_fallback: None,
            content: None,
            confirm_label: None,
            cancel_label: None,
            on_confirm: None,
            on_cancel: None,
        }
    }

    /// Resolves the trigger label from the active RustUI message catalog.
    pub fn localized_trigger(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Dialog trigger requires a non-empty localization key"
        );
        self.localized_trigger = Some(message_key);
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        let title = title.into();
        assert!(!title.trim().is_empty(), "Dialog title must not be blank");
        self.title = Some(DialogText::Literal(title));
        self
    }

    /// Resolves the title from the active RustUI message catalog when opened.
    pub fn localized_title(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Dialog title requires a non-empty localization key"
        );
        self.title = Some(DialogText::Message(message_key));
        self
    }

    /// Names the dialog content for assistive technology.
    ///
    /// By default, the visible title is used when present; otherwise the
    /// trigger label is used.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Dialog accessibility label must not be empty"
        );
        self.accessibility_label = Some(DialogText::Literal(label));
        self.accessibility_fallback = None;
        self
    }

    /// Resolves the dialog's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Dialog accessibility label localization key must not be empty"
        );
        if let Some(DialogText::Literal(label)) = &self.accessibility_label {
            self.accessibility_fallback = Some(label.clone());
        }
        self.accessibility_label = Some(DialogText::Message(message_key));
        self
    }

    #[cfg(test)]
    fn accessibility_label_text(&self) -> &DialogText {
        self.accessibility_label
            .as_ref()
            .or(self.title.as_ref())
            .unwrap_or(&self.trigger)
    }

    fn resolve_accessibility_label(&self, context: &RenderContext<'_>) -> SharedString {
        let trigger = self.resolve_trigger_label(context);
        let explicit = self
            .accessibility_label
            .as_ref()
            .and_then(|label| match label {
                DialogText::Literal(label) => Some(label.clone()),
                DialogText::Message(key) => {
                    resolved_localized_accessibility_candidate(key, label.resolve(context))
                        .or_else(|| self.accessibility_fallback.clone())
                }
            });
        let title = self.title.iter().filter_map(|label| {
            let resolved = label.resolve(context);
            match label {
                DialogText::Literal(_) => Some(resolved),
                DialogText::Message(key) => {
                    resolved_localized_accessibility_candidate(key, resolved)
                }
            }
        });
        first_nonblank_accessibility_label(
            explicit
                .into_iter()
                .chain(title)
                .chain(std::iter::once(trigger)),
            self.id.as_str().into(),
        )
    }

    fn resolve_trigger_label(&self, context: &RenderContext<'_>) -> SharedString {
        first_nonblank_trigger_label(
            self.localized_trigger.as_ref(),
            self.localized_trigger
                .as_ref()
                .map(|key| SharedString::from(context.localized_message(key.as_ref()))),
            self.trigger.resolve(context),
        )
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
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Dialog confirm label requires a non-empty localization key"
        );
        self.confirm_label = Some(DialogText::Message(message_key));
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = Some(DialogText::Literal(label.into()));
        self
    }

    /// Resolves the cancel label from the active RustUI message catalog.
    pub fn localized_cancel_label(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Dialog cancel label requires a non-empty localization key"
        );
        self.cancel_label = Some(DialogText::Message(message_key));
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
            let accessibility_label = self.resolve_accessibility_label(context);
            let content = self.content.as_ref().map(|content| {
                let props = DialogContentProps {
                    content: Rc::clone(content),
                    seed: context.seed(),
                    label: accessibility_label.clone(),
                };
                context
                    .retained_component_with_props(
                        Identity::from(format!("{}:content", self.id)),
                        || DialogContent {
                            content: Rc::clone(content),
                            seed: props.seed.clone(),
                            accessibility_id: format!("{}:dialog", self.id).into(),
                            accessibility_label: accessibility_label.clone(),
                        },
                        &props,
                        update_dialog_content,
                    )
                    .0
            });
            let trigger = self.resolve_trigger_label(context);
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
                if window.has_active_dialog(cx) || super::drawer::is_active(window, cx) {
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
                        // Components places its close button at the physical right edge.
                        // Reserve room there even when localized text is right-aligned.
                        dialog = dialog.title(div().pr(px(32.0)).child(title));
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
    #[should_panic(expected = "Dialog trigger requires a non-empty visible label")]
    fn blank_visible_trigger_is_rejected() {
        Dialog("settings", " \t");
    }

    #[test]
    #[should_panic(expected = "Dialog trigger requires a non-empty localization key")]
    fn blank_localized_trigger_key_is_rejected() {
        Dialog("settings", "Open").localized_trigger(" \t");
    }

    #[test]
    #[should_panic(expected = "Dialog title requires a non-empty localization key")]
    fn blank_localized_title_key_is_rejected() {
        Dialog("settings", "Open").localized_title(" \t");
    }

    #[test]
    #[should_panic(expected = "Dialog title must not be blank")]
    fn blank_literal_title_is_rejected() {
        Dialog("settings", "Open").title(" \t");
    }

    #[test]
    #[should_panic(expected = "Dialog confirm label requires a non-empty localization key")]
    fn blank_localized_confirm_label_key_is_rejected() {
        Dialog("settings", "Open").localized_confirm_label(" \t");
    }

    #[test]
    #[should_panic(expected = "Dialog cancel label requires a non-empty localization key")]
    fn blank_localized_cancel_label_key_is_rejected() {
        Dialog("settings", "Open").localized_cancel_label(" \t");
    }

    #[test]
    fn blank_localized_trigger_falls_back_to_its_visible_label() {
        assert_eq!(
            first_nonblank_trigger_label(None, Some(" \t".into()), "Open settings".into()),
            "Open settings"
        );
    }

    #[test]
    fn nonblank_localized_trigger_takes_precedence() {
        assert_eq!(
            first_nonblank_trigger_label(
                Some(&"dialog.settings".into()),
                Some("Ouvrir les réglages".into()),
                "Open settings".into()
            ),
            "Ouvrir les réglages"
        );
    }

    #[test]
    fn unresolved_localized_trigger_falls_back_to_its_visible_label() {
        let key: SharedString = "dialog.settings".into();
        assert_eq!(
            first_nonblank_trigger_label(
                Some(&key),
                Some(" \tdialog.settings\n".into()),
                "Open settings".into()
            ),
            "Open settings"
        );
    }

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
        use gpui::Element as _;
        for label in ["Confirm changes", "Confirmer les changements"] {
            let element = dialog_semantic_element(
                "confirm:dialog".into(),
                label.into(),
                div().into_any_element(),
            );
            assert_eq!(element.a11y_role(), Some(Role::Dialog));
            let mut node = gpui::accesskit::Node::new(Role::Dialog);
            element.write_a11y_info(&mut node);
            assert_eq!(node.label(), Some(label));
        }
    }

    #[gpui::test]
    fn retained_dialog_reconciles_content_and_accessible_name(cx: &mut gpui::TestAppContext) {
        use crate::runtime::component_host::RetainedEntities;
        use std::any::TypeId;

        cx.update(gpui_component::init);
        let (host, window) =
            cx.add_window_view(|_, cx| crate::runtime::ComponentHost::new(EmptyView, cx));
        window.update(|window, app| {
            let environment = rustui_core::EnvironmentValues::default();
            let mut retained = RetainedEntities::default();
            let seed = RenderContext::new(
                Rc::new(|_, _| {}),
                host.entity_id(),
                TypeId::of::<()>(),
                Rc::new(()),
                px(14.0),
                &environment,
                None,
                None,
                window,
                app,
                &mut retained,
            )
            .seed();
            let old = Rc::new(AnyView::new(EmptyView));
            let new = Rc::new(AnyView::new(EmptyView));
            let mut content = DialogContent {
                content: old.clone(),
                seed: seed.clone(),
                accessibility_id: "confirm:dialog".into(),
                accessibility_label: "Confirm changes".into(),
            };
            let props = DialogContentProps {
                content: new.clone(),
                seed,
                label: "Confirmer les changements".into(),
            };
            update_dialog_content(&mut content, &props);
            assert!(Rc::ptr_eq(&content.content, &new));
            assert_eq!(content.accessibility_label, props.label);
        });
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

    #[test]
    fn explicit_accessible_name_rejects_whitespace_only_text() {
        assert!(
            std::panic::catch_unwind(|| Dialog("confirm", "Open").accessibility_label(" \t\n "))
                .is_err()
        );
    }

    #[test]
    #[should_panic(expected = "Dialog accessibility label localization key must not be empty")]
    fn explicit_localized_accessible_name_rejects_blank_key() {
        Dialog("confirm", "Open").localized_accessibility_label(" \t\n ");
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text_verbatim() {
        let dialog = Dialog("confirm", "Open").accessibility_label(" Confirm changes ");

        assert!(matches!(
        dialog.accessibility_label_text(),
        DialogText::Literal(label) if label.as_ref() == " Confirm changes "
        ));
    }

    #[test]
    fn blank_resolved_accessible_name_falls_back_to_the_next_candidate() {
        assert_eq!(
            first_nonblank_accessibility_label(
                [" \t ".into(), "Confirm changes".into()],
                "confirm".into()
            ),
            "Confirm changes"
        );
    }

    #[test]
    fn all_blank_resolved_accessible_name_candidates_fall_back_to_identity() {
        assert_eq!(
            first_nonblank_accessibility_label([" ".into(), "\n".into()], "confirm".into()),
            "confirm"
        );
    }

    #[test]
    fn unresolved_localized_accessible_name_falls_back_to_visible_title() {
        let key: SharedString = "dialog.confirm".into();
        let unresolved = resolved_localized_accessibility_candidate(&key, key.clone());

        assert_eq!(
            first_nonblank_accessibility_label(
                unresolved.into_iter().chain(["Confirm changes".into()]),
                "confirm".into()
            ),
            "Confirm changes"
        );
    }

    #[test]
    fn padded_unresolved_localized_accessible_name_falls_back_to_visible_title() {
        let key: SharedString = "dialog.confirm".into();
        let unresolved =
            resolved_localized_accessibility_candidate(&key, " dialog.confirm ".into());

        assert_eq!(
            first_nonblank_accessibility_label(
                unresolved.into_iter().chain(["Confirm changes".into()]),
                "confirm".into()
            ),
            "Confirm changes"
        );
    }
}
