//! Canonical registry for stories currently reachable in RustUI Playground.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum StoryId {
    #[default]
    Button,
    Text,
    Layout,
    Input,
    List,
    Table,
    Tree,
    Drawer,
    GraphView,
    NodeEditor,
    SplitView,
    Outline,
    Tabs,
    Breadcrumb,
    Navigation,
}

impl StoryId {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Button => "Button",
            Self::Text => "Text",
            Self::Layout => "VStack + HStack",
            Self::Input => "Text inputs",
            Self::List => "List",
            Self::Table => "Table",
            Self::Tree => "Tree",
            Self::Drawer => "Drawer",
            Self::GraphView => "GraphView",
            Self::NodeEditor => "NodeEditor",
            Self::SplitView => "SplitView + ResizablePanel",
            Self::Outline => "Outline",
            Self::Tabs => "Tabs",
            Self::Breadcrumb => "Breadcrumb",
            Self::Navigation => "NavigationStack",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct StoryDefinition {
    pub id: StoryId,
    pub title: &'static str,
    pub summary: &'static str,
}

pub const STORIES: [StoryDefinition; 15] = [
    StoryDefinition {
        id: StoryId::Button,
        title: "Button",
        summary: "Actions, variants, disabled state, identity, and invalidation.",
    },
    StoryDefinition {
        id: StoryId::Text,
        title: "Text",
        summary: "String content lowered through RustUI into GPUI text.",
    },
    StoryDefinition {
        id: StoryId::Layout,
        title: "Stacks",
        summary: "Change child titles from the parent while counters and notes stay with keyed children.",
    },
    StoryDefinition {
        id: StoryId::Input,
        title: "Text inputs",
        summary: "Retained editing state synchronized through a RustUI binding.",
    },
    StoryDefinition {
        id: StoryId::List,
        title: "List",
        summary: "Virtualized, searchable rows with controlled source-index selection.",
    },
    StoryDefinition {
        id: StoryId::Table,
        title: "Table",
        summary: "Virtualized rows, controlled selection, and named table accessibility semantics.",
    },
    StoryDefinition {
        id: StoryId::Tree,
        title: "Tree",
        summary: "Virtualized hierarchy with expansion and controlled stable-ID selection.",
    },
    StoryDefinition {
        id: StoryId::Tabs,
        title: "Tabs",
        summary: "Switch editors, remove or restore Details, and show a conditional editor.",
    },
    StoryDefinition {
        id: StoryId::Breadcrumb,
        title: "Breadcrumb",
        summary: "Ordered navigation actions rendered by GPUI Components.",
    },
    StoryDefinition {
        id: StoryId::Navigation,
        title: "NavigationStack",
        summary: "Edit root and destinations, navigate back, and toggle a root editor.",
    },
    StoryDefinition {
        id: StoryId::Outline,
        title: "Outline",
        summary: "Document hierarchy with owner-controlled selected and expanded stable IDs.",
    },
    StoryDefinition {
        id: StoryId::Drawer,
        title: "Drawer",
        summary: "Controlled modal drawer with physical edges, owner refresh, and focus restoration.",
    },
    StoryDefinition {
        id: StoryId::GraphView,
        title: "GraphView",
        summary: "Positioned graph with stable nodes, named ports and connections, controlled viewport and keyboard activation.",
    },
    StoryDefinition {
        id: StoryId::NodeEditor,
        title: "NodeEditor",
        summary: "Controlled graph editing with typed ports, node dragging, keyboard movement and connection controls.",
    },
    StoryDefinition {
        id: StoryId::SplitView,
        title: "SplitView + ResizablePanel",
        summary: "ResizablePanel panes with stable child identity, divider dragging, axis changes, reorder, insertion, removal and size limits.",
    },
];

pub fn filtered_story_indices(query: &str) -> Vec<usize> {
    let query = query.trim().to_ascii_lowercase();
    STORIES
        .iter()
        .enumerate()
        .filter(|(_, story)| {
            query.is_empty()
                || story.id.label().to_ascii_lowercase().contains(&query)
                || story.title.to_ascii_lowercase().contains(&query)
                || story.summary.to_ascii_lowercase().contains(&query)
        })
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::filtered_story_indices;

    #[test]
    fn story_search_trims_query_and_ignores_ascii_case() {
        assert_eq!(filtered_story_indices("  nAvIgAtIoNsTaCk  "), vec![9]);
        assert_eq!(filtered_story_indices("  CONDITIONAL EDITOR "), vec![7]);
    }

    #[test]
    fn story_search_returns_no_indices_for_no_match() {
        assert!(filtered_story_indices("no such framework demo").is_empty());
    }

    #[test]
    fn story_search_preserves_original_registry_indices() {
        assert_eq!(filtered_story_indices("selection"), vec![4, 5, 6]);
    }

    #[test]
    fn story_search_matches_the_visible_story_label() {
        assert_eq!(filtered_story_indices("HStack"), vec![2]);
    }
    #[test]
    fn graph_stories_are_individually_searchable_and_reachable() {
        for (query, id) in [
            (" GraphView ", super::StoryId::GraphView),
            (" nOdEeDiToR ", super::StoryId::NodeEditor),
        ] {
            let index = super::STORIES
                .iter()
                .position(|story| story.id == id)
                .expect("standalone graph story");
            assert_eq!(filtered_story_indices(query), vec![index]);
            assert_eq!(
                crate::story_for(if id == super::StoryId::GraphView {
                    "graph_view"
                } else {
                    "node_editor"
                }),
                Some(id)
            );
            assert!(
                !crate::stories::code_preview::source_for(id)
                    .trim()
                    .is_empty()
            );
            assert_eq!(
                crate::stories::test_runner::status_for(id),
                crate::stories::test_runner::VerificationStatus::InteractionAndVisualPending
            );
        }
    }

    #[test]
    fn resizable_panels_are_directly_searchable_with_the_existing_split_gallery() {
        let index = super::STORIES
            .iter()
            .position(|story| story.id == super::StoryId::SplitView)
            .expect("standalone split story");
        for query in [" SplitView ", " rEsIzAbLePaNeL "] {
            assert_eq!(filtered_story_indices(query), vec![index]);
        }
        let source = crate::stories::code_preview::source_for(super::StoryId::SplitView);
        assert!(source.contains("SplitView::horizontal"));
        assert!(source.contains("ResizablePanel::new"));
    }
}
