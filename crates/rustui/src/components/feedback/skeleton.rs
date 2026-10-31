//! Skeleton loading placeholders backed by pinned GPUI Components.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, Role, SharedString,
    StatefulInteractiveElement, Styled, div, px,
};
use gpui_component::skeleton::Skeleton as NativeSkeleton;
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, View};

#[derive(Clone, Debug)]
enum SkeletonAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

/// An animated placeholder with optional explicit size and corner radius.
#[derive(Clone, Debug, Default)]
pub struct SkeletonView {
    size: Option<(f32, f32)>,
    corner_radius: Option<f32>,
    secondary: bool,
    accessibility_label: Option<SkeletonAccessibilityLabel>,
    id: Option<Identity>,
}

impl SkeletonView {
    /// Sets explicit logical-pixel width and height.
    pub fn size(mut self, width: f32, height: f32) -> Self {
        assert!(
            width.is_finite() && width > 0.0 && height.is_finite() && height > 0.0,
            "skeleton dimensions must be finite and positive"
        );
        self.size = Some((width, height));
        self
    }

    /// Sets a logical-pixel corner radius.
    pub fn corner_radius(mut self, radius: f32) -> Self {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "skeleton corner radius must be finite and nonnegative"
        );
        self.corner_radius = Some(radius);
        self
    }

    /// Uses the native secondary skeleton color.
    pub fn secondary(mut self) -> Self {
        self.secondary = true;
        self
    }

    /// Makes the placeholder meaningful to assistive technology.
    ///
    /// Skeletons are decorative by default and are omitted from the
    /// accessibility tree unless a label is supplied.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "skeleton accessibility label must not be empty"
        );
        self.accessibility_label = Some(SkeletonAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the placeholder name from the active locale catalog.
    ///
    /// The current literal label is retained as a fallback; otherwise the
    /// fallback is "Loading". A localized label keeps the message key as its
    /// generated accessibility ID across locale changes.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "skeleton accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(SkeletonAccessibilityLabel::Literal(label))
            | Some(SkeletonAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => "Loading".into(),
        };
        self.accessibility_label = Some(SkeletonAccessibilityLabel::Message { id, fallback });
        self
    }

    /// Sets a stable accessibility element ID for a labeled placeholder.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> Option<SharedString> {
        match &self.accessibility_label {
            None => None,
            Some(SkeletonAccessibilityLabel::Literal(label)) => Some(label.clone()),
            Some(SkeletonAccessibilityLabel::Message { id, fallback }) => {
                let localized = localized_message(id.as_ref());
                Some(
                    if localized.trim().is_empty() || localized.trim() == id.trim() {
                        fallback.clone()
                    } else {
                        localized.into()
                    },
                )
            }
        }
    }

    fn resolved_accessibility_id(&self) -> Option<Identity> {
        self.accessibility_label.as_ref().map(|label| {
            self.id.clone().unwrap_or_else(|| match label {
                SkeletonAccessibilityLabel::Literal(label) => {
                    Identity::from(format!("rustui-skeleton:{label}"))
                }
                SkeletonAccessibilityLabel::Message { id, .. } => {
                    Identity::from(format!("rustui-skeleton-message:{id}"))
                }
            })
        })
    }
}

/// Creates the native animated skeleton placeholder.
#[allow(non_snake_case)]
pub fn Skeleton() -> SkeletonView {
    SkeletonView::default()
}

impl View for SkeletonView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut skeleton = NativeSkeleton::new();
        if self.secondary {
            skeleton = skeleton.secondary();
        }
        if let Some((width, height)) = self.size {
            skeleton = skeleton.w(px(width)).h(px(height));
        }
        if let Some(radius) = self.corner_radius {
            skeleton = skeleton.rounded(px(radius));
        }
        let skeleton = skeleton.into_any_element();
        match self.resolved_accessibility_label(|id| context.localized_message(id)) {
            Some(label) => {
                let id = self
                    .resolved_accessibility_id()
                    .expect("labeled skeleton has an accessibility ID");
                div()
                    .id(id.as_str().to_owned())
                    .role(Role::ProgressIndicator)
                    .aria_label(label)
                    .child(skeleton)
                    .into_any_element()
            }
            None => skeleton,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_positive_size_and_nonnegative_radius() {
        let skeleton = Skeleton().size(120.0, 18.0).corner_radius(0.0);
        assert_eq!(skeleton.size, Some((120.0, 18.0)));
        assert_eq!(skeleton.corner_radius, Some(0.0));
    }

    #[test]
    #[should_panic(expected = "skeleton dimensions must be finite and positive")]
    fn rejects_non_positive_dimensions() {
        Skeleton().size(0.0, 18.0);
    }

    #[test]
    #[should_panic(expected = "skeleton dimensions must be finite and positive")]
    fn rejects_non_finite_dimensions() {
        Skeleton().size(f32::INFINITY, 18.0);
    }

    #[test]
    #[should_panic(expected = "skeleton corner radius must be finite and nonnegative")]
    fn rejects_negative_corner_radius() {
        Skeleton().corner_radius(-1.0);
    }

    #[test]
    #[should_panic(expected = "skeleton corner radius must be finite and nonnegative")]
    fn rejects_non_finite_corner_radius() {
        Skeleton().corner_radius(f32::NAN);
    }

    #[test]
    #[should_panic(expected = "skeleton accessibility label must not be empty")]
    fn rejects_empty_accessibility_label() {
        Skeleton().accessibility_label("  ");
    }

    #[test]
    fn remains_decorative_without_an_accessibility_label() {
        assert_eq!(
            Skeleton().resolved_accessibility_label(|_| unreachable!()),
            None
        );
        assert_eq!(Skeleton().resolved_accessibility_id(), None);
    }

    #[test]
    fn localized_accessibility_label_uses_explicit_fallback() {
        let skeleton = Skeleton()
            .accessibility_label("Loading profile")
            .localized_accessibility_label("skeleton.profile");
        assert_eq!(
            skeleton.resolved_accessibility_label(|id| format!("{id} traduit")),
            Some("skeleton.profile traduit".into())
        );
        assert_eq!(
            skeleton.resolved_accessibility_label(|_| "skeleton.profile".to_owned()),
            Some("Loading profile".into())
        );
        assert_eq!(
            skeleton.resolved_accessibility_label(|_| " skeleton.profile \t".to_owned()),
            Some("Loading profile".into())
        );
        assert_eq!(
            skeleton.resolved_accessibility_label(|_| " \t ".to_owned()),
            Some("Loading profile".into())
        );
    }

    #[test]
    fn localized_accessibility_label_defaults_to_loading() {
        let skeleton = Skeleton().localized_accessibility_label("skeleton.loading");
        assert_eq!(
            skeleton.resolved_accessibility_label(|_| String::new()),
            Some("Loading".into())
        );
    }

    #[test]
    fn localized_accessibility_id_is_stable_across_resolutions() {
        let skeleton = Skeleton().localized_accessibility_label("skeleton.profile");
        let before = skeleton.resolved_accessibility_id();
        let english = skeleton.resolved_accessibility_label(|_| "Loading profile".to_owned());
        let french = skeleton.resolved_accessibility_label(|_| "Chargement du profil".to_owned());

        assert_eq!(english, Some("Loading profile".into()));
        assert_eq!(french, Some("Chargement du profil".into()));
        assert_eq!(before, skeleton.resolved_accessibility_id());
        assert_eq!(
            before.unwrap().as_str(),
            "rustui-skeleton-message:skeleton.profile"
        );
    }

    #[test]
    #[should_panic(expected = "skeleton accessibility localization key must not be blank")]
    fn rejects_blank_accessibility_message_id() {
        Skeleton().localized_accessibility_label(" \t ");
    }
}
