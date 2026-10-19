//! Fixed-bounds GPUI cache pilot with independent and props-driven children.

use std::{any::type_name, cell::RefCell, rc::Rc};

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("cache-gallery", CacheGallery::default)
}

#[component]
struct CacheGallery {
    snapshot: Rc<RefCell<String>>,
    #[state(0)]
    props_revision: u32,
    #[state(0)]
    parent_ticks: u32,
    #[state(0)]
    display_revision: u32,
}

impl CacheGallery {
    fn tick_parent(&mut self) {
        self.parent_ticks += 1;
    }

    fn change_props(&mut self) {
        self.props_revision += 1;
    }

    fn refresh_display(&mut self) {
        self.display_revision += 1;
    }
}

impl View for CacheGallery {
    fn body(&self) -> impl View {
        let parent_ticks = self.parent_ticks();
        let props_revision = self.props_revision();
        let display_revision = self.display_revision();
        let snapshot = self.snapshot.clone();

        OwnerActionReader(move |target: OwnerAction<CacheGallery>| {
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
            .then(target.action(CacheGallery::refresh_display));

            VStack::new()
                .spacing(10.0)
                .child(Text("Fixed-bounds cache pilot").font_weight(TextWeight::Semibold))
                .child(Text("Counts are sampled before the display refresh. No full-window refresh is used."))
                .child(
                    HStack::new()
                        .spacing(6.0)
                        .child(Button("Enable diagnostics", AppAction::enable_render_diagnostics()
                            .then(target.action(CacheGallery::refresh_display)))
                            .id("cache-enable-diagnostics"))
                        .child(Button("Clear counters", AppAction::clear_render_diagnostics()
                            .then(target.action(CacheGallery::refresh_display)))
                            .id("cache-clear-diagnostics"))
                        .child(Button("Sample counters", read_snapshot)
                            .id("cache-sample-counters"))
                        .child(Button("Disable diagnostics", AppAction::disable_render_diagnostics()
                            .then(target.action(CacheGallery::refresh_display)))
                            .id("cache-disable-diagnostics")),
                )
                .child(Button("Rerender parent only", target.action(CacheGallery::tick_parent))
                    .id("cache-parent-tick"))
                .child(Text(format!("Parent ticks: {parent_ticks}; display revision: {display_revision}")))
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
                )
                .child(Text(snapshot.borrow().clone()))
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
