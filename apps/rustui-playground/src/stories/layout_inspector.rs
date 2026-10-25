//! Readable layout facts for the selected story.

use super::component_catalog::StoryId;

pub fn summary_for(story: StoryId) -> &'static str {
    match story {
        StoryId::Button => "Layout: VStack containing HStack button groups",
        StoryId::Text => "Layout: VStack of GPUI-backed text elements",
        StoryId::Layout => "Layout: nested VStack and HStack flex containers",
        StoryId::Input => "Layout: retained GPUI input entities inside a RustUI VStack",
        StoryId::List => "Layout: fixed-height GPUI virtual list inside RustUI VStack",
        StoryId::Table => "Layout: fixed-height GPUI virtual table inside RustUI VStack",
        StoryId::Outline => "Layout: named outline group containing a retained GPUI virtual tree",
        StoryId::Drawer => {
            "Layout: window-hosted modal sheet with an owner-controlled physical edge"
        }
        StoryId::Tree => "Layout: fixed-height GPUI virtual tree inside RustUI VStack",
        StoryId::Tabs => "Layout: GPUI tab bar above the selected RustUI child",
        StoryId::Breadcrumb => "Layout: ordered GPUI breadcrumb items with separators",
        StoryId::Navigation => "Layout: back control above the selected route destination",
    }
}
