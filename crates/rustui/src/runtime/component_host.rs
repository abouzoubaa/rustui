//! GPUI entity-backed ownership for root and retained child components.

use std::{
    any::{Any, TypeId, type_name},
    collections::{HashMap, HashSet},
    rc::Rc,
    time::Instant,
};

use gpui::{App, Context, Entity, IntoElement, Render, Subscription, Window};
use rustui_core::{EnvironmentValues, Identity};

use super::{render_diagnostics, tasks::TaskRegistry};
use crate::view::{ComponentMutation, ComponentUpdater, RenderContext, RetentionScope, View};

pub(crate) struct RetainedEntry {
    entity: Box<dyn Any>,
    _subscriptions: Vec<Subscription>,
}

struct RetainedComponent {
    entity: Box<dyn Any>,
    props: Option<Box<dyn Any>>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
struct RetainedKey {
    scope: Vec<RetentionScope>,
    id: Identity,
}

#[derive(Default)]
pub(crate) struct RetainedEntities {
    entries: HashMap<RetainedKey, RetainedEntry>,
    components: HashMap<RetainedKey, RetainedComponent>,
    scope: Vec<RetentionScope>,
    seen: HashSet<RetainedKey>,
    seen_components: HashSet<RetainedKey>,
    seen_keys: HashSet<Vec<RetentionScope>>,
    preserved: Vec<Vec<RetentionScope>>,
}

impl RetainedEntities {
    /// Reconcile one visited lazy cell without touching other cells.
    pub(crate) fn begin_cell(&mut self, id: &Identity) {
        debug_assert!(self.scope.is_empty());
        let in_cell = |scope: &[RetentionScope]| matches!(scope.first(), Some(RetentionScope::Key(key)) if key == id);
        self.seen.retain(|key| !in_cell(&key.scope));
        self.seen_components.retain(|key| !in_cell(&key.scope));
        self.seen_keys.retain(|scope| !in_cell(scope));
        self.preserved.retain(|scope| !in_cell(scope));
    }

    pub(crate) fn finish_cell(&mut self, id: &Identity) {
        debug_assert!(self.scope.is_empty());
        let in_cell = |scope: &[RetentionScope]| matches!(scope.first(), Some(RetentionScope::Key(key)) if key == id);
        let seen = &self.seen;
        let seen_components = &self.seen_components;
        let preserved = &self.preserved;
        self.entries.retain(|key, _| {
            !in_cell(&key.scope)
                || seen.contains(key)
                || preserved.iter().any(|prefix| key.scope.starts_with(prefix))
        });
        self.components.retain(|key, _| {
            !in_cell(&key.scope)
                || seen_components.contains(key)
                || preserved.iter().any(|prefix| key.scope.starts_with(prefix))
        });
    }

    pub(crate) fn prune_cells(&mut self, current: &HashSet<Identity>) {
        let keep = |scope: &[RetentionScope]| matches!(scope.first(), Some(RetentionScope::Key(id)) if current.contains(id));
        self.entries.retain(|key, _| keep(&key.scope));
        self.components.retain(|key, _| keep(&key.scope));
        self.seen.retain(|key| keep(&key.scope));
        self.seen_components.retain(|key| keep(&key.scope));
        self.seen_keys.retain(|scope| keep(scope));
        self.preserved.retain(|scope| keep(scope));
    }

    pub(crate) fn begin_frame(&mut self) {
        self.scope.clear();
        self.seen.clear();
        self.seen_components.clear();
        self.seen_keys.clear();
        self.preserved.clear();
    }

    pub(crate) fn finish_frame(&mut self) {
        debug_assert!(
            self.scope.is_empty(),
            "retention scope left open after rendering"
        );
        let seen = &self.seen;
        let preserved = &self.preserved;
        self.entries.retain(|key, _| {
            seen.contains(key) || preserved.iter().any(|prefix| key.scope.starts_with(prefix))
        });
        let seen_components = &self.seen_components;
        self.components.retain(|key, _| {
            seen_components.contains(key)
                || preserved.iter().any(|prefix| key.scope.starts_with(prefix))
        });
    }

    pub(crate) fn push_scope(&mut self, scope: RetentionScope) {
        self.scope.push(scope);
        if matches!(
            self.scope.last(),
            Some(
                RetentionScope::Key(_)
                    | RetentionScope::Popover(_)
                    | RetentionScope::ContextMenu(_)
                    | RetentionScope::SplitView(_)
                    | RetentionScope::Semantics(_)
                    | RetentionScope::Focus(_)
                    | RetentionScope::Keyboard(_)
                    | RetentionScope::Hover(_)
                    | RetentionScope::Tap(_)
                    | RetentionScope::PropertyInspector(_)
                    | RetentionScope::LazyGrid(_)
                    | RetentionScope::Collection(_)
                    | RetentionScope::GeometryReader(_)
                    | RetentionScope::ScrollView(_)
                    | RetentionScope::ScrollViewContent
                    | RetentionScope::Background(_)
                    | RetentionScope::Border(_)
                    | RetentionScope::Shadow(_)
                    | RetentionScope::Card(_)
                    | RetentionScope::DragSource(_)
                    | RetentionScope::DropTarget(_)
                    | RetentionScope::SpringOpacity(_)
                    | RetentionScope::AnimatedOpacity(_)
            )
        ) {
            assert!(
                self.seen_keys.insert(self.scope.clone()),
                "keyed identity used twice in the same scope during one render"
            );
        }
    }

    pub(crate) fn pop_scope(&mut self) {
        self.scope.pop().expect("retention scope stack underflow");
    }

    pub(crate) fn preserve_scope(&mut self, scope: RetentionScope) {
        let mut prefix = self.scope.clone();
        prefix.push(scope);
        self.preserved.push(prefix);
    }

    fn key(&self, id: &Identity) -> RetainedKey {
        RetainedKey {
            scope: self.scope.clone(),
            id: id.clone(),
        }
    }

    pub(crate) fn get<T: 'static>(&mut self, id: &Identity) -> Option<Entity<T>> {
        let key = self.key(id);
        assert!(
            self.seen.insert(key.clone()),
            "retained identity `{id}` used twice in the same scope during one render"
        );
        self.entries.get(&key).map(|entry| {
            entry
                .entity
                .downcast_ref::<Entity<T>>()
                .unwrap_or_else(|| {
                    panic!("retained identity `{id}` is used by different control types")
                })
                .clone()
        })
    }

    pub(crate) fn insert<T: 'static>(&mut self, id: Identity, entity: Entity<T>) {
        let key = self.key(&id);
        self.entries.insert(
            key,
            RetainedEntry {
                entity: Box::new(entity),
                _subscriptions: Vec::new(),
            },
        );
    }

    pub(crate) fn component<T: View>(
        &mut self,
        id: Identity,
        create: impl FnOnce() -> Entity<ComponentHost<T>>,
    ) -> (Entity<ComponentHost<T>>, bool) {
        let key = self.key(&id);
        assert!(
            self.seen_components.insert(key.clone()),
            "component identity `{id}` used twice in the same scope during one render"
        );
        if let Some(entity) = self
            .components
            .get(&key)
            .and_then(|entry| entry.entity.downcast_ref::<Entity<ComponentHost<T>>>())
        {
            return (entity.clone(), false);
        }

        // A new concrete component type resets the old host and its native element ID.
        let entity = create();
        self.components.insert(
            key,
            RetainedComponent {
                entity: Box::new(entity.clone()),
                props: None,
            },
        );
        (entity, true)
    }

    pub(crate) fn clear_component_props(&mut self, id: &Identity) {
        let key = self.key(id);
        self.components
            .get_mut(&key)
            .expect("component props require an existing child")
            .props = None;
    }

    pub(crate) fn component_props_changed<P: Clone + PartialEq + 'static>(
        &mut self,
        id: &Identity,
        props: &P,
    ) -> bool {
        let key = self.key(id);
        let entry = self
            .components
            .get_mut(&key)
            .expect("component props require an existing child");
        if entry
            .props
            .as_ref()
            .and_then(|previous| previous.downcast_ref::<P>())
            == Some(props)
        {
            return false;
        }
        entry.props = Some(Box::new(props.clone()));
        true
    }

    pub(crate) fn retain_subscription(&mut self, id: &Identity, subscription: Subscription) {
        let key = self.key(id);
        self.entries
            .get_mut(&key)
            .expect("retained subscriptions require an existing entity")
            ._subscriptions
            .push(subscription);
    }
}

/// Owns a RustUI component inside GPUI's entity and rendering lifecycle.
pub struct ComponentHost<V> {
    pub(crate) component: V,
    environment: EnvironmentValues,
    retained: RetainedEntities,
    pub(crate) tasks: TaskRegistry,
    _release: Subscription,
    _localization: Subscription,
}

impl<V: View> ComponentHost<V> {
    pub fn new(component: V, cx: &mut Context<Self>) -> Self {
        Self::with_environment(component, EnvironmentValues::default(), cx)
    }

    pub fn with_environment(
        mut component: V,
        environment: EnvironmentValues,
        cx: &mut Context<Self>,
    ) -> Self {
        let release = cx.on_release(|host, cx| host.component.on_unmount(cx));
        let localization = crate::localization::invalidation::observe(cx);
        component.on_mount(cx);
        Self {
            component,
            environment,
            retained: RetainedEntities::default(),
            tasks: TaskRegistry::default(),
            _release: release,
            _localization: localization,
        }
    }

    pub fn component(&self) -> &V {
        &self.component
    }

    pub fn component_mut(&mut self) -> &mut V {
        &mut self.component
    }

    pub fn environment(&self) -> &EnvironmentValues {
        &self.environment
    }

    pub(crate) fn refresh_environment(&mut self, environment: EnvironmentValues) {
        self.environment = environment;
    }
}

impl<V: View> Render for ComponentHost<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let control_font_size = gpui_component::Theme::global(cx).font_size;
        let button_theme =
            crate::theme::registry::active_theme(cx).map(|theme| theme.button.clone());
        let localization = crate::localization::active_localization(cx).cloned();
        let owner = cx.weak_entity();
        let update_component: ComponentUpdater =
            Rc::new(move |mutation: ComponentMutation, cx: &mut App| {
                owner
                    .update(cx, |host, cx| {
                        if mutation(&mut host.component) {
                            cx.notify();
                        }
                    })
                    .ok();
            });
        self.retained.begin_frame();
        let mut context = RenderContext::new(
            update_component,
            cx.entity_id(),
            TypeId::of::<V>(),
            Rc::new(cx.weak_entity()),
            control_font_size,
            &self.environment,
            button_theme,
            localization,
            window,
            cx,
            &mut self.retained,
        );

        // Route through the root itself so primitive roots can use their own
        // lowering implementation instead of expanding through `body()`.
        let started =
            render_diagnostics::render_diagnostics_enabled(context.app()).then(Instant::now);
        let element = self.component.into_element(&mut context);
        let elapsed = started.map(|started| started.elapsed());
        drop(context);
        if let Some(elapsed) = elapsed {
            render_diagnostics::record_host_lowering(
                cx,
                TypeId::of::<V>(),
                type_name::<V>(),
                elapsed,
            );
        }
        self.retained.finish_frame();
        element
    }
}

#[cfg(test)]
mod context_menu_scope_tests {
    use super::{RetainedComponent, RetainedEntities, RetainedEntry, RetainedKey};
    use crate::view::RetentionScope;
    use rustui_core::Identity;
    use std::collections::HashSet;

    #[test]
    fn wrapper_scopes_partition_descendants_and_reject_duplicate_siblings() {
        let id = Identity::from("same");
        let child = Identity::from("input");
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let mut child_keys = HashSet::new();
        for scope in [
            RetentionScope::Background(id.clone()),
            RetentionScope::Border(id.clone()),
            RetentionScope::Shadow(id.clone()),
            RetentionScope::Card(id.clone()),
            RetentionScope::Tap(id.clone()),
            RetentionScope::PropertyInspector(id.clone()),
            RetentionScope::DragSource(id.clone()),
            RetentionScope::DropTarget(id),
        ] {
            retained.push_scope(scope.clone());
            assert!(child_keys.insert(retained.key(&child)));
            retained.pop_scope();
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    retained.push_scope(scope);
                }))
                .is_err()
            );
            retained.pop_scope();
        }
    }

    #[test]
    fn card_and_shadow_with_same_id_partition_children_and_reject_duplicate_card() {
        let id = Identity::from("surface");
        let child = Identity::from("editor");
        let mut retained = RetainedEntities::default();
        retained.begin_frame();

        retained.push_scope(RetentionScope::Card(id.clone()));
        let card_child = retained.key(&child);
        retained.pop_scope();

        retained.push_scope(RetentionScope::Shadow(id.clone()));
        let shadow_child = retained.key(&child);
        retained.pop_scope();
        assert!(card_child != shadow_child);

        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                retained.push_scope(RetentionScope::Card(id));
            }))
            .is_err()
        );
    }

    #[test]
    fn scroll_internal_state_and_user_content_keys_are_separate() {
        let mut retained = RetainedEntities::default();
        let id = Identity::from("scroll-state");
        retained.push_scope(RetentionScope::ScrollView(Identity::from("reader")));
        let internal = retained.key(&id);
        retained.push_scope(RetentionScope::ScrollViewContent);
        let user_control = retained.key(&id);
        assert!(internal != user_control);
        retained.pop_scope();
        retained.pop_scope();
    }

    #[test]
    fn scroll_view_scope_rejects_duplicates_without_key_collisions() {
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let id = Identity::from("scroll");
        for scope in [
            RetentionScope::Key(id.clone()),
            RetentionScope::Keyboard(id.clone()),
            RetentionScope::GeometryReader(id.clone()),
        ] {
            retained.push_scope(scope);
            retained.pop_scope();
        }
        let scope = RetentionScope::ScrollView(id);
        retained.push_scope(scope.clone());
        retained.pop_scope();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| retained.push_scope(scope)))
                .is_err()
        );
    }

    #[test]
    fn geometry_reader_scope_is_typed_and_rejects_duplicates() {
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let id = Identity::from("reader");
        for scope in [
            RetentionScope::Key(id.clone()),
            RetentionScope::LazyGrid(id.clone()),
            RetentionScope::Keyboard(id.clone()),
        ] {
            retained.push_scope(scope);
            retained.pop_scope();
        }
        let scope = RetentionScope::GeometryReader(id);
        retained.push_scope(scope.clone());
        retained.pop_scope();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| retained.push_scope(scope)))
                .is_err()
        );
    }

    #[test]
    fn opacity_wrappers_partition_same_id_descendants_and_reject_duplicate_siblings() {
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let child = Identity::from("shared-child");
        let first = RetentionScope::SpringOpacity(Identity::from("first"));
        let second = RetentionScope::SpringOpacity(Identity::from("second"));
        let animated = RetentionScope::AnimatedOpacity(Identity::from("first"));

        retained.push_scope(first.clone());
        let first_child = retained.key(&child);
        retained.pop_scope();
        retained.push_scope(second);
        let second_child = retained.key(&child);
        retained.pop_scope();
        retained.push_scope(animated);
        let animated_child = retained.key(&child);
        retained.pop_scope();

        assert!(first_child != second_child);
        assert!(first_child != animated_child);
        assert!(second_child != animated_child);
        for scope in [
            first,
            RetentionScope::AnimatedOpacity(Identity::from("first")),
        ] {
            let mut duplicates = RetainedEntities::default();
            duplicates.begin_frame();
            duplicates.push_scope(scope.clone());
            duplicates.pop_scope();
            assert!(
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(
                    || duplicates.push_scope(scope)
                ))
                .is_err()
            );
        }
    }

    #[test]
    fn dedicated_reader_store_releases_omitted_branch() {
        let mut retained = RetainedEntities::default();
        let key = RetainedKey {
            scope: Vec::new(),
            id: Identity::from("branch"),
        };
        retained.entries.insert(
            key.clone(),
            RetainedEntry {
                entity: Box::new(()),
                _subscriptions: Vec::new(),
            },
        );
        retained.begin_frame();
        retained.seen.insert(key.clone());
        retained.finish_frame();
        assert!(retained.entries.contains_key(&key));
        retained.begin_frame();
        retained.finish_frame();
        assert!(!retained.entries.contains_key(&key));
    }

    #[test]
    fn duplicate_context_menu_siblings_are_rejected() {
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let scope = RetentionScope::ContextMenu(Identity::from("target"));
        retained.push_scope(scope.clone());
        retained.pop_scope();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| retained.push_scope(scope)))
                .is_err()
        );
    }

    #[test]
    fn duplicate_split_view_siblings_are_rejected_without_key_collisions() {
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let id = Identity::from("target");
        retained.push_scope(RetentionScope::Key(id.clone()));
        retained.pop_scope();
        let scope = RetentionScope::SplitView(id);
        retained.push_scope(scope.clone());
        retained.pop_scope();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| retained.push_scope(scope)))
                .is_err()
        );
    }

    #[test]
    fn duplicate_semantics_siblings_are_rejected_without_key_collisions() {
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let id = Identity::from("target");
        retained.push_scope(RetentionScope::Key(id.clone()));
        retained.pop_scope();
        let scope = RetentionScope::Semantics(id);
        retained.push_scope(scope.clone());
        retained.pop_scope();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| retained.push_scope(scope)))
                .is_err()
        );
    }

    #[test]
    fn duplicate_focus_siblings_are_rejected_without_key_collisions() {
        let mut retained = RetainedEntities::default();
        retained.begin_frame();
        let id = Identity::from("target");
        retained.push_scope(RetentionScope::Key(id.clone()));
        retained.pop_scope();
        let scope = RetentionScope::Focus(id);
        retained.push_scope(scope.clone());
        retained.pop_scope();
        assert!(
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| retained.push_scope(scope)))
                .is_err()
        );
    }

    fn cell_key(cell: &str, child: &str) -> RetainedKey {
        RetainedKey {
            scope: vec![
                RetentionScope::Key(Identity::from(cell)),
                RetentionScope::Key(Identity::from(child)),
            ],
            id: Identity::from("state"),
        }
    }

    #[test]
    fn revisiting_one_cell_prunes_its_unseen_children_only() {
        let mut retained = RetainedEntities::default();
        let stale = cell_key("a", "conditional");
        let kept = cell_key("b", "conditional");
        for key in [stale.clone(), kept.clone()] {
            retained.entries.insert(
                key.clone(),
                RetainedEntry {
                    entity: Box::new(()),
                    _subscriptions: Vec::new(),
                },
            );
            retained.components.insert(
                key,
                RetainedComponent {
                    entity: Box::new(()),
                    props: None,
                },
            );
        }
        retained.begin_cell(&Identity::from("a"));
        retained.finish_cell(&Identity::from("a"));
        assert!(!retained.entries.contains_key(&stale));
        assert!(!retained.components.contains_key(&stale));
        assert!(retained.entries.contains_key(&kept));
        assert!(retained.components.contains_key(&kept));
    }

    #[test]
    fn removed_cell_pruning_keeps_current_offscreen_cell() {
        let mut retained = RetainedEntities::default();
        let removed = cell_key("removed", "child");
        let offscreen = cell_key("offscreen", "child");
        for key in [removed.clone(), offscreen.clone()] {
            retained.entries.insert(
                key,
                RetainedEntry {
                    entity: Box::new(()),
                    _subscriptions: Vec::new(),
                },
            );
        }
        retained.prune_cells(&HashSet::from([Identity::from("offscreen")]));
        assert!(!retained.entries.contains_key(&removed));
        assert!(retained.entries.contains_key(&offscreen));
    }
}

#[cfg(test)]
mod collection_eviction_tests {
    use super::*;
    use std::cell::Cell;
    struct ReleaseProbe(Rc<Cell<usize>>);
    impl Drop for ReleaseProbe {
        fn drop(&mut self) {
            self.0.set(self.0.get() + 1);
        }
    }
    #[test]
    fn final_visible_ids_release_measurement_rows_and_offscreen_children() {
        let released = Rc::new(Cell::new(0));
        let mut retained = RetainedEntities::default();
        for id in ["measurement", "visible", "offscreen"] {
            let key = RetainedKey {
                scope: vec![RetentionScope::Key(Identity::from(id))],
                id: Identity::from("control"),
            };
            retained.entries.insert(
                key.clone(),
                RetainedEntry {
                    entity: Box::new(ReleaseProbe(released.clone())),
                    _subscriptions: Vec::new(),
                },
            );
            retained.components.insert(
                key,
                RetainedComponent {
                    entity: Box::new(ReleaseProbe(released.clone())),
                    props: None,
                },
            );
        }
        retained.prune_cells(&HashSet::from([Identity::from("visible")]));
        assert_eq!(released.get(), 4);
        assert_eq!(retained.entries.len(), 1);
        assert_eq!(retained.components.len(), 1);
        retained.prune_cells(&HashSet::new());
        assert_eq!(released.get(), 6);
    }
}
