use rustui::{
    HStack, VStack,
    components::basic::text::Text,
};

#[test]
fn nested_conditional_branches_typecheck_through_public_stack_macros() {
    let outer = true;
    let inner = false;

    let _vertical = VStack! {
        Text("before")
        if outer {
            Text("outer")
            if inner {
                Text("inner")
            } else {
                Text("inner fallback")
            }
            Text("outer sibling")
        } else {
            Text("outer fallback")
        }
        Text("after")
    };

    let _horizontal = HStack! {
        Text("before")
        if outer {
            Text("outer")
            if inner {
                Text("inner")
            } else {
                Text("inner fallback")
            }
            Text("outer sibling")
        } else {
            Text("outer fallback")
        }
        Text("after")
    };
}
