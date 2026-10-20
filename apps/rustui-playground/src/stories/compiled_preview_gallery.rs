//! D1c schema-driven local preview inspector; all controls use public RustUI APIs.
use rustui::{prelude::*, preview::*};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

pub fn demo() -> impl View {
    Component("compiled-preview-gallery", CompiledPreviewGallery::default)
}

struct FactoryContext {
    owner: OwnerAction<CompiledPreviewGallery>,
    text: Binding<String>,
    checked: Binding<bool>,
}

#[derive(Clone)]
struct PropertyBindingContext {
    property: PropertyId,
    stamp: Rc<RefCell<Option<PreviewStamp>>>,
}

fn property_string(snapshot: &PreviewSnapshot, name: &str) -> Option<String> {
    match snapshot
        .preview
        .properties
        .get(&PropertyId::new(name).ok()?)?
    {
        PropertyValue::String(value) => Some(value.clone()),
        _ => None,
    }
}

fn property_boolean(snapshot: &PreviewSnapshot, name: &str) -> Option<bool> {
    match snapshot
        .preview
        .properties
        .get(&PropertyId::new(name).ok()?)?
    {
        PropertyValue::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn property_number(snapshot: &PreviewSnapshot, name: &str) -> Option<f64> {
    match snapshot
        .preview
        .properties
        .get(&PropertyId::new(name).ok()?)?
    {
        PropertyValue::Number(value) => Some(*value),
        _ => None,
    }
}

struct CompiledPreviewGallery {
    host: PreviewHost<FactoryContext>,
    stamp_lease: Rc<RefCell<Option<PreviewStamp>>>,
    text: String,
    enabled: bool,
    amount: f64,
    amount_draft: String,
    stale_amount_callback: Option<Binding<String>>,
    activations: usize,
    renders: usize,
    status: String,
    descriptor_json: String,
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
                properties: BTreeMap::from([
                    (
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
                    ),
                    (
                        PropertyId::new("enabled").unwrap(),
                        PropertySchema {
                            kind: PropertyKind::Boolean,
                            default: Some(PropertyValue::Boolean(true)),
                            required: false,
                            editable: true,
                            minimum: None,
                            maximum: None,
                            min_length: None,
                            max_length: None,
                            allowed_values: vec![],
                        },
                    ),
                    (
                        PropertyId::new("amount").unwrap(),
                        PropertySchema {
                            kind: PropertyKind::Number,
                            default: Some(PropertyValue::Number(0.375)),
                            required: false,
                            editable: true,
                            minimum: Some(0.0),
                            maximum: Some(1.0),
                            min_length: None,
                            max_length: None,
                            allowed_values: vec![],
                        },
                    ),
                ]),
                events,
            };
            registry
                .register(schema, move |snapshot, context: &FactoryContext| {
                    let PropertyValue::String(text) =
                        &snapshot.preview.properties[&PropertyId::new("text").unwrap()]
                    else {
                        unreachable!()
                    };
                    let PropertyValue::Boolean(enabled) =
                        &snapshot.preview.properties[&PropertyId::new("enabled").unwrap()]
                    else {
                        unreachable!()
                    };
                    let PropertyValue::Number(amount) =
                        &snapshot.preview.properties[&PropertyId::new("amount").unwrap()]
                    else {
                        unreachable!()
                    };
                    let value_label = format!("{text} · enabled={enabled} · amount={amount}");
                    match name {
                        "text" => AnyView::new(Text(value_label)),
                        "button" => AnyView::new(
                            Button(
                                value_label,
                                context.owner.action_with(
                                    snapshot.signal(EventId::new("activate").unwrap()),
                                    CompiledPreviewGallery::activate,
                                ),
                            )
                            .id("compiled-preview-button"),
                        ),
                        _ => AnyView::new(
                            VStack::new()
                                .spacing(4.0)
                                .child(
                                    TextField(text.clone(), context.text.clone())
                                        .id(format!(
                                            "compiled-preview-text-field-{}",
                                            snapshot.stamp.generation
                                        ))
                                        .accessibility_label("Compiled preview text field"),
                                )
                                .child(Text(format!("enabled={enabled} · amount={amount}"))),
                        ),
                    }
                })
                .unwrap();
        }
        let toggle_schema = ComponentSchema {
            id: ComponentId::new("toggle").unwrap(),
            version: 1,
            display_name: "Toggle".into(),
            properties: BTreeMap::from([
                (
                    PropertyId::new("label").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("Preview toggle".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: Some(200),
                        allowed_values: vec![],
                    },
                ),
                (
                    PropertyId::new("checked").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::Boolean,
                        default: Some(PropertyValue::Boolean(false)),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                ),
            ]),
            events: BTreeMap::new(),
        };
        registry
            .register(toggle_schema, |snapshot, context: &FactoryContext| {
                let label = property_string(snapshot, "label").expect("validated toggle label");
                AnyView::new(Toggle(label, context.checked.clone()).id(format!(
                    "compiled-preview-toggle-{}",
                    snapshot.stamp.generation
                )))
            })
            .unwrap();
        registry
            .register_action(ActionRef::new("increment").unwrap())
            .unwrap();
        let mut result = Self {
            host: PreviewHost::new(registry),
            stamp_lease: Rc::new(RefCell::new(None)),
            text: "Preview text".into(),
            enabled: true,
            amount: 0.375,
            amount_draft: "0.375".into(),
            stale_amount_callback: None,
            activations: 0,
            renders: 0,
            status: "Mounted TextField".into(),
            descriptor_json: String::new(),
        };
        result.mount("text-field");
        result.export_descriptor();
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
        if component == "toggle" {
            document.properties.insert(
                PropertyId::new("label").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("checked").unwrap(),
                PropertyValue::Boolean(self.enabled),
            );
        } else {
            document.properties.insert(
                PropertyId::new("text").unwrap(),
                PropertyValue::String(self.text.clone()),
            );
            document.properties.insert(
                PropertyId::new("enabled").unwrap(),
                PropertyValue::Boolean(self.enabled),
            );
            document.properties.insert(
                PropertyId::new("amount").unwrap(),
                PropertyValue::Number(self.amount),
            );
        }
        if component == "button" {
            document.actions.insert(
                EventId::new("activate").unwrap(),
                ActionRef::new("increment").unwrap(),
            );
        }
        self.status = match self.host.mount(document) {
            Ok(stamp) => {
                // A new lease per generation keeps callbacks from an older mount stale.
                self.stamp_lease = Rc::new(RefCell::new(Some(stamp.clone())));
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
    fn mount_toggle(&mut self) {
        self.mount("toggle");
    }
    fn rerender(&mut self) {
        self.renders += 1;
    }
    fn export_descriptor(&mut self) {
        let Some(snapshot) = self.host.snapshot() else {
            self.status = "No mounted preview to export".into();
            return;
        };
        // The descriptor is the editable/importable representation; export
        // resolved values from the live host so accepted binding feedback is
        // included even if a compiled factory rendered an older snapshot.
        let mut document = snapshot.preview.document.clone();
        document.properties.clone_from(&snapshot.preview.properties);
        match encode_preview(&document, self.host.catalog()) {
            Ok(json) => {
                self.descriptor_json = json;
                self.status = "Exported normalized preview descriptor".into();
            }
            Err(error) => self.status = format!("Export rejected: {error}"),
        }
    }
    fn import_descriptor(&mut self) {
        let parsed = match decode_preview(&self.descriptor_json, self.host.catalog()) {
            Ok(preview) => preview.document,
            Err(error) => {
                self.status = format!("Import rejected: {error}");
                return;
            }
        };
        let is_toggle = parsed.component_id.as_str() == "toggle";
        match self.host.mount(parsed) {
            Ok(stamp) => {
                if let Some(snapshot) = self.host.snapshot() {
                    self.text = property_string(snapshot, if is_toggle { "label" } else { "text" })
                        .unwrap_or_default();
                    self.enabled =
                        property_boolean(snapshot, if is_toggle { "checked" } else { "enabled" })
                            .unwrap_or(true);
                    self.amount = property_number(snapshot, "amount").unwrap_or(0.375);
                    self.amount_draft = self.amount.to_string();
                }
                self.stamp_lease = Rc::new(RefCell::new(Some(stamp.clone())));
                self.status = format!(
                    "Imported descriptor for {} · generation {}",
                    stamp.preview_id, stamp.generation
                );
                let status = self.status.clone();
                self.export_descriptor();
                self.status = status;
            }
            Err(error) => self.status = format!("Import rejected: {error}"),
        }
    }
    fn read_descriptor(owner: &Self) -> String {
        owner.descriptor_json.clone()
    }
    fn write_descriptor(owner: &mut Self, value: String) {
        owner.descriptor_json = value;
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
    fn capture_stale_amount_callback(&mut self) {
        let Some(stamp) = self.host.snapshot().map(|snapshot| snapshot.stamp.clone()) else {
            self.status = "No mounted preview to capture".into();
            return;
        };
        self.stale_amount_callback = Some(Binding::new_with(
            self.amount_draft.clone(),
            PropertyBindingContext {
                property: PropertyId::new("amount").unwrap(),
                stamp: Rc::new(RefCell::new(Some(stamp.clone()))),
            },
            Self::read_amount_draft,
            Self::write_amount_draft,
        ));
        self.status = format!(
            "Captured amount callback at generation {} revision {}",
            stamp.generation, stamp.revision
        );
    }
    fn replay_stale_amount_callback(&mut self) {
        let Some(callback) = self.stale_amount_callback.clone() else {
            self.status = "Capture an amount callback before replay".into();
            return;
        };
        let draft_before = self.amount_draft.clone();
        let amount_before = self
            .host
            .snapshot()
            .and_then(|snapshot| property_number(snapshot, "amount"));
        let accepted = callback.write_to(self, "0.987".into());
        let amount_after = self
            .host
            .snapshot()
            .and_then(|snapshot| property_number(snapshot, "amount"));
        let unchanged = self.amount_draft == draft_before && amount_after == amount_before;
        self.status =
            format!("Stale amount callback accepted={accepted}; state unchanged={unchanged}");
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
    fn read_text(&self, _context: &PropertyBindingContext) -> String {
        self.text.clone()
    }
    fn write_text(&mut self, context: &PropertyBindingContext, text: String) -> bool {
        let stamp = context.stamp.borrow().clone();
        let Some(stamp) = stamp else {
            return false;
        };
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::String(text.clone()),
        ) {
            Ok(next) => {
                self.text = text;
                self.status = "Binding feedback accepted".into();
                // The same retained InputState subscription will serve the next
                // edit, so advance its lease only after this exact revision accepts.
                *context.stamp.borrow_mut() = Some(next);
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }
    fn read_enabled(&self, _context: &PropertyBindingContext) -> bool {
        self.enabled
    }
    fn write_enabled(&mut self, context: &PropertyBindingContext, enabled: bool) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Boolean(enabled),
        ) {
            Ok(next) => {
                self.enabled = enabled;
                *context.stamp.borrow_mut() = Some(next);
                self.status = "Boolean property accepted".into();
                true
            }
            Err(error) => {
                self.status = error.to_string();
                false
            }
        }
    }
    fn read_amount_draft(&self, _context: &PropertyBindingContext) -> String {
        self.amount_draft.clone()
    }
    fn write_amount_draft(&mut self, context: &PropertyBindingContext, draft: String) -> bool {
        let Some(stamp) = context.stamp.borrow().clone() else {
            return false;
        };
        if self.host.snapshot().map(|snapshot| &snapshot.stamp) != Some(&stamp) {
            return false;
        }
        self.amount_draft = draft.clone();
        let Ok(amount) = draft.trim().parse::<f64>() else {
            self.status = "Enter a finite number from 0 to 1".into();
            return true;
        };
        if !amount.is_finite() {
            self.status = "Enter a finite number from 0 to 1".into();
            return true;
        }
        match self.host.binding_feedback(
            &stamp,
            context.property.clone(),
            PropertyValue::Number(amount),
        ) {
            Ok(next) => {
                self.amount = amount;
                *context.stamp.borrow_mut() = Some(next);
                self.status = format!("Accepted f64 value {}", amount);
                true
            }
            Err(error) => {
                self.status = error.to_string();
                true
            }
        }
    }
}

impl View for CompiledPreviewGallery {
    fn body(&self) -> impl View {
        let render_preview = self.host.render_factory();
        let inspector_rows = self
            .host
            .snapshot()
            .and_then(|snapshot| {
                self.host
                    .catalog()
                    .components
                    .get(&snapshot.preview.document.component_id)
            })
            .map(|schema| {
                let generation = self
                    .host
                    .snapshot()
                    .map(|snapshot| snapshot.stamp.generation)
                    .unwrap_or_default();
                schema
                    .properties
                    .iter()
                    .filter(|(_, property)| property.editable)
                    .filter_map(|(id, property)| {
                        // A remount is a new binding lease. Scope each editor
                        // to that generation so retained input state cannot
                        // keep the previous generation's captured lease.
                        let row_id = format!("{}-generation-{generation}", id.as_str());
                        let context = PropertyBindingContext {
                            property: id.clone(),
                            stamp: self.stamp_lease.clone(),
                        };
                        match property.kind {
                            PropertyKind::String => Some(PropertyInspectorRow::text(
                                row_id.clone(),
                                id.as_str(),
                                Binding::new_with(
                                    self.text.clone(),
                                    context,
                                    Self::read_text,
                                    Self::write_text,
                                ),
                            )),
                            PropertyKind::Boolean => Some(PropertyInspectorRow::toggle(
                                row_id.clone(),
                                id.as_str(),
                                Binding::new_with(
                                    self.enabled,
                                    context,
                                    Self::read_enabled,
                                    Self::write_enabled,
                                ),
                            )),
                            PropertyKind::Number => Some(PropertyInspectorRow::text(
                                row_id,
                                format!("{} (f64)", id.as_str()),
                                Binding::new_with(
                                    self.amount_draft.clone(),
                                    context,
                                    Self::read_amount_draft,
                                    Self::write_amount_draft,
                                ),
                            )),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let descriptor_binding = Binding::new::<Self>(
            self.descriptor_json.clone(),
            Self::read_descriptor,
            Self::write_descriptor,
        );
        let binding = Binding::new_with(
            self.text.clone(),
            PropertyBindingContext {
                property: PropertyId::new("text").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            Self::read_text,
            Self::write_text,
        );
        let checked_binding = Binding::new_with(
            self.enabled,
            PropertyBindingContext {
                property: PropertyId::new("checked").unwrap(),
                stamp: self.stamp_lease.clone(),
            },
            Self::read_enabled,
            Self::write_enabled,
        );
        VStack::new()
            .spacing(8.0)
            .child(Text("Compiled preview host"))
            .child(Text(format!(
                "Owner: text={} enabled={} amount={} · mounted stamp: {:?}",
                self.text,
                self.enabled,
                self.amount,
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
                    ))
                    .child(Button(
                        "Preview Toggle",
                        rustui::runtime::Action::new(Self::mount_toggle),
                    )),
            )
            .child(Text("Schema inspector"))
            .child(
                PropertyInspector("compiled-preview-inspector", inspector_rows)
                    .expect("schema property IDs are unique and valid"),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Rerender preview parent",
                        rustui::runtime::Action::new(Self::rerender),
                    ))
                    .child(Button(
                        "Teardown preview",
                        rustui::runtime::Action::new(Self::teardown),
                    )),
            )
            .child(Text("Preview descriptor JSON"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Import descriptor",
                        rustui::runtime::Action::new(Self::import_descriptor),
                    ))
                    .child(Button(
                        "Export normalized descriptor",
                        rustui::runtime::Action::new(Self::export_descriptor),
                    )),
            )
            .child(OwnerActionReader(
                move |owner: OwnerAction<CompiledPreviewGallery>| {
                    KeyboardShortcuts(
                        "compiled-preview-import-shortcut",
                        TextEditor(
                            "Import or export normalized preview JSON",
                            descriptor_binding.clone(),
                        )
                        .id("compiled-preview-descriptor-json")
                        .height(150.0),
                    )
                    .shortcut(
                        "secondary-shift-enter",
                        owner.action(Self::import_descriptor),
                    )
                },
            ))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Capture amount callback",
                        rustui::runtime::Action::new(Self::capture_stale_amount_callback),
                    ))
                    .child(Button(
                        "Replay captured amount callback",
                        rustui::runtime::Action::new(Self::replay_stale_amount_callback),
                    ))
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
                        checked: checked_binding.clone(),
                    })
                    .expect("registered compiled factory")
                },
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_preview_declares_label_and_checked_and_accepts_checked_feedback() {
        let mut gallery = CompiledPreviewGallery::default();
        let toggle_id = ComponentId::new("toggle").unwrap();
        let schema = &gallery.host.catalog().components[&toggle_id];
        assert_eq!(schema.properties.len(), 2);
        assert_eq!(
            schema.properties[&PropertyId::new("label").unwrap()].kind,
            PropertyKind::String
        );
        assert_eq!(
            schema.properties[&PropertyId::new("checked").unwrap()].kind,
            PropertyKind::Boolean
        );

        let stamp = gallery
            .host
            .mount(PreviewDocument::new(
                PreviewId::new("toggle-test").unwrap(),
                toggle_id,
                1,
            ))
            .unwrap();
        let next = gallery
            .host
            .binding_feedback(
                &stamp,
                PropertyId::new("checked").unwrap(),
                PropertyValue::Boolean(true),
            )
            .unwrap();

        assert_eq!(next.revision, stamp.revision + 1);
        assert_eq!(
            gallery.host.snapshot().unwrap().preview.properties
                [&PropertyId::new("checked").unwrap()],
            PropertyValue::Boolean(true),
        );
    }
}
