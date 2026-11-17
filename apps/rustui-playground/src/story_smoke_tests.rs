use super::{CATALOG, Playground, STORIES, StoryHost, bundled_theme, stories};
use rustui::gpui::{self, AppContext};
use rustui::gpui_component::Root;
use rustui::runtime::component_host::ComponentHost;
use rustui::theme::{AppearanceMode, activate_theme};

fn f0_acceptance_window(cx: &mut gpui::TestAppContext) -> gpui::VisualTestContext {
    let theme = bundled_theme().expect("bundled theme");
    cx.update(|app| {
        rustui::gpui_component::init(app);
        activate_theme(&theme, Some(AppearanceMode::Dark), app).expect("activate theme");
    });
    let playground = Playground {
        catalog_selection: CATALOG
            .iter()
            .position(|item| item.key == "f0_native_acceptance"),
        theme: Some(theme),
        ..Playground::default()
    };
    let handle = cx.open_window(
        gpui::size(gpui::px(1400.0), gpui::px(1200.0)),
        |window, cx| {
            let owner = cx.new(|cx| ComponentHost::new(playground, cx));
            Root::new(owner, window, cx)
        },
    );
    let mut window = gpui::VisualTestContext::from_window(handle.into(), cx);
    window.update(|window, app| {
        window.set_a11y_active_for_tests(true);
        window.draw(app).clear(app);
    });
    window
}

fn click_label(window: &mut gpui::VisualTestContext, label: &str) {
    let position = window.update(|window, _| {
        let tree: serde_json::Value =
            serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap();
        let node = tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .find(|node| node["aria"]["label"].as_str() == Some(label))
            .unwrap_or_else(|| panic!("missing accessible label {label:?}"));
        let bounds = &node["bounds"];
        let scale = tree["frame"]["scale_factor"].as_f64().unwrap();
        let center = |origin: &str, extent: &str| {
            gpui::px(
                ((bounds[origin].as_f64().unwrap() + bounds[extent].as_f64().unwrap() / 2.0)
                    / scale) as f32,
            )
        };
        gpui::point(center("x", "width"), center("y", "height"))
    });
    window.simulate_click(position, gpui::Modifiers::default());
}

#[gpui::test]
fn f0_acceptance_collection_accepts_a_click_below_the_old_canvas_limit(
    cx: &mut gpui::TestAppContext,
) {
    let mut window = f0_acceptance_window(cx);
    click_label(&mut window, "F0 row 0020");
    window.run_until_parked();
    window.update(|window, app| {
        window.draw(app).clear(app);
        let tree = window.debug_a11y_tree_json().unwrap();
        assert!(tree.contains("Selected row: f0-row-0020"));
    });
}

#[gpui::test]
fn f0_editor_popover_stays_open_across_idle_frames_and_closes_on_escape(
    cx: &mut gpui::TestAppContext,
) {
    let mut window = f0_acceptance_window(cx);
    let parent = window.update(|window, _| window.window_handle());
    click_label(&mut window, "Menus & popovers");
    window.update(|window, app| window.draw(app).clear(app));
    click_label(&mut window, "Open editor popover");
    let popup = window
        .cx
        .windows()
        .into_iter()
        .find(|handle| *handle != parent)
        .expect("clicking the editor trigger should open a native window");

    window.update(|window, app| window.draw(app).clear(app));
    // A native frame request can deliver callbacks without redrawing a clean window.
    window.update(|window, app| {
        window.simulate_next_frame(app);
        assert!(app.windows().contains(&popup), "closed on the next frame");
        window.simulate_next_frame(app);
        window.draw(app).clear(app);
    });
    assert!(
        window.cx.windows().contains(&popup),
        "closed on the extra draw"
    );

    window.cx.simulate_keystrokes(popup, "escape");
    window.update(|window, app| window.draw(app).clear(app));
    assert_eq!(window.cx.windows(), vec![parent]);
}

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
