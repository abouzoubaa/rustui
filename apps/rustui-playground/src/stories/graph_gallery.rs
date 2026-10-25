//! Positioned graph visualization and controlled topology editing through public RustUI.
use rustui::{component, prelude::*};

pub fn sample_document() -> GraphDocument {
    let node = |id: &str, name: &str, x: f32, y: f32| {
        GraphNode::new(
            id,
            name,
            CanvasRect::new(x, y, 150.0, 100.0).unwrap(),
            [
                GraphPort::new("in", "Input", GraphPortDirection::Input).unwrap(),
                GraphPort::new("out", "Output", GraphPortDirection::Output).unwrap(),
            ],
        )
        .unwrap()
    };
    GraphDocument::new(
        [
            node("source", "Source", 20.0, 30.0),
            node("transform", "Transform", 250.0, 130.0),
        ],
        [GraphEdge::new(
            "source-transform",
            GraphEndpoint::new("source", "out"),
            GraphEndpoint::new("transform", "in"),
        )
        .unwrap()],
    )
    .unwrap()
}
fn viewport() -> GraphViewport {
    GraphViewport::new(CanvasSize::new(520.0, 320.0).unwrap(), 0.0, 0.0, 1.0).unwrap()
}
pub fn graph_demo() -> impl View {
    Component("graph-gallery", || GraphGallery {
        editable: false,
        ..GraphGallery::default()
    })
}
pub fn editor_demo() -> impl View {
    Component("node-editor-gallery", GraphGallery::default)
}
pub fn demo() -> impl View {
    VStack::new()
        .spacing(12.0)
        .child(graph_demo())
        .child(editor_demo())
}

#[component]
struct GraphGallery {
    #[state(sample_document())]
    document: GraphDocument,
    #[state(None)]
    selection: Option<String>,
    #[state(viewport())]
    viewport: GraphViewport,
    #[state(true)]
    editable: bool,
    #[state(false)]
    disabled: bool,
    #[state(true)]
    mounted: bool,
    #[state(0_u64)]
    next_id: u64,
    #[state(0_u64)]
    activations: u64,
    #[state(String::new())]
    status: String,
}
impl GraphGallery {
    fn edit(&mut self, edit: GraphEdit) {
        match self.document.edited(edit) {
            Ok(next) => {
                self.document = next;
                self.status.clear();
            }
            Err(error) => self.status = error.to_string(),
        }
    }
    fn add(&mut self) {
        let id = format!("node-{}", self.next_id);
        self.next_id += 1;
        self.edit(GraphEdit::AddNode(
            GraphNode::new(
                id,
                "New node",
                CanvasRect::new(180.0, 30.0, 150.0, 100.0).unwrap(),
                [
                    GraphPort::new("in", "Input", GraphPortDirection::Input).unwrap(),
                    GraphPort::new("out", "Output", GraphPortDirection::Output).unwrap(),
                ],
            )
            .unwrap(),
        ));
    }
    fn rename(&mut self) {
        if let Some(id) = self.selection.clone() {
            self.edit(GraphEdit::RenameNode {
                id,
                label: "Renamed node".into(),
            });
        }
    }
    fn remove(&mut self) {
        if let Some(id) = self.selection.take() {
            self.edit(GraphEdit::RemoveNode(id));
        }
    }
    fn reset(&mut self) {
        self.document = sample_document();
        self.selection = None;
        self.viewport = viewport();
        self.status.clear();
    }
    fn toggle_disabled(&mut self) {
        self.disabled = !self.disabled;
    }
    fn toggle_mount(&mut self) {
        self.mounted = !self.mounted;
    }
    fn activate(&mut self) {
        self.activations += 1;
    }
}
impl View for GraphGallery {
    fn body(&self) -> impl View {
        let surface = if !self.mounted() {
            AnyView::new(Text("Graph omitted"))
        } else if self.editable() {
            AnyView::new(
                NodeEditor(
                    "surface",
                    self.document_binding(),
                    self.selection_binding(),
                    self.viewport_binding(),
                )
                .disabled(self.disabled())
                .on_activate(self.action(Self::activate))
                .accessibility_label("Example node editor"),
            )
        } else {
            AnyView::new(
                GraphView(
                    "surface",
                    self.document().clone(),
                    self.selection_binding(),
                    self.viewport_binding(),
                )
                .disabled(self.disabled())
                .on_activate(self.action(Self::activate))
                .accessibility_label("Example dependency graph"),
            )
        };
        VStack::new().spacing(8.0)
            .child(Text(if self.editable() {"NodeEditor"} else {"GraphView"}))
            .child(Text(format!("Selected: {} · Activations: {}",self.selection().as_deref().unwrap_or("none"),self.activations())))
            .child(HStack::new().spacing(8.0).child(Button("Reset",self.action(Self::reset))).child(Button("Toggle disabled",self.action(Self::toggle_disabled))).child(Button("Omit / remount",self.action(Self::toggle_mount))))
            .child(if self.editable() {AnyView::new(HStack::new().spacing(8.0).child(Button("Add node",self.action(Self::add))).child(Button("Rename selected",self.action(Self::rename)).disabled(self.selection().is_none())).child(Button("Remove selected",self.action(Self::remove)).disabled(self.selection().is_none())))} else {AnyView::new(Text("Arrow keys select nodes. Enter activates the selection."))})
            .child(Text(if self.editable() {"Drag nodes or use arrows to move. Brackets select. Delete removes. Select two ports to connect; × disconnects. Tab reaches nodes, ports and connections."} else {"Tab reaches graph nodes. Enter or Space selects and focuses the graph."}))
            .child(Text("A/D/W/S pan · PageUp/PageDown zoom · Home resets view · Escape cancels"))
            .child(surface).child(Text(self.status().clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn gallery_uses_real_topology_and_owner_edits() {
        let mut gallery = GraphGallery::default();
        assert_eq!(gallery.document.nodes().len(), 2);
        assert_eq!(gallery.document.edges().len(), 1);
        gallery.add();
        assert_eq!(gallery.document.nodes().len(), 3);
        gallery.selection = Some("node-0".into());
        gallery.rename();
        assert_eq!(
            gallery.document.node("node-0").unwrap().label(),
            "Renamed node"
        );
        gallery.remove();
        assert_eq!(gallery.document.nodes().len(), 2);
        gallery.selection = Some("source".into());
        gallery.remove();
        assert!(gallery.document.edges().is_empty());
        gallery.reset();
        assert_eq!(gallery.document, sample_document());
    }
}
