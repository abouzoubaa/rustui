//! Reference menu-bar composition built from RustUI's public Menu API.

use rustui::prelude::*;

const MENU_LABELS: [&str; 6] = ["File", "Edit", "View", "Item", "Window", "Help"];
const MENU_ACTION: &str = "Show menu actions";

pub fn demo() -> impl View {
    Component("menu-bar-reference-gallery", MenuBarReferenceGallery::default)
}

#[component]
struct MenuBarReferenceGallery {
    #[state(String::from("Choose a menu action"))]
    status: String,
    #[state(true)]
    sidebar_visible: bool,
}

impl MenuBarReferenceGallery {
    fn select_file(&mut self) {
        self.status = "File menu action selected".into();
    }

    fn select_edit(&mut self) {
        self.status = "Edit menu action selected".into();
    }

    fn toggle_sidebar(&mut self) {
        self.sidebar_visible = !self.sidebar_visible;
        self.status = if self.sidebar_visible {
            "Sidebar shown"
        } else {
            "Sidebar hidden"
        }
        .into();
    }

    fn sidebar_action_label(&self) -> &'static str {
        if self.sidebar_visible() {
            "Hide Sidebar"
        } else {
            "Show Sidebar"
        }
    }

    fn select_item(&mut self) {
        self.status = "Item menu action selected".into();
    }

    fn select_window(&mut self) {
        self.status = "Window menu action selected".into();
    }

    fn select_help(&mut self) {
        self.status = "Help menu action selected".into();
    }
}

impl View for MenuBarReferenceGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(
                HStack::new()
                    .spacing(10.0)
                    .child(Text("Finder"))
                    .child(
                        Menu(
                            MENU_LABELS[0],
                            [MenuItem::action(
                                "show-actions",
                                MENU_ACTION,
                                self.action(Self::select_file),
                            )],
                        )
                        .id("menu-bar-file"),
                    )
                    .child(
                        Menu(
                            MENU_LABELS[1],
                            [MenuItem::action(
                                "show-actions",
                                MENU_ACTION,
                                self.action(Self::select_edit),
                            )],
                        )
                        .id("menu-bar-edit"),
                    )
                    .child(
                        Menu(
                            MENU_LABELS[2],
                            [MenuItem::action(
                                "toggle-sidebar",
                                self.sidebar_action_label(),
                                self.action(Self::toggle_sidebar),
                            )],
                        )
                        .id("menu-bar-view"),
                    )
                    .child(
                        Menu(
                            MENU_LABELS[3],
                            [MenuItem::action(
                                "show-actions",
                                MENU_ACTION,
                                self.action(Self::select_item),
                            )],
                        )
                        .id("menu-bar-item"),
                    )
                    .child(
                        Menu(
                            MENU_LABELS[4],
                            [MenuItem::action(
                                "show-actions",
                                MENU_ACTION,
                                self.action(Self::select_window),
                            )],
                        )
                        .id("menu-bar-window"),
                    )
                    .child(
                        Menu(
                            MENU_LABELS[5],
                            [MenuItem::action(
                                "show-actions",
                                MENU_ACTION,
                                self.action(Self::select_help),
                            )],
                        )
                        .id("menu-bar-help"),
                    )
                    .child(Text("Wed Apr 1"))
                    .child(Text("9:41 AM")),
            )
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(
                        self.sidebar_visible().then_some(
                            VStack::new()
                                .spacing(4.0)
                                .child(Text("Favorites"))
                                .child(Text("Desktop"))
                                .child(Text("Downloads")),
                        ),
                    )
                    .child(Text("Documents")),
            )
            .child(Text(self.status()))
    }
}

#[cfg(test)]
mod tests {
    use super::{MENU_ACTION, MENU_LABELS, MenuBarReferenceGallery};

    #[test]
    fn menu_bar_composes_public_menu_controls_and_dispatches_owner_actions() {
        assert_eq!(MENU_LABELS, ["File", "Edit", "View", "Item", "Window", "Help"]);
        assert_eq!(MENU_ACTION, "Show menu actions");

        let mut gallery = MenuBarReferenceGallery::default();
        assert!(gallery.sidebar_visible);
        assert_eq!(gallery.sidebar_action_label(), "Hide Sidebar");
        gallery.toggle_sidebar();
        assert!(!gallery.sidebar_visible);
        assert_eq!(gallery.sidebar_action_label(), "Show Sidebar");
        assert_eq!(gallery.status, "Sidebar hidden");
        gallery.toggle_sidebar();
        assert!(gallery.sidebar_visible);
        assert_eq!(gallery.sidebar_action_label(), "Hide Sidebar");
        assert_eq!(gallery.status, "Sidebar shown");

        gallery.select_file();
        assert_eq!(gallery.status, "File menu action selected");
        gallery.select_edit();
        assert_eq!(gallery.status, "Edit menu action selected");
        gallery.select_item();
        assert_eq!(gallery.status, "Item menu action selected");
        gallery.select_window();
        assert_eq!(gallery.status, "Window menu action selected");
        gallery.select_help();
        assert_eq!(gallery.status, "Help menu action selected");
    }
}
