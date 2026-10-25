use rustui::{
    HStack, VStack,
    components::basic::text::Text,
};

macro_rules! macro_view {
    ($value:literal) => {
        Text($value)
    };
}

#[test]
fn bracket_delimited_macro_child_keeps_adjacent_siblings() {
    let _vertical = VStack! {
        macro_view!["macro child"]
        Text("sibling")
    };

    let _horizontal = HStack! {
        macro_view!["macro child"]
        Text("sibling")
    };
}
