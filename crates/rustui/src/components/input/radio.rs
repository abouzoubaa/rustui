//! Controlled radio input with RustUI theme artwork and GPUI Base behavior.

use gpui::{
    AnyElement, FontWeight, InteractiveElement as _, IntoElement, ParentElement as _, SharedString,
    Styled as _, div, prelude::FluentBuilder as _, px,
};
use gpui_base::Radio as BaseRadio;
use rustui_core::{Binding, Identity};

use super::checkbox::{appearance_color, rectangular_artwork_size};
use crate::components::theme::{required_style_with_artwork, selector};
use crate::theme::registry::active_theme;
use crate::view::{EmptyView, RenderContext, View};

pub struct RadioView {
    label: SharedString,
    selected: Binding<bool>,
    id: Option<Identity>,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    tab_index: isize,
    focusable: bool,
}

impl RadioView {
    pub fn new(label: impl Into<SharedString>, selected: Binding<bool>) -> Self {
        Self {
            label: label.into(),
            selected,
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
            .unwrap_or_else(|| Identity::from(format!("radio:{}", self.label)))
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
        let selected = self.selected.value();
        let identity = self.identity();
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
            .when(selected, |this| {
                this.child(div().size(px(6.0)).rounded(px(3.0)).bg(foreground))
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
        BaseRadio::new(identity.to_string())
            .flex()
            .gap_2()
            .items_center()
            .focus(move |style| style.border(px(focus_width)).border_color(focus_color))
            .when(self.disabled, |this| this.opacity(0.5))
            .child(indicator)
            .child(label)
            .checked(selected)
            .disabled(self.disabled)
            .tab_index(self.tab_index)
            .tab_stop(self.focusable)
            .accessibility_label(
                self.accessibility_label
                    .clone()
                    .unwrap_or_else(|| self.label.clone()),
            )
            .on_change(move |selected, _event, _window, cx| write(selected, cx))
            .into_any_element()
    }
}
