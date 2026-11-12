use gpui::{
    App, Context, Div, Entity, IntoElement, Render, Role, Stateful, TestAppContext, Window, div,
    prelude::*, px,
};
use gpui_component::table::{Column, DataTable, TableDelegate, TableState};

struct Delegate;

impl TableDelegate for Delegate {
    fn columns_count(&self, _: &App) -> usize {
        1
    }
    fn rows_count(&self, _: &App) -> usize {
        1
    }
    fn column(&self, _: usize, _: &App) -> Column {
        Column::new("name", "Name")
    }

    fn render_header(
        &mut self,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        div()
            .id("delegate-header-row")
            .role(Role::Row)
            .aria_row_index(1)
    }

    fn render_th(
        &mut self,
        _: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
            .id("delegate-header")
            .role(Role::ColumnHeader)
            .aria_label("Name")
            .aria_column_index(1)
            .aria_sort(gpui::accesskit::SortDirection::Ascending)
            .child("Name")
    }

    fn render_tr(
        &mut self,
        row: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> Stateful<Div> {
        div()
            .id(("delegate-row", row))
            .role(Role::Row)
            .aria_row_index(2)
    }

    fn render_td(
        &mut self,
        _: usize,
        _: usize,
        _: &mut Window,
        _: &mut Context<TableState<Self>>,
    ) -> impl IntoElement {
        div()
            .id("delegate-cell")
            .role(Role::Cell)
            .aria_label("Ada")
            .aria_column_index(1)
            .aria_readonly(true)
            .child("Ada")
    }
}

struct TableView(Entity<TableState<Delegate>>);

impl Render for TableView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(360.)).h(px(240.)).child(DataTable::new(&self.0))
    }
}

#[gpui::test]
fn table_delegate_accessibility_nodes_keep_roles_indices_and_readonly() {
    let mut cx = TestAppContext::single();
    cx.update(gpui_component::init);
    let (_, window) = cx.add_window_view(|window, cx| {
        TableView(cx.new(|cx| TableState::new(Delegate, window, cx).delegate_accessibility(true)))
    });
    window.update(|window, cx| window.draw(cx).clear(cx));
    let tree: serde_json::Value = window
        .update(|window, _| serde_json::from_str(&window.debug_a11y_tree_json().unwrap()).unwrap());
    let nodes = tree["nodes"].as_object().unwrap();
    let role_count = |role: &str| {
        nodes
            .values()
            .filter(|node| node["aria"]["role"] == role)
            .count()
    };
    assert_eq!(
        role_count("ColumnHeader"),
        1,
        "delegate owns header role: {tree}"
    );
    assert_eq!(role_count("Cell"), 1, "delegate owns cell role: {tree}");
    assert_eq!(role_count("Row"), 2, "delegate owns row roles: {tree}");
    let header = nodes
        .values()
        .find(|node| node["aria"]["role"] == "ColumnHeader")
        .unwrap();
    let cell = nodes
        .values()
        .find(|node| node["aria"]["role"] == "Cell")
        .unwrap();
    assert_eq!(header["aria"]["column_index"], 1);
    assert_eq!(header["aria"]["sort_direction"], "Ascending");
    assert_eq!(cell["aria"]["column_index"], 1);
    assert_eq!(cell["aria"]["read_only"], true);
}
