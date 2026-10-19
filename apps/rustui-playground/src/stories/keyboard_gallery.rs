//! Focus-scoped shortcuts exercised through RustUI's public API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("keyboard-gallery", KeyboardGallery::default)
}

#[component]
struct KeyboardGallery {
    #[state(0)]
    primary_hits: u32,
    #[state(0)]
    optional_hits: u32,
    #[state(true)]
    optional_enabled: bool,
    #[state(String::new())]
    inside_text: String,
    #[state(String::new())]
    outside_text: String,
}

impl KeyboardGallery {
    fn primary(&mut self) {
        self.primary_hits += 1;
    }

    fn optional(&mut self) {
        self.optional_hits += 1;
    }

    fn toggle_optional(&mut self) {
        self.optional_enabled = !self.optional_enabled;
    }
}

impl View for KeyboardGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(10.0)
            .child(Text("Scoped keyboard shortcuts"))
            .child(Text("Focus a control inside the scope. Secondary-Shift-K increments the first count; Secondary-Shift-J increments the second count only while enabled."))
            .child(
                KeyboardShortcuts(
                    "keyboard-gallery-scope",
                    VStack::new()
                        .spacing(8.0)
                        .child(TextField("Inside scope", self.inside_text_binding()).id("keyboard-inside-input"))
                        .child(Button("Focus here too", self.action(Self::primary)).id("keyboard-inside-button"))
                        .child(Text(format!("Primary hits: {} · Optional hits: {}", self.primary_hits(), self.optional_hits()))),
                )
                .shortcut("secondary-shift-k", self.action(Self::primary))
                .shortcut_enabled(
                    "secondary-shift-j",
                    self.action(Self::optional),
                    self.optional_enabled(),
                ),
            )
            .child(Button(
                if self.optional_enabled() { "Disable optional shortcut" } else { "Enable optional shortcut" },
                self.action(Self::toggle_optional),
            ).id("keyboard-toggle-optional"))
            .child(TextField("Outside scope", self.outside_text_binding()).id("keyboard-outside-input"))
            .child(Text("Typing in either field must still edit text. The outside field must not trigger either scoped shortcut."))
    }
}
