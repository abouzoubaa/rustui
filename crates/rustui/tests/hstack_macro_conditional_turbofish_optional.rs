use rustui::{
    HStack,
    components::basic::text::{Text, TextView},
};

#[test]
fn conditional_turbofish_optional_child_typechecks_through_public_hstack_macro() {
    let visible = true;
    let _view = HStack! {
        Text("before")
        if visible {
            (visible).then::<TextView, _>(|| Text("optional"))
        } else {
            Text("fallback")
        }
        Text("after")
    };
}
