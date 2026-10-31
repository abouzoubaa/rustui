use rustui::{
    HStack,
    components::basic::text::{Text, TextView},
};

fn labeled_text(value: &str) -> TextView {
    Text(value)
}

fn make_view() -> Result<fn(&str) -> TextView, &'static str> {
    Ok(labeled_text)
}

#[test]
fn try_wrapped_constructor_and_sibling_typecheck_through_public_hstack_macro()
-> Result<(), &'static str> {
    let _stack = HStack! {
        make_view()? ("label")
        Text("sibling")
    };

    Ok(())
}
