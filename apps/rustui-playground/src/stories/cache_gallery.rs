//! Fixed-bounds GPUI cache pilot with independent and props-driven children.

use std::{any::type_name, cell::RefCell, rc::Rc};

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("cache-gallery", CacheGallery::default)
}

#[component]
struct CacheGallery {
    #[state(0)]
    props_revision: u32,
    #[state(0)]
    parent_ticks: u32,
}

impl CacheGallery {
    fn tick_parent(&mut self) {
        self.parent_ticks += 1;
    }

    fn change_props(&mut self) {
        self.props_revision += 1;
    }
}

impl View for CacheGallery {
    fn body(&self) -> impl View {
        let parent_ticks = self.parent_ticks();
        let props_revision = self.props_revision();
        OwnerActionReader(move |target: OwnerAction<CacheGallery>| {
            KeyboardShortcuts(
                "cache-gallery-shortcuts",
                VStack::new()
                    .spacing(10.0)
                    .child(Text("Fixed-bounds cache pilot").font_weight(TextWeight::Semibold))
                    .child(Component("cache-diagnostics", CacheDiagnostics::default))
                    .child(Button("Rerender parent only", target.action(CacheGallery::tick_parent))
                        .id("cache-parent-tick"))
                    .child(Text(format!("Parent ticks: {parent_ticks}")))
                    .child(
                        HStack::new()
                            .spacing(12.0)
                            .child(Component("cached-leaf", CachedLeaf::default).cached_fixed(240.0, 130.0))
                            .child(Component("uncached-leaf", UncachedLeaf::default)),
                    )
                    .child(Button("Change cached child props", target.action(CacheGallery::change_props))
                        .id("cache-change-props"))
                    .child(Text("Parent-only rerenders keep incoming props unchanged. Changing props updates the retained child; its local count is preserved."))
                    .child(
                        Component("cached-props-leaf", CachedPropsLeaf::default)
                            .props(props_revision, CachedPropsLeaf::apply_revision)
                            .cached_fixed(320.0, 160.0),
                    ),
            )
            .shortcut("secondary-shift-p", target.action(CacheGallery::tick_parent))
        })
    }
}

#[component]
struct CacheDiagnostics {
    snapshot: Rc<RefCell<String>>,
    accessibility_activity: Rc<RefCell<String>>,
    #[state(0)]
    display_revision: u32,
}

impl CacheDiagnostics {
    fn refresh(&mut self) {
        self.display_revision += 1;
    }
}

impl View for CacheDiagnostics {
    fn body(&self) -> impl View {
        let _display_revision = self.display_revision();
        let snapshot = self.snapshot.clone();
        let accessibility_activity = self.accessibility_activity.clone();

        OwnerActionReader(move |target: OwnerAction<CacheDiagnostics>| {
            let snapshot_for_action = snapshot.clone();
            let read_snapshot = AppAction::read_render_diagnostics_snapshot(move |observed| {
                let count = |component_type: &str| {
                    observed
                        .hosts
                        .iter()
                        .find(|host| host.component_type == component_type)
                        .map_or(0, |host| host.count)
                };
                *snapshot_for_action.borrow_mut() = format!(
                    "CachedLeaf: {}; UncachedLeaf: {}; CachedPropsLeaf: {}; CacheGallery: {}",
                    count(type_name::<CachedLeaf>()),
                    count(type_name::<UncachedLeaf>()),
                    count(type_name::<CachedPropsLeaf>()),
                    count(type_name::<CacheGallery>())
                );
            })
            .then(target.action(CacheDiagnostics::refresh));
            let accessibility_for_action = accessibility_activity.clone();
            let read_accessibility_activity = AppAction::read_accessibility_active(move |active| {
                *accessibility_for_action.borrow_mut() =
                    format!("GPUI accessibility active: {active}");
            })
            .then(target.action(CacheDiagnostics::refresh));

            KeyboardShortcuts(
                "cache-diagnostics-shortcuts",
                VStack::new()
                    .spacing(8.0)
                    .child(Text("Counts are sampled before their display update; the cache parent is not refreshed."))
                    .child(Text("Focus Enable diagnostics once, then keep focus there: Secondary-Shift-C clears counters, Secondary-Shift-P ticks the gallery parent, and Secondary-Shift-S samples counters. Use Secondary-Shift-A to sample accessibility separately."))
                    .child(
                        HStack::new()
                            .spacing(6.0)
                            .child(Button("Enable diagnostics", AppAction::enable_render_diagnostics())
                                .id("cache-enable-diagnostics"))
                            .child(Button("Clear counters", AppAction::clear_render_diagnostics())
                                .id("cache-clear-diagnostics"))
                            .child(Button("Sample counters", read_snapshot.clone())
                                .id("cache-sample-counters"))
                            .child(Button("Sample accessibility state", read_accessibility_activity.clone())
                                .id("cache-sample-accessibility"))
                            .child(Button("Disable diagnostics", AppAction::disable_render_diagnostics())
                                .id("cache-disable-diagnostics")),
                    )
                    .child(Text(if snapshot.borrow().is_empty() {
                        "Not sampled yet".to_owned()
                    } else {
                        snapshot.borrow().clone()
                    }))
                    .child(Text(if accessibility_activity.borrow().is_empty() {
                        "Not sampled yet".to_owned()
                    } else {
                        accessibility_activity.borrow().clone()
                    })),
            )
            .shortcut("secondary-shift-c", AppAction::clear_render_diagnostics())
            .shortcut("secondary-shift-s", read_snapshot)
            .shortcut("secondary-shift-a", read_accessibility_activity)
        })
    }
}

#[component]
struct CachedLeaf {
    #[state(0)]
    count: u32,
}

impl CachedLeaf {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for CachedLeaf {
    fn body(&self) -> impl View {
        Container(
            VStack::new()
                .spacing(8.0)
                .child(Text("Cached child"))
                .child(Text(format!("Local count: {}", self.count())))
                .child(
                    Button("Increment cached child", self.action(Self::increment))
                        .id("cache-increment-cached-child"),
                ),
        )
        .width(240.0)
        .height(130.0)
        .padding(8.0)
        .border()
    }
}

#[component]
struct UncachedLeaf {
    #[state(0)]
    count: u32,
}

impl UncachedLeaf {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for UncachedLeaf {
    fn body(&self) -> impl View {
        Container(
            VStack::new()
                .spacing(8.0)
                .child(Text("Uncached child"))
                .child(Text(format!("Local count: {}", self.count())))
                .child(
                    Button("Increment uncached child", self.action(Self::increment))
                        .id("cache-increment-uncached-child"),
                ),
        )
        .width(240.0)
        .height(130.0)
        .padding(8.0)
        .border()
    }
}

#[component]
struct CachedPropsLeaf {
    revision: u32,
    #[state(0)]
    count: u32,
}

impl CachedPropsLeaf {
    fn apply_revision(&mut self, revision: &u32) {
        self.revision = *revision;
    }

    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for CachedPropsLeaf {
    fn body(&self) -> impl View {
        Container(
            VStack::new()
                .spacing(8.0)
                .child(Text("Cached child with props"))
                .child(Text(format!("Incoming revision: {}", self.revision)))
                .child(Text(format!("Local count: {}", self.count())))
                .child(
                    Button("Increment props child", self.action(Self::increment))
                        .id("cache-increment-props-child"),
                ),
        )
        .width(320.0)
        .height(160.0)
        .padding(8.0)
        .border()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_only_tick_preserves_child_props_revision() {
        let mut gallery = CacheGallery::default();

        gallery.tick_parent();

        assert_eq!(gallery.parent_ticks(), 1);
        assert_eq!(gallery.props_revision(), 0);
    }
}
