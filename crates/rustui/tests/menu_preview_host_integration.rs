use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use rustui::{
    prelude::{AppAction, Menu, MenuItem},
    preview::{
        ActionRef, ComponentId, ComponentSchema, EventId, EventSchema, PreviewDocument,
        PreviewHost, PreviewId, PreviewRegistry, PropertyId, PropertyKind, PropertySchema,
        PropertyValue,
    },
    view::AnyView,
};

#[derive(Clone, Default)]
struct MenuFactoryContext {
    built_menus: Rc<RefCell<Vec<(String, rustui::preview::PreviewSignal)>>>,
}

#[test]
fn mounted_menu_preview_factory_builds_menu_from_snapshot_and_signal() {
    let component_id = ComponentId::new("menu-specimen").unwrap();
    let trigger_label_id = PropertyId::new("trigger_label").unwrap();
    let select_event_id = EventId::new("select").unwrap();
    let action_ref = ActionRef::new("increment").unwrap();
    let factory_trigger_label_id = trigger_label_id.clone();
    let factory_select_event_id = select_event_id.clone();
    let mut registry = PreviewRegistry::new();
    registry.register_action(action_ref.clone()).unwrap();
    registry
        .register(
            ComponentSchema {
                id: component_id.clone(),
                version: 1,
                display_name: "Menu".into(),
                properties: BTreeMap::from([(
                    trigger_label_id.clone(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview menu".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(80),
                        allowed_values: vec![],
                    },
                )]),
                events: BTreeMap::from([(
                    select_event_id.clone(),
                    EventSchema {
                        display_name: "Select menu action".into(),
                        payload: BTreeMap::new(),
                    },
                )]),
            },
            move |snapshot, context: &MenuFactoryContext| {
                let PropertyValue::String(trigger_label) =
                    &snapshot.preview.properties[&factory_trigger_label_id]
                else {
                    unreachable!("validated menu trigger label")
                };
                let signal = snapshot.signal(factory_select_event_id.clone());
                context
                    .built_menus
                    .borrow_mut()
                    .push((trigger_label.clone(), signal));

                AnyView::new(Menu(
                    trigger_label.clone(),
                    [MenuItem::action(
                        "select",
                        "Run preview action",
                        AppAction::new(|_| {}),
                    )],
                ))
            },
        )
        .unwrap();

    let mut host = PreviewHost::new(registry);
    let mut document =
        PreviewDocument::new(PreviewId::new("menu-preview").unwrap(), component_id, 1);
    document
        .properties
        .insert(trigger_label_id, PropertyValue::String("Actions".into()));
    document
        .actions
        .insert(select_event_id.clone(), action_ref.clone());
    let stamp = host.mount(document).unwrap();
    let context = MenuFactoryContext::default();

    let rendered = host.render_factory()(&context).unwrap();

    assert!(rendered.is_some());
    let built_menus = context.built_menus.borrow();
    assert_eq!(built_menus.len(), 1);
    assert_eq!(built_menus[0].0, "Actions");
    assert_eq!(built_menus[0].1.stamp, stamp);
    assert_eq!(built_menus[0].1.event, select_event_id);
    assert_eq!(host.dispatch(&built_menus[0].1).unwrap(), action_ref);
}
