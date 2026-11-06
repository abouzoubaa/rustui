//! RustUI layout primitives.

pub mod alignment;
pub mod card;
pub mod container;
pub mod geometry;
pub mod grid;
pub mod group;
pub mod hstack;
pub mod lazy_grid;
pub mod scroll_view;
pub mod spacer;
pub mod split_view;
pub mod vstack;
pub mod zstack;

pub(super) fn validate_gap_span(gap: f32, child_count: usize, stack: &str) {
    let gap_count = child_count.saturating_sub(1);
    assert!(
        f64::from(gap) * gap_count as f64 <= f64::from(f32::MAX),
        "{stack} total gap must be finite"
    );
}

#[cfg(test)]
mod tests {
    use super::validate_gap_span;

    #[test]
    fn rejects_total_gap_when_f32_rounding_understates_gap_count() {
        let gap = f32::from_bits(0x737f_ffff);
        let child_count = 16_777_218;

        assert!(
            std::panic::catch_unwind(|| {
                validate_gap_span(gap, child_count, "Stack");
            })
            .is_err()
        );
    }
}

pub use card::{Card, CardView};
pub use container::{Container, ContainerView, Surface};
pub use geometry::{GeometryReader, GeometrySize};
pub use hstack::{HStack, VerticalAlignment};
pub use lazy_grid::{LazyGrid, LazyGridItem};
pub use scroll_view::{ScrollAxis, ScrollView, ScrollViewView};
pub use spacer::{Spacer, SpacerAxis, SpacerView};
pub use split_view::{SplitPanel, SplitView};
pub use vstack::{HorizontalAlignment, VStack};

pub use grid::Grid;
pub use zstack::{ZStack, ZStackAlignment};

pub use group::Group;
