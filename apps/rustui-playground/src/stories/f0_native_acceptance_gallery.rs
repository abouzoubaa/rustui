//! Focused Playground panels for the F0 native acceptance workloads.

use std::rc::Rc;

use rustui::prelude::*;

const COLLECTION_ROWS: usize = 5_000;
const EDITOR_LINES: usize = 10_000;
const COLLECTION_VIEWPORT_HEIGHT: f32 = 480.0;
const EDITOR_VIEWPORT_HEIGHT: f32 = 680.0;
const GRID_VIEWPORT_HEIGHT: f32 = 680.0;

const ACCEPTANCE_TABS: [&str; 5] = [
    "Collection",
    "Code editor",
    "Editable grid",
    "Menus & popovers",
    "Appearance",
];

const OVERLAY_TABS: [&str; 3] = ["Popover", "Menu", "Context menu"];

pub fn demo() -> impl View {
    Component(
        "f0-native-acceptance-gallery",
        F0NativeAcceptanceGallery::default,
    )
}

#[component]
struct F0NativeAcceptanceGallery {
    #[state(String::from("collection"))]
    selected_panel: String,
}

impl F0NativeAcceptanceGallery {
    fn read_panel(owner: &Self) -> String {
        owner.selected_panel()
    }

    fn write_panel(owner: &mut Self, panel: String) {
        owner.selected_panel = panel;
    }
}

impl View for F0NativeAcceptanceGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(16.0)
            .child(Text("F0 Native Acceptance"))
            .child(Text(
                "Choose a panel to give each native check its own workspace. Use the Playground header to switch the global appearance.",
            ))
            .child(
                TabView(
                    [
                        TabItem::new(
                            "collection",
                            ACCEPTANCE_TABS[0],
                            collection_panel(),
                        ),
                        TabItem::new("editor", ACCEPTANCE_TABS[1], editor_panel()),
                        TabItem::new("grid", ACCEPTANCE_TABS[2], grid_panel()),
                        TabItem::new("overlays", ACCEPTANCE_TABS[3], overlays_panel()),
                        TabItem::new("appearance", ACCEPTANCE_TABS[4], appearance_panel()),
                    ],
                    Binding::new(self.selected_panel(), Self::read_panel, Self::write_panel),
                )
                .id("f0-native-acceptance-panels")
                .style(TabViewStyle::Segmented),
            )
    }
}

fn collection_panel() -> impl View {
    VStack::new()
        .spacing(12.0)
        .child(Text("5,000-row virtualized collection"))
        .child(Text(
            "Scroll the list, select rows with mouse or keyboard, and confirm the selected row below.",
        ))
        .child(Component(
            "f0-acceptance-collection",
            AcceptanceCollection::default,
        ))
}

thread_local! {
    static ACCEPTANCE_ROWS: Rc<[CollectionItem<usize>]> = (0..COLLECTION_ROWS)
        .map(|row| {
            CollectionItem::new(format!("f0-row-{row:04}"), row)
                .label(format!("F0 row {row:04}"))
        })
        .collect::<Vec<_>>()
        .into();
}

fn acceptance_rows() -> Rc<[CollectionItem<usize>]> {
    ACCEPTANCE_ROWS.with(Rc::clone)
}

#[component]
struct AcceptanceCollection {
    #[state(None)]
    selected: Option<Identity>,
}

impl AcceptanceCollection {
    fn selection_binding(&self) -> Binding<Option<Identity>> {
        Binding::new(self.selected(), Self::read_selection, Self::write_selection)
    }

    fn read_selection(owner: &Self) -> Option<Identity> {
        owner.selected()
    }

    fn write_selection(owner: &mut Self, selection: Option<Identity>) {
        owner.selected = selection;
    }
}

impl View for AcceptanceCollection {
    fn body(&self) -> impl View {
        let selected = self
            .selected()
            .map(|identity| format!("Selected row: {}", identity.as_str()))
            .unwrap_or_else(|| "Selected row: none".into());
        VStack::new()
            .spacing(8.0)
            .child(
                Collection::new(
                    "f0-five-thousand-rows",
                    acceptance_rows(),
                    COLLECTION_VIEWPORT_HEIGHT,
                    30.0,
                    |_index, row: &usize| Text(format!("F0 row {row:04}")),
                )
                .label("F0 5,000-row collection")
                .selection(self.selection_binding()),
            )
            .child(Text(selected))
    }
}

fn editor_panel() -> impl View {
    VStack::new()
        .spacing(12.0)
        .child(Text("Large Rust CodeEditor · 10,000 lines"))
        .child(Text(
            "Click inside the editor, move the caret, select text, and type a small edit.",
        ))
        .child(Component("f0-large-rust-editor", LargeRustEditor::default))
}

fn large_rust_source() -> String {
    (0..EDITOR_LINES)
        .map(|line| {
            format!("fn value_{line}() -> &'static str {{ let keyword = \"text\"; keyword }}\n")
        })
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
        .height(EDITOR_VIEWPORT_HEIGHT)
        .accessibility_label("F0 large Rust source editor")
    }
}

fn grid_panel() -> impl View {
    VStack::new()
        .spacing(12.0)
        .child(Text("EditableDataGrid · 1,000 rows"))
        .child(Text(
            "Use arrows or Tab to move; Enter or F2 edits; Escape cancels. Tab from the final cell to leave the grid.",
        ))
        .child(super::editable_grid_gallery::acceptance_demo(GRID_VIEWPORT_HEIGHT))
}

#[component]
struct OverlaysAcceptancePanel {
    #[state(String::from("popover"))]
    selected_panel: String,
}

impl OverlaysAcceptancePanel {
    fn read_panel(owner: &Self) -> String {
        owner.selected_panel()
    }

    fn write_panel(owner: &mut Self, panel: String) {
        owner.selected_panel = panel;
    }
}

impl View for OverlaysAcceptancePanel {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(12.0)
            .child(Text(
                "Open each surface, try its keyboard and pointer path, then dismiss it.",
            ))
            .child(
                TabView(
                    [
                        TabItem::new("popover", OVERLAY_TABS[0], super::popover_gallery::demo()),
                        TabItem::new("menu", OVERLAY_TABS[1], super::menu_gallery::demo()),
                        TabItem::new(
                            "context-menu",
                            OVERLAY_TABS[2],
                            super::context_menu_gallery::demo(),
                        ),
                    ],
                    Binding::new(self.selected_panel(), Self::read_panel, Self::write_panel),
                )
                .id("f0-native-overlay-panels")
                .style(TabViewStyle::Segmented),
            )
    }
}

fn overlays_panel() -> impl View {
    Component(
        "f0-native-overlay-acceptance",
        OverlaysAcceptancePanel::default,
    )
}

fn appearance_panel() -> impl View {
    VStack::new()
        .spacing(12.0)
        .child(Text("Appearance and theme response"))
        .child(Text(
            "Use the window header’s Light and Dark actions, then inspect the card surfaces and controls below.",
        ))
        .child(super::card_gallery::demo())
}

#[cfg(test)]
mod tests {
    use super::{
        ACCEPTANCE_TABS, AcceptanceCollection, COLLECTION_ROWS, EDITOR_LINES, OVERLAY_TABS,
        acceptance_rows, large_rust_source,
    };
    use rustui::prelude::Identity;

    #[test]
    fn acceptance_workloads_have_the_expected_size() {
        let rows = acceptance_rows();
        let source = large_rust_source();
        assert_eq!(rows.len(), 5_000);
        assert_eq!(source.lines().count(), 10_000);
        assert!(source.contains("fn value_9999()"));
        assert_eq!(EDITOR_LINES, 10_000);
        assert_eq!(COLLECTION_ROWS, 5_000);
    }

    #[test]
    fn acceptance_page_lists_every_required_test_panel() {
        assert_eq!(
            ACCEPTANCE_TABS,
            [
                "Collection",
                "Code editor",
                "Editable grid",
                "Menus & popovers",
                "Appearance"
            ]
        );
        assert_eq!(OVERLAY_TABS, ["Popover", "Menu", "Context menu"]);
    }

    #[test]
    fn collection_binding_retains_a_selected_row() {
        let mut owner = AcceptanceCollection::default();
        let selection = Identity::from("f0-row-0042");

        assert!(
            owner
                .selection_binding()
                .write_to(&mut owner, Some(selection.clone()))
        );
        assert_eq!(owner.selected(), Some(selection));
    }
}
