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
                .accessibility_label("Node editor"),
            document,
        }
    }
    pub fn accessibility_label(mut self, name: impl Into<String>) -> Self {
        self.surface = self.surface.accessibility_label(name);
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
