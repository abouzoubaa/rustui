//! Overlay stack layout backed by a single-cell GPUI grid.

use gpui::{AnyElement, IntoElement, ParentElement, Styled, div};
use rustui_i18n::LayoutDirection;

use crate::{
    layout::{HorizontalAlignment, VerticalAlignment},
    view::{EmptyView, RenderContext, View},
};

type ChildRenderer = Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;

/// Logical horizontal and vertical alignment for a [`ZStack`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZStackAlignment {
    horizontal: HorizontalAlignment,
    vertical: VerticalAlignment,
}

impl ZStackAlignment {
    /// Creates an explicit logical two-axis alignment.
    pub const fn new(horizontal: HorizontalAlignment, vertical: VerticalAlignment) -> Self {
        Self {
            horizontal,
            vertical,
        }
    }

    /// Returns the logical horizontal alignment.
    pub const fn horizontal(self) -> HorizontalAlignment {
        self.horizontal
    }

    /// Returns the vertical alignment.
    pub const fn vertical(self) -> VerticalAlignment {
        self.vertical
    }
}

impl Default for ZStackAlignment {
    fn default() -> Self {
        Self::new(HorizontalAlignment::Center, VerticalAlignment::Center)
    }
}

/// Layers RustUI views in one shared layout cell.
///
/// Children are lowered in insertion order; later children paint above earlier
/// children. Use [`crate::view::Keyed`] around dynamic children that need stable
/// retained identity.
pub struct ZStack {
    children: Vec<ChildRenderer>,
    alignment: ZStackAlignment,
    fill: bool,
    grow: bool,
}

impl ZStack {
    /// Creates an empty, centered overlay stack.
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            alignment: ZStackAlignment::default(),
            fill: false,
            grow: false,
        }
    }

    /// Sets logical horizontal and vertical overlay alignment.
    pub fn alignment(mut self, alignment: ZStackAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    /// Sets both dimensions to the available parent size.
    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    /// Takes available space along the parent's flex axis.
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }

    /// Appends one overlay child. Later children are painted above earlier ones.
    pub fn child<V: View>(mut self, child: V) -> Self {
        self.children
            .push(Box::new(move |context| child.into_element(context)));
        self
    }

    /// Appends overlay children in back-to-front order.
    pub fn children<V: View>(mut self, children: impl IntoIterator<Item = V>) -> Self {
        for child in children {
            self = self.child(child);
        }
        self
    }

    /// Inserts a group's children as direct siblings without an extra layout node.
    pub fn group(mut self, group: super::group::Group) -> Self {
        self.children.extend(group.into_renderers());
        self
    }
}

impl Default for ZStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for ZStack {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let direction = context.layout_direction();
        let children = self.children.iter().map(|child| {
            let content = child(context);
            let content = if self.alignment.horizontal == HorizontalAlignment::Stretch {
                div().w_full().child(content).into_any_element()
            } else {
                content
            };
            let mut frame = div().flex().flex_row();
            if reverse_horizontal_flow(direction) {
                frame = frame.flex_row_reverse();
            }
            frame = match self.alignment.horizontal {
                HorizontalAlignment::Start => frame.justify_start(),
                HorizontalAlignment::Center => frame.justify_center(),
                HorizontalAlignment::End => frame.justify_end(),
                HorizontalAlignment::Stretch => frame,
            };
            frame = match self.alignment.vertical {
                VerticalAlignment::Start => frame.items_start(),
                VerticalAlignment::Center => frame.items_center(),
                VerticalAlignment::End => frame.items_end(),
                VerticalAlignment::Baseline => frame.items_baseline(),
                VerticalAlignment::Stretch => frame.items_stretch(),
            };
            frame.col_start(1).row_start(1).child(content)
        });

        let mut stack = div().grid().grid_cols(1).grid_rows(1);
        if self.fill {
            stack = stack.size_full().min_w_0().min_h_0();
        }
        if self.grow {
            stack = stack.flex_1().min_w_0().min_h_0();
        }
        stack.children(children).into_any_element()
    }
}

fn reverse_horizontal_flow(direction: LayoutDirection) -> bool {
    direction == LayoutDirection::RightToLeft
}

#[cfg(test)]
mod tests {
    use super::{ZStack, ZStackAlignment, reverse_horizontal_flow};
    use crate::layout::{HorizontalAlignment, VerticalAlignment};
    use rustui_i18n::LayoutDirection;

    #[test]
    fn defaults_to_centered_two_axis_alignment() {
        let alignment = ZStack::new().alignment;
        assert_eq!(alignment.horizontal(), HorizontalAlignment::Center);
        assert_eq!(alignment.vertical(), VerticalAlignment::Center);
        assert_eq!(ZStackAlignment::default(), alignment);
    }

    #[test]
    fn logical_start_and_end_follow_locale_flow() {
        assert!(!reverse_horizontal_flow(LayoutDirection::LeftToRight));
        assert!(reverse_horizontal_flow(LayoutDirection::RightToLeft));
        let alignment = ZStackAlignment::new(HorizontalAlignment::Start, VerticalAlignment::End);
        assert_eq!(alignment.horizontal(), HorizontalAlignment::Start);
        assert_eq!(alignment.vertical(), VerticalAlignment::End);
    }
}
