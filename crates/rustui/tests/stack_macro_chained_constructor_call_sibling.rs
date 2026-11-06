use rustui::{
    HStack, VStack,
    components::basic::text::{Text, TextView},
};

fn make_view() -> impl FnOnce(&str) -> TextView {
    |label| Text(label)
}

#[test]
fn chained_constructor_call_keeps_adjacent_siblings() {
    let _vertical = VStack! {
        make_view()("first")
        Text("sibling")
    };

    let _horizontal = HStack! {
        make_view()("first")
        Text("sibling")
    };
}

#[test]
fn turbofish_method_chain_on_nested_views_keeps_adjacent_siblings() {
    let _vertical = VStack! {
        rustui::layout::VStack::new().child::<TextView>(Text("nested"))
        Text("sibling")
    };

    let _horizontal = HStack! {
        rustui::layout::VStack::new().child::<TextView>(Text("nested"))
        Text("sibling")
    };
}
