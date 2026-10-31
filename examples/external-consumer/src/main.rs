use rustui::prelude::*;
use rustui::theme::{AppearanceMode, activate_theme, parse_theme};

#[component]
struct Counter {
    #[state(0)]
    count: i32,
}

impl Counter {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for Counter {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(12.0)
            .child(Text(format!("Count: {}", self.count())))
            .child(Button("Increment", self.action(Self::increment)))
    }
}

fn main() {
    let theme = parse_theme(include_str!("../../../themes/macos27.json"))
        .expect("bundled RustUI theme must be valid");
    RustUI::new()
        .window_size(420.0, 240.0)
        .title("RustUI source consumer")
        .on_launch(move |cx| {
            activate_theme(&theme, Some(AppearanceMode::Dark), cx)
                .expect("bundled RustUI theme must activate");
        })
        .run(Counter::default());
}
