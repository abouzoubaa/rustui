use std::collections::{BTreeMap, BTreeSet};

use crate::preview::model::{
    ActionRef, CATALOG_FORMAT, CATALOG_FORMAT_VERSION, ComponentCatalog, ComponentId,
    ComponentSchema, MAX_DISPLAY_NAME_BYTES, MAX_EVENTS_PER_COMPONENT,
    MAX_PROPERTIES_PER_COMPONENT, MAX_STRING_VALUE_BYTES, PREVIEW_FORMAT, PREVIEW_FORMAT_VERSION,
    PreviewDocument, PreviewError, PreviewId, PropertyId, PropertyKind, PropertySchema,
    PropertyValue, ValidatedPreview, is_valid_stable_id,
};

impl ComponentCatalog {
    pub fn new() -> Self {
        Self {
            format: CATALOG_FORMAT.to_owned(),
            format_version: CATALOG_FORMAT_VERSION,
            components: BTreeMap::new(),
            actions: Default::default(),
        }
    }

    /// Adds one compiled component's serializable descriptor to the catalog.
    pub fn register_component(&mut self, schema: ComponentSchema) -> Result<(), PreviewError> {
        validate_component_schema(&schema)?;
        if self.components.contains_key(&schema.id) {
            return Err(PreviewError::Invalid(format!(
                "component `{}` is already in the catalog",
                schema.id
            )));
        }
        self.components.insert(schema.id.clone(), schema);
        Ok(())
    }

    /// Declares an action reference that a later host may bind to local code.
    /// This records an identifier only; it never registers or executes a handler.
    pub fn register_action(&mut self, action: ActionRef) -> Result<(), PreviewError> {
        validate_id(action.as_str(), "action reference")?;
        if !self.actions.insert(action.clone()) {
            return Err(PreviewError::Invalid(format!(
                "action reference `{action}` is already in the catalog"
            )));
        }
        Ok(())
    }
}

impl Default for ComponentCatalog {
    fn default() -> Self {
        Self::new()
    }
}

impl PreviewDocument {
    pub fn new(preview_id: PreviewId, component_id: ComponentId, component_version: u32) -> Self {
        Self {
            format: PREVIEW_FORMAT.to_owned(),
            format_version: PREVIEW_FORMAT_VERSION,
            preview_id,
            component_id,
            component_version,
            properties: BTreeMap::new(),
            actions: BTreeMap::new(),
        }
    }
}

impl ValidatedPreview {
    /// Applies a complete proposed property batch atomically.
    ///
    /// The original value is immutable: on any error no partially updated
    /// preview can escape this call.
    pub fn apply_batch(
        &self,
        catalog: &ComponentCatalog,
        updates: impl IntoIterator<Item = (PropertyId, PropertyValue)>,
    ) -> Result<Self, PreviewError> {
        self.apply_batch_with(catalog, updates, |_| Ok(()))
    }

    /// Applies a batch and runs a pure component-specific constraint over the
    /// complete resolved property set before returning it.
    pub fn apply_batch_with(
        &self,
        catalog: &ComponentCatalog,
        updates: impl IntoIterator<Item = (PropertyId, PropertyValue)>,
        validate_properties: impl FnOnce(&BTreeMap<PropertyId, PropertyValue>) -> Result<(), String>,
    ) -> Result<Self, PreviewError> {
        let schema = catalog
            .components
            .get(&self.document.component_id)
            .ok_or_else(|| PreviewError::UnknownComponent(self.document.component_id.clone()))?;
        let mut document = self.document.clone();
        let mut changed = BTreeSet::new();
        for (property, value) in updates {
            if !changed.insert(property.clone()) {
                return Err(PreviewError::Invalid(format!(
                    "property `{property}` appears more than once in the update batch"
                )));
            }
            let property_schema =
                schema
                    .properties
                    .get(&property)
                    .ok_or_else(|| PreviewError::UnknownProperty {
                        component: schema.id.clone(),
                        property: property.clone(),
                    })?;
            if !property_schema.editable {
                return Err(PreviewError::NotEditable(property));
            }
            document.properties.insert(property, value);
        }
        let candidate = validate_preview(catalog, document)?;
        validate_properties(&candidate.properties).map_err(PreviewError::Invalid)?;
        Ok(candidate)
    }
}

pub fn validate_catalog(catalog: &ComponentCatalog) -> Result<(), PreviewError> {
    if catalog.format != CATALOG_FORMAT {
        return Err(PreviewError::UnsupportedCatalogFormat(
            catalog.format.clone(),
        ));
    }
    if catalog.format_version != CATALOG_FORMAT_VERSION {
        return Err(PreviewError::UnsupportedCatalogVersion(
            catalog.format_version,
        ));
    }
    if catalog.components.is_empty() {
        return Err(PreviewError::Invalid(
            "component catalog must register at least one component".to_owned(),
        ));
    }
    for action in &catalog.actions {
        validate_id(action.as_str(), "action reference")?;
    }
    for (id, schema) in &catalog.components {
        if id != &schema.id {
            return Err(PreviewError::Invalid(format!(
                "component catalog key `{id}` does not match descriptor id `{}`",
                schema.id
            )));
        }
        validate_component_schema(schema)?;
    }
    Ok(())
}

pub fn validate_preview(
    catalog: &ComponentCatalog,
    document: PreviewDocument,
) -> Result<ValidatedPreview, PreviewError> {
    validate_catalog(catalog)?;
    if document.format != PREVIEW_FORMAT {
        return Err(PreviewError::UnsupportedPreviewFormat(document.format));
    }
    if document.format_version != PREVIEW_FORMAT_VERSION {
        return Err(PreviewError::UnsupportedPreviewVersion(
            document.format_version,
        ));
    }
    validate_id(document.preview_id.as_str(), "preview id")?;
    validate_id(document.component_id.as_str(), "component id")?;
    if document.component_version == 0 {
        return Err(PreviewError::Invalid(
            "component version must be greater than zero".to_owned(),
        ));
    }

    let schema = catalog
        .components
        .get(&document.component_id)
        .ok_or_else(|| PreviewError::UnknownComponent(document.component_id.clone()))?;
    if document.component_version != schema.version {
        return Err(PreviewError::ComponentVersionMismatch {
            component: schema.id.clone(),
            expected: schema.version,
            found: document.component_version,
        });
    }

    for (property_id, _) in &document.properties {
        if !schema.properties.contains_key(property_id) {
            return Err(PreviewError::UnknownProperty {
                component: schema.id.clone(),
                property: property_id.clone(),
            });
        }
    }
    for (event_id, action) in &document.actions {
        if !schema.events.contains_key(event_id) {
            return Err(PreviewError::UnknownEvent {
                component: schema.id.clone(),
                event: event_id.clone(),
            });
        }
        validate_id(action.as_str(), "action reference")?;
        if !catalog.actions.contains(action) {
            return Err(PreviewError::UnresolvedAction(action.clone()));
        }
    }

    let mut properties = BTreeMap::new();
    for (property_id, property_schema) in &schema.properties {
        match document.properties.get(property_id) {
            Some(value) => {
                validate_property_value(property_id, property_schema, value)?;
                properties.insert(property_id.clone(), value.clone());
            }
            None if property_schema.required => {
                return Err(PreviewError::MissingRequiredProperty(property_id.clone()));
            }
            None => {
                if let Some(value) = &property_schema.default {
                    properties.insert(property_id.clone(), value.clone());
                }
            }
        }
    }

    Ok(ValidatedPreview {
        document,
        properties,
    })
}

fn validate_component_schema(schema: &ComponentSchema) -> Result<(), PreviewError> {
    validate_id(schema.id.as_str(), "component id")?;
    validate_display_name(&schema.display_name, "component")?;
    if schema.version == 0 {
        return Err(PreviewError::Invalid(format!(
            "component `{}` version must be greater than zero",
            schema.id
        )));
    }
    if schema.properties.len() > MAX_PROPERTIES_PER_COMPONENT {
        return Err(PreviewError::Invalid(format!(
            "component `{}` exceeds the property limit of {MAX_PROPERTIES_PER_COMPONENT}",
            schema.id
        )));
    }
    if schema.events.len() > MAX_EVENTS_PER_COMPONENT {
        return Err(PreviewError::Invalid(format!(
            "component `{}` exceeds the event limit of {MAX_EVENTS_PER_COMPONENT}",
            schema.id
        )));
    }
    for (id, property) in &schema.properties {
        validate_id(id.as_str(), "property id")?;
        validate_property_schema(id, property)?;
    }
    for (id, event) in &schema.events {
        validate_id(id.as_str(), "event id")?;
        validate_display_name(&event.display_name, "event")?;
        if event.payload.len() > MAX_PROPERTIES_PER_COMPONENT {
            return Err(PreviewError::Invalid(format!(
                "event `{id}` exceeds the payload property limit"
            )));
        }
        for (property_id, property) in &event.payload {
            validate_id(property_id.as_str(), "event payload property id")?;
            validate_property_schema(property_id, property)?;
        }
    }
    Ok(())
}

fn validate_property_schema(id: &PropertyId, schema: &PropertySchema) -> Result<(), PreviewError> {
    if schema.required && schema.default.is_some() {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` cannot be both required and have a default"
        )));
    }
    if schema.minimum.is_some_and(|value| !value.is_finite())
        || schema.maximum.is_some_and(|value| !value.is_finite())
    {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` numeric bounds must be finite"
        )));
    }
    if matches!((schema.minimum, schema.maximum), (Some(min), Some(max)) if min > max) {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` minimum exceeds maximum"
        )));
    }
    if schema.kind == PropertyKind::Integer {
        let first_allowed = schema
            .minimum
            .map_or(i64::MIN, |minimum| minimum.ceil() as i64);
        let has_no_value = schema
            .minimum
            .is_some_and(|minimum| integer_below_bound(i64::MAX, minimum))
            || schema
                .maximum
                .is_some_and(|maximum| integer_above_bound(i64::MIN, maximum))
            || schema
                .maximum
                .is_some_and(|maximum| integer_above_bound(first_allowed, maximum));
        if has_no_value {
            return Err(PreviewError::Invalid(format!(
                "property `{id}` integer bounds contain no representable value"
            )));
        }
    }
    if !matches!(schema.kind, PropertyKind::Integer | PropertyKind::Number)
        && (schema.minimum.is_some() || schema.maximum.is_some())
    {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` has numeric bounds but is not a number"
        )));
    }
    if schema.kind != PropertyKind::String
        && (schema.min_length.is_some()
            || schema.max_length.is_some()
            || !schema.allowed_values.is_empty())
    {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` has string constraints but is not a string"
        )));
    }
    if matches!((schema.min_length, schema.max_length), (Some(min), Some(max)) if min > max) {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` minimum length exceeds maximum"
        )));
    }
    if schema
        .min_length
        .is_some_and(|length| length > MAX_STRING_VALUE_BYTES)
        || schema
            .max_length
            .is_some_and(|length| length > MAX_STRING_VALUE_BYTES)
    {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` string length bound exceeds the {MAX_STRING_VALUE_BYTES} byte limit"
        )));
    }
    let mut allowed_values = schema.allowed_values.clone();
    allowed_values.sort();
    allowed_values.dedup();
    if allowed_values.len() != schema.allowed_values.len() {
        return Err(PreviewError::Invalid(format!(
            "property `{id}` has duplicate allowed values"
        )));
    }
    for value in &schema.allowed_values {
        if value.len() > MAX_STRING_VALUE_BYTES {
            return Err(PreviewError::Invalid(format!(
                "property `{id}` allowed value exceeds the string size limit"
            )));
        }
        let character_count = value.chars().count();
        if schema
            .min_length
            .is_some_and(|minimum| character_count < minimum)
            || schema
                .max_length
                .is_some_and(|maximum| character_count > maximum)
        {
            return Err(PreviewError::Invalid(format!(
                "property `{id}` allowed value violates its string length bounds"
            )));
        }
        if !schema.allowed_values.is_empty() {
            validate_id(value, "enumeration value")?;
        }
    }
    if let Some(default) = &schema.default {
        validate_property_value(id, schema, default)?;
    }
    Ok(())
}

fn validate_property_value(
    id: &PropertyId,
    schema: &PropertySchema,
    value: &PropertyValue,
) -> Result<(), PreviewError> {
    let invalid = |reason: &str| PreviewError::InvalidPropertyValue {
        property: id.clone(),
        reason: reason.to_owned(),
    };
    match (schema.kind, value) {
        (PropertyKind::String, PropertyValue::String(value)) => {
            if value.len() > MAX_STRING_VALUE_BYTES {
                return Err(invalid("string exceeds the 16384 byte limit"));
            }
            if schema
                .min_length
                .is_some_and(|min| value.chars().count() < min)
            {
                return Err(invalid("string is shorter than the minimum length"));
            }
            if schema
                .max_length
                .is_some_and(|max| value.chars().count() > max)
            {
                return Err(invalid("string is longer than the maximum length"));
            }
            if !schema.allowed_values.is_empty() && !schema.allowed_values.contains(value) {
                return Err(invalid("string is not one of the allowed values"));
            }
        }
        (PropertyKind::Boolean, PropertyValue::Boolean(_)) => {}
        (PropertyKind::Integer, PropertyValue::Integer(value)) => {
            if schema
                .minimum
                .is_some_and(|minimum| integer_below_bound(*value, minimum))
            {
                return Err(invalid("integer is below the minimum"));
            }
            if schema
                .maximum
                .is_some_and(|maximum| integer_above_bound(*value, maximum))
            {
                return Err(invalid("integer is above the maximum"));
            }
        }
        (PropertyKind::Number, PropertyValue::Number(value)) => {
            if !value.is_finite() {
                return Err(invalid("number must be finite"));
            }
            if schema.minimum.is_some_and(|minimum| *value < minimum) {
                return Err(invalid("number is below the minimum"));
            }
            if schema.maximum.is_some_and(|maximum| *value > maximum) {
                return Err(invalid("number is above the maximum"));
            }
        }
        _ => return Err(invalid("value type does not match the declared kind")),
    }
    Ok(())
}

fn integer_below_bound(value: i64, bound: f64) -> bool {
    if bound <= i64::MIN as f64 {
        false
    } else if bound >= 2_f64.powi(63) {
        true
    } else {
        value < bound.ceil() as i64
    }
}

fn integer_above_bound(value: i64, bound: f64) -> bool {
    if bound >= 2_f64.powi(63) {
        false
    } else if bound < i64::MIN as f64 {
        true
    } else {
        value > bound.floor() as i64
    }
}

fn validate_id(value: &str, label: &str) -> Result<(), PreviewError> {
    if is_valid_stable_id(value) {
        Ok(())
    } else {
        Err(PreviewError::Invalid(format!(
            "{label} `{value}` must use 1-128 lowercase ASCII letters, digits, dots, underscores, or hyphens and start with a letter or digit"
        )))
    }
}

fn validate_display_name(value: &str, label: &str) -> Result<(), PreviewError> {
    if value.trim().is_empty() || value.len() > MAX_DISPLAY_NAME_BYTES {
        return Err(PreviewError::Invalid(format!(
            "{label} display name must be non-empty and at most {MAX_DISPLAY_NAME_BYTES} bytes"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview::model::{EventId, EventSchema};

    fn id<T>(value: &str, make: impl FnOnce(String) -> Result<T, String>) -> T {
        make(value.to_owned()).unwrap()
    }

    fn catalog() -> ComponentCatalog {
        let mut catalog = ComponentCatalog::new();
        let mut properties = BTreeMap::new();
        properties.insert(
            id("label", PropertyId::new),
            PropertySchema {
                kind: PropertyKind::String,
                default: Some(PropertyValue::String("Run".to_owned())),
                required: false,
                editable: true,
                minimum: None,
                maximum: None,
                min_length: Some(1),
                max_length: Some(64),
                allowed_values: Vec::new(),
            },
        );
        properties.insert(
            id("disabled", PropertyId::new),
            PropertySchema {
                kind: PropertyKind::Boolean,
                default: Some(PropertyValue::Boolean(false)),
                required: false,
                editable: true,
                minimum: None,
                maximum: None,
                min_length: None,
                max_length: None,
                allowed_values: Vec::new(),
            },
        );
        let mut events = BTreeMap::new();
        events.insert(
            id("activate", EventId::new),
            EventSchema {
                display_name: "Activate".to_owned(),
                payload: BTreeMap::new(),
            },
        );
        catalog
            .register_component(ComponentSchema {
                id: id("rustui.button", ComponentId::new),
                version: 1,
                display_name: "Button".to_owned(),
                properties,
                events,
            })
            .unwrap();
        catalog
            .register_action(id("host.preview.report", ActionRef::new))
            .unwrap();
        catalog
    }

    fn document() -> PreviewDocument {
        let mut document = PreviewDocument::new(
            id("primary-button-example", PreviewId::new),
            id("rustui.button", ComponentId::new),
            1,
        );
        document.actions.insert(
            id("activate", EventId::new),
            id("host.preview.report", ActionRef::new),
        );
        document
    }

    #[test]
    fn validation_resolves_defaults_and_checks_action_registration() {
        let catalog = catalog();
        let validated = validate_preview(&catalog, document()).unwrap();
        assert_eq!(
            validated.properties.get(&id("label", PropertyId::new)),
            Some(&PropertyValue::String("Run".to_owned()))
        );
        assert_eq!(
            validated.properties.get(&id("disabled", PropertyId::new)),
            Some(&PropertyValue::Boolean(false))
        );
    }

    #[test]
    fn duplicate_action_registration_preserves_existing_allowlist_entry() {
        let mut catalog = catalog();
        let action = ActionRef::new("host.preview.report").unwrap();

        assert!(catalog.register_action(action).is_err());
        assert!(validate_preview(&catalog, document()).is_ok());
    }

    #[test]
    fn validation_rejects_unknown_component_and_version() {
        let catalog = catalog();
        let mut unknown = document();
        unknown.component_id = id("rustui.unknown", ComponentId::new);
        assert!(matches!(
            validate_preview(&catalog, unknown),
            Err(PreviewError::UnknownComponent(_))
        ));
        let mut wrong_version = document();
        wrong_version.component_version = 2;
        assert!(matches!(
            validate_preview(&catalog, wrong_version),
            Err(PreviewError::ComponentVersionMismatch { .. })
        ));
    }

    #[test]
    fn validation_rejects_unknown_properties_wrong_types_and_unresolved_actions() {
        let catalog = catalog();
        let mut unknown = document();
        unknown
            .properties
            .insert(id("missing", PropertyId::new), PropertyValue::Boolean(true));
        assert!(matches!(
            validate_preview(&catalog, unknown),
            Err(PreviewError::UnknownProperty { .. })
        ));

        let mut wrong_type = document();
        wrong_type.properties.insert(
            id("disabled", PropertyId::new),
            PropertyValue::String("yes".to_owned()),
        );
        assert!(matches!(
            validate_preview(&catalog, wrong_type),
            Err(PreviewError::InvalidPropertyValue { .. })
        ));

        let mut unresolved = document();
        unresolved.actions.insert(
            id("activate", EventId::new),
            id("host.unregistered.action", ActionRef::new),
        );
        assert!(matches!(
            validate_preview(&catalog, unresolved),
            Err(PreviewError::UnresolvedAction(_))
        ));
    }

    #[test]
    fn batch_updates_are_rejected_as_a_unit() {
        let catalog = catalog();
        let original = validate_preview(&catalog, document()).unwrap();
        let result = original.apply_batch(
            &catalog,
            [
                (
                    id("label", PropertyId::new),
                    PropertyValue::String("Execute".to_owned()),
                ),
                (
                    id("disabled", PropertyId::new),
                    PropertyValue::String("not a boolean".to_owned()),
                ),
            ],
        );
        assert!(matches!(
            result,
            Err(PreviewError::InvalidPropertyValue { .. })
        ));
        assert_eq!(
            original.properties.get(&id("label", PropertyId::new)),
            Some(&PropertyValue::String("Run".to_owned()))
        );
    }

    #[test]
    fn batch_updates_run_cross_property_validation_after_full_resolution() {
        let catalog = catalog();
        let original = validate_preview(&catalog, document()).unwrap();
        let result = original.apply_batch_with(
            &catalog,
            [
                (
                    id("label", PropertyId::new),
                    PropertyValue::String(" ".to_owned()),
                ),
                (
                    id("disabled", PropertyId::new),
                    PropertyValue::Boolean(true),
                ),
            ],
            |properties| match (
                properties.get(&id("label", PropertyId::new)),
                properties.get(&id("disabled", PropertyId::new)),
            ) {
                (Some(PropertyValue::String(label)), Some(PropertyValue::Boolean(true)))
                    if label.trim().is_empty() =>
                {
                    Err("disabled button needs a visible label".to_owned())
                }
                _ => Ok(()),
            },
        );
        assert!(matches!(result, Err(PreviewError::Invalid(_))));
        assert_eq!(
            original.properties.get(&id("label", PropertyId::new)),
            Some(&PropertyValue::String("Run".to_owned()))
        );
    }

    #[test]
    fn required_default_conflict_does_not_poison_component_registration() {
        let baseline = catalog();
        let component_id = ComponentId::new("rustui.button").unwrap();
        let property_id = PropertyId::new("label").unwrap();
        let corrected = baseline.components[&component_id].clone();
        let mut invalid = corrected.clone();
        invalid.properties.get_mut(&property_id).unwrap().required = true;
        let mut accepted = ComponentCatalog::new();

        assert!(matches!(
            accepted.register_component(invalid),
            Err(PreviewError::Invalid(_))
        ));
        accepted.register_component(corrected).unwrap();
        accepted
            .register_action(ActionRef::new("host.preview.report").unwrap())
            .unwrap();
        assert!(validate_preview(&accepted, document()).is_ok());
    }

    #[test]
    fn schema_rejects_invalid_bounds_and_identifier_values() {
        let mut catalog = catalog();
        let schema = catalog
            .components
            .get_mut(&id("rustui.button", ComponentId::new))
            .unwrap();
        let label = schema
            .properties
            .get_mut(&id("label", PropertyId::new))
            .unwrap();
        label.minimum = Some(2.0);
        assert!(matches!(
            validate_catalog(&catalog),
            Err(PreviewError::Invalid(_))
        ));
        assert!(PropertyId::new("Not-Lowercase").is_err());
    }

    #[test]
    fn integer_schema_rejects_ranges_with_no_representable_value() {
        let mut catalog = catalog();
        let schema = catalog
            .components
            .get_mut(&id("rustui.button", ComponentId::new))
            .unwrap();
        schema.properties.insert(
            id("count", PropertyId::new),
            PropertySchema {
                kind: PropertyKind::Integer,
                default: None,
                required: false,
                editable: true,
                minimum: Some(1.1),
                maximum: Some(1.9),
                min_length: None,
                max_length: None,
                allowed_values: Vec::new(),
            },
        );

        assert!(matches!(
            validate_catalog(&catalog),
            Err(PreviewError::Invalid(_))
        ));
    }

    #[test]
    fn schema_rejects_allowed_values_outside_string_length_bounds() {
        let mut catalog = catalog();
        {
            let schema = catalog
                .components
                .get_mut(&id("rustui.button", ComponentId::new))
                .unwrap()
                .properties
                .get_mut(&id("label", PropertyId::new))
                .unwrap();
            schema.default = None;
            schema.min_length = Some(2);
            schema.allowed_values = vec!["a".to_owned()];
        }

        assert!(matches!(
            validate_catalog(&catalog),
            Err(PreviewError::Invalid(_))
        ));

        {
            let schema = catalog
                .components
                .get_mut(&id("rustui.button", ComponentId::new))
                .unwrap()
                .properties
                .get_mut(&id("label", PropertyId::new))
                .unwrap();
            schema.min_length = None;
            schema.max_length = Some(0);
        }
        assert!(matches!(
            validate_catalog(&catalog),
            Err(PreviewError::Invalid(_))
        ));
    }
}
