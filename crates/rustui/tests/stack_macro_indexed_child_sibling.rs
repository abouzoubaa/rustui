use rustui::{
    HStack, VStack,
    components::basic::text::Text,
};

#[test]
fn indexed_constructor_keeps_adjacent_siblings() {
    let views = [Text("first")];

    let _vertical = VStack! {
        views[0].clone()
        Text("sibling")
    };

    let _horizontal = HStack! {
        views[0].clone()
        Text("sibling")
    };
}
