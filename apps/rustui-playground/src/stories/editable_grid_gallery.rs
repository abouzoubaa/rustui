//! Editable grid examples using public RustUI APIs.
use rustui::prelude::*;

pub fn demo() -> impl View {
    Component("editable-grid-gallery", EditableGridGallery::default)
}

#[derive(Clone, Debug, PartialEq)]
struct Product {
    id: String,
    name: String,
    quantity: i32,
    category: String,
}

fn initial_rows() -> Vec<Product> {
    (0..1_000)
        .map(|index| Product {
            id: format!("product-{index}"),
            name: format!("Product {}", index + 1),
            quantity: (index % 100) as i32,
            category: if index % 2 == 0 { "Food" } else { "Other" }.into(),
        })
        .collect()
}

#[component]
struct EditableGridGallery {
    #[state(initial_rows())]
    rows: Vec<Product>,
}

impl View for EditableGridGallery {
    fn body(&self) -> impl View {
        VStack::new()
            .spacing(10.0)
            .child(Text("Editable grid"))
            .child(Text(
                "Use arrows or Tab to select a cell; Enter or F2 edits, Escape cancels.",
            ))
            .child(
                EditableDataGrid(
                    "Products",
                    self.rows_binding(),
                    |row: &Product| row.id.clone().into(),
                    vec![
                        DataGridColumn::text(
                            "name",
                            "Name",
                            |row: &Product| row.name.clone(),
                            |value| Ok(value.to_owned()),
                            |row, value| row.name = value,
                        )
                        .validate(|value| {
                            if value.trim().is_empty() {
                                Err("Enter a name".into())
                            } else {
                                Ok(())
                            }
                        })
                        .sortable_by(|a: &Product, b: &Product| a.name.cmp(&b.name)),
                        DataGridColumn::text(
                            "quantity",
                            "Quantity",
                            |row: &Product| row.quantity,
                            |value| {
                                value
                                    .parse::<i32>()
                                    .map_err(|_| "Enter an integer".to_owned())
                            },
                            |row, value| row.quantity = value,
                        )
                        .validate(|value| {
                            if value.parse::<i32>().is_ok_and(|number| number >= 0) {
                                Ok(())
                            } else {
                                Err("Enter a nonnegative integer".into())
                            }
                        })
                        .sortable_by(|a: &Product, b: &Product| a.quantity.cmp(&b.quantity)),
                        DataGridColumn::text(
                            "category",
                            "Category (read-only)",
                            |row: &Product| row.category.clone(),
                            |value| Ok(value.to_owned()),
                            |row, value| row.category = value,
                        )
                        .read_only(),
                    ],
                )
                .id("playground-editable-grid"),
            )
    }
}
