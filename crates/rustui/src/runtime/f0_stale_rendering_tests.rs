//! F0-5 regression checks: ordinary RustUI updates must be visible without a forced refresh.
use std::rc::Rc;

use gpui::{AnyElement, Entity, Pixels, Point, SharedString, point, px};
use rustui_core::Binding;
use serde_json::Value;

use crate::{
    animation::transition::Transition,
    components::data::editable_data_grid::{DataGridColumn, EditableDataGrid},
    prelude::{
        Collection, CollectionItem, Environment, EnvironmentReader, Keyed, Menu, MenuItem, Text,
        TextField, VStack,
    },
    runtime::{AppAction, ComponentHost},
    view::{EmptyView, RenderContext, View},
};

#[derive(Clone, Debug, PartialEq, Eq)]
struct GridRow {
    id: &'static str,
    value: String,
}

struct ProbeOwner {
    owner_value: String,
    environment_value: String,
    list_values: Vec<String>,
    field_value: String,
    grid_rows: Vec<GridRow>,
    show_transition: bool,
}

fn read_field(owner: &ProbeOwner) -> String {
    owner.field_value.clone()
}

fn write_field(owner: &mut ProbeOwner, value: String) {
    owner.field_value = value;
}

fn read_grid(owner: &ProbeOwner) -> Vec<GridRow> {
    owner.grid_rows.clone()
}

fn write_grid(owner: &mut ProbeOwner, rows: Vec<GridRow>) {
    owner.grid_rows = rows;
}

impl View for ProbeOwner {
    fn body(&self) -> impl View {
        let environment = Environment(
            self.environment_value.clone(),
            EnvironmentReader(|value: Option<&String>| {
                Text(format!(
                    "environment:{}",
                    value.map_or("missing", String::as_str)
                ))
                .id("f0-environment-output")
            }),
        );
        let list_items: Rc<[CollectionItem<String>]> = self
            .list_values
            .iter()
            .map(|value| CollectionItem::new(value.clone(), value.clone()).label(value.clone()))
            .collect::<Vec<_>>()
            .into();
        let collection = Collection::new("f0-list", list_items, 100.0, 24.0, |_, value| {
            Text(value.clone())
        })
        .label("F0 list");
        let field = TextField(
            "F0 text input",
            Binding::new(self.field_value.clone(), read_field, write_field),
        )
        .id("f0-text-input");
        let grid = EditableDataGrid(
            "F0 grid",
            Binding::new(self.grid_rows.clone(), read_grid, write_grid),
            |row: &GridRow| SharedString::from(row.id),
            vec![DataGridColumn::text(
                "value",
                "Grid value",
                |row: &GridRow| row.value.clone(),
                |value| Ok(value.to_owned()),
                |row, value| row.value = value,
            )],
        )
        .id("f0-grid");
        let menu = Menu(
            "F0 actions",
            [MenuItem::action(
                "f0-run",
                "F0 menu action",
                AppAction::new(|_| {}),
            )],
        )
        .id("f0-menu");

        let mut stack = VStack::new()
            .child(Text(self.owner_value.clone()).id("f0-owner-output"))
            .child(environment)
            .child(collection)
            .child(field)
            .child(grid)
            .child(menu);
        if self.show_transition {
            stack = stack.child(
                Keyed("f0-transition", Text("transition inserted")).transition(Transition::fade()),
            );
        }
        stack
    }
}

struct ThemeRevisionOutput;

impl View for ThemeRevisionOutput {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let revision = crate::theme::registry::active_theme(context.app())
            .map(|theme| theme.revision())
            .unwrap_or_default();
        Text(format!("theme revision:{revision}")).into_element(context)
    }
}

struct ThemeOwner;

impl View for ThemeOwner {
    fn body(&self) -> impl View {
        ThemeRevisionOutput
    }
}

struct MenuOwner;

impl View for MenuOwner {
    fn body(&self) -> impl View {
        Menu(
            "F0 menu trigger",
            [MenuItem::action(
                "f0-item",
                "F0 popup item",
                AppAction::new(|_| {}),
            )],
        )
        .id("f0-menu-trigger")
    }
}

fn activate_test_theme(cx: &mut gpui::TestAppContext) {
    let theme = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    cx.update(|app| {
        gpui_component::init(app);
        crate::theme::activate_theme(&theme, Some(crate::theme::AppearanceMode::Dark), app)
            .unwrap();
    });
}

fn draw_tree(window: &mut gpui::VisualTestContext) -> Value {
    window.update(|window, app| {
        let _ = window.draw(app);
        serde_json::from_str(
            &window
                .debug_a11y_tree_json()
                .expect("active accessibility tree"),
        )
        .unwrap()
    })
}

fn tree_text(tree: &Value) -> String {
    tree.to_string()
}

fn point_for_label(window: &mut gpui::VisualTestContext, label: &str) -> Point<Pixels> {
    let tree = draw_tree(window);
    let node = tree["nodes"]
        .as_object()
        .unwrap()
        .values()
        .find(|node| node["aria"]["label"] == label)
        .unwrap_or_else(|| panic!("accessibility node {label:?} missing: {tree}"));
    let bounds = &node["bounds"];
    let scale = tree["frame"]["scale_factor"].as_f64().unwrap();
    point(
        px(
            ((bounds["x"].as_f64().unwrap() + bounds["width"].as_f64().unwrap() / 2.0) / scale)
                as f32,
        ),
        px(
            ((bounds["y"].as_f64().unwrap() + bounds["height"].as_f64().unwrap() / 2.0) / scale)
                as f32,
        ),
    )
}

fn next_frame(window: &mut gpui::VisualTestContext) -> Value {
    window.update(|window, app| window.simulate_next_frame(app));
    draw_tree(window)
}

fn setup_probe(
    cx: &mut gpui::TestAppContext,
) -> (
    Entity<ComponentHost<ProbeOwner>>,
    &mut gpui::VisualTestContext,
) {
    activate_test_theme(cx);
    cx.add_window_view(|_, app| {
        ComponentHost::new(
            ProbeOwner {
                owner_value: "owner:initial".into(),
                environment_value: "initial".into(),
                list_values: vec!["row:alpha".into()],
                field_value: "field:initial".into(),
                grid_rows: vec![GridRow {
                    id: "a",
                    value: "grid:initial".into(),
                }],
                show_transition: false,
            },
            app,
        )
    })
}

#[gpui::test]
fn f0_stale_rendering_owner_environment_list_and_transition_matrix(cx: &mut gpui::TestAppContext) {
    let (owner, window) = setup_probe(cx);
    let initial = draw_tree(window);
    assert!(tree_text(&initial).contains("owner:initial"));
    assert!(tree_text(&initial).contains("environment:initial"));
    assert!(tree_text(&initial).contains("row:alpha"));
    assert!(!tree_text(&initial).contains("transition inserted"));

    // Each row names a RustUI owner update path. The next ordinary frame must
    // expose its new output; this test never calls Window::refresh.
    let cases = [
        "owner update",
        "environment update",
        "list insert",
        "transition insert",
    ];
    for case in cases {
        owner.update(window, |host, cx| {
            let owner = host.component_mut();
            match case {
                "owner update" => owner.owner_value = "owner:updated".into(),
                "environment update" => owner.environment_value = "updated".into(),
                "list insert" => owner.list_values.push("row:beta".into()),
                "transition insert" => owner.show_transition = true,
                _ => unreachable!(),
            }
            cx.notify();
        });
        let tree = draw_tree(window);
        let output = tree_text(&tree);
        match case {
            "owner update" => assert!(output.contains("owner:updated"), "{case}: {output}"),
            "environment update" => {
                assert!(output.contains("environment:updated"), "{case}: {output}")
            }
            "list insert" => assert!(output.contains("row:beta"), "{case}: {output}"),
            "transition insert" => {
                assert!(output.contains("transition inserted"), "{case}: {output}")
            }
            _ => unreachable!(),
        }
    }

    owner.update(window, |host, cx| {
        host.component_mut()
            .list_values
            .retain(|value| value != "row:alpha");
        cx.notify();
    });
    let tree = draw_tree(window);
    assert!(tree_text(&tree).contains("row:beta"));
    assert!(!tree_text(&tree).contains("row:alpha"));
}

#[gpui::test]
fn f0_stale_rendering_theme_change_invalidates_visible_output(cx: &mut gpui::TestAppContext) {
    activate_test_theme(cx);
    let light = crate::theme::parse_theme(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../themes/macos27.json"
    )))
    .unwrap();
    let (_owner, window) = cx.add_window_view(|_, app| ComponentHost::new(ThemeOwner, app));
    let before = tree_text(&draw_tree(window));
    assert!(before.contains("theme revision:1"), "{before}");
    window.update(|_, app| {
        crate::theme::activate_theme(&light, Some(crate::theme::AppearanceMode::Light), app)
            .unwrap();
    });
    let after = tree_text(&draw_tree(window));
    assert!(after.contains("theme revision:2"), "{after}");
}

#[gpui::test]
fn f0_stale_rendering_text_input_binding_updates_without_forced_refresh(
    cx: &mut gpui::TestAppContext,
) {
    let (owner, window) = setup_probe(cx);
    let _ = draw_tree(window);
    let input = point_for_label(window, "F0 text input");
    window.simulate_click(input, Default::default());
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("field:typed");
    let tree = next_frame(window);
    assert!(tree_text(&tree).contains("field:typed"), "{tree}");
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().field_value, "field:typed");
    });
}

#[gpui::test]
fn f0_stale_rendering_grid_edit_updates_cell_without_forced_refresh(cx: &mut gpui::TestAppContext) {
    let (owner, window) = setup_probe(cx);
    let _ = draw_tree(window);
    let cell = window
        .debug_bounds("editable-grid-cell:a:value")
        .expect("RustUI grid cell bounds");
    window.simulate_click(cell.center(), Default::default());
    let _ = next_frame(window);
    window.simulate_keystrokes("enter");
    let _ = next_frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("grid:edited");
    window.simulate_keystrokes("enter");
    let tree = next_frame(window);
    assert!(tree_text(&tree).contains("grid:edited"), "{tree}");
    window.update(|_, app| {
        assert_eq!(
            owner.read(app).component().grid_rows[0].value,
            "grid:edited"
        );
    });
}

#[gpui::test]
fn f0_stale_rendering_popup_menu_open_and_close_updates_tree(cx: &mut gpui::TestAppContext) {
    activate_test_theme(cx);
    let (_owner, window) = cx.add_window_view(|_, app| ComponentHost::new(MenuOwner, app));
    let initial = tree_text(&draw_tree(window));
    assert!(!initial.contains("\"role\":\"Menu\""));
    let trigger = point_for_label(window, "F0 menu trigger");
    window.simulate_click(trigger, Default::default());
    window.cx.run_until_parked();
    let opened = tree_text(&next_frame(window));
    assert!(opened.contains("\"role\":\"Menu\""), "{opened}");
    assert!(opened.contains("\"role\":\"MenuItem\""), "{opened}");
    window.simulate_keystrokes("escape");
    window.cx.run_until_parked();
    let closed = tree_text(&next_frame(window));
    assert!(!closed.contains("\"role\":\"Menu\""), "{closed}");
}
