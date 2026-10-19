//! Trigger-driven side Sheet demonstration using only the public RustUI API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Container(
        VStack::new()
            .spacing(10.0)
            .child(Text(
                "Side sheet using the upstream default placement and size.",
            ))
            .child(
                Sheet("playground-details-sheet", "Open sheet")
                    .title("Details")
                    .accessibility_label("Details and sheet contents")
                    .content(Text("This content is hosted inside the sheet.")),
            ),
    )
    .width(560.0)
    .padding(8.0)
    .border()
}
