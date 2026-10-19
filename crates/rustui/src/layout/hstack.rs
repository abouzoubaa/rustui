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

    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing.max(0.0);
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
        self
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
