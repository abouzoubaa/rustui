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

    pub fn alignment(mut self, alignment: HorizontalAlignment) -> Self {
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
