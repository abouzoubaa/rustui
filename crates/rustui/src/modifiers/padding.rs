//! Logical insets resolved against RustUI's active layout direction.

use std::{error::Error, fmt};

use rustui_i18n::LayoutDirection;

/// Insets in logical block and inline directions, measured in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LogicalInsets {
    top: f32,
    bottom: f32,
    start: f32,
    end: f32,
}

impl LogicalInsets {
    /// Creates insets, rejecting negative and non-finite values.
    pub fn new(top: f32, bottom: f32, start: f32, end: f32) -> Result<Self, LogicalInsetsError> {
        for (name, value) in [
            ("top", top),
            ("bottom", bottom),
            ("start", start),
            ("end", end),
        ] {
            if !value.is_finite() || value < 0.0 {
                return Err(LogicalInsetsError { edge: name });
            }
        }
        Ok(Self {
            top,
            bottom,
            start,
            end,
        })
    }

    /// Resolves logical start/end into physical left/right edges.
    pub fn resolve(self, direction: LayoutDirection) -> PhysicalInsets {
        let (left, right) = match direction {
            LayoutDirection::LeftToRight => (self.start, self.end),
            LayoutDirection::RightToLeft => (self.end, self.start),
        };
        PhysicalInsets {
            top: self.top,
            right,
            bottom: self.bottom,
            left,
        }
    }
}

/// Insets resolved to physical top/right/bottom/left edges.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalInsets {
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub left: f32,
}

/// Error returned when a logical inset is negative or non-finite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LogicalInsetsError {
    edge: &'static str,
}

impl fmt::Display for LogicalInsetsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "logical inset `{}` must be finite and nonnegative",
            self.edge
        )
    }
}

impl Error for LogicalInsetsError {}

#[cfg(test)]
mod tests {
    use super::LogicalInsets;
    use rustui_i18n::LayoutDirection;

    #[test]
    fn resolves_logical_edges_for_ltr_and_rtl() {
        let insets = LogicalInsets::new(1.0, 2.0, 3.0, 4.0).unwrap();
        assert_eq!(
            insets.resolve(LayoutDirection::LeftToRight),
            super::PhysicalInsets {
                top: 1.0,
                right: 4.0,
                bottom: 2.0,
                left: 3.0,
            }
        );
        assert_eq!(
            insets.resolve(LayoutDirection::RightToLeft),
            super::PhysicalInsets {
                top: 1.0,
                right: 3.0,
                bottom: 2.0,
                left: 4.0,
            }
        );
    }

    #[test]
    fn rejects_negative_and_non_finite_insets() {
        assert!(LogicalInsets::new(-1.0, 0.0, 0.0, 0.0).is_err());
        assert!(LogicalInsets::new(0.0, f32::NAN, 0.0, 0.0).is_err());
        assert!(LogicalInsets::new(0.0, 0.0, 0.0, f32::INFINITY).is_err());
    }
}
