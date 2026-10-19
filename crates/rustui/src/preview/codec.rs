use crate::preview::model::{
    ComponentCatalog, MAX_DOCUMENT_BYTES, PreviewDocument, PreviewError, ValidatedPreview,
};
use crate::preview::schema::{validate_catalog, validate_preview};

/// Decodes and validates a versioned compiled-component catalog.
pub fn decode_catalog(source: &str) -> Result<ComponentCatalog, PreviewError> {
    check_size(source)?;
    let catalog: ComponentCatalog = serde_json::from_str(source)?;
    validate_catalog(&catalog)?;
    Ok(catalog)
}

/// Encodes a validated catalog with deterministic key ordering.
pub fn encode_catalog(catalog: &ComponentCatalog) -> Result<String, PreviewError> {
    validate_catalog(catalog)?;
    encode_json(catalog)
}

/// Decodes a preview descriptor and resolves its defaults against the catalog.
pub fn decode_preview(
    source: &str,
    catalog: &ComponentCatalog,
) -> Result<ValidatedPreview, PreviewError> {
    check_size(source)?;
    let document: PreviewDocument = serde_json::from_str(source)?;
    validate_preview(catalog, document)
}

/// Encodes a preview descriptor only after validating its full schema contract.
pub fn encode_preview(
    document: &PreviewDocument,
    catalog: &ComponentCatalog,
) -> Result<String, PreviewError> {
    validate_preview(catalog, document.clone())?;
    encode_json(document)
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
    let source = serde_json::to_string_pretty(value)?;
    check_size(&source)?;
    Ok(source)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::preview::model::{
        ActionRef, CATALOG_FORMAT, ComponentId, ComponentSchema, EventId, EventSchema,
        PREVIEW_FORMAT, PreviewId, PropertyId, PropertyKind, PropertySchema, PropertyValue,
    };
    use std::collections::BTreeMap;

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
}
