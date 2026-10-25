use rustui::{
    HStack, VStack,
    components::basic::text::Text,
};

macro_rules! make_view {
    () => {
        Text("macro child")
    };
}

#[test]
#[allow(unused_parens)]
fn parenthesized_macro_child_keeps_optional_child_boundary() {
    let visible = true;

    let _vertical = VStack! {
        (make_view![])
        (visible).then(|| Text("optional"))
    };

    let _horizontal = HStack! {
        (make_view![])
        (visible).then(|| Text("optional"))
    };
}
