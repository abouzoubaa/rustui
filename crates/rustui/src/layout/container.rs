//! A styled box for composing RustUI views.

use gpui::{AnyElement, BoxShadow, Hsla, IntoElement, ParentElement, Styled, div, px};

use crate::modifiers::padding::LogicalInsets;
use crate::view::{EmptyView, RenderContext, View};

/// Semantic surfaces resolved from the active GPUI Components theme.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Background,
    Sidebar,
    Panel,
}

/// Wraps a RustUI view in an explicit layout box.
#[allow(non_snake_case)]
pub fn Container<V: View>(content: V) -> ContainerView<V> {
    ContainerView {
        content,
        width: None,
        height: None,
        padding: None,
        padding_edges: None,
        radius: None,
        fill: false,
        grow: false,
        border: false,
        background: None,
        opacity: None,
        shadow: None,
    }
}

/// A column layout box with optional size and theme styling.
pub struct ContainerView<V: View> {
    content: V,
    width: Option<f32>,
    height: Option<f32>,
    padding: Option<f32>,
    padding_edges: Option<LogicalInsets>,
    radius: Option<f32>,
    fill: bool,
    grow: bool,
    border: bool,
    background: Option<Surface>,
    opacity: Option<f32>,
    shadow: Option<BoxShadow>,
}

impl<V: View> ContainerView<V> {
    /// Sets a fixed width in logical pixels.
    pub fn width(mut self, width: f32) -> Self {
        assert!(
            width.is_finite() && width > 0.0,
            "width must be finite and positive"
        );
        self.width = Some(width);
        self
    }

    /// Sets a fixed height in logical pixels.
    pub fn height(mut self, height: f32) -> Self {
        assert!(
            height.is_finite() && height > 0.0,
            "height must be finite and positive"
        );
        self.height = Some(height);
        self
    }

    /// Adds equal padding on all sides in logical pixels.
    pub fn padding(mut self, padding: f32) -> Self {
        assert!(
            padding.is_finite() && padding >= 0.0,
            "padding must be finite and nonnegative"
        );
        self.padding = Some(padding);
        self.padding_edges = None;
        self
    }

    /// Sets logical top, bottom, start, and end insets in logical pixels.
    ///
    /// Start/end resolve against the active layout direction when this view is
    /// lowered. The last call to `padding` or `padding_edges` wins.
    pub fn padding_edges(mut self, insets: LogicalInsets) -> Self {
        self.padding = None;
        self.padding_edges = Some(insets);
        self
    }

    /// Sets both dimensions to the available parent size.
    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    /// Takes available space along the parent's flex axis.
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }

    /// Draws a one pixel border using the active theme.
    pub fn border(mut self) -> Self {
        self.border = true;
        self
    }

    /// Rounds the box corners by the given logical pixel radius.
    pub fn radius(mut self, radius: f32) -> Self {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "radius must be finite and nonnegative"
        );
        self.radius = Some(radius);
        self
    }

    /// Fills the box with a semantic surface from the active theme.
    pub fn background(mut self, surface: Surface) -> Self {
        self.background = Some(surface);
        self
    }

    /// Sets the opacity of this box and its child, from transparent to opaque.
    pub fn opacity(mut self, opacity: f32) -> Self {
        assert!(
            opacity.is_finite() && (0.0..=1.0).contains(&opacity),
            "opacity must be finite and in 0..=1"
        );
        self.opacity = Some(opacity);
        self
    }

    /// Sets one outer shadow using logical-pixel offsets and radii.
    ///
    /// Offsets must be finite. Blur radius must be finite and nonnegative;
    /// spread radius may be negative, matching GPUI's shadow geometry.
    pub fn shadow(
        mut self,
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        spread_radius: f32,
        color: Hsla,
    ) -> Self {
        assert!(
            offset_x.is_finite() && offset_y.is_finite(),
            "shadow offsets must be finite"
        );
        assert!(
            blur_radius.is_finite() && blur_radius >= 0.0,
            "shadow blur radius must be finite and nonnegative"
        );
        assert!(
            spread_radius.is_finite(),
            "shadow spread radius must be finite"
        );
        self.shadow = Some(
            BoxShadow::new(px(offset_x), px(offset_y), color)
                .blur_radius(px(blur_radius))
                .spread_radius(px(spread_radius)),
        );
        self
    }
}

impl<V: View> View for ContainerView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let theme = gpui_component::Theme::global(context.app());
        let colors = &theme.colors;
        let mut box_view = div()
            .flex()
            .flex_col()
            .text_color(colors.foreground)
            .text_size(theme.font_size);
        if self.fill {
            box_view = box_view.size_full().min_w_0().min_h_0();
        }
        if self.grow {
            box_view = box_view.flex_1().min_w_0().min_h_0();
        }
        if let Some(width) = self.width {
            box_view = box_view.w(px(width)).flex_shrink_0();
        }
        if let Some(height) = self.height {
            box_view = box_view.h(px(height)).flex_shrink_0();
        }
        if let Some(padding) = self.padding {
            box_view = box_view.p(px(padding));
        }
        if let Some(insets) = self.padding_edges {
            let insets = insets.resolve(context.layout_direction());
            box_view = box_view
                .pt(px(insets.top))
                .pr(px(insets.right))
                .pb(px(insets.bottom))
                .pl(px(insets.left));
        }
        if let Some(radius) = self.radius {
            box_view = box_view.rounded(px(radius));
        }
        if self.border {
            box_view = box_view.border_1().border_color(colors.border);
        }
        if let Some(surface) = self.background {
            let color = match surface {
                Surface::Background => colors.background,
                Surface::Sidebar => colors.sidebar,
                Surface::Panel => colors.secondary,
            };
            box_view = box_view.bg(color);
        }
        if let Some(opacity) = self.opacity {
            box_view = box_view.opacity(opacity);
        }
        if let Some(shadow) = &self.shadow {
            box_view = box_view.shadow(vec![shadow.clone()]);
        }
        box_view
            .child(self.content.into_element(context))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::Container;
    use crate::view::EmptyView;
    use gpui::Hsla;

    #[test]
    fn validates_static_opacity_and_keeps_opaque_default() {
        assert_eq!(Container(EmptyView).opacity, None);
        for value in [0.0, 0.5, 1.0] {
            assert_eq!(Container(EmptyView).opacity(value).opacity, Some(value));
        }
        assert_eq!(
            Container(EmptyView).opacity(0.25).opacity(0.75).opacity,
            Some(0.75)
        );
        for value in [-0.01, 1.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(std::panic::catch_unwind(|| Container(EmptyView).opacity(value)).is_err());
        }
    }

    #[test]
    fn validates_and_replaces_container_shadow() {
        let color = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 0.5,
        };
        let no_shadow = Container(EmptyView);
        assert!(no_shadow.shadow.is_none());

        let first = Container(EmptyView).shadow(2.0, -3.0, 4.0, -1.5, color);
        let shadow = first.shadow.as_ref().expect("shadow is stored");
        assert_eq!(shadow.offset.x, gpui::px(2.0));
        assert_eq!(shadow.offset.y, gpui::px(-3.0));
        assert_eq!(shadow.blur_radius, gpui::px(4.0));
        assert_eq!(shadow.spread_radius, gpui::px(-1.5));
        assert_eq!(shadow.color, color);
        assert!(!shadow.inset);

        let replacement = first.shadow(0.0, 1.0, 2.0, 3.0, color);
        assert_eq!(
            replacement.shadow.as_ref().map(|shadow| shadow.offset.y),
            Some(gpui::px(1.0))
        );
    }

    #[test]
    fn rejects_non_finite_offsets_and_invalid_blur_or_spread() {
        let color = Hsla {
            h: 0.0,
            s: 0.0,
            l: 0.0,
            a: 1.0,
        };
        for (offset_x, offset_y) in [
            (f32::NAN, 0.0),
            (0.0, f32::INFINITY),
            (f32::NEG_INFINITY, 0.0),
        ] {
            assert!(
                std::panic::catch_unwind(|| {
                    Container(EmptyView).shadow(offset_x, offset_y, 0.0, 0.0, color)
                })
                .is_err()
            );
        }
        for blur in [-0.1, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                std::panic::catch_unwind(|| {
                    Container(EmptyView).shadow(0.0, 0.0, blur, 0.0, color)
                })
                .is_err()
            );
        }
        for spread in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                std::panic::catch_unwind(|| {
                    Container(EmptyView).shadow(0.0, 0.0, 0.0, spread, color)
                })
                .is_err()
            );
        }
    }
}
