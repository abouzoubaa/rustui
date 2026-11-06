use super::model::localized_graph_message;
#[cfg(test)]
use super::rendering::{
    DISCONNECT_CONTROL_SIZE, disconnect_control_origin, incident_edges, projected_port_endpoint,
};
use super::validation::{identity, label};
use super::*;

pub struct GraphViewControl {
    pub(crate) id: Identity,
    pub(crate) graph: GraphDocument,
    pub(crate) selection: Binding<Option<String>>,
    pub(crate) viewport: Binding<GraphViewport>,
    pub(crate) style: Option<GraphStyle>,
    pub(crate) label: GraphAccessibilityLabel,
    pub(crate) activate: Option<ControlAction>,
    pub(crate) disabled: bool,
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

pub(super) struct Interaction {
    pub(super) focus: FocusHandle,
    pub(super) drag: Option<Drag>,
    pub(super) pending: Option<GraphEndpoint>,
    pub(super) error: Option<GraphError>,
    pub(super) viewport: GraphViewport,
}
#[derive(Clone)]
pub(super) struct Drag {
    pub(super) id: String,
    pub(super) anchor_pointer: gpui::Point<gpui::Pixels>,
    pub(super) previous_pointer: gpui::Point<gpui::Pixels>,
    pub(super) anchor_rect: CanvasRect,
    pub(super) last_target: CanvasRect,
}

pub(super) fn finite_pointer(pointer: gpui::Point<gpui::Pixels>) -> bool {
    f32::from(pointer.x).is_finite() && f32::from(pointer.y).is_finite()
}

pub(super) fn drag_for_node(
    graph: &GraphDocument,
    id: &str,
    pointer: gpui::Point<gpui::Pixels>,
) -> Option<Drag> {
    if !finite_pointer(pointer) {
        return None;
    }
    let rect = graph.node(id)?.rect;
    Some(Drag {
        id: id.to_owned(),
        anchor_pointer: pointer,
        previous_pointer: pointer,
        anchor_rect: rect,
        last_target: rect,
    })
}

pub(super) fn drag_move(
    drag: &mut Drag,
    graph: &GraphDocument,
    pointer: gpui::Point<gpui::Pixels>,
    zoom: f32,
) -> Option<GraphEdit> {
    if !finite_pointer(pointer) {
        return None;
    }
    let current = graph.node(&drag.id)?.rect;
    let (anchor_rect, anchor_pointer) = if current != drag.last_target {
        (current, drag.previous_pointer)
    } else {
        (drag.anchor_rect, drag.anchor_pointer)
    };
    let dx = f32::from(pointer.x - anchor_pointer.x) / zoom;
    let dy = f32::from(pointer.y - anchor_pointer.y) / zoom;
    let next = CanvasRect::new(
        anchor_rect.x + dx,
        anchor_rect.y + dy,
        current.width,
        current.height,
    )
    .ok()?;
    drag.anchor_rect = anchor_rect;
    drag.anchor_pointer = anchor_pointer;
    drag.previous_pointer = pointer;
    drag.last_target = next;
    Some(GraphEdit::MoveNode {
        id: drag.id.clone(),
        x: next.x,
        y: next.y,
    })
}
#[derive(Clone)]
pub(super) struct Lease {
    pub(super) epoch: Weak<Cell<u64>>,
    pub(super) generation: u64,
}
impl Lease {
    pub(super) fn current(&self) -> bool {
        self.epoch
            .upgrade()
            .is_some_and(|epoch| epoch.get() == self.generation)
    }
}

pub(super) fn connection(
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
    graph.validate_new_edge(&edge)?;
    Ok(edge)
}

pub(super) fn next_connection_id(edges: &[GraphEdge]) -> String {
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

pub(super) fn eligible(event: &gpui::KeyDownEvent) -> bool {
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

pub(super) fn apply_edit(current: &mut GraphDocument, edit: GraphEdit) -> bool {
    if let GraphEdit::MoveNode { id, x, y } = &edit {
        let Some(&index) = current.node_index.get(id) else {
            return false;
        };
        let node = &mut current.nodes[index];
        let Ok(rect) = CanvasRect::new(*x, *y, node.rect.width, node.rect.height) else {
            return false;
        };
        if rect == node.rect {
            return false;
        }
        node.rect = rect;
        return true;
    }
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
    use super::super::model::resolved_localized_message;
    use super::*;

    #[test]
    fn in_place_node_moves_match_atomic_document_edits() {
        for edit in [
            GraphEdit::MoveNode {
                id: "a".into(),
                x: 20.0,
                y: 30.0,
            },
            GraphEdit::MoveNode {
                id: "a".into(),
                x: 0.0,
                y: 0.0,
            },
            GraphEdit::MoveNode {
                id: "missing".into(),
                x: 20.0,
                y: 30.0,
            },
            GraphEdit::MoveNode {
                id: "a".into(),
                x: f32::NAN,
                y: 0.0,
            },
            GraphEdit::MoveNode {
                id: "a".into(),
                x: f32::MAX,
                y: f32::MAX,
            },
        ] {
            let original = graph();
            let expected = original.edited(edit.clone());
            let mut actual = original.clone();
            let changed = apply_edit(&mut actual, edit);
            match expected {
                Ok(expected) => {
                    assert_eq!(changed, expected != original);
                    assert_eq!(actual, expected);
                }
                Err(_) => {
                    assert!(!changed);
                    assert_eq!(actual, original);
                }
            }
        }
    }

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
            let mut edges = graph.edges.clone();
            edges.push(edge.clone());
            GraphDocument::new(graph.nodes.clone(), edges)?;
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

    #[test]
    fn connect_edit_matches_full_reconstruction() {
        let populated = graph().edited(GraphEdit::Connect(edge())).unwrap();
        let endpoints = [
            GraphEndpoint::new("a", "out"),
            GraphEndpoint::new("a", "in"),
            GraphEndpoint::new("b", "out"),
            GraphEndpoint::new("b", "in"),
            GraphEndpoint::new("missing", "out"),
            GraphEndpoint::new("a", "missing"),
        ];
        let existing = edge();
        for document in [graph(), populated] {
            for id in ["new-edge", existing.id()] {
                for from in &endpoints {
                    for to in &endpoints {
                        let candidate = GraphEdge::new(id, from.clone(), to.clone()).unwrap();
                        let mut edges = document.edges.clone();
                        edges.push(candidate.clone());
                        assert_eq!(
                            document.edited(GraphEdit::Connect(candidate)),
                            GraphDocument::new(document.nodes.clone(), edges),
                            "id={id}, from={from:?}, to={to:?}"
                        );
                    }
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
    fn drag_starts_only_for_current_owner_nodes() {
        let rendered = graph();
        let current = rendered
            .edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: 90.0,
                y: 35.0,
            })
            .unwrap();
        let pointer = gpui::point(gpui::px(20.0), gpui::px(30.0));
        let drag = drag_for_node(&current, "a", pointer).unwrap();
        assert_eq!(drag.id, "a");
        assert_eq!(drag.anchor_pointer, pointer);
        let removed = current.edited(GraphEdit::RemoveNode("a".into())).unwrap();
        assert!(drag_for_node(&removed, "a", pointer).is_none());
    }

    #[test]
    fn drag_accumulates_subpixel_world_moves_and_rebases_on_owner_change() {
        let mut graph = graph()
            .edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: 16_777_216.0,
                y: 10.0,
            })
            .unwrap();
        let point = |x| gpui::point(gpui::px(x), gpui::px(0.0));
        let mut drag = drag_for_node(&graph, "a", point(0.0)).unwrap();
        for x in 1..=4 {
            let edit = drag_move(&mut drag, &graph, point(x as f32), 1.0).unwrap();
            apply_edit(&mut graph, edit);
        }
        assert_eq!(graph.node("a").unwrap().rect().x, 16_777_220.0);
        graph = graph
            .edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: 20_000_000.0,
                y: 10.0,
            })
            .unwrap();
        for x in 5..=8 {
            let edit = drag_move(&mut drag, &graph, point(x as f32), 1.0).unwrap();
            apply_edit(&mut graph, edit);
        }
        assert_eq!(graph.node("a").unwrap().rect().x, 20_000_004.0);
        // An owner that rejects an attempted move must not receive it again.
        let _rejected = drag_move(&mut drag, &graph, point(10.0), 1.0).unwrap();
        let accepted = drag_move(&mut drag, &graph, point(12.0), 1.0).unwrap();
        apply_edit(&mut graph, accepted);
        assert_eq!(graph.node("a").unwrap().rect().x, 20_000_006.0);
    }

    #[test]
    fn nonfinite_pointer_does_not_poison_a_later_owner_rebase() {
        let mut graph = graph();
        let point = |x| gpui::point(gpui::px(x), gpui::px(0.0));
        assert!(drag_for_node(&graph, "a", point(f32::INFINITY)).is_none());
        assert!(
            drag_for_node(&graph, "a", gpui::point(gpui::px(0.0), gpui::px(f32::NAN)),).is_none()
        );
        let mut drag = drag_for_node(&graph, "a", point(0.0)).unwrap();
        assert!(drag_move(&mut drag, &graph, point(f32::NAN), 1.0).is_none());
        graph = graph
            .edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: 50.0,
                y: 10.0,
            })
            .unwrap();
        let edit = drag_move(&mut drag, &graph, point(1.0), 1.0).unwrap();
        apply_edit(&mut graph, edit);
        assert_eq!(graph.node("a").unwrap().rect().x, 51.0);
    }

    #[test]
    fn overflowed_drag_does_not_poison_a_later_owner_rebase() {
        let mut graph = graph();
        let point = |x| gpui::point(gpui::px(x), gpui::px(0.0));
        let mut drag = drag_for_node(&graph, "a", point(0.0)).unwrap();
        assert!(drag_move(&mut drag, &graph, point(10.0), f32::MIN_POSITIVE).is_none());
        graph = graph
            .edited(GraphEdit::MoveNode {
                id: "a".into(),
                x: 50.0,
                y: 10.0,
            })
            .unwrap();
        let edit = drag_move(&mut drag, &graph, point(1.0), 1.0).unwrap();
        apply_edit(&mut graph, edit);
        assert_eq!(graph.node("a").unwrap().rect().x, 51.0);
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
            include_str!("../../../../../../locales/fr/rustui.ftl"),
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
            include_str!("../../../../../../locales/en/rustui.ftl"),
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
pub(super) enum ViewportCommand {
    Pan(f32, f32),
    Scale(f32),
    Reset,
}
pub(super) fn viewport_command(
    viewport: GraphViewport,
    command: ViewportCommand,
) -> Result<GraphViewport, GraphError> {
    match command {
        ViewportCommand::Pan(x, y) => viewport.pan(x / viewport.zoom, y / viewport.zoom),
        ViewportCommand::Scale(factor) => viewport.with_zoom(viewport.zoom * factor),
        ViewportCommand::Reset => GraphViewport::new(viewport.size, 0.0, 0.0, 1.0),
    }
}

pub(super) fn projection_error(
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

pub(super) fn graph_element_id(kind: &str, parts: &[&str]) -> String {
    let mut id = format!("rustui-graph-{kind}");
    for part in parts {
        use std::fmt::Write as _;
        write!(&mut id, " {}:{}", part.len(), part).unwrap();
    }
    id
}
pub(super) fn translate_node(current: &mut GraphDocument, event: (String, f32, f32)) -> bool {
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

pub(super) fn surface_keyboard_event_eligible(
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

pub(super) fn current_keyboard_selection(
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
        // Change owner geometry after rendering but before the mouse-down reaches
        // the existing node handler. It must not start from the rendered snapshot.
        let original = window.update(|_, app| {
            let original = owner.read(app).component().graph.clone();
            let moved = original
                .edited(GraphEdit::MoveNode {
                    id: "a".into(),
                    x: 150.0,
                    y: 60.0,
                })
                .unwrap();
            owner.update(app, |host, _| host.component_mut().graph = moved);
            original
        });
        window.simulate_mouse_down(start, MouseButton::Left, gpui::Modifiers::default());
        let first_move = gpui::point(start.x + px(30.0), start.y + px(20.0));
        window.simulate_mouse_move(first_move, MouseButton::Left, gpui::Modifiers::default());
        window.update(|window, app| {
            let rect = owner.read(app).component().graph.node("a").unwrap().rect();
            assert_eq!((rect.x, rect.y), (180.0, 80.0));
            let updated = owner
                .read(app)
                .component()
                .graph
                .edited(GraphEdit::MoveNode {
                    id: "a".into(),
                    x: 300.0,
                    y: 100.0,
                })
                .unwrap();
            owner.update(app, |host, _| host.component_mut().graph = updated);
            let _ = window.draw(app);
        });
        let second_move = gpui::point(first_move.x + px(10.0), first_move.y + px(5.0));
        window.simulate_mouse_move(second_move, MouseButton::Left, gpui::Modifiers::default());
        window.update(|_, app| {
            let rect = owner.read(app).component().graph.node("a").unwrap().rect();
            assert_eq!((rect.x, rect.y), (310.0, 105.0));
        });
        window.simulate_mouse_up(second_move, MouseButton::Left, gpui::Modifiers::default());
        window.update(|window, app| {
            owner.update(app, |host, _| host.component_mut().graph = original);
            let _ = window.draw(app);
        });
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
        // Release beyond the graph after a real in-surface move. A later held
        // pointer move must not resume the old drag.
        let node = window.debug_bounds("graph-test-node:a").unwrap();
        let drag_start = gpui::point(node.origin.x + px(10.0), node.origin.y + px(10.0));
        let drag_inside = gpui::point(drag_start.x + px(10.0), drag_start.y + px(5.0));
        let drag_outside = gpui::point(drag_start.x + px(1_000.0), drag_start.y);
        window.simulate_mouse_down(drag_start, MouseButton::Left, gpui::Modifiers::default());
        window.simulate_mouse_move(drag_inside, MouseButton::Left, gpui::Modifiers::default());
        window.update(|window, app| {
            let _ = window.draw(app);
            let rect = owner.read(app).component().graph.node("a").unwrap().rect();
            assert_eq!((rect.x, rect.y), (50.0, 30.0));
            let state = owner
                .read(app)
                .retained_entity_for_test::<Interaction>(
                    vec![],
                    Identity::from("graph-interaction:keyboard-editor"),
                )
                .unwrap();
            assert!(state.read(app).drag.is_some());
        });
        window.simulate_mouse_up(drag_outside, MouseButton::Left, gpui::Modifiers::default());
        window.update(|_, app| {
            let state = owner
                .read(app)
                .retained_entity_for_test::<Interaction>(
                    vec![],
                    Identity::from("graph-interaction:keyboard-editor"),
                )
                .unwrap();
            assert!(state.read(app).drag.is_none());
        });
        window.simulate_mouse_move(drag_inside, MouseButton::Left, gpui::Modifiers::default());
        window.update(|window, app| {
            let _ = window.draw(app);
            let state = owner
                .read(app)
                .retained_entity_for_test::<Interaction>(
                    vec![],
                    Identity::from("graph-interaction:keyboard-editor"),
                )
                .unwrap();
            assert!(state.read(app).drag.is_none());
            let graph = &owner.read(app).component().graph;
            let rect = graph.node("a").unwrap().rect();
            assert_eq!((rect.x, rect.y), (50.0, 30.0));
            let restored = graph
                .edited(GraphEdit::MoveNode {
                    id: "a".into(),
                    x: 40.0,
                    y: 25.0,
                })
                .unwrap();
            owner.update(app, |host, cx| {
                host.component_mut().graph = restored;
                cx.notify();
            });
            let _ = window.draw(app);
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

pub(super) fn advance_selection(
    graph: &GraphDocument,
    current: &mut Option<String>,
    forward: bool,
) -> bool {
    let next = next_node(graph, current.as_deref(), forward);
    if *current == next {
        return false;
    }
    *current = next;
    true
}
