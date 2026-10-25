use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use rustui::{
    preview::{
        ComponentId, ComponentSchema, PreviewDocument, PreviewHost, PreviewId, PreviewRegistry,
        PropertyId, PropertyKind, PropertySchema, PropertyValue,
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

#[test]
fn captured_factory_keeps_its_revision_while_a_new_factory_sees_an_edit() {
    let rendered_values = Rc::new(RefCell::new(Vec::new()));
    let component_id = ComponentId::new("editable-text").unwrap();
    let property_id = PropertyId::new("text").unwrap();
    let factory_property_id = property_id.clone();
    let factory_rendered_values = Rc::clone(&rendered_values);
    let mut registry = PreviewRegistry::new();
    registry
        .register(
            ComponentSchema {
                id: component_id.clone(),
                version: 1,
                display_name: "Editable text".into(),
                properties: BTreeMap::from([(
                    property_id.clone(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("before".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                )]),
                events: BTreeMap::new(),
            },
            move |snapshot, _| {
                let PropertyValue::String(value) =
                    &snapshot.preview.properties[&factory_property_id]
                else {
                    unreachable!("validated text property")
                };
                factory_rendered_values
                    .borrow_mut()
                    .push((snapshot.stamp.revision, value.clone()));
                AnyView::new(EmptyView)
            },
        )
        .unwrap();

    let mut host = PreviewHost::new(registry);
    let initial = host
        .mount(PreviewDocument::new(
            PreviewId::new("editable-text-preview").unwrap(),
            component_id,
            1,
        ))
        .unwrap();
    let captured_factory = host.render_factory();
    assert!(captured_factory(&()).unwrap().is_some());
    assert_eq!(
        *rendered_values.borrow(),
        [(initial.revision, "before".into())]
    );

    let updated = host
        .edit(
            &initial,
            [(property_id, PropertyValue::String("after".into()))],
        )
        .unwrap();
    assert_eq!(updated.revision, initial.revision + 1);

    assert!(captured_factory(&()).unwrap().is_some());
    let current_factory = host.render_factory();
    assert!(current_factory(&()).unwrap().is_some());
    assert_eq!(
        *rendered_values.borrow(),
        [
            (initial.revision, "before".into()),
            (initial.revision, "before".into()),
            (updated.revision, "after".into()),
        ]
    );
}
