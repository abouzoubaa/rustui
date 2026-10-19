//! Theme border decoration for arbitrary RustUI children.

use gpui::{AnyElement, Div, Hsla, IntoElement as _, ParentElement as _, Styled as _, div, px};
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, RetentionScope, View};

/// Semantic upstream palette entries usable for an outline.
/// FocusRing selects a color only; it does not request focus or track focus state.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum BorderColor {
    #[default]
    Border,
    Accent,
    FocusRing,
}

/// A native border box. Width participates in native box layout rather than
/// painting an overlay. Child layout, environment, actions and bindings remain
/// native; rounding does not clip descendants.
pub struct BorderView<V: View> {
    id: Identity,
    child: V,
    width: f32,
    radius: f32,
    color: BorderColor,
}

/// Wraps a child in a one-logical-pixel active-theme border with square corners.
/// Give sibling wrappers unique stable IDs. IDs use a typed Border namespace,
/// distinct from Background, so the two families may reuse the same ID.
/// Panics if the ID is empty or contains only whitespace.
#[allow(non_snake_case)]
pub fn Border<V: View>(id: impl Into<Identity>, child: V) -> BorderView<V> {
    let id = id.into();
    assert!(
        !id.as_str().trim().is_empty(),
        "Border identity must be nonempty"
    );
    BorderView {
        id,
        child,
        width: 1.0,
        radius: 0.0,
        color: BorderColor::Border,
    }
}

impl<V: View> BorderView<V> {
    /// Sets border width in logical pixels; zero hides the stroke.
    /// Panics for negative or nonfinite values.
    pub fn width(mut self, width: f32) -> Self {
        validate_dimension(width, "width");
        self.width = width;
        self
    }

    /// Sets corner radius without clipping the child.
    /// Panics for negative or nonfinite logical-pixel values.
    pub fn radius(mut self, radius: f32) -> Self {
        validate_dimension(radius, "radius");
        self.radius = radius;
        self
    }

    pub fn color(mut self, color: BorderColor) -> Self {
        self.color = color;
        self
    }

    fn native_box(&self, color: Hsla) -> Div {
        div()
            .flex()
            .flex_col()
            .border(px(self.width))
            .border_color(color)
            .rounded(px(self.radius))
    }
}

fn validate_dimension(value: f32, name: &str) {
    assert!(
        value.is_finite() && value >= 0.0,
        "Border {name} must be finite and nonnegative"
    );
}

fn border_color(token: BorderColor, border: Hsla, accent: Hsla, focus: Hsla) -> Hsla {
    match token {
        BorderColor::Border => border,
        BorderColor::Accent => accent,
        BorderColor::FocusRing => focus,
    }
}

impl<V: View> View for BorderView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let colors = &gpui_component::Theme::global(context.app()).colors;
        let color = border_color(self.color, colors.border, colors.primary, colors.ring);
        let native = self.native_box(color);
        context.with_element_scope(RetentionScope::Border(self.id.clone()), |context| {
            native
                .child(self.child.into_element(context))
                .into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_rejects_empty_and_whitespace_without_normalizing_valid_ids() {
        for id in ["", " \t\n"] {
            assert!(std::panic::catch_unwind(|| Border(id, EmptyView)).is_err());
        }
        assert_eq!(Border(" outline ", EmptyView).id.as_str(), " outline ");
    }

    #[test]
    fn defaults_and_custom_geometry_use_native_box_rules_without_clipping() {
        let border = Border("outline", EmptyView);
        assert_eq!(
            (border.width, border.radius, border.color),
            (1.0, 0.0, BorderColor::Border)
        );
        let mut native = border
            .width(2.5)
            .radius(6.0)
            .native_box(gpui::hsla(0.0, 0.0, 0.4, 1.0));
        let style = native.style();
        assert_eq!(style.border_widths.top, Some(px(2.5).into()));
        assert_eq!(style.border_widths.right, Some(px(2.5).into()));
        assert_eq!(style.border_widths.bottom, Some(px(2.5).into()));
        assert_eq!(style.border_widths.left, Some(px(2.5).into()));
        assert_eq!(style.corner_radii.top_left, Some(px(6.0).into()));
        assert_eq!(style.overflow, gpui::StyleRefinement::default().overflow);
    }

    #[test]
    fn geometry_accepts_zero_and_rejects_invalid_values() {
        let view = Border("outline", EmptyView).width(0.0).radius(0.0);
        assert_eq!((view.width, view.radius), (0.0, 0.0));
        for value in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(
                std::panic::catch_unwind(|| Border("outline", EmptyView).width(value)).is_err()
            );
            assert!(
                std::panic::catch_unwind(|| Border("outline", EmptyView).radius(value)).is_err()
            );
        }
    }

    #[test]
    fn semantic_border_colors_resolve_from_the_current_palette() {
        let border = gpui::hsla(0.0, 0.0, 0.1, 1.0);
        let accent = gpui::hsla(0.3, 0.6, 0.5, 1.0);
        let focus = gpui::hsla(0.6, 0.8, 0.5, 1.0);
        for (token, expected) in [
            (BorderColor::Border, border),
            (BorderColor::Accent, accent),
            (BorderColor::FocusRing, focus),
        ] {
            assert_eq!(border_color(token, border, accent, focus), expected);
        }
    }
}
