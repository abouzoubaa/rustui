//! Responsive content generated from GPUI's resolved container dimensions.

use std::rc::Rc;

use gpui::{
    AnyElement, Div, IntoElement as _, ParentElement as _, Pixels, Size, Styled as _,
    container_query, div, px,
};
use rustui_core::Identity;

use crate::{
    runtime::component_host::RetainedEntities,
    view::{EmptyView, RenderContext, RenderSeed, RetentionScope, View},
};

/// Resolved reader dimensions in logical pixels, supplied by native layout.
/// These describe the container, not the intrinsic size of its generated child.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GeometrySize {
    pub width: f32,
    pub height: f32,
}

/// Generates a child after GPUI has assigned this container its size.
///
/// The default fills available width and height. Its contents cannot establish
/// its intrinsic size: provide a bounded parent or explicit dimensions as
/// appropriate for the surrounding layout. Zero dimensions are valid.
///
/// The factory may run whenever native layout/prepaint runs, even when size is
/// unchanged. Keep it free of owner mutations and other observable effects.
/// Use stable child identities to preserve state across size changes.
/// IDs must be unique among sibling readers in the current retained scope.
/// Size, environment, theme and locale are snapshots of the current render.
/// This is a size-only API, not a coordinate-space or geometry observer API.
pub struct GeometryReader<V: View> {
    id: Identity,
    factory: Rc<dyn Fn(GeometrySize) -> V>,
    width: Option<f32>,
    height: Option<f32>,
}

struct ReaderState {
    seed: RenderSeed,
    retained: RetainedEntities,
    generation: u64,
}

impl<V: View> GeometryReader<V> {
    pub fn new(id: impl Into<Identity>, factory: impl Fn(GeometrySize) -> V + 'static) -> Self {
        Self {
            id: id.into(),
            factory: Rc::new(factory),
            width: None,
            height: None,
        }
    }

    /// Sets a finite nonnegative width in logical pixels.
    /// Panics for negative or nonfinite values, like other layout builders.
    pub fn width(mut self, width: f32) -> Self {
        validate_dimension(width, "width");
        self.width = Some(width);
        self
    }

    /// Sets a finite nonnegative height in logical pixels.
    /// Panics for negative or nonfinite values, like other layout builders.
    pub fn height(mut self, height: f32) -> Self {
        validate_dimension(height, "height");
        self.height = Some(height);
        self
    }
}

impl<V: View> View for GeometryReader<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let seed = context.seed();
        let factory = self.factory.clone();
        context.with_element_scope(RetentionScope::GeometryReader(self.id.clone()), |context| {
            let (reader, _) =
                context.retained_entity(Identity::from("reader-state"), |_, _| ReaderState {
                    seed: seed.clone(),
                    retained: RetainedEntities::default(),
                    generation: 0,
                });
            let generation = context.update_entity(&reader, |state, _, _| {
                state.seed = seed;
                state.generation = state.generation.wrapping_add(1);
                state.generation
            });
            let weak = reader.downgrade();
            let mut query = container_query(move |size, window, cx| {
                weak.update(cx, |state, cx| {
                    if !generation_is_current(state.generation, generation) {
                        return div().into_any_element();
                    }
                    let seed = state.seed.clone();
                    state.retained.begin_frame();
                    let mut context =
                        seed.geometry_context(cx.entity_id(), window, cx, &mut state.retained);
                    let child = factory(GeometrySize {
                        width: f32::from(size.width),
                        height: f32::from(size.height),
                    })
                    .into_element(&mut context);
                    drop(context);
                    state.retained.finish_frame();
                    native_content_holder(size).child(child).into_any_element()
                })
                .unwrap_or_else(|_| div().into_any_element())
            })
            .min_w_0()
            .min_h_0();
            if let Some(width) = self.width {
                query = query.w(px(width));
            }
            if let Some(height) = self.height {
                query = query.h(px(height));
            }
            if self.width.is_some() || self.height.is_some() {
                query = query.flex_shrink_0();
            }
            query.into_any_element()
        })
    }
}

fn native_content_holder(size: Size<Pixels>) -> Div {
    // Definite available space does not assign an auto-width layout root its
    // width. Establish the containing block before lowering percentage-sized
    // descendants; native stretch still respects a child's explicit width.
    div()
        .flex()
        .flex_col()
        .items_stretch()
        .w(size.width)
        .h(size.height)
        .min_w_0()
        .min_h_0()
}

fn validate_dimension(value: f32, name: &str) {
    assert!(
        value.is_finite() && value >= 0.0,
        "GeometryReader {name} must be finite and nonnegative"
    );
}

fn generation_is_current(current: u64, callback: u64) -> bool {
    current == callback
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_content_has_a_definite_flex_containing_block() {
        for (width, height) in [(540.0, 240.0), (280.0, 240.0), (0.0, 0.0)] {
            let mut holder = native_content_holder(Size {
                width: px(width),
                height: px(height),
            });
            let style = holder.style();
            assert_eq!(style.size.width, Some(px(width).into()));
            assert_eq!(style.size.height, Some(px(height).into()));
            assert_eq!(style.display, Some(gpui::Display::Flex));
            assert_eq!(style.flex_direction, Some(gpui::FlexDirection::Column));
            assert_eq!(style.align_items, Some(gpui::AlignItems::Stretch));
            assert_eq!(style.overflow, gpui::StyleRefinement::default().overflow);
        }
    }

    #[test]
    fn dimensions_allow_zero_and_reject_invalid_values() {
        let reader = GeometryReader::new("reader", |_| EmptyView)
            .width(0.0)
            .height(0.0);
        assert_eq!((reader.width, reader.height), (Some(0.0), Some(0.0)));
        for value in [-1.0, f32::NAN, f32::INFINITY] {
            assert!(
                std::panic::catch_unwind(
                    || GeometryReader::new("reader", |_| EmptyView).width(value)
                )
                .is_err()
            );
            assert!(
                std::panic::catch_unwind(
                    || GeometryReader::new("reader", |_| EmptyView).height(value)
                )
                .is_err()
            );
        }
    }

    #[test]
    fn defaults_are_parent_sized_and_factory_observes_dimensions() {
        let observed = Rc::new(std::cell::Cell::new(None));
        let captured = observed.clone();
        let reader = GeometryReader::new("reader", move |size| {
            captured.set(Some(size));
            EmptyView
        });
        assert_eq!((reader.width, reader.height), (None, None));
        let size = GeometrySize {
            width: 320.0,
            height: 180.0,
        };
        (reader.factory)(size);
        assert_eq!(observed.get(), Some(size));
    }

    #[test]
    fn stale_generations_are_rejected() {
        assert!(generation_is_current(4, 4));
        assert!(!generation_is_current(5, 4));
    }
}
