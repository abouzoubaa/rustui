//! Small interaction hints implemented with native GPUI styling.

use gpui::{AnyElement, Div, IntoElement as _, ParentElement as _, Styled as _, div};

use crate::view::{EmptyView, RenderContext, View};

/// A child view that shows the native pointing-hand cursor while hovered,
/// provided the child does not request its own cursor style.
///
/// Nested GPUI elements that request a cursor style keep their own style; this
/// wrapper does not override those requests. It is only a cursor hint: it does
/// not add a click handler, focus target, accessibility role, or independent
/// interaction runtime. Events and control semantics continue to come from the
/// child.
pub struct PointerCursorView<V: View> {
    child: V,
}

/// Shows the native pointing-hand cursor over `child` when it has no cursor
/// request of its own. Nested GPUI cursor requests take precedence.
///
/// This wrapper does not handle events or add focus/accessibility semantics.
#[allow(non_snake_case)]
pub fn PointerCursor<V: View>(child: V) -> PointerCursorView<V> {
    PointerCursorView { child }
}

impl<V: View> PointerCursorView<V> {
    fn native_element(&self) -> Div {
        div().cursor_pointer()
    }
}

impl<V: View> View for PointerCursorView<V> {
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
    fn native_wrapper_sets_pointer_hint_without_other_style_changes() {
        let mut native = PointerCursor(EmptyView).native_element();
        let style = native.style();

        assert_eq!(style.mouse_cursor, Some(gpui::CursorStyle::PointingHand));
        assert_eq!(style.position, None);
        assert_eq!(style.background, None);
    }
}
