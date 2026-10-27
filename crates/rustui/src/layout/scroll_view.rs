//! Bounded viewports using GPUI scrolling and optional upstream scrollbars.

use gpui::{
    AnyElement, Div, InteractiveElement, IntoElement, ParentElement, Pixels, Point, ScrollHandle,
    Stateful, StatefulInteractiveElement, Styled, div, px,
};
use gpui_component::{
    ActiveTheme as _, ElementExt as _, InteractiveElementExt as _,
    scroll::{Scrollbar, ScrollbarAxis},
};
use rustui_core::Identity;
use rustui_i18n::LayoutDirection;
use std::{cell::Cell, rc::Rc};

use crate::components::theme::{required_style, selector};
use crate::view::{EmptyView, RenderContext, RetentionScope, View};

/// Physical axes enabled for a scroll viewport.
/// Content still inherits its normal locale and layout direction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollAxis {
    #[default]
    Vertical,
    Horizontal,
    /// Both axes are enabled; native GPUI selects the dominant axis per delta.
    Both,
}

/// A viewport whose content may extend beyond its visible bounds.
/// Keep its identity stable to retain native scroll position and child state.
/// Scrolled dimensions need bounded parent space or explicit dimensions.
/// This adapter does not virtualize content or expose a second scrolling runtime.
pub struct ScrollViewView<V> {
    content: V,
    id: Identity,
    width: Option<f32>,
    height: Option<f32>,
    axis: ScrollAxis,
    scrollbars: bool,
}

/// Create a vertical viewport that fills available space by default.
#[allow(non_snake_case)]
pub fn ScrollView<V: View>(content: V) -> ScrollViewView<V> {
    ScrollViewView {
        content,
        id: Identity::from("scroll-view"),
        width: None,
        height: None,
        axis: ScrollAxis::Vertical,
        scrollbars: false,
    }
}

impl<V: View> ScrollViewView<V> {
    /// Give sibling or dynamic scroll views unique, stable identities.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = id.into();
        self
    }

    /// Select physical scrolling axes. Coordinates on disabled axes are cleared.
    /// Enabled coordinates remain subject to native clamping after layout.
    pub fn axis(mut self, axis: ScrollAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Show upstream overlay scrollbars using the current native theme policy.
    /// Disabled by default. Changing this preserves the tracked scroll handle.
    pub fn scrollbars(mut self, visible: bool) -> Self {
        self.scrollbars = visible;
        self
    }

    /// Request an explicit finite positive width in logical pixels.
    /// Panics for nonfinite or nonpositive values. Parent flex rules still apply.
    pub fn width(mut self, width: f32) -> Self {
        validate_dimension(width, "width");
        self.width = Some(width);
        self
    }

    /// Set an explicit finite, positive height in GPUI points.
    /// Panics when `height` is non-finite or not positive.
    pub fn height(mut self, height: f32) -> Self {
        validate_dimension(height, "height");
        self.height = Some(height);
        self
    }
}

struct ScrollState {
    handle: ScrollHandle,
    axis: ScrollAxis,
    direction: LayoutDirection,
    anchor_rtl: Rc<Cell<bool>>,
    last_horizontal_extent: Rc<Cell<Pixels>>,
}

impl ScrollState {
    fn handle_for_layout(
        &mut self,
        axis: ScrollAxis,
        direction: LayoutDirection,
    ) -> (ScrollHandle, Rc<Cell<bool>>, Rc<Cell<Pixels>>) {
        if self.direction != direction {
            self.direction = direction;
            self.anchor_rtl
                .set(direction == LayoutDirection::RightToLeft);
            if direction == LayoutDirection::LeftToRight {
                let mut offset = self.handle.offset();
                offset.x = px(0.0);
                self.handle.set_offset(offset);
            }
        }
        if self.axis != axis {
            let previous = self.handle.offset();
            let next = enabled_offset(previous, axis);
            if next != previous {
                self.handle.set_offset(next);
            }
            if self.axis == ScrollAxis::Vertical && direction == LayoutDirection::RightToLeft {
                self.anchor_rtl.set(true);
            }
            self.axis = axis;
        }
        (
            self.handle.clone(),
            self.anchor_rtl.clone(),
            self.last_horizontal_extent.clone(),
        )
    }
}

fn rtl_leading_offset(
    offset: Pixels,
    previous_extent: Pixels,
    current_extent: Pixels,
    pending: bool,
) -> Option<Pixels> {
    (current_extent > px(0.0)
        && (pending || (current_extent != previous_extent && offset == -previous_extent)))
        .then_some(-current_extent)
}

fn enabled_offset(mut offset: Point<Pixels>, axis: ScrollAxis) -> Point<Pixels> {
    match axis {
        ScrollAxis::Vertical => offset.x = px(0.0),
        ScrollAxis::Horizontal => offset.y = px(0.0),
        ScrollAxis::Both => {}
    }
    offset
}

fn native_axis(axis: ScrollAxis) -> ScrollbarAxis {
    match axis {
        ScrollAxis::Vertical => ScrollbarAxis::Vertical,
        ScrollAxis::Horizontal => ScrollbarAxis::Horizontal,
        ScrollAxis::Both => ScrollbarAxis::Both,
    }
}

fn native_viewport(axis: ScrollAxis, handle: &ScrollHandle) -> Stateful<Div> {
    let viewport = div()
        .id("viewport")
        .flex()
        .size_full()
        .min_w_0()
        .min_h_0()
        .track_scroll(handle);
    match axis {
        ScrollAxis::Vertical => viewport.flex_col().overflow_y_scroll().lock_scroll_axis(),
        ScrollAxis::Horizontal => viewport.flex_row().overflow_x_scroll().lock_scroll_axis(),
        // Cross-axis stretch would constrain the auto-height content wrapper
        // to the viewport height, hiding descendant vertical overflow from the
        // native handle's immediate-child extent calculation.
        // Preserve GPUI's dominant-axis wheel policy; only layout changes here.
        ScrollAxis::Both => viewport.flex_row().items_start().overflow_scroll(),
    }
}

fn measurement(
    measurements: Option<&serde_json::Map<String, serde_json::Value>>,
    name: &str,
    fallback: f32,
    strictly_positive: bool,
) -> f32 {
    measurements
        .and_then(|measurements| measurements.get(name))
        .and_then(serde_json::Value::as_f64)
        .map(|value| value as f32)
        .filter(|value| {
            value.is_finite()
                && if strictly_positive {
                    *value > 0.0
                } else {
                    *value >= 0.0
                }
        })
        .unwrap_or(fallback)
}

fn themed_scrollbar(
    handle: &ScrollHandle,
    context: &RenderContext<'_>,
    axis: ScrollAxis,
) -> Scrollbar {
    let mut query = selector(context);
    let (part, orientation, horizontal) = match axis {
        ScrollAxis::Vertical => ("scrollbar_vertical", "vertical", false),
        ScrollAxis::Horizontal => ("scrollbar_horizontal", "horizontal", true),
        // GPUI applies one scrollbar style to both axes. macOS 27 uses the same
        // thickness, radius, track colors, and 8x minimum length for both.
        ScrollAxis::Both => ("scrollbar_vertical", "vertical", false),
    };
    query.orientation = orientation.into();
    let style = required_style(context, "scroll_view", part, &query);
    let measurements = style
        .record_metadata
        .get("svg_measurements")
        .and_then(serde_json::Value::as_object);
    let thickness = measurement(measurements, "thumb_thickness_px", 6.0, true);
    let radius = style
        .lengths
        .get("corner_radius")
        .copied()
        .unwrap_or_else(|| measurement(measurements, "thumb_corner_radius_px", 3.0, false));
    let opacity = measurement(measurements, "thumb_fill_opacity", 0.5, false).clamp(0.0, 1.0);
    let min_length = measurement(
        measurements,
        if horizontal {
            "horizontal_thumb_length_content_8x_px"
        } else {
            "vertical_thumb_length_content_8x_px"
        },
        8.0,
        true,
    );
    let background = style
        .colors
        .get("background")
        .copied()
        .map(crate::theme::registry::hsla);
    let border = style
        .colors
        .get("border")
        .copied()
        .map(crate::theme::registry::hsla);
    let thumb_fill = context.app().theme().muted_foreground.opacity(opacity);

    Scrollbar::new(handle)
        .axis(native_axis(axis))
        .viewport_from_layout()
        .styles(move |styles| {
            let styles = styles.track(|track| {
                let track = track.width(px(thickness));
                let track = if let Some(background) = background {
                    track.bg(background)
                } else {
                    track
                };
                if let Some(border) = border {
                    track.border_color(border)
                } else {
                    track
                }
            });
            styles.thumb(|thumb| {
                thumb
                    .bg(thumb_fill)
                    .width(px(thickness))
                    .min_length(px(min_length))
                    .radius(px(radius))
            })
        })
}

fn native_content(axis: ScrollAxis) -> Stateful<Div> {
    let content = div().id("content").flex().flex_col().flex_none();
    match axis {
        ScrollAxis::Vertical => content.w_full(),
        ScrollAxis::Horizontal => content.w_auto().min_w_full().h_full(),
        ScrollAxis::Both => content.size_auto().min_size_full(),
    }
}

impl<V: View> View for ScrollViewView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::ScrollView(self.id.clone()), |context| {
            let direction = context.layout_direction();
            let (state, _) =
                context.retained_entity(Identity::from("scroll-state"), |_, _| ScrollState {
                    handle: ScrollHandle::default(),
                    axis: self.axis,
                    direction,
                    anchor_rtl: Rc::new(Cell::new(direction == LayoutDirection::RightToLeft)),
                    last_horizontal_extent: Rc::new(Cell::new(px(0.0))),
                });
            let (handle, anchor_rtl, last_horizontal_extent) = context.update_entity(&state, |state, _, _| {
                state.handle_for_layout(self.axis, direction)
            });
            let content = context
                .with_element_scope(RetentionScope::ScrollViewContent, |context| {
                    self.content.into_element(context)
                });
            let diagnostics = std::env::var_os("RUSTUI_SCROLL_DIAGNOSTICS")
                .as_deref() == Some(std::ffi::OsStr::new("1"));
            let mut viewport =
                native_viewport(self.axis, &handle).child(native_content(self.axis).child(content));
            if diagnostics {
                let id = self.id.clone();
                let axis = self.axis;
                let observed = handle.clone();
                viewport = viewport.on_scroll_wheel(move |event, _, _| {
                    eprintln!("RUSTUI_SCROLL_DIAGNOSTICS phase=wheel id={} axis={axis:?} position={:?} delta={:?}",
                        id.as_str(), event.position, event.delta);
                    log_scroll_snapshot("wheel-handler", &id, axis, &observed);
                });
            }
            // Keep the same outer/viewport/content structure when bars are toggled.
            // The upstream bar is a sibling overlay, not scrolled content.
            let mut overlay = div().id("scrollbar-overlay").absolute().inset_0();
            if self.scrollbars {
                overlay = overlay.child(themed_scrollbar(&handle, context, self.axis).id("scrollbar"));
            }
            if diagnostics {
                let id = self.id.clone();
                let axis = self.axis;
                let bars = self.scrollbars;
                overlay = overlay.on_prepaint(move |bounds, _, _| {
                    eprintln!("RUSTUI_SCROLL_DIAGNOSTICS phase=scrollbar-overlay-prepaint id={} axis={axis:?} bars={bars} overlay_bounds={bounds:?}", id.as_str());
                });
            }
            let mut root = div()
                .id("root")
                .relative()
                .flex()
                .flex_col()
                .size_full()
                .min_w_0()
                .min_h_0();
            if let Some(width) = self.width {
                root = root.w(px(width));
            }
            if let Some(height) = self.height {
                root = root.flex_grow_0().h(px(height));
            }
            let root = root.child(viewport).child(overlay);
            let root = if direction == LayoutDirection::RightToLeft
                && self.axis != ScrollAxis::Vertical
            {
                let scroll_handle = handle.clone();
                root.on_prepaint(move |_, window, cx| {
                let extent = scroll_handle.max_offset().x;
                let offset = scroll_handle.offset().x;
                let next = rtl_leading_offset(
                    offset,
                    last_horizontal_extent.get(),
                    extent,
                    anchor_rtl.get(),
                );
                if extent > px(0.0) {
                    last_horizontal_extent.set(extent);
                }
                if let Some(next) = next {
                        let mut offset = scroll_handle.offset();
                    offset.x = next;
                        scroll_handle.set_offset(offset);
                        anchor_rtl.set(false);
                        let window_handle = window.window_handle();
                        cx.defer(move |cx| {
                            let _ = window_handle.update(cx, |_, window, _| window.refresh());
                        });
                    }
                })
            } else {
                root
            };
            if diagnostics {
                let id = self.id.clone();
                let axis = self.axis;
                root.on_prepaint(move |bounds, _, _| {
                    eprintln!("RUSTUI_SCROLL_DIAGNOSTICS phase=root-prepaint id={} root_bounds={bounds:?}", id.as_str());
                    log_scroll_snapshot("post-children-prepaint", &id, axis, &handle);
                }).into_any_element()
            } else {
                root.into_any_element()
            }
        })
    }
}

fn log_scroll_snapshot(phase: &str, id: &Identity, axis: ScrollAxis, handle: &ScrollHandle) {
    let bounds = handle.bounds();
    let max = handle.max_offset();
    eprintln!(
        "RUSTUI_SCROLL_DIAGNOSTICS phase={phase} id={} axis={axis:?} offset={:?} max_offset={max:?} viewport_bounds={bounds:?} content_size_derived=({:?}, {:?}) first_child_bounds={:?}",
        id.as_str(),
        handle.offset(),
        bounds.size.width + max.x,
        bounds.size.height + max.y,
        handle.bounds_for_item(0),
    );
}

fn validate_dimension(value: f32, name: &str) {
    assert!(
        value.is_finite() && value > 0.0,
        "ScrollView {name} must be finite and positive"
    );
}

#[cfg(test)]
mod rendered_overflow_tests {
    use super::*;
    use crate::{
        layout::{HorizontalAlignment, VStack},
        localization::activate_localization,
        runtime::ComponentHost,
    };
    use rustui_i18n::{Catalogs, Locale};
    use std::{cell::Cell, rc::Rc};

    struct WideLeaf(Rc<Cell<Option<gpui::Bounds<Pixels>>>>);

    impl View for WideLeaf {
        fn body(&self) -> impl View {
            EmptyView
        }
        fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
            let observed = self.0.clone();
            div()
                .w(px(948.0))
                .h(px(32.0))
                .flex_none()
                .on_prepaint(move |bounds, _, _| observed.set(Some(bounds)))
                .into_any_element()
        }
    }

    struct PreviewProbe {
        axis: ScrollAxis,
        observed: Rc<Cell<Option<gpui::Bounds<Pixels>>>>,
    }

    impl View for PreviewProbe {
        fn body(&self) -> impl View {
            EmptyView
        }
        fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
            ScrollView(
                VStack::new()
                    .alignment(HorizontalAlignment::Start)
                    .child(WideLeaf(self.observed.clone())),
            )
            .id("wide-preview")
            .width(240.0)
            .height(160.0)
            .axis(self.axis)
            .into_element(context)
        }
    }

    #[gpui::test]
    fn both_axes_preserve_wide_content_in_ltr_and_rtl(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        for (axis, locale) in [
            (ScrollAxis::Vertical, "en"),
            (ScrollAxis::Vertical, "ar"),
            (ScrollAxis::Both, "en"),
            (ScrollAxis::Both, "ar"),
        ] {
            cx.update(|app| {
                activate_localization(
                    Catalogs::new(Locale::parse("en").unwrap()),
                    Locale::parse(locale).unwrap(),
                    app,
                )
            });
            let observed = Rc::new(Cell::new(None));
            let sample = observed.clone();
            let (owner, window) = cx.add_window_view(|_, app| {
                ComponentHost::new(
                    PreviewProbe {
                        axis,
                        observed: sample,
                    },
                    app,
                )
            });
            window.update(|window, app| {
                let _ = window.draw(app);
                let native = owner.read(app).retained_entity_for_test::<ScrollState>(
                    vec![RetentionScope::ScrollView(Identity::from("wide-preview"))],
                    Identity::from("scroll-state"),
                ).expect("rendered ScrollView state");
                let handle = native.read(app).handle.clone();
                let leaf = observed.get().expect("prepaint must measure the wide leaf");
                let viewport = handle.bounds();
                let relative_x = (leaf.origin.x - viewport.origin.x).as_f32();
                eprintln!("wide-preview axis={axis:?} locale={locale} leaf={leaf:?} viewport={viewport:?} max={:?}", handle.max_offset());
                assert!((leaf.size.width.as_f32() - 948.0).abs() < 0.5);
                assert!((viewport.size.width.as_f32() - 240.0).abs() < 0.5);
                if axis == ScrollAxis::Both {
                    assert!(handle.max_offset().x.as_f32() >= 708.0 - 0.5,
                        "declared width must be reachable by horizontal scrolling");
                    if locale == "ar" {
                        assert!((relative_x + 708.0).abs() < 0.5,
                            "RTL content must initially show its logical leading edge");
                        assert!((handle.offset().x.as_f32() + 708.0).abs() < 0.5);
                    } else {
                        assert!(relative_x.abs() < 0.5);
                        assert!(handle.offset().x.as_f32().abs() < 0.5);
                    }
                } else if locale == "ar" {
                    assert!(relative_x < -600.0, "control must reproduce clipped RTL content");
                } else {
                    assert!(relative_x.abs() < 0.5);
                }
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rtl_leading_edge_tracks_new_content_width_without_overriding_manual_scroll() {
        assert_eq!(
            rtl_leading_offset(px(0.0), px(0.0), px(600.0), true),
            Some(px(-600.0))
        );
        assert_eq!(
            rtl_leading_offset(px(-600.0), px(600.0), px(1_200.0), false),
            Some(px(-1_200.0))
        );
        assert_eq!(
            rtl_leading_offset(px(-400.0), px(600.0), px(1_200.0), false),
            None
        );
        assert_eq!(
            rtl_leading_offset(px(-600.0), px(600.0), px(600.0), false),
            None
        );
    }

    #[test]
    fn scrollbar_dimensions_fall_back_from_zero_but_opacity_and_radius_allow_zero() {
        let measurements = serde_json::json!({
            "thumb_thickness_px": 0.0,
            "vertical_thumb_length_content_8x_px": 0.0,
            "thumb_corner_radius_px": 0.0,
            "thumb_fill_opacity": 0.0,
        });
        let measurements = measurements.as_object();

        assert_eq!(
            measurement(measurements, "thumb_thickness_px", 6.0, true),
            6.0
        );
        assert_eq!(
            measurement(
                measurements,
                "vertical_thumb_length_content_8x_px",
                8.0,
                true
            ),
            8.0
        );
        assert_eq!(
            measurement(measurements, "thumb_corner_radius_px", 3.0, false),
            0.0
        );
        assert_eq!(
            measurement(measurements, "thumb_fill_opacity", 0.5, false),
            0.0
        );
    }

    #[test]
    fn default_contract_and_dimensions_remain_bounded_vertical() {
        let view = ScrollView(EmptyView);
        assert_eq!(view.axis, ScrollAxis::Vertical);
        assert!(!view.scrollbars);
        assert_eq!((view.width, view.height), (None, None));
        for value in [0.0, -1.0, f32::NAN, f32::INFINITY] {
            assert!(std::panic::catch_unwind(|| ScrollView(EmptyView).width(value)).is_err());
            assert!(std::panic::catch_unwind(|| ScrollView(EmptyView).height(value)).is_err());
        }
        let view = ScrollView(EmptyView).width(320.0).height(180.0);
        assert_eq!((view.width, view.height), (Some(320.0), Some(180.0)));
    }

    #[test]
    fn native_overflow_and_wheel_policy_follow_axis() {
        for (axis, x, y, locked) in [
            (
                ScrollAxis::Vertical,
                None,
                Some(gpui::Overflow::Scroll),
                true,
            ),
            (
                ScrollAxis::Horizontal,
                Some(gpui::Overflow::Scroll),
                None,
                true,
            ),
            (
                ScrollAxis::Both,
                Some(gpui::Overflow::Scroll),
                Some(gpui::Overflow::Scroll),
                false,
            ),
        ] {
            let mut viewport = native_viewport(axis, &ScrollHandle::default());
            let style = viewport.style();
            assert_eq!(style.overflow.x, x);
            assert_eq!(style.overflow.y, y);
            #[cfg(not(target_family = "wasm"))]
            assert_eq!(style.restrict_scroll_to_axis, locked.then_some(true));
            #[cfg(target_family = "wasm")]
            let _ = locked;
            assert_eq!(style.allow_concurrent_scroll, None);
        }
    }

    #[test]
    fn content_is_nonshrinking_and_cross_axis_bounded() {
        let mut vertical = native_content(ScrollAxis::Vertical);
        assert_eq!(
            vertical.style().size.width,
            Some(gpui::relative(1.0).into())
        );
        assert_eq!(vertical.style().flex_shrink, Some(0.0));
        let mut horizontal = native_content(ScrollAxis::Horizontal);
        assert_eq!(
            horizontal.style().size.height,
            Some(gpui::relative(1.0).into())
        );
        assert_eq!(
            horizontal.style().min_size.width,
            Some(gpui::relative(1.0).into())
        );
        let mut both = native_content(ScrollAxis::Both);
        assert_eq!(
            both.style().min_size.width,
            Some(gpui::relative(1.0).into())
        );
        assert_eq!(
            both.style().min_size.height,
            Some(gpui::relative(1.0).into())
        );
    }

    #[test]
    fn both_axis_keeps_intrinsic_content_height_instead_of_stretching_it() {
        let mut viewport = native_viewport(ScrollAxis::Both, &ScrollHandle::default());
        assert_eq!(
            viewport.style().flex_direction,
            Some(gpui::FlexDirection::Row)
        );
        assert_eq!(
            viewport.style().align_items,
            Some(gpui::AlignItems::FlexStart)
        );
        let mut content = native_content(ScrollAxis::Both);
        assert_eq!(content.style().size.height, Some(gpui::Length::Auto));
        assert_eq!(
            content.style().min_size.height,
            Some(gpui::relative(1.0).into())
        );
        assert_eq!(content.style().flex_shrink, Some(0.0));
    }

    #[test]
    fn native_handle_survives_rerender_and_only_disabled_coordinates_clear() {
        let mut state = ScrollState {
            handle: ScrollHandle::default(),
            axis: ScrollAxis::Both,
            direction: LayoutDirection::LeftToRight,
            anchor_rtl: Rc::new(Cell::new(false)),
            last_horizontal_extent: Rc::new(Cell::new(px(0.0))),
        };
        state.handle.set_offset(Point {
            x: px(-23.0),
            y: px(-47.0),
        });
        let copy = state
            .handle_for_layout(ScrollAxis::Both, LayoutDirection::LeftToRight)
            .0;
        assert_eq!(
            copy.offset(),
            Point {
                x: px(-23.0),
                y: px(-47.0)
            }
        );
        let vertical = state
            .handle_for_layout(ScrollAxis::Vertical, LayoutDirection::LeftToRight)
            .0;
        assert_eq!(
            vertical.offset(),
            Point {
                x: px(0.0),
                y: px(-47.0)
            }
        );
        vertical.set_offset(Point {
            x: px(0.0),
            y: px(-80.0),
        });
        assert_eq!(
            state
                .handle_for_layout(ScrollAxis::Vertical, LayoutDirection::LeftToRight)
                .0
                .offset()
                .y,
            px(-80.0)
        );
        assert_eq!(
            state
                .handle_for_layout(ScrollAxis::Both, LayoutDirection::LeftToRight)
                .0
                .offset()
                .y,
            px(-80.0)
        );
        assert_eq!(
            state
                .handle_for_layout(ScrollAxis::Horizontal, LayoutDirection::LeftToRight)
                .0
                .offset(),
            Point::default()
        );
    }
}
