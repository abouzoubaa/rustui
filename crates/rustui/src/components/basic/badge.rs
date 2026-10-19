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
    accessibility_label: Option<gpui::SharedString>,
    accessibility_id: Option<Identity>,
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
        self.accessibility_label = Some(label.into());
        self
    }

    /// Sets a stable accessibility element ID when identical badges are siblings.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.accessibility_id = Some(id.into());
        self
    }

    fn resolved_accessibility_label(&self) -> Option<gpui::SharedString> {
        self.accessibility_label.clone().or_else(|| {
            if self.dot {
                Some("Notification available".into())
            } else if self.count > 0 {
                Some(if self.count == 1 {
                    "1 notification".to_owned().into()
                } else {
                    format!("{} notifications", self.count).into()
                })
            } else {
                None
            }
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
        let badge = badge.into_any_element();
        match self.resolved_accessibility_label() {
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
