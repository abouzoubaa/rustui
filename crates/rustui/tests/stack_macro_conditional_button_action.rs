use rustui::{component, prelude::*};

#[component]
struct ConditionalActionOwner {
    visible: bool,
}

impl ConditionalActionOwner {
    fn activate(&mut self) {}
}

impl View for ConditionalActionOwner {
    fn body(&self) -> impl View {
        VStack! {
            if self.visible {
                Button("Vertical action", activate)
            } else {
                HStack! {
                    Button("Horizontal action", activate)
                }
            }
        }
    }
}

#[test]
fn conditional_stack_branches_rewrite_bare_button_actions_through_public_macros() {
    let _view = ConditionalActionOwner::default().body();
}
