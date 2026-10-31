//! Controlled node editing on the same validated graph and native surface as GraphView.

use super::graph_view::{
    GraphDocument, GraphError, GraphStyle, GraphView, GraphViewControl, GraphViewport, render_graph,
};
use crate::view::{ControlAction, EmptyView, RenderContext, View};
use gpui::AnyElement;
use rustui_core::{Binding, Identity};

/// A node editor owns only focus and in-progress pointer/connection gestures.
/// The document, selected node and viewport remain in the enclosing component.
///
/// Drag a node to move it. Select an output and an input port to connect them;
/// ports are also Tab-focusable and respond to Enter/Space and AX Click. Each
/// connection has a named Disconnect control. With the surface focused, arrows
/// move the selected node by ten world points, brackets select adjacent nodes,
/// and Delete removes a node and its incident connections. Escape cancels the
/// gesture and clears selection. A/D/W/S pan, PageUp/PageDown zoom and Home resets
/// the viewport. IME, text-entry and modified key events are left untouched.
///
/// Use [`super::graph_view::GraphEdit`] for owner toolbar actions such as adding
/// and renaming nodes. Invalid edits return errors without changing the document;
/// stale UI edits become no-ops against the latest owner document.
/// Caller IDs must be unique and stable for each independently mounted editor.
pub struct NodeEditorView {
    surface: GraphViewControl,
    document: Binding<GraphDocument>,
}
#[allow(non_snake_case)]
pub fn NodeEditor(
    id: impl Into<Identity>,
    document: Binding<GraphDocument>,
    selection: Binding<Option<String>>,
    viewport: Binding<GraphViewport>,
) -> NodeEditorView {
    NodeEditorView::new(id, document, selection, viewport)
}
impl NodeEditorView {
    pub fn new(
        id: impl Into<Identity>,
        document: Binding<GraphDocument>,
        selection: Binding<Option<String>>,
        viewport: Binding<GraphViewport>,
    ) -> Self {
        Self {
            surface: GraphView(id, document.value(), selection, viewport)
                .accessibility_label("Node editor")
                .localized_accessibility_label("rustui-node-editor"),
            document,
        }
    }
    pub fn accessibility_label(mut self, name: impl Into<String>) -> Self {
        self.surface = self.surface.accessibility_label(name);
        self
    }

    /// Resolve the editor's accessible name from the active locale catalog.
    /// The current default or caller-supplied label remains its fallback.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<String>) -> Self {
        self.surface = self.surface.localized_accessibility_label(message_id);
        self
    }
    pub fn style(mut self, style: GraphStyle) -> Result<Self, GraphError> {
        self.surface = self.surface.style(style)?;
        Ok(self)
    }
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.surface = self.surface.disabled(disabled);
        self
    }
    pub fn on_activate(mut self, action: impl Into<ControlAction>) -> Self {
        self.surface = self.surface.on_activate(action);
        self
    }
}
impl View for NodeEditorView {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        render_graph(&self.surface, Some(self.document.clone()), context)
    }
}

#[cfg(test)]
mod tests {
    use super::super::canvas::CanvasSize;
    use super::*;
    use crate::components::advanced::graph_view::{GraphAccessibilityLabel, GraphEdge, GraphNode};

    fn binding<T: Clone + 'static>(value: T) -> Binding<T> {
        fn read<T: Clone>(value: &T) -> T {
            value.clone()
        }
        fn write<T>(value: &mut T, next: T) {
            *value = next;
        }
        Binding::new::<T>(value, read::<T>, write::<T>)
    }

    fn editor() -> NodeEditorView {
        let document =
            GraphDocument::new(Vec::<GraphNode>::new(), Vec::<GraphEdge>::new()).unwrap();
        let viewport =
            GraphViewport::new(CanvasSize::new(20.0, 20.0).unwrap(), 0.0, 0.0, 1.0).unwrap();
        NodeEditor(
            "localized-node-editor",
            binding(document),
            binding(None::<String>),
            binding(viewport),
        )
    }

    #[test]
    fn localized_accessibility_name_keeps_default_and_explicit_fallbacks() {
        let default = editor();
        assert!(matches!(
            default.surface.label,
            GraphAccessibilityLabel::Message { ref id, ref fallback }
                if id == "rustui-node-editor" && fallback == "Node editor"
        ));

        let custom = editor()
            .accessibility_label("Workflow editor")
            .localized_accessibility_label("rustui-node-editor");
        assert!(matches!(
            custom.surface.label,
            GraphAccessibilityLabel::Message { ref id, ref fallback }
                if id == "rustui-node-editor" && fallback == "Workflow editor"
        ));
    }
}
