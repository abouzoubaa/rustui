//! Opt-in measurements of RustUI component lowering inside GPUI hosts.

use std::{any::TypeId, cell::RefCell, collections::HashMap, time::Duration};

use gpui::{App, Global};

/// Aggregate lowering measurements for one concrete component type.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HostLoweringStat {
    pub component_type: &'static str,
    pub count: u64,
    pub total: Duration,
    pub max: Duration,
}

/// A point-in-time copy of recorded host lowering measurements.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RenderDiagnosticsSnapshot {
    pub hosts: Vec<HostLoweringStat>,
}

#[derive(Default)]
pub(crate) struct RenderDiagnostics {
    hosts: RefCell<HashMap<TypeId, HostLoweringStat>>,
}

impl Global for RenderDiagnostics {}

impl RenderDiagnostics {
    fn record(&self, component_id: TypeId, component_type: &'static str, elapsed: Duration) {
        let mut hosts = self.hosts.borrow_mut();
        let stat = hosts.entry(component_id).or_insert(HostLoweringStat {
            component_type,
            count: 0,
            total: Duration::ZERO,
            max: Duration::ZERO,
        });
        stat.count = stat.count.saturating_add(1);
        stat.total = stat.total.saturating_add(elapsed);
        stat.max = stat.max.max(elapsed);
    }

    fn snapshot(&self) -> RenderDiagnosticsSnapshot {
        let mut hosts: Vec<_> = self.hosts.borrow().values().cloned().collect();
        hosts.sort_unstable_by_key(|stat| stat.component_type);
        RenderDiagnosticsSnapshot { hosts }
    }

    fn clear(&self) {
        self.hosts.borrow_mut().clear();
    }
}

/// Starts a new diagnostic session with empty counters.
pub fn enable_render_diagnostics(cx: &mut App) {
    cx.set_global(RenderDiagnostics::default());
}

/// Stops recording and discards the current counters.
pub fn disable_render_diagnostics(cx: &mut App) {
    if cx.has_global::<RenderDiagnostics>() {
        cx.remove_global::<RenderDiagnostics>();
    }
}

/// Returns whether host lowering measurements are enabled.
pub fn render_diagnostics_enabled(cx: &App) -> bool {
    cx.has_global::<RenderDiagnostics>()
}

/// Returns a copy of the current aggregate, or an empty snapshot when disabled.
pub fn render_diagnostics_snapshot(cx: &App) -> RenderDiagnosticsSnapshot {
    cx.try_global::<RenderDiagnostics>()
        .map(RenderDiagnostics::snapshot)
        .unwrap_or_default()
}

/// Resets the current aggregate without ending the diagnostic session.
pub fn clear_render_diagnostics(cx: &mut App) {
    if let Some(diagnostics) = cx.try_global::<RenderDiagnostics>() {
        diagnostics.clear();
    }
}

pub(crate) fn record_host_lowering(
    cx: &App,
    component_id: TypeId,
    component_type: &'static str,
    elapsed: Duration,
) {
    if let Some(diagnostics) = cx.try_global::<RenderDiagnostics>() {
        diagnostics.record(component_id, component_type, elapsed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Alpha;
    struct Beta;

    #[test]
    fn aggregates_by_type_and_resets() {
        let diagnostics = RenderDiagnostics::default();
        diagnostics.record(TypeId::of::<Beta>(), "Beta", Duration::from_millis(4));
        diagnostics.record(TypeId::of::<Alpha>(), "Alpha", Duration::from_millis(2));
        diagnostics.record(TypeId::of::<Alpha>(), "Alpha", Duration::from_millis(5));

        let snapshot = diagnostics.snapshot();
        assert_eq!(snapshot.hosts.len(), 2);
        assert_eq!(snapshot.hosts[0].component_type, "Alpha");
        assert_eq!(snapshot.hosts[0].count, 2);
        assert_eq!(snapshot.hosts[0].total, Duration::from_millis(7));
        assert_eq!(snapshot.hosts[0].max, Duration::from_millis(5));
        assert_eq!(snapshot.hosts[1].component_type, "Beta");

        diagnostics.clear();
        assert!(diagnostics.snapshot().hosts.is_empty());
    }
}
