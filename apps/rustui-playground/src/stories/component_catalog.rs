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
        summary: "Virtualized string cells with controlled row selection.",
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
