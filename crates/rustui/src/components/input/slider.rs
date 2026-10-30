//! Controlled single-value slider backed by a retained GPUI SliderState.

use std::{cell::RefCell, ops::RangeInclusive, rc::Rc};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AccessibleAction, AnyElement, App, AppContext as _, Axis, InteractiveElement, IntoElement,
    MouseButton, Orientation, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window, div, px, relative,
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

type SliderEventWriter<Context> = Rc<dyn Fn(f32, &mut Context)>;

fn replace_event_writer<Context>(
    slot: &RefCell<Option<SliderEventWriter<Context>>>,
    writer: SliderEventWriter<Context>,
) {
    *slot.borrow_mut() = Some(writer);
}

fn write_with_current_writer<Context>(
    slot: &RefCell<Option<SliderEventWriter<Context>>>,
    value: f32,
    cx: &mut Context,
) {
    let writer = slot.borrow().clone();
    if let Some(writer) = writer {
        writer(value, cx);
    }
}

fn bounded_value(value: f32, min: f32, max: f32) -> f32 {
    if value.is_nan() {
        min
    } else {
        value.clamp(min, max)
    }
}

fn bounded_event_writer<Context: 'static>(
    writer: SliderEventWriter<Context>,
    min: f32,
    max: f32,
) -> SliderEventWriter<Context> {
    Rc::new(move |value, cx| writer(bounded_value(value, min, max), cx))
}

struct RetainedSlider {
    state: gpui::Entity<SliderState>,
    writer: Rc<RefCell<Option<SliderEventWriter<App>>>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum SliderLabel {
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

fn slider_accessibility_label(label: Option<SharedString>) -> SharedString {
    label
        .filter(|label| !label.trim().is_empty())
        .unwrap_or_else(|| "Slider".into())
}

pub struct SliderView {
    value: Binding<f32>,
    range: RangeInclusive<f32>,
    step: f32,
    id: Option<Identity>,
    accessibility_label: Option<SliderLabel>,
    disabled: bool,
    vertical: bool,
    reverse: bool,
}

impl SliderView {
    pub fn new(value: Binding<f32>, range: RangeInclusive<f32>) -> Self {
        assert!(
            range.start().is_finite()
                && range.end().is_finite()
                && range.start() < range.end()
                && (range.end() - range.start()).is_finite(),
            "Slider range must contain finite ascending values with a finite span"
        );
        Self {
            value,
            range,
            step: 1.0,
            id: None,
            accessibility_label: None,
            disabled: false,
            vertical: false,
            reverse: false,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Slider IDs must be nonempty"
        );
        self.id = Some(id);
        self
    }

    /// Names the slider for assistive technology.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "slider accessibility label must not be empty"
        );
        self.accessibility_label = Some(SliderLabel::Literal(label));
        self
    }

    /// Resolves the slider's accessible name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "slider accessibility localization keys must be nonempty"
        );
        let fallback = match &self.accessibility_label {
            Some(SliderLabel::Literal(label))
            | Some(SliderLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => SharedString::from("Slider"),
        };
        self.accessibility_label = Some(SliderLabel::Message {
            id: message_id,
            fallback,
        });
        self
    }

    pub fn step(mut self, step: f32) -> Self {
        let snapping_stays_finite = |value: f32| {
            let quotient = value / step;
            quotient.is_finite() && (quotient.round() * step).is_finite()
        };
        assert!(
            step.is_finite()
                && step > 0.0
                && snapping_stays_finite(*self.range.start())
                && snapping_stays_finite(*self.range.end()),
            "Slider step must be positive, finite, and safe for its range"
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

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| allocate_implicit(Identity::from("slider")))
    }

    fn clamped_value(&self) -> f32 {
        bounded_value(self.value.value(), *self.range.start(), *self.range.end())
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
        let identity = self.identity(context);
        let min = *self.range.start();
        let max = *self.range.end();
        let step = self.step;
        let initial_value = self.clamped_value();
        let (retained, created) =
            context.retained_entity(identity.clone(), move |_window, cx| RetainedSlider {
                state: cx.new(|_cx| configured_slider_state(min, max, step, initial_value)),
                writer: Rc::new(RefCell::new(None)),
            });
        let (state, writer) = {
            let retained = retained.read(context.app());
            (retained.state.clone(), retained.writer.clone())
        };

        let configuration_changed = {
            let retained = state.read(context.app());
            retained.min_value() != min
                || retained.max_value() != max
                || retained.step_value() != step
        };
        if configuration_changed {
            context.update_entity(&state, move |state, _window, _cx| {
                refresh_slider_configuration(state, min, max, step, initial_value);
            });
        }

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

        let binding_writer = context.binding_writer(self.value.clone());
        let bounded_writer = bounded_event_writer(binding_writer, min, max);
        replace_event_writer(&writer, bounded_writer);
        if created {
            let current_writer = writer.clone();
            context.subscribe_retained(
                &identity,
                &state,
                move |_state, event: &SliderEvent, cx| {
                    if let SliderEvent::Change(SliderValue::Single(value)) = event {
                        write_with_current_writer(&current_writer, *value, cx);
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
            accessibility_label: self.accessibility_label.as_ref().map(|label| match label {
                SliderLabel::Literal(label) => label.clone(),
                SliderLabel::Message { id, fallback } => {
                    resolved_message_label(id, context.localized_message(id.as_ref()), fallback)
                }
            }),
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

fn configured_slider_state(min: f32, max: f32, step: f32, value: f32) -> SliderState {
    SliderState::new()
        .min(min)
        .max(max)
        .step(step)
        .default_value(value)
}

fn refresh_slider_configuration(
    state: &mut SliderState,
    min: f32,
    max: f32,
    step: f32,
    value: f32,
) -> bool {
    if state.min_value() == min && state.max_value() == max && state.step_value() == step {
        return false;
    }
    *state = configured_slider_state(min, max, step, value);
    true
}

/// Uses GPUI Base slider primitives for pointer and touch behavior. RustUI owns
/// the semantic root so AX SetValue can update both retained state and binding.
#[derive(IntoElement)]
struct ThemedSlider {
    state: gpui::Entity<SliderState>,
    axis: Axis,
    disabled: bool,
    reverse: bool,
    accessibility_label: Option<SharedString>,
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
            .aria_label(slider_accessibility_label(self.accessibility_label))
            .when(self.disabled, |slider| {
                slider.a11y_synthetic_children(|builder| {
                    set_slider_accessibility_disabled(builder.parent_node());
                })
            });

        if !self.disabled {
            let slider_state = self.state.clone();
            let keyboard_state = slider_state.clone();
            slider = slider
                .focusable()
                .tab_stop(true)
                .on_key_down(move |event, window, cx| {
                    if event.prefer_character_input
                        || event.keystroke.is_ime_in_progress()
                        || event.keystroke.modifiers.modified()
                    {
                        return;
                    }
                    let (current, min, max, step) = {
                        let state = keyboard_state.read(cx);
                        let current = match state.value() {
                            SliderValue::Single(value) => value,
                            SliderValue::Range(_, _) => {
                                unreachable!("RustUI Slider retains single-value state")
                            }
                        };
                        (
                            current,
                            state.min_value(),
                            state.max_value(),
                            state.step_value(),
                        )
                    };
                    let Some(value) = slider_keyboard_value(
                        axis,
                        event.keystroke.key.as_ref(),
                        current,
                        min,
                        max,
                        step,
                    ) else {
                        return;
                    };
                    if value != current {
                        keyboard_state.update(cx, |state, cx| {
                            state.set_value(value, window, cx);
                            cx.emit(SliderEvent::Change(SliderValue::Single(value)));
                        });
                    }
                    cx.stop_propagation();
                })
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

pub(super) fn slider_keyboard_value(
    axis: Axis,
    key: &str,
    current: f32,
    min: f32,
    max: f32,
    step: f32,
) -> Option<f32> {
    let value = match (axis, key) {
        (_, "home") => return Some(min),
        (_, "end") => return Some(max),
        (Axis::Horizontal, "left") | (Axis::Vertical, "down") => current - step,
        (Axis::Horizontal, "right") | (Axis::Vertical, "up") => current + step,
        _ => return None,
    };
    Some(value.clamp(min, max))
}

fn normalize_accessibility_value(value: f32, min: f32, max: f32, step: f32) -> f32 {
    let clamped = value.clamp(min, max);
    let quotient = clamped / step;
    if quotient.is_finite() {
        // Match GPUI's pointer path, which divides, rounds, and multiplies in f32.
        return (quotient.round() * step).clamp(min, max);
    }

    // Preserve useful normalization for direct calls with subnormal steps even
    // though SliderView rejects steps whose configured range overflows in f32.
    let min = f64::from(min);
    let max = f64::from(max);
    let step = f64::from(step);
    ((f64::from(clamped).clamp(min, max) / step).round() * step).clamp(min, max) as f32
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
        accessibility_set_value, bounded_event_writer, bounded_value,
        normalize_accessibility_value, replace_event_writer, set_slider_accessibility_disabled,
        slider_keyboard_value, slider_role_color, write_with_current_writer,
    };
    use crate::components::theme::selector_for_mode;
    use crate::theme::{AppearanceMode, loader::parse_theme, resolver::resolve_component_style};
    use gpui::Axis;
    use gpui_component::theme::try_parse_color;
    use rustui_core::Binding;
    use std::{cell::RefCell, rc::Rc};

    #[test]
    #[should_panic(
        expected = "Slider range must contain finite ascending values with a finite span"
    )]
    fn rejects_ranges_with_an_overflowing_span() {
        let binding = Binding::new(0.0, |value: &f32| *value, |value, next| *value = next);
        super::Slider(binding, -f32::MAX..=f32::MAX);
    }

    #[test]
    fn accepts_large_ranges_with_a_finite_span() {
        let binding = Binding::new(0.0, |value: &f32| *value, |value, next| *value = next);
        let slider = super::Slider(binding, -f32::MAX..=0.0);

        assert_eq!(*slider.range.start(), -f32::MAX);
        assert_eq!(*slider.range.end(), 0.0);
    }

    #[test]
    fn retained_slider_events_use_the_latest_owner_binding() {
        struct Owner {
            old_value: f32,
            current_value: f32,
        }
        let old_binding = Binding::new_with::<Owner, &'static str>(
            0.0,
            "old",
            |owner, field| match *field {
                "old" => owner.old_value,
                _ => owner.current_value,
            },
            |owner, field, next| {
                match *field {
                    "old" => owner.old_value = next,
                    _ => owner.current_value = next,
                }
                true
            },
        );
        let current_binding = Binding::new_with::<Owner, &'static str>(
            0.0,
            "current",
            |owner, field| match *field {
                "old" => owner.old_value,
                _ => owner.current_value,
            },
            |owner, field, next| {
                match *field {
                    "old" => owner.old_value = next,
                    _ => owner.current_value = next,
                }
                true
            },
        );
        let writer = RefCell::new(None);
        let mut owner = Owner {
            old_value: 0.0,
            current_value: 0.0,
        };
        replace_event_writer(
            &writer,
            Rc::new(move |value, owner: &mut Owner| {
                old_binding.write_to(owner, value);
            }),
        );
        replace_event_writer(
            &writer,
            bounded_event_writer(
                Rc::new(move |value, owner: &mut Owner| {
                    current_binding.write_to(owner, value);
                }),
                0.0,
                10.0,
            ),
        );
        write_with_current_writer(&writer, f32::INFINITY, &mut owner);
        assert_eq!(owner.old_value, 0.0);
        assert_eq!(owner.current_value, 10.0);
        write_with_current_writer(&writer, f32::NAN, &mut owner);
        assert_eq!(owner.current_value, 0.0);
    }

    #[test]
    fn slider_accessibility_label_is_optional_and_owner_settable() {
        let binding = rustui_core::Binding::new(
            50.0_f32,
            |value: &f32| *value,
            |value, next| {
                *value = next;
            },
        );
        let unnamed = super::Slider(binding.clone(), 0.0..=100.0);
        assert_eq!(unnamed.accessibility_label, None);
        assert_eq!(
            super::Slider(binding, 0.0..=100.0)
                .accessibility_label(" Volume ")
                .accessibility_label,
            Some(super::SliderLabel::Literal(" Volume ".into()))
        );
    }

    #[test]
    fn slider_without_an_override_uses_a_nonblank_accessible_name() {
        assert_eq!(super::slider_accessibility_label(None), "Slider");
        assert_eq!(
            super::slider_accessibility_label(Some("Volume".into())),
            "Volume"
        );
    }

    #[test]
    fn keyboard_adjustment_tracks_orientation_and_clamps_to_range() {
        assert_eq!(
            slider_keyboard_value(Axis::Horizontal, "right", 5.0, 0.0, 10.0, 2.0),
            Some(7.0)
        );
        assert_eq!(
            slider_keyboard_value(Axis::Horizontal, "left", 1.0, 0.0, 10.0, 2.0),
            Some(0.0)
        );
        assert_eq!(
            slider_keyboard_value(Axis::Vertical, "up", 5.0, 0.0, 10.0, 2.0),
            Some(7.0)
        );
        assert_eq!(
            slider_keyboard_value(Axis::Vertical, "down", 5.0, 0.0, 10.0, 2.0),
            Some(3.0)
        );
        assert_eq!(
            slider_keyboard_value(Axis::Horizontal, "home", 5.0, 0.0, 10.0, 2.0),
            Some(0.0)
        );
        assert_eq!(
            slider_keyboard_value(Axis::Vertical, "end", 5.0, 0.0, 10.0, 2.0),
            Some(10.0)
        );
        assert_eq!(
            slider_keyboard_value(Axis::Horizontal, "up", 5.0, 0.0, 10.0, 2.0),
            None
        );
    }

    #[test]
    fn localized_accessibility_label_resolves_message_and_preserves_fallback() {
        let binding =
            rustui_core::Binding::new(50.0_f32, |value: &f32| *value, |value, next| *value = next);
        let view = super::Slider(binding, 0.0..=100.0)
            .accessibility_label("Volume")
            .localized_accessibility_label("slider.volume");
        let Some(super::SliderLabel::Message { id, fallback }) = view.accessibility_label else {
            panic!("expected localized slider accessibility label");
        };

        assert_eq!(id.as_ref(), "slider.volume");
        assert_eq!(fallback.as_ref(), "Volume");
        assert_eq!(
            super::resolved_message_label(&id, "Volume sonore".into(), &fallback).as_ref(),
            "Volume sonore"
        );
    }

    #[test]
    fn localized_accessibility_label_uses_slider_fallback_for_blank_messages() {
        let binding =
            rustui_core::Binding::new(50.0_f32, |value: &f32| *value, |value, next| *value = next);
        let view =
            super::Slider(binding, 0.0..=100.0).localized_accessibility_label("slider.volume");
        let Some(super::SliderLabel::Message { id, fallback }) = view.accessibility_label else {
            panic!("expected localized slider accessibility label");
        };

        assert_eq!(fallback.as_ref(), "Slider");
        assert_eq!(
            super::resolved_message_label(&id, " \t\n".into(), &fallback),
            fallback
        );
    }

    #[test]
    fn explicit_accessibility_label_must_not_be_blank() {
        for label in ["", " ", "\t\n"] {
            assert!(
                std::panic::catch_unwind(|| {
                    let binding = rustui_core::Binding::new(
                        50.0_f32,
                        |value: &f32| *value,
                        |value, next| *value = next,
                    );
                    super::Slider(binding, 0.0..=100.0).accessibility_label(label);
                })
                .is_err()
            );
        }
    }

    #[test]
    fn localized_accessibility_label_requires_nonblank_message_id() {
        for message_id in ["", " ", "\t\n"] {
            assert!(
                std::panic::catch_unwind(|| {
                    let binding = rustui_core::Binding::new(
                        50.0_f32,
                        |value: &f32| *value,
                        |value, next| *value = next,
                    );
                    super::Slider(binding, 0.0..=100.0).localized_accessibility_label(message_id);
                })
                .is_err()
            );
        }
    }

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
    fn slider_accessibility_set_value_matches_pointer_rounding_in_range() {
        let value = -15_199.308_593_75_f32;
        let step = 188.811_294_555_664_06_f32;
        let pointer_value = (value / step).round() * step;

        assert_eq!(
            normalize_accessibility_value(value, -20_000.0, 0.0, step),
            pointer_value
        );
    }

    #[test]
    fn slider_accessibility_set_value_handles_subnormal_steps() {
        assert_eq!(
            normalize_accessibility_value(0.5, 0.0, 1.0, f32::from_bits(1)),
            0.5
        );
    }

    #[test]
    #[should_panic(expected = "Slider step must be positive, finite, and safe for its range")]
    fn rejects_steps_that_overflow_gpui_step_rounding() {
        let binding = Binding::new(0.5, |value: &f32| *value, |value, next| *value = next);
        let _ = super::Slider(binding, 0.0..=1.0).step(f32::from_bits(1));
    }

    #[test]
    #[should_panic(expected = "Slider step must be positive, finite, and safe for its range")]
    fn rejects_steps_whose_snapped_endpoint_overflows() {
        let binding = Binding::new(
            3.0e38_f32,
            |value: &f32| *value,
            |value, next| *value = next,
        );
        let _ = super::Slider(binding, 3.0e38_f32..=f32::MAX).step(1.2e38_f32);
    }

    #[test]
    fn controlled_slider_value_is_finite_and_within_bounds() {
        let binding = |value| {
            rustui_core::Binding::new(value, |value: &f32| *value, |value, next| *value = next)
        };

        for (input, expected) in [
            (f32::NAN, -10.0),
            (f32::NEG_INFINITY, -10.0),
            (f32::INFINITY, 10.0),
            (-11.0, -10.0),
            (11.0, 10.0),
            (3.5, 3.5),
        ] {
            assert_eq!(
                super::Slider(binding(input), -10.0..=10.0).clamped_value(),
                expected
            );
        }
    }

    #[test]
    fn slider_event_values_are_finite_and_within_current_bounds() {
        assert_eq!(bounded_value(f32::NAN, -10.0, 10.0), -10.0);
        assert_eq!(bounded_value(f32::NEG_INFINITY, -10.0, 10.0), -10.0);
        assert_eq!(bounded_value(f32::INFINITY, -10.0, 10.0), 10.0);
        assert_eq!(bounded_value(11.0, -10.0, 10.0), 10.0);
        assert_eq!(bounded_value(3.5, -10.0, 10.0), 3.5);
    }

    #[test]
    fn retained_slider_state_refreshes_changed_range_and_step() {
        let mut state = super::configured_slider_state(0.0, 100.0, 1.0, 50.0);
        assert_eq!(state.min_value(), 0.0);
        assert_eq!(state.max_value(), 100.0);
        assert_eq!(state.step_value(), 1.0);

        assert!(super::refresh_slider_configuration(
            &mut state, 20.0, 80.0, 5.0, 50.0
        ));
        assert_eq!(state.min_value(), 20.0);
        assert_eq!(state.max_value(), 80.0);
        assert_eq!(state.step_value(), 5.0);
        assert_eq!(state.value().end(), 50.0);
        assert!(!super::refresh_slider_configuration(
            &mut state, 20.0, 80.0, 5.0, 50.0
        ));
    }

    #[test]
    fn implicit_slider_identity_distinguishes_siblings_and_is_stable_by_slot() {
        let binding = Binding::new(50.0_f32, |value: &f32| *value, |value, next| *value = next);
        let slider = super::Slider(binding, 0.0..=100.0);
        let allocate_at_slot = |slot: usize| {
            move |base: rustui_core::Identity| {
                rustui_core::Identity::new(format!(
                    "rustui-text-owner-7:implicit-retained:{slot}:{base}"
                ))
            }
        };

        let first_sibling = slider.identity_with(allocate_at_slot(0));
        let second_sibling = slider.identity_with(allocate_at_slot(1));
        let first_sibling_after_rerender = slider.identity_with(allocate_at_slot(0));
        let reconfigured = super::Slider(
            Binding::new(50.0_f32, |value: &f32| *value, |value, next| *value = next),
            20.0..=80.0,
        )
        .step(5.0);
        let first_sibling_after_reconfiguration = reconfigured.identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_sibling_after_rerender);
        assert_eq!(first_sibling, first_sibling_after_reconfiguration);
    }

    #[test]
    fn explicit_slider_identity_bypasses_implicit_allocator() {
        let binding = Binding::new(50.0_f32, |value: &f32| *value, |value, next| *value = next);
        let slider = super::Slider(binding, 0.0..=100.0).id(" volume ");

        assert_eq!(
            slider.identity_with(|_| panic!("explicit identity must not allocate a fallback")),
            rustui_core::Identity::from(" volume ")
        );
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
