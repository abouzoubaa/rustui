use rustui::{
    HStack, VStack,
    components::basic::text::{Text, TextView},
};

struct ViewSource {
    view: TextView,
}

fn make_views() -> (ViewSource,) {
    (ViewSource { view: Text("first") },)
}

#[test]
fn tuple_field_constructor_keeps_adjacent_siblings() {
    let _vertical = VStack! {
        make_views().0.view
        Text("sibling")
    };

    let _horizontal = HStack! {
        make_views().0.view
        Text("sibling")
    };
}
