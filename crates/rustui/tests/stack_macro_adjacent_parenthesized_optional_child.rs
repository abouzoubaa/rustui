use rustui::{HStack, VStack, components::basic::text::Text};

#[test]
fn adjacent_parenthesized_optional_children_remain_separate() {
    let visible = true;
    let ready = true;

    let _vertical = VStack! {
        Text("before")
        (visible && ready).then(|| Text("optional"))
        Text("after")
    };

    let _horizontal = HStack! {
        Text("before")
        (visible && ready).then(|| Text("optional"))
        Text("after")
    };
}
