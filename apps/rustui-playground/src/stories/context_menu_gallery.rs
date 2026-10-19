//! Context menu action and removal review through the public RustUI API.

use rustui::prelude::*;

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
            .child(Text("Context menu review"))
            .child(Text(format!("Target owner visible: {}", self.visible())))
            .child(OwnerActionReader::<Self, _, _>(move |owner| {
                let owner = owner.clone();
                visible.then(|| {
                    Component("context-target-owner", move || ContextTarget::new(owner.clone()))
                })
            }))
            .child(
                Button("Remove or restore target owner", self.action(Self::toggle_visible))
                    .id("context-gallery-visible"),
            )
            .child(Text("Right-click the target and choose Remove owner to inspect dismissal and stale-action behavior."))
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
            MenuItem::action("increment", "Increment", self.action(Self::increment)),
            MenuItem::action(
                "remove-owner",
                "Remove owner",
                gallery.action(ContextMenuGallery::remove_owner),
            ),
            MenuItem::separator(),
            MenuItem::action("reset", "Reset", self.action(Self::reset)),
            MenuItem::action(
                "unavailable",
                "Unavailable",
                self.action(Self::disabled_hit),
            )
            .disabled(true),
            MenuItem::action("localized-close", "Close", self.action(Self::reset))
                .localized("rustui-action-close"),
        ];
        if self.reverse() {
            items.reverse();
        }

        VStack::new()
            .spacing(8.0)
            .child(Text(format!(
                "Count: {} · disabled hits: {} · reversed: {}",
                self.count(),
                self.disabled_hits(),
                self.reverse()
            )))
            .child(ContextMenu(
                "context-gallery-target",
                Button(
                    "Target: left-click increments; right-click opens menu",
                    self.action(Self::increment),
                )
                .id("context-gallery-primary"),
                items,
            ))
            .child(
                Button("Reverse item order", self.action(Self::toggle_order))
                    .id("context-gallery-reverse"),
            )
    }
}
