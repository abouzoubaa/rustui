//! Accessibility wrapper for a table with selectable rows.

use gpui::{
    AnyElement, InteractiveElement as _, IntoElement, ParentElement as _, Role, SharedString,
    Stateful, StatefulInteractiveElement as _, Styled as _, div,
};
use rustui_core::Identity;

use crate::view::{EmptyView, RenderContext, View};

use super::table::TableView;

/// Gives an existing [`TableView`] a named native table role.
///
/// GPUI Components already exposes each data row with the native row role.
/// This wrapper names the table and supplies its missing table-level role.
/// The wrapped [`TableView`] delegate supplies named column-header and cell
/// roles; the wrapper does not establish native header-to-cell associations
/// or change keyboard behavior.
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
        let table = table.id(table_identity(&label, None));
        Self {
            table,
            label,
            id: None,
        }
    }

    /// Sets a stable unique identity when sibling grids can share a label.
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        let id = id.into();
        self.table = self.table.id(table_identity(&self.label, Some(&id)));
        self.id = Some(id);
        self
    }
}

fn table_identity(label: &SharedString, id: Option<&SharedString>) -> Identity {
    assert!(
        id.is_none_or(|id| !id.trim().is_empty()),
        "DataGrid IDs must be nonempty"
    );
    Identity::from(format!("rustui-data-grid-table:{}", id.unwrap_or(label)))
}

fn data_grid_element(
    id: String,
    label: SharedString,
    row_count: usize,
    column_count: usize,
) -> Stateful<gpui::Div> {
    div()
        .id(id)
        .role(Role::Table)
        .aria_label(label)
        .aria_row_count(row_count)
        .aria_column_count(column_count)
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
        let row_count = self.table.aria_row_count();
        let column_count = self.table.aria_column_count();

        data_grid_element(
            format!(
                "rustui-data-grid:{}",
                self.id.as_ref().unwrap_or(&self.label)
            ),
            self.label.clone(),
            row_count,
            column_count,
        )
        .w_full()
        .child(self.table.into_element(context))
        .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::{data_grid_element, table_identity};
    use gpui::{Element, Role, SharedString, accesskit::Node};

    #[test]
    fn grid_wrapper_emits_accesskit_table_metadata() {
        let label = SharedString::from("Quarterly revenue");
        let element = data_grid_element("quarterly-grid".to_owned(), label, 3, 2);

        assert_eq!(element.a11y_role(), Some(Role::Table));
        let mut node = Node::new(Role::Table);
        element.write_a11y_info(&mut node);
        assert_eq!(node.role(), Role::Table);
        assert_eq!(node.label(), Some("Quarterly revenue"));
        assert_eq!(node.row_count(), Some(3));
        assert_eq!(node.column_count(), Some(2));
    }

    #[test]
    fn explicit_grid_ids_separate_the_retained_table_entities() {
        let label = SharedString::from("Quarterly revenue");
        let first = SharedString::from("quarterly-a");
        let second = SharedString::from("quarterly-b");

        assert_ne!(
            table_identity(&label, Some(&first)),
            table_identity(&label, Some(&second))
        );
        assert_ne!(
            table_identity(&label, Some(&first)),
            table_identity(&label, None)
        );
    }

    #[test]
    #[should_panic(expected = "DataGrid IDs must be nonempty")]
    fn whitespace_only_grid_ids_are_rejected() {
        let label = SharedString::from("Quarterly revenue");
        let id = SharedString::from("  \t");

        table_identity(&label, Some(&id));
    }
}
