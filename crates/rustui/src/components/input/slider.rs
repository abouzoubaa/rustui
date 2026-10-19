//! Controlled single-value slider backed by a retained GPUI SliderState.

use std::ops::RangeInclusive;

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AccessibleAction, AnyElement, App, Axis, InteractiveElement, IntoElement, MouseButton,
    Orientation, ParentElement, RenderOnce, Role, StatefulInteractiveElement, Styled, Window, div,
    px, relative,
};
use gpui_base::{SliderIndicator, SliderThumb, SliderTrack};
use gpui_component::{
    AxisExt as _,
    slider::{SliderEvent, SliderState, SliderValue},
    theme::try_parse_color,
};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector},
    theme::{AppearanceMode, registry::active_theme},
    view::{EmptyView, RenderContext, View},
};

pub struct SliderView {
    value: Binding<f32>,
    range: RangeInclusive<f32>,
    step: f32,
    id: Option<Identity>,
    disabled: bool,
    vertical: bool,
    reverse: bool,
}

impl SliderView {
    pub fn new(value: Binding<f32>, range: RangeInclusive<f32>) -> Self {
        assert!(
            range.start().is_finite() && range.end().is_finite() && range.start() < range.end(),
            "Slider range must contain two finite ascending values"
        );
        Self {
            value,
            range,
            step: 1.0,
            id: None,
            disabled: false,
            vertical: false,
            reverse: false,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        assert!(
            step.is_finite() && step > 0.0,
            "Slider step must be positive"
        );
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn vertical(mut self, vertical: bool) -> Self {
        self.vertical = vertical;
        self
    }

    pub fn reverse(mut self, reverse: bool) -> Self {
        self.reverse = reverse;
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "slider:{}:{}:{}",
                self.range.start(),
                self.range.end(),
                self.step
            ))
        })
    }

    fn clamped_value(&self) -> f32 {
        self.value
            .value()
            .clamp(*self.range.start(), *self.range.end())
    }
}

#[allow(non_snake_case)]
pub fn Slider(value: Binding<f32>, range: RangeInclusive<f32>) -> SliderView {
    SliderView::new(value, range)
}

impl View for SliderView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity();
        let min = *self.range.start();
        let max = *self.range.end();
        let step = self.step;
        let initial_value = self.clamped_value();
        let (state, created) = context.retained_entity(identity.clone(), move |_window, _cx| {
            SliderState::new()
                .min(min)
                .max(max)
                .step(step)
                .default_value(initial_value)
        });

        let current = self.clamped_value();
        let retained = match state.read(context.app()).value() {
            SliderValue::Single(value) => value,
            SliderValue::Range(_, _) => unreachable!("RustUI Slider retains single-value state"),
        };
        if retained != current {
            context.update_entity(&state, move |state, window, cx| {
                state.set_value(current, window, cx);
            });
        }

        if created {
            let write = context.binding_writer(self.value.clone());
            context.subscribe_retained(
                &identity,
                &state,
                move |_state, event: &SliderEvent, cx| {
                    if let SliderEvent::Change(SliderValue::Single(value)) = event {
                        write(*value, cx);
                    }
                },
            );
        }

        let mut selector = selector(context);
        selector.chrome = "utility".into();
        let style = required_style(context, "slider", "sliders", &selector);
        let thumb_width = *style.lengths.get("width").expect("slider thumb width");
        let thumb_height = *style.lengths.get("height").expect("slider thumb height");
        let active_theme =
            active_theme(context.app()).expect("a RustUI component theme requires an active theme");
        ThemedSlider {
            state,
            axis: if self.vertical {
                Axis::Vertical
            } else {
                Axis::Horizontal
            },
            disabled: self.disabled,
            reverse: self.reverse,
            thumb_width,
            thumb_height,
            track_color: slider_role_color(
                &style,
                "track",
                &active_theme.profile,
                active_theme.mode,
            ),
            thumb_color: slider_role_color(
                &style,
                "thumb",
                &active_theme.profile,
                active_theme.mode,
            ),
            disabled_color: slider_role_color(
                &style,
                "disabled",
                &active_theme.profile,
                active_theme.mode,
            ),
        }
        .into_any_element()
    }
}

/// Uses GPUI Base slider primitives for pointer and touch behavior. RustUI owns
/// the semantic root so AX SetValue can update both retained state and binding.
#[derive(IntoElement)]
struct ThemedSlider {
    state: gpui::Entity<SliderState>,
    axis: Axis,
    disabled: bool,
    reverse: bool,
    thumb_width: f32,
    thumb_height: f32,
    track_color: gpui::Hsla,
    thumb_color: gpui::Hsla,
    disabled_color: gpui::Hsla,
}

impl RenderOnce for ThemedSlider {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let percentage = self.state.read(cx).percentage();
        let axis = self.axis;
        let horizontal = axis.is_horizontal();
        let fill_color = if self.disabled {
            self.disabled_color.opacity(0.5)
        } else {
            self.track_color
        };
        let thumb_color = if self.disabled {
            self.disabled_color
        } else {
            self.thumb_color
        };
        let fill_start = if self.reverse { percentage.end } else { 0.0 };
        let fill_end = if self.reverse { 1.0 } else { percentage.end };

        let mut slider = div()
            .id(("slider", self.state.entity_id()))
            .role(Role::Slider)
            .aria_numeric_value(self.state.read(cx).value().end() as f64)
            .aria_min_numeric_value(self.state.read(cx).min_value() as f64)
            .aria_max_numeric_value(self.state.read(cx).max_value() as f64)
            .aria_numeric_value_step(self.state.read(cx).step_value() as f64)
            .aria_orientation(if axis.is_vertical() {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            })
            .flex()
            .items_center()
            .justify_center()
            .when(horizontal, |slider| {
                slider.w_full().h(px(self.thumb_height))
            })
            .when(!horizontal, |slider| {
                slider.w(px(self.thumb_width)).h(px(120.0))
            })
            .when(self.disabled, |slider| {
                slider.a11y_synthetic_children(|builder| {
                    set_slider_accessibility_disabled(builder.parent_node());
                })
            });

        if !self.disabled {
            let slider_state = self.state.clone();
            slider = slider
                .on_a11y_action(AccessibleAction::SetValue, {
                    let state = slider_state.clone();
                    move |data, window, cx| {
                        let Some(value) = accessibility_set_value(
                            data,
                            false,
                            state.read(cx).min_value(),
                            state.read(cx).max_value(),
                            state.read(cx).step_value(),
                        ) else {
                            return;
                        };
                        state.update(cx, |state, cx| {
                            state.set_value(value, window, cx);
                            cx.emit(SliderEvent::Change(SliderValue::Single(value)));
                        });
                    }
                })
                .on_a11y_action(AccessibleAction::Increment, {
                    let state = slider_state.clone();
                    move |_, window, cx| {
                        state.update(cx, |state, cx| {
                            let value =
                                (state.value().end() + state.step_value()).min(state.max_value());
                            state.set_value(value, window, cx);
                            cx.emit(SliderEvent::Change(SliderValue::Single(value)));
                        });
                    }
                })
                .on_a11y_action(AccessibleAction::Decrement, {
                    move |_, window, cx| {
                        slider_state.update(cx, |state, cx| {
                            let value =
                                (state.value().end() - state.step_value()).max(state.min_value());
                            state.set_value(value, window, cx);
                            cx.emit(SliderEvent::Change(SliderValue::Single(value)));
                        });
                    }
                });
        }
        if !self.disabled {
            slider = slider
                .on_mouse_up(
                    MouseButton::Left,
                    window.listener_for(&self.state, |state, _, _, cx| state.handle_release(cx)),
                )
                .on_mouse_up_out(
                    MouseButton::Left,
                    window.listener_for(&self.state, |state, _, _, cx| state.handle_release(cx)),
                );
        }
        slider.child(
            SliderTrack::new(&self.state)
                .axis(axis)
                .disabled(self.disabled)
                .flex()
                .items_center()
                .justify_center()
                .when(horizontal, |track| track.w_full().h(px(self.thumb_height)))
                .when(!horizontal, |track| track.h_full().w(px(self.thumb_width)))
                .child(
                    SliderIndicator::new(&self.state)
                        .relative()
                        .when(horizontal, |indicator| indicator.w_full().h(px(6.0)))
                        .when(!horizontal, |indicator| indicator.h_full().w(px(6.0)))
                        .rounded_full()
                        .bg(self
                            .track_color
                            .opacity(if self.disabled { 0.1 } else { 0.2 }))
                        .child(
                            div()
                                .absolute()
                                .when(horizontal, |fill| {
                                    fill.h_full()
                                        .left(relative(fill_start))
                                        .right(relative(1.0 - fill_end))
                                })
                                .when(!horizontal, |fill| {
                                    fill.w_full()
                                        .bottom(relative(fill_start))
                                        .top(relative(1.0 - fill_end))
                                })
                                .rounded_full()
                                .bg(fill_color),
                        )
                        .child(
                            SliderThumb::new(&self.state)
                                .axis(axis)
                                .disabled(self.disabled)
                                .absolute()
                                .when(horizontal, |thumb| {
                                    thumb
                                        .top(px((6.0 - self.thumb_height) / 2.0))
                                        .left(relative(percentage.end))
                                        .ml(px(-self.thumb_width / 2.0))
                                })
                                .when(!horizontal, |thumb| {
                                    thumb
                                        .left(px((6.0 - self.thumb_width) / 2.0))
                                        .bottom(relative(percentage.end))
                                        .mb(px(-self.thumb_height / 2.0))
                                })
                                .w(px(self.thumb_width))
                                .h(px(self.thumb_height))
                                .rounded_full()
                                .bg(thumb_color),
                        ),
                ),
        )
    }
}

fn accessibility_set_value(
    data: Option<&gpui::accesskit::ActionData>,
    disabled: bool,
    min: f32,
    max: f32,
    step: f32,
) -> Option<f32> {
    if disabled {
        return None;
    }
    let Some(gpui::accesskit::ActionData::NumericValue(value)) = data else {
        return None;
    };
    value
        .is_finite()
        .then(|| normalize_accessibility_value(*value as f32, min, max, step))
}

fn normalize_accessibility_value(value: f32, min: f32, max: f32, step: f32) -> f32 {
    ((value.clamp(min, max) / step).round() * step).clamp(min, max)
}

fn set_slider_accessibility_disabled(node: &mut gpui::accesskit::Node) {
    node.set_disabled();
}

fn slider_role_color(
    style: &crate::theme::ResolvedComponentStyle,
    role: &str,
    profile: &crate::theme::ThemeProfile,
    mode: AppearanceMode,
) -> gpui::Hsla {
    let role_name = style.record_metadata["theme_roles"][role]
        .as_str()
        .unwrap_or_else(|| panic!("slider theme is missing the `{role}` color role"));
    let colors = profile.colors(Some(mode));
    // macos27's component roles resolve through its global theme roles to these
    // profile color tokens: secondary -> toolbar, primary -> accent, and
    // muted_foreground -> secondary_foreground.
    let token = match role_name {
        "secondary" => &colors.toolbar_background,
        "primary" => &colors.accent,
        "muted_foreground" => &colors.secondary_text,
        other => panic!("slider theme color role `{other}` is not supported"),
    };
    try_parse_color(token)
        .unwrap_or_else(|error| panic!("invalid resolved RustUI slider color `{token}`: {error}"))
}

#[cfg(test)]
mod tests {
    use super::{
        accessibility_set_value, normalize_accessibility_value, set_slider_accessibility_disabled,
        slider_role_color,
    };
    use crate::components::theme::selector_for_mode;
    use crate::theme::{AppearanceMode, loader::parse_theme, resolver::resolve_component_style};
    use gpui_component::theme::try_parse_color;

    #[test]
    fn macos27_slider_thumb_dimensions_are_available_in_both_modes() {
        let theme = parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let mut selector = selector_for_mode(mode);
            selector.chrome = "utility".into();
            let style = resolve_component_style(&theme, "slider", "sliders", &selector).unwrap();
            assert_eq!(style.lengths.get("width"), Some(&24.0));
            assert_eq!(style.lengths.get("height"), Some(&24.0));
        }
    }

    #[test]
    fn macos27_slider_colors_resolve_from_component_roles_and_profile_tokens() {
        let theme = parse_theme(include_str!("../../../../../themes/macos27.json")).unwrap();
        for mode in [AppearanceMode::Light, AppearanceMode::Dark] {
            let mut selector = selector_for_mode(mode);
            selector.chrome = "utility".into();
            let style = resolve_component_style(&theme, "slider", "sliders", &selector).unwrap();

            assert_eq!(
                slider_role_color(&style, "track", &theme.platform_snapshot, mode),
                try_parse_color(
                    &theme
                        .platform_snapshot
                        .colors(Some(mode))
                        .toolbar_background
                )
                .unwrap()
            );
            assert_eq!(
                slider_role_color(&style, "thumb", &theme.platform_snapshot, mode),
                try_parse_color(&theme.platform_snapshot.colors(Some(mode)).accent).unwrap()
            );
            assert_eq!(
                slider_role_color(&style, "disabled", &theme.platform_snapshot, mode),
                try_parse_color(&theme.platform_snapshot.colors(Some(mode)).secondary_text)
                    .unwrap()
            );
        }
    }

    #[test]
    fn slider_accessibility_set_value_clamps_and_applies_step() {
        assert_eq!(normalize_accessibility_value(37.0, 0.0, 100.0, 5.0), 35.0);
        assert_eq!(normalize_accessibility_value(-4.0, 0.0, 100.0, 5.0), 0.0);
        assert_eq!(normalize_accessibility_value(108.0, 0.0, 100.0, 5.0), 100.0);
    }

    #[test]
    fn disabled_slider_rejects_accessibility_set_value() {
        let data = gpui::accesskit::ActionData::NumericValue(42.0);
        assert_eq!(
            accessibility_set_value(Some(&data), true, 0.0, 100.0, 5.0),
            None
        );
        assert_eq!(
            accessibility_set_value(Some(&data), false, 0.0, 100.0, 5.0),
            Some(40.0)
        );
    }

    #[test]
    fn disabled_slider_marks_accesskit_node_disabled() {
        let mut node = gpui::accesskit::Node::new(gpui::accesskit::Role::Slider);
        set_slider_accessibility_disabled(&mut node);
        assert!(node.is_disabled());
    }
}
