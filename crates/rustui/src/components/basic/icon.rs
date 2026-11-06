//! Named icons adapted from the pinned GPUI Components catalog.

use gpui::InteractiveElement as _;
use gpui::{
    AnyElement, IntoElement, ParentElement, SharedString, StatefulInteractiveElement, Styled, div,
    px,
};
use rustui_core::Identity;

use crate::{
    components::theme::{required_style, required_style_with_artwork, selector},
    view::{EmptyView, RenderContext, View},
};

/// The pinned GPUI Components icon-name enum and its supported icon subset.
pub use gpui_component::IconName;

#[derive(Clone)]
enum IconAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

/// A named icon with optional explicit sizing and an accessible name.
#[derive(Clone)]
pub struct IconView {
    name: IconName,
    size: Option<f32>,
    clear: bool,
    clear_pressed: bool,
    accessibility_label: Option<IconAccessibilityLabel>,
    id: Option<Identity>,
}

impl IconView {
    pub fn new(name: IconName) -> Self {
        Self {
            name,
            size: None,
            clear: false,
            clear_pressed: false,
            accessibility_label: None,
            id: None,
        }
    }

    /// Sets the icon's width and height in logical pixels.
    pub fn size(mut self, size: f32) -> Self {
        assert!(
            size.is_finite() && size > 0.0,
            "icon size must be finite and positive"
        );
        self.size = Some(size);
        self
    }

    /// Renders the macOS 27 clear-icon artwork in its resting state.
    ///
    /// The icon name is ignored in this role; the path comes from the active theme.
    pub fn clear(mut self) -> Self {
        self.clear = true;
        self.clear_pressed = false;
        self
    }

    /// Renders the macOS 27 clear-icon artwork in its pressed state.
    ///
    /// Bind this state to the enclosing control's interaction state.
    pub fn clear_pressed(mut self) -> Self {
        self.clear = true;
        self.clear_pressed = true;
        self
    }

    /// Names the icon for assistive technology when it conveys meaning alone.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Icon accessibility label must not be empty"
        );
        self.accessibility_label = Some(IconAccessibilityLabel::Literal(label));
        self
    }

    /// Localizes the icon's accessible name through the active RustUI catalog.
    ///
    /// Set an explicit `.accessibility_label(...)` first; icons have no visible
    /// text to use as a fallback when the catalog entry is missing.
    pub fn localized_accessibility_label(mut self, id: impl Into<SharedString>) -> Self {
        let id = id.into();
        assert!(
            !id.trim().is_empty(),
            "Icon accessibility message ID must not be empty"
        );
        let fallback = match self.accessibility_label.as_ref() {
            Some(IconAccessibilityLabel::Literal(label)) => label.clone(),
            Some(IconAccessibilityLabel::Message { fallback, .. }) => fallback.clone(),
            None => {
                panic!("Icon localized accessibility label requires an explicit fallback label")
            }
        };
        self.accessibility_label = Some(IconAccessibilityLabel::Message { id, fallback });
        self
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> Option<SharedString> {
        match &self.accessibility_label {
            Some(IconAccessibilityLabel::Literal(label)) => Some(label.clone()),
            Some(IconAccessibilityLabel::Message { id, fallback }) => Some(resolved_message_label(
                localized_message(id.as_ref()),
                id,
                fallback,
            )),
            None => None,
        }
    }

    /// Sets a stable element identity. Repeated labeled icons should use distinct IDs.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Icon identity must not be empty"
        );
        self.id = Some(id);
        self
    }
}

fn resolved_message_label(
    localized: String,
    id: &SharedString,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.clone()
    } else {
        localized.into()
    }
}

/// Creates a RustUI icon from the pinned GPUI Components icon-name catalog.
#[allow(non_snake_case)]
pub fn Icon(name: IconName) -> IconView {
    IconView::new(name)
}

impl View for IconView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut style_selector = selector(context);
        let part = if self.clear {
            style_selector.interaction = if self.clear_pressed {
                "pressed"
            } else {
                "rest"
            }
            .into();
            "clear"
        } else {
            "icon"
        };
        let style = if self.clear {
            required_style_with_artwork(context, "icon", part, &style_selector)
        } else {
            required_style(context, "icon", part, &style_selector)
        };
        let mut icon = if self.clear {
            let vector_path = style.extra_properties["vector_path"]["value"]["data"]
                .as_str()
                .expect("icon/clear theme vector_path must contain SVG path data");
            // The theme path is authored in a 56×90 artboard, with the resting
            // and pressed marks at separate vertical offsets. Crop to the
            // selected mark so GPUI's normal icon sizing fills the element.
            let view_box = if self.clear_pressed {
                "20 54 16 16"
            } else {
                "20 20 16 16"
            };
            let svg = format!(
                "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{view_box}\"><path d=\"{vector_path}\" fill=\"currentColor\"/></svg>"
            );
            let mut icon = gpui_component::Icon::default().data(svg.as_bytes());
            if let Some(color) = style.colors.get("background") {
                icon = icon.text_color(crate::theme::registry::hsla(*color));
            }
            icon
        } else {
            gpui_component::Icon::new(self.name.clone())
        };
        if let Some(size) = self.size {
            icon = icon.size(px(size));
        } else if !self.clear {
            if let Some(width) = style.lengths.get("width") {
                icon = icon.w(px(*width));
            }
            if let Some(height) = style.lengths.get("height") {
                icon = icon.h(px(*height));
            }
        }
        if let Some(label) = self.resolved_accessibility_label(|id| context.localized_message(id)) {
            let id = self
                .id
                .clone()
                .unwrap_or_else(|| Identity::from(format!("icon:{label}")));
            div()
                .id(id.to_string())
                .role(gpui::Role::Image)
                .aria_label(label.clone())
                .child(icon)
                .into_any_element()
        } else if let Some(id) = &self.id {
            div().id(id.to_string()).child(icon).into_any_element()
        } else {
            icon.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explicit_size_must_be_finite_and_positive() {
        assert!(Icon(IconName::Search).size(16.0).size.is_some());
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text_verbatim() {
        let icon = Icon(IconName::Search).accessibility_label(" Search ");
        assert!(matches!(
            icon.accessibility_label,
            Some(IconAccessibilityLabel::Literal(label)) if label.as_ref() == " Search "
        ));
    }

    #[test]
    #[should_panic(expected = "Icon accessibility label must not be empty")]
    fn explicit_accessible_name_rejects_whitespace_only_text() {
        Icon(IconName::Search).accessibility_label(" \t\n ");
    }

    #[test]
    fn rejects_blank_explicit_identity() {
        for id in ["", " \t\n"] {
            assert!(std::panic::catch_unwind(|| Icon(IconName::Search).id(id)).is_err());
        }
    }

    #[test]
    fn localized_accessible_name_uses_catalog_and_explicit_fallback() {
        let icon = Icon(IconName::Search)
            .accessibility_label("Search")
            .localized_accessibility_label("icon.search");
        assert!(matches!(
            &icon.accessibility_label,
            Some(IconAccessibilityLabel::Message { id, fallback })
                if id.as_ref() == "icon.search" && fallback.as_ref() == "Search"
        ));
        assert_eq!(
            icon.resolved_accessibility_label(|_| "Rechercher".to_owned())
                .as_deref(),
            Some("Rechercher")
        );
        assert_eq!(
            icon.resolved_accessibility_label(|_| "icon.search".to_owned())
                .as_deref(),
            Some("Search")
        );
        assert_eq!(
            icon.resolved_accessibility_label(|_| " icon.search \t".to_owned())
                .as_deref(),
            Some("Search")
        );
        assert_eq!(
            icon.resolved_accessibility_label(|_| " \t".to_owned())
                .as_deref(),
            Some("Search")
        );
    }

    #[test]
    #[should_panic(expected = "Icon accessibility message ID must not be empty")]
    fn localized_accessible_name_rejects_blank_message_id() {
        Icon(IconName::Search)
            .accessibility_label("Search")
            .localized_accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(
        expected = "Icon localized accessibility label requires an explicit fallback label"
    )]
    fn localized_accessible_name_requires_explicit_fallback() {
        Icon(IconName::Search).localized_accessibility_label("icon.search");
    }

    #[test]
    fn clear_icon_exposes_both_theme_interaction_states() {
        assert!(Icon(IconName::Search).clear().clear);
        assert!(!Icon(IconName::Search).clear().clear_pressed);
        assert!(Icon(IconName::Search).clear_pressed().clear_pressed);
    }

    #[test]
    #[should_panic(expected = "icon size must be finite and positive")]
    fn explicit_size_rejects_zero() {
        Icon(IconName::Search).size(0.0);
    }

    #[test]
    #[should_panic(expected = "icon size must be finite and positive")]
    fn explicit_size_rejects_non_finite_values() {
        Icon(IconName::Search).size(f32::NAN);
    }
}
