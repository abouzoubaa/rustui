use gpui::{
    AnyElement, FontWeight, InteractiveElement as _, IntoElement, ParentElement, Role,
    SharedString, StatefulInteractiveElement, Styled, div, px,
};

use crate::text::font_policy::FontFallbackPolicy;
use crate::view::{EmptyView, RenderContext, View};
use crate::{
    components::theme::{required_style, selector},
    localization::MessageArguments,
    theme::ButtonStyleSelector,
};
use rustui_core::Identity;

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
    weight: Option<TextWeight>,
    font_fallbacks: Option<FontFallbackPolicy>,
    vibrant: bool,
    arguments: Option<MessageArguments>,
    id: Option<Identity>,
}

#[derive(Clone, Debug)]
enum TextContent {
    Literal(SharedString),
    Message(SharedString),
}

impl TextView {
    pub fn new(value: impl Into<SharedString>) -> Self {
        let value = value.into();
        Self {
            content: TextContent::Literal(value),
            size: None,
            weight: None,
            font_fallbacks: None,
            vibrant: false,
            arguments: None,
            id: None,
        }
    }

    pub fn localized(identifier: impl Into<SharedString>) -> Self {
        let identifier = identifier.into();
        assert!(
            !identifier.trim().is_empty(),
            "LocalizedText message IDs must be nonempty"
        );
        Self {
            content: TextContent::Message(identifier),
            size: None,
            weight: None,
            font_fallbacks: None,
            vibrant: false,
            arguments: None,
            id: None,
        }
    }

    /// Interpolates named arguments in the active-locale message at render time.
    /// Interpolation failures remain typed at the render-context API and are
    /// displayed here so malformed catalogs or missing arguments are visible.
    pub fn with_arguments(mut self, arguments: MessageArguments) -> Self {
        self.arguments = Some(arguments);
        self
    }

    /// Sets a stable identity within this render owner, geometry reader, or grid cell.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Text identity must not be empty"
        );
        self.id = Some(id);
        self
    }

    pub fn font_size(mut self, size: f32) -> Self {
        assert!(size.is_finite(), "Text font size must be finite");
        self.size = Some(size.max(1.0));
        self
    }

    pub fn font_weight(mut self, weight: TextWeight) -> Self {
        self.weight = Some(weight);
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

fn resolved_font_weight(weight: Option<TextWeight>, theme_weight: f32) -> FontWeight {
    weight.map_or(FontWeight(theme_weight), TextWeight::gpui)
}

fn nonempty_accessible_label(value: &SharedString) -> Option<SharedString> {
    (!value.trim().is_empty()).then(|| value.clone())
}

impl View for TextView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let value = match &self.content {
            TextContent::Literal(value) => value.clone(),
            TextContent::Message(identifier) => {
                let value = match &self.arguments {
                    Some(arguments) => context
                        .localized_message_with_arguments(identifier.as_ref(), arguments)
                        .unwrap_or_else(|error| format!("[localization error: {error}]")),
                    None => context.localized_message(identifier.as_ref()),
                };
                SharedString::from(value)
            }
        };
        let accessible_label = nonempty_accessible_label(&value);
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
            .id(context.next_text_element_id_with(self.id.as_ref()))
            .role(Role::Label);
        if let Some(label) = accessible_label {
            text = text.aria_label(label);
        }
        text = text
            .font_family(typography.family.clone())
            .text_color(foreground)
            .text_size(px(self.size.unwrap_or(typography.size)))
            .font_weight(resolved_font_weight(self.weight, typography.weight));
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
    fn explicit_identity_is_validated() {
        assert!(std::panic::catch_unwind(|| Text("Title").id(" \t ")).is_err());
        assert_eq!(
            Text("Title").id("heading").id.unwrap(),
            Identity::from("heading")
        );
    }

    #[test]
    fn explicit_regular_weight_overrides_theme_weight() {
        assert_eq!(resolved_font_weight(None, 510.0), FontWeight(510.0));
        assert_eq!(
            resolved_font_weight(Some(TextWeight::Regular), 510.0),
            FontWeight::NORMAL
        );
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

    #[test]
    fn opaque_text_uses_primary_foreground_in_each_appearance() {
        let document =
            crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json"))
                .expect("macOS 27 theme parses");

        for (mode, expected_channel) in [
            (crate::theme::AppearanceMode::Light, 26.0 / 255.0),
            (crate::theme::AppearanceMode::Dark, 1.0),
        ] {
            let mut query = crate::components::theme::selector_for_mode(mode);
            let part = text_style_part(&mut query, false);
            let style =
                crate::theme::resolve_component_style_with(&document, "text", &part, &query, &[])
                    .expect("opaque text style resolves");
            let foreground = style
                .colors
                .get("foreground")
                .expect("opaque text declares its foreground");

            assert_eq!(foreground.r, expected_channel);
            assert_eq!(foreground.g, expected_channel);
            assert_eq!(foreground.b, expected_channel);
            assert_eq!(foreground.a, 1.0);
        }
    }

    #[test]
    fn blank_literal_text_is_allowed_without_an_accessible_name() {
        for value in ["", " \t "] {
            let text = Text(value);
            assert!(
                matches!(text.content, TextContent::Literal(ref content) if content.as_ref() == value)
            );
            assert_eq!(nonempty_accessible_label(&SharedString::from(value)), None);
        }
    }

    #[test]
    fn rejects_blank_localized_message_ids() {
        assert!(std::panic::catch_unwind(|| LocalizedText(" \t ")).is_err());
    }

    #[test]
    fn blank_localized_text_after_interpolation_omits_accessible_label() {
        let value =
            crate::localization::interpolate("{name}", &MessageArguments::new().with("name", ""))
                .unwrap();
        assert!(value.is_empty());
        assert_eq!(nonempty_accessible_label(&SharedString::from(value)), None);
        assert_eq!(nonempty_accessible_label(&SharedString::from(" \t ")), None);
        assert_eq!(
            nonempty_accessible_label(&SharedString::from("Welcome, Ada")),
            Some(SharedString::from("Welcome, Ada"))
        );
    }

    #[test]
    fn font_size_rejects_non_finite_values_and_clamps_finite_values() {
        for size in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(std::panic::catch_unwind(|| Text("Title").font_size(size)).is_err());
        }

        assert_eq!(Text("Title").font_size(0.0).size, Some(1.0));
        assert_eq!(Text("Title").font_size(-4.0).size, Some(1.0));
        assert_eq!(Text("Title").font_size(14.0).size, Some(14.0));
    }
}
