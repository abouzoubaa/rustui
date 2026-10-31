//! Named table semantics backed by the pinned GPUI Components table rows.

use rustui::{components::data::data_grid::DataGrid, prelude::*};

pub fn demo() -> impl View {
    Component("data-grid-accessibility", DataGridAccessibility::default)
}

#[component]
struct DataGridAccessibility {
    #[state(None)]
    selection: Option<usize>,
}

impl View for DataGridAccessibility {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(8.0)
            .child(Text("Named table role with owner-controlled row selection"))
            .child(
                DataGrid(
                    Table(
                        ["Quarter", "Revenue", "Status"],
                        [
                            ["Q1", "$18.2M", "Closed"],
                            ["Q2", "$20.4M", "Closed"],
                            ["Q3", "$22.1M", "In progress"],
                        ],
                        self.selection_binding(),
                    )
                    .id("data-grid-accessibility-table"),
                    "Quarterly revenue",
                )
                .id("quarterly-revenue"),
            )
            .child(Text(format!("Selected source row: {:?}", self.selection())))
    }
}
