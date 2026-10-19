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
        Self {
            id,
            message: AlertText::Literal(message.into()),
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
        self.title = Some(AlertText::Message(message_key.into()));
        self
    }

    /// Resolves the message from the active RustUI message catalog.
    pub fn localized_message(mut self, message_key: impl Into<SharedString>) -> Self {
        self.message = AlertText::Message(message_key.into());
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

        let close_label = SharedString::from(context.localized_message("rustui-action-close"));
        let close = Button(close_label.clone(), AppAction::new(move |cx| update(cx)))
            .id(format!("{}:close", self.id))
            .accessibility_label(close_label)
            .button_style(ButtonStyle::Plain)
            .into_element(context);

        div()
            .id(format!("{}:group", self.id))
            .role(gpui::Role::Group)
            .flex()
            .flex_col()
            .gap(px(4.0))
            .w_full()
            .child(alert)
            .child(div().flex().justify_end().child(close))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
