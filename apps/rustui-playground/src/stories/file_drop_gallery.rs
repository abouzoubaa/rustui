//! Finder file-drop example using the public RustUI API.
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("file-drop-gallery", FileDropGallery::default)
}

#[component]
struct FileDropGallery {
    #[state(None)]
    dropped: Option<FileDrop>,
}

impl View for FileDropGallery {
    fn body(&self) -> impl View {
        let mut content = VStack::new().spacing(8.0).child(Text("File drop")).child(
            DropTarget::files(
                "image-files",
                self.dropped_binding(),
                Frame(Text("Drop PNG, JPG, JPEG or GIF images here"))
                    .width(420.0)
                    .height(120.0),
            )
            .extensions(["png", "jpg", "jpeg", "gif"]),
        );
        if let Some(drop) = self.dropped() {
            content = content.child(Text("Accepted files:"));
            for path in drop.accepted {
                content = content.child(Text(
                    path.file_name()
                        .unwrap_or(path.as_os_str())
                        .to_string_lossy()
                        .into_owned(),
                ));
            }
            if !drop.rejected.is_empty() {
                let names = drop
                    .rejected
                    .iter()
                    .map(|path| {
                        path.file_name()
                            .unwrap_or(path.as_os_str())
                            .to_string_lossy()
                            .into_owned()
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                content = content.child(ErrorState(format!("Rejected: {names}")));
            }
        }
        content
    }
}
