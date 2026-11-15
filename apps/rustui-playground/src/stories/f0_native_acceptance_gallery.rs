//! One Playground page containing the workloads needed for the F0 native check.

use rustui::prelude::*;

const COLLECTION_ROWS: usize = 5_000;
const EDITOR_LINES: usize = 10_000;

pub fn demo() -> impl View {
    let rows = (0..COLLECTION_ROWS)
        .map(|row| {
            CollectionItem::new(format!("f0-row-{row:04}"), row).label(format!("F0 row {row:04}"))
        })
        .collect::<Vec<_>>()
        .into();

    VStack::new()
        .spacing(24.0)
        .child(Text("F0 native acceptance workloads"))
        .child(Text("1. Scroll the 5,000-row collection and select rows."))
        .child(
            Collection::new(
                "f0-five-thousand-rows",
                rows,
                240.0,
                30.0,
                |_index, row: &usize| Text(format!("F0 row {row:04}")),
            )
            .label("F0 5,000-row collection"),
        )
        .child(Text("2. Edit the large Rust source sample."))
        .child(Component("f0-large-rust-editor", LargeRustEditor::default))
        .child(Text(
            "3. Edit cells, move between cells, and leave the grid with Tab.",
        ))
        .child(super::editable_grid_gallery::demo())
        .child(Text("4. Open and dismiss menus and popovers."))
        .child(super::popover_gallery::demo())
        .child(super::menu_gallery::demo())
        .child(super::context_menu_gallery::demo())
        .child(Text(
            "5. Switch appearance, then check that controls follow the theme.",
        ))
        .child(super::card_gallery::demo())
}

fn large_rust_source() -> String {
    (0..EDITOR_LINES)
        .map(|line| format!("fn value_{line}() -> &'static str {{ \"text\" }}\n"))
        .collect()
}

#[component]
struct LargeRustEditor {
    #[state(large_rust_source())]
    source: String,
}

impl LargeRustEditor {
    fn read_source(owner: &Self) -> String {
        owner.source()
    }

    fn write_source(owner: &mut Self, source: String) {
        owner.source = source;
    }
}

impl View for LargeRustEditor {
    fn body(&self) -> impl View {
        CodeEditor(
            "f0-large-rust-source",
            Binding::new(self.source(), Self::read_source, Self::write_source),
        )
        .language("rust")
        .line_numbers(true)
        .height(560.0)
        .accessibility_label("F0 large Rust source editor")
    }
}

#[cfg(test)]
mod tests {
    use super::{COLLECTION_ROWS, EDITOR_LINES, large_rust_source};

    #[test]
    fn acceptance_workloads_have_the_expected_size() {
        let rows = (0..COLLECTION_ROWS).collect::<Vec<_>>();
        let source = large_rust_source();
        assert_eq!(rows.len(), 5_000);
        assert_eq!(source.lines().count(), 10_000);
        assert!(source.contains("fn value_9999()"));
        assert_eq!(EDITOR_LINES, 10_000);
    }
}
