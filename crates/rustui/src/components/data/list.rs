//! Controlled virtualized string list backed by retained GPUI ListState.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

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

fn selection_event_matches_current(event: &ListEvent, selected: Option<IndexPath>) -> bool {
    match event {
        ListEvent::Select(index) | ListEvent::Confirm(index) => selected == Some(*index),
        ListEvent::Cancel => selected.is_none(),
    }
}

fn clear_cancel_request(selectable: bool, requested: &Cell<Option<usize>>) -> bool {
    if !selectable {
        return false;
    }
    requested.set(None);
    true
}

fn clear_invalid_selection(current: &mut Option<usize>, invalid: usize) -> bool {
    if *current != Some(invalid) {
        return false;
    }
    *current = None;
    true
}

fn resolve_list_items(
    items: &[String],
    localized_labels: &[String],
    localized_suffixes: &[String],
    mut resolve: impl FnMut(&str) -> String,
) -> Vec<String> {
    debug_assert!(localized_labels.is_empty() || items.len() == localized_labels.len());
    debug_assert!(localized_suffixes.is_empty() || items.len() == localized_suffixes.len());
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let label = localized_labels
                .get(index)
                .map_or_else(|| item.clone(), |message_id| resolve(message_id));
            if let Some(message_id) = localized_suffixes.get(index) {
                format!("{label} · {}", resolve(message_id))
            } else {
                label
            }
        })
        .collect()
}

fn list_data_changed(
    current_items: &[String],
    current_row_height: f32,
    next_items: &[String],
    next_row_height: f32,
) -> bool {
    current_items != next_items || current_row_height != next_row_height
}

struct StringListDelegate {
    items: Vec<String>,
    visible: Vec<usize>,
    query: String,
    row_height: f32,
    owner_selection: Rc<Cell<Option<usize>>>,
    requested_selection: Rc<Cell<Option<usize>>>,
    selection_update: Rc<RefCell<Option<SelectionUpdater<App>>>>,
    searchable: bool,
    selectable: bool,
}

impl StringListDelegate {
    fn new(
        items: Vec<String>,
        row_height: f32,
        selection: Option<usize>,
        searchable: bool,
        selectable: bool,
    ) -> Self {
        let visible = (0..items.len()).collect();
        Self {
            items,
            visible,
            query: String::new(),
            row_height,
            owner_selection: Rc::new(Cell::new(selection)),
            requested_selection: Rc::new(Cell::new(None)),
            selection_update: Rc::new(RefCell::new(None)),
            searchable,
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

    fn options_match(&self, searchable: bool, selectable: bool) -> bool {
        self.searchable == searchable && self.selectable == selectable
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
    localized_labels: Vec<String>,
    localized_suffixes: Vec<String>,
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
        let items: Vec<String> = items.into_iter().map(Into::into).collect();
        assert!(
            items.iter().all(|item| !item.trim().is_empty()),
            "List item labels must be nonempty"
        );

        Self {
            items,
            localized_labels: Vec::new(),
            localized_suffixes: Vec::new(),
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
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "List identity must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        assert!(height.is_finite(), "List height must be finite");
        self.height = height.max(1.0);
        self
    }

    pub fn row_height(mut self, row_height: f32) -> Self {
        assert!(row_height.is_finite(), "List row height must be finite");
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

    /// Replaces each displayed item with an active-locale message while keeping
    /// source-index selection stable.
    pub fn localized_labels<I, S>(mut self, message_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.localized_labels = message_ids.into_iter().map(Into::into).collect();
        assert!(
            self.localized_labels.is_empty() || self.localized_labels.len() == self.items.len(),
            "Localized list labels must match item count"
        );
        assert!(
            self.localized_labels.iter().all(|id| !id.trim().is_empty()),
            "Localized list label IDs must be nonempty"
        );
        self
    }

    /// Appends the active-locale message for each corresponding item.
    pub fn localized_suffixes<I, S>(mut self, message_ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.localized_suffixes = message_ids.into_iter().map(Into::into).collect();
        assert!(
            self.localized_suffixes.is_empty() || self.localized_suffixes.len() == self.items.len(),
            "Localized list suffixes must match the item count"
        );
        assert!(
            self.localized_suffixes
                .iter()
                .all(|id| !id.trim().is_empty()),
            "Localized list suffix message IDs must be nonempty"
        );
        self
    }

    fn identity_with(&self, allocate: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            allocate(Identity::from(format!(
                "list:{}:{}",
                self.items.len(),
                self.items.first().map_or("", String::as_str)
            )))
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
        let identity = self.identity_with(|base| context.next_implicit_retained_identity(base));
        let items = resolve_list_items(
            &self.items,
            &self.localized_labels,
            &self.localized_suffixes,
            |message_id| context.localized_message(message_id),
        );
        let initial_selection = self.selection.value();
        let row_height = self.row_height;
        let searchable = self.searchable;
        let selectable = self.selectable;
        let initial_items = items.clone();
        let (state, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let delegate = StringListDelegate::new(
                initial_items,
                row_height,
                initial_selection,
                searchable,
                selectable,
            );
            let selected = delegate.visible_index(initial_selection);
            let mut state = ListState::new(delegate, window, cx)
                .searchable(searchable)
                .selectable(selectable);
            state.set_selected_index(selected, window, cx);
            state
        });

        let (owner_selection, requested_selection, selection_update) = {
            let delegate = state.read(context.app()).delegate();
            (
                delegate.owner_selection.clone(),
                delegate.requested_selection.clone(),
                delegate.selection_update.clone(),
            )
        };
        owner_selection.set(self.selection.value());
        let marker = owner_selection.clone();
        let current_request = requested_selection.clone();
        replace_selection_updater(
            &selection_update,
            context.binding_updater(self.selection.clone(), move |current| {
                apply_selection_request(current, &marker, current_request.get())
            }),
        );

        let data_changed = {
            let state = state.read(context.app());
            list_data_changed(
                &state.delegate().items,
                state.delegate().row_height,
                &items,
                self.row_height,
            )
        };
        if data_changed {
            let items = items.clone();
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
            let current_update = selection_update.clone();
            context.subscribe_retained(&identity, &state, move |state, event: &ListEvent, cx| {
                if !selection_event_matches_current(event, state.read(cx).selected_index()) {
                    return;
                }
                match event {
                    ListEvent::Select(index) | ListEvent::Confirm(index) => {
                        if let Some(source_index) = state.read(cx).delegate().source_index(*index) {
                            requested_selection.set(Some(source_index));
                            apply_selection_updater(&current_update, cx);
                        }
                    }
                    ListEvent::Cancel => {
                        let selectable = state.read(cx).delegate().selectable;
                        if clear_cancel_request(selectable, &requested_selection) {
                            apply_selection_updater(&current_update, cx);
                        }
                    }
                }
            });
        }

        let options_match = state
            .read(context.app())
            .delegate()
            .options_match(searchable, selectable);
        if !options_match {
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

        if let Some(invalid) = self
            .selection
            .value()
            .filter(|index| *index >= self.items.len())
        {
            let normalize = context.binding_updater(self.selection.clone(), move |current| {
                clear_invalid_selection(current, invalid)
            });
            context.update_entity(&state, move |_state, _window, cx| {
                cx.defer(move |cx| normalize(cx));
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

    struct KeyboardListOwner {
        selection: Option<usize>,
        writes: usize,
    }

    impl View for KeyboardListOwner {
        fn body(&self) -> impl View {
            EmptyView
        }

        fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
            let binding = Binding::new(
                self.selection,
                |owner: &Self| owner.selection,
                |owner, value| {
                    owner.selection = value;
                    owner.writes += 1;
                },
            );
            div()
                .w(px(320.0))
                .h(px(280.0))
                .child(
                    List(["first", "second", "third"], binding)
                        .id("keyboard-list")
                        .height(260.0)
                        .into_element(context),
                )
                .into_any_element()
        }
    }

    #[gpui::test]
    fn keyboard_selection_updates_controlled_owner(cx: &mut gpui::TestAppContext) {
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
                KeyboardListOwner {
                    selection: Some(0),
                    writes: 0,
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            let state = owner
                .read(app)
                .retained_entity_for_test::<ListState<StringListDelegate>>(
                    vec![],
                    Identity::from("keyboard-list"),
                )
                .expect("rendered List state");
            state.update(app, |state, cx| state.focus(window, cx));
        });
        window.simulate_keystrokes("down");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().selection, Some(1));
            assert_eq!(owner.read(app).component().writes, 1);
        });

        window.update(|window, app| {
            let state = owner
                .read(app)
                .retained_entity_for_test::<ListState<StringListDelegate>>(
                    vec![],
                    Identity::from("keyboard-list"),
                )
                .unwrap();
            state.update(app, |state, cx| {
                state.delegate_mut().query = "ir".to_owned();
                state.delegate_mut().apply_query();
                cx.notify();
            });
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("down");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().selection, Some(0));
        });
        window.simulate_keystrokes("down");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().selection, Some(2));
        });
    }

    struct Owner;

    fn selection_binding() -> Binding<Option<usize>> {
        Binding::new(None, |_: &Owner| None, |_: &mut Owner, _| {})
    }

    #[test]
    fn implicit_list_identity_distinguishes_siblings_and_is_stable_by_slot() {
        let list = ListView::new(["first", "second"], selection_binding());
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first = list.identity_with(allocate_at_slot(0));
        let second = list.identity_with(allocate_at_slot(1));
        let first_after_rerender = list.identity_with(allocate_at_slot(0));

        assert_ne!(first, second);
        assert_eq!(first, first_after_rerender);
    }

    #[test]
    #[should_panic(expected = "List item labels must be nonempty")]
    fn blank_row_accessible_names_are_rejected() {
        ListView::new(["  ", "Valid row"], selection_binding());
    }

    #[test]
    fn nonblank_row_labels_are_preserved_verbatim() {
        let view = ListView::new(["  Valid row  "], selection_binding());

        assert_eq!(view.items, ["  Valid row  "]);
    }

    #[test]
    fn localized_labels_replace_rows_and_trigger_retained_list_update() {
        let view = ListView::new(["Text inputs"], selection_binding())
            .localized_labels(["lab-story-input-title"]);
        let english = resolve_list_items(&view.items, &view.localized_labels, &[], |_| {
            String::from("Text inputs")
        });
        let french = resolve_list_items(&view.items, &view.localized_labels, &[], |_| {
            String::from("Champs de texte")
        });
        assert_eq!(french, ["Champs de texte"]);
        assert!(list_data_changed(&english, 36.0, &french, 36.0));
    }

    #[test]
    fn localized_suffixes_preserve_item_prefixes_and_resolve_by_active_locale() {
        let message_id = "lab-catalog-coverage-partial";
        let view = ListView::new(["slider"], selection_binding()).localized_suffixes([message_id]);
        let mut catalogs =
            rustui_i18n::Catalogs::new(rustui_i18n::Locale::parse("en").expect("English locale"));
        for (locale, label) in [("en", "Partial"), ("fr", "Partiel"), ("ar", "جزئي")] {
            catalogs
                .insert(
                    rustui_i18n::Catalog::parse(
                        rustui_i18n::Locale::parse(locale).expect("test locale"),
                        &format!("lab-catalog-coverage-partial = {label}"),
                    )
                    .expect("test catalog"),
                )
                .expect("insert test catalog");
        }

        for (locale, expected) in [
            ("en", "slider · Partial"),
            ("fr", "slider · Partiel"),
            ("ar", "slider · جزئي"),
        ] {
            let locale = rustui_i18n::Locale::parse(locale).expect("test locale");
            let label =
                resolve_list_items(&view.items, &[], &view.localized_suffixes, |message_id| {
                    catalogs.resolve(&locale, message_id).to_owned()
                });
            assert_eq!(label, [expected]);
        }
    }

    #[test]
    fn localized_rows_remain_stable_until_the_resolved_locale_changes() {
        let items = vec![String::from("slider")];
        let suffixes = vec![String::from("coverage-partial")];
        let english = resolve_list_items(&items, &[], &suffixes, |_| String::from("Partial"));
        let french = resolve_list_items(&items, &[], &suffixes, |_| String::from("Partiel"));

        assert!(!list_data_changed(&english, 36.0, &english, 36.0));
        assert!(list_data_changed(&english, 36.0, &french, 36.0));
    }

    #[test]
    fn retained_delegate_keeps_resolved_rows_across_rerender_and_locale_change() {
        let english = vec![String::from("slider · Partial")];
        let french = vec![String::from("slider · Partiel")];
        let mut delegate = StringListDelegate::new(english.clone(), 36.0, None, false, true);

        if list_data_changed(&delegate.items, delegate.row_height, &english, 36.0) {
            delegate.replace_items(english.clone(), 36.0);
        }
        assert_eq!(delegate.items, english);

        if list_data_changed(&delegate.items, delegate.row_height, &french, 36.0) {
            delegate.replace_items(french.clone(), 36.0);
        }
        assert_eq!(delegate.items, french);
    }

    #[test]
    #[should_panic(expected = "List identity must be nonempty")]
    fn rejects_whitespace_only_retained_identity() {
        ListView::new(["row"], selection_binding()).id("  \t");
    }

    #[test]
    fn retained_list_options_detect_search_and_selection_changes() {
        let delegate = StringListDelegate::new(Vec::new(), 36.0, None, false, true);

        assert!(delegate.options_match(false, true));
        assert!(!delegate.options_match(true, true));
        assert!(!delegate.options_match(false, false));
    }

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
    fn queued_events_do_not_overwrite_reconciled_owner_selection() {
        let expected = Cell::new(Some(0));
        let mut owner = Some(2);
        // Reconciliation has already observed the newer owner and selected its row.
        expected.set(owner);
        let selected = Some(IndexPath::new(2));
        for event in [
            ListEvent::Select(IndexPath::new(1)),
            ListEvent::Confirm(IndexPath::new(1)),
            ListEvent::Cancel,
        ] {
            if selection_event_matches_current(&event, selected) {
                let requested = match event {
                    ListEvent::Select(_) | ListEvent::Confirm(_) => Some(1),
                    ListEvent::Cancel => None,
                };
                apply_selection_request(&mut owner, &expected, requested);
            }
            assert_eq!(owner, Some(2));
        }

        // A subsequent matching UI selection and cancellation still reach the owner.
        let next = IndexPath::new(1);
        assert!(selection_event_matches_current(
            &ListEvent::Select(next),
            Some(next)
        ));
        assert!(apply_selection_request(&mut owner, &expected, Some(1)));
        assert!(selection_event_matches_current(&ListEvent::Cancel, None));
        assert!(apply_selection_request(&mut owner, &expected, None));
        assert_eq!(owner, None);
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

    #[test]
    fn invalid_source_index_is_normalized_after_items_are_removed() {
        let mut owner = Some(3);

        assert!(clear_invalid_selection(&mut owner, 3));
        assert_eq!(owner, None);
    }

    #[test]
    fn stale_invalid_source_index_does_not_clear_a_newer_selection() {
        let mut owner = Some(1);

        assert!(!clear_invalid_selection(&mut owner, 3));
        assert_eq!(owner, Some(1));
    }

    #[test]
    fn retained_selection_event_uses_latest_owner_binding_writer() {
        let slot = RefCell::new(None);
        let old_writer_called = Rc::new(Cell::new(false));
        let new_writer_called = Rc::new(Cell::new(false));

        let old = old_writer_called.clone();
        replace_selection_updater(&slot, Rc::new(move |_: &mut ()| old.set(true)));
        let new = new_writer_called.clone();
        replace_selection_updater(&slot, Rc::new(move |_: &mut ()| new.set(true)));

        apply_selection_updater(&slot, &mut ());

        assert!(!old_writer_called.get());
        assert!(new_writer_called.get());
    }
}
