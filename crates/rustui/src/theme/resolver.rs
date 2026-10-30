//! Typed resolution of RustUI component theme records.

use std::{collections::BTreeMap, error::Error, fmt};

use crate::theme::{
    AppearanceMode, ButtonStyleRule, ButtonStyleSelector, ButtonTypographyValue, ColorProperty,
    ComponentThemeRecord, LengthProperty, RgbaValue, ThemeDocument, ThemeProfile,
    TypographyProperty,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedButtonTheme {
    pub width: f32,
    pub height: f32,
    pub horizontal_padding: f32,
    pub corner_radius: f32,
    pub background: RgbaValue,
    pub hover_background: RgbaValue,
    pub pressed_background: RgbaValue,
    pub disabled_background: RgbaValue,
    /// Platform disabled text color, kept separate from the shared muted role.
    pub disabled_foreground: RgbaValue,
    pub focus_background: RgbaValue,
    pub hover_has_style: bool,
    pub pressed_has_style: bool,
    pub foreground: RgbaValue,
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: f32,
}

pub fn resolve_button(
    document: &ThemeDocument,
    mode: Option<AppearanceMode>,
) -> Result<ResolvedButtonTheme, ThemeResolveError> {
    let profile = &document.platform_snapshot;
    let mode = mode.unwrap_or(profile.appearance.default);
    let appearance = match mode {
        AppearanceMode::Light => "light",
        AppearanceMode::Dark => "dark",
    };
    resolve_button_with_selector(
        document,
        &ButtonStyleSelector {
            appearance: appearance.into(),
            material: "any".into(),
            interaction: "any".into(),
            text_presence: "any".into(),
            size: "any".into(),
            orientation: "any".into(),
            chrome: "any".into(),
            modality: "any".into(),
        },
    )
}

/// Resolve a button using the requested appearance and component variants.
/// Interaction styles are resolved for the requested selector's other fields.
pub fn resolve_button_with_selector(
    document: &ThemeDocument,
    selector: &ButtonStyleSelector,
) -> Result<ResolvedButtonTheme, ThemeResolveError> {
    let profile = &document.platform_snapshot;
    let mode = match selector.appearance.as_str() {
        "light" => AppearanceMode::Light,
        "dark" => AppearanceMode::Dark,
        "any" => profile.appearance.default,
        other => {
            return Err(ThemeResolveError::Invalid(format!(
                "unsupported button appearance selector `{other}`"
            )));
        }
    };
    let mut query = selector.clone();
    if query.appearance == "any" {
        query.appearance = match mode {
            AppearanceMode::Light => "light",
            AppearanceMode::Dark => "dark",
        }
        .into();
    }
    let button = document
        .button_theme()
        .map_err(ThemeResolveError::Invalid)?;
    let rule = button_rule(&button.styles, &query, "rest")?;
    let hover_rule = button_rule(&button.styles, &query, "hover")?;
    let pressed_rule = button_rule(&button.styles, &query, "pressed")?;
    let disabled_rule = button_rule(&button.styles, &query, "disabled")?;
    let focus_rule = button_rule(&button.styles, &query, "focused")?;
    if (hover_rule.when.interaction != "hover" && button.theme_roles.hover != "primary")
        || (pressed_rule.when.interaction != "pressed" && button.theme_roles.pressed != "primary")
        || document.theme_roles.get("primary").map(String::as_str) != Some("accent")
    {
        return Err(ThemeResolveError::Invalid(
            "button interaction roles cannot be mapped to platform accent colors".into(),
        ));
    }

    let padding_alias = resolve_alias(
        &document.token_sources,
        "spacing",
        &button.metrics.horizontal_padding,
    )?;
    let radius_alias = resolve_alias(
        &document.token_sources,
        "radii",
        &button.metrics.corner_radius,
    )?;
    let expected_height = resolve_dimension(profile, &button.metrics.height)?;
    let horizontal_padding = resolve_spacing(profile, padding_alias)?;
    let corner_radius = resolve_radius(profile, radius_alias)?;
    let properties = &rule.style.properties;

    if (properties.height.value.px - expected_height).abs() > f32::EPSILON {
        return Err(ThemeResolveError::Invalid(format!(
            "button height {} does not match metric reference {}",
            properties.height.value.px, expected_height
        )));
    }

    Ok(ResolvedButtonTheme {
        width: properties.width.value.px,
        height: properties.height.value.px,
        horizontal_padding,
        corner_radius,
        background: properties.background.value.rgba,
        hover_background: hover_rule.style.properties.background.value.rgba,
        pressed_background: pressed_rule.style.properties.background.value.rgba,
        disabled_background: disabled_rule.style.properties.background.value.rgba,
        disabled_foreground: resolve_platform_color(
            &profile.colors(Some(mode)).disabled_text,
            "disabled_text",
        )?,
        focus_background: focus_rule.style.properties.background.value.rgba,
        hover_has_style: hover_rule.when.interaction == "hover",
        pressed_has_style: pressed_rule.when.interaction == "pressed",
        foreground: properties.foreground.value.rgba,
        font_family: properties.typography.value.family.clone(),
        font_size: properties.typography.value.size,
        font_weight: properties.typography.value.weight,
    })
}

fn resolve_platform_color(value: &str, name: &str) -> Result<RgbaValue, ThemeResolveError> {
    let digits = value.strip_prefix('#').ok_or_else(|| {
        ThemeResolveError::Invalid(format!(
            "platform color `{name}` must be a six-digit hex color"
        ))
    })?;
    if digits.len() != 6 || !digits.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ThemeResolveError::Invalid(format!(
            "platform color `{name}` must be a six-digit hex color"
        )));
    }
    let channel = |offset| {
        u8::from_str_radix(&digits[offset..offset + 2], 16)
            .map(|value| f32::from(value) / 255.0)
            .map_err(|error| ThemeResolveError::Invalid(error.to_string()))
    };
    Ok(RgbaValue {
        r: channel(0)?,
        g: channel(2)?,
        b: channel(4)?,
        a: 1.0,
    })
}

fn button_rule<'a>(
    rules: &'a [ButtonStyleRule],
    selector: &ButtonStyleSelector,
    interaction: &str,
) -> Result<&'a ButtonStyleRule, ThemeResolveError> {
    let mut query = selector.clone();
    query.interaction = interaction.into();
    let mut best = None;
    let mut best_score = 0;
    let mut tied = false;
    for rule in rules.iter().filter(|rule| selector_matches(rule, &query)) {
        let score = selector_specificity(&rule.when);
        if best.is_none() || score > best_score {
            best = Some(rule);
            best_score = score;
            tied = false;
        } else if score == best_score {
            tied = true;
        }
    }
    if tied {
        return Err(ThemeResolveError::Invalid(format!(
            "button has ambiguous body styles for {}/{}",
            selector.appearance, interaction
        )));
    }
    best.ok_or_else(|| {
        ThemeResolveError::Invalid(format!(
            "button has no body style for {}/{}",
            selector.appearance, interaction
        ))
    })
}

fn selector_matches(rule: &ButtonStyleRule, query: &ButtonStyleSelector) -> bool {
    rule.part == "body"
        && selector_fields(&rule.when)
            .into_iter()
            .zip(selector_fields(query))
            .all(|(rule, query)| rule == "any" || rule == query)
}

fn selector_fields(selector: &ButtonStyleSelector) -> [&str; 8] {
    [
        &selector.appearance,
        &selector.material,
        &selector.interaction,
        &selector.text_presence,
        &selector.size,
        &selector.orientation,
        &selector.chrome,
        &selector.modality,
    ]
}

fn resolve_alias<'a>(
    token_sources: &'a serde_json::Value,
    category: &str,
    reference: &str,
) -> Result<&'a str, ThemeResolveError> {
    let (prefix, key) = reference.split_once('.').ok_or_else(|| {
        ThemeResolveError::Invalid(format!("invalid token reference `{reference}`"))
    })?;
    if prefix != category {
        return Err(ThemeResolveError::Invalid(format!(
            "token reference `{reference}` must use `{category}`"
        )));
    }
    token_sources
        .get(category)
        .and_then(|values| values.get(key))
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| {
            ThemeResolveError::Invalid(format!("unresolved token reference `{reference}`"))
        })
}

fn resolve_dimension(profile: &ThemeProfile, reference: &str) -> Result<f32, ThemeResolveError> {
    match reference {
        "dimensions.control_height" => Ok(profile.dimensions.control_height as f32),
        _ => Err(ThemeResolveError::Invalid(format!(
            "unsupported button dimension reference `{reference}`"
        ))),
    }
}

fn resolve_spacing(profile: &ThemeProfile, alias: &str) -> Result<f32, ThemeResolveError> {
    let spacing = &profile.spacing;
    match alias {
        "space_1" => Ok(spacing.space_1 as f32),
        "space_2" => Ok(spacing.space_2 as f32),
        "space_3" => Ok(spacing.space_3 as f32),
        "space_4" => Ok(spacing.space_4 as f32),
        "space_5" => Ok(spacing.space_5 as f32),
        "space_6" => Ok(spacing.space_6 as f32),
        _ => Err(ThemeResolveError::Invalid(format!(
            "unsupported button spacing alias `{alias}`"
        ))),
    }
}

fn resolve_radius(profile: &ThemeProfile, alias: &str) -> Result<f32, ThemeResolveError> {
    let radii = &profile.radii;
    match alias {
        "sm" => Ok(radii.sm as f32),
        "md" => Ok(radii.md as f32),
        "lg" => Ok(radii.lg as f32),
        "segment" => Ok(radii.segment as f32),
        "window" => Ok(radii.window as f32),
        _ => Err(ThemeResolveError::Invalid(format!(
            "unsupported button radius alias `{alias}`"
        ))),
    }
}

/// A single resolved style rule. No component renderer consumes this automatically.
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedComponentStyle {
    pub lengths: BTreeMap<String, f32>,
    pub colors: BTreeMap<String, RgbaValue>,
    pub typography: Option<ButtonTypographyValue>,
    /// Extra properties explicitly accepted by a renderer; the resolver never interprets them.
    pub extra_properties: BTreeMap<String, serde_json::Value>,
    pub record_metadata: BTreeMap<String, serde_json::Value>,
    pub part_metadata: serde_json::Value,
    pub rule_metadata: BTreeMap<String, serde_json::Value>,
    pub style_metadata: BTreeMap<String, serde_json::Value>,
}

/// Resolve one declared part and selector without interpreting paths, filters, or artwork.
pub fn resolve_component_style(
    document: &ThemeDocument,
    component: &str,
    part: &str,
    selector: &ButtonStyleSelector,
) -> Result<ResolvedComponentStyle, ThemeResolveError> {
    resolve_component_style_with(document, component, part, selector, &[])
}

/// Resolve common fields while requiring the caller to name every extra field it handles.
pub fn resolve_component_style_with(
    document: &ThemeDocument,
    component: &str,
    part: &str,
    selector: &ButtonStyleSelector,
    handled_extras: &[&str],
) -> Result<ResolvedComponentStyle, ThemeResolveError> {
    let record = document
        .component_theme(component)
        .map_err(ThemeResolveError::Invalid)?;
    resolve_component_style_from_record(
        document,
        component,
        part,
        selector,
        handled_extras,
        &record,
    )
}

fn resolve_component_style_from_record(
    document: &ThemeDocument,
    component: &str,
    part: &str,
    selector: &ButtonStyleSelector,
    handled_extras: &[&str],
    record: &ComponentThemeRecord,
) -> Result<ResolvedComponentStyle, ThemeResolveError> {
    let part_metadata = record
        .parts
        .get(part)
        .ok_or_else(|| ThemeResolveError::Invalid(format!("`{component}` has no `{part}` part")))?;

    let mut candidates = record
        .styles
        .iter()
        .filter(|rule| rule.part == part && component_selector_matches(&rule.when, selector));
    let mut best = candidates.next().ok_or_else(|| {
        ThemeResolveError::Invalid(format!("`{component}` has no matching `{part}` style"))
    })?;
    let mut score = selector_specificity(&best.when);
    let mut tied = false;
    for candidate in candidates {
        let candidate_score = selector_specificity(&candidate.when);
        if candidate_score > score {
            best = candidate;
            score = candidate_score;
            tied = false;
        } else if candidate_score == score {
            tied = true;
        }
    }
    if tied {
        return Err(ThemeResolveError::Invalid(format!(
            "`{component}` has ambiguous `{part}` styles"
        )));
    }

    let mut lengths = BTreeMap::new();
    let mut colors = BTreeMap::new();
    let mut typography = None;
    let mut extra_properties = BTreeMap::new();
    for (name, value) in &best.style.properties {
        match name.as_str() {
            "width" | "height" | "corner_radius" | "border_width" => {
                let property: LengthProperty =
                    serde_json::from_value(value.clone()).map_err(|error| {
                        ThemeResolveError::Invalid(format!("`{component}` `{name}`: {error}"))
                    })?;
                let px = property.value.px;
                if !px.is_finite()
                    || (matches!(name.as_str(), "width" | "height") && px <= 0.0)
                    || px < 0.0
                {
                    return Err(ThemeResolveError::Invalid(format!(
                        "`{component}` `{name}` must be a finite nonnegative length"
                    )));
                }
                lengths.insert(name.clone(), px);
            }
            "background" | "foreground" | "border" => {
                let property: ColorProperty =
                    serde_json::from_value(value.clone()).map_err(|error| {
                        ThemeResolveError::Invalid(format!("`{component}` `{name}`: {error}"))
                    })?;
                let rgba = property.value.rgba;
                if ![rgba.r, rgba.g, rgba.b, rgba.a]
                    .into_iter()
                    .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel))
                {
                    return Err(ThemeResolveError::Invalid(format!(
                        "`{component}` `{name}` has invalid RGBA channels"
                    )));
                }
                colors.insert(name.clone(), rgba);
            }
            "typography" => {
                let property: TypographyProperty =
                    serde_json::from_value(value.clone()).map_err(|error| {
                        ThemeResolveError::Invalid(format!("`{component}` `{name}`: {error}"))
                    })?;
                let value = property.value;
                if value.family.trim().is_empty()
                    || !value.size.is_finite()
                    || value.size <= 0.0
                    || !value.weight.is_finite()
                    || !(1.0..=1000.0).contains(&value.weight)
                {
                    return Err(ThemeResolveError::Invalid(format!(
                        "`{component}` has invalid typography"
                    )));
                }
                typography = Some(value);
            }
            _ => {
                if !handled_extras.contains(&name.as_str()) {
                    return Err(ThemeResolveError::Invalid(format!(
                        "`{component}` `{part}` uses unsupported runtime property `{name}`"
                    )));
                }
                extra_properties.insert(name.clone(), value.clone());
            }
        }
    }
    if lengths.is_empty()
        && colors.is_empty()
        && typography.is_none()
        && extra_properties.is_empty()
    {
        return Err(ThemeResolveError::Invalid(format!(
            "`{component}` `{part}` has no resolvable properties"
        )));
    }
    Ok(ResolvedComponentStyle {
        lengths,
        colors,
        typography,
        extra_properties,
        record_metadata: record.metadata.clone(),
        part_metadata: part_metadata.clone(),
        rule_metadata: best.metadata.clone(),
        style_metadata: best.style.metadata.clone(),
    })
}

/// Validate one typed component record using the same selector and property
/// rules used by runtime resolution. Extra properties stay opaque here.
pub(super) fn validate_component_theme(
    document: &ThemeDocument,
    component: &str,
    record: &ComponentThemeRecord,
) -> Result<(), ThemeResolveError> {
    if record.rustui_type.trim().is_empty() {
        return Err(ThemeResolveError::Invalid(
            "rustui_type must not be empty".into(),
        ));
    }

    for (index, rule) in record.styles.iter().enumerate() {
        if rule.part.trim().is_empty() || !record.parts.contains_key(&rule.part) {
            return Err(ThemeResolveError::Invalid(format!(
                "style rule {index} refers to undeclared part `{}`",
                rule.part
            )));
        }
        if !matches!(rule.when.appearance.as_str(), "any" | "light" | "dark") {
            return Err(ThemeResolveError::Invalid(format!(
                "style rule {index} has unsupported appearance selector `{}`",
                rule.when.appearance
            )));
        }
        if component == "slider" && rule.part == "sliders" {
            for property in ["width", "height"] {
                if !rule.style.properties.contains_key(property) {
                    return Err(ThemeResolveError::Invalid(format!(
                        "slider style rule {index} is missing renderer-required `{property}` property"
                    )));
                }
            }
        }
        for (other_index, other) in record.styles.iter().enumerate().skip(index + 1) {
            if rule.part == other.part
                && selector_specificity(&rule.when) == selector_specificity(&other.when)
                && component_selectors_overlap(&rule.when, &other.when)
            {
                return Err(ThemeResolveError::Invalid(format!(
                    "style rules {index} and {other_index} have ambiguous `{}` selectors",
                    rule.part
                )));
            }
        }

        let handled_extras = rule
            .style
            .properties
            .keys()
            .filter(|name| {
                !matches!(
                    name.as_str(),
                    "width"
                        | "height"
                        | "corner_radius"
                        | "border_width"
                        | "background"
                        | "foreground"
                        | "border"
                        | "typography"
                )
            })
            .map(String::as_str)
            .collect::<Vec<_>>();
        resolve_component_style_from_record(
            document,
            component,
            &rule.part,
            &rule.when,
            &handled_extras,
            record,
        )
        .map_err(|error| ThemeResolveError::Invalid(format!("style rule {index}: {error}")))?;
    }

    Ok(())
}

fn component_selector_matches(rule: &ButtonStyleSelector, query: &ButtonStyleSelector) -> bool {
    component_selector_fields(rule)
        .into_iter()
        .zip(component_selector_fields(query))
        .all(|(rule, query)| rule == "any" || rule == query)
}

pub(super) fn component_selectors_overlap(
    left: &ButtonStyleSelector,
    right: &ButtonStyleSelector,
) -> bool {
    component_selector_fields(left)
        .into_iter()
        .zip(component_selector_fields(right))
        .all(|(left, right)| left == "any" || right == "any" || left == right)
}

fn component_selector_fields(selector: &ButtonStyleSelector) -> [&str; 8] {
    [
        &selector.appearance,
        &selector.material,
        &selector.interaction,
        &selector.text_presence,
        &selector.size,
        &selector.orientation,
        &selector.chrome,
        &selector.modality,
    ]
}

pub(super) fn selector_specificity(selector: &ButtonStyleSelector) -> usize {
    component_selector_fields(selector)
        .into_iter()
        .filter(|value| *value != "any")
        .count()
}

/// Renderer-declared properties it actually applies for a component part.
pub struct ComponentStyleCapability {
    pub component: &'static str,
    pub part: &'static str,
    /// Selector constraints such as `("material", "opaque")`.
    pub when: &'static [(&'static str, &'static str)],
    pub properties: &'static [&'static str],
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UncoveredComponentRule {
    pub component: String,
    pub part: String,
    pub appearance: String,
    pub reason: String,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ThemeCoverageReport {
    pub total_records: usize,
    pub total_rules: usize,
    pub covered_rules: usize,
    pub uncovered_rules: Vec<UncoveredComponentRule>,
    pub reference_only_rules: usize,
    pub not_applicable_rules: usize,
    pub unresolved_zero_style_records: Vec<String>,
    pub reference_only_records: Vec<ThemeRecordDisposition>,
    pub not_applicable_records: Vec<ThemeRecordDisposition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeRecordDisposition {
    pub record: String,
    pub reason: String,
}

impl ThemeCoverageReport {
    pub fn is_complete(&self) -> bool {
        self.uncovered_rules.is_empty() && self.unresolved_zero_style_records.is_empty()
    }
}

/// Report every mapped record that cannot be fully applied by the declared renderers.
pub fn assess_theme_coverage_with_capabilities(
    document: &ThemeDocument,
    mapped_components: &[&str],
    reference_only_components: &[(&str, &str)],
    capabilities: &[ComponentStyleCapability],
) -> ThemeCoverageReport {
    let mut report = ThemeCoverageReport {
        total_records: document.components.len(),
        total_rules: document
            .components
            .values()
            .filter_map(|value| value.get("styles").and_then(serde_json::Value::as_array))
            .map(Vec::len)
            .sum(),
        ..ThemeCoverageReport::default()
    };
    for &component in mapped_components {
        let record = match document.component_theme(component) {
            Ok(record) => record,
            Err(reason) => {
                report.uncovered_rules.push(UncoveredComponentRule {
                    component: component.into(),
                    part: "*".into(),
                    appearance: "any".into(),
                    reason,
                });
                continue;
            }
        };
        if record.styles.is_empty() {
            report.unresolved_zero_style_records.push(component.into());
        }
        for rule in &record.styles {
            let matching: Vec<_> = capabilities
                .iter()
                .filter(|capability| {
                    capability.component == component
                        && capability.part == rule.part
                        && capability.when.iter().all(|(field, expected)| {
                            selector_field(&rule.when, field) == Some(*expected)
                        })
                })
                .collect();
            let missing: Vec<_> = rule
                .style
                .properties
                .keys()
                .filter(|name| {
                    !matching
                        .iter()
                        .any(|capability| capability.properties.contains(&name.as_str()))
                })
                .cloned()
                .collect();
            let reason = if matching.is_empty() {
                Some("no renderer capability for this rule".to_owned())
            } else if !missing.is_empty() {
                Some(format!("unapplied properties: {}", missing.join(", ")))
            } else {
                let extras: Vec<_> = rule
                    .style
                    .properties
                    .keys()
                    .filter(|name| {
                        !matches!(
                            name.as_str(),
                            "width"
                                | "height"
                                | "corner_radius"
                                | "background"
                                | "foreground"
                                | "typography"
                        )
                    })
                    .map(String::as_str)
                    .collect();
                resolve_component_style_with(document, component, &rule.part, &rule.when, &extras)
                    .err()
                    .map(|error| error.to_string())
            };
            if let Some(reason) = reason {
                report.uncovered_rules.push(UncoveredComponentRule {
                    component: component.into(),
                    part: rule.part.clone(),
                    appearance: rule.when.appearance.clone(),
                    reason,
                });
            } else {
                report.covered_rules += 1;
            }
        }
    }
    for name in document
        .components
        .keys()
        .filter(|name| !mapped_components.contains(&name.as_str()))
    {
        if let Some((_, reason)) = reference_only_components
            .iter()
            .find(|(record, _)| *record == name)
        {
            report.reference_only_rules += document.components[name]["styles"]
                .as_array()
                .map_or(0, Vec::len);
            report.reference_only_records.push(ThemeRecordDisposition {
                record: name.clone(),
                reason: (*reason).into(),
            });
        } else {
            report.not_applicable_rules += document.components[name]["styles"]
                .as_array()
                .map_or(0, Vec::len);
            report.not_applicable_records.push(ThemeRecordDisposition {
                record: name.clone(),
                reason: "no mapped RustUI component renderer".into(),
            });
        }
    }
    report
}

fn selector_field<'a>(selector: &'a ButtonStyleSelector, field: &str) -> Option<&'a str> {
    match field {
        "appearance" => Some(&selector.appearance),
        "material" => Some(&selector.material),
        "interaction" => Some(&selector.interaction),
        "text_presence" => Some(&selector.text_presence),
        "size" => Some(&selector.size),
        "orientation" => Some(&selector.orientation),
        "chrome" => Some(&selector.chrome),
        "modality" => Some(&selector.modality),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ThemeResolveError {
    Invalid(String),
}

impl fmt::Display for ThemeResolveError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Invalid(message) => formatter.write_str(message),
        }
    }
}

impl Error for ThemeResolveError {}

#[cfg(test)]
mod component_tests {
    use super::*;

    fn document() -> ThemeDocument {
        crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap()
    }

    #[test]
    fn any_appearance_resolves_using_the_profile_default() {
        let document = document();
        let selector = ButtonStyleSelector {
            appearance: "any".into(),
            material: "any".into(),
            interaction: "any".into(),
            text_presence: "any".into(),
            size: "any".into(),
            orientation: "any".into(),
            chrome: "any".into(),
            modality: "any".into(),
        };

        assert_eq!(
            resolve_button_with_selector(&document, &selector).unwrap(),
            resolve_button(&document, None).unwrap()
        );
    }

    #[test]
    fn button_uses_declared_interaction_rule_without_alpha_overlay() {
        let mut document = document();
        let base = document.components["button"]["styles"][0].clone();
        let button = document.components.get_mut("button").unwrap();
        let mut pressed = base;
        pressed["when"]["interaction"] = serde_json::json!("pressed");
        pressed["style"]["properties"]["background"]["value"]["rgba"]["a"] = serde_json::json!(0.5);
        button["styles"].as_array_mut().unwrap().push(pressed);
        let resolved = resolve_button(&document, Some(AppearanceMode::Dark)).unwrap();
        assert_eq!(resolved.background.a, 0.08);
        assert_eq!(resolved.hover_background.a, 0.08);
        assert!(!resolved.hover_has_style);
        assert_eq!(resolved.pressed_background.a, 0.5);
        assert!(resolved.pressed_has_style);
    }

    #[test]
    fn button_resolves_requested_size_variant_metrics() {
        let mut document = document();
        let styles = document.components.get_mut("button").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let mut small_variant = styles
            .iter()
            .find(|rule| rule["when"]["appearance"] == "light")
            .unwrap()
            .clone();
        small_variant["when"]["size"] = serde_json::json!("small");
        small_variant["style"]["properties"]["width"]["value"]["px"] = serde_json::json!(48.0);
        styles.push(small_variant);

        let base_selector = document
            .component_theme("button")
            .unwrap()
            .styles
            .into_iter()
            .find(|rule| rule.when.appearance == "light")
            .unwrap()
            .when;
        let mut small_selector = base_selector.clone();
        small_selector.size = "small".into();
        let mut large_selector = base_selector;
        large_selector.size = "large".into();

        let small = resolve_button_with_selector(&document, &small_selector).unwrap();
        let large = resolve_button_with_selector(&document, &large_selector).unwrap();
        let default = resolve_button(&document, Some(AppearanceMode::Light)).unwrap();

        assert_eq!(small.width, 48.0);
        assert_eq!(large.width, 67.0);
        assert_eq!(default.width, 67.0);
    }

    #[test]
    fn button_resolves_platform_disabled_text_without_changing_muted_role() {
        let document = document();
        let resolved = resolve_button(&document, Some(AppearanceMode::Dark)).unwrap();
        let disabled_text = &document
            .platform_snapshot
            .colors(Some(AppearanceMode::Dark))
            .disabled_text;
        assert_eq!(disabled_text, "#A6A6A6");
        assert_eq!(
            resolved.disabled_foreground,
            RgbaValue {
                r: 166.0 / 255.0,
                g: 166.0 / 255.0,
                b: 166.0 / 255.0,
                a: 1.0
            }
        );
        assert_ne!(resolved.disabled_foreground, resolved.foreground);
    }

    #[test]
    fn resolves_supported_component_properties_and_keeps_metadata() {
        let mut document = document();
        let toggle = document.components.get_mut("toggle").unwrap();
        toggle["custom_metadata"] = serde_json::json!({"source": "example"});
        toggle["styles"][0]["custom_rule"] = serde_json::json!(42);
        toggle["styles"][0]["style"]["custom_style"] = serde_json::json!(true);
        let selector = document.component_theme("toggle").unwrap().styles[0]
            .when
            .clone();
        let result = resolve_component_style(&document, "toggle", "body", &selector).unwrap();
        assert_eq!(result.lengths.get("width"), Some(&36.0));
        assert!(result.colors.contains_key("background"));
        assert_eq!(
            result.record_metadata["custom_metadata"]["source"],
            "example"
        );
        assert_eq!(result.rule_metadata["custom_rule"], 42);
        assert_eq!(result.style_metadata["custom_style"], true);
    }

    #[test]
    fn accepts_zero_width_border_as_a_valid_nonnegative_length() {
        let mut document = document();
        document.components.get_mut("toggle").unwrap()["styles"][0]["style"]["properties"]["border_width"] =
            serde_json::json!({"kind": "length", "value": {"px": 0.0}});
        let selector = document.component_theme("toggle").unwrap().styles[0]
            .when
            .clone();

        let resolved = resolve_component_style(&document, "toggle", "body", &selector).unwrap();

        assert_eq!(resolved.lengths.get("border_width"), Some(&0.0));
    }

    #[test]
    fn slider_theme_requires_renderer_thumb_dimensions() {
        let mut document = document();
        let styles = document.components.get_mut("slider").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        let sliders = styles
            .iter_mut()
            .find(|rule| rule["part"] == "sliders")
            .unwrap();
        sliders["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("width");

        assert!(
            document
                .validate()
                .unwrap_err()
                .contains("slider style rule 0 is missing renderer-required `width` property")
        );
    }

    #[test]
    fn refuses_vector_artwork_as_a_runtime_style() {
        let document = document();
        let selector = document.component_theme("checkbox").unwrap().styles[0]
            .when
            .clone();
        let error = resolve_component_style(&document, "checkbox", "body", &selector).unwrap_err();
        assert!(error.to_string().contains("vector_path"));
    }

    #[test]
    fn refuses_filter_records_and_undeclared_parts() {
        let document = document();
        let selector = document.component_theme("text_field").unwrap().styles[0]
            .when
            .clone();
        let error =
            resolve_component_style(&document, "text_field", "search", &selector).unwrap_err();
        assert!(error.to_string().contains("blend_mode"), "{error}");
        let mut document_without_blend = document.clone();
        document_without_blend
            .components
            .get_mut("text_field")
            .unwrap()["styles"][0]["style"]["properties"]
            .as_object_mut()
            .unwrap()
            .remove("blend_mode");
        let error =
            resolve_component_style(&document_without_blend, "text_field", "search", &selector)
                .unwrap_err();
        assert!(error.to_string().contains("filter"), "{error}");
        let error = resolve_component_style(&document, "toggle", "missing", &selector).unwrap_err();
        assert!(error.to_string().contains("no `missing` part"));
    }

    #[test]
    fn coverage_reports_unapplied_artwork_and_reference_only_records() {
        let document = document();
        let capabilities = [ComponentStyleCapability {
            component: "checkbox",
            part: "body",
            when: &[],
            properties: &["background", "foreground", "typography"],
        }];
        let report = assess_theme_coverage_with_capabilities(
            &document,
            &["checkbox", "toolbar"],
            &[],
            &capabilities,
        );
        assert_eq!(report.total_records, 41);
        assert_eq!(report.total_rules, 420);
        assert_eq!(
            report.covered_rules
                + report.uncovered_rules.len()
                + report.reference_only_rules
                + report.not_applicable_rules,
            report.total_rules
        );
        assert!(!report.is_complete());
        assert_eq!(report.covered_rules, 0);
        assert_eq!(report.unresolved_zero_style_records, vec!["toolbar"]);
        assert!(
            report
                .uncovered_rules
                .iter()
                .all(|rule| rule.reason.contains("vector_path"))
        );
    }
}
