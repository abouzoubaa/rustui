//! Verification status shown without converting compilation into GUI acceptance.

use super::component_catalog::StoryId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    InteractionAndVisualPending,
    SomeNativeChecksRecorded,
    BoundedNativeChecksRecorded,
}

impl VerificationStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::InteractionAndVisualPending => {
                "Implementation present · interaction and visual verification pending"
            }
            Self::SomeNativeChecksRecorded => {
                "Some bounded native checks recorded · broader story acceptance remains pending"
            }
            Self::BoundedNativeChecksRecorded => {
                "Bounded native checks recorded · broader visual, accessibility, and platform acceptance remains pending"
            }
        }
    }
}

pub const fn status_for(story: StoryId) -> VerificationStatus {
    match story {
        StoryId::Button | StoryId::Input => VerificationStatus::SomeNativeChecksRecorded,
        StoryId::Layout | StoryId::List | StoryId::Table | StoryId::Tree | StoryId::Navigation => {
            VerificationStatus::BoundedNativeChecksRecorded
        }
        StoryId::Text
        | StoryId::Card
        | StoryId::Tabs
        | StoryId::Breadcrumb
        | StoryId::Outline
        | StoryId::GraphView
        | StoryId::NodeEditor
        | StoryId::SplitView
        | StoryId::Drawer => VerificationStatus::InteractionAndVisualPending,
    }
}

#[cfg(test)]
mod tests {
    use super::{VerificationStatus, status_for};
    use crate::stories::component_catalog::StoryId;

    #[test]
    fn status_reflects_story_scoped_native_evidence_without_claiming_full_acceptance() {
        for story in [StoryId::Button, StoryId::Input] {
            assert_eq!(
                status_for(story),
                VerificationStatus::SomeNativeChecksRecorded
            );
        }
        for story in [
            StoryId::Layout,
            StoryId::List,
            StoryId::Table,
            StoryId::Tree,
            StoryId::Navigation,
        ] {
            assert_eq!(
                status_for(story),
                VerificationStatus::BoundedNativeChecksRecorded
            );
            assert!(status_for(story).label().contains("remains pending"));
        }
        for story in [
            StoryId::Text,
            StoryId::Card,
            StoryId::Tabs,
            StoryId::Breadcrumb,
            StoryId::Outline,
            StoryId::GraphView,
            StoryId::NodeEditor,
        ] {
            assert_eq!(
                status_for(story),
                VerificationStatus::InteractionAndVisualPending
            );
        }
    }
}
