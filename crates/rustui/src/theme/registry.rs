//! Active resolved RustUI theme stored in GPUI's application globals.

use std::{error::Error, fmt};

use gpui::{App, Global, Hsla, Rgba};
use gpui_component::Theme;

use crate::{
    adapters::themes::{ThemeApplyError, apply_profile},
    theme::{
        AppearanceMode, ButtonStyleSelector, ComponentStyleCapability, ResolvedButtonTheme,
        ResolvedComponentStyle, ThemeCoverageReport, ThemeDocument, ThemeResolveError,
        resolve_button, resolve_component_style, resolve_component_style_with,
    },
};

use super::resolver::assess_theme_coverage_with_capabilities;

/// Assess how much of a theme document the current RustUI renderers apply.
pub fn assess_theme_coverage(document: &ThemeDocument) -> ThemeCoverageReport {
    assess_theme_coverage_with_capabilities(
        document,
        MAPPED_COMPONENTS,
        REFERENCE_ONLY_COMPONENTS,
        RENDERER_CAPABILITIES,
    )
}

// Records with a corresponding RustUI renderer. A record with no styles remains reference-only.
const MAPPED_COMPONENTS: &[&str] = &[
    "button",
    "link",
    "text",
    "label",
    "checkbox",
    "divider",
    "icon",
    "image",
    "slider",
    "toggle",
    "picker",
    "stepper",
    "text_field",
    "search_field",
    "radio_button",
    "combo_box",
    "color_picker",
    "disclosure_control",
    "menu",
    "notification",
    "toolbar",
    "sidebar",
    "tab_bar",
    "table",
    "outline_view",
    "scroll_view",
    "menu_item",
];

const REFERENCE_ONLY_COMPONENTS: &[(&str, &str)] = &[
    (
        "list",
        "theme record defines a table column header; RustUI ListView has no header part",
    ),
    (
        "alert_dialog",
        "theme records describe source dialog compositions; RustUI AlertView is an inline alert",
    ),
    (
        "menu_separator",
        "GPUI owns popup menu separators; the theme records list and menu-bar separators instead",
    ),
    (
        "search_field_examples",
        "example composition, not a RustUI component",
    ),
    (
        "form_accessories",
        "form artwork composition, not a RustUI component",
    ),
    (
        "save_dialog",
        "source dialog composition, not a RustUI component",
    ),
    (
        "window_chrome",
        "window artwork composition, not a RustUI component",
    ),
    (
        "text_style_guide",
        "source typography catalogue, not a runtime component",
    ),
    (
        "utility_window",
        "source window composition, not a RustUI component",
    ),
    (
        "button_group",
        "source control composition, not a RustUI component",
    ),
    ("form", "source form composition, not a RustUI component"),
    ("menu_bar", "no runtime styles or RustUI menu bar renderer"),
    ("segmented_control", "no RustUI segmented control renderer"),
    (
        "window_controls",
        "source titlebar composition, not a RustUI component",
    ),
];

macro_rules! capability {
    ($component:literal, $part:literal, when: [$(($field:literal, $value:literal)),+ $(,)?], [$($property:literal),* $(,)?]) => {
        ComponentStyleCapability {
            component: $component,
            part: $part,
            when: &[$(($field, $value)),+],
            properties: &[$($property),*],
        }
    };
    ($component:literal, $part:literal, [$($property:literal),* $(,)?]) => {
        ComponentStyleCapability {
            component: $component,
            part: $part,
            when: &[],
            properties: &[$($property),*],
        }
    };
}

// Add a capability only when the renderer or a GPUI Theme role applies every named property.
const RENDERER_CAPABILITIES: &[ComponentStyleCapability] = &[
    capability!(
        "button",
        "body",
        ["width", "height", "background", "foreground", "typography"]
    ),
    capability!("link", "body", ["foreground", "typography"]),
    capability!(
        "checkbox",
        "body",
        ["background", "foreground", "typography"]
    ),
    capability!(
        "radio_button",
        "body",
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "typography"
        ]
    ),
    capability!("slider", "sliders", ["width", "height"]),
    capability!(
        "text_field",
        "body",
        when: [("size", "small")],
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "border",
            "typography"
        ]
    ),
    capability!(
        "text_field",
        "body",
        when: [("size", "large")],
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "border",
            "typography"
        ]
    ),
    capability!(
        "search_field",
        "list_search",
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "typography"
        ]
    ),
    capability!(
        "color_picker",
        "color_wheel",
        ["width", "height", "corner_radius", "background"]
    ),
    capability!("color_picker", "swatches", ["width", "height"]),
    capability!("color_picker", "crayons", ["width", "height"]),
    capability!(
        "scroll_view",
        "scrollbar_vertical",
        ["background", "border", "corner_radius"]
    ),
    capability!(
        "scroll_view",
        "scrollbar_horizontal",
        ["background", "border", "corner_radius"]
    ),
    capability!(
        "text",
        "text_styles_left_aligned_light_opaque_body",
        ["foreground", "typography"]
    ),
    capability!(
        "text",
        "text_styles_left_aligned_dark_opaque_body",
        ["foreground", "typography"]
    ),
    capability!(
        "label",
        "text_styles_left_aligned_light_opaque_label_color",
        ["foreground", "typography"]
    ),
    capability!(
        "label",
        "text_styles_left_aligned_dark_opaque_label_color",
        ["foreground", "typography"]
    ),
    capability!("icon", "icon", ["width", "height"]),
    capability!("image", "image", ["width", "height", "corner_radius"]),
    capability!("divider", "list_separator", ["background"]),
    capability!(
        "divider",
        "menubar_separator",
        ["width", "height", "background"]
    ),
    capability!(
        "toggle",
        "body",
        ["width", "height", "corner_radius", "background"]
    ),
    capability!(
        "picker",
        "body",
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "typography"
        ]
    ),
    capability!(
        "combo_box",
        "body",
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "typography"
        ]
    ),
    capability!(
        "stepper",
        "increment_button",
        ["width", "height", "background"]
    ),
    capability!(
        "stepper",
        "decrement_button",
        ["width", "height", "background"]
    ),
    capability!(
        "stepper",
        "separator",
        ["width", "height", "corner_radius", "background"]
    ),
    capability!("stepper", "increment_glyph", ["foreground", "typography"]),
    capability!("stepper", "decrement_glyph", ["foreground", "typography"]),
    capability!(
        "table",
        "table_rows",
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "typography"
        ]
    ),
    capability!(
        "table",
        "table_column_headers",
        ["background", "foreground", "typography"]
    ),
    capability!(
        "outline_view",
        "outline_view",
        ["width", "height", "background"]
    ),
    capability!("menu", "body", ["background"]),
    capability!("menu_item", "body", ["foreground", "typography"]),
    capability!(
        "disclosure_control",
        "disclosure",
        ["foreground", "typography"]
    ),
    capability!(
        "notification",
        "body",
        [
            "width",
            "height",
            "corner_radius",
            "background",
            "foreground",
            "typography"
        ]
    ),
];

#[derive(Clone)]
pub struct ActiveTheme {
    revision: u64,
    /// Effective appearance used by the most recent successful activation.
    pub mode: AppearanceMode,
    /// Active platform profile supplying shared component metrics and tokens.
    pub profile: crate::theme::ThemeProfile,
    pub button: ResolvedButtonTheme,
    pub coverage: ThemeCoverageReport,
    document: ThemeDocument,
}

impl ActiveTheme {
    /// Identifies the successful theme activation currently owned by this app.
    /// A new successful activation advances this value, even for the same mode.
    pub fn revision(&self) -> u64 {
        self.revision
    }

    pub fn component_style(
        &self,
        component: &str,
        part: &str,
        selector: &ButtonStyleSelector,
    ) -> Result<ResolvedComponentStyle, ThemeResolveError> {
        resolve_component_style(&self.document, component, part, selector)
    }

    pub fn component_style_with(
        &self,
        component: &str,
        part: &str,
        selector: &ButtonStyleSelector,
        handled_extras: &[&str],
    ) -> Result<ResolvedComponentStyle, ThemeResolveError> {
        resolve_component_style_with(&self.document, component, part, selector, handled_extras)
    }
}

impl Global for ActiveTheme {}

pub fn active_theme(cx: &App) -> Option<&ActiveTheme> {
    cx.try_global::<ActiveTheme>()
}

pub fn theme_coverage(cx: &App) -> Option<&ThemeCoverageReport> {
    active_theme(cx).map(|theme| &theme.coverage)
}

pub fn activate_theme(
    document: &ThemeDocument,
    mode: Option<AppearanceMode>,
    cx: &mut App,
) -> Result<(), ThemeActivationError> {
    let revision = active_theme(cx)
        .map_or(0, |theme| theme.revision)
        .checked_add(1)
        .expect("theme activation revision exhausted");
    document.validate().map_err(ThemeResolveError::Invalid)?;
    let button = resolve_button(document, mode)?;
    let mode = mode.unwrap_or(document.platform_snapshot.appearance.default);
    let (table_head, table_head_foreground) = table_header_colors(document, mode)?;
    let menu_popover = menu_background(document, mode)?;
    let coverage = assess_theme_coverage(document);
    apply_profile(&document.platform_snapshot, Some(mode), cx)?;

    let background = hsla(button.background);
    let (role_hover, role_pressed) = {
        let colors = &Theme::global(cx).colors;
        (colors.primary_hover, colors.primary_active)
    };
    let hover = if button.hover_has_style {
        hsla(button.hover_background)
    } else {
        role_hover
    };
    let pressed = if button.pressed_has_style {
        hsla(button.pressed_background)
    } else {
        role_pressed
    };
    let foreground = hsla(button.foreground);
    Theme::update(cx, |theme| {
        let colors = &mut theme.colors;
        colors.button = background;
        colors.button_hover = hover;
        colors.button_active = pressed;
        colors.button_foreground = foreground;
        colors.button_secondary = background;
        colors.button_secondary_hover = hover;
        colors.button_secondary_active = pressed;
        colors.button_secondary_foreground = foreground;
        colors.table_head = table_head;
        colors.table_head_foreground = table_head_foreground;
        colors.popover = opaque_popover_fallback(colors.popover, menu_popover);
    });
    cx.set_global(ActiveTheme {
        revision,
        mode,
        profile: document.platform_snapshot.clone(),
        button,
        coverage,
        document: document.clone(),
    });
    Ok(())
}

fn table_header_colors(
    document: &ThemeDocument,
    mode: AppearanceMode,
) -> Result<(Hsla, Hsla), ThemeResolveError> {
    // The vector path is source artwork. Coverage still reports it as unresolved.
    let style = resolve_component_style_with(
        document,
        "table",
        "table_column_headers",
        &crate::components::theme::selector_for_mode(mode),
        &["vector_path"],
    )?;
    let background =
        style.colors.get("background").copied().ok_or_else(|| {
            ThemeResolveError::Invalid("table header style has no background".into())
        })?;
    let foreground =
        style.colors.get("foreground").copied().ok_or_else(|| {
            ThemeResolveError::Invalid("table header style has no foreground".into())
        })?;
    Ok((hsla(background), hsla(foreground)))
}

fn opaque_popover_fallback(surface: Hsla, tint: Hsla) -> Hsla {
    // Native backdrop blur is not available here; composite the material tint
    // over the elevated surface so content behind a popup stays legible.
    let mut base = Rgba::from(surface);
    base.a = 1.0;
    Hsla::from(base.blend(Rgba::from(tint)))
}

fn menu_background(
    document: &ThemeDocument,
    mode: AppearanceMode,
) -> Result<Hsla, ThemeResolveError> {
    // These three source SVG properties are reported as unresolved coverage.
    let style = resolve_component_style_with(
        document,
        "menu",
        "body",
        &crate::components::theme::selector_for_mode(mode),
        &["filter", "blend_mode", "vector_path"],
    )?;
    let background =
        style.colors.get("background").copied().ok_or_else(|| {
            ThemeResolveError::Invalid("menu body style has no background".into())
        })?;
    Ok(hsla(background))
}

pub(crate) fn hsla(value: crate::theme::RgbaValue) -> Hsla {
    Rgba {
        r: value.r,
        g: value.g,
        b: value.b,
        a: value.a,
    }
    .into()
}

#[derive(Debug)]
pub enum ThemeActivationError {
    Resolve(ThemeResolveError),
    Apply(ThemeApplyError),
}

impl fmt::Display for ThemeActivationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Resolve(error) => write!(formatter, "could not resolve theme: {error}"),
            Self::Apply(error) => write!(formatter, "could not apply theme: {error}"),
        }
    }
}

impl Error for ThemeActivationError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Resolve(error) => Some(error),
            Self::Apply(error) => Some(error),
        }
    }
}

impl From<ThemeResolveError> for ThemeActivationError {
    fn from(error: ThemeResolveError) -> Self {
        Self::Resolve(error)
    }
}

impl From<ThemeApplyError> for ThemeActivationError {
    fn from(error: ThemeApplyError) -> Self {
        Self::Apply(error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[gpui::test]
    fn successful_activations_advance_revision_and_failures_preserve_it(
        cx: &mut gpui::TestAppContext,
    ) {
        let mut document =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            activate_theme(&document, Some(AppearanceMode::Dark), app).unwrap();
            let initial = active_theme(app).unwrap().revision();
            activate_theme(&document, Some(AppearanceMode::Dark), app).unwrap();
            let next = active_theme(app).unwrap().revision();
            assert_eq!(next, initial + 1);
            document.format = "invalid".into();
            assert!(activate_theme(&document, None, app).is_err());
            assert_eq!(active_theme(app).unwrap().revision(), next);
        });
    }

    #[test]
    fn popover_fallback_composites_tint_on_an_opaque_surface() {
        let surface = Hsla::from(Rgba {
            r: 0.1,
            g: 0.2,
            b: 0.3,
            a: 0.5,
        });
        let tint = Hsla::from(Rgba {
            r: 1.0,
            g: 1.0,
            b: 1.0,
            a: 0.2,
        });
        let result = Rgba::from(opaque_popover_fallback(surface, tint));

        assert!((result.r - 0.28).abs() < 0.0001);
        assert!((result.g - 0.36).abs() < 0.0001);
        assert!((result.b - 0.44).abs() < 0.0001);
        assert_eq!(result.a, 1.0);
    }

    #[test]
    fn table_header_colors_resolve_any_appearance_rules() {
        let mut document =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json"))
                .expect("macOS 27 theme parses");
        let rules = document.components.get_mut("table").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        rules.retain(|rule| {
            rule["part"] != "table_column_headers" || rule["when"]["appearance"] == "light"
        });
        let light = rules
            .iter_mut()
            .find(|rule| rule["part"] == "table_column_headers")
            .expect("light table header rule");
        light["when"]["appearance"] = serde_json::json!("any");

        document.validate().expect("wildcard header theme is valid");
        let expected = resolve_component_style_with(
            &document,
            "table",
            "table_column_headers",
            &crate::components::theme::selector_for_mode(AppearanceMode::Light),
            &["vector_path"],
        )
        .expect("wildcard header style resolves");
        let expected = (
            hsla(expected.colors["background"]),
            hsla(expected.colors["foreground"]),
        );

        assert_eq!(
            table_header_colors(&document, AppearanceMode::Light).unwrap(),
            expected
        );
        assert_eq!(
            table_header_colors(&document, AppearanceMode::Dark).unwrap(),
            expected
        );
    }

    #[test]
    fn menu_background_resolves_any_appearance_rules() {
        let mut document =
            crate::theme::parse_theme(include_str!("../../../../themes/macos27.json"))
                .expect("macOS 27 theme parses");
        let rules = document.components.get_mut("menu").unwrap()["styles"]
            .as_array_mut()
            .unwrap();
        rules.retain(|rule| rule["part"] != "body" || rule["when"]["appearance"] == "light");
        let light = rules
            .iter_mut()
            .find(|rule| rule["part"] == "body")
            .expect("light menu body rule");
        light["when"]["appearance"] = serde_json::json!("any");

        document.validate().expect("wildcard menu theme is valid");
        let expected = resolve_component_style_with(
            &document,
            "menu",
            "body",
            &crate::components::theme::selector_for_mode(AppearanceMode::Light),
            &["filter", "blend_mode", "vector_path"],
        )
        .expect("wildcard menu style resolves")
        .colors["background"];

        assert_eq!(
            menu_background(&document, AppearanceMode::Light).unwrap(),
            hsla(expected)
        );
        assert_eq!(
            menu_background(&document, AppearanceMode::Dark).unwrap(),
            hsla(expected)
        );
    }

    #[test]
    fn scroll_view_track_border_and_radius_are_counted_as_applied() {
        let document = crate::theme::parse_theme(include_str!("../../../../themes/macos27.json"))
            .expect("macOS 27 theme parses");
        let coverage = assess_theme_coverage(&document);

        for part in ["scrollbar_vertical", "scrollbar_horizontal"] {
            let uncovered = coverage
                .uncovered_rules
                .iter()
                .find(|rule| rule.component == "scroll_view" && rule.part == part);
            assert!(
                uncovered.is_none_or(|rule| {
                    !rule
                        .reason
                        .split(": ")
                        .last()
                        .unwrap_or_default()
                        .split(", ")
                        .any(|name| matches!(name, "border" | "corner_radius"))
                }),
                "{part} border and corner radius should be covered by the renderer capability: {uncovered:?}"
            );
        }
    }
}
