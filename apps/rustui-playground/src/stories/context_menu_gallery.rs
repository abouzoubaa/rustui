//! Context menu action and removal review through the public RustUI API.

use rustui::{localization::MessageArguments, prelude::*};

pub fn demo() -> impl View {
    Component("context-menu-gallery", ContextMenuGallery::default)
}

#[component]
struct ContextMenuGallery {
    #[state(true)]
    visible: bool,
}

impl ContextMenuGallery {
    fn remove_owner(&mut self) {
        self.visible = false;
    }

    fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }
}

impl View for ContextMenuGallery {
    fn body(&self) -> impl View {
        let visible = self.visible();
        VStack::new()
            .spacing(8.0)
            .child(LocalizedText("lab-context-heading"))
            .child(LocalizedText(if visible {
                "lab-context-visible"
            } else {
                "lab-context-hidden"
            }))
            .child(OwnerActionReader::<Self, _, _>(move |owner| {
                let owner = owner.clone();
                visible.then(|| {
                    Component("context-target-owner", move || {
                        ContextTarget::new(owner.clone())
                    })
                })
            }))
            .child(
                Button(
                    "Remove or restore target owner",
                    self.action(Self::toggle_visible),
                )
                .id("context-gallery-visible")
                .localized_label("lab-context-toggle-owner"),
            )
            .child(LocalizedText("lab-context-help"))
    }
}

#[component]
struct ContextTarget {
    #[state(0)]
    count: u32,
    #[state(0)]
    disabled_hits: u32,
    #[state(false)]
    reverse: bool,
    gallery: Option<OwnerAction<ContextMenuGallery>>,
}

impl ContextTarget {
    fn new(gallery: OwnerAction<ContextMenuGallery>) -> Self {
        Self {
            gallery: Some(gallery),
            ..Self::default()
        }
    }

    fn increment(&mut self) {
        self.count += 1;
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn disabled_hit(&mut self) {
        self.disabled_hits += 1;
    }

    fn toggle_order(&mut self) {
        self.reverse = !self.reverse;
    }
}

impl View for ContextTarget {
    fn body(&self) -> impl View {
        let gallery = self.gallery.as_ref().expect("gallery owner is supplied");
        let mut items = vec![
            MenuItem::action("increment", "Increment", self.action(Self::increment))
                .localized("lab-context-increment"),
            MenuItem::action(
                "remove-owner",
                "Remove owner",
                gallery.action(ContextMenuGallery::remove_owner),
            )
            .localized("lab-context-remove-owner"),
            MenuItem::separator(),
            MenuItem::action("reset", "Reset", self.action(Self::reset))
                .localized("lab-context-reset"),
            MenuItem::action(
                "unavailable",
                "Unavailable",
                self.action(Self::disabled_hit),
            )
            .localized("lab-context-unavailable")
            .disabled(true),
            MenuItem::action("localized-close", "Close", self.action(Self::reset))
                .localized("rustui-action-close"),
        ];
        if self.reverse() {
            items.reverse();
        }

        VStack::new()
            .spacing(8.0)
            .child(
                LocalizedText(if self.reverse() {
                    "lab-context-counts-reversed"
                } else {
                    "lab-context-counts-normal"
                })
                .with_arguments(
                    MessageArguments::new()
                        .with("count", self.count() as i64)
                        .with("disabled", self.disabled_hits() as i64),
                ),
            )
            .child(ContextMenu(
                "context-gallery-target",
                Button(
                    "Target: left-click increments; right-click opens menu",
                    self.action(Self::increment),
                )
                .id("context-gallery-primary")
                .localized_label("lab-context-target"),
                items,
            ))
            .child(
                Button("Reverse item order", self.action(Self::toggle_order))
                    .id("context-gallery-reverse")
                    .localized_label("lab-context-reverse"),
            )
    }
}

#[cfg(test)]
mod tests {
    use rustui::localization::{Locale, MessageArguments, format_integer};

    #[test]
    fn context_menu_copy_resolves_in_supported_locales() {
        let catalogs = super::super::catalogs();
        for tag in ["en", "fr", "ar"] {
            let locale = Locale::parse(tag).unwrap();
            for key in [
                "lab-context-heading",
                "lab-context-visible",
                "lab-context-hidden",
                "lab-context-help",
                "lab-context-toggle-owner",
                "lab-context-target",
                "lab-context-reverse",
                "lab-context-increment",
                "lab-context-remove-owner",
                "lab-context-reset",
                "lab-context-unavailable",
            ] {
                let text = catalogs
                    .format(&locale, key, &MessageArguments::new())
                    .unwrap();
                assert!(!text.trim().is_empty() && text != key, "{tag}: {key}");
            }
            for key in ["lab-context-counts-normal", "lab-context-counts-reversed"] {
                let text = catalogs
                    .format(
                        &locale,
                        key,
                        &MessageArguments::new()
                            .with("count", 12_i64)
                            .with("disabled", 34_i64),
                    )
                    .unwrap();
                assert!(text.contains(&format_integer(12, &locale)), "{tag}: {text}");
                assert!(text.contains(&format_integer(34, &locale)), "{tag}: {text}");
            }
        }
    }
}
