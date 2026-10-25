//! Controlled window-hosted Drawer over the existing GPUI Root and Sheet host.
use crate::{
    components::basic::button::Button,
    runtime::{AppAction, component_host::RetainedEntities},
    view::{AnyView, EmptyView, RenderContext, RenderSeed, RetentionScope, View},
};
use gpui::{
    AnyElement, AnyWindowHandle, App, AppContext as _, Context, Entity, EntityId, FocusHandle,
    InteractiveElement as _, IntoElement as _, ParentElement as _, Render, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, WeakFocusHandle,
    Window, div, px,
};
use gpui_base::{Root, RootPlugin, Sheet as SheetHost, TextSelection};
use gpui_component::scroll::ScrollableElement as _;
use gpui_component::{ActiveTheme as _, WindowExt as _};
use rustui_core::{Binding, Identity};
use std::rc::Rc;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DrawerEdge {
    Left,
    #[default]
    Right,
    Top,
    Bottom,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DrawerError {
    ModalConflict,
    MissingRootHost,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct Token {
    owner: EntityId,
    identity: Identity,
    generation: u64,
}
#[derive(Clone)]
struct Props {
    id: Identity,
    edge: DrawerEdge,
    size: f32,
    title: SharedString,
    name: SharedString,
    close_label: SharedString,
    content: Rc<AnyView>,
    seed: RenderSeed,
    close_owner: Rc<dyn Fn(&mut App)>,
}
struct Presentation {
    token: Token,
    lease: WeakEntity<Lease>,
    props: Props,
    focus: FocusHandle,
    previous: Option<WeakFocusHandle>,
    retained: RetainedEntities,
}
struct Lease {
    token: Option<Token>,
    revision: u64,
    host: Option<WeakEntity<DrawerHost>>,
    window: AnyWindowHandle,
    _release: Subscription,
}
struct DrawerHost {
    generation: u64,
    active: Option<Presentation>,
}
fn host(window: &Window, app: &App) -> Option<Entity<DrawerHost>> {
    window
        .root::<Root>()
        .flatten()?
        .read(app)
        .plugin::<DrawerHost>()
}
pub(crate) fn init(app: &mut App) {
    Root::register_plugin::<DrawerHost>(app, |_, _| DrawerHost {
        generation: 0,
        active: None,
    });
}
/// Existing Sheet/Dialog paths only guard against an active Drawer; their
/// existing native combination and replacement behavior remains unchanged.
pub(crate) fn is_active(window: &Window, app: &App) -> bool {
    host(window, app).is_some_and(|host| host.read(app).active.is_some())
}

/// A new Drawer never joins or replaces an existing native modal or Drawer.
fn can_open_drawer(window: &mut Window, app: &mut App) -> bool {
    !window.has_active_sheet(app) && !window.has_active_dialog(app) && !is_active(window, app)
}

impl DrawerHost {
    fn dismiss(
        &mut self,
        token: &Token,
        notify_owner: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self
            .active
            .as_ref()
            .is_none_or(|active| active.token != *token)
        {
            return;
        }
        let active = self.active.take().unwrap();
        if active.focus.contains_focused(window, cx) {
            if let Some(previous) = active.previous.and_then(|handle| handle.upgrade()) {
                window.focus(&previous, cx);
            }
        }
        TextSelection::clear(window, cx);
        if notify_owner {
            (active.props.close_owner)(cx);
        }
        cx.notify();
    }
}
fn dismiss(token: &Token, notify_owner: bool, window: &mut Window, app: &mut App) {
    if let Some(host) = host(window, app) {
        host.update(app, |host, cx| {
            host.dismiss(token, notify_owner, window, cx)
        });
    }
}
impl RootPlugin for DrawerHost {}

impl Render for DrawerHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl gpui::IntoElement {
        let instance = cx.entity_id();
        let Some(active) = self.active.as_mut() else {
            return div().into_any_element();
        };
        let foreign_modal = window.has_active_sheet(cx) || window.has_active_dialog(cx);
        let owner_alive = active.lease.upgrade().is_some();
        if foreign_modal || !owner_alive {
            let notify_owner = foreign_modal && owner_alive;
            let token = active.token.clone();
            let weak = cx.weak_entity();
            let handle = window.window_handle();
            cx.defer(move |cx| {
                let _ = handle.update(cx, |_, window, app| {
                    let _ = weak.update(app, |host, cx| {
                        host.dismiss(&token, notify_owner, window, cx)
                    });
                });
            });
            return div().into_any_element();
        }
        let props = active.props.clone();
        let token = active.token.clone();
        let close_token = token.clone();
        let close = AppAction::with_window_action(move |_, window, app| {
            dismiss(&close_token, true, window, app)
        });
        active.retained.begin_frame();
        let mut context = props
            .seed
            .overlay_context(instance, window, cx, &mut active.retained);
        let content = props.content.into_element(&mut context);
        let close_button = Button(props.close_label.clone(), close)
            .id(format!("{}:close", props.id))
            .into_element(&mut context);
        active.retained.finish_frame();
        let viewport = window.viewport_size();
        let mut surface = div()
            .id(format!("{}:dialog", props.id))
            .role(gpui::Role::Dialog)
            .aria_label(props.name)
            .absolute()
            .occlude()
            .flex()
            .flex_col()
            .overflow_hidden()
            .bg(cx.theme().tokens.background)
            .text_color(cx.theme().foreground)
            .border_1()
            .border_color(cx.theme().border)
            .shadow_xl();
        surface = match props.edge {
            DrawerEdge::Left => surface
                .left_0()
                .top_0()
                .bottom_0()
                .w(px(props.size.min(viewport.width.as_f32()))),
            DrawerEdge::Right => surface
                .right_0()
                .top_0()
                .bottom_0()
                .w(px(props.size.min(viewport.width.as_f32()))),
            DrawerEdge::Top => surface
                .top_0()
                .left_0()
                .right_0()
                .h(px(props.size.min(viewport.height.as_f32()))),
            DrawerEdge::Bottom => surface
                .bottom_0()
                .left_0()
                .right_0()
                .h(px(props.size.min(viewport.height.as_f32()))),
        };
        surface = surface
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .child(
                        div()
                            .id("rustui-drawer-title")
                            .role(gpui::Role::Label)
                            .aria_label(props.title.clone())
                            .child(props.title),
                    )
                    .child(close_button),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p_3()
                    .child(content),
            );
        SheetHost::new(cx)
            .focus_handle(active.focus.clone())
            .request_close(move |window, app| dismiss(&token, true, window, app))
            .overlay(div().size_full().occlude().bg(gpui::black().opacity(0.35)))
            .surface(surface)
            .into_any_element()
    }
}
pub struct DrawerView {
    id: Identity,
    visible: Binding<bool>,
    edge: DrawerEdge,
    size: f32,
    title: SharedString,
    title_key: Option<SharedString>,
    name: Option<SharedString>,
    name_key: Option<SharedString>,
    close_label: SharedString,
    close_key: Option<SharedString>,
    content: Rc<AnyView>,
    error: Option<Binding<Option<DrawerError>>>,
}
fn nonblank(value: impl Into<SharedString>, field: &str) -> SharedString {
    let value = value.into();
    assert!(!value.trim().is_empty(), "Drawer {field} must not be blank");
    value
}
fn text(
    context: &RenderContext<'_>,
    key: Option<&SharedString>,
    fallback: &SharedString,
) -> SharedString {
    key.map(|key| (key, context.localized_message(key.as_ref())))
        .filter(|(key, value)| !value.trim().is_empty() && value.trim() != key.trim())
        .map(|(_, value)| value.into())
        .unwrap_or_else(|| fallback.clone())
}
impl DrawerView {
    pub fn new(id: impl Into<Identity>, visible: Binding<bool>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Drawer identity must not be blank"
        );
        Self {
            id,
            visible,
            edge: DrawerEdge::Right,
            size: 350.0,
            title: "Drawer".into(),
            title_key: None,
            name: None,
            name_key: None,
            close_label: "Close".into(),
            close_key: Some("rustui-action-close".into()),
            content: Rc::new(AnyView::new(EmptyView)),
            error: None,
        }
    }
    pub fn edge(mut self, edge: DrawerEdge) -> Self {
        self.edge = edge;
        self
    }
    pub fn size(mut self, size: f32) -> Self {
        assert!(
            size.is_finite() && size > 0.0,
            "Drawer size must be finite and positive"
        );
        self.size = size;
        self
    }
    pub fn title(mut self, value: impl Into<SharedString>) -> Self {
        self.title = nonblank(value, "title");
        self.title_key = None;
        self
    }
    pub fn localized_title(mut self, key: impl Into<SharedString>) -> Self {
        self.title_key = Some(nonblank(key, "title key"));
        self
    }
    pub fn accessibility_label(mut self, value: impl Into<SharedString>) -> Self {
        self.name = Some(nonblank(value, "accessible name"));
        self.name_key = None;
        self
    }
    pub fn localized_accessibility_label(mut self, key: impl Into<SharedString>) -> Self {
        self.name_key = Some(nonblank(key, "accessible name key"));
        self
    }
    pub fn close_label(mut self, value: impl Into<SharedString>) -> Self {
        self.close_label = nonblank(value, "close label");
        self.close_key = None;
        self
    }
    pub fn localized_close_label(mut self, key: impl Into<SharedString>) -> Self {
        self.close_key = Some(nonblank(key, "close label key"));
        self
    }
    pub fn content<V: View>(mut self, content: V) -> Self {
        self.content = Rc::new(AnyView::new(content));
        self
    }
    pub fn presentation_error(mut self, error: Binding<Option<DrawerError>>) -> Self {
        self.error = Some(error);
        self
    }
}
#[allow(non_snake_case)]
pub fn Drawer(id: impl Into<Identity>, visible: Binding<bool>) -> DrawerView {
    DrawerView::new(id, visible)
}
impl View for DrawerView {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let close_owner = context.binding_updater(self.visible.clone(), |visible| {
            if !*visible {
                false
            } else {
                *visible = false;
                true
            }
        });
        let title = text(context, self.title_key.as_ref(), &self.title);
        let fallback = self.name.clone().unwrap_or_else(|| title.clone());
        let props = Props {
            id: self.id.clone(),
            edge: self.edge,
            size: self.size,
            title,
            name: text(context, self.name_key.as_ref(), &fallback),
            close_label: text(context, self.close_key.as_ref(), &self.close_label),
            content: self.content.clone(),
            seed: context.seed(),
            close_owner: close_owner.clone(),
        };
        let error = self.error.clone();
        let conflict = error.as_ref().map(|binding| {
            context.binding_updater(binding.clone(), |value| {
                if *value == Some(DrawerError::ModalConflict) {
                    false
                } else {
                    *value = Some(DrawerError::ModalConflict);
                    true
                }
            })
        });
        let missing = error.as_ref().map(|binding| {
            context.binding_updater(binding.clone(), |value| {
                if *value == Some(DrawerError::MissingRootHost) {
                    false
                } else {
                    *value = Some(DrawerError::MissingRootHost);
                    true
                }
            })
        });
        let clear = error.as_ref().map(|binding| {
            context.binding_updater(binding.clone(), |value| value.take().is_some())
        });
        let visible = self.visible.value();
        context.with_element_scope(RetentionScope::Key(self.id.clone()), |context| {
            let (lease, _) =
                context.retained_entity(Identity::from("drawer-lease"), |window, cx| {
                    let release = cx.on_release(|lease: &mut Lease, app| {
                        if let (Some(token), Some(host)) = (lease.token.clone(), lease.host.clone())
                        {
                            let handle = lease.window;
                            app.defer(move |app| {
                                let _ = handle.update(app, |_, window, app| {
                                    let _ = host.update(app, |host, cx| {
                                        host.dismiss(&token, false, window, cx)
                                    });
                                });
                            });
                        }
                    });
                    Lease {
                        token: None,
                        revision: 0,
                        host: None,
                        window: window.window_handle(),
                        _release: release,
                    }
                });
            let weak = lease.downgrade();
            context.update_entity(&lease, |lease, window, cx| {
                lease.revision = lease
                    .revision
                    .checked_add(1)
                    .expect("Drawer render revisions exhausted");
                let revision = lease.revision;
                let Some(host) = host(window, cx) else {
                    if visible {
                        cx.defer(move |cx| {
                            if weak
                                .update(cx, |lease, _| lease.revision == revision)
                                .unwrap_or(false)
                            {
                                close_owner(cx);
                                if let Some(missing) = missing {
                                    missing(cx);
                                }
                            }
                        });
                    }
                    return;
                };
                lease.host = Some(host.downgrade());
                if !visible {
                    if let Some(token) = lease.token.take() {
                        host.update(cx, |host, cx| host.dismiss(&token, false, window, cx));
                    }
                    return;
                }
                if lease.token.as_ref().is_some_and(|token| {
                    host.read(cx)
                        .active
                        .as_ref()
                        .is_some_and(|active| active.token == *token)
                }) {
                    // Root's plugin view is rendered after application content in this
                    // same frame. Notifying here would create a render/notify loop.
                    host.update(cx, |host, _| {
                        host.active.as_mut().unwrap().props = props;
                    });
                    return;
                }
                if !can_open_drawer(window, cx) {
                    cx.defer(move |cx| {
                        if weak
                            .update(cx, |lease, _| lease.revision == revision)
                            .unwrap_or(false)
                        {
                            close_owner(cx);
                            if let Some(conflict) = conflict {
                                conflict(cx);
                            }
                        }
                    });
                    return;
                }
                let owner = cx.entity_id();
                let clear_lease = weak.clone();
                let token = host.update(cx, |host, cx| {
                    host.generation = host
                        .generation
                        .checked_add(1)
                        .expect("Drawer generations exhausted");
                    let token = Token {
                        owner,
                        identity: props.id.clone(),
                        generation: host.generation,
                    };
                    let previous = window.focused(cx).map(|handle| handle.downgrade());
                    let focus = cx.focus_handle();
                    focus.focus(window, cx);
                    TextSelection::clear(window, cx);
                    host.active = Some(Presentation {
                        token: token.clone(),
                        lease: weak,
                        props,
                        focus,
                        previous,
                        retained: RetainedEntities::default(),
                    });
                    cx.notify();
                    token
                });
                lease.token = Some(token.clone());
                let clear_host = host.downgrade();
                if let Some(clear) = clear {
                    cx.defer(move |cx| {
                        if clear_host
                            .update(cx, |host, _| {
                                host.active
                                    .as_ref()
                                    .is_some_and(|active| active.token == token)
                            })
                            .unwrap_or(false)
                            && clear_lease
                                .update(cx, |lease, _| {
                                    lease.revision == revision
                                        && lease.token.as_ref() == Some(&token)
                                })
                                .unwrap_or(false)
                        {
                            clear(cx);
                        }
                    });
                }
            });
            div().into_any_element()
        })
    }
}
#[cfg(test)]
#[path = "drawer_tests.rs"]
mod tests;
