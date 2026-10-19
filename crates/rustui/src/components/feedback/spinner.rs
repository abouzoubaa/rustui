//! Indeterminate loading spinner backed by pinned GPUI Components.

use gpui::{
    AnyElement, InteractiveElement, IntoElement, ParentElement, Role, SharedString,
    StatefulInteractiveElement, div,
};
use gpui_component::spinner::Spinner as GpuiSpinner;
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, View};

/// An indeterminate loading indicator.
#[derive(Clone)]
pub struct SpinnerView {
    id: Option<Identity>,
    accessibility_label: SharedString,
}

impl Default for SpinnerView {
    fn default() -> Self {
        Self {
            id: None,
            accessibility_label: "Loading".into(),
        }
    }
}

impl SpinnerView {
    /// Sets a stable ID when multiple spinners share one view tree.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Names the operation for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = label.into();
        self
    }
}

/// Creates an indeterminate loading indicator.
#[allow(non_snake_case)]
pub fn Spinner() -> SpinnerView {
    SpinnerView::default()
}

impl View for SpinnerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        div()
            .id(SharedString::from(
                self.id
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| "rustui-spinner".to_owned()),
            ))
            .role(Role::ProgressIndicator)
            .aria_label(self.accessibility_label.clone())
            .child(GpuiSpinner::new())
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spinner_has_an_accessible_name() {
        assert_eq!(Spinner().accessibility_label.as_ref(), "Loading");
        assert_eq!(
            Spinner()
                .accessibility_label("Downloading")
                .accessibility_label
                .as_ref(),
            "Downloading"
        );
    }

    #[test]
    fn spinner_accepts_an_explicit_id() {
        assert_eq!(
            Spinner().id("upload-spinner").id,
            Some(Identity::from("upload-spinner"))
        );
    }
}
