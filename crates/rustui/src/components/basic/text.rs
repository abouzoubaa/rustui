use gpui::{
    AnyElement, FontWeight, InteractiveElement as _, IntoElement, ParentElement, Role,
    SharedString, StatefulInteractiveElement, Styled, div, px,
};

use crate::text::font_policy::FontFallbackPolicy;
use crate::view::{EmptyView, RenderContext, View};
use crate::{
    components::theme::{required_style, selector},
    theme::ButtonStyleSelector,
};

/// A plain text view rendered by GPUI.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TextWeight {
    Light,
    #[default]
    Regular,
    Medium,
    Semibold,
    Bold,
}

impl TextWeight {
    fn gpui(self) -> FontWeight {
        match self {
            Self::Light => FontWeight::LIGHT,
            Self::Regular => FontWeight::NORMAL,
            Self::Medium => FontWeight::MEDIUM,
            Self::Semibold => FontWeight::SEMIBOLD,
            Self::Bold => FontWeight::BOLD,
        }
    }
}

#[derive(Clone, Debug)]
pub struct TextView {
    content: TextContent,
    size: Option<f32>,
    weight: TextWeight,
    font_fallbacks: Option<FontFallbackPolicy>,
    vibrant: bool,
}

#[derive(Clone, Debug)]
enum TextContent {
    Literal(SharedString),
    Message(SharedString),
}

impl TextView {
    pub fn new(value: impl Into<SharedString>) -> Self {
        Self {
            content: TextContent::Literal(value.into()),
            size: None,
            weight: TextWeight::Regular,
            font_fallbacks: None,
            vibrant: false,
        }
    }

    pub fn localized(identifier: impl Into<SharedString>) -> Self {
        Self {
            content: TextContent::Message(identifier.into()),
            size: None,
            weight: TextWeight::Regular,
            font_fallbacks: None,
            vibrant: false,
        }
    }

    pub fn font_size(mut self, size: f32) -> Self {
        self.size = Some(size.max(1.0));
        self
    }

    pub fn font_weight(mut self, weight: TextWeight) -> Self {
        self.weight = weight;
        self
    }

    /// Uses the theme's vibrant text treatment for placement over a vibrant
    /// surface. This selects foreground styling; it does not draw that surface.
    pub fn vibrant(mut self) -> Self {
        self.vibrant = true;
        self
    }

    /// Sets caller supplied fallback families for GPUI text shaping.
    pub fn font_fallbacks(mut self, policy: FontFallbackPolicy) -> Self {
        self.font_fallbacks = Some(policy);
        self
    }
}

/// Creates a text view using SwiftUI-like constructor syntax.
#[allow(non_snake_case)]
pub fn Text(value: impl Into<SharedString>) -> TextView {
    TextView::new(value)
}

/// Creates text resolved from the active RustUI message catalogs.
#[allow(non_snake_case)]
pub fn LocalizedText(identifier: impl Into<SharedString>) -> TextView {
    TextView::localized(identifier)
}

fn text_style_part(query: &mut ButtonStyleSelector, vibrant: bool) -> String {
    query.material = if vibrant { "vibrant" } else { "opaque" }.into();
    format!(
        "text_styles_left_aligned_{}_{}_body",
        query.appearance, query.material
    )
}

impl View for TextView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let value = match &self.content {
            TextContent::Literal(value) => value.clone(),
            TextContent::Message(identifier) => {
                SharedString::from(context.localized_message(identifier.as_ref()))
            }
        };
        let mut query = selector(context);
        let part = text_style_part(&mut query, self.vibrant);
        let style = if self.vibrant {
            // macos27's vibrant text rule also carries blend_mode. Accept the
            // declared extra so foreground and typography can resolve; this
            // component intentionally does not apply blend modes yet.
            context
                .component_style_with("text", &part, &query, &["blend_mode"])
                .unwrap_or_else(|error| panic!("text/{part} theme cannot resolve: {error}"))
        } else {
            required_style(context, "text", &part, &query)
        };
        let typography = style
            .typography
            .as_ref()
            .expect("text theme needs typography");
        let foreground = crate::theme::registry::hsla(
            *style
                .colors
                .get("foreground")
                .expect("text theme needs foreground"),
        );
        let mut text = div()
            .id(context.next_text_element_id())
            .role(Role::Label)
            .aria_label(value.clone())
            .font_family(typography.family.clone())
            .text_color(foreground)
            .text_size(px(self.size.unwrap_or(typography.size)))
            .font_weight(if self.weight == TextWeight::Regular {
                FontWeight(typography.weight)
            } else {
                self.weight.gpui()
            });
        if let Some(policy) = &self.font_fallbacks {
            text.style().text.font_fallbacks = Some(policy.as_gpui().clone());
        }
        text = text.child(value);
        text.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_defaults_to_opaque_and_can_select_vibrant_treatment() {
        let opaque = Text("Title");
        let vibrant = Text("Title").vibrant();

        assert!(!opaque.vibrant);
        assert!(vibrant.vibrant);
    }

    #[test]
    fn text_material_selects_each_macos27_appearance_rule() {
        let document =
            crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();

        for (appearance, expected_part) in [
            ("light", "text_styles_left_aligned_light_vibrant_body"),
            ("dark", "text_styles_left_aligned_dark_vibrant_body"),
        ] {
            let mut query = crate::components::theme::selector_for_mode(if appearance == "light" {
                crate::theme::AppearanceMode::Light
            } else {
                crate::theme::AppearanceMode::Dark
            });
            let part = text_style_part(&mut query, true);
            assert_eq!(part, expected_part);
            let style = crate::theme::resolve_component_style_with(
                &document,
                "text",
                &part,
                &query,
                &["blend_mode"],
            )
            .unwrap();
            assert!(style.colors.contains_key("foreground"));
            assert!(style.extra_properties.contains_key("blend_mode"));
        }
    }
}
