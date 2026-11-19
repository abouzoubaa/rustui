//! Stateful workloads for the F0 native acceptance and backend comparison.
use rustui::prelude::*;
use rustui::theme::{AppearanceMode, ThemeDocument, activate_theme};
use std::{
    cell::Cell,
    rc::Rc,
    sync::{Arc, OnceLock},
};

const COLLECTION_ROWS: usize = 5_000;
#[cfg(test)]
const EDITOR_LINES: usize = 10_000;
const ACCEPTANCE_TABS: [&str; 6] = [
    "Collection",
    "List",
    "Code editor",
    "Editable grid",
    "Menus & popovers",
    "Appearance",
];

pub fn demo() -> impl View {
    GeometryReader::new("f0-acceptance-space", |size| {
        Component(
            "f0-native-acceptance-gallery",
            F0NativeAcceptanceGallery::default,
        )
        .props(size.height, |owner, height| {
            owner.available_height = *height
        })
    })
}

fn initial_panel() -> String {
    match std::env::var("RUSTUI_F0_PANEL").as_deref() {
        Ok("list" | "keyed") => "lists".into(),
        Ok("editor") => "editor".into(),
        Ok("grid") => "grid".into(),
        _ => "collection".into(),
    }
}

fn viewport_height(available: f32) -> f32 {
    (available - 150.0).max(400.0)
}

#[component]
struct F0NativeAcceptanceGallery {
    #[state(initial_panel())]
    selected_panel: String,
    #[state(600.0)]
    available_height: f32,
    alternate_theme: Rc<Cell<bool>>,
}

fn themes() -> &'static [Arc<ThemeDocument>; 2] {
    static THEMES: OnceLock<[Arc<ThemeDocument>; 2]> = OnceLock::new();
    THEMES.get_or_init(|| {
        let standard = crate::bundled_theme().expect("bundled theme");
        let mut alternate = (*standard).clone();
        alternate.profile = "f0-purple".into();
        alternate.platform_snapshot.id = "f0-purple".into();
        alternate.platform_snapshot.appearance.light.accent = "#8530C0".into();
        alternate.platform_snapshot.appearance.dark.accent = "#C18AFF".into();
        alternate
            .platform_snapshot
            .appearance
            .light
            .selected_row_background = "#E8D4F5".into();
        alternate
            .platform_snapshot
            .appearance
            .dark
            .selected_row_background = "#49325C".into();
        [standard, Arc::new(alternate)]
    })
}

impl F0NativeAcceptanceGallery {
    fn appearance(&self, mode: Option<AppearanceMode>, toggle: bool) -> AppAction {
        let choice = self.alternate_theme.clone();
        AppAction::new(move |app| {
            if toggle {
                choice.set(!choice.get());
            }
            let mode =
                mode.or_else(|| rustui::theme::registry::active_theme(app).map(|theme| theme.mode));
            activate_theme(&themes()[usize::from(choice.get())], mode, app)
                .expect("acceptance themes are valid");
        })
    }
}

impl View for F0NativeAcceptanceGallery {
    fn body(&self) -> impl View {
        let height = viewport_height(self.available_height());
        VStack::new()
            .spacing(10.0)
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(
                        Button("Light", self.appearance(Some(AppearanceMode::Light), false))
                            .id("f0-light"),
                    )
                    .child(
                        Button("Dark", self.appearance(Some(AppearanceMode::Dark), false))
                            .id("f0-dark"),
                    )
                    .child(Button("Switch theme", self.appearance(None, true)).id("f0-theme")),
            )
            .child(
                TabView(
                    [
                        TabItem::new(
                            "collection",
                            ACCEPTANCE_TABS[0],
                            rows_panel("collection", height),
                        ),
                        TabItem::new("lists", ACCEPTANCE_TABS[1], lists_panel(height)),
                        TabItem::new("editor", ACCEPTANCE_TABS[2], editor_panel(height)),
                        TabItem::new("overlays", ACCEPTANCE_TABS[4], overlays_panel()),
                        TabItem::new(
                            "appearance",
                            ACCEPTANCE_TABS[5],
                            super::card_gallery::demo(),
                        ),
                    ],
                    self.selected_panel_binding(),
                )
                .id("f0-native-acceptance-panels")
                .style(TabViewStyle::Segmented),
            )
    }
}

#[derive(Clone)]
struct RowsSnapshot {
    ids: Rc<[usize]>,
    collection: Rc<[CollectionItem<usize>]>,
}
impl PartialEq for RowsSnapshot {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.ids, &other.ids)
    }
}

impl RowsSnapshot {
    fn new(ids: Vec<usize>) -> Self {
        let collection = ids
            .iter()
            .map(|id| CollectionItem::new(row_id(*id), *id).label(row_label(*id)))
            .collect::<Vec<_>>()
            .into();
        Self {
            ids: ids.into(),
            collection,
        }
    }
}

impl Default for RowsSnapshot {
    fn default() -> Self {
        Self::new((0..COLLECTION_ROWS).collect())
    }
}

fn row_id(id: usize) -> String {
    format!("f0-row-{id:04}")
}
fn row_label(id: usize) -> String {
    format!("F0 row {id:04}")
}

#[component]
struct AcceptanceRows {
    #[state(RowsSnapshot::default())]
    rows: RowsSnapshot,
    #[state(None)]
    selected: Option<Identity>,
    #[state(COLLECTION_ROWS)]
    next_id: usize,
    #[state(String::from("collection"))]
    kind: String,
    #[state(400.0)]
    height: f32,
}

impl AcceptanceRows {
    fn add_row(&mut self) {
        self.insert(1, false);
    }
    fn insert_top(&mut self) {
        self.insert(100, true);
    }
    fn insert(&mut self, count: usize, top: bool) {
        let mut ids = self.rows.ids.to_vec();
        let added = self.next_id..self.next_id + count;
        if top {
            ids.splice(0..0, added);
        } else {
            ids.extend(added);
        }
        self.next_id += count;
        self.rows = RowsSnapshot::new(ids);
    }
    fn delete_selected(&mut self) {
        let Some(selected) = self.selected.take() else {
            return;
        };
        let ids = self
            .rows
            .ids
            .iter()
            .copied()
            .filter(|id| row_id(*id) != selected.as_str())
            .collect();
        self.rows = RowsSnapshot::new(ids);
    }
    fn delete_top(&mut self) {
        self.rows = RowsSnapshot::new(self.rows.ids.iter().skip(100).copied().collect());
        if self.selected.as_ref().is_some_and(|selected| {
            !self
                .rows
                .ids
                .iter()
                .any(|id| row_id(*id) == selected.as_str())
        }) {
            self.selected = None;
        }
    }
    fn list_binding(&self) -> Binding<Option<usize>> {
        let rows = self.rows.ids.clone();
        Binding::new_with(
            self.selected
                .as_ref()
                .and_then(|selected| rows.iter().position(|id| row_id(*id) == selected.as_str())),
            rows,
            |owner: &Self, _| {
                owner.selected.as_ref().and_then(|selected| {
                    owner
                        .rows
                        .ids
                        .iter()
                        .position(|id| row_id(*id) == selected.as_str())
                })
            },
            |owner, snapshot, index| {
                let selected = index.and_then(|index| snapshot.get(index)).copied();
                if selected.is_some_and(|id| !owner.rows.ids.contains(&id)) {
                    return false;
                }
                owner.selected = selected.map(|id| row_id(id).into());
                true
            },
        )
    }
    fn keyed_binding(&self) -> Binding<Option<String>> {
        Binding::new(
            self.selected.as_ref().map(ToString::to_string),
            |owner: &Self| owner.selected.as_ref().map(ToString::to_string),
            |owner, value| owner.selected = value.map(Into::into),
        )
    }
}

impl View for AcceptanceRows {
    fn body(&self) -> impl View {
        let rows = self.rows();
        let list = match self.kind().as_str() {
            "list" => AnyView::new(
                List(
                    rows.ids.iter().map(|id| row_label(*id)),
                    self.list_binding(),
                )
                .id("f0-list")
                .height(self.height())
                .row_height(30.0),
            ),
            "keyed" => AnyView::new(
                KeyedList(
                    "f0-keyed-list",
                    rows.ids
                        .iter()
                        .map(|id| KeyedListItem::new(row_id(*id), row_label(*id))),
                    self.keyed_binding(),
                )
                .height(self.height())
                .row_height(30.0),
            ),
            _ => AnyView::new(
                Collection::new(
                    "f0-five-thousand-rows",
                    rows.collection.clone(),
                    self.height(),
                    30.0,
                    |_index, row: &usize| Text(row_label(*row)),
                )
                .label("F0 5,000-row collection")
                .selection(self.selected_binding()),
            ),
        };
        VStack::new()
            .spacing(8.0)
            .child(
                HStack::new()
                    .spacing(8.0)
                    .child(Button("Add row", self.action(Self::add_row)))
                    .child(
                        Button("Delete selected", self.action(Self::delete_selected))
                            .disabled(self.selected().is_none()),
                    )
                    .child(Button("Insert 100 at top", self.action(Self::insert_top)))
                    .child(Button("Delete 100 at top", self.action(Self::delete_top))),
            )
            .child(Text(format!(
                "{} rows · Selected row: {}",
                rows.ids.len(),
                self.selected()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "none".into())
            )))
            .child(crate::f0_frame_capture::measured(self.kind(), list))
    }
}

fn rows_panel(kind: &'static str, height: f32) -> impl View {
    Component(format!("f0-{kind}-rows"), AcceptanceRows::default).props(
        (kind, height),
        |owner, (kind, height)| {
            owner.kind = (*kind).into();
            owner.height = *height;
        },
    )
}

#[component]
struct ListsPanel {
    #[state(if std::env::var("RUSTUI_F0_PANEL").as_deref() == Ok("keyed") { "keyed".into() } else { "list".into() })]
    selected: String,
    #[state(400.0)]
    height: f32,
}

impl View for ListsPanel {
    fn body(&self) -> impl View {
        TabView(
            [
                TabItem::new("list", "RustUI List", rows_panel("list", self.height())),
                TabItem::new("keyed", "KeyedList", rows_panel("keyed", self.height())),
            ],
            self.selected_binding(),
        )
        .id("f0-lists")
        .style(TabViewStyle::Segmented)
    }
}
fn lists_panel(height: f32) -> impl View {
    Component("f0-lists-panel", ListsPanel::default)
        .props((height - 40.0).max(400.0), |owner, height| {
            owner.height = *height
        })
}

fn large_rust_source() -> String {
    let mut source = String::from(
        "// RustUI acceptance workload. Generated deterministically.
",
    );
    for module in 0..400 {
        source.push_str(&format!(r#"
/// A deterministic document processor for batch {module}.
mod batch_{module} {{
    use std::collections::BTreeMap;

    #[derive(Clone, Debug)]
    pub struct Document {{
        pub title: String,
        pub counts: BTreeMap<String, usize>,
    }}

    impl Document {{
        // Unicode commentary: café, 日本語, مرحباً بالعالم، اختبار التمرير.
        pub fn summarize(&self, limit: usize) -> Vec<String> {{
            let mut result = Vec::new();
            for (name, count) in &self.counts {{
                if *count > limit {{
                    result.push(format!("{{name}}: {{count}}"));
                }} else {{
                    match name.as_str() {{
                        "empty" => result.push(String::new()),
                        _ => result.push(self.title.clone()),
                    }}
                }}
            }}
            let message = "A deliberately long diagnostic string includes file paths, quoted values, request identifiers, context about the processing stage, and enough repeated detail to exceed two hundred characters so horizontal scrolling and long-line layout are exercised.";
            result.push(message.to_owned());
            result
        }}
    }}
}}
"#));
    }
    if let Ok(path) = std::env::var("RUSTUI_F0_SOURCE_DUMP") {
        std::fs::write(path, &source).expect("write source fixture");
    }
    source
}

#[component]
struct LargeRustEditor {
    #[state(large_rust_source())]
    source: String,
    #[state(400.0)]
    height: f32,
}
impl View for LargeRustEditor {
    fn body(&self) -> impl View {
        crate::f0_frame_capture::measured(
            "editor",
            CodeEditor("f0-large-rust-source", self.source_binding())
                .language("rust")
                .line_numbers(true)
                .height(self.height())
                .accessibility_label("F0 large Rust source editor"),
        )
    }
}
fn editor_panel(height: f32) -> impl View {
    Component("f0-large-rust-editor", LargeRustEditor::default)
        .props(height, |owner, height| owner.height = *height)
}

#[component]
struct OverlaysAcceptancePanel {
    #[state(String::from("popover"))]
    selected: String,
    #[state(String::from("Rust"))]
    choice: String,
    #[state(String::from("GPUI"))]
    combo: String,
}
impl View for OverlaysAcceptancePanel {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(12.0)
            .child(
                HStack::new()
                    .spacing(12.0)
                    .child(
                        Tooltip(
                            "Tooltip stays attached to this button.",
                            Button("Hover for tooltip", self.action(|_| {})),
                        )
                        .id("f0-tooltip"),
                    )
                    .child(
                        Picker("Select", self.choice_binding(), ["Rust", "Swift", "Kotlin"])
                            .id("f0-select"),
                    )
                    .child(
                        Picker("Picker", self.choice_binding(), ["Rust", "Swift", "Kotlin"])
                            .id("f0-picker"),
                    )
                    .child(
                        ComboBox("ComboBox", self.combo_binding(), ["GPUI", "AppKit", "Skia"])
                            .id("f0-combo"),
                    ),
            )
            .child(
                TabView(
                    [
                        TabItem::new("popover", "Popover", super::popover_gallery::demo()),
                        TabItem::new("menu", "Menu", super::menu_gallery::demo()),
                        TabItem::new(
                            "context-menu",
                            "Context menu",
                            super::context_menu_gallery::demo(),
                        ),
                    ],
                    self.selected_binding(),
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

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn viewport_has_a_minimum_and_grows_with_available_space() {
        assert_eq!(viewport_height(200.0), 400.0);
        assert_eq!(viewport_height(800.0), 650.0);
        assert_eq!(viewport_height(1200.0), 1050.0);
    }
    #[test]
    fn realistic_source_contains_nested_unicode_and_long_lines() {
        let source = large_rust_source();
        assert!(source.lines().count() >= EDITOR_LINES);
        assert!(source.contains("مرح"));
        assert!(source.lines().any(|line| line.len() > 200));
        assert!(source.contains("match name.as_str()"));
        assert_eq!(source, large_rust_source());
    }
    #[test]
    fn mutations_preserve_surviving_selection_and_unique_ids() {
        let mut owner = AcceptanceRows::default();
        owner.selected = Some(row_id(4000).into());
        owner.add_row();
        owner.insert_top();
        owner.delete_top();
        assert_eq!(owner.selected(), Some(row_id(4000).into()));
        assert_eq!(owner.rows.ids.len(), 5001);
        owner.delete_selected();
        assert!(owner.selected().is_none());
        assert_eq!(owner.rows.ids.len(), 5000);
    }
}
