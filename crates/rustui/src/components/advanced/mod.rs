//! Advanced component adapters and planned extensions.

pub mod canvas;
pub mod code_editor;
pub mod graph_view;
pub mod node_editor;
pub mod property_inspector;
pub mod resizable_panel;

pub use code_editor::{CodeEditor, CodeEditorView};
pub use resizable_panel::ResizablePanel;

pub use canvas::{
    Canvas, CanvasError, CanvasPoint, CanvasRect, CanvasScene, CanvasSceneBuilder, CanvasSize,
    CanvasView,
};

pub use graph_view::{
    GraphDocument, GraphEdge, GraphEdit, GraphEndpoint, GraphError, GraphNode, GraphPort,
    GraphPortDirection, GraphStyle, GraphView, GraphViewControl, GraphViewport,
};
pub use node_editor::{NodeEditor, NodeEditorView};
