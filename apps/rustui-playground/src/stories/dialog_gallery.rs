//! Trigger-driven Dialog demonstration using only the public RustUI API.

use rustui::{localization::MessageArguments, prelude::*};

pub fn demo() -> impl View {
    HStack::new().child(
        Container(Component("dialog-gallery", DialogGallery::default))
            .width(560.0)
            .padding(8.0)
            .border(),
    )
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum DialogResult {
    Idle,
    Saved { name: String, format: String },
    Cancelled,
}

#[component]
struct DialogGallery {
    #[state(DialogResult::Idle)]
    result: DialogResult,
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
            .child(
                HStack::new().child(
                    TextField("Save As", self.save_as_binding())
                        .id("save-dialog-name")
                        .localized_placeholder("lab-save-dialog-name"),
                ),
            )
            .child(
                HStack::new().child(
                    TextField("Tags", self.tags_binding())
                        .id("save-dialog-tags")
                        .localized_placeholder("lab-save-dialog-tags"),
                ),
            )
            .child(
                ComboBox(
                    "File Format",
                    self.file_format_binding(),
                    ["Rich Text Format", "Plain Text", "PDF"],
                )
                .id("save-dialog-format")
                .localized_placeholder("lab-save-dialog-format")
                .localized_option("Rich Text Format", "lab-save-dialog-rich-text")
                .localized_option("Plain Text", "lab-save-dialog-plain-text")
                .localized_option("PDF", "lab-save-dialog-pdf"),
            )
    }

    fn confirmed(&mut self) {
        self.result = DialogResult::Saved {
            name: self.save_as.clone(),
            format: self.file_format.clone(),
        };
    }

    fn cancelled(&mut self) {
        self.result = DialogResult::Cancelled;
    }
}

impl View for DialogGallery {
    fn body(&self) -> impl View {
        let (status, args) = match self.result() {
            DialogResult::Idle => ("lab-save-dialog-idle", MessageArguments::new()),
            DialogResult::Cancelled => ("lab-save-dialog-cancelled", MessageArguments::new()),
            DialogResult::Saved { name, format } => {
                let key = match format.as_str() {
                    "Rich Text Format" => "lab-save-dialog-saved-rich",
                    "Plain Text" => "lab-save-dialog-saved-plain",
                    "PDF" => "lab-save-dialog-saved-pdf",
                    _ => "lab-save-dialog-saved",
                };
                (
                    key,
                    MessageArguments::new()
                        .with("name", name)
                        .with("format", format),
                )
            }
        };
        VStack::new()
            .spacing(10.0)
            .child(LocalizedText("lab-save-dialog-heading"))
            .child(LocalizedText("lab-save-dialog-help"))
            .child(
                HStack::new().child(
                    Dialog("playground-save-dialog", "Open dialog")
                        .localized_trigger("lab-save-dialog-open")
                        .title("Save As")
                        .localized_title("lab-save-dialog-title")
                        .accessibility_label("Confirm saving changes")
                        .localized_accessibility_label("lab-save-dialog-accessibility")
                        .content(self.save_form())
                        .confirm_label("Save")
                        .localized_confirm_label("lab-save-dialog-save")
                        .cancel_label("Cancel")
                        .localized_cancel_label("lab-save-dialog-cancel")
                        .on_confirm(self.action(Self::confirmed))
                        .on_cancel(self.action(Self::cancelled)),
                ),
            )
            .child(Container(LocalizedText(status).with_arguments(args)))
    }
}

#[cfg(test)]
mod tests {
    use super::{DialogGallery, DialogResult};

    #[test]
    fn saved_result_keeps_snapshot_until_next_dialog_action() {
        let mut gallery = DialogGallery::default();
        assert!(
            gallery
                .save_as_binding()
                .write_to(&mut gallery, "Notes.rtf".into())
        );
        assert!(
            gallery
                .file_format_binding()
                .write_to(&mut gallery, "PDF".into())
        );
        gallery.confirmed();
        assert!(
            gallery
                .save_as_binding()
                .write_to(&mut gallery, "Later.rtf".into())
        );
        assert_eq!(
            gallery.result(),
            DialogResult::Saved {
                name: "Notes.rtf".into(),
                format: "PDF".into()
            }
        );
        gallery.cancelled();
        assert_eq!(gallery.result(), DialogResult::Cancelled);
    }

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
