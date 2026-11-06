//! Owner-state-driven illustrative window composition for the Playground.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("window-chrome-gallery", WindowChromeGallery::default)
}

#[component]
struct WindowChromeGallery {
    #[state(true)]
    document_open: bool,
    #[state(String::from("Roadmap.md"))]
    document_title: String,
    #[state(String::from("Stage 3 implementation notes"))]
    document_body: String,
}

impl WindowChromeGallery {
    fn create_document(&mut self) {
        self.document_open = true;
        self.document_title = String::from("Untitled document");
        self.document_body = String::from("Start writing your new document.");
    }

    fn close_document(&mut self) {
        self.document_open = false;
        self.document_title = String::from("Window preview");
        self.document_body = String::from("No document is open.");
    }

    fn document_status_value(&self) -> String {
        format!("{}. {}", self.document_title(), self.document_body())
    }
}

impl View for WindowChromeGallery {
    fn body(&self) -> impl View {
        let title = self.document_title();
        let body = self.document_body();
        let is_open = self.document_open();
        let document_status = self.document_status_value();
        let toolbar_action = if is_open {
            Button("Close document", self.action(Self::close_document))
                .id("window-chrome-close-document")
        } else {
            Button("New document", self.action(Self::create_document))
                .id("window-chrome-new-document")
        };

        VStack::new()
            .spacing(12.0)
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(Text(title).font_weight(TextWeight::Semibold))
                    .child(toolbar_action),
            )
            .child(
                Semantics("window-chrome-document-status", Text(body))
                    .live_region_value(document_status, LivePoliteness::Polite),
            )
            .child(Text(
                "Illustrative composition only; it does not create a native titlebar or control a system window.",
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::WindowChromeGallery;

    #[test]
    fn document_actions_update_the_preview_title_and_body() {
        let mut gallery = WindowChromeGallery::default();
        assert!(gallery.document_open());
        assert_eq!(gallery.document_title(), "Roadmap.md");
        assert_eq!(gallery.document_body(), "Stage 3 implementation notes");
        assert_eq!(
            gallery.document_status_value(),
            "Roadmap.md. Stage 3 implementation notes"
        );

        gallery.close_document();
        assert!(!gallery.document_open());
        assert_eq!(gallery.document_title(), "Window preview");
        assert_eq!(gallery.document_body(), "No document is open.");
        assert_eq!(
            gallery.document_status_value(),
            "Window preview. No document is open."
        );

        gallery.create_document();
        assert!(gallery.document_open());
        assert_eq!(gallery.document_title(), "Untitled document");
        assert_eq!(gallery.document_body(), "Start writing your new document.");
        assert_eq!(
            gallery.document_status_value(),
            "Untitled document. Start writing your new document."
        );
    }
}
