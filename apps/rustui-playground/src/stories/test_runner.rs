//! Verification status shown without converting compilation into GUI acceptance.

use super::component_catalog::StoryId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VerificationStatus {
    InteractionAndVisualPending,
}

impl VerificationStatus {
    pub const fn label(self) -> &'static str {
        match self {
            Self::InteractionAndVisualPending => {
                "Implementation present · interaction and visual verification pending"
            }
        }
    }
}

pub const fn status_for(_story: StoryId) -> VerificationStatus {
    VerificationStatus::InteractionAndVisualPending
}
