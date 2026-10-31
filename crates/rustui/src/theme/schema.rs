//! RustUI's versioned platform-appearance theme contract.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Current on-disk RustUI appearance format version.
pub const THEME_FORMAT_VERSION: u8 = 1;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeDocument {
    pub format: String,
    #[serde(deserialize_with = "deserialize_theme_format_version")]
    pub format_version: u8,
    pub framework: FrameworkIdentity,
    pub profile: String,
    pub source: serde_json::Value,
    pub token_sources: serde_json::Value,
    pub theme_roles: BTreeMap<String, String>,
    pub components: BTreeMap<String, serde_json::Value>,
    pub derivation: serde_json::Value,
    pub platform_snapshot: ThemeProfile,
}

fn deserialize_theme_format_version<'de, D>(deserializer: D) -> Result<u8, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_integral_number(deserializer, "theme format version")
}

fn deserialize_integral_u16<'de, D>(deserializer: D) -> Result<u16, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_integral_number(deserializer, "theme integer")
}

fn deserialize_integral_u32<'de, D>(deserializer: D) -> Result<u32, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_integral_number(deserializer, "theme integer")
}

fn deserialize_optional_integral_u32<'de, D>(deserializer: D) -> Result<Option<u32>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptionalIntegerVisitor;

    impl<'de> serde::de::Visitor<'de> for OptionalIntegerVisitor {
        type Value = Option<u32>;

        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            formatter.write_str("an optional nonnegative integer")
        }

        fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
            Ok(None)
        }

        fn visit_some<D: serde::Deserializer<'de>>(
            self,
            deserializer: D,
        ) -> Result<Self::Value, D::Error> {
            deserialize_integral_u32(deserializer).map(Some)
        }
    }

    deserializer.deserialize_option(OptionalIntegerVisitor)
}

fn deserialize_integral_number<'de, D, T>(
    deserializer: D,
    field: &'static str,
) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: TryFrom<u64>,
{
    let raw = Box::<serde_json::value::RawValue>::deserialize(deserializer)?;
    let value = parse_integral_json_number(raw.get()).ok_or_else(|| {
        serde::de::Error::custom(format!("{} must be a nonnegative integer", field))
    })?;
    T::try_from(value).map_err(|_| serde::de::Error::custom(format!("{} is out of range", field)))
}

fn parse_integral_json_number(raw: &str) -> Option<u64> {
    let (negative, raw) = raw
        .strip_prefix('-')
        .map_or((false, raw), |raw| (true, raw));
    let (mantissa, exponent) = raw
        .split_once(['e', 'E'])
        .map_or((raw, 0_i64), |(mantissa, exponent)| {
            (mantissa, exponent.parse::<i64>().unwrap_or(i64::MAX))
        });
    let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
    let mut digits = String::with_capacity(whole.len() + fraction.len());
    digits.push_str(whole);
    digits.push_str(fraction);
    if digits.is_empty() || !digits.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    if digits.bytes().all(|byte| byte == b'0') {
        return Some(0);
    }
    if negative || exponent == i64::MAX {
        return None;
    }

    let decimal_places = i64::try_from(fraction.len()).ok()?.checked_sub(exponent)?;
    if decimal_places > 0 {
        let places = usize::try_from(decimal_places).ok()?;
        if places > digits.len()
            || !digits.as_bytes()[digits.len() - places..]
                .iter()
                .all(|b| *b == b'0')
        {
            return None;
        }
        digits.truncate(digits.len() - places);
    } else {
        let zeroes = usize::try_from(decimal_places.checked_neg()?).ok()?;
        if digits.trim_start_matches('0').len().checked_add(zeroes)? > 20 {
            return None;
        }
        digits.extend(std::iter::repeat_n('0', zeroes));
    }
    let significant = digits.trim_start_matches('0');
    if significant.is_empty() || significant.len() > 20 {
        return None;
    }
    significant.parse().ok()
}

fn validate_slider_theme_roles(
    metadata: &BTreeMap<String, serde_json::Value>,
) -> Result<(), String> {
    let roles = metadata
        .get("theme_roles")
        .and_then(serde_json::Value::as_object)
        .ok_or_else(|| "slider theme_roles must be an object".to_owned())?;

    for role in ["track", "thumb", "disabled"] {
        let color_role = roles.get(role).and_then(serde_json::Value::as_str);
        if !matches!(
            color_role,
            Some("secondary" | "primary" | "muted_foreground")
        ) {
            return Err(format!(
                "slider theme_roles.{role} must name a supported profile color role"
            ));
        }
    }

    Ok(())
}

fn validate_text_renderer_roles(record: &ComponentThemeRecord) -> Result<(), String> {
    for (index, rule) in record.styles.iter().enumerate() {
        for role in ["foreground", "typography"] {
            if !rule.style.properties.contains_key(role) {
                return Err(format!(
                    "text style rule {index} is missing renderer-required `{role}` role"
                ));
            }
        }
    }
    Ok(())
}

fn validate_renderer_required_properties(
    component: &str,
    record: &ComponentThemeRecord,
    required: &[&str],
) -> Result<(), String> {
    validate_renderer_required_properties_where(component, record, required, |_| true)
}

fn validate_renderer_required_properties_for_part(
    component: &str,
    record: &ComponentThemeRecord,
    part: &str,
    required: &[&str],
) -> Result<(), String> {
    validate_renderer_required_properties_where(component, record, required, |rule| {
        rule.part == part
    })
}

fn validate_text_field_renderer_rules(record: &ComponentThemeRecord) -> Result<(), String> {
    for appearance in ["light", "dark"] {
        for size in ["small", "large"] {
            let has_matching_body_rule = record.styles.iter().any(|rule| {
                rule.part == "body"
                    && (rule.when.appearance == "any" || rule.when.appearance == appearance)
                    && (rule.when.size == "any" || rule.when.size == size)
            });
            if !has_matching_body_rule {
                return Err(format!(
                    "text_field is missing renderer-required `body` style rule for appearance `{appearance}` and size `{size}`"
                ));
            }
        }
    }
    Ok(())
}

fn validate_search_field_renderer_rules(record: &ComponentThemeRecord) -> Result<(), String> {
    for appearance in ["light", "dark"] {
        for size in ["small", "large"] {
            let has_matching_rule = record.styles.iter().any(|rule| {
                rule.part == "list_search"
                    && (rule.when.appearance == "any" || rule.when.appearance == appearance)
                    && (rule.when.size == "any" || rule.when.size == size)
            });
            if !has_matching_rule {
                return Err(format!(
                    "search_field is missing renderer-required `list_search` style rule for appearance `{appearance}` and size `{size}`"
                ));
            }
        }
    }
    Ok(())
}

fn validate_stepper_renderer_rules(record: &ComponentThemeRecord) -> Result<(), String> {
    for part in [
        "increment_button",
        "decrement_button",
        "separator",
        "increment_glyph",
        "decrement_glyph",
    ] {
        for appearance in ["light", "dark"] {
            let has_matching_rule = record.styles.iter().any(|rule| {
                rule.part == part
                    && (rule.when.appearance == "any" || rule.when.appearance == appearance)
            });
            if !has_matching_rule {
                return Err(format!(
                    "stepper is missing renderer-required `{part}` style rule for appearance `{appearance}`"
                ));
            }
        }
    }
    Ok(())
}

fn validate_renderer_required_properties_where(
    component: &str,
    record: &ComponentThemeRecord,
    required: &[&str],
    applies_to_rule: impl Fn(&ComponentStyleRule) -> bool,
) -> Result<(), String> {
    for (index, rule) in record.styles.iter().enumerate() {
        if !applies_to_rule(rule) {
            continue;
        }
        for property in required {
            if !rule.style.properties.contains_key(*property) {
                return Err(format!(
                    "{component} style rule {index} is missing renderer-required `{property}` property"
                ));
            }
        }
    }
    Ok(())
}

fn validate_checkbox_radio_artwork(
    component: &str,
    record: &ComponentThemeRecord,
) -> Result<(), String> {
    const IDENTITY_TRANSFORM: [f64; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];

    if !record.styles.iter().any(|rule| rule.part == "body") {
        return Err(format!(
            "{component} is missing renderer-required `body` style rule"
        ));
    }

    for (index, rule) in record.styles.iter().enumerate() {
        let artwork = rule
            .style
            .properties
            .get("vector_path")
            .and_then(|property| property.get("value"));
        let data = artwork
            .and_then(|value| value.get("data"))
            .and_then(serde_json::Value::as_str)
            .filter(|data| !data.trim().is_empty());
        let Some(data) = data else {
            return Err(format!(
                "{component} style rule {index} has invalid renderer `vector_path` data"
            ));
        };

        let commands: String = data.chars().filter(char::is_ascii_alphabetic).collect();
        let coordinates = data
            .split(|ch: char| ch.is_ascii_alphabetic() || ch.is_whitespace())
            .filter(|part| !part.is_empty())
            .map(str::parse::<f32>)
            .collect::<Result<Vec<_>, _>>();
        let Ok(coordinates) = coordinates else {
            return Err(format!(
                "{component} style rule {index} has invalid renderer `vector_path` coordinates"
            ));
        };
        if commands != "MHVHVZ"
            || coordinates.len() != 6
            || coordinates.iter().any(|coordinate| !coordinate.is_finite())
            || coordinates[0] != coordinates[4]
            || coordinates[1] != coordinates[5]
            || !(coordinates[2] - coordinates[0]).is_finite()
            || !(coordinates[3] - coordinates[1]).is_finite()
            || coordinates[2] - coordinates[0] <= 0.0
            || coordinates[3] - coordinates[1] <= 0.0
        {
            return Err(format!(
                "{component} style rule {index} has unsupported renderer `vector_path` geometry"
            ));
        }

        let transform = artwork
            .and_then(|value| value.get("transform"))
            .and_then(serde_json::Value::as_array);
        if !matches!(transform, Some(values) if values.len() == IDENTITY_TRANSFORM.len()
            && values.iter().zip(IDENTITY_TRANSFORM).all(|(actual, expected)| actual.as_f64() == Some(expected)))
        {
            return Err(format!(
                "{component} style rule {index} has unsupported renderer `vector_path` transform"
            ));
        }
    }

    Ok(())
}

impl ThemeDocument {
    /// Validate cross-field constraints that JSON's structural schema cannot express.
    pub fn validate(&self) -> Result<(), String> {
        if self.format != "rustui-appearance" {
            return Err(format!(
                "unsupported theme format `{}`; expected `rustui-appearance`",
                self.format
            ));
        }
        if self.format_version != THEME_FORMAT_VERSION {
            return Err(format!(
                "unsupported theme format version {}; expected {}",
                self.format_version, THEME_FORMAT_VERSION
            ));
        }
        if self.framework.name != "RustUI" || !is_semver(&self.framework.version) {
            return Err("theme framework must be RustUI with a numeric x.y.z version".into());
        }
        if self.profile != self.platform_snapshot.id {
            return Err(format!(
                "theme profile `{}` does not match platform snapshot `{}`",
                self.profile, self.platform_snapshot.id
            ));
        }
        for (field, value) in [
            ("source", &self.source),
            ("token_sources", &self.token_sources),
            ("derivation", &self.derivation),
        ] {
            if !value.is_object() {
                return Err(format!("theme `{field}` must be an object"));
            }
        }
        if self.theme_roles.is_empty() {
            return Err("theme must contain at least one theme role".into());
        }
        if self.components.is_empty() {
            return Err("theme must contain RustUI component definitions".into());
        }
        validate_nonempty_component_selectors(&self.components)?;
        self.button_theme()?.validate()?;
        for (name, value) in &self.components {
            if name == "button" {
                continue;
            }
            let Some(object) = value.as_object() else {
                return Err(format!("component `{name}` must be an object"));
            };
            // Component records using the typed parts/styles envelope receive
            // semantic validation. Opaque extension records remain preserved.
            if object.contains_key("parts") || object.contains_key("styles") {
                let record = self.component_theme(name)?;
                super::resolver::validate_component_theme(self, name, &record)
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                if name == "slider" {
                    validate_slider_theme_roles(&record.metadata)?;
                }
                if name == "text" {
                    validate_text_renderer_roles(&record)
                        .map_err(|error| format!("invalid `text` component theme: {error}"))?;
                }
                if name == "link" {
                    validate_renderer_required_properties(
                        name,
                        &record,
                        &["foreground", "typography"],
                    )
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "label" {
                    validate_renderer_required_properties(
                        name,
                        &record,
                        &["foreground", "typography"],
                    )
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "menu" {
                    validate_renderer_required_properties_for_part(
                        name,
                        &record,
                        "body",
                        &["background"],
                    )
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "text_field" {
                    validate_text_field_renderer_rules(&record)
                        .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "search_field" {
                    validate_search_field_renderer_rules(&record)
                        .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                    validate_renderer_required_properties_for_part(
                        name,
                        &record,
                        "list_search",
                        &[
                            "width",
                            "height",
                            "corner_radius",
                            "background",
                            "foreground",
                            "typography",
                        ],
                    )
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "stepper" {
                    validate_stepper_renderer_rules(&record)
                        .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "outline_view" {
                    for appearance in ["light", "dark"] {
                        let has_matching_rule = record.styles.iter().any(|rule| {
                            rule.part == "outline_view"
                                && (rule.when.appearance == "any"
                                    || rule.when.appearance == appearance)
                        });
                        if !has_matching_rule {
                            return Err(format!(
                                "outline_view is missing renderer-required `outline_view` style rule for appearance `{appearance}`"
                            ));
                        }
                    }
                }
                if name == "checkbox" {
                    validate_renderer_required_properties(
                        name,
                        &record,
                        &["background", "foreground", "typography", "vector_path"],
                    )
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                    validate_checkbox_radio_artwork(name, &record)
                        .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "toggle" {
                    validate_renderer_required_properties(
                        name,
                        &record,
                        &["background", "width", "height", "corner_radius"],
                    )
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
                if name == "icon" {
                    if !record.styles.iter().any(|rule| rule.part == "clear") {
                        return Err(
                            "invalid `icon` component theme: missing renderer-required `clear` style rule"
                                .into(),
                        );
                    }
                    for (index, rule) in record
                        .styles
                        .iter()
                        .enumerate()
                        .filter(|(_, rule)| rule.part == "clear")
                    {
                        let path_data = rule
                            .style
                            .properties
                            .get("vector_path")
                            .and_then(|property| property.get("value"))
                            .and_then(|value| value.get("data"))
                            .and_then(serde_json::Value::as_str)
                            .filter(|data| !data.trim().is_empty());
                        if path_data.is_none() {
                            return Err(format!(
                                "invalid `icon` component theme: clear style rule {index} requires nonblank renderer `vector_path.value.data`"
                            ));
                        }
                    }
                }
                if name == "color_picker" {
                    validate_renderer_required_properties(name, &record, &["width", "height"])
                        .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                    for (index, rule) in record
                        .styles
                        .iter()
                        .enumerate()
                        .filter(|(_, rule)| rule.part == "color_wheel")
                    {
                        for property in ["corner_radius", "background"] {
                            if !rule.style.properties.contains_key(property) {
                                return Err(format!(
                                    "color_picker style rule {index} is missing renderer-required `{property}` property"
                                ));
                            }
                        }
                    }
                }
                if name == "radio_button" {
                    validate_renderer_required_properties(
                        name,
                        &record,
                        &[
                            "background",
                            "foreground",
                            "typography",
                            "width",
                            "height",
                            "corner_radius",
                            "vector_path",
                        ],
                    )
                    .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                    validate_checkbox_radio_artwork(name, &record)
                        .map_err(|error| format!("invalid `{name}` component theme: {error}"))?;
                }
            }
        }
        self.platform_snapshot.validate()?;

        // The resolver reads the document directly and does not call validate,
        // so both appearance paths can be checked here without recursion.
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let appearance = match mode {
                AppearanceMode::Light => "light",
                AppearanceMode::Dark => "dark",
            };
            super::resolver::resolve_button(self, Some(mode))
                .map_err(|error| format!("invalid button theme for {appearance}: {error}"))?;
        }

        Ok(())
    }

    pub fn button_theme(&self) -> Result<ButtonTheme, String> {
        let value = self
            .components
            .get("button")
            .ok_or_else(|| "theme must contain a button component".to_owned())?;
        serde_json::from_value(value.clone())
            .map_err(|error| format!("invalid button component theme: {error}"))
    }

    pub fn component_theme(&self, name: &str) -> Result<ComponentThemeRecord, String> {
        let value = self
            .components
            .get(name)
            .ok_or_else(|| format!("theme has no `{name}` component"))?;
        serde_json::from_value(value.clone())
            .map_err(|error| format!("invalid `{name}` component theme: {error}"))
    }

    /// Select this document's platform snapshot when it matches the host version.
    pub fn select_profile(&self, platform: &str, version: u32) -> Option<&ThemeProfile> {
        self.platform_snapshot
            .matches(platform, version)
            .then_some(&self.platform_snapshot)
    }
}

fn validate_nonempty_component_selectors(
    components: &BTreeMap<String, serde_json::Value>,
) -> Result<(), String> {
    const FIELDS: [&str; 8] = [
        "appearance",
        "material",
        "interaction",
        "text_presence",
        "size",
        "orientation",
        "chrome",
        "modality",
    ];

    for (component, record) in components {
        let Some(rules) = record.get("styles").and_then(serde_json::Value::as_array) else {
            continue;
        };
        for (index, rule) in rules.iter().enumerate() {
            let Some(selector) = rule.get("when").and_then(serde_json::Value::as_object) else {
                continue;
            };
            for field in FIELDS {
                if selector
                    .get(field)
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|value| value.trim().is_empty())
                {
                    return Err(format!(
                        "{component} style rule {index} has an empty `{field}` selector"
                    ));
                }
            }
        }
    }
    Ok(())
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonTheme {
    pub rustui_type: String,
    pub metrics: ButtonMetricReferences,
    pub theme_roles: ButtonThemeRoles,
    pub parts: BTreeMap<String, serde_json::Value>,
    pub styles: Vec<ButtonStyleRule>,
}

impl ButtonTheme {
    fn validate(&self) -> Result<(), String> {
        if self.rustui_type != "Button" {
            return Err("button component rustui_type must be `Button`".into());
        }
        if !self.parts.contains_key("body") {
            return Err("button component must define its body part".into());
        }
        if self.styles.is_empty() {
            return Err("button component must define at least one style rule".into());
        }
        for (index, rule) in self.styles.iter().enumerate() {
            rule.validate()?;
            for (other_index, other) in self.styles.iter().enumerate().skip(index + 1) {
                if super::resolver::selector_specificity(&rule.when)
                    == super::resolver::selector_specificity(&other.when)
                    && super::resolver::component_selectors_overlap(&rule.when, &other.when)
                {
                    return Err(format!(
                        "button style rules {index} and {other_index} have ambiguous selectors"
                    ));
                }
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ButtonMetricReferences {
    pub height: String,
    pub horizontal_padding: String,
    pub corner_radius: String,
    pub label: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ButtonThemeRoles {
    pub background: String,
    pub foreground: String,
    pub hover: String,
    pub pressed: String,
    pub disabled: String,
    pub focus: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonStyleRule {
    pub part: String,
    pub when: ButtonStyleSelector,
    pub style: ButtonStyleDefinition,
}

impl ButtonStyleRule {
    fn validate(&self) -> Result<(), String> {
        if self.part != "body" {
            return Err(format!(
                "button style rule uses unsupported part `{}`",
                self.part
            ));
        }
        if !matches!(self.when.appearance.as_str(), "any" | "light" | "dark") {
            return Err(format!(
                "button style rule has unsupported appearance selector `{}`",
                self.when.appearance
            ));
        }
        self.style.properties.validate()
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ButtonStyleSelector {
    pub appearance: String,
    pub material: String,
    pub interaction: String,
    pub text_presence: String,
    pub size: String,
    pub orientation: String,
    pub chrome: String,
    pub modality: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonStyleDefinition {
    pub properties: ButtonStyleProperties,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonStyleProperties {
    pub width: LengthProperty,
    pub height: LengthProperty,
    pub background: ColorProperty,
    pub foreground: ColorProperty,
    pub typography: TypographyProperty,
}

impl ButtonStyleProperties {
    fn validate(&self) -> Result<(), String> {
        self.width.validate("width")?;
        self.height.validate("height")?;
        self.background.validate("background")?;
        self.foreground.validate("foreground")?;
        self.typography.validate()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LengthProperty {
    pub kind: LengthKind,
    pub value: LengthValue,
}

impl LengthProperty {
    fn validate(&self, name: &str) -> Result<(), String> {
        if self.value.px.is_finite() && self.value.px > 0.0 {
            Ok(())
        } else {
            Err(format!("button {name} must be a positive finite length"))
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LengthKind {
    Length,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct LengthValue {
    pub px: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ColorProperty {
    pub kind: ColorKind,
    pub value: ColorValue,
}

impl ColorProperty {
    fn validate(&self, name: &str) -> Result<(), String> {
        self.value.rgba.validate(name)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ColorKind {
    Color,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ColorValue {
    pub rgba: RgbaValue,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RgbaValue {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RgbaValue {
    fn validate(&self, name: &str) -> Result<(), String> {
        if [self.r, self.g, self.b, self.a]
            .into_iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
        {
            Ok(())
        } else {
            Err(format!(
                "button {name} RGBA channels must be between 0 and 1"
            ))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TypographyProperty {
    pub kind: TypographyKind,
    pub value: ButtonTypographyValue,
}

impl TypographyProperty {
    fn validate(&self) -> Result<(), String> {
        if self.value.family.trim().is_empty()
            || !self.value.size.is_finite()
            || self.value.size <= 0.0
            || !self.value.weight.is_finite()
            || !(1.0..=1000.0).contains(&self.value.weight)
        {
            return Err(
                "button typography must have a family, positive size, and valid weight".into(),
            );
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TypographyKind {
    Typography,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ButtonTypographyValue {
    pub family: String,
    pub size: f32,
    pub weight: f32,
}

/// A component record whose non-style metadata remains available to callers.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ComponentThemeRecord {
    pub rustui_type: String,
    pub parts: BTreeMap<String, serde_json::Value>,
    #[serde(default)]
    pub styles: Vec<ComponentStyleRule>,
    #[serde(flatten)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ComponentStyleRule {
    pub part: String,
    pub when: ButtonStyleSelector,
    pub style: ComponentStyleDefinition,
    #[serde(flatten)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct ComponentStyleDefinition {
    pub properties: BTreeMap<String, serde_json::Value>,
    #[serde(flatten)]
    pub metadata: BTreeMap<String, serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FrameworkIdentity {
    pub name: String,
    pub version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeProfile {
    pub id: String,
    pub platform: String,
    #[serde(deserialize_with = "deserialize_integral_u32")]
    pub min_version: u32,
    #[serde(default, deserialize_with = "deserialize_optional_integral_u32")]
    pub max_version: Option<u32>,
    pub measure_unit: MeasureUnit,
    pub appearance: Appearance,
    pub dimensions: Dimensions,
    pub radii: Radii,
    pub borders: Borders,
    pub spacing: Spacing,
    pub materials: Materials,
    pub accents: Accents,
    pub window_controls: WindowControls,
    pub accessibility: Accessibility,
    pub typography: Typography,
}

impl ThemeProfile {
    pub fn matches(&self, platform: &str, version: u32) -> bool {
        self.platform == platform
            && version >= self.min_version
            && self.max_version.is_none_or(|maximum| version <= maximum)
    }

    /// Resolve explicit light/dark mode or this profile's declared default.
    pub fn colors(&self, mode: Option<AppearanceMode>) -> &AppearanceColors {
        match mode.unwrap_or(self.appearance.default) {
            AppearanceMode::Light => &self.appearance.light,
            AppearanceMode::Dark => &self.appearance.dark,
        }
    }

    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("profile id must not be empty".into());
        }
        if !matches!(
            self.platform.as_str(),
            "macos" | "ios" | "ipados" | "windows" | "linux" | "android" | "web"
        ) {
            return Err(format!(
                "profile `{}` uses unsupported platform `{}`",
                self.id, self.platform
            ));
        }
        if self
            .max_version
            .is_some_and(|maximum| maximum < self.min_version)
        {
            return Err(format!(
                "profile `{}` has max_version below min_version",
                self.id
            ));
        }

        self.appearance.light.validate(&self.id, "light")?;
        self.appearance.dark.validate(&self.id, "dark")?;
        self.dimensions.validate(&self.id)?;
        self.radii.validate(&self.id)?;
        self.borders.validate(&self.id)?;
        self.spacing.validate(&self.id)?;
        self.materials.validate(&self.id)?;
        self.accents.validate(&self.id)?;
        self.window_controls.validate(&self.id)?;
        self.accessibility.validate(&self.id)?;
        self.typography.validate(&self.id)
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MeasureUnit {
    LogicalPx,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AppearanceMode {
    Light,
    Dark,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Appearance {
    pub default: AppearanceMode,
    pub light: AppearanceColors,
    pub dark: AppearanceColors,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AppearanceColors {
    pub window_background: String,
    pub sidebar_background: String,
    pub content_background: String,
    pub elevated_panel_background: String,
    pub toolbar_background: String,
    pub selected_row_background: String,
    pub hover_row_background: String,
    pub pressed_row_background: String,
    pub separator: String,
    pub border: String,
    pub primary_text: String,
    pub secondary_text: String,
    pub tertiary_text: String,
    pub accent: String,
    pub success: String,
    pub warning: String,
    pub error: String,
    pub disabled_text: String,
    pub focus_ring: String,
    pub badge_background: String,
}

impl AppearanceColors {
    fn validate(&self, profile: &str, mode: &str) -> Result<(), String> {
        let colors = [
            ("window_background", &self.window_background),
            ("sidebar_background", &self.sidebar_background),
            ("content_background", &self.content_background),
            ("elevated_panel_background", &self.elevated_panel_background),
            ("toolbar_background", &self.toolbar_background),
            ("selected_row_background", &self.selected_row_background),
            ("hover_row_background", &self.hover_row_background),
            ("pressed_row_background", &self.pressed_row_background),
            ("separator", &self.separator),
            ("border", &self.border),
            ("primary_text", &self.primary_text),
            ("secondary_text", &self.secondary_text),
            ("tertiary_text", &self.tertiary_text),
            ("accent", &self.accent),
            ("success", &self.success),
            ("warning", &self.warning),
            ("error", &self.error),
            ("disabled_text", &self.disabled_text),
            ("focus_ring", &self.focus_ring),
            ("badge_background", &self.badge_background),
        ];
        for (name, value) in colors {
            validate_hex(value, &format!("profile `{profile}` {mode}.{name}"))?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    pub sidebar_width: f64,
    pub toolbar_height: f64,
    pub navigation_row_height: f64,
    pub control_height: f64,
    pub window_control_inset: f64,
    pub status_dot: f64,
    pub badge_size: f64,
    pub avatar_size: f64,
    pub sidebar_toggle_width: f64,
    pub sidebar_toggle_inset: f64,
    pub window_traffic_light_inset: f64,
    pub sidebar_toggle_height: f64,
    pub window_traffic_light_width: f64,
    pub window_traffic_light_gap: f64,
}

impl Dimensions {
    fn validate(&self, profile: &str) -> Result<(), String> {
        let values = [
            self.sidebar_width,
            self.toolbar_height,
            self.navigation_row_height,
            self.control_height,
            self.window_control_inset,
            self.status_dot,
            self.badge_size,
            self.avatar_size,
            self.sidebar_toggle_width,
            self.sidebar_toggle_inset,
            self.window_traffic_light_inset,
            self.sidebar_toggle_height,
            self.window_traffic_light_width,
            self.window_traffic_light_gap,
        ];
        validate_positive_numbers(profile, "dimensions", values)?;
        if values.iter().any(|value| {
            let narrowed = *value as f32;
            !narrowed.is_finite() || narrowed <= 0.0
        }) {
            return Err(format!(
                "profile `{profile}` dimensions values must be representable as f32"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Radii {
    pub sm: f64,
    pub md: f64,
    pub lg: f64,
    pub segment: f64,
    pub window: f64,
}

impl Radii {
    fn validate(&self, profile: &str) -> Result<(), String> {
        let values = [self.sm, self.md, self.lg, self.segment, self.window];
        if values.iter().any(|value| {
            let narrowed = *value as f32;
            !value.is_finite() || *value <= 0.0 || !narrowed.is_finite() || narrowed <= 0.0
        }) {
            return Err(format!(
                "profile `{profile}` radii values must be positive, finite, and representable as f32"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Borders {
    pub thin: f64,
    pub medium: f64,
}

impl Borders {
    fn validate(&self, profile: &str) -> Result<(), String> {
        let values = [self.thin, self.medium];
        validate_nonnegative_numbers(profile, "borders", values)?;
        if values.iter().any(|value| {
            let narrowed = *value as f32;
            !narrowed.is_finite() || (*value > 0.0 && narrowed == 0.0)
        }) {
            return Err(format!(
                "profile `{profile}` border widths must be representable as f32"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Spacing {
    pub space_1: f64,
    pub space_2: f64,
    pub space_3: f64,
    pub space_4: f64,
    pub space_5: f64,
    pub space_6: f64,
}

impl Spacing {
    fn validate(&self, profile: &str) -> Result<(), String> {
        let values = [
            self.space_1,
            self.space_2,
            self.space_3,
            self.space_4,
            self.space_5,
            self.space_6,
        ];
        validate_nonnegative_numbers(profile, "spacing", values)?;
        if values.iter().any(|value| {
            let narrowed = *value as f32;
            !narrowed.is_finite() || (*value > 0.0 && narrowed == 0.0)
        }) {
            return Err(format!(
                "profile `{profile}` spacing values must be representable as f32"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Materials {
    pub material_opacity: f64,
    pub sidebar_blur_radius: f64,
    pub sidebar_opacity: f64,
    pub toolbar_blur_radius: f64,
    pub toolbar_opacity: f64,
}

impl Materials {
    fn validate(&self, profile: &str) -> Result<(), String> {
        for (name, value) in [
            ("material_opacity", self.material_opacity),
            ("sidebar_opacity", self.sidebar_opacity),
            ("toolbar_opacity", self.toolbar_opacity),
        ] {
            let narrowed = value as f32;
            if !value.is_finite()
                || !(0.0..=1.0).contains(&value)
                || !narrowed.is_finite()
                || (value > 0.0 && narrowed == 0.0)
            {
                return Err(format!(
                    "profile `{profile}` materials.{name} must be between 0 and 1 and representable as f32"
                ));
            }
        }
        validate_nonnegative_numbers(
            profile,
            "material blur radii",
            [self.sidebar_blur_radius, self.toolbar_blur_radius],
        )
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Accents {
    pub blue: AccentColors,
    pub graphite: AccentColors,
    pub green: AccentColors,
    pub orange: AccentColors,
    pub pink: AccentColors,
    pub purple: AccentColors,
    pub red: AccentColors,
    pub selected: String,
    pub yellow: AccentColors,
}

impl Accents {
    fn validate(&self, profile: &str) -> Result<(), String> {
        for (name, colors) in [
            ("blue", &self.blue),
            ("graphite", &self.graphite),
            ("green", &self.green),
            ("orange", &self.orange),
            ("pink", &self.pink),
            ("purple", &self.purple),
            ("red", &self.red),
            ("yellow", &self.yellow),
        ] {
            validate_hex(
                &colors.light,
                &format!("profile `{profile}` accents.{name}.light"),
            )?;
            validate_hex(
                &colors.dark,
                &format!("profile `{profile}` accents.{name}.dark"),
            )?;
        }
        if ![
            "blue", "graphite", "green", "orange", "pink", "purple", "red", "yellow",
        ]
        .contains(&self.selected.as_str())
        {
            return Err(format!(
                "profile `{profile}` selects unknown accent `{}`",
                self.selected
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AccentColors {
    pub light: String,
    pub dark: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct WindowControls {
    pub close: String,
    pub inactive: String,
    pub minimize: String,
    pub zoom: String,
}

impl WindowControls {
    fn validate(&self, profile: &str) -> Result<(), String> {
        for (name, value) in [
            ("close", &self.close),
            ("inactive", &self.inactive),
            ("minimize", &self.minimize),
            ("zoom", &self.zoom),
        ] {
            validate_hex(
                value,
                &format!("profile `{profile}` window_controls.{name}"),
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Accessibility {
    pub focus_offset: f64,
    pub focus_width: f64,
    pub min_body_contrast: f64,
    pub min_focus_contrast: f64,
    pub min_large_contrast: f64,
    pub reduced_motion: ReducedMotion,
}

impl Accessibility {
    fn validate(&self, profile: &str) -> Result<(), String> {
        validate_nonnegative_numbers(
            profile,
            "accessibility",
            [self.focus_offset, self.focus_width],
        )?;
        validate_positive_numbers(
            profile,
            "accessibility contrast thresholds",
            [
                self.min_body_contrast,
                self.min_focus_contrast,
                self.min_large_contrast,
            ],
        )?;
        let narrowed_focus_width = self.focus_width as f32;
        if !narrowed_focus_width.is_finite()
            || (self.focus_width > 0.0 && narrowed_focus_width == 0.0)
        {
            return Err(format!(
                "profile `{profile}` accessibility focus width must be representable as f32"
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReducedMotion {
    System,
    Reduce,
    Allow,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Typography {
    pub font_family: String,
    pub font_family_display: String,
    #[serde(deserialize_with = "deserialize_integral_u16")]
    pub font_weight_normal: u16,
    #[serde(deserialize_with = "deserialize_integral_u16")]
    pub font_weight_medium: u16,
    #[serde(deserialize_with = "deserialize_integral_u16")]
    pub font_weight_semibold: u16,
    #[serde(deserialize_with = "deserialize_integral_u16")]
    pub font_weight_bold: u16,
    #[serde(deserialize_with = "deserialize_integral_u16")]
    pub font_weight_heavy: u16,
    pub styles: BTreeMap<String, FontStyle>,
    pub font_size_page: f64,
    pub font_size_title: f64,
    pub font_size_body: f64,
    pub font_size_small: f64,
    pub font_size_caption: f64,
}

impl Typography {
    fn validate(&self, profile: &str) -> Result<(), String> {
        if self.font_family.trim().is_empty() || self.font_family_display.trim().is_empty() {
            return Err(format!(
                "profile `{profile}` font families must not be empty"
            ));
        }
        validate_weights(
            profile,
            "typography",
            [
                self.font_weight_normal,
                self.font_weight_medium,
                self.font_weight_semibold,
                self.font_weight_bold,
                self.font_weight_heavy,
            ],
        )?;
        let font_sizes = [
            self.font_size_page,
            self.font_size_title,
            self.font_size_body,
            self.font_size_small,
            self.font_size_caption,
        ];
        validate_positive_numbers(profile, "typography font sizes", font_sizes)?;
        if font_sizes.iter().any(|size| {
            let narrowed = *size as f32;
            !narrowed.is_finite() || narrowed <= 0.0
        }) {
            return Err(format!(
                "profile `{profile}` typography font sizes must be representable as f32"
            ));
        }
        if self.styles.is_empty() {
            return Err(format!("profile `{profile}` must define text styles"));
        }
        for (name, style) in &self.styles {
            if !is_style_name(name) {
                return Err(format!(
                    "profile `{profile}` has invalid text style name `{name}`"
                ));
            }
            let narrowed_size = style.size as f32;
            if !style.size.is_finite()
                || style.size <= 0.0
                || !narrowed_size.is_finite()
                || narrowed_size <= 0.0
            {
                return Err(format!(
                    "profile `{profile}` typography style `{name}` size must be positive and representable as f32"
                ));
            }
            validate_weights(
                profile,
                &format!("typography style `{name}`"),
                [style.weight],
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct FontStyle {
    pub size: f64,
    #[serde(deserialize_with = "deserialize_integral_u16")]
    pub weight: u16,
}

fn validate_hex(value: &str, field: &str) -> Result<(), String> {
    if value.len() != 7
        || !value.starts_with('#')
        || !value[1..].bytes().all(|byte| byte.is_ascii_hexdigit())
    {
        return Err(format!("{field} must be a six-digit hex color"));
    }
    Ok(())
}

fn is_semver(version: &str) -> bool {
    let parts = version.split('.').collect::<Vec<_>>();
    parts.len() == 3
        && parts.iter().all(|part| {
            !part.is_empty()
                && part.bytes().all(|byte| byte.is_ascii_digit())
                && (*part == "0" || !part.starts_with('0'))
        })
}

fn is_style_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(byte) if byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn validate_positive_numbers<const N: usize>(
    profile: &str,
    field: &str,
    values: [f64; N],
) -> Result<(), String> {
    if values
        .iter()
        .any(|value| !value.is_finite() || *value <= 0.0)
    {
        return Err(format!(
            "profile `{profile}` {field} values must be positive"
        ));
    }
    Ok(())
}

fn validate_nonnegative_numbers<const N: usize>(
    profile: &str,
    field: &str,
    values: [f64; N],
) -> Result<(), String> {
    if values
        .iter()
        .any(|value| !value.is_finite() || *value < 0.0)
    {
        return Err(format!(
            "profile `{profile}` {field} values must be nonnegative"
        ));
    }
    Ok(())
}

fn validate_weights<const N: usize>(
    profile: &str,
    field: &str,
    values: [u16; N],
) -> Result<(), String> {
    if values.iter().any(|weight| !(1..=1000).contains(weight)) {
        return Err(format!(
            "profile `{profile}` {field} font weights must be from 1 through 1000"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::{
        Accessibility, Borders, Dimensions, FontStyle, Materials, Radii, ReducedMotion, Spacing,
        ThemeDocument, Typography, is_semver,
    };

    #[test]
    fn theme_roles_must_not_be_empty() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.theme_roles.clear();

        assert_eq!(
            document.validate().unwrap_err(),
            "theme must contain at least one theme role"
        );
    }

    #[test]
    fn slider_theme_roles_must_be_supported_by_runtime() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("slider").unwrap()["theme_roles"]["thumb"] =
            serde_json::json!("unsupported");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("slider theme_roles.thumb")
        );
    }

    #[test]
    fn text_theme_requires_renderer_foreground_role() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let styles = document.components.get_mut("text").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let body = styles
            .iter_mut()
            .find(|rule| rule["part"] == "text_styles_left_aligned_light_opaque_body")
            .unwrap();
        body["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("foreground");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("text style rule 0 is missing renderer-required `foreground` role")
        );
    }

    #[test]
    fn link_theme_requires_renderer_foreground_and_typography() {
        for property in ["foreground", "typography"] {
            let mut document: ThemeDocument =
                crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
            let styles = document.components.get_mut("link").unwrap()["styles"]
                .as_array_mut()
                .unwrap();
            let body = styles
                .iter_mut()
                .find(|rule| rule["part"] == "body")
                .unwrap();
            body["style"]["properties"]
                .as_object_mut()
                .unwrap()
                .remove(property);

            assert!(document.validate().unwrap_err().contains(&format!(
                "link style rule 0 is missing renderer-required `{property}` property"
            )));
        }
    }

    #[test]
    fn label_theme_requires_renderer_foreground_and_typography() {
        for property in ["foreground", "typography"] {
            let mut document: ThemeDocument =
                crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
            let styles = document.components.get_mut("label").unwrap()["styles"]
                .as_array_mut()
                .unwrap();
            let body = styles
                .iter_mut()
                .find(|rule| rule["part"] == "text_styles_left_aligned_light_opaque_label_color")
                .unwrap();
            body["style"]["properties"]
                .as_object_mut()
                .unwrap()
                .remove(property);

            assert!(document.validate().unwrap_err().contains(&format!(
                "label style rule 0 is missing renderer-required `{property}` property"
            )));
        }
    }

    #[test]
    fn menu_body_theme_requires_renderer_background() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let styles = document.components.get_mut("menu").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let body = styles
            .iter_mut()
            .find(|rule| rule["part"] == "body")
            .unwrap();
        body["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("background");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("menu style rule 0 is missing renderer-required `background` property")
        );
    }

    #[test]
    fn text_field_theme_requires_each_renderer_appearance_and_size_rule() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("text_field").unwrap()["styles"]
            .as_array_mut()
            .unwrap()
            .retain(|rule| {
                !(rule["part"] == "body"
                    && rule["when"]["appearance"] == "dark"
                    && rule["when"]["size"] == "small")
            });

        assert!(document.validate().unwrap_err().contains(
            "text_field is missing renderer-required `body` style rule for appearance `dark` and size `small`"
        ));
    }

    #[test]
    fn component_theme_rejects_empty_selector_values() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("text_field").unwrap()["styles"][0]["when"]["interaction"] =
            serde_json::json!(" ");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("text_field style rule 0 has an empty `interaction` selector")
        );
    }

    #[test]
    fn component_theme_rejects_unresolvable_appearance_selectors() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("divider").unwrap()["styles"][0]["when"]["appearance"] =
            serde_json::json!("system");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("style rule 0 has unsupported appearance selector `system`")
        );
    }

    #[test]
    fn search_field_theme_requires_renderer_rules_and_properties() {
        let mut missing_rule: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        missing_rule.components.get_mut("search_field").unwrap()["styles"]
            .as_array_mut()
            .unwrap()
            .retain(|rule| {
                !(rule["part"] == "list_search" && rule["when"]["appearance"] == "light")
            });
        assert!(missing_rule.validate().unwrap_err().contains(
            "search_field is missing renderer-required `list_search` style rule for appearance `light` and size `small`"
        ));

        let mut missing_property: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        missing_property.components.get_mut("search_field").unwrap()["styles"]
            .as_array_mut()
            .unwrap()
            .iter_mut()
            .find(|rule| rule["part"] == "list_search" && rule["when"]["appearance"] == "light")
            .unwrap()["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("height");
        assert!(
            missing_property.validate().unwrap_err().contains(
                "search_field style rule 3 is missing renderer-required `height` property"
            )
        );
    }

    #[test]
    fn stepper_theme_requires_renderer_rules_for_both_appearances() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("stepper").unwrap()["styles"]
            .as_array_mut()
            .unwrap()
            .retain(|rule| {
                !(rule["part"] == "increment_glyph" && rule["when"]["appearance"] == "dark")
            });

        assert!(document.validate().unwrap_err().contains(
            "stepper is missing renderer-required `increment_glyph` style rule for appearance `dark`"
        ));
    }

    #[test]
    fn outline_view_theme_requires_renderer_style_for_both_appearances() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("outline_view").unwrap()["styles"]
            .as_array_mut()
            .unwrap()
            .retain(|rule| {
                !(rule["part"] == "outline_view" && rule["when"]["appearance"] == "dark")
            });

        assert!(document.validate().unwrap_err().contains(
            "outline_view is missing renderer-required `outline_view` style rule for appearance `dark`"
        ));
    }

    #[test]
    fn checkbox_theme_requires_renderer_background_color() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let styles = document.components.get_mut("checkbox").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let body = styles
            .iter_mut()
            .find(|rule| rule["part"] == "body")
            .unwrap();
        body["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("background");

        assert!(
            document.validate().unwrap_err().contains(
                "checkbox style rule 0 is missing renderer-required `background` property"
            )
        );
    }

    #[test]
    fn toggle_theme_requires_renderer_geometry() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let styles = document.components.get_mut("toggle").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let body = styles
            .iter_mut()
            .find(|rule| rule["part"] == "body")
            .unwrap();
        body["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("corner_radius");

        assert!(
            document.validate().unwrap_err().contains(
                "toggle style rule 0 is missing renderer-required `corner_radius` property"
            )
        );
    }

    #[test]
    fn icon_theme_requires_a_renderer_clear_style_rule() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("icon").unwrap()["styles"]
            .as_array_mut()
            .unwrap()
            .retain(|rule| rule["part"] != "clear");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("missing renderer-required `clear` style rule")
        );
    }

    #[test]
    fn icon_clear_theme_requires_renderer_path_data() {
        let document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();

        for path in [None, Some(serde_json::json!({"value": {"data": " "}}))] {
            let mut invalid = document.clone();
            let clear = invalid.components.get_mut("icon").unwrap()["styles"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|rule| rule["part"] == "clear")
                .unwrap();
            let properties = clear["style"]["properties"].as_object_mut().unwrap();
            if let Some(path) = path {
                properties.insert("vector_path".into(), path);
            } else {
                properties.remove("vector_path");
            }

            assert!(invalid.validate().unwrap_err().contains(
                "clear style rule 3 requires nonblank renderer `vector_path.value.data`"
            ));
        }
    }

    #[test]
    fn radio_theme_requires_renderer_geometry() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let styles = document.components.get_mut("radio_button").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let body = styles
            .iter_mut()
            .find(|rule| rule["part"] == "body")
            .unwrap();
        body["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("corner_radius");

        assert!(document.validate().unwrap_err().contains(
            "radio_button style rule 0 is missing renderer-required `corner_radius` property"
        ));
    }

    #[test]
    fn checkbox_and_radio_themes_require_renderer_artwork() {
        let document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();

        for component in ["checkbox", "radio_button"] {
            let mut invalid = document.clone();
            let styles = invalid.components.get_mut(component).unwrap()["styles"]
                .as_array_mut()
                .unwrap();
            let body = styles
                .iter_mut()
                .find(|rule| rule["part"] == "body")
                .unwrap();
            body["style"]["properties"]
                .as_object_mut()
                .unwrap()
                .remove("vector_path");

            assert!(invalid.validate().unwrap_err().contains(&format!(
                "{component} style rule 0 is missing renderer-required `vector_path` property"
            )));
        }
    }

    #[test]
    fn checkbox_and_radio_themes_require_body_style_rules() {
        let document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();

        for component in ["checkbox", "radio_button"] {
            let mut invalid = document.clone();
            invalid.components.get_mut(component).unwrap()["styles"]
                .as_array_mut()
                .unwrap()
                .retain(|rule| rule["part"] != "body");

            assert!(invalid.validate().unwrap_err().contains(&format!(
                "{component} is missing renderer-required `body` style rule"
            )));
        }
    }

    #[test]
    fn checkbox_and_radio_themes_require_renderer_typography() {
        let document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();

        for component in ["checkbox", "radio_button"] {
            let mut invalid = document.clone();
            let body = invalid.components.get_mut(component).unwrap()["styles"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|rule| rule["part"] == "body")
                .unwrap();
            body["style"]["properties"]
                .as_object_mut()
                .unwrap()
                .remove("typography");

            assert!(invalid.validate().unwrap_err().contains(&format!(
                "{component} style rule 0 is missing renderer-required `typography` property"
            )));
        }
    }

    #[test]
    fn checkbox_and_radio_themes_reject_unsupported_renderer_artwork() {
        let document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let invalid_artwork = [
            serde_json::json!({
                "value": {
                    "data": "M0 0 L16 16",
                    "transform": [1, 0, 0, 1, 0, 0]
                }
            }),
            serde_json::json!({
                "value": {
                    "data": "M0 0 Hx V16 H0 V0 Z",
                    "transform": [1, 0, 0, 1, 0, 0]
                }
            }),
            serde_json::json!({
                "value": {
                    "data": "M0 0 H16 V16 H0 V0 Z",
                    "transform": [1, 0, 0, 1, 1, 0]
                }
            }),
        ];

        for component in ["checkbox", "radio_button"] {
            for artwork in &invalid_artwork {
                let mut invalid = document.clone();
                let body = invalid.components.get_mut(component).unwrap()["styles"]
                    .as_array_mut()
                    .unwrap()
                    .iter_mut()
                    .find(|rule| rule["part"] == "body")
                    .unwrap();
                body["style"]["properties"]["vector_path"] = artwork.clone();

                assert!(
                    invalid
                        .validate()
                        .unwrap_err()
                        .contains("renderer `vector_path`")
                );
            }
        }
    }

    #[test]
    fn color_picker_theme_requires_renderer_dimensions() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let styles = document.components.get_mut("color_picker").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let swatches = styles
            .iter_mut()
            .find(|rule| rule["part"] == "swatches")
            .unwrap();
        swatches["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("height");

        assert!(
            document.validate().unwrap_err().contains(
                "color_picker style rule 1 is missing renderer-required `height` property"
            )
        );
    }

    #[test]
    fn color_picker_wheel_requires_renderer_shape_and_fill() {
        for property in ["corner_radius", "background"] {
            let mut document: ThemeDocument =
                crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
            let wheel = document.components.get_mut("color_picker").unwrap()["styles"]
                .as_array_mut()
                .unwrap()
                .iter_mut()
                .find(|rule| rule["part"] == "color_wheel")
                .unwrap();
            wheel["style"]["properties"]
                .as_object_mut()
                .unwrap()
                .remove(property);

            assert!(document.validate().unwrap_err().contains(&format!(
                "color_picker style rule 0 is missing renderer-required `{property}` property"
            )));
        }
    }

    #[test]
    fn button_theme_rejects_variant_only_ambiguous_selectors() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let styles = document.components.get_mut("button").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        styles[0]["when"]["appearance"] = serde_json::json!("dark");
        styles[1]["when"]["appearance"] = serde_json::json!("any");
        styles[1]["when"]["material"] = serde_json::json!("opaque");

        let mut query = document.button_theme().unwrap().styles[0].when.clone();
        query.material = "opaque".into();
        assert!(matches!(
            crate::theme::resolve_button_with_selector(&document, &query),
            Err(crate::theme::ThemeResolveError::Invalid(message))
                if message.contains("ambiguous body styles")
        ));
        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("button style rules 0 and 1 have ambiguous selectors")
        );
    }

    #[test]
    fn button_theme_rejects_unresolvable_appearance_selectors() {
        let mut document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        document.components.get_mut("button").unwrap()["styles"][0]["when"]["appearance"] =
            serde_json::json!("system");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("button style rule has unsupported appearance selector `system`")
        );

        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../../themes/theme.schema.json")).unwrap();
        assert_eq!(
            schema["$defs"]["buttonTheme"]["properties"]["styles"]["items"]["properties"]["when"]["properties"]
                ["appearance"]["enum"],
            serde_json::json!(["any", "light", "dark"])
        );
    }

    #[test]
    fn component_theme_schema_restricts_appearance_selectors_for_known_components() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../../themes/theme.schema.json")).unwrap();

        let rule = &schema["$defs"]["componentTheme"]["properties"]["styles"]["items"];
        assert_eq!(
            rule["required"],
            serde_json::json!(["part", "when", "style"])
        );
        assert_eq!(rule["properties"]["part"]["minLength"], 1);
        assert_eq!(
            rule["properties"]["style"]["required"],
            serde_json::json!(["properties"])
        );
        let selector = &rule["properties"]["when"];
        assert_eq!(
            selector["required"],
            serde_json::json!([
                "appearance",
                "material",
                "interaction",
                "text_presence",
                "size",
                "orientation",
                "chrome",
                "modality"
            ])
        );
        for field in [
            "material",
            "interaction",
            "text_presence",
            "size",
            "orientation",
            "chrome",
            "modality",
        ] {
            assert_eq!(selector["properties"][field]["minLength"], 1, "{field}");
        }
        assert_eq!(selector["additionalProperties"], false);
        assert_eq!(
            selector["properties"]["appearance"]["enum"],
            serde_json::json!(["any", "light", "dark"])
        );
        assert_eq!(
            schema["$defs"]["theme"]["properties"]["components"]["properties"]["divider"]["$ref"],
            "#/$defs/componentTheme"
        );
    }

    #[test]
    fn metadata_fields_must_be_objects() {
        let document: ThemeDocument =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();

        for field in ["source", "token_sources", "derivation"] {
            let mut invalid = document.clone();
            match field {
                "source" => invalid.source = serde_json::Value::Null,
                "token_sources" => invalid.token_sources = serde_json::Value::Null,
                "derivation" => invalid.derivation = serde_json::Value::Null,
                _ => unreachable!(),
            }
            assert_eq!(
                invalid.validate().unwrap_err(),
                format!("theme `{field}` must be an object")
            );
        }
    }

    #[test]
    fn theme_schema_rejects_unknown_root_fields_like_runtime() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../../themes/theme.schema.json")).unwrap();
        assert_eq!(schema["$defs"]["theme"]["additionalProperties"], false);

        let source = include_str!("../../../../themes/macos27.json").replace(
            "\"format\": \"rustui-appearance\",",
            "\"format\": \"rustui-appearance\",\n  \"unexpected\": true,",
        );
        assert!(crate::theme::parse_theme(&source).is_err());
    }

    #[test]
    fn theme_schema_rejects_whitespace_only_profile_ids_like_runtime() {
        let schema: serde_json::Value =
            serde_json::from_str(include_str!("../../../../themes/theme.schema.json")).unwrap();
        assert_eq!(
            schema["$defs"]["profile"]["properties"]["id"]["pattern"],
            "\\S"
        );

        let source = include_str!("../../../../themes/macos27.json")
            .replace("\"profile\": \"macos27\"", "\"profile\": \"   \"")
            .replace("\"id\": \"macos27\"", "\"id\": \"   \"");
        assert!(
            crate::theme::parse_theme(&source)
                .unwrap_err()
                .to_string()
                .contains("profile id must not be empty")
        );
    }

    #[test]
    fn framework_version_requires_three_semver_numeric_identifiers() {
        assert!(is_semver("0.1.0"));
        assert!(is_semver("12.34.56"));
        assert!(!is_semver("01.2.3"));
        assert!(!is_semver("1.02.3"));
        assert!(!is_semver("1.2.03"));
        assert!(!is_semver("1.2"));
    }

    #[test]
    fn format_version_accepts_integral_json_numbers_as_the_schema_does() {
        let source = include_str!("../../../../themes/macos27.json")
            .replace("\"format_version\": 1,", "\"format_version\": 1.0,");
        assert!(crate::theme::parse_theme(&source).is_ok());

        let exponent = source.replace("\"format_version\": 1.0,", "\"format_version\": 1e0,");
        assert!(crate::theme::parse_theme(&exponent).is_ok());

        let fractional = source.replace("\"format_version\": 1.0,", "\"format_version\": 1.5,");
        assert!(crate::theme::parse_theme(&fractional).is_err());
    }

    #[test]
    fn integer_fields_reject_fractional_json_values_even_when_f64_rounds_them() {
        for (field, integral) in [
            ("format_version", "1"),
            ("min_version", "27"),
            ("font_weight_normal", "400"),
        ] {
            let integer = format!("\"{field}\": {integral}");
            let fractional = match field {
                "format_version" => format!("\"{field}\": 1.0000000000000000001"),
                "min_version" => format!("\"{field}\": 27.0000000000000000001"),
                _ => format!("\"{field}\": 400.0000000000000000001"),
            };
            let source =
                include_str!("../../../../themes/macos27.json").replace(&integer, &fractional);
            assert!(
                crate::theme::parse_theme(&source).is_err(),
                "fractional {field} must not round to an integer"
            );
        }
    }

    #[test]
    fn schema_integer_fields_accept_integral_json_numbers() {
        let source = include_str!("../../../../themes/macos27.json")
            .replace("\"min_version\": 27,", "\"min_version\": 27.0,")
            .replace("\"max_version\": null,", "\"max_version\": 27.0,")
            .replace(
                "\"font_weight_normal\": 400,",
                "\"font_weight_normal\": 400.0,",
            )
            .replacen("\"weight\": 400,", "\"weight\": 400.0,", 1);
        assert!(crate::theme::parse_theme(&source).is_ok());

        let fractional = source.replace("\"min_version\": 27.0,", "\"min_version\": 27.5,");
        assert!(crate::theme::parse_theme(&fractional).is_err());
    }

    #[test]
    fn integer_fields_accept_integral_json_numbers_with_leading_zeroes() {
        let integer = format!("0.{}1e34", "0".repeat(32));
        let source = include_str!("../../../../themes/macos27.json").replace(
            "\"min_version\": 27,",
            &format!("\"min_version\": {integer},"),
        );
        assert!(crate::theme::parse_theme(&source).is_ok());
    }

    fn borders(thin: f64, medium: f64) -> Borders {
        Borders { thin, medium }
    }

    fn accessibility(focus_width: f64) -> Accessibility {
        Accessibility {
            focus_offset: 0.0,
            focus_width,
            min_body_contrast: 4.5,
            min_focus_contrast: 3.0,
            min_large_contrast: 3.0,
            reduced_motion: ReducedMotion::System,
        }
    }

    fn materials(opacity: f64) -> Materials {
        Materials {
            material_opacity: opacity,
            sidebar_blur_radius: 0.0,
            sidebar_opacity: opacity,
            toolbar_blur_radius: 0.0,
            toolbar_opacity: opacity,
        }
    }

    fn typography(font_size: f64, style_size: f64) -> Typography {
        Typography {
            font_family: "System".into(),
            font_family_display: "System".into(),
            font_weight_normal: 400,
            font_weight_medium: 500,
            font_weight_semibold: 600,
            font_weight_bold: 700,
            font_weight_heavy: 800,
            styles: BTreeMap::from([(
                "body".into(),
                FontStyle {
                    size: style_size,
                    weight: 400,
                },
            )]),
            font_size_page: font_size,
            font_size_title: font_size,
            font_size_body: font_size,
            font_size_small: font_size,
            font_size_caption: font_size,
        }
    }

    fn dimensions(value: f64) -> Dimensions {
        Dimensions {
            sidebar_width: value,
            toolbar_height: value,
            navigation_row_height: value,
            control_height: value,
            window_control_inset: value,
            status_dot: value,
            badge_size: value,
            avatar_size: value,
            sidebar_toggle_width: value,
            sidebar_toggle_inset: value,
            window_traffic_light_inset: value,
            sidebar_toggle_height: value,
            window_traffic_light_width: value,
            window_traffic_light_gap: value,
        }
    }

    fn radii(value: f64) -> Radii {
        Radii {
            sm: value,
            md: value,
            lg: value,
            segment: value,
            window: value,
        }
    }

    fn spacing_with(index: usize, value: f64) -> Spacing {
        let mut spacing = Spacing {
            space_1: 0.0,
            space_2: 0.0,
            space_3: 0.0,
            space_4: 0.0,
            space_5: 0.0,
            space_6: 0.0,
        };
        match index {
            0 => spacing.space_1 = value,
            1 => spacing.space_2 = value,
            2 => spacing.space_3 = value,
            3 => spacing.space_4 = value,
            4 => spacing.space_5 = value,
            5 => spacing.space_6 = value,
            _ => unreachable!("spacing index must be in 0..6"),
        }
        spacing
    }

    #[test]
    fn rejects_spacing_overflow_and_positive_underflow_for_every_token() {
        for index in 0..6 {
            assert!(spacing_with(index, f64::MAX).validate("test").is_err());
            assert!(
                spacing_with(index, f64::from_bits(1))
                    .validate("test")
                    .is_err()
            );
        }
    }

    #[test]
    fn accepts_zero_and_representable_f32_spacing_boundaries_for_every_token() {
        for index in 0..6 {
            for value in [0.0, f32::from_bits(1) as f64, f32::MAX as f64] {
                assert!(spacing_with(index, value).validate("test").is_ok());
            }
        }
    }

    #[test]
    fn rejects_radius_overflow_to_f32_infinity() {
        assert!(radii(f64::MAX).validate("test").is_err());
    }

    #[test]
    fn rejects_zero_radius() {
        assert!(radii(0.0).validate("test").is_err());
    }

    #[test]
    fn rejects_positive_radius_underflow_to_f32_zero() {
        assert!(radii(f64::from_bits(1)).validate("test").is_err());
    }

    #[test]
    fn accepts_positive_representable_f32_radius_boundaries() {
        for value in [f32::from_bits(1) as f64, f32::MAX as f64] {
            assert!(radii(value).validate("test").is_ok());
        }
    }

    #[test]
    fn rejects_dimension_overflow_and_positive_underflow_to_f32() {
        for value in [f64::MAX, f64::from_bits(1)] {
            assert!(dimensions(value).validate("test").is_err());
        }
    }

    #[test]
    fn accepts_positive_f32_representable_dimension_boundaries() {
        for value in [f32::from_bits(1) as f64, f32::MAX as f64] {
            assert!(dimensions(value).validate("test").is_ok());
        }
    }

    #[test]
    fn rejects_typography_font_and_style_sizes_not_representable_as_f32() {
        for value in [f64::MAX, f64::from_bits(1)] {
            assert!(typography(value, 12.0).validate("test").is_err());
            assert!(typography(12.0, value).validate("test").is_err());
        }
    }

    #[test]
    fn accepts_typography_font_and_style_f32_boundaries() {
        for value in [f32::from_bits(1) as f64, f32::MAX as f64] {
            assert!(typography(value, value).validate("test").is_ok());
        }
    }

    #[test]
    fn rejects_border_width_overflow_and_positive_underflow_to_f32() {
        for value in [f64::MAX, f64::from_bits(1)] {
            assert!(borders(value, 0.0).validate("test").is_err());
            assert!(borders(0.0, value).validate("test").is_err());
            assert!(accessibility(value).validate("test").is_err());
        }
    }

    #[test]
    fn accepts_zero_and_representable_f32_border_and_focus_widths() {
        for value in [0.0, f32::from_bits(1) as f64, f32::MAX as f64] {
            assert!(borders(value, value).validate("test").is_ok());
            assert!(accessibility(value).validate("test").is_ok());
        }
    }

    #[test]
    fn material_opacity_must_remain_representable_at_the_f32_boundary() {
        assert!(materials(f64::from_bits(1)).validate("test").is_err());
        for value in [0.0, f32::from_bits(1) as f64, 1.0] {
            assert!(materials(value).validate("test").is_ok());
        }
    }
}
