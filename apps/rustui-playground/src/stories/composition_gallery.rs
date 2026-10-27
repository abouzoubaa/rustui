//! Structural composition editing through RustUI's public compiled-factory API.
use rustui::{component, prelude::*, preview::*, view::EmptyView};
use std::collections::{BTreeMap, BTreeSet};

pub fn demo() -> impl View {
    Component("composition-gallery", CompositionGallery::new)
}

#[cfg(target_family = "wasm")]
pub(crate) fn reference_demo() -> impl View {
    Component("browser-composition-reference", || {
        let mut owner = CompositionGallery::new();
        if let Some(Ok(model)) = &mut owner.model {
            model.host.unmount();
        }
        owner.reference_session = Some(crate::preview_transport::Session::default());
        owner.reference_bridge = Some(crate::preview_transport_web::Bridge::new(
            CompositionGallery::reference_request,
            CompositionGallery::reference_close,
        ));
        owner
    })
}

struct Model {
    host: CompositionHost<()>,
    stamp: CompositionStamp,
    next_id: u64,
    status: String,
}

impl Model {
    fn new() -> Result<Self, PreviewError> {
        let mut registry = PreviewRegistry::<()>::new();
        let column = ComponentId::new("composition-column").map_err(PreviewError::Invalid)?;
        let text = ComponentId::new("composition-text").map_err(PreviewError::Invalid)?;
        let body = SlotId::new("body")?;
        registry.register(
            ComponentSchema {
                id: text.clone(),
                version: 1,
                display_name: "Text".into(),
                properties: BTreeMap::from([(
                    PropertyId::new("label").map_err(PreviewError::Invalid)?,
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: Some(PropertyValue::String("New child".into())),
                        required: false,
                        editable: true,
                        minimum: None,
                        maximum: None,
                        min_length: Some(1),
                        max_length: Some(120),
                        allowed_values: vec![],
                    },
                )]),
                events: BTreeMap::new(),
            },
            |snapshot, _| {
                let label = snapshot
                    .preview
                    .properties
                    .values()
                    .find_map(|value| match value {
                        PropertyValue::String(text) => Some(text.clone()),
                        _ => None,
                    })
                    .unwrap_or_default();
                AnyView::new(Text(label))
            },
        )?;
        registry.register_composable(
            ComponentSchema {
                id: column.clone(),
                version: 1,
                display_name: "Column".into(),
                properties: BTreeMap::new(),
                events: BTreeMap::new(),
            },
            BTreeMap::from([(
                body.clone(),
                SlotSchema {
                    allowed_children: BTreeSet::from([text]),
                    min_children: 0,
                    max_children: 8,
                },
            )]),
            |_, _, slots| {
                let mut column = VStack::new().spacing(8.0);
                for child in slots.into_values().flatten() {
                    column = column.child(child);
                }
                AnyView::new(column)
            },
        )?;
        let preview_id = PreviewId::new("composition-gallery").map_err(PreviewError::Invalid)?;
        let root = NodeId::new("root")?;
        let mut host = CompositionHost::new(registry);
        let stamp = host.mount(CompositionDocument {
            format: COMPOSITION_FORMAT.into(),
            format_version: COMPOSITION_FORMAT_VERSION,
            preview_id: preview_id.clone(),
            root: root.clone(),
            nodes: BTreeMap::from([(
                root,
                CompositionNode {
                    preview: PreviewDocument::new(preview_id, column, 1),
                    slots: BTreeMap::new(),
                },
            )]),
        })?;
        let mut model = Self {
            host,
            stamp,
            next_id: 1,
            status: String::new(),
        };
        model.add()?;
        Ok(model)
    }

    fn add(&mut self) -> Result<(), PreviewError> {
        let root = self
            .host
            .document()
            .ok_or_else(|| PreviewError::Invalid("composition unmounted".into()))?
            .root
            .clone();
        let next_id = self
            .next_id
            .checked_add(1)
            .ok_or_else(|| PreviewError::Invalid("node identity exhausted".into()))?;
        let mut preview = PreviewDocument::new(
            self.stamp.preview_id.clone(),
            ComponentId::new("composition-text").map_err(PreviewError::Invalid)?,
            1,
        );
        preview.properties.insert(
            PropertyId::new("label").map_err(PreviewError::Invalid)?,
            PropertyValue::String(format!("Child {}", self.next_id)),
        );
        self.stamp = self.host.edit(
            &self.stamp,
            &[CompositionEdit::Insert {
                parent: root,
                slot: SlotId::new("body")?,
                node: NodeId::new(format!("child-{}", self.next_id))?,
                value: CompositionNode {
                    preview,
                    slots: BTreeMap::new(),
                },
            }],
        )?;
        self.next_id = next_id;
        Ok(())
    }

    fn children(&self) -> Vec<NodeId> {
        self.host
            .document()
            .and_then(|document| document.nodes.get(&document.root))
            .and_then(|node| node.slots.values().next())
            .cloned()
            .unwrap_or_default()
    }

    fn rename(&mut self) -> Result<(), PreviewError> {
        if let Some(node) = self.children().first() {
            self.stamp = self.host.edit(
                &self.stamp,
                &[CompositionEdit::SetProperty {
                    node: node.clone(),
                    property: PropertyId::new("label").map_err(PreviewError::Invalid)?,
                    value: PropertyValue::String("Renamed child".into()),
                }],
            )?;
        }
        Ok(())
    }

    fn remove(&mut self) -> Result<(), PreviewError> {
        if let Some(node) = self.children().last() {
            self.stamp = self.host.edit(
                &self.stamp,
                &[CompositionEdit::RemoveSubtree { node: node.clone() }],
            )?;
        }
        Ok(())
    }

    fn move_first(&mut self) -> Result<(), PreviewError> {
        if let Some(node) = self.children().first() {
            let root = self
                .host
                .document()
                .ok_or_else(|| PreviewError::Invalid("composition unmounted".into()))?
                .root
                .clone();
            self.stamp = self.host.edit(
                &self.stamp,
                &[CompositionEdit::Move {
                    node: node.clone(),
                    parent: root,
                    slot: SlotId::new("body")?,
                }],
            )?;
        }
        Ok(())
    }
}

#[component]
struct CompositionGallery {
    reference_session: Option<crate::preview_transport::Session>,
    #[cfg(target_family = "wasm")]
    reference_bridge: Option<crate::preview_transport_web::Bridge<Self>>,
    model: Option<Result<Model, PreviewError>>,
    #[state(0_u64)]
    revision: u64,
}
impl CompositionGallery {
    fn reference_owner_binding(&self) -> AnyView {
        #[cfg(target_family = "wasm")]
        if let Some(bridge) = self.reference_bridge.clone() {
            return AnyView::new(OwnerActionReader(move |owner: OwnerAction<Self>| {
                bridge.bind_owner(owner);
                EmptyView
            }));
        }
        AnyView::new(EmptyView)
    }
    fn reference_close(&mut self) -> bool {
        if let Some(mut session) = self.reference_session.take() {
            if let Some(Ok(model)) = &mut self.model {
                session.close(crate::preview_transport::Host::Composition(&mut model.host));
            }
        }
        true
    }

    fn reference_request(
        &mut self,
        delivery: crate::preview_transport::Delivery,
    ) -> (crate::preview_transport::Reply, bool) {
        use crate::preview_transport::{Host, Reply};
        let Some(mut session) = self.reference_session.take() else {
            return (Reply::unavailable(&delivery.request), false);
        };
        let Some(Ok(model)) = &mut self.model else {
            self.reference_session = Some(session);
            return (Reply::unavailable(&delivery.request), false);
        };
        // This actual registered catalog has no action targets; future owners
        // opt in with a compiled action map rather than trusting wire names.
        let actions = std::collections::BTreeSet::new();
        let outcome = session.handle(
            delivery.request,
            Host::Composition(&mut model.host),
            &actions,
            &delivery.fresh,
        );
        self.reference_session = Some(session);
        if let Some(stamp) = model.host.stamp().cloned() {
            model.stamp = stamp;
        }
        if outcome.changed {
            model.status = "Browser reference request accepted".into();
            self.revision = self.revision.saturating_add(1);
        }
        (outcome.reply, outcome.changed)
    }
    fn new() -> Self {
        Self {
            reference_session: None,
            #[cfg(target_family = "wasm")]
            reference_bridge: None,
            model: Some(Model::new()),
            revision: 0,
        }
    }
    fn change(&mut self, operation: fn(&mut Model) -> Result<(), PreviewError>) {
        if let Some(Ok(model)) = &mut self.model {
            model.status = operation(model)
                .err()
                .map(|error| error.to_string())
                .unwrap_or_default();
        }
        self.revision = self.revision.saturating_add(1);
    }
    fn add(&mut self) {
        self.change(Model::add);
    }
    fn remove(&mut self) {
        self.change(Model::remove);
    }
    fn rename(&mut self) {
        self.change(Model::rename);
    }
    fn move_first(&mut self) {
        self.change(Model::move_first);
    }
}
impl View for CompositionGallery {
    fn on_mount(&mut self, cx: &mut rustui::App) {
        #[cfg(target_family = "wasm")]
        if let Some(bridge) = &self.reference_bridge {
            bridge.install(cx);
        }
        #[cfg(not(target_family = "wasm"))]
        let _ = cx;
    }
    fn on_unmount(&mut self, _cx: &mut rustui::App) {
        if let Some(mut session) = self.reference_session.take() {
            if let Some(Ok(model)) = &mut self.model {
                session.close(crate::preview_transport::Host::Composition(&mut model.host));
            }
        }
        #[cfg(target_family = "wasm")]
        if let Some(bridge) = self.reference_bridge.take() {
            bridge.close();
        }
    }
    fn body(&self) -> impl View {
        let _revision = self.revision();
        let (view, status, count) = match self.model.as_ref() {
            Some(Ok(model)) => (
                match model.host.render(&()) {
                    Ok(Some(view)) => view,
                    Ok(None) => AnyView::new(Text("Composition unmounted")),
                    Err(error) => AnyView::new(Text(error.to_string())),
                },
                model.status.clone(),
                model.children().len(),
            ),
            Some(Err(error)) => (AnyView::new(Text(error.to_string())), String::new(), 0),
            None => (
                AnyView::new(Text("Composition unmounted")),
                String::new(),
                0,
            ),
        };
        let can_add = self
            .model
            .as_ref()
            .and_then(|model| model.as_ref().ok())
            .and_then(|model| model.host.document())
            .and_then(|document| document.nodes.get(&document.root))
            .is_some_and(|root| root.preview.component_id.as_str() == "composition-column");
        VStack::new()
            .spacing(8.0)
            .child(self.reference_owner_binding())
            .child(LocalizedText("lab-composition-title"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Add child", self.action(Self::add))
                            .id("composition-add")
                            .disabled(!can_add || count >= 8),
                    )
                    .child(
                        Button("Rename first child", self.action(Self::rename))
                            .id("composition-rename")
                            .disabled(count == 0),
                    )
                    .child(
                        Button("Remove last child", self.action(Self::remove))
                            .id("composition-remove")
                            .disabled(count == 0),
                    )
                    .child(
                        Button("Move first to end", self.action(Self::move_first))
                            .id("composition-move")
                            .disabled(count < 2),
                    ),
            )
            .child(view)
            .child(Text(status))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_actions_follow_document_root_identity() {
        let mut model = Model::new().unwrap();
        let mut document = model.host.document().unwrap().clone();
        let original_root = document.root.clone();
        document.root = NodeId::new("custom-root").unwrap();
        let root = document.nodes.remove(&original_root).unwrap();
        document.nodes.insert(document.root.clone(), root);
        model.stamp = model.host.mount(document).unwrap();

        model.add().unwrap();
        model.move_first().unwrap();
        assert_eq!(
            model
                .children()
                .iter()
                .map(NodeId::as_str)
                .collect::<Vec<_>>(),
            ["child-2", "child-1"]
        );
    }

    #[test]
    fn public_factory_composition_add_move_remove_and_rejection_are_live() {
        let mut model = Model::new().unwrap();
        model.add().unwrap();
        assert_eq!(
            model
                .children()
                .iter()
                .map(NodeId::as_str)
                .collect::<Vec<_>>(),
            ["child-1", "child-2"]
        );
        model.move_first().unwrap();
        assert_eq!(
            model
                .children()
                .iter()
                .map(NodeId::as_str)
                .collect::<Vec<_>>(),
            ["child-2", "child-1"]
        );
        model.remove().unwrap();
        assert_eq!(model.children()[0].as_str(), "child-2");
        model.rename().unwrap();
        assert_eq!(
            model.host.document().unwrap().nodes[&model.children()[0]]
                .preview
                .properties[&PropertyId::new("label").unwrap()],
            PropertyValue::String("Renamed child".into())
        );
        assert!(model.host.render(&()).unwrap().is_some());
        for _ in 0..7 {
            model.add().unwrap();
        }
        let stamp = model.stamp.clone();
        assert!(model.add().is_err());
        assert_eq!(model.stamp, stamp);
        assert_eq!(model.children().len(), 8);
    }
}
