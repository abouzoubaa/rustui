use super::*;
use crate::runtime::ComponentHost;
use gpui::{Entity, Render};

#[derive(Clone, Debug, PartialEq, Eq)]
struct Row {
    id: &'static str,
    name: String,
    quantity: i32,
    note: String,
}

struct Owner {
    rows: Vec<Row>,
    writes: usize,
    note_editable: bool,
}

impl View for Owner {
    fn body(&self) -> impl View {
        EditableDataGrid(
            "Products",
            Binding::new(
                self.rows.clone(),
                |owner: &Self| owner.rows.clone(),
                |owner, rows| {
                    owner.rows = rows;
                    owner.writes += 1;
                },
            ),
            |row: &Row| row.id.into(),
            vec![
                DataGridColumn::text(
                    "name",
                    "Name",
                    |row: &Row| row.name.clone(),
                    |value| Ok(value.to_owned()),
                    |row, value| {
                        if value == "change-id" {
                            row.id = "changed";
                        }
                        row.name = value;
                    },
                )
                .validate(|value| {
                    if value.trim().is_empty() {
                        Err("Name required".into())
                    } else {
                        Ok(())
                    }
                })
                .sortable_by(|a: &Row, b: &Row| a.name.cmp(&b.name)),
                DataGridColumn::text(
                    "quantity",
                    "Quantity",
                    |row: &Row| row.quantity,
                    |value| {
                        value
                            .parse::<i32>()
                            .map_err(|_| "Enter an integer".to_owned())
                    },
                    |row, value| row.quantity = value,
                )
                .validate(|value| {
                    if value.starts_with('-') {
                        Err("Must be nonnegative".into())
                    } else {
                        Ok(())
                    }
                })
                .sortable_by(|a: &Row, b: &Row| a.quantity.cmp(&b.quantity)),
                {
                    let column = DataGridColumn::text(
                        "note",
                        "Note",
                        |row: &Row| row.note.clone(),
                        |value| Ok(value.to_owned()),
                        |row, value| row.note = value,
                    );
                    if self.note_editable {
                        column
                    } else {
                        column.read_only()
                    }
                },
            ],
        )
        .id("test-editable-grid")
    }
}

struct Content {
    owner: Entity<ComponentHost<Owner>>,
    outside: FocusHandle,
}

impl Render for Content {
    fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl gpui::IntoElement {
        div().flex().flex_col().child(self.owner.clone()).child(
            div()
                .id("outside")
                .track_focus(&self.outside)
                .tab_stop(true)
                .h(px(25.0))
                .child("Outside"),
        )
    }
}

fn setup(
    cx: &mut gpui::TestAppContext,
) -> (
    Entity<ComponentHost<Owner>>,
    &mut gpui::VisualTestContext,
    FocusHandle,
) {
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
    let outside = cx.update(|app| app.focus_handle().tab_stop(true));
    let slot = Rc::new(RefCell::new(None));
    let save = slot.clone();
    let (_, window) = cx.add_window_view({
        let outside = outside.clone();
        move |window, cx| {
            let owner = cx.new(|cx| {
                ComponentHost::new(
                    Owner {
                        rows: vec![
                            Row {
                                id: "a",
                                name: "Alpha".into(),
                                quantity: 1,
                                note: "fixed A".into(),
                            },
                            Row {
                                id: "b",
                                name: "Beta".into(),
                                quantity: 2,
                                note: "fixed B".into(),
                            },
                        ],
                        writes: 0,
                        note_editable: false,
                    },
                    cx,
                )
            });
            *save.borrow_mut() = Some(owner.clone());
            let content = cx.new(|_| Content { owner, outside });
            gpui_component::Root::new(content, window, cx)
        }
    });
    let owner = slot.borrow_mut().take().unwrap();
    draw(window);
    window.update(|window, app| {
        window.activate_window();
        let grid = owner
            .read(app)
            .retained_entity_for_test::<GridState<Row>>(
                vec![],
                Identity::from("test-editable-grid"),
            )
            .unwrap();
        let focus = grid.read(app).focus.clone();
        focus.focus(window, app);
    });
    draw(window);
    (owner, window, outside)
}

fn draw(window: &mut gpui::VisualTestContext) {
    window.update(|window, app| {
        window.refresh();
        window.draw(app).clear(app);
    });
    window.run_until_parked();
}

fn frame(window: &mut gpui::VisualTestContext) {
    draw(window);
    window.update(|window, app| window.simulate_next_frame(app));
    window.run_until_parked();
    draw(window);
}

fn grid(owner: &Entity<ComponentHost<Owner>>, app: &App) -> Entity<GridState<Row>> {
    owner
        .read(app)
        .retained_entity_for_test::<GridState<Row>>(vec![], Identity::from("test-editable-grid"))
        .unwrap()
}

#[gpui::test]
fn enter_commits_once_and_escape_cancels(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| assert!(grid(&owner, app).read(app).edit.is_some()));
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("Changed");
    draw(window);
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "Changed");
        assert_eq!(owner.read(app).component().writes, 1);
        assert!(grid(&owner, app).read(app).edit.is_none());
    });
    window.simulate_keystrokes("f2");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("Discarded");
    draw(window);
    window.simulate_keystrokes("escape");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "Changed");
        assert_eq!(owner.read(app).component().writes, 1);
        assert!(grid(&owner, app).read(app).edit.is_none());
    });
}

#[gpui::test]
fn invalid_draft_blocks_commit_and_tab_then_correction_commits(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("right");
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("-1");
    frame(window);
    assert!(
        window
            .debug_bounds("validation-error:Must be nonnegative")
            .is_some()
    );
    window.simulate_keystrokes("tab");
    frame(window);
    window.update(|_, app| {
        let state = grid(&owner, app);
        assert!(state.read(app).edit.is_some());
        assert_eq!(owner.read(app).component().rows[0].quantity, 1);
        assert_eq!(owner.read(app).component().writes, 0);
    });
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("3");
    frame(window);
    assert!(
        window
            .debug_bounds("validation-error:Must be nonnegative")
            .is_none()
    );
    window.simulate_keystrokes("tab");
    frame(window);
    window.update(|_, app| {
        let state = grid(&owner, app);
        assert!(state.read(app).edit.is_none());
        assert_eq!(state.read(app).active.as_ref().unwrap().1.as_ref(), "note");
        assert_eq!(owner.read(app).component().rows[0].quantity, 3);
        assert_eq!(owner.read(app).component().writes, 1);
    });
    window.simulate_keystrokes("shift-tab");
    frame(window);
    window.update(|_, app| {
        assert_eq!(
            grid(&owner, app)
                .read(app)
                .active
                .as_ref()
                .unwrap()
                .1
                .as_ref(),
            "quantity"
        )
    });
}

#[gpui::test]
fn focus_leaving_commits_once_without_stealing_focus(cx: &mut gpui::TestAppContext) {
    let (owner, window, outside) = setup(cx);
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("Outside commit");
    frame(window);
    window.update(|window, app| outside.focus(window, app));
    frame(window);
    window.update(|window, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "Outside commit");
        assert_eq!(owner.read(app).component().writes, 1);
        assert_eq!(window.focused(app).as_ref(), Some(&outside));
        assert!(grid(&owner, app).read(app).edit.is_none());
    });
}

#[gpui::test]
fn typing_starts_edit_and_read_only_cell_rejects_edit(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("z");
    frame(window);
    window.update(|_, app| {
        let state = grid(&owner, app);
        assert_eq!(&*state.read(app).edit.as_ref().unwrap().draft.borrow(), "z");
    });
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("right right");
    frame(window);
    window.update(|_, app| {
        assert_eq!(
            grid(&owner, app)
                .read(app)
                .active
                .as_ref()
                .unwrap()
                .1
                .as_ref(),
            "note"
        )
    });
    window.simulate_keystrokes("enter");
    window.simulate_keystrokes("z");
    frame(window);
    window.update(|_, app| {
        assert!(grid(&owner, app).read(app).edit.is_none());
        assert_eq!(owner.read(app).component().rows[0].name, "z");
        assert_eq!(owner.read(app).component().rows[0].note, "fixed A");
        assert_eq!(owner.read(app).component().writes, 1);
    });
}

#[gpui::test]
fn edit_follows_stable_row_id_after_owner_reorder(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("Moved");
    frame(window);
    owner.update(window, |owner, cx| {
        owner.component_mut().rows.swap(0, 1);
        cx.notify();
    });
    frame(window);
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        let owner = owner.read(app);
        assert_eq!(owner.component().rows[0].name, "Beta");
        assert_eq!(owner.component().rows[1].name, "Moved");
        assert_eq!(owner.component().writes, 1);
    });
}

#[gpui::test]
fn invalid_edit_restores_focus_when_leaving_and_escape_discards(cx: &mut gpui::TestAppContext) {
    let (owner, window, outside) = setup(cx);
    window.simulate_keystrokes("right enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("-1");
    frame(window);
    window.update(|window, app| outside.focus(window, app));
    frame(window);
    window.update(|window, app| {
        let state = grid(&owner, app);
        let edit = state
            .read(app)
            .edit
            .clone()
            .expect("invalid draft stays open");
        assert_eq!(window.focused(app).as_ref(), edit.focus.borrow().as_ref());
        assert_eq!(owner.read(app).component().writes, 0);
    });
    assert!(
        window
            .debug_bounds("validation-error:Must be nonnegative")
            .is_some()
    );
    window.simulate_keystrokes("escape");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].quantity, 1);
        assert_eq!(owner.read(app).component().writes, 0);
        assert!(grid(&owner, app).read(app).edit.is_none());
    });
}

#[gpui::test]
fn double_click_starts_the_clicked_cell_editor(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    let position = window
        .debug_bounds("editable-grid-cell:b:name")
        .expect("second row is rendered")
        .center();
    window.simulate_event(gpui::MouseDownEvent {
        button: gpui::MouseButton::Left,
        position,
        modifiers: gpui::Modifiers::default(),
        click_count: 2,
        first_mouse: false,
    });
    window.simulate_event(gpui::MouseUpEvent {
        button: gpui::MouseButton::Left,
        position,
        modifiers: gpui::Modifiers::default(),
        click_count: 2,
    });
    frame(window);
    window.update(|_, app| {
        let state = grid(&owner, app);
        let edit = state
            .read(app)
            .edit
            .clone()
            .expect("double click starts editing");
        assert_eq!(edit.row.as_ref(), "b");
        assert_eq!(edit.column.as_ref(), "name");
    });
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("Second");
    frame(window);
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[1].name, "Second");
        assert_eq!(owner.read(app).component().writes, 1);
    });
}

#[gpui::test]
fn shift_tab_commits_and_navigates_to_previous_cell(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("right enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("7");
    frame(window);
    window.simulate_keystrokes("shift-tab");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].quantity, 7);
        assert_eq!(owner.read(app).component().writes, 1);
        let state = grid(&owner, app);
        assert!(state.read(app).edit.is_none());
        assert_eq!(state.read(app).active.as_ref().unwrap().1.as_ref(), "name");
    });
}

#[gpui::test]
fn parse_error_is_visible_and_keeps_draft_uncommitted(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("right enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("abc");
    frame(window);
    assert!(
        window
            .debug_bounds("validation-error:Enter an integer")
            .is_some()
    );
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert!(grid(&owner, app).read(app).edit.is_some());
        assert_eq!(owner.read(app).component().rows[0].quantity, 1);
        assert_eq!(owner.read(app).component().writes, 0);
    });
}

#[gpui::test]
#[ignore = "Tab exit: needs GPUI focus traversal from a dropped editor"]
fn tab_navigation_crosses_rows_and_leaves_grid_at_end(cx: &mut gpui::TestAppContext) {
    let (owner, window, outside) = setup(cx);
    window.simulate_keystrokes("tab tab tab");
    frame(window);
    window.update(|_, app| {
        let state = grid(&owner, app);
        let active = state.read(app).active.clone().unwrap();
        assert_eq!(active.0.as_ref(), "b");
        assert_eq!(active.1.as_ref(), "name");
    });
    window.simulate_keystrokes("tab tab tab");
    frame(window);
    window.update(|window, app| assert_eq!(window.focused(app).as_ref(), Some(&outside)));
}

#[gpui::test]
fn removing_an_edited_row_discards_draft_without_a_write(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("unsaved");
    frame(window);
    owner.update(window, |owner, cx| {
        owner.component_mut().rows.remove(0);
        cx.notify();
    });
    frame(window);
    window.update(|_, app| {
        assert!(grid(&owner, app).read(app).edit.is_none());
        assert_eq!(owner.read(app).component().rows[0].id, "b");
        assert_eq!(owner.read(app).component().writes, 0);
    });
}

#[test]
#[should_panic(expected = "EditableDataGrid column IDs must be unique")]
fn duplicate_column_ids_are_rejected() {
    let column = || {
        DataGridColumn::text(
            "same",
            "Name",
            |row: &Row| row.name.clone(),
            |text| Ok(text.to_owned()),
            |row, value| row.name = value,
        )
    };
    let rows = Binding::new(
        Vec::<Row>::new(),
        |owner: &Owner| owner.rows.clone(),
        |owner, value| owner.rows = value,
    );
    let _ = EditableDataGrid(
        "Products",
        rows,
        |row: &Row| row.id.into(),
        vec![column(), column()],
    );
}

#[gpui::test]
#[ignore = "Tab exit: needs GPUI focus traversal from a dropped editor"]
fn committing_last_cell_with_tab_moves_focus_outside(cx: &mut gpui::TestAppContext) {
    let (owner, window, outside) = setup(cx);
    owner.update(window, |owner, cx| {
        owner.component_mut().note_editable = true;
        cx.notify();
    });
    frame(window);
    window.simulate_keystrokes("tab tab tab tab tab enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("changed");
    frame(window);
    window.simulate_keystrokes("tab");
    frame(window);
    window.update(|window, app| {
        assert_eq!(owner.read(app).component().rows[1].note, "changed");
        assert_eq!(owner.read(app).component().writes, 1);
        assert_eq!(window.focused(app).as_ref(), Some(&outside));
        assert!(grid(&owner, app).read(app).edit.is_none());
    });
}

#[gpui::test]
fn changing_row_identity_is_rejected_with_error(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("change-id");
    frame(window);
    window.simulate_keystrokes("enter");
    frame(window);
    assert!(
        window
            .debug_bounds("validation-error:Row ID cannot be changed")
            .is_some()
    );
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].id, "a");
        assert_eq!(owner.read(app).component().writes, 0);
        assert!(grid(&owner, app).read(app).edit.is_some());
    });
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("safe");
    frame(window);
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "safe");
        assert_eq!(owner.read(app).component().writes, 1);
    });
}

#[test]
#[should_panic(expected = "EditableDataGrid row IDs must be nonempty and unique")]
fn duplicate_row_ids_are_rejected_before_any_write() {
    let rows = vec![
        Row {
            id: "same",
            name: "First".into(),
            quantity: 1,
            note: String::new(),
        },
        Row {
            id: "same",
            name: "Second".into(),
            quantity: 2,
            note: String::new(),
        },
    ];
    let binding = Binding::new(
        rows,
        |owner: &Owner| owner.rows.clone(),
        |owner, value| owner.rows = value,
    );
    let _ = EditableDataGrid("Products", binding, |row: &Row| row.id.into(), Vec::new());
}

#[gpui::test]
fn scrolling_away_and_back_preserves_the_active_draft(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    owner.update(window, |owner, cx| {
        for id in [
            "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q", "r", "s",
            "t",
        ] {
            owner.component_mut().rows.push(Row {
                id,
                name: id.into(),
                quantity: 1,
                note: String::new(),
            });
        }
        cx.notify();
    });
    frame(window);
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("pending");
    frame(window);
    window.update(|window, app| {
        let grid = grid(&owner, app);
        let table = grid.read(app).table.as_ref().unwrap().upgrade().unwrap();
        table.update(app, |table, cx| table.scroll_to_row(15, cx));
        window.refresh();
    });
    frame(window);
    window.update(|_, app| {
        let state = grid(&owner, app);
        assert_eq!(
            &*state
                .read(app)
                .edit
                .as_ref()
                .expect("draft survives scrolling")
                .draft
                .borrow(),
            "pending"
        );
        assert_eq!(owner.read(app).component().writes, 0);
    });
    window.update(|window, app| {
        let state = grid(&owner, app);
        let table = state.read(app).table.as_ref().unwrap().upgrade().unwrap();
        table.update(app, |table, cx| table.scroll_to_row(0, cx));
        window.refresh();
    });
    frame(window);
    window.update(|window, app| {
        let state = grid(&owner, app);
        let focus = state
            .read(app)
            .edit
            .as_ref()
            .unwrap()
            .focus
            .borrow()
            .clone()
            .unwrap();
        focus.focus(window, app);
    });
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "pending");
        assert_eq!(owner.read(app).component().writes, 1);
    });
}

#[gpui::test]
fn virtualized_editor_entity_is_mounted_and_focused(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|window, app| {
        let state = grid(&owner, app);
        let state = state.read(app);
        let edit = state.edit.as_ref().expect("edit stays open");
        assert!(state.editor.borrow().is_some());
        let focus = edit.focus.borrow().clone().expect("editor rendered");
        assert_eq!(window.focused(app), Some(focus));
    });
}

#[gpui::test]
fn virtualized_table_renders_fewer_than_all_thousand_rows(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    owner.update(window, |owner, cx| {
        owner.component_mut().rows = (0..1000)
            .map(|index| Row {
                id: Box::leak(format!("row-{index}").into_boxed_str()),
                name: format!("Row {index}"),
                quantity: index,
                note: String::new(),
            })
            .collect();
        cx.notify();
    });
    window.update(|_, app| {
        grid(&owner, app)
            .read(app)
            .rendered_rows
            .borrow_mut()
            .clear()
    });
    frame(window);
    window.update(|_, app| {
        let grid = grid(&owner, app);
        let count = grid.read(app).rendered_rows.borrow().len();
        assert!(count > 0 && count < 1000, "rendered {count} rows");
    });
}

#[gpui::test]
fn virtualized_scroll_preserves_editor_draft_focus_and_single_commit(
    cx: &mut gpui::TestAppContext,
) {
    let (owner, window, _) = setup(cx);
    owner.update(window, |owner, cx| {
        for index in 0..1000 {
            owner.component_mut().rows.push(Row {
                id: Box::leak(format!("scroll-{index}").into_boxed_str()),
                name: format!("Row {index}"),
                quantity: index,
                note: String::new(),
            });
        }
        cx.notify();
    });
    frame(window);
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("retained draft");
    frame(window);
    let (editor, focus) = window.update(|_, app| {
        let grid = grid(&owner, app);
        let state = grid.read(app);
        (
            state.editor.borrow().as_ref().unwrap().entity_id(),
            state.edit.as_ref().unwrap().focus.borrow().clone().unwrap(),
        )
    });
    window.update(|_, app| {
        let grid = grid(&owner, app);
        let table = grid.read(app).table.as_ref().unwrap().upgrade().unwrap();
        table.update(app, |table, cx| table.scroll_to_row(500, cx));
    });
    frame(window);
    window.update(|_, app| {
        let grid = grid(&owner, app);
        let state = grid.read(app);
        assert_eq!(state.editor.borrow().as_ref().unwrap().entity_id(), editor);
        assert_eq!(
            &*state.edit.as_ref().unwrap().draft.borrow(),
            "retained draft"
        );
        assert_eq!(owner.read(app).component().writes, 0);
        let table = state.table.as_ref().unwrap().upgrade().unwrap();
        table.update(app, |table, cx| table.scroll_to_row(0, cx));
    });
    frame(window);
    window.update(|window, app| {
        let grid = grid(&owner, app);
        let state = grid.read(app);
        assert_eq!(state.editor.borrow().as_ref().unwrap().entity_id(), editor);
        assert_eq!(
            state.edit.as_ref().unwrap().focus.borrow().as_ref(),
            Some(&focus)
        );
        assert_eq!(window.focused(app), Some(focus));
    });
    window.simulate_keystrokes("enter");
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "retained draft");
        assert_eq!(owner.read(app).component().writes, 1);
    });
}

fn click_grid_header(window: &mut gpui::VisualTestContext, name: &str) {
    let selector = match name {
        "name" => "editable-grid-header:name",
        "note" => "editable-grid-header:note",
        _ => panic!("unknown header"),
    };
    let position = window.debug_bounds(selector).unwrap().center();
    window.simulate_event(gpui::MouseDownEvent {
        button: gpui::MouseButton::Left,
        position,
        modifiers: gpui::Modifiers::default(),
        click_count: 1,
        first_mouse: false,
    });
    window.simulate_event(gpui::MouseUpEvent {
        button: gpui::MouseButton::Left,
        position,
        modifiers: gpui::Modifiers::default(),
        click_count: 1,
    });
    frame(window);
}

#[gpui::test]
fn sort_click_cycle_is_stable_and_never_writes_bound_rows(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    owner.update(window, |owner, cx| {
        owner.component_mut().rows = vec![
            Row {
                id: "a",
                name: "Able".into(),
                quantity: 2,
                note: "".into(),
            },
            Row {
                id: "b",
                name: "Zed".into(),
                quantity: 1,
                note: "".into(),
            },
            Row {
                id: "c",
                name: "Able".into(),
                quantity: 1,
                note: "".into(),
            },
        ];
        cx.notify();
    });
    frame(window);
    let original = window.update(|_, app| owner.read(app).component().rows.clone());
    for expected in [["a", "c", "b"], ["b", "a", "c"], ["a", "b", "c"]] {
        click_grid_header(window, "name");
        window.update(|_, app| {
            assert_eq!(
                grid(&owner, app)
                    .read(app)
                    .view_order
                    .iter()
                    .map(|x| x.as_ref())
                    .collect::<Vec<_>>(),
                expected
            );
            assert_eq!(owner.read(app).component().rows, original);
            assert_eq!(owner.read(app).component().writes, 0);
            assert_eq!(
                grid(&owner, app)
                    .read(app)
                    .active
                    .as_ref()
                    .unwrap()
                    .0
                    .as_ref(),
                "a"
            );
        });
    }
    click_grid_header(window, "note");
    window.update(|_, app| assert!(grid(&owner, app).read(app).sort.is_none()));
}

#[gpui::test]
fn sort_keyboard_enter_space_cycles_and_owner_updates_reapply(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.update(|window, app| {
        let grid = grid(&owner, app);
        let focus = grid.read(app).header_focus[0].clone();
        focus.focus(window, app);
    });
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert_eq!(
            grid(&owner, app).read(app).sort.as_ref().unwrap().1,
            ColumnSort::Ascending
        )
    });
    window.simulate_keystrokes("space");
    frame(window);
    window.update(|_, app| {
        assert_eq!(
            grid(&owner, app).read(app).sort.as_ref().unwrap().1,
            ColumnSort::Descending
        )
    });
    owner.update(window, |owner, cx| {
        owner.component_mut().rows[0].name = "Zulu".into();
        cx.notify();
    });
    frame(window);
    window.update(|_, app| assert_eq!(grid(&owner, app).read(app).view_order[0].as_ref(), "a"));
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| assert!(grid(&owner, app).read(app).sort.is_none()));
}

#[gpui::test]
fn sort_freezes_during_edit_then_resorts_after_commit(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    click_grid_header(window, "name");
    window.update(|window, app| {
        let grid = grid(&owner, app);
        let focus = grid.read(app).focus.clone();
        focus.focus(window, app);
    });
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("Zulu");
    frame(window);
    owner.update(window, |owner, cx| {
        owner.component_mut().rows[1].name = "Aardvark".into();
        cx.notify();
    });
    frame(window);
    window.update(|_, app| assert_eq!(grid(&owner, app).read(app).view_order[0].as_ref(), "a"));
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert_eq!(grid(&owner, app).read(app).view_order[0].as_ref(), "b");
        assert_eq!(
            grid(&owner, app)
                .read(app)
                .active
                .as_ref()
                .unwrap()
                .0
                .as_ref(),
            "a"
        );
        assert_eq!(owner.read(app).component().writes, 1);
    });
}

#[gpui::test]
fn sort_table_delegate_hook_cycles_current_sort(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    for expected in [
        Some(ColumnSort::Ascending),
        Some(ColumnSort::Descending),
        None,
    ] {
        window.update(|window, app| {
            let grid = grid(&owner, app);
            let table = grid.read(app).table.as_ref().unwrap().upgrade().unwrap();
            table.update(app, |table, cx| {
                table
                    .delegate_mut()
                    .perform_sort(0, ColumnSort::Default, window, cx);
            });
        });
        frame(window);
        window.update(|_, app| {
            assert_eq!(
                grid(&owner, app)
                    .read(app)
                    .sort
                    .as_ref()
                    .map(|(_, order)| *order),
                expected
            );
        });
    }
}

#[gpui::test]
fn navigation_home_end_and_page_keys_clamp_and_preserve_column(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    owner.update(window, |owner, cx| {
        for index in 0..60 {
            owner.component_mut().rows.push(Row {
                id: Box::leak(format!("nav-{index}").into_boxed_str()),
                name: format!("Row {index}"),
                quantity: index,
                note: String::new(),
            });
        }
        cx.notify();
    });
    frame(window);
    window.simulate_keystrokes("right end");
    frame(window);
    window.update(|_, app| {
        assert_eq!(
            grid(&owner, app)
                .read(app)
                .active
                .as_ref()
                .unwrap()
                .1
                .as_ref(),
            "note"
        )
    });
    window.simulate_keystrokes("home");
    frame(window);
    window.update(|_, app| {
        assert_eq!(
            grid(&owner, app)
                .read(app)
                .active
                .as_ref()
                .unwrap()
                .1
                .as_ref(),
            "name"
        )
    });
    window.simulate_keystrokes("cmd-end");
    frame(window);
    window.update(|_, app| {
        let grid = grid(&owner, app);
        let grid = grid.read(app);
        assert_eq!(
            grid.active.as_ref().unwrap().0,
            *grid.view_order.last().unwrap()
        );
        assert_eq!(grid.active.as_ref().unwrap().1.as_ref(), "note");
    });
    window.simulate_keystrokes("cmd-home right pagedown");
    frame(window);
    window.update(|_, app| {
        let grid = grid(&owner, app);
        let grid = grid.read(app);
        assert!(grid.view_index(&grid.active.as_ref().unwrap().0).unwrap() > 0);
        assert_eq!(grid.active.as_ref().unwrap().1.as_ref(), "quantity");
    });
    window.simulate_keystrokes("pageup");
    frame(window);
    window.update(|_, app| {
        let grid = grid(&owner, app);
        let grid = grid.read(app);
        assert_eq!(grid.active.as_ref().unwrap().0.as_ref(), "a");
        assert_eq!(grid.active.as_ref().unwrap().1.as_ref(), "quantity");
    });
}

#[gpui::test]
fn external_cell_conflict_preserves_draft_and_enter_overwrites_escape_discards(
    cx: &mut gpui::TestAppContext,
) {
    let (owner, window, _) = setup(cx);
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("draft");
    frame(window);
    owner.update(window, |owner, cx| {
        owner.component_mut().rows[0].name = "external".into();
        cx.notify();
    });
    frame(window);
    assert!(
        window
            .debug_bounds("validation-error:Value changed elsewhere")
            .is_some()
    );
    window.update(|_, app| {
        let grid = grid(&owner, app);
        assert_eq!(
            &*grid.read(app).edit.as_ref().unwrap().draft.borrow(),
            "draft"
        );
        assert_eq!(owner.read(app).component().writes, 0);
    });
    window.simulate_keystrokes("enter");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "draft");
        assert_eq!(owner.read(app).component().writes, 1);
    });
    window.simulate_keystrokes("enter");
    frame(window);
    window.simulate_keystrokes("cmd-a");
    window.simulate_input("discarded");
    frame(window);
    owner.update(window, |owner, cx| {
        owner.component_mut().rows[0].name = "new owner value".into();
        cx.notify();
    });
    frame(window);
    assert!(
        window
            .debug_bounds("validation-error:Value changed elsewhere")
            .is_some()
    );
    window.simulate_keystrokes("escape");
    frame(window);
    window.update(|_, app| {
        assert_eq!(owner.read(app).component().rows[0].name, "new owner value");
        assert_eq!(owner.read(app).component().writes, 1);
    });
}

#[test]
fn editable_grid_accessibility_elements_expose_roles_counts_and_indices() {
    use gpui::{Element, accesskit::Node};
    fn node(element: &impl Element) -> Node {
        let mut node = Node::new(element.a11y_role().unwrap());
        element.write_a11y_info(&mut node);
        node
    }
    let grid = node(&grid_accessibility("grid".into(), "Products".into(), 3, 2));
    assert_eq!(grid.role(), Role::Grid);
    assert_eq!(grid.label(), Some("Products"));
    assert_eq!(grid.row_count(), Some(4));
    assert_eq!(grid.column_count(), Some(2));
    let row = node(&grid_row_accessibility(1));
    assert_eq!(row.role(), Role::Row);
    assert_eq!(row.row_index(), Some(3));
    let header = node(&grid_header_accessibility(1, "Quantity, ascending".into()));
    assert_eq!(header.role(), Role::ColumnHeader);
    assert_eq!(header.column_index(), Some(2));
    assert_eq!(header.label(), Some("Quantity, ascending"));
    let cell = node(&grid_cell_accessibility(
        1,
        1,
        "Quantity, 2, read only".into(),
        true,
    ));
    assert_eq!(cell.role(), Role::Cell);
    assert_eq!(cell.row_index(), Some(3));
    assert_eq!(cell.column_index(), Some(2));
    assert_eq!(cell.is_selected(), Some(true));
    assert_eq!(cell.label(), Some("Quantity, 2, read only"));
}

#[gpui::test]
fn navigation_home_end_follow_logical_order_in_rtl(cx: &mut gpui::TestAppContext) {
    let (owner, window, _) = setup(cx);
    window.update(|_, app| {
        let grid = grid(&owner, app);
        grid.update(app, |grid, cx| {
            grid.rtl = true;
            grid.active = Some(("b".into(), "quantity".into()));
            assert!(grid.navigate_boundary("home", false, cx));
            assert_eq!(grid.active.as_ref().unwrap().1.as_ref(), "name");
            assert!(grid.navigate_boundary("end", false, cx));
            assert_eq!(grid.active.as_ref().unwrap().1.as_ref(), "note");
            assert!(grid.navigate_boundary("home", true, cx));
            assert_eq!(grid.active.as_ref().unwrap().0.as_ref(), "a");
        });
    });
}
