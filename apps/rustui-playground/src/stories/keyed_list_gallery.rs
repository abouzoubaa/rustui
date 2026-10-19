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

        VStack::new()
            .spacing(8.0)
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
