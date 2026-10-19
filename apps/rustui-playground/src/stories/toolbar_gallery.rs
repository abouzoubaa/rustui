//! Toolbar action, order, disabled, and localization review through public RustUI.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("toolbar-gallery", ToolbarGallery::default)
}

#[component]
struct ToolbarGallery {
    #[state(0)]
    count: u32,
    #[state(false)]
    reverse: bool,
    #[state(false)]
    disabled: bool,
    #[state(0)]
    disabled_hits: u32,
}

impl ToolbarGallery {
    fn increment(&mut self) {
        self.count += 1;
    }

    fn reset(&mut self) {
        self.count = 0;
    }

    fn toggle_order(&mut self) {
        self.reverse = !self.reverse;
    }

    fn toggle_disabled(&mut self) {
        self.disabled = !self.disabled;
    }

    fn disabled_hit(&mut self) {
        self.disabled_hits += 1;
    }
}

impl View for ToolbarGallery {
    fn body(&self) -> impl View {
        let mut items = vec![
            ToolbarItem::action("increment", "Increment", self.action(Self::increment)),
            ToolbarItem::action("reset", "Reset", self.action(Self::reset)),
            ToolbarItem::action(
                "unavailable",
                "Unavailable",
                self.action(Self::disabled_hit),
            )
            .disabled(true),
            ToolbarItem::action("localized-close", "Close", self.action(Self::reset))
                .localized("rustui-action-close"),
        ];
        if self.reverse() {
            items.reverse();
        }

        VStack::new()
            .spacing(8.0)
            .child(Text("Toolbar review"))
            .child(Text(format!(
                "Count: {} · order reversed: {} · disabled: {} · disabled hits: {}",
                self.count(), self.reverse(), self.disabled(), self.disabled_hits()
            )))
            .child(Toolbar("toolbar-gallery-actions", items).disabled(self.disabled()))
            .child(Button("Reverse order", self.action(Self::toggle_order)).id("toolbar-reverse"))
            .child(Button("Toggle disabled", self.action(Self::toggle_disabled)).id("toolbar-disabled"))
            .child(Text("Tab into the toolbar, then use arrow keys to move among actions. Change locale to inspect Close."))
    }
}
