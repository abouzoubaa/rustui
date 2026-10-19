//! Internal bridge from active RustUI component records to renderers.

use gpui::{App, Div, FontWeight, Styled, px};

use crate::{
    theme::{AppearanceMode, ButtonStyleSelector, ResolvedComponentStyle, registry::active_theme},
    view::RenderContext,
};

pub(crate) fn selector_for_mode(mode: AppearanceMode) -> ButtonStyleSelector {
    let appearance = match mode {
        AppearanceMode::Light => "light",
        AppearanceMode::Dark => "dark",
    };
    ButtonStyleSelector {
        appearance: appearance.into(),
        material: "opaque".into(),
        interaction: "rest".into(),
        text_presence: "present".into(),
        size: "large".into(),
        orientation: "horizontal".into(),
        chrome: "unified_toolbar".into(),
        modality: "modeless".into(),
    }
}

pub(crate) fn selector(context: &RenderContext<'_>) -> ButtonStyleSelector {
    selector_for_mode(
        active_theme(context.app())
            .expect("a RustUI component theme requires an active theme")
            .mode,
    )
}

pub(crate) fn required_style(
    context: &RenderContext<'_>,
    component: &str,
    part: &str,
    selector: &ButtonStyleSelector,
) -> ResolvedComponentStyle {
    context
        .component_style(component, part, selector)
        .unwrap_or_else(|error| panic!("{component}/{part} theme cannot resolve: {error}"))
}

/// SVG source coordinates and filter graphs are reference artwork, not GPUI
/// element properties. Callers must still apply every returned typed property.
pub(crate) fn required_style_with_artwork(
    context: &RenderContext<'_>,
    component: &str,
    part: &str,
    selector: &ButtonStyleSelector,
) -> ResolvedComponentStyle {
    context
        .component_style_with(
            component,
            part,
            selector,
            &["vector_path", "filter", "blend_mode"],
        )
        .unwrap_or_else(|error| panic!("{component}/{part} theme cannot resolve: {error}"))
}

pub(crate) fn required_app_style_with_artwork(
    cx: &App,
    component: &str,
    part: &str,
) -> ResolvedComponentStyle {
    let active = active_theme(cx).expect("a RustUI component theme requires an active theme");
    active
        .component_style_with(
            component,
            part,
            &selector_for_mode(active.mode),
            &["vector_path", "filter", "blend_mode"],
        )
        .unwrap_or_else(|error| panic!("{component}/{part} theme cannot resolve: {error}"))
}

/// Apply common resolved properties to a GPUI container around a native control.
pub(crate) fn styled_element<T: Styled>(mut container: T, style: &ResolvedComponentStyle) -> T {
    if let Some(color) = style.colors.get("background") {
        container = container.bg(crate::theme::registry::hsla(*color));
    }
    if let Some(color) = style.colors.get("foreground") {
        container = container.text_color(crate::theme::registry::hsla(*color));
    }
    if let Some(value) = style.lengths.get("width") {
        container = container.w(px(*value));
    }
    if let Some(value) = style.lengths.get("height") {
        container = container.h(px(*value));
    }
    if let Some(value) = style.lengths.get("corner_radius") {
        container = container.rounded(px(*value));
    }
    // A color by itself cannot draw a border on a fresh GPUI container.
    if let (Some(color), Some(width)) = (
        style.colors.get("border"),
        style.lengths.get("border_width"),
    ) {
        container = container
            .border(px(*width))
            .border_color(crate::theme::registry::hsla(*color));
    }
    if let Some(typography) = &style.typography {
        container = container
            .font_family(typography.family.clone())
            .text_size(px(typography.size))
            .font_weight(FontWeight(typography.weight));
    }
    container
}

pub(crate) fn styled_container(container: Div, style: &ResolvedComponentStyle) -> Div {
    styled_element(container, style)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::theme::{parse_theme, resolve_component_style, resolve_component_style_with};

    #[test]
    fn paired_border_properties_reach_gpui_style_refinement() {
        let document = parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let selector = selector_for_mode(AppearanceMode::Dark);
        let resolved = resolve_component_style_with(
            &document,
            "window_controls",
            "close",
            &selector,
            &["blend_mode"],
        )
        .unwrap();
        let mut element = styled_container(gpui::div(), &resolved);
        let refinement = element.style();
        let width = *resolved.lengths.get("border_width").unwrap();
        assert!(
            refinement.border_color
                == Some(crate::theme::registry::hsla(
                    *resolved.colors.get("border").unwrap()
                ))
        );
        let expected_width = gpui::AbsoluteLength::Pixels(px(width));
        assert_eq!(refinement.border_widths.top, Some(expected_width));
        assert_eq!(refinement.border_widths.right, Some(expected_width));
        assert_eq!(refinement.border_widths.bottom, Some(expected_width));
        assert_eq!(refinement.border_widths.left, Some(expected_width));
    }

    #[test]
    fn rendered_parts_resolve_for_both_appearances() {
        let document = parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let query = selector_for_mode(mode);
            let parts = [
                ("link", "body", false),
                (
                    "text",
                    if mode == AppearanceMode::Light {
                        "text_styles_left_aligned_light_opaque_body"
                    } else {
                        "text_styles_left_aligned_dark_opaque_body"
                    },
                    false,
                ),
                (
                    "label",
                    if mode == AppearanceMode::Light {
                        "text_styles_left_aligned_light_opaque_label_color"
                    } else {
                        "text_styles_left_aligned_dark_opaque_label_color"
                    },
                    false,
                ),
                ("icon", "icon", false),
                ("image", "image", true),
                ("divider", "list_separator", true),
                ("divider", "menubar_separator", true),
                ("outline_view", "outline_view", false),
                ("table", "table_rows", false),
                ("table", "table_column_headers", true),
                ("disclosure_control", "disclosure", false),
                ("menu", "body", true),
                ("notification", "body", true),
            ];
            for (component, part, artwork) in parts {
                let result = if artwork {
                    resolve_component_style_with(
                        &document,
                        component,
                        part,
                        &query,
                        &["vector_path", "filter", "blend_mode"],
                    )
                } else {
                    resolve_component_style(&document, component, part, &query)
                };
                assert!(result.is_ok(), "{component}/{part} {mode:?}: {result:?}");
            }
        }
    }

    #[test]
    fn source_artwork_requires_explicit_classification() {
        let document = parse_theme(include_str!("../../../../themes/macos27.json")).unwrap();
        let query = selector_for_mode(AppearanceMode::Dark);
        assert!(resolve_component_style(&document, "image", "image", &query).is_err());
        for component in ["sidebar", "toolbar", "tab_bar"] {
            assert!(resolve_component_style(&document, component, "body", &query).is_err());
        }
    }
}
