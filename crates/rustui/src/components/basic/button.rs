use gpui::{AnyElement, FontWeight, IntoElement, ParentElement, SharedString, Styled, div, px};
use gpui_component::{
    Disableable as _,
    button::{Button as GpuiButton, ButtonVariant, ButtonVariants as _},
};
use rustui_core::Identity;

use crate::view::{ControlAction, EmptyView, RenderContext, View};

/// Native button appearances backed by GPUI Components.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonStyle {
    #[default]
    Automatic,
    Plain,
    Bordered,
    BorderedProminent,
    Borderless,
}

impl ButtonStyle {
    fn variant(self) -> ButtonVariant {
        match self {
            Self::Automatic | Self::Bordered => ButtonVariant::Secondary,
            Self::BorderedProminent => ButtonVariant::Primary,
            Self::Plain => ButtonVariant::Ghost,
            Self::Borderless => ButtonVariant::Text,
        }
    }
}

pub(crate) fn themed_button(
    context: &RenderContext<'_>,
    id: SharedString,
    label: SharedString,
    accessibility_label: SharedString,
    style: ButtonStyle,
    disabled: bool,
    tab_index: isize,
    focusable: bool,
) -> GpuiButton {
    let resolved_theme = context.button_theme().cloned();
    let control_font_size = resolved_theme
        .as_ref()
        .map(|theme| px(theme.font_size))
        .unwrap_or_else(|| context.control_font_size());
    let mut label_element = div().text_size(control_font_size).child(label);
    if let Some(theme) = &resolved_theme {
        label_element = label_element
            .font_family(theme.font_family.clone())
            .font_weight(FontWeight(theme.font_weight));
    }

    let mut button = GpuiButton::new(id)
        .accessibility_label(accessibility_label)
        .child(label_element)
        .with_variant(style.variant())
        .px(control_font_size)
        .disabled(disabled)
        .tab_index(tab_index)
        .tab_stop(focusable);
    if let Some(theme) = &resolved_theme {
        button = button
            .min_w(px(theme.width))
            .h(px(theme.height))
            .px(px(theme.horizontal_padding))
            .rounded(px(theme.corner_radius));
        if matches!(style, ButtonStyle::Automatic | ButtonStyle::Bordered) {
            button = button
                .bg(crate::theme::registry::hsla(theme.background))
                .text_color(crate::theme::registry::hsla(theme.foreground));
        }
    }
    button
}

/// A RustUI button that dispatches an action on its owning component.
pub struct ButtonView {
    label: SharedString,
    action: ControlAction,
    id: Option<Identity>,
    style: ButtonStyle,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    help: Option<SharedString>,
    tab_index: isize,
    focusable: bool,
}

impl ButtonView {
    pub fn new(label: impl Into<SharedString>, action: impl Into<ControlAction>) -> Self {
        Self {
            label: label.into(),
            action: action.into(),
            id: None,
            style: ButtonStyle::Automatic,
            disabled: false,
            accessibility_label: None,
            help: None,
            tab_index: 0,
            focusable: true,
        }
    }

    /// Sets an explicit GPUI element ID when labels are repeated.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Selects a native GPUI Components button appearance.
    pub fn button_style(mut self, style: ButtonStyle) -> Self {
        self.style = style;
        self
    }

    /// Disables pointer and keyboard activation while retaining native styling.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Replaces the name announced by assistive technologies.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    /// Adds explanatory hover help without changing the visible label.
    pub fn help(mut self, help: impl Into<SharedString>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Sets keyboard traversal order. The default is zero.
    pub fn tab_index(mut self, tab_index: isize) -> Self {
        self.tab_index = tab_index;
        self
    }

    /// Includes or excludes this button from keyboard focus traversal.
    pub fn focusable(mut self, focusable: bool) -> Self {
        self.focusable = focusable;
        self
    }
}

/// Creates a button whose action comes from `self.action(Self::handler)`.
#[allow(non_snake_case)]
pub fn Button(label: impl Into<SharedString>, action: impl Into<ControlAction>) -> ButtonView {
    ButtonView::new(label, action)
}

impl View for ButtonView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let id = self
            .id
            .as_ref()
            .map(ToString::to_string)
            .map(SharedString::from)
            .unwrap_or_else(|| self.label.clone());
        let label = self.label.clone();
        let accessibility_label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| label.clone());
        let mut button = themed_button(
            context,
            id,
            label,
            accessibility_label,
            self.style,
            self.disabled,
            self.tab_index,
            self.focusable,
        )
        .on_click(context.bind_control_action(self.action.clone()));
        if let Some(help) = &self.help {
            button = button.tooltip(help.clone());
        }
        div()
            .self_start()
            .items_start()
            .child(button)
            .into_any_element()
    }
}
