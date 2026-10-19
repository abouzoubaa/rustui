//! Controlled date selection backed by a retained GPUI DatePickerState.

use chrono::NaiveDate;
use gpui::{AnyElement, IntoElement, SharedString};
use gpui_component::{
    Disableable as _,
    calendar::Date as CalendarDate,
    date_picker::{DatePicker as GpuiDatePicker, DatePickerEvent, DatePickerState, DateTime},
};
use rustui_core::{Binding, Identity};

use crate::view::{EmptyView, RenderContext, View};

/// A controlled optional date picker whose calendar state survives body rebuilds.
pub struct DatePickerView {
    label: SharedString,
    value: Binding<Option<NaiveDate>>,
    id: Option<Identity>,
    disabled: bool,
    cleanable: bool,
    number_of_months: usize,
    appearance: bool,
}

impl DatePickerView {
    pub fn new(label: impl Into<SharedString>, value: Binding<Option<NaiveDate>>) -> Self {
        Self {
            label: label.into(),
            value,
            id: None,
            disabled: false,
            cleanable: true,
            number_of_months: 1,
            appearance: true,
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

    pub fn cleanable(mut self, cleanable: bool) -> Self {
        self.cleanable = cleanable;
        self
    }

    pub fn number_of_months(mut self, number_of_months: usize) -> Self {
        self.number_of_months = number_of_months.max(1);
        self
    }

    pub fn appearance(mut self, appearance: bool) -> Self {
        self.appearance = appearance;
        self
    }

    fn identity(&self) -> Identity {
        self.id
            .clone()
            .unwrap_or_else(|| Identity::from(format!("date-picker:{}", self.label)))
    }
}

#[allow(non_snake_case)]
pub fn DatePicker(
    label: impl Into<SharedString>,
    value: Binding<Option<NaiveDate>>,
) -> DatePickerView {
    DatePickerView::new(label, value)
}

impl View for DatePickerView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.identity();
        let initial_value = self.value.value();
        let (state, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let mut state = DatePickerState::new(window, cx);
            state.set_date(CalendarDate::Single(initial_value), window, cx);
            state
        });

        let current = self.value.value();
        let retained = match state.read(context.app()).date() {
            CalendarDate::Single(value) => value,
            CalendarDate::Range(_, _) => unreachable!("RustUI DatePicker retains one date"),
        };
        if retained != current {
            context.update_entity(&state, move |state, window, cx| {
                state.set_date(CalendarDate::Single(current), window, cx);
            });
        }

        if created {
            let write = context.binding_writer(self.value.clone());
            context.subscribe_retained(
                &identity,
                &state,
                move |_state, event: &DatePickerEvent, cx| {
                    let DatePickerEvent::Change(value) = event;
                    match *value {
                        DateTime::Single(value) => write(value.map(|value| value.date()), cx),
                        DateTime::Range(_, _) => {
                            unreachable!("RustUI DatePicker retains one date")
                        }
                    }
                },
            );
        }

        GpuiDatePicker::new(&state)
            .placeholder(self.label.clone())
            .cleanable(self.cleanable)
            .number_of_months(self.number_of_months)
            .appearance(self.appearance)
            .disabled(self.disabled)
            .into_any_element()
    }
}
