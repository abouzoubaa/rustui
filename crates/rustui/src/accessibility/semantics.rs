//! Structural accessibility metadata for a RustUI subtree.

use gpui::{AnyElement, Role, SharedString, accesskit::Live};
use rustui_core::Identity;

use crate::{
    runtime::scoped_element::{SemanticInfo, semantic_scoped_element},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

/// Announcement urgency for a [`SemanticsView`] live-region value.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LivePoliteness {
    Polite,
    Assertive,
}

impl From<LivePoliteness> for Live {
    fn from(value: LivePoliteness) -> Self {
        match value {
            LivePoliteness::Polite => Self::Polite,
            LivePoliteness::Assertive => Self::Assertive,
        }
    }
}

/// Structural roles supported by [`Semantics`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticsRole {
    Group,
    Region,
    Section,
    Navigation,
    Status,
    Alert,
}

impl From<SemanticsRole> for Role {
    fn from(role: SemanticsRole) -> Self {
        match role {
            SemanticsRole::Group => Self::Group,
            SemanticsRole::Region => Self::Region,
            SemanticsRole::Section => Self::Section,
            SemanticsRole::Navigation => Self::Navigation,
            SemanticsRole::Status => Self::Status,
            SemanticsRole::Alert => Self::Alert,
        }
    }
}

#[derive(Clone)]
enum SemanticText {
    Literal(SharedString),
    Localized(SharedString),
}

impl SemanticText {
    fn resolve(&self, context: &RenderContext<'_>) -> String {
        match self {
            Self::Literal(text) => text.to_string(),
            Self::Localized(key) => context.localized_message(key.as_ref()),
        }
    }
}

/// Names and describes a subtree without changing its visual layout.
///
/// The identity must be stable and unique among sibling Semantics views. A
/// label names the group itself; descendant controls retain their own names.
pub struct SemanticsView<V> {
    id: Identity,
    child: V,
    role: Option<SemanticsRole>,
    label: Option<SemanticText>,
    description: Option<SemanticText>,
    live_value: Option<(SemanticText, Live)>,
}

impl<V: View> SemanticsView<V> {
    pub fn new(id: impl Into<Identity>, child: V) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Semantics identity must be nonempty"
        );
        Self {
            id,
            child,
            role: None,
            label: None,
            description: None,
            live_value: None,
        }
    }

    pub fn role(mut self, role: SemanticsRole) -> Self {
        self.role = Some(role);
        self
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(SemanticText::Literal(label.into()));
        self
    }

    pub fn localized_label(mut self, message_key: impl Into<SharedString>) -> Self {
        self.label = Some(SemanticText::Localized(message_key.into()));
        self
    }

    pub fn description(mut self, description: impl Into<SharedString>) -> Self {
        self.description = Some(SemanticText::Literal(description.into()));
        self
    }

    pub fn localized_description(mut self, message_key: impl Into<SharedString>) -> Self {
        self.description = Some(SemanticText::Localized(message_key.into()));
        self
    }

    /// Exposes changing status text as an AccessKit live-region value.
    /// Assigns the Status role unless a later `.role(...)` call overrides it.
    pub fn live_region_value(
        mut self,
        value: impl Into<SharedString>,
        politeness: LivePoliteness,
    ) -> Self {
        self.role = Some(SemanticsRole::Status);
        self.live_value = Some((SemanticText::Literal(value.into()), politeness.into()));
        self
    }

    /// Exposes a localized changing status as an AccessKit live-region value.
    /// Assigns the Status role unless a later `.role(...)` call overrides it.
    pub fn localized_live_region_value(
        mut self,
        message_key: impl Into<SharedString>,
        politeness: LivePoliteness,
    ) -> Self {
        self.role = Some(SemanticsRole::Status);
        self.live_value = Some((
            SemanticText::Localized(message_key.into()),
            politeness.into(),
        ));
        self
    }

    fn semantic_info(&self, context: &RenderContext<'_>) -> Option<SemanticInfo> {
        if self.role.is_none()
            && self.label.is_none()
            && self.description.is_none()
            && self.live_value.is_none()
        {
            return None;
        }
        Some(SemanticInfo {
            role: self.role.unwrap_or(SemanticsRole::Group).into(),
            label: self.label.as_ref().map(|text| text.resolve(context)),
            description: self.description.as_ref().map(|text| text.resolve(context)),
            live_value: self
                .live_value
                .as_ref()
                .map(|(text, politeness)| (text.resolve(context), *politeness)),
        })
    }
}

#[allow(non_snake_case)]
pub fn Semantics<V: View>(id: impl Into<Identity>, child: V) -> SemanticsView<V> {
    SemanticsView::new(id, child)
}

impl<V: View> View for SemanticsView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let scope = RetentionScope::Semantics(self.id.clone());
        let semantics = self.semantic_info(context);
        let child =
            context.with_retention_scope(scope.clone(), |context| self.child.into_element(context));
        semantic_scoped_element(&scope, child, semantics)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::basic::text::Text;

    #[test]
    fn no_metadata_has_no_role() {
        let group = Semantics("group", Text("child"));
        assert!(group.role.is_none());
        assert!(group.label.is_none());
        assert!(group.description.is_none());
    }

    #[test]
    fn structural_roles_map_to_accesskit() {
        assert_eq!(Role::from(SemanticsRole::Group), Role::Group);
        assert_eq!(Role::from(SemanticsRole::Region), Role::Region);
        assert_eq!(Role::from(SemanticsRole::Section), Role::Section);
        assert_eq!(Role::from(SemanticsRole::Navigation), Role::Navigation);
        assert_eq!(Role::from(SemanticsRole::Status), Role::Status);
        assert_eq!(Role::from(SemanticsRole::Alert), Role::Alert);
    }

    #[test]
    fn live_region_builder_sets_value_and_politeness() {
        let status = Semantics("status", Text("child"))
            .live_region_value("3 files saved", LivePoliteness::Polite);
        assert_eq!(status.role, Some(SemanticsRole::Status));
        assert!(matches!(
            status.live_value.as_ref(),
            Some((SemanticText::Literal(value), Live::Polite)) if value.as_ref() == "3 files saved"
        ));

        let localized = Semantics("localized-status", Text("child"))
            .localized_live_region_value("status.saved", LivePoliteness::Assertive);
        assert_eq!(localized.role, Some(SemanticsRole::Status));
        assert!(matches!(
            localized.live_value.as_ref(),
            Some((SemanticText::Localized(key), Live::Assertive)) if key.as_ref() == "status.saved"
        ));

        let explicit_role = Semantics("status-region", Text("child"))
            .live_region_value("3 files saved", LivePoliteness::Polite)
            .role(SemanticsRole::Region);
        assert_eq!(explicit_role.role, Some(SemanticsRole::Region));
    }

    #[test]
    fn builder_replaces_literal_and_localized_fields_independently() {
        let group = Semantics("group", Text("child"))
            .label("Literal label")
            .description("Literal description")
            .localized_label("group.label")
            .localized_description("group.description");
        assert!(matches!(
            group.label.as_ref(),
            Some(SemanticText::Localized(key)) if key.as_ref() == "group.label"
        ));
        assert!(matches!(
            group.description.as_ref(),
            Some(SemanticText::Localized(key)) if key.as_ref() == "group.description"
        ));

        let group = group
            .label("Replacement label")
            .description("Replacement description");
        assert!(matches!(
            group.label.as_ref(),
            Some(SemanticText::Literal(text)) if text.as_ref() == "Replacement label"
        ));
        assert!(matches!(
            group.description.as_ref(),
            Some(SemanticText::Literal(text)) if text.as_ref() == "Replacement description"
        ));
    }

    #[test]
    #[should_panic(expected = "Semantics identity must be nonempty")]
    fn blank_identity_is_rejected() {
        let _ = Semantics("  ", Text("child"));
    }
}
