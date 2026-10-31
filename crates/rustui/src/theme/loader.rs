//! Loading and validating RustUI theme documents.

use std::{collections::BTreeSet, error::Error, fmt, fs, io, path::Path};

use serde::de::{self, Deserialize, Deserializer, MapAccess, SeqAccess, Visitor};

use crate::theme::schema::ThemeDocument;

/// Load and validate a RustUI theme document from JSON text.
pub fn parse_theme(source: &str) -> Result<ThemeDocument, ThemeLoadError> {
    serde_json::from_str::<UniqueJson>(source)?;
    let document: ThemeDocument = serde_json::from_str(source)?;
    document.validate().map_err(ThemeLoadError::Invalid)?;
    Ok(document)
}

// Preserve the original JSON for the typed parser, which validates raw numeric
// tokens, while rejecting ambiguous object keys at every depth.
struct UniqueJson;

impl<'de> Deserialize<'de> for UniqueJson {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct UniqueVisitor;
        impl<'de> Visitor<'de> for UniqueVisitor {
            type Value = UniqueJson;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("JSON with unique object keys")
            }
            fn visit_bool<E: de::Error>(self, _: bool) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_i64<E: de::Error>(self, _: i64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_u64<E: de::Error>(self, _: u64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_f64<E: de::Error>(self, _: f64) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_str<E: de::Error>(self, _: &str) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_unit<E: de::Error>(self) -> Result<Self::Value, E> {
                Ok(UniqueJson)
            }
            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                while seq.next_element::<UniqueJson>()?.is_some() {}
                Ok(UniqueJson)
            }
            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
                let mut keys = BTreeSet::new();
                while let Some(key) = map.next_key::<String>()? {
                    if !keys.insert(key.clone()) {
                        return Err(de::Error::custom(format!("duplicate object key `{key}`")));
                    }
                    map.next_value::<UniqueJson>()?;
                }
                Ok(UniqueJson)
            }
        }
        deserializer.deserialize_any(UniqueVisitor)
    }
}

/// Serialize a validated theme document as normalized, pretty-printed JSON.
///
/// This exports the typed format-v1 document, not the original source bytes.
/// Whitespace, object ordering, and numeric spelling may change; absent optional
/// fields may be emitted explicitly. Arbitrary JSON values stored in the
/// document are preserved as JSON values. Fields rejected by the parser as
/// unknown cannot be recovered. Non-Button records with the typed `parts` or
/// `styles` envelope are validated against the shared component resolver;
/// opaque extension records and unsupported style properties remain intact.
pub fn serialize_theme(document: &ThemeDocument) -> Result<String, ThemeSerializeError> {
    document.validate().map_err(ThemeSerializeError::Invalid)?;
    serde_json::to_string_pretty(document).map_err(ThemeSerializeError::Json)
}

/// Read, parse, and validate a RustUI theme document from disk.
pub fn load_theme(path: impl AsRef<Path>) -> Result<ThemeDocument, ThemeLoadError> {
    let source = fs::read_to_string(path)?;
    parse_theme(&source)
}

#[derive(Debug)]
pub enum ThemeLoadError {
    Io(io::Error),
    Json(serde_json::Error),
    Invalid(String),
}

impl fmt::Display for ThemeLoadError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "could not read theme: {error}"),
            Self::Json(error) => write!(formatter, "invalid theme JSON: {error}"),
            Self::Invalid(error) => write!(formatter, "invalid theme: {error}"),
        }
    }
}

impl Error for ThemeLoadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Io(error) => Some(error),
            Self::Json(error) => Some(error),
            Self::Invalid(_) => None,
        }
    }
}

impl From<io::Error> for ThemeLoadError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

impl From<serde_json::Error> for ThemeLoadError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

#[derive(Debug)]
pub enum ThemeSerializeError {
    Invalid(String),
    Json(serde_json::Error),
}

impl fmt::Display for ThemeSerializeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(error) => write!(formatter, "invalid theme: {error}"),
            Self::Json(error) => write!(formatter, "could not serialize theme JSON: {error}"),
        }
    }
}

impl Error for ThemeSerializeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Invalid(_) => None,
            Self::Json(error) => Some(error),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const THEME_SOURCE: &str = include_str!("../../../../themes/macos27.json");

    #[test]
    fn validated_theme_serialization_round_trips_the_typed_document() {
        let theme = parse_theme(THEME_SOURCE).expect("source theme is valid");
        let serialized = serialize_theme(&theme).expect("valid theme serializes");
        let round_tripped = parse_theme(&serialized).expect("serialized theme parses");

        assert_eq!(round_tripped, theme);
    }

    #[test]
    fn serialization_preserves_arbitrary_json_theme_records() {
        let mut theme = parse_theme(THEME_SOURCE).expect("source theme is valid");
        theme.source["external_metadata"] = serde_json::json!({"owner": "tooling"});
        theme.token_sources["external_metadata"] = serde_json::json!({"revision": 3});
        theme.derivation["external_metadata"] = serde_json::json!("preserve");
        theme
            .components
            .get_mut("button")
            .expect("button component exists")["parts"]["custom_metadata"] =
            serde_json::json!({"supported": false});
        theme.components.insert(
            "custom_widget".into(),
            serde_json::json!({"rustui_type": "CustomWidget", "config": [1, true]}),
        );
        let future_artwork = serde_json::json!({
            "kind": "future_artwork",
            "value": {"path": "custom://toggle-thumb"}
        });
        theme.components.get_mut("toggle").unwrap()["styles"][0]["style"]["properties"]["future_artwork"] =
            future_artwork.clone();

        let serialized = serialize_theme(&theme).expect("valid theme serializes");
        let round_tripped = parse_theme(&serialized).expect("serialized theme parses");

        assert_eq!(round_tripped.source, theme.source);
        assert_eq!(round_tripped.token_sources, theme.token_sources);
        assert_eq!(round_tripped.derivation, theme.derivation);
        assert_eq!(
            round_tripped.components.get("custom_widget"),
            theme.components.get("custom_widget")
        );
        assert_eq!(
            round_tripped.components["button"]["parts"]["custom_metadata"],
            theme.components["button"]["parts"]["custom_metadata"]
        );
        assert_eq!(
            round_tripped.components["toggle"]["styles"][0]["style"]["properties"]["future_artwork"],
            future_artwork
        );
    }

    #[test]
    fn parser_rejects_non_object_custom_component_records() {
        let mut theme: ThemeDocument = serde_json::from_str(THEME_SOURCE).unwrap();
        theme
            .components
            .insert("custom_widget".into(), serde_json::json!("opaque"));
        let source = serde_json::to_string(&theme).unwrap();

        assert!(matches!(
            parse_theme(&source),
            Err(ThemeLoadError::Invalid(message))
                if message.contains("custom_widget") && message.contains("must be an object")
        ));
    }

    #[test]
    fn parser_accepts_typed_component_records_with_intentionally_empty_styles() {
        let theme = parse_theme(THEME_SOURCE).expect("source theme is valid");

        for component in ["toolbar", "sidebar", "menu_bar", "tab_bar"] {
            assert!(theme.component_theme(component).unwrap().styles.is_empty());
        }
    }

    #[test]
    fn parser_rejects_non_button_style_rules_that_reference_undeclared_parts() {
        let mut theme: ThemeDocument = serde_json::from_str(THEME_SOURCE).unwrap();
        theme.components.get_mut("toggle").unwrap()["styles"][0]["part"] =
            serde_json::json!("missing_part");
        let source = serde_json::to_string(&theme).unwrap();

        assert!(matches!(
            parse_theme(&source),
            Err(ThemeLoadError::Invalid(message))
                if message.contains("toggle") && message.contains("missing_part")
        ));
    }

    #[test]
    fn parser_rejects_ambiguous_equal_specificity_component_selectors() {
        let mut theme: ThemeDocument = serde_json::from_str(THEME_SOURCE).unwrap();
        let styles = theme.components.get_mut("toggle").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        styles[0]["when"]["appearance"] = serde_json::json!("dark");
        styles[0]["when"]["interaction"] = serde_json::json!("rest");
        styles[1]["when"]["appearance"] = serde_json::json!("any");
        styles[1]["when"]["material"] = serde_json::json!("opaque");
        styles[1]["when"]["interaction"] = serde_json::json!("rest");
        let source = serde_json::to_string(&theme).unwrap();

        assert!(matches!(
            parse_theme(&source),
            Err(ThemeLoadError::Invalid(message))
                if message.contains("toggle") && message.contains("ambiguous")
        ));
    }

    #[test]
    fn parser_rejects_invalid_recognized_property_on_non_default_rule() {
        let mut theme: ThemeDocument = serde_json::from_str(THEME_SOURCE).unwrap();
        theme.components.get_mut("toggle").unwrap()["styles"][0]["style"]["properties"]["width"]
            ["value"]["px"] = serde_json::json!(0.0);
        let source = serde_json::to_string(&theme).unwrap();

        assert!(matches!(
            parse_theme(&source),
            Err(ThemeLoadError::Invalid(message))
                if message.contains("toggle") && message.contains("width")
        ));
    }

    #[test]
    fn serialization_rejects_invalid_documents() {
        let mut theme = parse_theme(THEME_SOURCE).expect("source theme is valid");
        theme.format_version = 2;

        assert!(matches!(
            serialize_theme(&theme),
            Err(ThemeSerializeError::Invalid(_))
        ));
    }

    #[test]
    fn parser_rejects_unresolved_button_token_references() {
        let source = THEME_SOURCE.replace("\"control_inline\": \"space_2\",", "");

        assert!(matches!(
            parse_theme(&source),
            Err(ThemeLoadError::Invalid(message))
                if message.contains("unresolved token reference `spacing.control_inline`")
        ));
    }

    #[test]
    fn parser_rejects_missing_button_appearance_rules() {
        let mut theme: ThemeDocument = serde_json::from_str(THEME_SOURCE).unwrap();
        theme
            .components
            .get_mut("button")
            .expect("button theme exists")["styles"]
            .as_array_mut()
            .unwrap()
            .retain(|rule| rule["when"]["appearance"] != "dark");
        let malformed = serde_json::to_string(&theme).unwrap();

        assert!(matches!(
            parse_theme(&malformed),
            Err(ThemeLoadError::Invalid(_))
        ));
    }

    #[test]
    fn serialization_rejects_button_style_metric_mismatches() {
        let mut theme = parse_theme(THEME_SOURCE).expect("source theme is valid");
        let dark_style = theme
            .components
            .get_mut("button")
            .expect("button theme exists")["styles"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|rule| rule["when"]["appearance"] == "dark")
            .unwrap();
        dark_style["style"]["properties"]["height"]["value"]["px"] = 25.into();

        assert!(matches!(
            serialize_theme(&theme),
            Err(ThemeSerializeError::Invalid(message))
                if message.contains("button height 25 does not match metric reference 24")
        ));
    }

    #[test]
    fn parser_rejects_unknown_typed_fields_instead_of_discarding_them() {
        let source = THEME_SOURCE.replacen(
            "\"format\": \"rustui-appearance\",",
            "\"format\": \"rustui-appearance\",\n  \"unmodeled_field\": true,",
            1,
        );

        assert!(matches!(parse_theme(&source), Err(ThemeLoadError::Json(_))));
    }

    #[test]
    fn parser_rejects_duplicate_nested_object_keys() {
        let source = THEME_SOURCE.replacen(
            "\"background\": \"surface\",",
            "\"background\": \"ignored\",\n    \"background\": \"surface\",",
            1,
        );
        assert_ne!(source, THEME_SOURCE);
        assert!(matches!(parse_theme(&source), Err(ThemeLoadError::Json(_))));
        assert!(
            serde_json::from_str::<super::UniqueJson>(r#"[{"name":1,"\u006eame":2}]"#).is_err()
        );
        assert!(
            serde_json::from_str::<super::UniqueJson>(r#"[{"name":null},{"name":true}]"#).is_ok()
        );
    }
}
