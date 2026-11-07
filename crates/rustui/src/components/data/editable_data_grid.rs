//! Owner-bound editable grid with stable row and column identities.
use std::{cell::RefCell, collections::HashSet, fmt::Display, rc::Rc};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement as _, KeyDownEvent, MouseButton, ParentElement as _, SharedString,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, Window, div, px,
};
use gpui_component::table::{Column, DataTable, TableDelegate, TableState};
use rustui_core::{Binding, Identity};

use crate::{
    components::input::text_field::TextField,
    runtime::ComponentHost,
    view::{Component, EmptyView, RenderContext, View},
};

gpui::actions!(editable_data_grid, [NextCell, PreviousCell]);

struct GridKeyBindings;
impl gpui::Global for GridKeyBindings {}

type CellReader<R> = Rc<dyn Fn(&R) -> String>;
type CellWriter<R> = Rc<dyn Fn(&mut R, &str) -> Result<(), String>>;
type CellValidator = Rc<dyn Fn(&str) -> Result<(), String>>;
type RowIdentity<R> = Rc<dyn Fn(&R) -> SharedString>;
type Committer = Rc<dyn Fn(Commit, &mut App) -> bool>;

/// A typed text column whose values are parsed before they are written.
pub struct DataGridColumn<R> {
    id: SharedString,
    title: SharedString,
    read: CellReader<R>,
    write: CellWriter<R>,
    parse: CellValidator,
    validate: Option<CellValidator>,
    editable: bool,
}

impl<R> Clone for DataGridColumn<R> {
    fn clone(&self) -> Self {
        Self {
            id: self.id.clone(),
            title: self.title.clone(),
            read: self.read.clone(),
            write: self.write.clone(),
            parse: self.parse.clone(),
            validate: self.validate.clone(),
            editable: self.editable,
        }
    }
}

impl<R: 'static> DataGridColumn<R> {
    /// Defines a column using typed read, parse and write functions.
    pub fn text<V: Clone + Display + 'static>(
        id: impl Into<SharedString>,
        title: impl Into<SharedString>,
        read: impl Fn(&R) -> V + 'static,
        parse: impl Fn(&str) -> Result<V, String> + 'static,
        write: impl Fn(&mut R, V) + 'static,
    ) -> Self {
        let id = id.into();
        assert!(
            !id.trim().is_empty(),
            "DataGrid column IDs must not be empty"
        );
        let parse = Rc::new(parse);
        let validate_parse = parse.clone();
        Self {
            id,
            title: title.into(),
            read: Rc::new(move |row| read(row).to_string()),
            write: Rc::new(move |row, value| {
                let parsed = parse(value)?;
                write(row, parsed);
                Ok(())
            }),
            parse: Rc::new(move |value| validate_parse(value).map(|_| ())),
            validate: None,
            editable: true,
        }
    }

    /// Adds synchronous validation for the editor's draft.
    pub fn validate(mut self, validator: impl Fn(&str) -> Result<(), String> + 'static) -> Self {
        self.validate = Some(Rc::new(validator));
        self
    }

    /// Keeps this column navigable while preventing edits.
    pub fn read_only(mut self) -> Self {
        self.editable = false;
        self
    }

    fn check(&self, value: &str) -> Result<(), String> {
        if let Some(validate) = &self.validate {
            validate(value)?;
        }
        (self.parse)(value)
    }
}

fn assert_unique_row_ids<R>(rows: &[R], row_id: &RowIdentity<R>) {
    let mut unique = HashSet::new();
    assert!(
        rows.iter().all(|row| {
            let id = row_id(row);
            !id.trim().is_empty() && unique.insert(id)
        }),
        "EditableDataGrid row IDs must be nonempty and unique"
    );
}

/// A separate editable grid; the existing DataGrid and Table remain unchanged.
pub struct EditableDataGridView<R: Clone + 'static> {
    label: SharedString,
    rows: Binding<Vec<R>>,
    row_id: RowIdentity<R>,
    columns: Rc<Vec<DataGridColumn<R>>>,
    id: Option<Identity>,
}

/// Creates an editable grid backed by the owner's vector of typed rows.
#[allow(non_snake_case)]
pub fn EditableDataGrid<R: Clone + 'static>(
    label: impl Into<SharedString>,
    rows: Binding<Vec<R>>,
    row_id: impl Fn(&R) -> SharedString + 'static,
    columns: Vec<DataGridColumn<R>>,
) -> EditableDataGridView<R> {
    let label = label.into();
    assert!(
        !label.trim().is_empty(),
        "EditableDataGrid label must not be empty"
    );
    let mut ids = HashSet::new();
    assert!(
        columns.iter().all(|column| ids.insert(column.id.clone())),
        "EditableDataGrid column IDs must be unique"
    );
    let row_id: RowIdentity<R> = Rc::new(row_id);
    assert_unique_row_ids(&rows.value(), &row_id);
    EditableDataGridView {
        label,
        rows,
        row_id,
        columns: Rc::new(columns),
        id: None,
    }
}

impl<R: Clone + 'static> EditableDataGridView<R> {
    /// Sets the retained identity of the grid.
    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "EditableDataGrid ID must not be empty"
        );
        self.id = Some(id);
        self
    }
}

#[derive(Clone)]
struct Commit {
    row: SharedString,
    column: SharedString,
    draft: String,
    error: Rc<RefCell<Option<String>>>,
}

#[derive(Clone)]
struct Editing {
    row: SharedString,
    column: SharedString,
    draft: Rc<RefCell<String>>,
    focus: Rc<RefCell<Option<FocusHandle>>>,
    error: Rc<RefCell<Option<String>>>,
    generation: usize,
}

struct GridState<R: Clone + 'static> {
    rows: Vec<R>,
    row_id: RowIdentity<R>,
    columns: Rc<Vec<DataGridColumn<R>>>,
    active: Option<(SharedString, SharedString)>,
    edit: Option<Editing>,
    focus: FocusHandle,
    table: Option<WeakEntity<TableState<EditableGridDelegate<R>>>>,
    editor: Rc<RefCell<Option<Entity<ComponentHost<CellEditor>>>>>,
    #[cfg(test)]
    rendered_rows: Rc<RefCell<HashSet<usize>>>,
    focus_out: Option<Subscription>,
    commit: Option<Committer>,
    generation: usize,
    rtl: bool,
}

impl<R: Clone + 'static> Drop for GridState<R> {
    fn drop(&mut self) {
        if let Some(edit) = self.edit.take() {
            *edit.focus.borrow_mut() = None;
        }
    }
}

impl<R: Clone + 'static> GridState<R> {
    fn row_index(&self, id: &str) -> Option<usize> {
        self.rows
            .iter()
            .position(|row| (self.row_id)(row).as_ref() == id)
    }

    fn column_index(&self, id: &str) -> Option<usize> {
        self.columns
            .iter()
            .position(|column| column.id.as_ref() == id)
    }

    fn begin_edit(&mut self, replacement: Option<String>, window: &mut Window, cx: &mut App) {
        let Some((row_id, column_id)) = self.active.as_ref() else {
            return;
        };
        let (Some(row), Some(column)) = (self.row_index(row_id), self.column_index(column_id))
        else {
            return;
        };
        let column = &self.columns[column];
        if !column.editable {
            return;
        }
        self.generation = self.generation.wrapping_add(1);
        self.edit = Some(Editing {
            row: row_id.clone(),
            column: column_id.clone(),
            draft: Rc::new(RefCell::new(
                replacement.unwrap_or_else(|| (column.read)(&self.rows[row])),
            )),
            focus: Rc::new(RefCell::new(None)),
            error: Rc::new(RefCell::new(None)),
            generation: self.generation,
        });
        let edit = self.edit.as_ref().unwrap();
        let shared = edit.draft.clone();
        let error = edit.error.clone();
        let validator_error = error.clone();
        let column = column.clone();
        let validate: CellValidator = Rc::new(move |value| {
            if let Some(error) = validator_error.borrow().as_ref() {
                return Err(error.clone());
            }
            column.check(value)
        });
        let editor = CellEditor {
            value: shared.borrow().clone(),
            shared: shared.clone(),
            focus: edit.focus.clone(),
            error,
            validate,
        };
        *self.editor.borrow_mut() = Some(cx.new(|cx| ComponentHost::new(editor, cx)));
        window.refresh();
        cx.stop_propagation();
    }

    fn cancel(&mut self, window: &mut Window, cx: &mut App) {
        self.focus.focus(window, cx);
        if let Some(edit) = self.edit.take() {
            *edit.focus.borrow_mut() = None;
        }
        *self.editor.borrow_mut() = None;
        window.refresh();
    }

    fn commit_edit(&mut self, window: &mut Window, cx: &mut App, restore_focus: bool) -> bool {
        let Some(edit) = self.edit.clone() else {
            return true;
        };
        let (Some(_row), Some(column)) =
            (self.row_index(&edit.row), self.column_index(&edit.column))
        else {
            self.cancel(window, cx);
            return true;
        };
        let draft = edit.draft.borrow().clone();
        if self.columns[column].check(&draft).is_err() {
            if let Some(focus) = edit.focus.borrow().as_ref() {
                focus.focus(window, cx);
            }
            return false;
        }
        let change = Commit {
            row: edit.row.clone(),
            column: edit.column.clone(),
            draft,
            error: edit.error.clone(),
        };
        let accepted = self
            .commit
            .as_ref()
            .is_some_and(|commit| commit(change, cx));
        if !accepted {
            if edit.error.borrow().is_none() {
                *edit.error.borrow_mut() = Some("Unable to update this row".into());
            }
            if let Some(focus) = edit.focus.borrow().as_ref() {
                focus.focus(window, cx);
            }
            window.refresh();
            return false;
        }
        if restore_focus {
            self.cancel(window, cx);
        } else {
            if let Some(edit) = self.edit.take() {
                *edit.focus.borrow_mut() = None;
            }
            *self.editor.borrow_mut() = None;
            window.refresh();
        }
        true
    }

    fn scroll_to_row(&self, row: usize, cx: &mut App) {
        if let Some(table) = self.table.as_ref().and_then(WeakEntity::upgrade) {
            table.update(cx, |table, cx| table.scroll_to_row(row, cx));
        }
    }

    fn advance(&mut self, row_delta: isize, column_delta: isize, cx: &mut App) -> bool {
        let Some((row, column)) = self.active.as_ref() else {
            return false;
        };
        let (Some(r), Some(c)) = (self.row_index(row), self.column_index(column)) else {
            return false;
        };
        let (Some(nr), Some(nc)) = (
            r.checked_add_signed(row_delta),
            c.checked_add_signed(column_delta),
        ) else {
            return false;
        };
        if nr >= self.rows.len() || nc >= self.columns.len() {
            return false;
        }
        self.active = Some(((self.row_id)(&self.rows[nr]), self.columns[nc].id.clone()));
        self.scroll_to_row(nr, cx);
        true
    }

    fn tab(&mut self, backward: bool, cx: &mut App) -> bool {
        let Some((row, column)) = self.active.as_ref() else {
            return false;
        };
        let (Some(r), Some(c)) = (self.row_index(row), self.column_index(column)) else {
            return false;
        };
        let width = self.columns.len();
        if width == 0 {
            return false;
        }
        let position = r * width + c;
        let next = if backward {
            position.checked_sub(1)
        } else {
            position
                .checked_add(1)
                .filter(|next| *next < self.rows.len() * width)
        };
        let Some(next) = next else { return false };
        self.active = Some((
            (self.row_id)(&self.rows[next / width]),
            self.columns[next % width].id.clone(),
        ));
        self.scroll_to_row(next / width, cx);
        true
    }

    fn navigate_tab(&mut self, backward: bool, window: &mut Window, cx: &mut App) {
        if self.edit.is_some() && !self.commit_edit(window, cx, true) {
            cx.stop_propagation();
            return;
        }
        self.focus.focus(window, cx);
        if self.tab(backward, cx) {
            window.refresh();
        } else if backward {
            window.focus_prev(cx);
        } else {
            window.focus_next(cx);
        }
        cx.stop_propagation();
    }

    fn keyboard(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        if self.edit.is_some() {
            match key {
                "escape" => {
                    self.cancel(window, cx);
                    cx.stop_propagation();
                }
                "enter" => {
                    self.commit_edit(window, cx, true);
                    cx.stop_propagation();
                }
                "tab" => self.navigate_tab(modifiers.shift, window, cx),
                _ => {}
            }
            return;
        }
        if modifiers.control || modifiers.platform || modifiers.alt {
            return;
        }
        let changed = match key {
            "up" => self.advance(-1, 0, cx),
            "down" => self.advance(1, 0, cx),
            "left" => self.advance(0, if self.rtl { 1 } else { -1 }, cx),
            "right" => self.advance(0, if self.rtl { -1 } else { 1 }, cx),
            "tab" => {
                self.navigate_tab(modifiers.shift, window, cx);
                return;
            }
            "enter" | "f2" => {
                self.begin_edit(None, window, cx);
                cx.stop_propagation();
                return;
            }
            _ => {
                let character = event.keystroke.key_char.as_deref().or_else(|| {
                    if key == "space" {
                        Some(" ")
                    } else if key.chars().count() == 1 {
                        Some(key)
                    } else {
                        None
                    }
                });
                if let Some(character) = character.filter(|s| !s.chars().any(char::is_control)) {
                    self.begin_edit(Some(character.to_owned()), window, cx);
                    cx.stop_propagation();
                }
                return;
            }
        };
        if changed {
            window.refresh();
        }
        if changed || matches!(key, "up" | "down" | "left" | "right") {
            cx.stop_propagation();
        }
    }
}

struct CellEditor {
    value: String,
    shared: Rc<RefCell<String>>,
    focus: Rc<RefCell<Option<FocusHandle>>>,
    error: Rc<RefCell<Option<String>>>,
    validate: CellValidator,
}

impl View for CellEditor {
    fn body(&self) -> impl View {
        let draft = Binding::new(
            self.value.clone(),
            |editor: &Self| editor.value.clone(),
            |editor, value| {
                editor.value = value;
                *editor.shared.borrow_mut() = editor.value.clone();
                *editor.error.borrow_mut() = None;
            },
        );
        let validate = self.validate.clone();
        TextField("", draft)
            .id("editable-data-grid-editor")
            .validate(move |value| validate(value))
            .editor_focus(self.focus.clone())
    }
}

struct EditableGridDelegate<R: Clone + 'static> {
    #[cfg(test)]
    rendered_rows: Rc<RefCell<HashSet<usize>>>,
    owner: WeakEntity<GridState<R>>,
    editor: Rc<RefCell<Option<Entity<ComponentHost<CellEditor>>>>>,
}

impl<R: Clone + 'static> TableDelegate for EditableGridDelegate<R> {
    fn columns_count(&self, cx: &App) -> usize {
        self.owner
            .upgrade()
            .map_or(0, |owner| owner.read(cx).columns.len())
    }
    fn rows_count(&self, cx: &App) -> usize {
        self.owner
            .upgrade()
            .map_or(0, |owner| owner.read(cx).rows.len())
    }
    fn column(&self, col_ix: usize, cx: &App) -> Column {
        let owner = self.owner.upgrade().expect("grid owner");
        let grid = owner.read(cx);
        let ix = if grid.rtl {
            grid.columns.len() - 1 - col_ix
        } else {
            col_ix
        };
        let column = &grid.columns[ix];
        Column::new(column.id.clone(), column.title.clone())
            .width(px(160.))
            .resizable(false)
            .movable(false)
    }
    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl gpui::IntoElement {
        #[cfg(test)]
        self.rendered_rows.borrow_mut().insert(row_ix);
        let Some(owner) = self.owner.upgrade() else {
            return div().into_any_element();
        };
        let grid = owner.read(cx);
        let Some(row) = grid.rows.get(row_ix) else {
            return div().into_any_element();
        };
        let ix = if grid.rtl {
            grid.columns.len() - 1 - col_ix
        } else {
            col_ix
        };
        let Some(column) = grid.columns.get(ix) else {
            return div().into_any_element();
        };
        let key = (grid.row_id)(row);
        let column_key = column.id.clone();
        let selected = grid
            .active
            .as_ref()
            .is_some_and(|cell| cell.0 == key && cell.1 == column_key);
        let editing = grid
            .edit
            .as_ref()
            .filter(|edit| edit.row == key && edit.column == column_key);
        let content = if let Some(edit) = editing {
            let focus = edit.focus.clone();
            if let Some(handle) = focus.borrow().clone() {
                if window.focused(cx).is_none() {
                    window.on_next_frame(move |window, cx| {
                        if window.focused(cx).is_none() {
                            handle.focus(window, cx);
                        }
                    });
                }
            }
            self.editor
                .borrow()
                .as_ref()
                .map(|editor| editor.clone().into_any_element())
                .unwrap_or_else(|| div().into_any_element())
        } else {
            div().child((column.read)(row)).into_any_element()
        };
        let cell_focus = grid.focus.clone();
        let weak = self.owner.clone();
        let selector = format!("editable-grid-cell:{key}:{column_key}");
        div()
            .id((gpui::ElementId::from(key.clone()), column_key.clone()))
            .w_full()
            .min_h(px(28.))
            .p(px(4.))
            .debug_selector(move || selector.clone())
            .when(selected, |cell| cell.border_1())
            .on_mouse_down(MouseButton::Left, move |event, window, cx| {
                let _ = weak.update(cx, |state, cx| {
                    if state.edit.is_some() && !state.commit_edit(window, cx, true) {
                        window.prevent_default();
                        return;
                    }
                    state.active = Some((key.clone(), column_key.clone()));
                    cell_focus.focus(window, cx);
                    if event.click_count >= 2 {
                        state.begin_edit(None, window, cx);
                    }
                    window.refresh();
                });
            })
            .child(content)
            .into_any_element()
    }
}

impl<R: Clone + 'static> View for EditableDataGridView<R> {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let identity = self.id.clone().unwrap_or_else(|| {
            context.next_implicit_retained_identity(Identity::from("editable-data-grid"))
        });
        let rows = self.rows.value();
        assert_unique_row_ids(&rows, &self.row_id);
        let initial_rows = rows.clone();
        let initial_columns = self.columns.clone();
        let row_id = self.row_id.clone();
        let (state, created) = context.retained_entity(identity.clone(), move |_, cx| {
            if !cx.has_global::<GridKeyBindings>() {
                cx.bind_keys([
                    gpui::KeyBinding::new("tab", NextCell, Some("EditableDataGrid")),
                    gpui::KeyBinding::new("shift-tab", PreviousCell, Some("EditableDataGrid")),
                    gpui::KeyBinding::new("tab", NextCell, Some("EditableDataGrid > Input")),
                    gpui::KeyBinding::new(
                        "shift-tab",
                        PreviousCell,
                        Some("EditableDataGrid > Input"),
                    ),
                ]);
                cx.set_global(GridKeyBindings);
            }

            let active = initial_rows
                .first()
                .zip(initial_columns.first())
                .map(|(row, column)| ((row_id)(row), column.id.clone()));
            GridState {
                rows: initial_rows,
                row_id,
                columns: initial_columns,
                active,
                edit: None,
                focus: cx.focus_handle().tab_stop(true),
                table: None,
                editor: Rc::new(RefCell::new(None)),
                #[cfg(test)]
                rendered_rows: Rc::new(RefCell::new(HashSet::new())),
                focus_out: None,
                commit: None,
                generation: 0,
                rtl: false,
            }
        });
        let columns = self.columns.clone();
        let identify = self.row_id.clone();
        let update =
            context.binding_event_try_updater(self.rows.clone(), move |rows, change: Commit| {
                let Some(row) = rows.iter_mut().find(|row| (identify)(row) == change.row) else {
                    return false;
                };
                let Some(column) = columns
                    .iter()
                    .find(|column| column.id == change.column && column.editable)
                else {
                    return false;
                };
                if column.check(&change.draft).is_err() {
                    return false;
                }
                let mut next = row.clone();
                if let Err(error) = (column.write)(&mut next, &change.draft) {
                    *change.error.borrow_mut() = Some(error);
                    return false;
                }
                if (identify)(&next) != change.row {
                    *change.error.borrow_mut() = Some("Row ID cannot be changed".into());
                    return false;
                }
                *row = next;
                true
            });
        let rtl = context.layout_direction() == rustui_i18n::LayoutDirection::RightToLeft;
        let columns = self.columns.clone();
        let row_id = self.row_id.clone();
        context.update_entity(&state, move |state, _, _| {
            state.rows = rows;
            state.columns = columns;
            state.row_id = row_id;
            state.rtl = rtl;
            state.commit = Some(update);
            if state.active.as_ref().is_some_and(|(row, column)| {
                state.row_index(row).is_none() || state.column_index(column).is_none()
            }) {
                if let Some(edit) = state.edit.take() {
                    *edit.focus.borrow_mut() = None;
                    *state.editor.borrow_mut() = None;
                }
                state.active = state
                    .rows
                    .first()
                    .zip(state.columns.first())
                    .map(|(row, column)| ((state.row_id)(row), column.id.clone()));
            }
        });
        let focus = state.read(context.app()).focus.clone();
        if created {
            let weak = state.downgrade();
            let focus_for_observer = focus.clone();
            let subscription = context.update_entity(&state, move |_, window, cx| {
                window.on_focus_out(&focus_for_observer, cx, move |_, window, cx| {
                    let _ = weak.update(cx, |state, cx| {
                        let editor = state
                            .edit
                            .as_ref()
                            .and_then(|edit| edit.focus.borrow().clone());
                        if editor.is_some() && window.focused(cx) == editor {
                            return;
                        }
                        state.commit_edit(window, cx, false);
                    });
                })
            });
            context.update_entity(&state, move |state, _, _| {
                state.focus_out = Some(subscription)
            });
        }
        let editor = state.read(context.app()).editor.clone();
        #[cfg(test)]
        let rendered_rows = state.read(context.app()).rendered_rows.clone();
        let owner = state.downgrade();
        let table_id = Identity::from(format!("{}:table", identity.as_str()));
        let (table, _) = context.retained_entity(table_id, move |window, cx| {
            TableState::new(
                EditableGridDelegate {
                    owner,
                    editor,
                    #[cfg(test)]
                    rendered_rows,
                },
                window,
                cx,
            )
            .row_selectable(false)
            .col_selectable(false)
            .sortable(false)
            .col_movable(false)
            .col_resizable(false)
        });
        context.update_entity(&table, |table, _, cx| table.refresh(cx));
        let table_weak = table.downgrade();
        context.update_entity(&state, move |state, _, _| state.table = Some(table_weak));
        let weak = state.downgrade();
        let next = weak.clone();
        let previous = weak.clone();
        let root = div()
            .id(identity.to_string())
            .track_focus(&focus)
            .key_context("EditableDataGrid")
            .on_action(move |_: &NextCell, window, cx| {
                let _ = next.update(cx, |state, cx| state.navigate_tab(false, window, cx));
            })
            .on_action(move |_: &PreviousCell, window, cx| {
                let _ = previous.update(cx, |state, cx| state.navigate_tab(true, window, cx));
            })
            .tab_stop(true)
            .flex()
            .flex_col()
            .capture_key_down(move |event, window, cx| {
                let _ = weak.update(cx, |state, cx| state.keyboard(event, window, cx));
            });
        let body = div().h(px(300.)).w_full().child(DataTable::new(&table));
        root.child(body)
            .aria_label(self.label.clone())
            .into_any_element()
    }
}

#[cfg(test)]
#[path = "editable_data_grid_tests.rs"]
mod tests;
