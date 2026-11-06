//! One-shot opacity animation for a RustUI child view.

use std::{error::Error, fmt, num::NonZeroU16, time::Duration};

use gpui::{
    Animation, AnimationExt, AnyElement, App, Bounds, Element, ElementId, GlobalElementId,
    InspectorElementId, IntoElement, LayoutId, ParentElement as _, Pixels, Styled as _, Window,
    div,
};
use rustui_core::Identity;

use crate::{
    animation::timing::TimingFunction,
    view::{EmptyView, RenderContext, RetentionScope, View},
};

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
    timing_function: TimingFunction,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct AnimationConfiguration {
    from: f32,
    to: f32,
    duration: Duration,
    max_fps: Option<NonZeroU16>,
    timing_function: TimingFunction,
}

impl AnimationConfiguration {
    fn native_animation(self) -> Animation {
        let animation = Animation::new(self.duration)
            .with_easing(move |progress| self.timing_function.sample(progress));
        match self.max_fps {
            Some(max_fps) => animation.with_max_fps(f32::from(max_fps.get())),
            None => animation,
        }
    }
}

/// Freezes the first requested configuration for this GPUI element identity.
struct AnimatedOpacityElement {
    id: ElementId,
    animation_id: ElementId,
    child: Option<AnyElement>,
    configuration: AnimationConfiguration,
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
        let id = id.into();
        if id.as_str().trim().is_empty() {
            return Err(OpacityAnimationError::EmptyIdentity);
        }
        validate_opacity("from", from)?;
        validate_opacity("to", to)?;
        validate_duration(duration)?;
        Ok(Self {
            id,
            child,
            from,
            to,
            duration,
            max_fps: None,
            timing_function: TimingFunction::Linear,
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

    /// Selects the mount-time curve. A stable identity keeps its first curve.
    pub fn timing_function(mut self, timing_function: TimingFunction) -> Self {
        self.timing_function = timing_function;
        self
    }

    fn configuration(&self) -> AnimationConfiguration {
        AnimationConfiguration {
            from: self.from,
            to: self.to,
            duration: self.duration,
            max_fps: self.max_fps,
            timing_function: self.timing_function,
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
        AnimatedOpacityElement {
            id: format!("rustui-animated-opacity-config-{}", self.id.as_str()).into(),
            animation_id: format!("rustui-animated-opacity-animation-{}", self.id.as_str()).into(),
            child: Some(child),
            configuration: self.configuration(),
        }
        .into_any_element()
    }
}

fn retain_initial_configuration(
    state: Option<AnimationConfiguration>,
    requested: AnimationConfiguration,
) -> AnimationConfiguration {
    state.unwrap_or(requested)
}

impl IntoElement for AnimatedOpacityElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for AnimatedOpacityElement {
    type RequestLayoutState = AnyElement;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        global_id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let requested = self.configuration;
        let animation_id = self.animation_id.clone();
        let child = self
            .child
            .take()
            .expect("AnimatedOpacityElement requests layout only once");
        window.with_element_state(
            global_id.expect("AnimatedOpacityElement always has an element ID"),
            |state, window| {
                let configuration = retain_initial_configuration(state, requested);
                let from = configuration.from;
                let to = configuration.to;
                let mut element = div()
                    .self_start()
                    .with_animation(
                        animation_id,
                        configuration.native_animation(),
                        move |element, progress| {
                            element.opacity(interpolate_opacity(from, to, progress))
                        },
                    )
                    .child(child)
                    .into_any_element();
                let result = (element.request_layout(window, cx), element);
                (result, configuration)
            },
        )
    }

    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.prepaint(window, cx);
    }

    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        element: &mut Self::RequestLayoutState,
        _: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        element.paint(window, cx);
    }
}

/// Invalid arguments passed to [`AnimatedOpacity::new`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpacityAnimationError {
    /// The animation identity was empty or whitespace-only.
    EmptyIdentity,
    /// An opacity endpoint was non-finite or outside `0..=1`.
    InvalidOpacity { endpoint: &'static str },
    /// The animation duration was zero.
    ZeroDuration,
}

impl fmt::Display for OpacityAnimationError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyIdentity => {
                formatter.write_str("opacity animation identity must be nonempty")
            }
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
        AnimatedOpacity, AnimationConfiguration, OpacityAnimationError, interpolate_opacity,
        retain_initial_configuration, validate_duration, validate_opacity,
    };
    use crate::{animation::timing::TimingFunction, view::EmptyView};

    #[test]
    fn defaults_to_native_unthrottled_one_shot() {
        let fade =
            AnimatedOpacity::new("fade", EmptyView, 0.0, 1.0, Duration::from_secs(1)).unwrap();
        let animation = fade.configuration().native_animation();
        assert_eq!(animation.max_fps, None);
        assert_eq!(animation.duration, Duration::from_secs(1));
        assert!(animation.oneshot);
        assert_eq!((animation.easing)(0.25), 0.25);
    }

    #[test]
    fn forwards_curve_with_duration_and_frame_limit_to_gpui() {
        let animation = AnimatedOpacity::new("ease", EmptyView, 0.0, 1.0, Duration::from_secs(2))
            .unwrap()
            .max_fps(NonZeroU16::new(30).unwrap())
            .timing_function(TimingFunction::EaseIn)
            .configuration()
            .native_animation();
        assert_eq!((animation.easing)(0.5), 0.125);
        assert_eq!(animation.duration, Duration::from_secs(2));
        assert_eq!(animation.max_fps, Some(30.0));
        assert!(animation.oneshot);
    }

    #[test]
    fn forwards_frame_limits_and_replaces_previous_configuration() {
        let mut fade =
            AnimatedOpacity::new("fade", EmptyView, 0.0, 1.0, Duration::from_secs(1)).unwrap();
        for limit in [1, 30, u16::MAX] {
            fade = fade.max_fps(NonZeroU16::new(limit).unwrap());
            let animation = fade.configuration().native_animation();
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
    fn rejects_blank_ids_and_preserves_nonblank_identity() {
        for id in ["", " \t\n"] {
            assert_eq!(
                AnimatedOpacity::new(id, EmptyView, 0.0, 1.0, Duration::from_secs(1)).err(),
                Some(OpacityAnimationError::EmptyIdentity)
            );
        }

        let fade =
            AnimatedOpacity::new(" fade ", EmptyView, 0.0, 1.0, Duration::from_secs(1)).unwrap();
        assert_eq!(fade.id.as_str(), " fade ");
    }

    #[test]
    fn interpolates_between_opacity_endpoints() {
        assert_eq!(interpolate_opacity(0.0, 1.0, 0.0), 0.0);
        assert_eq!(interpolate_opacity(0.0, 1.0, 0.5), 0.5);
        assert_eq!(interpolate_opacity(0.0, 1.0, 1.0), 1.0);
        assert_eq!(interpolate_opacity(1.0, 0.0, 0.25), 0.75);
    }

    #[test]
    fn stable_identity_keeps_mount_time_animation_configuration() {
        let initial = AnimationConfiguration {
            from: 0.2,
            to: 0.8,
            duration: Duration::from_secs(2),
            max_fps: NonZeroU16::new(30),
            timing_function: TimingFunction::EaseIn,
        };
        let updated = AnimationConfiguration {
            from: 0.0,
            to: 1.0,
            duration: Duration::from_secs(5),
            max_fps: None,
            timing_function: TimingFunction::EaseOut,
        };

        assert_eq!(retain_initial_configuration(None, initial), initial);
        assert_eq!(
            retain_initial_configuration(Some(initial), updated),
            initial
        );
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
