//! Controlled Table selection across source-row changes.

use rustui::prelude::*;

pub fn demo() -> impl View {
    Component(
        "table-reconciliation-gallery",
        TableReconciliationGallery::default,
    )
}

#[component]
struct TableReconciliationGallery {
    #[state(initial_rows())]
    rows: Vec<String>,
    #[state(None)]
    selection: Option<usize>,
    #[state(false)]
    reversed: bool,
}

fn initial_rows() -> Vec<String> {
    ["Alpha", "Beta", "Gamma", "Delta"]
        .into_iter()
        .map(String::from)
        .collect()
}

impl TableReconciliationGallery {
    fn select_first(&mut self) {
        self.selection = Some(0);
    }

    fn select_third(&mut self) {
        self.selection = Some(2);
    }

    fn clear_selection(&mut self) {
        self.selection = None;
    }

    fn select_invalid(&mut self) {
        self.selection = Some(self.rows.len() + 3);
    }

    fn reverse_rows(&mut self) {
        self.rows.reverse();
        self.reversed = !self.reversed;
    }

    fn remove_selected_row(&mut self) {
        if let Some(index) = self.selection.filter(|index| *index < self.rows.len()) {
            self.rows.remove(index);
        }
    }

    fn restore_rows(&mut self) {
        self.rows = initial_rows();
        self.reversed = false;
    }
}

impl View for TableReconciliationGallery {
    fn body(&self) -> impl View {
        let rows = self
            .rows()
            .iter()
            .enumerate()
            .map(|(index, name)| vec![index.to_string(), name.clone()])
            .collect::<Vec<_>>();

        VStack::new()
            .spacing(8.0)
            .child(Text("Table · controlled source-index selection"))
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button("Select row 0", self.action(Self::select_first)).id("table-reconcile-select-0"))
                    .child(Button("Select row 2", self.action(Self::select_third)).id("table-reconcile-select-2"))
                    .child(Button("Clear selection", self.action(Self::clear_selection)).id("table-reconcile-clear"))
                    .child(Button("Set invalid index", self.action(Self::select_invalid)).id("table-reconcile-invalid")),
            )
            .child(
                HStack::new()
                    .spacing(6.0)
                    .child(Button("Reverse rows", self.action(Self::reverse_rows)).id("table-reconcile-reverse"))
                    .child(Button("Remove selected row", self.action(Self::remove_selected_row)).id("table-reconcile-remove"))
                    .child(Button("Restore rows", self.action(Self::restore_rows)).id("table-reconcile-restore")),
            )
            .child(Text(format!(
                "Rows: {:?} · order: {} · bound selection: {:?}",
                self.rows(),
                if self.reversed() { "reversed" } else { "forward" },
                self.selection(),
            )))
            .child(Text(
                "Selection stores a source index. Reordering keeps the index, so it may refer to a different row. An out-of-range index is normalized to None, including when no row was selected.",
            ))
            .child(
                Table(["Source index", "Row"], rows, self.selection_binding())
                    .id("table-reconciliation-rows")
                    .height(240.0)
                    .striped(true),
            )
    }
}
