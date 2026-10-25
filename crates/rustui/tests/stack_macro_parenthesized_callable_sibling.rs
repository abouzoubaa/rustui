use rustui::{
    HStack, VStack,
    components::basic::text::{Text, TextView},
};

fn make_view(label: &str) -> TextView {
    Text(label)
}

#[test]
fn parenthesized_callable_constructor_keeps_adjacent_siblings() {
    let _vertical = VStack! {
        (make_view)("first")
        Text("sibling")
    };

    let _horizontal = HStack! {
        (make_view)("first")
        Text("sibling")
    };
}
