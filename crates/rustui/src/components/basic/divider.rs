//! Semantic horizontal and vertical separators.

use crate::{
    components::theme::{required_style_with_artwork, selector, styled_container},
    view::{EmptyView, RenderContext, View},
};
use gpui::{AnyElement, IntoElement, Styled, div, px};

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
            styled_container(div(), &style)
                .w(px(1.))
                .h_full()
                .into_any_element()
        } else {
            let style = required_style_with_artwork(context, "divider", "list_separator", &query);
            styled_container(div(), &style)
                .w_full()
                .h(px(1.))
                .into_any_element()
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
}
