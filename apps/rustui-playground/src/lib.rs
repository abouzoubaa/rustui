//! RustUI's component playground, implemented through the public framework API.

mod catalog;
mod preview_transport;
#[cfg(target_family = "wasm")]
mod preview_transport_web;
mod stories;

use catalog::{CATALOG, CatalogItem, MISSING_REFERENCES, filtered_indices};
use rustui::{
    localization::MessageArguments,
    prelude::*,
    theme::{AppearanceMode, ThemeDocument, activate_theme, parse_theme},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::{Arc, OnceLock},
};
use stories::{STORIES, StoryHost, StoryId, filtered_story_indices};

const REPOSITORY_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

#[derive(Clone, PartialEq)]
enum PlaygroundStatus {
    Ready,
    NoReferences,
    Opened { count: usize, key: String },
    OpenFailed { path: String, error: String },
}

#[component]
struct Playground {
    #[state(String::new())]
    query: String,
    #[state(Some(1))]
    catalog_selection: Option<usize>,
    #[state(None)]
    story_selection: Option<usize>,
    #[state(PlaygroundStatus::Ready)]
    status: PlaygroundStatus,
    theme: Option<Arc<ThemeDocument>>,
}

impl Playground {
    fn catalog_binding(&self) -> Binding<Option<usize>> {
        let rows = filtered_catalog_indices(&self.query);
        Binding::new_with(
            self.catalog_selection
                .and_then(|selected| rows.iter().position(|index| *index == selected)),
            rows,
            |owner: &Self, rows: &Vec<usize>| {
                owner
                    .catalog_selection
                    .and_then(|selected| rows.iter().position(|index| *index == selected))
            },
            |owner, rows, row| {
                owner.catalog_selection = row.and_then(|row| rows.get(row).copied());
                if owner.catalog_selection.is_some() {
                    owner.story_selection = None;
                }
                true
            },
        )
    }

    fn story_binding(&self) -> Binding<Option<usize>> {
        let rows = filtered_story_indices(&self.query);
        Binding::new_with(
            self.story_selection
                .and_then(|selected| rows.iter().position(|index| *index == selected)),
            rows,
            |owner: &Self, rows: &Vec<usize>| {
                owner
                    .story_selection
                    .and_then(|selected| rows.iter().position(|index| *index == selected))
            },
            |owner, rows, row| {
                owner.story_selection = row.and_then(|row| rows.get(row).copied());
                if owner.story_selection.is_some() {
                    owner.catalog_selection = None;
                }
                true
            },
        )
    }

    fn item(&self) -> Option<CatalogItem> {
        self.catalog_selection
            .and_then(|index| CATALOG.get(index).copied())
    }

    fn active_story(&self) -> Option<StoryId> {
        self.story_selection
            .and_then(|index| STORIES.get(index))
            .map(|story| story.id)
            .or_else(|| self.item().and_then(|item| story_for(item.key)))
    }

    fn appearance(&self, mode: AppearanceMode) -> AppAction {
        let theme = self
            .theme
            .as_ref()
            .expect("Playground theme initialized")
            .clone();
        AppAction::new(move |cx| {
            if let Err(error) = activate_theme(&theme, Some(mode), cx) {
                eprintln!("Could not switch Playground appearance: {error}");
            }
        })
    }

    fn open_references(&mut self) {
        let Some(item) = self.item() else {
            return;
        };
        let mut opened = 0;
        for source in item.sources.split('|') {
            let path = reference_path(source);
            if !path.is_file() {
                continue;
            }
            match Command::new("open").arg(&path).spawn() {
                Ok(_) => opened += 1,
                Err(error) => {
                    self.status = PlaygroundStatus::OpenFailed {
                        path: path.display().to_string(),
                        error: error.to_string(),
                    };
                    return;
                }
            }
        }
        self.status = if opened == 0 {
            PlaygroundStatus::NoReferences
        } else {
            PlaygroundStatus::Opened {
                count: opened,
                key: item.key.to_owned(),
            }
        };
    }

    fn status_view(&self) -> impl View {
        let (message, arguments) = match self.status() {
            PlaygroundStatus::Ready => ("lab-status-ready", MessageArguments::new()),
            PlaygroundStatus::NoReferences => ("lab-status-no-references", MessageArguments::new()),
            PlaygroundStatus::Opened { count, key } => (
                "lab-status-opened",
                MessageArguments::new()
                    .with("count", count as i64)
                    .with("key", key.clone()),
            ),
            PlaygroundStatus::OpenFailed { path, error } => (
                "lab-status-open-failed",
                MessageArguments::new()
                    .with("path", path.clone())
                    .with("error", error.clone()),
            ),
        };
        LocalizedText(message)
            .with_arguments(arguments)
            .font_size(12.0)
    }

    fn sidebar(&self) -> impl View {
        let catalog_indices = filtered_catalog_indices(&self.query);
        let rows = catalog_indices
            .iter()
            .map(|&index| CATALOG[index].key)
            .collect::<Vec<_>>();
        let coverage_messages = catalog_indices
            .into_iter()
            .map(|index| CATALOG[index].coverage.message_id())
            .collect::<Vec<_>>();
        let mut sidebar = VStack::new()
            .child(
                LocalizedText("lab-sidebar-components")
                    .font_size(12.0)
                    .font_weight(TextWeight::Bold),
            )
            .child(
                TextField("Search components and stories", self.query_binding())
                    .id("playground-search")
                    .localized_placeholder("lab-search-placeholder"),
            )
            .child(
                List(rows, self.catalog_binding())
                    .localized_suffixes(coverage_messages)
                    // Reset the virtualized viewport when filtering so a match cannot
                    // remain hidden beyond the old scroll offset.
                    .id(format!("playground-catalog:{}", self.query))
                    .height(340.0),
            )
            .child(
                LocalizedText("lab-sidebar-framework-demos")
                    .font_size(12.0)
                    .font_weight(TextWeight::Bold),
            )
            .child(
                List(
                    filtered_story_indices(&self.query)
                        .into_iter()
                        .map(|index| STORIES[index].id.label())
                        .collect::<Vec<_>>(),
                    self.story_binding(),
                )
                // Reset the virtualized viewport so filtered matches remain visible.
                .id(format!("playground-stories:{}", self.query))
                .height(220.0),
            );
        if !self.query.trim().is_empty() && !search_has_results(&self.query) {
            sidebar = sidebar.child(LocalizedText("lab-search-no-matches").font_size(12.0));
        }
        Container(ScrollView(sidebar.spacing(12.0)).id("playground-sidebar-scroll"))
            .fill()
            .width(250.0)
            .padding(12.0)
            .background(Surface::Sidebar)
            .border()
    }

    fn preview(&self) -> impl View {
        let selected_story = self
            .story_selection
            .and_then(|index| STORIES.get(index))
            .copied();
        let item = selected_story.is_none().then(|| self.item()).flatten();
        let title = selected_story
            .map(|story| story.title)
            .or_else(|| item.map(|item| item.key))
            .unwrap_or("Choose a component or demo");
        let coverage = item.map_or_else(
            || Text(""),
            |item| LocalizedText(item.coverage.message_id()),
        );
        let note = selected_story.map_or_else(
            || {
                item.map_or_else(
                    || LocalizedText("lab-select-preview"),
                    |item| LocalizedText(item.note_message_id),
                )
            },
            |story| Text(story.summary),
        );
        let catalog_demo = self
            .item()
            .filter(|_| self.story_selection.is_none())
            .and_then(|item| stories::catalog_demo(item.key));
        let active_story = self.active_story();
        let content = match (catalog_demo, active_story) {
            (Some(demo), _) => demo,
            (None, Some(story)) => AnyView::new(
                Component("playground-story-host", move || StoryHost::new(story))
                    .props(story, StoryHost::apply_story),
            ),
            (None, None) => AnyView::new(LocalizedText("lab-demo-pending")),
        };
        Container(
            ScrollView(
                VStack::new()
                    .child(Text(title).font_size(20.0).font_weight(TextWeight::Bold))
                    .child(coverage.font_size(12.0))
                    .child(note.font_size(13.0))
                    .child(active_story.map(|_| {
                        VStack! {
                            LocalizedText("lab-navigation-locales")
                            stories::LocaleSelector::new()
                        }
                        .spacing(8.0)
                    }))
                    .child(
                        ScrollView(Container(content).padding(12.0).radius(10.0).border())
                            .axis(ScrollAxis::Both)
                            .scrollbars(true)
                            .height(360.0)
                            .id(format!("playground-component-canvas-{title}")),
                    )
                    .spacing(12.0),
            )
            .axis(ScrollAxis::Vertical)
            .scrollbars(true)
            .id("playground-preview-scroll"),
        )
        .fill()
        .grow()
        .padding(16.0)
        .background(Surface::Background)
    }

    fn references(&self) -> impl View {
        let mut panel = VStack::new().child(
            LocalizedText("lab-macos-references")
                .font_size(16.0)
                .font_weight(TextWeight::Bold),
        );
        if let Some(item) = self.item() {
            if cfg!(target_family = "wasm") {
                for source in item.sources.split('|') {
                    let name = reference_filename(source);
                    panel = panel.child(
                        Link(format!("Open {name}"), reference_route(source))
                            .localized_label_with_arguments(
                                "lab-open-reference",
                                MessageArguments::new().with("name", name),
                            ),
                    );
                }
            } else {
                for source in item.sources.split('|') {
                    let name = reference_filename(source);
                    let available = reference_path(source).is_file();
                    panel = panel.child(
                        LocalizedText(if available {
                            "lab-reference-available"
                        } else {
                            "lab-reference-missing"
                        })
                        .with_arguments(MessageArguments::new().with("name", name))
                        .font_size(12.0),
                    );
                }
                let available = item
                    .sources
                    .split('|')
                    .any(|source| reference_path(source).is_file());
                panel = panel.child(
                    Button("Open references", self.action(Self::open_references))
                        .id("playground-open-references")
                        .localized_label("lab-open-references")
                        .disabled(!available),
                );
            }
        } else {
            panel = panel.child(LocalizedText("lab-select-reference").font_size(12.0));
        }
        if MISSING_REFERENCES.is_empty() {
            panel = panel.child(LocalizedText("lab-no-source-gaps").font_size(11.0));
        } else {
            panel = panel.child(
                LocalizedText("lab-known-source-gaps")
                    .font_size(13.0)
                    .font_weight(TextWeight::Semibold),
            );
            for missing in MISSING_REFERENCES {
                panel = panel.child(Text(missing).font_size(11.0));
            }
        }
        Container(ScrollView(panel.spacing(12.0)).id("playground-references-scroll"))
            .fill()
            .width(300.0)
            .padding(12.0)
            .background(Surface::Background)
    }
}

fn catalog_search_catalogs() -> &'static rustui::localization::Catalogs {
    static CATALOGS: OnceLock<rustui::localization::Catalogs> = OnceLock::new();
    CATALOGS.get_or_init(stories::catalogs)
}

fn filtered_catalog_indices(query: &str) -> Vec<usize> {
    let query = query.trim().to_lowercase();
    let mut matches = filtered_indices(&query);
    if query.is_empty() {
        return matches;
    }

    let catalogs = catalog_search_catalogs();
    for (index, item) in CATALOG.iter().enumerate() {
        if catalogs.iter().any(|(locale, _)| {
            catalogs
                .resolve(locale, item.coverage.message_id())
                .to_lowercase()
                .contains(&query)
        }) {
            matches.push(index);
        }
    }
    matches.sort_unstable();
    matches.dedup();
    matches
}

fn search_has_results(query: &str) -> bool {
    !filtered_catalog_indices(query).is_empty() || !filtered_story_indices(query).is_empty()
}

impl View for Playground {
    fn body(&self) -> impl View {
        Container(
            VStack::new()
                .child(
                    Container(
                        HStack::new()
                            .child(
                                Container(
                                    VStack::new()
                                        .child(
                                            LocalizedText("lab-component-playground-title")
                                                .font_size(20.0)
                                                .font_weight(TextWeight::Bold),
                                        )
                                        .child(
                                            LocalizedText("lab-catalog-summary")
                                                .with_arguments(
                                                    MessageArguments::new()
                                                        .with("catalog_count", CATALOG.len() as i64)
                                                        .with("demo_count", STORIES.len() as i64),
                                                )
                                                .font_size(12.0),
                                        )
                                        .spacing(4.0),
                                )
                                .grow(),
                            )
                            .child(
                                Button("Light", self.appearance(AppearanceMode::Light))
                                    .id("playground-light")
                                    .localized_label("lab-appearance-light"),
                            )
                            .child(
                                Button("Dark", self.appearance(AppearanceMode::Dark))
                                    .id("playground-dark")
                                    .localized_label("lab-appearance-dark"),
                            )
                            .spacing(8.0)
                            .alignment(VerticalAlignment::Center),
                    )
                    .padding(12.0)
                    .border(),
                )
                .child(
                    HStack::new()
                        .child(self.sidebar())
                        .child(self.preview())
                        .child(self.references())
                        .grow(),
                )
                .child(Container(self.status_view()).padding(8.0).border())
                .fill(),
        )
        .fill()
        .background(Surface::Background)
    }
}

fn story_for(key: &str) -> Option<StoryId> {
    match key {
        "button" => Some(StoryId::Button),
        "text" | "text_style_guide" => Some(StoryId::Text),
        "button_group" | "scroll_view" => Some(StoryId::Layout),
        "checkbox"
        | "slider"
        | "toggle"
        | "picker"
        | "stepper"
        | "text_field"
        | "search_field_examples"
        | "radio_button"
        | "combo_box"
        | "color_picker"
        | "search_field"
        | "form_accessories" => Some(StoryId::Input),
        "list" => Some(StoryId::List),
        "table" => Some(StoryId::Table),
        "drawer" => Some(StoryId::Drawer),
        "graph_view" => Some(StoryId::GraphView),
        "node_editor" => Some(StoryId::NodeEditor),
        "outline_view" => Some(StoryId::Outline),
        "tab_bar" | "segmented_control" => Some(StoryId::Tabs),
        _ => None,
    }
}

fn reference_path(source: &str) -> PathBuf {
    reference_path_from_root(&reference_root(), source)
}

fn reference_root() -> PathBuf {
    let executable = std::env::current_exe().ok();
    reference_root_for_executable(executable.as_deref(), Path::new(REPOSITORY_ROOT))
}

fn reference_root_for_executable(executable: Option<&Path>, repository_root: &Path) -> PathBuf {
    let packaged_root = executable
        .and_then(Path::parent)
        .map(|directory| directory.join("../Resources/references"));
    if let Some(root) = packaged_root.filter(|root| root.is_dir()) {
        return root;
    }

    repository_root.join("apps/rustui-playground/references")
}

fn reference_path_from_root(reference_root: &Path, source: &str) -> PathBuf {
    let flattened = reference_root.join(reference_filename(source));
    if flattened.is_file() {
        return flattened;
    }

    let source = source.strip_prefix("docs/ui/macos-27/").unwrap_or(source);
    reference_root.join(source)
}

fn reference_filename(source: &str) -> &str {
    Path::new(source)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(source)
}

fn reference_route(source: &str) -> String {
    let mut route = String::from("./references/");
    for byte in reference_filename(source).bytes() {
        if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
            route.push(byte as char);
        } else {
            route.push_str(&format!("%{byte:02X}"));
        }
    }
    route
}

/// Run the canonical component playground through RustUI's application runtime.
pub fn run() {
    let theme = Arc::new(
        parse_theme(include_str!("../../../themes/macos27.json"))
            .expect("RustUI macOS 27 theme must be valid"),
    );
    let startup_theme = theme.clone();
    let catalogs = stories::catalogs();
    let initial_locale = stories::initial_locale();
    let playground = Playground {
        theme: Some(theme),
        ..Playground::default()
    };
    let assets = rustui::component_assets();
    let assets = rustui::with_embedded_asset(
        assets,
        "assets/playground-sample.png",
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../assets/playground-sample.png"
        )),
    );

    let app = RustUI::new()
        .assets(assets)
        .window_size(1320.0, 820.0)
        .blurred_window()
        .title("RustUI Component Playground")
        .on_launch(move |cx| {
            activate_theme(&startup_theme, Some(AppearanceMode::Dark), cx)
                .expect("macOS 27 theme must resolve and apply");
            rustui::localization::activate_localization(catalogs, initial_locale, cx);
        });
    #[cfg(target_family = "wasm")]
    if let Some(reference) = stories::browser_reference(&preview_transport_web::requested_profile())
    {
        app.run(reference);
        return;
    }
    app.run(playground);
}

#[cfg(test)]
mod tests {
    use super::*;
    use catalog::Coverage;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn reference_and_status_messages_resolve_in_supported_locales() {
        let catalogs = stories::catalogs();
        for (tag, ready, missing, open_reference, available, absent) in [
            (
                "en",
                "Ready",
                "No local SVG",
                "Open Form.svg",
                "Available · Form.svg",
                "Missing · Form.svg",
            ),
            (
                "fr",
                "Prêt",
                "Aucune référence SVG",
                "Ouvrir Form.svg",
                "Disponible · Form.svg",
                "Manquant · Form.svg",
            ),
            (
                "ar",
                "جاهز",
                "لا تتوفر مراجع SVG",
                "فتح Form.svg",
                "متوفر · Form.svg",
                "غير متوفر · Form.svg",
            ),
        ] {
            let locale = rustui::localization::Locale::parse(tag).expect("supported locale");
            assert!(
                catalogs
                    .resolve(&locale, "lab-status-ready")
                    .contains(ready)
            );
            assert!(
                catalogs
                    .resolve(&locale, "lab-status-no-references")
                    .contains(missing)
            );
            let opened = catalogs
                .format(
                    &locale,
                    "lab-status-opened",
                    &MessageArguments::new()
                        .with("count", 2_i64)
                        .with("key", "button"),
                )
                .expect("opening status formats");
            assert!(opened.contains("button"));
            let failed = catalogs
                .format(
                    &locale,
                    "lab-status-open-failed",
                    &MessageArguments::new()
                        .with("path", "/tmp/reference.svg")
                        .with("error", "unavailable"),
                )
                .expect("error status formats");
            assert!(failed.contains("/tmp/reference.svg"));
            assert!(failed.contains("unavailable"));
            assert_eq!(
                catalogs
                    .format(
                        &locale,
                        "lab-open-reference",
                        &MessageArguments::new().with("name", "Form.svg"),
                    )
                    .expect("reference link formats"),
                open_reference
            );
            for (message, expected) in [
                ("lab-reference-available", available),
                ("lab-reference-missing", absent),
            ] {
                assert_eq!(
                    catalogs
                        .format(
                            &locale,
                            message,
                            &MessageArguments::new().with("name", "Form.svg"),
                        )
                        .expect("reference state formats"),
                    expected
                );
            }
        }
    }

    #[test]
    fn catalog_summary_formats_counts_in_supported_locales() {
        let catalogs = stories::catalogs();
        for (tag, label) in [
            ("en", "catalog rows"),
            ("fr", "éléments du catalogue"),
            ("ar", "صفًا في الدليل"),
        ] {
            let locale = rustui::localization::Locale::parse(tag).expect("supported locale");
            let summary = catalogs
                .format(
                    &locale,
                    "lab-catalog-summary",
                    &MessageArguments::new()
                        .with("catalog_count", CATALOG.len() as i64)
                        .with("demo_count", STORIES.len() as i64),
                )
                .expect("catalog summary formats");
            assert!(
                summary.contains(&rustui::localization::format_integer(
                    CATALOG.len() as i64,
                    &locale
                )),
                "{tag}: {summary}"
            );
            assert!(
                summary.contains(&rustui::localization::format_integer(
                    STORIES.len() as i64,
                    &locale
                )),
                "{tag}: {summary}"
            );
            assert!(summary.contains(label), "{tag}: {summary}");
        }
    }

    static NEXT_TEMP_DIRECTORY: AtomicUsize = AtomicUsize::new(0);

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "rustui-playground-reference-test-{}-{}",
                std::process::id(),
                NEXT_TEMP_DIRECTORY.fetch_add(1, Ordering::Relaxed)
            ));
            std::fs::create_dir_all(&path).expect("create test directory");
            Self(path)
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn bundled_references_take_precedence_over_the_checkout() {
        let temporary = TempDirectory::new();
        let executable = temporary
            .0
            .join("RustUI Playground.app/Contents/MacOS/rustui-playground");
        let bundled_references = temporary
            .0
            .join("RustUI Playground.app/Contents/Resources/references");
        let checkout_root = temporary.0.join("checkout");
        let checkout_references = checkout_root.join("apps/rustui-playground/references");
        std::fs::create_dir_all(executable.parent().expect("executable directory"))
            .expect("create bundle executable directory");
        std::fs::create_dir_all(&bundled_references).expect("create bundled references");
        std::fs::create_dir_all(&checkout_references).expect("create checkout references");
        std::fs::write(bundled_references.join("Example.svg"), "bundle")
            .expect("write bundled reference");
        std::fs::write(checkout_references.join("Example.svg"), "checkout")
            .expect("write checkout reference");

        let resolved_root = reference_root_for_executable(Some(&executable), &checkout_root);

        assert_eq!(
            resolved_root.canonicalize().expect("resolve bundle path"),
            bundled_references
                .canonicalize()
                .expect("resolve expected bundle path")
        );
        assert_eq!(
            std::fs::read(reference_path_from_root(&resolved_root, "Example.svg"))
                .expect("read selected bundled reference"),
            b"bundle"
        );
        assert_eq!(
            std::fs::read(reference_path_from_root(
                &resolved_root,
                "docs/ui/macos-27/Example.svg"
            ))
            .expect("read catalog-style bundled reference"),
            b"bundle"
        );
    }

    #[test]
    fn development_reference_lookup_falls_back_to_the_checkout() {
        let temporary = TempDirectory::new();
        let checkout_root = temporary.0.join("checkout");
        let checkout_references = checkout_root.join("apps/rustui-playground/references");
        std::fs::create_dir_all(&checkout_references).expect("create checkout references");
        let executable = checkout_root.join("target/debug/rustui-playground");

        assert_eq!(
            reference_root_for_executable(Some(&executable), &checkout_root),
            checkout_references
        );
        assert_eq!(
            reference_root_for_executable(None, &checkout_root),
            checkout_references
        );
    }

    #[test]
    fn nested_catalog_reference_resolves_to_packaged_flat_filename() {
        let temporary = TempDirectory::new();
        let bundled = temporary.0.join("Header.svg");
        std::fs::write(&bundled, "<svg/>").expect("write packaged reference");

        assert_eq!(
            reference_path_from_root(&temporary.0, "docs/ui/macos-27/Text Styles/Header.svg"),
            bundled
        );
    }

    #[test]
    fn nested_catalog_reference_resolves_to_packaged_nested_path() {
        let temporary = TempDirectory::new();
        let nested = temporary.0.join("Text Styles/Header.svg");
        std::fs::create_dir_all(nested.parent().expect("nested reference directory"))
            .expect("create nested reference directory");
        std::fs::write(&nested, "<svg/>").expect("write nested packaged reference");

        assert_eq!(
            reference_path_from_root(&temporary.0, "docs/ui/macos-27/Text Styles/Header.svg"),
            nested
        );
    }

    #[test]
    fn catalog_references_resolve_and_web_routes_encode_filename_segments() {
        let references = Path::new(env!("CARGO_MANIFEST_DIR")).join("references");
        for item in CATALOG {
            for source in item.sources.split('|') {
                assert!(
                    reference_path_from_root(&references, source).is_file(),
                    "unresolved catalog reference for {}: {source}",
                    item.key
                );
            }
        }

        assert_eq!(
            reference_route("docs/ui/macos-27/Utility Window.svg"),
            "./references/Utility%20Window.svg"
        );
        assert_eq!(
            reference_route("docs/ui/macos-27/Unified Compact Toolbar + Title.svg"),
            "./references/Unified%20Compact%20Toolbar%20%2B%20Title.svg"
        );
    }

    #[test]
    fn filtered_catalog_selection_uses_original_row_and_clears_demo_selection() {
        let mut app = Playground::default();
        app.query = "combo".into();
        app.story_selection = Some(9);
        assert!(app.catalog_binding().write_to(&mut app, Some(0)));
        assert_eq!(app.item().map(|item| item.key), Some("combo_box"));
        assert_eq!(app.active_story(), Some(StoryId::Input));
        assert_eq!(app.story_selection, None);
    }

    #[test]
    fn localized_coverage_label_filters_and_selects_catalog_row() {
        let catalogs = catalog_search_catalogs();
        let french = rustui::localization::Locale::parse("fr").expect("French locale");
        let query = catalogs.resolve(&french, Coverage::Partial.message_id());
        assert_eq!(query, "Partiel");
        assert!(filtered_indices(query).is_empty());

        let rows = filtered_catalog_indices(query);
        let button_index = CATALOG
            .iter()
            .position(|item| item.key == "button")
            .expect("button catalog row");
        let slider_index = CATALOG
            .iter()
            .position(|item| item.key == "slider")
            .expect("slider catalog row");
        let button_row = rows
            .iter()
            .position(|index| *index == button_index)
            .expect("French coverage search should include the button row");
        let slider_row = rows
            .iter()
            .position(|index| *index == slider_index)
            .expect("French coverage search should include the slider row");
        let mut app = Playground {
            query: query.into(),
            ..Playground::default()
        };

        assert!(app.catalog_binding().write_to(&mut app, Some(button_row)));
        assert_eq!(app.item().map(|item| item.key), Some("button"));
        assert!(app.catalog_binding().write_to(&mut app, Some(slider_row)));
        assert_eq!(app.item().map(|item| item.key), Some("slider"));

        let arabic = rustui::localization::Locale::parse("ar").expect("Arabic locale");
        let query = catalogs.resolve(&arabic, Coverage::Partial.message_id());
        assert_eq!(query, "جزئي");
        assert!(filtered_indices(query).is_empty());

        let rows = filtered_catalog_indices(query);
        let button_row = rows
            .iter()
            .position(|index| *index == button_index)
            .expect("Arabic coverage search should include the button row");
        let slider_row = rows
            .iter()
            .position(|index| *index == slider_index)
            .expect("Arabic coverage search should include the slider row");
        let mut app = Playground {
            query: query.into(),
            ..Playground::default()
        };

        assert!(app.catalog_binding().write_to(&mut app, Some(button_row)));
        assert_eq!(app.item().map(|item| item.key), Some("button"));
        assert!(app.catalog_binding().write_to(&mut app, Some(slider_row)));
        assert_eq!(app.item().map(|item| item.key), Some("slider"));
    }

    #[test]
    fn search_empty_state_counts_catalog_story_and_localized_matches() {
        assert!(search_has_results(""));
        assert!(search_has_results("button"));
        assert!(search_has_results("NavigationStack"));
        assert!(!search_has_results("no matching component or demo"));

        let catalogs = catalog_search_catalogs();
        let french = rustui::localization::Locale::parse("fr").expect("French locale");
        let localized_query = catalogs.resolve(&french, Coverage::Partial.message_id());
        assert!(filtered_indices(localized_query).is_empty());
        assert!(search_has_results(localized_query));
    }

    #[test]
    fn framework_selection_reaches_navigation_and_clears_catalog_selection() {
        let mut app = Playground::default();
        let index = STORIES
            .iter()
            .position(|story| story.id == StoryId::Navigation)
            .unwrap();
        assert!(app.story_binding().write_to(&mut app, Some(index)));
        assert_eq!(app.active_story(), Some(StoryId::Navigation));
        assert!(app.item().is_none());
    }

    #[test]
    fn stale_story_binding_keeps_the_row_mapping_from_its_rendered_filter() {
        let mut app = Playground {
            query: "navigation".into(),
            ..Playground::default()
        };
        let binding = app.story_binding();
        let old_filter_first_row = filtered_story_indices(&app.query)[0];
        app.query = "input".into();

        assert!(binding.write_to(&mut app, Some(0)));
        assert_eq!(app.story_selection, Some(old_filter_first_row));
        assert_eq!(app.active_story(), Some(STORIES[old_filter_first_row].id));
    }

    #[test]
    fn text_framework_story_is_reachable_for_semantics_examples() {
        let mut app = Playground::default();
        let index = STORIES
            .iter()
            .position(|story| story.id == StoryId::Text)
            .unwrap();
        assert!(app.story_binding().write_to(&mut app, Some(index)));
        assert_eq!(app.active_story(), Some(StoryId::Text));
        assert!(app.item().is_none());
    }

    #[test]
    fn text_style_guide_catalog_item_reaches_its_dedicated_gallery() {
        let mut app = Playground::default();
        let index = CATALOG
            .iter()
            .position(|item| item.key == "text_style_guide")
            .expect("text style guide catalog item");

        assert!(app.catalog_binding().write_to(&mut app, Some(index)));
        assert_eq!(app.item().map(|item| item.key), Some("text_style_guide"));
        assert_eq!(app.active_story(), Some(StoryId::Text));
        assert!(stories::catalog_demo("text_style_guide").is_some());
    }

    #[test]
    fn form_accessories_catalog_item_reaches_partial_input_story() {
        let mut app = Playground::default();
        let index = CATALOG
            .iter()
            .position(|item| item.key == "form_accessories")
            .expect("form accessories catalog item");

        assert_eq!(CATALOG[index].coverage, catalog::Coverage::Partial);
        assert!(app.catalog_binding().write_to(&mut app, Some(index)));
        assert_eq!(app.item().map(|item| item.key), Some("form_accessories"));
        assert_eq!(app.active_story(), Some(StoryId::Input));
    }

    #[test]
    fn implemented_display_catalog_entries_reach_existing_demos() {
        for key in ["link", "label", "icon", "image", "menu", "notification"] {
            assert!(CATALOG.iter().any(|item| item.key == key));
            assert!(stories::catalog_demo(key).is_some(), "missing demo: {key}");
        }
    }

    #[test]
    fn every_catalog_item_reaches_a_demo_or_framework_story() {
        for item in &CATALOG {
            assert!(
                stories::catalog_demo(item.key).is_some() || story_for(item.key).is_some(),
                "catalog item has no preview route: {}",
                item.key
            );
        }
    }
}
