//! Translation of a RustUI view using GPUI's native relative positioning.

use gpui::{AnyElement, Div, IntoElement as _, ParentElement as _, Styled as _, div, px};

use crate::view::{EmptyView, RenderContext, View};

/// A child view translated in logical pixels without changing its layout slot.
///
/// GPUI applies relative-position offsets after layout, so neighboring items
/// keep their original positions. The child is lowered under the translated
/// native element, allowing GPUI to use the moved bounds for pointer hit
/// testing and accessibility geometry. This modifier does not scale or rotate
/// arbitrary views.
pub struct OffsetView<V: View> {
    child: V,
    x: f32,
    y: f32,
}

/// Creates a child view with a zero translation.
///
/// Configure signed logical-pixel offsets with [`OffsetView::x`] and
/// [`OffsetView::y`]. Values must be finite.
#[allow(non_snake_case)]
pub fn Offset<V: View>(child: V) -> OffsetView<V> {
    OffsetView {
        child,
        x: 0.0,
        y: 0.0,
    }
}

impl<V: View> OffsetView<V> {
    /// Sets the horizontal logical-pixel offset. Negative values move left.
    ///
    /// Panics if `x` is not finite.
    pub fn x(mut self, x: f32) -> Self {
        check_offset("x", x);
        self.x = x;
        self
    }

    /// Sets the vertical logical-pixel offset. Negative values move up.
    ///
    /// Panics if `y` is not finite.
    pub fn y(mut self, y: f32) -> Self {
        check_offset("y", y);
        self.y = y;
        self
    }

    /// Sets both signed logical-pixel offsets.
    ///
    /// Panics if either coordinate is not finite.
    pub fn translation(mut self, x: f32, y: f32) -> Self {
        check_offset("x", x);
        check_offset("y", y);
        self.x = x;
        self.y = y;
        self
    }

    fn native_element(&self) -> Div {
        div().relative().left(px(self.x)).top(px(self.y))
    }
}

fn check_offset(axis: &'static str, value: f32) {
    assert!(value.is_finite(), "Offset {axis} must be finite");
}

impl<V: View> View for OffsetView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        self.native_element()
            .child(self.child.into_element(context))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::EmptyView;

    #[test]
    fn native_style_uses_relative_offsets_and_preserves_signed_values() {
        let offset = Offset(EmptyView).translation(-12.5, 8.0);
        let mut native = offset.native_element();
        let style = native.style();

        assert_eq!(style.position, Some(gpui::Position::Relative));
        assert_eq!(style.inset.left, Some(px(-12.5).into()));
        assert_eq!(style.inset.top, Some(px(8.0).into()));
        assert_eq!(style.inset.right, None);
        assert_eq!(style.inset.bottom, None);
    }

    #[test]
    fn axis_setters_replace_only_the_selected_offset() {
        let offset = Offset(EmptyView).x(-4.0).y(9.5).x(2.0);

        assert_eq!(offset.x, 2.0);
        assert_eq!(offset.y, 9.5);
    }

    #[test]
    fn rejects_non_finite_coordinates() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(std::panic::catch_unwind(|| Offset(EmptyView).x(value)).is_err());
            assert!(std::panic::catch_unwind(|| Offset(EmptyView).y(value)).is_err());
            assert!(
                std::panic::catch_unwind(|| Offset(EmptyView).translation(0.0, value)).is_err()
            );
        }
    }
}
