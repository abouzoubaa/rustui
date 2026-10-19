//! Flexible layout space.

use gpui::{AnyElement, IntoElement, Styled, div, px};

use crate::view::{EmptyView, RenderContext, View};

/// Explicit axis used when assigning a spacer's minimum length.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpacerAxis {
    Horizontal,
    Vertical,
}

/// A flexible view that takes available space along its parent's main axis.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SpacerView {
    min_length: Option<(SpacerAxis, f32)>,
}

impl SpacerView {
    /// Sets a minimum length along an explicitly selected axis.
    ///
    /// The axis should match the parent stack's main axis. The length must be
    /// finite and nonnegative.
    pub fn min_length(mut self, axis: SpacerAxis, length: f32) -> Self {
        assert!(
            length.is_finite() && length >= 0.0,
            "spacer minimum length must be finite and nonnegative"
        );
        self.min_length = Some((axis, length));
        self
    }
}

/// Creates a flexible spacer for the parent's main axis.
#[allow(non_snake_case)]
pub fn Spacer() -> SpacerView {
    SpacerView::default()
}

impl View for SpacerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        let mut spacer = div().flex_1();
        if let Some((axis, length)) = self.min_length {
            spacer = match axis {
                SpacerAxis::Horizontal => spacer.min_w(px(length)),
                SpacerAxis::Vertical => spacer.min_h(px(length)),
            };
        }
        spacer.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimum_length_accepts_finite_nonnegative_values() {
        assert_eq!(
            Spacer().min_length(SpacerAxis::Horizontal, 12.5).min_length,
            Some((SpacerAxis::Horizontal, 12.5))
        );
        assert_eq!(
            Spacer().min_length(SpacerAxis::Vertical, 0.0).min_length,
            Some((SpacerAxis::Vertical, 0.0))
        );
    }

    #[test]
    #[should_panic(expected = "spacer minimum length must be finite and nonnegative")]
    fn minimum_length_rejects_negative_values() {
        Spacer().min_length(SpacerAxis::Horizontal, -2.0);
    }

    #[test]
    #[should_panic(expected = "spacer minimum length must be finite and nonnegative")]
    fn minimum_length_rejects_non_finite_values() {
        Spacer().min_length(SpacerAxis::Horizontal, f32::INFINITY);
    }

    #[test]
    #[should_panic(expected = "spacer minimum length must be finite and nonnegative")]
    fn minimum_length_rejects_nan() {
        Spacer().min_length(SpacerAxis::Horizontal, f32::NAN);
    }

    #[test]
    fn spacer_defaults_to_flexible_main_axis_without_an_axis_minimum() {
        assert_eq!(Spacer(), SpacerView::default());
        assert_eq!(
            Spacer().min_length(SpacerAxis::Horizontal, 8.0).min_length,
            Some((SpacerAxis::Horizontal, 8.0))
        );
    }
}
