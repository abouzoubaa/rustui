//! Layout-neutral grouping of children for RustUI layout containers.

use gpui::AnyElement;

use crate::view::{RenderContext, View};

/// A deferred child lowered with the layout container's render context.
pub(crate) type ChildRenderer = Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;

/// An ordered collection of views inserted directly into a layout container.
///
/// `Group` is a child-list builder, not a [`View`]. Its children become siblings
/// in the receiving container, without an extra GPUI element or layout rule.
/// Wrap dynamic children in [`crate::view::Keyed`] to preserve their retained
/// identity when items are reordered or removed.
#[derive(Default)]
pub struct Group {
    children: Vec<ChildRenderer>,
}

impl Group {
    /// Creates an empty group.
    pub fn new() -> Self {
        Self::default()
    }

    /// Appends one child. Repeated calls may use different view types.
    pub fn child<V: View>(mut self, child: V) -> Self {
        self.children
            .push(Box::new(move |context| child.into_element(context)));
        self
    }

    /// Appends children of one view type in source order.
    pub fn children<V: View>(mut self, children: impl IntoIterator<Item = V>) -> Self {
        for child in children {
            self = self.child(child);
        }
        self
    }

    /// Appends another group's children without introducing a layout node.
    pub fn group(mut self, group: Group) -> Self {
        self.children.extend(group.children);
        self
    }

    /// Transfers child renderers to the receiving layout container.
    pub(crate) fn into_renderers(self) -> Vec<ChildRenderer> {
        self.children
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildRenderer, Group};

    fn marker(id: u8) -> ChildRenderer {
        Box::new(move |_| {
            std::hint::black_box(id);
            unreachable!("ordering checks do not lower children")
        })
    }

    fn identity(renderer: &ChildRenderer) -> *const () {
        renderer.as_ref() as *const _ as *const ()
    }

    #[test]
    fn nested_groups_flatten_in_source_order_without_wrappers() {
        let a = marker(0);
        let b = marker(1);
        let c = marker(2);
        let d = marker(3);
        let expected = [identity(&a), identity(&b), identity(&c), identity(&d)];
        let group = Group { children: vec![a] }
            .group(Group { children: vec![b] })
            .group(Group::new().group(Group {
                children: vec![c, d],
            }));

        let children = group.into_renderers();
        assert_eq!(children.len(), 4);
        assert_eq!(children.iter().map(identity).collect::<Vec<_>>(), expected);
    }
}
