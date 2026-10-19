//! Menu action and disabled-item acceptance using RustUI's public API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("menu-gallery", MenuGallery::default)
}

#[component]
struct MenuGallery {
    #[state(0)]
    count: u32,
    #[state(0)]
    disabled_hits: u32,
}

impl MenuGallery {
    fn increment(&mut self) {
        self.count += 1;
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn disabled_hit(&mut self) {
        self.disabled_hits += 1;
    }
}

impl View for MenuGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(10.0)
            .child(Text("Menu actions"))
            .child(Text("Open Actions, choose Increment or Reset, and verify Unavailable stays disabled."))
            .child(
                Menu(
                    format!("Actions: {} (disabled hits: {})", self.count(), self.disabled_hits()),
                    [
                        MenuItem::action("increment", "Increment", self.action(Self::increment)),
                        MenuItem::separator(),
                        MenuItem::action("reset", "Reset", self.action(Self::reset)),
                        MenuItem::action("disabled", "Unavailable", self.action(Self::disabled_hit))
                            .disabled(true),
                    ],
                )
                .id("menu-gallery-actions"),
            )
            .child(Text(format!("Count: {} · disabled activations: {}", self.count(), self.disabled_hits())))
            .child(Text("Menu(\"Actions\", [MenuItem::action(\"increment\", \"Increment\", self.action(Self::increment))]).id(\"actions\")").font_size(12.0))
    }
}
