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

    pub const fn message_id(self) -> &'static str {
        match self {
            Self::Partial => "lab-catalog-coverage-partial",
            Self::Missing => "lab-catalog-coverage-missing",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CatalogItem {
    pub key: &'static str,
    pub api: &'static str,
    pub coverage: Coverage,
    pub note: &'static str,
    pub note_message_id: &'static str,
    pub sources: &'static str,
}

macro_rules! item {
    ($key:literal, $api:literal, $coverage:ident, $note:literal, $sources:literal) => {
        CatalogItem {
            key: $key,
            api: $api,
            coverage: Coverage::$coverage,
            note: $note,
            note_message_id: concat!("lab-catalog-note-", $key),
            sources: $sources,
        }
    };
}

/// The first 41 entries mirror `themes/macos27.json`; app-only gallery entries follow.
#[cfg(test)]
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
        "Public RustUI Button story with a live count, Increment and Reset actions, and a disabled Unavailable state; native visual, keyboard, and accessibility acceptance remain pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "text",
        "Text",
        Partial,
        "Standalone public RustUI Text specimen with heading, body, and wrapping samples; the framework Text story remains a catalog-message localization gallery with interpolation and plural/ordinal/select examples. Native visual, keyboard, and accessibility acceptance remains pending.",
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
        "Routes to the shared Text inputs story, which renders six Search and Token Field examples plus all six Dark Search Field combinations (Disabled, Focused, and Idle with text absent or present). Coverage remains Partial pending native visual and accessibility acceptance.",
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
        "Input",
        Partial,
        "Public-API Form Accessories composition with a filter field and detail, info, and action controls; native visual and accessibility acceptance remain pending.",
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
        "Public RustUI feedback gallery with action and kind-specific Toasts, EmptyState, ErrorState, and Skeleton examples; native visual and accessibility acceptance remain pending.",
        "docs/ui/macos-27/Notifications.svg|docs/ui/macos-27/Notifications-1.svg"
    ),
    item!(
        "window_chrome",
        "VStack composition",
        Partial,
        "Owner-state public RustUI VStack, HStack, Text, and Button composition; creating or closing a sample document updates its title and body. Illustrative only: no native titlebar or system-window action; dedicated adapter and native acceptance remain open.",
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
        "Text Style Guide",
        Partial,
        "Public RustUI Text examples for reference sizes and regular/bold weights; the separate framework Text story contains the localization gallery. Native visual, keyboard, and accessibility acceptance remains pending.",
        "docs/ui/macos-27/Text Styles/Header.svg|docs/ui/macos-27/Text Styles/Text Styles - Left Aligned - Dark Opaque.svg"
    ),
    item!(
        "utility_window",
        "VStack composition",
        Partial,
        "Preview mirrors the Utility Panel title and its five named category tabs; native visual, keyboard, and accessibility acceptance remain pending.",
        "docs/ui/macos-27/Utility Window.svg|docs/ui/macos-27/Utility Window-1.svg"
    ),
    item!(
        "button_group",
        "Layout",
        Partial,
        "Public Button/HStack composition with three related actions; native visual, keyboard, and accessibility acceptance remain pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/Windows/Unified Compact Toolbar + Title.svg"
    ),
    item!(
        "color_picker",
        "Input",
        Partial,
        "Public RustUI ColorPicker with a retained color binding in the shared Text inputs story alongside DatePicker and other controls; native visual, keyboard, and accessibility acceptance remain pending.",
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
        "Public Form Accessories composition",
        Partial,
        "Reuses the public Form Accessories composition with a filter field and detail, info, and action controls; native visual and accessibility acceptance remain pending.",
        "docs/ui/macos-27/Form.svg|docs/ui/macos-27/Form-1.svg"
    ),
    item!(
        "menu_bar",
        "Public Menu composition",
        Partial,
        "Six interactive RustUI Menu controls dispatch owner actions; Finder and date/time are labels. Native menu-bar chrome, icons, and visual/accessibility acceptance remain pending.",
        "docs/ui/macos-27/Menubar.svg|docs/ui/macos-27/Menubar-1.svg"
    ),
    item!(
        "menu_item",
        "PopupMenuItem",
        Partial,
        "Routes to the public RustUI Menu gallery, which demonstrates MenuItem actions, separators, disabled-item handling, and removing/restoring the menu owner; native visual and accessibility acceptance remain pending.",
        "docs/ui/macos-27/Menubar.svg|docs/ui/macos-27/Menubar-1.svg"
    ),
    item!(
        "menu_separator",
        "MenuItem::separator",
        Partial,
        "Menu separator is demonstrated in the existing menu action gallery; standalone native visual and accessibility acceptance pending.",
        "docs/ui/macos-27/Menubar.svg|docs/ui/macos-27/List.svg"
    ),
    item!(
        "outline_view",
        "Tree",
        Partial,
        "Public RustUI Tree gallery with a controlled stable-ID hierarchy, expansion, branch removal/restoration, and reorder/relabel controls; native interaction and accessibility acceptance remain pending.",
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
        "Finder view modes are represented by mutually exclusive public Button controls; RustUI has no dedicated Segmented Control renderer, and native visual/accessibility acceptance remains pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/Windows/Unified Compact Toolbar + Title.svg"
    ),
    item!(
        "tab_bar",
        "Tabs",
        Partial,
        "Public RustUI TabView with Overview, Activity, and Settings; native visual, keyboard, and accessibility acceptance remain pending.",
        "docs/ui/macos-27/Utility Window.svg|docs/ui/macos-27/Utility Window-1.svg"
    ),
    item!(
        "table",
        "Table",
        Partial,
        "Public RustUI Table story combines controlled row selection, reordering, removal/restoration, and a DataGrid accessibility example; native visual, keyboard, and accessibility acceptance remain pending.",
        "docs/ui/macos-27/List.svg|docs/ui/macos-27/List-1.svg"
    ),
    item!(
        "window_controls",
        "HStack composition",
        Partial,
        "Illustrative public Button composition shows active red/yellow/green and inactive traffic-light states; clicks only change the preview state. RustUI has no public native window-control/titlebar renderer, so system actions and native chrome are not represented.",
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

pub const MISSING_REFERENCES: [&str; 0] = [];

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
        assert!(CATALOG[THEME_CATALOG_LEN].note.contains("keyboard divider"));
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
            } else if key == "button_group" {
                assert!(item.note.contains("Public Button/HStack composition"));
                assert!(item.note.contains(
                    "native visual, keyboard, and accessibility acceptance remain pending"
                ));
            } else if key == "segmented_control" {
                assert!(
                    item.note
                        .contains("mutually exclusive public Button controls")
                );
                assert!(
                    item.note
                        .contains("native visual/accessibility acceptance remains pending")
                );
            } else if key == "search_field_examples" {
                assert!(item.note.contains("shared Text inputs story"));
                assert!(item.note.contains("all six Dark Search Field combinations"));
                assert!(item.note.contains("native visual and accessibility acceptance"));
            } else {
                assert!(!item.note.trim().is_empty(), "missing note for {key}");
            }
        }
        assert_eq!(filtered_indices("definitely absent"), Vec::<usize>::new());
    }

    #[test]
    fn catalog_notes_and_coverage_resolve_in_each_playground_locale() {
        let catalogs = crate::stories::catalogs();
        let english = rustui::localization::Locale::parse("en")
            .expect("English catalog locale should parse");
        for (identifier, partial, missing) in [
            ("en", "Partial", "Missing"),
            ("fr", "Partiel", "Manquant"),
            ("ar", "جزئي", "غير متوفر"),
        ] {
            let locale = rustui::localization::Locale::parse(identifier)
                .expect("catalog locale should parse");
            assert_eq!(
                catalogs.resolve(&locale, Coverage::Partial.message_id()),
                partial
            );
            assert_eq!(
                catalogs.resolve(&locale, Coverage::Missing.message_id()),
                missing
            );
            for item in &CATALOG {
                let note = catalogs.resolve(&locale, item.note_message_id);
                let english_note = catalogs.resolve(&english, item.note_message_id);
                assert!(!note.trim().is_empty(), "empty note for {} in {identifier}", item.key);
                assert_ne!(note, item.note_message_id, "missing note for {} in {identifier}", item.key);
                if identifier != "en" {
                    assert_ne!(note, english_note, "untranslated note for {} in {identifier}", item.key);
                }
            }
        }
    }

    #[test]
    fn dark_search_reference_combinations_are_covered_by_the_input_story() {
        assert!(MISSING_REFERENCES.is_empty());
    }

    #[test]
    fn search_field_examples_routes_to_input_story_with_reference_gaps() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "search_field_examples")
            .expect("search field examples catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("shared Text inputs story"));
        assert!(item.note.contains("all six Dark Search Field combinations"));
        assert!(item.note.contains("native visual and accessibility acceptance"));
        assert_eq!(
            crate::story_for(item.key),
            Some(crate::stories::StoryId::Input)
        );
    }

    #[test]
    fn text_style_guide_catalog_describes_its_public_typography_gallery() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "text_style_guide")
            .expect("text style guide catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert_eq!(item.api, "Text Style Guide");
        assert!(item.note.contains("Public RustUI Text examples"));
        assert!(
            item.note
                .contains("reference sizes and regular/bold weights")
        );
        assert!(
            item.note
                .contains("separate framework Text story contains the localization gallery")
        );
        assert!(
            item.note
                .contains("Native visual, keyboard, and accessibility acceptance remains pending")
        );
        assert!(crate::stories::catalog_demo(item.key).is_some());
        assert_eq!(
            crate::story_for(item.key),
            Some(crate::stories::StoryId::Text)
        );
    }

    #[test]
    fn text_catalog_describes_its_specimen_and_preserves_localization_route() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "text")
            .expect("text catalog row");
        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Standalone public RustUI Text specimen"));
        assert!(item.note.contains("heading, body, and wrapping samples"));
        assert!(
            item.note
                .contains("framework Text story remains a catalog-message localization gallery")
        );
        assert!(
            item.note
                .contains("Native visual, keyboard, and accessibility acceptance remains pending")
        );
        assert_eq!(
            crate::story_for(item.key),
            Some(crate::stories::StoryId::Text)
        );
        assert!(crate::stories::catalog_demo(item.key).is_some());
    }

    #[test]
    fn menu_item_catalog_describes_the_public_menu_gallery() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "menu_item")
            .expect("menu item catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("public RustUI Menu gallery"));
        assert!(item.note.contains("MenuItem actions, separators"));
        assert!(item.note.contains("disabled-item handling"));
        assert!(item.note.contains("removing/restoring the menu owner"));
        assert!(item.note.contains("native visual and accessibility acceptance remain pending"));
        assert!(crate::stories::catalog_demo(item.key).is_some());
    }

    #[test]
    fn notification_catalog_describes_the_public_feedback_composition() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "notification")
            .expect("notification catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Public RustUI feedback gallery"));
        assert!(item.note.contains("action and kind-specific Toasts"));
        assert!(item.note.contains("EmptyState, ErrorState, and Skeleton"));
        assert!(item.note.contains("native visual and accessibility acceptance remain pending"));
        assert!(crate::stories::catalog_demo(item.key).is_some());
    }

    #[test]
    fn tab_bar_catalog_describes_the_public_tab_view_composition() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "tab_bar")
            .expect("tab bar catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Public RustUI TabView"));
        assert!(item.note.contains("Overview, Activity, and Settings"));
        assert!(
            item.note
                .contains("native visual, keyboard, and accessibility acceptance remain pending")
        );
        assert!(crate::stories::catalog_demo(item.key).is_some());
    }

    #[test]
    fn button_catalog_describes_the_public_button_story() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "button")
            .expect("button catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Public RustUI Button story"));
        assert!(
            item.note
                .contains("live count, Increment and Reset actions")
        );
        assert!(item.note.contains("disabled Unavailable state"));
        assert!(
            item.note
                .contains("native visual, keyboard, and accessibility acceptance remain pending")
        );
        assert_eq!(
            crate::story_for(item.key),
            Some(crate::stories::StoryId::Button)
        );
    }

    #[test]
    fn color_picker_catalog_describes_the_shared_input_story() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "color_picker")
            .expect("color picker catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Public RustUI ColorPicker"));
        assert!(item.note.contains("retained color binding"));
        assert!(item.note.contains("shared Text inputs story"));
        assert!(item.note.contains("native visual, keyboard, and accessibility acceptance remain pending"));
        assert_eq!(
            crate::story_for(item.key),
            Some(crate::stories::StoryId::Input)
        );
    }

    #[test]
    fn outline_view_catalog_describes_the_tree_story() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "outline_view")
            .expect("outline view catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Public RustUI Tree gallery"));
        assert!(item.note.contains("controlled stable-ID hierarchy"));
        assert!(item.note.contains("branch removal/restoration"));
        assert!(item.note.contains("reorder/relabel controls"));
        assert!(item.note.contains("native interaction and accessibility acceptance remain pending"));
        assert_eq!(crate::story_for(item.key), Some(crate::stories::StoryId::Tree));
    }

    #[test]
    fn table_catalog_describes_the_controlled_table_story() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "table")
            .expect("table catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Public RustUI Table story"));
        assert!(item.note.contains("controlled row selection, reordering"));
        assert!(item.note.contains("removal/restoration"));
        assert!(item.note.contains("DataGrid accessibility example"));
        assert!(item.note.contains("native visual, keyboard, and accessibility acceptance remain pending"));
        assert_eq!(crate::story_for(item.key), Some(crate::stories::StoryId::Table));
    }

    #[test]
    fn window_controls_catalog_documents_the_illustrative_button_composition() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "window_controls")
            .expect("window controls catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(item.note.contains("Illustrative public Button composition"));
        assert!(item.note.contains("active red/yellow/green"));
        assert!(item.note.contains("inactive traffic-light states"));
        assert!(item.note.contains("clicks only change the preview state"));
        assert!(item.note.contains("no public native window-control/titlebar renderer"));
        assert!(item.note.contains("system actions and native chrome are not represented"));
    }

    #[test]
    fn segmented_control_catalog_documents_the_public_button_composition() {
        let item = CATALOG
            .iter()
            .find(|item| item.key == "segmented_control")
            .expect("segmented control catalog row");

        assert_eq!(item.coverage, Coverage::Partial);
        assert!(
            item.note
                .contains("mutually exclusive public Button controls")
        );
        assert!(item.note.contains("no dedicated Segmented Control renderer"));
        assert!(
            item.note
                .contains("native visual/accessibility acceptance remains pending")
        );
    }
}
