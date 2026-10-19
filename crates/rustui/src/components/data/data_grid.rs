//! Accessibility wrapper for a table with selectable rows.

use gpui::{
    div, AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, Role, SharedString,
    StatefulInteractiveElement as _, Styled as _,
};

use crate::view::{EmptyView, RenderContext, View};

use super::table::TableView;

/// Gives an existing [`TableView`] a named native table role.
///
/// GPUI Components already exposes each data row with the native row role.
/// This wrapper names the table and supplies its missing table-level role.
/// It does not add column-header or cell roles, or change keyboard behavior.
pub struct DataGridView {
    table: TableView,
    label: SharedString,
    id: Option<SharedString>,
}

impl DataGridView {
    /// Wrap a table with its accessible name.
    pub fn new(table: TableView, label: impl Into<SharedString>) -> Self {
        let label = label.into();
        assert!(
            !label.trim().is_empty(),
            "DataGrid accessible label must be nonempty"
        );
        Self {
            table,
            label,
            id: None,
        }
    }

    /// Sets a stable unique identity when sibling grids can share a label.
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        self.id = Some(id.into());
        self
    }
}

/// Adds native table semantics and an accessible name to a [`TableView`].
#[allow(non_snake_case)]
pub fn DataGrid(table: TableView, label: impl Into<SharedString>) -> DataGridView {
    DataGridView::new(table, label)
}

impl View for DataGridView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        div()
            .id(format!(
                "rustui-data-grid:{}",
                self.id.as_ref().unwrap_or(&self.label)
            ))
            .role(Role::Table)
            .aria_label(self.label.clone())
            .w_full()
            .child(self.table.into_element(context))
            .into_any_element()
    }
}
