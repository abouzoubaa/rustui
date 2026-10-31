//! Bounded progress curves for animations scheduled by GPUI.

/// A non-overshooting curve applied to normalized animation progress.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimingFunction {
    #[default]
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

impl TimingFunction {
    /// Samples the curve, clamping progress to `0..=1` and propagating NaN.
    pub fn sample(self, progress: f32) -> f32 {
        let t = progress.clamp(0.0, 1.0);
        match self {
            Self::Linear => t,
            Self::EaseIn => t.powi(3),
            Self::EaseOut => 1.0 - (1.0 - t).powi(3),
            Self::EaseInOut if t < 0.5 => 4.0 * t.powi(3),
            Self::EaseInOut => 1.0 - (-2.0 * t + 2.0).powi(3) / 2.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TimingFunction::*;

    #[test]
    fn curves_preserve_endpoints_bounds_order_and_invalid_progress_contract() {
        for curve in [Linear, EaseIn, EaseOut, EaseInOut] {
            assert_eq!(curve.sample(0.0), 0.0);
            assert_eq!(curve.sample(1.0), 1.0);
            assert_eq!(curve.sample(-1.0), 0.0);
            assert_eq!(curve.sample(2.0), 1.0);
            assert_eq!(curve.sample(f32::NEG_INFINITY), 0.0);
            assert_eq!(curve.sample(f32::INFINITY), 1.0);
            assert!(curve.sample(f32::NAN).is_nan());
            let mut previous = 0.0;
            for step in 0..=1000 {
                let value = curve.sample(step as f32 / 1000.0);
                assert!((0.0..=1.0).contains(&value));
                assert!(value >= previous);
                previous = value;
            }
        }
        assert_eq!(Linear.sample(0.25), 0.25);
        assert_eq!(EaseIn.sample(0.5), 0.125);
        assert_eq!(EaseOut.sample(0.5), 0.875);
        assert_eq!(EaseInOut.sample(0.25), 0.0625);
        assert_eq!(EaseInOut.sample(0.5), 0.5);
        assert_eq!(EaseInOut.sample(0.75), 0.9375);
    }
}
