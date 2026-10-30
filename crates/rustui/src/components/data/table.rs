//! Controlled virtualized string table backed by retained GPUI TableState.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use gpui::{
    AnyElement, App, Context, InteractiveElement as _, IntoElement, ParentElement as _, Role,
    SharedString, Stateful, StatefulInteractiveElement as _, Styled as _, Window, div, px,
};
use gpui_component::table::{Column, DataTable, TableDelegate, TableEvent, TableState};
use rustui_core::{Binding, Identity};
use rustui_i18n::LayoutDirection;

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
    requested_selection: Rc<Cell<Option<usize>>>,
    selection_update: Rc<RefCell<Option<SelectionUpdater<App>>>>,
}

type SelectionUpdater<T> = Rc<dyn Fn(&mut T)>;

fn table_header_element() -> Stateful<gpui::Div> {
    div()
        .id("rustui-table-header")
        .role(Role::Row)
        .aria_row_index(TABLE_HEADER_ROW_INDEX)
}

fn table_column_header_element(
    col_ix: usize,
    name: String,
    accessible_name: String,
) -> Stateful<gpui::Div> {
    div()
        .id(("rustui-table-column-header", col_ix))
        .size_full()
        .role(Role::ColumnHeader)
        .aria_label(accessible_name)
        .aria_column_index(col_ix + 1)
        .child(SharedString::from(name))
}

fn table_data_row_element(row_ix: usize) -> Stateful<gpui::Div> {
    div()
        .id(("rustui-table-row", row_ix))
        .aria_row_index(table_data_row_aria_index(row_ix))
}

fn should_report_active_descendant(selected: bool, table_focused: bool) -> bool {
    selected && table_focused
}

fn table_data_cell_element(
    row_ix: usize,
    col_ix: usize,
    value: String,
    accessible_name: String,
) -> Stateful<gpui::Div> {
    div()
        .id(format!("rustui-table-cell-content:{row_ix}:{col_ix}"))
        .size_full()
        .role(Role::Cell)
        .aria_label(accessible_name)
        .aria_column_index(col_ix + 1)
        .flex()
        .items_center()
        .px_3()
        .child(SharedString::from(value))
}

fn replace_selection_updater<T>(
    slot: &RefCell<Option<SelectionUpdater<T>>>,
    updater: SelectionUpdater<T>,
) {
    *slot.borrow_mut() = Some(updater);
}

fn apply_selection_updater<T>(slot: &RefCell<Option<SelectionUpdater<T>>>, context: &mut T) {
    let updater = slot.borrow().clone();
    if let Some(updater) = updater {
        updater(context);
    }
}

fn apply_owner_selection_request(
    current: &mut Option<usize>,
    last_owner_selection: &Cell<Option<usize>>,
    requested: Option<usize>,
) -> bool {
    if *current != last_owner_selection.get() || *current == requested {
        return false;
    }
    *current = requested;
    last_owner_selection.set(requested);
    true
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
            requested_selection: Rc::new(Cell::new(None)),
            selection_update: Rc::new(RefCell::new(None)),
        }
    }

    fn replace_data(&mut self, columns: Vec<String>, rows: Vec<Vec<String>>, column_width: f32) {
        self.columns = columns;
        self.rows = rows;
        self.column_width = column_width;
    }

    fn accessible_column_name(&self, col_ix: usize) -> String {
        self.columns
            .get(col_ix)
            .filter(|name| !name.trim().is_empty())
            .cloned()
            .unwrap_or_else(|| format!("Column {}", col_ix + 1))
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

    fn render_header(
        &mut self,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> Stateful<gpui::Div> {
        table_header_element()
    }

    fn render_th(
        &mut self,
        col_ix: usize,
        _window: &mut Window,
        _cx: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        let name = self.columns.get(col_ix).cloned().unwrap_or_default();
        let accessible_name = self.accessible_column_name(col_ix);
        // GPUI Components paints the header surface with mapped theme colors.
        // Style only the label here so its background remains visible.
        let mut label = table_column_header_element(col_ix, name, accessible_name);
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
        window: &mut Window,
        cx: &mut Context<TableState<Self>>,
    ) -> Stateful<gpui::Div> {
        let selected = self.last_owner_selection.get() == Some(row_ix);
        let focused = cx.focus_handle().is_focused(window);
        let mut row = styled_element(table_data_row_element(row_ix), &self.row_style);
        if should_report_active_descendant(selected, focused) {
            row = row.aria_active_descendant();
        }
        row
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
        let accessible_name = format!("{}: {value}", self.accessible_column_name(col_ix));
        table_data_cell_element(row_ix, col_ix, value, accessible_name)
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
///
/// Right-to-left locales display columns in reverse visual order while retaining
/// each header's association with its cells and preserving source-row selection.
pub struct TableView {
    columns: Vec<String>,
    column_messages: Option<Vec<String>>,
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
            column_messages: None,
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

    /// Resolves one localization message per column, falling back to its original label.
    ///
    /// Panics if the number of message IDs differs from the number of columns or
    /// any ID is blank. Labels are resolved again when the active locale changes.
    pub fn localized_columns<I, S>(mut self, messages: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let messages: Vec<String> = messages.into_iter().map(Into::into).collect();
        assert!(
            messages.len() == self.columns.len(),
            "Table needs one message ID per column"
        );
        assert!(
            messages.iter().all(|id| !id.trim().is_empty()),
            "Table column message IDs must be nonempty"
        );
        self.column_messages = Some(messages);
        self
    }

    fn resolved_columns(&self, mut resolve: impl FnMut(&str) -> String) -> Vec<String> {
        match &self.column_messages {
            None => self.columns.clone(),
            Some(messages) => self
                .columns
                .iter()
                .zip(messages)
                .map(|(fallback, id)| {
                    let label = resolve(id);
                    if label.trim().is_empty() || label.trim() == id.trim() {
                        fallback.clone()
                    } else {
                        label
                    }
                })
                .collect(),
        }
    }

    pub(crate) fn aria_row_count(&self) -> usize {
        self.rows.len() + TABLE_HEADER_ROW_INDEX
    }

    pub(crate) fn aria_column_count(&self) -> usize {
        self.columns.len()
    }

    pub fn id(mut self, id: impl Into<Identity>) -> Self {
        let id = id.into();
        assert!(
            !id.as_str().trim().is_empty(),
            "Table identity must be nonempty"
        );
        self.id = Some(id);
        self
    }

    pub fn height(mut self, height: f32) -> Self {
        self.height = finite_dimension(height);
        self
    }

    pub fn column_width(mut self, width: f32) -> Self {
        self.column_width = finite_dimension(width);
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

    fn identity(&self, context: &mut RenderContext<'_>) -> Identity {
        let Some(id) = &self.id else {
            return context.next_implicit_retained_identity(Identity::from("table"));
        };
        id.clone()
    }
}

fn presentation_data(
    mut columns: Vec<String>,
    mut rows: Vec<Vec<String>>,
    direction: LayoutDirection,
) -> (Vec<String>, Vec<Vec<String>>) {
    if direction == LayoutDirection::RightToLeft {
        columns.reverse();
        for row in &mut rows {
            row.reverse();
        }
    }
    (columns, rows)
}

fn selection_for_rows(selection: Option<usize>, row_count: usize) -> Option<usize> {
    selection.filter(|index| *index < row_count)
}

fn finite_dimension(value: f32) -> f32 {
    if value.is_finite() {
        value.max(1.0)
    } else {
        1.0
    }
}

const TABLE_HEADER_ROW_INDEX: usize = 1;

fn table_data_row_aria_index(row_ix: usize) -> usize {
    // ARIA row indices are one-based, and this table has one header row.
    row_ix + TABLE_HEADER_ROW_INDEX + 1
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
        let identity = self.identity(context);
        let (display_columns, display_rows) = presentation_data(
            self.resolved_columns(|id| context.localized_message(id)),
            self.rows.clone(),
            context.layout_direction(),
        );
        let columns = display_columns.clone();
        let rows = display_rows.clone();
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
            delegate.columns != display_columns
                || delegate.rows != display_rows
                || delegate.column_width != self.column_width
                || delegate.row_style != row_style
                || delegate.header_style != header_style
        };
        if data_changed {
            let columns = display_columns;
            let rows = display_rows;
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

        let (last_owner_selection, requested_selection, selection_update) = {
            let delegate = state.read(context.app()).delegate();
            (
                delegate.last_owner_selection.clone(),
                delegate.requested_selection.clone(),
                delegate.selection_update.clone(),
            )
        };
        last_owner_selection.set(owner_selection);
        let marker = last_owner_selection.clone();
        let current_request = requested_selection.clone();
        let updater = context.binding_updater(self.selection.clone(), move |current| {
            apply_owner_selection_request(current, &marker, current_request.get())
        });
        replace_selection_updater(&selection_update, updater);

        if created {
            let current_update = selection_update.clone();
            let requested = requested_selection.clone();
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
                apply_selection_updater(&current_update, cx);
            });
        }

        if let Some(invalid) = owner_selection.filter(|index| *index >= self.rows.len()) {
            let normalize = context.binding_updater(self.selection.clone(), move |current| {
                clear_invalid_selection(current, invalid)
            });
            let current_state = state.clone();
            context.update_entity(&state, move |_state, _window, cx| {
                cx.defer(move |cx| {
                    if invalid >= current_state.read(cx).delegate().rows.len() {
                        normalize(cx);
                    }
                });
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
    use super::View;
    use gpui::{Element, Role, accesskit::Node};

    struct RestoredRowOwner {
        selection: Option<usize>,
        rows: usize,
        writes: usize,
    }

    impl super::View for RestoredRowOwner {
        fn body(&self) -> impl super::View {
            super::EmptyView
        }

        fn into_element(&self, context: &mut super::RenderContext<'_>) -> gpui::AnyElement {
            let selection = super::Binding::new(
                self.selection,
                |owner: &Self| owner.selection,
                |owner, value| {
                    owner.selection = value;
                    owner.writes += 1;
                },
            );
            let rows = (0..self.rows).map(|index| vec![index.to_string()]);
            super::TableView::new(["Value"], rows, selection).into_element(context)
        }
    }

    #[gpui::test]
    fn restored_row_survives_queued_selection_clear(cx: &mut gpui::TestAppContext) {
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
        let (owner, window) = cx.add_window_view(|_, app| {
            crate::runtime::ComponentHost::new(
                RestoredRowOwner {
                    selection: Some(1),
                    rows: 2,
                    writes: 0,
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            owner.update(app, |host, cx| {
                host.component_mut().rows = 1;
                cx.notify();
            });
            let _ = window.draw(app);
            owner.update(app, |host, cx| {
                host.component_mut().rows = 2;
                cx.notify();
            });
            let _ = window.draw(app);
            assert_eq!(
                owner.read(app).component().selection,
                Some(1),
                "after restoration draw"
            );
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(owner.read(app).component().selection, Some(1));
            assert_eq!(owner.read(app).component().writes, 0);
        });
        window.update(|window, app| {
            owner.update(app, |host, cx| {
                host.component_mut().rows = 1;
                cx.notify();
            });
            let _ = window.draw(app);
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(owner.read(app).component().selection, None);
            assert_eq!(owner.read(app).component().writes, 1);
        });
    }

    use super::{
        SelectionUpdater, TableView, apply_owner_selection_request, apply_selection_updater,
        clear_invalid_selection, finite_dimension, replace_selection_updater, selection_for_rows,
        should_report_active_descendant, table_column_header_element, table_data_cell_element,
        table_data_row_element, table_header_element,
    };
    use rustui_core::Binding;
    use std::{cell::Cell, cell::RefCell, rc::Rc};

    struct Owner {
        primary: Option<usize>,
        secondary: Option<usize>,
    }

    fn read_primary(owner: &Owner) -> Option<usize> {
        owner.primary
    }

    fn write_primary(owner: &mut Owner, selection: Option<usize>) {
        owner.primary = selection;
    }

    fn read_secondary(owner: &Owner) -> Option<usize> {
        owner.secondary
    }

    fn write_secondary(owner: &mut Owner, selection: Option<usize>) {
        owner.secondary = selection;
    }

    fn read_selection(_: &()) -> Option<usize> {
        None
    }

    fn write_selection(_: &mut (), _: Option<usize>) {}

    #[test]
    fn localized_columns_resolve_and_fall_back_per_label() {
        let table = TableView::new(
            ["Index", "Row"],
            [["0", "Alpha"]],
            Binding::new(None, read_selection, write_selection),
        )
        .localized_columns(["table.index", "table.row"]);
        assert_eq!(
            table.resolved_columns(|id| match id {
                "table.index" => "Indice".into(),
                _ => id.into(),
            }),
            ["Indice", "Row"]
        );
        assert_eq!(table.resolved_columns(|_| " ".into()), ["Index", "Row"]);
        assert_eq!(table.resolved_columns(|_| "صف".into()), ["صف", "صف"]);
    }

    #[test]
    fn rtl_presentation_reorders_columns_without_reordering_rows() {
        let columns = vec!["Index".to_owned(), "Name".to_owned()];
        let rows = vec![
            vec!["0".to_owned(), "Alpha".to_owned()],
            vec!["1".to_owned(), "Beta".to_owned()],
        ];
        assert_eq!(
            super::presentation_data(
                columns.clone(),
                rows.clone(),
                rustui_i18n::LayoutDirection::LeftToRight,
            ),
            (columns.clone(), rows.clone())
        );
        assert_eq!(
            super::presentation_data(columns, rows, rustui_i18n::LayoutDirection::RightToLeft),
            (
                vec!["Name".to_owned(), "Index".to_owned()],
                vec![
                    vec!["Alpha".to_owned(), "0".to_owned()],
                    vec!["Beta".to_owned(), "1".to_owned()],
                ],
            )
        );
    }

    #[test]
    #[should_panic(expected = "Table needs one message ID per column")]
    fn localized_columns_reject_wrong_count() {
        let _ = TableView::new(
            ["Index", "Row"],
            [["0", "Alpha"]],
            Binding::new(None, read_selection, write_selection),
        )
        .localized_columns(["table.index"]);
    }

    #[test]
    #[should_panic(expected = "Table column message IDs must be nonempty")]
    fn localized_columns_reject_blank_ids() {
        let _ = TableView::new(
            ["Index"],
            [["0"]],
            Binding::new(None, read_selection, write_selection),
        )
        .localized_columns([" "]);
    }

    #[test]
    fn accessibility_counts_include_header_row_and_all_columns() {
        let table = TableView::new(
            ["Quarter", "Revenue"],
            [["Q1", "$18.2M"], ["Q2", "$20.4M"]],
            Binding::new(None, read_selection, write_selection),
        );

        assert_eq!(table.aria_row_count(), 3);
        assert_eq!(table.aria_column_count(), 2);
    }

    #[test]
    fn non_finite_table_dimensions_fall_back_to_the_minimum_size() {
        assert_eq!(finite_dimension(f32::INFINITY), 1.0);
        assert_eq!(finite_dimension(f32::NEG_INFINITY), 1.0);
        assert_eq!(finite_dimension(f32::NAN), 1.0);
        assert_eq!(finite_dimension(24.0), 24.0);
        assert_eq!(finite_dimension(0.0), 1.0);
    }

    fn accesskit_node(element: &impl Element, fallback_role: Option<Role>) -> Node {
        let role = element
            .a11y_role()
            .or(fallback_role)
            .expect("table accessibility element has a role");
        let mut node = Node::new(role);
        element.write_a11y_info(&mut node);
        node
    }

    #[test]
    fn header_and_cell_elements_emit_accesskit_names_roles_and_indices() {
        let header_row = accesskit_node(&table_header_element(), None);
        assert_eq!(header_row.role(), Role::Row);
        assert_eq!(header_row.row_index(), Some(1));

        let header = table_column_header_element(1, "Revenue".into(), "Revenue".into());
        let header = accesskit_node(&header, None);
        assert_eq!(header.role(), Role::ColumnHeader);
        assert_eq!(header.label(), Some("Revenue"));
        assert_eq!(header.column_index(), Some(2));

        let cell = table_data_cell_element(1, 0, "$20.4M".into(), "Quarter: $20.4M".into());
        let cell = accesskit_node(&cell, None);
        assert_eq!(cell.role(), Role::Cell);
        assert_eq!(cell.label(), Some("Quarter: $20.4M"));
        assert_eq!(cell.column_index(), Some(1));

        // The pinned DataTable adds the Row role to this same delegate element.
        let row = accesskit_node(&table_data_row_element(1), Some(Role::Row));
        assert_eq!(row.row_index(), Some(3));
    }

    #[test]
    fn active_descendant_requires_the_selected_row_and_table_focus() {
        assert!(!should_report_active_descendant(false, false));
        assert!(!should_report_active_descendant(false, true));
        assert!(!should_report_active_descendant(true, false));
        assert!(should_report_active_descendant(true, true));
    }

    #[test]
    #[should_panic(expected = "Table identity must be nonempty")]
    fn rejects_whitespace_only_retained_identity() {
        TableView::new(
            ["Column"],
            [["value"]],
            Binding::new(None, read_selection, write_selection),
        )
        .id(" \t");
    }

    #[test]
    fn retained_table_subscription_uses_the_latest_selection_binding() {
        let primary = Binding::new(Some(0), read_primary, write_primary);
        let secondary = Binding::new(Some(1), read_secondary, write_secondary);
        let last_owner_selection = Rc::new(Cell::new(Some(1)));
        let make_updater = |binding: Binding<Option<usize>>| {
            let marker = last_owner_selection.clone();
            Rc::new(move |owner: &mut Owner| {
                binding.update_on(owner, |current| {
                    apply_owner_selection_request(current, &marker, Some(2))
                });
            }) as SelectionUpdater<Owner>
        };
        let current_update = RefCell::new(None);

        replace_selection_updater(&current_update, make_updater(primary));
        replace_selection_updater(&current_update, make_updater(secondary));

        let mut owner = Owner {
            primary: Some(0),
            secondary: Some(1),
        };
        apply_selection_updater(&current_update, &mut owner);

        assert_eq!(owner.primary, Some(0));
        assert_eq!(owner.secondary, Some(2));
    }

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
