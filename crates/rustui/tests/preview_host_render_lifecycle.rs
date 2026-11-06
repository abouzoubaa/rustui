use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use rustui::{
    preview::{
        ComponentId, ComponentSchema, PreviewDocument, PreviewHost, PreviewId, PreviewRegistry,
    },
    view::{AnyView, EmptyView},
};

#[test]
fn teardown_clears_new_factory_but_captured_factory_stays_snapshot_scoped() {
    let rendered_generations = Rc::new(RefCell::new(Vec::new()));
    let mut registry = PreviewRegistry::new();
    let generations = Rc::clone(&rendered_generations);
    registry
        .register(
            ComponentSchema {
                id: ComponentId::new("text").unwrap(),
                version: 1,
                display_name: "Text".into(),
                properties: BTreeMap::new(),
                events: BTreeMap::new(),
            },
            move |snapshot, _| {
                generations.borrow_mut().push(snapshot.stamp.generation);
                AnyView::new(EmptyView)
            },
        )
        .unwrap();

    let mut host = PreviewHost::new(registry);
    let document = || {
        PreviewDocument::new(
            PreviewId::new("specimen").unwrap(),
            ComponentId::new("text").unwrap(),
            1,
        )
    };
    let first = host.mount(document()).unwrap();
    let captured_factory = host.render_factory();

    assert!(captured_factory(&()).unwrap().is_some());
    assert_eq!(*rendered_generations.borrow(), [first.generation]);

    host.teardown(&first).unwrap();
    assert!(host.render(&()).unwrap().is_none());
    assert!(host.render_factory()(&()).unwrap().is_none());
    assert!(host.edit(&first, []).is_err());

    assert!(captured_factory(&()).unwrap().is_some());
    assert_eq!(
        *rendered_generations.borrow(),
        [first.generation, first.generation]
    );

    let second = host.mount(document()).unwrap();
    assert!(second.generation > first.generation);
    assert!(host.render_factory()(&()).unwrap().is_some());
    assert_eq!(
        *rendered_generations.borrow(),
        [first.generation, first.generation, second.generation]
    );
}
