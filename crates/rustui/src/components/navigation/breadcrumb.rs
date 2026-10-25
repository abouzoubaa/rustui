//! RustUI actions presented through GPUI Components breadcrumbs.

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, div,
};
use gpui_component::breadcrumb::{
    Breadcrumb as GpuiBreadcrumb, BreadcrumbItem as GpuiBreadcrumbItem,
};
use rustui_core::Identity;

use crate::view::{ControlAction, EmptyView, RenderContext, View};

/// One breadcrumb label with an optional navigation action.
pub struct BreadcrumbItem {
    label: SharedString,
    action: Option<ControlAction>,
    disabled: bool,
}

impl BreadcrumbItem {
    pub fn new(label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Breadcrumb label must not be blank"
        );
        Self {
            label,
            action: None,
            disabled: false,
        }
    }

    pub fn action(mut self, action: impl Into<ControlAction>) -> Self {
        self.action = Some(action.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// An ordered breadcrumb path.
pub struct BreadcrumbView {
    items: Vec<BreadcrumbItem>,
    id: Option<Identity>,
    accessibility_label: Option<SharedString>,
}

impl BreadcrumbView {
    pub fn new(items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            id: None,
            accessibility_label: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Breadcrumb identity must not be blank"
        );
        self.id = Some(id);
        self
    }

    /// Names the breadcrumb navigation landmark for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "breadcrumb accessibility label must not be empty"
        );
        self.accessibility_label = Some(label);
        self
    }

    fn identity(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate_implicit(Identity::from("breadcrumb")))
    }

    fn has_navigation_content(&self) -> bool {
        !self.items.is_empty()
    }
}

fn navigable_action(
    item: &BreadcrumbItem,
    index: usize,
    item_count: usize,
) -> Option<&ControlAction> {
    (index + 1 < item_count)
        .then_some(item.action.as_ref())
        .flatten()
}

#[allow(non_snake_case)]
pub fn Breadcrumb(items: impl IntoIterator<Item = BreadcrumbItem>) -> BreadcrumbView {
    BreadcrumbView::new(items)
}

impl View for BreadcrumbView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        if !self.has_navigation_content() {
            return EmptyView.into_element(context);
        }

        let mut breadcrumb = GpuiBreadcrumb::new();
        for (index, item) in self.items.iter().enumerate() {
            let mut upstream = GpuiBreadcrumbItem::new(item.label.clone()).disabled(item.disabled);
            if let Some(action) = navigable_action(item, index, self.items.len()) {
                upstream = upstream.on_click(context.bind_control_action(action.clone()));
            }
            breadcrumb = breadcrumb.child(upstream);
        }

        let identity = self.identity(|base| context.next_implicit_retained_identity(base));
        let mut navigation = div().id(identity.to_string()).role(Role::Navigation);
        if let Some(label) = &self.accessibility_label {
            navigation = navigation.aria_label(label.clone());
        }
        navigation.child(breadcrumb).into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn final_breadcrumb_does_not_keep_a_navigation_action() {
        use crate::runtime::AppAction;

        let first = BreadcrumbItem::new("Home").action(AppAction::with_window_action(|_, _, _| {}));
        let current =
            BreadcrumbItem::new("Details").action(AppAction::with_window_action(|_, _, _| {}));
        let items = [first, current];

        assert!(navigable_action(&items[0], 0, items.len()).is_some());
        assert!(navigable_action(&items[1], 1, items.len()).is_none());
    }

    #[test]
    fn breadcrumb_accessibility_name_is_optional_and_configurable() {
        let unnamed = Breadcrumb(std::iter::empty());
        assert_eq!(unnamed.accessibility_label, None);
        assert_eq!(
            Breadcrumb(std::iter::empty())
                .accessibility_label("Workspace path")
                .accessibility_label
                .as_deref(),
            Some("Workspace path")
        );
    }

    #[test]
    fn empty_breadcrumb_has_no_navigation_landmark_content() {
        assert!(!Breadcrumb(std::iter::empty()).has_navigation_content());
        assert!(Breadcrumb([BreadcrumbItem::new("Home")]).has_navigation_content());
    }

    #[test]
    #[should_panic(expected = "breadcrumb accessibility label must not be empty")]
    fn rejects_blank_accessibility_name() {
        Breadcrumb(std::iter::empty()).accessibility_label(" \t ");
    }

    #[test]
    fn preserves_explicit_identity_without_allocating_a_fallback() {
        let expected = Identity::from(" caller breadcrumb ");
        let breadcrumb = Breadcrumb(std::iter::empty()).id(expected.clone());

        assert_eq!(
            breadcrumb.identity(|_| panic!("explicit identity must bypass fallback allocation")),
            expected
        );
    }

    #[test]
    #[should_panic(expected = "Breadcrumb identity must not be blank")]
    fn rejects_blank_explicit_identity() {
        Breadcrumb(std::iter::empty()).id(" \t ");
    }

    #[test]
    fn fallback_identity_uses_the_breadcrumb_base_and_owner_sibling_slot() {
        fn allocate(owner: &str, slot: &mut usize, base: Identity) -> Identity {
            let current = *slot;
            *slot += 1;
            Identity::from(format!("{owner}:implicit-retained:{current}:{base}"))
        }

        let mut slot = 0;
        let first =
            Breadcrumb([BreadcrumbItem::new("Home"), BreadcrumbItem::new("A")]).identity(|base| {
                assert_eq!(base, Identity::from("breadcrumb"));
                allocate("owner", &mut slot, base)
            });
        let second =
            Breadcrumb([BreadcrumbItem::new("Home"), BreadcrumbItem::new("B")]).identity(|base| {
                assert_eq!(base, Identity::from("breadcrumb"));
                allocate("owner", &mut slot, base)
            });

        assert_eq!(
            first,
            Identity::from("owner:implicit-retained:0:breadcrumb")
        );
        assert_eq!(
            second,
            Identity::from("owner:implicit-retained:1:breadcrumb")
        );
        assert_ne!(first, second);

        let mut rerender_slot = 0;
        let rerendered_first = Breadcrumb([BreadcrumbItem::new("Home"), BreadcrumbItem::new("A")])
            .identity(|base| allocate("owner", &mut rerender_slot, base));
        assert_eq!(rerendered_first, first);
    }
}
