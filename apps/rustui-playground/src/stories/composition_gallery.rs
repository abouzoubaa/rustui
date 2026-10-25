//! Structural composition editing through RustUI's public compiled-factory API.
use rustui::{component, prelude::*, preview::*};
use std::collections::{BTreeMap, BTreeSet};

pub fn demo() -> impl View {
    Component("composition-gallery", CompositionGallery::new)
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
                parent: NodeId::new("root")?,
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
            self.stamp = self.host.edit(
                &self.stamp,
                &[CompositionEdit::Move {
                    node: node.clone(),
                    parent: NodeId::new("root")?,
                    slot: SlotId::new("body")?,
                }],
            )?;
        }
        Ok(())
    }
}

#[component]
struct CompositionGallery {
    model: Option<Result<Model, PreviewError>>,
    #[state(0_u64)]
    revision: u64,
}
impl CompositionGallery {
    fn new() -> Self {
        Self {
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
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-composition-title"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Add child", self.action(Self::add))
                            .id("composition-add")
                            .disabled(count >= 8),
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
