use rustui::{component, prelude::*};

#[component]
struct ActionOwner {}

impl ActionOwner {
    fn increment(&mut self) {}
}

impl View for ActionOwner {
    fn body(&self) -> impl View {
        VStack! {
            Button("Vertical action", increment)
            HStack! {
                Button("Horizontal action", increment)
            }
            Text("Sibling")
        }
    }
}

#[test]
fn stack_macros_rewrite_bare_button_handlers_to_owner_actions() {
    let _view = ActionOwner::default().body();
}
