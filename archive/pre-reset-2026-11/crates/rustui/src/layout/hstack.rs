use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use rustui_i18n::LayoutDirection;

use crate::view::{EmptyView, RenderContext, View};

type ChildRenderer = Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;

/// A horizontal container for composed RustUI views.
pub struct HStack {
    children: Vec<ChildRenderer>,
    spacing: f32,
    alignment: VerticalAlignment,
    fill: bool,
    grow: bool,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum VerticalAlignment {
    Start,
    Center,
    End,
    Baseline,
    #[default]
    Stretch,
}

impl HStack {
    pub fn new() -> Self {
        Self {
            children: Vec::new(),
            spacing: 0.0,
            alignment: VerticalAlignment::Stretch,
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
            "HStack spacing must be finite and nonnegative"
        );
        super::validate_gap_span(spacing, self.children.len(), "HStack");
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

    pub fn alignment(mut self, alignment: VerticalAlignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn child<V: View>(mut self, child: V) -> Self {
        self.children
            .push(Box::new(move |context| child.into_element(context)));
        super::validate_gap_span(self.spacing, self.children.len(), "HStack");
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
        super::validate_gap_span(self.spacing, self.children.len(), "HStack");
        self
    }
}

#[cfg(test)]
mod tests {
    use super::HStack;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn spacing_accepts_finite_nonnegative_values() {
        assert_eq!(HStack::new().spacing(0.0).spacing, 0.0);
        assert_eq!(HStack::new().spacing(8.0).spacing, 8.0);
    }

    #[test]
    fn total_spacing_span_must_remain_finite() {
        use crate::view::EmptyView;

        assert!(
            std::panic::catch_unwind(|| {
                HStack::new()
                    .spacing(f32::MAX)
                    .child(EmptyView)
                    .child(EmptyView)
                    .child(EmptyView)
            })
            .is_err()
        );

        let stack = HStack::new()
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
            assert!(catch_unwind(AssertUnwindSafe(|| HStack::new().spacing(spacing))).is_err());
        }
    }
}

impl Default for HStack {
    fn default() -> Self {
        Self::new()
    }
}

impl View for HStack {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let direction = context.layout_direction();
        let children = self.children.iter().map(|child| child(context));
        let mut stack = div().flex().gap(px(self.spacing));
        if self.fill {
            stack = stack.size_full().min_w_0().min_h_0();
        }
        if self.grow {
            stack = stack.flex_1().min_w_0().min_h_0();
        }
        let stack = match direction {
            LayoutDirection::LeftToRight => stack.flex_row(),
            LayoutDirection::RightToLeft => stack.flex_row_reverse(),
        }
        .children(children);
        match self.alignment {
            VerticalAlignment::Start => stack.items_start(),
            VerticalAlignment::Center => stack.items_center(),
            VerticalAlignment::End => stack.items_end(),
            VerticalAlignment::Baseline => stack.items_baseline(),
            VerticalAlignment::Stretch => stack.items_stretch(),
        }
        .into_any_element()
    }
}
