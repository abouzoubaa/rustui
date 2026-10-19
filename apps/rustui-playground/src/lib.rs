//! RustUI's component playground, implemented through the public framework API.

mod catalog;
mod stories;

use catalog::{CATALOG, CatalogItem, MISSING_REFERENCES, filtered_indices};
use rustui::{
    prelude::*,
    theme::{AppearanceMode, ThemeDocument, activate_theme, parse_theme},
};
use std::{
    path::{Path, PathBuf},
    process::Command,
    sync::Arc,
};
use stories::{STORIES, StoryHost, StoryId};

const REPOSITORY_ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

#[component]
struct Playground {
    #[state(String::new())]
    query: String,
    #[state(Some(1))]
    catalog_selection: Option<usize>,
    #[state(None)]
    story_selection: Option<usize>,
    #[state(String::from("Ready · macOS 27 · new RustUI framework"))]
    status: String,
    theme: Option<Arc<ThemeDocument>>,
}

impl Playground {
    fn catalog_binding(&self) -> Binding<Option<usize>> {
        Binding::new(
            self.visible_catalog_selection(),
            Self::visible_catalog_selection,
            |owner, row| {
                owner.catalog_selection =
                    row.and_then(|row| filtered_indices(&owner.query).get(row).copied());
                if owner.catalog_selection.is_some() {
                    owner.story_selection = None;
                }
            },
        )
    }

    fn visible_catalog_selection(&self) -> Option<usize> {
        let selected = self.catalog_selection?;
        filtered_indices(&self.query)
            .iter()
            .position(|index| *index == selected)
    }

    fn story_binding(&self) -> Binding<Option<usize>> {
        Binding::new(
            self.story_selection,
            |owner: &Self| owner.story_selection,
            |owner, row| {
                owner.story_selection = row.filter(|row| *row < STORIES.len());
                if owner.story_selection.is_some() {
                    owner.catalog_selection = None;
                }
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
                    self.status = format!("Could not open {}: {error}", path.display());
                    return;
                }
            }
        }
        self.status = if opened == 0 {
            "No local SVG references available".into()
        } else {
            format!("Requested opening {opened} reference(s) for {}", item.key)
        };
    }

    fn sidebar(&self) -> impl View {
        let rows = filtered_indices(&self.query)
            .into_iter()
            .map(|index| {
                let item = CATALOG[index];
                format!("{} · {}", item.key, item.coverage.label())
            })
            .collect::<Vec<_>>();
        Container(
            ScrollView(
                VStack::new()
                    .child(
                        Text("COMPONENTS")
                            .font_size(12.0)
                            .font_weight(TextWeight::Bold),
                    )
                    .child(
                        TextField(
                            format!("Search {} components", CATALOG.len()),
                            self.query_binding(),
                        )
                        .id("playground-search"),
                    )
                    .child(
                        List(rows, self.catalog_binding())
                            // Reset the virtualized viewport when filtering so a match cannot
                            // remain hidden beyond the old scroll offset.
                            .id(format!("playground-catalog:{}", self.query))
                            .height(340.0),
                    )
                    .child(
                        Text("FRAMEWORK DEMOS")
                            .font_size(12.0)
                            .font_weight(TextWeight::Bold),
                    )
                    .child(
                        List(
                            STORIES.iter().map(|story| story.id.label()),
                            self.story_binding(),
                        )
                        .id("playground-stories")
                        .height(220.0),
                    )
                    .spacing(12.0),
            )
            .id("playground-sidebar-scroll"),
        )
        .fill()
        .width(250.0)
        .padding(12.0)
        .background(Surface::Sidebar)
        .border()
    }

    fn preview(&self) -> impl View {
        let title = self
            .story_selection
            .and_then(|index| STORIES.get(index))
            .map(|story| story.title)
            .or_else(|| self.item().map(|item| item.key))
            .unwrap_or("Choose a component or demo");
        let note = self
            .story_selection
            .and_then(|index| STORIES.get(index))
            .map(|story| story.summary)
            .or_else(|| self.item().map(|item| item.note))
            .unwrap_or("Use the Playground sidebar to select a preview.");
        let catalog_demo = self
            .item()
            .filter(|_| self.story_selection.is_none())
            .and_then(|item| stories::catalog_demo(item.key));
        let content = match (catalog_demo, self.active_story()) {
            (Some(demo), _) => demo,
            (None, Some(story)) => AnyView::new(
                Component("playground-story-host", move || StoryHost::new(story))
                    .props(story, StoryHost::apply_story),
            ),
            (None, None) => AnyView::new(Text(
                "A RustUI demonstration for this catalog item is still pending.",
            )),
        };
        Container(
            ScrollView(
                VStack::new()
                    .child(Text(title).font_size(20.0).font_weight(TextWeight::Bold))
                    .child(Text(note).font_size(13.0))
                    .child(Container(content).padding(12.0).radius(10.0).border())
                    .spacing(12.0),
            )
            .id("playground-preview-scroll"),
        )
        .fill()
        .grow()
        .padding(16.0)
        .background(Surface::Background)
    }

    fn references(&self) -> impl View {
        let mut panel = VStack::new().child(
            Text("macOS 27 references")
                .font_size(16.0)
                .font_weight(TextWeight::Bold),
        );
        if let Some(item) = self.item() {
            for source in item.sources.split('|') {
                let name = Path::new(source)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(source);
                let available = reference_path(source).is_file();
                panel = panel.child(
                    Text(format!(
                        "{} · {name}",
                        if available { "Available" } else { "Missing" }
                    ))
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
                    .disabled(!available),
            );
        } else {
            panel = panel.child(
                Text("Select a component catalog row to inspect its SVG references.")
                    .font_size(12.0),
            );
        }
        panel = panel.child(
            Text("Known source gaps")
                .font_size(13.0)
                .font_weight(TextWeight::Semibold),
        );
        for missing in MISSING_REFERENCES {
            panel = panel.child(Text(missing).font_size(11.0));
        }
        Container(ScrollView(panel.spacing(12.0)).id("playground-references-scroll"))
            .fill()
            .width(300.0)
            .padding(12.0)
            .background(Surface::Panel)
            .border()
    }
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
                                            Text("RustUI Component Playground")
                                                .font_size(20.0)
                                                .font_weight(TextWeight::Bold),
                                        )
                                        .child(
                                            Text(format!(
                                                "{} catalog rows · {} framework demos · macOS 27",
                                                CATALOG.len(),
                                                STORIES.len()
                                            ))
                                            .font_size(12.0),
                                        )
                                        .spacing(4.0),
                                )
                                .grow(),
                            )
                            .child(
                                Button("Light", self.appearance(AppearanceMode::Light))
                                    .id("playground-light"),
                            )
                            .child(
                                Button("Dark", self.appearance(AppearanceMode::Dark))
                                    .id("playground-dark"),
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
                .child(
                    Container(Text(self.status()).font_size(12.0))
                        .padding(8.0)
                        .border(),
                )
                .fill(),
        )
        .fill()
        .background(Surface::Background)
    }
}

fn story_for(key: &str) -> Option<StoryId> {
    match key {
        "button" => Some(StoryId::Button),
        "text" => Some(StoryId::Text),
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
        | "search_field" => Some(StoryId::Input),
        "list" => Some(StoryId::List),
        "table" => Some(StoryId::Table),
        "outline_view" => Some(StoryId::Tree),
        "tab_bar" | "segmented_control" => Some(StoryId::Tabs),
        _ => None,
    }
}

fn reference_path(source: &str) -> PathBuf {
    Path::new(REPOSITORY_ROOT)
        .join("apps/rustui-playground/references")
        .join(source.strip_prefix("docs/ui/macos-27/").unwrap_or(source))
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
    RustUI::new()
        .window_size(1320.0, 820.0)
        .blurred_window()
        .title("RustUI Component Playground")
        .on_launch(move |cx| {
            activate_theme(&startup_theme, Some(AppearanceMode::Dark), cx)
                .expect("macOS 27 theme must resolve and apply");
            rustui::localization::activate_localization(catalogs, initial_locale, cx);
        })
        .run(playground);
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn implemented_display_catalog_entries_reach_existing_demos() {
        for key in ["link", "label", "icon", "image", "menu"] {
            assert!(CATALOG.iter().any(|item| item.key == key));
            assert!(stories::catalog_demo(key).is_some(), "missing demo: {key}");
        }
        assert!(stories::catalog_demo("notification").is_none());
    }
}
