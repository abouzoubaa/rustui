//! Skeleton loading placeholders backed by pinned GPUI Components.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, Role, SharedString,
    StatefulInteractiveElement, Styled, div, px,
};
use gpui_component::skeleton::Skeleton as NativeSkeleton;
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, View};

/// An animated placeholder with optional explicit size and corner radius.
#[derive(Clone, Debug, Default)]
pub struct SkeletonView {
    size: Option<(f32, f32)>,
    corner_radius: Option<f32>,
    secondary: bool,
    accessibility_label: Option<SharedString>,
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
        self.accessibility_label = Some(label);
        self
    }

    /// Sets a stable accessibility element ID for a labeled placeholder.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
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

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
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
        match &self.accessibility_label {
            Some(label) => {
                let id = self
                    .id
                    .clone()
                    .unwrap_or_else(|| Identity::from(format!("rustui-skeleton:{label}")));
                div()
                    .id(id.as_str().to_owned())
                    .role(Role::ProgressIndicator)
                    .aria_label(label.clone())
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
}
