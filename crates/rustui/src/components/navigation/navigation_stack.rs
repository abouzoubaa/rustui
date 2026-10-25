//! Bound navigation path with heterogeneous RustUI destinations.

use std::collections::HashSet;

use crate::components::basic::button::{Button, ButtonStyle};
use crate::runtime::AppAction;
use gpui::{AnyElement, IntoElement, ParentElement as _, SharedString, Styled as _, div};
use rustui_core::{Binding, Identity};

use crate::view::{AnyView, EmptyView, RenderContext, RetentionScope, View};

/// A stable route and its RustUI destination.
pub struct NavigationDestination {
    id: String,
    content: AnyView,
}

impl NavigationDestination {
    pub fn new<V: View>(id: impl Into<String>, content: V) -> Self {
        let id = id.into();
        assert!(
            !id.trim().is_empty(),
            "RustUI NavigationDestination route ID must be nonempty"
        );
        Self {
            id,
            content: AnyView::new(content),
        }
    }
}

/// A control that appends one route to a navigation path.
pub struct NavigationLinkView {
    label: SharedString,
    accessibility_label: Option<SharedString>,
    route: String,
    path: Binding<Vec<String>>,
    id: Option<Identity>,
    disabled: bool,
}

impl NavigationLinkView {
    pub fn new(
        label: impl Into<SharedString>,
        route: impl Into<String>,
        path: Binding<Vec<String>>,
    ) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Navigation link label must provide a non-empty accessible name"
        );
        let route = route.into();
        assert!(
            !route.trim().is_empty(),
            "RustUI NavigationLink route ID must be nonempty"
        );
        Self {
            label,
            accessibility_label: None,
            route,
            path,
            id: None,
            disabled: false,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "NavigationLink identity must not be blank"
        );
        self.id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the visible label used as this link's accessible name.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "Navigation link accessibility label must not be blank"
        );
        self.accessibility_label = Some(label);
        self
    }

    fn resolved_accessibility_label(&self) -> SharedString {
        self.accessibility_label
            .clone()
            .unwrap_or_else(|| self.label.clone())
    }
}

#[allow(non_snake_case)]
pub fn NavigationLink(
    label: impl Into<SharedString>,
    route: impl Into<String>,
    path: Binding<Vec<String>>,
) -> NavigationLinkView {
    NavigationLinkView::new(label, route, path)
}

impl View for NavigationLinkView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let id = self.id.clone().unwrap_or_else(|| {
            context.next_implicit_retained_identity(Identity::from(format!(
                "navigation-link:{}",
                self.route
            )))
        });
        let route = self.route.clone();
        let update = context.binding_updater(self.path.clone(), move |path| {
            path.push(route.clone());
            true
        });
        Button(self.label.clone(), AppAction::new(move |cx| update(cx)))
            .id(id)
            .accessibility_label(self.resolved_accessibility_label())
            .button_style(ButtonStyle::Bordered)
            .disabled(self.disabled)
            .into_element(context)
    }
}

#[cfg(test)]
mod navigation_link_accessibility_tests {
    use super::{NavigationDestination, NavigationLink, NavigationLinkView};
    use rustui_core::Binding;

    fn path() -> Binding<Vec<String>> {
        Binding::new(
            Vec::new(),
            |path: &Vec<String>| path.clone(),
            |path: &mut Vec<String>, next| *path = next,
        )
    }

    fn link() -> NavigationLinkView {
        NavigationLink("Continue", "details", path())
    }

    #[test]
    fn visible_label_is_the_default_accessible_name() {
        assert_eq!(link().resolved_accessibility_label(), "Continue");
    }

    #[test]
    fn explicit_accessible_name_preserves_nonblank_text() {
        assert_eq!(
            link()
                .accessibility_label("Continue to account settings")
                .resolved_accessibility_label(),
            "Continue to account settings"
        );
    }

    #[test]
    #[should_panic(expected = "Navigation link accessibility label must not be blank")]
    fn blank_accessible_name_is_rejected() {
        link().accessibility_label(" \t ");
    }

    #[test]
    #[should_panic(expected = "Navigation link label must provide a non-empty accessible name")]
    fn blank_visible_label_is_rejected() {
        NavigationLink(" \t ", "details", path());
    }

    #[test]
    #[should_panic(expected = "RustUI NavigationDestination route ID must be nonempty")]
    fn blank_destination_route_is_rejected() {
        NavigationDestination::new(" \t ", super::EmptyView);
    }

    #[test]
    #[should_panic(expected = "RustUI NavigationLink route ID must be nonempty")]
    fn blank_link_route_is_rejected() {
        NavigationLink("Continue", " \t ", path());
    }

    #[test]
    #[should_panic(expected = "NavigationLink identity must not be blank")]
    fn blank_link_identity_is_rejected() {
        link().id(" \t ");
    }

    #[test]
    fn nonblank_link_identity_is_preserved_verbatim() {
        let link = link().id(" continue ");

        assert_eq!(link.id.as_ref().unwrap().as_str(), " continue ");
    }
}

#[cfg(test)]
mod navigation_stack_identity_tests {
    use super::{EmptyView, NavigationDestination, NavigationStackView};
    use rustui_core::{Binding, Identity};

    fn path() -> Binding<Vec<String>> {
        Binding::new(
            Vec::new(),
            |path: &Vec<String>| path.clone(),
            |path: &mut Vec<String>, next| *path = next,
        )
    }

    #[test]
    fn implicit_stack_identity_distinguishes_siblings_and_is_stable_by_slot() {
        let first = NavigationStackView::new(EmptyView, std::iter::empty(), path());
        let second = NavigationStackView::new(EmptyView, std::iter::empty(), path());
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first_sibling = first.identity_with(allocate_at_slot(0));
        let second_sibling = second.identity_with(allocate_at_slot(1));
        let first_after_rerender = first.identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_after_rerender);
    }

    #[test]
    #[should_panic(expected = "RustUI NavigationStack identity must be nonempty")]
    fn blank_stack_identity_is_rejected() {
        NavigationStackView::new(EmptyView, std::iter::empty(), path()).id(" \t ");
    }

    #[test]
    #[should_panic(expected = "every RustUI NavigationDestination must have a unique route ID")]
    fn duplicate_destination_route_ids_are_rejected() {
        NavigationStackView::new(
            EmptyView,
            [
                NavigationDestination::new("details", EmptyView),
                NavigationDestination::new("details", EmptyView),
            ],
            path(),
        );
    }
}

/// A root view and route destinations controlled by a bound path.
pub struct NavigationStackView {
    root: AnyView,
    destinations: Vec<NavigationDestination>,
    path: Binding<Vec<String>>,
    id: Option<Identity>,
}

impl NavigationStackView {
    pub fn new<V: View>(
        root: V,
        destinations: impl IntoIterator<Item = NavigationDestination>,
        path: Binding<Vec<String>>,
    ) -> Self {
        let destinations = destinations.into_iter().collect::<Vec<_>>();
        let mut ids = HashSet::new();
        assert!(
            destinations.iter().all(|item| ids.insert(item.id.as_str())),
            "every RustUI NavigationDestination must have a unique route ID"
        );
        Self {
            root: AnyView::new(root),
            destinations,
            path,
            id: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "RustUI NavigationStack identity must be nonempty"
        );
        self.id = Some(id);
        self
    }

    fn identity_with(&self, allocate: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate(Identity::from("navigation-stack")))
    }
}

#[allow(non_snake_case)]
pub fn NavigationStack<V: View>(
    root: V,
    destinations: impl IntoIterator<Item = NavigationDestination>,
    path: Binding<Vec<String>>,
) -> NavigationStackView {
    NavigationStackView::new(root, destinations, path)
}

impl View for NavigationStackView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let path = self.path.value();
        let id = self.identity_with(|base| context.next_implicit_retained_identity(base));
        context.with_element_scope(RetentionScope::Navigation(id.clone()), |context| {
            if !path.is_empty() {
                context.preserve_retention_scope(RetentionScope::Page(Vec::new()));
            }
            for depth in 1..path.len() {
                if self
                    .destinations
                    .iter()
                    .any(|item| item.id == path[depth - 1])
                {
                    context.preserve_retention_scope(RetentionScope::Page(path[..depth].to_vec()));
                }
            }

            let content = match path.last() {
                None => context.with_element_scope(RetentionScope::Page(Vec::new()), |context| {
                    self.root.into_element(context)
                }),
                Some(route) => match self.destinations.iter().find(|item| &item.id == route) {
                    Some(destination) => context
                        .with_element_scope(RetentionScope::Page(path.clone()), |context| {
                            destination.content.into_element(context)
                        }),
                    None => div()
                        .flex()
                        .flex_col()
                        .child(
                            context.localized_message("rustui-navigation-unavailable-destination"),
                        )
                        .child(route.clone())
                        .into_any_element(),
                },
            };

            let mut root = div().flex().flex_col().gap_3();
            if !path.is_empty() {
                let update =
                    context.binding_updater(self.path.clone(), |path| path.pop().is_some());
                root = root.child(
                    Button(
                        context.localized_message("rustui-navigation-back"),
                        AppAction::new(move |cx| update(cx)),
                    )
                    .id(format!("{}-back", id))
                    .button_style(ButtonStyle::Plain)
                    .into_element(context),
                );
            }
            root.child(content).into_any_element()
        })
    }
}
