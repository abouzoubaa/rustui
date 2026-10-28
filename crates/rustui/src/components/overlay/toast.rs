//! In-app toast notifications backed by GPUI Components' notification layer.

use std::sync::atomic::{AtomicU64, Ordering};

use gpui::{IntoElement, SharedString, accesskit::Live};
use gpui_component::{
    WindowExt as _,
    button::Button as NativeButton,
    notification::{Notification, NotificationDelivery, NotificationType},
};
use rustui_core::Identity;

use crate::{
    components::theme::{required_app_style_with_artwork, styled_element},
    runtime::{
        AppAction,
        scoped_element::{SemanticInfo, semantic_scoped_element},
    },
    view::RetentionScope,
};

static NEXT_TOAST_SEMANTICS_ID: AtomicU64 = AtomicU64::new(0);

fn next_toast_announcement_id() -> Identity {
    Identity::from(format!(
        "toast-announcement-{}",
        NEXT_TOAST_SEMANTICS_ID.fetch_add(1, Ordering::Relaxed)
    ))
}

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
    message_key: Option<SharedString>,
    title_key: Option<SharedString>,
    action_key: Option<SharedString>,
    logical_placement: bool,
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
            message_key: None,
            title_key: None,
            action_key: None,
            logical_placement: false,
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

    /// Resolves the toast message when it is shown, falling back to its literal message.
    pub fn localized_message(mut self, key: impl Into<SharedString>) -> Self {
        let key = key.into();
        assert!(
            !key.trim().is_empty(),
            "toast localization key must not be empty"
        );
        self.message_key = Some(key);
        self
    }

    /// Resolves the title when present, falling back to its literal title.
    pub fn localized_title(mut self, key: impl Into<SharedString>) -> Self {
        let key = key.into();
        assert!(
            !key.trim().is_empty(),
            "toast localization key must not be empty"
        );
        self.title_key = Some(key);
        self
    }

    /// Resolves the action label when an action is present.
    pub fn localized_action_label(mut self, key: impl Into<SharedString>) -> Self {
        let key = key.into();
        assert!(
            !key.trim().is_empty(),
            "toast localization key must not be empty"
        );
        self.action_key = Some(key);
        self
    }

    /// Mirrors the configured notification placement in right-to-left locales.
    /// The default keeps the application-configured physical placement unchanged.
    pub fn logical_placement(mut self) -> Self {
        self.logical_placement = true;
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
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Toast action label must provide a non-empty accessible name"
        );
        self.action = Some((label, action));
        self
    }

    fn resolved_content(
        &self,
        resolve: impl Fn(&str) -> String,
    ) -> (SharedString, Option<SharedString>, Option<SharedString>) {
        let message =
            crate::localization::resolve_text(&self.message, self.message_key.as_ref(), &resolve);
        let title = self.title.as_ref().map(|title| {
            crate::localization::resolve_text(title, self.title_key.as_ref(), &resolve)
        });
        let action = self.action.as_ref().map(|(label, _)| {
            crate::localization::resolve_text(label, self.action_key.as_ref(), &resolve)
        });
        (message, title, action)
    }

    /// Returns an application action that pushes this toast through the active window root.
    pub fn show(self) -> AppAction {
        assert!(
            self.title
                .as_deref()
                .is_some_and(|title| !title.trim().is_empty())
                || !self.message.trim().is_empty(),
            "Toast title or message must provide a non-empty accessible name"
        );
        let announcement_kind = self.kind;
        AppAction::with_window_action(move |_, window, cx| {
            let resolve = |key: &str| {
                crate::localization::active_localization(cx)
                    .and_then(|active| active.formatted_message(key).ok())
                    .unwrap_or_else(|| key.to_owned())
            };
            let (message, title, action_label) = self.resolved_content(resolve);
            let announcement_value = toast_announcement_value(title.as_deref(), &message);
            let style = required_app_style_with_artwork(cx, "notification", "body");
            let mut notification = Notification::new()
                .message(message)
                .with_type(self.kind.notification_type())
                .delivery(NotificationDelivery::InApp)
                .autohide(self.autohide);
            if self.logical_placement
                && crate::localization::active_localization(cx).is_some_and(|locale| {
                    locale.direction() == crate::localization::LayoutDirection::RightToLeft
                })
            {
                notification = notification.placement(mirror_anchor(
                    gpui_component::Theme::global(cx).notification.placement,
                ));
            }
            if let Some(title) = &title {
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
                let label = action_label.clone().unwrap_or_else(|| label.clone());
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
            let announcement_id = next_toast_announcement_id();
            let announcement_value = announcement_value.clone();
            notification = notification.content(move |_, _, _| {
                semantic_scoped_element(
                    &RetentionScope::Semantics(announcement_id.clone()),
                    gpui::div().into_any_element(),
                    Some(toast_announcement_semantics(
                        announcement_kind,
                        announcement_value.clone(),
                    )),
                )
            });
            window.push_notification(styled_element(notification, &style), cx);
        })
    }
}

fn mirror_anchor(anchor: gpui::Anchor) -> gpui::Anchor {
    use gpui::Anchor::*;
    match anchor {
        TopLeft => TopRight,
        TopRight => TopLeft,
        BottomLeft => BottomRight,
        BottomRight => BottomLeft,
        LeftCenter => RightCenter,
        RightCenter => LeftCenter,
        TopCenter => TopCenter,
        BottomCenter => BottomCenter,
    }
}

fn toast_announcement_value(title: Option<&str>, message: &str) -> String {
    let title = title.filter(|title| !title.trim().is_empty());
    let has_message = !message.trim().is_empty();
    match (title, has_message) {
        (Some(title), true) => format!("{title}: {message}"),
        (Some(title), false) => title.to_owned(),
        (None, true) => message.to_owned(),
        (None, false) => unreachable!("Toast::show validates an accessible title or message"),
    }
}

fn toast_announcement_semantics(kind: ToastKind, value: String) -> SemanticInfo {
    let (role, live) = if kind == ToastKind::Error {
        (gpui::Role::Alert, Live::Assertive)
    } else {
        (gpui::Role::Status, Live::Polite)
    };
    SemanticInfo {
        role,
        label: Some(value.clone()),
        description: None,
        live_value: Some((value, live)),
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
    fn localized_content_resolves_and_falls_back() {
        let toast = Toast("Saved changes")
            .title("Saved")
            .action("Undo", AppAction::new(|_| {}))
            .localized_message("toast.body")
            .localized_title("toast.title")
            .localized_action_label("toast.action");
        let (body, title, action) = toast.resolved_content(|key| match key {
            "toast.body" => "Modifications enregistrées".into(),
            "toast.title" => "Enregistré".into(),
            "toast.action" => "Annuler".into(),
            _ => key.into(),
        });
        assert_eq!(body.as_ref(), "Modifications enregistrées");
        assert_eq!(title.as_deref(), Some("Enregistré"));
        assert_eq!(action.as_deref(), Some("Annuler"));
        let (body, title, action) = toast.resolved_content(|key| key.into());
        assert_eq!(body.as_ref(), "Saved changes");
        assert_eq!(title.as_deref(), Some("Saved"));
        assert_eq!(action.as_deref(), Some("Undo"));
    }

    #[test]
    fn logical_placement_mirrors_horizontal_anchors_only() {
        use gpui::Anchor::*;
        for (left, right) in [
            (TopLeft, TopRight),
            (BottomLeft, BottomRight),
            (LeftCenter, RightCenter),
        ] {
            assert_eq!(mirror_anchor(left), right);
            assert_eq!(mirror_anchor(right), left);
        }
        assert_eq!(mirror_anchor(TopCenter), TopCenter);
        assert_eq!(mirror_anchor(BottomCenter), BottomCenter);
        assert!(!Toast("Message").logical_placement);
        assert!(Toast("Message").logical_placement().logical_placement);
    }

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

    #[test]
    #[should_panic(expected = "Toast action label must provide a non-empty accessible name")]
    fn toast_rejects_blank_action_label() {
        Toast("Saved").action(" \t", AppAction::new(|_| {}));
    }

    #[test]
    #[should_panic(expected = "Toast title or message must provide a non-empty accessible name")]
    fn toast_rejects_blank_title_and_message_before_showing() {
        Toast(" \t").title(" ").show();
    }

    #[test]
    fn toast_title_can_name_a_toast_with_an_empty_message() {
        let _action = Toast(" ").title("Saved").show();
    }

    #[test]
    fn toast_announcement_uses_status_or_assertive_alert_semantics() {
        assert_eq!(
            toast_announcement_value(Some("Saved"), "File"),
            "Saved: File"
        );
        assert_eq!(toast_announcement_value(Some("Saved"), " "), "Saved");
        assert_eq!(toast_announcement_value(None, "File"), "File");

        let status = toast_announcement_semantics(ToastKind::Success, "Saved: File".into());
        assert_eq!(status.role, gpui::Role::Status);
        assert_eq!(status.label.as_deref(), Some("Saved: File"));
        assert_eq!(
            status.live_value,
            Some(("Saved: File".into(), Live::Polite))
        );

        let alert = toast_announcement_semantics(ToastKind::Error, "Save failed".into());
        assert_eq!(alert.role, gpui::Role::Alert);
        assert_eq!(alert.label.as_deref(), Some("Save failed"));
        assert_eq!(
            alert.live_value,
            Some(("Save failed".into(), Live::Assertive))
        );
    }

    #[test]
    fn each_toast_dispatch_gets_a_distinct_announcement_identity() {
        assert_ne!(next_toast_announcement_id(), next_toast_announcement_id());
    }
}
