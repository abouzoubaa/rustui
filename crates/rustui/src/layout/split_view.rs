//! Keyed, owner-retained composition using official GPUI Components resizing.

use std::ops::Range;

use gpui::{
    AnyElement, AppContext as _, Axis, Entity, InteractiveElement as _, IntoElement, Orientation,
    ParentElement as _, Pixels, Role, StatefulInteractiveElement as _, Styled as _, div,
    prelude::FluentBuilder as _, px,
};
use gpui_component::resizable::{ResizableState, h_resizable, resizable_panel, v_resizable};
use rustui_core::Identity;
use rustui_i18n::LayoutDirection;

use crate::view::{EmptyView, RenderContext, RetentionScope, View};

type ChildRenderer = Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;

const SPLITTER_ACCESSIBLE_NAME_ID: &str = "rustui-splitter-resize-divider";
const SPLITTER_ACCESSIBLE_NAME_FALLBACK: &str = "Resize divider";

fn splitter_accessible_name(localized: &str) -> String {
    if localized.trim() == SPLITTER_ACCESSIBLE_NAME_ID {
        SPLITTER_ACCESSIBLE_NAME_FALLBACK.to_owned()
    } else {
        localized.to_owned()
    }
}

/// One keyed split panel containing any public RustUI view.
pub struct SplitPanel {
    id: Identity,
    content: ChildRenderer,
    initial_size: Option<f32>,
    size_range: Range<f32>,
}

impl SplitPanel {
    /// Creates a flexible panel. IDs must be unique within the split.
    pub fn new<V: View>(id: impl Into<Identity>, content: V) -> Self {
        let id = checked_id(id.into());
        Self {
            id,
            content: Box::new(move |context| content.into_element(context)),
            initial_size: None,
            size_range: 100.0..f32::MAX,
        }
    }

    /// Sets the initial width or height in points, according to the split axis.
    /// Applies on insertion; subsequent renders preserve native resize state.
    /// Panics for non-finite or non-positive values.
    pub fn initial_size(mut self, size: f32) -> Self {
        assert!(
            size.is_finite() && size > 0.0,
            "SplitPanel initial size must be finite and positive"
        );
        self.initial_size = Some(size);
        self
    }

    /// Limits the native resize extent. The default minimum is 100 points.
    /// Panics unless both bounds are finite, nonnegative, and ascending.
    pub fn size_range(mut self, range: Range<f32>) -> Self {
        assert!(
            range.start.is_finite()
                && range.end.is_finite()
                && range.start >= 0.0
                && range.start < range.end,
            "SplitPanel size range must be finite, nonnegative, and ascending"
        );
        self.size_range = range;
        self
    }
}

/// Resizable horizontal or vertical composition, independent of navigation.
///
/// The enclosing component owns native resize state. Stable split and panel
/// IDs preserve child-local state across reorder. Omitted panels are released;
/// reinsertion starts fresh. Omitting the whole split releases its sizes too.
/// Axis changes reset size preferences but preserve surviving child identities.
/// Put this view in a parent with bounded width and height.
pub struct SplitView {
    id: Identity,
    axis: Axis,
    panels: Vec<SplitPanel>,
}

impl SplitView {
    /// Creates a split with panels arranged from left to right.
    pub fn horizontal(id: impl Into<Identity>) -> Self {
        Self {
            id: checked_id(id.into()),
            axis: Axis::Horizontal,
            panels: Vec::new(),
        }
    }

    /// Creates a split with panels arranged from top to bottom.
    pub fn vertical(id: impl Into<Identity>) -> Self {
        Self {
            axis: Axis::Vertical,
            ..Self::horizontal(id)
        }
    }

    /// Appends a uniquely identified panel. Duplicate IDs panic.
    pub fn panel(mut self, panel: SplitPanel) -> Self {
        assert!(
            !self.panels.iter().any(|existing| existing.id == panel.id),
            "duplicate SplitPanel ID: {}",
            panel.id
        );
        self.panels.push(panel);
        self
    }

    /// Appends panels in iteration order.
    pub fn panels(mut self, panels: impl IntoIterator<Item = SplitPanel>) -> Self {
        for panel in panels {
            self = self.panel(panel);
        }
        self
    }
}

fn checked_id(id: Identity) -> Identity {
    assert!(
        !id.as_str().trim().is_empty(),
        "SplitView and SplitPanel IDs must be nonempty"
    );
    id
}

fn panel_order(axis: Axis, direction: LayoutDirection, count: usize) -> Vec<usize> {
    let mut order: Vec<_> = (0..count).collect();
    if axis == Axis::Horizontal && direction == LayoutDirection::RightToLeft {
        order.reverse();
    }
    order
}

struct RetainedSplit {
    axis: Axis,
    ids: Vec<Identity>,
    size_ranges: Vec<Range<f32>>,
    initial_sizes: Vec<Option<f32>>,
    native: Entity<ResizableState>,
}

fn needs_reconciliation(
    old_ids: &[Identity],
    new_ids: &[Identity],
    old_ranges: &[Range<f32>],
    new_ranges: &[Range<f32>],
    old_axis: Axis,
    new_axis: Axis,
) -> bool {
    old_ids != new_ids || old_ranges != new_ranges || old_axis != new_axis
}

fn measured_preferences(container_size: Pixels, sizes: &[Pixels]) -> Vec<f32> {
    // Fresh native state holds positional placeholders until panel prepaint.
    // Its public container extent is recorded after those panel prepaints.
    if container_size > px(0.0) {
        sizes.iter().map(|size| size.as_f32()).collect()
    } else {
        Vec::new()
    }
}

fn divider_keyboard_size(
    axis: Axis,
    key: &str,
    current: Pixels,
    range: &Range<Pixels>,
) -> Option<Pixels> {
    // Arrow keys move the physical divider; RTL changes panel order, not key direction.
    let delta = match (axis, key) {
        (Axis::Horizontal, "left") => px(-10.0),
        (Axis::Horizontal, "right") => px(10.0),
        (Axis::Vertical, "up") => px(-10.0),
        (Axis::Vertical, "down") => px(10.0),
        _ => return None,
    };
    Some((current + delta).max(range.start).min(range.end))
}

fn divider_effective_range(
    container_size: Pixels,
    index: usize,
    configured_ranges: &[Range<f32>],
    panel_sizes: &[Pixels],
    current: Pixels,
) -> Range<Pixels> {
    let Some(configured) = configured_ranges.get(index) else {
        return px(0.0)..Pixels::MAX;
    };
    let configured = px(configured.start)..px(configured.end);
    if container_size <= px(0.0) {
        return configured;
    }

    let preceding_size = panel_sizes
        .iter()
        .take(index)
        .map(|size| f64::from(size.as_f32()))
        .sum::<f64>();
    let other_min = configured_ranges
        .iter()
        .skip(index + 1)
        .map(|range| f64::from(range.start))
        .sum::<f64>();
    let extent = f64::from(container_size.as_f32());
    // GPUI's resize_panel_at_handle expands this panel by shrinking only
    // panels to its right, but shrinking it transfers space only to the
    // immediately adjacent panel. Earlier panels therefore keep their size.
    let immediate_right_growth = panel_sizes
        .get(index + 1)
        .zip(configured_ranges.get(index + 1))
        .map(|(size, range)| f64::from(size.as_f32()) - f64::from(range.end))
        .unwrap_or(0.0);
    let min = f64::from(configured.start.as_f32())
        .max(f64::from(current.as_f32()) + immediate_right_growth);
    let max = f64::from(configured.end.as_f32()).min(extent - preceding_size - other_min);
    if min <= max {
        px(min as f32)..px(max as f32)
    } else {
        current..current
    }
}

fn divider_aria_values(
    current: Option<Pixels>,
    fallback: Pixels,
    range: &Range<Pixels>,
) -> (f64, f64, f64) {
    let value = current.unwrap_or(fallback).max(range.start).min(range.end);
    (
        value.as_f32() as f64,
        range.start.as_f32() as f64,
        range.end.as_f32() as f64,
    )
}

// Upstream native sizes are positional. Topology and range reconciliation map
// size preferences by ID; official GPUI still owns drag, fitting, and clamping.
fn reconcile_sizes<'a>(
    old_ids: &[Identity],
    measured: &[f32],
    previous: &[Option<f32>],
    panels: impl IntoIterator<Item = &'a SplitPanel>,
    same_axis: bool,
) -> Vec<Option<f32>> {
    panels
        .into_iter()
        .map(|panel| {
            let size = if same_axis {
                old_ids
                    .iter()
                    .position(|id| *id == panel.id)
                    .and_then(|index| {
                        measured
                            .get(index)
                            .copied()
                            .filter(|size| size.is_finite() && *size >= 0.0)
                            .or_else(|| previous.get(index).copied().flatten())
                    })
                    .or(panel.initial_size)
            } else {
                panel.initial_size
            };
            size.map(|size| size.clamp(panel.size_range.start, panel.size_range.end))
        })
        .collect()
}

impl View for SplitView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::SplitView(self.id.clone()), |context| {
            let direction = context.layout_direction();
            // GPUI's resizable group uses physical row order. Match RustUI's
            // other horizontal layouts by placing the first logical panel on
            // the right in RTL; reconciliation below maps native sizes by ID.
            let panels: Vec<_> = panel_order(self.axis, direction, self.panels.len())
                .into_iter()
                .map(|index| &self.panels[index])
                .collect();
            let ids: Vec<_> = panels.iter().map(|panel| panel.id.clone()).collect();
            let size_ranges: Vec<_> = panels
                .iter()
                .map(|panel| panel.size_range.clone())
                .collect();
            let (retained, _) =
                context.retained_entity(Identity::from("split-state"), |_, cx| RetainedSplit {
                    axis: self.axis,
                    ids: ids.clone(),
                    size_ranges: size_ranges.clone(),
                    initial_sizes: panels.iter().map(|panel| panel.initial_size).collect(),
                    native: cx.new(|_| ResizableState::default()),
                });
            let state = retained.read(context.app());
            if needs_reconciliation(
                &state.ids,
                &ids,
                &state.size_ranges,
                &size_ranges,
                state.axis,
                self.axis,
            ) {
                let native = state.native.read(context.app());
                let measured = measured_preferences(native.container_size(), native.sizes());
                let initial_sizes = reconcile_sizes(
                    &state.ids,
                    &measured,
                    &state.initial_sizes,
                    panels.iter().copied(),
                    state.axis == self.axis,
                );
                context.update_entity(&retained, |state, _, cx| {
                    state.ids = ids;
                    state.axis = self.axis;
                    state.size_ranges = size_ranges;
                    state.initial_sizes = initial_sizes;
                    state.native = cx.new(|_| ResizableState::default());
                });
            }
            if self.panels.is_empty() {
                return div().size_full().into_any_element();
            }
            let state = retained.read(context.app());
            let native = state.native.clone();
            let initial_sizes = state.initial_sizes.clone();
            let configured_ranges = state.size_ranges.clone();
            let mut group = match self.axis {
                Axis::Horizontal => h_resizable("rustui-split"),
                Axis::Vertical => v_resizable("rustui-split"),
            }
            .with_state(&native);
            for (index, panel) in panels.iter().enumerate() {
                let content = context
                    .with_element_scope(RetentionScope::Key(panel.id.clone()), |context| {
                        (panel.content)(context)
                    });
                let content = if index + 1 < panels.len() {
                    let axis = self.axis;
                    let native = native.clone();
                    let native_state = native.read(context.app());
                    let fallback = px(initial_sizes[index].unwrap_or(panel.size_range.start));
                    let current = native_state.sizes().get(index).copied().unwrap_or(fallback);
                    let range = divider_effective_range(
                        native_state.container_size(),
                        index,
                        &configured_ranges,
                        native_state.sizes(),
                        current,
                    );
                    let (value, min_value, max_value) =
                        divider_aria_values(Some(current), fallback, &range);
                    let divider = div()
                        .id(format!("rustui-split-divider-{}", panel.id.as_str()))
                        .absolute()
                        .when(axis == Axis::Horizontal, |this| {
                            this.top_0().right_0().bottom_0().w(px(9.0))
                        })
                        .when(axis == Axis::Vertical, |this| {
                            this.left_0().right_0().bottom_0().h(px(9.0))
                        })
                        .focusable()
                        .tab_stop(true)
                        .role(Role::Splitter)
                        .aria_label(splitter_accessible_name(
                            &context.localized_message(SPLITTER_ACCESSIBLE_NAME_ID),
                        ))
                        .aria_orientation(match axis {
                            Axis::Horizontal => Orientation::Vertical,
                            Axis::Vertical => Orientation::Horizontal,
                        })
                        .aria_numeric_value(value)
                        .aria_min_numeric_value(min_value)
                        .aria_max_numeric_value(max_value)
                        .aria_numeric_value_step(10.0)
                        .on_key_down(move |event, window, cx| {
                            if event.prefer_character_input
                                || event.keystroke.is_ime_in_progress()
                                || event.keystroke.modifiers.modified()
                            {
                                return;
                            }
                            let Some(current) = native.read(cx).sizes().get(index).copied() else {
                                return;
                            };
                            let Some(size) = divider_keyboard_size(
                                axis,
                                event.keystroke.key.as_ref(),
                                current,
                                &range,
                            ) else {
                                return;
                            };
                            native.update(&mut *cx, |state, cx| {
                                state.resize_panel(index, size, window, cx);
                            });
                            cx.stop_propagation();
                        });
                    div()
                        .relative()
                        .size_full()
                        .child(content)
                        .child(divider)
                        .into_any_element()
                } else {
                    content
                };
                let mut native_panel = resizable_panel()
                    .size_range(px(panel.size_range.start)..px(panel.size_range.end))
                    .child(content);
                if let Some(size) = initial_sizes[index] {
                    native_panel = native_panel.size(px(size));
                }
                group = group.child(native_panel);
            }
            group.into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splitter_name_uses_a_human_readable_fallback_without_localization() {
        assert_eq!(
            splitter_accessible_name(SPLITTER_ACCESSIBLE_NAME_ID),
            SPLITTER_ACCESSIBLE_NAME_FALLBACK
        );
        assert_eq!(
            splitter_accessible_name(" \trustui-splitter-resize-divider\n"),
            SPLITTER_ACCESSIBLE_NAME_FALLBACK
        );
        assert_eq!(
            splitter_accessible_name("Redimensionner le séparateur"),
            "Redimensionner le séparateur"
        );
    }

    fn panel(id: &str, size: f32) -> SplitPanel {
        SplitPanel::new(id, EmptyView).initial_size(size)
    }

    #[test]
    fn horizontal_panels_follow_layout_direction_while_vertical_panels_do_not() {
        assert_eq!(
            panel_order(Axis::Horizontal, LayoutDirection::LeftToRight, 3),
            [0, 1, 2]
        );
        assert_eq!(
            panel_order(Axis::Horizontal, LayoutDirection::RightToLeft, 3),
            [2, 1, 0]
        );
        assert_eq!(
            panel_order(Axis::Vertical, LayoutDirection::RightToLeft, 3),
            [0, 1, 2]
        );

        let logical_panels = [panel("sidebar", 180.0), panel("content", 320.0)];
        let ltr_ids = [Identity::from("sidebar"), Identity::from("content")];
        let rtl_panels: Vec<_> = panel_order(
            Axis::Horizontal,
            LayoutDirection::RightToLeft,
            logical_panels.len(),
        )
        .into_iter()
        .map(|index| &logical_panels[index])
        .collect();
        assert_eq!(
            reconcile_sizes(&ltr_ids, &[180.0, 320.0], &[None, None], rtl_panels, true),
            [Some(320.0), Some(180.0)]
        );
    }

    #[test]
    fn reconcile_reorder_remove_insert_and_axis_change() {
        let ids = vec![Identity::from("a"), Identity::from("b")];
        let panels = vec![panel("b", 120.0), panel("c", 140.0), panel("a", 160.0)];
        assert_eq!(
            reconcile_sizes(&ids, &[210.0, 290.0], &[None, None], &panels, true),
            vec![Some(290.0), Some(140.0), Some(210.0)]
        );
        let kept = vec![panel("b", 120.0)];
        assert_eq!(
            reconcile_sizes(&ids, &[210.0, 290.0], &[None, None], &kept, true),
            vec![Some(290.0)]
        );
        // A removed ID has no stored preference on a later reinsertion.
        assert_eq!(
            reconcile_sizes(&[Identity::from("b")], &[290.0], &[None], &panels, true),
            vec![Some(290.0), Some(140.0), Some(160.0)]
        );
        assert_eq!(
            reconcile_sizes(&ids, &[210.0, 290.0], &[None, None], &panels, false),
            vec![Some(120.0), Some(140.0), Some(160.0)]
        );
    }

    #[test]
    fn unmeasured_panels_keep_their_initial_preference() {
        assert_eq!(
            reconcile_sizes(
                &[Identity::from("a")],
                &[],
                &[Some(180.0)],
                &[panel("a", 200.0)],
                true
            ),
            vec![Some(180.0)]
        );
    }

    #[test]
    fn divider_keyboard_adjustment_moves_physically_and_clamps() {
        assert_eq!(
            panel_order(Axis::Horizontal, LayoutDirection::RightToLeft, 2),
            [1, 0]
        );
        let horizontal = px(100.0)..px(300.0);
        assert_eq!(
            divider_keyboard_size(Axis::Horizontal, "left", px(150.0), &horizontal,),
            Some(px(140.0))
        );
        assert_eq!(
            divider_keyboard_size(Axis::Horizontal, "right", px(150.0), &horizontal,),
            Some(px(160.0))
        );
        assert_eq!(
            divider_keyboard_size(Axis::Horizontal, "left", px(100.0), &horizontal,),
            Some(px(100.0))
        );
        assert_eq!(
            divider_keyboard_size(Axis::Horizontal, "up", px(150.0), &horizontal,),
            None
        );
        assert_eq!(
            divider_keyboard_size(Axis::Horizontal, "left", px(150.0), &horizontal,),
            Some(px(140.0))
        );
        assert_eq!(
            divider_keyboard_size(Axis::Horizontal, "right", px(150.0), &horizontal,),
            Some(px(160.0))
        );

        let vertical = px(80.0)..px(240.0);
        assert_eq!(
            divider_keyboard_size(Axis::Vertical, "up", px(120.0), &vertical,),
            Some(px(110.0))
        );
        assert_eq!(
            divider_keyboard_size(Axis::Vertical, "down", px(120.0), &vertical,),
            Some(px(130.0))
        );
        assert_eq!(
            divider_keyboard_size(Axis::Vertical, "down", px(240.0), &vertical,),
            Some(px(240.0))
        );
        assert_eq!(
            divider_keyboard_size(Axis::Vertical, "right", px(120.0), &vertical,),
            None
        );
    }

    #[test]
    fn divider_accessibility_reports_current_and_configured_size_bounds() {
        let range = px(80.0)..px(240.0);
        assert_eq!(
            divider_aria_values(Some(px(120.0)), px(80.0), &range),
            (120.0, 80.0, 240.0)
        );
        assert_eq!(
            divider_aria_values(None, px(100.0), &range),
            (100.0, 80.0, 240.0)
        );
    }

    #[test]
    fn divider_range_respects_container_and_sibling_constraints() {
        let ranges = [100.0..300.0, 100.0..300.0];
        assert_eq!(
            divider_effective_range(px(250.0), 0, &ranges, &[px(140.0), px(110.0)], px(140.0)),
            px(100.0)..px(150.0)
        );
        let three_panel_ranges = [200.0..240.0, 100.0..300.0, 70.0..90.0];
        assert_eq!(
            divider_effective_range(
                px(500.0),
                1,
                &three_panel_ranges,
                &[px(200.0), px(200.0), px(100.0)],
                px(200.0)
            ),
            px(210.0)..px(230.0)
        );
        assert_eq!(
            divider_effective_range(
                px(700.0),
                1,
                &[300.0..500.0, 0.0..400.0, 100.0..300.0],
                &[px(300.0), px(200.0), px(200.0)],
                px(200.0)
            ),
            px(100.0)..px(300.0)
        );
        let equal_ranges = [100.0..300.0, 100.0..300.0, 100.0..300.0];
        assert_eq!(
            divider_effective_range(
                px(500.0),
                1,
                &equal_ranges,
                &[px(200.0), px(150.0), px(150.0)],
                px(150.0)
            ),
            px(100.0)..px(200.0)
        );
        assert_eq!(
            divider_effective_range(
                px(500.0),
                0,
                &[0.0..500.0, 100.0..200.0, 100.0..500.0],
                &[px(200.0), px(200.0), px(100.0)],
                px(200.0)
            ),
            px(200.0)..px(300.0)
        );
        assert_eq!(
            divider_effective_range(px(0.0), 0, &ranges, &[px(140.0), px(110.0)], px(140.0)),
            px(100.0)..px(300.0)
        );
        assert_eq!(
            divider_effective_range(px(150.0), 0, &ranges, &[px(140.0), px(110.0)], px(140.0)),
            px(140.0)..px(140.0)
        );
    }

    #[test]
    fn native_placeholders_do_not_replace_initial_preferences_before_prepaint() {
        let ids = [Identity::from("a")];
        let previous = [Some(180.0)];
        let panels = [panel("a", 200.0)];
        let unmeasured = measured_preferences(px(0.0), &[px(100.0)]);
        assert_eq!(
            reconcile_sizes(&ids, &unmeasured, &previous, &panels, true),
            vec![Some(180.0)]
        );

        // A genuine measured 100-point panel must remain a valid preference.
        let measured = measured_preferences(px(400.0), &[px(100.0)]);
        assert_eq!(
            reconcile_sizes(&ids, &measured, &previous, &panels, true),
            vec![Some(100.0)]
        );

        let narrowed = [panel("a", 200.0).size_range(80.0..150.0)];
        assert_eq!(
            reconcile_sizes(&ids, &unmeasured, &previous, &narrowed, true),
            vec![Some(150.0)]
        );
    }

    #[test]
    fn empty_and_single_panel_topologies_have_no_stale_preferences() {
        let ids = [Identity::from("a")];
        assert!(reconcile_sizes(&ids, &[210.0], &[Some(180.0)], &[], true).is_empty());
        let single = [panel("a", 180.0)];
        assert_eq!(
            reconcile_sizes(&[], &[], &[], &single, true),
            vec![Some(180.0)]
        );
        assert!(SplitView::horizontal("empty").panels([]).panels.is_empty());
        assert_eq!(
            SplitView::vertical("single")
                .panel(panel("a", 180.0))
                .panels
                .len(),
            1
        );
    }

    #[test]
    fn changed_constraints_clamp_reconciled_metrics_and_allow_zero_minimum() {
        let ids = [Identity::from("a")];
        let narrowed = [panel("a", 180.0).size_range(80.0..150.0)];
        assert_eq!(
            reconcile_sizes(&ids, &[300.0], &[None], &narrowed, true),
            vec![Some(150.0)]
        );
        assert_eq!(
            reconcile_sizes(&ids, &[20.0], &[None], &narrowed, true),
            vec![Some(80.0)]
        );
        let zero = [panel("a", 180.0).size_range(0.0..150.0)];
        assert_eq!(
            reconcile_sizes(&ids, &[0.0], &[None], &zero, true),
            vec![Some(0.0)]
        );
    }

    #[test]
    fn range_only_change_reconciles_once_and_clamps_measured_sizes() {
        let ids = [Identity::from("a"), Identity::from("b")];
        let old_ranges = [60.0..800.0, 60.0..800.0];
        let unchanged = [
            panel("a", 180.0).size_range(60.0..800.0),
            panel("b", 180.0).size_range(60.0..800.0),
        ];
        let same_ranges = unchanged
            .iter()
            .map(|panel| panel.size_range.clone())
            .collect::<Vec<_>>();
        assert!(!needs_reconciliation(
            &ids,
            &ids,
            &old_ranges,
            &same_ranges,
            Axis::Horizontal,
            Axis::Horizontal,
        ));

        let narrowed = [
            panel("a", 180.0).size_range(60.0..150.0),
            panel("b", 180.0).size_range(60.0..800.0),
        ];
        let new_ranges = narrowed
            .iter()
            .map(|panel| panel.size_range.clone())
            .collect::<Vec<_>>();
        assert!(needs_reconciliation(
            &ids,
            &ids,
            &old_ranges,
            &new_ranges,
            Axis::Horizontal,
            Axis::Horizontal,
        ));
        assert_eq!(
            reconcile_sizes(&ids, &[220.0, 250.0], &[None, None], &narrowed, true),
            vec![Some(150.0), Some(250.0)]
        );
    }

    #[test]
    #[should_panic(expected = "duplicate SplitPanel ID")]
    fn duplicate_panel_ids_are_rejected() {
        let _ = SplitView::horizontal("split")
            .panel(panel("a", 120.0))
            .panel(panel("a", 140.0));
    }

    #[test]
    #[should_panic(expected = "initial size must be finite and positive")]
    fn nonfinite_initial_size_is_rejected() {
        let _ = panel("a", f32::NAN);
    }

    #[test]
    #[should_panic(expected = "size range must be finite")]
    fn descending_range_is_rejected() {
        let _ = panel("a", 120.0).size_range(150.0..100.0);
    }

    #[test]
    #[should_panic(expected = "IDs must be nonempty")]
    fn empty_split_id_is_rejected() {
        let _ = SplitView::horizontal(" ");
    }
}
