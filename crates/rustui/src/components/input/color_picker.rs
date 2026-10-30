//! Controlled color selection backed by a retained GPUI ColorPickerState.

use std::{cell::RefCell, rc::Rc};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, AppContext as _, Focusable as _, Hsla, InteractiveElement as _, IntoElement,
    ParentElement as _, SharedString, StatefulInteractiveElement as _, Styled as _, div, hsla, px,
};

type EventWriter<Value, Context> = Rc<dyn Fn(Value, &mut Context)>;

fn replace_event_writer<Value, Context>(
    slot: &RefCell<Option<EventWriter<Value, Context>>>,
    writer: EventWriter<Value, Context>,
) {
    *slot.borrow_mut() = Some(writer);
}

fn write_with_current_writer<Value, Context>(
    slot: &RefCell<Option<EventWriter<Value, Context>>>,
    value: Value,
    context: &mut Context,
) {
    let writer = slot.borrow().clone();
    if let Some(writer) = writer {
        writer(value, context);
    }
}

fn clear_picker_value(
    state: &mut ColorPickerState,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<ColorPickerState>,
) {
    state.clear_value(window, cx);
    reset_empty_sliders(state, window, cx);
}

fn reset_empty_sliders(
    state: &ColorPickerState,
    window: &mut gpui::Window,
    cx: &mut gpui::Context<ColorPickerState>,
) {
    let sliders = state.sliders().clone();
    for (slider, value) in [
        (sliders.hue(), 0.0),
        (sliders.saturation(), 0.0),
        (sliders.lightness(), 0.0),
        (sliders.alpha(), 1.0),
    ] {
        slider.update(cx, |slider, cx| slider.set_value(value, window, cx));
    }
}

fn set_picker_open(
    state: &gpui::Entity<ColorPickerState>,
    open: bool,
    window: &mut gpui::Window,
    app: &mut gpui::App,
) {
    state.update(app, |state, cx| {
        if state.is_open() && !open {
            match state.value() {
                Some(value) => state.set_value(value, window, cx),
                None => clear_picker_value(state, window, cx),
            }
        }
        state.set_open(open, cx);
    });
}

struct RetainedColorPicker {
    state: gpui::Entity<ColorPickerState>,
    writer: Rc<RefCell<Option<EventWriter<Option<Hsla>, gpui::App>>>>,
}
use gpui_base::{ColorPicker as BaseColorPicker, ColorSwatch};
use gpui_component::{
    Selectable as _, Sizable as _, Size,
    button::{Button as GpuiButton, ButtonVariants as _},
    color_picker::{ColorPickerEvent, ColorPickerState},
    input::Input,
    popover::Popover as GpuiPopover,
    slider::{Slider as GpuiSlider, SliderEvent, SliderState, SliderValue},
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

const HEX_INPUT_ACCESSIBILITY_MESSAGE_ID: &str = "color_picker.hex_value";
const HEX_INPUT_ACCESSIBILITY_FALLBACK: &str = "Hex color value";

/// Controlled optional color selection with a retained GPUI picker state.
pub struct ColorPickerView {
    label: SharedString,
    localized_label_key: Option<SharedString>,
    value: Binding<Option<Hsla>>,
    id: Option<Identity>,
    compact: bool,
    featured_colors: Option<Vec<Hsla>>,
    accessibility_label: Option<ColorPickerLabel>,
}

#[derive(Clone)]
enum ColorPickerLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

impl ColorPickerLabel {
    fn resolve(&self, context: &RenderContext<'_>) -> SharedString {
        match self {
            Self::Literal(label) => label.clone(),
            Self::Message { id, fallback } => {
                resolved_message_label(context.localized_message(id.as_ref()), id, fallback)
            }
        }
    }
}

fn resolved_message_label(
    localized: String,
    id: &SharedString,
    fallback: &SharedString,
) -> SharedString {
    if localized.trim().is_empty() || localized.trim() == id.trim() {
        fallback.clone()
    } else {
        SharedString::from(localized)
    }
}

fn assert_valid_color(color: Hsla) {
    assert!(
        [color.h, color.s, color.l, color.a]
            .into_iter()
            .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel)),
        "ColorPicker colors must have finite HSLA channels between 0 and 1"
    );
}

fn same_picker_color(left: Hsla, right: Hsla) -> bool {
    left.h.rem_euclid(1.0) == right.h.rem_euclid(1.0)
        && left.s == right.s
        && left.l == right.l
        && left.a == right.a
}

impl ColorPickerView {
    pub fn new(label: impl Into<SharedString>, value: Binding<Option<Hsla>>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "ColorPicker label must not be blank"
        );
        Self {
            label,
            localized_label_key: None,
            value,
            id: None,
            compact: false,
            featured_colors: None,
            accessibility_label: None,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "ColorPicker identity must not be blank"
        );
        self.id = Some(id);
        self
    }

    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }

    /// Resolve the visible label from the active catalog, keeping the constructor
    /// label as a fallback when the message is unavailable.
    pub fn localized_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "ColorPicker label localization key must not be blank"
        );
        self.localized_label_key = Some(id);
        self
    }

    fn visible_label(&self, localized_message: impl FnOnce(&str) -> String) -> SharedString {
        self.localized_label_key.as_ref().map_or_else(
            || self.label.clone(),
            |id| resolved_message_label(localized_message(id.as_ref()), id, &self.label),
        )
    }

    pub fn featured_colors(mut self, colors: impl Into<Vec<Hsla>>) -> Self {
        let colors = colors.into();
        colors.iter().copied().for_each(assert_valid_color);
        self.featured_colors = Some(colors);
        self
    }

    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "ColorPicker accessibility label must not be blank"
        );
        self.accessibility_label = Some(ColorPickerLabel::Literal(label));
        self
    }

    /// Set an accessible name resolved from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "ColorPicker accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(ColorPickerLabel::Literal(label))
            | Some(ColorPickerLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(ColorPickerLabel::Message { id, fallback });
        self
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            // The visible label can change with locale without changing the picker.
            .unwrap_or_else(|| allocate_implicit(Identity::from("color-picker")))
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
        let selected = state
            .read(context.app())
            .value()
            .is_some_and(|value| same_picker_color(value, color));
        let preview_state = state.clone();
        let select_state = state.clone();
        let keyboard_state = state.clone();
        let style = required_style(context, "color_picker", part, selector);
        let width = style.lengths["width"];
        let height = style.lengths["height"];

        styled_element(
            ColorSwatch::new(id, color)
                .on_action(move |_: &gpui_base::actions::Confirm, window, cx| {
                    cx.stop_propagation();
                    keyboard_state.update(cx, |state, cx| state.select_color(color, window, cx));
                })
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

        let direction = context.layout_direction();
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
                super::logical_input_row(gpui_component::h_flex(), direction)
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
        let keyboard_tab_state = state.clone();
        let palette_label = popup_label(context, "rustui-color-picker-palette", "Palette");
        let palette = Tab::new().flex_1().label(palette_label);
        let hsla = Tab::new().flex_1().label("HSLA");
        #[cfg(test)]
        let palette = palette.debug_selector(|| "color-mode-palette".to_owned());
        #[cfg(test)]
        let hsla = hsla.debug_selector(|| "color-mode-hsla".to_owned());
        let mut mode_tabs = vec![palette, hsla];
        if direction == rustui_i18n::LayoutDirection::RightToLeft {
            mode_tabs.reverse();
        }
        let displayed_tab = color_mode_index(active_tab, direction).unwrap_or(active_tab);
        let mut panel = gpui_component::v_flex()
            .w(px(288.0))
            .gap_3()
            .child(wheel)
            .child(
                div()
                    .id("color-picker-mode-keyboard")
                    .focusable()
                    .tab_stop(true)
                    .on_key_down(move |event, _, cx| {
                        if event.prefer_character_input
                            || event.keystroke.is_ime_in_progress()
                            || event.keystroke.modifiers.modified()
                        {
                            return;
                        }
                        let index = match event.keystroke.key.as_ref() {
                            "left" => color_mode_index(0, direction).unwrap(),
                            "right" => color_mode_index(1, direction).unwrap(),
                            "home" => 0,
                            "end" => 1,
                            _ => return,
                        };
                        keyboard_tab_state.update(cx, |state, cx| state.set_active_tab(index, cx));
                        cx.stop_propagation();
                    })
                    .child(
                        TabBar::new("color-picker-mode")
                            .segmented()
                            .selected_index(displayed_tab)
                            .on_click(move |index: &usize, _, cx| {
                                if let Some(logical) = color_mode_index(*index, direction) {
                                    active_tab_state
                                        .update(cx, |state, cx| state.set_active_tab(logical, cx));
                                }
                            })
                            .children(mode_tabs),
                    ),
            );

        if active_tab == 0 {
            panel = panel
                .child(
                    super::logical_input_row(gpui_component::h_flex(), direction)
                        .flex_wrap()
                        .gap_1()
                        .children(swatches),
                )
                .children(crayons);
        } else {
            let hex_accessibility_id = SharedString::from(HEX_INPUT_ACCESSIBILITY_MESSAGE_ID);
            let hex_accessibility_fallback = SharedString::from(HEX_INPUT_ACCESSIBILITY_FALLBACK);
            let hex_accessibility_label = resolved_message_label(
                context.localized_message(HEX_INPUT_ACCESSIBILITY_MESSAGE_ID),
                &hex_accessibility_id,
                &hex_accessibility_fallback,
            );
            panel = panel
                .child(slider_row(
                    popup_label(context, "rustui-color-picker-hue", "Hue"),
                    keyboard_color_slider("hue", sliders.hue()),
                    direction,
                ))
                .child(slider_row(
                    popup_label(context, "rustui-color-picker-saturation", "Saturation"),
                    keyboard_color_slider("saturation", sliders.saturation()),
                    direction,
                ))
                .child(slider_row(
                    popup_label(context, "rustui-color-picker-lightness", "Lightness"),
                    keyboard_color_slider("lightness", sliders.lightness()),
                    direction,
                ))
                .child(slider_row(
                    popup_label(context, "rustui-color-picker-opacity", "Opacity"),
                    keyboard_color_slider("alpha", sliders.alpha()),
                    direction,
                ))
                .child(
                    div()
                        .id("color-picker-hex-input")
                        .on_action(|_: &gpui_base::actions::Confirm, _, cx| cx.stop_propagation())
                        .child(
                            Input::new(&hex_input)
                                .small()
                                .px_2p5()
                                .aria_label(hex_accessibility_label),
                        ),
                );
        }

        panel.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn binding() -> Binding<Option<Hsla>> {
        Binding::new(
            None,
            |value: &Option<Hsla>| value.clone(),
            |value, next| *value = next,
        )
    }

    #[test]
    fn popup_labels_are_present_for_runtime_locales_and_rows_follow_direction() {
        use gpui::Styled;
        use rustui_i18n::{Catalog, LayoutDirection, Locale};
        for (tag, source, palette) in [
            (
                "en",
                include_str!("../../../../../locales/en/rustui.ftl"),
                "Palette",
            ),
            (
                "fr",
                include_str!("../../../../../locales/fr/rustui.ftl"),
                "Palette",
            ),
            (
                "ar",
                include_str!("../../../../../locales/ar/rustui.ftl"),
                "لوحة الألوان",
            ),
        ] {
            let catalog = Catalog::parse(Locale::parse(tag).unwrap(), source).unwrap();
            assert_eq!(catalog.get("rustui-color-picker-palette"), Some(palette));
            for key in [
                "rustui-color-picker-hue",
                "rustui-color-picker-saturation",
                "rustui-color-picker-lightness",
                "rustui-color-picker-opacity",
            ] {
                assert!(
                    catalog
                        .get(key)
                        .is_some_and(|value| !value.trim().is_empty()),
                    "{tag}: {key}"
                );
            }
        }
        for (direction, expected) in [
            (LayoutDirection::LeftToRight, gpui::FlexDirection::Row),
            (
                LayoutDirection::RightToLeft,
                gpui::FlexDirection::RowReverse,
            ),
        ] {
            let mut row = super::slider_row("Hue".into(), gpui::div(), direction);
            assert_eq!(row.style().flex_direction, Some(expected));
        }
    }

    #[test]
    fn featured_colors_require_finite_unit_hsla_channels() {
        let picker =
            ColorPicker("Accent color", binding()).featured_colors(vec![hsla(0.0, 0.5, 0.5, 1.0)]);
        assert_eq!(picker.featured_colors, Some(vec![hsla(0.0, 0.5, 0.5, 1.0)]));

        for color in [
            Hsla {
                h: f32::NAN,
                s: 0.5,
                l: 0.5,
                a: 1.0,
            },
            Hsla {
                h: 0.0,
                s: f32::INFINITY,
                l: 0.5,
                a: 1.0,
            },
            Hsla {
                h: 0.0,
                s: 0.5,
                l: -0.1,
                a: 1.0,
            },
            Hsla {
                h: 0.0,
                s: 0.5,
                l: 0.5,
                a: 1.1,
            },
        ] {
            assert!(
                std::panic::catch_unwind(|| {
                    ColorPicker("Accent color", binding()).featured_colors(vec![color]);
                })
                .is_err()
            );
        }
    }

    #[test]
    fn hue_endpoints_match_the_same_selected_color() {
        let zero_hue = hsla(0.0, 1.0, 0.5, 1.0);
        let full_turn = hsla(1.0, 1.0, 0.5, 1.0);

        assert_eq!(zero_hue.to_rgb(), full_turn.to_rgb());
        assert!(same_picker_color(zero_hue, full_turn));
    }

    #[test]
    #[should_panic(expected = "ColorPicker identity must not be blank")]
    fn rejects_blank_explicit_identity() {
        ColorPicker("Accent color", binding()).id(" \t ");
    }

    #[test]
    fn preserves_nonblank_explicit_identity() {
        let picker = ColorPicker("Accent color", binding()).id(" picker ");

        assert_eq!(picker.id.as_ref().unwrap().as_str(), " picker ");
    }

    #[test]
    #[should_panic(expected = "ColorPicker accessibility label must not be blank")]
    fn rejects_blank_explicit_accessible_name() {
        ColorPicker("Accent color", binding()).accessibility_label(" \t ");
    }

    #[test]
    fn preserves_nonblank_explicit_accessible_name() {
        let picker = ColorPicker("Accent color", binding())
            .accessibility_label("  Choose an accent color  ");

        assert!(matches!(
            picker.accessibility_label,
            Some(ColorPickerLabel::Literal(label)) if label.as_ref() == "  Choose an accent color  "
        ));
    }

    #[test]
    #[should_panic(expected = "ColorPicker accessibility localization key must not be blank")]
    fn rejects_blank_localized_accessible_name_key() {
        ColorPicker("Accent color", binding()).localized_accessibility_label(" \t ");
    }

    #[test]
    fn localized_accessible_name_keeps_literal_override_as_fallback() {
        let picker = ColorPicker("Accent color", binding())
            .accessibility_label("Choose an accent")
            .localized_accessibility_label("color_picker.accent");

        assert!(matches!(
            picker.accessibility_label,
            Some(ColorPickerLabel::Message { id, fallback })
                if id.as_ref() == "color_picker.accent" && fallback.as_ref() == "Choose an accent"
        ));
    }

    #[test]
    fn localized_accessible_name_defaults_to_visible_label_fallback() {
        let picker = ColorPicker("Accent color", binding())
            .localized_accessibility_label("color_picker.accent");

        assert!(matches!(
            picker.accessibility_label,
            Some(ColorPickerLabel::Message { id, fallback })
                if id.as_ref() == "color_picker.accent" && fallback.as_ref() == "Accent color"
        ));
    }

    #[test]
    fn localized_accessible_name_uses_catalog_value_or_fallback() {
        let id = SharedString::from("color_picker.accent");
        let fallback = SharedString::from("Choose an accent");

        assert_eq!(
            resolved_message_label("Choisir une couleur accentuée".into(), &id, &fallback).as_ref(),
            "Choisir une couleur accentuée"
        );
        assert_eq!(
            resolved_message_label(id.to_string(), &id, &fallback),
            fallback
        );
        assert_eq!(
            resolved_message_label(" \t".into(), &id, &fallback),
            fallback
        );
    }

    #[test]
    fn localized_visible_label_uses_catalog_value_or_constructor_fallback() {
        let picker = ColorPicker("Accent color", binding()).localized_label("color_picker.accent");
        assert_eq!(
            picker
                .visible_label(|_| "Couleur d’accent".to_owned())
                .as_ref(),
            "Couleur d’accent"
        );
        assert_eq!(
            picker
                .visible_label(|_| "color_picker.accent".to_owned())
                .as_ref(),
            "Accent color"
        );
    }

    #[test]
    #[should_panic(expected = "ColorPicker label localization key must not be blank")]
    fn rejects_blank_localized_visible_label_key() {
        ColorPicker("Accent color", binding()).localized_label(" \t ");
    }

    #[test]
    fn hex_input_accessible_name_uses_catalog_translation_or_english_fallback() {
        let id = SharedString::from(HEX_INPUT_ACCESSIBILITY_MESSAGE_ID);
        let fallback = SharedString::from(HEX_INPUT_ACCESSIBILITY_FALLBACK);

        assert_eq!(
            resolved_message_label("Valeur de couleur hexadécimale".into(), &id, &fallback)
                .as_ref(),
            "Valeur de couleur hexadécimale"
        );
        assert_eq!(
            resolved_message_label(id.to_string(), &id, &fallback),
            fallback
        );
    }

    #[test]
    #[should_panic(expected = "ColorPicker label must not be blank")]
    fn rejects_blank_default_accessible_name() {
        ColorPicker(" \t ", binding());
    }

    #[test]
    fn retained_events_use_the_most_recent_owner_binding() {
        let writer = RefCell::new(None);
        let mut writes = Vec::new();
        replace_event_writer(
            &writer,
            Rc::new(|_value: Option<Hsla>, writes: &mut Vec<&'static str>| {
                writes.push("old");
            }),
        );
        replace_event_writer(
            &writer,
            Rc::new(|_value: Option<Hsla>, writes: &mut Vec<&'static str>| {
                writes.push("current");
            }),
        );

        write_with_current_writer(&writer, None, &mut writes);

        assert_eq!(writes, ["current"]);
    }
}

fn popup_label(
    context: &RenderContext<'_>,
    id: &'static str,
    fallback: &'static str,
) -> SharedString {
    resolved_message_label(context.localized_message(id), &id.into(), &fallback.into())
}

// GPUI Components supplies pointer behavior; use the same keyboard policy as
// RustUI's public Slider and emit the event that updates the retained color.
fn keyboard_color_slider(id: &'static str, state: &gpui::Entity<SliderState>) -> impl IntoElement {
    let keyboard_state = state.clone();
    div()
        .id(format!("color-picker-hsla-{id}"))
        .flex_1()
        .focusable()
        .tab_stop(true)
        .on_key_down(move |event, window, cx| {
            if event.prefer_character_input
                || event.keystroke.is_ime_in_progress()
                || event.keystroke.modifiers.modified()
            {
                return;
            }
            let slider = keyboard_state.read(cx);
            let current = slider.value().end();
            let Some(value) = super::slider::slider_keyboard_value(
                gpui::Axis::Horizontal,
                event.keystroke.key.as_ref(),
                current,
                slider.min_value(),
                slider.max_value(),
                slider.step_value(),
            ) else {
                return;
            };
            if value != current {
                keyboard_state.update(cx, |slider, cx| {
                    slider.set_value(value, window, cx);
                    cx.emit(SliderEvent::Change(SliderValue::Single(value)));
                });
            }
            cx.stop_propagation();
        })
        .child(GpuiSlider::new(state).w_full())
}

// The two-mode display mapping is its own inverse.
fn color_mode_index(index: usize, direction: rustui_i18n::LayoutDirection) -> Option<usize> {
    if index >= 2 {
        return None;
    }
    Some(match direction {
        rustui_i18n::LayoutDirection::LeftToRight => index,
        rustui_i18n::LayoutDirection::RightToLeft => 1 - index,
    })
}

fn slider_row(
    label: SharedString,
    slider: impl IntoElement,
    direction: rustui_i18n::LayoutDirection,
) -> gpui::Div {
    super::logical_input_row(gpui_component::h_flex(), direction)
        .items_center()
        .gap_2()
        .child(div().w(px(92.0)).flex_shrink_0().child(label))
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
        let identity = self.identity(context);
        let initial_value = self.value.value();
        let (retained, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let state = cx.new(|cx| {
                let state = ColorPickerState::new(window, cx);
                let mut state = match initial_value {
                    Some(value) => state.default_value(value),
                    None => state,
                };
                state.sync_pending_value(window, cx);
                if initial_value.is_none() {
                    reset_empty_sliders(&state, window, cx);
                }
                state
            });
            RetainedColorPicker {
                state,
                writer: Rc::new(RefCell::new(None)),
            }
        });
        let (state, writer) = {
            let retained = retained.read(context.app());
            (retained.state.clone(), retained.writer.clone())
        };
        replace_event_writer(&writer, context.binding_writer(self.value.clone()));

        let current = self.value.value();
        let retained = state.read(context.app()).value();
        if retained != current {
            context.update_entity(&state, move |state, window, cx| match current {
                Some(value) => state.set_value(value, window, cx),
                None => clear_picker_value(state, window, cx),
            });
        }

        if created {
            let current_writer = writer.clone();
            context.subscribe_retained(
                &identity,
                &state,
                move |_state, event: &ColorPickerEvent, cx| {
                    let ColorPickerEvent::Change(value) = event;
                    write_with_current_writer(&current_writer, *value, cx);
                },
            );
        }

        let selector = Self::theme_selector(context);
        let open = state.read(context.app()).is_open();
        let visible_label = self.visible_label(|id| context.localized_message(id));
        let label = self
            .accessibility_label
            .as_ref()
            .map_or_else(|| visible_label.clone(), |label| label.resolve(context));
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
                super::logical_input_row(gpui_component::h_flex(), context.layout_direction())
                    .items_center()
                    .gap_2()
                    .child(trigger_swatch)
                    .child(if self.compact {
                        visible_label.clone()
                    } else {
                        SharedString::from(
                            selected
                                .map_or_else(|| visible_label.to_string(), |value| value.to_hex()),
                        )
                    })
                    .child("⌄"),
            );

        let popup = self.render_popup(&state, context, &selector);
        let popup_change_state = popup_state.clone();
        let popup_element = GpuiPopover::new(format!("{identity}:popover"))
            .open(open)
            .on_open_change(move |open: &bool, window, cx| {
                set_picker_open(&popup_change_state, *open, window, cx);
            })
            .trigger(trigger)
            .child(popup);

        BaseColorPicker::new(format!("{identity}:root"))
            .open(open)
            .track_focus(&state.focus_handle(context.app()))
            .accessibility_label(label)
            .on_open_change(move |open, window, cx| {
                set_picker_open(&root_state, open, window, cx);
            })
            .child(popup_element)
            .into_any_element()
    }
}

#[cfg(test)]
mod implicit_identity_tests {
    use super::*;

    fn binding() -> Binding<Option<Hsla>> {
        Binding::new(
            None,
            |value: &Option<Hsla>| *value,
            |value, next| *value = next,
        )
    }

    #[test]
    fn same_label_siblings_get_distinct_stable_implicit_identities() {
        let first = ColorPicker("Accent color", binding());
        let second = ColorPicker("Accent color", binding());
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first_sibling = first.identity_with(allocate_at_slot(0));
        let second_sibling = second.identity_with(allocate_at_slot(1));
        let first_after_rerender = first.identity_with(allocate_at_slot(0));
        let first_after_locale_change =
            ColorPicker("Couleur d’accent", binding()).identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_after_rerender);
        assert_eq!(first_sibling, first_after_locale_change);
    }
}

#[cfg(test)]
mod rendered_locale_tests {
    use super::*;
    use crate::{
        localization::{Catalog, Catalogs, Locale, activate_localization, set_locale},
        runtime::ComponentHost,
        theme::{AppearanceMode, activate_theme, parse_theme},
    };
    use gpui_component::Root;

    struct Owner {
        color: Option<Hsla>,
    }
    impl View for Owner {
        fn body(&self) -> impl View {
            ColorPicker(
                "Color",
                Binding::new(self.color, |o: &Owner| o.color, |o, value| o.color = value),
            )
            .id("locale-color-picker")
        }
    }

    #[gpui::test]
    fn empty_picker_sliders_match_opaque_black_without_writing_owner(
        cx: &mut gpui::TestAppContext,
    ) {
        let theme = parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            activate_theme(&theme, Some(AppearanceMode::Dark), app).unwrap();
        });
        let (owner, window) =
            cx.add_window_view(|_, cx| ComponentHost::new(Owner { color: None }, cx));
        window.update(|window, app| {
            let _ = window.draw(app);
            let retained = owner
                .read(app)
                .retained_entity_for_test::<RetainedColorPicker>(
                    vec![],
                    Identity::from("locale-color-picker"),
                )
                .unwrap();
            let state = retained.read(app).state.read(app);
            assert_eq!(state.value(), None);
            let sliders = state.sliders();
            assert_eq!(sliders.hue().read(app).value().end(), 0.0);
            assert_eq!(sliders.saturation().read(app).value().end(), 0.0);
            assert_eq!(sliders.lightness().read(app).value().end(), 0.0);
            assert_eq!(sliders.alpha().read(app).value().end(), 1.0);
            assert_eq!(owner.read(app).component().color, None);
        });
    }

    #[gpui::test]
    fn open_picker_retains_color_and_mode_across_locale_switch(cx: &mut gpui::TestAppContext) {
        let en = Locale::parse("en").unwrap();
        let ar = Locale::parse("ar").unwrap();
        let mut catalogs = Catalogs::new(en.clone());
        catalogs
            .insert(
                Catalog::parse(
                    en.clone(),
                    include_str!("../../../../../locales/en/rustui.ftl"),
                )
                .unwrap(),
            )
            .unwrap();
        catalogs
            .insert(
                Catalog::parse(
                    ar.clone(),
                    include_str!("../../../../../locales/ar/rustui.ftl"),
                )
                .unwrap(),
            )
            .unwrap();
        let theme = parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            activate_theme(&theme, Some(AppearanceMode::Dark), app).unwrap();
            activate_localization(catalogs, en, app);
        });
        let owner_slot = Rc::new(RefCell::new(None));
        let saved = owner_slot.clone();
        let color = hsla(0.25, 0.5, 0.4, 1.0);
        let (_, window) = cx.add_window_view(|window, cx| {
            let owner = cx.new(|cx| ComponentHost::new(Owner { color: Some(color) }, cx));
            *saved.borrow_mut() = Some(owner.clone());
            Root::new(owner, window, cx)
        });
        let owner = owner_slot.borrow().clone().unwrap();
        let state = window.update(|window, app| {
            let _ = window.draw(app);
            let retained = owner
                .read(app)
                .retained_entity_for_test::<RetainedColorPicker>(
                    vec![],
                    Identity::from("locale-color-picker"),
                )
                .unwrap();
            let state = retained.read(app).state.clone();
            window.focus_next(app);
            let _ = window.draw(app);
            state
        });
        let key = gpui::Keystroke::parse("enter").unwrap();
        window.simulate_event(gpui::KeyDownEvent {
            keystroke: key.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        window.simulate_event(gpui::KeyUpEvent { keystroke: key });
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(
                state.read(app).is_open(),
                "keyboard trigger did not open the picker"
            );
            state.update(app, |state, cx| state.set_active_tab(1, cx));
            let _ = window.draw(app);
            set_locale(ar, app);
            let _ = window.draw(app);
            assert!(state.read(app).is_open());
            assert_eq!(state.read(app).active_tab(), 1);
            let snapshot = window.debug_a11y_tree_json().expect("rendered picker tree");
            assert!(snapshot.contains("لوحة الألوان"));
            let tree: serde_json::Value = serde_json::from_str(&snapshot).unwrap();
            let nodes = tree["nodes"].as_object().unwrap();
            let tabs = nodes
                .values()
                .find(|node| node["aria"]["role"] == "TabList")
                .unwrap();
            let order: Vec<_> = tabs["children"]
                .as_array()
                .unwrap()
                .iter()
                .map(|id| {
                    nodes[id.as_str().unwrap()]["aria"]["label"]
                        .as_str()
                        .unwrap()
                })
                .collect();
            assert_eq!(order, ["HSLA", "لوحة الألوان"]);
            let selected = tabs["children"][0].as_str().unwrap();
            assert_eq!(nodes[selected]["aria"]["selected"], true);
            assert_eq!(state.read(app).value(), Some(color));
            assert_eq!(owner.read(app).component().color, Some(color));
        });
        let palette = window.debug_bounds("color-mode-palette").unwrap();
        let hsla = window.debug_bounds("color-mode-hsla").unwrap();
        assert!(hsla.center().x < palette.center().x);
        window.simulate_click(palette.center(), gpui::Modifiers::default());
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(state.read(app).active_tab(), 0);
        });
        let hsla = window.debug_bounds("color-mode-hsla").unwrap();
        window.simulate_click(hsla.center(), gpui::Modifiers::default());
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(state.read(app).active_tab(), 1);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(state.read(app).active_tab(), 0);
        });
        window.simulate_keystrokes("left");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(state.read(app).active_tab(), 1);
            state.update(app, |state, cx| state.set_active_tab(0, cx));
            let _ = window.draw(app);
            window.focus_next(app);
            let _ = window.draw(app);
            let tree: serde_json::Value =
                serde_json::from_str(&window.debug_a11y_tree_json().expect("rendered picker tree"))
                    .unwrap();
            assert!(state.read(app).is_open(), "closed during focus traversal");
            let focus = tree["gpui_focus"].as_str().unwrap();
            assert_eq!(
                tree["nodes"][focus]["element_id"],
                "NamedInteger(\"color-picker-featured\", 0)"
            );
        });
        let key = gpui::Keystroke::parse("space").unwrap();
        window.simulate_event(gpui::KeyDownEvent {
            keystroke: key.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        window.simulate_event(gpui::KeyUpEvent { keystroke: key });
        window.update(|window, app| {
            let _ = window.draw(app);
            let selected = default_swatch_colors()[0];
            assert_eq!(owner.read(app).component().color, Some(selected));
            assert_eq!(state.read(app).value(), Some(selected));
            assert!(!state.read(app).is_open());
            state.update(app, |state, cx| state.set_open(true, cx));
            let _ = window.draw(app);
            for _ in 0..3 {
                window.focus_next(app);
                let _ = window.draw(app);
            }
            let tree: serde_json::Value =
                serde_json::from_str(&window.debug_a11y_tree_json().expect("rendered picker tree"))
                    .unwrap();
            let focus = tree["gpui_focus"].as_str().unwrap();
            assert_eq!(
                tree["nodes"][focus]["element_id"],
                "NamedInteger(\"color-picker-featured\", 1)"
            );
        });
        let key = gpui::Keystroke::parse("enter").unwrap();
        window.simulate_event(gpui::KeyDownEvent {
            keystroke: key.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        window.simulate_event(gpui::KeyUpEvent { keystroke: key });
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(
                owner.read(app).component().color,
                Some(default_swatch_colors()[1])
            );
            assert!(!state.read(app).is_open());
        });
    }
    #[gpui::test]
    fn hsla_keyboard_controls_and_hex_commit_update_owner(cx: &mut gpui::TestAppContext) {
        let theme = parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            activate_theme(&theme, Some(AppearanceMode::Dark), app).unwrap();
        });
        let owner_slot = Rc::new(RefCell::new(None));
        let saved = owner_slot.clone();
        let color = hsla(0.25, 0.5, 0.4, 1.0);
        let (_, window) = cx.add_window_view(|window, cx| {
            let owner = cx.new(|cx| ComponentHost::new(Owner { color: Some(color) }, cx));
            *saved.borrow_mut() = Some(owner.clone());
            Root::new(owner, window, cx)
        });
        let owner = owner_slot.borrow().clone().unwrap();
        let state = window.update(|window, app| {
            let _ = window.draw(app);
            let retained = owner
                .read(app)
                .retained_entity_for_test::<RetainedColorPicker>(
                    vec![],
                    Identity::from("locale-color-picker"),
                )
                .unwrap();
            let state = retained.read(app).state.clone();
            window.focus_next(app);
            let _ = window.draw(app);
            state
        });
        let key = gpui::Keystroke::parse("enter").unwrap();
        window.simulate_event(gpui::KeyDownEvent {
            keystroke: key.clone(),
            is_held: false,
            prefer_character_input: false,
        });
        window.simulate_event(gpui::KeyUpEvent { keystroke: key });
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(state.read(app).is_open());
            window.focus_next(app);
            let _ = window.draw(app);
        });
        let palette = window.debug_bounds("color-mode-palette").unwrap();
        let hsla_bounds = window.debug_bounds("color-mode-hsla").unwrap();
        assert!(palette.center().x < hsla_bounds.center().x);
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(state.read(app).active_tab(), 1);
        });
        window.simulate_keystrokes("left");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(state.read(app).active_tab(), 0);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(state.read(app).active_tab(), 1);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!((owner.read(app).component().color.unwrap().h - 0.26).abs() < 0.0001);
        });
        window.simulate_keystrokes("end");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().color.unwrap().h, 1.0);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().color.unwrap().h, 1.0);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("home");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().color.unwrap().s, 0.0);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("end");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().color.unwrap().l, 1.0);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("home");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().color.unwrap().a, 0.0);
            window.focus_next(app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("cmd-a");
        window.simulate_input("#FF");
        window.simulate_keystrokes("enter");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(
                state.read(app).is_open(),
                "incomplete hex must not close the picker"
            );
            assert_eq!(owner.read(app).component().color.unwrap().a, 0.0);
        });
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().color = None;
                cx.notify();
            });
            let _ = window.draw(app);
            assert_eq!(state.read(app).value(), None);
            assert!(state.read(app).hex_input().read(app).value().is_empty());
            let sliders = state.read(app).sliders();
            assert_eq!(sliders.hue().read(app).value().end(), 0.0);
            assert_eq!(sliders.saturation().read(app).value().end(), 0.0);
            assert_eq!(sliders.lightness().read(app).value().end(), 0.0);
            assert_eq!(sliders.alpha().read(app).value().end(), 1.0);
        });
        let external = hsla(0.5, 1.0, 0.5, 0.0);
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().color = Some(external);
                cx.notify();
            });
            let _ = window.draw(app);
            assert_eq!(state.read(app).value(), Some(external));
            assert_ne!(
                state.read(app).hex_input().read(app).value().as_ref(),
                "#FF"
            );
        });
        window.simulate_keystrokes("cmd-a");
        window.simulate_input("#FF");
        window.simulate_keystrokes("escape");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert!(!state.read(app).is_open());
            assert_ne!(
                state.read(app).hex_input().read(app).value().as_ref(),
                "#FF"
            );
            assert_eq!(owner.read(app).component().color, Some(external));
            assert_eq!(state.read(app).value(), Some(external));
            state.update(app, |state, cx| state.set_open(true, cx));
            let _ = window.draw(app);
            let focus = state.read(app).hex_input().read(app).focus_handle(app);
            window.focus(&focus, app);
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("cmd-a");
        window.simulate_input("#00FF00");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(
                state.read(app).displayed_color().unwrap().to_rgb(),
                hsla(1.0 / 3.0, 1.0, 0.5, 1.0).to_rgb()
            );
            assert_eq!(owner.read(app).component().color, Some(external));
        });
        window.simulate_keystrokes("cmd-a");
        window.simulate_input("#FF0000");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(
                state.read(app).hex_input().read(app).value().as_ref(),
                "#FF0000"
            );
            assert_eq!(owner.read(app).component().color.unwrap().a, 0.0);
        });
        window.simulate_keystrokes("enter");
        window.update(|window, app| {
            let _ = window.draw(app);
            let color = owner.read(app).component().color.unwrap();
            assert_eq!(color.to_rgb(), hsla(0.0, 1.0, 0.5, 1.0).to_rgb());
            assert!(!state.read(app).is_open());
        });
    }
}
