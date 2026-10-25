//! Keeps the independent foundation demonstrations in a compact tab group.

use rustui::prelude::*;

#[component]
pub struct FoundationDemos {
    #[state(String::from("owners"))]
    selection: String,
}

impl View for FoundationDemos {
    fn body(&self) -> impl View {
        TabView(
            [
                TabItem::new("owners", "Parents", super::owner_callbacks::demo()),
                TabItem::new("feedback", "Feedback", super::feedback_gallery::demo()),
                TabItem::new("cache", "Cache", super::cache_gallery::demo()),
                TabItem::new("timings", "Timings", super::render_timings::demo()),
                TabItem::new(
                    "supplementary",
                    "Links",
                    super::supplementary_gallery::demo(),
                ),
                TabItem::new(
                    "primitives",
                    "Basics",
                    VStack::new()
                        .spacing(12.0)
                        .child(super::primitive_gallery::demo())
                        .child(super::group_gallery::demo())
                        .child(super::conditional_gallery::demo())
                        .child(super::property_inspector_gallery::demo()),
                ),
                TabItem::new("canvas", "Canvas", super::canvas_gallery::demo()),
                TabItem::new(
                    "grid",
                    "Grid",
                    VStack::new()
                        .spacing(12.0)
                        .child(super::grid_overlays::demo())
                        .child(super::frame_gallery::demo())
                        .child(super::geometry_gallery::demo())
                        .child(super::scroll_gallery::demo()),
                ),
                TabItem::new("lazy-grid", "Lazy Grid", super::lazy_grid_gallery::demo()),
                TabItem::new(
                    "focus",
                    "Focus",
                    VStack::new()
                        .spacing(12.0)
                        .child(super::focus_gallery::demo())
                        .child(super::keyboard_gallery::demo())
                        .child(super::hover_gallery::demo())
                        .child(super::drag_drop_gallery::demo())
                        .child(super::tap_gallery::demo()),
                ),
                TabItem::new("tasks", "Tasks", super::owner_tasks::demo()),
                TabItem::new("release", "Release", super::task_release::demo()),
                TabItem::new(
                    "theme",
                    "Theme",
                    VStack::new()
                        .spacing(12.0)
                        .child(super::decoration_gallery::demo())
                        .child(super::shadow_gallery::demo())
                        .child(super::card_gallery::demo())
                        .child(super::foundation_additions::foundation_additions()),
                ),
                TabItem::new("split", "Split", super::split_view_demo()),
            ],
            self.selection_binding(),
        )
        .id("foundation-demos")
        .style(TabViewStyle::Segmented)
    }
}
