use rustui::{
    HStack, VStack,
    components::basic::text::{Text, TextView},
};

fn make_view(label: &str) -> TextView {
    Text(label)
}

macro_rules! view_factory {
    () => {
        make_view
    };
}

#[test]
fn parenthesized_macro_callable_keeps_adjacent_siblings() {
    let _vertical = VStack! {
        (view_factory![])("first")
        Text("sibling")
    };

    let _horizontal = HStack! {
        (view_factory![])("first")
        Text("sibling")
    };
}
