//! Typed, same-process drag and drop using GPUI's native gesture lifecycle.

use std::{
    cell::{Cell, RefCell},
    path::{Path, PathBuf},
    rc::Rc,
};

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

/// Entry point for external file-drop targets alongside the typed in-app target.
pub struct DropTarget {
    _private: (),
}

/// The accepted and rejected paths in an external file drop.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileDrop {
    pub accepted: Vec<PathBuf>,
    pub rejected: Vec<PathBuf>,
}

type FilePredicate = Rc<dyn Fn(&Path) -> bool>;

/// A target for files dragged from outside the application.
pub struct FileDropTargetView<V: View> {
    id: Identity,
    dropped: Binding<Option<FileDrop>>,
    child: V,
    accept: FilePredicate,
    description: String,
    multiple: bool,
    enabled: bool,
}

impl DropTarget {
    pub fn files<V: View>(
        id: impl Into<Identity>,
        dropped: Binding<Option<FileDrop>>,
        child: V,
    ) -> FileDropTargetView<V> {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "DropTarget identity must be nonempty"
        );
        FileDropTargetView {
            id,
            dropped,
            child,
            accept: Rc::new(|_| true),
            description: "Accepts all files".into(),
            multiple: true,
            enabled: true,
        }
    }
}

impl<V: View> FileDropTargetView<V> {
    pub fn accept(mut self, accept: impl Fn(&Path) -> bool + 'static) -> Self {
        self.accept = Rc::new(accept);
        self.description = "Accepted files are filtered".into();
        self
    }
    /// Replace the predicate and describe accepted files to accessibility clients.
    ///
    /// Like `accept`, this replaces a previous predicate. A later `extensions`
    /// call narrows the predicate and supplies its usual file-type description.
    pub fn accept_described(
        self,
        description: impl Into<SharedString>,
        accept: impl Fn(&Path) -> bool + 'static,
    ) -> Self {
        let mut target = self.accept(accept);
        target.description = format!("Accepts {}", description.into());
        target
    }

    pub fn extensions<I, S>(mut self, extensions: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let extensions: Vec<String> = extensions
            .into_iter()
            .map(|s| s.as_ref().trim_start_matches('.').to_ascii_lowercase())
            .collect();
        self.description = format!("Accepted file types: {}", extensions.join(", "));
        let previous = self.accept.clone();
        self.accept = Rc::new(move |path| {
            previous(path)
                && path
                    .extension()
                    .and_then(|ext| ext.to_str())
                    .is_some_and(|ext| {
                        extensions
                            .iter()
                            .any(|allowed| allowed.eq_ignore_ascii_case(ext))
                    })
        });
        self
    }
    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

fn classify_files(paths: &[PathBuf], accept: &FilePredicate, multiple: bool) -> FileDrop {
    let mut result = FileDrop {
        accepted: Vec::new(),
        rejected: Vec::new(),
    };
    for path in paths {
        if accept(path) && (multiple || result.accepted.is_empty()) {
            result.accepted.push(path.clone());
        } else {
            result.rejected.push(path.clone());
        }
    }
    result
}

struct FileDropHover(Rc<Cell<Option<bool>>>);

impl<V: View> View for FileDropTargetView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }
    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::DropTarget(self.id.clone()), |context| {
            let (stored, _) = context.retained_entity(Identity::from("file-drop-hover"), |_, _| {
                FileDropHover(Rc::new(Cell::new(None)))
            });
            let hover = stored.read(context.app()).0.clone();
            if !self.enabled {
                hover.set(None);
            }
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
            let colors = &gpui_component::Theme::global(context.app()).colors;
            let (accent, danger) = (colors.primary, colors.danger);
            let mut target = div()
                .id(format!("rustui-file-drop-{}", self.id.as_str()))
                .role(gpui::Role::Group)
                .aria_label("File drop target")
                .aria_disabled(!self.enabled)
                .aria_description(self.description.clone())
                .child(child);
            if let Some(accepted) = hover.get() {
                target = target
                    .border_1()
                    .border_color(if accepted { accent } else { danger })
                    .debug_selector(move || {
                        if accepted {
                            "rustui-file-drop-accepted".into()
                        } else {
                            "rustui-file-drop-rejected".into()
                        }
                    });
            }
            if !self.enabled {
                return target.into_any_element();
            }
            let move_hover = hover.clone();
            let move_filter = self.accept.clone();
            let drop_filter = self.accept.clone();
            let exit_hover = hover.clone();
            let multiple = self.multiple;
            target
                .on_drag_move(
                    move |event: &gpui::DragMoveEvent<gpui::ExternalPaths>, window, app| {
                        let state = event
                            .bounds
                            .contains(&event.event.position)
                            .then(|| event.drag(app).paths().iter().any(|p| move_filter(p)));
                        if move_hover.replace(state) != state {
                            window.refresh();
                        }
                    },
                )
                .on_file_drop_exit(move |_, window, _| {
                    if exit_hover.replace(None).is_some() {
                        window.refresh();
                    }
                })
                .on_drop(move |paths: &gpui::ExternalPaths, window, app| {
                    hover.set(None);
                    window.refresh();
                    let result = classify_files(paths.paths(), &drop_filter, multiple);
                    if !result.accepted.is_empty() {
                        *pending.borrow_mut() = Some(result);
                        update(app);
                    }
                })
                .into_any_element()
        })
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

#[cfg(test)]
mod file_drop_event_probe {
    use gpui::{
        App, Context, ExternalPaths, FileDropEvent, InteractiveElement as _, IntoElement,
        ParentElement as _, Render, StatefulInteractiveElement as _, Styled as _, Window, div, px,
    };
    use std::{cell::Cell, path::PathBuf, rc::Rc};
    struct Probe(Rc<Cell<bool>>);
    impl Render for Probe {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let hit = self.0.clone();
            div()
                .id("file-drop-probe")
                .w(px(200.))
                .h(px(100.))
                .on_drop(move |_: &ExternalPaths, _, _: &mut App| hit.set(true))
                .child("Drop")
        }
    }
    #[gpui::test]
    fn simulated_file_drop_event_reaches_test_window(cx: &mut gpui::TestAppContext) {
        let hit = Rc::new(Cell::new(false));
        let flag = hit.clone();
        let (_, window) = cx.add_window_view(move |_, _| Probe(flag));
        window.update(|window, app| {
            window.activate_window();
            window.draw(app).clear(app);
        });
        let position = gpui::point(px(30.), px(30.));
        window.simulate_event(FileDropEvent::Entered {
            position,
            paths: ExternalPaths([PathBuf::from("/tmp/t18-probe.png")].into_iter().collect()),
        });
        window.update(|window, app| {
            window.draw(app).clear(app);
        });
        window.simulate_event(FileDropEvent::Submit { position });
        assert!(
            hit.get(),
            "simulated file drop was not delivered to the target"
        );
    }
}

#[cfg(test)]
#[path = "file_drop_tests.rs"]
mod file_drop_tests;
