use super::interaction::*;
use super::model::localized_graph_message;
use super::*;

fn graph_display_label(context: &RenderContext<'_>, key: Option<&str>, fallback: &str) -> String {
    match key {
        Some(key) => localized_graph_message(context, key, &MessageArguments::new(), fallback),
        None => fallback.to_owned(),
    }
}

fn localized_graph_error(context: &RenderContext<'_>, error: &GraphError) -> String {
    let id = error.localization_key();
    localized_graph_message(context, id, &MessageArguments::new(), &error.to_string())
}

pub(super) const DISCONNECT_CONTROL_SIZE: f32 = 24.0;

pub(super) fn incident_edges(graph: &GraphDocument) -> HashMap<&str, Vec<&GraphEdge>> {
    let mut incident: HashMap<&str, Vec<&GraphEdge>> = HashMap::new();
    for edge in &graph.edges {
        incident.entry(&edge.from.node).or_default().push(edge);
        if edge.from.node != edge.to.node {
            incident.entry(&edge.to.node).or_default().push(edge);
        }
    }
    incident
}

pub(super) fn projected_port_endpoint(
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

pub(super) fn disconnect_control_origin(
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
    pub(super) fn validate(self) -> Result<Self, GraphError> {
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
pub(crate) enum GraphAccessibilityLabel {
    Literal(String),
    Message { id: String, fallback: String },
}

impl GraphAccessibilityLabel {
    pub(super) fn fallback(&self) -> &str {
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

impl View for GraphViewControl {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        render_graph(self, None, context)
    }
}

pub(crate) fn render_graph(
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
            crate::components::input::leave_disabled_focus(true, &state.focus, window, cx);
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
            let guard = lease.clone();
            let drag_focus = focus.clone();
            let writer = context.binding_writer(view.selection.clone());
            let read_graph = context.binding_reader(editor.as_ref().unwrap().clone());
            card = card.on_mouse_down(MouseButton::Left, move |event, window, cx| {
                if !guard.current() {
                    return;
                }
                let Some(drag) =
                    read_graph(cx).and_then(|graph| drag_for_node(&graph, &id, event.position))
                else {
                    return;
                };
                drag_focus.focus(window, cx);
                writer(Some(id.clone()), cx);
                drag_state.update(cx, |s, _| s.drag = Some(drag));
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
        let read_graph = context.binding_reader(binding.clone());
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
            if drag_state.read(cx).drag.is_none() {
                return;
            }
            let Some(graph) = read_graph(cx) else {
                return;
            };
            let edit = drag_state.update(cx, |state, _| {
                drag_move(state.drag.as_mut()?, &graph, event.position, viewport.zoom)
            });
            if let Some(edit) = edit {
                update(edit, cx);
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
