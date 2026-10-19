//! Controlled virtualized string list backed by retained GPUI ListState.

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, Task, Window, div, px,
};
use gpui_component::{
    IndexPath,
    list::{List as GpuiList, ListDelegate, ListEvent, ListItem, ListState},
};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, View};

fn apply_selection_request(
    current: &mut Option<usize>,
    expected: &Cell<Option<usize>>,
    requested: Option<usize>,
) -> bool {
    if *current != expected.get() || *current == requested {
        return false;
    }
    *current = requested;
    expected.set(requested);
    true
}

fn clear_cancel_request(selectable: bool, requested: &Cell<Option<usize>>) -> bool {
    if !selectable {
        return false;
    }
    requested.set(None);
    true
}

struct StringListDelegate {
    items: Vec<String>,
    visible: Vec<usize>,
    query: String,
    row_height: f32,
    owner_selection: Rc<Cell<Option<usize>>>,
    selectable: bool,
}

impl StringListDelegate {
    fn new(
        items: Vec<String>,
        row_height: f32,
        selection: Option<usize>,
        selectable: bool,
    ) -> Self {
        let visible = (0..items.len()).collect();
        Self {
            items,
            visible,
            query: String::new(),
            row_height,
            owner_selection: Rc::new(Cell::new(selection)),
            selectable,
        }
    }

    fn replace_items(&mut self, items: Vec<String>, row_height: f32) {
        self.items = items;
        self.row_height = row_height;
        self.apply_query();
    }

    fn apply_query(&mut self) {
        let query = self.query.to_lowercase();
        self.visible = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| query.is_empty() || item.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
    }

    fn source_index(&self, index: IndexPath) -> Option<usize> {
        (index.section == 0)
            .then(|| self.visible.get(index.row).copied())
            .flatten()
    }

    fn visible_index(&self, source_index: Option<usize>) -> Option<IndexPath> {
        let source_index = source_index?;
        self.visible
            .iter()
            .position(|index| *index == source_index)
            .map(IndexPath::new)
    }
}

impl ListDelegate for StringListDelegate {
    type Item = ListItem;

    fn items_count(&self, section: usize, _cx: &App) -> usize {
        if section == 0 { self.visible.len() } else { 0 }
    }

    fn render_item(
        &mut self,
        index: IndexPath,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let source_index = self.source_index(index)?;
        let label = SharedString::from(self.items.get(source_index)?.clone());
        Some(
            ListItem::new(("rustui-list-row", source_index))
                .h(px(self.row_height))
                .accessibility_label(label.clone())
                .child(
                    div()
                        .id(("rustui-list-label", source_index))
                        .role(gpui::Role::Label)
                        .aria_label(label.clone())
                        .child(label),
                ),
        )
    }

    fn set_selected_index(
        &mut self,
        _index: Option<IndexPath>,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
    }

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.query = query.to_owned();
        self.apply_query();
        cx.notify();
        Task::ready(())
    }
}

/// A virtualized string list with controlled source-index selection.
pub struct ListView {
    items: Vec<String>,
    selection: Binding<Option<usize>>,
    id: Option<Identity>,
    height: f32,
    row_height: f32,
    searchable: bool,
    selectable: bool,
    scrollbar_visible: bool,
    search_placeholder: Option<SharedString>,
}

impl ListView {
    pub fn new<I, S>(items: I, selection: Binding<Option<usize>>) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            items: items.into_iter().map(Into::into).collect(),
            selection,
            id: None,
            height: 240.0,
            row_height: 36.0,
            searchable: false,
            selectable: true,
            scrollbar_visible: true,
            search_placeholder: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    pub fn row_height(mut self, row_height: f32) -> Self {
        self.row_height = row_height.max(1.0);
        self
    }

    pub fn searchable(mut self, searchable: bool) -> Self {
        self.searchable = searchable;
        self
    }

    pub fn selectable(mut self, selectable: bool) -> Self {
        self.selectable = selectable;
        self
    }

    pub fn scrollbar_visible(mut self, visible: bool) -> Self {
        self.scrollbar_visible = visible;
        self
    }

    pub fn search_placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.search_placeholder = Some(placeholder.into());
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "list:{}:{}",
                self.items.len(),
                self.items.first().map_or("", String::as_str)
            ))
        })
    }
}

#[allow(non_snake_case)]
pub fn List<I, S>(items: I, selection: Binding<Option<usize>>) -> ListView
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    ListView::new(items, selection)
}

impl View for ListView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity();
        let items = self.items.clone();
        let initial_selection = self.selection.value();
        let row_height = self.row_height;
        let searchable = self.searchable;
        let selectable = self.selectable;
        let (state, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let delegate =
                StringListDelegate::new(items, row_height, initial_selection, selectable);
            let selected = delegate.visible_index(initial_selection);
            let mut state = ListState::new(delegate, window, cx)
                .searchable(searchable)
                .selectable(selectable);
            state.set_selected_index(selected, window, cx);
            state
        });

        let owner_selection = state.read(context.app()).delegate().owner_selection.clone();
        owner_selection.set(self.selection.value());

        let data_changed = {
            let state = state.read(context.app());
            state.delegate().items != self.items || state.delegate().row_height != self.row_height
        };
        if data_changed {
            let items = self.items.clone();
            let selection = self.selection.value();
            let row_height = self.row_height;
            context.update_entity(&state, move |state, window, cx| {
                state.delegate_mut().replace_items(items, row_height);
                let selected = state.delegate().visible_index(selection);
                state.set_selected_index(selected, window, cx);
                cx.notify();
            });
        }

        let current = self.selection.value();
        let (selected, desired) = {
            let state = state.read(context.app());
            (
                state.selected_index(),
                state.delegate().visible_index(current),
            )
        };
        if selected != desired {
            context.update_entity(&state, move |state, window, cx| {
                state.set_selected_index(desired, window, cx);
            });
        }

        if created {
            let requested = Rc::new(Cell::new(None));
            let current_request = requested.clone();
            let expected_selection = owner_selection.clone();
            let update = context.binding_updater(self.selection.clone(), move |current| {
                let selected = current_request.get();
                apply_selection_request(current, &expected_selection, selected)
            });
            context.subscribe_retained(&identity, &state, move |state, event: &ListEvent, cx| {
                match event {
                    ListEvent::Select(index) | ListEvent::Confirm(index) => {
                        if let Some(source_index) = state.read(cx).delegate().source_index(*index) {
                            requested.set(Some(source_index));
                            update(cx);
                        }
                    }
                    ListEvent::Cancel => {
                        let selectable = state.read(cx).delegate().selectable;
                        if clear_cancel_request(selectable, &requested) {
                            update(cx);
                        }
                    }
                }
            });
        }

        let mut list = GpuiList::new(&state)
            .h(px(self.height))
            .scrollbar_visible(self.scrollbar_visible);
        if let Some(placeholder) = &self.search_placeholder {
            list = list.search_placeholder(placeholder.clone());
        }
        list.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_event_preserves_newer_owner_selection() {
        let expected = Cell::new(Some(0));
        let mut owner = Some(2);

        assert!(!apply_selection_request(&mut owner, &expected, Some(1)));
        assert!(!apply_selection_request(&mut owner, &expected, None));
        assert_eq!(owner, Some(2));
        assert_eq!(expected.get(), Some(0));
    }

    #[test]
    fn accepted_selection_advances_expected_owner_value() {
        let expected = Cell::new(Some(0));
        let mut owner = Some(0);

        assert!(apply_selection_request(&mut owner, &expected, Some(1)));
        assert!(apply_selection_request(&mut owner, &expected, None));
        assert_eq!(owner, None);
        assert_eq!(expected.get(), None);
    }

    #[test]
    fn cancel_preserves_requested_and_owner_selection_when_not_selectable() {
        let requested = Cell::new(Some(0));
        let expected = Cell::new(Some(0));
        let mut owner = Some(0);

        if clear_cancel_request(false, &requested) {
            apply_selection_request(&mut owner, &expected, requested.get());
        }

        assert_eq!(requested.get(), Some(0));
        assert_eq!(owner, Some(0));
        assert_eq!(expected.get(), Some(0));
    }

    #[test]
    fn cancel_clears_requested_and_owner_selection_when_selectable() {
        let requested = Cell::new(Some(0));
        let expected = Cell::new(Some(0));
        let mut owner = Some(0);

        if clear_cancel_request(true, &requested) {
            apply_selection_request(&mut owner, &expected, requested.get());
        }

        assert_eq!(requested.get(), None);
        assert_eq!(owner, None);
        assert_eq!(expected.get(), None);
    }
}
