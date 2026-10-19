//! RustUI actions presented through GPUI Components breadcrumbs.

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, SharedString, div,
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
        Self {
            label: label.into(),
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
}

impl BreadcrumbView {
    pub fn new(items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        Self {
            items: items.into_iter().collect(),
            id: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "breadcrumb:{}:{}",
                self.items.len(),
                self.items.first().map_or("", |item| item.label.as_ref())
            ))
        })
    }
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
        let mut breadcrumb = GpuiBreadcrumb::new();
        for item in &self.items {
            let mut upstream = GpuiBreadcrumbItem::new(item.label.clone()).disabled(item.disabled);
            if let Some(action) = &item.action {
                upstream = upstream.on_click(context.bind_control_action(action.clone()));
            }
            breadcrumb = breadcrumb.child(upstream);
        }

        div()
            .id(self.identity().to_string())
            .child(breadcrumb)
            .into_any_element()
    }
}
