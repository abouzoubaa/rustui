use rustui::{
    prelude::*,
    theme::{activate_theme, parse_theme},
};

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
        VStack! {
            Text(format!("Count: {}", self.count()))
            HStack! {
                Button("Increment", increment)
            }
        }
    }
}

fn main() {
    let theme =
        parse_theme(include_str!("../../../themes/macos27.json")).expect("valid RustUI theme");

    RustUI::new()
        .on_launch(move |cx| {
            activate_theme(&theme, None, cx).expect("RustUI theme activation succeeds");
        })
        .run(Counter::default());
}
