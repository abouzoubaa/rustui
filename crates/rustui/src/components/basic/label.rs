//! Text labels with optional secondary content and an adjacent icon.

use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, SharedString, Styled, div, px};
use gpui_component::label::Label as GpuiLabel;
use rustui_i18n::LayoutDirection;

use crate::{
    components::{
        basic::icon::{IconName, IconView},
        theme::{required_style, selector},
    },
    theme::ButtonStyleSelector,
    view::{EmptyView, RenderContext, View},
};

/// A semantic GPUI Components label with optional secondary text and icon.
#[derive(Clone)]
pub struct LabelView {
    text: SharedString,
    secondary: Option<SharedString>,
    icon: Option<IconName>,
    vibrant: bool,
}

impl LabelView {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            secondary: None,
            icon: None,
            vibrant: false,
        }
    }

    /// Adds muted secondary text using GPUI Components' label styling.
    pub fn secondary(mut self, text: impl Into<SharedString>) -> Self {
        self.secondary = Some(text.into());
        self
    }

    /// Adds a decorative icon before the visible label text.
    pub fn icon(mut self, icon: IconName) -> Self {
        self.icon = Some(icon);
        self
    }

    /// Uses the theme's vibrant label treatment for placement over a vibrant
    /// surface. This selects foreground styling; it does not draw that surface.
    pub fn vibrant(mut self) -> Self {
        self.vibrant = true;
        self
    }
}

/// Creates a RustUI text label.
#[allow(non_snake_case)]
pub fn Label(text: impl Into<SharedString>) -> LabelView {
    LabelView::new(text)
}

fn label_style_part(query: &mut ButtonStyleSelector, vibrant: bool) -> String {
    query.material = if vibrant { "vibrant" } else { "opaque" }.into();
    format!(
        "text_styles_left_aligned_{}_{}_label_color",
        query.appearance, query.material
    )
}

impl View for LabelView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut query = selector(context);
        let part = label_style_part(&mut query, self.vibrant);
        let style = required_style(context, "label", &part, &query);
        let typography = style
            .typography
            .as_ref()
            .expect("label theme needs typography");
        let mut label = GpuiLabel::new(self.text.clone())
            .text_color(crate::theme::registry::hsla(
                *style
                    .colors
                    .get("foreground")
                    .expect("label theme needs foreground"),
            ))
            .font_family(typography.family.clone())
            .text_size(px(typography.size))
            .font_weight(FontWeight(typography.weight));
        if let Some(secondary) = &self.secondary {
            label = label.secondary(secondary.clone());
        }

        if let Some(icon) = &self.icon {
            let row = div().flex().items_center().gap(px(6.0));
            let row = match context.layout_direction() {
                LayoutDirection::LeftToRight => row.flex_row(),
                LayoutDirection::RightToLeft => row.flex_row_reverse(),
            };
            row.child(IconView::new(icon.clone()).into_element(context))
                .child(label)
                .into_any_element()
        } else {
            label.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn label_accepts_secondary_text_and_an_icon() {
        let label = Label("Search").secondary("⌘K").icon(IconName::Search);
        assert_eq!(label.text.as_ref(), "Search");
        assert_eq!(label.secondary.as_deref(), Some("⌘K"));
        assert!(label.icon.is_some());
    }

    #[test]
    fn label_defaults_to_opaque_and_can_select_vibrant_treatment() {
        let opaque = Label("Search");
        let vibrant = Label("Search").vibrant();

        assert!(!opaque.vibrant);
        assert!(vibrant.vibrant);
    }

    #[test]
    fn label_material_selects_each_macos27_appearance_rule() {
        let document =
            crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();

        for (appearance, expected_part) in [
            (
                "light",
                "text_styles_left_aligned_light_vibrant_label_color",
            ),
            ("dark", "text_styles_left_aligned_dark_vibrant_label_color"),
        ] {
            let mut query = crate::components::theme::selector_for_mode(if appearance == "light" {
                crate::theme::AppearanceMode::Light
            } else {
                crate::theme::AppearanceMode::Dark
            });
            let part = label_style_part(&mut query, true);
            assert_eq!(part, expected_part);
            let style =
                crate::theme::resolve_component_style(&document, "label", &part, &query).unwrap();
            assert!(style.colors.contains_key("foreground"));
            assert!(style.typography.is_some());
            assert!(!style.extra_properties.contains_key("blend_mode"));
        }
    }
}
