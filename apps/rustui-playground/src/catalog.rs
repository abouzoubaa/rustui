#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Coverage {
    Partial,
    Missing,
}

impl Coverage {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Partial => "Partial",
            Self::Missing => "Missing",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CatalogItem {
    pub key: &'static str,
    pub api: &'static str,
    pub coverage: Coverage,
    pub note: &'static str,
    pub sources: &'static str,
}

macro_rules! item {
    ($key:literal, $api:literal, $coverage:ident, $note:literal, $sources:literal) => {
        CatalogItem {
            key: $key,
            api: $api,
            coverage: Coverage::$coverage,
            note: $note,
            sources: $sources,
        }
    };
}

/// The first 41 entries mirror `themes/macos27.json`; app-only gallery entries follow.
const THEME_CATALOG_LEN: usize = 41;
pub const CATALOG: [CatalogItem; 46] = [
    item!(
        "link",
        "Link",
        Partial,
        "Implemented Link; native mouse, keyboard, disabled, focus, and local navigation checks passed. External navigation and screen-reader announcements remain unverified.",
        "docs/ui/macos-27/Text Styles/Header.svg"
    ),
    item!(
        "button",
        "Button",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "text",
        "Text",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Text Styles/Text Styles - Left Aligned - Dark Opaque.svg|docs/ui/macos-27/Text Styles/Text Styles - Left Aligned - Light Opaque.svg"
    ),
    item!(
        "label",
        "Label",
        Partial,
        "Label text and secondary content implemented; icon asset registration pending.",
        "docs/ui/macos-27/Text Styles/Text Styles - Left Aligned - Dark Opaque.svg|docs/ui/macos-27/Text Styles/Text Styles - Left Aligned - Light Opaque.svg"
    ),
    item!(
        "checkbox",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "divider",
        "Divider",
        Partial,
        "Horizontal and vertical Divider implemented and visually checked.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/Menubar.svg"
    ),
    item!(
        "icon",
        "Icon",
        Partial,
        "Named icon wrapper implemented; runtime asset registration and visual acceptance pending.",
        "docs/ui/macos-27/_Magnifying Glass.svg|docs/ui/macos-27/_Clear.svg"
    ),
    item!(
        "image",
        "Image",
        Partial,
        "File and application-asset Image adapter implemented; local PNG visual check passed.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/Utility Window.svg"
    ),
    item!(
        "slider",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Utility Window.svg"
    ),
    item!(
        "toggle",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "picker",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "stepper",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "scroll_view",
        "Layout",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Scrollbar - Vertical.svg|docs/ui/macos-27/Scrollbar - Horizontal.svg"
    ),
    item!(
        "text_field",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Dialog/Save - Large.svg|docs/ui/macos-27/Dialog/Save - Small.svg"
    ),
    item!(
        "search_field_examples",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Dark Examples.svg|docs/ui/macos-27/Light Examples.svg"
    ),
    item!(
        "radio_button",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "combo_box",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Form.svg"
    ),
    item!(
        "form_accessories",
        "—",
        Missing,
        "Catalog reference retained; standalone RustUI does not expose — yet.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "alert_dialog",
        "Alert",
        Partial,
        "Inline Alert uses the GPUI Components adapter with owner-controlled visibility; native visual and accessibility acceptance pending.",
        "docs/ui/macos-27/Alert/Side-by-side.svg|docs/ui/macos-27/Alert/Side-by-side-1.svg"
    ),
    item!(
        "save_dialog",
        "Dialog",
        Partial,
        "RustUI Dialog story demonstrates trigger, confirm, cancel, and visible action results; native GUI acceptance pending.",
        "docs/ui/macos-27/Dialog/Save - Large.svg|docs/ui/macos-27/Dialog/Save - Large-1.svg"
    ),
    item!(
        "menu",
        "Menu",
        Partial,
        "RustUI action menu using the upstream popup; M30 accepted keyboard/pointer actions and disabled callback suppression. Enter dismisses a disabled row; native row identity is positional. M31 native layout and child Button action (0 to 1) were observed; tooltip display and dismissal remain unverified because pointer-only hover dwell was unavailable. No tooltip failure is established. ContextMenu gallery is bundled on this route; native context-menu behavior and owner-removal acceptance remain pending.",
        "docs/ui/macos-27/Menubar.svg|docs/ui/macos-27/Menubar-1.svg"
    ),
    item!(
        "list",
        "List",
        Partial,
        "KeyedList gallery with 1,000 stable IDs, selected-row ID, and reverse, prepend, remove, and reset controls; native acceptance pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/List-1.svg"
    ),
    item!(
        "notification",
        "VStack composition",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Notifications.svg|docs/ui/macos-27/Notifications-1.svg"
    ),
    item!(
        "window_chrome",
        "VStack composition",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Windows/Unified Toolbar + Title.svg|docs/ui/macos-27/Windows/Unified Toolbar + Title-1.svg"
    ),
    item!(
        "toolbar",
        "Toolbar",
        Partial,
        "Ordered Toolbar actions use the GPUI Components adapter; native interaction and accessibility acceptance pending.",
        "docs/ui/macos-27/Windows/Expaned Toolbar + Title.svg|docs/ui/macos-27/Windows/Unified Toolbar + Title.svg"
    ),
    item!(
        "sidebar",
        "Sidebar",
        Partial,
        "Controlled flat Sidebar with stable item IDs, external selection, disabled Settings, reorder/relabel controls, and visible unmatched-selection policy; native focus and keyboard acceptance pending.",
        "docs/ui/macos-27/Windows/Default with Titlebar and Sidebar.svg|docs/ui/macos-27/Windows/Default with Titlebar and Sidebar-1.svg"
    ),
    item!(
        "text_style_guide",
        "VStack composition",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Text Styles/Header.svg|docs/ui/macos-27/Text Styles/Text Styles - Left Aligned - Dark Opaque.svg"
    ),
    item!(
        "utility_window",
        "VStack composition",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Utility Window.svg|docs/ui/macos-27/Utility Window-1.svg"
    ),
    item!(
        "button_group",
        "Layout",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/Windows/Unified Compact Toolbar + Title.svg"
    ),
    item!(
        "color_picker",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Utility Window.svg|docs/ui/macos-27/Utility Window-1.svg"
    ),
    item!(
        "disclosure_control",
        "Disclosure",
        Partial,
        "Controlled GPUI Components Collapsible with bound open state, disabled activation, stable identity, and unmounted collapsed content; native interaction, visual, and accessibility acceptance remain pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/Form.svg"
    ),
    item!(
        "form",
        "VStack composition",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "menu_bar",
        "HStack composition",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Menubar.svg|docs/ui/macos-27/Menubar-1.svg"
    ),
    item!(
        "menu_item",
        "PopupMenuItem",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Menubar.svg|docs/ui/macos-27/Menubar-1.svg"
    ),
    item!(
        "menu_separator",
        "—",
        Missing,
        "Catalog reference retained; standalone RustUI does not expose — yet.",
        "docs/ui/macos-27/Menubar.svg|docs/ui/macos-27/List.svg"
    ),
    item!(
        "outline_view",
        "Tree",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/List-1.svg"
    ),
    item!(
        "search_field",
        "Input",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Search Field.svg|docs/ui/macos-27/List.svg"
    ),
    item!(
        "segmented_control",
        "Tabs",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/Windows/Unified Compact Toolbar + Title.svg"
    ),
    item!(
        "tab_bar",
        "Tabs",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/Utility Window.svg|docs/ui/macos-27/Utility Window-1.svg"
    ),
    item!(
        "table",
        "Table",
        Partial,
        "RustUI baseline demo; interaction/visual acceptance pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/List-1.svg"
    ),
    item!(
        "window_controls",
        "HStack composition",
        Partial,
        "Preview uses current RustUI VStack, HStack, Text, and Button composition; a dedicated adapter is not implemented.",
        "docs/ui/macos-27/Windows/Default with Titlebar and Sidebar.svg|docs/ui/macos-27/Utility Window.svg"
    ),
    item!(
        "navigation_split",
        "NavigationSplit",
        Partial,
        "M137 canonical Playground pointer resize and destination switching passed; keyboard divider, full accessibility, locale, adaptive collapse, and cross-platform acceptance remain pending.",
        "docs/ui/macos-27/Windows/Default with Titlebar and Sidebar.svg|docs/ui/macos-27/Windows/Default with Titlebar and Sidebar-1.svg"
    ),
    item!(
        "offset",
        "Offset",
        Partial,
        "Signed relative positioning demo; native interaction and accessibility acceptance pending.",
        "docs/ui/macos-27/Form.svg"
    ),
    item!(
        "collection",
        "Collection",
        Partial,
        "Virtualized stable-ID row demo; native interaction and lifecycle acceptance pending.",
        "docs/ui/macos-27/List.svg"
    ),
    item!(
        "sheet",
        "Sheet",
        Partial,
        "Trigger-opened GPUI Components side sheet with optional title and body; native GUI acceptance pending.",
        "docs/ui/macos-27/Utility Window.svg"
    ),
    item!(
        "drag_drop",
        "Drag and drop",
        Partial,
        "Typed same-process Copy and Transfer gallery; M142 verified keyboard alternatives and M148 verified target removal during a held drag on macOS.",
        "docs/ui/macos-27/Form.svg"
    ),
];

pub const MISSING_REFERENCES: [&str; 6] = [
    "Dark Search Field · Disabled · Text absent",
    "Dark Search Field · Disabled · Text present",
    "Dark Search Field · Focused · Text absent",
    "Dark Search Field · Focused · Text present",
    "Dark Search Field · Idle · Text absent",
    "Dark Search Field · Idle · Text present",
];

pub fn filtered_indices(query: &str) -> Vec<usize> {
    let query = query.trim().to_ascii_lowercase();
    CATALOG
        .iter()
        .enumerate()
        .filter(|(_, item)| {
            query.is_empty()
                || item.key.contains(&query)
                || item.api.to_ascii_lowercase().contains(&query)
                || item.coverage.label().to_ascii_lowercase().contains(&query)
        })
        .map(|(index, _)| index)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const PACK: &str = include_str!("../../../themes/macos27.json");

    #[test]
    fn catalog_preserves_theme_pack_and_appends_gallery_rows() {
        let document = rustui::theme::parse_theme(PACK).expect("appearance pack parses");
        let components = &document.components;
        assert_eq!(CATALOG.len(), THEME_CATALOG_LEN + 5);
        assert_eq!(components.len(), THEME_CATALOG_LEN);
        for item in CATALOG.iter().take(THEME_CATALOG_LEN) {
            assert!(components.contains_key(item.key), "missing {}", item.key);
        }
        for item in &CATALOG {
            assert!(
                !item.sources.is_empty(),
                "{} has no source action",
                item.key
            );
        }
        assert_eq!(CATALOG[THEME_CATALOG_LEN].key, "navigation_split");
        assert_eq!(CATALOG[THEME_CATALOG_LEN].api, "NavigationSplit");
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 1].key, "offset");
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 1].api, "Offset");
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 1].coverage, Coverage::Partial);
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 2].key, "collection");
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 2].api, "Collection");
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 2].coverage, Coverage::Partial);
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 3].key, "sheet");
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 4].key, "drag_drop");
        assert_eq!(CATALOG[THEME_CATALOG_LEN + 3].api, "Sheet");
        assert_eq!(CATALOG[THEME_CATALOG_LEN].coverage, Coverage::Partial);
        assert!(
            CATALOG[THEME_CATALOG_LEN]
                .note
                .contains("M137 canonical Playground pointer resize")
        );
        assert!(
            CATALOG[THEME_CATALOG_LEN]
                .note
                .contains("keyboard divider")
        );
    }

    #[test]
    fn search_matches_name_api_and_status() {
        assert_eq!(filtered_indices("combo"), vec![16]);
        let link = CATALOG
            .iter()
            .find(|item| item.key == "link")
            .expect("link catalog row");
        assert_eq!(link.coverage, Coverage::Partial);
        assert!(
            link.note
                .contains("native mouse, keyboard, disabled, focus")
        );

        let supported = [
            "button",
            "text",
            "checkbox",
            "slider",
            "toggle",
            "picker",
            "stepper",
            "text_field",
            "search_field_examples",
            "radio_button",
            "combo_box",
            "color_picker",
            "search_field",
            "list",
            "table",
            "outline_view",
            "tab_bar",
            "segmented_control",
            "button_group",
            "scroll_view",
        ];
        for key in supported {
            let item = CATALOG
                .iter()
                .find(|item| item.key == key)
                .expect("catalog key");
            assert_eq!(item.coverage, Coverage::Partial, "{key}");
            if key == "list" {
                assert!(item.note.contains("KeyedList"));
                assert!(item.note.contains("1,000 stable IDs"));
            } else {
                assert_eq!(
                    item.note, "RustUI baseline demo; interaction/visual acceptance pending.",
                    "{key}"
                );
            }
        }
        assert_eq!(filtered_indices("definitely absent"), Vec::<usize>::new());
    }

    #[test]
    fn dark_search_reference_gap_stays_explicit() {
        assert_eq!(MISSING_REFERENCES.len(), 6);
        assert!(
            MISSING_REFERENCES
                .iter()
                .all(|row| row.starts_with("Dark Search Field"))
        );
    }
}
