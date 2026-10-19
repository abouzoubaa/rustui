//! D1b local compiled preview specimens; all controls use the public RustUI API.
use rustui::{prelude::*, preview::*};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub fn demo() -> impl View {
    Component("compiled-preview-gallery", CompiledPreviewGallery::default)
}

struct FactoryContext {
    owner: OwnerAction<CompiledPreviewGallery>,
    text: Binding<String>,
}

struct CompiledPreviewGallery {
    host: PreviewHost<FactoryContext>,
    text_stamp: Rc<RefCell<Option<PreviewStamp>>>,
    text: String,
    activations: usize,
    renders: usize,
    status: String,
}

impl Default for CompiledPreviewGallery {
    fn default() -> Self {
        let mut registry = PreviewRegistry::new();
        for name in ["text", "button", "text-field"] {
            let mut events = BTreeMap::new();
            if name == "button" {
                events.insert(
                    EventId::new("activate").unwrap(),
                    EventSchema {
                        display_name: "Activate".into(),
                        payload: BTreeMap::new(),
                    },
                );
            }
            let schema = ComponentSchema {
                id: ComponentId::new(name).unwrap(),
                version: 1,
                display_name: name.into(),
                properties: BTreeMap::from([(
                    PropertyId::new("text").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview text".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                )]),
                events,
            };
            registry
                .register(schema, move |snapshot, context: &FactoryContext| {
                    let PropertyValue::String(text) =
                        &snapshot.preview.properties[&PropertyId::new("text").unwrap()]
                    else {
                        unreachable!()
                    };
                    match name {
                        "text" => AnyView::new(Text(text.clone())),
                        "button" => AnyView::new(
                            Button(
                                text.clone(),
                                context.owner.action_with(
                                    snapshot.signal(EventId::new("activate").unwrap()),
                                    CompiledPreviewGallery::activate,
                                ),
                            )
                            .id("compiled-preview-button"),
                        ),
                        _ => AnyView::new(
                            TextField(text.clone(), context.text.clone())
                                .id("compiled-preview-text-field")
                                .accessibility_label("Compiled preview text field"),
                        ),
                    }
                })
                .unwrap();
        }
        registry
            .register_action(ActionRef::new("increment").unwrap())
            .unwrap();
        let mut result = Self {
            host: PreviewHost::new(registry),
            text_stamp: Rc::new(RefCell::new(None)),
            text: "Preview text".into(),
            activations: 0,
            renders: 0,
            status: "Mounted TextField".into(),
        };
        result.mount("text-field");
        result
    }
}

impl CompiledPreviewGallery {
    fn mount(&mut self, component: &str) {
        let mut document = PreviewDocument::new(
            PreviewId::new("compiled-gallery").unwrap(),
            ComponentId::new(component).unwrap(),
            1,
        );
        document.properties.insert(
            PropertyId::new("text").unwrap(),
            PropertyValue::String(self.text.clone()),
        );
        if component == "button" {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        self.status = match self.host.mount(document) {
            Ok(stamp) => {
                // A new lease per generation keeps callbacks from an older mount stale.
                self.text_stamp = Rc::new(RefCell::new(Some(stamp.clone())));
                format!("Mounted {component}, generation {}", stamp.generation)
            }
            Err(error) => error.to_string(),
        };
    }
    fn mount_text(&mut self) {
        self.mount("text");
    }
    fn mount_button(&mut self) {
        self.mount("button");
    }
    fn mount_field(&mut self) {
        self.mount("text-field");
    }
    fn rerender(&mut self) {
        self.renders += 1;
    }
    fn edit(&mut self) {
        let Some(stamp) = self.host.snapshot().map(|s| s.stamp.clone()) else {
            return;
        };
        self.text = "Inspector edit".into();
        let result = self.host.edit(
            &stamp,
            [(
                PropertyId::new("text").unwrap(),
                PropertyValue::String(self.text.clone()),
            )],
        );
        if let Ok(next) = &result {
            *self.text_stamp.borrow_mut() = Some(next.clone());
        }
        self.status = format!("Inspector: {result:?}");
    }
    fn teardown(&mut self) {
        if let Some(stamp) = self.host.snapshot().map(|s| s.stamp.clone()) {
            self.status = format!("Teardown: {:?}", self.host.teardown(&stamp));
        }
    }
    fn activate(&mut self, signal: &PreviewSignal) {
        match self.host.dispatch(signal) {
            Ok(target) if target.as_str() == "increment" => {
                self.activations += 1;
                self.status = "Allowlisted increment action".into();
            }
            result => self.status = format!("Rejected signal: {result:?}"),
        }
    }
    fn reject_stale(&mut self) {
        let Some(current) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            return;
        };
        let stale = PreviewStamp {
            generation: 0,
            ..current
        };
        let signal = PreviewSignal {
            stamp: stale.clone(),
            event: EventId::new("activate").unwrap(),
        };
        let action_rejected = self.host.dispatch(&signal).is_err();
        let feedback_rejected = self
            .host
            .binding_feedback(
                &stale,
                PropertyId::new("text").unwrap(),
                PropertyValue::String("stale write".into()),
            )
            .is_err();
        self.status = format!(
            "Stale action rejected: {action_rejected}; stale binding rejected: {feedback_rejected}"
        );
    }
    fn reject_wrong_type(&mut self) {
        let Some(stamp) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            return;
        };
        let rejected = self
            .host
            .edit(
                &stamp,
                [(
                    PropertyId::new("text").unwrap(),
                    PropertyValue::Boolean(true),
                )],
            )
            .is_err();
        self.status = format!("Wrong property type rejected: {rejected}");
    }
    fn read_text(&self, _stamp: &Rc<RefCell<Option<PreviewStamp>>>) -> String {
        self.text.clone()
    }
    fn write_text(
        &mut self,
        stamp_lease: &Rc<RefCell<Option<PreviewStamp>>>,
        text: String,
    ) -> bool {
        let stamp = stamp_lease.borrow().clone();
        let Some(stamp) = stamp else {
            return false;
        };
        match self.host.binding_feedback(
            &stamp,
            PropertyId::new("text").unwrap(),
            PropertyValue::String(text.clone()),
        ) {
            Ok(next) => {
                self.text = text;
                self.status = "Binding feedback accepted".into();
                // The same retained InputState subscription will serve the next
                // edit, so advance its lease only after this exact revision accepts.
                *stamp_lease.borrow_mut() = Some(next);
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }
}

impl View for CompiledPreviewGallery {
    fn body(&self) -> impl View {
        let render_preview = self.host.render_factory();
        let binding = Binding::new_with(
            self.text.clone(),
            self.text_stamp.clone(),
            Self::read_text,
            Self::write_text,
        );
        VStack::new()
            .spacing(8.0)
            .child(Text("Compiled preview host"))
            .child(Text(format!(
                "Binding owner value: {} · mounted stamp: {:?}",
                self.text,
                self.host.snapshot().map(|snapshot| &snapshot.stamp)
            )))
            .child(Text(format!(
                "{} · activations {} · parent rerenders {}",
                self.status, self.activations, self.renders
            )))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Preview Text",
                        rustui::runtime::Action::new(Self::mount_text),
                    ))
                    .child(Button(
                        "Preview Button",
                        rustui::runtime::Action::new(Self::mount_button),
                    ))
                    .child(Button(
                        "Preview TextField",
                        rustui::runtime::Action::new(Self::mount_field),
                    )),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Edit preview property",
                        rustui::runtime::Action::new(Self::edit),
                    ))
                    .child(Button(
                        "Rerender preview parent",
                        rustui::runtime::Action::new(Self::rerender),
                    ))
                    .child(Button(
                        "Teardown preview",
                        rustui::runtime::Action::new(Self::teardown),
                    )),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Reject stale preview callbacks",
                        rustui::runtime::Action::new(Self::reject_stale),
                    ))
                    .child(Button(
                        "Reject wrong preview type",
                        rustui::runtime::Action::new(Self::reject_wrong_type),
                    )),
            )
            .child(OwnerActionReader(
                move |owner: OwnerAction<CompiledPreviewGallery>| {
                    render_preview(&FactoryContext {
                        owner,
                        text: binding.clone(),
                    })
                    .expect("registered compiled factory")
                },
            ))
    }
}
