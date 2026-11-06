//! Retargetable translation using two GPUI-owned scalar springs.

use crate::view::{EmptyView, RenderContext, RetentionScope, View};
use gpui::{
    AnimationExt as _, AnyElement, IntoElement as _, ParentElement as _, Styled as _, div, px,
};
use rustui_core::Identity;
use std::{error::Error, fmt};

/// Moves a child toward state-driven signed logical-pixel translation.
///
/// Keep `id` stable while changing targets so GPUI retains each axis spring's
/// position and velocity. Layout keeps the child's original slot. GPUI's
/// translated native bounds govern hit testing and accessibility geometry;
/// clipping and overlap remain subject to the surrounding native layout.
pub struct SpringOffset<V: View> {
    id: Identity,
    child: V,
    x: f32,
    y: f32,
}

impl<V: View> SpringOffset<V> {
    /// Creates a spring translation target. The ID must be nonblank and
    /// coordinates must be finite.
    pub fn new(
        id: impl Into<Identity>,
        child: V,
        x: f32,
        y: f32,
    ) -> Result<Self, SpringOffsetError> {
        let id = id.into();
        if id.as_str().trim().is_empty() {
            return Err(SpringOffsetError::EmptyIdentity);
        }
        validate_offset("x", x)?;
        validate_offset("y", y)?;
        Ok(Self { id, child, x, y })
    }

    /// Changes the horizontal target while preserving the spring identity.
    pub fn x(mut self, x: f32) -> Result<Self, SpringOffsetError> {
        validate_offset("x", x)?;
        self.x = x;
        Ok(self)
    }

    /// Changes the vertical target while preserving the spring identity.
    pub fn y(mut self, y: f32) -> Result<Self, SpringOffsetError> {
        validate_offset("y", y)?;
        self.y = y;
        Ok(self)
    }

    /// Changes both targets while preserving the spring identity.
    pub fn translation(mut self, x: f32, y: f32) -> Result<Self, SpringOffsetError> {
        validate_offset("x", x)?;
        validate_offset("y", y)?;
        self.x = x;
        self.y = y;
        Ok(self)
    }
}

impl<V: View> View for SpringOffset<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let child = context
            .with_element_scope(RetentionScope::SpringOffset(self.id.clone()), |context| {
                self.child.into_element(context)
            });
        let id = self.id.as_str();
        div()
            .self_start()
            .relative()
            .with_spring(
                format!("rustui-spring-offset-{id}-x"),
                gpui::SpringAnimation::new(gpui::SpringConfig::new(170.0, 26.0, 1.0))
                    .to(px(self.x)),
                |element, value| element.left(value),
            )
            .child(
                div()
                    .self_start()
                    .relative()
                    .with_spring(
                        format!("rustui-spring-offset-{id}-y"),
                        gpui::SpringAnimation::new(gpui::SpringConfig::new(170.0, 26.0, 1.0))
                            .to(px(self.y)),
                        |element, value| element.top(value),
                    )
                    .child(child),
            )
            .into_any_element()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpringOffsetError {
    EmptyIdentity,
    NonFinite { axis: &'static str },
}

impl fmt::Display for SpringOffsetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentity => f.write_str("spring offset identity must be nonempty"),
            Self::NonFinite { axis } => write!(f, "spring offset `{axis}` must be finite"),
        }
    }
}
impl Error for SpringOffsetError {}

fn validate_offset(axis: &'static str, value: f32) -> Result<(), SpringOffsetError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(SpringOffsetError::NonFinite { axis })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::EmptyView;

    #[test]
    fn validates_each_axis_and_updates_targets() {
        let offset = SpringOffset::new("moving", EmptyView, -8.0, 4.0)
            .unwrap()
            .x(12.0)
            .unwrap()
            .translation(2.0, -6.0)
            .unwrap();
        assert_eq!((offset.x, offset.y), (2.0, -6.0));
        for value in [f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(SpringOffset::new("bad", EmptyView, value, 0.0).is_err());
            assert!(SpringOffset::new("bad", EmptyView, 0.0, value).is_err());
        }
    }

    #[test]
    fn rejects_blank_ids_and_preserves_nonblank_identity() {
        for id in ["", " \t\n"] {
            assert_eq!(
                SpringOffset::new(id, EmptyView, 0.0, 0.0).err(),
                Some(SpringOffsetError::EmptyIdentity)
            );
        }

        let offset = SpringOffset::new(" moving ", EmptyView, 0.0, 0.0).unwrap();
        assert_eq!(offset.id.as_str(), " moving ");
    }
}
