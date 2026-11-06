//! A document outline with owner-controlled selection and branch expansion.

use std::collections::HashSet;

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement as _, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, div,
};
use rustui_core::{Binding, Identity};

use super::tree::{TreeNode, TreeView};
use crate::view::{ControlAction, EmptyView, RenderContext, View};

/// A single-column hierarchy whose owner controls both selected and expanded IDs.
///
/// IDs must be unique and nonblank; labels must be nonblank. Only branch IDs are
/// retained in `expanded`; removed IDs and leaf IDs are normalized away. A hidden
/// selected descendant remains selected in the owner binding without reopening
/// its ancestors. Per-node `TreeNode::expanded` declarations are ignored.
///
/// Call [`Self::accessibility_label`] before rendering. The name labels the outer
/// group; the shared Tree adapter supplies the named tree, row hierarchy levels
/// and selected/expanded semantics. Home/End select the first/last enabled
/// visible row, matching the Tree keyboard contract.
pub struct OutlineView {
    tree: TreeView,
    id: Identity,
    accessibility_label: Option<SharedString>,
}

impl OutlineView {
    pub fn new(
        id: impl Into<Identity>,
        nodes: impl IntoIterator<Item = TreeNode>,
        selection: Binding<Option<String>>,
        expanded: Binding<HashSet<String>>,
    ) -> Self {
        let id = id.into();
        let tree = TreeView::new(nodes, selection)
            .id(id.clone())
            .controlled_expansion(expanded);
        Self {
            tree,
            id,
            accessibility_label: None,
        }
    }

    /// Names the group containing the upstream semantic tree. Required before render.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Outline accessibility label must not be blank"
        );
        self.accessibility_label = Some(label);
        self
    }

    /// Sets the viewport height; finite values are clamped to at least one.
    pub fn height(mut self, height: f32) -> Self {
        self.tree = self.tree.height(height);
        self
    }

    /// Activates the selected enabled leaf; folders retain upstream expansion behavior.
    pub fn on_activate(mut self, action: impl Into<ControlAction>) -> Self {
        self.tree = self.tree.on_activate(action);
        self
    }
}

/// Creates an outline with stable identity and owner-controlled branch expansion.
#[allow(non_snake_case)]
pub fn Outline(
    id: impl Into<Identity>,
    nodes: impl IntoIterator<Item = TreeNode>,
    selection: Binding<Option<String>>,
    expanded: Binding<HashSet<String>>,
) -> OutlineView {
    OutlineView::new(id, nodes, selection, expanded)
}

impl View for OutlineView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let label = self
            .accessibility_label
            .as_ref()
            .expect("Outline requires an explicit accessibility_label before rendering");
        div()
            .id(format!("{}:outline", self.id))
            .role(Role::Group)
            .aria_label(label.clone())
            .child(self.tree.into_element(context))
            .into_any_element()
    }
}
