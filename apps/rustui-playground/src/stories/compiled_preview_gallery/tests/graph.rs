use super::*;
#[test]
fn graph_factories_mount_and_render_through_the_real_preview_registry() {
    let mut registry = PreviewRegistry::<()>::new();
    super::super::register_graph_specimens(&mut registry);
    let mut host = PreviewHost::new(registry);
    for id in ["graph-view-specimen", "node-editor-specimen"] {
        host.mount(PreviewDocument::new(
            PreviewId::new("graph-preview-test").unwrap(),
            ComponentId::new(id).unwrap(),
            1,
        ))
        .unwrap();
        assert!(host.render(&()).unwrap().is_some());
    }
}
