//! Controlled, stable-ID string list backed by retained GPUI ListState.

use std::{
    cell::{Cell, RefCell},
    collections::{HashMap, HashSet},
    rc::Rc,
};

use gpui::{
    AnimationExt as _, AnyElement, App, Context, ElementId, FocusHandle, InteractiveElement as _,
    IntoElement, ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _,
    Task, Window, div, px,
};
use gpui_component::{
    IndexPath,
    list::{List as GpuiList, ListDelegate, ListEvent, ListItem, ListState},
};
use rustui_core::{Binding, Identity};

use crate::{
    animation::transition::{Edge, Transition},
    view::{EmptyView, RenderContext, View},
};
use rustui_i18n::LayoutDirection;

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

fn clear_removed_selection(
    current: &mut Option<String>,
    removed_id: &str,
    resolved_selection: Option<&str>,
) -> bool {
    if current.as_deref() != Some(removed_id) || resolved_selection == Some(removed_id) {
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

struct RowAnimation {
    progress: Rc<Cell<f32>>,
    finished: Rc<Cell<bool>>,
    focus: FocusHandle,
}

struct KeyedListDelegate {
    source_items: Vec<KeyedListItem>,
    exits: HashMap<String, Rc<RowAnimation>>,
    animations: HashMap<String, Rc<RowAnimation>>,
    container_focus: Option<FocusHandle>,
    transition: Option<Transition>,
    direction: LayoutDirection,
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
            source_items: items.clone(),
            exits: HashMap::new(),
            animations: HashMap::new(),
            container_focus: None,
            transition: None,
            direction: LayoutDirection::LeftToRight,
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
        self.source_items = items.clone();
        self.items = items;
        self.row_height = row_height;
        self.apply_query();
    }

    fn replace_transition_items(
        &mut self,
        items: Vec<KeyedListItem>,
        row_height: f32,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        let ids: HashSet<_> = items.iter().map(|item| item.id.clone()).collect();
        for id in &ids {
            if let Some(exit) = self.exits.remove(id) {
                exit.finished.set(false);
                window.set_focus_scope_inert(&exit.focus, false);
            }
        }
        for id in &ids {
            self.animations.entry(id.clone()).or_insert_with(|| {
                Rc::new(RowAnimation {
                    progress: Rc::new(Cell::new(0.0)),
                    finished: Rc::new(Cell::new(false)),
                    focus: cx.focus_handle(),
                })
            });
        }
        let mut combined = items.clone();
        for (index, item) in self.items.iter().enumerate() {
            if !ids.contains(&item.id) {
                let animation = self
                    .animations
                    .entry(item.id.clone())
                    .or_insert_with(|| {
                        Rc::new(RowAnimation {
                            progress: Rc::new(Cell::new(1.0)),
                            finished: Rc::new(Cell::new(true)),
                            focus: cx.focus_handle(),
                        })
                    })
                    .clone();
                if !self.exits.contains_key(&item.id) {
                    animation.finished.set(false);
                    if animation.focus.contains_focused(window, cx) {
                        window.focus_next(cx);
                        if animation.focus.contains_focused(window, cx) {
                            if let Some(container) = &self.container_focus {
                                container.focus(window, cx);
                            }
                        }
                    }
                    window.set_focus_scope_inert(&animation.focus, true);
                    self.exits.insert(item.id.clone(), animation);
                }
                combined.insert(index.min(combined.len()), item.clone());
            }
        }
        self.source_items = items;
        self.items = combined;
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
            .filter(|item| !self.exits.contains_key(&item.id))
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

fn schedule_exits(window: &mut Window, weak: gpui::WeakEntity<ListState<KeyedListDelegate>>) {
    window.on_next_frame(move |window, app| {
        let Some(state) = weak.upgrade() else {
            return;
        };
        let more = state.update(app, |state, cx| {
            let delegate = state.delegate_mut();
            delegate.exits.retain(|_, exit| {
                if exit.finished.get() {
                    window.set_focus_scope_inert(&exit.focus, false);
                    false
                } else {
                    true
                }
            });
            let source: HashSet<_> = delegate
                .source_items
                .iter()
                .map(|item| item.id.clone())
                .collect();
            let before = delegate.items.len();
            delegate
                .items
                .retain(|item| source.contains(&item.id) || delegate.exits.contains_key(&item.id));
            if before != delegate.items.len() {
                delegate
                    .animations
                    .retain(|id, _| source.contains(id) || delegate.exits.contains_key(id));
                delegate.apply_query();
                cx.notify();
            }
            !delegate.exits.is_empty()
        });
        if more {
            schedule_exits(window, weak);
        }
    });
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
        let exiting = self.exits.get(&item.id);
        let label_element = div()
            .id((ElementId::from("rustui-keyed-list-label"), item.id.clone()))
            .role(gpui::Role::Label)
            .aria_label(label.clone())
            .child(label.clone());
        #[cfg(test)]
        let label_element = if item.id == "a" {
            label_element.debug_selector(|| "t17-exiting-list-row".into())
        } else {
            label_element
        };
        let mut content = div().relative().w_full().h_full().child(label_element);
        if let Some(animation) = self.animations.get(&item.id) {
            content = content.track_focus(&animation.focus);
        }
        if exiting.is_some() {
            content = content.child(div().absolute().inset_0().occlude());
        }
        if let (Some(transition), Some(animation)) =
            (self.transition, self.animations.get(&item.id))
        {
            let progress = animation.progress.get().clamp(0.0, 1.0);
            if transition.fade {
                content = content.opacity(progress);
            }
            let offset = 16.0 * (1.0 - progress);
            match transition.slide {
                Some(Edge::Leading) => {
                    content = content.left(px(if self.direction == LayoutDirection::RightToLeft {
                        offset
                    } else {
                        -offset
                    }))
                }
                Some(Edge::Trailing) => {
                    content = content.left(px(if self.direction == LayoutDirection::RightToLeft {
                        -offset
                    } else {
                        offset
                    }))
                }
                Some(Edge::Top) => content = content.top(px(-offset)),
                Some(Edge::Bottom) => content = content.top(px(offset)),
                None => {}
            }
        }
        let content = content.into_any_element();
        Some(
            ListItem::new((ElementId::from("rustui-keyed-list-row"), item.id.clone()))
                .h(px(self.row_height))
                .accessibility_label(label)
                .disabled(exiting.is_some())
                .child(content),
        )
    }

    fn set_selected_index(
        &mut self,
        index: Option<IndexPath>,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        if index.is_some_and(|index| {
            self.source_index(index)
                .is_some_and(|source| self.exits.contains_key(&self.items[source].id))
        }) {
            let state = cx.entity();
            let selected = self.selection_id.clone();
            window.defer(cx, move |window, app| {
                state.update(app, |state, cx| {
                    let index = state.delegate().visible_index(selected.borrow().as_deref());
                    state.set_selected_index(index, window, cx);
                    cx.notify();
                })
            });
        }
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
    transition: Option<Transition>,
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
            transition: None,
        }
    }

    pub fn transition(mut self, transition: Transition) -> Self {
        self.transition = Some(transition);
        self
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
    /// Shows a search field. Turning search off clears its filter while retaining selection.
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
        let row_height = self.row_height;
        let searchable = self.searchable;
        let selectable = self.selectable;
        let transition = self.transition.filter(|_| !context.app().reduce_motion());
        let direction = context.layout_direction();
        let (state, created) = context.retained_entity(identity.clone(), |window, cx| {
            let initial_selection = valid_selection(&self.items, self.selection.value().as_deref());
            let delegate = KeyedListDelegate::new(
                self.items.clone(),
                row_height,
                initial_selection.clone(),
                searchable,
                selectable,
            );
            let mut delegate = delegate;
            delegate.transition = transition;
            delegate.direction = direction;
            if transition.is_some() {
                delegate.container_focus = Some(cx.focus_handle());
                for item in &delegate.items {
                    delegate.animations.insert(
                        item.id.clone(),
                        Rc::new(RowAnimation {
                            progress: Rc::new(Cell::new(0.0)),
                            finished: Rc::new(Cell::new(false)),
                            focus: cx.focus_handle(),
                        }),
                    );
                }
            }
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
            context.update_entity(&state, move |state, window, cx| {
                if state.delegate().searchable != searchable {
                    state.set_searchable(searchable, cx);
                    state.delegate_mut().searchable = searchable;
                    if !searchable {
                        state.set_query("", window, cx);
                    }
                }
                if state.delegate().selectable != selectable {
                    state.set_selectable(selectable, cx);
                    state.delegate_mut().selectable = selectable;
                }
            });
        }
        let data_changed = {
            let state = state.read(context.app());
            state.delegate().source_items != self.items
                || state.delegate().row_height != self.row_height
                || state.delegate().transition != transition
                || state.delegate().direction != direction
        };
        if data_changed {
            let items = self.items.clone();
            let selection = desired.clone();
            let row_height = self.row_height;
            let transition = self.transition.filter(|_| !context.app().reduce_motion());
            let direction = context.layout_direction();
            context.update_entity(&state, move |state, window, cx| {
                let delegate = state.delegate_mut();
                delegate.transition = transition;
                delegate.direction = direction;
                if transition.is_some() {
                    if delegate.container_focus.is_none() {
                        delegate.container_focus = Some(cx.focus_handle());
                    }
                    delegate.replace_transition_items(items, row_height, window, cx);
                    if !delegate.exits.is_empty() {
                        schedule_exits(window, cx.entity().downgrade());
                    }
                } else {
                    delegate.exits.clear();
                    delegate.animations.clear();
                    delegate.replace_items(items, row_height);
                }
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
            let latest_selection = selection_id.clone();
            let clear = context.binding_updater(self.selection.clone(), move |current| {
                clear_removed_selection(current, &invalid_id, latest_selection.borrow().as_deref())
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
        let trackers = {
            let delegate = state.read(context.app()).delegate();
            if delegate.transition.is_none() {
                Vec::new()
            } else {
                delegate
                    .items
                    .iter()
                    .filter_map(|item| {
                        let animation = delegate.animations.get(&item.id)?.clone();
                        let target = if delegate.exits.contains_key(&item.id) {
                            0.0
                        } else {
                            1.0
                        };
                        let progress = animation.progress.clone();
                        let finished = animation.finished.clone();
                        Some(
                            div()
                                .absolute()
                                .size_0()
                                .with_spring(
                                    (
                                        ElementId::from("rustui-keyed-list-tracker"),
                                        item.id.clone(),
                                    ),
                                    gpui::SpringAnimation::new(gpui::SpringConfig::new(
                                        170.0, 26.0, 1.0,
                                    ))
                                    .to(target)
                                    .from(animation.progress.get())
                                    .with_epsilon(0.001),
                                    move |element, value: f32| {
                                        progress.set(value);
                                        if value == target {
                                            finished.set(true);
                                        }
                                        element
                                    },
                                )
                                .into_any_element(),
                        )
                    })
                    .collect::<Vec<_>>()
            }
        };
        if trackers.is_empty() {
            list.into_any_element()
        } else {
            let focus = state
                .read(context.app())
                .delegate()
                .container_focus
                .clone()
                .expect("animated list has a focus container");
            div()
                .relative()
                .track_focus(&focus)
                .child(list)
                .children(trackers)
                .into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui_component::IndexPath;

    struct SearchOwner {
        selection: Option<String>,
        searchable: bool,
        include_a: bool,
        writes: usize,
    }

    impl View for SearchOwner {
        fn body(&self) -> impl View {
            EmptyView
        }

        fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
            let selection = Binding::new(
                self.selection.clone(),
                |owner: &Self| owner.selection.clone(),
                |owner, value| {
                    owner.selection = value;
                    owner.writes += 1;
                },
            );
            div()
                .w(px(320.0))
                .h(px(280.0))
                .child(
                    KeyedList(
                        "keyed-search",
                        [
                            self.include_a.then(|| KeyedListItem::new("a", "Alpha")),
                            Some(KeyedListItem::new("b", "Beta")),
                        ]
                        .into_iter()
                        .flatten(),
                        selection,
                    )
                    .searchable(self.searchable)
                    .into_element(context),
                )
                .into_any_element()
        }
    }

    #[gpui::test]
    fn restored_item_survives_queued_selection_clear(cx: &mut gpui::TestAppContext) {
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (owner, window) = cx.add_window_view(|_, app| {
            crate::runtime::ComponentHost::new(
                SearchOwner {
                    selection: Some("a".into()),
                    searchable: false,
                    include_a: true,
                    writes: 0,
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            owner.update(app, |host, cx| {
                host.component_mut().include_a = false;
                cx.notify();
            });
            let _ = window.draw(app);
            owner.update(app, |host, cx| {
                host.component_mut().include_a = true;
                cx.notify();
            });
            let _ = window.draw(app);
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(owner.read(app).component().selection.as_deref(), Some("a"));
            assert_eq!(owner.read(app).component().writes, 0);
        });
    }

    #[gpui::test]
    fn disabling_search_clears_filter_and_preserves_keyed_selection(cx: &mut gpui::TestAppContext) {
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (owner, window) = cx.add_window_view(|_, app| {
            crate::runtime::ComponentHost::new(
                SearchOwner {
                    selection: Some("a".into()),
                    searchable: true,
                    include_a: true,
                    writes: 0,
                },
                app,
            )
        });
        let state = window.update(|window, app| {
            let _ = window.draw(app);
            let state = owner
                .read(app)
                .retained_entity_for_test::<ListState<KeyedListDelegate>>(
                    vec![],
                    Identity::from("keyed-search"),
                )
                .unwrap();
            state.update(app, |state, cx| state.set_query("Beta", window, cx));
            assert_eq!(state.read(app).delegate().visible, [1]);
            state
        });
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().searchable = false;
                cx.notify();
            });
            let _ = window.draw(app);
            assert_eq!(state.read(app).delegate().visible, [0, 1]);
            assert!(state.read(app).delegate().query.is_empty());
            assert_eq!(owner.read(app).component().selection.as_deref(), Some("a"));
            assert_eq!(owner.read(app).component().writes, 0);
            owner.update(app, |host, cx| {
                host.component_mut().searchable = true;
                cx.notify();
            });
            let _ = window.draw(app);
            assert_eq!(state.read(app).delegate().visible, [0, 1]);
            assert!(state.read(app).delegate().query.is_empty());
            assert_eq!(state.read(app).selected_index(), Some(IndexPath::new(0)));
        });
    }

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
        assert!(clear_removed_selection(&mut selected, "missing", None));
        assert_eq!(selected, None);
        let mut new_owner_selection = Some("a".to_owned());
        assert!(!clear_removed_selection(
            &mut new_owner_selection,
            "missing",
            None
        ));
        assert_eq!(new_owner_selection.as_deref(), Some("a"));
        let mut restored_selection = Some("missing".to_owned());
        assert!(!clear_removed_selection(
            &mut restored_selection,
            "missing",
            Some("missing")
        ));
        assert_eq!(restored_selection.as_deref(), Some("missing"));
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
    struct TransitionOwner {
        show: bool,
        animated: bool,
        selection: Option<String>,
        writes: usize,
    }
    impl View for TransitionOwner {
        fn body(&self) -> impl View {
            EmptyView
        }
        fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
            let binding = Binding::new(
                self.selection.clone(),
                |owner: &Self| owner.selection.clone(),
                |owner, value| {
                    owner.selection = value;
                    owner.writes += 1;
                },
            );
            let list = KeyedList(
                "transition-list",
                [
                    self.show.then(|| KeyedListItem::new("a", "Alpha")),
                    Some(KeyedListItem::new("b", "Beta")),
                ]
                .into_iter()
                .flatten(),
                binding,
            );
            let list = if self.animated {
                list.transition(Transition::fade_slide(Edge::Leading))
            } else {
                list
            };
            div()
                .w(px(320.0))
                .h(px(280.0))
                .child(list.into_element(context))
                .into_any_element()
        }
    }

    fn transition_setup(
        cx: &mut gpui::TestAppContext,
        reduced: bool,
        animated: bool,
    ) -> (
        gpui::Entity<crate::runtime::ComponentHost<TransitionOwner>>,
        &mut gpui::VisualTestContext,
        gpui::Entity<ListState<KeyedListDelegate>>,
    ) {
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            app.set_reduce_motion(reduced);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (owner, window) = cx.add_window_view(|_, app| {
            crate::runtime::ComponentHost::new(
                TransitionOwner {
                    show: true,
                    animated,
                    selection: None,
                    writes: 0,
                },
                app,
            )
        });
        let state = window.update(|window, app| {
            window.draw(app).clear(app);
            owner
                .read(app)
                .retained_entity_for_test::<ListState<KeyedListDelegate>>(
                    vec![],
                    Identity::from("transition-list"),
                )
                .unwrap()
        });
        (owner, window, state)
    }
    fn transition_draw(window: &mut gpui::VisualTestContext) {
        window.update(|window, app| {
            window.refresh();
            window.draw(app).clear(app);
        });
        window.cx.run_until_parked();
    }
    fn transition_frames(
        window: &mut gpui::VisualTestContext,
        duration: std::time::Duration,
    ) -> usize {
        std::thread::sleep(duration);
        window.cx.executor().advance_clock(duration);
        let count = window.update(|window, app| window.simulate_next_frame(app));
        window.cx.run_until_parked();
        transition_draw(window);
        count
    }
    fn transition_show(
        owner: &gpui::Entity<crate::runtime::ComponentHost<TransitionOwner>>,
        window: &mut gpui::VisualTestContext,
        show: bool,
    ) {
        owner.update(window, |owner, cx| {
            owner.component_mut().show = show;
            cx.notify();
        });
        transition_draw(window);
    }

    #[gpui::test]
    fn keyed_list_insertion_exit_and_reinsertion_use_one_native_spring(
        cx: &mut gpui::TestAppContext,
    ) {
        let (owner, window, state) = transition_setup(cx, false, true);
        let initial_x = window
            .debug_bounds("t17-exiting-list-row")
            .unwrap()
            .origin
            .x;
        window.update(|_, app| {
            assert!(state.read(app).delegate().animations["a"].progress.get() < 1.0)
        });
        transition_frames(window, std::time::Duration::from_secs(2));
        assert!(
            window
                .debug_bounds("t17-exiting-list-row")
                .unwrap()
                .origin
                .x
                > initial_x
        );
        window.update(|_, app| {
            assert_eq!(
                state.read(app).delegate().animations["a"].progress.get(),
                1.0
            )
        });
        transition_show(&owner, window, false);
        window.update(|_, app| {
            assert!(state.read(app).delegate().exits.contains_key("a"));
            assert_eq!(state.read(app).delegate().items.len(), 2);
        });
        transition_show(&owner, window, true);
        window.update(|_, app| {
            let delegate = state.read(app).delegate();
            assert!(!delegate.exits.contains_key("a"));
            assert_eq!(
                delegate.items.iter().filter(|item| item.id == "a").count(),
                1
            );
        });
        transition_show(&owner, window, false);
        transition_frames(window, std::time::Duration::from_secs(2));
        transition_frames(window, std::time::Duration::from_millis(20));
        window.update(|_, app| {
            let delegate = state.read(app).delegate();
            assert!(!delegate.exits.contains_key("a"));
            assert_eq!(delegate.items.len(), 1);
        });
    }

    #[gpui::test]
    fn keyed_list_exiting_row_cannot_be_selected(cx: &mut gpui::TestAppContext) {
        let (owner, window, state) = transition_setup(cx, false, true);
        transition_show(&owner, window, false);
        window.update(|window, app| {
            state.update(app, |state, cx| {
                state.set_selected_index(Some(IndexPath::new(0)), window, cx);
            });
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(state.read(app).selected_index(), None);
            assert_eq!(owner.read(app).component().selection, None);
            assert_eq!(owner.read(app).component().writes, 0);
        });
    }

    #[gpui::test]
    fn clicking_an_exiting_keyed_list_row_does_not_select_or_confirm(
        cx: &mut gpui::TestAppContext,
    ) {
        let (owner, window, state) = transition_setup(cx, false, true);
        transition_frames(window, std::time::Duration::from_secs(2));
        transition_show(&owner, window, false);
        let point = window
            .debug_bounds("t17-exiting-list-row")
            .expect("exit still rendered")
            .center();
        window.simulate_click(point, gpui::Modifiers::default());
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(state.read(app).selected_index(), None);
            assert_eq!(owner.read(app).component().selection, None);
            assert_eq!(owner.read(app).component().writes, 0);
        });
    }

    #[gpui::test]
    fn keyed_list_exit_moves_focus_and_blocks_refocus(cx: &mut gpui::TestAppContext) {
        let (owner, window, state) = transition_setup(cx, false, true);
        let focus =
            window.update(|_, app| state.read(app).delegate().animations["a"].focus.clone());
        window.update(|window, app| {
            focus.focus(window, app);
            assert_eq!(window.focused(app), Some(focus.clone()));
        });
        transition_show(&owner, window, false);
        window.update(|window, app| {
            assert_ne!(window.focused(app), Some(focus.clone()));
            let previous = window.focused(app);
            focus.focus(window, app);
            assert_eq!(window.focused(app), previous);
        });
        window.simulate_keystrokes("tab");
        window.update(|window, app| assert_ne!(window.focused(app), Some(focus.clone())));
        window.simulate_keystrokes("shift-tab");
        window.update(|window, app| assert_ne!(window.focused(app), Some(focus.clone())));
    }

    #[gpui::test]
    fn keyed_list_reduced_motion_and_no_transition_remove_immediately(
        cx: &mut gpui::TestAppContext,
    ) {
        let (owner, window, state) = transition_setup(cx, true, true);
        transition_show(&owner, window, false);
        window.update(|_, app| {
            assert!(state.read(app).delegate().exits.is_empty());
            assert_eq!(state.read(app).delegate().items.len(), 1);
        });
        assert_eq!(
            transition_frames(window, std::time::Duration::from_millis(30)),
            0
        );
        owner.update(window, |owner, cx| {
            owner.component_mut().animated = false;
            owner.component_mut().show = true;
            cx.notify();
        });
        transition_draw(window);
        transition_show(&owner, window, false);
        window.update(|_, app| {
            assert!(state.read(app).delegate().exits.is_empty());
            assert_eq!(state.read(app).delegate().items.len(), 1);
        });
    }
}
