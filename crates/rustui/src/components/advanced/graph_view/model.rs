use super::validation::{identity, label};
use super::*;

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

pub(super) fn resolved_localized_message(id: &str, localized: String, fallback: &str) -> String {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.to_owned()
    } else {
        localized
    }
}

pub(super) fn localized_graph_message(
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphPortDirection {
    Input,
    Output,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPort {
    pub(super) id: String,
    pub(super) label: String,
    pub(super) label_key: Option<String>,
    pub(super) direction: GraphPortDirection,
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
    pub(super) id: String,
    pub(super) label: String,
    pub(super) label_key: Option<String>,
    pub(super) rect: CanvasRect,
    pub(super) ports: Vec<GraphPort>,
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
    pub(super) id: String,
    pub(super) from: GraphEndpoint,
    pub(super) to: GraphEndpoint,
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
    pub(super) nodes: Vec<GraphNode>,
    pub(super) edges: Vec<GraphEdge>,
    pub(super) node_index: HashMap<String, usize>,
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
    pub(super) fn endpoint(
        &self,
        endpoint: &GraphEndpoint,
    ) -> Result<(&GraphNode, &GraphPort), GraphError> {
        let node = self.node(&endpoint.node).ok_or(GraphError::MissingNode)?;
        Ok((
            node,
            node.ports
                .iter()
                .find(|port| port.id == endpoint.port)
                .ok_or(GraphError::MissingPort)?,
        ))
    }

    /// Applies one atomic edit to a clone. Failure leaves the original untouched.
    pub fn edited(&self, edit: GraphEdit) -> Result<Self, GraphError> {
        if let GraphEdit::Connect(edge) = &edit {
            self.validate_new_edge(edge)?;
        }
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
            GraphEdit::Connect(edge) => {
                next.edges.push(edge);
                return Ok(next);
            }
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
    pub(super) size: CanvasSize,
    pub(super) x: f32,
    pub(super) y: f32,
    pub(super) zoom: f32,
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
    pub(super) fn rect(self, rect: CanvasRect) -> Result<CanvasRect, GraphError> {
        CanvasRect::new(
            ((f64::from(rect.x) - f64::from(self.x)) * f64::from(self.zoom)) as f32,
            ((f64::from(rect.y) - f64::from(self.y)) * f64::from(self.zoom)) as f32,
            (f64::from(rect.width) * f64::from(self.zoom)) as f32,
            (f64::from(rect.height) * f64::from(self.zoom)) as f32,
        )
        .map_err(|_| GraphError::InvalidGeometry)
    }
}
