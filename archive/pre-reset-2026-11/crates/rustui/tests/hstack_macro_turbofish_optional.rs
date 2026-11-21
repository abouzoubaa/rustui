use rustui::{
    HStack,
    components::basic::text::{Text, TextView},
};

#[test]
fn turbofish_optional_children_typecheck_through_public_hstack_macro() {
    let visible = true;

    let _then = HStack! {
        Text("before")
        (visible).then::<TextView, _>(|| Text("optional"))
        Text("after")
    };

    let _then_some = HStack! {
        Text("before")
        (visible).then_some::<TextView>(Text("optional"))
        Text("after")
    };
}

#[test]
fn parenthesized_option_and_then_child_typechecks_through_public_hstack_macro() {
    let optional = Some(Text("optional"));

    let _stack = HStack! {
        Text("before")
        (optional).and_then(|child| Some(child))
        Text("after")
    };
}
