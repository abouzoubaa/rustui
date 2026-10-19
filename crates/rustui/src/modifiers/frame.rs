//! Native size constraints and logical child alignment.

use gpui::{AnyElement, Div, IntoElement as _, ParentElement as _, Styled as _, div, px};
use rustui_i18n::LayoutDirection;

use crate::{
    layout::{HorizontalAlignment, VerticalAlignment},
    view::{EmptyView, RenderContext, View},
};

/// A native layout box with optional constraints around one RustUI child.
pub struct FrameView<V: View> {
    child: V,
    width: Option<f32>,
    height: Option<f32>,
    min_width: Option<f32>,
    max_width: Option<f32>,
    min_height: Option<f32>,
    max_height: Option<f32>,
    horizontal: HorizontalAlignment,
    vertical: VerticalAlignment,
}

/// Creates a naturally sized frame with centered child alignment.
/// Parent layout can stretch unconstrained dimensions. No clipping is added.
#[allow(non_snake_case)]
pub fn Frame<V: View>(child: V) -> FrameView<V> {
    FrameView {
        child,
        width: None,
        height: None,
        min_width: None,
        max_width: None,
        min_height: None,
        max_height: None,
        horizontal: HorizontalAlignment::Center,
        vertical: VerticalAlignment::Center,
    }
}

impl<V: View> FrameView<V> {
    /// Requests a fixed width. Native min/max constraints clamp this request.
    pub fn width(mut self, width: f32) -> Self {
        check_size(width);
        self.width = Some(width);
        self
    }

    /// Requests a fixed height. Native min/max constraints clamp this request.
    pub fn height(mut self, height: f32) -> Self {
        check_size(height);
        self.height = Some(height);
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        check_size(min_width);
        check_range(Some(min_width), self.max_width);
        self.min_width = Some(min_width);
        self
    }

    /// Sets a finite width ceiling; this does not request filling the parent.
    pub fn max_width(mut self, max_width: f32) -> Self {
        check_size(max_width);
        check_range(self.min_width, Some(max_width));
        self.max_width = Some(max_width);
        self
    }

    pub fn min_height(mut self, min_height: f32) -> Self {
        check_size(min_height);
        check_range(Some(min_height), self.max_height);
        self.min_height = Some(min_height);
        self
    }

    /// Sets a finite height ceiling; this does not request filling the parent.
    pub fn max_height(mut self, max_height: f32) -> Self {
        check_size(max_height);
        check_range(self.min_height, Some(max_height));
        self.max_height = Some(max_height);
        self
    }

    /// Aligns the child inside the frame using logical horizontal edges.
    /// Stretch fills the corresponding resolved frame dimension. Baseline
    /// delegates to native baseline alignment; a single child has no peer baseline.
    pub fn alignment(
        mut self,
        horizontal: HorizontalAlignment,
        vertical: VerticalAlignment,
    ) -> Self {
        self.horizontal = horizontal;
        self.vertical = vertical;
        self
    }

    fn native_holder(&self) -> Div {
        let mut holder = div().flex_none();
        if self.horizontal == HorizontalAlignment::Stretch {
            holder = holder.w_full().min_w_0().flex_shrink_1();
        }
        if self.vertical == VerticalAlignment::Stretch {
            holder = holder.h_full().min_h_0();
        }
        holder
    }

    fn native_frame(&self, direction: LayoutDirection) -> Div {
        let mut frame = div().flex().flex_row().min_w_0().min_h_0();
        frame = match (self.horizontal, direction) {
            (HorizontalAlignment::Start, LayoutDirection::RightToLeft)
            | (HorizontalAlignment::End, LayoutDirection::LeftToRight) => frame.justify_end(),
            (HorizontalAlignment::Start, LayoutDirection::LeftToRight)
            | (HorizontalAlignment::End, LayoutDirection::RightToLeft) => frame.justify_start(),
            (HorizontalAlignment::Center | HorizontalAlignment::Stretch, _) => {
                frame.justify_center()
            }
        };
        frame = match self.vertical {
            VerticalAlignment::Start => frame.items_start(),
            VerticalAlignment::Center => frame.items_center(),
            VerticalAlignment::End => frame.items_end(),
            VerticalAlignment::Baseline => frame.items_baseline(),
            VerticalAlignment::Stretch => frame.items_stretch(),
        };
        if self.width.is_some() || self.height.is_some() {
            frame = frame.flex_shrink_0();
        }
        if let Some(width) = self.width {
            frame = frame.w(px(width));
        }
        if let Some(height) = self.height {
            frame = frame.h(px(height));
        }
        if let Some(min) = self.min_width {
            frame = frame.min_w(px(min));
        }
        if let Some(max) = self.max_width {
            frame = frame.max_w(px(max));
        }
        if let Some(min) = self.min_height {
            frame = frame.min_h(px(min));
        }
        if let Some(max) = self.max_height {
            frame = frame.max_h(px(max));
        }
        frame
    }
}

fn check_size(size: f32) {
    assert!(
        size.is_finite() && size >= 0.0,
        "Frame sizes must be finite and nonnegative"
    );
}

fn check_range(min: Option<f32>, max: Option<f32>) {
    if let (Some(min), Some(max)) = (min, max) {
        assert!(min <= max, "Frame minimum must not exceed maximum");
    }
}

impl<V: View> View for FrameView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let frame = self.native_frame(context.layout_direction());
        frame
            .child(self.native_holder().child(self.child.into_element(context)))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::{AlignItems, FlexDirection, JustifyContent};

    #[test]
    fn native_constraints_accept_zero_and_leave_fixed_requests_for_native_clamping() {
        let frame = Frame(EmptyView)
            .width(300.0)
            .height(0.0)
            .min_width(20.0)
            .max_width(100.0)
            .min_height(0.0)
            .max_height(0.0);
        let mut native = frame.native_frame(LayoutDirection::LeftToRight);
        let style = native.style();
        assert_eq!(style.size.width, Some(px(300.0).into()));
        assert_eq!(style.size.height, Some(px(0.0).into()));
        assert_eq!(style.min_size.width, Some(px(20.0).into()));
        assert_eq!(style.max_size.width, Some(px(100.0).into()));
        assert_eq!(style.min_size.height, Some(px(0.0).into()));
        assert_eq!(style.max_size.height, Some(px(0.0).into()));
        assert_eq!(style.flex_shrink, Some(0.0));
    }

    #[test]
    fn native_alignment_is_centered_by_default_and_follows_logical_direction() {
        let mut centered = Frame(EmptyView).native_frame(LayoutDirection::LeftToRight);
        assert_eq!(
            centered.style().justify_content,
            Some(JustifyContent::Center)
        );
        assert_eq!(centered.style().align_items, Some(AlignItems::Center));
        assert_eq!(centered.style().size.width, None);
        assert_eq!(centered.style().size.height, None);
        for (direction, expected) in [
            (LayoutDirection::LeftToRight, JustifyContent::Start),
            (LayoutDirection::RightToLeft, JustifyContent::End),
        ] {
            let mut native = Frame(EmptyView)
                .alignment(HorizontalAlignment::Start, VerticalAlignment::End)
                .native_frame(direction);
            assert_eq!(native.style().flex_direction, Some(FlexDirection::Row));
            assert_eq!(native.style().justify_content, Some(expected));
            assert_eq!(native.style().align_items, Some(AlignItems::FlexEnd));
        }
    }

    #[test]
    fn rejects_invalid_sizes_and_contradictory_ranges_in_both_orders() {
        for value in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(std::panic::catch_unwind(|| Frame(EmptyView).width(value)).is_err());
            assert!(std::panic::catch_unwind(|| Frame(EmptyView).max_height(value)).is_err());
        }
        assert!(
            std::panic::catch_unwind(|| Frame(EmptyView).min_width(20.0).max_width(10.0)).is_err()
        );
        assert!(
            std::panic::catch_unwind(|| Frame(EmptyView).max_width(10.0).min_width(20.0)).is_err()
        );
        assert!(
            std::panic::catch_unwind(|| Frame(EmptyView).min_height(20.0).max_height(10.0))
                .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| Frame(EmptyView).max_height(10.0).min_height(20.0))
                .is_err()
        );
    }

    #[test]
    fn stretch_and_baseline_delegate_to_native_styles() {
        let frame =
            Frame(EmptyView).alignment(HorizontalAlignment::Stretch, VerticalAlignment::Stretch);
        let mut native = frame.native_frame(LayoutDirection::LeftToRight);
        let mut holder = frame.native_holder();
        assert_eq!(native.style().align_items, Some(AlignItems::Stretch));
        assert_eq!(holder.style().size.width, Some(gpui::relative(1.0).into()));
        assert_eq!(holder.style().size.height, Some(gpui::relative(1.0).into()));
        let mut baseline = Frame(EmptyView)
            .alignment(HorizontalAlignment::Center, VerticalAlignment::Baseline)
            .native_frame(LayoutDirection::LeftToRight);
        assert_eq!(baseline.style().align_items, Some(AlignItems::Baseline));
    }
}
