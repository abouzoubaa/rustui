//! Ordered typed property rows composed from RustUI's controlled inputs.

use std::{cell::RefCell, collections::HashSet, rc::Rc};

use gpui::{
    AnyElement, App, AppContext, Entity, IntoElement, ParentElement, SharedString, Styled, div, px,
};
use gpui_component::{
    Sizable as _,
    input::{Input, InputEvent, InputState},
};
use rustui_core::{Binding, Identity};

use crate::{
    components::input::{picker::Picker, stepper::Stepper, text_field::TextField, toggle::Toggle},
    view::{EmptyView, RenderContext, RetentionScope, View},
};

/// A controlled editor whose value stays in its enclosing component owner.
pub enum PropertyEditor {
    Text(Binding<String>),
    Choice {
        value: Binding<String>,
        options: Vec<String>,
    },
    Toggle(Binding<bool>),
    Integer(Binding<i32>),
    IntegerValue(Binding<i64>),
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

    /// Creates a string property row whose value is selected from `options`.
    ///
    /// The caller is responsible for supplying unique options and ensuring the
    /// current binding value is one of them. Schema-backed inspectors meet both
    /// conditions through catalog validation.
    pub fn choice<I, S>(
        id: impl Into<String>,
        label: impl Into<SharedString>,
        value: Binding<String>,
        options: I,
    ) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self::new(
            id,
            label,
            PropertyEditor::Choice {
                value,
                options: options.into_iter().map(Into::into).collect(),
            },
        )
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

    /// Creates an `i64` integer property row. Invalid or out-of-range drafts
    /// remain in the field and are not written to the component owner.
    pub fn integer_value(
        id: impl Into<String>,
        label: impl Into<SharedString>,
        value: Binding<i64>,
    ) -> Self {
        Self::new(id, label, PropertyEditor::IntegerValue(value))
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
    EmptyRowLabel { index: usize },
    DuplicateRowId(String),
}

/// An ordered, explicitly identified collection of typed property editors.
pub struct PropertyInspectorView {
    id: Identity,
    rows: Vec<PropertyInspectorRow>,
    disabled: bool,
}

struct RetainedIntegerValueField {
    state: Entity<InputState>,
    owner_value: i64,
    writer: Rc<RefCell<Rc<dyn Fn(i64, &mut App)>>>,
}

fn parse_integer_value_draft(draft: &str) -> Option<i64> {
    draft.parse().ok()
}

fn with_current_writer<T: Clone, Writer: Clone, Result>(
    writer: &RefCell<Writer>,
    value: T,
    apply: impl FnOnce(Writer, T) -> Result,
) -> Result {
    apply(writer.borrow().clone(), value)
}

fn editor_has_visible_label(editor: &PropertyEditor) -> bool {
    matches!(
        editor,
        PropertyEditor::Choice { .. } | PropertyEditor::Toggle(_) | PropertyEditor::Integer(_)
    )
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
        if row.label.trim().is_empty() {
            return Err(PropertyInspectorError::EmptyRowLabel { index });
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
                // A wide preview sibling must not push the row editors beyond
                // the visible inspector pane through `justify_between`.
                let mut inspector = div().flex().flex_col().gap_2().w_full().max_w(px(600.0));
                for row in &self.rows {
                    let rendered_row = context.with_element_scope(
                        RetentionScope::Key(Identity::from(row.id.clone())),
                        |context| {
                            let control_id = format!("property-inspector:{}", row.id);
                            let disabled = self.disabled || row.disabled;
                            let mut rendered_row =
                                div().flex().items_center().justify_between().gap_4();
                            if !editor_has_visible_label(&row.editor) {
                                rendered_row = rendered_row.child(row.label.clone());
                            }
                            let control = match &row.editor {
                                PropertyEditor::Text(value) => TextField("", value.clone())
                                    .id(control_id)
                                    .disabled(disabled)
                                    .accessibility_label(row.label.clone())
                                    .into_element(context),
                                PropertyEditor::Choice { value, options } => Picker(
                                    row.label.clone(),
                                    value.clone(),
                                    options.iter().cloned(),
                                )
                                .id(control_id)
                                .disabled(disabled)
                                .into_element(context),
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
                                PropertyEditor::IntegerValue(value) => {
                                    let current_value = value.value();
                                    let field_id = Identity::from(format!(
                                        "property-inspector-integer:{control_id}"
                                    ));
                                    let writer = context.binding_writer(value.clone());
                                    let initial_writer = writer.clone();
                                    let (retained, created) = context.retained_entity(
                                        field_id.clone(),
                                        move |window, cx| RetainedIntegerValueField {
                                            state: cx.new(|cx| {
                                                InputState::new(window, cx)
                                                    .default_value(current_value.to_string())
                                            }),
                                            owner_value: current_value,
                                            writer: Rc::new(RefCell::new(initial_writer)),
                                        },
                                    );
                                    let (state, previous_owner_value, current_writer) = {
                                        let retained = retained.read(context.app());
                                        (
                                            retained.state.clone(),
                                            retained.owner_value,
                                            retained.writer.clone(),
                                        )
                                    };
                                    *current_writer.borrow_mut() = writer;
                                    if current_value != previous_owner_value {
                                        let draft = current_value.to_string();
                                        context.update_entity(&state, move |state, window, cx| {
                                            state.set_value(draft, window, cx);
                                        });
                                        context.update_entity(&retained, move |retained, _, _| {
                                            retained.owner_value = current_value;
                                        });
                                    }
                                    if created {
                                        let current_writer = current_writer.clone();
                                        context.subscribe_retained(
                                            &field_id,
                                            &state,
                                            move |state, event: &InputEvent, cx| {
                                                if matches!(event, InputEvent::Change) {
                                                    let draft = state.read(cx).value().to_string();
                                                    if let Some(value) =
                                                        parse_integer_value_draft(&draft)
                                                    {
                                                        with_current_writer(
                                                            &current_writer,
                                                            value,
                                                            |write, value| write(value, cx),
                                                        );
                                                    }
                                                }
                                            },
                                        );
                                    }
                                    Input::new(&state)
                                        .id(field_id.to_string())
                                        .small()
                                        .disabled(disabled)
                                        .aria_label(row.label.clone())
                                        .into_any_element()
                                }
                            };
                            rendered_row.child(control).into_any_element()
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
        integer_value: i64,
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

    fn integer_value_binding() -> Binding<i64> {
        Binding::new::<Owner>(
            0,
            |owner| owner.integer_value,
            |owner, value| {
                owner.integer_value = value;
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
    fn rejects_blank_accessible_row_labels() {
        assert_eq!(
            PropertyInspector(
                "inspector",
                [PropertyInspectorRow::text("name", " \t", text_binding())]
            )
            .err(),
            Some(PropertyInspectorError::EmptyRowLabel { index: 0 })
        );
    }

    #[test]
    fn avoids_repeating_labels_owned_by_choice_toggle_and_stepper_editors() {
        assert!(editor_has_visible_label(&PropertyEditor::Toggle(
            toggle_binding()
        )));
        assert!(editor_has_visible_label(&PropertyEditor::Integer(
            integer_binding()
        )));
        assert!(!editor_has_visible_label(&PropertyEditor::Text(
            text_binding()
        )));
        assert!(editor_has_visible_label(&PropertyEditor::Choice {
            value: text_binding(),
            options: vec!["one".into()],
        }));
        assert!(!editor_has_visible_label(&PropertyEditor::IntegerValue(
            integer_value_binding()
        )));
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

    #[test]
    fn choice_rows_keep_options_and_forward_selection_to_the_owner() {
        let row = PropertyInspectorRow::choice(
            "layout",
            "Layout",
            text_binding(),
            ["compact", "comfortable", "spacious"],
        );
        let mut owner = Owner::default();
        let owner_any: &mut dyn Any = &mut owner;
        match &row.editor {
            PropertyEditor::Choice { value, options } => {
                assert_eq!(
                    options,
                    &vec![
                        "compact".to_owned(),
                        "comfortable".to_owned(),
                        "spacious".to_owned()
                    ]
                );
                assert!(value.write_to(owner_any, "spacious".into()));
            }
            _ => unreachable!(),
        }
        assert_eq!(owner.text, "spacious");
    }

    #[test]
    fn i64_integer_rows_accept_in_range_values_and_reject_invalid_drafts() {
        let binding = integer_value_binding();
        let row = PropertyInspectorRow::integer_value("limit", "Limit", binding.clone());
        assert!(matches!(row.editor, PropertyEditor::IntegerValue(_)));

        let mut owner = Owner::default();
        for invalid in ["1.5", "9223372036854775808", "-9223372036854775809"] {
            if let Some(parsed) = parse_integer_value_draft(invalid) {
                binding.write_to(&mut owner, parsed);
            }
        }
        assert_eq!(owner.integer_value, 0);

        let parsed = parse_integer_value_draft("-9223372036854775808").unwrap();
        assert!(binding.write_to(&mut owner, parsed));
        assert_eq!(owner.integer_value, i64::MIN);
    }

    #[test]
    fn retained_integer_editor_uses_the_latest_binding_writer() {
        let writes = Rc::new(RefCell::new(Vec::new()));
        let first_writes = writes.clone();
        let current = RefCell::new(
            Rc::new(move |value| first_writes.borrow_mut().push(value)) as Rc<dyn Fn(i64)>
        );

        let latest_writes = writes.clone();
        *current.borrow_mut() = Rc::new(move |value| latest_writes.borrow_mut().push(value));

        with_current_writer(&current, 42, |write, value| write(value));
        assert_eq!(*writes.borrow(), [42]);
    }
}
