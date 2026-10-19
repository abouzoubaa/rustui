//! Structural accessibility metadata for a RustUI subtree.

use gpui::{AnyElement, Role, SharedString};
use rustui_core::Identity;

use crate::{
    runtime::scoped_element::{SemanticInfo, semantic_scoped_element},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

/// Structural roles supported by [`Semantics`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SemanticsRole {
    Group,
    Region,
    Section,
}

impl From<SemanticsRole> for Role {
    fn from(role: SemanticsRole) -> Self {
        match role {
            SemanticsRole::Group => Self::Group,
            SemanticsRole::Region => Self::Region,
            SemanticsRole::Section => Self::Section,
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

    fn semantic_info(&self, context: &RenderContext<'_>) -> Option<SemanticInfo> {
        if self.role.is_none() && self.label.is_none() && self.description.is_none() {
            return None;
        }
        Some(SemanticInfo {
            role: self.role.unwrap_or(SemanticsRole::Group).into(),
            label: self.label.as_ref().map(|text| text.resolve(context)),
            description: self.description.as_ref().map(|text| text.resolve(context)),
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
