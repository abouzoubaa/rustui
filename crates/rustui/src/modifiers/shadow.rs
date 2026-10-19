//! Native drop shadows with theme elevations and validated explicit parameters.

use std::{error::Error, fmt};

use crate::view::{EmptyView, RenderContext, RetentionScope, View};
use gpui::{
    AnyElement, BoxShadow, Div, Hsla, IntoElement as _, ParentElement as _, Styled as _, div, px,
};
use rustui_core::Identity;

/// Elevations supplied by the active upstream theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ShadowElevation {
    Small,
    #[default]
    Medium,
    Large,
}

/// Validated parameters for one native outer shadow, in logical pixels.
/// Negative offsets and spread are supported. Blur must be nonnegative.
/// Color channels must be finite and in 0..=1, matching Hsla's native contract.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShadowStyle {
    offset_x: f32,
    offset_y: f32,
    blur_radius: f32,
    spread_radius: f32,
    color: Hsla,
}

impl ShadowStyle {
    pub fn new(
        offset_x: f32,
        offset_y: f32,
        blur_radius: f32,
        spread_radius: f32,
        color: Hsla,
    ) -> Result<Self, ShadowStyleError> {
        if !offset_x.is_finite() || !offset_y.is_finite() {
            return Err(ShadowStyleError::InvalidOffset);
        }
        if !blur_radius.is_finite() || blur_radius < 0.0 {
            return Err(ShadowStyleError::InvalidBlur);
        }
        if !spread_radius.is_finite() {
            return Err(ShadowStyleError::InvalidSpread);
        }
        if ![color.h, color.s, color.l, color.a]
            .into_iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(&value))
        {
            return Err(ShadowStyleError::InvalidColor);
        }
        Ok(Self {
            offset_x,
            offset_y,
            blur_radius,
            spread_radius,
            color,
        })
    }

    fn native(self) -> BoxShadow {
        BoxShadow::new(px(self.offset_x), px(self.offset_y), self.color)
            .blur_radius(px(self.blur_radius))
            .spread_radius(px(self.spread_radius))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShadowStyleError {
    InvalidOffset,
    InvalidBlur,
    InvalidSpread,
    InvalidColor,
}

impl fmt::Display for ShadowStyleError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidOffset => "shadow offsets must be finite",
            Self::InvalidBlur => "shadow blur must be finite and nonnegative",
            Self::InvalidSpread => "shadow spread must be finite (signed values are supported)",
            Self::InvalidColor => "shadow Hsla channels must be finite in 0..=1",
        })
    }
}
impl Error for ShadowStyleError {}

#[derive(Clone, Copy, Debug, PartialEq)]
enum ShadowSource {
    Theme(ShadowElevation),
    Explicit(ShadowStyle),
}

/// A native shadow box around an arbitrary RustUI child.
/// Shadows do not reserve layout space, add padding, change hit testing or clip
/// descendants. Parent clipping still applies. IDs must be unique among sibling
/// shadows; keep them stable to preserve native child editing state.
pub struct ShadowView<V: View> {
    id: Identity,
    child: V,
    source: ShadowSource,
    radius: f32,
}

/// Uses the active theme's Medium elevation by default, including its disabled
/// shadow policy. Shadow has its own typed child namespace, so its ID may match
/// a nested Background or Border ID without colliding.
/// Panics if the ID is empty or contains only whitespace.
#[allow(non_snake_case)]
pub fn Shadow<V: View>(id: impl Into<Identity>, child: V) -> ShadowView<V> {
    let id = id.into();
    assert!(
        !id.as_str().trim().is_empty(),
        "Shadow identity must be nonempty"
    );
    ShadowView {
        id,
        child,
        source: ShadowSource::Theme(ShadowElevation::Medium),
        radius: 0.0,
    }
}

impl<V: View> ShadowView<V> {
    /// Selects current theme elevation. Last elevation/style call wins.
    pub fn elevation(mut self, elevation: ShadowElevation) -> Self {
        self.source = ShadowSource::Theme(elevation);
        self
    }

    /// Uses one explicit outer shadow instead of theme elevation layers.
    /// Explicit styles are independent of the theme's shadow enable flag.
    pub fn style(mut self, style: ShadowStyle) -> Self {
        self.source = ShadowSource::Explicit(style);
        self
    }

    /// Sets native shadow-box corner radius without clipping the child.
    /// Panics for negative or nonfinite logical-pixel values.
    pub fn radius(mut self, radius: f32) -> Self {
        assert!(
            radius.is_finite() && radius >= 0.0,
            "Shadow radius must be finite and nonnegative"
        );
        self.radius = radius;
        self
    }

    fn native_box(&self, shadows: Vec<BoxShadow>) -> Div {
        div()
            .flex()
            .flex_col()
            .rounded(px(self.radius))
            .shadow(shadows)
    }
}

fn theme_shadows(theme: &gpui_component::Theme, elevation: ShadowElevation) -> Vec<BoxShadow> {
    let tokens = theme.shadow_tokens();
    match elevation {
        ShadowElevation::Small => tokens.sm,
        ShadowElevation::Medium => tokens.md,
        ShadowElevation::Large => tokens.lg,
    }
}

impl<V: View> View for ShadowView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let shadows = match self.source {
            ShadowSource::Theme(elevation) => {
                theme_shadows(gpui_component::Theme::global(context.app()), elevation)
            }
            ShadowSource::Explicit(style) => vec![style.native()],
        };
        let native = self.native_box(shadows);
        context.with_element_scope(RetentionScope::Shadow(self.id.clone()), |context| {
            native
                .child(self.child.into_element(context))
                .into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_rejects_empty_and_whitespace_without_normalizing_valid_ids() {
        for id in ["", " \t\n"] {
            assert!(std::panic::catch_unwind(|| Shadow(id, EmptyView)).is_err());
        }
        assert_eq!(Shadow(" shadow ", EmptyView).id.as_str(), " shadow ");
    }

    fn color() -> Hsla {
        gpui::hsla(0.0, 0.0, 0.0, 0.3)
    }

    #[test]
    fn explicit_parameters_forward_zero_blur_and_signed_native_spread() {
        let zero = ShadowStyle::new(0.0, 0.0, 0.0, 0.0, color())
            .unwrap()
            .native();
        assert_eq!(zero.blur_radius, px(0.0));
        assert_eq!(zero.spread_radius, px(0.0));
        assert!(!zero.inset);
        let native = ShadowStyle::new(-3.0, 5.0, 8.0, -2.0, color())
            .unwrap()
            .native();
        assert_eq!(native.offset.x, px(-3.0));
        assert_eq!(native.offset.y, px(5.0));
        assert_eq!(native.blur_radius, px(8.0));
        assert_eq!(native.spread_radius, px(-2.0));
        assert_eq!(native.color, color());
    }

    #[test]
    fn geometry_rejects_nonfinite_and_negative_blur() {
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                ShadowStyle::new(value, 0.0, 0.0, 0.0, color()),
                Err(ShadowStyleError::InvalidOffset)
            );
            assert_eq!(
                ShadowStyle::new(0.0, value, 0.0, 0.0, color()),
                Err(ShadowStyleError::InvalidOffset)
            );
            assert_eq!(
                ShadowStyle::new(0.0, 0.0, value, 0.0, color()),
                Err(ShadowStyleError::InvalidBlur)
            );
            assert_eq!(
                ShadowStyle::new(0.0, 0.0, 0.0, value, color()),
                Err(ShadowStyleError::InvalidSpread)
            );
        }
        assert_eq!(
            ShadowStyle::new(0.0, 0.0, -1.0, 0.0, color()),
            Err(ShadowStyleError::InvalidBlur)
        );
    }

    #[test]
    fn all_color_channels_are_validated() {
        for invalid in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            for color in [
                Hsla {
                    h: invalid,
                    ..color()
                },
                Hsla {
                    s: invalid,
                    ..color()
                },
                Hsla {
                    l: invalid,
                    ..color()
                },
                Hsla {
                    a: invalid,
                    ..color()
                },
            ] {
                assert_eq!(
                    ShadowStyle::new(0.0, 0.0, 0.0, 0.0, color),
                    Err(ShadowStyleError::InvalidColor)
                );
            }
        }
    }

    #[test]
    fn source_selection_and_radius_preserve_unclipped_native_box() {
        let style = ShadowStyle::new(0.0, 4.0, 8.0, -2.0, color()).unwrap();
        let view = Shadow("shadow", EmptyView);
        assert_eq!(view.source, ShadowSource::Theme(ShadowElevation::Medium));
        let view = view.elevation(ShadowElevation::Large).style(style);
        assert_eq!(view.source, ShadowSource::Explicit(style));
        let view = view.elevation(ShadowElevation::Small).radius(8.0);
        assert_eq!(view.source, ShadowSource::Theme(ShadowElevation::Small));
        let mut native = view.native_box(vec![style.native()]);
        assert_eq!(native.style().box_shadow.as_ref().unwrap().len(), 1);
        assert_eq!(native.style().corner_radii.top_left, Some(px(8.0).into()));
        assert_eq!(
            native.style().overflow,
            gpui::StyleRefinement::default().overflow
        );
        assert_eq!(Shadow("shadow", EmptyView).radius(0.0).radius, 0.0);
        for radius in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(
                std::panic::catch_unwind(|| Shadow("shadow", EmptyView).radius(radius)).is_err()
            );
        }
    }

    #[test]
    fn semantic_elevations_use_upstream_tokens_and_disabled_policy() {
        let mut theme = gpui_component::Theme::default();
        theme.shadow = true;
        let tokens = theme.shadow_tokens();
        assert_eq!(theme_shadows(&theme, ShadowElevation::Small), tokens.sm);
        assert_eq!(theme_shadows(&theme, ShadowElevation::Medium), tokens.md);
        assert_eq!(theme_shadows(&theme, ShadowElevation::Large), tokens.lg);
        theme.shadow = false;
        for elevation in [
            ShadowElevation::Small,
            ShadowElevation::Medium,
            ShadowElevation::Large,
        ] {
            assert!(theme_shadows(&theme, elevation).is_empty());
        }
    }
}
