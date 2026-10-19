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

/// A named icon with optional explicit sizing and an accessible name.
#[derive(Clone)]
pub struct IconView {
    name: IconName,
    size: Option<f32>,
    clear: bool,
    clear_pressed: bool,
    accessibility_label: Option<SharedString>,
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
        self.accessibility_label = Some(label.into());
        self
    }

    /// Sets a stable element identity. Repeated labeled icons should use distinct IDs.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
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
        if let Some(label) = &self.accessibility_label {
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
