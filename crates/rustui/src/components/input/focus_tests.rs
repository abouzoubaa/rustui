use crate::{prelude::*, runtime::ComponentHost};
use rustui_core::{Action, Binding};

struct Controls {
    before: usize,
    after: usize,
    writes: usize,
    choices: [bool; 3],
    value: f32,
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
                    .disabled(true)
                    .id("checkbox"),
            )
            .child(
                Radio("Disabled radio", self.choice(1))
                    .disabled(true)
                    .id("radio"),
            )
            .child(
                Toggle("Disabled toggle", self.choice(2))
                    .disabled(true)
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
                .disabled(true)
                .id("slider"),
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
