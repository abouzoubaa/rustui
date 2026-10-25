use rustui::{
    HStack, VStack,
    components::basic::text::Text,
};

#[test]
fn block_bodied_optional_child_with_nested_closure_typechecks() {
    let visible = true;
    let vertical_label = String::from("vertical optional");
    let horizontal_label = String::from("horizontal optional");

    let _vertical = VStack! {
        Text("before")
        visible.then(|| {
            let make_child = move || Text(vertical_label);
            make_child()
        })
        Text("after")
    };

    let _horizontal = HStack! {
        Text("before")
        visible.then(|| {
            let make_child = move || Text(horizontal_label);
            make_child()
        })
        Text("after")
    };
}
