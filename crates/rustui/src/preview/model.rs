use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

use serde::{Deserialize, Serialize};

pub const PREVIEW_FORMAT: &str = "rustui.preview";
pub const PREVIEW_FORMAT_VERSION: u32 = 1;
pub const CATALOG_FORMAT: &str = "rustui.component-catalog";
pub const CATALOG_FORMAT_VERSION: u32 = 1;

/// Bounds applied before parsing and while validating descriptor metadata.
pub const MAX_DOCUMENT_BYTES: usize = 1_048_576;
pub const MAX_ID_BYTES: usize = 128;
pub const MAX_DISPLAY_NAME_BYTES: usize = 256;
pub const MAX_STRING_VALUE_BYTES: usize = 16_384;
pub const MAX_PROPERTIES_PER_COMPONENT: usize = 256;
pub const MAX_EVENTS_PER_COMPONENT: usize = 128;

macro_rules! stable_id {
    ($name:ident) => {
        #[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, String> {
                let value = value.into();
                if is_valid_stable_id(&value) {
                    Ok(Self(value))
                } else {
                    Err(format!("invalid {} `{value}`", stringify!($name)))
                }
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str(&self.0)
            }
        }
    };
}

stable_id!(ComponentId);
stable_id!(PropertyId);
stable_id!(EventId);
stable_id!(ActionRef);
stable_id!(PreviewId);

pub(crate) fn is_valid_stable_id(value: &str) -> bool {
    let bytes = value.as_bytes();
    !bytes.is_empty()
        && bytes.len() <= MAX_ID_BYTES
        && (bytes[0].is_ascii_lowercase() || bytes[0].is_ascii_digit())
        && bytes
            .iter()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || b"._-".contains(byte))
}

/// A JSON-compatible scalar exposed by a compiled component schema.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PropertyValue {
    String(String),
    Boolean(bool),
    Integer(i64),
    Number(f64),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PropertyKind {
    String,
    Boolean,
    Integer,
    Number,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PropertySchema {
    pub kind: PropertyKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<PropertyValue>,
    #[serde(default)]
    pub required: bool,
    #[serde(default = "default_editable")]
    pub editable: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_length: Option<usize>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_values: Vec<String>,
}

fn default_editable() -> bool {
    true
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EventSchema {
    pub display_name: String,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub payload: BTreeMap<PropertyId, PropertySchema>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSchema {
    pub id: ComponentId,
    pub version: u32,
    pub display_name: String,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub properties: BTreeMap<PropertyId, PropertySchema>,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub events: BTreeMap<EventId, EventSchema>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentCatalog {
    pub format: String,
    pub format_version: u32,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub components: BTreeMap<ComponentId, ComponentSchema>,
    #[serde(default, deserialize_with = "deserialize_unique_set")]
    pub actions: BTreeSet<ActionRef>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PreviewDocument {
    pub format: String,
    pub format_version: u32,
    pub preview_id: PreviewId,
    pub component_id: ComponentId,
    pub component_version: u32,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub properties: BTreeMap<PropertyId, PropertyValue>,
    #[serde(default, deserialize_with = "deserialize_unique_map")]
    pub actions: BTreeMap<EventId, ActionRef>,
}

/// A descriptor with its schema defaults resolved into an owned scalar map.
#[derive(Clone, Debug, PartialEq)]
pub struct ValidatedPreview {
    pub document: PreviewDocument,
    pub properties: BTreeMap<PropertyId, PropertyValue>,
}

/// Structured errors returned by descriptor parsing and validation.
#[derive(Debug)]
pub enum PreviewError {
    Json(serde_json::Error),
    Invalid(String),
    UnsupportedCatalogFormat(String),
    UnsupportedCatalogVersion(u32),
    UnsupportedPreviewFormat(String),
    UnsupportedPreviewVersion(u32),
    UnknownComponent(ComponentId),
    ComponentVersionMismatch {
        component: ComponentId,
        expected: u32,
        found: u32,
    },
    UnknownProperty {
        component: ComponentId,
        property: PropertyId,
    },
    MissingRequiredProperty(PropertyId),
    InvalidPropertyValue {
        property: PropertyId,
        reason: String,
    },
    UnknownEvent {
        component: ComponentId,
        event: EventId,
    },
    UnresolvedAction(ActionRef),
    NotEditable(PropertyId),
}

impl fmt::Display for PreviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(error) => write!(formatter, "invalid preview JSON: {error}"),
            Self::Invalid(message) => formatter.write_str(message),
            Self::UnsupportedCatalogFormat(format) => {
                write!(formatter, "unsupported component catalog format `{format}`")
            }
            Self::UnsupportedCatalogVersion(version) => {
                write!(formatter, "unsupported component catalog version {version}")
            }
            Self::UnsupportedPreviewFormat(format) => {
                write!(formatter, "unsupported preview format `{format}`")
            }
            Self::UnsupportedPreviewVersion(version) => {
                write!(formatter, "unsupported preview format version {version}")
            }
            Self::UnknownComponent(component) => {
                write!(formatter, "component `{component}` is not in the catalog")
            }
            Self::ComponentVersionMismatch {
                component,
                expected,
                found,
            } => write!(
                formatter,
                "component `{component}` requires version {expected}, found {found}"
            ),
            Self::UnknownProperty {
                component,
                property,
            } => {
                write!(
                    formatter,
                    "component `{component}` has no property `{property}`"
                )
            }
            Self::MissingRequiredProperty(property) => {
                write!(formatter, "required property `{property}` is missing")
            }
            Self::InvalidPropertyValue { property, reason } => {
                write!(
                    formatter,
                    "invalid value for property `{property}`: {reason}"
                )
            }
            Self::UnknownEvent { component, event } => {
                write!(formatter, "component `{component}` has no event `{event}`")
            }
            Self::UnresolvedAction(action) => {
                write!(
                    formatter,
                    "action reference `{action}` is not declared in the catalog"
                )
            }
            Self::NotEditable(property) => {
                write!(formatter, "property `{property}` is read-only")
            }
        }
    }
}

impl Error for PreviewError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for PreviewError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub(crate) fn deserialize_unique_map<'de, D, K, V>(
    deserializer: D,
) -> Result<BTreeMap<K, V>, D::Error>
where
    D: serde::Deserializer<'de>,
    K: Deserialize<'de> + Ord + fmt::Display,
    V: Deserialize<'de>,
{
    struct UniqueMapVisitor<K, V>(std::marker::PhantomData<(K, V)>);

    impl<'de, K, V> serde::de::Visitor<'de> for UniqueMapVisitor<K, V>
    where
        K: Deserialize<'de> + Ord + fmt::Display,
        V: Deserialize<'de>,
    {
        type Value = BTreeMap<K, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map with unique keys")
        }

        fn visit_map<A>(self, mut access: A) -> Result<Self::Value, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            let mut values = BTreeMap::new();
            while let Some((key, value)) = access.next_entry()? {
                if values.insert(key, value).is_some() {
                    return Err(serde::de::Error::custom("duplicate identifier in map"));
                }
            }
            Ok(values)
        }
    }

    deserializer.deserialize_map(UniqueMapVisitor(std::marker::PhantomData))
}

pub(crate) fn deserialize_unique_set<'de, D, T>(deserializer: D) -> Result<BTreeSet<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de> + Ord,
{
    let values = Vec::<T>::deserialize(deserializer)?;
    let mut result = BTreeSet::new();
    for value in values {
        if !result.insert(value) {
            return Err(serde::de::Error::custom("duplicate identifier in set"));
        }
    }
    Ok(result)
}
