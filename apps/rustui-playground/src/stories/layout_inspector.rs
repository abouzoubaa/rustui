//! Catalog keys for layout facts shown in the selected story.

use super::component_catalog::StoryId;

pub fn summary_key_for(story: StoryId) -> &'static str {
    match story {
        StoryId::Button => "lab-layout-button",
        StoryId::Text => "lab-layout-text",
        StoryId::Card => "lab-layout-card",
        StoryId::Layout => "lab-layout-layout",
        StoryId::Input => "lab-layout-input",
        StoryId::List => "lab-layout-list",
        StoryId::Table => "lab-layout-table",
        StoryId::Outline => "lab-layout-outline",
        StoryId::GraphView => "lab-layout-graph-view",
        StoryId::NodeEditor => "lab-layout-node-editor",
        StoryId::SplitView => "lab-layout-split-view",
        StoryId::Drawer => "lab-layout-drawer",
        StoryId::Tree => "lab-layout-tree",
        StoryId::Tabs => "lab-layout-tabs",
        StoryId::Breadcrumb => "lab-layout-breadcrumb",
        StoryId::Navigation => "lab-layout-navigation",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_story_summary_exists_in_each_runtime_catalog() {
        let catalogs = super::super::locale_selector::catalogs();
        for language in ["en", "fr", "ar"] {
            let locale = rustui::localization::Locale::parse(language).unwrap();
            let catalog = catalogs.get(&locale).unwrap();
            for story in [
                StoryId::Button,
                StoryId::Text,
                StoryId::Card,
                StoryId::Layout,
                StoryId::Input,
                StoryId::List,
                StoryId::Table,
                StoryId::Outline,
                StoryId::GraphView,
                StoryId::NodeEditor,
                StoryId::SplitView,
                StoryId::Drawer,
                StoryId::Tree,
                StoryId::Tabs,
                StoryId::Breadcrumb,
                StoryId::Navigation,
            ] {
                assert!(
                    catalog.get(summary_key_for(story)).is_some(),
                    "{language}: {story:?}"
                );
            }
        }
    }
}
