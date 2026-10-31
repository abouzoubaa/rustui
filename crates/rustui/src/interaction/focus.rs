//! Native GPUI focus ownership for a declarative RustUI subtree.

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, App, FocusHandle, InteractiveElement as _, IntoElement, ParentElement as _, Role,
    SharedString, StatefulInteractiveElement as _, Subscription, div,
};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, RetentionScope, View};

type FocusUpdate = Rc<dyn Fn(&mut App)>;

enum FocusTransition {
    None,
    Focus,
    Blur,
}

fn focus_transition(
    previous: Option<bool>,
    requested: bool,
    contains_focus: bool,
) -> FocusTransition {
    if previous == Some(requested) {
        return FocusTransition::None;
    }
    match (requested, contains_focus) {
        (true, false) => FocusTransition::Focus,
        (false, true) if previous == Some(true) => FocusTransition::Blur,
        _ => FocusTransition::None,
    }
}

struct FocusOwner {
    handle: FocusHandle,
    last_requested: Option<bool>,
    on_in_writer: Rc<RefCell<FocusUpdate>>,
    on_out_writer: Rc<RefCell<FocusUpdate>>,
    _on_in: Option<Subscription>,
    _on_out: Option<Subscription>,
}

/// A native focus container synchronized with a component-owned boolean binding.
///
/// The binding reports whether the container or one of its descendants is
/// focused. Setting it to true requests focus on the container; setting it to
/// false blurs only when focus remains inside this scope.
pub struct FocusScopeView<V> {
    id: Identity,
    focused: Binding<bool>,
    child: V,
    tab_stop: bool,
    label: Option<SharedString>,
}

impl<V: View> FocusScopeView<V> {
    pub fn new(id: impl Into<Identity>, focused: Binding<bool>, child: V) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "FocusScope identity must be nonempty"
        );
        Self {
            id,
            focused,
            child,
            tab_stop: false,
            label: None,
        }
    }

    /// Include the container itself in keyboard tab traversal.
    pub fn tab_stop(mut self, enabled: bool) -> Self {
        self.tab_stop = enabled;
        self
    }

    /// Name the container when assistive technology focuses it.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "FocusScope accessible label must be nonempty"
        );
        self.label = Some(label);
        self
    }
}

#[allow(non_snake_case)]
pub fn FocusScope<V: View>(
    id: impl Into<Identity>,
    focused: Binding<bool>,
    child: V,
) -> FocusScopeView<V> {
    FocusScopeView::new(id, focused, child)
}

impl<V: View> View for FocusScopeView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let scope = RetentionScope::Focus(self.id.clone());
        context.with_element_scope(scope, |context| {
            let requested = self.focused.value();
            let (owner, created) =
                context.retained_entity(Identity::from("focus-handle"), |_, cx| {
                    let no_op: FocusUpdate = Rc::new(|_| {});
                    FocusOwner {
                        handle: cx.focus_handle(),
                        last_requested: None,
                        on_in_writer: Rc::new(RefCell::new(no_op.clone())),
                        on_out_writer: Rc::new(RefCell::new(no_op)),
                        _on_in: None,
                        _on_out: None,
                    }
                });
            let write_in = context.binding_updater(self.focused.clone(), |value| {
                if *value {
                    return false;
                }
                *value = true;
                true
            });
            let write_out = context.binding_updater(self.focused.clone(), |value| {
                if !*value {
                    return false;
                }
                *value = false;
                true
            });
            let tab_stop = self.tab_stop;
            let handle = context.update_entity(&owner, |state, window, cx| {
                *state.on_in_writer.borrow_mut() = write_in;
                *state.on_out_writer.borrow_mut() = write_out;
                if created {
                    let writer = state.on_in_writer.clone();
                    state._on_in = Some(window.on_focus_in(&state.handle, cx, move |_, cx| {
                        let update = writer.borrow().clone();
                        update(cx);
                    }));
                    let writer = state.on_out_writer.clone();
                    state._on_out =
                        Some(window.on_focus_out(&state.handle, cx, move |_, _, cx| {
                            let update = writer.borrow().clone();
                            update(cx);
                        }));
                }

                let contains_focus = state.handle.contains_focused(window, cx);
                match focus_transition(state.last_requested, requested, contains_focus) {
                    FocusTransition::Focus => window.focus(&state.handle, cx),
                    FocusTransition::Blur => window.blur(cx),
                    FocusTransition::None => {}
                }
                state.last_requested = Some(requested);
                state.handle = state.handle.clone().tab_stop(tab_stop);
                state.handle.clone()
            });

            let child = self.child.into_element(context);
            let mut element = div()
                .id("rustui-focus-target")
                .track_focus(&handle)
                .role(Role::Group)
                .child(child);
            if let Some(label) = &self.label {
                element = element.aria_label(label.clone());
            }
            element.into_any_element()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::basic::text::Text;

    #[test]
    fn only_transitions_act_and_descendant_focus_is_preserved() {
        assert!(matches!(
            focus_transition(None, true, false),
            FocusTransition::Focus
        ));
        assert!(matches!(
            focus_transition(Some(false), true, true),
            FocusTransition::None
        ));
        assert!(matches!(
            focus_transition(Some(true), true, false),
            FocusTransition::None
        ));
        assert!(matches!(
            focus_transition(Some(true), false, true),
            FocusTransition::Blur
        ));
        assert!(matches!(
            focus_transition(Some(true), false, false),
            FocusTransition::None
        ));
    }

    #[test]
    #[should_panic(expected = "FocusScope identity must be nonempty")]
    fn blank_identity_is_rejected() {
        let binding = Binding::new(false, |_: &bool| false, |_, _| {});
        let _ = FocusScope("  ", binding, Text("child"));
    }

    #[test]
    #[should_panic(expected = "FocusScope accessible label must be nonempty")]
    fn blank_accessible_label_is_rejected() {
        let binding = Binding::new(false, |_: &bool| false, |_, _| {});
        let _ = FocusScope("focus", binding, Text("child")).label(" \t ");
    }
}
