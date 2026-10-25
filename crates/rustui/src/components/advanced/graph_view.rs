//! Positioned, controlled graph visualization using RustUI Canvas and GPUI input.

use super::canvas::{Canvas, CanvasPoint, CanvasRect, CanvasScene, CanvasSize};
use crate::view::{ControlAction, EmptyView, RenderContext, View};
use gpui::{
    AnyElement, FocusHandle, Hsla, InteractiveElement as _, IntoElement as _, MouseButton,
    ParentElement as _, Role, StatefulInteractiveElement as _, Styled as _, div, px,
};
use rustui_core::{Binding, Identity};
use std::{
    cell::Cell,
    collections::HashSet,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphPortDirection {
    Input,
    Output,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPort {
    id: String,
    label: String,
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
    pub fn direction(&self) -> GraphPortDirection {
        self.direction
    }
}
/// Caller-supplied world-space geometry. No automatic layout or domain policy is imposed.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    id: String,
    label: String,
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
}
impl GraphDocument {
    pub fn new(
        nodes: impl IntoIterator<Item = GraphNode>,
        edges: impl IntoIterator<Item = GraphEdge>,
    ) -> Result<Self, GraphError> {
        let graph = Self {
            nodes: nodes.into_iter().collect(),
            edges: edges.into_iter().collect(),
        };
        graph.validate()?;
        Ok(graph)
    }
    pub fn nodes(&self) -> &[GraphNode] {
        &self.nodes
    }
    pub fn edges(&self) -> &[GraphEdge] {
        &self.edges
    }
    pub fn node(&self, id: &str) -> Option<&GraphNode> {
        self.nodes.iter().find(|node| node.id == id)
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
        let mut ids = HashSet::new();
        for node in &self.nodes {
            node.validate()?;
            if !ids.insert(&node.id) {
                return Err(GraphError::DuplicateNode);
            }
        }
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
            if self.endpoint(&edge.from)?.1.direction != GraphPortDirection::Output
                || self.endpoint(&edge.to)?.1.direction != GraphPortDirection::Input
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
                next.nodes
                    .iter_mut()
                    .find(|n| n.id == id)
                    .ok_or(GraphError::MissingNode)?
                    .label = name;
            }
            GraphEdit::MoveNode { id, x, y } => {
                let n = next
                    .nodes
                    .iter_mut()
                    .find(|n| n.id == id)
                    .ok_or(GraphError::MissingNode)?;
                n.rect = CanvasRect::new(x, y, n.rect.width, n.rect.height)
                    .map_err(|_| GraphError::InvalidGeometry)?;
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

pub struct GraphViewControl {
    pub(super) id: Identity,
    pub(super) graph: GraphDocument,
    pub(super) selection: Binding<Option<String>>,
    pub(super) viewport: Binding<GraphViewport>,
    pub(super) style: Option<GraphStyle>,
    pub(super) label: String,
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
            label: "Graph".into(),
            activate: None,
            disabled: false,
        }
    }
    pub fn style(mut self, style: GraphStyle) -> Result<Self, GraphError> {
        self.style = Some(style.validate()?);
        Ok(self)
    }
    pub fn accessibility_label(mut self, name: impl Into<String>) -> Self {
        self.label = name.into();
        label(&self.label).expect("GraphView requires an accessible name");
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
    let mut ordinal = 0_u64;
    let id = loop {
        let id = format!("connection-{ordinal}");
        if !graph.edges.iter().any(|edge| edge.id == id) {
            break id;
        }
        ordinal += 1;
    };
    let edge = GraphEdge::new(id, from, to)?;
    graph.edited(GraphEdit::Connect(edge.clone()))?;
    Ok(edge)
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
        let endpoint = |endpoint: &GraphEndpoint| -> Result<CanvasPoint, GraphError> {
            let (node, port) = graph.endpoint(endpoint)?;
            let rect = viewport.rect(node.rect)?;
            let index = node.ports.iter().position(|p| p.id == port.id).unwrap();
            CanvasPoint::new(
                if port.direction == GraphPortDirection::Input {
                    rect.x
                } else {
                    rect.x + rect.width
                },
                rect.y + rect.height * ((index as f32 + 1.5) / (node.ports.len() + 1) as f32),
            )
            .map_err(|_| GraphError::InvalidGeometry)
        };
        let drawn = endpoint(&edge.from).and_then(|from| {
            endpoint(&edge.to).and_then(|to| {
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
        .aria_label(view.label.clone())
        .aria_description(if editor.is_some() {"Arrows move selected node; brackets select; Delete removes. A D W S pan; PageUp and PageDown zoom; Home resets; Escape cancels. Tab reaches ports; select output and input to connect."} else {"Arrows select nodes; Enter activates. A D W S pan; PageUp and PageDown zoom; Home resets; Escape clears selection."})
        .child(canvas);
    if !view.disabled {
        root = root
            .track_focus(&focus)
            .focus_visible(move |style_ref| style_ref.border_color(style.selection));
    }
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
        let relationships = graph
            .edges
            .iter()
            .filter(|edge| edge.from.node == node.id || edge.to.node == node.id)
            .map(|edge| {
                let from = graph.endpoint(&edge.from).unwrap();
                let to = graph.endpoint(&edge.to).unwrap();
                format!(
                    "{} {} to {} {}",
                    from.0.label, from.1.label, to.0.label, to.1.label
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        let node_label = if relationships.is_empty() {
            node.label.clone()
        } else {
            format!("{}; {}", node.label, relationships)
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
                    .child(node.label.clone()),
            );
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
                let graph = graph.clone();
                let binding = editor.as_ref().unwrap().clone();
                let update = context.binding_event_updater(binding, apply_edit);
                let refresh =
                    context.binding_event_updater(view.selection.clone(), |_, (): ()| true);
                let handler: Rc<dyn Fn(&mut gpui::Window, &mut gpui::App)> =
                    Rc::new(move |_, cx| {
                        if !guard.current() {
                            return;
                        }
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
                let port_label = format!(
                    "{} {} {}",
                    node.label,
                    if port.direction == GraphPortDirection::Input {
                        "input"
                    } else {
                        "output"
                    },
                    port.label
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
                        port.label
                    ));
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
                    port.label
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
        let description = format!(
            "{} {} to {} {}",
            source.0.label, source.1.label, target.0.label, target.1.label
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
            let from = viewport.rect(source.0.rect).unwrap();
            let to = viewport.rect(target.0.rect).unwrap();
            root = root.child(
                div()
                    .id(graph_element_id(
                        "edge",
                        &[view.id.as_str(), edge.id.as_str()],
                    ))
                    .absolute()
                    .left(px(((f64::from(from.x) + f64::from(to.x)) / 2.0) as f32))
                    .top(px(((f64::from(from.y) + f64::from(to.y)) / 2.0) as f32))
                    .w(px(24.0))
                    .h(px(24.0))
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
                    .aria_label(format!("Disconnect {description}"))
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
                .child(error.to_string()),
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
    fn edge() -> GraphEdge {
        GraphEdge::new(
            "ab",
            GraphEndpoint::new("a", "out"),
            GraphEndpoint::new("b", "in"),
        )
        .unwrap()
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
    let mut reset = div()
        .id(graph_element_id("graph", &[view.id.as_str()]))
        .role(Role::Button)
        .aria_disabled(disabled)
        .aria_label("Graph view is out of range. Reset view")
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
        .child("View is out of range. Reset view");
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
