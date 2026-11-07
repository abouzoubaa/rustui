//! Typed, same-process drag and drop using GPUI's native gesture lifecycle.

use std::{cell::RefCell, rc::Rc};

use gpui::{
    AnyElement, AppContext as _, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    Render, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, RetentionScope, View};

/// The application-level operation requested by a drag source.
///
/// The drop target delivers this intent; the binding owner applies copy or
/// transfer to its own model. RustUI does not delete source data automatically.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DragOperation {
    #[default]
    Copy,
    Transfer,
}

/// A typed drop delivered to the target's component-owned binding.
#[derive(Clone, Debug, PartialEq)]
pub struct DropEvent<T> {
    pub source: Identity,
    pub target: Identity,
    pub value: T,
    pub operation: DragOperation,
}

struct DragItem<T> {
    source: Identity,
    value: T,
    operation: DragOperation,
    preview: SharedString,
}

struct DragPreview {
    label: SharedString,
}

impl Render for DragPreview {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = &gpui_component::Theme::global(cx).colors;
        div()
            .px(px(8.0))
            .py(px(5.0))
            .rounded(px(5.0))
            .border_1()
            .border_color(colors.border)
            .bg(colors.popover)
            .text_color(colors.popover_foreground)
            .child(self.label.clone())
    }
}

/// Makes a RustUI child the native drag origin for one typed value.
pub struct DragSourceView<T: Clone + 'static, V: View> {
    id: Identity,
    value: T,
    child: V,
    operation: DragOperation,
    preview: SharedString,
    enabled: bool,
}

impl<T: Clone + 'static, V: View> DragSourceView<T, V> {
    pub fn new(id: impl Into<Identity>, value: T, child: V) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "DragSource identity must be nonempty"
        );
        let preview = id.as_str().to_owned().into();
        Self {
            id,
            value,
            child,
            operation: DragOperation::Copy,
            preview,
            enabled: true,
        }
    }

    /// Selects copy or transfer intent delivered with the eventual drop event.
    pub fn operation(mut self, operation: DragOperation) -> Self {
        self.operation = operation;
        self
    }

    /// Sets the text shown in GPUI's native drag preview.
    pub fn preview(mut self, label: impl Into<SharedString>) -> Self {
        self.preview = label.into();
        self
    }

    /// Disables drag initiation while leaving the child rendered.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[allow(non_snake_case)]
pub fn DragSource<T: Clone + 'static, V: View>(
    id: impl Into<Identity>,
    value: T,
    child: V,
) -> DragSourceView<T, V> {
    DragSourceView::new(id, value, child)
}

impl<T: Clone + 'static, V: View> View for DragSourceView<T, V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::DragSource(self.id.clone()), |context| {
            let child = self.child.into_element(context);
            let source = div()
                .id(format!("rustui-drag-source-{}", self.id.as_str()))
                .child(child);
            if self.enabled {
                source
                    .cursor_move()
                    .on_drag(
                        DragItem {
                            source: self.id.clone(),
                            value: self.value.clone(),
                            operation: self.operation,
                            preview: self.preview.clone(),
                        },
                        |item: &DragItem<T>, _, _, cx| {
                            cx.new(|_| DragPreview {
                                label: item.preview.clone(),
                            })
                        },
                    )
                    .into_any_element()
            } else {
                source.into_any_element()
            }
        })
    }
}

/// Accepts exactly the source value type and writes an owned drop event.
pub struct DropTargetView<T: Clone + 'static, V: View> {
    id: Identity,
    dropped: Binding<Option<DropEvent<T>>>,
    child: V,
    enabled: bool,
}

impl<T: Clone + 'static, V: View> DropTargetView<T, V> {
    pub fn new(id: impl Into<Identity>, dropped: Binding<Option<DropEvent<T>>>, child: V) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "DropTarget identity must be nonempty"
        );
        Self {
            id,
            dropped,
            child,
            enabled: true,
        }
    }

    /// Disables drop handling while leaving the child rendered.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[allow(non_snake_case)]
pub fn DropTarget<T: Clone + 'static, V: View>(
    id: impl Into<Identity>,
    dropped: Binding<Option<DropEvent<T>>>,
    child: V,
) -> DropTargetView<T, V> {
    DropTargetView::new(id, dropped, child)
}

impl<T: Clone + 'static, V: View> View for DropTargetView<T, V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::DropTarget(self.id.clone()), |context| {
            let pending = Rc::new(RefCell::new(None));
            let queued = pending.clone();
            let update = context.binding_updater(self.dropped.clone(), move |current| {
                let Some(event) = queued.borrow_mut().take() else {
                    return false;
                };
                *current = Some(event);
                true
            });
            let child = self.child.into_element(context);
            let target = div()
                .id(format!("rustui-drop-target-{}", self.id.as_str()))
                .child(child);
            if self.enabled {
                let id = self.id.clone();
                target
                    .on_drop(move |item: &DragItem<T>, _, cx| {
                        *pending.borrow_mut() = Some(DropEvent {
                            source: item.source.clone(),
                            target: id.clone(),
                            value: item.value.clone(),
                            operation: item.operation,
                        });
                        update(cx);
                    })
                    .into_any_element()
            } else {
                target.into_any_element()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dropped_binding() -> Binding<Option<DropEvent<String>>> {
        Binding::new(None, |_: &()| None, |_: &mut (), _| {})
    }

    #[test]
    fn disabled_source_keeps_its_identity_and_value() {
        let source = DragSource("source", "value".to_owned(), EmptyView).enabled(false);
        assert_eq!(source.id.as_str(), "source");
        assert_eq!(source.value, "value");
        assert!(!source.enabled);
    }

    #[test]
    fn disabled_target_keeps_its_identity_and_binding() {
        let target = DropTarget("target", dropped_binding(), EmptyView).enabled(false);
        assert_eq!(target.id.as_str(), "target");
        assert!(target.dropped.value().is_none());
        assert!(!target.enabled);
    }

    #[test]
    fn typed_drop_event_supports_component_state_equality() {
        let event = DropEvent {
            source: "source".into(),
            target: "target".into(),
            value: "value".to_owned(),
            operation: DragOperation::Copy,
        };
        assert_eq!(Some(event.clone()), Some(event));
    }

    #[test]
    #[should_panic(expected = "DragSource identity must be nonempty")]
    fn source_rejects_whitespace_identity() {
        let _ = DragSource("  ", "value", EmptyView);
    }

    #[test]
    #[should_panic(expected = "DropTarget identity must be nonempty")]
    fn target_rejects_whitespace_identity() {
        let _ = DropTarget("  ", dropped_binding(), EmptyView);
    }
}
