//! Controlled radio input with RustUI theme artwork and GPUI Base behavior.

use gpui::{
    AnyElement, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_base::Radio as BaseRadio;
use gpui_component::theme::try_parse_color;
use rustui_core::{Binding, Identity};

use super::checkbox::{appearance_color, rectangular_artwork_size};
use crate::components::theme::{required_style_with_artwork, selector};
use crate::theme::registry::active_theme;
use crate::theme::{AppearanceMode, ThemeProfile};
use crate::view::{EmptyView, RenderContext, View};

fn radio_label_color(
    profile: &ThemeProfile,
    mode: AppearanceMode,
    disabled: bool,
    theme_foreground: Option<gpui::Hsla>,
) -> Option<gpui::Hsla> {
    if disabled {
        Some(
            try_parse_color(&profile.colors(Some(mode)).disabled_text)
                .expect("validated RustUI radio disabled text color"),
        )
    } else {
        theme_foreground
    }
}

#[derive(Clone)]
enum RadioAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_message_label(
    localized: SharedString,
    id: &SharedString,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.clone()
    } else {
        localized
    }
}

pub struct RadioView {
    label: SharedString,
    label_message_id: Option<SharedString>,
    selected: Binding<bool>,
    id: Option<Identity>,
    disabled: bool,
    accessibility_label: Option<RadioAccessibilityLabel>,
    tab_index: isize,
    focusable: bool,
}

impl RadioView {
    pub fn new(label: impl Into<SharedString>, selected: Binding<bool>) -> Self {
        let label = label.into();
        Self {
            label,
            label_message_id: None,
            selected,
            id: None,
            disabled: false,
            accessibility_label: None,
            tab_index: 0,
            focusable: true,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(!id.as_str().trim().is_empty(), "Radio IDs must be nonempty");
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
            "radio label localization key must not be blank"
        );
        self.label_message_id = Some(id);
        self
    }

    fn resolved_label(&self, localized_message: impl FnOnce(&str) -> String) -> SharedString {
        self.label_message_id.as_ref().map_or_else(
            || self.label.clone(),
            |id| resolved_message_label(localized_message(id).into(), id, &self.label),
        )
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "radio accessibility label must not be empty"
        );
        self.accessibility_label = Some(RadioAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves this radio's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "Radio accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(RadioAccessibilityLabel::Literal(label))
            | Some(RadioAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(RadioAccessibilityLabel::Message { id, fallback });
        self
    }

    fn resolved_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> SharedString {
        let label = match &self.accessibility_label {
            Some(RadioAccessibilityLabel::Literal(label)) => label.clone(),
            Some(RadioAccessibilityLabel::Message { id, fallback }) => {
                resolved_message_label(localized_message(id.as_ref()).into(), id, fallback)
            }
            None => self.resolved_label(localized_message),
        };
        assert!(
            !label.trim().is_empty(),
            "radio needs a non-empty accessible name"
        );
        label
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
            .unwrap_or_else(|| allocate_implicit(Identity::from("radio")))
    }
}

#[cfg(test)]
mod accessibility_tests {
    use super::*;

    fn binding() -> Binding<bool> {
        Binding::new(false, |value: &bool| *value, |value, next| *value = next)
    }

    #[test]
    fn visible_label_provides_the_default_accessible_name() {
        let radio = Radio("Monthly", binding());

        assert_eq!(radio.label.as_ref(), "Monthly");
        assert!(radio.accessibility_label.is_none());
    }

    #[test]
    fn localized_accessible_name_uses_active_catalog_and_preserves_fallback() {
        let radio = Radio("Monthly", binding())
            .accessibility_label("Monthly plan")
            .localized_accessibility_label("radio.monthly");
        let Some(RadioAccessibilityLabel::Message { id, fallback }) = &radio.accessibility_label
        else {
            panic!("expected localized Radio accessibility name");
        };
        assert_eq!(id.as_ref(), "radio.monthly");
        assert_eq!(fallback.as_ref(), "Monthly plan");
        assert_eq!(
            radio
                .resolved_accessibility_label(|_| "Forfait mensuel".to_owned())
                .as_ref(),
            "Forfait mensuel"
        );
        assert_eq!(
            radio.resolved_accessibility_label(|_| "radio.monthly".to_owned()),
            fallback.clone()
        );
        assert_eq!(
            radio.resolved_accessibility_label(|_| " \tradio.monthly\n".to_owned()),
            fallback.clone()
        );
        assert_eq!(
            radio.resolved_accessibility_label(|_| " \t".to_owned()),
            fallback.clone()
        );
        let visible_fallback = Radio("Monthly", binding())
            .localized_accessibility_label("radio.monthly")
            .resolved_accessibility_label(|_| String::new());
        assert_eq!(visible_fallback.as_ref(), "Monthly");
    }

    #[test]
    fn localized_accessible_name_rejects_blank_message_id() {
        assert!(
            std::panic::catch_unwind(|| {
                Radio("Monthly", binding()).localized_accessibility_label(" \t");
            })
            .is_err()
        );
    }

    #[test]
    fn indicator_only_radio_can_use_an_explicit_accessible_name() {
        let radio = Radio("", binding()).accessibility_label("Choose monthly");

        assert!(radio.label.is_empty());
        assert_eq!(
            radio
                .resolved_accessibility_label(|_| unreachable!())
                .as_ref(),
            "Choose monthly"
        );

        assert!(
            std::panic::catch_unwind(|| {
                Radio(" \t", binding()).resolved_accessibility_label(|_| unreachable!());
            })
            .is_err()
        );
    }

    #[test]
    fn explicit_accessible_name_must_not_be_blank() {
        assert!(
            std::panic::catch_unwind(|| {
                Radio("Monthly", binding()).accessibility_label("  ");
            })
            .is_err()
        );
    }

    #[test]
    fn disabled_label_uses_the_active_platform_disabled_text_color() {
        let profile = crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json"))
            .unwrap()
            .platform_snapshot;
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let foreground = gpui::hsla(0.25, 0.5, 0.75, 1.0);
            assert_eq!(
                radio_label_color(&profile, mode, true, Some(foreground)),
                Some(try_parse_color(&profile.colors(Some(mode)).disabled_text).unwrap())
            );
        }
    }

    #[test]
    fn enabled_label_keeps_the_resolved_theme_foreground() {
        let profile = crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json"))
            .unwrap()
            .platform_snapshot;
        let document =
            crate::theme::parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let style = crate::theme::resolve_component_style_with(
                &document,
                "radio_button",
                "body",
                &crate::components::theme::selector_for_mode(mode),
                &["foreground", "vector_path", "blend_mode"],
            )
            .unwrap();
            let foreground = crate::theme::registry::hsla(style.colors["foreground"]);
            assert_eq!(
                radio_label_color(&profile, mode, false, Some(foreground)),
                Some(foreground)
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
        let original = Radio("Monthly", binding()).identity_with(allocate_at_slot(0));
        let translated = Radio("Mensuel", binding()).identity_with(allocate_at_slot(0));
        let sibling = Radio("Monthly", binding()).identity_with(allocate_at_slot(1));

        assert_eq!(original, translated);
        assert_ne!(original, sibling);
    }
}

#[allow(non_snake_case)]
pub fn Radio(label: impl Into<SharedString>, selected: Binding<bool>) -> RadioView {
    RadioView::new(label, selected)
}

impl View for RadioView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let style =
            required_style_with_artwork(context, "radio_button", "body", &selector(context));
        let (width, height) = rectangular_artwork_size(&style);
        assert_eq!(
            style.lengths["width"], width,
            "radio width disagrees with artwork"
        );
        assert_eq!(
            style.lengths["height"], height,
            "radio height disagrees with artwork"
        );
        let radius = style.lengths["corner_radius"];
        let accent = crate::theme::registry::hsla(style.colors["background"]);
        let foreground = crate::theme::registry::hsla(style.colors["foreground"]);
        // macos27 declares the unselected role as `border`.
        // The component record maps its `border` role to the profile separator.
        let active = active_theme(context.app()).expect("radio requires an active RustUI theme");
        let unchecked_border = appearance_color(&active.profile, active.mode, "border");
        // No unchecked fill is declared in JSON, so leave the indicator clear.
        let unchecked_background = gpui::hsla(0.0, 0.0, 0.0, 0.0);
        let focus_color = appearance_color(&active.profile, active.mode, "focus");
        let border_width = active.profile.borders.thin as f32;
        let focus_width = active.profile.accessibility.focus_width as f32;
        let label_color = radio_label_color(
            &active.profile,
            active.mode,
            self.disabled,
            style
                .colors
                .get("foreground")
                .map(|color| crate::theme::registry::hsla(*color)),
        );
        let selected = self.selected.value();
        let identity = self.identity(context);
        let write = context.binding_writer(self.selected.clone());
        let indicator = div()
            .w(px(width))
            .h(px(height))
            .rounded(px(radius))
            .flex()
            .items_center()
            .justify_center()
            .border(px(border_width))
            .border_color(if selected { accent } else { unchecked_border })
            .bg(if selected {
                accent
            } else {
                unchecked_background
            })
            .when(self.disabled, |this| this.opacity(0.5))
            .when(selected, |this| {
                this.child(div().size(px(6.0)).rounded(px(3.0)).bg(foreground))
            });
        let mut label = div().child(self.resolved_label(|id| context.localized_message(id)));
        if let Some(color) = label_color {
            label = label.text_color(color);
        }
        if let Some(typography) = &style.typography {
            label = label
                .font_family(typography.family.clone())
                .text_size(px(typography.size))
                .font_weight(FontWeight(typography.weight));
        }
        super::logical_input_row(
            BaseRadio::new(identity.to_string()),
            context.layout_direction(),
        )
        .gap_2()
        .items_center()
        .focus(move |style| style.border(px(focus_width)).border_color(focus_color))
        .child(indicator)
        .child(label)
        .checked(selected)
        .disabled(self.disabled)
        .tab_index(self.tab_index)
        .tab_stop(self.focusable)
        .accessibility_label(self.resolved_accessibility_label(|id| context.localized_message(id)))
        .when(self.disabled, |control| {
            control.a11y_synthetic_children(|builder| {
                super::mark_accessibility_node_disabled(builder.parent_node());
            })
        })
        .on_change(move |selected, _event, _window, cx| write(selected, cx))
        .into_any_element()
    }
}

#[cfg(test)]
mod visible_label_localization_tests {
    use super::*;

    #[test]
    fn translated_label_uses_catalog_or_constructor_fallback() {
        let binding = Binding::new(false, |value: &bool| *value, |value, next| *value = next);
        let control = Radio("Selected", binding).localized_label("input.visible");
        assert_eq!(
            control
                .resolved_label(|_| "Sélectionné".to_owned())
                .as_ref(),
            "Sélectionné"
        );
        assert_eq!(control.resolved_label(str::to_owned).as_ref(), "Selected");
    }
}
