use rustui::prelude::*;

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
    RustUI::new()
        .window_size(420.0, 240.0)
        .title("RustUI source consumer")
        .run(Counter::default());
}
