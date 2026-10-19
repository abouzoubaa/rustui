//! Initials and image avatars backed by GPUI Components.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, SharedString,
    StatefulInteractiveElement, div, px,
};
use gpui_component::{Sizable, Size, avatar::Avatar as NativeAvatar};
use rustui_core::Identity;

use crate::{
    components::basic::image::ImageSource,
    view::{EmptyView, RenderContext, View},
};

/// An initials or image avatar with an optional explicit size.
#[derive(Clone, Debug)]
pub struct AvatarView {
    name: Option<SharedString>,
    source: Option<ImageSource>,
    size: Option<f32>,
    accessibility_label: Option<SharedString>,
    accessibility_id: Option<Identity>,
}

impl AvatarView {
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: normalize_name(name.into().as_ref()),
            source: None,
            size: None,
            accessibility_label: None,
            accessibility_id: None,
        }
    }

    /// Loads an embedded asset or filesystem image into the native avatar.
    pub fn source(mut self, source: impl Into<ImageSource>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Sets the avatar diameter in logical pixels.
    pub fn size(mut self, size: f32) -> Self {
        assert!(
            size.is_finite() && size > 0.0,
            "avatar size must be finite and positive"
        );
        self.size = Some(size);
        self
    }

    /// Overrides the accessible name. Defaults to the normalized name or
    /// "User avatar" when the avatar has no name.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Sets a stable accessibility element ID when identical avatars are siblings.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.accessibility_id = Some(id.into());
        self
    }

    fn resolved_accessibility_label(&self) -> SharedString {
        self.accessibility_label.clone().unwrap_or_else(|| {
            self.name
                .clone()
                .unwrap_or_else(|| SharedString::from("User avatar"))
        })
    }

    fn resolved_accessibility_id(&self) -> String {
        self.accessibility_id
            .as_ref()
            .map(ToString::to_string)
            .unwrap_or_else(|| {
                self.name.as_ref().map_or_else(
                    || "rustui-avatar:anonymous".to_owned(),
                    |name| format!("rustui-avatar:{name}"),
                )
            })
    }
}

/// Creates a native avatar using initials derived from the supplied name.
#[allow(non_snake_case)]
pub fn Avatar(name: impl Into<SharedString>) -> AvatarView {
    AvatarView::new(name)
}

fn normalize_name(name: &str) -> Option<SharedString> {
    let normalized = name.split_whitespace().collect::<Vec<_>>().join(" ");
    (!normalized.is_empty()).then(|| normalized.into())
}

impl View for AvatarView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        let mut avatar = NativeAvatar::new();
        if let Some(name) = &self.name {
            avatar = avatar.name(name.clone());
        }
        if let Some(source) = &self.source {
            let source = match source {
                ImageSource::Asset(path) => gpui::ImageSource::from(path.clone()),
                ImageSource::File(path) => gpui::ImageSource::from(path.clone()),
            };
            avatar = avatar.src(source);
        }
        if let Some(size) = self.size {
            avatar = avatar.with_size(Size::Size(px(size)));
        }
        div()
            .id(self.resolved_accessibility_id())
            .role(gpui::Role::Image)
            .aria_label(self.resolved_accessibility_label())
            .child(avatar)
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_whitespace_for_native_initials() {
        let avatar = Avatar("  Ada   Lovelace ");
        assert_eq!(avatar.name.as_deref(), Some("Ada Lovelace"));
    }

    #[test]
    fn empty_name_uses_native_anonymous_placeholder() {
        let avatar = Avatar(" \t\n ");
        assert_eq!(avatar.name, None);
        assert_eq!(
            avatar.resolved_accessibility_label().as_ref(),
            "User avatar"
        );
    }

    #[test]
    fn normalized_name_is_the_default_accessible_name() {
        let avatar = Avatar("  Ada   Lovelace ");
        assert_eq!(
            avatar.resolved_accessibility_label().as_ref(),
            "Ada Lovelace"
        );
    }

    #[test]
    fn explicit_id_overrides_stable_name_based_default() {
        assert_eq!(
            Avatar("Ada Lovelace").resolved_accessibility_id(),
            "rustui-avatar:Ada Lovelace"
        );
        assert_eq!(
            Avatar("Ada Lovelace")
                .id("profile-ada")
                .resolved_accessibility_id(),
            "profile-ada"
        );
    }

    #[test]
    fn accepts_finite_positive_diameter() {
        assert_eq!(Avatar("Ada").size(32.0).size, Some(32.0));
    }

    #[test]
    #[should_panic(expected = "avatar size must be finite and positive")]
    fn rejects_non_positive_diameter() {
        Avatar("Ada").size(-1.0);
    }

    #[test]
    #[should_panic(expected = "avatar size must be finite and positive")]
    fn rejects_non_finite_diameter() {
        Avatar("Ada").size(f32::NAN);
    }
}
