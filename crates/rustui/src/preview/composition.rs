//! Additive, versioned descriptor-first composition over compiled factories.
use super::{
    host::{PreviewSnapshot, PreviewStamp},
    model::{
        ActionRef, ComponentId, EventId, MAX_DOCUMENT_BYTES, MAX_ID_BYTES, PreviewDocument,
        PreviewError, PreviewId, PropertyId, PropertyValue, deserialize_unique_map,
    },
    registry::PreviewRegistry,
    schema::validate_preview_with_validated_catalog,
    validate_preview,
};
use crate::view::{AnyView, Keyed};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    rc::Rc,
};

pub const COMPOSITION_FORMAT: &str = "rustui.composition";
pub const COMPOSITION_FORMAT_VERSION: u32 = 1;
pub const MAX_COMPOSITION_NODES: usize = 4096;
/// Keeps validation and recursive GPUI view construction bounded.
pub const MAX_COMPOSITION_DEPTH: usize = 128;
pub const MAX_SLOTS_PER_COMPONENT: usize = 128;
pub const MAX_COMPOSITION_EDITS: usize = MAX_COMPOSITION_NODES * 2;

macro_rules! stable_id {
    ($name:ident, $label:literal) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);
        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, PreviewError> {
                let value = value.into();
                let b = value.as_bytes();
                if b.is_empty()
                    || b.len() > MAX_ID_BYTES
                    || !(b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
                    || !b
                        .iter()
                        .all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || b"._-".contains(x))
                {
                    return Err(PreviewError::Invalid(format!(
                        "invalid {} `{value}`",
                        $label
                    )));
                }
                Ok(Self(value))
            }
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(&self.0)
            }
        }
    };
}
stable_id!(NodeId, "composition node id");
stable_id!(SlotId, "composition slot id");

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SlotSchema {
    #[serde(default, deserialize_with = "super::model::deserialize_unique_set")]
    pub allowed_children: BTreeSet<ComponentId>,
    #[serde(default)]
    pub min_children: usize,
    pub max_children: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionDocument {
    pub format: String,
    pub format_version: u32,
    pub preview_id: PreviewId,
    pub root: NodeId,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub nodes: BTreeMap<NodeId, CompositionNode>,
}
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompositionNode {
    pub preview: PreviewDocument,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub slots: BTreeMap<SlotId, Vec<NodeId>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionStamp {
    pub preview_id: PreviewId,
    pub generation: u64,
    pub revision: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub enum CompositionEdit {
    SetProperty {
        node: NodeId,
        property: PropertyId,
        value: PropertyValue,
    },
    Insert {
        parent: NodeId,
        slot: SlotId,
        node: NodeId,
        value: CompositionNode,
    },
    RemoveSubtree {
        node: NodeId,
    },
    Move {
        node: NodeId,
        parent: NodeId,
        slot: SlotId,
    },
}

fn validate_shape(d: &CompositionDocument) -> Result<(), PreviewError> {
    if d.format != COMPOSITION_FORMAT {
        return Err(PreviewError::UnsupportedCompositionFormat(d.format.clone()));
    }
    if d.format_version != COMPOSITION_FORMAT_VERSION {
        return Err(PreviewError::UnsupportedCompositionVersion(
            d.format_version,
        ));
    }
    if d.nodes.is_empty() || d.nodes.len() > MAX_COMPOSITION_NODES || !d.nodes.contains_key(&d.root)
    {
        return Err(PreviewError::Invalid(
            "composition requires one registered root and a bounded nonempty node map".into(),
        ));
    }
    let mut parents = BTreeMap::<NodeId, usize>::new();
    for (id, n) in &d.nodes {
        NodeId::new(id.as_str())?;
        if n.slots.len() > MAX_SLOTS_PER_COMPONENT {
            return Err(PreviewError::Invalid(
                "composition node exceeds slot limit".into(),
            ));
        }
        if n.preview.preview_id != d.preview_id {
            return Err(PreviewError::Invalid(format!(
                "node `{id}` has a different preview id"
            )));
        }
        for (slot, children) in &n.slots {
            SlotId::new(slot.as_str())?;
            for child in children {
                NodeId::new(child.as_str())?;
                if !d.nodes.contains_key(child) {
                    return Err(PreviewError::Invalid(format!(
                        "slot references missing node `{child}`"
                    )));
                }
                let count = parents.entry(child.clone()).or_default();
                *count = count.checked_add(1).ok_or_else(|| {
                    PreviewError::Invalid("composition parent count overflow".into())
                })?;
                if *count > 1 {
                    return Err(PreviewError::Invalid(
                        "composition node has multiple parents".into(),
                    ));
                }
            }
        }
    }
    if parents.contains_key(&d.root)
        || d.nodes
            .keys()
            .any(|id| id != &d.root && parents.get(id) != Some(&1))
    {
        return Err(PreviewError::Invalid(
            "composition root has a parent or the graph contains an orphan".into(),
        ));
    }
    let mut seen = BTreeSet::new();
    let mut todo = vec![(d.root.clone(), 1usize)];
    while let Some((id, depth)) = todo.pop() {
        if depth > MAX_COMPOSITION_DEPTH {
            return Err(PreviewError::Invalid(
                "composition exceeds maximum depth".into(),
            ));
        }
        if !seen.insert(id.clone()) {
            return Err(PreviewError::Invalid("composition contains a cycle".into()));
        }
        for cs in d.nodes[&id].slots.values() {
            if !cs.is_empty() && depth == MAX_COMPOSITION_DEPTH {
                return Err(PreviewError::Invalid(
                    "composition exceeds maximum depth".into(),
                ));
            }
            todo.extend(cs.iter().cloned().map(|child| (child, depth + 1)));
        }
    }
    if seen.len() != d.nodes.len() {
        return Err(PreviewError::Invalid(
            "composition contains unreachable nodes".into(),
        ));
    }
    Ok(())
}

fn validate_composition<C>(
    registry: &PreviewRegistry<C>,
    d: &CompositionDocument,
) -> Result<(), PreviewError> {
    validate_shape(d)?;
    let mut catalog_validated = false;
    for n in d.nodes.values() {
        let component = &n.preview.component_id;
        if !registry.catalog().components.contains_key(component) {
            return Err(PreviewError::UnknownComponent(component.clone()));
        }
        let empty_contract = BTreeMap::new();
        let contract = registry.slot_contract(component).unwrap_or(&empty_contract);
        for (slot, children) in &n.slots {
            let schema = contract.get(slot).ok_or_else(|| {
                PreviewError::Invalid(format!("component `{component}` has no slot `{slot}`"))
            })?;
            if children.len() < schema.min_children || children.len() > schema.max_children {
                return Err(PreviewError::Invalid(format!(
                    "slot `{slot}` child count is outside its bounds"
                )));
            }
            for child in children {
                let child_component = &d.nodes[child].preview.component_id;
                if !schema.allowed_children.contains(child_component) {
                    return Err(PreviewError::Invalid(format!(
                        "component `{child_component}` is not allowed in slot `{slot}`"
                    )));
                }
            }
        }
        for (slot, schema) in contract {
            let count = n.slots.get(slot).map_or(0, Vec::len);
            if count < schema.min_children {
                return Err(PreviewError::Invalid(format!(
                    "required slot `{slot}` is below minimum cardinality"
                )));
            }
        }
        if catalog_validated {
            validate_preview_with_validated_catalog(registry.catalog(), n.preview.clone())?;
        } else {
            validate_preview(registry.catalog(), n.preview.clone())?;
            catalog_validated = true;
        }
    }
    Ok(())
}

pub fn encode_composition<C>(
    d: &CompositionDocument,
    registry: &PreviewRegistry<C>,
) -> Result<String, PreviewError> {
    super::codec::check_serialized_text_budget(
        d.nodes.values().map(|node| &node.preview),
        "composition document exceeds size limit",
    )?;
    let mut d = d.clone();
    for node in d.nodes.values_mut() {
        super::codec::normalize_numeric_properties(registry.catalog(), &mut node.preview)?;
    }
    validate_composition(registry, &d)?;
    super::codec::encode_json_bounded(&d, "composition document exceeds size limit")
}
pub fn decode_composition<C>(
    source: &str,
    registry: &PreviewRegistry<C>,
) -> Result<CompositionDocument, PreviewError> {
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err(PreviewError::Invalid(
            "composition document exceeds size limit".into(),
        ));
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RawDocument<'a> {
        format: String,
        format_version: u32,
        preview_id: PreviewId,
        root: NodeId,
        #[serde(borrow, deserialize_with = "deserialize_unique_map")]
        nodes: BTreeMap<NodeId, RawNode<'a>>,
    }
    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct RawNode<'a> {
        #[serde(borrow)]
        preview: &'a serde_json::value::RawValue,
        #[serde(default, deserialize_with = "deserialize_unique_map")]
        slots: BTreeMap<SlotId, Vec<NodeId>>,
    }
    let raw: RawDocument<'_> = serde_json::from_str(source)?;
    let mut nodes = BTreeMap::new();
    for (id, raw_node) in raw.nodes {
        let preview = super::codec::decode_preview(raw_node.preview.get(), registry.catalog())?;
        nodes.insert(
            id,
            CompositionNode {
                preview: preview.document,
                slots: raw_node.slots,
            },
        );
    }
    let d = CompositionDocument {
        format: raw.format,
        format_version: raw.format_version,
        preview_id: raw.preview_id,
        root: raw.root,
        nodes,
    };
    validate_composition(registry, &d)?;
    Ok(d)
}

/// Ordered child descriptions grouped by slot name, transferred into the compiled factory.
pub type CompositionSlots = BTreeMap<SlotId, Vec<AnyView>>;
#[derive(Clone, Debug, PartialEq)]
pub struct CompositionNodeSnapshot {
    pub id: NodeId,
    pub snapshot: PreviewSnapshot,
}

/// A local signal bound to one node and the exact mounted composition revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CompositionSignal {
    pub stamp: CompositionStamp,
    pub node: NodeId,
    pub event: EventId,
}
impl CompositionNodeSnapshot {
    pub fn signal(&self, event: EventId) -> CompositionSignal {
        CompositionSignal {
            stamp: CompositionStamp {
                preview_id: self.snapshot.stamp.preview_id.clone(),
                generation: self.snapshot.stamp.generation,
                revision: self.snapshot.stamp.revision,
            },
            node: self.id.clone(),
            event,
        }
    }
}

pub struct CompositionHost<C> {
    registry: Rc<PreviewRegistry<C>>,
    mounted: Option<(CompositionStamp, CompositionDocument)>,
    generation: u64,
}

fn node_identity(stamp: &CompositionStamp, id: &NodeId) -> String {
    // IDs may contain hyphens; length-prefix the preview ID to keep scopes unique.
    format!(
        "composition-node-{}:{}-{}-{}",
        stamp.preview_id.as_str().len(),
        stamp.preview_id,
        stamp.generation,
        id
    )
}
impl<C> CompositionHost<C> {
    pub fn new(registry: PreviewRegistry<C>) -> Self {
        Self {
            registry: Rc::new(registry),
            mounted: None,
            generation: 0,
        }
    }
    /// Registered schemas used by this compiled composition host.
    pub fn catalog(&self) -> &super::ComponentCatalog {
        self.registry.catalog()
    }

    /// Registered composition slot contracts; no component state is exposed.
    pub fn composition_slots(&self) -> &BTreeMap<ComponentId, BTreeMap<SlotId, SlotSchema>> {
        self.registry.composition_slots()
    }

    pub fn mount(&mut self, d: CompositionDocument) -> Result<CompositionStamp, PreviewError> {
        validate_composition(&self.registry, &d)?;
        let generation = self
            .generation
            .checked_add(1)
            .ok_or_else(|| PreviewError::Invalid("composition generation exhausted".into()))?;
        let stamp = CompositionStamp {
            preview_id: d.preview_id.clone(),
            generation,
            revision: 0,
        };
        self.generation = generation;
        self.mounted = Some((stamp.clone(), d));
        Ok(stamp)
    }
    pub fn document(&self) -> Option<&CompositionDocument> {
        self.mounted.as_ref().map(|(_, d)| d)
    }
    /// Exact revision of the current mounted composition, if any.
    pub fn stamp(&self) -> Option<&CompositionStamp> {
        self.mounted.as_ref().map(|(stamp, _)| stamp)
    }

    pub fn unmount(&mut self) {
        self.mounted = None;
    }

    /// Encodes the current validated document; an unmounted host has no document.
    pub fn encode(&self) -> Result<Option<String>, PreviewError> {
        self.document()
            .map(|document| encode_composition(document, &self.registry))
            .transpose()
    }

    /// Decodes before mounting, preserving the current document on rejection.
    pub fn mount_source(&mut self, source: &str) -> Result<CompositionStamp, PreviewError> {
        let document = decode_composition(source, &self.registry)?;
        self.mount(document)
    }

    pub fn snapshot(&self, id: &NodeId) -> Result<Option<CompositionNodeSnapshot>, PreviewError> {
        let Some((stamp, document)) = &self.mounted else {
            return Ok(None);
        };
        let node = document
            .nodes
            .get(id)
            .ok_or_else(|| PreviewError::Invalid("composition node missing".into()))?;
        Ok(Some(CompositionNodeSnapshot {
            id: id.clone(),
            snapshot: PreviewSnapshot {
                stamp: PreviewStamp {
                    preview_id: stamp.preview_id.clone(),
                    generation: stamp.generation,
                    revision: stamp.revision,
                },
                preview: validate_preview(self.registry.catalog(), node.preview.clone())?,
            },
        }))
    }

    /// Returns an allowlisted local target; the owner invokes its compiled handler.
    pub fn dispatch(&self, signal: &CompositionSignal) -> Result<ActionRef, PreviewError> {
        let (_, document) = self
            .mounted
            .as_ref()
            .filter(|(stamp, _)| stamp == &signal.stamp)
            .ok_or_else(|| PreviewError::Invalid("stale or unmounted composition signal".into()))?;
        document
            .nodes
            .get(&signal.node)
            .and_then(|node| node.preview.actions.get(&signal.event))
            .cloned()
            .ok_or_else(|| {
                PreviewError::Invalid("composition event has no registered action target".into())
            })
    }
    pub fn edit(
        &mut self,
        stamp: &CompositionStamp,
        edits: &[CompositionEdit],
    ) -> Result<CompositionStamp, PreviewError> {
        if edits.len() > MAX_COMPOSITION_EDITS {
            return Err(PreviewError::Invalid(
                "composition edit batch exceeds limit".into(),
            ));
        }
        let (current, doc) = self
            .mounted
            .as_ref()
            .filter(|(s, _)| s == stamp)
            .ok_or_else(|| {
                PreviewError::Invalid("stale or unmounted composition revision".into())
            })?;
        let mut next = doc.clone();
        for op in edits {
            match op {
                CompositionEdit::SetProperty {
                    node,
                    property,
                    value,
                } => {
                    let current = &next
                        .nodes
                        .get(node)
                        .ok_or_else(|| PreviewError::Invalid("property target missing".into()))?
                        .preview;
                    let validated = validate_preview(self.registry.catalog(), current.clone())?;
                    let updated = validated.apply_batch(
                        self.registry.catalog(),
                        [(property.clone(), value.clone())],
                    )?;
                    next.nodes
                        .get_mut(node)
                        .ok_or_else(|| PreviewError::Invalid("property target missing".into()))?
                        .preview = updated.document;
                }
                CompositionEdit::Insert {
                    parent,
                    slot,
                    node,
                    value,
                } => {
                    if next.nodes.len() >= MAX_COMPOSITION_NODES {
                        return Err(PreviewError::Invalid(
                            "composition node limit exceeded".into(),
                        ));
                    }
                    if next.nodes.contains_key(node) {
                        return Err(PreviewError::Invalid("node id already exists".into()));
                    }
                    next.nodes
                        .get_mut(parent)
                        .ok_or_else(|| PreviewError::Invalid("insert parent missing".into()))?
                        .slots
                        .entry(slot.clone())
                        .or_default()
                        .push(node.clone());
                    next.nodes.insert(node.clone(), value.clone());
                }
                CompositionEdit::RemoveSubtree { node } => {
                    if node == &next.root {
                        return Err(PreviewError::Invalid(
                            "cannot remove composition root".into(),
                        ));
                    }
                    let mut todo = vec![node.clone()];
                    let mut remove = Vec::new();
                    let mut visited = BTreeSet::new();
                    while let Some(id) = todo.pop() {
                        if !visited.insert(id.clone()) {
                            return Err(PreviewError::Invalid(
                                "cannot remove a cyclic composition subtree".into(),
                            ));
                        }
                        if visited.len() > MAX_COMPOSITION_NODES {
                            return Err(PreviewError::Invalid(
                                "composition subtree exceeds node limit".into(),
                            ));
                        }
                        let n = next
                            .nodes
                            .get(&id)
                            .ok_or_else(|| PreviewError::Invalid("remove target missing".into()))?;
                        todo.extend(n.slots.values().flatten().cloned());
                        remove.push(id);
                    }
                    for n in next.nodes.values_mut() {
                        for children in n.slots.values_mut() {
                            children.retain(|id| id != node);
                        }
                    }
                    for id in remove {
                        next.nodes.remove(&id);
                    }
                }
                CompositionEdit::Move { node, parent, slot } => {
                    if node == &next.root || !next.nodes.contains_key(node) {
                        return Err(PreviewError::Invalid(
                            "cannot move root or missing node".into(),
                        ));
                    }
                    for n in next.nodes.values_mut() {
                        for children in n.slots.values_mut() {
                            children.retain(|id| id != node);
                        }
                    }
                    next.nodes
                        .get_mut(parent)
                        .ok_or_else(|| PreviewError::Invalid("move parent missing".into()))?
                        .slots
                        .entry(slot.clone())
                        .or_default()
                        .push(node.clone());
                }
            }
        }
        validate_composition(&self.registry, &next)?;
        if *doc == next {
            return Ok(current.clone());
        }
        let revision = current
            .revision
            .checked_add(1)
            .ok_or_else(|| PreviewError::Invalid("composition revision exhausted".into()))?;
        let updated = CompositionStamp {
            revision,
            ..current.clone()
        };
        self.mounted = Some((updated.clone(), next));
        Ok(updated)
    }
    pub fn render(&self, context: &C) -> Result<Option<AnyView>, PreviewError> {
        let Some((stamp, doc)) = &self.mounted else {
            return Ok(None);
        };
        // Recheck the invariant before entering recursive view construction.
        validate_composition(&self.registry, doc)?;
        fn render_node<C>(
            registry: &PreviewRegistry<C>,
            d: &CompositionDocument,
            stamp: &CompositionStamp,
            id: &NodeId,
            context: &C,
            depth: usize,
        ) -> Result<AnyView, PreviewError> {
            if depth > MAX_COMPOSITION_DEPTH {
                return Err(PreviewError::Invalid(
                    "composition exceeds maximum render depth".into(),
                ));
            }
            let node = &d.nodes[id];
            let mut slots = CompositionSlots::new();
            for (slot, children) in &node.slots {
                let mut rendered = Vec::with_capacity(children.len());
                for child in children {
                    rendered.push(render_node(registry, d, stamp, child, context, depth + 1)?);
                }
                slots.insert(slot.clone(), rendered);
            }
            let validated = validate_preview(registry.catalog(), node.preview.clone())?;
            let snapshot = PreviewSnapshot {
                stamp: PreviewStamp {
                    preview_id: stamp.preview_id.clone(),
                    generation: stamp.generation,
                    revision: stamp.revision,
                },
                preview: validated,
            };
            let view = registry.render_composition(
                &CompositionNodeSnapshot {
                    id: id.clone(),
                    snapshot,
                },
                context,
                slots,
            )?;
            Ok(AnyView::new(Keyed(node_identity(stamp, id), view)))
        }
        Ok(Some(render_node(
            &self.registry,
            doc,
            stamp,
            &doc.root,
            context,
            1,
        )?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        preview::*,
        view::{AnyView, EmptyView},
    };
    use std::collections::BTreeMap;

    #[test]
    fn validates_later_node_properties_after_catalog_reuse() {
        let (registry, mut document) = fixture();
        let root = document.nodes.get_mut(&document.root).unwrap();
        root.preview.properties.insert(
            PropertyId::new("unexpected").unwrap(),
            PropertyValue::Boolean(true),
        );
        assert!(matches!(
            validate_composition(&registry, &document),
            Err(PreviewError::UnknownProperty { .. })
        ));
    }
    #[test]
    fn consumer_metadata_preserves_registered_catalog_and_slot_contracts() {
        let (registry, document) = fixture();
        let catalog = registry.catalog().clone();
        let slots = registry.composition_slots().clone();
        assert!(!slots.is_empty());
        let mut host = CompositionHost::new(registry);
        assert_eq!(host.catalog(), &catalog);
        assert_eq!(host.composition_slots(), &slots);
        host.mount(document).unwrap();
        host.unmount();
        assert_eq!(host.catalog(), &catalog);
        assert_eq!(host.composition_slots(), &slots);
    }

    fn fixture() -> (PreviewRegistry<()>, CompositionDocument) {
        let mut r = PreviewRegistry::new();
        let container = ComponentId::new("container").unwrap();
        let leaf = ComponentId::new("leaf").unwrap();
        let schema = |id: ComponentId| ComponentSchema {
            id,
            version: 1,
            display_name: "Specimen".into(),
            properties: BTreeMap::new(),
            events: BTreeMap::new(),
        };
        r.register_composable(
            schema(container.clone()),
            BTreeMap::from([(
                SlotId::new("body").unwrap(),
                SlotSchema {
                    allowed_children: BTreeSet::from([leaf.clone()]),
                    min_children: 1,
                    max_children: 2,
                },
            )]),
            |_, _, mut children| {
                let children = children.remove(&SlotId::new("body").unwrap()).unwrap();
                assert_eq!(children.len(), 1);
                let mut stack = crate::prelude::VStack::new();
                for child in children {
                    stack = stack.child(child);
                }
                AnyView::new(stack)
            },
        )
        .unwrap();
        r.register_composable(schema(leaf.clone()), BTreeMap::new(), |_, _, _| {
            AnyView::new(EmptyView)
        })
        .unwrap();
        let pid = PreviewId::new("sample").unwrap();
        let node = |component_id| CompositionNode {
            preview: PreviewDocument::new(pid.clone(), component_id, 1),
            slots: BTreeMap::new(),
        };
        let root = NodeId::new("root").unwrap();
        let child = NodeId::new("child").unwrap();
        let mut root_node = node(container);
        root_node
            .slots
            .insert(SlotId::new("body").unwrap(), vec![child.clone()]);
        (
            r,
            CompositionDocument {
                format: COMPOSITION_FORMAT.into(),
                format_version: 1,
                preview_id: pid.clone(),
                root: root.clone(),
                nodes: BTreeMap::from([(root, root_node), (child, node(leaf))]),
            },
        )
    }
    #[test]
    fn recursive_composable_render_and_codec_keep_leaf_api_separate() {
        let (r, d) = fixture();
        let encoded = encode_composition(&d, &r).unwrap();
        assert_eq!(decode_composition(&encoded, &r).unwrap(), d);
        let mut host = CompositionHost::new(r);
        assert_eq!(host.mount(d).unwrap().revision, 0);
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn composition_decoder_rejects_unknown_document_and_node_fields() {
        let (registry, document) = fixture();
        let encoded = encode_composition(&document, &registry).unwrap();
        let original: serde_json::Value = serde_json::from_str(&encoded).unwrap();

        let mut extra_document = original.clone();
        extra_document["unexpected"] = serde_json::json!(true);
        assert!(decode_composition(&extra_document.to_string(), &registry).is_err());

        let mut extra_node = original;
        extra_node["nodes"][document.root.as_str()]["unexpected"] = serde_json::json!(true);
        assert!(decode_composition(&extra_node.to_string(), &registry).is_err());
    }
    #[test]
    fn stale_and_invalid_edits_are_atomic() {
        let (r, d) = fixture();
        let mut h = CompositionHost::new(r);
        let s = h.mount(d).unwrap();
        assert_eq!(h.edit(&s, &[]).unwrap(), s);
        let bad = CompositionEdit::Move {
            node: NodeId::new("child").unwrap(),
            parent: NodeId::new("child").unwrap(),
            slot: SlotId::new("body").unwrap(),
        };
        let remove_cycle = CompositionEdit::RemoveSubtree {
            node: NodeId::new("child").unwrap(),
        };
        assert!(h.edit(&s, &[bad, remove_cycle]).is_err());
        assert_eq!(h.document().unwrap().nodes.len(), 2);
        assert_eq!(
            h.edit(
                &CompositionStamp {
                    revision: 1,
                    ..s.clone()
                },
                &[]
            )
            .is_err(),
            true
        );
    }

    #[test]
    fn leaf_host_rejects_a_factory_with_required_child_slots() {
        let (registry, document) = fixture();
        let mut host = super::super::PreviewHost::new(registry);
        host.mount(document.nodes[&document.root].preview.clone())
            .unwrap();
        assert!(host.render(&()).is_err());
    }

    #[test]
    fn signals_are_node_scoped_and_rejected_after_edit_unmount_or_remount() {
        use super::super::EventSchema;
        let mut registry = PreviewRegistry::<()>::new();
        let component = ComponentId::new("action-item").unwrap();
        let event = EventId::new("activate").unwrap();
        let slot = SlotId::new("children").unwrap();
        registry
            .register_composable(
                ComponentSchema {
                    id: component.clone(),
                    version: 1,
                    display_name: "Action item".into(),
                    properties: BTreeMap::new(),
                    events: BTreeMap::from([(
                        event.clone(),
                        EventSchema {
                            display_name: "Activate".into(),
                            payload: BTreeMap::new(),
                        },
                    )]),
                },
                BTreeMap::from([(
                    slot.clone(),
                    SlotSchema {
                        allowed_children: BTreeSet::from([component.clone()]),
                        min_children: 0,
                        max_children: 1,
                    },
                )]),
                |_, _, children| {
                    let mut stack = crate::prelude::VStack::new();
                    for child in children.into_values().flatten() {
                        stack = stack.child(child);
                    }
                    AnyView::new(stack)
                },
            )
            .unwrap();
        let preview_id = PreviewId::new("node-actions").unwrap();
        let root = NodeId::new("root").unwrap();
        let child = NodeId::new("child").unwrap();
        let mut nodes = BTreeMap::new();
        for id in [&root, &child] {
            let action = ActionRef::new(format!("{}-action", id)).unwrap();
            registry.register_action(action.clone()).unwrap();
            let mut preview = PreviewDocument::new(preview_id.clone(), component.clone(), 1);
            preview.actions.insert(event.clone(), action);
            nodes.insert(
                id.clone(),
                CompositionNode {
                    preview,
                    slots: BTreeMap::new(),
                },
            );
        }
        nodes
            .get_mut(&root)
            .unwrap()
            .slots
            .insert(slot, vec![child.clone()]);
        let document = CompositionDocument {
            format: COMPOSITION_FORMAT.into(),
            format_version: COMPOSITION_FORMAT_VERSION,
            preview_id,
            root: root.clone(),
            nodes,
        };
        let mut host = CompositionHost::new(registry);
        let stamp = host.mount(document.clone()).unwrap();
        let encoded = host.encode().unwrap().unwrap();
        assert!(host.mount_source("invalid").is_err());
        assert_eq!(host.document(), Some(&document));
        let root_signal = host.snapshot(&root).unwrap().unwrap().signal(event.clone());
        let child_signal = host.snapshot(&child).unwrap().unwrap().signal(event);
        assert_eq!(host.dispatch(&root_signal).unwrap().as_str(), "root-action");
        assert_eq!(
            host.dispatch(&child_signal).unwrap().as_str(),
            "child-action"
        );
        assert!(host.render(&()).unwrap().is_some());
        let next = host
            .edit(&stamp, &[CompositionEdit::RemoveSubtree { node: child }])
            .unwrap();
        assert!(next.revision > stamp.revision);
        assert!(host.dispatch(&root_signal).is_err());
        assert!(host.dispatch(&child_signal).is_err());
        let current = host
            .snapshot(&root)
            .unwrap()
            .unwrap()
            .signal(EventId::new("activate").unwrap());
        host.unmount();
        assert!(host.dispatch(&current).is_err());
        assert!(host.render(&()).unwrap().is_none());
        host.mount_source(&encoded).unwrap();
        assert_eq!(host.document(), Some(&document));
        assert!(host.dispatch(&root_signal).is_err());
    }
    #[test]
    fn unsupported_versions_and_invalid_slots_reject() {
        let (r, mut d) = fixture();
        d.format_version = 2;
        assert!(encode_composition(&d, &r).is_err());
        d.format_version = 1;
        d.nodes
            .get_mut(&d.root)
            .unwrap()
            .slots
            .insert(SlotId::new("unknown").unwrap(), vec![]);
        assert!(encode_composition(&d, &r).is_err());
    }

    #[test]
    fn byte_and_node_count_limits_reject_before_recursive_work() {
        let (r, _) = fixture();
        let oversized = " ".repeat(super::super::model::MAX_DOCUMENT_BYTES + 1);
        assert!(decode_composition(&oversized, &r).is_err());

        let mut d = fixture().1;
        let root = d.root.clone();
        let node = d.nodes[&root].clone();
        for i in 0..=MAX_COMPOSITION_NODES {
            let id = NodeId::new(format!("n{i}")).unwrap();
            d.nodes.insert(id, node.clone());
        }
        assert!(validate_shape(&d).is_err());
    }

    #[test]
    fn depth_is_bounded_and_render_recurses_for_valid_nested_tree() {
        let mut registry: PreviewRegistry<()> = PreviewRegistry::new();
        let component = ComponentId::new("nest").unwrap();
        registry
            .register_composable(
                ComponentSchema {
                    id: component.clone(),
                    version: 1,
                    display_name: "Nest".into(),
                    properties: BTreeMap::new(),
                    events: BTreeMap::new(),
                },
                BTreeMap::from([(
                    SlotId::new("next").unwrap(),
                    SlotSchema {
                        allowed_children: BTreeSet::from([component.clone()]),
                        min_children: 0,
                        max_children: 1,
                    },
                )]),
                |_, _, _| AnyView::new(EmptyView),
            )
            .unwrap();
        let pid = PreviewId::new("deep-tree").unwrap();
        let ids: Vec<_> = (0..MAX_COMPOSITION_DEPTH + 1)
            .map(|i| NodeId::new(format!("n{i}")).unwrap())
            .collect();
        let mut nodes = BTreeMap::new();
        for (index, id) in ids.iter().enumerate() {
            let mut node = CompositionNode {
                preview: PreviewDocument::new(pid.clone(), component.clone(), 1),
                slots: BTreeMap::new(),
            };
            if let Some(child) = ids.get(index + 1) {
                node.slots
                    .insert(SlotId::new("next").unwrap(), vec![child.clone()]);
            }
            nodes.insert(id.clone(), node);
        }
        let too_deep = CompositionDocument {
            format: COMPOSITION_FORMAT.into(),
            format_version: 1,
            preview_id: pid.clone(),
            root: ids[0].clone(),
            nodes,
        };
        assert!(validate_composition(&registry, &too_deep).is_err());

        let mut shallow = too_deep;
        let last = ids[MAX_COMPOSITION_DEPTH - 1].clone();
        shallow.nodes.remove(&ids[MAX_COMPOSITION_DEPTH]);
        shallow.nodes.get_mut(&last).unwrap().slots.clear();
        let mut host = CompositionHost::new(registry);
        host.mount(shallow).unwrap();
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn node_action_must_be_allowlisted_and_slot_contracts_are_validated() {
        let mut registry: PreviewRegistry<()> = PreviewRegistry::new();
        let component = ComponentId::new("action-node").unwrap();
        let event = EventId::new("activate").unwrap();
        registry
            .register_composable(
                ComponentSchema {
                    id: component.clone(),
                    version: 1,
                    display_name: "Action".into(),
                    properties: BTreeMap::new(),
                    events: BTreeMap::from([(
                        event.clone(),
                        EventSchema {
                            display_name: "Activate".into(),
                            payload: BTreeMap::new(),
                        },
                    )]),
                },
                BTreeMap::new(),
                |_, _, _| AnyView::new(EmptyView),
            )
            .unwrap();
        let pid = PreviewId::new("action-tree").unwrap();
        let id = NodeId::new("root").unwrap();
        let mut preview = PreviewDocument::new(pid.clone(), component, 1);
        preview
            .actions
            .insert(event, ActionRef::new("unapproved").unwrap());
        let doc = CompositionDocument {
            format: COMPOSITION_FORMAT.into(),
            format_version: 1,
            preview_id: pid.clone(),
            root: id.clone(),
            nodes: BTreeMap::from([(
                id,
                CompositionNode {
                    preview,
                    slots: BTreeMap::new(),
                },
            )]),
        };
        assert!(validate_composition(&registry, &doc).is_err());
        registry
            .register_action(ActionRef::new("unapproved").unwrap())
            .unwrap();
        assert!(validate_composition(&registry, &doc).is_ok());
    }

    #[test]
    fn slot_registration_rejects_invalid_ids_and_cardinality_atomically() {
        let mut registry: PreviewRegistry<()> = PreviewRegistry::new();
        let invalid_slot: SlotId = serde_json::from_str("\"BadSlot\"").unwrap();
        let schema = ComponentSchema {
            id: ComponentId::new("invalid-slots").unwrap(),
            version: 1,
            display_name: "Invalid slots".into(),
            properties: BTreeMap::new(),
            events: BTreeMap::new(),
        };
        let bad_id = BTreeMap::from([(
            invalid_slot,
            SlotSchema {
                allowed_children: BTreeSet::new(),
                min_children: 0,
                max_children: 1,
            },
        )]);
        assert!(
            registry
                .register_composable(schema.clone(), bad_id, |_, _, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let invalid_bounds = BTreeMap::from([(
            SlotId::new("body").unwrap(),
            SlotSchema {
                allowed_children: BTreeSet::new(),
                min_children: 2,
                max_children: 1,
            },
        )]);
        assert!(
            registry
                .register_composable(schema, invalid_bounds, |_, _, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());
    }

    #[test]
    fn slot_minimum_and_maximum_cardinality_are_enforced() {
        let (registry, mut document) = fixture();
        let root = document.root.clone();
        let body = SlotId::new("body").unwrap();
        document.nodes.get_mut(&root).unwrap().slots.remove(&body);
        document.nodes.remove(&NodeId::new("child").unwrap());
        assert!(validate_composition(&registry, &document).is_err());

        let (registry, mut document) = fixture();
        let root = document.root.clone();
        let child_template = document.nodes[&NodeId::new("child").unwrap()].clone();
        document.nodes.remove(&NodeId::new("child").unwrap());
        let children: Vec<_> = (0..3)
            .map(|i| NodeId::new(format!("child-{i}")).unwrap())
            .collect();
        for id in &children {
            document.nodes.insert(id.clone(), child_template.clone());
        }
        document
            .nodes
            .get_mut(&root)
            .unwrap()
            .slots
            .insert(SlotId::new("body").unwrap(), children);
        assert!(validate_composition(&registry, &document).is_err());
    }
}
#[test]
fn node_identity_separates_documents_and_mounts_but_survives_edits() {
    let a = CompositionStamp {
        preview_id: PreviewId::new("a-1").unwrap(),
        generation: 1,
        revision: 0,
    };
    let b = CompositionStamp {
        preview_id: PreviewId::new("a").unwrap(),
        ..a.clone()
    };
    let x = NodeId::new("x").unwrap();
    let other_x = NodeId::new("1-x").unwrap();
    assert_ne!(node_identity(&a, &x), node_identity(&b, &other_x));
    assert_eq!(
        node_identity(&a, &x),
        node_identity(
            &CompositionStamp {
                revision: 1,
                ..a.clone()
            },
            &x
        )
    );
    assert_ne!(
        node_identity(&a, &x),
        node_identity(
            &CompositionStamp {
                generation: 2,
                ..a.clone()
            },
            &x
        )
    );
}
