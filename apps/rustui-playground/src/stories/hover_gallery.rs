//! Native hover transitions through the public RustUI API.

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
                    .child(Text("Move the pointer into this region and out again."))
                    .child(
                        TextField("Hover region input", self.child_text_binding())
                            .id("hover-child-input"),
                    )
                    .child(
                        Button("Count child click", self.action(Self::increment_child))
                            .id("hover-child-button"),
                    )
                    .child(
                        Button("Hide while hovered", self.action(Self::hide))
                            .id("hover-hide-button"),
                    ),
            )
            .on_enter(self.action(Self::enter))
            .on_exit(self.action(Self::exit))
        });

        VStack::new()
            .spacing(10.0)
            .child(Text("Hover region"))
            .child(
                Semantics("hover-gallery-status", Text(status.clone()))
                    .live_region_value(status, LivePoliteness::Polite),
            )
            .child(region)
            .child(
                Button("Restore region", self.action(Self::show))
                    .id("hover-restore-button")
                    .disabled(self.shown()),
            )
            .child(Text("Hiding clears the owner's hover flag explicitly. Enter and exit are pointer events; the input and buttons remain usable by keyboard."))
    }
}
