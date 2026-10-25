//! Controlled checkbox with RustUI theme artwork and GPUI Base behavior.

use gpui::{
    AnyElement, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_base::{Checkbox as BaseCheckbox, CheckboxState};
use gpui_component::theme::try_parse_color;
use rustui_core::{Binding, Identity};

use crate::components::theme::{required_style_with_artwork, selector};
use crate::theme::{AppearanceMode, ResolvedComponentStyle, ThemeProfile, registry::active_theme};
use crate::view::{EmptyView, RenderContext, View};

/// The macOS 27 source art is an axis-aligned SVG rectangle. Read its actual
/// bounds so a changed path cannot silently keep an unrelated fixed size.
pub(super) fn rectangular_artwork_size(style: &ResolvedComponentStyle) -> (f32, f32) {
    let artwork = &style.extra_properties["vector_path"]["value"];
    let data = artwork["data"]
        .as_str()
        .expect("checkbox/radio vector artwork needs SVG path data");
    let commands: String = data.chars().filter(char::is_ascii_alphabetic).collect();
    assert_eq!(commands, "MHVHVZ", "unsupported checkbox/radio artwork");
    let coordinates: Vec<f32> = data
        .split(|ch: char| ch.is_ascii_alphabetic() || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            part.parse()
                .expect("invalid checkbox/radio path coordinate")
        })
        .collect();
    assert_eq!(coordinates.len(), 6, "invalid checkbox/radio rectangle");
    assert_eq!(
        coordinates[0], coordinates[4],
        "path does not close at left"
    );
    assert_eq!(coordinates[1], coordinates[5], "path does not close at top");
    let transform = artwork["transform"]
        .as_array()
        .expect("checkbox/radio artwork transform is missing");
    assert_eq!(transform.len(), 6, "invalid artwork transform");
    let identity = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
    assert!(
        transform
            .iter()
            .zip(identity)
            .all(|(actual, expected)| actual.as_f64() == Some(expected)),
        "transformed checkbox/radio artwork is unsupported"
    );
    let width = coordinates[2] - coordinates[0];
    let height = coordinates[3] - coordinates[1];
    assert!(width > 0.0 && height > 0.0, "invalid artwork size");
    (width, height)
}

pub(super) fn appearance_color(
    profile: &ThemeProfile,
    mode: AppearanceMode,
    role: &str,
) -> gpui::Hsla {
    let colors = profile.colors(Some(mode));
    let value = match role {
        // macos27 maps the component's `border` role to the global `separator` role.
        "border" => &colors.separator,
        "focus" => &colors.focus_ring,
        _ => panic!("unsupported checkbox theme role `{role}`"),
    };
    try_parse_color(value).expect("validated RustUI appearance color")
}

fn disabled_label_color(profile: &ThemeProfile, mode: AppearanceMode) -> gpui::Hsla {
    try_parse_color(&profile.colors(Some(mode)).disabled_text)
        .expect("validated RustUI checkbox disabled text color")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::components::theme::selector_for_mode;
    use crate::theme::{AppearanceMode, parse_theme, resolve_component_style_with};

    #[test]
    fn macos27_choice_artwork_has_real_sixteen_pixel_bounds() {
        let document = parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            for component in ["checkbox", "radio_button"] {
                let style = resolve_component_style_with(
                    &document,
                    component,
                    "body",
                    &selector_for_mode(mode),
                    &["vector_path", "blend_mode"],
                )
                .unwrap();
                assert_eq!(rectangular_artwork_size(&style), (16.0, 16.0));
            }
        }
    }

    #[test]
    fn unchecked_choice_fallback_uses_declared_border_role() {
        let document = parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        for (component, state) in [("checkbox", "unchecked"), ("radio_button", "unselected")] {
            let record = document.component_theme(component).unwrap();
            assert_eq!(record.metadata["theme_roles"][state], "border");
        }
    }

    #[test]
    fn choice_semantic_roles_resolve_to_active_profile_colors() {
        let document = parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let colors = document.platform_snapshot.colors(Some(mode));
            assert_eq!(
                appearance_color(&document.platform_snapshot, mode, "border"),
                try_parse_color(&colors.separator).unwrap()
            );
            assert_eq!(
                appearance_color(&document.platform_snapshot, mode, "focus"),
                try_parse_color(&colors.focus_ring).unwrap()
            );
        }
    }

    #[test]
    fn disabled_label_uses_the_platform_disabled_text_color() {
        let profile = parse_theme(include_str!("../../../../../themes/macos27.json"))
            .unwrap()
            .platform_snapshot;
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            assert_eq!(
                disabled_label_color(&profile, mode),
                try_parse_color(&profile.colors(Some(mode)).disabled_text).unwrap()
            );
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum CheckboxAccessibilityLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_message_label(id: &str, localized: String, fallback: &SharedString) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

pub struct CheckboxView {
    label: SharedString,
    checked: Binding<bool>,
    id: Option<Identity>,
    disabled: bool,
    accessibility_label: Option<CheckboxAccessibilityLabel>,
    tab_index: isize,
    focusable: bool,
}

impl CheckboxView {
    /// Creates a checkbox whose non-empty visible label is also its default
    /// accessible name.
    pub fn new(label: impl Into<SharedString>, checked: Binding<bool>) -> Self {
        let label = label.into();
        Self {
            label,
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
            "Checkbox IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Overrides the accessible name with a non-empty label.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "checkbox accessibility label must not be empty"
        );
        self.accessibility_label = Some(CheckboxAccessibilityLabel::Literal(label));
        self
    }

    /// Resolves checkbox's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "checkbox accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(CheckboxAccessibilityLabel::Literal(label))
            | Some(CheckboxAccessibilityLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(CheckboxAccessibilityLabel::Message { id, fallback });
        self
    }

    fn resolved_accessibility_label(&self, context: &RenderContext<'_>) -> SharedString {
        let label = match &self.accessibility_label {
            Some(CheckboxAccessibilityLabel::Literal(label)) => label.clone(),
            Some(CheckboxAccessibilityLabel::Message { id, fallback }) => resolved_message_label(
                id.as_ref(),
                context.localized_message(id.as_ref()),
                fallback,
            ),
            None => self.label.clone(),
        };
        assert!(
            !label.trim().is_empty(),
            "checkbox needs a non-empty accessible name"
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
            .unwrap_or_else(|| allocate_implicit(Identity::from("checkbox")))
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
        let checkbox = Checkbox("Accepted", binding());

        assert_eq!(checkbox.label.as_ref(), "Accepted");
        assert_eq!(checkbox.accessibility_label, None);
    }

    #[test]
    fn indicator_only_checkbox_can_use_an_explicit_accessible_name() {
        let checkbox = Checkbox("", binding()).accessibility_label("Accept the terms");

        assert!(checkbox.label.is_empty());
        assert_eq!(
            checkbox.accessibility_label,
            Some(CheckboxAccessibilityLabel::Literal("Accept the terms".into()))
        );
    }

    #[test]
    fn explicit_accessible_name_must_not_be_blank() {
        let result = std::panic::catch_unwind(|| {
            Checkbox("Accepted", binding()).accessibility_label("  ");
        });

        assert!(result.is_err());
    }

    #[test]
    fn localized_accessible_name_preserves_fallback_and_replaces_message_key() {
        let checkbox = Checkbox("Accepted", binding())
            .accessibility_label("Accept the terms")
            .localized_accessibility_label("checkbox.accept_terms")
            .localized_accessibility_label("checkbox.terms");

        assert!(matches!(
            checkbox.accessibility_label,
            Some(CheckboxAccessibilityLabel::Message { id, fallback })
                if id.as_ref() == "checkbox.terms" && fallback.as_ref() == "Accept the terms"
        ));
    }

    #[test]
    fn localized_accessible_name_uses_catalog_value_or_fallback() {
        let id = SharedString::from("checkbox.accepted");
        let fallback = SharedString::from("Accepted");

        assert_eq!(
            resolved_message_label(id.as_ref(), "Accepté".to_owned(), &fallback,).as_ref(),
            "Accepté"
        );
        assert_eq!(
            resolved_message_label(id.as_ref(), id.to_string(), &fallback),
            fallback
        );
        assert_eq!(
            resolved_message_label(id.as_ref(), " \t".to_owned(), &fallback),
            fallback
        );
    }

    #[test]
    #[should_panic(expected = "checkbox accessibility localization key must not be blank")]
    fn localized_accessible_name_rejects_blank_message_id() {
        Checkbox("Accepted", binding()).localized_accessibility_label(" \t ");
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
        let original = Checkbox("Accepted", binding()).identity_with(allocate_at_slot(0));
        let translated = Checkbox("Accepté", binding()).identity_with(allocate_at_slot(0));
        let sibling = Checkbox("Accepted", binding()).identity_with(allocate_at_slot(1));

        assert_eq!(original, translated);
        assert_ne!(original, sibling);
    }
}

#[allow(non_snake_case)]
pub fn Checkbox(label: impl Into<SharedString>, checked: Binding<bool>) -> CheckboxView {
    CheckboxView::new(label, checked)
}

impl View for CheckboxView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let style = required_style_with_artwork(context, "checkbox", "body", &selector(context));
        let (width, height) = rectangular_artwork_size(&style);
        let accent = crate::theme::registry::hsla(style.colors["background"]);
        let foreground = crate::theme::registry::hsla(style.colors["foreground"]);
        // macos27 declares the unchecked role as `border`; the artwork only
        // defines the checked appearance.
        // macos27 defines `unchecked: border` and maps that role to separator.
        let active = active_theme(context.app()).expect("checkbox requires an active RustUI theme");
        let unchecked_border = appearance_color(&active.profile, active.mode, "border");
        // The JSON has no unchecked fill property; keep the unchecked artwork clear.
        let unchecked_background = gpui::hsla(0.0, 0.0, 0.0, 0.0);
        let focus_color = appearance_color(&active.profile, active.mode, "focus");
        let border_width = active.profile.borders.thin as f32;
        let focus_width = active.profile.accessibility.focus_width as f32;
        let label_disabled_color = self
            .disabled
            .then(|| disabled_label_color(&active.profile, active.mode));
        let checked = self.checked.value();
        let identity = self.identity(context);
        let write = context.binding_writer(self.checked.clone());
        let indicator = div()
            .w(px(width))
            .h(px(height))
            .flex()
            .items_center()
            .justify_center()
            .border(px(border_width))
            .border_color(if checked { accent } else { unchecked_border })
            .bg(if checked {
                accent
            } else {
                unchecked_background
            })
            // Keep disabled mark/fill/border dimming while leaving label color
            // to the active profile's semantic disabled-text token.
            .when(self.disabled, |this| this.opacity(0.5))
            .when(checked, |this| {
                this.child(div().text_color(foreground).text_size(px(12.0)).child("✓"))
            });
        let mut label = div().child(self.label.clone());
        if let Some(color) = label_disabled_color {
            label = label.text_color(color);
        } else if let Some(color) = style.colors.get("foreground") {
            label = label.text_color(crate::theme::registry::hsla(*color));
        }
        if let Some(typography) = &style.typography {
            label = label
                .font_family(typography.family.clone())
                .text_size(px(typography.size))
                .font_weight(FontWeight(typography.weight));
        }
        BaseCheckbox::new(identity.to_string())
            .flex()
            .gap_2()
            .items_center()
            .focus(move |style| style.border(px(focus_width)).border_color(focus_color))
            .child(indicator)
            .child(label)
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
            .on_change(move |state, _event, _window, cx| write(state == CheckboxState::Checked, cx))
            .into_any_element()
    }
}
