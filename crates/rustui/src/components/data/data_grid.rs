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
        Self {
            table,
            label,
            id: None,
        }
    }

    /// Sets a stable identity for a grid whose position may change.
    pub fn id(mut self, id: impl Into<SharedString>) -> Self {
        let id = id.into();
        self.table = self.table.id(table_identity(&id));
        self.id = Some(id);
        self
    }
}

fn table_identity(id: &SharedString) -> Identity {
    assert!(!id.trim().is_empty(), "DataGrid IDs must be nonempty");
    Identity::from(format!("rustui-data-grid-table:{id}"))
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
        let identity = match &self.id {
            Some(id) => Identity::from(format!("rustui-data-grid:{id}")),
            None => context.next_implicit_retained_identity(Identity::from("data-grid")),
        };

        data_grid_element(
            identity.to_string(),
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

    #[gpui::test]
    fn same_label_implicit_grids_keep_selections_independent(cx: &mut gpui::TestAppContext) {
        use super::{DataGrid, TableView};
        use crate::runtime::{Binding, ComponentHost};
        use crate::view::{EmptyView, RenderContext, View};
        use gpui::{IntoElement, ParentElement, div};

        struct Owner {
            first: Option<usize>,
            second: Option<usize>,
            first_present: bool,
        }
        impl View for Owner {
            fn body(&self) -> impl View {
                EmptyView
            }

            fn into_element(&self, context: &mut RenderContext<'_>) -> gpui::AnyElement {
                let first = Binding::new(
                    self.first,
                    |owner: &Owner| owner.first,
                    |owner, value| owner.first = value,
                );
                let second = Binding::new(
                    self.second,
                    |owner: &Owner| owner.second,
                    |owner, value| owner.second = value,
                );
                let first_rows = self.first_present.then(|| vec!["first"]);
                let first = DataGrid(TableView::new(["Value"], first_rows, first), "Same label")
                    .into_element(context);
                let second = DataGrid(
                    TableView::new(["Value"], [vec!["second"]], second),
                    "Same label",
                )
                .into_element(context);
                div().child(first).child(second).into_any_element()
            }
        }
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
            ComponentHost::new(
                Owner {
                    first: Some(0),
                    second: Some(0),
                    first_present: true,
                },
                app,
            )
        });
        window.update(|window, app| {
            let _ = window.draw(app);
            owner.update(app, |host, cx| {
                host.component_mut().first_present = false;
                cx.notify();
            });
            let _ = window.draw(app);
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(owner.read(app).component().first, None);
            assert_eq!(owner.read(app).component().second, Some(0));
        });
    }

    #[gpui::test]
    fn label_change_preserves_restored_grid_selection(cx: &mut gpui::TestAppContext) {
        use super::{DataGrid, TableView};
        use crate::runtime::{Binding, ComponentHost};
        use crate::view::View;

        struct Owner {
            selection: Option<usize>,
            rows: usize,
            changed_label: bool,
            writes: usize,
        }
        impl View for Owner {
            fn body(&self) -> impl View {
                let selection = Binding::new(
                    self.selection,
                    |owner: &Owner| owner.selection,
                    |owner, value| {
                        owner.selection = value;
                        owner.writes += 1;
                    },
                );
                let rows = (0..self.rows).map(|index| vec![index.to_string()]);
                DataGrid(
                    TableView::new(["Value"], rows, selection),
                    if self.changed_label {
                        "Updated"
                    } else {
                        "Initial"
                    },
                )
            }
        }
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
            ComponentHost::new(
                Owner {
                    selection: Some(1),
                    rows: 2,
                    changed_label: false,
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
                host.component_mut().changed_label = true;
                cx.notify();
            });
            let _ = window.draw(app);
        });
        window.cx.run_until_parked();
        window.update(|_, app| {
            assert_eq!(owner.read(app).component().selection, Some(1));
            assert_eq!(owner.read(app).component().writes, 0);
        });
    }

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
        let first = SharedString::from("quarterly-a");
        let second = SharedString::from("quarterly-b");

        assert_ne!(table_identity(&first), table_identity(&second));
    }

    #[test]
    #[should_panic(expected = "DataGrid IDs must be nonempty")]
    fn whitespace_only_grid_ids_are_rejected() {
        let id = SharedString::from("  \t");

        table_identity(&id);
    }
}
