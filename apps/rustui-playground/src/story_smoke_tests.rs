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

fn acceptance_tree(window: &mut gpui::VisualTestContext) -> serde_json::Value {
    window.run_until_parked();
    window.update(|window, app| window.draw(app).clear(app));
    window.run_until_parked();
    window.update(|window, app| {
        window.simulate_next_frame(app);
        window.draw(app).clear(app);
        serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap()
    })
}

fn acceptance_has_label(tree: &serde_json::Value, label: &str) -> bool {
    tree["nodes"].as_object().unwrap().values().any(|node| {
        node["aria"]["label"]
            .as_str()
            .is_some_and(|value| value.contains(label))
    })
}

fn acceptance_select_tab(window: &mut gpui::VisualTestContext, label: &str) {
    click_label(window, label);
    acceptance_tree(window);
}

#[gpui::test]
fn f0_collection_mutations_work_while_scrolled_and_keep_selection(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    click_label(&mut window, "F0 row 0001");
    window.simulate_keystrokes("end");
    acceptance_tree(&mut window);
    let tree = acceptance_tree(&mut window);
    let label = tree["nodes"]
        .as_object()
        .unwrap()
        .values()
        .filter_map(|node| node["aria"]["label"].as_str())
        .find(|label| label.starts_with("F0 row "))
        .expect("scrolled rows")
        .to_owned();
    assert_ne!(label, "F0 row 0000");
    click_label(&mut window, &label);
    acceptance_tree(&mut window);
    for (button, count) in [
        ("Add row", 5001),
        ("Insert 100 at top", 5101),
        ("Delete 100 at top", 5001),
    ] {
        click_label(&mut window, button);
        let tree = acceptance_tree(&mut window);
        assert!(
            acceptance_has_label(&tree, &format!("{count} rows")),
            "{button}"
        );
        assert!(
            acceptance_has_label(&tree, &format!("Selected row: f0-row-{}", &label[7..])),
            "{button} lost selection"
        );
    }
    click_label(&mut window, "Delete selected");
    let tree = acceptance_tree(&mut window);
    assert!(acceptance_has_label(
        &tree,
        "5000 rows · Selected row: none"
    ));
}

#[gpui::test]
fn f0_list_and_keyed_list_each_have_live_mutation_controls(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    acceptance_select_tab(&mut window, "List");
    for kind in ["RustUI List", "KeyedList"] {
        acceptance_select_tab(&mut window, kind);
        assert!(
            acceptance_has_label(&acceptance_tree(&mut window), "5000 rows"),
            "{kind}"
        );
        click_label(&mut window, "F0 row 0001");
        acceptance_tree(&mut window);
        for (button, count) in [
            ("Add row", 5001),
            ("Insert 100 at top", 5101),
            ("Delete 100 at top", 5001),
            ("Delete selected", 5000),
        ] {
            click_label(&mut window, button);
            assert!(
                acceptance_has_label(&acceptance_tree(&mut window), &format!("{count} rows")),
                "{kind}: {button}"
            );
        }
    }
}

#[gpui::test]
fn f0_overlays_offer_tooltip_and_all_dropdown_triggers(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    acceptance_select_tab(&mut window, "Menus & popovers");
    let tree = acceptance_tree(&mut window);
    for label in [
        "Hover for tooltip",
        "Select",
        "Picker",
        "ComboBox",
        "Popover",
        "Menu",
        "Context menu",
    ] {
        assert!(acceptance_has_label(&tree, label), "missing {label}");
    }
}

#[gpui::test]
fn f0_theme_controls_work_with_each_workload_visible(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    for tab in [
        "Collection",
        "List",
        "Code editor",
        "Editable grid",
        "Menus & popovers",
    ] {
        acceptance_select_tab(&mut window, tab);
        for (label, mode) in [
            ("Light", AppearanceMode::Light),
            ("Dark", AppearanceMode::Dark),
        ] {
            let tree = acceptance_tree(&mut window);
            let candidates: Vec<_> = tree["nodes"]
                .as_object()
                .unwrap()
                .values()
                .filter(|n| n["aria"]["label"].as_str() == Some(label))
                .collect();
            let node = candidates
                .iter()
                .max_by(|a, b| {
                    a["bounds"]["y"]
                        .as_f64()
                        .unwrap()
                        .total_cmp(&b["bounds"]["y"].as_f64().unwrap())
                })
                .unwrap();
            let scale = tree["frame"]["scale_factor"].as_f64().unwrap();
            let b = &node["bounds"];
            window.simulate_click(
                gpui::point(
                    gpui::px(
                        ((b["x"].as_f64().unwrap() + b["width"].as_f64().unwrap() / 2.0) / scale)
                            as f32,
                    ),
                    gpui::px(
                        ((b["y"].as_f64().unwrap() + b["height"].as_f64().unwrap() / 2.0) / scale)
                            as f32,
                    ),
                ),
                gpui::Modifiers::default(),
            );
            acceptance_tree(&mut window);
            window.update(|_, app| {
                assert_eq!(
                    rustui::theme::registry::active_theme(app).unwrap().mode,
                    mode,
                    "{tab}"
                )
            });
        }
        click_label(&mut window, "Switch theme");
        acceptance_tree(&mut window);
    }
}

#[gpui::test]
fn f0_tab_switch_keeps_rows_and_editor_text_and_caret(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    click_label(&mut window, "F0 row 0020");
    acceptance_tree(&mut window);
    acceptance_select_tab(&mut window, "Code editor");
    click_label(&mut window, "F0 large Rust source editor");
    window.simulate_keystrokes("cmd-up");
    window.simulate_input("// persisted");
    acceptance_tree(&mut window);
    acceptance_select_tab(&mut window, "Collection");
    assert!(acceptance_has_label(
        &acceptance_tree(&mut window),
        "Selected row: f0-row-0020"
    ));
    acceptance_select_tab(&mut window, "Code editor");
    let before = acceptance_tree(&mut window);
    assert!(
        before["nodes"]
            .as_object()
            .unwrap()
            .values()
            .any(|node| node["aria"]["value"]
                .as_str()
                .is_some_and(|text| text.starts_with("// persisted"))),
        "editor text was not retained"
    );
    // Keyboard navigation enters the preserved editor without positioning its caret.
    window.simulate_keystrokes("tab");
    window.simulate_input("!");
    let tree = acceptance_tree(&mut window);
    assert!(
        tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .any(|node| node["aria"]["value"]
                .as_str()
                .is_some_and(|text| text.starts_with("// persisted!"))),
        "editor text/caret was not retained"
    );
}

#[gpui::test]
fn f0_viewports_follow_window_resize_with_a_400_point_minimum(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    let height = |window: &mut gpui::VisualTestContext| {
        let tree = acceptance_tree(window);
        let scale = tree["frame"]["scale_factor"].as_f64().unwrap();
        tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .find(|node| node["aria"]["label"] == "F0 5,000-row collection")
            .unwrap()["bounds"]["height"]
            .as_f64()
            .unwrap()
            / scale
    };
    let large = height(&mut window);
    assert!(large >= 400.0, "large viewport: {large}");
    window.simulate_resize(gpui::size(gpui::px(1400.0), gpui::px(850.0)));
    let smaller = height(&mut window);
    assert!(
        smaller >= 400.0 && smaller < large,
        "resize: {large} → {smaller}"
    );
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

#[gpui::test]
fn f0_grid_edits_survive_switching_tabs(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    acceptance_select_tab(&mut window, "Editable grid");
    let position = window
        .debug_bounds("editable-grid-cell:product-0:name")
        .expect("first product cell")
        .center();
    window.simulate_click(position, gpui::Modifiers::default());
    acceptance_tree(&mut window);
    window.simulate_keystrokes("enter");
    acceptance_tree(&mut window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("Persisted grid edit");
    window.simulate_keystrokes("enter");
    assert!(acceptance_has_label(
        &acceptance_tree(&mut window),
        "Persisted grid edit"
    ));
    acceptance_select_tab(&mut window, "Collection");
    acceptance_select_tab(&mut window, "Editable grid");
    assert!(acceptance_has_label(
        &acceptance_tree(&mut window),
        "Persisted grid edit"
    ));
}

#[gpui::test]
fn f0_collection_scroll_position_survives_tab_switch(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    click_label(&mut window, "F0 row 0001");
    window.simulate_keystrokes("end escape");
    acceptance_tree(&mut window);
    let visible_labels = |tree: serde_json::Value| {
        let mut labels: Vec<_> = tree["nodes"]
            .as_object()
            .unwrap()
            .values()
            .filter(|node| node["aria"]["role"] == "ListItem")
            .filter_map(|node| node["aria"]["label"].as_str().map(str::to_owned))
            .collect();
        labels.sort();
        labels
    };
    let tree = acceptance_tree(&mut window);

    let before = visible_labels(tree);
    assert!(!before.is_empty());
    assert!(
        !before.iter().any(|label| label == "F0 row 0000"),
        "scroll did not reach a later range"
    );
    acceptance_select_tab(&mut window, "Code editor");
    acceptance_select_tab(&mut window, "Collection");
    assert_eq!(visible_labels(acceptance_tree(&mut window)), before);
}

#[gpui::test]
fn f0_list_keyed_grid_and_editor_resize_with_the_window(cx: &mut gpui::TestAppContext) {
    let mut window = f0_acceptance_window(cx);
    for (tab, subtab, role, label) in [
        ("List", Some("RustUI List"), "ListBox", None),
        ("List", Some("KeyedList"), "ListBox", None),
        ("Editable grid", None, "Grid", Some("Products")),
        (
            "Code editor",
            None,
            "MultilineTextInput",
            Some("F0 large Rust source editor"),
        ),
    ] {
        window.simulate_resize(gpui::size(gpui::px(1400.0), gpui::px(1200.0)));
        acceptance_tree(&mut window);
        acceptance_select_tab(&mut window, tab);
        if let Some(subtab) = subtab {
            acceptance_select_tab(&mut window, subtab);
        }
        let viewport_height = |window: &mut gpui::VisualTestContext| {
            let tree = acceptance_tree(window);
            let scale = tree["frame"]["scale_factor"].as_f64().unwrap();
            let width = tree["frame"]["viewport_size"]["width"].as_f64().unwrap();
            tree["nodes"]
                .as_object()
                .unwrap()
                .values()
                .filter(|node| {
                    node["aria"]["role"] == role
                        && label.is_none_or(|label| node["aria"]["label"] == label)
                        && node["bounds"]["x"].as_f64().unwrap() / scale > width * 0.18
                })
                .map(|node| node["bounds"]["height"].as_f64().unwrap() / scale)
                .max_by(f64::total_cmp)
                .unwrap_or_else(|| panic!("missing {tab}/{subtab:?} viewport"))
        };
        let large = viewport_height(&mut window);
        window.simulate_resize(gpui::size(gpui::px(1400.0), gpui::px(850.0)));
        let small = viewport_height(&mut window);
        assert!(
            small >= 400.0 && small < large,
            "{tab}/{subtab:?}: {large} → {small}"
        );
    }
}
