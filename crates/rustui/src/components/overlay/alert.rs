//! Inline alerts backed by GPUI Components' alert element.

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, px,
};
use gpui_component::alert::{Alert as NativeAlert, AlertVariant as NativeAlertVariant};
use rustui_core::{Binding, Identity};

use crate::{
    components::basic::button::{Button, ButtonStyle},
    runtime::AppAction,
    view::{EmptyView, RenderContext, View},
};

#[derive(Clone)]
enum AlertText {
    Literal(SharedString),
    Message(SharedString),
}

/// Semantic appearance for an inline alert.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum AlertVariant {
    #[default]
    Default,
    Info,
    Success,
    Warning,
    Error,
}

impl AlertVariant {
    fn native(self) -> NativeAlertVariant {
        match self {
            Self::Default => NativeAlertVariant::Default,
            Self::Info => NativeAlertVariant::Info,
            Self::Success => NativeAlertVariant::Success,
            Self::Warning => NativeAlertVariant::Warning,
            Self::Error => NativeAlertVariant::Error,
        }
    }
}

fn close_if_open(visible: &mut bool) -> bool {
    if !*visible {
        return false;
    }
    *visible = false;
    true
}

fn require_alert_message(message: SharedString) -> SharedString {
    assert!(
        !message.trim().is_empty(),
        "Alert requires a non-empty accessible message"
    );
    message
}

fn alert_message_with_fallback(
    source: &AlertText,
    message: SharedString,
    title_source: Option<&AlertText>,
    title: Option<&SharedString>,
    id: &Identity,
) -> SharedString {
    let message_is_unresolved =
        matches!(source, AlertText::Message(key) if key.as_ref() == message.as_ref());
    let title_is_unresolved = matches!(
        (title_source, title),
        (Some(AlertText::Message(key)), Some(resolved)) if key.as_ref() == resolved.as_ref()
    );
    std::iter::once((message, message_is_unresolved))
        .chain(title.cloned().map(|title| (title, title_is_unresolved)))
        .chain(std::iter::once((SharedString::from(id.to_string()), false)))
        .find(|(text, unresolved)| !unresolved && !text.trim().is_empty())
        .map(|(text, _)| text)
        .expect("Alert identity must be nonempty")
}

fn alert_accessible_title(
    title: Option<&AlertText>,
    resolved: Option<&SharedString>,
) -> Option<SharedString> {
    let (Some(title), Some(resolved)) = (title, resolved) else {
        return None;
    };
    let is_unresolved_key = match title {
        AlertText::Literal(_) => false,
        AlertText::Message(key) => key.as_ref() == resolved.as_ref(),
    };
    (!is_unresolved_key && !resolved.trim().is_empty()).then(|| resolved.clone())
}

fn alert_close_label(localized: SharedString) -> SharedString {
    let trimmed = localized.trim();
    if trimmed.is_empty() || trimmed == "rustui-action-close" {
        "Close".into()
    } else {
        localized
    }
}

/// An inline semantic message with owner-controlled visibility.
pub struct AlertView {
    id: Identity,
    message: AlertText,
    title: Option<AlertText>,
    variant: AlertVariant,
    visible: Binding<bool>,
}

impl AlertView {
    pub fn new(
        id: impl Into<Identity>,
        message: impl Into<SharedString>,
        visible: Binding<bool>,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Alert identity must be nonempty"
        );
        let message = require_alert_message(message.into());
        Self {
            id,
            message: AlertText::Literal(message),
            title: None,
            variant: AlertVariant::default(),
            visible,
        }
    }

    /// Sets the optional title shown above the message.
    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(AlertText::Literal(title.into()));
        self
    }

    /// Resolves the title from the active RustUI message catalog.
    pub fn localized_title(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Alert requires a non-empty title localization key"
        );
        self.title = Some(AlertText::Message(message_key));
        self
    }

    /// Resolves the message from the active RustUI message catalog.
    pub fn localized_message(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Alert requires a non-empty localization key"
        );
        self.message = AlertText::Message(message_key);
        self
    }

    /// Selects the semantic appearance provided by GPUI Components.
    pub fn variant(mut self, variant: AlertVariant) -> Self {
        self.variant = variant;
        self
    }
}

/// Creates an inline alert with a stable identity and owner-controlled visibility.
#[allow(non_snake_case)]
pub fn Alert(
    id: impl Into<Identity>,
    message: impl Into<SharedString>,
    visible: Binding<bool>,
) -> AlertView {
    AlertView::new(id, message, visible)
}

impl View for AlertView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let message = match &self.message {
            AlertText::Literal(message) => message.clone(),
            AlertText::Message(key) => SharedString::from(context.localized_message(key.as_ref())),
        };
        let title = self.title.as_ref().map(|title| match title {
            AlertText::Literal(title) => title.clone(),
            AlertText::Message(key) => SharedString::from(context.localized_message(key.as_ref())),
        });
        let message = alert_message_with_fallback(
            &self.message,
            message,
            self.title.as_ref(),
            title.as_ref(),
            &self.id,
        );
        let accessibility_title = alert_accessible_title(self.title.as_ref(), title.as_ref());
        let accessibility_label = accessibility_title.as_ref().unwrap_or(&message).clone();
        let accessibility_description = accessibility_title.map(|_| message.clone());
        let visible = self.visible.value();
        let update = context.binding_updater(self.visible.clone(), close_if_open);
        let mut alert = NativeAlert::new(self.id.to_string(), message)
            .with_variant(self.variant.native())
            .visible(visible);
        if let Some(title) = title {
            alert = alert.title(title);
        }
        if !visible {
            return alert.into_any_element();
        }

        let close_label =
            alert_close_label(context.localized_message("rustui-action-close").into());
        let close = Button(close_label.clone(), AppAction::new(move |cx| update(cx)))
            .id(format!("{}:close", self.id))
            .accessibility_label(close_label)
            .button_style(ButtonStyle::Plain)
            .into_element(context);

        let mut alert_group = div()
            .id(format!("{}:group", self.id))
            .role(gpui::Role::Group)
            .aria_label(accessibility_label)
            .flex()
            .flex_col()
            .gap(px(4.0))
            .w_full()
            .child(alert)
            .child(div().flex().justify_end().child(close));
        if let Some(description) = accessibility_description {
            alert_group = alert_group.aria_description(description);
        }
        alert_group.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Alert requires a non-empty accessible message")]
    fn blank_alert_messages_are_rejected() {
        Alert("network-alert", " \t ", Binding::new(false, read, write));
    }

    #[test]
    #[should_panic(expected = "Alert requires a non-empty localization key")]
    fn blank_localized_message_keys_are_rejected() {
        Alert(
            "network-alert",
            "Connection lost",
            Binding::new(false, read, write),
        )
        .localized_message(" \t ");
    }

    #[test]
    fn empty_resolved_message_falls_back_to_resolved_title() {
        let id: Identity = "network-alert".into();
        assert_eq!(
            alert_message_with_fallback(
                &AlertText::Literal(" \t ".into()),
                " \t ".into(),
                Some(&AlertText::Literal("Offline".into())),
                Some(&"Offline".into()),
                &id,
            ),
            "Offline"
        );
    }

    #[test]
    fn unresolved_localized_message_falls_back_without_exposing_key_as_name() {
        let id: Identity = "network-alert".into();
        let key: SharedString = "network.body".into();
        assert_eq!(
            alert_message_with_fallback(
                &AlertText::Message(key.clone()),
                key.clone(),
                Some(&AlertText::Literal("Offline".into())),
                Some(&"Offline".into()),
                &id,
            ),
            "Offline"
        );
        assert_eq!(
            alert_message_with_fallback(
                &AlertText::Message(key.clone()),
                key.clone(),
                None,
                None,
                &id,
            ),
            "network-alert"
        );
    }

    #[test]
    fn unresolved_localized_title_does_not_name_alert_with_translation_key() {
        let key: SharedString = "network.title".into();
        assert_eq!(
            alert_accessible_title(Some(&AlertText::Message(key.clone())), Some(&key)),
            None
        );
        assert_eq!(
            alert_accessible_title(
                Some(&AlertText::Message(key)),
                Some(&"Connexion perdue".into())
            ),
            Some("Connexion perdue".into())
        );
    }

    #[test]
    fn all_blank_resolved_text_falls_back_to_validated_identity() {
        let id: Identity = "network-alert".into();
        let blank_title: SharedString = " \t ".into();
        assert_eq!(
            alert_message_with_fallback(
                &AlertText::Literal("\n".into()),
                "\n".into(),
                Some(&AlertText::Literal(blank_title.clone())),
                Some(&blank_title),
                &id,
            ),
            "network-alert"
        );
    }

    #[test]
    fn blank_resolved_close_label_uses_nonblank_fallback() {
        assert_eq!(alert_close_label(" \t ".into()), "Close");
        assert_eq!(alert_close_label("Fermer".into()), "Fermer");
    }

    #[test]
    fn alert_defaults_and_builders_preserve_contract() {
        let alert = Alert(
            "network-alert",
            "Connection lost",
            Binding::new(false, read, write),
        )
        .title("Offline")
        .variant(AlertVariant::Warning);
        assert_eq!(alert.id.as_str(), "network-alert");
        assert!(matches!(alert.message, AlertText::Literal(_)));
        assert!(matches!(alert.title, Some(AlertText::Literal(_))));
        assert_eq!(alert.variant, AlertVariant::Warning);
        assert!(!alert.visible.value());
    }

    #[test]
    fn localized_content_uses_message_keys() {
        let alert = Alert(
            "localized-alert",
            "ignored",
            Binding::new(false, read, write),
        )
        .localized_title("network.title")
        .localized_message("network.body");
        assert!(
            matches!(alert.title, Some(AlertText::Message(key)) if key.as_ref() == "network.title")
        );
        assert!(matches!(alert.message, AlertText::Message(key) if key.as_ref() == "network.body"));
    }

    #[test]
    fn close_changes_only_visible_state() {
        let mut visible = true;
        assert!(close_if_open(&mut visible));
        assert!(!visible);
        assert!(!close_if_open(&mut visible));
    }

    #[test]
    fn variants_map_to_upstream_alert_variants() {
        assert_eq!(AlertVariant::Default.native(), NativeAlertVariant::Default);
        assert_eq!(AlertVariant::Info.native(), NativeAlertVariant::Info);
        assert_eq!(AlertVariant::Success.native(), NativeAlertVariant::Success);
        assert_eq!(AlertVariant::Warning.native(), NativeAlertVariant::Warning);
        assert_eq!(AlertVariant::Error.native(), NativeAlertVariant::Error);
    }

    fn read(owner: &AlertOwner) -> bool {
        owner.visible
    }

    fn write(owner: &mut AlertOwner, visible: bool) {
        owner.visible = visible;
    }

    struct AlertOwner {
        visible: bool,
    }
}
