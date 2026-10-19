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
        Self {
            id: id.into(),
            content: AnyView::new(content),
        }
    }
}

/// A control that appends one route to a navigation path.
pub struct NavigationLinkView {
    label: SharedString,
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
        Self {
            label: label.into(),
            route: route.into(),
            path,
            id: None,
            disabled: false,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
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
        let id = self
            .id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("navigation-link:{}", self.route)));
        let route = self.route.clone();
        let update = context.binding_updater(self.path.clone(), move |path| {
            path.push(route.clone());
            true
        });
        Button(self.label.clone(), AppAction::new(move |cx| update(cx)))
            .id(id)
            .button_style(ButtonStyle::Bordered)
            .disabled(self.disabled)
            .into_element(context)
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
        self.id = Some(id.into());
        self
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
        let id = self
            .id
            .clone()
            .unwrap_or_else(|| Identity::from("navigation-stack"));
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
