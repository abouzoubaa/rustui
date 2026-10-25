use rustui::{
    HStack, VStack,
    components::basic::text::Text,
};

#[test]
fn child_attributes_gate_generated_stack_builder_calls() {
    let _vertical = VStack! {
        #[cfg(any())]
        MissingView()
        Text("sibling")
    };

    let _horizontal = HStack! {
        #[cfg_attr(all(), cfg(any()))]
        MissingView()
        Text("sibling")
    };
}
