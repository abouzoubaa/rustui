//! Theme surface decoration for arbitrary RustUI children.

use gpui::{AnyElement, Div, Hsla, IntoElement as _, ParentElement as _, Styled as _, div, px};
use rustui_core::Identity;

use crate::{
    layout::Surface,
    view::{EmptyView, RenderContext, RetentionScope, View},
};

/// A native layout box painted with an active semantic surface.
/// This wrapper adds no padding, clipping, event handler or independent runtime.
/// Parent layout and the child determine its size, as with Container.
pub struct BackgroundView<V: View> {
    id: Identity,
    child: V,
    surface: Surface,
    radius: f32,
}

/// Decorates `child` using the existing Container surface tokens.
/// Give sibling wrappers unique stable IDs; changing the surface on the same
/// identity preserves retained child state. Border and Background IDs occupy
/// different typed namespaces.
/// Panics if the ID is empty or contains only whitespace.
#[allow(non_snake_case)]
pub fn Background<V: View>(
    id: impl Into<Identity>,
    child: V,
    surface: Surface,
) -> BackgroundView<V> {
    let id = id.into();
    assert!(
        !id.as_str().trim().is_empty(),
        "Background identity must be nonempty"
    );
    BackgroundView {
        id,
        child,
        surface,
        radius: 0.0,
    }
}

impl<V: View> BackgroundView<V> {
    /// Rounds the background without clipping the child.
    /// Panics for negative or nonfinite logical-pixel values.
    pub fn radius(mut self, radius: f32) -> Self {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "Background radius must be finite and nonnegative"
        );
        self.radius = radius;
        self
    }

    fn native_box(&self, color: Hsla) -> Div {
        div().flex().flex_col().bg(color).rounded(px(self.radius))
    }
}

fn surface_color(surface: Surface, background: Hsla, sidebar: Hsla, panel: Hsla) -> Hsla {
    match surface {
        Surface::Background => background,
        Surface::Sidebar => sidebar,
        Surface::Panel => panel,
    }
}

impl<V: View> View for BackgroundView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let colors = &gpui_component::Theme::global(context.app()).colors;
        let color = surface_color(
            self.surface,
            colors.background,
            colors.sidebar,
            colors.secondary,
        );
        let native = self.native_box(color);
        context.with_element_scope(RetentionScope::Background(self.id.clone()), |context| {
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
            assert!(
                std::panic::catch_unwind(|| Background(id, EmptyView, Surface::Panel)).is_err()
            );
        }
        assert_eq!(
            Background(" paint ", EmptyView, Surface::Panel).id.as_str(),
            " paint "
        );
    }

    #[test]
    fn defaults_and_radius_keep_native_unclipped_box() {
        let view = Background("paint", EmptyView, Surface::Panel);
        assert_eq!(view.radius, 0.0);
        let mut native = view.radius(8.0).native_box(gpui::hsla(0.2, 0.3, 0.4, 1.0));
        let style = native.style();
        assert_eq!(style.display, Some(gpui::Display::Flex));
        assert_eq!(style.flex_direction, Some(gpui::FlexDirection::Column));
        assert_eq!(style.corner_radii.top_left, Some(px(8.0).into()));
        assert_eq!(style.overflow, gpui::StyleRefinement::default().overflow);
        assert!(style.background.is_some());
    }

    #[test]
    fn radius_accepts_zero_and_rejects_invalid_values() {
        assert_eq!(
            Background("paint", EmptyView, Surface::Panel)
                .radius(0.0)
                .radius,
            0.0
        );
        for value in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(
                std::panic::catch_unwind(
                    || Background("paint", EmptyView, Surface::Panel).radius(value)
                )
                .is_err()
            );
        }
    }

    #[test]
    fn existing_surface_tokens_map_to_current_palette_values() {
        let background = gpui::hsla(0.0, 0.0, 0.1, 1.0);
        let sidebar = gpui::hsla(0.0, 0.0, 0.2, 1.0);
        let panel = gpui::hsla(0.0, 0.0, 0.3, 1.0);
        for (token, expected) in [
            (Surface::Background, background),
            (Surface::Sidebar, sidebar),
            (Surface::Panel, panel),
        ] {
            assert_eq!(surface_color(token, background, sidebar, panel), expected);
        }
    }
}
