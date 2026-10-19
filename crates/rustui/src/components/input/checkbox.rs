//! Controlled checkbox with RustUI theme artwork and GPUI Base behavior.

use gpui::{
    AnyElement, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    Styled as _, div, prelude::FluentBuilder as _, px,
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
}

pub struct CheckboxView {
    label: SharedString,
    checked: Binding<bool>,
    id: Option<Identity>,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    tab_index: isize,
    focusable: bool,
}

impl CheckboxView {
    pub fn new(label: impl Into<SharedString>, checked: Binding<bool>) -> Self {
        Self {
            label: label.into(),
            checked,
            id: None,
            disabled: false,
            accessibility_label: None,
            tab_index: 0,
            focusable: true,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }

    fn identity(&self) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("checkbox:{}", self.label)))
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
        let checked = self.checked.value();
        let identity = self.identity();
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
            .when(checked, |this| {
                this.child(div().text_color(foreground).text_size(px(12.0)).child("✓"))
            });
        let mut label = div().child(self.label.clone());
        if let Some(color) = style.colors.get("foreground") {
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
            .when(self.disabled, |this| this.opacity(0.5))
            .child(indicator)
            .child(label)
            .checked(checked)
            .disabled(self.disabled)
            .tab_index(self.tab_index)
            .tab_stop(self.focusable)
            .accessibility_label(
                self.accessibility_label
                    .clone()
                    .unwrap_or_else(|| self.label.clone()),
            )
            .on_change(move |state, _event, _window, cx| write(state == CheckboxState::Checked, cx))
            .into_any_element()
    }
}
