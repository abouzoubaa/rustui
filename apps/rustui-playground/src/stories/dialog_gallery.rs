//! Trigger-driven Dialog demonstration using only the public RustUI API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Container(Component("dialog-gallery", DialogGallery::default))
        .width(560.0)
        .padding(8.0)
        .border()
}

#[component]
struct DialogGallery {
    #[state(String::from("No dialog action yet."))]
    result: String,
}

impl DialogGallery {
    fn confirmed(&mut self) {
        self.result = String::from("Confirmed: the action was accepted.");
    }

    fn cancelled(&mut self) {
        self.result = String::from("Cancelled: no action was taken.");
    }
}

impl View for DialogGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(10.0)
            .child(Text("Single topmost dialog"))
            .child(Text(
                "Open the dialog, then choose Confirm or Cancel. The result appears here.",
            ))
            .child(
                Dialog("playground-save-dialog", "Open dialog")
                    .title("Save changes?")
                    .accessibility_label("Confirm saving changes")
                    .content(Text("Save the current changes before continuing?"))
                    .confirm_label("Confirm")
                    .cancel_label("Cancel")
                    .on_confirm(self.action(Self::confirmed))
                    .on_cancel(self.action(Self::cancelled)),
            )
            .child(Container(Text(self.result())))
    }
}
