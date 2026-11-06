//! Positioned, controlled graph visualization using RustUI Canvas and GPUI input.

use super::canvas::{Canvas, CanvasPoint, CanvasRect, CanvasScene, CanvasSize};
use crate::{
    localization::MessageArguments,
    view::{ControlAction, EmptyView, RenderContext, View},
};
use gpui::{
    AnyElement, FocusHandle, Hsla, InteractiveElement as _, IntoElement as _, MouseButton,
    ParentElement as _, Role, StatefulInteractiveElement as _, Styled as _, div, px,
};
use rustui_core::{Binding, Identity};
use std::{
    cell::Cell,
    collections::{HashMap, HashSet},
    error::Error,
    fmt,
    rc::{Rc, Weak},
};

mod interaction;
mod model;
mod rendering;
mod validation;

pub use interaction::{GraphView, GraphViewControl};
pub use model::{
    GraphDocument, GraphEdge, GraphEdit, GraphEndpoint, GraphError, GraphNode, GraphPort,
    GraphPortDirection, GraphViewport,
};
pub(crate) use rendering::GraphAccessibilityLabel;
pub use rendering::GraphStyle;
pub(super) use rendering::render_graph;
