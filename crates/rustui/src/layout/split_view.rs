//! Keyed, owner-retained composition using official GPUI Components resizing.

use std::ops::Range;

use gpui::{
    AnyElement, AppContext as _, Axis, Entity, IntoElement, ParentElement as _, Pixels,
    Styled as _, div, px,
};
use gpui_component::resizable::{ResizableState, h_resizable, resizable_panel, v_resizable};
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, RetentionScope, View};

type ChildRenderer = Box<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>;

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

// Upstream native sizes are positional. Topology and range reconciliation map
// size preferences by ID; official GPUI still owns drag, fitting, and clamping.
fn reconcile_sizes(
    old_ids: &[Identity],
    measured: &[f32],
    previous: &[Option<f32>],
    panels: &[SplitPanel],
    same_axis: bool,
) -> Vec<Option<f32>> {
    panels
        .iter()
        .map(|panel| {
            if same_axis {
                if let Some(index) = old_ids.iter().position(|id| *id == panel.id) {
                    return measured
                        .get(index)
                        .copied()
                        .filter(|size| size.is_finite() && *size >= 0.0)
                        .or_else(|| previous.get(index).copied().flatten())
                        .or(panel.initial_size);
                }
            }
            panel.initial_size
        })
        .zip(panels)
        .map(|(size, panel)| {
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
            let ids: Vec<_> = self.panels.iter().map(|panel| panel.id.clone()).collect();
            let size_ranges: Vec<_> = self
                .panels
                .iter()
                .map(|panel| panel.size_range.clone())
                .collect();
            let (retained, _) =
                context.retained_entity(Identity::from("split-state"), |_, cx| RetainedSplit {
                    axis: self.axis,
                    ids: ids.clone(),
                    size_ranges: size_ranges.clone(),
                    initial_sizes: self.panels.iter().map(|panel| panel.initial_size).collect(),
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
                    &self.panels,
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
            let mut group = match self.axis {
                Axis::Horizontal => h_resizable("rustui-split"),
                Axis::Vertical => v_resizable("rustui-split"),
            }
            .with_state(&native);
            for (index, panel) in self.panels.iter().enumerate() {
                let content = context
                    .with_element_scope(RetentionScope::Key(panel.id.clone()), |context| {
                        (panel.content)(context)
                    });
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

    fn panel(id: &str, size: f32) -> SplitPanel {
        SplitPanel::new(id, EmptyView).initial_size(size)
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
