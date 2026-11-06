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

#[derive(Clone, Debug)]
enum AvatarAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: Option<SharedString>,
    },
}

/// An initials or image avatar with an optional explicit size.
#[derive(Clone, Debug)]
pub struct AvatarView {
    name: Option<SharedString>,
    source: Option<ImageSource>,
    size: Option<f32>,
    accessibility_label: Option<AvatarAccessibilityLabel>,
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
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Avatar accessibility label must not be empty"
        );
        self.accessibility_label = Some(AvatarAccessibilityLabel::Literal(label));
        self
    }

    /// Sets a stable accessibility element ID when identical avatars are siblings.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Avatar identity must not be blank"
        );
        self.accessibility_id = Some(id);
        self
    }

    /// Resolves the accessible name using a localization message ID.
    /// The explicit label, or normalized avatar name, is the fallback.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Avatar accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(AvatarAccessibilityLabel::Literal(label)) => Some(label.clone()),
            Some(AvatarAccessibilityLabel::Message { fallback, .. }) => fallback.clone(),
            None => None,
        };
        self.accessibility_label = Some(AvatarAccessibilityLabel::Message { id, fallback });
        self
    }

    fn default_accessibility_label(&self) -> SharedString {
        self.name
            .clone()
            .unwrap_or_else(|| SharedString::from("User avatar"))
    }

    #[cfg(test)]
    fn resolved_accessibility_label(&self) -> SharedString {
        match &self.accessibility_label {
            Some(AvatarAccessibilityLabel::Literal(label)) => label.clone(),
            Some(AvatarAccessibilityLabel::Message { fallback, .. }) => fallback
                .clone()
                .unwrap_or_else(|| self.default_accessibility_label()),
            None => self.default_accessibility_label(),
        }
    }

    fn resolve_accessibility_label(&self, context: &RenderContext<'_>) -> SharedString {
        match &self.accessibility_label {
            Some(AvatarAccessibilityLabel::Literal(label)) => label.clone(),
            Some(AvatarAccessibilityLabel::Message { id, fallback }) => {
                let fallback = fallback
                    .clone()
                    .unwrap_or_else(|| self.default_accessibility_label());
                let localized = context.localized_message(id.as_ref());
                if localized.trim().is_empty() || localized.trim() == id.trim() {
                    fallback
                } else {
                    localized.into()
                }
            }
            None => self.default_accessibility_label(),
        }
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

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
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
            .aria_label(self.resolve_accessibility_label(context))
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
    fn explicit_accessible_name_rejects_whitespace_only_text() {
        assert!(std::panic::catch_unwind(|| Avatar("Ada").accessibility_label(" \t\n ")).is_err());
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text_verbatim() {
        let avatar = Avatar("Ada").accessibility_label(" Ada's profile ");

        assert_eq!(
            avatar.resolved_accessibility_label().as_ref(),
            " Ada's profile "
        );
    }

    #[test]
    fn localized_accessible_name_uses_explicit_or_visible_name_fallback() {
        let visible = Avatar(" Ada Lovelace ").localized_accessibility_label("avatar.ada");
        assert!(matches!(
            visible.accessibility_label,
            Some(AvatarAccessibilityLabel::Message { ref id, fallback: None })
                if id.as_ref() == "avatar.ada"
        ));
        assert_eq!(
            visible.resolved_accessibility_label().as_ref(),
            "Ada Lovelace"
        );

        let explicit = Avatar("Ada")
            .accessibility_label("Ada's profile")
            .localized_accessibility_label("avatar.ada");
        assert!(matches!(
            explicit.accessibility_label,
            Some(AvatarAccessibilityLabel::Message { ref id, ref fallback })
                if id.as_ref() == "avatar.ada" && fallback.as_deref() == Some("Ada's profile")
        ));
        assert_eq!(
            explicit.resolved_accessibility_label().as_ref(),
            "Ada's profile"
        );
    }

    #[test]
    #[should_panic(expected = "Avatar accessibility localization key must not be blank")]
    fn localized_accessible_name_rejects_blank_message_id() {
        Avatar("Ada").localized_accessibility_label(" 	 ");
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
    #[should_panic(expected = "Avatar identity must not be blank")]
    fn rejects_blank_explicit_identity() {
        Avatar("Ada").id(" \t ");
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
