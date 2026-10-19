//! RustUI's versioned platform-appearance theme contract.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Current on-disk RustUI appearance format version.
pub const THEME_FORMAT_VERSION: u8 = 1;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ThemeDocument {
    pub format: String,
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
        if self.components.is_empty() {
            return Err("theme must contain RustUI component definitions".into());
        }
        self.button_theme()?.validate()?;
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
        for rule in &self.styles {
            rule.validate()?;
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
    pub min_version: u32,
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
        validate_positive_numbers(
            profile,
            "dimensions",
            [
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
            ],
        )
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
        validate_nonnegative_numbers(
            profile,
            "radii",
            [self.sm, self.md, self.lg, self.segment, self.window],
        )
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
        validate_nonnegative_numbers(profile, "borders", [self.thin, self.medium])
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
        validate_nonnegative_numbers(
            profile,
            "spacing",
            [
                self.space_1,
                self.space_2,
                self.space_3,
                self.space_4,
                self.space_5,
                self.space_6,
            ],
        )
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
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(format!(
                    "profile `{profile}` materials.{name} must be between 0 and 1"
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
        )
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
    pub font_weight_normal: u16,
    pub font_weight_medium: u16,
    pub font_weight_semibold: u16,
    pub font_weight_bold: u16,
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
        validate_positive_numbers(
            profile,
            "typography font sizes",
            [
                self.font_size_page,
                self.font_size_title,
                self.font_size_body,
                self.font_size_small,
                self.font_size_caption,
            ],
        )?;
        if self.styles.is_empty() {
            return Err(format!("profile `{profile}` must define text styles"));
        }
        for (name, style) in &self.styles {
            if !is_style_name(name) {
                return Err(format!(
                    "profile `{profile}` has invalid text style name `{name}`"
                ));
            }
            if !style.size.is_finite() || style.size <= 0.0 {
                return Err(format!(
                    "profile `{profile}` typography style `{name}` size must be positive"
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
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit()))
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
