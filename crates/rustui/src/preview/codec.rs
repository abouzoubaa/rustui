use crate::preview::model::{
    ComponentCatalog, ComponentId, EventId, MAX_DOCUMENT_BYTES, PreviewDocument, PreviewError,
    PropertyId, PropertyKind, PropertySchema, PropertyValue, ValidatedPreview,
};
use crate::preview::schema::{validate_catalog, validate_preview};

/// Decodes and validates a versioned compiled-component catalog.
pub fn decode_catalog(source: &str) -> Result<ComponentCatalog, PreviewError> {
    check_size(source)?;
    let mut catalog: ComponentCatalog = serde_json::from_str(source)?;
    validate_catalog_number_default_tokens(source)?;
    normalize_numeric_defaults(&mut catalog)?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

/// Encodes a validated catalog with deterministic key ordering.
pub fn encode_catalog(catalog: &ComponentCatalog) -> Result<String, PreviewError> {
    check_catalog_text_budget(catalog)?;
    let mut catalog = catalog.clone();
    normalize_numeric_defaults(&mut catalog)?;
    validate_catalog(&catalog)?;
    encode_json(&catalog)
}

// Each counted string is emitted once in the catalog JSON. Quotes and escaping
// can only increase its encoded length, so this lower bound rejects oversized
// caller-owned catalogs before cloning them without excluding valid output.
fn check_catalog_text_budget(catalog: &ComponentCatalog) -> Result<(), PreviewError> {
    let mut bytes = 0usize;
    let mut add = |text: &str| {
        bytes = bytes.saturating_add(text.len()).saturating_add(2);
        if bytes > MAX_DOCUMENT_BYTES {
            Err(PreviewError::Invalid(format!(
                "preview document exceeds {MAX_DOCUMENT_BYTES} byte limit"
            )))
        } else {
            Ok(())
        }
    };
    add(&catalog.format)?;
    for action in &catalog.actions {
        add(action.as_str())?;
    }
    for (id, component) in &catalog.components {
        add(id.as_str())?;
        add(component.id.as_str())?;
        add(&component.display_name)?;
        for (event_id, event) in &component.events {
            add(event_id.as_str())?;
            add(&event.display_name)?;
        }
        for properties in std::iter::once(&component.properties)
            .chain(component.events.values().map(|event| &event.payload))
        {
            for (property_id, schema) in properties {
                add(property_id.as_str())?;
                if let Some(PropertyValue::String(value)) = &schema.default {
                    add(value)?;
                }
                for value in &schema.allowed_values {
                    add(value)?;
                }
            }
        }
    }
    Ok(())
}

/// Decodes a preview descriptor and resolves its defaults against the catalog.
pub fn decode_preview(
    source: &str,
    catalog: &ComponentCatalog,
) -> Result<ValidatedPreview, PreviewError> {
    check_size(source)?;
    let mut document: PreviewDocument = serde_json::from_str(source)?;
    validate_number_integer_tokens(source, catalog, &document)?;
    normalize_numeric_properties(catalog, &mut document)?;
    validate_preview(catalog, document)
}

/// Decodes one JSON edit value using the property's declared numeric kind.
/// Integer tokens for Number properties must be exactly representable as f64.
pub fn decode_property_value(
    source: &str,
    id: &PropertyId,
    schema: &PropertySchema,
) -> Result<PropertyValue, PreviewError> {
    check_size(source)?;
    let raw: &serde_json::value::RawValue = serde_json::from_str(source)?;
    validate_raw_number(id, schema.kind, Some(raw))?;
    let value: PropertyValue = serde_json::from_str(source)?;
    if let (PropertyKind::Number, PropertyValue::Integer(integer)) = (schema.kind, &value) {
        return Ok(PropertyValue::Number(exact_integer_as_number(
            *integer, id,
        )?));
    }
    Ok(value)
}

/// Encodes a preview descriptor only after validating its full schema contract.
pub fn encode_preview(
    document: &PreviewDocument,
    catalog: &ComponentCatalog,
) -> Result<String, PreviewError> {
    check_serialized_text_budget(
        std::iter::once(document),
        &format!("preview document exceeds {MAX_DOCUMENT_BYTES} byte limit"),
    )?;
    let mut document = document.clone();
    normalize_numeric_properties(catalog, &mut document)?;
    let validated = validate_preview(catalog, document)?;
    encode_json(&validated.document)
}

// Every property string is emitted in the JSON output. Its UTF-8 length is a
// lower bound: quoting and escaping can only increase the number of bytes.
// Rejecting this bound before cloning cannot reject an output within the cap.
pub(super) fn check_serialized_text_budget<'a>(
    documents: impl IntoIterator<Item = &'a PreviewDocument>,
    limit_message: &str,
) -> Result<(), PreviewError> {
    let mut bytes = 0usize;
    for document in documents {
        for value in document.properties.values() {
            if let PropertyValue::String(text) = value {
                bytes = bytes.saturating_add(text.len());
                if bytes > MAX_DOCUMENT_BYTES {
                    return Err(PreviewError::Invalid(limit_message.to_owned()));
                }
            }
        }
    }
    Ok(())
}

fn normalize_numeric_defaults(catalog: &mut ComponentCatalog) -> Result<(), PreviewError> {
    for component in catalog.components.values_mut() {
        normalize_property_defaults(&mut component.properties)?;
        for event in component.events.values_mut() {
            normalize_property_defaults(&mut event.payload)?;
        }
    }
    Ok(())
}

fn validate_catalog_number_default_tokens(source: &str) -> Result<(), PreviewError> {
    let raw: RawCatalog<'_> = serde_json::from_str(source)?;
    for component in raw.components.values() {
        for (id, schema) in &component.properties {
            validate_raw_number(id, schema.kind, schema.default)?;
        }
        for event in component.events.values() {
            for (id, schema) in &event.payload {
                validate_raw_number(id, schema.kind, schema.default)?;
            }
        }
    }
    Ok(())
}

fn validate_raw_number(
    property: &PropertyId,
    kind: PropertyKind,
    value: Option<&serde_json::value::RawValue>,
) -> Result<(), PreviewError> {
    if kind == PropertyKind::Number
        && value
            .is_some_and(|raw| is_integer_token(raw.get()) && !is_exact_integer_token(raw.get()))
    {
        return Err(PreviewError::InvalidPropertyValue {
            property: property.clone(),
            reason: "integer cannot be represented exactly as number".to_owned(),
        });
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct RawCatalog<'a> {
    #[serde(borrow, default)]
    components: std::collections::BTreeMap<ComponentId, RawComponent<'a>>,
}

#[derive(serde::Deserialize)]
struct RawComponent<'a> {
    #[serde(borrow, default)]
    properties: std::collections::BTreeMap<PropertyId, RawPropertySchema<'a>>,
    #[serde(borrow, default)]
    events: std::collections::BTreeMap<EventId, RawEventSchema<'a>>,
}

#[derive(serde::Deserialize)]
struct RawEventSchema<'a> {
    #[serde(borrow, default)]
    payload: std::collections::BTreeMap<PropertyId, RawPropertySchema<'a>>,
}

#[derive(serde::Deserialize)]
struct RawPropertySchema<'a> {
    kind: PropertyKind,
    #[serde(borrow, default)]
    default: Option<&'a serde_json::value::RawValue>,
}

fn normalize_property_defaults(
    properties: &mut std::collections::BTreeMap<PropertyId, PropertySchema>,
) -> Result<(), PreviewError> {
    for (id, schema) in properties {
        if schema.kind == PropertyKind::Number {
            if let Some(PropertyValue::Integer(value)) = &schema.default {
                schema.default = Some(PropertyValue::Number(exact_integer_as_number(*value, id)?));
            }
        }
    }
    Ok(())
}

pub(super) fn normalize_numeric_properties(
    catalog: &ComponentCatalog,
    document: &mut PreviewDocument,
) -> Result<(), PreviewError> {
    let Some(schema) = catalog.components.get(&document.component_id) else {
        return Ok(());
    };
    for (id, value) in &mut document.properties {
        if schema
            .properties
            .get(id)
            .is_some_and(|property| property.kind == PropertyKind::Number)
        {
            if let PropertyValue::Integer(integer) = value {
                *value = PropertyValue::Number(exact_integer_as_number(*integer, id)?);
            }
        }
    }
    Ok(())
}

fn validate_number_integer_tokens(
    source: &str,
    catalog: &ComponentCatalog,
    document: &PreviewDocument,
) -> Result<(), PreviewError> {
    let Some(component) = catalog.components.get(&document.component_id) else {
        return Ok(());
    };
    let raw: RawPreviewProperties<'_> = serde_json::from_str(source)?;
    for (id, value) in raw.properties {
        if component
            .properties
            .get(&id)
            .is_some_and(|schema| schema.kind == PropertyKind::Number)
        {
            validate_raw_number(&id, PropertyKind::Number, Some(value))?;
        }
    }
    Ok(())
}

#[derive(serde::Deserialize)]
struct RawPreviewProperties<'a> {
    #[serde(borrow, default)]
    properties: std::collections::BTreeMap<PropertyId, &'a serde_json::value::RawValue>,
}

fn is_integer_token(token: &str) -> bool {
    token
        .strip_prefix('-')
        .unwrap_or(token)
        .bytes()
        .all(|byte| byte.is_ascii_digit())
}

fn is_exact_integer_token(token: &str) -> bool {
    let Ok(value) = token.parse::<f64>() else {
        return false;
    };
    if !value.is_finite() {
        return false;
    }
    let digits = token.strip_prefix('-').unwrap_or(token);
    let mut limbs = Vec::<u32>::new();
    for chunk in digits.as_bytes().rchunks(9).rev() {
        let Ok(chunk) = std::str::from_utf8(chunk) else {
            return false;
        };
        let Ok(chunk) = chunk.parse::<u32>() else {
            return false;
        };
        let mut carry = chunk;
        for limb in &mut limbs {
            let next = u64::from(*limb) * 1_000_000_000 + u64::from(carry);
            *limb = (next & 0xffff_ffff) as u32;
            carry = (next >> 32) as u32;
        }
        while carry != 0 {
            limbs.push(carry);
            carry = 0;
        }
    }
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
    if limbs.is_empty() {
        return true;
    }
    let mut probe = limbs.clone();
    let mut bit_length = 0usize;
    while !probe.is_empty() {
        divide_big_integer_by_two(&mut probe);
        bit_length += 1;
    }
    probe = limbs;
    for _ in 0..bit_length.saturating_sub(53) {
        if divide_big_integer_by_two(&mut probe) {
            return false;
        }
    }
    true
}

fn divide_big_integer_by_two(limbs: &mut Vec<u32>) -> bool {
    let mut remainder = 0u64;
    for limb in limbs.iter_mut().rev() {
        let value = (remainder << 32) | u64::from(*limb);
        *limb = (value / 2) as u32;
        remainder = value % 2;
    }
    while limbs.last() == Some(&0) {
        limbs.pop();
    }
    remainder != 0
}

fn exact_integer_as_number(value: i64, property: &PropertyId) -> Result<f64, PreviewError> {
    let number = value as f64;
    if number as i128 == i128::from(value) {
        Ok(number)
    } else {
        Err(PreviewError::InvalidPropertyValue {
            property: property.clone(),
            reason: "integer cannot be represented exactly as a number".to_owned(),
        })
    }
}

fn check_size(source: &str) -> Result<(), PreviewError> {
    if source.len() > MAX_DOCUMENT_BYTES {
        return Err(PreviewError::Invalid(format!(
            "preview document exceeds the {MAX_DOCUMENT_BYTES} byte limit"
        )));
    }
    Ok(())
}

fn encode_json(value: &impl serde::Serialize) -> Result<String, PreviewError> {
    encode_json_bounded(
        value,
        &format!("preview document exceeds {MAX_DOCUMENT_BYTES} byte limit"),
    )
}

pub(super) fn encode_json_bounded(
    value: &impl serde::Serialize,
    limit_message: &str,
) -> Result<String, PreviewError> {
    use std::io::{self, Write};

    struct BoundedOutput {
        bytes: Vec<u8>,
        exceeded: bool,
    }
    impl Write for BoundedOutput {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            if bytes.len() > MAX_DOCUMENT_BYTES - self.bytes.len() {
                self.exceeded = true;
                return Err(io::Error::other("preview output size limit exceeded"));
            }
            self.bytes.extend_from_slice(bytes);
            Ok(bytes.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    let mut output = BoundedOutput {
        bytes: Vec::new(),
        exceeded: false,
    };
    if let Err(error) = serde_json::to_writer_pretty(&mut output, value) {
        return if output.exceeded {
            Err(PreviewError::Invalid(limit_message.to_owned()))
        } else {
            Err(error.into())
        };
    }
    // serde_json emits UTF-8 when writing JSON.
    Ok(String::from_utf8(output.bytes).expect("JSON output is UTF-8"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview::model::{
        ActionRef, CATALOG_FORMAT, ComponentId, ComponentSchema, EventId, EventSchema,
        PREVIEW_FORMAT, PreviewId, PropertyId, PropertyKind, PropertySchema, PropertyValue,
    };
    use std::collections::BTreeMap;

    #[test]
    fn oversized_valid_catalog_is_rejected_before_clone() {
        let mut catalog = build_catalog();
        let schema = catalog
            .components
            .values_mut()
            .next()
            .unwrap()
            .properties
            .values_mut()
            .next()
            .unwrap();
        schema.default = None;
        schema.max_length = None;
        schema.allowed_values = (0..9_000)
            .map(|index| format!("value_{index:05}{}", "x".repeat(110)))
            .collect();

        assert!(validate_catalog(&catalog).is_ok());
        assert!(matches!(
            check_catalog_text_budget(&catalog),
            Err(PreviewError::Invalid(_))
        ));
        assert!(matches!(
            encode_catalog(&catalog),
            Err(PreviewError::Invalid(_))
        ));
        assert!(check_catalog_text_budget(&build_catalog()).is_ok());
    }

    #[test]
    fn serialized_text_budget_counts_utf8_across_documents() {
        let mut first = build_document();
        let mut second = build_document();
        first.properties.clear();
        second.properties.clear();
        first.properties.insert(
            PropertyId::new("label").unwrap(),
            PropertyValue::String("é".repeat(MAX_DOCUMENT_BYTES / 4)),
        );
        second.properties.insert(
            PropertyId::new("label").unwrap(),
            PropertyValue::String("x".repeat(MAX_DOCUMENT_BYTES / 2)),
        );
        assert!(check_serialized_text_budget([&first, &second], "too large").is_ok());
        second.properties.insert(
            PropertyId::new("extra").unwrap(),
            PropertyValue::String("x".to_owned()),
        );
        assert!(matches!(
            check_serialized_text_budget([&first, &second], "too large"),
            Err(PreviewError::Invalid(message)) if message == "too large"
        ));
    }

    #[test]
    fn bounded_json_encoding_preserves_exact_limit_and_rejects_larger_output() {
        let at_limit = "x".repeat(MAX_DOCUMENT_BYTES - 2);
        let encoded = encode_json(&at_limit).expect("JSON at the byte limit is accepted");
        assert_eq!(encoded.len(), MAX_DOCUMENT_BYTES);
        assert_eq!(encoded, serde_json::to_string_pretty(&at_limit).unwrap());

        let over_limit = "x".repeat(MAX_DOCUMENT_BYTES - 1);
        assert!(matches!(
            encode_json(&over_limit),
            Err(PreviewError::Invalid(_))
        ));
        let unicode_over_limit = "é".repeat(MAX_DOCUMENT_BYTES / 2);
        assert!(matches!(
            encode_json(&unicode_over_limit),
            Err(PreviewError::Invalid(_))
        ));
    }

    #[test]
    fn bounded_json_encoding_stops_serializing_when_limit_is_reached() {
        use serde::ser::SerializeSeq;
        use std::cell::Cell;

        struct ManyValues<'a>(&'a Cell<usize>);
        impl serde::Serialize for ManyValues<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                let mut sequence = serializer.serialize_seq(None)?;
                for _ in 0..MAX_DOCUMENT_BYTES {
                    self.0.set(self.0.get() + 1);
                    sequence.serialize_element(&"value")?;
                }
                sequence.end()
            }
        }

        let serialized = Cell::new(0);
        assert!(matches!(
            encode_json(&ManyValues(&serialized)),
            Err(PreviewError::Invalid(_))
        ));
        assert!(serialized.get() < MAX_DOCUMENT_BYTES);
    }

    fn build_catalog() -> ComponentCatalog {
        let mut properties = BTreeMap::new();
        properties.insert(
            PropertyId::new("label").unwrap(),
            PropertySchema {
                kind: PropertyKind::String,
                default: Some(PropertyValue::String("Run".to_owned())),
                required: false,
                editable: true,
                minimum: None,
                maximum: None,
                min_length: None,
                max_length: Some(64),
                allowed_values: Vec::new(),
            },
        );
        let mut events = BTreeMap::new();
        events.insert(
            EventId::new("activate").unwrap(),
            EventSchema {
                display_name: "Activate".to_owned(),
                payload: BTreeMap::new(),
            },
        );
        let mut catalog = ComponentCatalog::new();
        catalog
            .register_component(ComponentSchema {
                id: ComponentId::new("rustui.button").unwrap(),
                version: 1,
                display_name: "Button".to_owned(),
                properties,
                events,
            })
            .unwrap();
        catalog
            .register_action(ActionRef::new("host.preview.report").unwrap())
            .unwrap();
        catalog
    }

    fn build_document() -> PreviewDocument {
        let mut document = PreviewDocument::new(
            PreviewId::new("primary-button-example").unwrap(),
            ComponentId::new("rustui.button").unwrap(),
            1,
        );
        document.properties.insert(
            PropertyId::new("label").unwrap(),
            PropertyValue::String("Run".to_owned()),
        );
        document.actions.insert(
            EventId::new("activate").unwrap(),
            ActionRef::new("host.preview.report").unwrap(),
        );
        document
    }

    #[test]
    fn encoding_rejects_valid_properties_whose_text_exceeds_the_document_limit() {
        let mut catalog = build_catalog();
        let mut document = build_document();
        let schema = catalog.components.get_mut(&document.component_id).unwrap();
        let mut property = schema.properties.values().next().unwrap().clone();
        property.default = None;
        property.max_length = None;
        schema.properties.clear();
        document.properties.clear();
        for index in 0..65 {
            let id = PropertyId::new(format!("field-{index}")).unwrap();
            schema.properties.insert(id.clone(), property.clone());
            document.properties.insert(
                id,
                PropertyValue::String("x".repeat(crate::preview::model::MAX_STRING_VALUE_BYTES)),
            );
        }
        assert!(validate_preview(&catalog, document.clone()).is_ok());
        assert!(matches!(
            encode_preview(&document, &catalog),
            Err(PreviewError::Invalid(_))
        ));
    }

    #[test]
    fn catalog_and_preview_round_trip_deterministically() {
        let catalog = build_catalog();
        let encoded_catalog = encode_catalog(&catalog).unwrap();
        assert_eq!(decode_catalog(&encoded_catalog).unwrap(), catalog);
        assert_eq!(
            encode_catalog(&decode_catalog(&encoded_catalog).unwrap()).unwrap(),
            encoded_catalog
        );
        assert!(encoded_catalog.contains(CATALOG_FORMAT));

        let document = build_document();
        let encoded = encode_preview(&document, &catalog).unwrap();
        assert_eq!(
            decode_preview(&encoded, &catalog).unwrap().document,
            document
        );
        assert_eq!(encode_preview(&document, &catalog).unwrap(), encoded);
        assert!(encoded.contains(PREVIEW_FORMAT));
    }

    #[test]
    fn decoder_rejects_unknown_versions_and_duplicate_property_keys() {
        let catalog = build_catalog();
        let missing_components =
            r#"{"format":"rustui.component-catalog","format_version":1,"actions":[]}"#;
        assert!(matches!(
            decode_catalog(missing_components),
            Err(PreviewError::Invalid(_))
        ));
        let unknown_catalog_version = r#"{"format":"rustui.component-catalog","format_version":9,"components":{},"actions":[]}"#;
        assert!(matches!(
            decode_catalog(unknown_catalog_version),
            Err(PreviewError::UnsupportedCatalogVersion(9))
        ));
        let unknown_version = r#"{"format":"rustui.preview","format_version":9,"preview_id":"sample","component_id":"rustui.button","component_version":1,"properties":{},"actions":{}}"#;
        assert!(matches!(
            decode_preview(unknown_version, &catalog),
            Err(PreviewError::UnsupportedPreviewVersion(9))
        ));

        let duplicate = r#"{"format":"rustui.preview","format_version":1,"preview_id":"sample","component_id":"rustui.button","component_version":1,"properties":{"label":"Run","label":"Stop"},"actions":{}}"#;
        assert!(matches!(
            decode_preview(duplicate, &catalog),
            Err(PreviewError::Json(_))
        ));
    }

    #[test]
    fn decoder_rejects_duplicate_schema_keys_and_malformed_action_references() {
        let duplicate_schema = r#"{"format":"rustui.component-catalog","format_version":1,"components":{"rustui.button":{"id":"rustui.button","version":1,"display_name":"Button","properties":{"label":{"kind":"string"},"label":{"kind":"string"}},"events":{}}},"actions":[]}"#;
        assert!(matches!(
            decode_catalog(duplicate_schema),
            Err(PreviewError::Json(_))
        ));

        let catalog = build_catalog();
        let malformed = r#"{"format":"rustui.preview","format_version":1,"preview_id":"sample","component_id":"rustui.button","component_version":1,"properties":{},"actions":{"activate":"Bad Action!"}}"#;
        assert!(matches!(
            decode_preview(malformed, &catalog),
            Err(PreviewError::Invalid(_))
        ));
    }

    #[test]
    fn codec_rejects_oversized_input() {
        let oversized = " ".repeat(MAX_DOCUMENT_BYTES + 1);
        assert!(matches!(
            decode_catalog(&oversized),
            Err(PreviewError::Invalid(_))
        ));
    }

    #[test]
    fn integer_values_round_trip_without_f64_precision_loss() {
        let mut catalog = build_catalog();
        let schema = catalog
            .components
            .get_mut(&ComponentId::new("rustui.button").unwrap())
            .unwrap();
        schema.properties.insert(
            PropertyId::new("count").unwrap(),
            PropertySchema {
                kind: PropertyKind::Integer,
                default: None,
                required: true,
                editable: true,
                minimum: Some(9_007_199_254_740_993.0),
                maximum: Some(9_223_372_036_854_775_807.0),
                min_length: None,
                max_length: None,
                allowed_values: Vec::new(),
            },
        );
        let source = r#"{"format":"rustui.preview","format_version":1,"preview_id":"sample","component_id":"rustui.button","component_version":1,"properties":{"label":"Run","count":9007199254740993},"actions":{}}"#;
        let decoded = decode_preview(source, &catalog).unwrap();
        assert_eq!(
            decoded.properties.get(&PropertyId::new("count").unwrap()),
            Some(&PropertyValue::Integer(9_007_199_254_740_993))
        );
        let encoded = encode_preview(&decoded.document, &catalog).unwrap();
        assert!(encoded.contains("9007199254740993"));
        assert_eq!(
            decode_preview(&encoded, &catalog).unwrap().document,
            decoded.document
        );
    }

    #[test]
    fn i64_extrema_round_trip_and_out_of_range_json_integers_are_rejected() {
        let mut catalog = build_catalog();
        let count = PropertyId::new("count").unwrap();
        catalog
            .components
            .get_mut(&ComponentId::new("rustui.button").unwrap())
            .unwrap()
            .properties
            .insert(
                count.clone(),
                PropertySchema {
                    kind: PropertyKind::Integer,
                    default: None,
                    required: true,
                    editable: true,
                    minimum: None,
                    maximum: Some(2_f64.powi(63)),
                    min_length: None,
                    max_length: None,
                    allowed_values: Vec::new(),
                },
            );
        let source_with_count = |count: &str| {
            format!(
                r#"{{"format":"rustui.preview","format_version":1,"preview_id":"sample","component_id":"rustui.button","component_version":1,"properties":{{"label":"Run","count":{count}}},"actions":{{}}}}"#
            )
        };

        for value in [i64::MIN, i64::MAX] {
            let decoded = decode_preview(&source_with_count(&value.to_string()), &catalog).unwrap();
            assert_eq!(
                decoded.properties.get(&count),
                Some(&PropertyValue::Integer(value))
            );
            let encoded = encode_preview(&decoded.document, &catalog).unwrap();
            assert!(encoded.contains(&value.to_string()));
            assert_eq!(
                decode_preview(&encoded, &catalog).unwrap().document,
                decoded.document
            );
        }

        for out_of_range in ["9223372036854775808", "-9223372036854775809"] {
            assert!(matches!(
                decode_preview(&source_with_count(out_of_range), &catalog),
                Err(PreviewError::InvalidPropertyValue { property, .. }) if property == count
            ));
        }
    }

    #[test]
    fn integer_schema_rejects_fractional_json_and_out_of_bounds_values() {
        let mut catalog = build_catalog();
        let schema = catalog
            .components
            .get_mut(&ComponentId::new("rustui.button").unwrap())
            .unwrap();
        schema.properties.insert(
            PropertyId::new("count").unwrap(),
            PropertySchema {
                kind: PropertyKind::Integer,
                default: None,
                required: true,
                editable: true,
                minimum: Some(2.0),
                maximum: Some(4.0),
                min_length: None,
                max_length: None,
                allowed_values: Vec::new(),
            },
        );
        for json_value in ["1.5", "1", "5"] {
            let source = format!(
                r#"{{"format":"rustui.preview","format_version":1,"preview_id":"sample","component_id":"rustui.button","component_version":1,"properties":{{"label":"Run","count":{json_value}}},"actions":{{}}}}"#
            );
            assert!(matches!(
                decode_preview(&source, &catalog),
                Err(PreviewError::InvalidPropertyValue { .. })
            ));
        }
    }

    #[test]
    fn number_properties_accept_exact_integer_tokens_and_reject_lossy_ones() {
        let mut catalog = build_catalog();
        let schema = catalog
            .components
            .get_mut(&ComponentId::new("rustui.button").unwrap())
            .unwrap();
        schema.properties.insert(
            PropertyId::new("amount").unwrap(),
            PropertySchema {
                kind: PropertyKind::Number,
                default: Some(PropertyValue::Integer(1)),
                required: false,
                editable: true,
                minimum: None,
                maximum: None,
                min_length: None,
                max_length: None,
                allowed_values: Vec::new(),
            },
        );

        let encoded_catalog = encode_catalog(&catalog).unwrap();
        let decoded_catalog = decode_catalog(&encoded_catalog).unwrap();
        assert_eq!(
            decoded_catalog.components[&ComponentId::new("rustui.button").unwrap()].properties
                [&PropertyId::new("amount").unwrap()]
                .default,
            Some(PropertyValue::Number(1.0))
        );
        let lossy_default =
            encoded_catalog.replace("\"default\": 1.0", "\"default\": 9223372036854775809");
        assert!(matches!(
            decode_catalog(&lossy_default),
            Err(PreviewError::InvalidPropertyValue { .. })
        ));
        let mut event_catalog = build_catalog();
        event_catalog
            .components
            .get_mut(&ComponentId::new("rustui.button").unwrap())
            .unwrap()
            .events
            .get_mut(&EventId::new("activate").unwrap())
            .unwrap()
            .payload
            .insert(
                PropertyId::new("amount").unwrap(),
                PropertySchema {
                    kind: PropertyKind::Number,
                    default: Some(PropertyValue::Integer(1)),
                    required: false,
                    editable: true,
                    minimum: None,
                    maximum: None,
                    min_length: None,
                    max_length: None,
                    allowed_values: Vec::new(),
                },
            );
        let encoded_event_catalog = encode_catalog(&event_catalog).unwrap();
        let lossy_event_default =
            encoded_event_catalog.replace("\"default\": 1.0", "\"default\": 9223372036854775809");
        assert!(matches!(
            decode_catalog(&lossy_event_default),
            Err(PreviewError::InvalidPropertyValue { .. })
        ));

        let source = r#"{"format":"rustui.preview","format_version":1,"preview_id":"sample","component_id":"rustui.button","component_version":1,"properties":{"label":"Run","amount":2},"actions":{}}"#;
        let decoded = decode_preview(source, &decoded_catalog).unwrap();
        assert_eq!(
            decoded.properties.get(&PropertyId::new("amount").unwrap()),
            Some(&PropertyValue::Number(2.0))
        );
        let mut integer_document = decoded.document.clone();
        integer_document.properties.insert(
            PropertyId::new("amount").unwrap(),
            PropertyValue::Integer(2),
        );
        let encoded_integer_document = encode_preview(&integer_document, &decoded_catalog).unwrap();
        assert_eq!(
            decode_preview(&encoded_integer_document, &decoded_catalog)
                .unwrap()
                .document,
            decoded.document
        );
        let encoded = encode_preview(&decoded.document, &decoded_catalog).unwrap();
        assert_eq!(
            decode_preview(&encoded, &decoded_catalog).unwrap().document,
            decoded.document
        );

        let lossy = source.replace("\"amount\":2", "\"amount\":9007199254740993");
        assert!(matches!(
            decode_preview(&lossy, &decoded_catalog),
            Err(PreviewError::InvalidPropertyValue { .. })
        ));

        for (token, accepted) in [
            ("9223372036854775808", true),
            ("9223372036854775809", false),
            ("18446744073709551615", false),
            ("18446744073709551616", true),
            ("18446744073709551617", false),
            ("1267650600228229401496703205376", true),
            ("1267650600228229401496703205377", false),
            ("-9223372036854775809", false),
            ("9223372036854775809.0", true),
        ] {
            let candidate = source.replace("\"amount\":2", &format!("\"amount\":{token}"));
            assert_eq!(
                decode_preview(&candidate, &decoded_catalog).is_ok(),
                accepted,
                "unexpected acceptance for Number token {token}"
            );
        }
    }
}
