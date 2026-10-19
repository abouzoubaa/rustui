//! Advanced component adapters and planned extensions.

pub mod canvas;
pub mod code_editor;
pub mod graph_view;
pub mod node_editor;
pub mod property_inspector;
pub mod resizable_panel;

pub use code_editor::{CodeEditor, CodeEditorView};

pub use canvas::{
    Canvas, CanvasError, CanvasPoint, CanvasRect, CanvasScene, CanvasSceneBuilder, CanvasSize,
    CanvasView,
};
