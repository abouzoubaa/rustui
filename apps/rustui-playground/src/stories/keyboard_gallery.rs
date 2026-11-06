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
        let inside_text = self.inside_text_binding();
        let primary_hits = self.primary_hits();
        let optional_hits = self.optional_hits();
        let optional_enabled = self.optional_enabled();

        VStack::new()
            .spacing(10.0)
            .child(Text("Scoped keyboard shortcuts"))
            .child(Text("Focus a control inside the scope. Secondary-Shift-K increments the first count; Secondary-Shift-J increments the second count only while enabled. Secondary-Shift-A runs the composed AppAction and refreshes windows."))
            .child(OwnerActionReader(move |owner: OwnerAction<KeyboardGallery>| {
                KeyboardShortcuts(
                    "keyboard-gallery-scope",
                    VStack::new()
                        .spacing(8.0)
                        .child(TextField("Inside scope", inside_text.clone()).id("keyboard-inside-input"))
                        .child(Button("Focus here too", owner.action(Self::primary)).id("keyboard-inside-button"))
                        .child(Text(format!("Primary hits: {primary_hits} · Optional hits: {optional_hits}"))),
                )
                .shortcut("secondary-shift-k", owner.action(Self::primary))
                .shortcut_enabled(
                    "secondary-shift-j",
                    owner.action(Self::optional),
                    optional_enabled,
                )
                .shortcut(
                    "secondary-shift-a",
                    owner.action(Self::primary).then(AppAction::refresh_windows()),
                )
            }))
            .child(Button(
                if self.optional_enabled() { "Disable optional shortcut" } else { "Enable optional shortcut" },
                self.action(Self::toggle_optional),
            ).id("keyboard-toggle-optional"))
            .child(TextField("Outside scope", self.outside_text_binding()).id("keyboard-outside-input"))
            .child(Text("Typing in either field must still edit text. The outside field must not trigger either scoped shortcut."))
    }
}
