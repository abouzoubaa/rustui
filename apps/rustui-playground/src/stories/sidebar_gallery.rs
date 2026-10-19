//! Stable-ID selection and item replacement demo for the public Sidebar API.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("sidebar-gallery", SidebarGallery::default)
}

#[component]
struct SidebarGallery {
    #[state(None)]
    selection: Option<String>,
    #[state(false)]
    reversed: bool,
    #[state(false)]
    relabelled: bool,
    #[state(true)]
    projects_present: bool,
}

impl SidebarGallery {
    fn select_projects(&mut self) {
        self.selection = Some(String::from("projects"));
    }

    fn select_inbox(&mut self) {
        self.selection = Some(String::from("inbox"));
    }

    fn reverse_items(&mut self) {
        self.reversed = !self.reversed;
    }

    fn relabel_items(&mut self) {
        self.relabelled = !self.relabelled;
    }

    fn toggle_projects(&mut self) {
        self.projects_present = !self.projects_present;
    }
}

impl View for SidebarGallery {
    fn body(&self) -> impl View {
        let projects_label = if self.relabelled() {
            "Workspaces"
        } else {
            "Projects"
        };
        let mut items = vec![SidebarItem::new("inbox", "Inbox")];
        if self.projects_present() {
            items.push(SidebarItem::new("projects", projects_label));
        }
        items.push(SidebarItem::new("settings", "Settings").disabled(true));
        if self.reversed() {
            items.reverse();
        }

        let selected = self.selection();
        let selected_status = match selected.as_deref() {
            None => String::from("No selection"),
            Some("projects") if !self.projects_present() => {
                format!("Selected ID: projects (no matching row; owner binding retained)")
            }
            Some(id @ ("inbox" | "projects" | "settings")) => format!("Selected ID: {id}"),
            Some(id) => format!("Selected ID: {id} (no matching row; owner binding retained)"),
        };

        VStack::new()
            .spacing(8.0)
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button(
                        "Select Projects",
                        self.action(Self::select_projects),
                    ))
                    .child(Button("Select Inbox", self.action(Self::select_inbox)))
                    .child(Button("Reverse", self.action(Self::reverse_items)))
                    .child(Button("Relabel", self.action(Self::relabel_items)))
                    .child(Button(
                        if self.projects_present() {
                            "Remove Projects"
                        } else {
                            "Restore Projects"
                        },
                        self.action(Self::toggle_projects),
                    )),
            )
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(
                        Container(Sidebar(
                            "sidebar-gallery-navigation",
                            items,
                            self.selection_binding(),
                        ))
                        .width(240.0)
                        .height(240.0)
                        .border(),
                    )
                    .child(Text(selected_status)),
            )
    }
}
