//! Tooltip hover and child-action demo using RustUI's public API.

use rustui::{localization::MessageArguments, prelude::*};

pub fn demo() -> impl View {
    Component("tooltip-gallery", TooltipGallery::default)
}

#[component]
struct TooltipGallery {
    #[state(0)]
    count: u32,
}

impl TooltipGallery {
    fn increment(&mut self) {
        self.count += 1;
    }
}

impl View for TooltipGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(10.0)
            .child(LocalizedText("lab-tooltip-heading"))
            .child(super::LocaleSelector::new())
            .child(LocalizedText("lab-tooltip-instructions"))
            .child(
                HStack::new().child(
                    Tooltip(
                        "This is help for plain text.",
                        LocalizedText("lab-tooltip-hover-text"),
                    )
                    .localized("lab-tooltip-text-help")
                    .id("tooltip-gallery-text"),
                ),
            )
            .child(
                HStack::new().child(
                    Tooltip(
                        "Increment the counter.",
                        Button("Increment", self.action(Self::increment))
                            .id("tooltip-gallery-button")
                            .localized_label("lab-tooltip-increment"),
                    )
                    .localized("lab-tooltip-increment-help")
                    .id("tooltip-gallery-button-help"),
                ),
            )
            .child(
                LocalizedText("lab-tooltip-count")
                    .with_arguments(MessageArguments::new().with("count", self.count() as i64)),
            )
            .child(
                HStack::new().child(
                    Tooltip(
                        "This action is unavailable.",
                        Button("Unavailable", self.action(Self::increment))
                            .id("tooltip-gallery-disabled-button")
                            .localized_label("lab-tooltip-unavailable")
                            .disabled(true),
                    )
                    .localized("lab-tooltip-unavailable-help")
                    .id("tooltip-gallery-disabled-help"),
                ),
            )
    }
}

#[cfg(test)]
mod tests {
    use rustui::localization::{Locale, MessageArguments, format_integer};

    #[test]
    fn tooltip_story_messages_and_count_resolve_in_supported_locales() {
        let catalogs = super::super::catalogs();
        for (tag, heading) in [
            ("en", "Tooltips"),
            ("fr", "Info-bulles"),
            ("ar", "التلميحات"),
        ] {
            let locale = Locale::parse(tag).unwrap();
            let args = MessageArguments::new().with("count", 12_i64);
            assert_eq!(
                catalogs
                    .format(&locale, "lab-tooltip-heading", &args)
                    .unwrap(),
                heading
            );
            for key in [
                "lab-tooltip-instructions",
                "lab-tooltip-hover-text",
                "lab-tooltip-text-help",
                "lab-tooltip-increment",
                "lab-tooltip-increment-help",
                "lab-tooltip-unavailable",
                "lab-tooltip-unavailable-help",
            ] {
                assert!(
                    !catalogs.format(&locale, key, &args).unwrap().is_empty(),
                    "{tag}: {key}"
                );
            }
            let count = catalogs
                .format(&locale, "lab-tooltip-count", &args)
                .unwrap();
            assert!(
                count.contains(&format_integer(12, &locale)),
                "{tag}: {count}"
            );
        }
    }
}
