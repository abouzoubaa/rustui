//! Trigger-driven side sheet backed by GPUI Components.

use std::rc::Rc;

use gpui::{AnyElement, ParentElement as _, SharedString, Window};
use gpui_component::WindowExt as _;
use rustui_core::Identity;

use crate::{
    components::basic::button::Button,
    runtime::AppAction,
    view::{AnyView, EmptyView, RenderContext, RetentionScope, View},
};

/// Keeps arbitrary RustUI sheet content owned across native builder rerenders.
struct SheetContent(Rc<AnyView>);

impl View for SheetContent {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        self.0.into_element(context)
    }
}

/// A trigger that opens one GPUI Components side sheet in its window.
///
/// The sheet uses the upstream default right placement and sizing. This first
/// slice has no controlled visibility or owner-driven close operation.
pub struct SheetView {
    id: Identity,
    trigger: SharedString,
    title: Option<SharedString>,
    content: Option<Rc<AnyView>>,
}

impl SheetView {
    pub fn new(id: impl Into<Identity>, trigger: impl Into<SharedString>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Sheet identity must be nonempty"
        );
        Self {
            id,
            trigger: trigger.into(),
            title: None,
            content: None,
        }
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Adds arbitrary RustUI content to the sheet body.
    pub fn content<V: View>(mut self, content: V) -> Self {
        self.content = Some(Rc::new(AnyView::new(content)));
        self
    }
}

/// Creates a trigger-driven side sheet using upstream default placement.
#[allow(non_snake_case)]
pub fn Sheet(id: impl Into<Identity>, trigger: impl Into<SharedString>) -> SheetView {
    SheetView::new(id, trigger)
}

impl View for SheetView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Key(self.id.clone()), |context| {
            let content = self.content.as_ref().map(|content| {
                context.retained_component(Identity::from(format!("{}:content", self.id)), || {
                    SheetContent(Rc::clone(content))
                })
            });
            let title = self.title.clone();
            let id = self.id.clone();
            let open = AppAction::with_window_action(move |_, window: &mut Window, cx| {
                let content = content.clone();
                let title = title.clone();
                window.open_sheet(cx, move |mut sheet, _, _| {
                    if let Some(title) = title.clone() {
                        sheet = sheet.title(title);
                    }
                    if let Some(body) = content.clone() {
                        sheet = sheet.child(body.clone());
                    }
                    sheet
                });
            });

            Button(self.trigger.clone(), open)
                .id(format!("{}:trigger", id))
                .into_element(context)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sheet_requires_a_nonempty_stable_identity() {
        assert_eq!(Sheet("details", "Open").id.as_str(), "details");
        assert!(std::panic::catch_unwind(|| Sheet(" ", "Open")).is_err());
    }

    #[test]
    fn title_and_content_are_optional() {
        let sheet = Sheet("details", "Open").title("Details").content(EmptyView);

        assert!(sheet.title.is_some());
        assert!(sheet.content.is_some());
    }
}
