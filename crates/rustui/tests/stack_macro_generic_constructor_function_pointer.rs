use rustui::{
    HStack, VStack,
    components::basic::text::{Text, TextView},
};

fn make_view<T>() -> TextView {
    Text(std::any::type_name::<T>())
}

#[test]
fn generic_constructor_with_function_pointer_type_keeps_adjacent_siblings() {
    let _vertical = VStack! {
        make_view::<fn() -> u8>()
        Text("sibling")
    };

    let _horizontal = HStack! {
        make_view::<fn() -> u8>()
        Text("sibling")
    };
}
