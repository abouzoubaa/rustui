//! Controlled date selection backed by a retained GPUI DatePickerState.

use std::{cell::RefCell, rc::Rc};

use chrono::{Datelike, NaiveDate, Weekday};
use gpui::{AnyElement, AppContext as _, IntoElement, SharedString};
use gpui_component::{
    Disableable as _,
    calendar::Date as CalendarDate,
    date_picker::{DatePicker as GpuiDatePicker, DatePickerEvent, DatePickerState, DateTime},
};
use rustui_core::{Binding, Identity};
use rustui_i18n::{
    CivilDate, DateLength, LayoutDirection, Locale, LocaleFormatter, Weekday as LocaleWeekday,
};

use crate::view::{EmptyView, RenderContext, View};

type EventWriter<Value, Context> = Rc<dyn Fn(Value, &mut Context)>;

fn localized_calendar(
    locale: Option<&Locale>,
    value: Option<NaiveDate>,
) -> (Weekday, Option<String>) {
    let Some(formatter) = locale.and_then(|locale| LocaleFormatter::try_new(locale).ok()) else {
        return (Weekday::Sun, None);
    };
    let week_start = match formatter.first_weekday() {
        Ok(LocaleWeekday::Monday) => Weekday::Mon,
        Ok(LocaleWeekday::Tuesday) => Weekday::Tue,
        Ok(LocaleWeekday::Wednesday) => Weekday::Wed,
        Ok(LocaleWeekday::Thursday) => Weekday::Thu,
        Ok(LocaleWeekday::Friday) => Weekday::Fri,
        Ok(LocaleWeekday::Saturday) => Weekday::Sat,
        Ok(LocaleWeekday::Sunday) | Err(_) => Weekday::Sun,
    };
    let title = value.and_then(|date| {
        formatter
            .format_date(
                CivilDate {
                    year: date.year(),
                    month: date.month() as u8,
                    day: date.day() as u8,
                },
                DateLength::Short,
            )
            .ok()
    });
    (week_start, title)
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DatePickerLabel {
    Literal(SharedString),
    Message {
        id: SharedString,
        fallback: SharedString,
    },
}

fn resolved_message_label(
    id: &str,
    localized: String,
    fallback: &SharedString,
) -> Option<SharedString> {
    if !localized.trim().is_empty() && localized.trim() != id.trim() {
        Some(SharedString::from(localized))
    } else {
        (!fallback.trim().is_empty()).then(|| fallback.clone())
    }
}

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

fn year_range_around(date: NaiveDate) -> (i32, i32) {
    (
        (date.year() - 50).max(NaiveDate::MIN.year()),
        (date.year() + 51).min(NaiveDate::MAX.year() + 1),
    )
}

struct RetainedDatePicker {
    state: gpui::Entity<DatePickerState>,
    writer: Rc<RefCell<Option<EventWriter<Option<NaiveDate>, gpui::App>>>>,
}

/// A controlled optional date picker whose calendar state survives body rebuilds.
pub struct DatePickerView {
    label: SharedString,
    localized_label_key: Option<SharedString>,
    accessibility_label: Option<DatePickerLabel>,
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
            localized_label_key: None,
            accessibility_label: None,
            value,
            id: None,
            disabled: false,
            cleanable: true,
            number_of_months: 1,
            appearance: true,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "DatePicker identity must not be blank"
        );
        self.id = Some(id);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Resolves the visible placeholder from the active catalog, falling back to the constructor label.
    pub fn localized_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let id = message_id.into();
        assert!(
            !id.trim().is_empty(),
            "DatePicker label localization key must not be blank"
        );
        self.localized_label_key = Some(id);
        self
    }

    fn visible_label(&self, localized_message: impl FnOnce(&str) -> String) -> SharedString {
        self.localized_label_key.as_ref().map_or_else(
            || self.label.clone(),
            |id| {
                resolved_message_label(id.as_ref(), localized_message(id.as_ref()), &self.label)
                    .unwrap_or_else(|| self.label.clone())
            },
        )
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

    /// Set a stable accessible name for the date-picker trigger.
    pub fn accessibility_label(mut self, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "DatePicker accessible name must not be blank"
        );
        self.accessibility_label = Some(DatePickerLabel::Literal(label));
        self
    }

    /// Resolve the trigger name from the active RustUI message catalog.
    pub fn localized_accessibility_label(mut self, message_id: impl Into<SharedString>) -> Self {
        let message_id = message_id.into();
        assert!(
            !message_id.trim().is_empty(),
            "DatePicker accessibility localization key must not be blank"
        );
        let fallback = match &self.accessibility_label {
            Some(DatePickerLabel::Literal(label))
            | Some(DatePickerLabel::Message {
                fallback: label, ..
            }) => label.clone(),
            None => self.label.clone(),
        };
        self.accessibility_label = Some(DatePickerLabel::Message {
            id: message_id,
            fallback,
        });
        self
    }

    fn trigger_accessibility_label(
        &self,
        localized_message: impl FnOnce(&str) -> String,
    ) -> Option<SharedString> {
        match &self.accessibility_label {
            Some(DatePickerLabel::Literal(label)) => Some(label.clone()),
            Some(DatePickerLabel::Message { id, fallback }) => {
                resolved_message_label(id.as_ref(), localized_message(id.as_ref()), fallback)
            }
            None => {
                let label = self.visible_label(localized_message);
                (!label.trim().is_empty()).then_some(label)
            }
        }
    }

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        self.identity_with(|base| context.next_implicit_retained_identity(base))
    }

    fn identity_with(&self, allocate_implicit: impl FnOnce(Identity) -> Identity) -> Identity {
        self.id
            .clone()
            // The visible placeholder can change with locale without changing the picker.
            .unwrap_or_else(|| allocate_implicit(Identity::from("date-picker")))
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
        let identity = self.identity(context);
        let initial_value = self.value.value();
        let (bridge, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let state = cx.new(|cx| {
                let mut state = DatePickerState::new(window, cx);
                state.set_date(CalendarDate::Single(initial_value), window, cx);
                if let Some(date) = initial_value {
                    state.set_year_range(year_range_around(date), cx);
                }
                state
            });
            RetainedDatePicker {
                state,
                writer: Rc::new(RefCell::new(None)),
            }
        });

        let state = bridge.read(context.app()).state.clone();
        let current = self.value.value();
        let (week_start, display_value) = localized_calendar(context.locale(), current);
        if state.read(context.app()).week_start() != week_start {
            context.update_entity(&state, move |state, _window, cx| {
                state.set_week_start(week_start, cx);
            });
        }
        let retained = match state.read(context.app()).date() {
            CalendarDate::Single(value) => value,
            CalendarDate::Range(_, _) => unreachable!("RustUI DatePicker retains one date"),
        };
        if retained != current {
            context.update_entity(&state, move |state, window, cx| {
                state.set_date(CalendarDate::Single(current), window, cx);
                if let Some(date) = current {
                    state.set_year_range(year_range_around(date), cx);
                }
            });
        }

        let writer = bridge.read(context.app()).writer.clone();
        replace_event_writer(&writer, context.binding_writer(self.value.clone()));

        if created {
            let current_writer = writer.clone();
            context.subscribe_retained(
                &identity,
                &state,
                move |_state, event: &DatePickerEvent, cx| {
                    let DatePickerEvent::Change(value) = event;
                    match *value {
                        DateTime::Single(value) => write_with_current_writer(
                            &current_writer,
                            value.map(|value| value.date()),
                            cx,
                        ),
                        DateTime::Range(_, _) => {
                            unreachable!("RustUI DatePicker retains one date")
                        }
                    }
                },
            );
        }

        let mut picker = GpuiDatePicker::new(&state)
            .placeholder(self.visible_label(|id| context.localized_message(id)))
            .right_to_left(context.layout_direction() == LayoutDirection::RightToLeft)
            .cleanable(self.cleanable)
            .number_of_months(self.number_of_months)
            .appearance(self.appearance)
            .disabled(self.disabled);
        if let Some(display_value) = display_value {
            picker = picker.display_value(display_value);
        }
        if let Some(label) = self.trigger_accessibility_label(|id| context.localized_message(id)) {
            picker = picker.accessibility_label(label);
        }
        picker.into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_uses_locale_week_rules_and_date_format() {
        let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
        let french = Locale::parse("fr-FR").unwrap();
        let arabic = Locale::parse("ar-EG").unwrap();
        let english = Locale::parse("en-US").unwrap();
        let (fr_week, fr_date) = localized_calendar(Some(&french), Some(date));
        let (ar_week, ar_date) = localized_calendar(Some(&arabic), Some(date));
        let (en_week, en_date) = localized_calendar(Some(&english), Some(date));
        assert_eq!(
            (fr_week, ar_week, en_week),
            (Weekday::Mon, Weekday::Sat, Weekday::Sun)
        );
        let (fr_date, ar_date, en_date) = (fr_date.unwrap(), ar_date.unwrap(), en_date.unwrap());
        assert_ne!(fr_date, en_date);
        assert_ne!(ar_date, en_date);
        assert_eq!(
            localized_calendar(Some(&french), None),
            (Weekday::Mon, None)
        );
        assert_eq!(localized_calendar(None, Some(date)), (Weekday::Sun, None));
    }

    #[gpui::test]
    fn keyboard_can_select_the_next_day(cx: &mut gpui::TestAppContext) {
        use gpui::Focusable as _;

        struct Owner {
            date: Option<NaiveDate>,
        }
        impl View for Owner {
            fn body(&self) -> impl View {
                DatePicker(
                    "Choose a date",
                    Binding::new(
                        self.date,
                        |owner: &Owner| owner.date,
                        |owner, value| {
                            owner.date = value;
                        },
                    ),
                )
                .id("keyboard-date")
            }
        }

        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let date = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
        let (owner, window) = cx.add_window_view(|_, app| {
            crate::runtime::ComponentHost::new(Owner { date: Some(date) }, app)
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            let retained = owner
                .read(app)
                .retained_entity_for_test::<RetainedDatePicker>(
                    vec![],
                    Identity::from("keyboard-date"),
                )
                .unwrap();
            let state = retained.read(app).state.clone();
            state.read(app).focus_handle(app).focus(window, app);
        });
        window.simulate_keystrokes("enter");
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().date, Some(date));
        });
        window.simulate_keystrokes("escape");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(owner.read(app).component().date, Some(date));
        });
        window.simulate_keystrokes("enter");
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("right");
        window.update(|window, app| {
            let _ = window.draw(app);
        });
        window.simulate_keystrokes("enter");
        window.update(|window, app| {
            let _ = window.draw(app);
            assert_eq!(
                owner.read(app).component().date,
                NaiveDate::from_ymd_opt(2026, 10, 29)
            );
        });
        for (limit, direction) in [(NaiveDate::MIN, "left"), (NaiveDate::MAX, "right")] {
            window.update(|window, app| {
                owner.update(app, |host, cx| {
                    host.component_mut().date = Some(limit);
                    cx.notify();
                });
                let _ = window.draw(app);
            });
            window.simulate_keystrokes("enter");
            window.update(|window, app| {
                let _ = window.draw(app);
            });
            window.simulate_keystrokes(direction);
            window.update(|window, app| {
                let _ = window.draw(app);
            });
            window.simulate_keystrokes("enter");
            window.update(|window, app| {
                let _ = window.draw(app);
                assert_eq!(owner.read(app).component().date, Some(limit));
            });
        }
    }

    #[gpui::test]
    fn keyboard_can_move_past_a_disabled_date(cx: &mut gpui::TestAppContext) {
        use gpui::{AppContext as _, Context, Entity, Focusable as _, Render, Window};
        use gpui_component::calendar::Matcher;

        struct Probe {
            state: Entity<DatePickerState>,
        }
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                GpuiDatePicker::new(&self.state).right_to_left(true)
            }
        }
        let initial = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
        let disabled = initial.succ_opt().unwrap();
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (probe, window) = cx.add_window_view(|window, cx| {
            let state = cx.new(|cx| {
                DatePickerState::new(window, cx)
                    .disabled_matcher(Matcher::custom(move |day| *day == disabled))
            });
            state.update(cx, |state, cx| state.set_date(initial, window, cx));
            Probe { state }
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            probe
                .read(app)
                .state
                .read(app)
                .focus_handle(app)
                .focus(window, app);
        });
        for key in ["enter", "right", "enter"] {
            window.simulate_keystrokes(key);
            window.update(|window, app| {
                let _ = window.draw(app);
            });
        }
        window.update(|_, app| {
            assert_eq!(
                probe.read(app).state.read(app).date(),
                CalendarDate::Single(Some(initial))
            );
        });
        for key in ["right", "enter"] {
            window.simulate_keystrokes(key);
            window.update(|window, app| {
                let _ = window.draw(app);
            });
        }
        window.update(|_, app| {
            assert_eq!(
                probe.read(app).state.read(app).date(),
                CalendarDate::Single(initial.succ_opt().unwrap().succ_opt())
            );
        });
    }

    #[gpui::test]
    fn keyboard_range_cancel_then_confirm(cx: &mut gpui::TestAppContext) {
        use gpui::{AppContext as _, Context, Entity, Focusable as _, Render, Window};

        struct Probe {
            state: Entity<DatePickerState>,
        }
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                GpuiDatePicker::new(&self.state)
            }
        }
        let start = NaiveDate::from_ymd_opt(2026, 10, 28).unwrap();
        let initial = CalendarDate::Range(Some(start), Some(start.succ_opt().unwrap()));
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (probe, window) = cx.add_window_view(|window, cx| {
            let state = cx.new(|cx| DatePickerState::range(window, cx));
            state.update(cx, |state, cx| state.set_date(initial, window, cx));
            Probe { state }
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            probe
                .read(app)
                .state
                .read(app)
                .focus_handle(app)
                .focus(window, app);
        });
        for key in ["enter", "right", "enter", "escape"] {
            window.simulate_keystrokes(key);
            window.update(|window, app| {
                let _ = window.draw(app);
            });
        }
        window.update(|_, app| assert_eq!(probe.read(app).state.read(app).date(), initial));
        for key in ["enter", "right", "enter", "right", "enter"] {
            window.simulate_keystrokes(key);
            window.update(|window, app| {
                let _ = window.draw(app);
            });
        }
        window.update(|_, app| {
            assert_eq!(
                probe.read(app).state.read(app).date(),
                CalendarDate::Range(start.succ_opt(), start.succ_opt().unwrap().succ_opt())
            )
        });
    }

    #[gpui::test]
    fn keyboard_date_selection_preserves_time(cx: &mut gpui::TestAppContext) {
        use gpui::{AppContext as _, Context, Entity, Focusable as _, Render, Window};
        use gpui_component::time_field::TimePrecision;

        struct Probe {
            state: Entity<DatePickerState>,
        }
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                GpuiDatePicker::new(&self.state)
            }
        }
        let initial = NaiveDate::from_ymd_opt(2026, 10, 28)
            .unwrap()
            .and_hms_opt(13, 45, 0)
            .unwrap();
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (probe, window) = cx.add_window_view(|window, cx| {
            let state =
                cx.new(|cx| DatePickerState::new(window, cx).time_precision(TimePrecision::Minute));
            state.update(cx, |state, cx| state.set_date_time(initial, window, cx));
            Probe { state }
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            probe
                .read(app)
                .state
                .read(app)
                .focus_handle(app)
                .focus(window, app);
        });
        for key in ["enter", "right", "enter"] {
            window.simulate_keystrokes(key);
            window.update(|window, app| {
                let _ = window.draw(app);
            });
        }
        window.update(|_, app| {
            assert_eq!(
                probe.read(app).state.read(app).date_time(),
                DateTime::Single(Some(initial + chrono::Duration::days(1)))
            )
        });
    }

    fn binding() -> Binding<Option<NaiveDate>> {
        Binding::new(
            None,
            |value: &Option<NaiveDate>| *value,
            |value, next| *value = next,
        )
    }

    #[test]
    fn zero_month_count_is_clamped_to_one() {
        let date = DatePicker("Choose a date", binding()).number_of_months(0);

        assert_eq!(date.number_of_months, 1);
    }

    #[test]
    fn selected_date_year_is_in_the_calendar_year_range() {
        let date = NaiveDate::from_ymd_opt(1900, 6, 15).unwrap();

        assert_eq!(year_range_around(date), (1850, 1951));
    }

    #[test]
    fn year_range_stays_within_chronos_representable_years() {
        for date in [NaiveDate::MIN, NaiveDate::MAX] {
            let (start, end) = year_range_around(date);
            assert!(start >= NaiveDate::MIN.year());
            assert!(end <= NaiveDate::MAX.year() + 1);
            assert!((start..end).contains(&date.year()));
        }
    }

    #[gpui::test]
    fn public_date_picker_retains_owner_updates_at_both_limits(cx: &mut gpui::TestAppContext) {
        use crate::runtime::component_host::ComponentHost;

        struct Owner {
            date: Option<NaiveDate>,
        }
        impl View for Owner {
            fn body(&self) -> impl View {
                DatePicker(
                    "Choose date",
                    Binding::new(
                        self.date,
                        |owner: &Owner| owner.date,
                        |owner, value| {
                            owner.date = value;
                        },
                    ),
                )
                .id("boundary-picker")
            }
        }
        let theme = crate::theme::parse_theme(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../themes/macos27.json"
        )))
        .unwrap();
        cx.update(|app| {
            gpui_component::init(app);
            crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
                .unwrap();
        });
        let (owner, window) = cx.add_window_view(|_, cx| {
            ComponentHost::new(
                Owner {
                    date: Some(NaiveDate::MIN),
                },
                cx,
            )
        });
        let mut retained_id = None;
        for date in [NaiveDate::MIN, NaiveDate::MAX, NaiveDate::MIN] {
            window.update(|window, app| {
                owner.update(app, |host, cx| {
                    host.component_mut().date = Some(date);
                    cx.notify();
                });
                let _ = window.draw(app);
                let tree: serde_json::Value =
                    serde_json::from_str(&window.debug_a11y_tree_json().expect("test window tree"))
                        .unwrap();
                let picker = tree["nodes"]
                    .as_object()
                    .unwrap()
                    .values()
                    .find(|node| node["aria"]["role"] == "ComboBox")
                    .expect("date picker trigger");
                assert_eq!(picker["aria"]["value"], date.format("%Y/%m/%d").to_string());
                let id = picker["accesskit_id"].as_str().unwrap().to_owned();
                assert_eq!(retained_id.get_or_insert(id.clone()), &id);
                assert_eq!(owner.read(app).component().date, Some(date));
            });
        }
    }

    #[gpui::test]
    fn calendar_renders_and_navigates_at_both_date_limits(cx: &mut gpui::TestAppContext) {
        use gpui::{AppContext as _, Context, Entity, IntoElement as _, Render, Window};
        use gpui_base::{Calendar, CalendarItemKind, CalendarState, Date};

        struct Probe {
            calendar: Entity<CalendarState>,
            items: Rc<RefCell<Vec<(CalendarItemKind, bool)>>>,
            first_day: Weekday,
        }
        impl Render for Probe {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
                self.items.borrow_mut().clear();
                let items = self.items.clone();
                Calendar::new("date-limits", &self.calendar)
                    .number_of_months(2)
                    .first_day_of_week(self.first_day)
                    .item(move |item, state, _, _| {
                        items.borrow_mut().push((state.kind(), state.is_disabled()));
                        item.into_any_element()
                    })
            }
        }
        let (probe, window) = cx.add_window_view(|window, cx| Probe {
            calendar: cx.new(|cx| CalendarState::new(window, cx)),
            items: Rc::new(RefCell::new(Vec::new())),
            first_day: Weekday::Sun,
        });
        for date in [NaiveDate::MIN, NaiveDate::MAX] {
            window.update(|window, app| {
                let calendar = probe.read(app).calendar.clone();
                calendar.update(app, |calendar, cx| {
                    calendar.set_date(Date::Single(Some(date)), window, cx);
                    if date == NaiveDate::MIN {
                        calendar.prev_month();
                    } else {
                        calendar.next_month();
                    }
                    assert_eq!(
                        (calendar.current_year(), calendar.current_month()),
                        (date.year(), date.month() as u8)
                    );
                    assert!(calendar.days().iter().flatten().any(|day| *day == date));
                });
                let _ = window.draw(app);
                let items = probe.read(app).items.borrow();
                let disabled_kind = if date == NaiveDate::MIN {
                    CalendarItemKind::Previous
                } else {
                    CalendarItemKind::Next
                };
                assert!(
                    items
                        .iter()
                        .any(|(kind, disabled)| *kind == disabled_kind && *disabled)
                );
            });
        }
        for first_day in [Weekday::Mon, Weekday::Sat] {
            window.update(|window, app| {
                probe.update(app, |probe, cx| {
                    probe.first_day = first_day;
                    cx.notify();
                });
                let _ = window.draw(app);
                let items = probe.read(app).items.borrow();
                assert!(
                    items
                        .iter()
                        .any(|(kind, disabled)| *kind == CalendarItemKind::Next && *disabled)
                );
            });
        }
    }

    #[test]
    #[should_panic(expected = "DatePicker identity must not be blank")]
    fn rejects_blank_explicit_identity() {
        DatePicker("Choose a date", binding()).id(" \t ");
    }

    #[test]
    fn preserves_nonblank_explicit_identity() {
        let date = DatePicker("Choose a date", binding()).id(" start-date ");

        assert_eq!(
            date.identity_with(|_| panic!("explicit identity must not allocate a fallback"))
                .as_str(),
            " start-date "
        );
    }

    #[test]
    fn implicit_identity_distinguishes_same_label_siblings_and_is_stable_by_slot() {
        let first_binding = binding();
        let second_binding = binding();
        let first = DatePicker("Choose a date", first_binding);
        let second = DatePicker("Choose a date", second_binding);
        let allocate_at_slot = |slot| {
            move |base: Identity| Identity::from(format!("owner:implicit-retained:{slot}:{base}"))
        };

        let first_sibling = first.identity_with(allocate_at_slot(0));
        let second_sibling = second.identity_with(allocate_at_slot(1));
        let first_after_rerender = first.identity_with(allocate_at_slot(0));
        let first_after_locale_change =
            DatePicker("Choisir une date", binding()).identity_with(allocate_at_slot(0));

        assert_ne!(first_sibling, second_sibling);
        assert_eq!(first_sibling, first_after_rerender);
        assert_eq!(first_sibling, first_after_locale_change);
    }

    #[test]
    fn trigger_name_defaults_to_visible_label_and_stays_stable_for_selected_values() {
        let date = DatePicker("Choose a date", binding());

        assert_eq!(
            date.trigger_accessibility_label(|id| id.to_owned())
                .as_deref(),
            Some("Choose a date")
        );
    }

    #[test]
    fn localized_visible_label_drives_default_trigger_name_and_falls_back() {
        let date = DatePicker("Choose a date", binding()).localized_label("date.choose");
        assert_eq!(
            date.visible_label(|_| "Choisir une date".into()).as_ref(),
            "Choisir une date"
        );
        assert_eq!(
            date.trigger_accessibility_label(|_| "Choisir une date".into())
                .as_deref(),
            Some("Choisir une date")
        );
        assert_eq!(date.visible_label(str::to_owned).as_ref(), "Choose a date");
        assert_eq!(
            date.trigger_accessibility_label(str::to_owned).as_deref(),
            Some("Choose a date")
        );
        let overridden = date.accessibility_label("Start date");
        assert_eq!(
            overridden
                .trigger_accessibility_label(|_| "Choisir une date".into())
                .as_deref(),
            Some("Start date")
        );
    }

    #[test]
    #[should_panic(expected = "DatePicker label localization key must not be blank")]
    fn rejects_blank_visible_label_localization_key() {
        DatePicker("Choose a date", binding()).localized_label(" \t ");
    }

    #[test]
    fn explicit_accessible_name_overrides_the_visible_placeholder() {
        let date = DatePicker("Choose a date", binding()).accessibility_label("Start date");

        assert_eq!(
            date.trigger_accessibility_label(|id| id.to_owned())
                .as_deref(),
            Some("Start date")
        );
    }

    #[test]
    fn blank_visible_label_has_no_implicit_accessible_name() {
        let date = DatePicker(" \t ", binding());

        assert_eq!(date.trigger_accessibility_label(|id| id.to_owned()), None);
    }

    #[test]
    #[should_panic(expected = "DatePicker accessible name must not be blank")]
    fn rejects_blank_explicit_accessible_name() {
        DatePicker("Choose a date", binding()).accessibility_label(" \t ");
    }

    #[test]
    fn localized_accessible_name_uses_the_visible_label_as_fallback() {
        let date = DatePicker("Choose a date", binding())
            .localized_accessibility_label("date_picker.choose");

        assert!(matches!(
            &date.accessibility_label,
            Some(DatePickerLabel::Message { id, fallback })
                if id.as_ref() == "date_picker.choose" && fallback.as_ref() == "Choose a date"
        ));
        assert_eq!(
            date.trigger_accessibility_label(|_| "Choisir une date".to_owned())
                .as_deref(),
            Some("Choisir une date")
        );
        assert_eq!(
            date.trigger_accessibility_label(|_| " \t".to_owned())
                .as_deref(),
            Some("Choose a date")
        );
        assert_eq!(
            date.trigger_accessibility_label(|id| id.to_owned())
                .as_deref(),
            Some("Choose a date")
        );
    }

    #[test]
    fn localized_accessible_name_preserves_explicit_fallback_and_repeat_changes_key() {
        let date = DatePicker("Choose a date", binding())
            .accessibility_label("Start date")
            .localized_accessibility_label("date_picker.start")
            .localized_accessibility_label("date_picker.begin");

        assert!(matches!(
            &date.accessibility_label,
            Some(DatePickerLabel::Message { id, fallback })
                if id.as_ref() == "date_picker.begin" && fallback.as_ref() == "Start date"
        ));
        assert_eq!(
            date.trigger_accessibility_label(|id| id.to_owned())
                .as_deref(),
            Some("Start date")
        );
    }

    #[test]
    fn blank_localized_result_uses_only_a_nonblank_fallback() {
        let date =
            DatePicker(" \t ", binding()).localized_accessibility_label("date_picker.choose");

        assert_eq!(date.trigger_accessibility_label(|_| " \t".to_owned()), None);
    }

    #[test]
    #[should_panic(expected = "DatePicker accessibility localization key must not be blank")]
    fn rejects_blank_accessibility_localization_key() {
        DatePicker("Choose a date", binding()).localized_accessibility_label(" \t ");
    }

    #[test]
    fn padded_untranslated_accessibility_key_uses_fallback() {
        let key = "date_picker.choose";
        let fallback: SharedString = "Choose a date".into();

        assert_eq!(
            resolved_message_label(key, format!(" \t{key}\n"), &fallback),
            Some(fallback)
        );
    }

    #[test]
    fn retained_events_use_the_most_recent_owner_binding() {
        let writer = RefCell::new(None);
        let mut writes = Vec::new();
        replace_event_writer(
            &writer,
            Rc::new(|value: bool, writes: &mut Vec<&'static str>| {
                if value {
                    writes.push("old");
                }
            }),
        );
        replace_event_writer(
            &writer,
            Rc::new(|value: bool, writes: &mut Vec<&'static str>| {
                if value {
                    writes.push("current");
                }
            }),
        );

        write_with_current_writer(&writer, true, &mut writes);

        assert_eq!(writes, ["current"]);
    }
}
