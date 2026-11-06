//! Native hover transitions through the public RustUI API.

use rustui::localization::MessageArguments;
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("hover-gallery", HoverGallery::default)
}

#[component]
struct HoverGallery {
    #[state(true)]
    shown: bool,
    #[state(false)]
    hovered: bool,
    #[state(0)]
    enters: u32,
    #[state(0)]
    exits: u32,
    #[state(0)]
    child_count: u32,
    #[state(String::new())]
    child_text: String,
}

impl HoverGallery {
    fn enter(&mut self) {
        if !self.hovered {
            self.hovered = true;
            self.enters += 1;
        }
    }

    fn exit(&mut self) {
        if self.hovered {
            self.hovered = false;
            self.exits += 1;
        }
    }

    fn hide(&mut self) {
        self.shown = false;
        if self.hovered {
            self.hovered = false;
            self.exits += 1;
        }
    }

    fn show(&mut self) {
        self.shown = true;
        self.hovered = false;
    }

    fn increment_child(&mut self) {
        self.child_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rustui::localization::Locale;

    #[test]
    fn hover_story_messages_resolve_in_supported_locales() {
        let catalogs = crate::stories::locale_selector::catalogs();
        let arguments = MessageArguments::new()
            .with("enters", 2_i32)
            .with("exits", 3_i32)
            .with("clicks", 4_i32);
        for tag in ["en", "fr", "ar"] {
            let locale = Locale::parse(tag).unwrap();
            for key in [
                "lab-hover-region",
                "lab-hover-instructions",
                "lab-hover-input",
                "lab-hover-count-click",
                "lab-hover-hide",
                "lab-hover-restore",
                "lab-hover-note",
            ] {
                assert_ne!(catalogs.resolve(&locale, key), key, "{tag}: {key}");
            }
            for key in ["lab-hover-status-hovered", "lab-hover-status-idle"] {
                let status = catalogs.format(&locale, key, &arguments).unwrap();
                assert_ne!(status, key);
                assert!(!status.contains('{'), "{tag}: {key}: {status}");
            }
        }
    }

    #[test]
    fn hiding_a_hovered_region_records_its_exit_once() {
        let mut gallery = HoverGallery::default();
        gallery.enter();
        gallery.hide();

        assert!(!gallery.hovered());
        assert_eq!(gallery.enters(), 1);
        assert_eq!(gallery.exits(), 1);

        gallery.exit(); // Delayed exit callback from the unmounted region.
        assert_eq!(gallery.exits(), 1);

        gallery.show();
        assert!(gallery.shown());
        assert!(!gallery.hovered());

        gallery.hide();
        assert!(!gallery.shown());
        assert_eq!(gallery.exits(), 1);

        gallery.hide();
        assert_eq!(gallery.exits(), 1);
    }
}

impl View for HoverGallery {
    fn body(&self) -> impl View {
        let status_key = if self.hovered() {
            "lab-hover-status-hovered"
        } else {
            "lab-hover-status-idle"
        };
        let status_arguments = MessageArguments::new()
            .with("enters", i64::from(self.enters()))
            .with("exits", i64::from(self.exits()))
            .with("clicks", i64::from(self.child_count()));
        let status = format!(
            "Hovered: {} · Enters: {} · Exits: {} · Child clicks: {}",
            self.hovered(),
            self.enters(),
            self.exits(),
            self.child_count(),
        );
        let region = self.shown().then(|| {
            HoverRegion(
                "hover-gallery-region",
                VStack::new()
                    .spacing(8.0)
                    .child(LocalizedText("lab-hover-instructions"))
                    .child(
                        TextField("Hover region input", self.child_text_binding())
                            .localized_placeholder("lab-hover-input")
                            .id("hover-child-input"),
                    )
                    .child(
                        Button("Count child click", self.action(Self::increment_child))
                            .localized_label("lab-hover-count-click")
                            .id("hover-child-button"),
                    )
                    .child(
                        Button("Hide while hovered", self.action(Self::hide))
                            .localized_label("lab-hover-hide")
                            .id("hover-hide-button"),
                    ),
            )
            .on_enter(self.action(Self::enter))
            .on_exit(self.action(Self::exit))
        });

        VStack::new()
            .spacing(10.0)
            .child(LocalizedText("lab-hover-region"))
            .child(
                Semantics(
                    "hover-gallery-status",
                    LocalizedText(status_key).with_arguments(status_arguments),
                )
                .live_region_value(status, LivePoliteness::Polite),
            )
            .child(region)
            .child(
                Button("Restore region", self.action(Self::show))
                    .localized_label("lab-hover-restore")
                    .id("hover-restore-button")
                    .disabled(self.shown()),
            )
            .child(LocalizedText("lab-hover-note"))
    }
}
