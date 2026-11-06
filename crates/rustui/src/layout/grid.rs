//! Fixed-column grid layout built on GPUI's native grid support.

use std::num::NonZeroU16;

use gpui::{AnyElement, IntoElement, ParentElement, Styled, div, px};
use rustui_i18n::LayoutDirection;

use crate::view::{EmptyView, RenderContext, View};

type ChildRenderer = Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;

/// A fixed-column grid with row-major, locale-aware item placement.
pub struct Grid {
    columns: NonZeroU16,
    children: Vec<ChildRenderer>,
    spacing: f32,
    fill: bool,
    grow: bool,
}

impl Grid {
    /// Creates a grid with a fixed nonzero number of equal-width columns.
    ///
    /// GPUI's placement API uses signed 16-bit grid lines, so the column count
    /// cannot exceed `i16::MAX`.
    pub fn new(columns: NonZeroU16) -> Self {
        assert!(
            columns.get() <= i16::MAX as u16,
            "Grid columns must not exceed the supported GPUI grid line range"
        );
        Self {
            columns,
            children: Vec::new(),
            spacing: 0.0,
            fill: false,
            grow: false,
        }
    }

    /// Sets the same gap between rows and columns in logical pixels.
    pub fn spacing(mut self, spacing: f32) -> Self {
        assert!(
            spacing.is_finite() && spacing >= 0.0,
            "Grid spacing must be finite and nonnegative"
        );
        self.spacing = spacing;
        self
    }

    /// Sets both dimensions to the available parent size.
    pub fn fill(mut self) -> Self {
        self.fill = true;
        self
    }

    /// Takes available space along the parent's flex axis.
    pub fn grow(mut self) -> Self {
        self.grow = true;
        self
    }

    /// Appends one grid item.
    pub fn child<V: View>(mut self, child: V) -> Self {
        self.children
            .push(Box::new(move |context| child.into_element(context)));
        self
    }

    /// Appends grid items in source order.
    pub fn children<V: View>(mut self, children: impl IntoIterator<Item = V>) -> Self {
        for child in children {
            self = self.child(child);
        }
        self
    }

    /// Inserts a group's children as direct siblings without an extra layout node.
    pub fn group(mut self, group: super::group::Group) -> Self {
        self.children.extend(group.into_renderers());
        self
    }
}

impl View for Grid {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let columns = self.columns.get();
        let rows = self.children.len().div_ceil(usize::from(columns));
        validate_gap_span(self.spacing, usize::from(columns - 1));
        validate_gap_span(self.spacing, rows.saturating_sub(1));
        let direction = context.layout_direction();
        let children = self.children.iter().enumerate().map(|(index, child)| {
            let (row, column) = grid_position(index, columns, direction)
                .expect("Grid item count exceeds GPUI's supported grid line range");
            div().col_start(column).row_start(row).child(child(context))
        });

        let mut grid = div().grid().grid_cols(columns).gap(px(self.spacing));
        if self.fill {
            grid = grid.size_full().min_w_0().min_h_0();
        }
        if self.grow {
            grid = grid.flex_1().min_w_0().min_h_0();
        }
        grid.children(children).into_any_element()
    }
}

fn grid_position(index: usize, columns: u16, direction: LayoutDirection) -> Option<(i16, i16)> {
    let columns = usize::from(columns);
    let row = index.checked_div(columns)?.checked_add(1)?;
    let logical_column = index % columns;
    let column = match direction {
        LayoutDirection::LeftToRight => logical_column + 1,
        LayoutDirection::RightToLeft => columns - logical_column,
    };
    Some((i16::try_from(row).ok()?, i16::try_from(column).ok()?))
}

fn validate_gap_span(spacing: f32, gap_count: usize) {
    assert!(
        f64::from(spacing) * gap_count as f64 <= f64::from(f32::MAX),
        "Grid total gap span must be finite"
    );
}

#[cfg(test)]
mod tests {
    use super::grid_position;
    use rustui_i18n::LayoutDirection;
    use std::num::NonZeroU16;

    #[test]
    fn places_items_row_major_and_mirrors_rtl_columns() {
        let ltr = LayoutDirection::LeftToRight;
        let rtl = LayoutDirection::RightToLeft;
        assert_eq!(grid_position(0, 3, ltr), Some((1, 1)));
        assert_eq!(grid_position(2, 3, ltr), Some((1, 3)));
        assert_eq!(grid_position(3, 3, ltr), Some((2, 1)));
        assert_eq!(grid_position(0, 3, rtl), Some((1, 3)));
        assert_eq!(grid_position(2, 3, rtl), Some((1, 1)));
        assert_eq!(grid_position(3, 3, rtl), Some((2, 3)));
    }

    #[test]
    fn rejects_grid_line_and_column_count_overflow() {
        assert_eq!(
            grid_position(i16::MAX as usize, 1, LayoutDirection::LeftToRight),
            None
        );
        assert!(
            std::panic::catch_unwind(|| { super::Grid::new(NonZeroU16::new(u16::MAX).unwrap()) })
                .is_err()
        );
    }

    #[test]
    fn rejects_invalid_spacing() {
        for spacing in [-1.0, f32::NAN, f32::INFINITY, f32::NEG_INFINITY] {
            assert!(
                std::panic::catch_unwind(
                    || super::Grid::new(NonZeroU16::new(1).unwrap()).spacing(spacing)
                )
                .is_err()
            );
        }
    }

    #[test]
    fn rejects_finite_spacing_when_grid_gap_span_overflows() {
        assert!(std::panic::catch_unwind(|| {
            super::validate_gap_span(f32::MAX * 0.6, 2)
        })
        .is_err());
        super::validate_gap_span(f32::MAX * 0.6, 1);
    }
}
