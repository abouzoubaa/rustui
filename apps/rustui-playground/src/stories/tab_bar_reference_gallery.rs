//! Reference tab-bar composition built from RustUI's public TabView API.

use rustui::prelude::*;

const TAB_ITEMS: [(&str, &str); 3] = [
    ("overview", "Overview"),
    ("activity", "Activity"),
    ("settings", "Settings"),
];

pub fn demo() -> impl View {
    Component("tab-bar-reference-gallery", TabBarReferenceGallery::default)
}

#[component]
struct TabBarReferenceGallery {
    #[state(String::from("overview"))]
    selection: String,
}

impl TabBarReferenceGallery {
    fn selected_tab_binding(&self) -> Binding<String> {
        Binding::new(
            self.selection.clone(),
            Self::read_selection,
            Self::write_selection,
        )
    }

    fn read_selection(&self) -> String {
        self.selection.clone()
    }

    fn write_selection(&mut self, selection: String) {
        if TAB_ITEMS.iter().any(|(id, _)| *id == selection) {
            self.selection = selection;
        }
    }
}

impl View for TabBarReferenceGallery {
    fn body(&self) -> impl View {
        TabView(
            [
                TabItem::new("overview", "Overview", Text("Project overview")),
                TabItem::new("activity", "Activity", Text("Recent activity")),
                TabItem::new("settings", "Settings", Text("Workspace settings")),
            ],
            self.selected_tab_binding(),
        )
        .id("tab-bar-reference")
        .accessibility_label("Workspace sections")
    }
}

#[cfg(test)]
mod tests {
    use super::{TAB_ITEMS, TabBarReferenceGallery};

    #[test]
    fn tab_bar_catalog_composes_public_tabs_and_rejects_unknown_selection() {
        assert_eq!(
            TAB_ITEMS,
            [
                ("overview", "Overview"),
                ("activity", "Activity"),
                ("settings", "Settings"),
            ]
        );

        let mut gallery = TabBarReferenceGallery::default();
        assert_eq!(gallery.selection, "overview");
        let selection = gallery.selected_tab_binding();
        assert!(selection.write_to(&mut gallery, "activity".into()));
        assert_eq!(gallery.selection, "activity");
        assert!(selection.write_to(&mut gallery, "unknown".into()));
        assert_eq!(gallery.selection, "activity");
    }
}
