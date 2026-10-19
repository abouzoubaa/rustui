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
        VStack! {
            Text(format!("Count: {}", self.count()))
            HStack! {
                Button("Increment", increment)
            }
        }
    }
}

fn main() {
    let _counter = Counter::default();
}
