//! Progress indicators backed by the pinned GPUI Components progress bar.

use gpui::{AnyElement, IntoElement, SharedString};
use gpui_component::progress::Progress as GpuiProgress;

use crate::view::{EmptyView, RenderContext, View};

/// A linear progress indicator whose value is a percentage from 0 to 100.
#[derive(Clone)]
pub struct ProgressView {
    id: SharedString,
    value: f32,
    indeterminate: bool,
    accessibility_label: SharedString,
}

impl ProgressView {
    pub fn new(id: impl Into<SharedString>, value: f32) -> Self {
        assert!(value.is_finite(), "progress value must be finite");
        Self {
            id: id.into(),
            value: value.clamp(0.0, 100.0),
            indeterminate: false,
            accessibility_label: "Progress".into(),
        }
    }

    /// Shows the upstream indeterminate loading animation instead of a value.
    ///
    /// GPUI Components applies the app's reduced-motion setting to this mode.
    pub fn indeterminate(mut self, indeterminate: bool) -> Self {
        self.indeterminate = indeterminate;
        self
    }

    /// Names the operation for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = label.into();
        self
    }
}

/// Creates a progress bar with a stable element ID; it is determinate by default.
#[allow(non_snake_case)]
pub fn Progress(id: impl Into<SharedString>, value: f32) -> ProgressView {
    ProgressView::new(id, value)
}

impl View for ProgressView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, _: &mut RenderContext<'_>) -> AnyElement {
        GpuiProgress::new(self.id.clone())
            .value(self.value)
            .loading(self.indeterminate)
            .accessibility_label(self.accessibility_label.clone())
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_is_bounded_to_percentage() {
        assert_eq!(Progress("upload", -10.0).value, 0.0);
        assert_eq!(Progress("upload", 30.0).value, 30.0);
        assert_eq!(Progress("upload", 120.0).value, 100.0);
    }

    #[test]
    fn indeterminate_mode_is_opt_in_and_does_not_change_the_value() {
        let determinate = Progress("upload", 30.0);
        assert!(!determinate.indeterminate);
        assert_eq!(determinate.value, 30.0);

        let loading = determinate.indeterminate(true);
        assert!(loading.indeterminate);
        assert_eq!(loading.value, 30.0);
        assert_eq!(loading.accessibility_label.as_ref(), "Progress");
    }

    #[test]
    #[should_panic(expected = "progress value must be finite")]
    fn non_finite_value_is_rejected() {
        Progress("upload", f32::NAN);
    }
}
