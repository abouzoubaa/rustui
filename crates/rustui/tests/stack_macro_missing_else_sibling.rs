use rustui::{
    HStack, VStack,
    components::basic::text::Text,
};

#[test]
fn missing_else_branch_keeps_adjacent_siblings() {
    let visible = true;

    let _vertical = VStack! {
        Text("before")
        if visible {
            Text("only")
        }
        Text("after")
    };

    let _horizontal = HStack! {
        Text("before")
        if visible {
            Text("only")
        }
        Text("after")
    };
}
