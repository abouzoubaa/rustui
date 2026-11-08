use super::{Playground, STORIES, StoryHost, bundled_theme, stories};
use rustui::gpui::{self, AppContext};
use rustui::gpui_component::Root;
use rustui::runtime::component_host::ComponentHost;
use rustui::theme::{AppearanceMode, activate_theme};

#[gpui::test]
fn every_registered_story_mounts_draws_and_exposes_its_title(cx: &mut gpui::TestAppContext) {
    let theme = bundled_theme().expect("bundled theme");
    cx.update(|app| {
        rustui::gpui_component::init(app);
        activate_theme(&theme, Some(AppearanceMode::Dark), app).expect("activate theme");
        rustui::localization::activate_localization(
            stories::catalogs(),
            stories::initial_locale(),
            app,
        );
    });
    for (index, story) in STORIES.iter().enumerate() {
        let mut playground = Playground::default();
        playground.story_selection = Some(index);
        playground.catalog_selection = None;
        playground.theme = Some(theme.clone());
        let (_, window) = cx.add_window_view(|window, app| {
            let owner = app.new(|cx| ComponentHost::new(playground, cx));
            Root::new(owner, window, app)
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            let tree = window.debug_a11y_tree_json().expect("accessibility tree");
            let tree: serde_json::Value =
                serde_json::from_str(&tree).expect("valid accessibility tree");
            let nodes = tree["nodes"].as_object().expect("accessibility nodes");
            assert!(
                nodes
                    .values()
                    .any(|node| node["aria"]["label"].as_str() == Some(story.title)
                        && node["view"]
                            .as_str()
                            .is_some_and(|view| view
                                .contains("ComponentHost<rustui_playground::Playground>"))),
                "{} preview title missing from rendered accessibility tree",
                story.title
            );
        });
        let (_, story_window) = cx.add_window_view(|window, app| {
            let owner = app.new(|cx| ComponentHost::new(StoryHost::new(story.id), cx));
            Root::new(owner, window, app)
        });
        story_window.update(|window, app| {
            let _ = window.draw(app);
            let tree = window
                .debug_a11y_tree_json()
                .expect("story accessibility tree");
            let tree: serde_json::Value = serde_json::from_str(&tree).expect("valid story tree");
            let nodes = tree["nodes"]
                .as_object()
                .expect("story accessibility nodes");
            assert!(
                nodes.values().any(|node| node["view"]
                    .as_str()
                    .is_some_and(|view| view.contains("StoryHost"))),
                "{} story rendered no elements",
                story.title
            );
        });
    }
    eprintln!("rendered {} registered Playground stories", STORIES.len());
}
