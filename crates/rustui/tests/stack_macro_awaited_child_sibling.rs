use rustui::{
    HStack, VStack,
    components::basic::text::{Text, TextView},
};

async fn make_view() -> TextView {
    Text("first")
}

#[test]
fn awaited_constructor_keeps_adjacent_siblings() {
    let _future = async {
        let _vertical = VStack! {
            make_view().await
            Text("sibling")
        };

        let _horizontal = HStack! {
            make_view().await
            Text("sibling")
        };
    };
}
