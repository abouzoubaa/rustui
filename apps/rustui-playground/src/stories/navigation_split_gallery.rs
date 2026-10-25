//! Controlled NavigationSplit ownership and destination reconciliation demo.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("navigation-split-gallery", NavigationSplitGallery::default)
}

#[component]
struct NavigationSplitGallery {
    #[state(Some(String::from("files")))]
    selection: Option<String>,
    #[state(String::from("Files note"))]
    files_text: String,
    #[state(String::from("Settings note"))]
    settings_text: String,
    #[state(String::from("Archive note"))]
    archive_text: String,
    #[state(0)]
    files_count: u32,
    #[state(0)]
    settings_count: u32,
    #[state(0)]
    archive_count: u32,
    #[state(false)]
    reversed: bool,
    #[state(560.0)]
    viewport_width: f32,
    #[state(false)]
    archive_disabled: bool,
    #[state(None)]
    removed_item: Option<String>,
}

impl NavigationSplitGallery {
    fn select_files(&mut self) {
        self.selection = Some(String::from("files"));
    }

    fn select_settings(&mut self) {
        self.selection = Some(String::from("settings"));
    }

    fn select_archive(&mut self) {
        self.selection = Some(String::from("archive"));
    }

    fn clear_selection(&mut self) {
        self.selection = None;
    }

    fn select_unknown(&mut self) {
        self.selection = Some(String::from("missing-page"));
    }

    fn set_viewport_width_480(&mut self) {
        self.viewport_width = 480.0;
    }

    fn set_viewport_width_560(&mut self) {
        self.viewport_width = 560.0;
    }

    fn set_viewport_width_640(&mut self) {
        self.viewport_width = 640.0;
    }

    fn toggle_order(&mut self) {
        self.reversed = !self.reversed;
    }

    fn toggle_archive_disabled(&mut self) {
        self.archive_disabled = !self.archive_disabled;
    }

    fn toggle_selected_item_presence(&mut self) {
        if self.removed_item.is_some() {
            self.removed_item = None;
            return;
        }

        if let Some(id @ ("files" | "settings" | "archive")) = self.selection.as_deref() {
            self.removed_item = Some(String::from(id));
        }
    }

    fn increment_files(&mut self) {
        self.files_count += 1;
    }

    fn increment_settings(&mut self) {
        self.settings_count += 1;
    }

    fn increment_archive(&mut self) {
        self.archive_count += 1;
    }
}

impl View for NavigationSplitGallery {
    fn body(&self) -> impl View {
        let mut items = Vec::with_capacity(3);
        if self.removed_item().as_deref() != Some("files") {
            items.push(NavigationSplitItem::new(
                "files",
                "Files",
                VStack::new()
                    .spacing(8.0)
                    .child(Text("Files detail"))
                    .child(
                        TextField("Files note", self.files_text_binding()).id("split-files-note"),
                    )
                    .child(
                        Button(
                            format!("Increment Files: {}", self.files_count()),
                            self.action(Self::increment_files),
                        )
                        .id("split-files-increment"),
                    ),
            ));
        }
        if self.removed_item().as_deref() != Some("settings") {
            items.push(NavigationSplitItem::new(
                "settings",
                "Settings",
                VStack::new()
                    .spacing(8.0)
                    .child(Text("Settings detail"))
                    .child(
                        TextField("Settings note", self.settings_text_binding())
                            .id("split-settings-note"),
                    )
                    .child(
                        Button(
                            format!("Increment Settings: {}", self.settings_count()),
                            self.action(Self::increment_settings),
                        )
                        .id("split-settings-increment"),
                    ),
            ));
        }
        if self.removed_item().as_deref() != Some("archive") {
            items.push(
                NavigationSplitItem::new(
                    "archive",
                    "Archive",
                    VStack::new()
                        .spacing(8.0)
                        .child(Text("Archive detail"))
                        .child(
                            TextField("Archive note", self.archive_text_binding())
                                .id("split-archive-note"),
                        )
                        .child(
                            Button(
                                format!("Increment Archive: {}", self.archive_count()),
                                self.action(Self::increment_archive),
                            )
                            .id("split-archive-increment"),
                        ),
                )
                .disabled(self.archive_disabled()),
            );
        }
        if self.reversed() {
            items.reverse();
        }

        let selected = self.selection();
        let selection_status = match selected.as_deref() {
            None => String::from("Selection: None · no detail destination"),
            Some(id) if self.removed_item().as_deref() == Some(id) => {
                format!("Selection: {id} · selected item is removed; binding remains set")
            }
            Some(id @ ("files" | "settings" | "archive")) => format!("Selection: {id}"),
            Some(id) => format!("Selection: {id} · no matching destination"),
        };
        let selected_item_label = if self.removed_item().is_some() {
            "Restore removed destination"
        } else {
            "Remove selected destination"
        };
        let can_remove_selected = self.removed_item().is_some()
            || matches!(selected.as_deref(), Some("files" | "settings" | "archive"));
        let requested_width = self.viewport_width();
        let split_selection = self.selection_binding();
        let measured_split = Container(
            GeometryReader::new("split-rendered-width", move |size| {
                VStack::new()
                    .child(Text(format!(
                        "Rendered NavigationSplit width: {:.1} logical px",
                        size.width
                    )))
                    .child(
                        Container(
                            NavigationSplit(
                                "navigation-split-gallery-view",
                                items.clone(),
                                split_selection.clone(),
                            )
                            .accessibility_label("Workspace navigation"),
                        )
                        .width(size.width)
                        .height(300.0),
                    )
            })
            .width(requested_width)
            .height(322.0),
        )
        .border();

        VStack::new()
            .spacing(10.0)
            .child(Text("NavigationSplit · bound selection and destination owners"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button("Select Files", self.action(Self::select_files)).id("split-select-files"))
                    .child(
                        Button("Select Settings", self.action(Self::select_settings))
                            .id("split-select-settings"),
                    )
                    .child(
                        Button("Select Archive", self.action(Self::select_archive))
                            .id("split-select-archive"),
                    )
                    .child(Button("Select None", self.action(Self::clear_selection)).id("split-select-none"))
                    .child(
                        Button("Select unknown", self.action(Self::select_unknown))
                            .id("split-select-unknown"),
                    ),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(
                        Button(
                            if self.reversed() { "Restore destination order" } else { "Reverse destinations" },
                            self.action(Self::toggle_order),
                        )
                        .id("split-reverse"),
                    )
                    .child(
                        Button(
                            if self.archive_disabled() {
                                "Enable Archive"
                            } else {
                                "Disable Archive"
                            },
                            self.action(Self::toggle_archive_disabled),
                        )
                        .id("split-toggle-archive-disabled"),
                    )
                    .child(
                        Button(selected_item_label, self.action(Self::toggle_selected_item_presence))
                            .id("split-toggle-selected-presence")
                            .disabled(!can_remove_selected),
                    ),
            )
        .child(
            HStack::new()
                .spacing(6.0)
                .child(
                    Button("480 px", self.action(Self::set_viewport_width_480))
                        .id("split-viewport-480"),
                )
                .child(
                    Button("560 px", self.action(Self::set_viewport_width_560))
                        .id("split-viewport-560"),
                )
                .child(
                    Button("640 px", self.action(Self::set_viewport_width_640))
                        .id("split-viewport-640"),
                ),
        )
        .child(Text(format!(
            "{} · requested width: {} logical px · order: {} · Archive disabled: {} · removed ID: {}",
                selection_status,
            self.viewport_width(),
                if self.reversed() { "reversed" } else { "normal" },
                self.archive_disabled(),
                self.removed_item().unwrap_or_else(|| String::from("None")),
            )))
        .child(measured_split)
            .child(Text(
                "The navigation tree is named Workspace navigation. The selected detail is a named region; the pinned resizable API does not expose divider accessibility metadata. Switching destinations releases detail controls.",
            ))
            .child(Text(format!(
                "Owner values · Files: {:?} / count {} · Settings: {:?} / count {} · Archive: {:?} / count {}",
                self.files_text(),
                self.files_count(),
                self.settings_text(),
                self.settings_count(),
                self.archive_text(),
                self.archive_count(),
            )))
    }
}
