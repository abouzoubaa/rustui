//! Controlled Popover over the pinned GPUI Components presentation.

use std::{cell::Cell, rc::Rc};

use gpui::{AnyElement, IntoElement, ParentElement as _, SharedString, Styled as _, div};
use gpui_component::popover::Popover as GpuiPopover;
use rustui_core::{Binding, Identity};

use crate::{
    components::basic::button::{ButtonStyle, themed_button},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

fn set_open_if_changed(current: &mut bool, requested: bool) -> bool {
    if *current == requested {
        return false;
    }
    *current = requested;
    true
}

fn first_nonblank_trigger_label(labels: impl IntoIterator<Item = SharedString>) -> SharedString {
    labels
        .into_iter()
        .find(|label| !label.trim().is_empty())
        .expect("Popover trigger accessible name must not be blank")
}

fn resolved_localized_trigger_label(
    key: &SharedString,
    localized: SharedString,
) -> Option<SharedString> {
    (localized.trim() != key.trim() && !localized.trim().is_empty()).then_some(localized)
}

/// A labeled trigger and arbitrary RustUI content with controlled open state.
pub struct PopoverView<V> {
    id: Identity,
    trigger_label: SharedString,
    localized_label: Option<SharedString>,
    is_open: Binding<bool>,
    content: Rc<std::cell::RefCell<V>>,
    native_popup_size: Option<(f32, f32)>,
    #[cfg(test)]
    force_native_unsupported: bool,
}

impl<V: View> PopoverView<V> {
    pub fn new(
        id: impl Into<Identity>,
        trigger_label: impl Into<SharedString>,
        is_open: Binding<bool>,
        content: V,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Popover identity must be nonempty"
        );
        let trigger_label = trigger_label.into();
        assert!(
            !trigger_label.trim().is_empty(),
            "Popover trigger requires a non-empty visible label"
        );
        Self {
            id,
            trigger_label,
            localized_label: None,
            is_open,
            content: Rc::new(std::cell::RefCell::new(content)),
            native_popup_size: None,
            #[cfg(test)]
            force_native_unsupported: false,
        }
    }

    /// Resolve the trigger label from the active RustUI message catalog.
    pub fn localized_label(mut self, message_key: impl Into<SharedString>) -> Self {
        let message_key = message_key.into();
        assert!(
            !message_key.trim().is_empty(),
            "Popover trigger requires a non-empty localization key"
        );
        self.localized_label = Some(message_key);
        self
    }

    /// Opt into a native anchored window, using the existing in-window popover
    /// automatically when the current platform does not support native popups.
    #[cfg(not(target_family = "wasm"))]
    pub fn native_popup(mut self, size: (f32, f32)) -> Self {
        self.native_popup_size = Some(size);
        self
    }

    #[cfg(test)]
    pub(crate) fn force_native_unsupported_for_test(mut self) -> Self {
        self.force_native_unsupported = true;
        self
    }
}

#[allow(non_snake_case)]
pub fn Popover<V: View>(
    id: impl Into<Identity>,
    trigger_label: impl Into<SharedString>,
    is_open: Binding<bool>,
    content: V,
) -> PopoverView<V> {
    PopoverView::new(id, trigger_label, is_open, content)
}

impl<V: View> View for PopoverView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Popover(self.id.clone()), |context| {
            let open = self.is_open.value();
            let localized_label = self.localized_label.as_ref().and_then(|key| {
                resolved_localized_trigger_label(
                    key,
                    SharedString::from(context.localized_message(key.as_ref())),
                )
            });
            let label = first_nonblank_trigger_label([
                localized_label.unwrap_or_default(),
                self.trigger_label.clone(),
            ]);
            let id = self.id.to_string();
            let trigger = themed_button(
                context,
                SharedString::from(format!("{id}:trigger")),
                label.clone(),
                label.clone(),
                ButtonStyle::Automatic,
                false,
                0,
                true,
            );
            #[cfg(not(target_family = "wasm"))]
            if let Some(size) = self.native_popup_size {
                #[cfg(test)]
                let force_unsupported = self.force_native_unsupported;
                #[cfg(not(test))]
                let force_unsupported = false;
                return native_popover_element(
                    self,
                    context,
                    trigger,
                    open,
                    size,
                    label,
                    force_unsupported,
                );
            }
            let requested = Rc::new(Cell::new(open));
            let requested_for_update = requested.clone();
            let update = context.binding_updater(self.is_open.clone(), move |current| {
                set_open_if_changed(current, requested_for_update.get())
            });
            let mut popover = GpuiPopover::new(id)
                .trigger(trigger)
                .open(open)
                .on_open_change(move |next, _window, cx| {
                    requested.set(*next);
                    update(cx);
                });
            if open {
                popover = popover.child(self.content.borrow().into_element(context));
            }
            div().flex().child(popover).into_any_element()
        })
    }
}

#[cfg(not(target_family = "wasm"))]
#[derive(Default)]
struct NativePopoverState {
    popup: Option<crate::runtime::window::AnchoredPopupHandle>,
    anchor: Option<gpui::Bounds<gpui::Pixels>>,
    fallback: bool,
}

#[cfg(not(target_family = "wasm"))]
struct NativePopoverRoot<V: View> {
    content: Rc<std::cell::RefCell<V>>,
    state: gpui::Entity<NativePopoverState>,
    seed: crate::view::RenderSeed,
    requested_open: Rc<Cell<bool>>,
    update_open: Rc<dyn Fn(&mut gpui::App)>,
}

#[cfg(not(target_family = "wasm"))]
impl<V: View> View for NativePopoverRoot<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn on_mount(&mut self, cx: &mut gpui::App) {
        self.content.borrow_mut().on_mount(cx);
    }

    fn on_unmount(&mut self, cx: &mut gpui::App) {
        self.content.borrow_mut().on_unmount(cx);
        self.state.update(cx, |state, _| state.popup = None);
        self.requested_open.set(false);
        (self.update_open)(cx);
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_seed_owner(&self.seed, |context| {
            self.content.borrow().into_element(context)
        })
    }
}

#[cfg(not(target_family = "wasm"))]
fn native_popover_element<V: View>(
    owner: &PopoverView<V>,
    context: &mut RenderContext<'_>,
    trigger: gpui_component::button::Button,
    open: bool,
    size: (f32, f32),
    label: SharedString,
    force_unsupported: bool,
) -> AnyElement {
    use crate::runtime::window::{
        AnchoredPopupError, AnchoredPopupOptions, try_open_anchored_popup,
    };
    use gpui::ParentElement as _;
    use gpui_component::ElementExt as _;

    let state_id = Identity::new(format!("{}:native-popup", owner.id.as_str()));
    let (state, _) = context.retained_entity(state_id, |_, _| NativePopoverState::default());
    let state_snapshot = state.read(context.app());
    let popup = state_snapshot.popup.clone();
    let fallback = state_snapshot.fallback;
    drop(state_snapshot);

    if open && popup.is_none() && !fallback {
        // A caller may open the binding directly. Preserve that public controlled
        // state by using the portable presentation for externally opened popovers.
        let (window, app) = context.transition_window_app();
        state.update(app, |state, _| state.fallback = true);
        let _ = window;
        return in_window_popover(owner, context, open, label, Some(state));
    }

    if !open && let Some(popup_to_close) = popup.clone() {
        let (window, app) = context.transition_window_app();
        let _ = popup_to_close.update(app, |_, popup_window, _| popup_window.remove_window());
        let _ = window;
        state.update(app, |state, _| state.popup = None);
    }

    if fallback {
        return in_window_popover(owner, context, open, label, Some(state));
    }

    let requested_open = Rc::new(Cell::new(open));
    let requested_for_update = requested_open.clone();
    let update_open = context.binding_updater(owner.is_open.clone(), move |current| {
        set_open_if_changed(current, requested_for_update.get())
    });
    let anchor_state = state.clone();
    let environment = context.environment_values();
    let seed = context.seed();
    let content = owner.content.clone();
    let state_for_click = state.clone();
    let update_for_click = update_open.clone();
    let content_for_root = content.clone();
    let state_for_root = state.clone();
    let seed_for_root = seed.clone();
    let update_for_root = update_open.clone();
    let requested_for_root = requested_open.clone();
    let trigger = trigger.on_click(move |_, window, cx| {
        let is_open = requested_open.get();
        let next = !is_open;
        requested_open.set(next);
        if next {
            if force_unsupported {
                state_for_click.update(cx, |state, _| {
                    state.popup = None;
                    state.fallback = true;
                });
                (update_for_click)(cx);
                return;
            }
            let anchor = state_for_click
                .read(cx)
                .anchor
                .expect("native popover trigger must be laid out before activation");
            let root = NativePopoverRoot {
                content: content_for_root.clone(),
                state: state_for_root.clone(),
                seed: seed_for_root.clone(),
                requested_open: requested_for_root.clone(),
                update_open: update_for_root.clone(),
            };
            let options = AnchoredPopupOptions {
                anchor_rect: (
                    f32::from(anchor.origin.x),
                    f32::from(anchor.origin.y),
                    f32::from(anchor.size.width),
                    f32::from(anchor.size.height),
                ),
                size,
            };
            match try_open_anchored_popup(root, environment.clone(), options, window, cx) {
                Ok(popup) => state_for_click.update(cx, |state, _| {
                    state.popup = Some(popup);
                    state.fallback = false;
                }),
                Err(AnchoredPopupError::Unsupported) => state_for_click.update(cx, |state, _| {
                    state.popup = None;
                    state.fallback = true;
                }),
                Err(_) => requested_open.set(false),
            }
        } else if let Some(popup) = state_for_click.read(cx).popup.clone() {
            let _ = popup.update(cx, |_, popup_window, _| popup_window.remove_window());
        }
        (update_for_click)(cx);
    });
    let track_popup = popup;
    let anchor_element = div()
        .on_prepaint(move |bounds, _, cx| {
            anchor_state.update(cx, |state, _| state.anchor = Some(bounds));
        })
        .child(trigger);
    match track_popup {
        Some(popup) => popup.track_anchor(anchor_element),
        None => anchor_element.into_any_element(),
    }
}

#[cfg(not(target_family = "wasm"))]
fn in_window_popover<V: View>(
    owner: &PopoverView<V>,
    context: &mut RenderContext<'_>,
    open: bool,
    label: SharedString,
    native_state: Option<gpui::Entity<NativePopoverState>>,
) -> AnyElement {
    let id = owner.id.to_string();
    let trigger = themed_button(
        context,
        SharedString::from(format!("{id}:trigger")),
        label.clone(),
        label,
        ButtonStyle::Automatic,
        false,
        0,
        true,
    );
    let requested = Rc::new(Cell::new(open));
    let requested_for_update = requested.clone();
    let update = context.binding_updater(owner.is_open.clone(), move |current| {
        set_open_if_changed(current, requested_for_update.get())
    });
    let mut popover = GpuiPopover::new(id)
        .trigger(trigger)
        .open(open)
        .on_open_change(move |next, _window, cx| {
            requested.set(*next);
            if !*next && let Some(state) = native_state.as_ref() {
                state.update(cx, |state, _| state.fallback = false);
            }
            update(cx);
        });
    if open {
        popover = popover.child(owner.content.borrow().into_element(context));
    }
    div().flex().child(popover).into_any_element()
}

#[cfg(test)]
mod tests {
    use super::{
        first_nonblank_trigger_label, resolved_localized_trigger_label, set_open_if_changed,
    };
    use crate::components::basic::text::Text;

    fn read(_: &PopoverOwner) -> bool {
        false
    }

    fn write(_: &mut PopoverOwner, _: bool) {}

    struct PopoverOwner;

    #[test]
    #[should_panic(expected = "Popover trigger requires a non-empty visible label")]
    fn blank_visible_trigger_labels_are_rejected() {
        super::Popover(
            "details",
            " \t",
            rustui_core::Binding::new(false, read, write),
            Text("Body"),
        );
    }

    #[test]
    #[should_panic(expected = "Popover trigger requires a non-empty localization key")]
    fn blank_localization_keys_are_rejected() {
        super::Popover(
            "details",
            "Details",
            rustui_core::Binding::new(false, read, write),
            Text("Body"),
        )
        .localized_label(" \t");
    }

    #[test]
    fn blank_localized_trigger_label_falls_back_to_visible_label() {
        assert_eq!(
            first_nonblank_trigger_label([" \t".into(), "Details".into()]),
            "Details"
        );
    }

    #[test]
    fn unresolved_localized_trigger_label_falls_back_to_visible_label() {
        let key: gpui::SharedString = "popover.details".into();
        assert_eq!(
            resolved_localized_trigger_label(&key, key.clone()).unwrap_or_default(),
            ""
        );
        assert_eq!(
            first_nonblank_trigger_label([
                resolved_localized_trigger_label(&key, key.clone()).unwrap_or_default(),
                "Details".into(),
            ]),
            "Details"
        );
    }

    #[test]
    fn padded_localized_trigger_key_echo_falls_back_to_visible_label() {
        let key: gpui::SharedString = "popover.details".into();
        let echoed_key: gpui::SharedString = " \tpopover.details \t".into();
        assert_eq!(
            resolved_localized_trigger_label(&key, echoed_key).unwrap_or_default(),
            ""
        );
        assert_eq!(
            first_nonblank_trigger_label([
                resolved_localized_trigger_label(&key, " \tpopover.details \t".into())
                    .unwrap_or_default(),
                "Details".into(),
            ]),
            "Details"
        );
    }

    #[test]
    fn resolved_localized_trigger_label_is_preserved() {
        let key: gpui::SharedString = "popover.details".into();
        assert_eq!(
            resolved_localized_trigger_label(&key, "Détails".into()),
            Some("Détails".into())
        );
    }

    #[test]
    #[should_panic(expected = "Popover trigger accessible name must not be blank")]
    fn all_blank_trigger_labels_are_rejected() {
        first_nonblank_trigger_label([" ".into(), "\n".into()]);
    }

    #[test]
    fn open_binding_changes_only_on_transition() {
        let mut open = false;
        assert!(!set_open_if_changed(&mut open, false));
        assert!(set_open_if_changed(&mut open, true));
        assert!(open);
        assert!(!set_open_if_changed(&mut open, true));
        assert!(set_open_if_changed(&mut open, false));
        assert!(!open);
    }
}
