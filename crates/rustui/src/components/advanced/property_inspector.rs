//! Ordered typed property rows composed from RustUI's controlled inputs.

use std::collections::HashSet;

use gpui::{AnyElement, IntoElement, ParentElement, SharedString, Styled, div};
use rustui_core::{Binding, Identity};

use crate::{
    components::input::{stepper::Stepper, text_field::TextField, toggle::Toggle},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

/// A controlled editor whose value stays in its enclosing component owner.
pub enum PropertyEditor {
    Text(Binding<String>),
    Toggle(Binding<bool>),
    Integer(Binding<i32>),
}

/// One labeled property with a stable identity independent of its position.
pub struct PropertyInspectorRow {
    id: String,
    label: SharedString,
    editor: PropertyEditor,
    disabled: bool,
}

impl PropertyInspectorRow {
    pub fn text(
        id: impl Into<String>,
        label: impl Into<SharedString>,
        value: Binding<String>,
    ) -> Self {
        Self::new(id, label, PropertyEditor::Text(value))
    }

    pub fn toggle(
        id: impl Into<String>,
        label: impl Into<SharedString>,
        value: Binding<bool>,
    ) -> Self {
        Self::new(id, label, PropertyEditor::Toggle(value))
    }

    pub fn integer(
        id: impl Into<String>,
        label: impl Into<SharedString>,
        value: Binding<i32>,
    ) -> Self {
        Self::new(id, label, PropertyEditor::Integer(value))
    }

    pub fn new(
        id: impl Into<String>,
        label: impl Into<SharedString>,
        editor: PropertyEditor,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            editor,
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Errors found while constructing a property inspector.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PropertyInspectorError {
    EmptyInspectorId,
    EmptyRowId { index: usize },
    DuplicateRowId(String),
}

/// An ordered, explicitly identified collection of typed property editors.
pub struct PropertyInspectorView {
    id: Identity,
    rows: Vec<PropertyInspectorRow>,
    disabled: bool,
}

/// Creates a property inspector. Row IDs must be unique and nonblank.
#[allow(non_snake_case)]
pub fn PropertyInspector(
    id: impl Into<Identity>,
    rows: impl IntoIterator<Item = PropertyInspectorRow>,
) -> Result<PropertyInspectorView, PropertyInspectorError> {
    let id = id.into();
    if id.as_str().trim().is_empty() {
        return Err(PropertyInspectorError::EmptyInspectorId);
    }
    let rows: Vec<_> = rows.into_iter().collect();
    let mut ids = HashSet::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        if row.id.trim().is_empty() {
            return Err(PropertyInspectorError::EmptyRowId { index });
        }
        if !ids.insert(row.id.as_str()) {
            return Err(PropertyInspectorError::DuplicateRowId(row.id.clone()));
        }
    }
    Ok(PropertyInspectorView {
        id,
        rows,
        disabled: false,
    })
}

impl PropertyInspectorView {
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn rows(&self) -> &[PropertyInspectorRow] {
        &self.rows
    }
}

impl View for PropertyInspectorView {
    fn body(&self) -> impl View {
        EmptyView
    }

    fn into_element(&self, context: &mut RenderContext<'_>) -> AnyElement {
        context.with_element_scope(
            RetentionScope::PropertyInspector(self.id.clone()),
            |context| {
                let mut inspector = div().flex().flex_col().gap_2();
                for row in &self.rows {
                    let rendered_row = context.with_element_scope(
                        RetentionScope::Key(Identity::from(row.id.clone())),
                        |context| {
                            let control_id = format!("property-inspector:{}", row.id);
                            let disabled = self.disabled || row.disabled;
                            match &row.editor {
                                PropertyEditor::Text(value) => div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_4()
                                    .child(row.label.clone())
                                    .child(
                                        TextField("", value.clone())
                                            .id(control_id)
                                            .disabled(disabled)
                                            .accessibility_label(row.label.clone())
                                            .into_element(context),
                                    )
                                    .into_any_element(),
                                PropertyEditor::Toggle(value) => {
                                    Toggle(row.label.clone(), value.clone())
                                        .id(control_id)
                                        .disabled(disabled)
                                        .into_element(context)
                                }
                                PropertyEditor::Integer(value) => {
                                    Stepper(row.label.clone(), value.clone())
                                        .id(control_id)
                                        .disabled(disabled)
                                        .into_element(context)
                                }
                            }
                        },
                    );
                    inspector = inspector.child(rendered_row);
                }
                inspector.into_any_element()
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[derive(Default)]
    struct Owner {
        text: String,
        enabled: bool,
        count: i32,
    }

    fn text_binding() -> Binding<String> {
        Binding::new::<Owner>(
            String::new(),
            |owner| owner.text.clone(),
            |owner, value| {
                owner.text = value;
            },
        )
    }

    fn toggle_binding() -> Binding<bool> {
        Binding::new::<Owner>(
            false,
            |owner| owner.enabled,
            |owner, value| {
                owner.enabled = value;
            },
        )
    }

    fn integer_binding() -> Binding<i32> {
        Binding::new::<Owner>(
            0,
            |owner| owner.count,
            |owner, value| {
                owner.count = value;
            },
        )
    }

    fn rows() -> Vec<PropertyInspectorRow> {
        vec![
            PropertyInspectorRow::text("name", "Name", text_binding()),
            PropertyInspectorRow::toggle("enabled", "Enabled", toggle_binding()),
            PropertyInspectorRow::integer("count", "Count", integer_binding()),
        ]
    }

    #[test]
    fn rejects_blank_inspector_and_row_ids_and_duplicate_row_ids() {
        assert_eq!(
            PropertyInspector(" ", Vec::new()).err(),
            Some(PropertyInspectorError::EmptyInspectorId)
        );
        assert_eq!(
            PropertyInspector(
                "inspector",
                [PropertyInspectorRow::text("\t", "Name", text_binding())]
            )
            .err(),
            Some(PropertyInspectorError::EmptyRowId { index: 0 })
        );
        assert_eq!(
            PropertyInspector(
                "inspector",
                [
                    PropertyInspectorRow::text("same", "First", text_binding()),
                    PropertyInspectorRow::text("same", "Second", text_binding()),
                ]
            )
            .err(),
            Some(PropertyInspectorError::DuplicateRowId("same".into()))
        );
    }

    #[test]
    fn preserves_input_order_and_stable_row_ids() {
        let inspector = PropertyInspector("inspector", rows()).unwrap();
        assert_eq!(
            inspector
                .rows()
                .iter()
                .map(PropertyInspectorRow::id)
                .collect::<Vec<_>>(),
            ["name", "enabled", "count"]
        );
        let reordered = PropertyInspector(
            "inspector",
            [
                PropertyInspectorRow::integer("count", "Count", integer_binding()),
                PropertyInspectorRow::text("name", "Name", text_binding()),
            ],
        )
        .unwrap();
        assert_eq!(
            reordered
                .rows()
                .iter()
                .map(PropertyInspectorRow::id)
                .collect::<Vec<_>>(),
            ["count", "name"]
        );
    }

    #[test]
    fn typed_bindings_forward_writes_to_the_same_owner() {
        let rows = rows();
        let mut owner = Owner::default();
        let owner_any: &mut dyn Any = &mut owner;
        match &rows[0].editor {
            PropertyEditor::Text(binding) => assert!(binding.write_to(owner_any, "Ada".into())),
            _ => unreachable!(),
        }
        match &rows[1].editor {
            PropertyEditor::Toggle(binding) => assert!(binding.write_to(owner_any, true)),
            _ => unreachable!(),
        }
        match &rows[2].editor {
            PropertyEditor::Integer(binding) => assert!(binding.write_to(owner_any, 7)),
            _ => unreachable!(),
        }
        assert_eq!(owner.text, "Ada");
        assert!(owner.enabled);
        assert_eq!(owner.count, 7);
    }
}
