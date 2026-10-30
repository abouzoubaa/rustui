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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphError {
    BlankIdentity,
    BlankLabel,
    DuplicateNode,
    DuplicatePort,
    DuplicateEdge,
    MissingNode,
    MissingEdge,
    MissingPort,
    PortDirection,
    DuplicateConnection,
    InvalidGeometry,
}
impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::BlankIdentity => "graph identities must not be blank",
            Self::BlankLabel => "graph labels must not be blank",
            Self::DuplicateNode => "duplicate graph node identity",
            Self::DuplicatePort => "duplicate port identity within node",
            Self::DuplicateEdge => "duplicate graph edge identity",
            Self::MissingNode => "graph node does not exist",
            Self::MissingEdge => "graph edge does not exist",
            Self::MissingPort => "graph port does not exist",
            Self::PortDirection => "connect an output port to an input port",
            Self::DuplicateConnection => "connection already exists",
            Self::InvalidGeometry => "graph geometry must be finite and representable",
        })
    }
}
impl Error for GraphError {}
fn identity(value: &str) -> Result<(), GraphError> {
    if value.trim().is_empty() {
        Err(GraphError::BlankIdentity)
    } else {
        Ok(())
    }
}
fn label(value: &str) -> Result<(), GraphError> {
    if value.trim().is_empty() {
        Err(GraphError::BlankLabel)
    } else {
        Ok(())
    }
}

fn resolved_localized_message(id: &str, localized: String, fallback: &str) -> String {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.to_owned()
    } else {
        localized
    }
}

fn localized_graph_message(
    context: &RenderContext<'_>,
    id: &str,
    arguments: &MessageArguments,
    fallback: &str,
) -> String {
    let localized = context
        .localized_message_with_arguments(id, arguments)
        .unwrap_or_else(|_| id.to_owned());
    resolved_localized_message(id, localized, fallback)
}

fn graph_display_label(context: &RenderContext<'_>, key: Option<&str>, fallback: &str) -> String {
    match key {
        Some(key) => localized_graph_message(context, key, &MessageArguments::new(), fallback),
        None => fallback.to_owned(),
    }
}

impl GraphError {
    /// Message identifier for resolving this error through RustUI's locale catalogs.
    pub fn localization_key(&self) -> &'static str {
        match self {
            GraphError::BlankIdentity => "rustui-graph-error-blank-identity",
            GraphError::BlankLabel => "rustui-graph-error-blank-label",
            GraphError::DuplicateNode => "rustui-graph-error-duplicate-node",
            GraphError::DuplicatePort => "rustui-graph-error-duplicate-port",
            GraphError::DuplicateEdge => "rustui-graph-error-duplicate-edge",
            GraphError::MissingNode => "rustui-graph-error-missing-node",
            GraphError::MissingEdge => "rustui-graph-error-missing-edge",
            GraphError::MissingPort => "rustui-graph-error-missing-port",
            GraphError::PortDirection => "rustui-graph-error-port-direction",
            GraphError::DuplicateConnection => "rustui-graph-error-duplicate-connection",
            GraphError::InvalidGeometry => "rustui-graph-error-invalid-geometry",
        }
    }
}

fn localized_graph_error(context: &RenderContext<'_>, error: &GraphError) -> String {
    let id = error.localization_key();
    localized_graph_message(context, id, &MessageArguments::new(), &error.to_string())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphPortDirection {
    Input,
    Output,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPort {
    id: String,
    label: String,
    label_key: Option<String>,
    direction: GraphPortDirection,
}
impl GraphPort {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        direction: GraphPortDirection,
    ) -> Result<Self, GraphError> {
        let port = Self {
            id: id.into(),
            label: name.into(),
            label_key: None,
            direction,
        };
        identity(&port.id)?;
        label(&port.label)?;
        Ok(port)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Resolve a display label at render time, falling back to the supplied label.
    pub fn localized_label(mut self, key: impl Into<String>) -> Self {
        self.label_key = Some(key.into());
        self
    }
    pub fn direction(&self) -> GraphPortDirection {
        self.direction
    }
}
/// Caller-supplied world-space geometry. No automatic layout or domain policy is imposed.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    id: String,
    label: String,
    label_key: Option<String>,
    rect: CanvasRect,
    ports: Vec<GraphPort>,
}
impl GraphNode {
    pub fn new(
        id: impl Into<String>,
        name: impl Into<String>,
        rect: CanvasRect,
        ports: impl IntoIterator<Item = GraphPort>,
    ) -> Result<Self, GraphError> {
        let node = Self {
            id: id.into(),
            label: name.into(),
            label_key: None,
            rect,
            ports: ports.into_iter().collect(),
        };
        node.validate()?;
        Ok(node)
    }
    fn validate(&self) -> Result<(), GraphError> {
        identity(&self.id)?;
        label(&self.label)?;
        CanvasRect::new(self.rect.x, self.rect.y, self.rect.width, self.rect.height)
            .map_err(|_| GraphError::InvalidGeometry)?;
        let mut ids = HashSet::new();
        for port in &self.ports {
            identity(&port.id)?;
            label(&port.label)?;
            if !ids.insert(&port.id) {
                return Err(GraphError::DuplicatePort);
            }
        }
        Ok(())
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn label(&self) -> &str {
        &self.label
    }
    /// Resolve a display label at render time, falling back to the supplied label.
    pub fn localized_label(mut self, key: impl Into<String>) -> Self {
        self.label_key = Some(key.into());
        self
    }
    pub fn rect(&self) -> CanvasRect {
        self.rect
    }
    pub fn ports(&self) -> &[GraphPort] {
        &self.ports
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEndpoint {
    pub node: String,
    pub port: String,
}
impl GraphEndpoint {
    pub fn new(node: impl Into<String>, port: impl Into<String>) -> Self {
        Self {
            node: node.into(),
            port: port.into(),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphEdge {
    id: String,
    from: GraphEndpoint,
    to: GraphEndpoint,
}
impl GraphEdge {
    pub fn new(
        id: impl Into<String>,
        from: GraphEndpoint,
        to: GraphEndpoint,
    ) -> Result<Self, GraphError> {
        let edge = Self {
            id: id.into(),
            from,
            to,
        };
        identity(&edge.id)?;
        identity(&edge.from.node)?;
        identity(&edge.from.port)?;
        identity(&edge.to.node)?;
        identity(&edge.to.port)?;
        Ok(edge)
    }
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn from(&self) -> &GraphEndpoint {
        &self.from
    }
    pub fn to(&self) -> &GraphEndpoint {
        &self.to
    }
}
/// Immutable validated topology. Cycles, self connections and fan-out are allowed;
/// duplicate directed endpoint pairs are rejected. Port IDs are local to a node.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphDocument {
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    node_index: HashMap<String, usize>,
}
impl GraphDocument {
    pub fn new(
        nodes: impl IntoIterator<Item = GraphNode>,
        edges: impl IntoIterator<Item = GraphEdge>,
    ) -> Result<Self, GraphError> {
        let mut graph = Self {
            nodes: nodes.into_iter().collect(),
            edges: edges.into_iter().collect(),
            node_index: HashMap::new(),
        };
        graph.validate()?;
        graph.rebuild_node_index();
        Ok(graph)
    }
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.node_index.get(id).map(|&index| &self.nodes[index])
    }

    fn rebuild_node_index(&mut self) {
        self.node_index = self
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| (node.id.clone(), index))
            .collect();
    }
    fn endpoint(&self, endpoint: &GraphEndpoint) -> Result<(&GraphNode, &GraphPort), GraphError> {
        let node = self.node(&endpoint.node).ok_or(GraphError::MissingNode)?;
        Ok((
            node,
            node.ports
                .iter()
                .find(|port| port.id == endpoint.port)
                .ok_or(GraphError::MissingPort)?,
        ))
    }
    fn validate(&self) -> Result<(), GraphError> {
        let mut nodes = HashMap::with_capacity(self.nodes.len());
        for node in &self.nodes {
            node.validate()?;
            if nodes.insert(&node.id, node).is_some() {
                return Err(GraphError::DuplicateNode);
            }
        }
        let endpoint = |endpoint: &GraphEndpoint| {
            let node = nodes.get(&endpoint.node).ok_or(GraphError::MissingNode)?;
            node.ports
                .iter()
                .find(|port| port.id == endpoint.port)
                .ok_or(GraphError::MissingPort)
        };
        let mut ids = HashSet::new();
        let mut endpoints = HashSet::new();
        for edge in &self.edges {
            identity(&edge.id)?;
            if !ids.insert(&edge.id) {
                return Err(GraphError::DuplicateEdge);
            }
            if !endpoints.insert((
                &edge.from.node,
                &edge.from.port,
                &edge.to.node,
                &edge.to.port,
            )) {
                return Err(GraphError::DuplicateConnection);
            }
            if endpoint(&edge.from)?.direction != GraphPortDirection::Output
                || endpoint(&edge.to)?.direction != GraphPortDirection::Input
            {
                return Err(GraphError::PortDirection);
            }
        }
        Ok(())
    }
    /// Applies one atomic edit to a clone. Failure leaves the original untouched.
    pub fn edited(&self, edit: GraphEdit) -> Result<Self, GraphError> {
        let mut next = self.clone();
        match edit {
            GraphEdit::AddNode(node) => next.nodes.push(node),
            GraphEdit::RenameNode { id, label: name } => {
                label(&name)?;
                let node = next
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == id)
                    .ok_or(GraphError::MissingNode)?;
                node.label = name;
                node.label_key = None;
            }
            GraphEdit::MoveNode { id, x, y } => {
                let n = next
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == id)
                    .ok_or(GraphError::MissingNode)?;
                n.rect = CanvasRect::new(x, y, n.rect.width, n.rect.height)
                    .map_err(|_| GraphError::InvalidGeometry)?;
                // Geometry cannot change topology or node indices in this validated document.
                return Ok(next);
            }
            GraphEdit::RemoveNode(id) => {
                if next.node(&id).is_none() {
                    return Err(GraphError::MissingNode);
                }
                next.nodes.retain(|n| n.id != id);
                next.edges.retain(|e| e.from.node != id && e.to.node != id);
            }
            GraphEdit::Connect(edge) => next.edges.push(edge),
            GraphEdit::Disconnect(id) => {
                let count = next.edges.len();
                next.edges.retain(|e| e.id != id);
                if count == next.edges.len() {
                    return Err(GraphError::MissingEdge);
                }
            }
        }
        next.validate()?;
        next.rebuild_node_index();
        Ok(next)
    }

    /// Rename a node with a locale-dependent display label and literal fallback.
    pub fn renamed_node_localized(
        &self,
        id: impl Into<String>,
        label: impl Into<String>,
        key: impl Into<String>,
    ) -> Result<Self, GraphError> {
        let id = id.into();
        let name = label.into();
        self::label(&name)?;
        let mut next = self.clone();
        let node = next
            .nodes
            .iter_mut()
            .find(|node| node.id == id)
            .ok_or(GraphError::MissingNode)?;
        node.label = name;
        node.label_key = Some(key.into());
        Ok(next)
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum GraphEdit {
    AddNode(GraphNode),
    RenameNode { id: String, label: String },
    MoveNode { id: String, x: f32, y: f32 },
    RemoveNode(String),
    Connect(GraphEdge),
    Disconnect(String),
}

/// World-space origin and positive zoom. Coordinates are transformed in f64 and
/// checked again before reaching GPUI. A zero-sized canvas is a valid collapsed view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GraphViewport {
    size: CanvasSize,
    x: f32,
    y: f32,
    zoom: f32,
}
impl GraphViewport {
    pub fn new(size: CanvasSize, x: f32, y: f32, zoom: f32) -> Result<Self, GraphError> {
        if !x.is_finite() || !y.is_finite() || !zoom.is_finite() || zoom <= 0.0 {
            return Err(GraphError::InvalidGeometry);
        }
        Ok(Self { size, x, y, zoom })
    }
    pub fn size(self) -> CanvasSize {
        self.size
    }
    pub fn origin(self) -> CanvasPoint {
        CanvasPoint {
            x: self.x,
            y: self.y,
        }
    }
    pub fn zoom(self) -> f32 {
        self.zoom
    }
    pub fn pan(self, dx: f32, dy: f32) -> Result<Self, GraphError> {
        Self::new(self.size, self.x + dx, self.y + dy, self.zoom)
    }
    pub fn with_zoom(self, zoom: f32) -> Result<Self, GraphError> {
        Self::new(self.size, self.x, self.y, zoom)
    }
    fn rect(self, rect: CanvasRect) -> Result<CanvasRect, GraphError> {
        CanvasRect::new(
            ((f64::from(rect.x) - f64::from(self.x)) * f64::from(self.zoom)) as f32,
            ((f64::from(rect.y) - f64::from(self.y)) * f64::from(self.zoom)) as f32,
            (f64::from(rect.width) * f64::from(self.zoom)) as f32,
            (f64::from(rect.height) * f64::from(self.zoom)) as f32,
        )
        .map_err(|_| GraphError::InvalidGeometry)
    }
}

const DISCONNECT_CONTROL_SIZE: f32 = 24.0;

fn incident_edges(graph: &GraphDocument) -> HashMap<&str, Vec<&GraphEdge>> {
    let mut incident: HashMap<&str, Vec<&GraphEdge>> = HashMap::new();
    for edge in &graph.edges {
        incident.entry(&edge.from.node).or_default().push(edge);
        if edge.from.node != edge.to.node {
            incident.entry(&edge.to.node).or_default().push(edge);
        }
    }
    incident
}

fn projected_port_endpoint(
    graph: &GraphDocument,
    viewport: GraphViewport,
    endpoint: &GraphEndpoint,
) -> Result<CanvasPoint, GraphError> {
    let (node, port) = graph.endpoint(endpoint)?;
    let rect = viewport.rect(node.rect)?;
    let index = node
        .ports
        .iter()
        .position(|candidate| candidate.id == port.id)
        .unwrap();
    CanvasPoint::new(
        if port.direction == GraphPortDirection::Input {
            rect.x
        } else {
            rect.x + rect.width
        },
        rect.y + rect.height * ((index as f32 + 1.5) / (node.ports.len() + 1) as f32),
    )
    .map_err(|_| GraphError::InvalidGeometry)
}

fn disconnect_control_origin(
    from: CanvasPoint,
    to: CanvasPoint,
) -> Result<CanvasPoint, GraphError> {
    let half_size = f64::from(DISCONNECT_CONTROL_SIZE) / 2.0;
    CanvasPoint::new(
        ((f64::from(from.x) + f64::from(to.x)) / 2.0 - half_size) as f32,
        ((f64::from(from.y) + f64::from(to.y)) / 2.0 - half_size) as f32,
    )
    .map_err(|_| GraphError::InvalidGeometry)
}

/// Colors are independent of graph topology and may be supplied from a public theme.
#[derive(Clone, Copy)]
pub struct GraphStyle {
    pub node: Hsla,
    pub text: Hsla,
    pub edge: Hsla,
    pub selection: Hsla,
}
impl Default for GraphStyle {
    fn default() -> Self {
        Self {
            node: gpui::rgb(0x27354a).into(),
            text: gpui::rgb(0xffffff).into(),
            edge: gpui::rgb(0x8296b4).into(),
            selection: gpui::rgb(0x64b5ff).into(),
        }
    }
}
impl GraphStyle {
    fn validate(self) -> Result<Self, GraphError> {
        for color in [self.node, self.text, self.edge, self.selection] {
            if ![color.h, color.s, color.l, color.a]
                .into_iter()
                .all(|v| v.is_finite() && (0.0..=1.0).contains(&v))
            {
                return Err(GraphError::InvalidGeometry);
            }
        }
        Ok(self)
    }
}

#[derive(Clone, Debug)]
pub(super) enum GraphAccessibilityLabel {
    Literal(String),
    Message { id: String, fallback: String },
}

impl GraphAccessibilityLabel {
    fn fallback(&self) -> &str {
        match self {
            Self::Literal(label)
            | Self::Message {
                fallback: label, ..
            } => label,
        }
    }

    fn resolve(&self, context: &RenderContext<'_>) -> String {
        match self {
            Self::Literal(label) => label.clone(),
            Self::Message { id, fallback } => {
                localized_graph_message(context, id, &MessageArguments::new(), fallback)
            }
        }
    }
}

pub struct GraphViewControl {
    pub(super) id: Identity,
    pub(super) graph: GraphDocument,
    pub(super) selection: Binding<Option<String>>,
    pub(super) viewport: Binding<GraphViewport>,
    pub(super) style: Option<GraphStyle>,
    pub(super) label: GraphAccessibilityLabel,
    pub(super) activate: Option<ControlAction>,
    pub(super) disabled: bool,
}
#[allow(non_snake_case)]
pub fn GraphView(
    id: impl Into<Identity>,
    graph: GraphDocument,
    selection: Binding<Option<String>>,
    viewport: Binding<GraphViewport>,
) -> GraphViewControl {
    GraphViewControl::new(id, graph, selection, viewport)
}
impl GraphViewControl {
    pub fn new(
        id: impl Into<Identity>,
        graph: GraphDocument,
        selection: Binding<Option<String>>,
        viewport: Binding<GraphViewport>,
    ) -> Self {
        let id = id.into();
        identity(id.as_str()).expect("GraphView requires a stable identity");
        Self {
            id,
            graph,
            selection,
            viewport,
            style: None,
            label: GraphAccessibilityLabel::Message {
                id: "rustui-graph-view".into(),
                fallback: "Graph".into(),
            },
            activate: None,
            disabled: false,
        }
    }
    pub fn style(mut self, style: GraphStyle) -> Result<Self, GraphError> {
        self.style = Some(style.validate()?);
        Ok(self)
    }
    pub fn accessibility_label(mut self, name: impl Into<String>) -> Self {
        let name = name.into();
        label(&name).expect("GraphView requires an accessible name");
        self.label = GraphAccessibilityLabel::Literal(name);
        self
    }

    /// Resolve the view's accessible name from the active locale catalog.
    /// The current default or caller-supplied label remains its fallback.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<String>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "GraphView accessibility localization key must not be blank"
        );
        let fallback = self.label.fallback().to_owned();
        self.label = GraphAccessibilityLabel::Message { id, fallback };
        self
    }
    pub fn on_activate(mut self, action: impl Into<ControlAction>) -> Self {
        self.activate = Some(action.into());
        self
    }
    pub fn disabled(mut self, value: bool) -> Self {
        self.disabled = value;
        self
    }
}
impl View for GraphViewControl {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        render_graph(self, None, context)
    }
}

struct Interaction {
    focus: FocusHandle,
    drag: Option<Drag>,
    pending: Option<GraphEndpoint>,
    error: Option<GraphError>,
    viewport: GraphViewport,
}
#[derive(Clone)]
struct Drag {
    id: String,
    pointer: gpui::Point<gpui::Pixels>,
    rect: CanvasRect,
}
#[derive(Clone)]
struct Lease {
    epoch: Weak<Cell<u64>>,
    generation: u64,
}
impl Lease {
    fn current(&self) -> bool {
        self.epoch
            .upgrade()
            .is_some_and(|epoch| epoch.get() == self.generation)
    }
}

fn connection(
    graph: &GraphDocument,
    first: GraphEndpoint,
    second: GraphEndpoint,
) -> Result<GraphEdge, GraphError> {
    let direction = graph.endpoint(&first)?.1.direction;
    let (from, to) = if direction == GraphPortDirection::Output {
        (first, second)
    } else {
        (second, first)
    };
    let id = next_connection_id(&graph.edges);
    let edge = GraphEdge::new(id, from, to)?;
    if graph
        .edges
        .iter()
        .any(|existing| existing.from == edge.from && existing.to == edge.to)
    {
        return Err(GraphError::DuplicateConnection);
    }
    if graph.endpoint(&edge.from)?.1.direction != GraphPortDirection::Output
        || graph.endpoint(&edge.to)?.1.direction != GraphPortDirection::Input
    {
        return Err(GraphError::PortDirection);
    }
    Ok(edge)
}

fn next_connection_id(edges: &[GraphEdge]) -> String {
    let used_ids: HashSet<&str> = edges.iter().map(|edge| edge.id.as_str()).collect();
    let mut ordinal = 0_u64;
    let id = loop {
        let id = format!("connection-{ordinal}");
        if !used_ids.contains(id.as_str()) {
            break id;
        }
        ordinal += 1;
    };
    id
}

pub(super) fn render_graph(
    view: &GraphViewControl,
    editor: Option<Binding<GraphDocument>>,
    context: &mut RenderContext<'_>,
) -> AnyElement {
    let graph = &view.graph;
    let viewport = view.viewport.value();
    let style = view
        .style
        .unwrap_or_else(|| implicit_graph_style(context, view.disabled));
    let (state, _) = context.retained_entity(
        Identity::from(format!("graph-interaction:{}", view.id)),
        |_, cx| Interaction {
            focus: cx.focus_handle(),
            drag: None,
            pending: None,
            error: None,
            viewport,
        },
    );
    let (epoch_owner, _) = context.retained_entity(
        Identity::from(format!("graph-callbacks:{}", view.id)),
        |_, _| Rc::new(Cell::new(0_u64)),
    );
    let epoch = epoch_owner.read(context.app());
    let generation = epoch.get().wrapping_add(1);
    epoch.set(generation);
    let lease = Lease {
        epoch: Rc::downgrade(epoch),
        generation,
    };
    context.update_entity(&state, |state, window, cx| {
        if state.viewport != viewport {
            state.drag = None;
            state.viewport = viewport;
        }
        state.focus.clone().tab_stop(!view.disabled);
        if view.disabled {
            if state.focus.contains_focused(window, cx) {
                window.blur(cx);
            }
            state.drag = None;
            state.pending = None;
        }
        if state
            .drag
            .as_ref()
            .is_some_and(|drag| graph.node(&drag.id).is_none())
        {
            state.drag = None;
        }
        if state
            .pending
            .as_ref()
            .is_some_and(|endpoint| graph.endpoint(endpoint).is_err())
        {
            state.pending = None;
        }
    });
    let retained = state.read(context.app());
    let focus = retained.focus.clone();
    let pending = retained.pending.clone();
    let error = retained.error;
    // Omitting the component drops the retained epoch owner. Old callbacks hold
    // only Weak references; every render invalidates the preceding callback set.
    let selected = view.selection.value().filter(|id| graph.node(id).is_some());
    let rects = graph
        .nodes
        .iter()
        .map(|node| viewport.rect(node.rect))
        .collect::<Result<Vec<_>, _>>();
    let Ok(rects) = rects else {
        return projection_error(view, &focus, &lease, context);
    };
    let mut scene = CanvasScene::builder();
    for edge in &graph.edges {
        let drawn = projected_port_endpoint(graph, viewport, &edge.from).and_then(|from| {
            projected_port_endpoint(graph, viewport, &edge.to).and_then(|to| {
                if from == to {
                    let corner = CanvasPoint::new(from.x + 12.0, from.y - 12.0)
                        .map_err(|_| GraphError::InvalidGeometry)?;
                    let bend = CanvasPoint::new(from.x + 24.0, from.y)
                        .map_err(|_| GraphError::InvalidGeometry)?;
                    for (a, b) in [(from, corner), (corner, bend), (bend, to)] {
                        scene
                            .stroke_line(a, b, 1.5, style.edge)
                            .map_err(|_| GraphError::InvalidGeometry)?;
                    }
                    return Ok(());
                }
                scene
                    .stroke_line(from, to, 1.5, style.edge)
                    .map(|_| ())
                    .map_err(|_| GraphError::InvalidGeometry)
            })
        });
        if drawn.is_err() {
            return projection_error(view, &focus, &lease, context);
        }
    }
    let canvas = Canvas(viewport.size, scene.build()).into_element(context);
    let mut root = div()
        .id(graph_element_id("graph", &[view.id.as_str()]))
        .relative()
        .w(px(viewport.size.width()))
        .h(px(viewport.size.height()))
        .overflow_hidden()
        .border_1().border_color(gpui::transparent_black())
        .role(Role::Group)
        .aria_disabled(view.disabled)
        .aria_label(view.label.resolve(context))
        .aria_description(localized_graph_message(
            context,
            if editor.is_some() {
                "rustui-graph-editor-help"
            } else {
                "rustui-graph-view-help"
            },
            &MessageArguments::new(),
            if editor.is_some() {
                "Arrows move selected node; brackets select; Delete removes. A, D, W, and S pan; Page Up and Page Down zoom; Home resets; Escape cancels. Tab reaches ports; select output and input to connect."
            } else {
                "Arrows select nodes; Enter activates. A, D, W, and S pan; Page Up and Page Down zoom; Home resets; Escape clears selection."
            },
        ))
        .child(canvas);
    if !view.disabled {
        root = root
            .track_focus(&focus)
            .focus_visible(move |style_ref| style_ref.border_color(style.selection));
    }
    let incident = incident_edges(graph);
    for (node, rect) in graph.nodes.iter().zip(rects) {
        let writer = context.binding_writer(view.selection.clone());
        let id = node.id.clone();
        let click_focus = focus.clone();
        let guard = lease.clone();
        let disabled = view.disabled;
        let key_writer = context.binding_writer(view.selection.clone());
        let key_id = node.id.clone();
        let key_guard = lease.clone();
        let key_focus = focus.clone();
        let ax_writer = context.binding_writer(view.selection.clone());
        let ax_id = node.id.clone();
        let ax_guard = lease.clone();
        let ax_focus = focus.clone();
        let relationships = incident
            .get(node.id.as_str())
            .into_iter()
            .flatten()
            .map(|edge| {
                let from = graph.endpoint(&edge.from).unwrap();
                let to = graph.endpoint(&edge.to).unwrap();
                format!(
                    "{} {} to {} {}",
                    graph_display_label(context, from.0.label_key.as_deref(), &from.0.label),
                    graph_display_label(context, from.1.label_key.as_deref(), &from.1.label),
                    graph_display_label(context, to.0.label_key.as_deref(), &to.0.label),
                    graph_display_label(context, to.1.label_key.as_deref(), &to.1.label),
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let display_node_label =
            graph_display_label(context, node.label_key.as_deref(), &node.label);
        let node_label = if relationships.is_empty() {
            display_node_label.clone()
        } else {
            format!("{}; {}", display_node_label, relationships)
        };
        let pointer_activate = view
            .activate
            .clone()
            .map(|action| context.bind_keyboard_action(action));
        let key_activate = view
            .activate
            .clone()
            .map(|action| context.bind_keyboard_action(action));
        let ax_activate = view
            .activate
            .clone()
            .map(|action| context.bind_keyboard_action(action));
        let row_height = rect.height / (node.ports.len() + 1) as f32;
        let mut card = div()
            .id(graph_element_id(
                "node",
                &[view.id.as_str(), node.id.as_str()],
            ))
            .absolute()
            .left(px(rect.x))
            .top(px(rect.y))
            .w(px(rect.width))
            .h(px(rect.height))
            .overflow_hidden()
            .bg(style.node)
            .text_color(style.text)
            .border_1()
            .border_color(if selected.as_deref() == Some(node.id.as_str()) {
                style.selection
            } else {
                style.edge
            })
            .role(Role::Group)
            .aria_label(node_label)
            .aria_disabled(view.disabled)
            .aria_selected(selected.as_deref() == Some(node.id.as_str()))
            .on_click(move |event, window, cx| {
                if !disabled && guard.current() {
                    click_focus.focus(window, cx);
                    writer(Some(id.clone()), cx);
                    if event.click_count() == 2 {
                        if let Some(activate) = &pointer_activate {
                            activate(window, cx);
                        }
                    }
                }
            })
            .on_key_down(move |event, window, cx| {
                if !disabled
                    && key_guard.current()
                    && eligible(event)
                    && matches!(event.keystroke.key.as_ref(), "enter" | "space")
                {
                    key_focus.focus(window, cx);
                    key_writer(Some(key_id.clone()), cx);
                    if event.keystroke.key == "enter" {
                        if let Some(activate) = &key_activate {
                            activate(window, cx);
                        }
                    }
                    cx.stop_propagation();
                }
            })
            .on_a11y_action(gpui::accesskit::Action::Click, move |_, window, cx| {
                if !disabled && ax_guard.current() {
                    ax_focus.focus(window, cx);
                    ax_writer(Some(ax_id.clone()), cx);
                    if let Some(activate) = &ax_activate {
                        activate(window, cx);
                    }
                }
            })
            .child(
                div()
                    .h(px(row_height))
                    .overflow_hidden()
                    .child(display_node_label.clone()),
            );
        #[cfg(test)]
        {
            let selector = format!("graph-test-node:{}", node.id);
            card = card.debug_selector(move || selector);
        }
        if !view.disabled {
            card = card
                .focusable()
                .focus_visible(move |style_ref| style_ref.border_color(style.selection));
        }
        if editor.is_some() && !view.disabled {
            let drag_state = state.clone();
            let id = node.id.clone();
            let world_rect = node.rect;
            let guard = lease.clone();
            let drag_focus = focus.clone();
            let writer = context.binding_writer(view.selection.clone());
            card = card.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                if !guard.current() {
                    return;
                }
                drag_focus.focus(window, cx);
                writer(Some(id.clone()), cx);
                drag_state.update(cx, |s, _| {
                    s.drag = Some(Drag {
                        id: id.clone(),
                        pointer: event.position,
                        rect: world_rect,
                    })
                });
                cx.stop_propagation();
            });
            for port in &node.ports {
                let endpoint = GraphEndpoint::new(node.id.clone(), port.id.clone());
                let port_state = state.clone();
                let guard = lease.clone();
                let binding = editor.as_ref().unwrap().clone();
                let read_graph = context.binding_reader(binding.clone());
                let update = context.binding_event_updater(binding, apply_edit);
                let refresh =
                    context.binding_event_updater(view.selection.clone(), |_, (): ()| true);
                let handler: Rc<dyn Fn(&mut gpui::Window, &mut gpui::App)> =
                    Rc::new(move |_, cx| {
                        if !guard.current() {
                            return;
                        }
                        let Some(graph) = read_graph(cx) else {
                            return;
                        };
                        let mut edit = None;
                        port_state.update(cx, |s, cx| {
                            if let Some(first) = s.pending.take() {
                                match connection(&graph, first, endpoint.clone()) {
                                    Ok(edge) => {
                                        edit = Some(GraphEdit::Connect(edge));
                                        s.error = None;
                                    }
                                    Err(error) => s.error = Some(error),
                                }
                            } else {
                                s.pending = Some(endpoint.clone());
                                s.error = None;
                            }
                            cx.notify();
                        });
                        if let Some(edit) = edit {
                            update(edit, cx);
                        } else {
                            refresh((), cx);
                        }
                    });
                let pointer = handler.clone();
                let keyboard = handler.clone();
                let accessible = handler;
                let port_selected =
                    pending.as_ref() == Some(&GraphEndpoint::new(node.id.clone(), port.id.clone()));
                let (message_id, direction) = match port.direction {
                    GraphPortDirection::Input => ("rustui-graph-port-input", "input"),
                    GraphPortDirection::Output => ("rustui-graph-port-output", "output"),
                };
                let display_port_label =
                    graph_display_label(context, port.label_key.as_deref(), &port.label);
                let port_label = localized_graph_message(
                    context,
                    message_id,
                    &MessageArguments::new()
                        .with("node", display_node_label.clone())
                        .with("port", display_port_label.clone()),
                    &format!(
                        "{} {} {}",
                        display_node_label, direction, display_port_label
                    ),
                );
                let port_control = div()
                    .id(graph_element_id(
                        "port",
                        &[view.id.as_str(), node.id.as_str(), port.id.as_str()],
                    ))
                    .h(px(row_height))
                    .overflow_hidden()
                    .role(Role::Button)
                    .focusable()
                    .border_1()
                    .border_color(if port_selected {
                        style.selection
                    } else {
                        gpui::transparent_black()
                    })
                    .focus_visible(move |style_ref| style_ref.border_color(style.selection))
                    .aria_label(port_label)
                    .aria_selected(port_selected)
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .on_click(move |_, window, cx| {
                        pointer(window, cx);
                        cx.stop_propagation();
                    })
                    .on_key_down(move |event, window, cx| {
                        if eligible(event)
                            && matches!(event.keystroke.key.as_ref(), "enter" | "space")
                        {
                            keyboard(window, cx);
                            cx.stop_propagation();
                        }
                    })
                    .on_a11y_action(gpui::accesskit::Action::Click, move |_, window, cx| {
                        accessible(window, cx)
                    })
                    .child(format!(
                        "{} {}",
                        if port.direction == GraphPortDirection::Input {
                            "←"
                        } else {
                            "→"
                        },
                        display_port_label
                    ));
                #[cfg(test)]
                let port_control = {
                    let selector = format!("graph-test-port:{}:{}", node.id, port.id);
                    port_control.debug_selector(move || selector)
                };
                card = card.child(port_control);
            }
        }
        if editor.is_none() || view.disabled {
            for port in &node.ports {
                card = card.child(div().h(px(row_height)).overflow_hidden().child(format!(
                    "{} {}",
                    if port.direction == GraphPortDirection::Input {
                        "←"
                    } else {
                        "→"
                    },
                    graph_display_label(context, port.label_key.as_deref(), &port.label)
                )));
            }
        }
        root = root.child(card);
    }
    // Every edge is named independently of the canvas, including for assistive
    // technology. Editor controls provide keyboard/AX disconnection parity.
    for edge in &graph.edges {
        let source = graph.endpoint(&edge.from).unwrap();
        let target = graph.endpoint(&edge.to).unwrap();
        let source_node =
            graph_display_label(context, source.0.label_key.as_deref(), &source.0.label);
        let source_port =
            graph_display_label(context, source.1.label_key.as_deref(), &source.1.label);
        let target_node =
            graph_display_label(context, target.0.label_key.as_deref(), &target.0.label);
        let target_port =
            graph_display_label(context, target.1.label_key.as_deref(), &target.1.label);
        let description = localized_graph_message(
            context,
            "rustui-graph-disconnect",
            &MessageArguments::new()
                .with("source_node", source_node.clone())
                .with("source_port", source_port.clone())
                .with("target_node", target_node.clone())
                .with("target_port", target_port.clone()),
            &format!(
                "Disconnect {} {} to {} {}",
                source_node, source_port, target_node, target_port
            ),
        );
        if let Some(binding) = editor.as_ref().filter(|_| !view.disabled) {
            let update = context.binding_event_updater(binding.clone(), apply_edit);
            let id = edge.id.clone();
            let guard = lease.clone();
            let disconnect_focus = focus.clone();
            let handler: Rc<dyn Fn(&mut gpui::Window, &mut gpui::App)> =
                Rc::new(move |window, cx| {
                    if guard.current() {
                        disconnect_focus.focus(window, cx);
                        update(GraphEdit::Disconnect(id.clone()), cx);
                    }
                });
            let pointer = handler.clone();
            let keyboard = handler.clone();
            let accessible = handler;
            let from = projected_port_endpoint(graph, viewport, &edge.from).unwrap();
            let to = projected_port_endpoint(graph, viewport, &edge.to).unwrap();
            let control_origin = disconnect_control_origin(from, to).unwrap();
            root = root.child(
                div()
                    .id(graph_element_id(
                        "edge",
                        &[view.id.as_str(), edge.id.as_str()],
                    ))
                    .absolute()
                    .left(px(control_origin.x))
                    .top(px(control_origin.y))
                    .w(px(DISCONNECT_CONTROL_SIZE))
                    .h(px(DISCONNECT_CONTROL_SIZE))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(style.node)
                    .text_color(style.text)
                    .border_1()
                    .border_color(style.edge)
                    .role(Role::Button)
                    .focusable()
                    .focus_visible(move |style_ref| style_ref.border_color(style.selection))
                    .aria_label(description.clone())
                    .on_click(move |_, window, cx| {
                        pointer(window, cx);
                        cx.stop_propagation();
                    })
                    .on_key_down(move |event, window, cx| {
                        if eligible(event)
                            && matches!(event.keystroke.key.as_ref(), "enter" | "space")
                        {
                            keyboard(window, cx);
                            cx.stop_propagation();
                        }
                    })
                    .on_a11y_action(gpui::accesskit::Action::Click, move |_, window, cx| {
                        accessible(window, cx)
                    })
                    .child("×"),
            );
        }
    }
    let read_selection = context.binding_reader(view.selection.clone());
    let read_document = editor
        .as_ref()
        .map(|binding| context.binding_reader(binding.clone()));
    let rendered_graph = graph.clone();
    let key_focus = focus.clone();
    let guard = lease.clone();
    let disabled = view.disabled;
    let select = context.binding_writer(view.selection.clone());
    let viewport_graph = graph.clone();
    let pan = context.binding_event_updater(view.viewport.clone(), move |current, command| {
        let Ok(next) = viewport_command(*current, command) else {
            return false;
        };
        if *current == next
            || viewport_graph
                .nodes
                .iter()
                .any(|node| next.rect(node.rect).is_err())
        {
            return false;
        }
        *current = next;
        true
    });
    let edit = editor
        .as_ref()
        .map(|binding| context.binding_event_updater(binding.clone(), apply_edit));
    let move_node = editor
        .as_ref()
        .map(|binding| context.binding_event_updater(binding.clone(), translate_node));
    let advance = context.binding_event_updater(
        view.selection.clone(),
        |current, (graph, forward): (GraphDocument, bool)| {
            advance_selection(&graph, current, forward)
        },
    );
    let activate = view
        .activate
        .clone()
        .map(|action| context.bind_keyboard_action(action));
    let cancel_state = state.clone();
    root = root.on_key_down(move |event, window, cx| {
        if !guard.current()
            || !surface_keyboard_event_eligible(
                event,
                disabled,
                key_focus.is_focused(window),
                key_focus.contains_focused(window, cx),
            )
        {
            return;
        }
        let key = event.keystroke.key.as_ref();
        let current_graph = if matches!(
            key,
            "left" | "right" | "up" | "down" | "enter" | "delete" | "backspace" | "[" | "]"
        ) {
            read_document
                .as_ref()
                .map_or_else(|| Some(rendered_graph.clone()), |read| read(cx))
        } else {
            None
        };
        let key_selected = current_keyboard_selection(read_selection(cx), current_graph.as_ref());
        let handled = match key {
            "escape" => {
                select(None, cx);
                cancel_state.update(cx, |s, cx| {
                    s.drag = None;
                    s.pending = None;
                    s.error = None;
                    cx.notify();
                });
                true
            }
            "enter" if key_selected.is_some() => {
                if let Some(activate) = &activate {
                    activate(window, cx);
                }
                true
            }
            "delete" | "backspace" if edit.is_some() && key_selected.is_some() => {
                edit.as_ref().unwrap()(GraphEdit::RemoveNode(key_selected.clone().unwrap()), cx);
                select(None, cx);
                true
            }
            "left" | "right" | "up" | "down" if edit.is_some() && key_selected.is_some() => {
                let (dx, dy) = match key {
                    "left" => (-10.0, 0.0),
                    "right" => (10.0, 0.0),
                    "up" => (0.0, -10.0),
                    _ => (0.0, 10.0),
                };
                move_node.as_ref().unwrap()((key_selected.clone().unwrap(), dx, dy), cx);
                true
            }
            "left" | "up" | "right" | "down" | "[" | "]" => {
                let forward = matches!(key, "right" | "down" | "]");
                if let Some(graph) = current_graph {
                    advance((graph, forward), cx);
                }
                true
            }
            "a" | "d" | "w" | "s" => {
                let (dx, dy) = match key {
                    "a" => (-40.0, 0.0),
                    "d" => (40.0, 0.0),
                    "w" => (0.0, -40.0),
                    _ => (0.0, 40.0),
                };
                pan(ViewportCommand::Pan(dx, dy), cx);
                true
            }
            "pageup" | "pagedown" => {
                pan(
                    ViewportCommand::Scale(if key == "pageup" { 1.25 } else { 0.8 }),
                    cx,
                );
                true
            }
            "home" => {
                pan(ViewportCommand::Reset, cx);
                true
            }
            _ => false,
        };
        if handled {
            cx.stop_propagation();
        }
    });
    if let Some(binding) = editor.filter(|_| !view.disabled) {
        let update = context.binding_event_updater(binding, apply_edit);
        let drag_state = state.clone();
        let guard = lease.clone();
        root = root.on_mouse_move(move |event, window, cx| {
            if !guard.current() {
                return;
            }
            if event.pressed_button != Some(MouseButton::Left)
                || !drag_state.read(cx).focus.is_focused(window)
            {
                drag_state.update(cx, |state, _| state.drag = None);
                return;
            }
            let drag = drag_state.read(cx).drag.clone();
            if let Some(drag) = drag {
                let dx = f32::from(event.position.x - drag.pointer.x) / viewport.zoom;
                let dy = f32::from(event.position.y - drag.pointer.y) / viewport.zoom;
                update(
                    GraphEdit::MoveNode {
                        id: drag.id,
                        x: drag.rect.x + dx,
                        y: drag.rect.y + dy,
                    },
                    cx,
                );
            }
        });
        let end = state.clone();
        let outside = state.clone();
        let end_guard = lease.clone();
        let outside_guard = lease.clone();
        root = root
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                if end_guard.current() {
                    end.update(cx, |s, _| s.drag = None)
                }
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                if outside_guard.current() {
                    outside.update(cx, |s, _| s.drag = None)
                }
            });
    }
    if let Some(error) = error {
        root = root.child(
            div()
                .id(graph_element_id("error", &[view.id.as_str()]))
                .absolute()
                .bottom_0()
                .role(Role::Status)
                .child(localized_graph_error(context, &error)),
        );
    }
    root.into_any_element()
}
fn eligible(event: &gpui::KeyDownEvent) -> bool {
    !event.prefer_character_input
        && !event.keystroke.is_ime_in_progress()
        && !event.keystroke.modifiers.modified()
}
fn next_node(graph: &GraphDocument, selected: Option<&str>, forward: bool) -> Option<String> {
    let count = graph.nodes.len();
    if count == 0 {
        return None;
    }
    let index = selected
        .and_then(|id| graph.nodes.iter().position(|n| n.id == id))
        .map_or(if forward { 0 } else { count - 1 }, |index| {
            if forward {
                (index + 1) % count
            } else {
                (index + count - 1) % count
            }
        });
    Some(graph.nodes[index].id.clone())
}

fn apply_edit(current: &mut GraphDocument, edit: GraphEdit) -> bool {
    match current.edited(edit) {
        Ok(next) if next != *current => {
            *current = next;
            true
        }
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_connection_ids_fill_gaps_and_keep_exact_names() {
        let edge = |id: String| {
            GraphEdge::new(
                id,
                GraphEndpoint::new("a", "out"),
                GraphEndpoint::new("b", "in"),
            )
            .unwrap()
        };
        let mut edges = vec![
            edge("connection-0".into()),
            edge("connection-2".into()),
            edge("connection-01".into()),
        ];
        assert_eq!(next_connection_id(&edges), "connection-1");
        edges = (0..1_024)
            .map(|id| edge(format!("connection-{id}")))
            .collect();
        assert_eq!(next_connection_id(&edges), "connection-1024");
    }

    #[test]
    fn connection_validation_matches_full_graph_validation() {
        fn reference(
            graph: &GraphDocument,
            first: GraphEndpoint,
            second: GraphEndpoint,
        ) -> Result<GraphEdge, GraphError> {
            let (from, to) = if graph.endpoint(&first)?.1.direction == GraphPortDirection::Output {
                (first, second)
            } else {
                (second, first)
            };
            let edge = GraphEdge::new(next_connection_id(&graph.edges), from, to)?;
            graph.edited(GraphEdit::Connect(edge.clone()))?;
            Ok(edge)
        }
        let populated = graph().edited(GraphEdit::Connect(edge())).unwrap();
        let endpoints = [
            GraphEndpoint::new("a", "out"),
            GraphEndpoint::new("a", "in"),
            GraphEndpoint::new("b", "out"),
            GraphEndpoint::new("b", "in"),
            GraphEndpoint::new("missing", "out"),
            GraphEndpoint::new("a", "missing"),
            GraphEndpoint::new("", "out"),
        ];
        for document in [graph(), populated] {
            for first in &endpoints {
                for second in &endpoints {
                    assert_eq!(
                        connection(&document, first.clone(), second.clone()),
                        reference(&document, first.clone(), second.clone()),
                        "first={first:?}, second={second:?}"
                    );
                }
            }
        }
    }
    fn node(id: &str, x: f32) -> GraphNode {
        GraphNode::new(
            id,
            id,
            CanvasRect::new(x, 10.0, 120.0, 90.0).unwrap(),
            [
                GraphPort::new("in", "Input", GraphPortDirection::Input).unwrap(),
                GraphPort::new("out", "Output", GraphPortDirection::Output).unwrap(),
            ],
        )
        .unwrap()
    }
    fn graph() -> GraphDocument {
        GraphDocument::new([node("a", 10.0), node("b", 240.0)], []).unwrap()
    }

    #[test]
    fn validates_large_graph_and_rejects_missing_late_endpoint() {
        let nodes: Vec<_> = (0..512)
            .map(|index| node(&format!("node-{index}"), index as f32))
            .collect();
        let mut edges: Vec<_> = (0..511)
            .map(|index| {
                GraphEdge::new(
                    format!("edge-{index}"),
                    GraphEndpoint::new(format!("node-{index}"), "out"),
                    GraphEndpoint::new(format!("node-{}", index + 1), "in"),
                )
                .unwrap()
            })
            .collect();
        assert!(GraphDocument::new(nodes.clone(), edges.clone()).is_ok());
        edges.push(
            GraphEdge::new(
                "missing-last",
                GraphEndpoint::new("node-511", "out"),
                GraphEndpoint::new("missing", "in"),
            )
            .unwrap(),
        );
        assert_eq!(
            GraphDocument::new(nodes, edges),
            Err(GraphError::MissingNode)
        );
    }

    #[test]
    fn node_lookup_tracks_removal_and_reinsertion() {
        let original = graph();
        let removed = original.edited(GraphEdit::RemoveNode("a".into())).unwrap();
        assert!(removed.node("a").is_none());
        assert_eq!(removed.node("b").unwrap().id(), "b");
        assert!(original.node("a").is_some());
        let restored = removed.edited(GraphEdit::AddNode(node("a", 50.0))).unwrap();
        assert_eq!(restored.node("a").unwrap().rect().x, 50.0);
        assert_eq!(restored.node("b").unwrap().id(), "b");
    }

    #[test]
    fn incident_edge_index_preserves_order_and_counts_self_edges_once() {
        let self_edge = GraphEdge::new(
            "self",
            GraphEndpoint::new("a", "out"),
            GraphEndpoint::new("a", "in"),
        )
        .unwrap();
        let graph =
            GraphDocument::new([node("a", 0.0), node("b", 1.0)], [edge(), self_edge]).unwrap();
        let incident = incident_edges(&graph);
        assert_eq!(
            incident["a"]
                .iter()
                .map(|edge| edge.id())
                .collect::<Vec<_>>(),
            ["ab", "self"]
        );
        assert_eq!(
            incident["b"]
                .iter()
                .map(|edge| edge.id())
                .collect::<Vec<_>>(),
            ["ab"]
        );
        assert!(incident.get("missing").is_none());
    }

    fn test_binding<T: Clone + 'static>(value: T) -> Binding<T> {
        fn read<T: Clone>(value: &T) -> T {
            value.clone()
        }
        fn write<T>(value: &mut T, next: T) {
            *value = next;
        }
        Binding::new::<T>(value, read::<T>, write::<T>)
    }

    fn label_test_view() -> GraphViewControl {
        let viewport =
            GraphViewport::new(CanvasSize::new(20.0, 20.0).unwrap(), 0.0, 0.0, 1.0).unwrap();
        GraphView(
            "localized-graph",
            GraphDocument::new(Vec::<GraphNode>::new(), Vec::<GraphEdge>::new()).unwrap(),
            test_binding(None),
            test_binding(viewport),
        )
    }

    #[test]
    fn graph_accessibility_names_resolve_catalogs_and_keep_literal_fallbacks() {
        let default = label_test_view();
        assert!(matches!(
            default.label,
            GraphAccessibilityLabel::Message { ref id, ref fallback }
                if id == "rustui-graph-view" && fallback == "Graph"
        ));

        let view = label_test_view()
            .accessibility_label("Custom graph")
            .localized_accessibility_label("rustui-graph-view");
        let GraphAccessibilityLabel::Message { id, fallback } = view.label else {
            panic!("expected localized GraphView label");
        };
        assert_eq!(id, "rustui-graph-view");
        assert_eq!(fallback, "Custom graph");

        let locale = crate::localization::Locale::parse("fr").unwrap();
        let catalog = crate::localization::Catalog::parse(
            locale.clone(),
            include_str!("../../../../../locales/fr/rustui.ftl"),
        )
        .unwrap();
        let translated = catalog.get(&id).unwrap().to_owned();
        assert_eq!(
            resolved_localized_message(&id, translated, &fallback),
            "Graphe"
        );
        assert_eq!(
            resolved_localized_message(&id, id.clone(), &fallback),
            "Custom graph"
        );
        assert_eq!(
            resolved_localized_message(&id, " ".into(), &fallback),
            "Custom graph"
        );

        let arguments = MessageArguments::new()
            .with("source_node", "Source")
            .with("source_port", "Sortie")
            .with("target_node", "Cible")
            .with("target_port", "Entrée");
        let template = catalog.get("rustui-graph-disconnect").unwrap();
        let disconnect =
            crate::localization::interpolate_with_locale(template, &arguments, &locale).unwrap();
        assert!(disconnect.contains("Déconnecter"));
        assert!(disconnect.contains("vers"));

        let port_args = MessageArguments::new()
            .with("node", "Source")
            .with("port", "Sortie");
        for key in ["rustui-graph-port-input", "rustui-graph-port-output"] {
            let template = catalog.get(key).unwrap();
            let translated =
                crate::localization::interpolate_with_locale(template, &port_args, &locale)
                    .unwrap();
            assert!(translated.contains("Source"));
            assert!(translated.contains("Sortie"));
        }
        assert_eq!(
            catalog.get("rustui-graph-reset-view").unwrap(),
            "La vue est hors limites. Réinitialiser la vue"
        );
        assert!(
            catalog
                .get("rustui-graph-editor-help")
                .unwrap()
                .contains("flèches")
        );
        assert!(
            catalog
                .get("rustui-graph-view-help")
                .unwrap()
                .contains("flèches")
        );

        let error = GraphError::MissingNode;
        let error_id = error.localization_key();
        assert_eq!(error.to_string(), "graph node does not exist");
        let english_locale = crate::localization::Locale::parse("en").unwrap();
        let english_catalog = crate::localization::Catalog::parse(
            english_locale,
            include_str!("../../../../../locales/en/rustui.ftl"),
        )
        .unwrap();
        assert_eq!(
            resolved_localized_message(
                error_id,
                english_catalog.get(error_id).unwrap().to_owned(),
                &error.to_string(),
            ),
            error.to_string()
        );
        assert_eq!(
            resolved_localized_message(
                error_id,
                catalog.get(error_id).unwrap().to_owned(),
                &error.to_string(),
            ),
            "Le nœud du graphe est introuvable."
        );
        assert_eq!(
            resolved_localized_message(error_id, error_id.to_owned(), &error.to_string(),),
            "graph node does not exist"
        );
    }
    fn edge() -> GraphEdge {
        GraphEdge::new(
            "ab",
            GraphEndpoint::new("a", "out"),
            GraphEndpoint::new("b", "in"),
        )
        .unwrap()
    }
    #[test]
    fn disconnect_control_tracks_projected_port_midpoint() {
        let graph = graph();
        let size = CanvasSize::new(500.0, 300.0).unwrap();
        let viewport = GraphViewport::new(size, 5.0, 10.0, 2.0).unwrap();
        let edge = edge();
        let from = projected_port_endpoint(&graph, viewport, edge.from()).unwrap();
        let to = projected_port_endpoint(&graph, viewport, edge.to()).unwrap();

        assert_eq!(from, CanvasPoint::new(250.0, 150.0).unwrap());
        assert_eq!(to, CanvasPoint::new(470.0, 90.0).unwrap());
        let origin = disconnect_control_origin(from, to).unwrap();
        assert_eq!(origin, CanvasPoint::new(348.0, 108.0).unwrap());
        assert_eq!(
            origin.x + DISCONNECT_CONTROL_SIZE / 2.0,
            (from.x + to.x) / 2.0
        );
        assert_eq!(
            origin.y + DISCONNECT_CONTROL_SIZE / 2.0,
            (from.y + to.y) / 2.0
        );
    }

    #[test]
    fn validates_identity_labels_and_references() {
        assert_eq!(
            GraphDocument::new([node("a", 0.0), node("a", 1.0)], []),
            Err(GraphError::DuplicateNode)
        );
        let missing = GraphEdge::new(
            "missing",
            GraphEndpoint::new("a", "out"),
            GraphEndpoint::new("x", "in"),
        )
        .unwrap();
        assert_eq!(
            graph().edited(GraphEdit::Connect(missing)),
            Err(GraphError::MissingNode)
        );
        let missing = GraphEdge::new(
            "missing",
            GraphEndpoint::new("a", "out"),
            GraphEndpoint::new("b", "unknown"),
        )
        .unwrap();
        assert_eq!(
            graph().edited(GraphEdit::Connect(missing)),
            Err(GraphError::MissingPort)
        );
        assert_eq!(
            GraphPort::new(" ", "Port", GraphPortDirection::Input),
            Err(GraphError::BlankIdentity)
        );
        assert_eq!(
            GraphPort::new("p", " ", GraphPortDirection::Input),
            Err(GraphError::BlankLabel)
        );
        assert_eq!(
            GraphNode::new(
                "a",
                "A",
                node("a", 0.0).rect(),
                [
                    GraphPort::new("p", "Port", GraphPortDirection::Input).unwrap(),
                    GraphPort::new("p", "Port", GraphPortDirection::Output).unwrap()
                ]
            ),
            Err(GraphError::DuplicatePort)
        );
    }
    #[test]
    fn connection_validation_is_atomic_and_directional() {
        let graph = graph();
        let connected = graph.edited(GraphEdit::Connect(edge())).unwrap();
        assert!(graph.edges().is_empty());
        assert_eq!(
            connected.edited(GraphEdit::Connect(edge())),
            Err(GraphError::DuplicateEdge)
        );
        let duplicate =
            GraphEdge::new("another", edge().from().clone(), edge().to().clone()).unwrap();
        assert_eq!(
            connected.edited(GraphEdit::Connect(duplicate)),
            Err(GraphError::DuplicateConnection)
        );
        let wrong = GraphEdge::new(
            "wrong",
            GraphEndpoint::new("a", "in"),
            GraphEndpoint::new("b", "out"),
        )
        .unwrap();
        assert_eq!(
            graph.edited(GraphEdit::Connect(wrong)),
            Err(GraphError::PortDirection)
        );
        assert_eq!(
            connection(
                &graph,
                GraphEndpoint::new("b", "in"),
                GraphEndpoint::new("a", "out")
            )
            .unwrap()
            .from(),
            &GraphEndpoint::new("a", "out")
        );
    }
    #[test]
    fn supports_cycles_and_removes_incident_edges_with_node() {
        let graph = graph().edited(GraphEdit::Connect(edge())).unwrap();
        let graph = graph
            .edited(GraphEdit::Connect(
                GraphEdge::new(
                    "ba",
                    GraphEndpoint::new("b", "out"),
                    GraphEndpoint::new("a", "in"),
                )
                .unwrap(),
            ))
            .unwrap();
        let graph = graph.edited(GraphEdit::RemoveNode("a".into())).unwrap();
        assert_eq!(graph.nodes().len(), 1);
        assert!(graph.edges().is_empty());
        assert_eq!(
            graph.edited(GraphEdit::RemoveNode("absent".into())),
            Err(GraphError::MissingNode)
        );
        assert_eq!(
            graph.edited(GraphEdit::Disconnect("absent".into())),
            Err(GraphError::MissingEdge)
        );
    }
    #[test]
    fn edits_preserve_ids_and_reject_invalid_geometry_without_partial_change() {
        let original = graph();
        assert_eq!(
            original.edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: f32::INFINITY,
                y: 0.0
            }),
            Err(GraphError::InvalidGeometry)
        );
        assert_eq!(
            original.edited(GraphEdit::RenameNode {
                id: "a".into(),
                label: " ".into()
            }),
            Err(GraphError::BlankLabel)
        );
        let moved = original
            .edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: -25.0,
                y: 30.0,
            })
            .unwrap();
        assert_eq!(moved.node("a").unwrap().rect().x, -25.0);
        assert_eq!(original.node("a").unwrap().rect().x, 10.0);
        let renamed = moved
            .edited(GraphEdit::RenameNode {
                id: "a".into(),
                label: "Renamed".into(),
            })
            .unwrap();
        assert_eq!(renamed.node("a").unwrap().label(), "Renamed");
    }
    #[test]
    fn moved_connected_graph_retains_valid_topology_and_node_index() {
        let original = graph().edited(GraphEdit::Connect(edge())).unwrap();
        let moved = original
            .edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: 45.0,
                y: -12.0,
            })
            .unwrap();
        let rebuilt = GraphDocument::new(moved.nodes().to_vec(), moved.edges().to_vec()).unwrap();
        assert_eq!(moved, rebuilt);
        assert_eq!(moved.node("a").unwrap().rect().x, 45.0);
        assert_eq!(original.node("a").unwrap().rect().x, 10.0);
        assert_eq!(
            moved
                .edited(GraphEdit::RemoveNode("a".into()))
                .unwrap()
                .edges()
                .len(),
            0
        );
        assert_eq!(
            moved.edited(GraphEdit::MoveNode {
                id: "missing".into(),
                x: 0.0,
                y: 0.0
            }),
            Err(GraphError::MissingNode)
        );
    }
    #[test]
    fn localized_node_labels_follow_explicit_renames() {
        let original =
            GraphDocument::new([node("a", 10.0).localized_label("sample-original")], []).unwrap();
        let localized = original
            .renamed_node_localized("a", "Renamed", "sample-renamed")
            .unwrap();
        assert_eq!(
            localized.node("a").unwrap().label_key.as_deref(),
            Some("sample-renamed")
        );
        assert_eq!(
            original.node("a").unwrap().label_key.as_deref(),
            Some("sample-original")
        );
        let literal = localized
            .edited(GraphEdit::RenameNode {
                id: "a".into(),
                label: "User text".into(),
            })
            .unwrap();
        assert_eq!(literal.node("a").unwrap().label(), "User text");
        assert_eq!(literal.node("a").unwrap().label_key, None);
    }

    #[test]
    fn event_edits_apply_to_latest_document_and_stale_edits_are_noops() {
        let mut latest = graph()
            .edited(GraphEdit::AddNode(node("c", 400.0)))
            .unwrap();
        assert!(apply_edit(
            &mut latest,
            GraphEdit::MoveNode {
                id: "a".into(),
                x: 20.0,
                y: 30.0
            }
        ));
        assert!(latest.node("c").is_some());
        assert!(apply_edit(&mut latest, GraphEdit::RemoveNode("a".into())));
        let before = latest.clone();
        assert!(!apply_edit(
            &mut latest,
            GraphEdit::MoveNode {
                id: "a".into(),
                x: 1.0,
                y: 1.0
            }
        ));
        assert_eq!(latest, before);
    }
    #[test]
    fn rejects_forged_public_canvas_geometry_and_nonfinite_viewports() {
        let forged = CanvasRect {
            x: f32::MAX,
            y: 0.0,
            width: f32::MAX,
            height: 1.0,
        };
        assert_eq!(
            GraphNode::new("n", "Node", forged, []),
            Err(GraphError::InvalidGeometry)
        );
        let size = CanvasSize::new(500.0, 300.0).unwrap();
        for value in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert_eq!(
                GraphViewport::new(size, 0.0, 0.0, value),
                Err(GraphError::InvalidGeometry)
            );
        }
        let viewport = GraphViewport::new(size, 0.0, 0.0, f32::MAX).unwrap();
        assert_eq!(
            viewport.rect(node("n", 10.0).rect()),
            Err(GraphError::InvalidGeometry)
        );
        let viewport = GraphViewport::new(size, 5.0, 10.0, 2.0).unwrap();
        assert_eq!(
            viewport
                .rect(CanvasRect::new(10.0, 20.0, 100.0, 50.0).unwrap())
                .unwrap(),
            CanvasRect::new(10.0, 20.0, 200.0, 100.0).unwrap()
        );
    }
    #[test]
    fn callback_lease_rejects_prior_renders_and_unmounted_owner() {
        let owner = Rc::new(Cell::new(1));
        let lease = Lease {
            epoch: Rc::downgrade(&owner),
            generation: 1,
        };
        assert!(lease.current());
        owner.set(2);
        assert!(!lease.current());
        let current = Lease {
            epoch: Rc::downgrade(&owner),
            generation: 2,
        };
        drop(owner);
        assert!(!current.current());
    }
    #[test]
    fn native_keyboard_events_preserve_text_input_and_modifiers() {
        let mut event = gpui::KeyDownEvent {
            keystroke: gpui::Keystroke::parse("left").unwrap(),
            is_held: false,
            prefer_character_input: false,
        };
        assert!(eligible(&event));
        event.prefer_character_input = true;
        assert!(!eligible(&event));
        event.prefer_character_input = false;
        event.keystroke = gpui::Keystroke::parse("ctrl-left").unwrap();
        assert!(!eligible(&event));
    }
    #[test]
    fn keyboard_selection_uses_stable_ids_and_handles_empty_and_removed_nodes() {
        let graph = graph();
        assert_eq!(next_node(&graph, None, true).as_deref(), Some("a"));
        assert_eq!(next_node(&graph, Some("a"), true).as_deref(), Some("b"));
        assert_eq!(next_node(&graph, Some("a"), false).as_deref(), Some("b"));
        assert_eq!(
            next_node(&graph, Some("removed"), false).as_deref(),
            Some("b")
        );
        assert_eq!(
            next_node(&GraphDocument::new([], []).unwrap(), None, true),
            None
        );
    }
}

#[derive(Clone, Copy)]
enum ViewportCommand {
    Pan(f32, f32),
    Scale(f32),
    Reset,
}
fn viewport_command(
    viewport: GraphViewport,
    command: ViewportCommand,
) -> Result<GraphViewport, GraphError> {
    match command {
        ViewportCommand::Pan(x, y) => viewport.pan(x / viewport.zoom, y / viewport.zoom),
        ViewportCommand::Scale(factor) => viewport.with_zoom(viewport.zoom * factor),
        ViewportCommand::Reset => GraphViewport::new(viewport.size, 0.0, 0.0, 1.0),
    }
}

fn projection_error(
    view: &GraphViewControl,
    focus: &FocusHandle,
    lease: &Lease,
    context: &mut RenderContext<'_>,
) -> AnyElement {
    let update = context.binding_event_updater(view.viewport.clone(), |current, (): ()| {
        let next = GraphViewport::new(current.size, 0.0, 0.0, 1.0).unwrap();
        if *current == next {
            return false;
        }
        *current = next;
        true
    });
    let guard = lease.clone();
    let disabled = view.disabled;
    let action: Rc<dyn Fn(&mut gpui::App)> = Rc::new(move |cx| {
        if !disabled && guard.current() {
            update((), cx);
        }
    });
    let pointer = action.clone();
    let keyboard = action.clone();
    let accessible = action;
    let reset_label = localized_graph_message(
        context,
        "rustui-graph-reset-view",
        &MessageArguments::new(),
        "View is out of range. Reset view",
    );
    let mut reset = div()
        .id(graph_element_id("graph", &[view.id.as_str()]))
        .role(Role::Button)
        .aria_disabled(disabled)
        .aria_label(reset_label.clone())
        .on_click(move |_, _, cx| pointer(cx))
        .on_key_down(move |event, _, cx| {
            if eligible(event) && matches!(event.keystroke.key.as_ref(), "home" | "enter" | "space")
            {
                keyboard(cx);
                cx.stop_propagation();
            }
        })
        .on_a11y_action(gpui::accesskit::Action::Click, move |_, _, cx| {
            accessible(cx)
        })
        .child(reset_label);
    if !disabled {
        reset = reset.track_focus(focus);
    }
    reset.into_any_element()
}

fn graph_element_id(kind: &str, parts: &[&str]) -> String {
    let mut id = format!("rustui-graph-{kind}");
    for part in parts {
        use std::fmt::Write as _;
        write!(&mut id, " {}:{}", part.len(), part).unwrap();
    }
    id
}
fn translate_node(current: &mut GraphDocument, event: (String, f32, f32)) -> bool {
    let (id, dx, dy) = event;
    let Some(node) = current.node(&id) else {
        return false;
    };
    let edit = GraphEdit::MoveNode {
        id,
        x: node.rect.x + dx,
        y: node.rect.y + dy,
    };
    apply_edit(current, edit)
}
fn implicit_graph_style(context: &RenderContext<'_>, disabled: bool) -> GraphStyle {
    if let Some(theme) = crate::theme::registry::active_theme(context.app()) {
        let colors = theme.profile.colors(Some(theme.mode));
        let parse = |color: &str| {
            gpui_component::theme::try_parse_color(color)
                .expect("validated RustUI semantic graph color")
        };
        GraphStyle {
            node: parse(&colors.elevated_panel_background),
            text: parse(if disabled {
                &colors.disabled_text
            } else {
                &colors.primary_text
            }),
            edge: parse(&colors.border),
            selection: parse(&colors.accent),
        }
    } else {
        let colors = &gpui_component::Theme::global(context.app()).colors;
        GraphStyle {
            node: colors.background,
            text: if disabled {
                colors.muted_foreground
            } else {
                colors.foreground
            },
            edge: colors.border,
            selection: colors.primary,
        }
    }
}

fn surface_keyboard_event_eligible(
    event: &gpui::KeyDownEvent,
    disabled: bool,
    focused: bool,
    contains_focus: bool,
) -> bool {
    !disabled && eligible(event) && contains_focus && (focused || event.keystroke.key == "escape")
}

#[cfg(test)]
mod interaction_followup_tests {
    use super::*;
    fn key(name: &str) -> gpui::KeyDownEvent {
        gpui::KeyDownEvent {
            keystroke: gpui::Keystroke::parse(name).unwrap(),
            is_held: false,
            prefer_character_input: false,
        }
    }
    #[test]
    fn native_ids_are_unique_with_delimiters_and_unicode() {
        assert_ne!(
            graph_element_id("port", &["graph", "a:b", "c"]),
            graph_element_id("port", &["graph", "a", "b:c"])
        );
        assert_ne!(
            graph_element_id("node", &["a:b", "c"]),
            graph_element_id("node", &["a", "b:c"])
        );
        assert_ne!(
            graph_element_id("edge", &["g", "é"]),
            graph_element_id("node", &["g", "é"])
        );
        assert_eq!(
            graph_element_id("port", &["g", "é", "p"]),
            "rustui-graph-port 1:g 2:é 1:p"
        );
    }
    #[test]
    fn repeated_relative_moves_read_latest_owner_geometry() {
        struct Owner {
            graph: GraphDocument,
        }
        let node = GraphNode::new(
            "n",
            "Node",
            CanvasRect::new(10.0, 0.0, 100.0, 80.0).unwrap(),
            [],
        )
        .unwrap();
        let graph = GraphDocument::new([node], []).unwrap();
        let binding = Binding::new::<Owner>(
            graph.clone(),
            |owner| owner.graph.clone(),
            |owner, graph| owner.graph = graph,
        );
        let mut owner = Owner {
            graph: graph
                .edited(GraphEdit::MoveNode {
                    id: "n".into(),
                    x: 100.0,
                    y: 0.0,
                })
                .unwrap(),
        };
        for _ in 0..2 {
            assert!(binding.update_on(&mut owner, |latest| translate_node(
                latest,
                ("n".into(), 10.0, 0.0)
            )));
        }
        assert_eq!(owner.graph.node("n").unwrap().rect().x, 120.0);
        assert!(!binding.update_on(&mut owner, |latest| translate_node(
            latest,
            ("n".into(), f32::INFINITY, 0.0)
        )));
        assert_eq!(owner.graph.node("n").unwrap().rect().x, 120.0);
    }
    #[test]
    fn disabled_surface_and_descendant_focus_have_correct_key_routing() {
        for event in [key("escape"), key("left"), key("enter")] {
            assert!(!surface_keyboard_event_eligible(&event, true, true, true));
        }
        assert!(surface_keyboard_event_eligible(
            &key("escape"),
            false,
            false,
            true
        ));
        assert!(!surface_keyboard_event_eligible(
            &key("left"),
            false,
            false,
            true
        ));
        assert!(!surface_keyboard_event_eligible(
            &key("escape"),
            false,
            false,
            false
        ));
        assert!(surface_keyboard_event_eligible(
            &key("left"),
            false,
            true,
            true
        ));
    }
}

fn current_keyboard_selection(
    selection: Option<Option<String>>,
    document: Option<&GraphDocument>,
) -> Option<String> {
    selection
        .flatten()
        .filter(|id| document.is_some_and(|graph| graph.node(id).is_some()))
}

#[cfg(test)]
mod current_selection_regressions {
    use super::*;

    #[gpui::test]
    fn rendered_node_editor_connects_moves_and_deletes_nodes(cx: &mut gpui::TestAppContext) {
        use crate::{components::advanced::node_editor::NodeEditor, runtime::ComponentHost};

        struct EditorOwner {
            graph: GraphDocument,
            selected: Option<String>,
            viewport: GraphViewport,
        }
        impl View for EditorOwner {
            fn body(&self) -> impl View {
                NodeEditor(
                    "keyboard-editor",
                    Binding::new(
                        self.graph.clone(),
                        |o: &Self| o.graph.clone(),
                        |o, v| o.graph = v,
                    ),
                    Binding::new(
                        self.selected.clone(),
                        |o: &Self| o.selected.clone(),
                        |o, v| o.selected = v,
                    ),
                    Binding::new(self.viewport, |o: &Self| o.viewport, |o, v| o.viewport = v),
                )
            }
        }
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let rect = CanvasRect::new(0.0, 0.0, 100.0, 80.0).unwrap();
        let output = GraphPort::new("out", "Output", GraphPortDirection::Output).unwrap();
        let input = GraphPort::new("in", "Input", GraphPortDirection::Input).unwrap();
        let output2 = GraphPort::new("out2", "Output 2", GraphPortDirection::Output).unwrap();
        let input2 = GraphPort::new("in2", "Input 2", GraphPortDirection::Input).unwrap();
        let graph = GraphDocument::new(
            [
                GraphNode::new("a", "A", rect, [output, output2]).unwrap(),
                GraphNode::new(
                    "b",
                    "B",
                    CanvasRect::new(200.0, 0.0, 100.0, 80.0).unwrap(),
                    [input, input2],
                )
                .unwrap(),
            ],
            [],
        )
        .unwrap();
        let viewport =
            GraphViewport::new(CanvasSize::new(600.0, 400.0).unwrap(), 0.0, 0.0, 1.0).unwrap();
        let (owner, window) = cx.add_window_view(|_, app| {
            ComponentHost::new(
                EditorOwner {
                    graph,
                    selected: Some("a".into()),
                    viewport,
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        let first = window.debug_bounds("graph-test-port:a:out").unwrap();
        let second = window.debug_bounds("graph-test-port:b:in").unwrap();
        let third = window.debug_bounds("graph-test-port:a:out2").unwrap();
        let fourth = window.debug_bounds("graph-test-port:b:in2").unwrap();
        window.simulate_click(first.center(), gpui::Modifiers::default());
        window.simulate_click(third.center(), gpui::Modifiers::default());
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(owner.read(app).component().graph.edges().is_empty());
            let state = owner
                .read(app)
                .retained_entity_for_test::<Interaction>(
                    vec![],
                    Identity::from("graph-interaction:keyboard-editor"),
                )
                .unwrap();
            assert_eq!(state.read(app).error, Some(GraphError::PortDirection));
            assert!(state.read(app).pending.is_none());
        });
        window.update(|window, app| {
            for position in [
                first.center(),
                second.center(),
                third.center(),
                fourth.center(),
            ] {
                window.dispatch_event(
                    gpui::PlatformInput::MouseDown(gpui::MouseDownEvent {
                        position,
                        button: MouseButton::Left,
                        modifiers: gpui::Modifiers::default(),
                        click_count: 1,
                        first_mouse: false,
                    }),
                    app,
                );
                window.dispatch_event(
                    gpui::PlatformInput::MouseUp(gpui::MouseUpEvent {
                        position,
                        button: MouseButton::Left,
                        modifiers: gpui::Modifiers::default(),
                        click_count: 1,
                    }),
                    app,
                );
            }
            assert_eq!(owner.read(app).component().graph.edges().len(), 2);
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            let edges = owner.read(app).component().graph.edges();
            assert_eq!(edges.len(), 2);
            assert_ne!(edges[0].id(), edges[1].id());
            assert_eq!(edges[0].from(), &GraphEndpoint::new("a", "out"));
            assert_eq!(edges[0].to(), &GraphEndpoint::new("b", "in"));
            assert_eq!(edges[1].from(), &GraphEndpoint::new("a", "out2"));
            assert_eq!(edges[1].to(), &GraphEndpoint::new("b", "in2"));
            let state = owner
                .read(app)
                .retained_entity_for_test::<Interaction>(
                    vec![],
                    Identity::from("graph-interaction:keyboard-editor"),
                )
                .unwrap();
            assert!(state.read(app).error.is_none());
            assert!(state.read(app).pending.is_none());
        });
        let node = window.debug_bounds("graph-test-node:a").unwrap();
        let start = gpui::point(node.origin.x + px(10.0), node.origin.y + px(10.0));
        let end = gpui::point(start.x + px(30.0), start.y + px(20.0));
        window.simulate_mouse_down(start, MouseButton::Left, gpui::Modifiers::default());
        window.simulate_mouse_move(end, MouseButton::Left, gpui::Modifiers::default());
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        let end = gpui::point(end.x + px(10.0), end.y + px(5.0));
        window.simulate_mouse_move(end, MouseButton::Left, gpui::Modifiers::default());
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        window.simulate_mouse_up(end, MouseButton::Left, gpui::Modifiers::default());
        window.simulate_mouse_move(
            gpui::point(end.x + px(25.0), end.y + px(25.0)),
            None,
            gpui::Modifiers::default(),
        );
        window.update(|window, app| {
            let _ = window.draw(app);
            let rect = owner.read(app).component().graph.node("a").unwrap().rect();
            assert_eq!((rect.x, rect.y), (40.0, 25.0));
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            let state = owner
                .read(app)
                .retained_entity_for_test::<Interaction>(
                    vec![],
                    Identity::from("graph-interaction:keyboard-editor"),
                )
                .unwrap();
            let focus = state.read(app).focus.clone();
            focus.focus(window, app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(
                owner
                    .read(app)
                    .component()
                    .graph
                    .node("a")
                    .unwrap()
                    .rect()
                    .x,
                50.0
            );
        });
        window.simulate_keystrokes("backspace");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(owner.read(app).component().graph.node("a").is_none());
            assert!(owner.read(app).component().graph.node("b").is_some());
            assert!(owner.read(app).component().graph.edges().is_empty());
            assert_eq!(owner.read(app).component().selected, None);
        });
        window.update(|window, app| {
            window.dispatch_event(
                gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("pageup").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                }),
                app,
            );
            assert_eq!(owner.read(app).component().viewport.zoom(), 1.25);
            window.dispatch_event(
                gpui::PlatformInput::KeyUp(gpui::KeyUpEvent {
                    keystroke: gpui::Keystroke::parse("pageup").unwrap(),
                }),
                app,
            );
            window.dispatch_event(
                gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("home").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                }),
                app,
            );
            assert_eq!(owner.read(app).component().viewport.zoom(), 1.0);
            window.dispatch_event(
                gpui::PlatformInput::KeyUp(gpui::KeyUpEvent {
                    keystroke: gpui::Keystroke::parse("home").unwrap(),
                }),
                app,
            );
            owner.update(app, |host, cx| {
                let rect = CanvasRect::new(f32::MAX * 0.9, 0.0, f32::MAX * 0.01, 80.0).unwrap();
                let node = GraphNode::new("b", "B", rect, []).unwrap();
                let next = GraphDocument::new([node], []).unwrap();
                host.component_mut().graph = next;
                cx.notify();
            });
            window.dispatch_event(
                gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("pageup").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                }),
                app,
            );
            assert_eq!(owner.read(app).component().viewport.zoom(), 1.0);
            window.dispatch_event(
                gpui::PlatformInput::KeyDown(gpui::KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("pagedown").unwrap(),
                    is_held: false,
                    prefer_character_input: false,
                }),
                app,
            );
            assert_eq!(owner.read(app).component().viewport.zoom(), 0.8);
        });
    }
    struct Owner {
        document: GraphDocument,
        selected: Option<String>,
        activations: Vec<String>,
    }
    fn owner() -> Owner {
        let node = |id: &str| {
            GraphNode::new(id, id, CanvasRect::new(0.0, 0.0, 100.0, 80.0).unwrap(), []).unwrap()
        };
        Owner {
            document: GraphDocument::new([node("a"), node("b"), node("c")], []).unwrap(),
            selected: Some("a".into()),
            activations: Vec::new(),
        }
    }
    fn bindings(owner: &Owner) -> (Binding<GraphDocument>, Binding<Option<String>>) {
        (
            Binding::new::<Owner>(
                owner.document.clone(),
                |owner| owner.document.clone(),
                |owner, value| owner.document = value,
            ),
            Binding::new::<Owner>(
                owner.selected.clone(),
                |owner| owner.selected.clone(),
                |owner, value| owner.selected = value,
            ),
        )
    }
    #[test]
    fn select_then_move_delete_and_activate_use_current_bindings_before_redraw() {
        let mut owner = owner();
        let (document, selection) = bindings(&owner);
        // Deliberately reuse the original render's bindings for every event.
        let rendered_graph = owner.document.clone();
        assert!(selection.update_on(&mut owner, |value| advance_selection(
            &rendered_graph,
            value,
            true
        )));
        let graph = document.read_from(&owner);
        let id = current_keyboard_selection(selection.read_from(&owner), graph.as_ref()).unwrap();
        assert!(document.update_on(&mut owner, |graph| translate_node(graph, (id, 10.0, 0.0))));
        assert_eq!(owner.document.node("a").unwrap().rect().x, 0.0);
        assert_eq!(owner.document.node("b").unwrap().rect().x, 10.0);
        let graph = document.read_from(&owner);
        let id = current_keyboard_selection(selection.read_from(&owner), graph.as_ref()).unwrap();
        let activate = crate::runtime::Action::new(|owner: &mut Owner| {
            owner.activations.push(owner.selected.clone().unwrap());
        });
        assert!(activate.dispatch(&mut owner).should_notify());
        assert!(document.update_on(&mut owner, |graph| apply_edit(
            graph,
            GraphEdit::RemoveNode(id)
        )));
        assert!(owner.document.node("a").is_some());
        assert!(owner.document.node("b").is_none());
        assert_eq!(owner.activations, ["b"]);
        let graph = document.read_from(&owner);
        assert_eq!(
            current_keyboard_selection(selection.read_from(&owner), graph.as_ref()),
            None
        );
        owner.document = owner
            .document
            .edited(GraphEdit::RemoveNode("c".into()))
            .unwrap();
        let graph = document.read_from(&owner).unwrap();
        assert!(selection.update_on(&mut owner, |value| advance_selection(&graph, value, true)));
        assert_eq!(owner.selected.as_deref(), Some("a"));
    }
}

fn advance_selection(graph: &GraphDocument, current: &mut Option<String>, forward: bool) -> bool {
    let next = next_node(graph, current.as_deref(), forward);
    if *current == next {
        return false;
    }
    *current = next;
    true
}
