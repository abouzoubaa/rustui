//! State-driven opacity using GPUI's retained native spring.

use crate::view::{EmptyView, RenderContext, RetentionScope, View};
use gpui::{AnimationExt as _, AnyElement, IntoElement as _, ParentElement as _, Styled as _, div};
use rustui_core::Identity;
use std::{error::Error, fmt};

/// Validated physical parameters for an opacity spring.
/// Positive damping permits settling. Unrepresentable native intermediates
/// are rejected. Defaults: stiffness 170, damping 26, mass 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpringConfig {
    stiffness: f32,
    damping: f32,
    mass: f32,
}

impl SpringConfig {
    pub fn new(stiffness: f32, damping: f32, mass: f32) -> Result<Self, SpringOpacityError> {
        let positive = |value: f32| value.is_finite() && value > 0.0;
        if ![stiffness, damping, mass].into_iter().all(positive) {
            return Err(SpringOpacityError::InvalidPhysics);
        }
        // Mirror pinned native canonical/propagator intermediates: finite inputs
        // alone do not prevent division/product overflow or underflow.
        let frequency_squared = stiffness / mass;
        let product = stiffness * mass;
        let denominator = 2.0 * product.sqrt();
        let frequency = frequency_squared.sqrt();
        let ratio = damping / denominator;
        let ratio_squared = ratio * ratio;
        let root_sum = ratio + (ratio_squared - 1.0).max(0.0).sqrt();
        if ![
            frequency_squared,
            product,
            denominator,
            frequency,
            frequency * frequency,
            ratio,
            ratio_squared,
            frequency * ratio,
            root_sum,
            frequency * root_sum,
            frequency / root_sum,
        ]
        .into_iter()
        .all(positive)
        {
            return Err(SpringOpacityError::InvalidPhysics);
        }
        Ok(Self {
            stiffness,
            damping,
            mass,
        })
    }

    fn native(self) -> gpui::SpringConfig {
        gpui::SpringConfig::new(self.stiffness, self.damping, self.mass)
    }
}

impl Default for SpringConfig {
    fn default() -> Self {
        Self {
            stiffness: 170.0,
            damping: 26.0,
            mass: 1.0,
        }
    }
}

/// A child whose opacity follows a state-driven target using a native spring.
/// Keep the ID stable across target changes: GPUI retains position and velocity.
/// A new identity starts at its target unless [`Self::from`] supplies an initial
/// value. Reduced motion resolves to the target. Presentation clamps overshoot;
/// the internal native spring can move outside `0..=1`.
/// IDs must be unique among sibling springs in the composition scope.
pub struct SpringOpacity<V: View> {
    id: Identity,
    child: V,
    target: f32,
    initial: Option<f32>,
    config: SpringConfig,
    epsilon: f32,
}

impl<V: View> SpringOpacity<V> {
    pub fn new(id: impl Into<Identity>, child: V, target: f32) -> Result<Self, SpringOpacityError> {
        validate_opacity("target", target)?;
        Ok(Self {
            id: id.into(),
            child,
            target,
            initial: None,
            config: SpringConfig::default(),
            epsilon: 0.001,
        })
    }

    /// Seeds only a newly mounted native identity; does not replay an existing one.
    pub fn from(mut self, initial: f32) -> Result<Self, SpringOpacityError> {
        validate_opacity("initial", initial)?;
        self.initial = Some(initial);
        Ok(self)
    }

    pub fn config(mut self, config: SpringConfig) -> Self {
        self.config = config;
        self
    }

    /// Native positional tolerance, finite in `[f32::EPSILON, 1]`.
    /// GPUI scales velocity tolerance by the natural frequency.
    pub fn epsilon(mut self, epsilon: f32) -> Result<Self, SpringOpacityError> {
        if !epsilon.is_finite() || !(f32::EPSILON..=1.0).contains(&epsilon) {
            return Err(SpringOpacityError::InvalidEpsilon);
        }
        self.epsilon = epsilon;
        Ok(self)
    }

    fn native_animation(&self) -> gpui::SpringAnimation<f32> {
        let animation = gpui::SpringAnimation::new(self.config.native())
            .to(self.target)
            .with_epsilon(self.epsilon);
        match self.initial {
            Some(initial) => animation.from(initial),
            None => animation,
        }
    }
}

impl<V: View> View for SpringOpacity<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let child = context
            .with_element_scope(RetentionScope::SpringOpacity(self.id.clone()), |context| {
                self.child.into_element(context)
            });
        div()
            .self_start()
            .with_spring(
                format!("rustui-spring-opacity-{}", self.id.as_str()),
                self.native_animation(),
                |element, value| element.opacity(presented_opacity(value)),
            )
            .child(child)
            .into_any_element()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpringOpacityError {
    InvalidOpacity { endpoint: &'static str },
    InvalidPhysics,
    InvalidEpsilon,
}

impl fmt::Display for SpringOpacityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidOpacity { endpoint } => write!(formatter, "opacity `{endpoint}` must be finite in 0..=1"),
            Self::InvalidPhysics => formatter.write_str("spring physics must be positive and finite with representable native intermediates"),
            Self::InvalidEpsilon => formatter.write_str("spring epsilon must be finite in [f32::EPSILON, 1]"),
        }
    }
}
impl Error for SpringOpacityError {}

fn validate_opacity(endpoint: &'static str, value: f32) -> Result<(), SpringOpacityError> {
    if value.is_finite() && (0.0..=1.0).contains(&value) {
        Ok(())
    } else {
        Err(SpringOpacityError::InvalidOpacity { endpoint })
    }
}

fn presented_opacity(value: f32) -> f32 {
    // Defensive fallback keeps unexpected nonfinite native output out of style.
    if value.is_finite() {
        value.clamp(0.0, 1.0)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_endpoints_initial_and_tolerance() {
        for value in [f32::NAN, f32::INFINITY, -0.1, 1.1] {
            assert!(SpringOpacity::new("test", EmptyView, value).is_err());
            assert!(
                SpringOpacity::new("test", EmptyView, 1.0)
                    .unwrap()
                    .from(value)
                    .is_err()
            );
        }
        for epsilon in [f32::NAN, f32::INFINITY, -1.0, 0.0, f32::MIN_POSITIVE, 1.1] {
            assert!(
                SpringOpacity::new("test", EmptyView, 0.0)
                    .unwrap()
                    .epsilon(epsilon)
                    .is_err()
            );
        }
        let spring = SpringOpacity::new("test", EmptyView, 1.0).unwrap();
        assert_eq!(spring.initial, None);
        assert_eq!(spring.epsilon, 0.001);
        assert_eq!(spring.from(0.0).unwrap().initial, Some(0.0));
    }

    #[test]
    fn rejects_invalid_and_unrepresentable_physics() {
        for value in [f32::NAN, f32::INFINITY, -1.0, 0.0] {
            assert!(SpringConfig::new(value, 26.0, 1.0).is_err());
            assert!(SpringConfig::new(170.0, value, 1.0).is_err());
            assert!(SpringConfig::new(170.0, 26.0, value).is_err());
        }
        assert!(SpringConfig::new(f32::MAX, 1.0, f32::MAX).is_err());
        assert!(SpringConfig::new(f32::MIN_POSITIVE, 1.0, f32::MIN_POSITIVE).is_err());
        assert!(SpringConfig::new(1.0, f32::MAX, 1.0).is_err());
        assert_eq!(
            SpringConfig::new(170.0, 26.0, 1.0).unwrap(),
            SpringConfig::default()
        );
        assert!(SpringConfig::new(170.0, 8.0, 1.0).is_ok());
    }

    #[test]
    fn maps_native_physics_and_clamps_presentation() {
        let native = SpringConfig::default().native();
        assert_eq!(
            (native.stiffness, native.damping, native.mass),
            (170.0, 26.0, 1.0)
        );
        for (input, expected) in [(-0.5, 0.0), (0.4, 0.4), (1.5, 1.0), (f32::NAN, 0.0)] {
            assert_eq!(presented_opacity(input), expected);
        }
    }
}
