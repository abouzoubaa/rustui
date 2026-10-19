//! Controlled switch input with a RustUI-themed track and GPUI Base behavior.

use gpui::{
    AbsoluteLength, AnyElement, App, EdgesRefinement, Hsla, IntoElement, ParentElement as _,
    RenderOnce, SharedString, Styled as _, Window, div, prelude::FluentBuilder as _, px,
};
use gpui_base::Switch as BaseSwitch;
use gpui_component::theme::try_parse_color;
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector},
    theme::{AppearanceMode, registry::active_theme},
    view::{EmptyView, RenderContext, View},
};

pub struct ToggleView {
    label: SharedString,
    checked: Binding<bool>,
    id: Option<Identity>,
    disabled: bool,
    accessibility_label: Option<SharedString>,
    tab_index: isize,
    focusable: bool,
}

impl ToggleView {
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
            .unwrap_or_else(|| Identity::from(format!("toggle:{}", self.label)))
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
        let identity = self.identity();
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
        let foreground_color =
            try_parse_color(&colors.primary_text).expect("validated RustUI primary text color");
        // macos27 declares `disabled: muted_foreground`; that global role resolves
        // to the profile's secondary text color.
        let disabled_color =
            try_parse_color(&colors.secondary_text).expect("validated RustUI secondary text color");
        let focus_color =
            try_parse_color(&colors.focus_ring).expect("validated RustUI focus ring color");
        let focus_border_width = active_theme.profile.borders.medium as f32;
        // The toggle record defines no thumb inset; preserve its explicit 2 px
        // geometry instead of borrowing a semantically unrelated profile token.
        let thumb_inset = 2.0;
        let label_color = if self.disabled {
            disabled_color
        } else {
            foreground_color
        };
        let checked = self.checked.value();
        let base = BaseSwitch::new(identity.to_string())
            .checked(checked)
            .disabled(self.disabled)
            .tab_index(self.tab_index)
            .tab_stop(self.focusable)
            .accessibility_label(
                self.accessibility_label
                    .clone()
                    .unwrap_or_else(|| self.label.clone()),
            )
            .on_change(move |checked, _, _, cx| write(checked, cx));

        ThemedSwitch {
            id: identity.to_string(),
            base,
            label: self.label.clone(),
            checked,
            disabled: self.disabled,
            width,
            height,
            radius,
            checked_color,
            unchecked_color,
            foreground_color,
            label_color,
            focus_color,
            focus_border_width,
            thumb_inset,
            label_gap: active_theme.profile.spacing.space_2 as f32,
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
    label_color: Hsla,
    focus_color: Hsla,
    focus_border_width: f32,
    thumb_inset: f32,
    label_gap: f32,
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
            self.label_color
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

        self.base
            .track_focus(&focus)
            .flex()
            .items_center()
            .gap(px(self.label_gap))
            .child(track)
            .child(div().text_color(self.label_color).child(self.label))
    }
}
