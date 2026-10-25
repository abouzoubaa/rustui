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

pub const STORIES: [StoryDefinition; 10] = [
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
}
