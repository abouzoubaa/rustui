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
    #[state(String::from("Untitled"))]
    save_as: String,
    #[state(String::new())]
    tags: String,
    #[state(String::from("Rich Text Format"))]
    file_format: String,
}

impl DialogGallery {
    fn save_form(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(TextField("Save As", self.save_as_binding()).id("save-dialog-name"))
            .child(TextField("Tags", self.tags_binding()).id("save-dialog-tags"))
            .child(
                ComboBox(
                    "File Format",
                    self.file_format_binding(),
                    ["Rich Text Format", "Plain Text", "PDF"],
                )
                .id("save-dialog-format"),
            )
    }

    fn confirmed(&mut self) {
        self.result = format!("Saved {} as {}.", self.save_as, self.file_format);
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
                "Open the save dialog, edit its name, tags, or format, then Confirm or Cancel.",
            ))
            .child(
                Dialog("playground-save-dialog", "Open dialog")
                    .title("Save As")
                    .accessibility_label("Confirm saving changes")
                    .content(self.save_form())
                    .confirm_label("Save")
                    .cancel_label("Cancel")
                    .on_confirm(self.action(Self::confirmed))
                    .on_cancel(self.action(Self::cancelled)),
            )
            .child(Container(Text(self.result())))
    }
}

#[cfg(test)]
mod tests {
    use super::DialogGallery;

    #[test]
    fn save_dialog_form_controls_update_their_owner_bindings() {
        let mut gallery = DialogGallery::default();
        let _form = gallery.save_form();

        let save_as = gallery.save_as_binding();
        let tags = gallery.tags_binding();
        let file_format = gallery.file_format_binding();
        assert_eq!(save_as.read_from(&gallery), Some("Untitled".into()));
        assert_eq!(tags.read_from(&gallery), Some(String::new()));
        assert_eq!(
            file_format.read_from(&gallery),
            Some("Rich Text Format".into())
        );

        assert!(save_as.write_to(&mut gallery, "Notes.rtf".into()));
        assert!(tags.write_to(&mut gallery, "Design".into()));
        assert!(file_format.write_to(&mut gallery, "PDF".into()));
        assert_eq!(gallery.save_as(), "Notes.rtf");
        assert_eq!(gallery.tags(), "Design");
        assert_eq!(gallery.file_format(), "PDF");
    }
}
