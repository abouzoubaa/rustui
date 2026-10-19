//! In-app toast notifications backed by GPUI Components' notification layer.

use gpui::SharedString;
use gpui_component::{
    WindowExt as _,
    button::Button as NativeButton,
    notification::{Notification, NotificationDelivery, NotificationType},
};

use crate::{
    components::theme::{required_app_style_with_artwork, styled_element},
    runtime::AppAction,
};

/// Semantic appearance for an in-app toast.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastKind {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

impl ToastKind {
    fn notification_type(self) -> NotificationType {
        match self {
            Self::Info => NotificationType::Info,
            Self::Success => NotificationType::Success,
            Self::Warning => NotificationType::Warning,
            Self::Error => NotificationType::Error,
        }
    }
}

/// Description of an in-app notification to show with [`ToastBuilder::show`].
#[derive(Clone)]
pub struct ToastBuilder {
    message: SharedString,
    title: Option<SharedString>,
    kind: ToastKind,
    id: Option<SharedString>,
    autohide: bool,
    on_click: Option<AppAction>,
    action: Option<(SharedString, AppAction)>,
}

impl ToastBuilder {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            message: message.into(),
            title: None,
            kind: ToastKind::Info,
            id: None,
            autohide: true,
            on_click: None,
            action: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn kind(mut self, kind: ToastKind) -> Self {
        self.kind = kind;
        self
    }

    /// Sets a stable notification key; a later toast with this key replaces it in the same window.
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        let id = id.into();
        assert!(!id.trim().is_empty(), "Toast identity must be nonempty");
        self.id = Some(id);
        self
    }

    /// Controls automatic dismissal. GPUI Components keeps action toasts visible.
    pub fn autohide(mut self, autohide: bool) -> Self {
        self.autohide = autohide;
        self
    }

    /// Runs an application action when the notification body is clicked.
    pub fn on_click(mut self, action: AppAction) -> Self {
        self.on_click = Some(action);
        self
    }

    /// Adds the native action button. GPUI Components keeps it visible until dismissed.
    pub fn action(mut self, label: impl Into<SharedString>, action: AppAction) -> Self {
        self.action = Some((label.into(), action));
        self
    }

    /// Returns an application action that pushes this toast through the active window root.
    pub fn show(self) -> AppAction {
        AppAction::with_window_action(move |_, window, cx| {
            let style = required_app_style_with_artwork(cx, "notification", "body");
            let mut notification = Notification::new()
                .message(self.message.clone())
                .with_type(self.kind.notification_type())
                .delivery(NotificationDelivery::InApp)
                .autohide(self.autohide);
            if let Some(title) = &self.title {
                notification = notification.title(title.clone());
            }
            if let Some(id) = &self.id {
                notification = notification.id1::<RustUiToast>(id.to_string());
            }
            if let Some(action) = &self.on_click {
                let action = action.clone();
                notification = notification.on_click(move |event, window, cx| {
                    action.dispatch(event, window, cx);
                });
            }
            if let Some((label, action)) = &self.action {
                let label = label.clone();
                let action = action.clone();
                notification = notification.action(move |_, _, _| {
                    let action = action.clone();
                    NativeButton::new("rustui-toast-action")
                        .label(label.clone())
                        .accessibility_label(label.clone())
                        .on_click(move |event, window, cx| {
                            cx.stop_propagation();
                            action.dispatch(event, window, cx);
                        })
                });
            }
            window.push_notification(styled_element(notification, &style), cx);
        })
    }
}

/// Creates an in-app toast description. Call `.show()` and pass its action to a control.
#[allow(non_snake_case)]
pub fn Toast(message: impl Into<SharedString>) -> ToastBuilder {
    ToastBuilder::new(message)
}

struct RustUiToast;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toast_defaults_to_info_and_autohide() {
        let toast = Toast("Saved");
        assert_eq!(toast.message.as_ref(), "Saved");
        assert_eq!(toast.kind, ToastKind::Info);
        assert!(toast.autohide);
        assert!(toast.title.is_none());
        assert!(toast.id.is_none());
        assert!(toast.on_click.is_none());
        assert!(toast.action.is_none());
    }

    #[test]
    fn toast_builders_preserve_notification_options() {
        let toast = Toast("Saved")
            .title("Complete")
            .kind(ToastKind::Success)
            .id("save-result")
            .autohide(false);
        assert_eq!(toast.title.as_deref(), Some("Complete"));
        assert_eq!(toast.kind, ToastKind::Success);
        assert_eq!(toast.id.as_deref(), Some("save-result"));
        assert!(!toast.autohide);
    }

    #[test]
    fn toast_kinds_map_to_upstream_notification_types() {
        assert!(matches!(
            ToastKind::Info.notification_type(),
            NotificationType::Info
        ));
        assert!(matches!(
            ToastKind::Success.notification_type(),
            NotificationType::Success
        ));
        assert!(matches!(
            ToastKind::Warning.notification_type(),
            NotificationType::Warning
        ));
        assert!(matches!(
            ToastKind::Error.notification_type(),
            NotificationType::Error
        ));
    }
}
