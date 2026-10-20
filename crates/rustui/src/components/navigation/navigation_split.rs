//! Sidebar selection and detail composition over the native split layout.

use std::{collections::HashSet, rc::Rc};

use crate::layout::{SplitPanel, SplitView};
use gpui::{AnyElement, SharedString};
use rustui_core::{Binding, Identity};

use crate::{
    accessibility::semantics::{Semantics, SemanticsRole},
    components::navigation::sidebar::{Sidebar, SidebarItem},
    view::{AnyView, EmptyView, RenderContext, RetentionScope, View},
};

#[derive(Clone)]
enum SplitLabel {
    Literal(SharedString),
    Message(SharedString),
}

/// A stable sidebar destination and its detail view.
pub struct NavigationSplitItem {
    id: String,
    label: SplitLabel,
    content: Rc<AnyView>,
    disabled: bool,
}

impl NavigationSplitItem {
    /// Creates a destination. IDs must be nonempty and unique in the split view.
    pub fn new<V: View>(id: impl Into<String>, label: impl Into<SharedString>, content: V) -> Self {
        let id = id.into();
        assert!(
            !id.trim().is_empty(),
            "NavigationSplit item IDs must be nonempty"
        );
        Self {
            id,
            label: SplitLabel::Literal(label.into()),
            content: Rc::new(AnyView::new(content)),
            disabled: false,
        }
    }

    /// Keeps this destination visible but prevents selecting it from the sidebar.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Resolves the sidebar label from the active RustUI message catalog.
    pub fn localized(mut self, message_id: impl Into<SharedString>) -> Self {
        self.label = SplitLabel::Message(message_id.into());
        self
    }
}

fn validate_items(items: Vec<NavigationSplitItem>) -> Vec<NavigationSplitItem> {
    let mut ids = HashSet::with_capacity(items.len());
    for item in &items {
        assert!(
            !item.id.trim().is_empty(),
            "NavigationSplit item IDs must be nonempty"
        );
        assert!(
            ids.insert(item.id.as_str()),
            "duplicate NavigationSplit item ID: {}",
            item.id
        );
    }
    items
}

fn selected_index(items: &[NavigationSplitItem], selection: Option<&str>) -> Option<usize> {
    let selection = selection?;
    items.iter().position(|item| item.id == selection)
}

fn detail_accessibility_label(label: &str) -> SharedString {
    SharedString::from(label)
}

struct NavigationSplitDetail {
    route: Option<(String, SharedString, Rc<AnyView>)>,
}

struct SharedAnyView(Rc<AnyView>);

impl View for SharedAnyView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        self.0.into_element(context)
    }
}

impl View for NavigationSplitDetail {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        match &self.route {
            Some((route, label, content)) => {
                let detail = Semantics(
                    format!("navigation-split-detail:{route}"),
                    SharedAnyView(Rc::clone(content)),
                )
                .role(SemanticsRole::Region)
                // Reuse the destination's resolved label so localized rows
                // do not acquire an English-only "details" suffix.
                .label(detail_accessibility_label(label));
                context.with_element_scope(RetentionScope::Page(vec![route.clone()]), |context| {
                    detail.into_element(context)
                })
            }
            None => EmptyView.into_element(context),
        }
    }
}

/// A controlled sidebar and selected detail pane.
///
/// The caller owns selection and bound application values. Only the selected
/// destination is lowered; switching away releases its retained native controls.
/// Like [`SplitView`], this view requires a parent with bounded width and height.
/// Use `Frame` or `Container` to set an explicit viewport inside an intrinsic stack.
pub struct NavigationSplitView {
    id: Identity,
    items: Vec<NavigationSplitItem>,
    selection: Binding<Option<String>>,
    accessibility_label: Option<SplitLabel>,
}

impl NavigationSplitView {
    pub fn new(
        id: impl Into<Identity>,
        items: impl IntoIterator<Item = NavigationSplitItem>,
        selection: Binding<Option<String>>,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "NavigationSplit ID must be nonempty"
        );
        Self {
            id,
            items: validate_items(items.into_iter().collect()),
            selection,
            accessibility_label: None,
        }
    }

    /// Names the navigation tree for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(SplitLabel::Literal(label.into()));
        self
    }

    /// Resolves the navigation tree name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(SplitLabel::Message(message_id.into()));
        self
    }
}

#[allow(non_snake_case)]
pub fn NavigationSplit(
    id: impl Into<Identity>,
    items: impl IntoIterator<Item = NavigationSplitItem>,
    selection: Binding<Option<String>>,
) -> NavigationSplitView {
    NavigationSplitView::new(id, items, selection)
}

impl View for NavigationSplitView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let selected = self.selection.value();
        let sidebar_items = self.items.iter().map(|item| {
            let label = match &item.label {
                SplitLabel::Literal(label) | SplitLabel::Message(label) => label.clone(),
            };
            let sidebar_item = SidebarItem::new(item.id.clone(), label).disabled(item.disabled);
            match &item.label {
                SplitLabel::Literal(_) => sidebar_item,
                SplitLabel::Message(message_id) => sidebar_item.localized(message_id.clone()),
            }
        });
        let sidebar = Sidebar(
            format!("{}:sidebar", self.id.as_str()),
            sidebar_items,
            self.selection.clone(),
        );
        let sidebar = match &self.accessibility_label {
            Some(SplitLabel::Literal(label)) => sidebar.accessibility_label(label.clone()),
            Some(SplitLabel::Message(message_id)) => {
                sidebar.accessibility_label(context.localized_message(message_id.as_ref()))
            }
            None => sidebar,
        };
        let route = selected
            .as_deref()
            .and_then(|id| selected_index(&self.items, Some(id)))
            .map(|index| {
                let item = &self.items[index];
                let label = match &item.label {
                    SplitLabel::Literal(label) => label.clone(),
                    SplitLabel::Message(message_id) => {
                        SharedString::from(context.localized_message(message_id.as_ref()))
                    }
                };
                (item.id.clone(), label, Rc::clone(&item.content))
            });
        let detail = NavigationSplitDetail { route };
        let layout = SplitView::horizontal(format!("{}:layout", self.id.as_str()))
            .panel(
                SplitPanel::new("sidebar", sidebar)
                    .initial_size(240.0)
                    .size_range(140.0..480.0),
            )
            .panel(SplitPanel::new("detail", detail));

        context.with_element_scope(
            RetentionScope::NavigationSplit(self.id.clone()),
            |context| layout.into_element(context),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{
        NavigationSplitItem, NavigationSplitView, detail_accessibility_label, selected_index,
        validate_items,
    };
    use crate::view::EmptyView;
    use rustui_core::Binding;

    struct Owner(Option<String>);

    fn read_selection(owner: &Owner) -> Option<String> {
        owner.0.clone()
    }

    fn write_selection(owner: &mut Owner, selection: Option<String>) {
        owner.0 = selection;
    }

    fn item(id: &str) -> NavigationSplitItem {
        NavigationSplitItem::new(id, id, EmptyView)
    }

    #[test]
    fn accessibility_label_is_optional_and_can_be_set_for_navigation_tree() {
        let selection = Binding::new(None, read_selection, write_selection);
        let view = NavigationSplitView::new("main", [item("home")], selection);
        assert!(view.accessibility_label.is_none());

        let view = view.accessibility_label("Main navigation");
        assert!(matches!(
            view.accessibility_label,
            Some(super::SplitLabel::Literal(label)) if label.as_ref() == "Main navigation"
        ));
    }

    #[test]
    fn localized_accessibility_label_replaces_literal_label() {
        let selection = Binding::new(None, read_selection, write_selection);
        let view = NavigationSplitView::new("main", [item("home")], selection)
            .accessibility_label("Main navigation")
            .localized_accessibility_label("navigation.main");
        assert!(matches!(
            view.accessibility_label,
            Some(super::SplitLabel::Message(message_id)) if message_id.as_ref() == "navigation.main"
        ));
    }

    #[test]
    fn localized_destination_label_is_used_for_its_detail_region() {
        assert_eq!(detail_accessibility_label("Fichiers").as_ref(), "Fichiers");
    }

    #[test]
    fn selection_tracks_stable_ids_and_missing_selection_is_empty() {
        let before = vec![item("files"), item("settings")];
        let after = vec![item("settings"), item("files")];

        assert_eq!(selected_index(&before, Some("settings")), Some(1));
        assert_eq!(selected_index(&after, Some("settings")), Some(0));
        assert_eq!(selected_index(&after, None), None);
        assert_eq!(selected_index(&after, Some("missing")), None);
    }

    #[test]
    fn rejects_empty_and_duplicate_item_ids() {
        assert!(std::panic::catch_unwind(|| item(" ")).is_err());
        assert!(
            std::panic::catch_unwind(|| validate_items(vec![item("files"), item("files")]))
                .is_err()
        );
    }
}
