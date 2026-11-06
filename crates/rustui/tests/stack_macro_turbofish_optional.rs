use rustui::{
    VStack,
    components::basic::text::{Text, TextView},
};

#[test]
fn turbofish_optional_children_typecheck_through_public_stack_macro() {
    let visible = true;

    let _then = VStack! {
        Text("before")
        (visible).then::<TextView, _>(|| Text("optional"))
        Text("after")
    };

    let _then_some = VStack! {
        Text("before")
        (visible).then_some::<TextView>(Text("optional"))
        Text("after")
    };
}
