//! Controlled color selection backed by a retained GPUI ColorPickerState.

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, Focusable as _, Hsla, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, Styled as _, div, hsla, px,
};
use gpui_base::{ColorPicker as BaseColorPicker, ColorSwatch};
use gpui_component::{
    Selectable as _, Sizable as _, Size,
    button::{Button as GpuiButton, ButtonVariants as _},
    color_picker::{ColorPickerEvent, ColorPickerState},
    input::Input,
    popover::Popover as GpuiPopover,
    slider::Slider as GpuiSlider,
    tab::{Tab, TabBar},
    theme::ColorName,
    theme::Colorize as _,
};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, required_style_with_artwork, selector, styled_element},
    theme::ButtonStyleSelector,
    view::{EmptyView, RenderContext, View},
};

/// Controlled optional color selection with a retained GPUI picker state.
pub struct ColorPickerView {
    label: SharedString,
    value: Binding<Option<Hsla>>,
    id: Option<Identity>,
    compact: bool,
    featured_colors: Option<Vec<Hsla>>,
    accessibility_label: Option<SharedString>,
}

impl ColorPickerView {
    pub fn new(label: impl Into<SharedString>, value: Binding<Option<Hsla>>) -> Self {
        Self {
            label: label.into(),
            value,
            id: None,
            compact: false,
            featured_colors: None,
            accessibility_label: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    pub fn featured_colors(mut self, colors: impl Into<Vec<Hsla>>) -> Self {
        self.featured_colors = Some(colors.into());
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        self.accessibility_label = Some(label.into());
        self
    }

    fn identity(&self) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("color-picker:{}", self.label)))
    }

    fn theme_selector(context: &RenderContext<'_>) -> ButtonStyleSelector {
        let mut selector = selector(context);
        selector.chrome = "utility".into();
        selector
    }

    fn render_swatch(
        &self,
        id: impl Into<gpui::ElementId>,
        color: Hsla,
        part: &str,
        state: &gpui::Entity<ColorPickerState>,
        context: &RenderContext<'_>,
        selector: &ButtonStyleSelector,
    ) -> ColorSwatch {
        let selected = state.read(context.app()).value() == Some(color);
        let preview_state = state.clone();
        let select_state = state.clone();
        let style = required_style(context, "color_picker", part, selector);
        let width = style.lengths["width"];
        let height = style.lengths["height"];

        styled_element(
            ColorSwatch::new(id, color)
                .selected(selected)
                .bg(color)
                .border_1()
                .border_color(color.darken(0.18))
                .hover(|this| this.border_color(color.darken(0.32)))
                .border_color(color.darken(0.45))
                .on_hover(move |color, entered, window, cx| {
                    preview_state.update(cx, |state, cx| {
                        if entered {
                            state.preview_color(color, window, cx);
                        } else {
                            state.clear_preview(window, cx);
                        }
                    });
                })
                .on_click(move |color, _, window, cx| {
                    select_state.update(cx, |state, cx| state.select_color(color, window, cx));
                }),
            &style,
        )
        .w(px(width))
        .h(px(height))
    }

    fn render_popup(
        &self,
        state: &gpui::Entity<ColorPickerState>,
        context: &mut RenderContext<'_>,
        selector: &ButtonStyleSelector,
    ) -> AnyElement {
        let (active_tab, preview, sliders, hex_input) = {
            let state = state.read(context.app());
            (
                state.active_tab(),
                state
                    .displayed_color()
                    .unwrap_or_else(|| hsla(0.0, 0.0, 0.0, 1.0)),
                state.sliders().clone(),
                state.hex_input().clone(),
            )
        };

        // The native picker keeps its popup private. This RustUI-owned preview
        // is the visible color_wheel part; the selected-color chip is nested
        // inside its material frame so the theme surface remains visible.
        let wheel_style =
            required_style_with_artwork(context, "color_picker", "color_wheel", selector);
        let wheel = styled_element(
            div()
                .flex()
                .items_center()
                .justify_center()
                .child(div().w(px(28.0)).h(px(16.0)).rounded(px(4.0)).bg(preview)),
            &wheel_style,
        );

        let featured = self
            .featured_colors
            .clone()
            .unwrap_or_else(default_swatch_colors);
        let swatches = featured.into_iter().enumerate().map(|(index, color)| {
            self.render_swatch(
                ("color-picker-featured", index),
                color,
                "swatches",
                state,
                context,
                selector,
            )
        });
        let crayons = crayon_color_groups()
            .into_iter()
            .enumerate()
            .map(|(family, colors)| {
                gpui_component::h_flex()
                    .id(("color-picker-crayon-row", family))
                    .gap_1()
                    .children(colors.into_iter().enumerate().map(|(index, color)| {
                        self.render_swatch(
                            ("color-picker-crayon", family * 11 + index),
                            color,
                            "crayons",
                            state,
                            context,
                            selector,
                        )
                    }))
            });

        let active_tab_state = state.clone();
        let mut panel = gpui_component::v_flex()
            .w(px(288.0))
            .gap_3()
            .child(wheel)
            .child(
                TabBar::new("color-picker-mode")
                    .segmented()
                    .selected_index(active_tab)
                    .on_click(move |index: &usize, _, cx| {
                        active_tab_state.update(cx, |state, cx| state.set_active_tab(*index, cx));
                    })
                    .child(Tab::new().flex_1().label("Palette"))
                    .child(Tab::new().flex_1().label("HSLA")),
            );

        if active_tab == 0 {
            panel = panel
                .child(
                    gpui_component::h_flex()
                        .flex_wrap()
                        .gap_1()
                        .children(swatches),
                )
                .children(crayons);
        } else {
            panel = panel
                .child(slider_row("H", GpuiSlider::new(sliders.hue()).flex_1()))
                .child(slider_row(
                    "S",
                    GpuiSlider::new(sliders.saturation()).flex_1(),
                ))
                .child(slider_row(
                    "L",
                    GpuiSlider::new(sliders.lightness()).flex_1(),
                ))
                .child(slider_row("A", GpuiSlider::new(sliders.alpha()).flex_1()))
                .child(Input::new(&hex_input).small().px_2p5());
        }

        panel.into_any_element()
    }
}

fn slider_row(label: &'static str, slider: impl IntoElement) -> impl IntoElement {
    gpui_component::h_flex()
        .items_center()
        .gap_2()
        .child(div().w(px(16.0)).child(label))
        .child(slider)
}

fn default_swatch_colors() -> Vec<Hsla> {
    [
        ColorName::Red,
        ColorName::Orange,
        ColorName::Yellow,
        ColorName::Green,
        ColorName::Cyan,
        ColorName::Blue,
        ColorName::Purple,
        ColorName::Pink,
        ColorName::Gray,
    ]
    .into_iter()
    .map(|name| name.scale(500))
    .collect()
}

fn crayon_color_groups() -> Vec<Vec<Hsla>> {
    let families = [
        ColorName::Neutral,
        ColorName::Red,
        ColorName::Orange,
        ColorName::Yellow,
        ColorName::Green,
        ColorName::Cyan,
        ColorName::Blue,
        ColorName::Purple,
        ColorName::Pink,
    ];
    let scales = [50, 100, 200, 300, 400, 500, 600, 700, 800, 900, 950];
    families
        .into_iter()
        .map(|family| {
            scales
                .into_iter()
                .rev()
                .map(|scale| family.scale(scale))
                .collect()
        })
        .collect()
}

#[allow(non_snake_case)]
pub fn ColorPicker(
    label: impl Into<SharedString>,
    value: Binding<Option<Hsla>>,
) -> ColorPickerView {
    ColorPickerView::new(label, value)
}

impl View for ColorPickerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity();
        let initial_value = self.value.value();
        let (state, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let state = ColorPickerState::new(window, cx);
            match initial_value {
                Some(value) => state.default_value(value),
                None => state,
            }
        });

        let current = self.value.value();
        let retained = state.read(context.app()).value();
        if retained != current {
            context.update_entity(&state, move |state, window, cx| match current {
                Some(value) => state.set_value(value, window, cx),
                None => state.clear_value(window, cx),
            });
        }

        if created {
            let write = context.binding_writer(self.value.clone());
            context.subscribe_retained(
                &identity,
                &state,
                move |_state, event: &ColorPickerEvent, cx| {
                    let ColorPickerEvent::Change(value) = event;
                    write(*value, cx);
                },
            );
        }

        let selector = Self::theme_selector(context);
        let open = state.read(context.app()).is_open();
        let label = self
            .accessibility_label
            .clone()
            .unwrap_or_else(|| self.label.clone());
        let popup_state = state.clone();
        let root_state = state.clone();

        let selected = state.read(context.app()).value();
        let trigger_swatch = div()
            .flex_shrink_0()
            .w(px(16.0))
            .h(px(16.0))
            .rounded(px(4.0))
            .border_1()
            .border_color(
                selected.map_or(gpui::hsla(0.0, 0.0, 0.5, 0.5), |color| color.darken(0.2)),
            )
            .when_some(selected, |this, color| this.bg(color));
        let trigger = GpuiButton::new(format!("{identity}:trigger"))
            .secondary()
            .with_size(if self.compact {
                Size::Small
            } else {
                Size::Medium
            })
            .when(!self.compact, |this| this.w_full())
            .open(open)
            .accessibility_label(label.clone())
            .child(
                gpui_component::h_flex()
                    .items_center()
                    .gap_2()
                    .child(trigger_swatch)
                    .child(if self.compact {
                        self.label.clone()
                    } else {
                        SharedString::from(
                            selected.map_or_else(|| self.label.to_string(), |value| value.to_hex()),
                        )
                    })
                    .child("⌄"),
            );

        let popup = self.render_popup(&state, context, &selector);
        let popup_change_state = popup_state.clone();
        let popup_element = GpuiPopover::new(format!("{identity}:popover"))
            .open(open)
            .on_open_change(move |open: &bool, _, cx| {
                popup_change_state.update(cx, |state, cx| state.set_open(*open, cx));
            })
            .trigger(trigger)
            .child(popup);

        BaseColorPicker::new(format!("{identity}:root"))
            .open(open)
            .track_focus(&state.focus_handle(context.app()))
            .accessibility_label(label)
            .on_open_change(move |open, _, cx| {
                root_state.update(cx, |state, cx| state.set_open(open, cx));
            })
            .child(popup_element)
            .into_any_element()
    }
}
