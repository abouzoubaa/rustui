//! Fixed-column grid with GPUI-owned vertical row virtualization.

use std::{collections::HashSet, marker::PhantomData, num::NonZeroU16, ops::Range, rc::Rc};

use gpui::{
    AnyElement, App, Div, EntityId, IntoElement, ParentElement as _, Styled as _, Window, div, px,
    uniform_list,
};
use rustui_core::Identity;
use rustui_i18n::LayoutDirection;

use crate::{
    runtime::component_host::RetainedEntities,
    view::{EmptyView, RenderContext, RenderSeed, RetentionScope, View},
};

/// A snapshot item with a stable identity, independent of its current index.
pub struct LazyGridItem<T> {
    id: Identity,
    value: T,
}

impl<T> LazyGridItem<T> {
    pub fn new(id: impl Into<Identity>, value: T) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "LazyGrid item identity must be nonempty"
        );
        Self { id, value }
    }

    pub fn id(&self) -> &Identity {
        &self.id
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

struct GridState {
    seed: RenderSeed,
    retained: RetainedEntities,
    generation: u64,
}

/// Virtualizes rows of arbitrary RustUI cell views with equal fixed columns.
pub struct LazyGrid<T, V, F>
where
    T: 'static,
    V: View,
    F: Fn(usize, &T) -> V + 'static,
{
    id: Identity,
    items: Rc<[LazyGridItem<T>]>,
    columns: NonZeroU16,
    factory: Rc<F>,
    viewport_height: f32,
    row_height: f32,
    column_gap: f32,
    row_gap: f32,
    view: PhantomData<fn() -> V>,
}

impl<T, V, F> LazyGrid<T, V, F>
where
    T: 'static,
    V: View,
    F: Fn(usize, &T) -> V + 'static,
{
    pub fn new(
        id: impl Into<Identity>,
        items: Rc<[LazyGridItem<T>]>,
        columns: NonZeroU16,
        viewport_height: f32,
        row_height: f32,
        factory: F,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "LazyGrid identity must be nonempty"
        );
        assert!(
            columns.get() <= i16::MAX as u16,
            "LazyGrid columns exceed GPUI's supported range"
        );
        validate_positive(viewport_height, "viewport height");
        validate_positive(row_height, "row height");
        validate_ids(&items);
        validate_row_span(
            row_height,
            0.0,
            items.len().div_ceil(usize::from(columns.get())),
        );
        Self {
            id,
            items,
            columns,
            factory: Rc::new(factory),
            viewport_height,
            row_height,
            column_gap: 0.0,
            row_gap: 0.0,
            view: PhantomData,
        }
    }

    pub fn column_gap(mut self, gap: f32) -> Self {
        validate_gap(gap, "column gap");
        let gaps = f32::from(self.columns.get() - 1);
        assert!(
            (gap * gaps).is_finite(),
            "LazyGrid total column gap must be finite"
        );
        self.column_gap = gap;
        self
    }

    pub fn row_gap(mut self, gap: f32) -> Self {
        validate_gap(gap, "row gap");
        validate_row_span(
            self.row_height,
            gap,
            self.items.len().div_ceil(usize::from(self.columns.get())),
        );
        self.row_gap = gap;
        self
    }
}

fn validate_positive(value: f32, name: &str) {
    assert!(
        value.is_finite() && value > 0.0,
        "LazyGrid {name} must be finite and positive"
    );
}

fn validate_gap(value: f32, name: &str) {
    assert!(
        value.is_finite() && value >= 0.0,
        "LazyGrid {name} must be finite and nonnegative"
    );
}

fn validate_row_span(row_height: f32, row_gap: f32, row_count: usize) {
    let row_extent = row_height + row_gap;
    assert!(
        row_extent.is_finite(),
        "LazyGrid row height plus gap must be finite"
    );
    assert!(
        f64::from(row_extent) * row_count as f64 <= f64::from(f32::MAX),
        "LazyGrid total row span must be finite"
    );
}

fn validate_ids<T>(items: &[LazyGridItem<T>]) -> HashSet<Identity> {
    let mut ids = HashSet::with_capacity(items.len());
    for item in items {
        assert!(
            !item.id.as_str().trim().is_empty(),
            "LazyGrid item identity must be nonempty"
        );
        assert!(
            ids.insert(item.id.clone()),
            "LazyGrid item identity used twice"
        );
    }
    ids
}

fn cell_range(row: usize, columns: usize, count: usize) -> Range<usize> {
    let start = row.saturating_mul(columns).min(count);
    start..start.saturating_add(columns).min(count)
}

fn map_row_cells<T, R>(
    row: usize,
    columns: usize,
    items: &[LazyGridItem<T>],
    mut render: impl FnMut(usize, &LazyGridItem<T>) -> R,
) -> Vec<R> {
    cell_range(row, columns, items.len())
        .map(|index| render(index, &items[index]))
        .collect()
}

fn native_row(direction: LayoutDirection, height: f32, gap: f32) -> Div {
    // uniform_list supplies available width, not a fixed row-root width.
    // Resolve it explicitly before distributing space to zero-basis cells.
    let row = div().flex().flex_row().w_full().gap(px(gap)).h(px(height));
    match direction {
        LayoutDirection::LeftToRight => row,
        LayoutDirection::RightToLeft => row.flex_row_reverse(),
    }
}

impl<T, V, F> View for LazyGrid<T, V, F>
where
    T: 'static,
    V: View,
    F: Fn(usize, &T) -> V + 'static,
{
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let seed = context.seed();
        let ids = validate_ids(&self.items);
        let grid_id = self.id.clone();
        let items = self.items.clone();
        let factory = self.factory.clone();
        let columns = usize::from(self.columns.get());
        let row_count = items.len().div_ceil(columns);
        let row_height = self.row_height;
        let row_gap = self.row_gap;
        let column_gap = self.column_gap;
        let viewport_height = self.viewport_height;
        let direction = context.layout_direction();
        context.with_element_scope(RetentionScope::LazyGrid(grid_id.clone()), |context| {
            let (state, _) =
                context.retained_entity(Identity::from("grid-state"), |_, _| GridState {
                    seed: seed.clone(),
                    retained: RetainedEntities::default(),
                    generation: 0,
                });
            let generation = context.update_entity(&state, |state, _, _| {
                state.seed = seed;
                state.retained.prune_cells(&ids);
                state.generation = state.generation.wrapping_add(1);
                state.generation
            });
            let weak = state.downgrade();
            uniform_list(
                "rustui-lazy-grid-list",
                row_count,
                move |rows, window, cx| {
                    weak.update(cx, |state, cx| {
                        if state.generation != generation {
                            return Vec::new();
                        }
                        render_rows(
                            state,
                            rows,
                            cx.entity_id(),
                            &items,
                            columns,
                            factory.as_ref(),
                            row_height,
                            row_gap,
                            column_gap,
                            direction,
                            window,
                            cx,
                        )
                    })
                    .unwrap_or_default()
                },
            )
            .h(px(viewport_height))
            .w_full()
            .min_w_0()
            .into_any_element()
        })
    }
}

#[allow(clippy::too_many_arguments)]
fn render_rows<T, V, F>(
    state: &mut GridState,
    rows: Range<usize>,
    grid_instance: EntityId,
    items: &[LazyGridItem<T>],
    columns: usize,
    factory: &F,
    row_height: f32,
    row_gap: f32,
    column_gap: f32,
    direction: LayoutDirection,
    window: &mut Window,
    cx: &mut App,
) -> Vec<AnyElement>
where
    T: 'static,
    V: View,
    F: Fn(usize, &T) -> V,
{
    let mut result = Vec::with_capacity(rows.len());
    for row in rows {
        let range = cell_range(row, columns, items.len());
        if range.is_empty() {
            continue;
        }
        let mut cells = map_row_cells(row, columns, items, |index, item| {
            state.retained.begin_cell(&item.id);
            let seed = state.seed.clone();
            let mut context =
                seed.cell_context(grid_instance, &item.id, window, cx, &mut state.retained);
            let cell = context
                .with_element_scope(RetentionScope::Key(item.id.clone()), |context| {
                    factory(index, &item.value).into_element(context)
                });
            drop(context);
            state.retained.finish_cell(&item.id);
            div()
                .flex_1()
                .flex_basis(px(0.0))
                .min_w_0()
                .h(px(row_height))
                .overflow_hidden()
                .child(cell)
                .into_any_element()
        });
        while cells.len() < columns {
            cells.push(
                div()
                    .flex_1()
                    .flex_basis(px(0.0))
                    .min_w_0()
                    .into_any_element(),
            );
        }
        let row_element = native_row(direction, row_height + row_gap, column_gap);
        result.push(row_element.children(cells).into_any_element());
    }
    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn total_row_span_must_remain_finite() {
        use super::{LazyGrid, LazyGridItem};
        use std::{
            num::NonZeroU16,
            panic::{AssertUnwindSafe, catch_unwind},
            rc::Rc,
        };

        let columns = NonZeroU16::new(1).unwrap();
        let too_many_rows = Rc::from([
            LazyGridItem::new("a", 0),
            LazyGridItem::new("b", 1),
            LazyGridItem::new("c", 2),
        ]);
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                LazyGrid::new(
                    "grid",
                    too_many_rows,
                    columns,
                    100.0,
                    f32::MAX / 2.0,
                    |_, _| crate::view::EmptyView,
                )
            }))
            .is_err()
        );

        let two_rows = Rc::from([LazyGridItem::new("a", 0), LazyGridItem::new("b", 1)]);
        let grid = LazyGrid::new("grid", two_rows, columns, 100.0, f32::MAX * 0.4, |_, _| {
            crate::view::EmptyView
        });
        assert!(catch_unwind(AssertUnwindSafe(|| grid.row_gap(f32::MAX * 0.2))).is_err());
    }

    use super::*;
    use std::cell::Cell;

    #[test]
    fn native_rows_fill_viewport_width_before_dividing_columns() {
        for (direction, expected) in [
            (LayoutDirection::LeftToRight, gpui::FlexDirection::Row),
            (
                LayoutDirection::RightToLeft,
                gpui::FlexDirection::RowReverse,
            ),
        ] {
            let mut row = native_row(direction, 88.0, 6.0);
            let style = row.style();
            assert_eq!(style.size.width, Some(gpui::relative(1.0).into()));
            assert_eq!(style.size.height, Some(px(88.0).into()));
            assert_eq!(style.flex_direction, Some(expected));
            assert_eq!(style.gap.width, Some(px(6.0).into()));
        }
    }

    #[test]
    fn row_mapping_handles_partial_last_row_and_empty_range() {
        assert_eq!(cell_range(0, 3, 8), 0..3);
        assert_eq!(cell_range(1, 3, 8), 3..6);
        assert_eq!(cell_range(2, 3, 8), 6..8);
        assert_eq!(cell_range(3, 3, 8), 8..8);
        assert_eq!(cell_range(0, 3, 0), 0..0);
    }

    #[test]
    fn duplicate_ids_are_rejected_across_full_snapshot() {
        let items = [LazyGridItem::new("a", 1), LazyGridItem::new("a", 2)];
        assert!(std::panic::catch_unwind(|| validate_ids(&items)).is_err());
    }

    #[test]
    fn dimensions_reject_invalid_values() {
        assert!(std::panic::catch_unwind(|| validate_positive(0.0, "row height")).is_err());
        assert!(
            std::panic::catch_unwind(|| validate_positive(f32::INFINITY, "viewport height"))
                .is_err()
        );
        assert!(std::panic::catch_unwind(|| validate_gap(-1.0, "row gap")).is_err());
        let empty: Rc<[LazyGridItem<i32>]> = Vec::new().into();
        let grid = LazyGrid::new(
            "grid",
            empty,
            NonZeroU16::new(1).unwrap(),
            100.0,
            f32::MAX,
            |_, _| crate::components::basic::text::Text("cell"),
        );
        assert!(std::panic::catch_unwind(|| grid.row_gap(f32::MAX)).is_err());
        let make_grid = |columns| {
            LazyGrid::new(
                "grid",
                Vec::<LazyGridItem<i32>>::new().into(),
                NonZeroU16::new(columns).unwrap(),
                100.0,
                20.0,
                |_, _| crate::components::basic::text::Text("cell"),
            )
        };
        assert!(std::panic::catch_unwind(|| make_grid(3).column_gap(f32::MAX)).is_err());
        assert_eq!(make_grid(1).column_gap(f32::MAX).column_gap, f32::MAX);
    }

    #[test]
    fn cell_factory_runs_only_for_requested_row() {
        let items: Vec<_> = (0..10_000)
            .map(|index| LazyGridItem::new(index.to_string(), index))
            .collect();
        let calls = Cell::new(0);
        let indices = map_row_cells(100, 4, &items, |index, _| {
            calls.set(calls.get() + 1);
            index
        });
        assert_eq!(indices, vec![400, 401, 402, 403]);
        assert_eq!(calls.get(), 4);
    }

    #[test]
    fn requested_row_uses_current_order_values_and_stable_ids() {
        let first = [
            LazyGridItem::new("a", 10),
            LazyGridItem::new("b", 20),
            LazyGridItem::new("c", 30),
        ];
        let updated = [
            LazyGridItem::new("c", 31),
            LazyGridItem::new("a", 11),
            LazyGridItem::new("b", 21),
        ];
        let collect = |items: &[LazyGridItem<i32>]| {
            map_row_cells(0, 2, items, |index, item| {
                (index, item.id().as_str().to_owned(), *item.value())
            })
        };

        assert_eq!(collect(&first), [(0, "a".into(), 10), (1, "b".into(), 20)]);
        assert_eq!(
            collect(&updated),
            [(0, "c".into(), 31), (1, "a".into(), 11)]
        );
    }
}
