//! Stable-ID virtual list workload and interaction demo.

use rustui::prelude::*;

const INITIAL_ROW_COUNT: usize = 1_000;

pub fn demo() -> impl View {
    Component("keyed-list-gallery", KeyedListGallery::default)
}

fn initial_row_ids() -> Vec<String> {
    (0..INITIAL_ROW_COUNT)
        .map(|index| format!("row-{index:04}"))
        .collect()
}

#[component]
struct KeyedListGallery {
    #[state(initial_row_ids())]
    rows: Vec<String>,
    #[state(1_000)]
    next_added_id: u64,
    #[state(None)]
    selected_row: Option<String>,
    #[state(None)]
    source_selection: Option<usize>,
    #[state(None)]
    stable_selection: Option<String>,
    #[state(true)]
    source_selectable: bool,
    #[state(true)]
    stable_selectable: bool,
    #[state(false)]
    reversed: bool,
}

impl KeyedListGallery {
    fn reverse_rows(&mut self) {
        self.rows.reverse();
        self.reversed = !self.reversed;
    }

    fn prepend_row(&mut self) {
        let id = format!("added-{:04}", self.next_added_id);
        self.next_added_id += 1;
        self.rows.insert(0, id);
    }

    fn remove_selected(&mut self) {
        let Some(selected) = self.selected_row.as_ref() else {
            return;
        };
        self.rows.retain(|row| row != selected);
    }

    fn preselect_escape_rows(&mut self) {
        self.source_selection = Some(1);
        self.stable_selection = Some("east".into());
    }

    fn toggle_source_selectable(&mut self) {
        self.source_selectable = !self.source_selectable;
    }

    fn toggle_stable_selectable(&mut self) {
        self.stable_selectable = !self.stable_selectable;
    }

    fn reset(&mut self) {
        self.rows = initial_row_ids();
        self.selected_row = None;
        self.reversed = false;
    }
}

impl View for KeyedListGallery {
    fn body(&self) -> impl View {
        let items = self
            .rows()
            .into_iter()
            .map(|id| KeyedListItem::new(id.clone(), format!("Item {id}")))
            .collect::<Vec<_>>();
        let selected = self.selected_row();
        let escape_items = ["north", "east", "south", "west"]
            .into_iter()
            .map(|id| KeyedListItem::new(id, id.to_uppercase()))
            .collect::<Vec<_>>();

        VStack::new()
            .spacing(8.0)
            .child(Text("Escape selection controls"))
            .child(Text(
                "Preselect row 2, disable selection, focus each list, then press Escape. The owner values should remain selected.",
            ))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Button(
                        "Preselect row 2 in both lists",
                        self.action(Self::preselect_escape_rows),
                    ).id("escape-lists-preselect"))
                    .child(Button(
                        if self.source_selectable() {
                            "Disable source List selection"
                        } else {
                            "Enable source List selection"
                        },
                        self.action(Self::toggle_source_selectable),
                    ).id("escape-source-list-toggle"))
                    .child(Button(
                        if self.stable_selectable() {
                            "Disable KeyedList selection"
                        } else {
                            "Enable KeyedList selection"
                        },
                        self.action(Self::toggle_stable_selectable),
                    ).id("escape-keyed-list-toggle")),
            )
            .child(Text(format!(
                "List owner (source index): {:?} (expected Some(1)) · KeyedList owner (stable ID): {} (expected east)",
                self.source_selection(),
                self.stable_selection().as_deref().unwrap_or("None"),
            )))
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(
                        VStack::new()
                            .spacing(4.0)
                            .child(Text("Source-index List"))
                            .child(
                                List(
                                    ["North", "East", "South", "West"],
                                    self.source_selection_binding(),
                                )
                                .id(format!(
                                    "escape-source-list-selectable-{}",
                                    self.source_selectable()
                                ))
                                .height(140.0)
                                .selectable(self.source_selectable()),
                            ),
                    )
                    .child(
                        VStack::new()
                            .spacing(4.0)
                            .child(Text("Stable-ID KeyedList"))
                            .child(
                                KeyedList(
                                    "escape-keyed-list",
                                    escape_items,
                                    self.stable_selection_binding(),
                                )
                                .height(140.0)
                                .row_height(32.0)
                                .searchable(false)
                                .selectable(self.stable_selectable()),
                            ),
                    ),
            )
            .child(Text("Keyed virtual list · 1,000 stable row IDs"))
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Reverse", self.action(Self::reverse_rows)).id("keyed-list-reverse"),
                    )
                    .child(
                        Button("Prepend new", self.action(Self::prepend_row))
                            .id("keyed-list-prepend"),
                    )
                    .child(
                        Button("Remove selected", self.action(Self::remove_selected))
                            .id("keyed-list-remove"),
                    )
                    .child(Button("Reset", self.action(Self::reset)).id("keyed-list-reset")),
            )
            .child(Text(format!(
                "Rows: {} · order: {} · selected ID: {}",
                self.rows().len(),
                if self.reversed() {
                    "reversed"
                } else {
                    "forward"
                },
                selected.as_deref().unwrap_or("none"),
            )))
            .child(
                KeyedList(
                    "keyed-list-gallery-rows",
                    items,
                    self.selected_row_binding(),
                )
                .height(420.0)
                .row_height(32.0)
                .searchable(true)
                .search_placeholder("Find a row")
                .scrollbar_visible(true),
            )
    }
}
