//! One-shot opacity animation for a RustUI child view.

use std::{error::Error, fmt, num::NonZeroU16, time::Duration};

use gpui::{
    Animation, AnimationExt, AnyElement, IntoElement as _, ParentElement as _, Styled as _, div,
};
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, RetentionScope, View};

/// A child view that fades between two opacity values when it is mounted.
///
/// The `id` must be unique among sibling animated opacity modifiers in the
/// current GPUI composition scope. The endpoints and duration are fixed for
/// the lifetime of this modifier; give it a new ID to replay the animation.
pub struct AnimatedOpacity<V: View> {
    id: Identity,
    child: V,
    from: f32,
    to: f32,
    duration: Duration,
    max_fps: Option<NonZeroU16>,
}

impl<V: View> AnimatedOpacity<V> {
    /// Creates a mount-only opacity animation for `child`.
    ///
    /// Both opacity values must be finite and within `0..=1`, and `duration`
    /// must be nonzero. GPUI's app-level reduced-motion setting is honored.
    pub fn new(
        id: impl Into<Identity>,
        child: V,
        from: f32,
        to: f32,
        duration: Duration,
    ) -> Result<Self, OpacityAnimationError> {
        validate_opacity("from", from)?;
        validate_opacity("to", to)?;
        validate_duration(duration)?;
        Ok(Self {
            id: id.into(),
            child,
            from,
            to,
            duration,
            max_fps: None,
        })
    }

    /// Limits how frequently GPUI schedules this animation's own updates.
    ///
    /// By default, GPUI requests every animation frame. This setting does not
    /// limit unrelated parent renders or measure presented frame rate. GPUI
    /// continues to own scheduling and reduced-motion endpoint handling.
    /// Configure this before mounting; use a new identity to replay it.
    pub fn max_fps(mut self, max_fps: NonZeroU16) -> Self {
        self.max_fps = Some(max_fps);
        self
    }

    fn native_animation(&self) -> Animation {
        let animation = Animation::new(self.duration);
        match self.max_fps {
            Some(max_fps) => animation.with_max_fps(f32::from(max_fps.get())),
            None => animation,
        }
    }
}

impl<V: View> View for AnimatedOpacity<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let child = context.with_element_scope(
            RetentionScope::AnimatedOpacity(self.id.clone()),
            |context| self.child.into_element(context),
        );
        let from = self.from;
        let to = self.to;
        div()
            .self_start()
            .with_animation(
                format!("rustui-animated-opacity-{}", self.id.as_str()),
                self.native_animation(),
                move |element, progress| element.opacity(interpolate_opacity(from, to, progress)),
            )
            .child(child)
            .into_any_element()
    }
}

/// Invalid arguments passed to [`AnimatedOpacity::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpacityAnimationError {
    /// An opacity endpoint was non-finite or outside `0..=1`.
    InvalidOpacity { endpoint: &'static str },
    /// The animation duration was zero.
    ZeroDuration,
}

impl fmt::Display for OpacityAnimationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpacity { endpoint } => {
                write!(
                    formatter,
                    "opacity `{endpoint}` must be finite and in 0..=1"
                )
            }
            Self::ZeroDuration => formatter.write_str("opacity animation duration must be nonzero"),
        }
    }
}

impl Error for OpacityAnimationError {}

fn validate_opacity(endpoint: &'static str, value: f32) -> Result<(), OpacityAnimationError> {
    if !value.is_finite() || !(0.0..=1.0).contains(&value) {
        return Err(OpacityAnimationError::InvalidOpacity { endpoint });
    }
    Ok(())
}

fn validate_duration(duration: Duration) -> Result<(), OpacityAnimationError> {
    if duration.is_zero() {
        Err(OpacityAnimationError::ZeroDuration)
    } else {
        Ok(())
    }
}

fn interpolate_opacity(from: f32, to: f32, progress: f32) -> f32 {
    from + (to - from) * progress
}

#[cfg(test)]
mod tests {
    use std::{num::NonZeroU16, time::Duration};

    use super::{
        AnimatedOpacity, OpacityAnimationError, interpolate_opacity, validate_duration,
        validate_opacity,
    };
    use crate::view::EmptyView;

    #[test]
    fn defaults_to_native_unthrottled_one_shot() {
        let fade =
            AnimatedOpacity::new("fade", EmptyView, 0.0, 1.0, Duration::from_secs(1)).unwrap();
        let animation = fade.native_animation();
        assert_eq!(animation.max_fps, None);
        assert_eq!(animation.duration, Duration::from_secs(1));
        assert!(animation.oneshot);
    }

    #[test]
    fn forwards_frame_limits_and_replaces_previous_configuration() {
        let mut fade =
            AnimatedOpacity::new("fade", EmptyView, 0.0, 1.0, Duration::from_secs(1)).unwrap();
        for limit in [1, 30, u16::MAX] {
            fade = fade.max_fps(NonZeroU16::new(limit).unwrap());
            let animation = fade.native_animation();
            assert_eq!(animation.max_fps, Some(f32::from(limit)));
            assert_eq!(animation.duration, Duration::from_secs(1));
            assert!(animation.oneshot);
            assert!(Duration::from_secs_f32(1.0 / f32::from(limit)) > Duration::ZERO);
        }
        assert_eq!((fade.from, fade.to), (0.0, 1.0));
    }

    #[test]
    fn validates_opacity_endpoints() {
        assert!(validate_opacity("from", 0.0).is_ok());
        assert!(validate_opacity("to", 1.0).is_ok());
        for invalid in [-0.01, 1.01, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert_eq!(
                validate_opacity("from", invalid),
                Err(OpacityAnimationError::InvalidOpacity { endpoint: "from" })
            );
        }
    }

    #[test]
    fn interpolates_between_opacity_endpoints() {
        assert_eq!(interpolate_opacity(0.0, 1.0, 0.0), 0.0);
        assert_eq!(interpolate_opacity(0.0, 1.0, 0.5), 0.5);
        assert_eq!(interpolate_opacity(0.0, 1.0, 1.0), 1.0);
        assert_eq!(interpolate_opacity(1.0, 0.0, 0.25), 0.75);
    }

    #[test]
    fn rejects_zero_duration() {
        assert_eq!(
            validate_duration(Duration::ZERO),
            Err(OpacityAnimationError::ZeroDuration)
        );
        assert!(validate_duration(Duration::from_nanos(1)).is_ok());
    }
}
