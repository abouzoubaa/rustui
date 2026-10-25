use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use rustui_i18n::LayoutDirection;

use crate::view::{EmptyView, RenderContext, View};

type ChildRenderer = Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;

/// A vertical container for composed RustUI views.
pub struct VStack {
    children: Vec<ChildRenderer>,
    spacing: f32,
    alignment: HorizontalAlignment,
    fill: bool,
    grow: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum HorizontalAlignment {
    Start,
    Center,
    End,
    #[default]
    Stretch,
}

impl VStack {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 0.0,
            alignment: HorizontalAlignment::Stretch,
            fill: false,
            grow: false,
        }
    }

    /// Sets the gap between children in logical pixels.
    ///
    /// Panics if `spacing` is negative or nonfinite.
    pub fn spacing(mut self, spacing: f32) -> Self {
        assert!(
            spacing.is_finite() && spacing >= 0.0,
            "VStack spacing must be finite and nonnegative"
        );
        super::validate_gap_span(spacing, self.children.len(), "VStack");
        self.spacing = spacing;
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

    pub fn alignment(mut self, alignment: HorizontalAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn child<V: View>(mut self, child: V) -> Self {
        self.children
            .push(Box::new(move |context| child.into_element(context)));
        super::validate_gap_span(self.spacing, self.children.len(), "VStack");
        self
    }

    pub fn children<V: View>(mut self, children: impl IntoIterator<Item = V>) -> Self {
        for child in children {
            self = self.child(child);
        }
        self
    }

    /// Inserts a group's children as direct siblings without an extra layout node.
    pub fn group(mut self, group: super::group::Group) -> Self {
        self.children.extend(group.into_renderers());
        super::validate_gap_span(self.spacing, self.children.len(), "VStack");
        self
    }
}

#[cfg(test)]
mod tests {
    use super::VStack;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn spacing_accepts_finite_nonnegative_values() {
        assert_eq!(VStack::new().spacing(0.0).spacing, 0.0);
        assert_eq!(VStack::new().spacing(8.0).spacing, 8.0);
    }

    #[test]
    fn total_spacing_span_must_remain_finite() {
        use crate::view::EmptyView;

        assert!(
            std::panic::catch_unwind(|| {
                VStack::new()
                    .spacing(f32::MAX)
                    .child(EmptyView)
                    .child(EmptyView)
                    .child(EmptyView)
            })
            .is_err()
        );

        let stack = VStack::new()
            .child(EmptyView)
            .child(EmptyView)
            .child(EmptyView);
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| { stack.spacing(f32::MAX) }))
                .is_err()
        );
    }

    #[test]
    fn spacing_rejects_negative_and_nonfinite_values() {
        for spacing in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(catch_unwind(AssertUnwindSafe(|| VStack::new().spacing(spacing))).is_err());
        }
    }
}

impl Default for VStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for VStack {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let direction = context.layout_direction();
        let children = self.children.iter().map(|child| child(context));
        let mut stack = div().flex().flex_col().gap(px(self.spacing));
        if self.fill {
            stack = stack.size_full().min_w_0().min_h_0();
        }
        if self.grow {
            stack = stack.flex_1().min_w_0().min_h_0();
        }
        let stack = stack.children(children);
        match (self.alignment, direction) {
            (HorizontalAlignment::Start, LayoutDirection::RightToLeft)
            | (HorizontalAlignment::End, LayoutDirection::LeftToRight) => stack.items_end(),
            (HorizontalAlignment::Start, LayoutDirection::LeftToRight)
            | (HorizontalAlignment::End, LayoutDirection::RightToLeft) => stack.items_start(),
            (HorizontalAlignment::Center, _) => stack.items_center(),
            (HorizontalAlignment::Stretch, _) => stack.items_stretch(),
        }
        .into_any_element()
    }
}
