use std::{cell::RefCell, rc::Rc};

use chrono::{Datelike, NaiveDate, Weekday};
use gpui::{
    AppContext as _, Context, Entity, InteractiveElement, IntoElement, Modifiers, MouseButton,
    ParentElement, Render, TestAppContext, VisualTestContext, Window, div,
};
use gpui_base::{CalendarState, Date};
use gpui_component::{
    calendar::Calendar,
    date_picker::{DatePicker, DatePickerState},
};

struct CalendarHost {
    calendar: Entity<CalendarState>,
    picker: Entity<DatePickerState>,
}

impl Render for CalendarHost {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .child(Calendar::new(&self.calendar).number_of_months(2))
            .child(
                div().debug_selector(|| "date-picker-control".into()).child(
                    DatePicker::new(&self.picker)
                        .accessibility_label("Due date")
                        .display_value("Selected date")
                        .right_to_left(true),
                ),
            )
    }
}

fn mount(
    cx: &mut TestAppContext,
) -> (
    &mut VisualTestContext,
    Entity<CalendarState>,
    Entity<DatePickerState>,
) {
    cx.update(gpui_component::init);
    let calendar_slot = Rc::new(RefCell::new(None));
    let picker_slot = Rc::new(RefCell::new(None));
    let calendar_capture = calendar_slot.clone();
    let picker_capture = picker_slot.clone();
    let (_, window) = cx.add_window_view(move |window, cx| {
        let calendar = cx.new(|cx| CalendarState::new(window, cx));
        calendar.update(cx, |state, cx| {
            state.set_date(
                Date::Single(Some(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap())),
                window,
                cx,
            );
            state.set_right_to_left(true);
        });
        let picker = cx.new(|cx| DatePickerState::new(window, cx));
        picker.update(cx, |state, cx| {
            state.set_date(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap(), window, cx);
            state.set_week_start(Weekday::Mon, cx);
        });
        *calendar_capture.borrow_mut() = Some(calendar.clone());
        *picker_capture.borrow_mut() = Some(picker.clone());
        CalendarHost { calendar, picker }
    });
    window.update(|window, cx| window.draw(cx).clear(cx));
    window.update(|window, cx| window.draw(cx).clear(cx));
    let calendar = calendar_slot.borrow_mut().take().unwrap();
    let picker = picker_slot.borrow_mut().take().unwrap();
    (window, calendar, picker)
}

#[gpui::test]
fn calendar_keyboard_cursor_stays_uncommitted_until_activation(cx: &mut TestAppContext) {
    let (window, calendar, _) = mount(cx);
    let initial = NaiveDate::from_ymd_opt(2026, 1, 15).unwrap();
    let target = NaiveDate::from_ymd_opt(2026, 1, 16).unwrap();
    window.update(|window, cx| {
        calendar.update(cx, |state, cx| {
            state.set_keyboard_cursor(Some(target), cx);
            assert_eq!(state.date().start(), Some(initial));
            assert_eq!(state.keyboard_cursor(), Some(target));
            assert!(state.activate_date(target, cx));
            assert_eq!(state.date().start(), Some(target));
        });
        window.draw(cx).clear(cx);
    });
}

#[gpui::test]
fn date_picker_arrow_cursor_activates_only_on_confirm(cx: &mut TestAppContext) {
    let (window, _, picker) = mount(cx);
    let bounds = window.debug_bounds("date-picker-control").unwrap();
    window.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::default());
    window.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::default());
    window.simulate_keystrokes("right");
    assert_eq!(
        window.update(|_, cx| picker.read(cx).date().start()),
        Some(NaiveDate::from_ymd_opt(2026, 1, 15).unwrap()),
        "arrow navigation should move the keyboard cursor without committing"
    );
    window.simulate_keystrokes("enter");
    assert_eq!(
        window.update(|_, cx| picker.read(cx).date().start()),
        Some(NaiveDate::from_ymd_opt(2026, 1, 14).unwrap()),
        "in RTL, confirm should commit the cursor moved left by the Right key"
    );
}

#[gpui::test]
fn rtl_multi_month_navigation_and_date_picker_accessibility_are_exposed(cx: &mut TestAppContext) {
    let (window, calendar, picker) = mount(cx);
    window.update(|_, cx| {
        assert!(calendar.read(cx).right_to_left());
        assert_eq!(picker.read(cx).week_start(), Weekday::Mon);
    });
    let tree = window.update(|window, _| window.debug_a11y_tree_json().unwrap());
    assert!(
        tree.contains("Due date"),
        "DatePicker label missing: {tree}"
    );
    assert!(
        tree.contains("Selected date"),
        "DatePicker display value missing: {tree}"
    );
}

#[gpui::test]
fn calendar_multi_month_count_stays_within_date_limits(cx: &mut TestAppContext) {
    let (window, calendar, _) = mount(cx);
    window.update(|window, cx| {
        calendar.update(cx, |state, cx| {
            state.set_date(Date::Single(Some(NaiveDate::MAX)), window, cx);
            state.set_number_of_months(2, window, cx);
            assert_eq!(state.number_of_months(), 1);
            state.set_date(Date::Single(Some(NaiveDate::MIN)), window, cx);
            assert_eq!(
                state.offset_year_month(usize::MAX),
                (NaiveDate::MAX.year(), 12)
            );
        });
        window.draw(cx).clear(cx);
    });
}
