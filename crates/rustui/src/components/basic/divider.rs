//! Semantic horizontal and vertical separators.

use crate::{
    components::theme::{required_style_with_artwork, selector, styled_container},
    view::{EmptyView, RenderContext, View},
};
use gpui::{AnyElement, Div, IntoElement, Styled, div, px};

fn divider_line(element: Div, vertical: bool) -> Div {
    let line = if vertical {
        element.w(px(1.)).h_full()
    } else {
        element.w_full().h(px(1.))
    };
    line.flex_shrink_0()
}

/// A decorative separator rendered with GPUI Components' active border color.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct DividerView {
    vertical: bool,
}

impl DividerView {
    /// Renders a vertical separator, for example between items in an HStack.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
}

/// Creates a horizontal separator.
#[allow(non_snake_case)]
pub fn Divider() -> DividerView {
    DividerView::default()
}

impl View for DividerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut query = selector(context);
        if self.vertical {
            query.orientation = "vertical".into();
            let style =
                required_style_with_artwork(context, "divider", "menubar_separator", &query);
            // The source record describes a menu screenshot's horizontal width.
            // The public vertical divider instead follows its parent height.
            divider_line(styled_container(div(), &style), true).into_any_element()
        } else {
            let style = required_style_with_artwork(context, "divider", "list_separator", &query);
            divider_line(styled_container(div(), &style), false).into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn divider_defaults_horizontal_and_can_be_vertical() {
        assert_eq!(Divider(), DividerView::default());
        assert!(Divider().vertical().vertical);
    }

    #[test]
    fn horizontal_and_vertical_lines_do_not_shrink() {
        let mut horizontal = divider_line(div(), false);
        assert_eq!(horizontal.style().flex_shrink, Some(0.0));
        assert_eq!(horizontal.style().size.height, Some(px(1.).into()));
        assert_eq!(
            horizontal.style().size.width,
            Some(gpui::relative(1.).into())
        );

        let mut vertical = divider_line(div(), true);
        assert_eq!(vertical.style().flex_shrink, Some(0.0));
        assert_eq!(vertical.style().size.width, Some(px(1.).into()));
        assert_eq!(
            vertical.style().size.height,
            Some(gpui::relative(1.).into())
        );
    }
}
