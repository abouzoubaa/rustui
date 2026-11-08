use super::*;

pub(in super::super) fn register_graph_specimens<Context: 'static>(
    registry: &mut PreviewRegistry<Context>,
) {
    for (id, name, editor) in [
        ("graph-view-specimen", "GraphView", false),
        ("node-editor-specimen", "NodeEditor", true),
    ] {
        registry
            .register(
                ComponentSchema {
                    id: ComponentId::new(id).unwrap(),
                    version: 1,
                    display_name: name.into(),
                    properties: BTreeMap::new(),
                    events: BTreeMap::new(),
                },
                move |_, _context: &Context| {
                    if editor {
                        AnyView::new(super::super::super::graph_gallery::editor_demo())
                    } else {
                        AnyView::new(super::super::super::graph_gallery::graph_demo())
                    }
                },
            )
            .unwrap();
    }
}
