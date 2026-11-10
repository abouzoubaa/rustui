//! Owner-bound editable grid with stable row and column identities.
use std::{
    cell::RefCell,
    cmp::Ordering,
    collections::{HashMap, HashSet},
    fmt::Display,
    rc::Rc,
};

use gpui::prelude::FluentBuilder as _;
use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FocusHandle, InteractiveElement as _,
    IntoElement as _, KeyDownEvent, MouseButton, ParentElement as _, Role, SharedString, Stateful,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakEntity, Window, div, px,
};
use gpui_component::table::{Column, ColumnSort, DataTable, TableDelegate, TableState};
use rustui_core::{Binding, Identity};

use crate::{
    components::input::text_field::TextField,
    runtime::ComponentHost,
    view::{EmptyView, RenderContext, View},
};

gpui::actions!(editable_data_grid, [NextCell, PreviousCell]);

struct GridKeyBindings;
impl gpui::Global for GridKeyBindings {}

type CellReader<R> = Rc<dyn Fn(&R) -> String>;
type CellWriter<R> = Rc<dyn Fn(&mut R, &str) -> Result<(), String>>;
type RowComparator<R> = Rc<dyn Fn(&R, &R) -> Ordering>;
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
    compare: Option<RowComparator<R>>,
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
            compare: self.compare.clone(),
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
            compare: None,
            editable: true,
        }
    }

    /// Adds synchronous validation for the editor's draft.
    pub fn validate(mut self, validator: impl Fn(&str) -> Result<(), String> + 'static) -> Self {
        self.validate = Some(Rc::new(validator));
        self
    }

    /// Enables stable, view-only sorting with a typed row comparator.
    pub fn sortable_by(mut self, compare: impl Fn(&R, &R) -> Ordering + 'static) -> Self {
        self.compare = Some(Rc::new(compare));
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
    original: String,
    focus: Rc<RefCell<Option<FocusHandle>>>,
    error: Rc<RefCell<Option<String>>>,
    generation: usize,
}

struct GridState<R: Clone + 'static> {
    rows: Vec<R>,
    view_order: Vec<SharedString>,
    sort: Option<(SharedString, ColumnSort)>,
    header_focus: Vec<FocusHandle>,
    header_nodes: Rc<RefCell<HashMap<usize, gpui::accesskit::NodeId>>>,
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

    fn view_index(&self, id: &str) -> Option<usize> {
        self.view_order.iter().position(|row| row.as_ref() == id)
    }

    fn update_view_order(&mut self) {
        if self.edit.is_some() {
            let ids: HashSet<_> = self.rows.iter().map(|row| (self.row_id)(row)).collect();
            self.view_order.retain(|id| ids.contains(id));
            for row in &self.rows {
                let id = (self.row_id)(row);
                if !self.view_order.contains(&id) {
                    self.view_order.push(id);
                }
            }
            return;
        }
        let mut indices: Vec<usize> = (0..self.rows.len()).collect();
        if let Some((column_id, direction)) = &self.sort {
            if let Some(compare) = self
                .columns
                .iter()
                .find(|c| &c.id == column_id)
                .and_then(|c| c.compare.as_ref())
            {
                indices.sort_by(|&a, &b| {
                    let result = compare(&self.rows[a], &self.rows[b]);
                    if *direction == ColumnSort::Descending {
                        result.reverse()
                    } else {
                        result
                    }
                });
            }
        }
        self.view_order = indices
            .into_iter()
            .map(|ix| (self.row_id)(&self.rows[ix]))
            .collect();
    }

    fn cycle_sort(&mut self, column: usize, window: &mut Window, cx: &mut App) {
        let Some(column) = self.columns.get(column) else {
            return;
        };
        if column.compare.is_none() {
            return;
        }
        let id = column.id.clone();
        self.sort = match &self.sort {
            Some((active, ColumnSort::Ascending)) if *active == id => {
                Some((id, ColumnSort::Descending))
            }
            Some((active, ColumnSort::Descending)) if *active == id => None,
            _ => Some((id, ColumnSort::Ascending)),
        };
        self.update_view_order();
        window.refresh();
        cx.stop_propagation();
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
            original: (column.read)(&self.rows[row]),
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
        self.update_view_order();
        window.refresh();
    }

    fn commit_edit(&mut self, window: &mut Window, cx: &mut App, restore_focus: bool) -> bool {
        self.commit_edit_inner(window, cx, restore_focus, false)
    }

    fn commit_edit_inner(
        &mut self,
        window: &mut Window,
        cx: &mut App,
        restore_focus: bool,
        retain_editor: bool,
    ) -> bool {
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
        if retain_editor {
            return true;
        }
        if restore_focus {
            self.cancel(window, cx);
        } else {
            if let Some(edit) = self.edit.take() {
                *edit.focus.borrow_mut() = None;
            }
            *self.editor.borrow_mut() = None;
            self.update_view_order();
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
        let (Some(r), Some(c)) = (self.view_index(row), self.column_index(column)) else {
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
        self.active = Some((self.view_order[nr].clone(), self.columns[nc].id.clone()));
        self.scroll_to_row(nr, cx);
        true
    }

    fn tab(&mut self, backward: bool, cx: &mut App) -> bool {
        let Some((row, column)) = self.active.as_ref() else {
            return false;
        };
        let (Some(r), Some(c)) = (self.view_index(row), self.column_index(column)) else {
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
            self.view_order[next / width].clone(),
            self.columns[next % width].id.clone(),
        ));
        self.scroll_to_row(next / width, cx);
        true
    }

    fn navigate_boundary(&mut self, key: &str, command: bool, cx: &mut App) -> bool {
        let Some((row, column)) = self.active.as_ref() else {
            return false;
        };
        let (Some(row), Some(column)) = (self.view_index(row), self.column_index(column)) else {
            return false;
        };
        if self.view_order.is_empty() || self.columns.is_empty() {
            return false;
        }
        let page = self
            .table
            .as_ref()
            .and_then(WeakEntity::upgrade)
            .map(|table| {
                let range = table.read(cx).visible_range().rows();
                range.end.saturating_sub(range.start).max(1)
            })
            .unwrap_or(1);
        let (target_row, target_column) = match key {
            "home" => (if command { 0 } else { row }, 0),
            "end" => (
                if command {
                    self.view_order.len() - 1
                } else {
                    row
                },
                self.columns.len() - 1,
            ),
            "pageup" => (row.saturating_sub(page), column),
            "pagedown" => ((row + page).min(self.view_order.len() - 1), column),
            _ => return false,
        };
        self.active = Some((
            self.view_order[target_row].clone(),
            self.columns[target_column].id.clone(),
        ));
        self.scroll_to_row(target_row, cx);
        true
    }

    fn navigate_tab(&mut self, backward: bool, window: &mut Window, cx: &mut App) {
        let edge = self.active.as_ref().is_some_and(|(row, column)| {
            self.view_index(row)
                .zip(self.column_index(column))
                .is_some_and(|(row, column)| {
                    if backward {
                        row == 0 && column == 0
                    } else {
                        row + 1 == self.view_order.len() && column + 1 == self.columns.len()
                    }
                })
        });
        if edge {
            if self.edit.is_some() && !self.commit_edit_inner(window, cx, false, true) {
                cx.stop_propagation();
                return;
            }
            window.set_focus_scope_inert(&self.focus, true);
            if backward {
                window.focus_prev(cx);
            } else {
                window.focus_next(cx);
            }
            if self.focus.contains_focused(window, cx) {
                window.blur(cx);
            }
            if let Some(edit) = self.edit.take() {
                *edit.focus.borrow_mut() = None;
            }
            *self.editor.borrow_mut() = None;
            window.set_focus_scope_inert(&self.focus, false);
            self.update_view_order();
            window.refresh();
            cx.stop_propagation();
            return;
        }
        if self.edit.is_some() && !self.commit_edit(window, cx, true) {
            cx.stop_propagation();
            return;
        }
        if self.tab(backward, cx) {
            self.focus.focus(window, cx);
            window.refresh();
            cx.stop_propagation();
        } else {
            cx.propagate();
        }
    }

    fn keyboard(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut App) {
        let key = event.keystroke.key.as_str();
        let modifiers = event.keystroke.modifiers;
        if self
            .header_focus
            .iter()
            .any(|focus| focus.is_focused(window))
        {
            return;
        }
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
        if matches!(key, "home" | "end" | "pageup" | "pagedown")
            && !modifiers.control
            && !modifiers.alt
            && (!modifiers.platform || matches!(key, "home" | "end"))
        {
            if self.navigate_boundary(key, modifiers.platform, cx) {
                window.refresh();
                cx.stop_propagation();
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
                if editor.error.borrow().as_deref() != Some("Value changed elsewhere") {
                    *editor.error.borrow_mut() = None;
                }
            },
        );
        let validate = self.validate.clone();
        TextField("", draft)
            .id("editable-data-grid-editor")
            .validate(move |value| validate(value))
            .editor_focus(self.focus.clone())
    }
}

fn grid_accessibility(
    id: String,
    label: SharedString,
    rows: usize,
    columns: usize,
) -> Stateful<gpui::Div> {
    div()
        .id(id)
        .role(Role::Grid)
        .aria_label(label)
        .aria_row_count(rows + 1)
        .aria_column_count(columns)
}

fn grid_row_accessibility(row: usize) -> Stateful<gpui::Div> {
    div()
        .id(("editable-grid-row", row))
        .role(Role::Row)
        .aria_row_index(row + 2)
}

fn grid_header_accessibility(col: usize, label: String) -> Stateful<gpui::Div> {
    div()
        .id(("editable-grid-header", col))
        .role(Role::ColumnHeader)
        .aria_label(label)
        .aria_column_index(col + 1)
}

fn grid_cell_accessibility(
    row: usize,
    col: usize,
    label: String,
    selected: bool,
) -> Stateful<gpui::Div> {
    div()
        .id(format!("editable-grid-cell-a11y:{row}:{col}"))
        .role(Role::Cell)
        .aria_label(label)
        .aria_row_index(row + 2)
        .aria_column_index(col + 1)
        .aria_selected(selected)
        .when(selected, |cell| cell.aria_active_descendant())
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
        let mut result = Column::new(column.id.clone(), column.title.clone())
            .width(px(160.))
            .resizable(false)
            .movable(false);
        if column.compare.is_some() {
            let current = grid
                .sort
                .as_ref()
                .filter(|(id, _)| *id == column.id)
                .map_or(ColumnSort::Default, |(_, direction)| *direction);
            result = result.sort(current);
        }
        result
    }
    fn perform_sort(
        &mut self,
        col_ix: usize,
        _sort: ColumnSort,
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) {
        let _ = self.owner.update(cx, |grid, cx| {
            let index = if grid.rtl {
                grid.columns.len() - 1 - col_ix
            } else {
                col_ix
            };
            grid.cycle_sort(index, window, cx);
        });
    }

    fn render_header(
        &mut self,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> Stateful<gpui::Div> {
        if let Some(owner) = self.owner.upgrade() {
            owner.read(cx).header_nodes.borrow_mut().clear();
        }
        div()
            .id("editable-grid-header-row")
            .role(Role::Row)
            .aria_row_index(1)
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> Stateful<gpui::Div> {
        grid_row_accessibility(row_ix)
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> impl gpui::IntoElement {
        let Some(owner) = self.owner.upgrade() else {
            return div().into_any_element();
        };
        let grid = owner.read(cx);
        let index = if grid.rtl {
            grid.columns.len() - 1 - col_ix
        } else {
            col_ix
        };
        let column = &grid.columns[index];
        let title = column.title.clone();
        let sort = grid
            .sort
            .as_ref()
            .filter(|(id, _)| *id == column.id)
            .map(|(_, direction)| *direction);
        let accessible_title = match sort {
            Some(ColumnSort::Ascending) => format!("{}, ascending", title),
            Some(ColumnSort::Descending) => format!("{}, descending", title),
            _ => title.to_string(),
        };
        let header_nodes = grid.header_nodes.clone();
        let header = grid_header_accessibility(col_ix, accessible_title).a11y_synthetic_children(
            move |tree| {
                header_nodes
                    .borrow_mut()
                    .insert(col_ix, tree.parent_node_id());
            },
        );
        if column.compare.is_none() {
            let selector = format!("editable-grid-header:{}", column.id);
            return header
                .size_full()
                .debug_selector(move || selector.clone())
                .child(title)
                .into_any_element();
        }
        let focus = grid.header_focus[index].clone();
        let click_focus = focus.clone();
        let click_owner = self.owner.clone();
        let key_owner = self.owner.clone();
        let selector = format!("editable-grid-header:{}", column.id);
        header
            .when_some(
                sort.map(|direction| match direction {
                    ColumnSort::Ascending => gpui::accesskit::SortDirection::Ascending,
                    ColumnSort::Descending => gpui::accesskit::SortDirection::Descending,
                    _ => gpui::accesskit::SortDirection::Other,
                }),
                |header, direction| header.aria_sort(direction),
            )
            .size_full()
            .track_focus(&focus)
            .tab_stop(true)
            .debug_selector(move || selector.clone())
            .on_mouse_down(MouseButton::Left, move |_, window, cx| {
                click_focus.focus(window, cx);
            })
            .on_click(move |_, window, cx| {
                let _ = click_owner.update(cx, |grid, cx| grid.cycle_sort(index, window, cx));
            })
            .on_key_down(move |event, window, cx| {
                if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                    let _ = key_owner.update(cx, |grid, cx| grid.cycle_sort(index, window, cx));
                }
            })
            .child(title)
            .into_any_element()
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
        let Some(id) = grid.view_order.get(row_ix) else {
            return div().into_any_element();
        };
        let Some(row) = grid.row_index(id).and_then(|index| grid.rows.get(index)) else {
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
        let accessible_value = editing
            .map(|edit| edit.draft.borrow().clone())
            .unwrap_or_else(|| (column.read)(row));
        let mut accessible_label = if column.editable {
            format!("{}, {}", column.title, accessible_value)
        } else {
            format!("{}, {}, read only", column.title, accessible_value)
        };
        if let Some(error) = editing.and_then(|edit| edit.error.borrow().clone()) {
            accessible_label.push_str(&format!(", {error}"));
        }
        let cell_focus = grid.focus.clone();
        let weak = self.owner.clone();
        let selector = format!("editable-grid-cell:{key}:{column_key}");
        let header_nodes = grid.header_nodes.clone();
        grid_cell_accessibility(row_ix, col_ix, accessible_label, selected)
            .a11y_synthetic_children(move |tree| {
                if let Some(header) = header_nodes.borrow().get(&col_ix).copied() {
                    tree.parent_node().set_labelled_by(vec![header]);
                }
            })
            .aria_readonly(!column.editable)
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
        let row_count = rows.len();
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
                view_order: initial_rows.iter().map(|row| row_id(row)).collect(),
                sort: None,
                header_focus: initial_columns
                    .iter()
                    .map(|_| cx.focus_handle().tab_stop(true))
                    .collect(),
                header_nodes: Rc::new(RefCell::new(HashMap::new())),
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
        context.update_entity(&state, move |state, _, cx| {
            state.rows = rows;
            state.columns = columns;
            state.row_id = row_id;
            if let Some(edit) = &state.edit {
                if let (Some(row), Some(column)) =
                    (state.row_index(&edit.row), state.column_index(&edit.column))
                {
                    let changed = (state.columns[column].read)(&state.rows[row]) != edit.original;
                    let mut error = edit.error.borrow_mut();
                    if changed {
                        *error = Some("Value changed elsewhere".into());
                    } else if error.as_deref() == Some("Value changed elsewhere") {
                        *error = None;
                    }
                    drop(error);
                    if let Some(editor) = state.editor.borrow().as_ref() {
                        editor.update(cx, |_, cx| cx.notify());
                    }
                }
            }
            state.rtl = rtl;
            state.commit = Some(update);
            while state.header_focus.len() < state.columns.len() {
                state.header_focus.push(cx.focus_handle().tab_stop(true));
            }
            state.update_view_order();
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
            .sortable(true)
            .col_movable(false)
            .col_resizable(false)
            .delegate_accessibility(true)
        });
        context.update_entity(&table, |table, _, cx| table.refresh(cx));
        let table_weak = table.downgrade();
        context.update_entity(&state, move |state, _, _| state.table = Some(table_weak));
        let weak = state.downgrade();
        let next = weak.clone();
        let previous = weak.clone();
        let root = grid_accessibility(
            identity.to_string(),
            self.label.clone(),
            row_count,
            self.columns.len(),
        )
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
        root.child(body).into_any_element()
    }
}

#[cfg(test)]
#[path = "editable_data_grid_tests.rs"]
mod tests;
