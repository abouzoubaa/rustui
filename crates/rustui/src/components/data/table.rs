//! Controlled virtualized string table backed by retained GPUI TableState.

use std::{cell::Cell, rc::Rc};

use gpui::{
    AnyElement, App, Context, InteractiveElement as _, IntoElement, ParentElement as _,
    SharedString, Stateful, Styled as _, Window, div, px,
};
use gpui_component::table::{Column, DataTable, TableDelegate, TableEvent, TableState};
use rustui_core::{Binding, Identity};

use crate::{
    components::theme::{required_style, required_style_with_artwork, selector, styled_element},
    theme::ResolvedComponentStyle,
    view::{EmptyView, RenderContext, View},
};

struct StringTableDelegate {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    column_width: f32,
    row_style: ResolvedComponentStyle,
    header_style: ResolvedComponentStyle,
    last_owner_selection: Rc<Cell<Option<usize>>>,
}

impl StringTableDelegate {
    fn new(
        columns: Vec<String>,
        rows: Vec<Vec<String>>,
        column_width: f32,
        row_style: ResolvedComponentStyle,
        header_style: ResolvedComponentStyle,
        owner_selection: Option<usize>,
    ) -> Self {
        Self {
            columns,
            rows,
            column_width,
            row_style,
            header_style,
            last_owner_selection: Rc::new(Cell::new(owner_selection)),
        }
    }

    fn replace_data(&mut self, columns: Vec<String>, rows: Vec<Vec<String>>, column_width: f32) {
        self.columns = columns;
        self.rows = rows;
        self.column_width = column_width;
    }
}

impl TableDelegate for StringTableDelegate {
    fn columns_count(&self, _cx: &App) -> usize {
        self.columns.len()
    }

    fn rows_count(&self, _cx: &App) -> usize {
        self.rows.len()
    }

    fn column(&self, col_ix: usize, _cx: &App) -> Column {
        let name = self.columns.get(col_ix).cloned().unwrap_or_default();
        Column::new(format!("rustui-table-column-{col_ix}"), name)
            .width(px(self.column_width))
            .resizable(false)
            .movable(false)
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let name = self.columns.get(col_ix).cloned().unwrap_or_default();
        // GPUI Components paints the header surface with mapped theme colors.
        // Style only the label here so its background remains visible.
        let mut label = div().size_full().child(SharedString::from(name));
        if let Some(typography) = &self.header_style.typography {
            label = label
                .font_family(typography.family.clone())
                .text_size(px(typography.size))
                .font_weight(gpui::FontWeight(typography.weight));
        }
        label
    }

    fn render_tr(
        &mut self,
        row_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> Stateful<gpui::Div> {
        styled_element(div().id(("rustui-table-row", row_ix)), &self.row_style)
    }

    fn render_td(
        &mut self,
        row_ix: usize,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let value = self
            .rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default();
        div()
            .size_full()
            .flex()
            .items_center()
            .px_3()
            .child(SharedString::from(value))
    }

    fn cell_text(&self, row_ix: usize, col_ix: usize, _cx: &App) -> String {
        self.rows
            .get(row_ix)
            .and_then(|row| row.get(col_ix))
            .cloned()
            .unwrap_or_default()
    }
}

/// A virtualized string table with controlled row selection.
pub struct TableView {
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    selection: Binding<Option<usize>>,
    id: Option<Identity>,
    height: f32,
    column_width: f32,
    striped: bool,
    bordered: bool,
    vertical_scrollbar: bool,
    horizontal_scrollbar: bool,
}

impl TableView {
    pub fn new<C, CS, R, RS, S>(columns: C, rows: R, selection: Binding<Option<usize>>) -> Self
    where
        C: IntoIterator<Item = CS>,
        CS: Into<String>,
        R: IntoIterator<Item = RS>,
        RS: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let columns = columns.into_iter().map(Into::into).collect::<Vec<_>>();
        let rows = rows
            .into_iter()
            .map(|row| row.into_iter().map(Into::into).collect::<Vec<_>>())
            .collect::<Vec<_>>();
        assert!(
            rows.iter().all(|row| row.len() == columns.len()),
            "every RustUI Table row must contain exactly one cell per column"
        );

        Self {
            columns,
            rows,
            selection,
            id: None,
            height: 300.0,
            column_width: 160.0,
            striped: false,
            bordered: true,
            vertical_scrollbar: true,
            horizontal_scrollbar: true,
        }
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        self.id = Some(id.into());
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = height.max(1.0);
        self
    }

    pub fn column_width(mut self, width: f32) -> Self {
        self.column_width = width.max(1.0);
        self
    }

    pub fn striped(mut self, striped: bool) -> Self {
        self.striped = striped;
        self
    }

    pub fn bordered(mut self, bordered: bool) -> Self {
        self.bordered = bordered;
        self
    }

    pub fn scrollbar_visible(mut self, vertical: bool, horizontal: bool) -> Self {
        self.vertical_scrollbar = vertical;
        self.horizontal_scrollbar = horizontal;
        self
    }

    fn identity(&self) -> Identity {
        self.id.clone().unwrap_or_else(|| {
            Identity::from(format!(
                "table:{}:{}:{}",
                self.columns.len(),
                self.rows.len(),
                self.columns.first().map_or("", String::as_str)
            ))
        })
    }
}

fn selection_for_rows(selection: Option<usize>, row_count: usize) -> Option<usize> {
    selection.filter(|index| *index < row_count)
}

fn clear_invalid_selection(current: &mut Option<usize>, invalid: usize) -> bool {
    if *current != Some(invalid) {
        return false;
    }
    *current = None;
    true
}

#[allow(non_snake_case)]
pub fn Table<C, CS, R, RS, S>(columns: C, rows: R, selection: Binding<Option<usize>>) -> TableView
where
    C: IntoIterator<Item = CS>,
    CS: Into<String>,
    R: IntoIterator<Item = RS>,
    RS: IntoIterator<Item = S>,
    S: Into<String>,
{
    TableView::new(columns, rows, selection)
}

impl View for TableView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        let query = selector(context);
        let row_style = required_style(context, "table", "table_rows", &query);
        let header_style =
            required_style_with_artwork(context, "table", "table_column_headers", &query);
        let identity = self.identity();
        let columns = self.columns.clone();
        let rows = self.rows.clone();
        let column_width = self.column_width;
        let initial_row_style = row_style.clone();
        let initial_header_style = header_style.clone();
        let owner_selection = self.selection.value();
        let initial_selection = selection_for_rows(owner_selection, rows.len());
        let (state, created) = context.retained_entity(identity.clone(), move |window, cx| {
            let delegate = StringTableDelegate::new(
                columns,
                rows,
                column_width,
                initial_row_style,
                initial_header_style,
                owner_selection,
            );
            let mut state = TableState::new(delegate, window, cx)
                .row_selectable(true)
                .col_selectable(false)
                .sortable(false)
                .col_movable(false)
                .col_resizable(false);
            if let Some(row) = initial_selection {
                state.set_selected_row(row, cx);
            }
            state
        });

        let data_changed = {
            let state = state.read(context.app());
            let delegate = state.delegate();
            delegate.columns != self.columns
                || delegate.rows != self.rows
                || delegate.column_width != self.column_width
                || delegate.row_style != row_style
                || delegate.header_style != header_style
        };
        if data_changed {
            let columns = self.columns.clone();
            let rows = self.rows.clone();
            let column_width = self.column_width;
            let row_style = row_style.clone();
            let header_style = header_style.clone();
            context.update_entity(&state, move |state, _window, cx| {
                state
                    .delegate_mut()
                    .replace_data(columns, rows, column_width);
                state.delegate_mut().row_style = row_style;
                state.delegate_mut().header_style = header_style;
                state.refresh(cx);
            });
        }

        let desired = selection_for_rows(owner_selection, self.rows.len());
        let selected = state.read(context.app()).selected_row();
        if selected != desired {
            context.update_entity(&state, move |state, _window, cx| match desired {
                Some(row) => state.set_selected_row(row, cx),
                None => state.clear_selection(cx),
            });
        }

        let last_owner_selection = state
            .read(context.app())
            .delegate()
            .last_owner_selection
            .clone();
        last_owner_selection.set(owner_selection);

        if created {
            let requested = Rc::new(Cell::new(None));
            let current_request = requested.clone();
            let last_owner_selection = last_owner_selection.clone();
            let update = context.binding_updater(self.selection.clone(), move |current| {
                let selected = current_request.get();
                if *current != last_owner_selection.get() || *current == selected {
                    return false;
                }
                *current = selected;
                last_owner_selection.set(selected);
                true
            });
            context.subscribe_retained(&identity, &state, move |state, event: &TableEvent, cx| {
                let selection = match event {
                    TableEvent::SelectRow(row) => Some(*row),
                    TableEvent::ClearSelection => None,
                    _ => return,
                };
                if state.read(cx).selected_row() != selection {
                    return;
                }
                requested.set(selection);
                update(cx);
            });
        }

        if let Some(invalid) = owner_selection.filter(|index| *index >= self.rows.len()) {
            let normalize = context.binding_updater(self.selection.clone(), move |current| {
                clear_invalid_selection(current, invalid)
            });
            context.update_entity(&state, move |_state, _window, cx| {
                cx.defer(move |cx| normalize(cx));
            });
        }

        div()
            .w_full()
            .h(px(self.height))
            .child(
                DataTable::new(&state)
                    .stripe(self.striped)
                    .bordered(self.bordered)
                    .scrollbar_visible(self.vertical_scrollbar, self.horizontal_scrollbar),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{clear_invalid_selection, selection_for_rows};

    #[test]
    fn initial_invalid_selection_is_normalized_even_without_native_selection() {
        let mut owner = Some(7);
        assert_eq!(selection_for_rows(owner, 4), None);
        assert!(clear_invalid_selection(&mut owner, 7));
        assert_eq!(owner, None);
    }

    #[test]
    fn shrinking_rows_invalidates_and_clears_the_selected_source_index() {
        let mut owner = Some(2);
        assert_eq!(selection_for_rows(owner, 2), None);
        assert!(clear_invalid_selection(&mut owner, 2));
        assert_eq!(owner, None);
    }

    #[test]
    fn reordering_rows_keeps_a_valid_source_index() {
        assert_eq!(selection_for_rows(Some(1), 3), Some(1));
    }

    #[test]
    fn stale_invalid_normalization_preserves_a_newer_valid_selection() {
        let mut owner = Some(1);
        assert!(!clear_invalid_selection(&mut owner, 7));
        assert_eq!(owner, Some(1));
    }
}
