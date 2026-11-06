//! Explicit registration of ordinary compiled factories. No code is deserialized.
use std::{collections::BTreeMap, rc::Rc};

use crate::view::AnyView;

use super::composition::{CompositionNodeSnapshot, CompositionSlots, NodeId, SlotId, SlotSchema};
use super::host::PreviewSnapshot;
use super::{ActionRef, ComponentCatalog, ComponentId, ComponentSchema, PreviewError};

type Factory<C> = dyn Fn(&PreviewSnapshot, &C) -> AnyView;
type ComposableFactory<C> = dyn Fn(&CompositionNodeSnapshot, &C, CompositionSlots) -> AnyView;

/// A local catalog paired with compiled Rust factories and typed owner context.
///
/// `C` may contain ordinary Rust actions and bindings. It is never serialized.
/// Register all entries before handing the registry to a host.
pub struct PreviewRegistry<C> {
    catalog: ComponentCatalog,
    factories: BTreeMap<ComponentId, Rc<Factory<C>>>,
    composable_factories: BTreeMap<ComponentId, Rc<ComposableFactory<C>>>,
    slot_contracts: BTreeMap<ComponentId, BTreeMap<SlotId, SlotSchema>>,
}

impl<C> Default for PreviewRegistry<C> {
    fn default() -> Self {
        Self {
            catalog: ComponentCatalog::new(),
            factories: BTreeMap::new(),
            composable_factories: BTreeMap::new(),
            slot_contracts: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::PreviewRegistry;
    use crate::{
        preview::*,
        view::{AnyView, EmptyView},
    };
    use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

    fn schema() -> ComponentSchema {
        ComponentSchema {
            id: ComponentId::new("registry-atomicity").unwrap(),
            version: 1,
            display_name: "Registry atomicity".into(),
            properties: BTreeMap::new(),
            events: BTreeMap::new(),
        }
    }

    #[test]
    fn duplicate_registration_keeps_the_original_factory() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let original_calls = calls.clone();
        let mut registry = PreviewRegistry::new();
        registry
            .register(schema(), move |_, _| {
                original_calls.borrow_mut().push("original");
                AnyView::new(EmptyView)
            })
            .unwrap();

        let replacement_calls = calls.clone();
        assert!(
            registry
                .register(schema(), move |_, _| {
                    replacement_calls.borrow_mut().push("replacement");
                    AnyView::new(EmptyView)
                })
                .is_err()
        );
        assert_eq!(registry.catalog().components.len(), 1);

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert!(host.render(&()).unwrap().is_some());
        assert_eq!(&*calls.borrow(), &["original"]);
    }

    #[test]
    fn impossible_i64_minimum_leaves_registry_available_for_maximum_boundary() {
        let value_id = PropertyId::new("value").unwrap();
        let mut registry = PreviewRegistry::new();
        let mut impossible = schema();
        impossible.properties.insert(
            value_id.clone(),
            PropertySchema {
                kind: PropertyKind::Integer,
                default: None,
                required: false,
                editable: true,
                minimum: Some(2_f64.powi(63)),
                maximum: None,
                min_length: None,
                max_length: None,
                allowed_values: vec![],
            },
        );
        assert!(
            registry
                .register(impossible, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut representable = schema();
        representable.properties.insert(
            value_id.clone(),
            PropertySchema {
                kind: PropertyKind::Integer,
                default: Some(PropertyValue::Integer(i64::MAX)),
                required: false,
                editable: true,
                minimum: None,
                maximum: Some(2_f64.powi(63)),
                min_length: None,
                max_length: None,
                allowed_values: vec![],
            },
        );
        registry
            .register(representable, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&value_id],
            PropertyValue::Integer(i64::MAX)
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn out_of_range_integer_default_keeps_registry_available_for_corrected_schema() {
        let property_id = PropertyId::new("count").unwrap();
        let integer_property = |default| PropertySchema {
            kind: PropertyKind::Integer,
            default: Some(PropertyValue::Integer(default)),
            required: false,
            editable: true,
            minimum: None,
            maximum: Some(4.0),
            min_length: None,
            max_length: None,
            allowed_values: vec![],
        };
        let mut registry = PreviewRegistry::new();
        let mut invalid = schema();
        invalid
            .properties
            .insert(property_id.clone(), integer_property(5));
        assert!(
            registry
                .register(invalid, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut corrected = schema();
        corrected
            .properties
            .insert(property_id.clone(), integer_property(4));
        registry
            .register(corrected, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&property_id],
            PropertyValue::Integer(4)
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn out_of_range_number_default_keeps_registry_available_for_corrected_schema() {
        let property_id = PropertyId::new("amount").unwrap();
        let number_property = |default| PropertySchema {
            kind: PropertyKind::Number,
            default: Some(PropertyValue::Number(default)),
            required: false,
            editable: true,
            minimum: None,
            maximum: Some(1.0),
            min_length: None,
            max_length: None,
            allowed_values: vec![],
        };
        let mut registry = PreviewRegistry::new();
        let mut invalid = schema();
        invalid
            .properties
            .insert(property_id.clone(), number_property(1.1));
        assert!(
            registry
                .register(invalid, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut corrected = schema();
        corrected
            .properties
            .insert(property_id.clone(), number_property(1.0));
        registry
            .register(corrected, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&property_id],
            PropertyValue::Number(1.0)
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn overlong_string_default_keeps_registry_available_for_corrected_schema() {
        let property_id = PropertyId::new("label").unwrap();
        let string_property = |default: &str| PropertySchema {
            kind: PropertyKind::String,
            default: Some(PropertyValue::String(default.into())),
            required: false,
            editable: true,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: Some(3),
            allowed_values: vec![],
        };
        let mut registry = PreviewRegistry::new();
        let mut invalid = schema();
        invalid
            .properties
            .insert(property_id.clone(), string_property("long"));
        assert!(
            registry
                .register(invalid, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut corrected = schema();
        corrected
            .properties
            .insert(property_id.clone(), string_property("run"));
        registry
            .register(corrected, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&property_id],
            PropertyValue::String("run".into())
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn overlong_unicode_string_default_keeps_registry_available_for_corrected_schema() {
        let property_id = PropertyId::new("label").unwrap();
        let string_property = |default: &str| PropertySchema {
            kind: PropertyKind::String,
            default: Some(PropertyValue::String(default.into())),
            required: false,
            editable: true,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: Some(2),
            allowed_values: vec![],
        };
        let mut registry = PreviewRegistry::new();
        let mut invalid = schema();
        invalid
            .properties
            .insert(property_id.clone(), string_property("ééé"));
        assert!(
            registry
                .register(invalid, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut corrected = schema();
        corrected
            .properties
            .insert(property_id.clone(), string_property("éé"));
        registry
            .register(corrected, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&property_id],
            PropertyValue::String("éé".into())
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn string_default_outside_allowed_values_keeps_registry_available_for_corrected_schema() {
        let property_id = PropertyId::new("label").unwrap();
        let string_property = |default: &str| PropertySchema {
            kind: PropertyKind::String,
            default: Some(PropertyValue::String(default.into())),
            required: false,
            editable: true,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            allowed_values: vec!["run".into()],
        };
        let mut registry = PreviewRegistry::new();
        let mut invalid = schema();
        invalid
            .properties
            .insert(property_id.clone(), string_property("stop"));
        assert!(
            registry
                .register(invalid, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut corrected = schema();
        corrected
            .properties
            .insert(property_id.clone(), string_property("run"));
        registry
            .register(corrected, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&property_id],
            PropertyValue::String("run".into())
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn mismatched_string_default_keeps_registry_available_for_corrected_schema() {
        let property_id = PropertyId::new("label").unwrap();
        let string_property = |default| PropertySchema {
            kind: PropertyKind::String,
            default: Some(default),
            required: false,
            editable: true,
            minimum: None,
            maximum: None,
            min_length: None,
            max_length: None,
            allowed_values: vec![],
        };
        let mut registry = PreviewRegistry::new();
        let mut invalid = schema();
        invalid.properties.insert(
            property_id.clone(),
            string_property(PropertyValue::Boolean(true)),
        );
        assert!(
            registry
                .register(invalid, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut corrected = schema();
        corrected.properties.insert(
            property_id.clone(),
            string_property(PropertyValue::String("ready".into())),
        );
        registry
            .register(corrected, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&property_id],
            PropertyValue::String("ready".into())
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn short_string_default_keeps_registry_available_for_corrected_schema() {
        let property_id = PropertyId::new("label").unwrap();
        let string_property = |default: &str| PropertySchema {
            kind: PropertyKind::String,
            default: Some(PropertyValue::String(default.into())),
            required: false,
            editable: true,
            minimum: None,
            maximum: None,
            min_length: Some(2),
            max_length: None,
            allowed_values: vec![],
        };
        let mut registry = PreviewRegistry::new();
        let mut invalid = schema();
        invalid
            .properties
            .insert(property_id.clone(), string_property("é"));
        assert!(
            registry
                .register(invalid, |_, _| AnyView::new(EmptyView))
                .is_err()
        );
        assert!(registry.catalog().components.is_empty());

        let mut corrected = schema();
        corrected
            .properties
            .insert(property_id.clone(), string_property("éé"));
        registry
            .register(corrected, |_, _| AnyView::new(EmptyView))
            .unwrap();

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert_eq!(
            host.snapshot().unwrap().preview.properties[&property_id],
            PropertyValue::String("éé".into())
        );
        assert!(host.render(&()).unwrap().is_some());
    }

    #[test]
    fn payload_event_duplicate_keeps_the_original_schema_and_factory() {
        let calls = Rc::new(RefCell::new(Vec::new()));
        let original_calls = calls.clone();
        let mut registry = PreviewRegistry::new();
        registry
            .register(schema(), move |_, _| {
                original_calls.borrow_mut().push("original");
                AnyView::new(EmptyView)
            })
            .unwrap();

        let mut invalid_replacement = schema();
        invalid_replacement.display_name = "Invalid replacement".into();
        invalid_replacement.events.insert(
            EventId::new("activate").unwrap(),
            EventSchema {
                display_name: "Activate".into(),
                payload: BTreeMap::from([(
                    PropertyId::new("payload").unwrap(),
                    PropertySchema {
                        kind: PropertyKind::String,
                        default: None,
                        required: false,
                        editable: false,
                        minimum: None,
                        maximum: None,
                        min_length: None,
                        max_length: None,
                        allowed_values: vec![],
                    },
                )]),
            },
        );
        let replacement_calls = calls.clone();
        assert!(
            registry
                .register(invalid_replacement, move |_, _| {
                    replacement_calls.borrow_mut().push("replacement");
                    AnyView::new(EmptyView)
                })
                .is_err()
        );
        let component_id = ComponentId::new("registry-atomicity").unwrap();
        let registered_schema = registry.catalog().components.get(&component_id).unwrap();
        assert_eq!(registered_schema.display_name, "Registry atomicity");
        assert!(registered_schema.events.is_empty());

        let mut host = PreviewHost::new(registry);
        host.mount(PreviewDocument::new(
            PreviewId::new("registry-atomicity").unwrap(),
            ComponentId::new("registry-atomicity").unwrap(),
            1,
        ))
        .unwrap();
        assert!(host.render(&()).unwrap().is_some());
        assert_eq!(&*calls.borrow(), &["original"]);
    }
}

impl<C> PreviewRegistry<C> {
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a signal-only component and its compiled factory atomically.
    pub fn register(
        &mut self,
        schema: ComponentSchema,
        factory: impl Fn(&PreviewSnapshot, &C) -> AnyView + 'static,
    ) -> Result<(), PreviewError> {
        if schema
            .events
            .values()
            .any(|event| !event.payload.is_empty())
        {
            return Err(PreviewError::Invalid(
                "compiled preview events must be signal-only".into(),
            ));
        }
        let id = schema.id.clone();
        self.catalog.register_component(schema)?;
        self.factories.insert(id, Rc::new(factory));
        Ok(())
    }

    /// Registers a descriptor-first component with named child slots. This is
    /// additive: existing `register` leaf factories and `PreviewHost` are intact.
    pub fn register_composable(
        &mut self,
        schema: ComponentSchema,
        slots: BTreeMap<SlotId, SlotSchema>,
        factory: impl Fn(&CompositionNodeSnapshot, &C, CompositionSlots) -> AnyView + 'static,
    ) -> Result<(), PreviewError> {
        let id = schema.id.clone();
        if slots.len() > super::composition::MAX_SLOTS_PER_COMPONENT {
            return Err(PreviewError::Invalid(
                "component exceeds composition slot limit".into(),
            ));
        }
        for (slot, contract) in &slots {
            SlotId::new(slot.as_str())?;
            if contract.min_children > contract.max_children
                || contract.max_children > super::composition::MAX_COMPOSITION_NODES
            {
                return Err(PreviewError::Invalid(
                    "invalid slot cardinality bounds".into(),
                ));
            }
            for child in &contract.allowed_children {
                ComponentId::new(child.as_str()).map_err(PreviewError::Invalid)?;
            }
        }
        // Reserve the ordinary registration path too, so duplicate component
        // IDs remain rejected by the existing atomic catalog insertion.
        let factory_for_leaf = Rc::new(factory);
        let leaf_factory = factory_for_leaf.clone();
        let leaf_node_id = NodeId::new(id.as_str())?;
        self.register(schema, move |snapshot, context| {
            leaf_factory(
                &CompositionNodeSnapshot {
                    id: leaf_node_id.clone(),
                    snapshot: snapshot.clone(),
                },
                context,
                CompositionSlots::new(),
            )
        })?;
        self.composable_factories
            .insert(id.clone(), factory_for_leaf);
        self.slot_contracts.insert(id, slots);
        Ok(())
    }

    /// Explicitly allowlists a local action target. Handlers stay with the owner.
    pub fn register_action(&mut self, action: ActionRef) -> Result<(), PreviewError> {
        self.catalog.register_action(action)
    }

    /// Read-only slot contracts for this registry's compiled composition factories.
    pub fn composition_slots(&self) -> &BTreeMap<ComponentId, BTreeMap<SlotId, SlotSchema>> {
        &self.slot_contracts
    }

    pub fn catalog(&self) -> &ComponentCatalog {
        &self.catalog
    }

    pub(super) fn slot_contract(&self, id: &ComponentId) -> Option<&BTreeMap<SlotId, SlotSchema>> {
        self.slot_contracts.get(id)
    }

    pub(super) fn render_composition(
        &self,
        snapshot: &CompositionNodeSnapshot,
        context: &C,
        slots: CompositionSlots,
    ) -> Result<AnyView, PreviewError> {
        let id = &snapshot.snapshot.preview.document.component_id;
        if let Some(factory) = self.composable_factories.get(id) {
            return Ok(factory(snapshot, context, slots));
        }
        if slots.is_empty() {
            return self.render(&snapshot.snapshot, context);
        }
        Err(PreviewError::Invalid(format!(
            "component `{id}` has child slots but no composable factory"
        )))
    }

    pub(super) fn render(
        &self,
        snapshot: &PreviewSnapshot,
        context: &C,
    ) -> Result<AnyView, PreviewError> {
        let id = &snapshot.preview.document.component_id;
        if self
            .slot_contracts
            .get(id)
            .is_some_and(|slots| slots.values().any(|slot| slot.min_children > 0))
        {
            return Err(PreviewError::Invalid(
                "component requires a composition document with child slots".into(),
            ));
        }
        let factory = self
            .factories
            .get(id)
            .ok_or_else(|| PreviewError::UnknownComponent(id.clone()))?;
        Ok(factory(snapshot, context))
    }
}
