//! Controlled switch input with a RustUI-themed track and GPUI Base behavior.

use gpui::{
    AbsoluteLength, AnyElement, App, EdgesRefinement, Hsla, IntoElement, ParentElement as _,
    RenderOnce, SharedString, StatefulInteractiveElement as _, Styled as _, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_base::Switch as BaseSwitch;
use gpui_component::theme::try_parse_color;
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector},
    theme::registry::active_theme,
    theme::{AppearanceMode, ThemeProfile},
    view::{EmptyView, RenderContext, View},
};

fn toggle_label_color(profile: &ThemeProfile, mode: AppearanceMode, disabled: bool) -> Hsla {
    let colors = profile.colors(Some(mode));
    let token = if disabled {
        &colors.disabled_text
    } else {
        &colors.primary_text
    };
    try_parse_color(token).expect("validated RustUI toggle label color")
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ToggleAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_message_label(
    id: &SharedString,
    localized: String,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

fn resolved_toggle_accessibility_label(
    label: &SharedString,
    accessibility_label: Option<&ToggleAccessibilityLabel>,
    localized_message: impl FnOnce(&str) -> String,
) -> SharedString {
    let label = match accessibility_label {
        Some(ToggleAccessibilityLabel::Literal(label)) => label.clone(),
        Some(ToggleAccessibilityLabel::Message { id, fallback }) => {
            resolved_message_label(id, localized_message(id.as_ref()), fallback)
        }
        None => label.clone(),
    };
    assert!(
        !label.trim().is_empty(),
        "toggle needs a non-empty accessible name"
    );
    label
}

pub struct ToggleView {
    label: SharedString,
    label_message_id: Option<SharedString>,
    checked: Binding<bool>,
    id: Option<Identity>,
    disabled: bool,
    accessibility_label: Option<ToggleAccessibilityLabel>,
    tab_index: isize,
    focusable: bool,
}

impl ToggleView {
    pub fn new(label: impl Into<SharedString>, checked: Binding<bool>) -> Self {
        let label = label.into();
        Self {
            label,
            label_message_id: None,
            checked,
            id: None,
            disabled: false,
            accessibility_label: None,
            tab_index: 0,
            focusable: true,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Toggle IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Resolves the visible label from the active catalog, retaining the constructor label as fallback.
    pub fn localized_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "toggle label localization key must not be blank"
        );
        self.label_message_id = Some(id);
        self
    }

    fn resolved_label(&self, localized_message: impl FnOnce(&str) -> String) -> SharedString {
        self.label_message_id.as_ref().map_or_else(
            || self.label.clone(),
            |id| resolved_message_label(id, localized_message(id), &self.label),
        )
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "toggle accessibility label must not be empty"
        );
        self.accessibility_label = Some(ToggleAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves the switch's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "toggle accessibility localization keys must be nonempty"
        );
        let fallback = match &self.accessibility_label {
            Some(ToggleAccessibilityLabel::Literal(label))
            | Some(ToggleAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(ToggleAccessibilityLabel::Message {
            id: message_id,
            fallback,
        });
        self
    }

    fn resolved_accessibility_label(&self, context: &RenderContext<'_>) -> SharedString {
        resolved_toggle_accessibility_label(
            &self.resolved_label(|id| context.localized_message(id)),
            self.accessibility_label.as_ref(),
            |id| context.localized_message(id),
        )
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate_implicit(Identity::from("toggle")))
    }
}

#[cfg(test)]
mod accessibility_tests {
    use super::*;
    use crate::theme::parse_theme;

    fn binding() -> Binding<bool> {
        Binding::new(false, |value: &bool| *value, |value, next| *value = next)
    }

    #[test]
    fn visible_label_provides_the_default_accessible_name() {
        let toggle = Toggle("Notifications", binding());

        assert_eq!(toggle.label.as_ref(), "Notifications");
        assert_eq!(toggle.accessibility_label, None);
    }

    #[test]
    fn indicator_only_toggle_can_use_an_explicit_accessible_name() {
        let toggle = Toggle("", binding()).accessibility_label("Notifications");

        assert!(toggle.label.is_empty());
        assert_eq!(
            resolved_toggle_accessibility_label(
                &toggle.label,
                toggle.accessibility_label.as_ref(),
                |_| unreachable!(),
            )
            .as_ref(),
            "Notifications"
        );
    }

    #[test]
    fn visible_name_must_not_be_blank_when_no_override_is_provided() {
        let toggle = Toggle(" \t", binding());
        assert!(
            std::panic::catch_unwind(|| {
                resolved_toggle_accessibility_label(&toggle.label, None, |_| unreachable!());
            })
            .is_err()
        );
    }

    #[test]
    fn explicit_accessible_name_must_not_be_blank() {
        assert!(
            std::panic::catch_unwind(|| {
                Toggle("Notifications", binding()).accessibility_label("  ");
            })
            .is_err()
        );
    }

    #[test]
    fn localized_accessible_name_uses_visible_label_as_untranslated_fallback() {
        let toggle = Toggle("Notifications", binding())
            .localized_accessibility_label("toggle.notifications");
        let Some(ToggleAccessibilityLabel::Message { id, fallback }) = toggle.accessibility_label
        else {
            panic!("expected localized toggle accessibility label");
        };

        assert_eq!(id.as_ref(), "toggle.notifications");
        assert_eq!(fallback.as_ref(), "Notifications");
        assert_eq!(
            resolved_message_label(&id, "Notifications sonores".into(), &fallback).as_ref(),
            "Notifications sonores"
        );
        assert_eq!(
            resolved_message_label(&id, id.to_string(), &fallback),
            fallback
        );
    }

    #[test]
    fn localized_accessible_name_rejects_blank_message_id() {
        assert!(
            std::panic::catch_unwind(|| {
                Toggle("Notifications", binding()).localized_accessibility_label(" \t ");
            })
            .is_err()
        );
    }

    #[test]
    fn disabled_label_uses_the_platform_disabled_text_color() {
        let profile = parse_theme(include_str!("../../../../../themes/macos27.json"))
            .unwrap()
            .platform_snapshot;
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            assert_eq!(
                toggle_label_color(&profile, mode, true),
                try_parse_color(&profile.colors(Some(mode)).disabled_text).unwrap()
            );
            assert_eq!(
                toggle_label_color(&profile, mode, false),
                try_parse_color(&profile.colors(Some(mode)).primary_text).unwrap()
            );
        }
    }
}

#[cfg(test)]
mod implicit_identity_tests {
    use super::*;

    fn binding() -> Binding<bool> {
        Binding::new(false, |value: &bool| *value, |value, next| *value = next)
    }

    #[test]
    fn identity_survives_label_localization_and_distinguishes_siblings() {
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };
        let original = Toggle("Notifications", binding()).identity_with(allocate_at_slot(0));
        let translated =
            Toggle("Notifications sonores", binding()).identity_with(allocate_at_slot(0));
        let sibling = Toggle("Notifications", binding()).identity_with(allocate_at_slot(1));

        assert_eq!(original, translated);
        assert_ne!(original, sibling);
    }
}

#[allow(non_snake_case)]
pub fn Toggle(label: impl Into<SharedString>, checked: Binding<bool>) -> ToggleView {
    ToggleView::new(label, checked)
}

impl View for ToggleView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity(context);
        let write = context.binding_writer(self.checked.clone());
        let style = required_style(context, "toggle", "body", &selector(context));
        let width = *style.lengths.get("width").expect("toggle track width");
        let height = *style.lengths.get("height").expect("toggle track height");
        let radius = *style
            .lengths
            .get("corner_radius")
            .expect("toggle track corner radius");
        let checked_color = crate::theme::registry::hsla(
            *style
                .colors
                .get("background")
                .expect("toggle checked background"),
        );
        let active_theme =
            active_theme(context.app()).expect("a RustUI component theme requires an active theme");
        let colors = active_theme.profile.colors(Some(active_theme.mode));
        // macos27 maps the `secondary` role to the toolbar surface.
        let unchecked_color = try_parse_color(&colors.toolbar_background)
            .expect("validated RustUI toolbar background");
        let foreground_color = toggle_label_color(&active_theme.profile, active_theme.mode, false);
        // macos27 declares `disabled: muted_foreground`; that global role resolves
        // to the profile's secondary text color.
        let disabled_thumb_color =
            try_parse_color(&colors.secondary_text).expect("validated RustUI secondary text color");
        let focus_color =
            try_parse_color(&colors.focus_ring).expect("validated RustUI focus ring color");
        let focus_border_width = active_theme.profile.borders.medium as f32;
        // The toggle record defines no thumb inset; preserve its explicit 2 px
        // geometry instead of borrowing a semantically unrelated profile token.
        let thumb_inset = 2.0;
        let label_color = if self.disabled {
            toggle_label_color(&active_theme.profile, active_theme.mode, true)
        } else {
            foreground_color
        };
        let checked = self.checked.value();
        let base = BaseSwitch::new(identity.to_string())
            .checked(checked)
            .disabled(self.disabled)
            .tab_index(self.tab_index)
            .tab_stop(self.focusable)
            .accessibility_label(self.resolved_accessibility_label(context))
            .when(self.disabled, |control| {
                control.a11y_synthetic_children(|builder| {
                    super::mark_accessibility_node_disabled(builder.parent_node());
                })
            })
            .on_change(move |checked, _, _, cx| write(checked, cx));

        ThemedSwitch {
            id: identity.to_string(),
            base,
            label: self.resolved_label(|id| context.localized_message(id)),
            checked,
            disabled: self.disabled,
            width,
            height,
            radius,
            checked_color,
            unchecked_color,
            foreground_color,
            disabled_thumb_color,
            label_color,
            focus_color,
            focus_border_width,
            thumb_inset,
            label_gap: active_theme.profile.spacing.space_2 as f32,
            direction: context.layout_direction(),
        }
        .into_any_element()
    }
}

#[derive(IntoElement)]
struct ThemedSwitch {
    id: String,
    base: BaseSwitch,
    label: SharedString,
    checked: bool,
    disabled: bool,
    width: f32,
    height: f32,
    radius: f32,
    checked_color: Hsla,
    unchecked_color: Hsla,
    foreground_color: Hsla,
    disabled_thumb_color: Hsla,
    label_color: Hsla,
    focus_color: Hsla,
    focus_border_width: f32,
    thumb_inset: f32,
    label_gap: f32,
    direction: rustui_i18n::LayoutDirection,
}

impl RenderOnce for ThemedSwitch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus = window
            .use_keyed_state(self.id, cx, |_, cx| cx.focus_handle())
            .read(cx)
            .clone();
        let focused = focus.is_focused(window);
        let track_color = if self.checked {
            self.checked_color
        } else {
            self.unchecked_color
        };
        let thumb_color = if self.disabled {
            self.disabled_thumb_color
        } else {
            self.foreground_color
        };

        let mut track = div()
            .relative()
            .flex_none()
            .w(px(self.width))
            .h(px(self.height))
            .rounded(px(self.radius))
            .bg(if self.disabled {
                track_color.opacity(0.5)
            } else {
                track_color
            })
            .child(
                div()
                    .absolute()
                    .top(px(self.thumb_inset))
                    .left(px(if self.checked {
                        self.width - self.height + self.thumb_inset
                    } else {
                        self.thumb_inset
                    }))
                    .size(px(self.height - (2.0 * self.thumb_inset)))
                    .rounded_full()
                    .bg(thumb_color),
            );
        if focused {
            let focus_border = Some(AbsoluteLength::Pixels(px(self.focus_border_width)));
            track.style().border_widths = EdgesRefinement {
                top: focus_border,
                right: focus_border,
                bottom: focus_border,
                left: focus_border,
            };
            track.style().border_color = Some(self.focus_color);
        }

        super::logical_input_row(self.base.track_focus(&focus), self.direction)
            .items_center()
            .gap(px(self.label_gap))
            .child(track)
            .child(div().text_color(self.label_color).child(self.label))
    }
}

#[cfg(test)]
mod visible_label_localization_tests {
    use super::*;

    #[test]
    fn translated_label_uses_catalog_or_constructor_fallback() {
        let binding = Binding::new(false, |value: &bool| *value, |value, next| *value = next);
        let control = Toggle("Enabled", binding).localized_label("input.visible");
        assert_eq!(
            control.resolved_label(|_| "Activé".to_owned()).as_ref(),
            "Activé"
        );
        assert_eq!(control.resolved_label(str::to_owned).as_ref(), "Enabled");
    }
}
