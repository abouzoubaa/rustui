//! Menu action disabled-item and owner-removal acceptance using RustUI's public API.

use rustui::{localization::MessageArguments, prelude::*};

pub fn demo() -> impl View {
    Component("menu-gallery", MenuGallery::default)
}

#[component]
struct MenuGallery {
    #[state(0)]
    count: u32,
    #[state(0)]
    disabled_hits: u32,
    #[state(true)]
    menu_present: bool,
}

impl MenuGallery {
    fn increment(&mut self) {
        self.count += 1;
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn disabled_hit(&mut self) {
        self.disabled_hits += 1;
    }

    fn toggle_menu(&mut self) {
        self.menu_present = !self.menu_present;
    }
}

impl View for MenuGallery {
    fn body(&self) -> impl View {
        let count = self.count();
        let disabled_hits = self.disabled_hits();
        let arguments = MessageArguments::new()
            .with("count", count as i64)
            .with("disabled", disabled_hits as i64);
        VStack::new()
            .spacing(10.0)
            .child(LocalizedText("lab-menu-heading"))
            .child(super::LocaleSelector::new())
            .child(LocalizedText("lab-menu-instructions"))
            .child(HStack::new().child(
                Button("Remove/restore menu owner", self.action(Self::toggle_menu))
                    .id("menu-gallery-owner-toggle")
                    .localized_label("lab-menu-toggle-owner"),
            ))
            .child(if self.menu_present() {
                AnyView::new(HStack::new().child(
                    Menu(
                        format!("Actions: {count} (disabled hits: {disabled_hits})"),
                        [
                            MenuItem::action("increment", "Increment", self.action(Self::increment))
                                .localized("lab-menu-increment"),
                            MenuItem::separator(),
                            MenuItem::action("reset", "Reset", self.action(Self::reset))
                                .localized("lab-menu-reset"),
                            MenuItem::action("disabled", "Unavailable", self.action(Self::disabled_hit))
                                .localized("lab-menu-unavailable")
                                .disabled(true),
                        ],
                    )
                    .localized_label_with_arguments("lab-menu-actions-count", arguments.clone())
                    .id("menu-gallery-actions"),
                ))
            } else {
                AnyView::new(LocalizedText("lab-menu-owner-removed"))
            })
            .child(LocalizedText("lab-menu-counts").with_arguments(arguments))
            .child(
                Text("Menu(\"Actions\", [MenuItem::action(\"increment\", \"Increment\", self.action(Self::increment))]).id(\"actions\")")
                    .font_size(12.0),
            )
    }
}

#[cfg(test)]
mod tests {
    use rustui::localization::{Locale, MessageArguments, format_integer};

    #[test]
    fn menu_counts_format_in_all_supported_locales() {
        let catalogs = super::super::catalogs();
        for tag in ["en", "fr", "ar"] {
            let locale = Locale::parse(tag).unwrap();
            let args = MessageArguments::new()
                .with("count", 12_i64)
                .with("disabled", 34_i64);
            for key in ["lab-menu-actions-count", "lab-menu-counts"] {
                let result = catalogs.format(&locale, key, &args).unwrap();
                assert!(
                    result.contains(&format_integer(12, &locale)),
                    "{tag}: {result}"
                );
                assert!(
                    result.contains(&format_integer(34, &locale)),
                    "{tag}: {result}"
                );
            }
        }
    }
}
