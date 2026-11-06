//! Keyed, fixed-height, virtualized rows containing arbitrary RustUI views.

use std::{cell::RefCell, collections::HashSet, marker::PhantomData, rc::Rc};

use gpui::{
    AnyElement, App, Bounds, ClickEvent, Element, ElementId, FocusHandle, GlobalElementId,
    InspectorElementId, InteractiveElement as _, IntoElement, KeyDownEvent, LayoutId,
    ListAlignment, ListOffset, ListState, Modifiers, MouseButton, ParentElement as _, Pixels, Role,
    SharedString, StatefulInteractiveElement as _, Styled as _, WeakEntity, Window, div, list, px,
};
use gpui_component::ActiveTheme as _;
use rustui_core::{Binding, Identity};

use crate::{
    runtime::component_host::RetainedEntities,
    view::{
        ControlAction, EmptyView, KeyboardActionHandler, RenderContext, RenderSeed, RetentionScope,
        View,
    },
};

type SelectionWriter = Rc<dyn Fn(Option<Identity>, &mut App)>;

/// A snapshot value identified independently of its position.
fn collection_list_element(
    collection_id: &Identity,
    label: SharedString,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(format!("rustui-collection:{}", collection_id.as_str()))
        .role(Role::List)
        .aria_label(label)
}

fn collection_row_element(
    id: &str,
    label: SharedString,
    position: usize,
    set_size: usize,
    selected: bool,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(id.to_owned())
        .role(Role::ListItem)
        .aria_label(label)
        .aria_position_in_set(position)
        .aria_size_of_set(set_size)
        .aria_selected(selected)
}

pub struct CollectionItem<T> {
    id: Identity,
    value: T,
    label: SharedString,
}

impl<T> CollectionItem<T> {
    /// The stable ID is also the default accessible name; use `label` for a human name.
    pub fn new(id: impl Into<Identity>, value: T) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Collection item identity must be nonempty"
        );
        Self {
            label: id.as_str().to_owned().into(),
            id,
            value,
        }
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        assert!(
            !self.label.trim().is_empty(),
            "Collection item label must be nonempty"
        );
        self
    }
    pub fn id(&self) -> &Identity {
        &self.id
    }
    pub fn value(&self) -> &T {
        &self.value
    }
}

/// Virtualizes equal-height rows using GPUI's own layout and scrolling.
///
/// Factories may run repeatedly for GPUI-requested visible rows.
/// Keep mutations in callbacks. Offscreen native controls and child component state
/// are released; persist durable values in the caller's model and pass bindings.
/// The supplied snapshot is authoritative: filtering it removes missing IDs and
/// clears a removed selection. There is no built-in search/hidden selection store.
pub struct Collection<T: 'static, V: View, F: Fn(usize, &T) -> V + 'static> {
    id: Identity,
    items: Rc<[CollectionItem<T>]>,
    height: f32,
    row_height: f32,
    factory: Rc<F>,
    selection: Option<Binding<Option<Identity>>>,
    label: SharedString,
    confirm: Option<ControlAction>,
    select_on_click: bool,
    view: PhantomData<fn() -> V>,
}

impl<T: 'static, V: View, F: Fn(usize, &T) -> V + 'static> Collection<T, V, F> {
    pub fn new(
        id: impl Into<Identity>,
        items: Rc<[CollectionItem<T>]>,
        viewport_height: f32,
        row_height: f32,
        factory: F,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Collection identity must be nonempty"
        );
        validate_height(viewport_height);
        validate_height(row_height);
        item_ids(&items);
        Self {
            id,
            items,
            height: viewport_height,
            row_height,
            factory: Rc::new(factory),
            selection: None,
            label: "Collection".into(),
            confirm: None,
            select_on_click: true,
            view: PhantomData,
        }
    }
    /// Enables a single native tab stop and owner-controlled stable-key selection.
    pub fn selection(mut self, selection: Binding<Option<Identity>>) -> Self {
        self.selection = Some(selection);
        self
    }
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        assert!(
            !self.label.trim().is_empty(),
            "Collection label must be nonempty"
        );
        self
    }
    /// Enter invokes this action while the collection itself has keyboard focus.
    /// Without a selection binding, this enables a native tab stop and confirms
    /// the collection. With selection enabled, Enter requires a selected row.
    pub fn on_confirm(mut self, action: impl Into<ControlAction>) -> Self {
        let action = action.into();
        assert!(
            !matches!(&action, ControlAction::Application(a) if !a.keyboard_compatible()),
            "Collection confirmation requires a keyboard-compatible action"
        );
        self.confirm = Some(action);
        self
    }
    /// Unconsumed child clicks bubble to row selection, following native GPUI rules.
    pub fn select_on_click(mut self, enabled: bool) -> Self {
        self.select_on_click = enabled;
        self
    }
}

fn validate_height(value: f32) {
    assert!(
        value.is_finite() && value > 0.0,
        "Collection dimensions must be finite and positive"
    );
}

fn item_ids<T>(items: &[CollectionItem<T>]) -> Vec<Identity> {
    let mut seen = HashSet::with_capacity(items.len());
    items
        .iter()
        .map(|item| {
            assert!(
                seen.insert(item.id.clone()),
                "Collection item identity used twice"
            );
            item.id.clone()
        })
        .collect()
}

fn normalize(ids: &[Identity], selected: Option<&Identity>) -> Option<Identity> {
    selected.filter(|id| ids.contains(id)).cloned()
}

fn update_selection(
    current: &mut Option<Identity>,
    expected: &RefCell<Option<Identity>>,
    desired: &Option<Identity>,
) -> bool {
    if *current != *expected.borrow() || current == desired {
        return false;
    }
    *current = desired.clone();
    *expected.borrow_mut() = desired.clone();
    true
}

fn next_selection(
    ids: &[Identity],
    selected: Option<&Identity>,
    key: &str,
) -> Option<Option<Identity>> {
    if ids.is_empty() && key != "escape" {
        return None;
    }
    let index = selected.and_then(|id| ids.iter().position(|item| item == id));
    let target = match key {
        "escape" => return Some(None),
        "home" => 0,
        "end" => ids.len().saturating_sub(1),
        "up" => index.map_or(ids.len().saturating_sub(1), |i| i.saturating_sub(1)),
        "down" => index.map_or(0, |i| (i + 1).min(ids.len().saturating_sub(1))),
        _ => return None,
    };
    Some(ids.get(target).cloned())
}

#[derive(Debug, PartialEq, Eq)]
enum KeyboardIntent {
    Confirm,
    Navigate,
    Ignore,
}

fn keyboard_event_eligible(
    focused: bool,
    prefer_character_input: bool,
    ime_in_progress: bool,
    modifiers: &Modifiers,
) -> bool {
    focused
        && !prefer_character_input
        && !ime_in_progress
        && !modifiers.control
        && !modifiers.platform
        && !modifiers.alt
        && !modifiers.shift
}

fn keyboard_intent(
    key: &str,
    selectable: bool,
    selected: bool,
    confirmable: bool,
) -> KeyboardIntent {
    match key {
        "enter" if confirmable && (!selectable || selected) => KeyboardIntent::Confirm,
        "up" | "down" | "home" | "end" | "escape" if selectable => KeyboardIntent::Navigate,
        _ => KeyboardIntent::Ignore,
    }
}

/// Native List discards uniform height hints when its width first resolves or
/// changes. Height-based reveal cannot locate an unmeasured offscreen row then;
/// use GPUI's logical index scroll when the path to the target has unknown heights.
fn reveal_selected_row(scroll: &ListState, index: usize) {
    let first = scroll.logical_scroll_top().item_ix;
    let measured_path = (first..=index).all(|row| scroll.bounds_for_item(row).is_some());
    if measured_path {
        scroll.scroll_to_reveal_item(index);
    } else {
        scroll.scroll_to(ListOffset {
            item_ix: index,
            offset_in_item: px(0.0),
        });
    }
}

struct CollectionState {
    seed: RenderSeed,
    retained: RetainedEntities,
    ids: Vec<Identity>,
    selected: Option<Identity>,
    writer: Option<SelectionWriter>,
    confirm: Option<KeyboardActionHandler>,
    focus: FocusHandle,
    scroll: ListState,
    row_height: f32,
    generation: u64,
}

impl CollectionState {
    fn select(&self, desired: Option<Identity>, _window: &mut Window, cx: &mut App) {
        if desired.as_ref().is_some_and(|id| !self.ids.contains(id)) {
            return;
        }
        if let Some(writer) = &self.writer {
            if let Some(index) = desired
                .as_ref()
                .and_then(|id| self.ids.iter().position(|i| i == id))
            {
                reveal_selected_row(&self.scroll, index);
            }
            writer(desired, cx);
        }
    }

    fn keyboard(&self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
        let modifiers = &event.keystroke.modifiers;
        if !keyboard_event_eligible(
            self.focus.is_focused(window),
            event.prefer_character_input,
            event.keystroke.is_ime_in_progress(),
            modifiers,
        ) {
            return;
        }
        match keyboard_intent(
            &event.keystroke.key,
            self.writer.is_some(),
            self.selected.is_some(),
            self.confirm.is_some(),
        ) {
            KeyboardIntent::Confirm => {
                if let Some(confirm) = &self.confirm {
                    confirm(window, cx);
                }
                cx.stop_propagation();
            }
            KeyboardIntent::Navigate => {
                if let Some(next) =
                    next_selection(&self.ids, self.selected.as_ref(), &event.keystroke.key)
                {
                    self.select(next, window, cx);
                    cx.stop_propagation();
                }
            }
            KeyboardIntent::Ignore => {}
        }
    }
}

/// Lifecycle forwarding only: GPUI owns every measurement, layout, and scroll.
/// Row visits are recorded in prepaint, even if GPUI reuses a rendered element.
#[derive(Default)]
struct PrepaintVisits(RefCell<HashSet<Identity>>);

impl PrepaintVisits {
    fn begin_list(&self) {
        self.0.borrow_mut().clear();
    }
    fn visit_row(&self, id: &Identity) {
        self.0.borrow_mut().insert(id.clone());
    }
}

struct CollectionElement {
    child: AnyElement,
    visits: Rc<PrepaintVisits>,
    kind: VisitKind,
}

enum VisitKind {
    Row(Identity),
    List {
        owner: WeakEntity<CollectionState>,
        generation: u64,
    },
}

impl IntoElement for CollectionElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for CollectionElement {
    type RequestLayoutState = ();
    type PrepaintState = ();
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        match &self.kind {
            VisitKind::Row(id) => {
                self.visits.visit_row(id);
            }
            VisitKind::List { .. } => self.visits.begin_list(),
        }
        self.child.prepaint(window, cx);
        if let VisitKind::List { owner, generation } = &self.kind {
            owner
                .update(cx, |state, _| {
                    if state.generation == *generation {
                        state.retained.prune_cells(&self.visits.0.borrow());
                    }
                })
                .ok();
        }
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        _: &mut (),
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        self.child.paint(window, cx);
    }
}

impl<T: 'static, V: View, F: Fn(usize, &T) -> V + 'static> View for Collection<T, V, F> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Collection(self.id.clone()), |context| {
            let ids = item_ids(&self.items);
            let requested = self.selection.as_ref().and_then(Binding::value);
            let selected = normalize(&ids, requested.as_ref());
            let seed = context.seed();
            let (state, _) =
                context.retained_entity(Identity::from("collection-state"), |_, cx| {
                    CollectionState {
                        seed: seed.clone(),
                        retained: RetainedEntities::default(),
                        ids: Vec::new(),
                        selected: None,
                        writer: None,
                        confirm: None,
                        focus: cx.focus_handle(),
                        scroll: ListState::new(self.items.len(), ListAlignment::Top, px(0.0))
                            .with_uniform_item_height(px(self.row_height)),
                        row_height: self.row_height,
                        generation: 0,
                    }
                });
            let writer: Option<SelectionWriter> = self.selection.as_ref().map(|binding| {
                let target = Rc::new(RefCell::new(None));
                let read_target = target.clone();
                let expected = Rc::new(RefCell::new(requested.clone()));
                let update_expected = expected.clone();
                let update = context.binding_updater(binding.clone(), move |current| {
                    let desired = read_target.borrow();
                    update_selection(current, &update_expected, &desired)
                });
                Rc::new(move |desired, cx: &mut App| {
                    *target.borrow_mut() = desired;
                    update(cx);
                }) as SelectionWriter
            });
            let confirm = self
                .confirm
                .as_ref()
                .map(|action| context.bind_keyboard_action(action.clone()));
            let (generation, focus, scroll) = context.update_entity(&state, |state, _, _| {
                let previous_index = state
                    .selected
                    .as_ref()
                    .and_then(|id| state.ids.iter().position(|i| i == id));
                let next_index = selected
                    .as_ref()
                    .and_then(|id| ids.iter().position(|i| i == id));
                let resized =
                    state.scroll.item_count() != ids.len() || state.row_height != self.row_height;
                if resized {
                    state
                        .scroll
                        .reset_with_uniform_height(ids.len(), px(self.row_height));
                    state.row_height = self.row_height;
                }
                if resized || state.selected != selected || previous_index != next_index {
                    if let Some(index) = next_index {
                        reveal_selected_row(&state.scroll, index);
                    }
                }
                state.retained.prune_cells(&ids.iter().cloned().collect());
                state.seed = seed;
                state.ids = ids;
                state.selected = selected.clone();
                state.writer = writer.clone();
                state.confirm = confirm;
                state.generation = state.generation.wrapping_add(1);
                (
                    state.generation,
                    state
                        .focus
                        .clone()
                        .tab_stop(self.selection.is_some() || self.confirm.is_some()),
                    state.scroll.clone(),
                )
            });
            if requested != selected
                && let (Some(missing), Some(writer)) = (requested, writer)
            {
                let current_state = state.clone();
                context.update_entity(&state, |_, _, cx| {
                    cx.defer(move |cx| {
                        if !current_state.read(cx).ids.contains(&missing) {
                            writer(None, cx);
                        }
                    })
                });
            }
            let weak = state.downgrade();
            let keyboard_owner = weak.clone();
            let items = self.items.clone();
            let factory = self.factory.clone();
            let row_height = self.row_height;
            let select_on_click = self.select_on_click;
            let visits = Rc::new(PrepaintVisits::default());
            let row_visits = visits.clone();
            let native_list = list(scroll, move |index, window, cx| {
                weak.update(cx, |state, cx| {
                    let Some(item) = items.get(index) else {
                        return div().into_any_element();
                    };
                    if state.generation != generation {
                        return div().into_any_element();
                    }
                    let instance = cx.entity_id();
                    state.retained.begin_cell(&item.id);
                    let seed = state.seed.clone();
                    let mut context =
                        seed.cell_context(instance, &item.id, window, cx, &mut state.retained);
                    let child = context
                        .with_element_scope(RetentionScope::Key(item.id.clone()), |context| {
                            factory(index, &item.value).into_element(context)
                        });
                    drop(context);
                    state.retained.finish_cell(&item.id);
                    let selected = state.selected.as_ref() == Some(&item.id);
                    let mut row = collection_row_element(
                        item.id.as_str(),
                        item.label.clone(),
                        index + 1,
                        items.len(),
                        selected,
                    )
                    .w_full()
                    .h(px(row_height))
                    .overflow_hidden()
                    .child(child);
                    if selected {
                        row = row.bg(cx.theme().list_active);
                    }
                    if selected && state.focus.is_focused(window) {
                        row = row.aria_active_descendant();
                    }
                    if state.writer.is_some() && select_on_click {
                        let focus = state.focus.clone();
                        row = row.on_mouse_down(MouseButton::Left, move |_, window, cx| {
                            // Descendant editors/buttons own mouse-down focus when they prevent default.
                            if !window.default_prevented() {
                                focus.focus(window, cx);
                                window.prevent_default();
                            }
                        });
                        let owner = cx.weak_entity();
                        let id = item.id.clone();
                        row = row.on_click(move |event, window, cx| {
                            owner
                                .update(cx, |state, cx| {
                                    if state.generation == generation {
                                        if matches!(event, ClickEvent::Keyboard(_)) {
                                            state.focus.focus(window, cx);
                                        }
                                        state.select(Some(id.clone()), window, cx);
                                    }
                                })
                                .ok();
                        });
                    }
                    CollectionElement {
                        child: row.into_any_element(),
                        visits: row_visits.clone(),
                        kind: VisitKind::Row(item.id.clone()),
                    }
                    .into_any_element()
                })
                .unwrap_or_else(|_| div().into_any_element())
            })
            .h(px(self.height))
            .w_full()
            .min_w_0();
            let list = CollectionElement {
                child: native_list.into_any_element(),
                visits,
                kind: VisitKind::List {
                    owner: state.downgrade(),
                    generation,
                },
            };

            collection_list_element(&self.id, self.label.clone())
                .track_focus(&focus)
                .on_key_down(move |event, window, cx| {
                    keyboard_owner
                        .update(cx, |state, cx| {
                            if state.generation == generation {
                                state.keyboard(event, window, cx);
                            }
                        })
                        .ok();
                })
                .child(list)
                .into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use gpui::{Element, Modifiers, Role, accesskit::Node};

    use super::*;

    struct RestoredCollectionOwner {
        selection: Option<Identity>,
        present: bool,
        writes: usize,
    }

    impl View for RestoredCollectionOwner {
        fn body(&self) -> impl View {
            crate::view::EmptyView
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
            let items: Rc<[CollectionItem<()>]> = self
                .present
                .then(|| CollectionItem::new("a", ()))
                .into_iter()
                .collect();
            Collection::new("restored", items, 200.0, 40.0, |_, _| {
                crate::view::EmptyView
            })
            .selection(selection)
            .into_element(context)
        }
    }

    #[gpui::test]
    fn restored_collection_item_survives_deferred_clear(cx: &mut gpui::TestAppContext) {
        cx.update(gpui_component::init);
        let (owner, window) = cx.add_window_view(|_, app| {
            crate::runtime::ComponentHost::new(
                RestoredCollectionOwner {
                    selection: Some(Identity::from("a")),
                    present: true,
                    writes: 0,
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            owner.update(app, |host, cx| {
                host.component_mut().present = false;
                cx.notify();
            });
            let _ = window.draw(app);
            owner.update(app, |host, cx| {
                host.component_mut().present = true;
                cx.notify();
            });
            let _ = window.draw(app);
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(
                owner.read(app).component().selection,
                Some(Identity::from("a"))
            );
            assert_eq!(owner.read(app).component().writes, 0);
        });
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().present = false;
                cx.notify();
            });
            let _ = window.draw(app);
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(owner.read(app).component().selection, None);
            assert_eq!(owner.read(app).component().writes, 1);
        });
    }

    fn accesskit_node(element: &impl Element) -> Node {
        let role = element
            .a11y_role()
            .expect("collection accessibility element has a role");
        let mut node = Node::new(role);
        element.write_a11y_info(&mut node);
        node
    }

    #[test]
    fn collection_elements_emit_list_and_list_item_accesskit_metadata() {
        let identity = Identity::from("saved-views");
        let list = accesskit_node(&collection_list_element(&identity, "Saved views".into()));
        assert_eq!(list.role(), Role::List);
        assert_eq!(list.label(), Some("Saved views"));

        let first = accesskit_node(&collection_row_element(
            "view-1",
            "Overview".into(),
            1,
            2,
            true,
        ));
        assert_eq!(first.role(), Role::ListItem);
        assert_eq!(first.label(), Some("Overview"));
        assert_eq!(first.position_in_set(), Some(1));
        assert_eq!(first.size_of_set(), Some(2));
        assert_eq!(first.is_selected(), Some(true));

        let second = accesskit_node(&collection_row_element(
            "view-2",
            "Activity".into(),
            2,
            2,
            false,
        ));
        assert_eq!(second.position_in_set(), Some(2));
        assert_eq!(second.size_of_set(), Some(2));
        assert_eq!(second.is_selected(), Some(false));

        let other = collection_list_element(&Identity::from("recent-files"), "Recent files".into());
        let element = collection_list_element(&identity, "Saved views".into());
        assert_ne!(Element::id(&element), Element::id(&other));
    }

    #[test]
    fn selection_scroll_addresses_unmeasured_native_rows_by_index() {
        // This matches native List's state after a width change invalidates all
        // height hints, including the first prepaint of a newly created list.
        let scroll = ListState::new(1_000, ListAlignment::Top, px(0.0));
        scroll.scroll_to_reveal_item(999);
        assert_eq!(scroll.logical_scroll_top().item_ix, 0);
        reveal_selected_row(&scroll, 999);
        assert_eq!(scroll.logical_scroll_top().item_ix, 999);
        assert_eq!(scroll.logical_scroll_top().offset_in_item, px(0.0));
        reveal_selected_row(&scroll, 500);
        assert_eq!(scroll.logical_scroll_top().item_ix, 500);
        reveal_selected_row(&scroll, 0);
        assert_eq!(scroll.logical_scroll_top().item_ix, 0);
    }

    #[test]
    fn prepaint_visits_follow_reused_rows_and_evict_prior_ranges() {
        let visits = PrepaintVisits::default();
        let a = Identity::from("a");
        let b = Identity::from("b");
        visits.begin_list();
        visits.visit_row(&a);
        visits.visit_row(&b);
        assert_eq!(*visits.0.borrow(), HashSet::from([a.clone(), b.clone()]));
        // The same row marker can be prepainted again without a new factory call.
        visits.begin_list();
        visits.visit_row(&b);
        visits.visit_row(&b);
        assert_eq!(*visits.0.borrow(), HashSet::from([b]));
        visits.begin_list();
        assert!(visits.0.borrow().is_empty());
    }

    #[test]
    fn keyboard_decision_helpers_cover_navigation_escape_confirmation_and_ignored_input() {
        let ids = vec![
            Identity::from("a"),
            Identity::from("b"),
            Identity::from("c"),
        ];

        assert_eq!(
            next_selection(&ids, Some(&ids[0]), "up"),
            Some(Some(ids[0].clone()))
        );
        assert_eq!(
            next_selection(&ids, Some(&ids[2]), "down"),
            Some(Some(ids[2].clone()))
        );
        assert_eq!(next_selection(&ids, None, "up"), Some(Some(ids[2].clone())));
        assert_eq!(
            next_selection(&ids, None, "down"),
            Some(Some(ids[0].clone()))
        );
        assert_eq!(
            next_selection(&ids, Some(&ids[1]), "home"),
            Some(Some(ids[0].clone()))
        );
        assert_eq!(
            next_selection(&ids, Some(&ids[1]), "end"),
            Some(Some(ids[2].clone()))
        );
        assert_eq!(next_selection(&ids, Some(&ids[1]), "escape"), Some(None));

        for key in ["up", "down", "home", "end"] {
            assert_eq!(next_selection(&[], None, key), None);
        }
        assert_eq!(next_selection(&[], None, "escape"), Some(None));

        assert_eq!(
            keyboard_intent("enter", false, false, true),
            KeyboardIntent::Confirm
        );
        assert_eq!(
            keyboard_intent("enter", true, false, true),
            KeyboardIntent::Ignore
        );
        assert_eq!(
            keyboard_intent("enter", true, true, true),
            KeyboardIntent::Confirm
        );
        assert_eq!(
            keyboard_intent("enter", false, false, false),
            KeyboardIntent::Ignore
        );

        for key in ["a", "é", "ime"] {
            assert_eq!(next_selection(&ids, Some(&ids[1]), key), None);
            assert_eq!(
                keyboard_intent(key, true, true, true),
                KeyboardIntent::Ignore
            );
        }
    }

    #[test]
    fn confirmation_without_selection_has_a_functional_keyboard_path() {
        assert_eq!(
            keyboard_intent("enter", false, false, true),
            KeyboardIntent::Confirm
        );
        for key in ["up", "down", "home", "end", "escape"] {
            assert_eq!(
                keyboard_intent(key, false, false, true),
                KeyboardIntent::Ignore
            );
            assert_eq!(
                keyboard_intent(key, true, false, true),
                KeyboardIntent::Navigate
            );
        }
        assert_eq!(
            keyboard_intent("enter", true, false, true),
            KeyboardIntent::Ignore
        );
        assert_eq!(
            keyboard_intent("enter", true, true, true),
            KeyboardIntent::Confirm
        );
        assert_eq!(
            keyboard_intent("enter", false, false, false),
            KeyboardIntent::Ignore
        );
    }

    #[test]
    fn keyboard_event_filter_rejects_unfocused_text_ime_and_modified_input() {
        let plain = Modifiers::default();
        assert!(keyboard_event_eligible(true, false, false, &plain));
        assert!(!keyboard_event_eligible(false, false, false, &plain));
        assert!(!keyboard_event_eligible(true, true, false, &plain));
        assert!(!keyboard_event_eligible(true, false, true, &plain));

        for modifiers in [
            Modifiers {
                control: true,
                ..plain
            },
            Modifiers {
                platform: true,
                ..plain
            },
            Modifiers { alt: true, ..plain },
            Modifiers {
                shift: true,
                ..plain
            },
        ] {
            assert!(!keyboard_event_eligible(true, false, false, &modifiers));
        }
    }

    #[test]
    fn selection_writes_check_the_current_owner_value() {
        let a = Some(Identity::from("a"));
        let b = Some(Identity::from("b"));
        let expected = RefCell::new(a.clone());
        let mut owner = b.clone();
        assert!(!update_selection(&mut owner, &expected, &None));
        assert_eq!(owner, b);
        *expected.borrow_mut() = b.clone();
        assert!(update_selection(&mut owner, &expected, &a));
        assert!(!update_selection(&mut owner, &expected, &a));
        // Equality is an ordinary stale guard, not A-B-A mutation provenance.
        assert!(update_selection(&mut owner, &expected, &None));
    }

    #[test]
    fn consecutive_selection_updates_advance_the_reconciliation_marker() {
        let a = Some(Identity::from("a"));
        let b = Some(Identity::from("b"));
        let c = Some(Identity::from("c"));
        let expected = RefCell::new(a.clone());
        let mut owner = a;

        assert!(update_selection(&mut owner, &expected, &b));
        assert_eq!(owner, b);
        assert_eq!(*expected.borrow(), owner);

        assert!(update_selection(&mut owner, &expected, &c));
        assert_eq!(owner, c);
        assert_eq!(*expected.borrow(), owner);
    }
    #[test]
    fn validates_ids_dimensions_and_owner_removal() {
        assert!(std::panic::catch_unwind(|| CollectionItem::new(" ", 0)).is_err());
        assert!(
            std::panic::catch_unwind(|| item_ids(&[
                CollectionItem::new("a", 0),
                CollectionItem::new("a", 1)
            ]))
            .is_err()
        );
        for invalid in [0.0, -1.0, f32::INFINITY, f32::NAN] {
            assert!(std::panic::catch_unwind(|| validate_height(invalid)).is_err());
        }
        let a = Identity::from("a");
        let b = Identity::from("b");
        assert_eq!(
            normalize(&[b.clone(), a.clone()], Some(&a)),
            Some(a.clone())
        );
        assert_eq!(normalize(&[b], Some(&a)), None);
    }
    #[test]
    fn navigation_follows_keys_and_handles_empty_snapshots() {
        let ids = vec![
            Identity::from("a"),
            Identity::from("b"),
            Identity::from("c"),
        ];
        assert_eq!(
            next_selection(&ids, Some(&ids[1]), "down"),
            Some(Some(ids[2].clone()))
        );
        assert_eq!(
            next_selection(&ids, Some(&ids[2]), "down"),
            Some(Some(ids[2].clone()))
        );
        assert_eq!(
            next_selection(&ids, Some(&ids[1]), "home"),
            Some(Some(ids[0].clone()))
        );
        assert_eq!(
            next_selection(&ids, None, "end"),
            Some(Some(ids[2].clone()))
        );
        assert_eq!(next_selection(&ids, None, "escape"), Some(None));
        assert_eq!(next_selection(&[], None, "down"), None);
        assert_eq!(next_selection(&ids, None, "x"), None);
    }
}
