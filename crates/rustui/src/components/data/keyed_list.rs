//! Controlled, stable-ID string list backed by retained GPUI ListState.

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use gpui::{
    AnyElement, App, Context, ElementId, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, StatefulInteractiveElement as _, Styled as _, Task, Window, div, px,
};
use gpui_component::{
    IndexPath,
    list::{List as GpuiList, ListDelegate, ListEvent, ListItem, ListState},
};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, View};

type SelectionUpdater<T> = Rc<dyn Fn(&mut T)>;

fn replace_selection_updater<T>(
    slot: &RefCell<Option<SelectionUpdater<T>>>,
    updater: SelectionUpdater<T>,
) {
    *slot.borrow_mut() = Some(updater);
}

fn apply_selection_updater<T>(slot: &RefCell<Option<SelectionUpdater<T>>>, context: &mut T) {
    let updater = slot.borrow().clone();
    if let Some(updater) = updater {
        updater(context);
    }
}

/// One string row with identity independent of its position and label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KeyedListItem {
    id: String,
    label: String,
}

impl KeyedListItem {
    /// Creates a row. IDs must be unique, and labels must contain a non-whitespace character.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        let id = id.into();
        let label = label.into();
        assert!(!id.trim().is_empty(), "KeyedList item IDs must be nonempty");
        assert!(
            !label.trim().is_empty(),
            "KeyedList item labels must be nonempty"
        );
        Self { id, label }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn label(&self) -> &str {
        &self.label
    }
}

fn collect_items(items: impl IntoIterator<Item = KeyedListItem>) -> Vec<KeyedListItem> {
    let items: Vec<_> = items.into_iter().collect();
    let mut ids = HashSet::with_capacity(items.len());
    for item in &items {
        assert!(
            !item.id.trim().is_empty(),
            "KeyedList item IDs must be nonempty"
        );
        assert!(
            ids.insert(item.id.as_str()),
            "duplicate KeyedList item ID: {}",
            item.id
        );
    }
    items
}

fn valid_selection(items: &[KeyedListItem], requested: Option<&str>) -> Option<String> {
    requested
        .filter(|id| items.iter().any(|item| item.id == *id))
        .map(str::to_owned)
}

fn clear_removed_selection(current: &mut Option<String>, removed_id: &str) -> bool {
    if current.as_deref() != Some(removed_id) {
        return false;
    }
    *current = None;
    true
}

fn apply_selection_request(
    current: &mut Option<String>,
    expected: &RefCell<Option<String>>,
    request: &Option<SelectionRequest>,
) -> bool {
    let Some(request) = request else {
        return false;
    };
    if current.as_ref() != request.expected_owner.as_ref()
        || expected.borrow().as_ref() != request.expected_owner.as_ref()
        || current.as_ref() == request.selection.as_ref()
    {
        return false;
    }
    *expected.borrow_mut() = request.selection.clone();
    *current = request.selection.clone();
    true
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct SelectionRequest {
    expected_owner: Option<String>,
    selection: Option<String>,
}

fn clear_selection_on_cancel(
    selectable: bool,
    selection: &mut Option<String>,
    requested: &mut Option<String>,
) -> bool {
    if !selectable {
        return false;
    }
    *selection = None;
    *requested = None;
    true
}

struct KeyedListDelegate {
    items: Vec<KeyedListItem>,
    visible: Vec<usize>,
    query: String,
    row_height: f32,
    selection_id: Rc<RefCell<Option<String>>>,
    expected_owner_selection: Rc<RefCell<Option<String>>>,
    requested_selection: Rc<RefCell<Option<SelectionRequest>>>,
    selection_update: Rc<RefCell<Option<SelectionUpdater<App>>>>,
    searchable: bool,
    selectable: bool,
}

impl KeyedListDelegate {
    fn new(
        items: Vec<KeyedListItem>,
        row_height: f32,
        selection_id: Option<String>,
        searchable: bool,
        selectable: bool,
    ) -> Self {
        let visible = (0..items.len()).collect();
        Self {
            items,
            visible,
            query: String::new(),
            row_height,
            selection_id: Rc::new(RefCell::new(selection_id.clone())),
            expected_owner_selection: Rc::new(RefCell::new(selection_id)),
            requested_selection: Rc::new(RefCell::new(None)),
            selection_update: Rc::new(RefCell::new(None)),
            searchable,
            selectable,
        }
    }

    fn replace_items(&mut self, items: Vec<KeyedListItem>, row_height: f32) {
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
            .filter(|(_, item)| query.is_empty() || item.label.to_lowercase().contains(&query))
            .map(|(index, _)| index)
            .collect();
    }

    fn source_index(&self, index: IndexPath) -> Option<usize> {
        (index.section == 0)
            .then(|| self.visible.get(index.row).copied())
            .flatten()
    }

    fn id_at(&self, index: IndexPath) -> Option<&str> {
        self.items
            .get(self.source_index(index)?)
            .map(|item| item.id.as_str())
    }

    fn visible_index(&self, id: Option<&str>) -> Option<IndexPath> {
        let id = id?;
        self.visible
            .iter()
            .position(|source| self.items[*source].id == id)
            .map(IndexPath::new)
    }
}

impl ListDelegate for KeyedListDelegate {
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
        let item = self.items.get(self.source_index(index)?)?;
        let label = SharedString::from(item.label.clone());
        Some(
            ListItem::new((ElementId::from("rustui-keyed-list-row"), item.id.clone()))
                .h(px(self.row_height))
                .accessibility_label(label.clone())
                .child(
                    div()
                        .id((ElementId::from("rustui-keyed-list-label"), item.id.clone()))
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
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Task<()> {
        self.query = query.to_owned();
        self.apply_query();
        // Upstream start_search selects its first row after this delegate call.
        // Restore the owner's stable ID once that update has finished.
        let state = cx.entity();
        let selection_id = self.selection_id.clone();
        window.defer(cx, move |window, cx| {
            state.update(cx, |state, cx| {
                let selected = state
                    .delegate()
                    .visible_index(selection_id.borrow().as_deref());
                if state.selected_index() != selected {
                    state.set_selected_index(selected, window, cx);
                    cx.notify();
                }
            });
        });
        cx.notify();
        Task::ready(())
    }
}

/// A virtualized string list whose owner selection follows a stable row ID.
pub struct KeyedListView {
    id: Identity,
    items: Vec<KeyedListItem>,
    selection: Binding<Option<String>>,
    height: f32,
    row_height: f32,
    searchable: bool,
    selectable: bool,
    scrollbar_visible: bool,
    search_placeholder: Option<SharedString>,
}

impl KeyedListView {
    pub fn new(
        id: impl Into<Identity>,
        items: impl IntoIterator<Item = KeyedListItem>,
        selection: Binding<Option<String>>,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "KeyedList identity must be nonempty"
        );
        Self {
            id,
            items: collect_items(items),
            selection,
            height: 240.0,
            row_height: 36.0,
            searchable: false,
            selectable: true,
            scrollbar_visible: true,
            search_placeholder: None,
        }
    }

    pub fn height(mut self, height: f32) -> Self {
        assert!(height.is_finite(), "KeyedList height must be finite");
        self.height = height.max(1.0);
        self
    }
    pub fn row_height(mut self, height: f32) -> Self {
        assert!(height.is_finite(), "KeyedList row height must be finite");
        self.row_height = height.max(1.0);
        self
    }
    pub fn searchable(mut self, enabled: bool) -> Self {
        self.searchable = enabled;
        self
    }
    pub fn selectable(mut self, enabled: bool) -> Self {
        self.selectable = enabled;
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
}

#[allow(non_snake_case)]
pub fn KeyedList(
    id: impl Into<Identity>,
    items: impl IntoIterator<Item = KeyedListItem>,
    selection: Binding<Option<String>>,
) -> KeyedListView {
    KeyedListView::new(id, items, selection)
}

impl View for KeyedListView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.id.clone();
        let items = self.items.clone();
        let initial_selection = valid_selection(&items, self.selection.value().as_deref());
        let row_height = self.row_height;
        let searchable = self.searchable;
        let selectable = self.selectable;
        let (state, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let delegate = KeyedListDelegate::new(
                items,
                row_height,
                initial_selection.clone(),
                searchable,
                selectable,
            );
            let selected = delegate.visible_index(initial_selection.as_deref());
            let mut state = ListState::new(delegate, window, cx)
                .searchable(searchable)
                .selectable(selectable);
            state.set_selected_index(selected, window, cx);
            state
        });
        let selection_id = state.read(context.app()).delegate().selection_id.clone();
        let expected_selection = state
            .read(context.app())
            .delegate()
            .expected_owner_selection
            .clone();
        let (requested_selection, selection_update) = {
            let delegate = state.read(context.app()).delegate();
            (
                delegate.requested_selection.clone(),
                delegate.selection_update.clone(),
            )
        };

        let current_request = requested_selection.clone();
        let expected = expected_selection.clone();
        replace_selection_updater(
            &selection_update,
            context.binding_updater(self.selection.clone(), move |current| {
                apply_selection_request(current, &expected, &current_request.borrow())
            }),
        );

        if created {
            let event_selection_id = selection_id.clone();
            let event_expected_selection = expected_selection.clone();
            let current_update = selection_update.clone();
            let requested = requested_selection.clone();
            context.subscribe_retained(&identity, &state, move |state, event: &ListEvent, cx| {
                let selected_index = state.read(cx).selected_index();
                match event {
                    ListEvent::Select(index) | ListEvent::Confirm(index) => {
                        if selected_index != Some(*index) {
                            return;
                        }
                        let selected = state.read(cx).delegate().id_at(*index).map(str::to_owned);
                        if let Some(id) = selected {
                            *event_selection_id.borrow_mut() = Some(id.clone());
                            *requested.borrow_mut() = Some(SelectionRequest {
                                expected_owner: event_expected_selection.borrow().clone(),
                                selection: Some(id),
                            });
                            apply_selection_updater(&current_update, cx);
                        }
                    }
                    ListEvent::Cancel => {
                        if selected_index.is_some() {
                            return;
                        }
                        let selectable = state.read(cx).delegate().selectable;
                        let mut cancelled_selection = event_selection_id.borrow().clone();
                        if clear_selection_on_cancel(
                            selectable,
                            &mut *event_selection_id.borrow_mut(),
                            &mut cancelled_selection,
                        ) {
                            *requested.borrow_mut() = Some(SelectionRequest {
                                expected_owner: event_expected_selection.borrow().clone(),
                                selection: cancelled_selection,
                            });
                            apply_selection_updater(&current_update, cx);
                        }
                    }
                }
            });
        }

        let requested = self.selection.value();
        *expected_selection.borrow_mut() = requested.clone();
        let desired = valid_selection(&self.items, requested.as_deref());
        *selection_id.borrow_mut() = desired.clone();
        let flags_changed = {
            let state = state.read(context.app());
            state.delegate().searchable != self.searchable
                || state.delegate().selectable != self.selectable
        };
        if flags_changed {
            let searchable = self.searchable;
            let selectable = self.selectable;
            context.update_entity(&state, move |state, _window, cx| {
                if state.delegate().searchable != searchable {
                    state.set_searchable(searchable, cx);
                    state.delegate_mut().searchable = searchable;
                }
                if state.delegate().selectable != selectable {
                    state.set_selectable(selectable, cx);
                    state.delegate_mut().selectable = selectable;
                }
            });
        }
        let data_changed = {
            let state = state.read(context.app());
            state.delegate().items != self.items || state.delegate().row_height != self.row_height
        };
        if data_changed {
            let items = self.items.clone();
            let selection = desired.clone();
            let row_height = self.row_height;
            context.update_entity(&state, move |state, window, cx| {
                state.delegate_mut().replace_items(items, row_height);
                let selected = state.delegate().visible_index(selection.as_deref());
                state.set_selected_index(selected, window, cx);
                cx.notify();
            });
        }

        let native_selected = state.read(context.app()).selected_index();
        let visible_selected = state
            .read(context.app())
            .delegate()
            .visible_index(desired.as_deref());
        if native_selected != visible_selected {
            context.update_entity(&state, move |state, window, cx| {
                state.set_selected_index(visible_selected, window, cx);
            });
        }

        if requested != desired {
            let invalid_id = requested.expect("only a missing selected ID needs normalization");
            let clear = context.binding_updater(self.selection.clone(), move |current| {
                clear_removed_selection(current, &invalid_id)
            });
            context.update_entity(&state, move |_state, _window, cx| {
                cx.defer(move |cx| clear(cx));
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
    use gpui_component::IndexPath;

    struct Owner(Option<String>);

    fn read_selection(owner: &Owner) -> Option<String> {
        owner.0.clone()
    }

    fn write_selection(owner: &mut Owner, selection: Option<String>) {
        owner.0 = selection;
    }

    fn list() -> KeyedListView {
        KeyedListView::new(
            "list",
            [],
            Binding::new(None, read_selection, write_selection),
        )
    }

    fn item(id: &str, label: &str) -> KeyedListItem {
        KeyedListItem::new(id, label)
    }

    #[test]
    fn rejects_empty_and_duplicate_ids() {
        assert!(std::panic::catch_unwind(|| item(" ", "blank")).is_err());
        assert!(
            std::panic::catch_unwind(|| collect_items([item("a", "A"), item("a", "Other")]))
                .is_err()
        );
    }

    #[test]
    fn rejects_blank_accessible_labels_and_preserves_nonblank_labels() {
        assert!(std::panic::catch_unwind(|| item("a", "")).is_err());
        assert!(std::panic::catch_unwind(|| item("b", " \t\n ")).is_err());

        assert_eq!(item("c", "  Cedar  ").label(), "  Cedar  ");
    }

    #[test]
    fn rejects_blank_list_identity() {
        assert!(
            std::panic::catch_unwind(|| {
                KeyedListView::new("", [], Binding::new(None, read_selection, write_selection))
            })
            .is_err()
        );
        assert!(
            std::panic::catch_unwind(|| {
                KeyedListView::new(
                    " \t\n ",
                    [],
                    Binding::new(None, read_selection, write_selection),
                )
            })
            .is_err()
        );
    }

    #[test]
    fn rejects_non_finite_viewport_and_row_heights() {
        for height in [f32::INFINITY, f32::NEG_INFINITY, f32::NAN] {
            assert!(std::panic::catch_unwind(|| list().height(height)).is_err());
            assert!(std::panic::catch_unwind(|| list().row_height(height)).is_err());
        }
    }

    #[test]
    fn selected_id_survives_insert_reorder_and_filter() {
        let mut delegate = KeyedListDelegate::new(
            vec![item("a", "Alpha"), item("b", "Beta")],
            36.0,
            Some("b".into()),
            false,
            true,
        );
        let selected = Some("b");
        assert_eq!(delegate.visible_index(selected), Some(IndexPath::new(1)));
        delegate.replace_items(
            vec![item("x", "Extra"), item("a", "Alpha"), item("b", "Beta")],
            36.0,
        );
        assert_eq!(valid_selection(&delegate.items, selected), Some("b".into()));
        assert_eq!(delegate.visible_index(selected), Some(IndexPath::new(2)));
        delegate.query = "Alpha".into();
        delegate.apply_query();
        assert_eq!(delegate.visible_index(selected), None);
        assert_eq!(delegate.id_at(IndexPath::new(0)), Some("a"));
        delegate.query.clear();
        delegate.apply_query();
        assert_eq!(delegate.visible_index(selected), Some(IndexPath::new(2)));
    }

    #[test]
    fn removal_requires_owner_normalization() {
        let items = [item("a", "Alpha")];
        assert_eq!(valid_selection(&items, Some("missing")), None);
        assert_eq!(valid_selection(&items, Some("a")), Some("a".into()));
        let mut selected = Some("missing".to_owned());
        assert!(clear_removed_selection(&mut selected, "missing"));
        assert_eq!(selected, None);
        let mut new_owner_selection = Some("a".to_owned());
        assert!(!clear_removed_selection(
            &mut new_owner_selection,
            "missing"
        ));
        assert_eq!(new_owner_selection.as_deref(), Some("a"));
    }

    #[test]
    fn queued_selection_preserves_newer_owner_value() {
        let expected = std::cell::RefCell::new(Some(String::from("first")));
        let mut owner = Some(String::from("newer"));
        let request = Some(SelectionRequest {
            expected_owner: Some(String::from("first")),
            selection: Some(String::from("stale")),
        });

        assert!(!apply_selection_request(&mut owner, &expected, &request));
        assert_eq!(owner.as_deref(), Some("newer"));
        assert_eq!(expected.borrow().as_deref(), Some("first"));
    }

    #[test]
    fn queued_selection_is_rejected_after_owner_rerender() {
        let expected = std::cell::RefCell::new(Some(String::from("A")));
        let request = Some(SelectionRequest {
            expected_owner: Some(String::from("A")),
            selection: Some(String::from("B")),
        });

        // The owner rerenders with C after B is queued but before its updater runs.
        let mut owner = Some(String::from("C"));
        *expected.borrow_mut() = owner.clone();

        assert!(!apply_selection_request(&mut owner, &expected, &request));
        assert_eq!(owner.as_deref(), Some("C"));
        assert_eq!(expected.borrow().as_deref(), Some("C"));
    }

    #[test]
    fn accepted_selection_advances_expected_owner_value() {
        let expected = std::cell::RefCell::new(Some(String::from("first")));
        let mut owner = Some(String::from("first"));
        let request = Some(SelectionRequest {
            expected_owner: Some(String::from("first")),
            selection: Some(String::from("second")),
        });

        assert!(apply_selection_request(&mut owner, &expected, &request));
        assert_eq!(owner.as_deref(), Some("second"));
        assert_eq!(expected.borrow().as_deref(), Some("second"));
    }

    #[test]
    fn cancel_preserves_selection_when_list_is_not_selectable() {
        let mut selection = Some("a".to_owned());
        let mut requested = Some("a".to_owned());

        assert!(!clear_selection_on_cancel(
            false,
            &mut selection,
            &mut requested
        ));
        assert_eq!(selection.as_deref(), Some("a"));
        assert_eq!(requested.as_deref(), Some("a"));
    }

    #[test]
    fn cancel_clears_selection_when_list_is_selectable() {
        let mut selection = Some("a".to_owned());
        let mut requested = Some("a".to_owned());

        assert!(clear_selection_on_cancel(
            true,
            &mut selection,
            &mut requested
        ));
        assert_eq!(selection, None);
        assert_eq!(requested, None);
    }

    #[test]
    fn retained_selection_event_uses_latest_owner_binding_writer() {
        let slot = RefCell::new(None);
        let old_writer_called = Rc::new(std::cell::Cell::new(false));
        let new_writer_called = Rc::new(std::cell::Cell::new(false));

        let old = old_writer_called.clone();
        replace_selection_updater(&slot, Rc::new(move |_: &mut ()| old.set(true)));
        let new = new_writer_called.clone();
        replace_selection_updater(&slot, Rc::new(move |_: &mut ()| new.set(true)));

        apply_selection_updater(&slot, &mut ());

        assert!(!old_writer_called.get());
        assert!(new_writer_called.get());
    }
}
