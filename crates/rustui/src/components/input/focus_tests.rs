use crate::{prelude::*, runtime::ComponentHost};
use rustui_core::{Action, Binding};

struct Controls {
    before: usize,
    after: usize,
    writes: usize,
    choices: [bool; 3],
    value: f32,
    selection: String,
    checkbox_disabled: bool,
    radio_disabled: bool,
    toggle_disabled: bool,
    slider_disabled: bool,
    picker_disabled: bool,
}

fn read_choice(owner: &Controls, index: &usize) -> bool {
    owner.choices[*index]
}

fn write_choice(owner: &mut Controls, index: &usize, value: bool) -> bool {
    owner.choices[*index] = value;
    owner.writes += 1;
    true
}

impl Controls {
    fn choice(&self, index: usize) -> Binding<bool> {
        Binding::new_with(self.choices[index], index, read_choice, write_choice)
    }
}

impl View for Controls {
    fn body(&self) -> impl View {
        VStack::new()
            .child(
                Button(
                    "Before",
                    Action::new(|owner: &mut Controls| owner.before += 1),
                )
                .id("before"),
            )
            .child(
                Button(
                    "Disabled button",
                    Action::new(|owner: &mut Controls| owner.writes += 1),
                )
                .disabled(true)
                .id("disabled-button"),
            )
            .child(
                Checkbox("Disabled checkbox", self.choice(0))
                    .disabled(self.checkbox_disabled)
                    .id("checkbox"),
            )
            .child(
                Radio("Disabled radio", self.choice(1))
                    .disabled(self.radio_disabled)
                    .id("radio"),
            )
            .child(
                Toggle("Disabled toggle", self.choice(2))
                    .disabled(self.toggle_disabled)
                    .id("toggle"),
            )
            .child(
                Slider(
                    Binding::new(
                        self.value,
                        |owner: &Controls| owner.value,
                        |owner, value| {
                            owner.value = value;
                            owner.writes += 1;
                        },
                    ),
                    0.0..=100.0,
                )
                .disabled(self.slider_disabled)
                .id("slider"),
            )
            .child(
                Picker(
                    "Choice",
                    Binding::new(
                        self.selection.clone(),
                        |owner: &Controls| owner.selection.clone(),
                        |owner, value| {
                            owner.selection = value;
                            owner.writes += 1;
                        },
                    ),
                    ["One", "Two"],
                )
                .disabled(self.picker_disabled)
                .id("picker"),
            )
            .child(
                Button(
                    "After",
                    Action::new(|owner: &mut Controls| owner.after += 1),
                )
                .id("after"),
            )
    }
}

fn press_space(window: &mut gpui::VisualTestContext) {
    window.update(|window, app| window.draw(app).clear(app));
    let keystroke = gpui::Keystroke::parse("space").unwrap();
    window.simulate_event(gpui::KeyDownEvent {
        keystroke: keystroke.clone(),
        is_held: false,
        prefer_character_input: false,
    });
    window.simulate_event(gpui::KeyUpEvent { keystroke });
}

#[gpui::test]
fn disabling_a_focused_checkbox_skips_disabled_controls_on_tab(cx: &mut gpui::TestAppContext) {
    use gpui::AppContext as _;

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
    let owner_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let saved_owner = owner_slot.clone();
    let (_, window) = cx.add_window_view(|window, app| {
        let owner = app.new(|app| {
            ComponentHost::new(
                Controls {
                    before: 0,
                    after: 0,
                    writes: 0,
                    choices: [false; 3],
                    value: 50.0,
                    selection: "One".to_owned(),
                    checkbox_disabled: false,
                    radio_disabled: true,
                    toggle_disabled: true,
                    slider_disabled: true,
                    picker_disabled: true,
                },
                app,
            )
        });
        *saved_owner.borrow_mut() = Some(owner.clone());
        gpui_component::Root::new(owner, window, app)
    });
    let owner = owner_slot.borrow().clone().unwrap();

    window.update(|window, app| {
        let _ = window.draw(app);
        window.focus_next(app);
    });
    press_space(window);
    window.update(|window, app| {
        assert_eq!(owner.read(app).component().before, 1);
        window.focus_next(app);
        owner.update(app, |host, cx| {
            host.component_mut().checkbox_disabled = true;
            cx.notify();
        });
        let _ = window.draw(app);
    });
    window.update(|window, app| {
        assert!(
            window.focused(app).is_some(),
            "disabling the focused Checkbox must transfer focus to its retained parent"
        );
    });
    window.simulate_keystrokes("tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().after,
            1,
            "Tab must reach the next enabled control after disabling the focused checkbox"
        );
        assert_eq!(owner.read(app).component().before, 1);
    });
    window.simulate_keystrokes("shift-tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().before,
            2,
            "Shift-Tab must return from the next enabled control"
        );
    });
}

#[gpui::test]
fn disabling_a_focused_toggle_skips_disabled_controls_on_tab(cx: &mut gpui::TestAppContext) {
    use gpui::AppContext as _;

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
    let owner_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let saved_owner = owner_slot.clone();
    let (_, window) = cx.add_window_view(|window, app| {
        let owner = app.new(|app| {
            ComponentHost::new(
                Controls {
                    before: 0,
                    after: 0,
                    writes: 0,
                    choices: [false; 3],
                    value: 50.0,
                    selection: "One".to_owned(),
                    checkbox_disabled: true,
                    radio_disabled: true,
                    toggle_disabled: false,
                    slider_disabled: true,
                    picker_disabled: true,
                },
                app,
            )
        });
        *saved_owner.borrow_mut() = Some(owner.clone());
        gpui_component::Root::new(owner, window, app)
    });
    let owner = owner_slot.borrow().clone().unwrap();

    window.update(|window, app| {
        let _ = window.draw(app);
        window.focus_next(app);
    });
    window.simulate_keystrokes("tab");
    window.update(|window, app| {
        owner.update(app, |host, cx| {
            host.component_mut().toggle_disabled = true;
            cx.notify();
        });
        let _ = window.draw(app);
        assert!(
            window.focused(app).is_some(),
            "disabling the focused Toggle must transfer focus to its retained parent"
        );
    });
    window.simulate_keystrokes("tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().after,
            1,
            "Tab must reach the next enabled control after disabling the focused Toggle"
        );
    });
    window.simulate_keystrokes("shift-tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().before, 1);
    });
}

#[gpui::test]
fn disabling_a_focused_radio_skips_disabled_controls_on_tab(cx: &mut gpui::TestAppContext) {
    use gpui::AppContext as _;

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
    let owner_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let saved_owner = owner_slot.clone();
    let (_, window) = cx.add_window_view(|window, app| {
        let owner = app.new(|app| {
            ComponentHost::new(
                Controls {
                    before: 0,
                    after: 0,
                    writes: 0,
                    choices: [false; 3],
                    value: 50.0,
                    selection: "One".to_owned(),
                    checkbox_disabled: true,
                    radio_disabled: false,
                    toggle_disabled: true,
                    slider_disabled: true,
                    picker_disabled: true,
                },
                app,
            )
        });
        *saved_owner.borrow_mut() = Some(owner.clone());
        gpui_component::Root::new(owner, window, app)
    });
    let owner = owner_slot.borrow().clone().unwrap();

    window.update(|window, app| {
        let _ = window.draw(app);
        window.focus_next(app);
    });
    window.simulate_keystrokes("tab");
    window.update(|window, app| {
        owner.update(app, |host, cx| {
            host.component_mut().radio_disabled = true;
            cx.notify();
        });
        let _ = window.draw(app);
        assert!(
            window.focused(app).is_some(),
            "disabling the focused Radio must transfer focus to its retained parent"
        );
    });
    window.simulate_keystrokes("tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().after,
            1,
            "Tab must reach the next enabled control after disabling the focused Radio"
        );
    });
    window.simulate_keystrokes("shift-tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().before, 1);
    });
}

#[gpui::test]
fn disabling_a_focused_slider_skips_it_on_tab(cx: &mut gpui::TestAppContext) {
    use gpui::AppContext as _;

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
    let owner_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let saved_owner = owner_slot.clone();
    let (_, window) = cx.add_window_view(|window, app| {
        let owner = app.new(|app| {
            ComponentHost::new(
                Controls {
                    before: 0,
                    after: 0,
                    writes: 0,
                    choices: [false; 3],
                    value: 50.0,
                    selection: "One".to_owned(),
                    checkbox_disabled: true,
                    radio_disabled: true,
                    toggle_disabled: true,
                    slider_disabled: false,
                    picker_disabled: true,
                },
                app,
            )
        });
        *saved_owner.borrow_mut() = Some(owner.clone());
        gpui_component::Root::new(owner, window, app)
    });
    let owner = owner_slot.borrow().clone().unwrap();

    window.update(|window, app| {
        let _ = window.draw(app);
        window.focus_next(app);
    });
    window.simulate_keystrokes("tab");
    window.update(|window, app| {
        owner.update(app, |host, cx| {
            host.component_mut().slider_disabled = true;
            cx.notify();
        });
        let _ = window.draw(app);
        assert!(
            window.focused(app).is_some(),
            "disabling the focused Slider must transfer focus to its retained parent"
        );
    });
    window.simulate_keystrokes("tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().after,
            1,
            "Tab must reach the next enabled control after disabling the focused Slider"
        );
    });
}

#[gpui::test]
fn disabling_a_focused_picker_skips_it_on_tab(cx: &mut gpui::TestAppContext) {
    use gpui::AppContext as _;

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
    let owner_slot = std::rc::Rc::new(std::cell::RefCell::new(None));
    let saved_owner = owner_slot.clone();
    let (_, window) = cx.add_window_view(|window, app| {
        let owner = app.new(|app| {
            ComponentHost::new(
                Controls {
                    before: 0,
                    after: 0,
                    writes: 0,
                    choices: [false; 3],
                    value: 50.0,
                    selection: "One".to_owned(),
                    checkbox_disabled: true,
                    radio_disabled: true,
                    toggle_disabled: true,
                    slider_disabled: true,
                    picker_disabled: false,
                },
                app,
            )
        });
        *saved_owner.borrow_mut() = Some(owner.clone());
        gpui_component::Root::new(owner, window, app)
    });
    let owner = owner_slot.borrow().clone().unwrap();

    window.update(|window, app| {
        let _ = window.draw(app);
        window.focus_next(app);
    });
    window.simulate_keystrokes("tab");
    window.update(|window, app| {
        owner.update(app, |host, cx| {
            host.component_mut().picker_disabled = true;
            cx.notify();
        });
        let _ = window.draw(app);
        assert!(
            window.focused(app).is_some(),
            "disabling the focused Picker must transfer focus to its retained parent"
        );
    });
    window.simulate_keystrokes("tab");
    press_space(window);
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().after,
            1,
            "Tab must reach the next enabled control after disabling the focused Picker"
        );
    });
}

#[gpui::test]
fn disabled_controls_do_not_enter_forward_or_reverse_keyboard_traversal(
    cx: &mut gpui::TestAppContext,
) {
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
    let (owner, window) = cx.add_window_view(|_, app| {
        ComponentHost::new(
            Controls {
                before: 0,
                after: 0,
                writes: 0,
                choices: [false; 3],
                value: 50.0,
                selection: "One".to_owned(),
                checkbox_disabled: true,
                radio_disabled: true,
                toggle_disabled: true,
                slider_disabled: true,
                picker_disabled: true,
            },
            app,
        )
    });
    window.update(|window, app| {
        let _ = window.draw(app);
        window.focus_next(app);
        assert!(window.focused(app).is_some());
    });
    press_space(window);
    window.update(|window, app| {
        assert_eq!(owner.read(app).component().before, 1);
        window.focus_next(app);
    });
    press_space(window);
    window.update(|window, app| {
        assert_eq!(
            owner.read(app).component().after,
            1,
            "disabled controls must be skipped"
        );
        window.focus_prev(app);
    });
    press_space(window);
    window.update(|_, app| {
        let controls = owner.read(app).component();
        assert_eq!((controls.before, controls.after), (2, 1));
        assert_eq!(controls.writes, 0);
        assert_eq!(controls.choices, [false; 3]);
        assert_eq!(controls.value, 50.0);
    });
}
