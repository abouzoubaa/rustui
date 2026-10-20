//! RustUI's composable view contract and GPUI rendering adapter.

use std::{
    any::{Any, TypeId},
    marker::PhantomData,
    rc::Rc,
};

use gpui::{
    AnyElement, App, AppContext as _, ClickEvent, Context, ElementId, Entity, EntityId,
    EventEmitter, IntoElement, ParentElement, Pixels, SharedString, StyleRefinement, Styled,
    Subscription, WeakEntity, Window, div, px,
};
use rustui_core::{Action, Binding, EnvironmentValues, Identity};
use rustui_i18n::LayoutDirection;

pub use crate::runtime::ComponentHost as ViewHost;
use crate::{
    localization::ActiveLocalization,
    runtime::{
        AppAction, ComponentHost, component_host::RetainedEntities, scoped_element::scoped_element,
        tasks::OwnerTask,
    },
    theme::{ButtonStyleSelector, ResolvedButtonTheme, ResolvedComponentStyle, ThemeResolveError},
};

/// Callback shape accepted by GPUI Components controls.
pub(crate) type ActionHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App)>;
pub(crate) type KeyboardActionHandler = Box<dyn Fn(&mut Window, &mut App)>;
pub(crate) type ComponentMutation = Box<dyn FnOnce(&mut dyn Any) -> bool>;
pub(crate) type ComponentUpdater = Rc<dyn Fn(ComponentMutation, &mut App)>;

/// An explicit callback target for the component that described a child view.
///
/// Obtain this in the owner's body with [`OwnerActionReader`] and pass it to a
/// child through props. Unlike a local `self.action`, it targets that exact
/// owner's GPUI entity, even when other owners have the same concrete type.
pub struct OwnerAction<C: 'static> {
    entity_id: EntityId,
    update_component: ComponentUpdater,
    marker: PhantomData<fn(&mut C)>,
}

impl<C: 'static> Clone for OwnerAction<C> {
    fn clone(&self) -> Self {
        Self {
            entity_id: self.entity_id,
            update_component: self.update_component.clone(),
            marker: PhantomData,
        }
    }
}

impl<C: 'static> PartialEq for OwnerAction<C> {
    fn eq(&self, other: &Self) -> bool {
        self.entity_id == other.entity_id
    }
}

impl<C: 'static> Eq for OwnerAction<C> {}

impl<C: 'static> OwnerAction<C> {
    /// Invoke an owner mutation and notify its GPUI host.
    pub fn action(&self, handler: fn(&mut C)) -> AppAction {
        let update_component = self.update_component.clone();
        AppAction::new(move |cx| {
            update_component(
                Box::new(move |owner| {
                    let owner = owner
                        .downcast_mut::<C>()
                        .expect("OwnerAction target has the wrong component type");
                    handler(owner);
                    true
                }),
                cx,
            );
        })
    }

    /// Captures a typed local value for a compiled owner mutation.
    /// Useful for rejecting callbacks from stale preview revisions.
    pub fn action_with<T: Clone + 'static>(&self, value: T, handler: fn(&mut C, &T)) -> AppAction {
        let update_component = self.update_component.clone();
        AppAction::new(move |cx| {
            let value = value.clone();
            update_component(
                Box::new(move |owner| {
                    let owner = owner
                        .downcast_mut::<C>()
                        .expect("OwnerAction target has the wrong component type");
                    handler(owner, &value);
                    true
                }),
                cx,
            );
        })
    }

    /// Notify the owner only when its mutation reports a change.
    pub fn action_if_changed(&self, handler: fn(&mut C) -> bool) -> AppAction {
        let update_component = self.update_component.clone();
        AppAction::new(move |cx| {
            update_component(
                Box::new(move |owner| {
                    let owner = owner
                        .downcast_mut::<C>()
                        .expect("OwnerAction target has the wrong component type");
                    handler(owner)
                }),
                cx,
            );
        })
    }
}

#[cfg(test)]
mod owner_action_tests {
    use super::*;

    struct Owner;

    #[test]
    fn target_equality_uses_entity_identity_without_owner_trait_bounds() {
        let updater: ComponentUpdater = Rc::new(|_, _| {});
        let target = OwnerAction::<Owner> {
            entity_id: EntityId::from(1_u64),
            update_component: updater.clone(),
            marker: PhantomData,
        };
        let same_owner_new_render = OwnerAction::<Owner> {
            entity_id: EntityId::from(1_u64),
            update_component: Rc::new(|_, _| {}),
            marker: PhantomData,
        };
        let sibling = OwnerAction::<Owner> {
            entity_id: EntityId::from(2_u64),
            update_component: updater,
            marker: PhantomData,
        };

        assert!(target == target.clone());
        assert!(target == same_owner_new_render);
        assert!(target != sibling);
    }
}

/// One segment of a retained control's composition path.
#[derive(Clone, PartialEq, Eq, Hash)]
pub(crate) enum RetentionScope {
    Key(Identity),
    Popover(Identity),
    ContextMenu(Identity),
    SplitView(Identity),
    Semantics(Identity),
    Focus(Identity),
    Keyboard(Identity),
    Hover(Identity),
    Tap(Identity),
    PropertyInspector(Identity),
    LazyGrid(Identity),
    Collection(Identity),
    GeometryReader(Identity),
    ScrollView(Identity),
    ScrollViewContent,
    Background(Identity),
    Border(Identity),
    Shadow(Identity),
    Card(Identity),
    DragSource(Identity),
    DropTarget(Identity),
    SpringOpacity(Identity),
    SpringOffset(Identity),
    AnimatedOpacity(Identity),
    Tabs(Identity),
    Tab(String),
    Navigation(Identity),
    NavigationSplit(Identity),
    Page(Vec<String>),
}

#[cfg(test)]
mod retention_scope_tests {
    use super::RetentionScope;
    use rustui_core::Identity;
    use std::collections::HashSet;

    #[test]
    fn overlay_and_keyed_views_have_distinct_paths() {
        let id = Identity::from("shared");
        let scopes = HashSet::from([
            RetentionScope::Key(id.clone()),
            RetentionScope::Popover(id.clone()),
            RetentionScope::ContextMenu(id.clone()),
            RetentionScope::SplitView(id.clone()),
            RetentionScope::Semantics(Identity::from("shared")),
            RetentionScope::Focus(Identity::from("shared")),
            RetentionScope::Tap(Identity::from("shared")),
            RetentionScope::PropertyInspector(Identity::from("shared")),
            RetentionScope::LazyGrid(Identity::from("shared")),
            RetentionScope::Shadow(id.clone()),
            RetentionScope::Card(id.clone()),
            RetentionScope::DragSource(id.clone()),
            RetentionScope::DropTarget(id),
        ]);
        assert_eq!(scopes.len(), 13);
    }
}

#[derive(Clone)]
#[doc(hidden)]
pub enum ControlAction {
    Component(Action),
    Application(AppAction),
}

impl From<Action> for ControlAction {
    fn from(action: Action) -> Self {
        Self::Component(action)
    }
}

impl From<AppAction> for ControlAction {
    fn from(action: AppAction) -> Self {
        Self::Application(action)
    }
}

/// Rendering services shared by views while lowering into GPUI elements.
pub struct RenderContext<'a> {
    update_component: ComponentUpdater,
    owner_id: EntityId,
    text_element_id_prefix: SharedString,
    text_element_slot: usize,
    owner_type: TypeId,
    task_owner: Rc<dyn Any>,
    control_font_size: Pixels,
    environment: &'a EnvironmentValues,
    button_theme: Option<ResolvedButtonTheme>,
    localization: Option<ActiveLocalization>,
    window: &'a mut Window,
    app: &'a mut App,
    retained: &'a mut RetainedEntities,
}

/// Owned inputs for lowering a deferred cell with the current component owner.
#[derive(Clone)]
pub(crate) struct RenderSeed {
    update_component: ComponentUpdater,
    owner_id: EntityId,
    owner_type: TypeId,
    task_owner: Rc<dyn Any>,
    control_font_size: Pixels,
    environment: EnvironmentValues,
    button_theme: Option<ResolvedButtonTheme>,
    localization: Option<ActiveLocalization>,
}

impl RenderSeed {
    /// Builds deferred geometry content while preserving its actual component owner.
    pub(crate) fn geometry_context<'a>(
        &'a self,
        reader_instance: EntityId,
        window: &'a mut Window,
        app: &'a mut App,
        retained: &'a mut RetainedEntities,
    ) -> RenderContext<'a> {
        RenderContext {
            update_component: self.update_component.clone(),
            owner_id: self.owner_id,
            text_element_id_prefix: geometry_text_prefix(self.owner_id, reader_instance),
            text_element_slot: 0,
            owner_type: self.owner_type,
            task_owner: self.task_owner.clone(),
            control_font_size: self.control_font_size,
            environment: &self.environment,
            button_theme: self.button_theme.clone(),
            localization: self.localization.clone(),
            window,
            app,
            retained,
        }
    }

    pub(crate) fn cell_context<'a>(
        &'a self,
        grid_instance: EntityId,
        cell: &Identity,
        window: &'a mut Window,
        app: &'a mut App,
        retained: &'a mut RetainedEntities,
    ) -> RenderContext<'a> {
        RenderContext {
            update_component: self.update_component.clone(),
            owner_id: self.owner_id,
            text_element_id_prefix: cell_text_prefix(self.owner_id, grid_instance, cell),
            text_element_slot: 0,
            owner_type: self.owner_type,
            task_owner: self.task_owner.clone(),
            control_font_size: self.control_font_size,
            environment: &self.environment,
            button_theme: self.button_theme.clone(),
            localization: self.localization.clone(),
            window,
            app,
            retained,
        }
    }
}

fn geometry_text_prefix(owner: EntityId, reader: EntityId) -> SharedString {
    format!(
        "rustui-text-{}:geometry-instance:{}",
        owner.as_u64(),
        reader.as_u64()
    )
    .into()
}

#[cfg(test)]
mod geometry_context_tests {
    use super::*;

    #[test]
    fn text_namespace_distinguishes_reader_owner_and_grid() {
        let owner = EntityId::from(1_u64);
        let reader = EntityId::from(2_u64);
        let prefix = geometry_text_prefix(owner, reader);
        assert_eq!(prefix, geometry_text_prefix(owner, reader));
        assert_ne!(prefix, geometry_text_prefix(owner, EntityId::from(3_u64)));
        assert_ne!(prefix, geometry_text_prefix(EntityId::from(4_u64), reader));
        assert_ne!(
            prefix,
            cell_text_prefix(owner, reader, &Identity::from("content"))
        );
    }
}

fn cell_text_prefix(owner: EntityId, grid: EntityId, cell: &Identity) -> SharedString {
    format!(
        "rustui-text-{}:grid-instance:{}:cell:{}:{}",
        owner.as_u64(),
        grid.as_u64(),
        cell.as_str().len(),
        cell.as_str()
    )
    .into()
}

#[cfg(test)]
mod lazy_cell_context_tests {
    use super::*;

    #[test]
    fn text_prefix_is_stable_per_cell_and_distinct_per_grid_and_owner() {
        let owner = EntityId::from(1_u64);
        let grid = EntityId::from(2_u64);
        let id = Identity::from("cell");
        let prefix = cell_text_prefix(owner, grid, &id);
        assert_eq!(prefix, cell_text_prefix(owner, grid, &id));
        assert_ne!(prefix, cell_text_prefix(owner, EntityId::from(3_u64), &id));
        assert_ne!(prefix, cell_text_prefix(EntityId::from(4_u64), grid, &id));
        assert_ne!(
            prefix,
            cell_text_prefix(owner, grid, &Identity::from("other"))
        );
    }
}

impl<'a> RenderContext<'a> {
    pub(crate) fn seed(&self) -> RenderSeed {
        RenderSeed {
            update_component: self.update_component.clone(),
            owner_id: self.owner_id,
            owner_type: self.owner_type,
            task_owner: self.task_owner.clone(),
            control_font_size: self.control_font_size,
            environment: (*self.environment).clone(),
            button_theme: self.button_theme.clone(),
            localization: self.localization.clone(),
        }
    }
    pub(crate) fn new(
        update_component: ComponentUpdater,
        owner_id: EntityId,
        owner_type: TypeId,
        task_owner: Rc<dyn Any>,
        control_font_size: Pixels,
        environment: &'a EnvironmentValues,
        button_theme: Option<ResolvedButtonTheme>,
        localization: Option<ActiveLocalization>,
        window: &'a mut Window,
        app: &'a mut App,
        retained: &'a mut RetainedEntities,
    ) -> Self {
        Self {
            update_component,
            owner_id,
            text_element_id_prefix: format!("rustui-text-{}", owner_id.as_u64()).into(),
            text_element_slot: 0,
            owner_type,
            task_owner,
            control_font_size,
            environment,
            button_theme,
            localization,
            window,
            app,
            retained,
        }
    }

    pub(crate) fn bind_action(&mut self, action: Action) -> ActionHandler {
        let update_component = self.update_component.clone();
        Box::new(move |_event, _window, cx| {
            let action = action.clone();
            update_component(
                Box::new(move |component| action.dispatch(component).should_notify()),
                cx,
            );
        })
    }

    pub(crate) fn bind_control_action(&mut self, action: ControlAction) -> ActionHandler {
        match action {
            ControlAction::Component(action) => self.bind_action(action),
            ControlAction::Application(action) => {
                Box::new(move |event, window, cx| action.dispatch(event, window, cx))
            }
        }
    }

    pub(crate) fn bind_keyboard_action(&mut self, action: ControlAction) -> KeyboardActionHandler {
        self.bind_context_action(action)
    }

    pub(crate) fn bind_context_action(&mut self, action: ControlAction) -> KeyboardActionHandler {
        match action {
            ControlAction::Component(action) => {
                let update_component = self.update_component.clone();
                Box::new(move |_window, cx| {
                    let action = action.clone();
                    update_component(
                        Box::new(move |component| action.dispatch(component).should_notify()),
                        cx,
                    );
                })
            }
            ControlAction::Application(action) => {
                Box::new(move |window, cx| action.dispatch_keyboard(window, cx))
            }
        }
    }

    pub(crate) fn control_font_size(&self) -> Pixels {
        self.control_font_size
    }

    pub(crate) fn next_text_element_id(&mut self) -> ElementId {
        let slot = self.text_element_slot;
        self.text_element_slot += 1;
        ElementId::named_usize(self.text_element_id_prefix.clone(), slot)
    }

    pub fn environment<T: 'static>(&self) -> Option<&T> {
        self.environment.get()
    }

    fn with_environment<R>(
        &mut self,
        environment: &EnvironmentValues,
        render: impl FnOnce(&mut RenderContext<'_>) -> R,
    ) -> R {
        let mut scoped = RenderContext {
            update_component: self.update_component.clone(),
            owner_id: self.owner_id,
            text_element_id_prefix: self.text_element_id_prefix.clone(),
            text_element_slot: self.text_element_slot,
            owner_type: self.owner_type,
            task_owner: self.task_owner.clone(),
            control_font_size: self.control_font_size,
            environment,
            button_theme: self.button_theme.clone(),
            localization: self.localization.clone(),
            window: &mut *self.window,
            app: &mut *self.app,
            retained: &mut *self.retained,
        };
        let result = render(&mut scoped);
        self.text_element_slot = scoped.text_element_slot;
        result
    }

    pub(crate) fn button_theme(&self) -> Option<&ResolvedButtonTheme> {
        self.button_theme.as_ref()
    }

    pub(crate) fn component_style(
        &self,
        component: &str,
        part: &str,
        selector: &ButtonStyleSelector,
    ) -> Result<ResolvedComponentStyle, ThemeResolveError> {
        crate::theme::registry::active_theme(self.app)
            .ok_or_else(|| ThemeResolveError::Invalid("no active RustUI theme".into()))?
            .component_style(component, part, selector)
    }

    pub(crate) fn component_style_with(
        &self,
        component: &str,
        part: &str,
        selector: &ButtonStyleSelector,
        handled_extras: &[&str],
    ) -> Result<ResolvedComponentStyle, ThemeResolveError> {
        crate::theme::registry::active_theme(self.app)
            .ok_or_else(|| ThemeResolveError::Invalid("no active RustUI theme".into()))?
            .component_style_with(component, part, selector, handled_extras)
    }

    pub(crate) fn card_defaults(&self) -> Option<(f32, f32)> {
        crate::theme::registry::active_theme(self.app).map(|theme| {
            (
                theme.profile.spacing.space_4 as f32,
                theme.profile.radii.lg as f32,
            )
        })
    }

    pub fn locale(&self) -> Option<&rustui_i18n::Locale> {
        self.localization.as_ref().map(ActiveLocalization::locale)
    }

    pub fn layout_direction(&self) -> LayoutDirection {
        self.localization
            .as_ref()
            .map_or(LayoutDirection::LeftToRight, ActiveLocalization::direction)
    }

    pub fn localized_message(&self, identifier: &str) -> String {
        self.localization.as_ref().map_or_else(
            || identifier.to_owned(),
            |active| active.message(identifier).to_owned(),
        )
    }

    pub(crate) fn binding_writer<T: 'static>(
        &self,
        binding: Binding<T>,
    ) -> Rc<dyn Fn(T, &mut App)> {
        let update_component = self.update_component.clone();
        Rc::new(move |value, cx| {
            let binding = binding.clone();
            update_component(
                Box::new(move |component| binding.write_to(component, value)),
                cx,
            );
        })
    }

    pub(crate) fn binding_updater<T: 'static>(
        &self,
        binding: Binding<T>,
        update: impl Fn(&mut T) -> bool + 'static,
    ) -> Rc<dyn Fn(&mut App)> {
        let update_component = self.update_component.clone();
        let update = Rc::new(update);
        Rc::new(move |cx| {
            let binding = binding.clone();
            let update = update.clone();
            update_component(
                Box::new(move |component| binding.update_on(component, |value| update(value))),
                cx,
            );
        })
    }

    pub(crate) fn with_retention_scope<R>(
        &mut self,
        scope: RetentionScope,
        render: impl FnOnce(&mut Self) -> R,
    ) -> R {
        self.retained.push_scope(scope);
        let result = render(self);
        self.retained.pop_scope();
        result
    }

    pub(crate) fn with_element_scope(
        &mut self,
        scope: RetentionScope,
        render: impl FnOnce(&mut Self) -> AnyElement,
    ) -> AnyElement {
        let element = self.with_retention_scope(scope.clone(), render);
        scoped_element(&scope, element)
    }

    pub(crate) fn preserve_retention_scope(&mut self, scope: RetentionScope) {
        self.retained.preserve_scope(scope);
    }

    pub(crate) fn retained_entity<T: 'static>(
        &mut self,
        id: Identity,
        initialize: impl FnOnce(&mut Window, &mut Context<T>) -> T,
    ) -> (Entity<T>, bool) {
        if let Some(entity) = self.retained.get(&id) {
            return (entity, false);
        }

        let window = &mut *self.window;
        let entity = self.app.new(|cx| initialize(window, cx));
        self.retained.insert(id, entity.clone());
        (entity, true)
    }

    pub(crate) fn retained_component<C: View>(
        &mut self,
        id: Identity,
        initialize: impl FnOnce() -> C,
    ) -> Entity<ComponentHost<C>> {
        let environment = self.environment;
        let app = &mut *self.app;
        let (child, created) = self.retained.component(id.clone(), || {
            let environment = (*environment).clone();
            app.new(|cx| ComponentHost::with_environment(initialize(), environment, cx))
        });
        if !created {
            child.update(self.app, |host, _| {
                host.refresh_environment((*environment).clone())
            });
        }
        self.retained.clear_component_props(&id);
        child
    }

    pub(crate) fn retained_component_with_props<C: View, P: Clone + PartialEq + 'static>(
        &mut self,
        id: Identity,
        initialize: impl FnOnce() -> C,
        props: &P,
        reconcile: fn(&mut C, &P),
    ) -> (Entity<ComponentHost<C>>, bool) {
        let environment = self.environment;
        let app = &mut *self.app;
        let (child, created) = self.retained.component(id.clone(), || {
            let mut component = initialize();
            reconcile(&mut component, props);
            let environment = (*environment).clone();
            app.new(|cx| ComponentHost::with_environment(component, environment, cx))
        });
        let changed = self.retained.component_props_changed(&id, props);
        if !created {
            child.update(self.app, |host, cx| {
                host.refresh_environment((*environment).clone());
                if changed {
                    reconcile(host.component_mut(), props);
                    cx.notify();
                }
            });
        }
        (child, changed && !created)
    }

    pub(crate) fn update_entity<T: 'static, R>(
        &mut self,
        entity: &Entity<T>,
        update: impl FnOnce(&mut T, &mut Window, &mut Context<T>) -> R,
    ) -> R {
        let window = &mut *self.window;
        entity.update(self.app, |state, cx| update(state, window, cx))
    }

    pub(crate) fn subscribe_retained<T, Event>(
        &mut self,
        id: &Identity,
        entity: &Entity<T>,
        listener: impl FnMut(Entity<T>, &Event, &mut App) + 'static,
    ) where
        T: EventEmitter<Event> + 'static,
        Event: 'static,
    {
        let subscription: Subscription = self.app.subscribe(entity, listener);
        self.retained.retain_subscription(id, subscription);
    }

    pub(crate) fn observe_retained<T: 'static>(
        &mut self,
        id: &Identity,
        entity: &Entity<T>,
        listener: impl FnMut(Entity<T>, &mut App) + 'static,
    ) {
        let subscription = self.app.observe(entity, listener);
        self.retained.retain_subscription(id, subscription);
    }

    pub(crate) fn app(&self) -> &App {
        self.app
    }
}

/// A RustUI view whose body is another RustUI view.
pub trait View: 'static {
    /// Describes this view's body using RustUI views.
    fn body(&self) -> impl View;

    /// Called once when an owned component enters its GPUI entity.
    /// Plain nested view descriptions do not receive this hook.
    fn on_mount(&mut self, _cx: &mut App) {}

    /// Called once when GPUI releases an owned component entity.
    /// Plain nested view descriptions do not receive this hook.
    fn on_unmount(&mut self, _cx: &mut App) {}

    /// Lowers this view into GPUI's element tree.
    #[doc(hidden)]
    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement
    where
        Self: Sized,
    {
        self.body().into_element(context)
    }
}

/// Empty primitive used as the body of leaf views.
#[doc(hidden)]
#[derive(Clone, Copy, Default)]
pub struct EmptyView;

/// A view with a stable identity within its current logical retention scope.
pub struct KeyedView<V> {
    key: Identity,
    view: V,
}

#[allow(non_snake_case)]
pub fn Keyed<V: View>(key: impl Into<Identity>, view: V) -> KeyedView<V> {
    KeyedView {
        key: key.into(),
        view,
    }
}

impl<V: View> View for KeyedView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Key(self.key.clone()), |context| {
            self.view.into_element(context)
        })
    }
}

/// Provides a typed value to a view and its descendant components.
pub struct EnvironmentView<T, V> {
    value: T,
    view: V,
}

#[allow(non_snake_case)]
pub fn Environment<T: Clone + 'static, V: View>(value: T, view: V) -> EnvironmentView<T, V> {
    EnvironmentView { value, view }
}

impl<T: Clone + 'static, V: View> View for EnvironmentView<T, V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let mut environment = (*context.environment).clone();
        environment.set(self.value.clone());
        context.with_environment(&environment, |context| self.view.into_element(context))
    }
}

/// Reads the current typed value and builds an owned view during rendering.
pub struct EnvironmentReaderView<T, V, F> {
    reader: F,
    marker: PhantomData<fn(&T) -> V>,
}

#[allow(non_snake_case)]
pub fn EnvironmentReader<T: 'static, V: View, F: Fn(Option<&T>) -> V + 'static>(
    reader: F,
) -> EnvironmentReaderView<T, V, F> {
    EnvironmentReaderView {
        reader,
        marker: PhantomData,
    }
}

/// Builds a view with a callback explicitly targeting the current component owner.
pub struct OwnerActionReaderView<C, V, F> {
    reader: F,
    marker: PhantomData<fn(&mut C) -> V>,
}

#[allow(non_snake_case)]
pub fn OwnerActionReader<C: 'static, V: View, F: Fn(OwnerAction<C>) -> V + 'static>(
    reader: F,
) -> OwnerActionReaderView<C, V, F> {
    OwnerActionReaderView {
        reader,
        marker: PhantomData,
    }
}

impl<C: 'static, V: View, F: Fn(OwnerAction<C>) -> V + 'static> View
    for OwnerActionReaderView<C, V, F>
{
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        assert_eq!(
            context.owner_type,
            TypeId::of::<C>(),
            "OwnerActionReader must name its current component owner type"
        );
        let owner = OwnerAction {
            entity_id: context.owner_id,
            update_component: context.update_component.clone(),
            marker: PhantomData,
        };
        (self.reader)(owner).into_element(context)
    }
}

/// Builds a view with a task controller for its current component owner.
pub struct OwnerTaskReaderView<C, V, F> {
    reader: F,
    marker: PhantomData<fn(&mut C) -> V>,
}

#[allow(non_snake_case)]
pub fn OwnerTaskReader<C: View, V: View, F: Fn(OwnerTask<C>) -> V + 'static>(
    reader: F,
) -> OwnerTaskReaderView<C, V, F> {
    OwnerTaskReaderView {
        reader,
        marker: PhantomData,
    }
}

impl<C: View, V: View, F: Fn(OwnerTask<C>) -> V + 'static> View for OwnerTaskReaderView<C, V, F> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        assert_eq!(
            context.owner_type,
            TypeId::of::<C>(),
            "OwnerTaskReader must name its current component owner type"
        );
        let owner = context
            .task_owner
            .downcast_ref::<WeakEntity<ComponentHost<C>>>()
            .expect("OwnerTaskReader host type does not match its owner")
            .clone();
        (self.reader)(OwnerTask::new(owner)).into_element(context)
    }
}

impl<T: 'static, V: View, F: Fn(Option<&T>) -> V + 'static> View
    for EnvironmentReaderView<T, V, F>
{
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let view = (self.reader)(context.environment());
        view.into_element(context)
    }
}

/// A retained child component with its own GPUI owner and lifecycle.
pub struct ComponentView<C: View> {
    key: Identity,
    initialize: Rc<dyn Fn() -> C>,
    cached_fixed: Option<(f32, f32)>,
}

/// A retained child with incoming props reconciled into its owned state.
pub struct ComponentPropsView<C: View, P: Clone + PartialEq + 'static> {
    component: ComponentView<C>,
    props: P,
    reconcile: fn(&mut C, &P),
    cached_fixed: Option<(f32, f32)>,
}

/// Describe a child component. The initializer runs only when its slot is created.
#[allow(non_snake_case)]
pub fn Component<C: View>(
    key: impl Into<Identity>,
    initialize: impl Fn() -> C + 'static,
) -> ComponentView<C> {
    ComponentView {
        key: key.into(),
        initialize: Rc::new(initialize),
        cached_fixed: None,
    }
}

impl<C: View> ComponentView<C> {
    /// Opts an independent child into GPUI's cache at a fixed logical-pixel size.
    /// Typed environment scopes, active native drags, and active accessibility
    /// use normal lowering. GPUI also misses the cache after a full-window
    /// refresh (including a focus change).
    pub fn cached_fixed(mut self, width: f32, height: f32) -> Self {
        assert_valid_cache_bounds(width, height);
        self.cached_fixed = Some((width, height));
        self
    }

    /// Project incoming props into the child before mount and whenever they change.
    /// Keep the reconciler stable for a given keyed component.
    pub fn props<P: Clone + PartialEq + 'static>(
        self,
        props: P,
        reconcile: fn(&mut C, &P),
    ) -> ComponentPropsView<C, P> {
        assert!(
            self.cached_fixed.is_none(),
            "cached_fixed does not support component props"
        );
        ComponentPropsView {
            component: self,
            props,
            reconcile,
            cached_fixed: None,
        }
    }
}

impl<C: View, P: Clone + PartialEq + 'static> ComponentPropsView<C, P> {
    /// Opts a props-driven child into GPUI's cache at fixed logical-pixel bounds.
    /// Call after `.props(...)`. A changed-props frame lowers normally at these
    /// bounds so its new value is visible immediately; caching resumes while
    /// props remain unchanged.
    pub fn cached_fixed(mut self, width: f32, height: f32) -> Self {
        assert_valid_cache_bounds(width, height);
        self.cached_fixed = Some((width, height));
        self
    }
}

fn assert_valid_cache_bounds(width: f32, height: f32) {
    assert!(
        width.is_finite() && height.is_finite() && width > 0.0 && height > 0.0,
        "cached component dimensions must be finite and positive"
    );
}

impl<C: View> View for ComponentView<C> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let child = context.retained_component(self.key.clone(), || (self.initialize)());
        if let Some((width, height)) = self.cached_fixed
            && cached_component_eligible(
                context.environment,
                context.app().has_active_drag(),
                context.window.is_a11y_active(),
            )
        {
            child
                .cached(StyleRefinement::default().w(px(width)).h(px(height)))
                .into_any_element()
        } else {
            child.into_any_element()
        }
    }
}

fn cached_component_eligible(
    environment: &EnvironmentValues,
    native_drag_active: bool,
    accessibility_active: bool,
) -> bool {
    environment.is_empty() && !native_drag_active && !accessibility_active
}

#[cfg(test)]
mod cached_component_tests {
    use super::*;

    #[test]
    fn fixed_dimensions_and_environment_fallback() {
        let view = Component("child", || EmptyView).cached_fixed(120.0, 48.0);
        assert_eq!(view.cached_fixed, Some((120.0, 48.0)));

        let mut environment = EnvironmentValues::default();
        assert!(cached_component_eligible(&environment, false, false));
        assert!(!cached_component_eligible(&environment, true, false));
        assert!(!cached_component_eligible(&environment, false, true));
        environment.set(1_u8);
        assert!(!cached_component_eligible(&environment, false, false));
    }

    #[test]
    fn props_cache_accepts_fixed_bounds_and_uses_all_fallbacks() {
        fn reconcile(_: &mut EmptyView, _: &u8) {}
        let view = Component("props-child", || EmptyView)
            .props(1_u8, reconcile)
            .cached_fixed(80.0, 32.0);
        assert_eq!(view.cached_fixed, Some((80.0, 32.0)));

        let mut environment = EnvironmentValues::default();
        assert!(cached_component_eligible(&environment, false, false));
        assert!(!cached_component_eligible(&environment, true, false));
        assert!(!cached_component_eligible(&environment, false, true));
        environment.set(1_u8);
        assert!(!cached_component_eligible(&environment, false, false));
    }

    #[test]
    #[should_panic(expected = "cached component dimensions must be finite and positive")]
    fn props_cache_rejects_non_positive_bounds() {
        fn reconcile(_: &mut EmptyView, _: &u8) {}
        Component("props-child", || EmptyView)
            .props(1_u8, reconcile)
            .cached_fixed(80.0, 0.0);
    }

    #[test]
    #[should_panic(expected = "cached component dimensions must be finite and positive")]
    fn fixed_cache_rejects_zero() {
        Component("child", || EmptyView).cached_fixed(0.0, 48.0);
    }

    #[test]
    #[should_panic(expected = "cached component dimensions must be finite and positive")]
    fn fixed_cache_rejects_non_finite_dimensions() {
        Component("child", || EmptyView).cached_fixed(120.0, f32::INFINITY);
    }

    #[test]
    #[should_panic(expected = "cached_fixed does not support component props")]
    fn fixed_cache_rejects_props() {
        fn reconcile(_: &mut EmptyView, _: &u8) {}
        Component("child", || EmptyView)
            .cached_fixed(120.0, 48.0)
            .props(1_u8, reconcile);
    }
}

impl<C: View, P: Clone + PartialEq + 'static> View for ComponentPropsView<C, P> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let (child, props_changed) = context.retained_component_with_props(
            self.component.key.clone(),
            || (self.component.initialize)(),
            &self.props,
            self.reconcile,
        );
        if let Some((width, height)) = self.cached_fixed {
            if props_changed {
                // GPUI drains view invalidations before lowering the frame.
                // A props notification raised during parent lowering is too
                // late to invalidate a cached child in that same frame.
                // Lower changed props normally so the current frame is fresh.
                div()
                    .w(px(width))
                    .h(px(height))
                    .child(child)
                    .into_any_element()
            } else if cached_component_eligible(
                context.environment,
                context.app().has_active_drag(),
                context.window.is_a11y_active(),
            ) {
                child
                    .cached(StyleRefinement::default().w(px(width)).h(px(height)))
                    .into_any_element()
            } else {
                child.into_any_element()
            }
        } else {
            child.into_any_element()
        }
    }
}

impl<V: View> View for Option<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        match self {
            Some(view) => view.into_element(context),
            None => gpui::Empty.into_any_element(),
        }
    }
}

/// Type-erased RustUI view used by heterogeneous declarative collections.
pub struct AnyView {
    render: Rc<dyn for<'a> Fn(&mut RenderContext<'a>) -> AnyElement>,
}

impl AnyView {
    pub fn new<V: View>(view: V) -> Self {
        Self {
            render: Rc::new(move |context| view.into_element(context)),
        }
    }
}

impl View for AnyView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        (self.render)(context)
    }
}

impl View for EmptyView {
    fn body(&self) -> impl View {
        Self
    }

    fn into_element(&self, _context: &mut RenderContext<'_>) -> AnyElement {
        div().into_any_element()
    }
}
