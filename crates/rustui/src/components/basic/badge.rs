//! Count and dot badges backed by GPUI Components.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, StatefulInteractiveElement, div,
};
use gpui_component::{Sizable, Size, badge::Badge as NativeBadge};
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, View};

/// Supported native badge sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BadgeSize {
    Small,
    #[default]
    Medium,
    Large,
}

#[derive(Clone)]
enum BadgeAccessibilityLabel {
    Literal(gpui::SharedString),
    Message {
        id: gpui::SharedString,
        fallback: Option<gpui::SharedString>,
    },
}

impl BadgeSize {
    fn native(self) -> Size {
        match self {
            Self::Small => Size::Small,
            Self::Medium => Size::Medium,
            Self::Large => Size::Large,
        }
    }
}

/// A native count or dot badge attached to content.
pub struct BadgeView<V: View> {
    content: V,
    count: usize,
    max: usize,
    dot: bool,
    size: BadgeSize,
    accessibility_label: Option<BadgeAccessibilityLabel>,
    accessibility_id: Option<Identity>,
}

fn is_localized_accessibility_translation(message_id: &str, localized: &str) -> bool {
    !localized.trim().is_empty() && localized.trim() != message_id.trim()
}

fn profile_badge_background(
    profile: &crate::theme::schema::ThemeProfile,
    mode: crate::theme::schema::AppearanceMode,
) -> gpui::Hsla {
    gpui_component::theme::try_parse_color(&profile.colors(Some(mode)).badge_background)
        .expect("validated RustUI badge background")
}

impl<V: View> BadgeView<V> {
    pub fn new(content: V) -> Self {
        Self {
            content,
            count: 0,
            max: 99,
            dot: false,
            size: BadgeSize::default(),
            accessibility_label: None,
            accessibility_id: None,
        }
    }

    /// Displays a count; zero hides the count badge.
    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self.dot = false;
        self
    }

    /// Displays a small dot instead of a count.
    pub fn dot(mut self) -> Self {
        self.dot = true;
        self
    }

    /// Caps the displayed count using the native `max+` format.
    pub fn max(mut self, max: usize) -> Self {
        self.max = max;
        self
    }

    /// Selects the supported native badge size.
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }

    /// Overrides the accessible status announcement for a visible badge.
    pub fn accessibility_label(mut self, label: impl Into<gpui::SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Badge accessibility label must not be empty"
        );
        self.accessibility_label = Some(BadgeAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the status announcement using a localization message ID.
    /// The explicit label, or visible badge announcement, is the fallback.
    pub fn localized_accessibility_label(
        mut self,
        message_id: impl Into<gpui::SharedString>,
    ) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Badge accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(BadgeAccessibilityLabel::Literal(label)) => Some(label.clone()),
            Some(BadgeAccessibilityLabel::Message { fallback, .. }) => fallback.clone(),
            None => None,
        };
        self.accessibility_label = Some(BadgeAccessibilityLabel::Message { id, fallback });
        self
    }

    /// Sets a stable accessibility element ID when identical badges are siblings.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Badge identity must not be empty"
        );
        self.accessibility_id = Some(id);
        self
    }

    fn default_accessibility_label(&self) -> Option<gpui::SharedString> {
        if !self.dot && self.count == 0 {
            return None;
        }
        Some(if self.dot {
            "Notification available".into()
        } else if self.count == 1 {
            "1 notification".into()
        } else {
            format!("{} notifications", self.count).into()
        })
    }

    #[cfg(test)]
    fn resolved_accessibility_label(&self) -> Option<gpui::SharedString> {
        self.default_accessibility_label()?;
        Some(match &self.accessibility_label {
            Some(BadgeAccessibilityLabel::Literal(label)) => label.clone(),
            Some(BadgeAccessibilityLabel::Message { fallback, .. }) => fallback
                .clone()
                .or_else(|| self.default_accessibility_label())?,
            None => self.default_accessibility_label()?,
        })
    }

    fn resolve_accessibility_label(
        &self,
        context: &RenderContext<'_>,
    ) -> Option<gpui::SharedString> {
        self.default_accessibility_label()?;
        Some(match &self.accessibility_label {
            Some(BadgeAccessibilityLabel::Literal(label)) => label.clone(),
            Some(BadgeAccessibilityLabel::Message { id, fallback }) => {
                let fallback = fallback
                    .clone()
                    .or_else(|| self.default_accessibility_label())?;
                let localized = context.localized_message(id.as_ref());
                if !is_localized_accessibility_translation(id.as_ref(), &localized) {
                    fallback
                } else {
                    localized.into()
                }
            }
            None => self.default_accessibility_label()?,
        })
    }

    fn resolved_accessibility_id(&self) -> String {
        self.accessibility_id
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| {
                if self.dot {
                    "rustui-badge:dot".to_owned()
                } else {
                    format!("rustui-badge:count:{}", self.count)
                }
            })
    }
}

/// Attaches a count or dot badge to content.
#[allow(non_snake_case)]
pub fn Badge<V: View>(content: V) -> BadgeView<V> {
    BadgeView::new(content)
}

impl<V: View> View for BadgeView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut badge = NativeBadge::new()
            .count(self.count)
            .max(self.max)
            .with_size(self.size.native())
            .child(self.content.into_element(context));
        if self.dot {
            badge = badge.dot();
        }
        if let Some(active) = crate::theme::registry::active_theme(context.app()) {
            badge = badge.color(profile_badge_background(&active.profile, active.mode));
        }
        let badge = badge.into_any_element();
        match self.resolve_accessibility_label(context) {
            Some(label) => div()
                .id(self.resolved_accessibility_id())
                .role(gpui::Role::Status)
                .aria_label(label)
                .child(badge)
                .into_any_element(),
            None => badge,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn badge_background_uses_the_profile_token_for_both_appearances() {
        let profile = crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json"))
            .unwrap()
            .platform_snapshot;

        for mode in [
            crate::theme::schema::AppearanceMode::Light,
            crate::theme::schema::AppearanceMode::Dark,
        ] {
            let colors = profile.colors(Some(mode));
            assert_eq!(
                profile_badge_background(&profile, mode),
                gpui_component::theme::try_parse_color(&colors.badge_background).unwrap()
            );
            assert_ne!(
                profile_badge_background(&profile, mode),
                gpui_component::theme::try_parse_color(&colors.error).unwrap()
            );
        }
    }

    #[test]
    fn accessible_label_describes_visible_count_or_dot() {
        let new_badge = || Badge(crate::components::basic::text::TextView::new("Inbox"));
        assert_eq!(
            new_badge()
                .count(1)
                .resolved_accessibility_label()
                .as_deref(),
            Some("1 notification")
        );
        assert_eq!(
            new_badge()
                .count(7)
                .resolved_accessibility_label()
                .as_deref(),
            Some("7 notifications")
        );
        assert_eq!(
            new_badge().dot().resolved_accessibility_label().as_deref(),
            Some("Notification available")
        );
        assert_eq!(new_badge().resolved_accessibility_label(), None);
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text_verbatim() {
        let badge = Badge(crate::components::basic::text::TextView::new("Inbox"))
            .count(3)
            .accessibility_label("  Unread messages  ");
        assert_eq!(
            badge.resolved_accessibility_label().as_deref(),
            Some("  Unread messages  ")
        );
    }

    #[test]
    fn hidden_zero_count_badge_does_not_expose_an_accessible_status() {
        let badge = Badge(crate::components::basic::text::TextView::new("Inbox"))
            .count(0)
            .accessibility_label("Unread messages");

        assert_eq!(badge.resolved_accessibility_label(), None);
    }

    #[test]
    fn rejects_blank_explicit_identity() {
        for id in ["", " \t\n"] {
            assert!(std::panic::catch_unwind(|| {
                Badge(crate::components::basic::text::TextView::new("Inbox")).id(id)
            })
            .is_err());
        }
    }

    #[test]
    #[should_panic(expected = "Badge accessibility label must not be empty")]
    fn explicit_accessible_name_rejects_whitespace_only_text() {
        Badge(crate::components::basic::text::TextView::new("Inbox"))
            .count(3)
            .accessibility_label(" \t\n ");
    }

    #[test]
    fn localized_accessible_name_uses_explicit_or_visible_announcement_fallback() {
        let visible = Badge(crate::components::basic::text::TextView::new("Inbox"))
            .localized_accessibility_label("badge.unread")
            .count(4);
        assert!(matches!(
            visible.accessibility_label,
            Some(BadgeAccessibilityLabel::Message { ref id, fallback: None })
                if id.as_ref() == "badge.unread"
        ));
        assert_eq!(
            visible.resolved_accessibility_label().as_deref(),
            Some("4 notifications")
        );

        let explicit = Badge(crate::components::basic::text::TextView::new("Inbox"))
            .count(1)
            .accessibility_label("Unread messages")
            .localized_accessibility_label("badge.unread");
        assert!(matches!(
            explicit.accessibility_label,
            Some(BadgeAccessibilityLabel::Message { ref id, ref fallback })
                if id.as_ref() == "badge.unread" && fallback.as_deref() == Some("Unread messages")
        ));
        assert_eq!(
            explicit.resolved_accessibility_label().as_deref(),
            Some("Unread messages")
        );
    }

    #[test]
    fn padded_localization_key_echo_uses_accessible_name_fallback() {
        assert!(!is_localized_accessibility_translation(
            "badge.unread",
            " badge.unread \t"
        ));
        assert!(is_localized_accessibility_translation(
            "badge.unread",
            "Unread notifications"
        ));
    }

    #[test]
    #[should_panic(expected = "Badge accessibility localization key must not be blank")]
    fn localized_accessible_name_rejects_blank_message_id() {
        Badge(crate::components::basic::text::TextView::new("Inbox"))
            .count(3)
            .localized_accessibility_label(" 	 ");
    }

    #[test]
    fn explicit_id_overrides_source_independent_default() {
        let new_badge = || Badge(crate::components::basic::text::TextView::new("Inbox"));
        assert_eq!(
            new_badge().count(4).resolved_accessibility_id(),
            "rustui-badge:count:4"
        );
        assert_eq!(
            new_badge()
                .count(4)
                .id("inbox-unread")
                .resolved_accessibility_id(),
            "inbox-unread"
        );
    }
}
