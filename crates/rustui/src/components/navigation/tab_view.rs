//! Controlled tab selection and selected-content composition.

use std::collections::HashSet;

use gpui::{AnyElement, IntoElement, ParentElement as _, Styled as _, div, px};
use gpui_component::{
    Sizable as _,
    tab::{Tab as GpuiTab, TabBar},
};
use rustui_core::{Binding, Identity};

use crate::{
    theme::registry::active_theme,
    view::{AnyView, EmptyView, RenderContext, RetentionScope, View},
};

/// A stable tab and its RustUI content.
pub struct TabItem {
    id: String,
    label: String,
    content: AnyView,
    disabled: bool,
}

impl TabItem {
    pub fn new<V: View>(id: impl Into<String>, label: impl Into<String>, content: V) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            content: AnyView::new(content),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// Visual style supplied by GPUI Components TabBar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TabViewStyle {
    #[default]
    Automatic,
    Pill,
    Outline,
    Segmented,
    Underline,
}

/// A controlled tab bar that renders the selected RustUI child.
pub struct TabViewControl {
    tabs: Vec<TabItem>,
    selection: Binding<String>,
    id: Option<Identity>,
    style: TabViewStyle,
}

impl TabViewControl {
    pub fn new(tabs: impl IntoIterator<Item = TabItem>, selection: Binding<String>) -> Self {
        let tabs = tabs.into_iter().collect::<Vec<_>>();
        let mut ids = HashSet::new();
        assert!(
            tabs.iter().all(|tab| ids.insert(tab.id.as_str())),
            "every RustUI TabItem must have a unique ID"
        );

        Self {
            tabs,
            selection,
            id: None,
            style: TabViewStyle::Automatic,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn style(mut self, style: TabViewStyle) -> Self {
        self.style = style;
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "tab-view:{}:{}",
                self.tabs.len(),
                self.tabs.first().map_or("", |tab| tab.id.as_str())
            ))
        })
    }
}

#[allow(non_snake_case)]
pub fn TabView(
    tabs: impl IntoIterator<Item = TabItem>,
    selection: Binding<String>,
) -> TabViewControl {
    TabViewControl::new(tabs, selection)
}

impl View for TabViewControl {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let selected = self.selection.value();
        let selected_index = self.tabs.iter().position(|tab| tab.id == selected);
        let identity = self.identity();
        let ids = self
            .tabs
            .iter()
            .map(|tab| tab.id.clone())
            .collect::<Vec<_>>();
        let write = context.binding_writer(self.selection.clone());

        let tabs = self.tabs.iter().map(|tab| {
            GpuiTab::new()
                .label(tab.label.clone())
                .disabled(tab.disabled)
        });
        let mut tab_bar =
            TabBar::new(identity.to_string())
                .children(tabs)
                .on_click(move |index, _window, cx| {
                    if let Some(id) = ids.get(*index) {
                        write(id.clone(), cx);
                    }
                });
        if let Some(index) = selected_index {
            tab_bar = tab_bar.selected_index(index);
        }
        tab_bar = match self.style {
            TabViewStyle::Automatic => tab_bar,
            TabViewStyle::Pill => tab_bar.pill(),
            TabViewStyle::Outline => tab_bar.outline(),
            TabViewStyle::Segmented => tab_bar.segmented(),
            TabViewStyle::Underline => tab_bar.underline(),
        };
        if let Some(theme) = active_theme(context.app()) {
            tab_bar = tab_bar
                .small()
                .h(px(theme.profile.dimensions.control_height as f32));
        }

        context.with_element_scope(RetentionScope::Tabs(identity), |context| {
            for (index, tab) in self.tabs.iter().enumerate() {
                if Some(index) != selected_index {
                    context.preserve_retention_scope(RetentionScope::Tab(tab.id.clone()));
                }
            }
            let content = selected_index
                .and_then(|index| self.tabs.get(index))
                .map(|tab| {
                    context.with_element_scope(RetentionScope::Tab(tab.id.clone()), |context| {
                        tab.content.into_element(context)
                    })
                });

            let mut root = div().w_full().child(tab_bar);
            if let Some(content) = content {
                root = root.child(div().pt_3().child(content));
            }
            root.into_any_element()
        })
    }
}
