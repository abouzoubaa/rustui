//! Native pointer interactions for arbitrary RustUI child subtrees.

use std::{error::Error, fmt};

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement as _, ParentElement as _,
    StatefulInteractiveElement as _, div,
};
use rustui_core::Identity;

use crate::view::{ControlAction, EmptyView, RenderContext, RetentionScope, View};

/// A pointer hover region around an arbitrary child view.
///
/// GPUI owns hit testing and emits transitions while the region is mounted.
/// The region adds no keyboard focus target. Essential information shown on
/// hover must also be available through a keyboard-accessible path.
pub struct HoverRegionView<V: View> {
    id: Identity,
    child: V,
    on_enter: Option<ControlAction>,
    on_exit: Option<ControlAction>,
}

impl<V: View> HoverRegionView<V> {
    pub fn new(id: impl Into<Identity>, child: V) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "HoverRegion identity must be nonempty"
        );
        Self {
            id,
            child,
            on_enter: None,
            on_exit: None,
        }
    }

    /// Runs an action when GPUI reports that the pointer entered this region.
    /// A click-only `AppAction` is rejected at view construction.
    pub fn on_enter(self, action: impl Into<ControlAction>) -> Self {
        self.try_on_enter(action)
            .unwrap_or_else(|error| panic!("invalid hover enter action: {error}"))
    }

    /// Runs an action when GPUI reports that the pointer exited this region.
    /// Removal of the region does not guarantee an exit event.
    pub fn on_exit(self, action: impl Into<ControlAction>) -> Self {
        self.try_on_exit(action)
            .unwrap_or_else(|error| panic!("invalid hover exit action: {error}"))
    }

    pub fn try_on_enter(
        mut self,
        action: impl Into<ControlAction>,
    ) -> Result<Self, HoverActionError> {
        self.on_enter = Some(validated_action(action.into())?);
        Ok(self)
    }

    pub fn try_on_exit(
        mut self,
        action: impl Into<ControlAction>,
    ) -> Result<Self, HoverActionError> {
        self.on_exit = Some(validated_action(action.into())?);
        Ok(self)
    }
}

#[allow(non_snake_case)]
pub fn HoverRegion<V: View>(id: impl Into<Identity>, child: V) -> HoverRegionView<V> {
    HoverRegionView::new(id, child)
}

impl<V: View> View for HoverRegionView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Hover(self.id.clone()), |context| {
            let on_enter = self
                .on_enter
                .clone()
                .map(|action| context.bind_context_action(action));
            let on_exit = self
                .on_exit
                .clone()
                .map(|action| context.bind_context_action(action));
            let child = self.child.into_element(context);
            div()
                .id(format!("rustui-hover-region-{}", self.id.as_str()))
                .on_hover(move |is_hovered, window, cx| {
                    let action = if *is_hovered { &on_enter } else { &on_exit };
                    if let Some(action) = action {
                        action(window, cx);
                    }
                })
                .child(child)
                .into_any_element()
        })
    }
}

fn validated_action(action: ControlAction) -> Result<ControlAction, HoverActionError> {
    if matches!(&action, ControlAction::Application(action) if !action.keyboard_compatible()) {
        Err(HoverActionError::ClickOnlyAction)
    } else {
        Ok(action)
    }
}

/// An application action that requires a click event cannot observe hover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HoverActionError {
    ClickOnlyAction,
}

impl fmt::Display for HoverActionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("action requires a click event")
    }
}

impl Error for HoverActionError {}

/// A pointer click region around an arbitrary child view.
///
/// The wrapper is not a keyboard focus target. Provide a separate keyboard
/// control for any essential action. GPUI delivers native clicks in the bubble
/// phase; this wrapper does not stop propagation or prevent defaults.
pub struct TapRegionView<V: View> {
    id: Identity,
    child: V,
    action: ControlAction,
    enabled: bool,
}

impl<V: View> TapRegionView<V> {
    pub fn new(id: impl Into<Identity>, child: V, action: impl Into<ControlAction>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "TapRegion identity must be nonempty"
        );
        Self {
            id,
            child,
            action: action.into(),
            enabled: true,
        }
    }

    /// Disables click handling while keeping the child rendered.
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[allow(non_snake_case)]
pub fn TapRegion<V: View>(
    id: impl Into<Identity>,
    child: V,
    action: impl Into<ControlAction>,
) -> TapRegionView<V> {
    TapRegionView::new(id, child, action)
}

impl<V: View> View for TapRegionView<V> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(RetentionScope::Tap(self.id.clone()), |context| {
            let child = self.child.into_element(context);
            let region = div()
                .id(format!("rustui-tap-region-{}", self.id.as_str()))
                .child(child);
            if self.enabled {
                region
                    .on_click(context.bind_control_action(self.action.clone()))
                    .into_any_element()
            } else {
                region.into_any_element()
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime::AppAction;

    #[test]
    fn stores_enter_and_exit_actions_independently() {
        let region = HoverRegion("scope", EmptyView)
            .on_enter(AppAction::new(|_| {}))
            .on_exit(AppAction::new(|_| {}));
        assert!(region.on_enter.is_some());
        assert!(region.on_exit.is_some());
    }

    #[test]
    fn rejects_click_only_and_composed_click_actions() {
        let click_only = AppAction::with_window_action(|_, _, _| {});
        assert!(matches!(
            HoverRegion("scope", EmptyView).try_on_enter(click_only.clone()),
            Err(HoverActionError::ClickOnlyAction)
        ));
        assert!(matches!(
            HoverRegion("scope", EmptyView).try_on_exit(AppAction::new(|_| {}).then(click_only)),
            Err(HoverActionError::ClickOnlyAction)
        ));
    }

    #[test]
    #[should_panic(expected = "HoverRegion identity must be nonempty")]
    fn rejects_empty_identity() {
        let _ = HoverRegion(" ", EmptyView);
    }

    #[test]
    fn tap_region_accepts_component_and_application_actions() {
        let component = rustui_core::Action::new(|_: &mut ()| {});
        let region = TapRegion("tap", EmptyView, component).enabled(false);
        assert_eq!(region.id.as_str(), "tap");
        assert!(!region.enabled);

        let app = TapRegion(
            "app",
            EmptyView,
            AppAction::with_window_action(|_, _, _| {}),
        );
        assert!(app.enabled);
    }

    #[test]
    #[should_panic(expected = "TapRegion identity must be nonempty")]
    fn tap_region_rejects_empty_identity() {
        let _ = TapRegion("  ", EmptyView, AppAction::new(|_| {}));
    }
}
