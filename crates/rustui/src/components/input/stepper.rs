//! Controlled integer stepper composed from GPUI Components buttons.

use std::ops::RangeInclusive;

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div};
use gpui_component::{
    Disableable as _, Sizable as _, Size,
    button::{Button as GpuiButton, ButtonVariant, ButtonVariants as _},
};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, selector, styled_element},
    view::{EmptyView, RenderContext, View},
};

pub struct StepperView {
    label: SharedString,
    value: Binding<i32>,
    range: RangeInclusive<i32>,
    step: i32,
    id: Option<Identity>,
    disabled: bool,
}

fn stepped_value(value: i32, minimum: i32, maximum: i32, step: i32, increment: bool) -> i32 {
    let value = value.clamp(minimum, maximum);
    let stepped = if increment {
        value.saturating_add(step)
    } else {
        value.saturating_sub(step)
    };
    stepped.clamp(minimum, maximum)
}

impl StepperView {
    pub fn new(label: impl Into<SharedString>, value: Binding<i32>) -> Self {
        Self {
            label: label.into(),
            value,
            range: i32::MIN..=i32::MAX,
            step: 1,
            id: None,
            disabled: false,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn range(mut self, range: RangeInclusive<i32>) -> Self {
        assert!(
            range.start() <= range.end(),
            "Stepper range must be ascending"
        );
        self.range = range;
        self
    }

    pub fn step(mut self, step: i32) -> Self {
        assert!(step > 0, "Stepper step must be positive");
        self.step = step;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    fn identity(&self) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("stepper:{}", self.label)))
    }
}

#[allow(non_snake_case)]
pub fn Stepper(label: impl Into<SharedString>, value: Binding<i32>) -> StepperView {
    StepperView::new(label, value)
}

impl View for StepperView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity();
        let current = self
            .value
            .value()
            .clamp(*self.range.start(), *self.range.end());
        let decrement = context.binding_updater(self.value.clone(), {
            let minimum = *self.range.start();
            let maximum = *self.range.end();
            let step = self.step;
            move |value| {
                let next = stepped_value(*value, minimum, maximum, step, false);
                if next == *value {
                    return false;
                }
                *value = next;
                true
            }
        });
        let increment = context.binding_updater(self.value.clone(), {
            let minimum = *self.range.start();
            let maximum = *self.range.end();
            let step = self.step;
            move |value| {
                let next = stepped_value(*value, minimum, maximum, step, true);
                if next == *value {
                    return false;
                }
                *value = next;
                true
            }
        });

        let selector = selector(context);
        let increment_style = required_style(context, "stepper", "increment_button", &selector);
        let decrement_style = required_style(context, "stepper", "decrement_button", &selector);
        let separator_style = required_style(context, "stepper", "separator", &selector);
        let increment_glyph = required_style(context, "stepper", "increment_glyph", &selector);
        let decrement_glyph = required_style(context, "stepper", "decrement_glyph", &selector);
        let increment_button = GpuiButton::new(format!("{identity}-increment"))
            .accessibility_label(format!("Increase {}", self.label))
            .compact()
            .with_size(Size::XSmall)
            .with_variant(ButtonVariant::Secondary)
            .disabled(self.disabled || current >= *self.range.end())
            .on_click(move |_, _, cx| increment(cx))
            .child(styled_element(div().child("+"), &increment_glyph));
        let decrement_button = GpuiButton::new(format!("{identity}-decrement"))
            .accessibility_label(format!("Decrease {}", self.label))
            .compact()
            .with_size(Size::XSmall)
            .with_variant(ButtonVariant::Secondary)
            .disabled(self.disabled || current <= *self.range.start())
            .on_click(move |_, _, cx| decrement(cx))
            .child(styled_element(div().child("−"), &decrement_glyph));

        div()
            .flex()
            .items_center()
            .gap_2()
            .child(self.label.clone())
            .child(current.to_string())
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(styled_element(increment_button, &increment_style))
                    .child(styled_element(div(), &separator_style))
                    .child(styled_element(decrement_button, &decrement_style)),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::stepped_value;

    #[test]
    fn stepper_clamps_owner_value_before_applying_each_step() {
        assert_eq!(stepped_value(4, 5, 15, 2, false), 5);
        assert_eq!(stepped_value(16, 5, 15, 2, true), 15);
        assert_eq!(stepped_value(10, 5, 15, 2, true), 12);
        assert_eq!(stepped_value(10, 5, 15, 2, false), 8);
    }

    #[test]
    fn stepper_saturates_at_integer_limits_and_configured_bounds() {
        assert_eq!(
            stepped_value(i32::MAX, i32::MIN, i32::MAX, 1, true),
            i32::MAX
        );
        assert_eq!(
            stepped_value(i32::MIN, i32::MIN, i32::MAX, 1, false),
            i32::MIN
        );
        assert_eq!(stepped_value(14, 5, 15, 4, true), 15);
        assert_eq!(stepped_value(6, 5, 15, 4, false), 5);
    }
}
