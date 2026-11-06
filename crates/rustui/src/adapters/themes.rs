//! Applies a RustUI appearance profile to GPUI Components' active theme.

use gpui::{App, Hsla, px, rgb};
use gpui_component::{Colorize, Theme, ThemeMode, theme::try_parse_color};

use crate::theme::{AppearanceMode, ThemeProfile};

/// Apply a profile's selected appearance to GPUI Components.
///
/// Call [`gpui_component::init`] before this function. RustUI buttons use
/// GPUI Components' global theme directly, so this updates existing and
/// subsequently rendered buttons without a separate RustUI rendering path.
///
/// Pass `None` to use the profile's default appearance. Call this again when
/// the app changes light/dark appearance so the matching palette is applied.
pub fn apply_profile(
    profile: &ThemeProfile,
    mode: Option<AppearanceMode>,
    cx: &mut App,
) -> Result<(), ThemeApplyError> {
    let mode = mode.unwrap_or(profile.appearance.default);
    let palette = ButtonPalette::parse(profile, mode)?;
    let font_size_body = positive_metric_as_f32(
        profile.typography.font_size_body,
        "typography.font_size_body",
    )?;
    let radius_md = positive_metric_as_f32(profile.radii.md, "radii.md")?;
    let radius_lg = positive_metric_as_f32(profile.radii.lg, "radii.lg")?;
    let material_opacity = unit_opacity_as_f32(
        profile.materials.material_opacity,
        "materials.material_opacity",
    )?;
    let sidebar_background = color_with_profile_opacity(
        palette.sidebar_background,
        profile.materials.sidebar_opacity,
        "materials.sidebar_opacity",
    )?;
    let toolbar_opacity = unit_opacity_as_f32(
        profile.materials.toolbar_opacity,
        "materials.toolbar_opacity",
    )?;
    let toolbar_background = palette.toolbar_background.opacity(toolbar_opacity);
    let secondary_background = gpui_secondary_background(&palette, toolbar_opacity);
    let font_family = gpui_font_family(
        &profile.typography.font_family,
        cfg!(target_family = "wasm"),
    );
    let button_surface = palette.surface.opacity(material_opacity);
    let button_hover = palette.surface_hover.opacity(material_opacity);
    let button_active = palette.surface_active.opacity(material_opacity);
    let component_mode = match mode {
        AppearanceMode::Light => ThemeMode::Light,
        AppearanceMode::Dark => ThemeMode::Dark,
    };

    Theme::change(component_mode, None, cx);
    Theme::update(cx, |theme| {
        theme.font_family = font_family.into();
        theme.font_size = px(font_size_body);
        theme.radius = px(radius_md);
        theme.radius_lg = px(radius_lg);

        let colors = &mut theme.colors;

        colors.background = palette.window_background;
        colors.foreground = palette.primary_text;
        colors.popover = palette.surface;
        colors.popover_foreground = palette.primary_text;
        colors.border = palette.border;
        colors.sidebar = sidebar_background;
        colors.sidebar_border = palette.border;
        colors.sidebar_accent = palette.selected_row;
        colors.sidebar_accent_foreground = palette.primary_text;
        colors.list = palette.content_background;
        colors.list_active = palette.selected_row;
        colors.list_active_border = palette.border;
        colors.list_even = palette.content_background;
        colors.list_head = toolbar_background;
        colors.list_hover = palette.surface_hover;
        colors.selection = palette.selected_row;
        colors.scrollbar = palette.content_background;
        colors.scrollbar_thumb = palette.border;
        colors.scrollbar_thumb_hover = palette.secondary_text;
        colors.slider_bar = palette.border;
        colors.slider_thumb = palette.accent;
        colors.link = palette.accent;
        colors.link_hover = palette.accent_hover;
        colors.link_active = palette.accent_active;
        colors.accordion = palette.surface;
        colors.group_box = palette.surface;
        colors.group_box_foreground = palette.primary_text;
        colors.caret = palette.accent;
        colors.chart_1 = palette.chart_1;
        colors.chart_2 = palette.chart_2;
        colors.chart_3 = palette.chart_3;
        colors.chart_4 = palette.chart_4;
        colors.chart_5 = palette.chart_5;
        colors.chart_bullish = palette.success;
        colors.chart_bearish = palette.error;
        colors.chart_grid = palette.border;
        colors.description_list_label = palette.surface;
        colors.description_list_label_foreground = palette.secondary_text;
        colors.drag_border = palette.accent;
        colors.drop_target = palette.selected_row;
        colors.muted = palette.surface_hover;
        colors.progress_bar = palette.accent;
        colors.sidebar_foreground = palette.primary_text;
        colors.sidebar_primary = palette.accent;
        colors.sidebar_primary_foreground = palette.accent_foreground;
        colors.skeleton = palette.surface_hover;
        colors.switch = palette.surface_hover;
        colors.switch_thumb = palette.primary_text;
        colors.tab = palette.content_background;
        colors.tab_active = palette.selected_row;
        colors.tab_active_foreground = palette.primary_text;
        colors.tab_bar = toolbar_background;
        colors.tab_bar_segmented = palette.surface_hover;
        colors.tab_foreground = palette.secondary_text;
        colors.table = palette.content_background;
        colors.table_active = palette.selected_row;
        colors.table_active_border = palette.border;
        colors.table_even = palette.content_background;
        colors.table_head = toolbar_background;
        colors.table_head_foreground = palette.secondary_text;
        colors.table_foot = toolbar_background;
        colors.table_foot_foreground = palette.secondary_text;
        colors.table_hover = palette.surface_hover;
        colors.table_row_border = palette.border;
        colors.title_bar = toolbar_background;
        colors.title_bar_border = palette.border;
        colors.status_bar = toolbar_background;
        colors.status_bar_border = palette.border;
        colors.overlay = palette.primary_text.opacity(0.2);
        colors.window_border = palette.border;
        colors.red = palette.error;
        colors.red_light = palette.error.opacity(0.15);
        colors.green = palette.success;
        colors.green_light = palette.success.opacity(0.15);
        colors.blue = palette.accent;
        colors.blue_light = palette.accent.opacity(0.15);
        colors.yellow = palette.warning;
        colors.yellow_light = palette.warning.opacity(0.15);
        colors.magenta = palette.chart_2;
        colors.magenta_light = palette.chart_2.opacity(0.15);
        colors.cyan = palette.chart_1;
        colors.cyan_light = palette.chart_1.opacity(0.15);
        // GPUI Components' default button variant uses the input surface.
        // Apply the profile material here so RustUI buttons actually follow it.
        colors.input = button_surface;
        colors.ring = palette.focus_ring;
        // The macOS 27 role map resolves `muted_foreground` to
        // `secondary_foreground`; disabled controls have their own component
        // rules and must not change the shared muted-text role.
        colors.muted_foreground = gpui_muted_foreground(&palette);

        colors.accent = palette.accent;
        colors.accent_foreground = palette.accent_foreground;
        colors.primary = palette.accent;
        colors.primary_hover = palette.accent_hover;
        colors.primary_active = palette.accent_active;
        colors.primary_foreground = palette.accent_foreground;
        colors.secondary = secondary_background;
        colors.secondary_hover = palette.surface_hover;
        colors.secondary_active = palette.surface_active;
        colors.secondary_foreground = palette.secondary_text;
        colors.danger = palette.error;
        colors.danger_hover = palette.error_hover;
        colors.danger_active = palette.error_active;
        colors.danger_foreground = palette.error_foreground;
        colors.warning = palette.warning;
        colors.warning_hover = palette.warning_hover;
        colors.warning_active = palette.warning_active;
        colors.warning_foreground = palette.warning_foreground;
        colors.success = palette.success;
        colors.success_hover = palette.success_hover;
        colors.success_active = palette.success_active;
        colors.success_foreground = palette.success_foreground;
        colors.info = palette.accent;
        colors.info_hover = palette.accent_hover;
        colors.info_active = palette.accent_active;
        colors.info_foreground = palette.accent_foreground;
        colors.link = palette.accent;

        colors.button = button_surface;
        colors.button_hover = button_hover;
        colors.button_active = button_active;
        colors.button_foreground = palette.primary_text;
        colors.button_primary = palette.accent;
        colors.button_primary_hover = palette.accent_hover;
        colors.button_primary_active = palette.accent_active;
        colors.button_primary_foreground = palette.accent_foreground;
        colors.button_secondary = button_surface;
        colors.button_secondary_hover = button_hover;
        colors.button_secondary_active = button_active;
        colors.button_secondary_foreground = palette.secondary_text;
        colors.button_danger = palette.error;
        colors.button_danger_hover = palette.error_hover;
        colors.button_danger_active = palette.error_active;
        colors.button_danger_foreground = palette.error_foreground;
        colors.button_warning = palette.warning;
        colors.button_warning_hover = palette.warning_hover;
        colors.button_warning_active = palette.warning_active;
        colors.button_warning_foreground = palette.warning_foreground;
        colors.button_success = palette.success;
        colors.button_success_hover = palette.success_hover;
        colors.button_success_active = palette.success_active;
        colors.button_success_foreground = palette.success_foreground;
        colors.button_info = palette.accent;
        colors.button_info_hover = palette.accent_hover;
        colors.button_info_active = palette.accent_active;
        colors.button_info_foreground = palette.accent_foreground;
    });

    Ok(())
}

/// A theme profile value could not be applied to GPUI Components.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ThemeApplyError {
    message: String,
}

impl std::fmt::Display for ThemeApplyError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for ThemeApplyError {}

fn positive_metric_as_f32(value: f64, name: &str) -> Result<f32, ThemeApplyError> {
    let converted = value as f32;
    if !value.is_finite() || value <= 0.0 || !converted.is_finite() || converted <= 0.0 {
        return Err(ThemeApplyError {
            message: format!(
                "RustUI theme metric `{name}` must be positive, finite, and representable as f32"
            ),
        });
    }
    Ok(converted)
}

fn unit_opacity_as_f32(value: f64, name: &str) -> Result<f32, ThemeApplyError> {
    let converted = value as f32;
    if !value.is_finite()
        || !(0.0..=1.0).contains(&value)
        || !converted.is_finite()
        || (value > 0.0 && converted == 0.0)
    {
        return Err(ThemeApplyError {
            message: format!(
                "RustUI theme opacity `{name}` must be finite, within 0..=1, and representable as f32"
            ),
        });
    }
    Ok(converted)
}

fn color_with_profile_opacity(
    color: Hsla,
    opacity: f64,
    name: &str,
) -> Result<Hsla, ThemeApplyError> {
    Ok(color.opacity(unit_opacity_as_f32(opacity, name)?))
}

struct ButtonPalette {
    window_background: Hsla,
    sidebar_background: Hsla,
    toolbar_background: Hsla,
    content_background: Hsla,
    selected_row: Hsla,
    chart_1: Hsla,
    chart_2: Hsla,
    chart_3: Hsla,
    chart_4: Hsla,
    chart_5: Hsla,
    surface: Hsla,
    surface_hover: Hsla,
    surface_active: Hsla,
    primary_text: Hsla,
    secondary_text: Hsla,
    border: Hsla,
    focus_ring: Hsla,
    accent: Hsla,
    accent_hover: Hsla,
    accent_active: Hsla,
    accent_foreground: Hsla,
    error: Hsla,
    error_hover: Hsla,
    error_active: Hsla,
    error_foreground: Hsla,
    warning: Hsla,
    warning_hover: Hsla,
    warning_active: Hsla,
    warning_foreground: Hsla,
    success: Hsla,
    success_hover: Hsla,
    success_active: Hsla,
    success_foreground: Hsla,
}

fn gpui_muted_foreground(palette: &ButtonPalette) -> Hsla {
    palette.secondary_text
}

fn gpui_secondary_background(palette: &ButtonPalette, toolbar_opacity: f32) -> Hsla {
    palette.toolbar_background.opacity(toolbar_opacity)
}

fn gpui_font_family(profile_family: &str, is_wasm: bool) -> String {
    match profile_family {
        "SF Pro" | "System" if is_wasm => String::from("Inter"),
        "SF Pro" | "System" => String::from(".SystemUIFont"),
        family => family.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_metrics_require_positive_finite_f32_values() {
        assert_eq!(
            positive_metric_as_f32(14.0, "font_size_body").unwrap(),
            14.0
        );

        for value in [
            0.0,
            -1.0,
            f64::NAN,
            f64::INFINITY,
            f64::MAX,
            f64::MIN_POSITIVE,
        ] {
            assert!(positive_metric_as_f32(value, "font_size_body").is_err());
        }
    }

    #[test]
    fn theme_material_opacity_requires_unit_interval_f32_values() {
        assert_eq!(
            unit_opacity_as_f32(0.0, "materials.material_opacity").unwrap(),
            0.0
        );
        assert_eq!(
            unit_opacity_as_f32(1.0, "materials.material_opacity").unwrap(),
            1.0
        );
        assert_eq!(
            unit_opacity_as_f32(0.42, "materials.material_opacity").unwrap(),
            0.42_f32
        );

        for value in [
            -1.0,
            f64::NAN,
            f64::INFINITY,
            f64::NEG_INFINITY,
            1.01,
            f64::MIN_POSITIVE,
        ] {
            assert!(unit_opacity_as_f32(value, "materials.material_opacity").is_err());
        }
    }

    #[test]
    fn sidebar_and_toolbar_profile_opacity_sets_color_alpha() {
        let color = gpui::hsla(0.5, 0.4, 0.3, 1.0);

        assert_eq!(
            color_with_profile_opacity(color, 0.35, "materials.sidebar_opacity")
                .unwrap()
                .a,
            0.35
        );
        assert_eq!(
            color_with_profile_opacity(color, 0.65, "materials.toolbar_opacity")
                .unwrap()
                .a,
            0.65
        );
    }

    #[test]
    fn custom_profile_font_family_reaches_gpui_theme() {
        assert_eq!(gpui_font_family("Avenir Next", false), "Avenir Next");
        assert_eq!(gpui_font_family("Avenir Next", true), "Avenir Next");
        assert_eq!(gpui_font_family("SF Pro", false), ".SystemUIFont");
        assert_eq!(gpui_font_family("SF Pro", true), "Inter");
        assert_eq!(gpui_font_family("System", false), ".SystemUIFont");
        assert_eq!(gpui_font_family("System", true), "Inter");
    }

    #[test]
    fn muted_foreground_matches_macos27_secondary_foreground_role() {
        let profile = crate::theme::parse_theme(include_str!("../../../../themes/macos27.json"))
            .expect("macOS 27 theme parses")
            .platform_snapshot;
        let palette = ButtonPalette::parse(&profile, AppearanceMode::Light)
            .expect("macOS 27 light palette resolves");

        let mapped = gpui_muted_foreground(&palette).to_rgb();
        assert_eq!(mapped, palette.secondary_text.to_rgb());
        let disabled_text = parse(&profile.colors(Some(AppearanceMode::Light)).disabled_text)
            .expect("macOS 27 disabled text color parses")
            .to_rgb();
        assert_ne!(mapped, disabled_text);
    }

    #[test]
    fn secondary_background_matches_macos27_toolbar_role() {
        let document = crate::theme::parse_theme(include_str!("../../../../themes/macos27.json"))
            .expect("macOS 27 theme parses");

        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let profile = &document.platform_snapshot;
            let palette =
                ButtonPalette::parse(profile, mode).expect("macOS 27 appearance palette resolves");
            let mapped = gpui_secondary_background(&palette, 1.0).to_rgb();
            let colors = profile.colors(Some(mode));
            let toolbar = parse(&colors.toolbar_background)
                .expect("macOS 27 toolbar color parses")
                .to_rgb();
            let elevated_surface = parse(&colors.elevated_panel_background)
                .expect("macOS 27 elevated surface color parses")
                .to_rgb();

            assert_eq!(mapped, toolbar, "{mode:?} secondary uses toolbar role");
            assert_ne!(
                mapped, elevated_surface,
                "{mode:?} distinguishes toolbar from panel"
            );
        }
    }

    #[test]
    fn accent_foreground_uses_contrasting_color_in_both_appearances() {
        let document = crate::theme::parse_theme(include_str!("../../../../themes/macos27.json"))
            .expect("macOS 27 theme parses");

        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let palette = ButtonPalette::parse(&document.platform_snapshot, mode)
                .expect("macOS 27 appearance palette resolves");
            assert_eq!(
                palette.accent_foreground.to_rgb(),
                contrasting_foreground(palette.accent).to_rgb(),
                "{mode:?} accent foreground contrasts with the accent surface"
            );
        }
    }
}

impl ButtonPalette {
    fn parse(profile: &ThemeProfile, mode: AppearanceMode) -> Result<Self, ThemeApplyError> {
        let colors = profile.colors(Some(mode));
        let window_background = parse(&colors.window_background)?;
        let sidebar_background = parse(&colors.sidebar_background)?;
        let toolbar_background = parse(&colors.toolbar_background)?;
        let content_background = parse(&colors.content_background)?;
        let selected_row = parse(&colors.selected_row_background)?;
        let chart_1 = parse(match mode {
            AppearanceMode::Light => &profile.accents.blue.light,
            AppearanceMode::Dark => &profile.accents.blue.dark,
        })?;
        let chart_2 = parse(match mode {
            AppearanceMode::Light => &profile.accents.purple.light,
            AppearanceMode::Dark => &profile.accents.purple.dark,
        })?;
        let chart_3 = parse(match mode {
            AppearanceMode::Light => &profile.accents.green.light,
            AppearanceMode::Dark => &profile.accents.green.dark,
        })?;
        let chart_4 = parse(match mode {
            AppearanceMode::Light => &profile.accents.orange.light,
            AppearanceMode::Dark => &profile.accents.orange.dark,
        })?;
        let chart_5 = parse(match mode {
            AppearanceMode::Light => &profile.accents.pink.light,
            AppearanceMode::Dark => &profile.accents.pink.dark,
        })?;
        let surface = parse(&colors.elevated_panel_background)?;
        let primary_text = parse(&colors.primary_text)?;
        let accent = parse(&colors.accent)?;
        let error = parse(&colors.error)?;
        let warning = parse(&colors.warning)?;
        let success = parse(&colors.success)?;
        let is_dark = matches!(mode, AppearanceMode::Dark);

        Ok(Self {
            window_background,
            sidebar_background,
            toolbar_background,
            content_background,
            selected_row,
            chart_1,
            chart_2,
            chart_3,
            chart_4,
            chart_5,
            surface,
            surface_hover: parse(&colors.hover_row_background)?,
            surface_active: parse(&colors.pressed_row_background)?,
            primary_text,
            secondary_text: parse(&colors.secondary_text)?,
            border: parse(&colors.border)?,
            focus_ring: parse(&colors.focus_ring)?,
            accent,
            accent_hover: shade(accent, is_dark, 0.08),
            accent_active: shade(accent, is_dark, 0.16),
            accent_foreground: contrasting_foreground(accent),
            error,
            error_hover: shade(error, is_dark, 0.08),
            error_active: shade(error, is_dark, 0.16),
            error_foreground: contrasting_foreground(error),
            warning,
            warning_hover: shade(warning, is_dark, 0.08),
            warning_active: shade(warning, is_dark, 0.16),
            warning_foreground: contrasting_foreground(warning),
            success,
            success_hover: shade(success, is_dark, 0.08),
            success_active: shade(success, is_dark, 0.16),
            success_foreground: contrasting_foreground(success),
        })
    }
}

fn parse(value: &str) -> Result<Hsla, ThemeApplyError> {
    try_parse_color(value).map_err(|_| ThemeApplyError {
        message: format!("could not parse RustUI theme color `{value}`"),
    })
}

fn shade(color: Hsla, dark_mode: bool, amount: f32) -> Hsla {
    let target = if dark_mode {
        rgb(0xffffff).into()
    } else {
        rgb(0x000000).into()
    };
    color.mix_oklab(target, 1.0 - amount)
}

fn contrasting_foreground(color: Hsla) -> Hsla {
    let components = color.to_rgb();
    let luminance = 0.2126 * linearize(components.r)
        + 0.7152 * linearize(components.g)
        + 0.0722 * linearize(components.b);
    let white_contrast = 1.05 / (luminance + 0.05);
    let black_contrast = (luminance + 0.05) / 0.05;

    if white_contrast > black_contrast {
        rgb(0xffffff).into()
    } else {
        rgb(0x000000).into()
    }
}

fn linearize(channel: f32) -> f32 {
    if channel <= 0.04045 {
        channel / 12.92
    } else {
        ((channel + 0.055) / 1.055).powf(2.4)
    }
}
